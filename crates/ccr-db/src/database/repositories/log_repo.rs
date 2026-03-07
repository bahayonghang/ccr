use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{Connection, Row, params, params_from_iter, types::Value as SqlValue};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub channel: Option<String>,
    pub event_type: Option<String>,
    pub source: String,
    pub message: String,
    pub correlation_id: Option<String>,
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
            channel: row.get(3)?,
            event_type: row.get(4)?,
            source: row.get(5)?,
            message: row.get(6)?,
            correlation_id: row.get(7)?,
            metadata_json: row.get(8)?,
        })
    }
}

#[allow(dead_code)]
pub fn insert_log(conn: &Connection, entry: &LogEntry) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO log_entries (
            id,
            timestamp,
            level,
            channel,
            event_type,
            source,
            message,
            correlation_id,
            metadata_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            entry.id,
            entry.timestamp.to_rfc3339(),
            entry.level,
            entry.channel,
            entry.event_type,
            entry.source,
            entry.message,
            entry.correlation_id,
            entry.metadata_json,
        ],
    )?;
    Ok(())
}

#[allow(dead_code)]
pub fn insert_logs_batch(
    conn: &Connection,
    entries: &[LogEntry],
) -> Result<usize, rusqlite::Error> {
    if entries.is_empty() {
        return Ok(0);
    }

    conn.execute_batch("BEGIN IMMEDIATE TRANSACTION")?;
    let result = (|| {
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO log_entries (
                id,
                timestamp,
                level,
                channel,
                event_type,
                source,
                message,
                correlation_id,
                metadata_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )?;

        for entry in entries {
            stmt.execute(params![
                entry.id,
                entry.timestamp.to_rfc3339(),
                entry.level,
                entry.channel,
                entry.event_type,
                entry.source,
                entry.message,
                entry.correlation_id,
                entry.metadata_json,
            ])?;
        }

        Ok::<(), rusqlite::Error>(())
    })();

    match result {
        Ok(()) => {
            conn.execute_batch("COMMIT")?;
            Ok(entries.len())
        }
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(error)
        }
    }
}
fn utc_day_bounds(date: &str) -> Option<(String, String)> {
    let day = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let start = day.and_hms_opt(0, 0, 0)?;
    let end_day = day + chrono::Duration::days(1);
    let end = end_day.and_hms_opt(0, 0, 0)?;

    Some((start.and_utc().to_rfc3339(), end.and_utc().to_rfc3339()))
}

fn base_select_sql() -> &'static str {
    "SELECT id, timestamp, level, channel, event_type, source, message, correlation_id, metadata_json FROM log_entries"
}

#[allow(dead_code)]
pub fn get_logs_by_date(
    conn: &Connection,
    date: &str,
    limit: usize,
) -> Result<Vec<LogEntry>, rusqlite::Error> {
    let (start, end) = match utc_day_bounds(date) {
        Some(bounds) => bounds,
        None => return Ok(Vec::new()),
    };

    let mut stmt = conn.prepare(&format!(
        "{} WHERE timestamp >= ?1 AND timestamp < ?2 ORDER BY timestamp DESC LIMIT ?3",
        base_select_sql()
    ))?;

    let logs = stmt
        .query_map(params![start, end, limit as i64], LogEntry::from_row)?
        .filter_map(|row| row.ok())
        .collect();

    Ok(logs)
}

#[allow(dead_code)]
pub fn get_today_logs(conn: &Connection, limit: usize) -> Result<Vec<LogEntry>, rusqlite::Error> {
    let date = Utc::now().format("%Y-%m-%d").to_string();
    get_logs_by_date(conn, &date, limit)
}

#[allow(dead_code)]
pub fn get_recent_logs(conn: &Connection, count: usize) -> Result<Vec<LogEntry>, rusqlite::Error> {
    let mut stmt = conn.prepare(&format!(
        "{} ORDER BY timestamp DESC LIMIT ?1",
        base_select_sql()
    ))?;

    let logs = stmt
        .query_map(params![count as i64], LogEntry::from_row)?
        .filter_map(|row| row.ok())
        .collect();

    Ok(logs)
}

#[allow(dead_code)]
pub fn get_logs_by_level(
    conn: &Connection,
    level: &str,
    limit: usize,
) -> Result<Vec<LogEntry>, rusqlite::Error> {
    query_logs(conn, Some(level), None, limit)
}

