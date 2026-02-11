// Log entries repository
// Handles CRUD operations for application log storage

use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{Connection, Row, params};
use serde::{Deserialize, Serialize};

/// Persisted log entry (SQLite storage)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub source: String,
    pub message: String,
    pub metadata_json: Option<String>,
}

impl LogEntry {
    fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        let timestamp_str: String = row.get(1)?;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(Self {
            id: row.get(0)?,
            timestamp,
            level: row.get(2)?,
            source: row.get(3)?,
            message: row.get(4)?,
            metadata_json: row.get(5)?,
        })
    }
}

/// Insert a log entry
pub fn insert_log(conn: &Connection, entry: &LogEntry) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO log_entries (id, timestamp, level, source, message, metadata_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            entry.id,
            entry.timestamp.to_rfc3339(),
            entry.level,
            entry.source,
            entry.message,
            entry.metadata_json,
        ],
    )?;
    Ok(())
}

/// Insert multiple log entries in a batch
pub fn insert_logs_batch(
    conn: &Connection,
    entries: &[LogEntry],
) -> Result<usize, rusqlite::Error> {
    let mut count = 0;
    for entry in entries {
        insert_log(conn, entry)?;
        count += 1;
    }
    Ok(count)
}

fn utc_day_bounds(date: &str) -> Option<(String, String)> {
    let day = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let start = day.and_hms_opt(0, 0, 0)?;
    let end_day = day + chrono::Duration::days(1);
    let end = end_day.and_hms_opt(0, 0, 0)?;

    Some((start.and_utc().to_rfc3339(), end.and_utc().to_rfc3339()))
}

/// Get log entries by date (YYYY-MM-DD, UTC)
pub fn get_logs_by_date(
    conn: &Connection,
    date: &str,
    limit: usize,
) -> Result<Vec<LogEntry>, rusqlite::Error> {
    let (start, end) = match utc_day_bounds(date) {
        Some(bounds) => bounds,
        None => return Ok(Vec::new()),
    };

    let mut stmt = conn.prepare(
        "SELECT id, timestamp, level, source, message, metadata_json
         FROM log_entries
         WHERE timestamp >= ?1 AND timestamp < ?2
         ORDER BY timestamp DESC
         LIMIT ?3",
    )?;

    let logs = stmt
        .query_map(params![start, end, limit as i64], LogEntry::from_row)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(logs)
}

/// Get today's log entries
#[allow(dead_code)]
pub fn get_today_logs(conn: &Connection, limit: usize) -> Result<Vec<LogEntry>, rusqlite::Error> {
    let date = Utc::now().format("%Y-%m-%d").to_string();
    get_logs_by_date(conn, &date, limit)
}

/// Get recent log entries (across multiple days if needed)
pub fn get_recent_logs(conn: &Connection, count: usize) -> Result<Vec<LogEntry>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, level, source, message, metadata_json
         FROM log_entries
         ORDER BY timestamp DESC
         LIMIT ?1",
    )?;

    let logs = stmt
        .query_map(params![count as i64], LogEntry::from_row)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(logs)
}

/// Get log entries by level
#[allow(dead_code)]
pub fn get_logs_by_level(
    conn: &Connection,
    level: &str,
    limit: usize,
) -> Result<Vec<LogEntry>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, timestamp, level, source, message, metadata_json
         FROM log_entries
         WHERE level = ?1
         ORDER BY timestamp DESC
         LIMIT ?2",
    )?;

    let logs = stmt
        .query_map(params![level, limit as i64], LogEntry::from_row)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(logs)
}

/// Delete logs older than specified days
pub fn delete_old_logs(conn: &Connection, retention_days: i64) -> Result<usize, rusqlite::Error> {
    let cutoff = Utc::now() - chrono::Duration::days(retention_days);
    let cutoff_str = cutoff.to_rfc3339();

    conn.execute(
        "DELETE FROM log_entries WHERE timestamp < ?1",
        params![cutoff_str],
    )
}

