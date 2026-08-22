use log::{error, warn};
use rusqlite::{Connection, params};

/// One row per event, holding the payload verbatim as JSON.
///
/// Everything worth querying is a generated column over that JSON, so a new
/// event kind or a new field in an existing one costs zero schema changes and
/// old rows simply return NULL for fields they predate. Generated columns are
/// VIRTUAL: they take no space and `->>` is cheap enough to evaluate on read.
///
/// Speed can be bought later without touching this, e.g.
/// ```sql
/// ALTER TABLE events ADD COLUMN board TEXT
///   GENERATED ALWAYS AS (body ->> '$.data.c.host.motherboard') VIRTUAL;
/// CREATE INDEX idx_events_board ON events(board);
/// ```
const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS events (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    ts             DATETIME DEFAULT CURRENT_TIMESTAMP,
    daemon_version TEXT NOT NULL,
    src            TEXT NOT NULL,
    body           TEXT NOT NULL CHECK (json_valid(body)),

    client_id  TEXT GENERATED ALWAYS AS (body ->> '$.id')      VIRTUAL,
    session_id TEXT GENERATED ALWAYS AS (body ->> '$.session') VIRTUAL,
    kind       TEXT GENERATED ALWAYS AS (body ->> '$.data.t')  VIRTUAL
);

CREATE INDEX IF NOT EXISTS idx_events_kind_ts ON events(kind, ts);
CREATE INDEX IF NOT EXISTS idx_events_client ON events(client_id);

CREATE TABLE IF NOT EXISTS raw_telemetry (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp      DATETIME DEFAULT CURRENT_TIMESTAMP,
    daemon_version TEXT NOT NULL,
    raw_data       BLOB NOT NULL
);
";

/// Pre-0.6 tables (`startup_events`, `status_events`, ...) are intentionally not
/// created here. Where they exist they keep their history; nothing new is
/// written to them.
pub fn open(path: &str) -> Connection {
    let conn = Connection::open(path).expect("Failed to open DB");
    conn.execute_batch("PRAGMA journal_mode = WAL;").expect("Failed to enable WAL");
    conn.execute_batch(SCHEMA).expect("Failed to apply schema");
    conn
}

/// `src` separates wire formats. Legacy payloads carry a different set of
/// fields under the same `kind`, so aggregates over the current schema must
/// filter on it rather than assume every row looks alike.
pub fn insert_event(conn: &Connection, version: &str, src: &str, body: &str) -> bool {
    match conn.execute(
        "INSERT INTO events (daemon_version, src, body) VALUES (?1, ?2, ?3)",
        params![version, src, body],
    ) {
        Ok(_) => true,
        Err(e) => {
            error!("Failed to insert event: {e}");
            false
        }
    }
}

/// Dead letter box for anything that did not decode. Keeping the bytes means a
/// decoder bug can be fixed after the fact instead of costing the data.
/// Returns the HTTP status: 202 (kept, not understood) or 500.
pub fn dead_letter(conn: &Connection, version: &str, raw: &[u8]) -> u16 {
    match conn.execute(
        "INSERT INTO raw_telemetry (daemon_version, raw_data) VALUES (?1, ?2)",
        params![version, raw],
    ) {
        Ok(_) => {
            warn!("Undecodable payload stored raw ({} bytes)", raw.len());
            202
        }
        Err(e) => {
            error!("Failed to store raw payload: {e}");
            500
        }
    }
}