#[allow(dead_code)]
pub fn query_logs(
    conn: &Connection,
    level: Option<&str>,
    channel: Option<&str>,
    limit: usize,
) -> Result<Vec<LogEntry>, rusqlite::Error> {
    let mut conditions = Vec::new();
    let mut values = Vec::new();

    if let Some(level) = level.filter(|value| !value.trim().is_empty()) {
        conditions.push("level = ?".to_string());
        values.push(SqlValue::from(level.to_string()));
    }

    if let Some(channel) = channel.filter(|value| !value.trim().is_empty()) {
        conditions.push("channel = ?".to_string());
        values.push(SqlValue::from(channel.to_string()));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let sql = format!(
        "{}{} ORDER BY timestamp DESC LIMIT ?",
        base_select_sql(),
        where_clause
    );
    values.push(SqlValue::from(limit as i64));

    let mut stmt = conn.prepare(&sql)?;
    let logs = stmt
        .query_map(params_from_iter(values.iter()), LogEntry::from_row)?
        .filter_map(|row| row.ok())
        .collect();

    Ok(logs)
}

#[allow(dead_code)]
pub fn delete_old_logs(conn: &Connection, retention_days: i64) -> Result<usize, rusqlite::Error> {
    let cutoff = Utc::now() - chrono::Duration::days(retention_days);
    conn.execute(
        "DELETE FROM log_entries WHERE timestamp < ?1",
        params![cutoff.to_rfc3339()],
    )
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn count_all_logs(conn: &Connection) -> Result<i64, rusqlite::Error> {
    conn.query_row("SELECT COUNT(*) FROM log_entries", [], |row| row.get(0))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    pub total_count: i64,
    pub today_count: i64,
    pub oldest_timestamp: Option<String>,
    pub newest_timestamp: Option<String>,
}

#[allow(dead_code)]
pub fn get_log_stats(conn: &Connection) -> Result<LogStats, rusqlite::Error> {
    let total_count = count_all_logs(conn)?;
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let today_count = count_logs_by_date(conn, &today)?;

    let oldest_timestamp = conn
        .query_row(
            "SELECT timestamp FROM log_entries ORDER BY timestamp ASC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    let newest_timestamp = conn
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

#[allow(dead_code)]
pub fn get_available_dates(conn: &Connection) -> Result<Vec<String>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT DISTINCT substr(timestamp, 1, 10) as date FROM log_entries ORDER BY date DESC",
    )?;

    let dates = stmt
        .query_map([], |row| row.get(0))?
        .filter_map(|row| row.ok())
        .collect();

    Ok(dates)
}

#[allow(dead_code)]
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

    fn sample_log(id: &str) -> LogEntry {
        LogEntry {
            id: id.to_string(),
            timestamp: Utc::now(),
            level: "info".to_string(),
            channel: Some("system".to_string()),
            event_type: Some("runtime.test".to_string()),
            source: "System".to_string(),
            message: "Test message".to_string(),
            correlation_id: Some("corr-1".to_string()),
            metadata_json: Some(r#"{"key":"value"}"#.to_string()),
        }
    }

    #[test]
    fn test_insert_and_get_log() {
        let conn = setup_test_db();
        insert_log(&conn, &sample_log("test-log-1")).unwrap();

        let logs = get_recent_logs(&conn, 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].id, "test-log-1");
        assert_eq!(logs[0].channel.as_deref(), Some("system"));
        assert_eq!(logs[0].event_type.as_deref(), Some("runtime.test"));
    }

    #[test]
    fn test_query_logs_by_channel() {
        let conn = setup_test_db();
        let mut other = sample_log("test-log-2");
        other.channel = Some("frontend".to_string());
        insert_logs_batch(&conn, &[sample_log("test-log-1"), other]).unwrap();

        let logs = query_logs(&conn, None, Some("frontend"), 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].channel.as_deref(), Some("frontend"));
    }

    #[test]
    fn test_query_logs_by_level() {
        let conn = setup_test_db();
        let mut warn_entry = sample_log("warn-log-1");
        warn_entry.level = "warn".to_string();
        let info_entry = sample_log("info-log-1");

        insert_logs_batch(&conn, &[info_entry, warn_entry]).unwrap();

        let logs = query_logs(&conn, Some("warn"), None, 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, "warn");
    }
    #[test]
    fn test_delete_old_logs() {
        let conn = setup_test_db();
        let mut entry = sample_log("old-log-1");
        entry.timestamp = Utc::now() - chrono::Duration::days(10);
        insert_log(&conn, &entry).unwrap();

        let deleted = delete_old_logs(&conn, 7).unwrap();
        assert_eq!(deleted, 1);
        assert!(get_recent_logs(&conn, 10).unwrap().is_empty());
    }

    #[test]
    fn test_get_log_stats() {
        let conn = setup_test_db();
        let entries = (0..5)
            .map(|index| sample_log(&format!("log-{index}")))
            .collect::<Vec<_>>();
        insert_logs_batch(&conn, &entries).unwrap();

        let stats = get_log_stats(&conn).unwrap();
        assert_eq!(stats.total_count, 5);
        assert!(stats.newest_timestamp.is_some());
    }
}
