//! SQL fragments shared by the report and the per-client view.

/// Plausible sensor range. A dead EC register reads back 0xFF and the RPM
/// counter overflows to five digits.
pub const SANE_TEMP: &str = "BETWEEN 1 AND 110";
pub const SANE_FAN: &str = "BETWEEN 0 AND 12000";

/// Event kinds that identify the machine rather than sample it.
pub const IDENTITY_KINDS: &[&str] = &["Startup", "Unsupported"];
pub const FAILURE_KINDS: &[&str] = &["Panic", "ErrorLog"];

/// Builds an expression rendering the age of `column` as "N min/h/d ago".
pub fn ago(column: &str) -> String {
    let delta = format!("(julianday('now') - julianday({column}))");
    format!(
        "CASE
             WHEN {column} IS NULL THEN '-'
             WHEN {delta} < 0.0007 THEN 'just now'
             WHEN {delta} < 0.04 THEN CAST(ROUND({delta} * 1440) AS INT) || ' min ago'
             WHEN {delta} < 1.0  THEN CAST(ROUND({delta} * 24) AS INT) || ' h ago'
             ELSE CAST(ROUND({delta}) AS INT) || ' d ago'
         END"
    )
}

/// Quotes a kind list for `IN (...)`. Callers pass crate constants; this does
/// not escape user input.
pub fn kind_list(kinds: &[&str]) -> String {
    kinds.iter().map(|k| format!("'{k}'")).collect::<Vec<_>>().join(", ")
}
