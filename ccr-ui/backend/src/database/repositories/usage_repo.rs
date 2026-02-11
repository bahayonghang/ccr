// Usage tracking repository
// Handles CRUD operations for usage sources and records

use chrono::{DateTime, Utc};
use rusqlite::{Connection, Row, params};
use serde::{Deserialize, Serialize};

/// Usage source - tracks imported files with offsets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSource {
    pub id: String,
    pub platform: String,
    pub file_path: String,
    pub file_hash: String,
    pub last_offset: i64,
    pub updated_at: DateTime<Utc>,
}

impl UsageSource {
    fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        let updated_at_str: String = row.get(5)?;
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(Self {
            id: row.get(0)?,
            platform: row.get(1)?,
            file_path: row.get(2)?,
            file_hash: row.get(3)?,
            last_offset: row.get(4)?,
            updated_at,
        })
    }
}

/// Usage record - individual usage entry from log files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: String,
    pub platform: String,
    pub project_path: String,
    pub record_json: String,
    pub recorded_at: DateTime<Utc>,
    pub source_id: String,
}

impl UsageRecord {
    #[allow(dead_code)]
    fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        let recorded_at_str: String = row.get(4)?;
        let recorded_at = DateTime::parse_from_rfc3339(&recorded_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(Self {
            id: row.get(0)?,
            platform: row.get(1)?,
            project_path: row.get(2)?,
            record_json: row.get(3)?,
            recorded_at,
            source_id: row.get(5)?,
        })
    }
}

// ═══════════════════════════════════════════════════════════
// Usage Sources CRUD
// ═══════════════════════════════════════════════════════════

/// Get source by file path
pub fn get_source_by_path(
    conn: &Connection,
    file_path: &str,
) -> Result<Option<UsageSource>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, platform, file_path, file_hash, last_offset, updated_at
         FROM usage_sources WHERE file_path = ?1",
    )?;

    let result = stmt.query_row(params![file_path], UsageSource::from_row);

    match result {
        Ok(source) => Ok(Some(source)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Upsert source (insert or update)
pub fn upsert_source(conn: &Connection, source: &UsageSource) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO usage_sources
         (id, platform, file_path, file_hash, last_offset, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            source.id,
            source.platform,
            source.file_path,
            source.file_hash,
            source.last_offset,
            source.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

/// Get all sources for a platform
#[allow(dead_code)]
pub fn get_sources_by_platform(
    conn: &Connection,
    platform: &str,
) -> Result<Vec<UsageSource>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, platform, file_path, file_hash, last_offset, updated_at
         FROM usage_sources WHERE platform = ?1
         ORDER BY file_path ASC",
    )?;

    let sources = stmt
        .query_map(params![platform], UsageSource::from_row)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(sources)
}

/// Delete source by ID
#[allow(dead_code)]
pub fn delete_source(conn: &Connection, id: &str) -> Result<usize, rusqlite::Error> {
    conn.execute("DELETE FROM usage_sources WHERE id = ?1", params![id])
}

// ═══════════════════════════════════════════════════════════
// Usage Records CRUD
// ═══════════════════════════════════════════════════════════

/// Insert a usage record
pub fn insert_record(conn: &Connection, record: &UsageRecord) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO usage_records
         (id, platform, project_path, record_json, recorded_at, source_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            record.id,
            record.platform,
            record.project_path,
            record.record_json,
            record.recorded_at.to_rfc3339(),
            record.source_id,
        ],
    )?;
    Ok(())
}

/// Insert multiple records in a batch
pub fn insert_records_batch(
    conn: &Connection,
    records: &[UsageRecord],
) -> Result<usize, rusqlite::Error> {
    let mut count = 0;
    for record in records {
        insert_record(conn, record)?;
        count += 1;
    }
    Ok(count)
}

/// Get recent records by platform
#[allow(dead_code)]
pub fn get_recent_records(
    conn: &Connection,
    platform: &str,
    limit: usize,
) -> Result<Vec<UsageRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, platform, project_path, record_json, recorded_at, source_id
         FROM usage_records WHERE platform = ?1
         ORDER BY recorded_at DESC
         LIMIT ?2",
    )?;

    let records = stmt
        .query_map(params![platform, limit as i64], UsageRecord::from_row)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(records)
}

/// Get records by source ID
#[allow(dead_code)]
pub fn get_records_by_source(
    conn: &Connection,
    source_id: &str,
) -> Result<Vec<UsageRecord>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, platform, project_path, record_json, recorded_at, source_id
         FROM usage_records WHERE source_id = ?1
         ORDER BY recorded_at DESC",
    )?;

    let records = stmt
        .query_map(params![source_id], UsageRecord::from_row)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(records)
}

/// Count records by platform
#[allow(dead_code)]
pub fn count_records_by_platform(
    conn: &Connection,
    platform: &str,
) -> Result<i64, rusqlite::Error> {
    conn.query_row(
        "SELECT COUNT(*) FROM usage_records WHERE platform = ?1",
        params![platform],
        |row| row.get(0),
    )
}

