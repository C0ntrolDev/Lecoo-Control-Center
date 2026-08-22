//! Client lookup, plus the cross-fleet views that are not part of the report.

use rusqlite::Connection;

use super::render::{self, Table};
use super::sql;

type Result = rusqlite::Result<()>;

/// Lists machines whose id, version, board, OS, CPU or chip matches `pattern`.
///
/// Only identity events are in scope, so `last boot` is the last Startup and
/// not the last event.
pub fn find(conn: &Connection, pattern: &str, limit: i64) -> Result {
    render::title(&format!("Machines matching '{pattern}'"));

    let table = Table::query(
        conn,
        &format!(
            "SELECT client_id,
                    COALESCE(version, '?') AS version,
                    COALESCE(board, motherboard, '?') AS board,
                    COALESCE(os, '?') AS os,
                    COALESCE(cpu, '?') AS cpu,
                    MAX(ts) AS last_boot,
                    {age}
             FROM ev
             WHERE client_id LIKE ?1 OR version LIKE ?1 OR board LIKE ?1
                OR motherboard LIKE ?1 OR os LIKE ?1 OR cpu LIKE ?1 OR chip LIKE ?1
             GROUP BY client_id
             ORDER BY last_boot DESC LIMIT ?2",
            age = sql::ago("MAX(ts)")
        ),
        rusqlite::params![format!("%{pattern}%"), limit],
        &["client", "version", "board", "os", "cpu", "last boot", "age"],
    )?;

    if table.is_empty() {
        render::note("  nothing matched");
    } else {
        table.print();
        render::note(&format!("\n  {} shown, inspect one with: client <id>", table.len()));
    }

    Ok(())
}

/// Panics and error logs grouped by message, ranked by affected machines.
pub fn errors(conn: &Connection, days: Option<i64>, limit: i64) -> Result {
    let window = match days {
        Some(d) => format!("AND ts >= datetime('now', '-{d} days')"),
        None => String::new(),
    };

    render::title(match days {
        Some(_) => "Panics and errors in window",
        None => "Panics and errors, all time",
    });

    let table = Table::query(
        conn,
        &format!(
            // `hits` is cumulative within a session, so occurrences are the max
            // per session summed across sessions. Summing the reports instead
            // would add 1+2+4 and claim seven where there were four.
            "WITH per_session AS (
                 SELECT kind, level, message, client_id, session_id,
                        COUNT(*) AS reports,
                        MAX(COALESCE(hits, 1)) AS occurrences,
                        MAX(version) AS version,
                        MIN(ts) AS first, MAX(ts) AS last
                 FROM ev
                 WHERE kind IN ({kinds}) AND message IS NOT NULL {window}
                 GROUP BY kind, level, message, client_id, session_id
             )
             SELECT COALESCE(level, kind) AS level,
                    COUNT(DISTINCT client_id) AS clients,
                    SUM(reports) AS reports,
                    SUM(occurrences) AS seen,
                    MIN(first) AS first,
                    MAX(last) AS last,
                    {age},
                    GROUP_CONCAT(DISTINCT version) AS versions,
                    message
             FROM per_session
             GROUP BY kind, level, message
             ORDER BY clients DESC, seen DESC LIMIT ?1",
            age = sql::ago("MAX(last)"),
            kinds = sql::kind_list(sql::FAILURE_KINDS)
        ),
        [limit],
        &["level", "clients", "reports", "seen", "first", "last", "age", "versions", "message"],
    )?;

    if table.is_empty() {
        println!("  {}none recorded{}", render::c(render::GREEN), render::c(render::RESET));
    } else {
        table.print();
    }

    Ok(())
}

/// Runs a caller-supplied statement against `ev`.
///
/// Refused unless SQLite reports it read-only: the connection has to be
/// read-write because a WAL database cannot be opened otherwise.
pub fn query(conn: &Connection, statement: &str) -> Result {
    let prepared = conn.prepare(statement)?;
    if !prepared.readonly() {
        drop(prepared);
        render::warn("Refused: only read-only statements are allowed here");
        return Ok(());
    }
    drop(prepared);

    let table = Table::query_free(conn, statement)?;
    println!();
    if table.is_empty() {
        render::note("  no rows");
    } else {
        table.print();
        render::note(&format!("\n  {} rows", table.len()));
    }

    Ok(())
}
