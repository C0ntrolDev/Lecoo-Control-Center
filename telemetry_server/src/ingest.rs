//! `/telemetry/v2`: JSON payloads from 0.6+ daemons.

use log::warn;
use rusqlite::Connection;
use serde_json::Value;

use crate::db;

/// Comfortably above the daemon's own pending-event cap.
const MAX_BATCH: usize = 64;

/// Parsed as an untyped `Value` on purpose: an event kind this build has never
/// heard of must land in the table verbatim, which a typed decoder would
/// quietly fold into its catch-all variant instead.
///
/// A body may be a single payload or an array of them; the daemon batches only
/// after a delivery failed, so arrays are the retry path.
pub fn store(conn: &Connection, version: &str, body: &[u8]) -> u16 {
    let Ok(value) = serde_json::from_slice::<Value>(body) else {
        return db::dead_letter(conn, version, body);
    };

    let events: Vec<&Value> = match &value {
        Value::Array(items) => items.iter().collect(),
        single => vec![single],
    };

    if events.is_empty() || !events.iter().all(|e| is_payload(e)) {
        warn!("Body is valid JSON but not a telemetry payload");
        return db::dead_letter(conn, version, body);
    }

    // The daemon never queues more than its own retry buffer holds. Without a
    // cap one maximum-size body of minimal events turns into ~15k inserts
    // holding the write lock.
    if events.len() > MAX_BATCH {
        warn!("Rejected a batch of {} events", events.len());
        return 413;
    }

    // All or nothing: a batch that fails halfway would otherwise leave rows
    // behind that the daemon sends again on its next retry.
    let Ok(tx) = conn.unchecked_transaction() else {
        return 500;
    };

    for event in events {
        if !db::insert_event(&tx, version, "v2", &event.to_string()) {
            return 500;
        }
    }

    if tx.commit().is_err() { 500 } else { 201 }
}

/// Shape check, not schema validation: only the parts the generated columns
/// read have to be there. Everything below `data.c` is the daemon's business
/// and may change freely.
fn is_payload(value: &Value) -> bool {
    value.get("id").and_then(Value::as_str).is_some()
        && value.get("data").and_then(|data| data.get("t")).and_then(Value::as_str).is_some()
}