/// Count log entries for a specific date
pub fn count_logs_by_date(conn: &Connection, date: &str) -> Result<i64, rusqlite::Error> {
    let (start, end) = match utc_day_bounds(date) {
        Some(bounds) => bounds,
        None => return Ok(0),
    };

    conn.query_row(
        "SELECT COUNT(*) FROM log_entries WHERE timestamp >= ?1 AND timestamp < ?2",
        params![start, end],
        |row| row.get(0),
    )
}

/// Get total log count
#[allow(dead_code)]
pub fn count_all_logs(conn: &Connection) -> Result<i64, rusqlite::Error> {
    conn.query_row("SELECT COUNT(*) FROM log_entries", [], |row| row.get(0))
}

/// Get log statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    pub total_count: i64,
    pub today_count: i64,
    pub oldest_timestamp: Option<String>,
    pub newest_timestamp: Option<String>,
}

pub fn get_log_stats(conn: &Connection) -> Result<LogStats, rusqlite::Error> {
    let total_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM log_entries", [], |row| row.get(0))?;

    let today = Utc::now().format("%Y-%m-%d").to_string();
    let today_count = count_logs_by_date(conn, &today)?;

    let oldest_timestamp: Option<String> = conn
        .query_row(
            "SELECT timestamp FROM log_entries ORDER BY timestamp ASC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    let newest_timestamp: Option<String> = conn
        .query_row(
            "SELECT timestamp FROM log_entries ORDER BY timestamp DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    Ok(LogStats {
        total_count,
        today_count,
        oldest_timestamp,
        newest_timestamp,
    })
}

/// Get list of available log dates (YYYY-MM-DD)
pub fn get_available_dates(conn: &Connection) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT substr(timestamp, 1, 10) as date
         FROM log_entries
         ORDER BY date DESC",
    )?;

    let dates = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(dates)
}

/// Delete logs by date (YYYY-MM-DD)
pub fn delete_logs_by_date(conn: &Connection, date: &str) -> Result<usize, rusqlite::Error> {
    let (start, end) = match utc_day_bounds(date) {
        Some(bounds) => bounds,
        None => return Ok(0),
    };

    conn.execute(
        "DELETE FROM log_entries WHERE timestamp >= ?1 AND timestamp < ?2",
        params![start, end],
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::database::schema::CREATE_TABLES_SQL;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(CREATE_TABLES_SQL).unwrap();
        conn
    }

    #[test]
    fn test_insert_and_get_log() {
        let conn = setup_test_db();

        let entry = LogEntry {
            id: "test-log-1".to_string(),
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            source: "System".to_string(),
            message: "Test message".to_string(),
            metadata_json: None,
        };

        insert_log(&conn, &entry).unwrap();

        let logs = get_recent_logs(&conn, 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].id, "test-log-1");
        assert_eq!(logs[0].message, "Test message");
    }

    #[test]
    fn test_delete_old_logs() {
        let conn = setup_test_db();

        // Insert an old log entry
        let old_timestamp = Utc::now() - chrono::Duration::days(10);
        let entry = LogEntry {
            id: "old-log-1".to_string(),
            timestamp: old_timestamp,
            level: "INFO".to_string(),
            source: "System".to_string(),
            message: "Old message".to_string(),
            metadata_json: None,
        };
        insert_log(&conn, &entry).unwrap();

        // Delete logs older than 7 days
        let deleted = delete_old_logs(&conn, 7).unwrap();
        assert_eq!(deleted, 1);

        // Verify it's deleted
        let logs = get_recent_logs(&conn, 10).unwrap();
        assert!(logs.is_empty());
    }

    #[test]
    fn test_get_log_stats() {
        let conn = setup_test_db();

        // Insert test logs
        for i in 0..5 {
            let entry = LogEntry {
                id: format!("log-{}", i),
                timestamp: Utc::now(),
                level: "INFO".to_string(),
                source: "System".to_string(),
                message: format!("Message {}", i),
                metadata_json: None,
            };
            insert_log(&conn, &entry).unwrap();
        }

        let stats = get_log_stats(&conn).unwrap();
        assert_eq!(stats.total_count, 5);
        assert!(stats.newest_timestamp.is_some());
    }
}