/// Delete records by source ID
pub fn delete_records_by_source(
    conn: &Connection,
    source_id: &str,
) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "DELETE FROM usage_records WHERE source_id = ?1",
        params![source_id],
    )
}

/// Delete old records by date
pub fn delete_old_records(
    conn: &Connection,
    retention_days: i64,
) -> Result<usize, rusqlite::Error> {
    let cutoff = Utc::now() - chrono::Duration::days(retention_days);
    let cutoff_str = cutoff.to_rfc3339();

    conn.execute(
        "DELETE FROM usage_records WHERE recorded_at < ?1",
        params![cutoff_str],
    )
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub total_sources: i64,
    pub total_records: i64,
    pub records_by_platform: Vec<(String, i64)>,
}

/// Get usage statistics
pub fn get_usage_stats(conn: &Connection) -> Result<UsageStats, rusqlite::Error> {
    let total_sources: i64 =
        conn.query_row("SELECT COUNT(*) FROM usage_sources", [], |row| row.get(0))?;

    let total_records: i64 =
        conn.query_row("SELECT COUNT(*) FROM usage_records", [], |row| row.get(0))?;

    let mut stmt =
        conn.prepare("SELECT platform, COUNT(*) FROM usage_records GROUP BY platform")?;

    let records_by_platform: Vec<(String, i64)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(UsageStats {
        total_sources,
        total_records,
        records_by_platform,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::database::schema::CREATE_TABLES_SQL;
    use uuid::Uuid;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(CREATE_TABLES_SQL).unwrap();
        conn
    }

    #[test]
    fn test_source_crud() {
        let conn = setup_test_db();

        let source = UsageSource {
            id: Uuid::new_v4().to_string(),
            platform: "claude".to_string(),
            file_path: "/home/user/.claude/projects/test/usage.jsonl".to_string(),
            file_hash: "abc123".to_string(),
            last_offset: 1024,
            updated_at: Utc::now(),
        };

        // Insert
        upsert_source(&conn, &source).unwrap();

        // Get by path
        let found = get_source_by_path(&conn, &source.file_path).unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.platform, "claude");
        assert_eq!(found.last_offset, 1024);

        // Update offset
        let mut updated = source.clone();
        updated.last_offset = 2048;
        upsert_source(&conn, &updated).unwrap();

        let found = get_source_by_path(&conn, &source.file_path)
            .unwrap()
            .unwrap();
        assert_eq!(found.last_offset, 2048);
    }

    #[test]
    fn test_record_crud() {
        let conn = setup_test_db();

        let source_id = Uuid::new_v4().to_string();
        let record = UsageRecord {
            id: Uuid::new_v4().to_string(),
            platform: "claude".to_string(),
            project_path: "/home/user/projects/test".to_string(),
            record_json: r#"{"input_tokens": 100, "output_tokens": 50}"#.to_string(),
            recorded_at: Utc::now(),
            source_id: source_id.clone(),
        };

        // Insert
        insert_record(&conn, &record).unwrap();

        // Get by platform
        let records = get_recent_records(&conn, "claude", 10).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].platform, "claude");

        // Count
        let count = count_records_by_platform(&conn, "claude").unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_batch_insert() {
        let conn = setup_test_db();

        let source_id = Uuid::new_v4().to_string();
        let records: Vec<UsageRecord> = (0..5)
            .map(|i| UsageRecord {
                id: Uuid::new_v4().to_string(),
                platform: "codex".to_string(),
                project_path: format!("/project/{}", i),
                record_json: format!(r#"{{"tokens": {}}}"#, i * 100),
                recorded_at: Utc::now(),
                source_id: source_id.clone(),
            })
            .collect();

        let inserted = insert_records_batch(&conn, &records).unwrap();
        assert_eq!(inserted, 5);

        let count = count_records_by_platform(&conn, "codex").unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_usage_stats() {
        let conn = setup_test_db();

        // Add some sources and records
        let source = UsageSource {
            id: Uuid::new_v4().to_string(),
            platform: "gemini".to_string(),
            file_path: "/test.jsonl".to_string(),
            file_hash: "hash123".to_string(),
            last_offset: 0,
            updated_at: Utc::now(),
        };
        upsert_source(&conn, &source).unwrap();

        let record = UsageRecord {
            id: Uuid::new_v4().to_string(),
            platform: "gemini".to_string(),
            project_path: "/project".to_string(),
            record_json: "{}".to_string(),
            recorded_at: Utc::now(),
            source_id: source.id.clone(),
        };
        insert_record(&conn, &record).unwrap();

        let stats = get_usage_stats(&conn).unwrap();
        assert_eq!(stats.total_sources, 1);
        assert_eq!(stats.total_records, 1);
        assert!(
            stats
                .records_by_platform
                .iter()
                .any(|(p, c)| p == "gemini" && *c == 1)
        );
    }
}
