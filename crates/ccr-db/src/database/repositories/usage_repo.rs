// Usage tracking repository
// Handles CRUD operations for usage sources and records

use chrono::{DateTime, Utc};
use rusqlite::{Connection, Row, params};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UsageSourceState {
    #[default]
    Live,
    Missing,
    DeletedByUser,
}

impl UsageSourceState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Missing => "missing",
            Self::DeletedByUser => "deleted_by_user",
        }
    }

    fn from_raw(value: &str) -> Self {
        match value {
            "missing" => Self::Missing,
            "deleted_by_user" => Self::DeletedByUser,
            _ => Self::Live,
        }
    }
}

/// Usage source - tracks imported files with offsets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSource {
    pub id: String,
    pub platform: String,
    pub file_path: String,
    pub file_hash: String,
    pub last_offset: i64,
    pub source_state: UsageSourceState,
    pub file_size: Option<i64>,
    pub modified_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub raw_deleted_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl UsageSource {
    fn from_row(row: &Row<'_>) -> Result<Self, rusqlite::Error> {
        let updated_at_str: String = row.get(10)?;
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let modified_at = row
            .get::<_, Option<String>>(7)?
            .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
            .map(|dt| dt.with_timezone(&Utc));
        let last_seen_at = row
            .get::<_, Option<String>>(8)?
            .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
            .map(|dt| dt.with_timezone(&Utc));
        let raw_deleted_at = row
            .get::<_, Option<String>>(9)?
            .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(Self {
            id: row.get(0)?,
            platform: row.get(1)?,
            file_path: row.get(2)?,
            file_hash: row.get(3)?,
            last_offset: row.get(4)?,
            source_state: UsageSourceState::from_raw(&row.get::<_, String>(5)?),
            file_size: row.get(6)?,
            modified_at,
            last_seen_at,
            raw_deleted_at,
            updated_at,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSourceStateCounts {
    pub live: i64,
    pub missing: i64,
    pub deleted_by_user: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageHistoryCursor {
    pub platform: String,
    pub recent_window_days: i64,
    pub last_history_file_path: Option<String>,
    pub last_history_file_modified_at: Option<DateTime<Utc>>,
    pub last_history_offset: i64,
    pub recent_completed_at: Option<DateTime<Utc>>,
    pub history_completed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageCodexCheckpoint {
    pub source_id: String,
    pub session_id: String,
    pub project_path: String,
    pub model: Option<String>,
    pub last_line_number: i64,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub prefers_turn_completed: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSessionArchiveEntry {
    pub archive_id: String,
    pub session_id: String,
    pub platform: String,
    pub title: Option<String>,
    pub cwd: String,
    pub file_path: String,
    pub file_hash: Option<String>,
    pub message_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source_state: UsageSourceState,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub raw_deleted_at: Option<DateTime<Utc>>,
    pub archived_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArchivePlatformSummary {
    pub platform: String,
    pub session_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArchiveDailyTrend {
    pub date: String,
    pub platform: String,
    pub session_count: i64,
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
    // v3 提取列
    pub model: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cost_usd: f64,
}

impl UsageRecord {
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
            model: row.get(6)?,
            input_tokens: row.get::<_, i64>(7).unwrap_or(0),
            output_tokens: row.get::<_, i64>(8).unwrap_or(0),
            cache_read_tokens: row.get::<_, i64>(9).unwrap_or(0),
            cost_usd: row.get::<_, f64>(10).unwrap_or(0.0),
        })
    }
}

// ═══════════════════════════════════════════════════════════
// Usage Sources CRUD
// ═══════════════════════════════════════════════════════════

/// Get source by file path
#[allow(dead_code)]
pub fn get_source_by_path(
    conn: &Connection,
    file_path: &str,
) -> Result<Option<UsageSource>, rusqlite::Error> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, platform, file_path, file_hash, last_offset, source_state,
                file_size, modified_at, last_seen_at, raw_deleted_at, updated_at
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
#[allow(dead_code)]
pub fn upsert_source(conn: &Connection, source: &UsageSource) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO usage_sources
         (id, platform, file_path, file_hash, last_offset, source_state,
          file_size, modified_at, last_seen_at, raw_deleted_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            source.id,
            source.platform,
            source.file_path,
            source.file_hash,
            source.last_offset,
            source.source_state.as_str(),
            source.file_size,
            source.modified_at.map(|value| value.to_rfc3339()),
            source.last_seen_at.map(|value| value.to_rfc3339()),
            source.raw_deleted_at.map(|value| value.to_rfc3339()),
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
    let mut stmt = conn.prepare_cached(
        "SELECT id, platform, file_path, file_hash, last_offset, source_state,
                file_size, modified_at, last_seen_at, raw_deleted_at, updated_at
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

pub fn get_source_state_counts(
    conn: &Connection,
    platform: Option<&str>,
) -> Result<UsageSourceStateCounts, rusqlite::Error> {
    let mut counts = UsageSourceStateCounts {
        live: 0,
        missing: 0,
        deleted_by_user: 0,
    };

    if let Some(platform) = platform {
        let mut stmt = conn.prepare_cached(
            "SELECT source_state, COUNT(*) FROM usage_sources WHERE platform = ?1 GROUP BY source_state",
        )?;
        let rows = stmt.query_map(params![platform], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        for row in rows.flatten() {
            match UsageSourceState::from_raw(&row.0) {
                UsageSourceState::Live => counts.live = row.1,
                UsageSourceState::Missing => counts.missing = row.1,
                UsageSourceState::DeletedByUser => counts.deleted_by_user = row.1,
            }
        }
    } else {
        let mut stmt = conn.prepare_cached(
            "SELECT source_state, COUNT(*) FROM usage_sources GROUP BY source_state",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        for row in rows.flatten() {
            match UsageSourceState::from_raw(&row.0) {
                UsageSourceState::Live => counts.live = row.1,
                UsageSourceState::Missing => counts.missing = row.1,
                UsageSourceState::DeletedByUser => counts.deleted_by_user = row.1,
            }
        }
    }

    Ok(counts)
}

pub fn mark_sources_missing_by_platform(
    conn: &Connection,
    platform: &str,
    seen_paths: &[String],
) -> Result<usize, rusqlite::Error> {
    let tx = conn.unchecked_transaction()?;
    let now = Utc::now().to_rfc3339();

    let mut stmt = tx.prepare_cached(
        "SELECT file_path FROM usage_sources
         WHERE platform = ?1 AND source_state = 'live'",
    )?;
    let rows = stmt.query_map(params![platform], |row| row.get::<_, String>(0))?;
    let seen: HashSet<&str> = seen_paths.iter().map(String::as_str).collect();
    let mut changed = 0usize;

    for row in rows.flatten() {
        if seen.contains(row.as_str()) {
            continue;
        }
        changed += tx.execute(
            "UPDATE usage_sources
             SET source_state = 'missing', raw_deleted_at = COALESCE(raw_deleted_at, ?1), updated_at = ?1
             WHERE platform = ?2 AND file_path = ?3",
            params![now, platform, row],
        )?;
    }

    drop(stmt);
    tx.commit()?;
    Ok(changed)
}

pub fn mark_source_deleted_by_path(
    conn: &Connection,
    platform: &str,
    file_path: &str,
) -> Result<usize, rusqlite::Error> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE usage_sources
         SET source_state = 'deleted_by_user', raw_deleted_at = COALESCE(raw_deleted_at, ?1), updated_at = ?1
         WHERE platform = ?2 AND file_path = ?3",
        params![now, platform, file_path],
    )
}

// ═══════════════════════════════════════════════════════════
// Usage Records CRUD
// ═══════════════════════════════════════════════════════════

/// Insert a usage record
#[allow(dead_code)]
pub fn insert_record(conn: &Connection, record: &UsageRecord) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO usage_records
         (id, platform, project_path, record_json, recorded_at, source_id,
          model, input_tokens, output_tokens, cache_read_tokens, cost_usd)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            record.id,
            record.platform,
            record.project_path,
            record.record_json,
            record.recorded_at.to_rfc3339(),
            record.source_id,
            record.model,
            record.input_tokens,
            record.output_tokens,
            record.cache_read_tokens,
            record.cost_usd,
        ],
    )?;
    // Sync daily aggregation
    upsert_daily_agg_from_records(conn, std::slice::from_ref(record))?;
    Ok(())
}

/// Insert multiple records in a batch
#[allow(dead_code)]
pub fn insert_records_batch(
    conn: &Connection,
    records: &[UsageRecord],
) -> Result<usize, rusqlite::Error> {
    if records.is_empty() {
        return Ok(0);
    }

    let tx = conn.unchecked_transaction()?;
    let mut stmt = tx.prepare(
        "INSERT OR REPLACE INTO usage_records
         (id, platform, project_path, record_json, recorded_at, source_id,
          model, input_tokens, output_tokens, cache_read_tokens, cost_usd)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
    )?;

    let mut count = 0;
    for record in records {
        stmt.execute(params![
            record.id,
            record.platform,
            record.project_path,
            record.record_json,
            record.recorded_at.to_rfc3339(),
            record.source_id,
            record.model,
            record.input_tokens,
            record.output_tokens,
            record.cache_read_tokens,
            record.cost_usd,
        ])?;
        count += 1;
    }

    drop(stmt);

    // Sync daily aggregation table
    upsert_daily_agg_from_records(&tx, records)?;

    tx.commit()?;
    Ok(count)
}

/// Update usage_daily_agg from a batch of records (within an existing transaction)
fn upsert_daily_agg_from_records(
    conn: &Connection,
    records: &[UsageRecord],
) -> Result<(), rusqlite::Error> {
    if records.is_empty() {
        return Ok(());
    }

    // Group by (date, platform)
    #[allow(clippy::type_complexity)]
    let mut agg: HashMap<(String, String), (i64, i64, i64, i64, f64)> = HashMap::new();
    for r in records {
        let date = r.recorded_at.format("%Y-%m-%d").to_string();
        let entry = agg.entry((date, r.platform.clone())).or_default();
        entry.0 += 1;
        entry.1 += r.input_tokens;
        entry.2 += r.output_tokens;
        entry.3 += r.cache_read_tokens;
        entry.4 += r.cost_usd;
    }

    let mut stmt = conn.prepare(
        "INSERT INTO usage_daily_agg (date, platform, request_count, input_tokens, output_tokens, cache_read_tokens, cost_usd)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(date, platform) DO UPDATE SET
           request_count = request_count + excluded.request_count,
           input_tokens = input_tokens + excluded.input_tokens,
           output_tokens = output_tokens + excluded.output_tokens,
           cache_read_tokens = cache_read_tokens + excluded.cache_read_tokens,
           cost_usd = cost_usd + excluded.cost_usd",
    )?;

    for ((date, platform), (count, input, output, cache, cost)) in &agg {
        stmt.execute(params![date, platform, count, input, output, cache, cost])?;
    }

    Ok(())
}

fn refresh_daily_agg_entry(
    conn: &Connection,
    date: &str,
    platform: &str,
) -> Result<(), rusqlite::Error> {
    let (request_count, input_tokens, output_tokens, cache_read_tokens, cost_usd): (
        i64,
        i64,
        i64,
        i64,
        f64,
    ) = conn.query_row(
        "SELECT COUNT(*),
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(cache_read_tokens), 0),
                COALESCE(SUM(cost_usd), 0)
         FROM usage_records
         WHERE substr(recorded_at, 1, 10) = ?1 AND platform = ?2",
        params![date, platform],
        |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        },
    )?;

    if request_count == 0 {
        conn.execute(
            "DELETE FROM usage_daily_agg WHERE date = ?1 AND platform = ?2",
            params![date, platform],
        )?;
        return Ok(());
    }

    conn.execute(
        "INSERT INTO usage_daily_agg (date, platform, request_count, input_tokens, output_tokens, cache_read_tokens, cost_usd)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(date, platform) DO UPDATE SET
           request_count = excluded.request_count,
           input_tokens = excluded.input_tokens,
           output_tokens = excluded.output_tokens,
           cache_read_tokens = excluded.cache_read_tokens,
           cost_usd = excluded.cost_usd",
        params![
            date,
            platform,
            request_count,
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cost_usd,
        ],
    )?;

    Ok(())
}

const USAGE_RECORD_COLUMNS: &str = "id, platform, project_path, record_json, recorded_at, source_id, model, input_tokens, output_tokens, cache_read_tokens, cost_usd";

/// Get recent records by platform
#[allow(dead_code)]
pub fn get_recent_records(
    conn: &Connection,
    platform: &str,
    limit: usize,
) -> Result<Vec<UsageRecord>, rusqlite::Error> {
    let sql = format!(
        "SELECT {} FROM usage_records WHERE platform = ?1 ORDER BY recorded_at DESC LIMIT ?2",
        USAGE_RECORD_COLUMNS
    );
    let mut stmt = conn.prepare(&sql)?;

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
    let sql = format!(
        "SELECT {} FROM usage_records WHERE source_id = ?1 ORDER BY recorded_at DESC",
        USAGE_RECORD_COLUMNS
    );
    let mut stmt = conn.prepare(&sql)?;

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
#[allow(dead_code)]
pub fn delete_records_by_source(
    conn: &Connection,
    source_id: &str,
) -> Result<usize, rusqlite::Error> {
    let mut affected_keys = HashSet::new();
    {
        let mut stmt = conn.prepare_cached(
            "SELECT DISTINCT substr(recorded_at, 1, 10), platform
             FROM usage_records
             WHERE source_id = ?1",
        )?;
        let rows = stmt.query_map(params![source_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows.flatten() {
            affected_keys.insert(row);
        }
    }

    let tx = conn.unchecked_transaction()?;
    let deleted = tx.execute(
        "DELETE FROM usage_records WHERE source_id = ?1",
        params![source_id],
    )?;

    for (date, platform) in affected_keys {
        refresh_daily_agg_entry(&tx, &date, &platform)?;
    }

    tx.commit()?;
    Ok(deleted)
}

pub fn get_history_cursor(
    conn: &Connection,
    platform: &str,
) -> Result<Option<UsageHistoryCursor>, rusqlite::Error> {
    let mut stmt = conn.prepare_cached(
        "SELECT platform, recent_window_days, last_history_file_path, last_history_file_modified_at,
                last_history_offset, recent_completed_at, history_completed_at, updated_at
         FROM usage_history_cursor
         WHERE platform = ?1",
    )?;

    let result = stmt.query_row(params![platform], |row| {
        Ok(UsageHistoryCursor {
            platform: row.get(0)?,
            recent_window_days: row.get(1)?,
            last_history_file_path: row.get(2)?,
            last_history_file_modified_at: row
                .get::<_, Option<String>>(3)?
                .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            last_history_offset: row.get(4)?,
            recent_completed_at: row
                .get::<_, Option<String>>(5)?
                .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            history_completed_at: row
                .get::<_, Option<String>>(6)?
                .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            updated_at: row
                .get::<_, String>(7)
                .ok()
                .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
        })
    });

    match result {
        Ok(cursor) => Ok(Some(cursor)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn upsert_history_cursor(
    conn: &Connection,
    cursor: &UsageHistoryCursor,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO usage_history_cursor (
            platform, recent_window_days, last_history_file_path, last_history_file_modified_at,
            last_history_offset, recent_completed_at, history_completed_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(platform) DO UPDATE SET
            recent_window_days = excluded.recent_window_days,
            last_history_file_path = excluded.last_history_file_path,
            last_history_file_modified_at = excluded.last_history_file_modified_at,
            last_history_offset = excluded.last_history_offset,
            recent_completed_at = excluded.recent_completed_at,
            history_completed_at = excluded.history_completed_at,
            updated_at = excluded.updated_at",
        params![
            cursor.platform,
            cursor.recent_window_days,
            cursor.last_history_file_path,
            cursor
                .last_history_file_modified_at
                .map(|value| value.to_rfc3339()),
            cursor.last_history_offset,
            cursor.recent_completed_at.map(|value| value.to_rfc3339()),
            cursor.history_completed_at.map(|value| value.to_rfc3339()),
            cursor.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

pub fn get_codex_checkpoint(
    conn: &Connection,
    source_id: &str,
) -> Result<Option<UsageCodexCheckpoint>, rusqlite::Error> {
    let mut stmt = conn.prepare_cached(
        "SELECT source_id, session_id, project_path, model, last_line_number,
                input_tokens, cached_input_tokens, output_tokens, prefers_turn_completed, updated_at
         FROM usage_codex_checkpoint
         WHERE source_id = ?1",
    )?;
    let result = stmt.query_row(params![source_id], |row| {
        Ok(UsageCodexCheckpoint {
            source_id: row.get(0)?,
            session_id: row.get(1)?,
            project_path: row.get(2)?,
            model: row.get(3)?,
            last_line_number: row.get(4)?,
            input_tokens: row.get(5)?,
            cached_input_tokens: row.get(6)?,
            output_tokens: row.get(7)?,
            prefers_turn_completed: row.get::<_, i64>(8).unwrap_or(0) > 0,
            updated_at: row
                .get::<_, String>(9)
                .ok()
                .and_then(|value| DateTime::parse_from_rfc3339(&value).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
        })
    });

    match result {
        Ok(checkpoint) => Ok(Some(checkpoint)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn upsert_codex_checkpoint(
    conn: &Connection,
    checkpoint: &UsageCodexCheckpoint,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO usage_codex_checkpoint (
            source_id, session_id, project_path, model, last_line_number,
            input_tokens, cached_input_tokens, output_tokens, prefers_turn_completed, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(source_id) DO UPDATE SET
            session_id = excluded.session_id,
            project_path = excluded.project_path,
            model = excluded.model,
            last_line_number = excluded.last_line_number,
            input_tokens = excluded.input_tokens,
            cached_input_tokens = excluded.cached_input_tokens,
            output_tokens = excluded.output_tokens,
            prefers_turn_completed = excluded.prefers_turn_completed,
            updated_at = excluded.updated_at",
        params![
            checkpoint.source_id,
            checkpoint.session_id,
            checkpoint.project_path,
            checkpoint.model,
            checkpoint.last_line_number,
            checkpoint.input_tokens,
            checkpoint.cached_input_tokens,
            checkpoint.output_tokens,
            if checkpoint.prefers_turn_completed {
                1
            } else {
                0
            },
            checkpoint.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

pub fn delete_codex_checkpoint(
    conn: &Connection,
    source_id: &str,
) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "DELETE FROM usage_codex_checkpoint WHERE source_id = ?1",
        params![source_id],
    )
}

pub fn upsert_session_archive_entry(
    conn: &Connection,
    entry: &UsageSessionArchiveEntry,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO usage_session_archive (
            archive_id, session_id, platform, title, cwd, file_path, file_hash,
            message_count, created_at, updated_at, source_state, last_seen_at, raw_deleted_at, archived_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
         ON CONFLICT(file_path) DO UPDATE SET
            archive_id = excluded.archive_id,
            session_id = excluded.session_id,
            platform = excluded.platform,
            title = excluded.title,
            cwd = excluded.cwd,
            file_hash = excluded.file_hash,
            message_count = excluded.message_count,
            created_at = excluded.created_at,
            updated_at = excluded.updated_at,
            source_state = excluded.source_state,
            last_seen_at = excluded.last_seen_at,
            raw_deleted_at = excluded.raw_deleted_at,
            archived_at = excluded.archived_at",
        params![
            entry.archive_id,
            entry.session_id,
            entry.platform,
            entry.title,
            entry.cwd,
            entry.file_path,
            entry.file_hash,
            entry.message_count,
            entry.created_at.to_rfc3339(),
            entry.updated_at.to_rfc3339(),
            entry.source_state.as_str(),
            entry.last_seen_at.map(|value| value.to_rfc3339()),
            entry.raw_deleted_at.map(|value| value.to_rfc3339()),
            entry.archived_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

pub fn mark_session_archive_missing_by_platform(
    conn: &Connection,
    platform: &str,
    seen_paths: &[String],
) -> Result<usize, rusqlite::Error> {
    let tx = conn.unchecked_transaction()?;
    let now = Utc::now().to_rfc3339();
    let seen: HashSet<&str> = seen_paths.iter().map(String::as_str).collect();
    let mut stmt = tx.prepare_cached(
        "SELECT file_path FROM usage_session_archive
         WHERE platform = ?1 AND source_state = 'live'",
    )?;
    let rows = stmt.query_map(params![platform], |row| row.get::<_, String>(0))?;
    let mut changed = 0usize;

    for row in rows.flatten() {
        if seen.contains(row.as_str()) {
            continue;
        }
        changed += tx.execute(
            "UPDATE usage_session_archive
             SET source_state = 'missing', raw_deleted_at = COALESCE(raw_deleted_at, ?1), updated_at = ?1
             WHERE platform = ?2 AND file_path = ?3",
            params![now, platform, row],
        )?;
    }

    drop(stmt);
    tx.commit()?;
    Ok(changed)
}

pub fn mark_session_archive_deleted_by_path(
    conn: &Connection,
    platform: &str,
    file_path: &str,
) -> Result<usize, rusqlite::Error> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE usage_session_archive
         SET source_state = 'deleted_by_user', raw_deleted_at = COALESCE(raw_deleted_at, ?1), updated_at = ?1
         WHERE platform = ?2 AND file_path = ?3",
        params![now, platform, file_path],
    )
}

pub fn has_any_session_archive(conn: &Connection) -> Result<bool, rusqlite::Error> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM usage_session_archive", [], |row| {
        row.get(0)
    })?;
    Ok(count > 0)
}

pub fn get_session_archive_platform_summaries(
    conn: &Connection,
    start: &Option<String>,
    end: &Option<String>,
) -> Result<Vec<SessionArchivePlatformSummary>, rusqlite::Error> {
    let mut clauses = Vec::new();
    let mut bind_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(start) = start {
        clauses.push("created_at >= ?");
        bind_params.push(Box::new(start.clone()));
    }
    if let Some(end) = end {
        clauses.push("created_at <= ?");
        bind_params.push(Box::new(end.clone()));
    }

    let where_sql = if clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", clauses.join(" AND "))
    };
    let sql = format!(
        "SELECT platform, COUNT(*)
         FROM usage_session_archive{}
         GROUP BY platform
         ORDER BY platform ASC",
        where_sql
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        bind_params.iter().map(|param| param.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(SessionArchivePlatformSummary {
                platform: row.get(0)?,
                session_count: row.get(1)?,
            })
        })?
        .filter_map(|row| row.ok())
        .collect();
    Ok(rows)
}

pub fn get_session_archive_daily_trends(
    conn: &Connection,
    start: &Option<String>,
    end: &Option<String>,
) -> Result<Vec<SessionArchiveDailyTrend>, rusqlite::Error> {
    let mut clauses = Vec::new();
    let mut bind_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(start) = start {
        clauses.push("created_at >= ?");
        bind_params.push(Box::new(start.clone()));
    }
    if let Some(end) = end {
        clauses.push("created_at <= ?");
        bind_params.push(Box::new(end.clone()));
    }

    let where_sql = if clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", clauses.join(" AND "))
    };
    let sql = format!(
        "SELECT substr(created_at, 1, 10), platform, COUNT(*)
         FROM usage_session_archive{}
         GROUP BY substr(created_at, 1, 10), platform
         ORDER BY substr(created_at, 1, 10) ASC, platform ASC",
        where_sql
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        bind_params.iter().map(|param| param.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(SessionArchiveDailyTrend {
                date: row.get(0)?,
                platform: row.get(1)?,
                session_count: row.get(2)?,
            })
        })?
        .filter_map(|row| row.ok())
        .collect();
    Ok(rows)
}

// ═══════════════════════════════════════════════════════════
// V2 聚合查询
// ═══════════════════════════════════════════════════════════

/// 使用量汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub total_requests: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cache_read_tokens: i64,
    pub total_cost_usd: f64,
    pub cache_efficiency: f64,
}

/// 每日趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyTrend {
    pub date: String,
    pub request_count: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cost_usd: f64,
}

/// 模型统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStat {
    pub model: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
}

/// 项目统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStat {
    pub project_path: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
}

/// 平台汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformSummaryStat {
    pub platform: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
}

/// 平台每日趋势
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformDailyTrend {
    pub date: String,
    pub platform: String,
    pub request_count: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cost_usd: f64,
}

/// 分页日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedLogs {
    pub records: Vec<UsageRecord>,
    pub total: Option<i64>,
    pub page: i64,
    pub page_size: i64,
    pub next_cursor: Option<String>,
}

/// 构建平台+时间范围的 WHERE 子句和参数
fn build_where_clause(
    platform: &Option<String>,
    start: &Option<String>,
    end: &Option<String>,
) -> (String, Vec<Box<dyn rusqlite::types::ToSql>>) {
    let mut conditions: Vec<String> = Vec::new();
    let mut bind_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(p) = platform {
        bind_params.push(Box::new(p.clone()));
        conditions.push(format!("platform = ?{}", bind_params.len()));
    }
    if let Some(s) = start {
        // 纯日期 "YYYY-MM-DD" 自然 <= 任何当天的 RFC3339 时间戳，无需补齐
        bind_params.push(Box::new(s.clone()));
        conditions.push(format!("recorded_at >= ?{}", bind_params.len()));
    }
    if let Some(e) = end {
        // 前端传纯日期 "YYYY-MM-DD"，但 recorded_at 是 RFC3339 格式
        // "2026-02-15" < "2026-02-15T00:00:00Z"，会排除当天记录
        // 追加 T23:59:59Z 确保包含当天所有记录
        let end_val = if e.len() == 10 && !e.contains('T') {
            format!("{}T23:59:59Z", e)
        } else {
            e.clone()
        };
        bind_params.push(Box::new(end_val));
        conditions.push(format!("recorded_at <= ?{}", bind_params.len()));
    }

    let where_sql = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    (where_sql, bind_params)
}

fn build_daily_agg_where_clause(
    platform: &Option<String>,
    start: &Option<String>,
    end: &Option<String>,
) -> (String, Vec<Box<dyn rusqlite::types::ToSql>>) {
    let mut conditions: Vec<String> = Vec::new();
    let mut bind_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(p) = platform {
        bind_params.push(Box::new(p.clone()));
        conditions.push(format!("platform = ?{}", bind_params.len()));
    }
    if let Some(s) = start {
        bind_params.push(Box::new(s.clone()));
        conditions.push(format!("date >= ?{}", bind_params.len()));
    }
    if let Some(e) = end {
        bind_params.push(Box::new(e.clone()));
        conditions.push(format!("date <= ?{}", bind_params.len()));
    }

    let where_sql = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    (where_sql, bind_params)
}

/// 构建日志分页查询的 WHERE 子句和参数
fn build_logs_where_clause(
    platform: &Option<String>,
    model_filter: &Option<String>,
    start: &Option<String>,
    end: &Option<String>,
) -> (String, Vec<Box<dyn rusqlite::types::ToSql>>) {
    let mut conditions: Vec<String> = Vec::new();
    let mut bind_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(p) = platform {
        bind_params.push(Box::new(p.clone()));
        conditions.push(format!("platform = ?{}", bind_params.len()));
    }
    if let Some(m) = model_filter {
        bind_params.push(Box::new(m.clone()));
        conditions.push(format!("model = ?{}", bind_params.len()));
    }
    if let Some(s) = start {
        bind_params.push(Box::new(s.clone()));
        conditions.push(format!("recorded_at >= ?{}", bind_params.len()));
    }
    if let Some(e) = end {
        let end_val = if e.len() == 10 && !e.contains('T') {
            format!("{}T23:59:59Z", e)
        } else {
            e.clone()
        };
        bind_params.push(Box::new(end_val));
        conditions.push(format!("recorded_at <= ?{}", bind_params.len()));
    }

    let where_sql = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };
    (where_sql, bind_params)
}

/// 获取使用量汇总
#[allow(dead_code)]
pub fn get_usage_summary(
    conn: &Connection,
    platform: &Option<String>,
    start: &Option<String>,
    end: &Option<String>,
) -> Result<UsageSummary, rusqlite::Error> {
    let (where_sql, bind_params) = build_where_clause(platform, start, end);
    let sql = format!(
        "SELECT COUNT(*), COALESCE(SUM(input_tokens),0), COALESCE(SUM(output_tokens),0),
                COALESCE(SUM(cache_read_tokens),0), COALESCE(SUM(cost_usd),0)
         FROM usage_records{}",
        where_sql
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        bind_params.iter().map(|p| p.as_ref()).collect();
    conn.query_row(&sql, params_ref.as_slice(), |row| {
        let total_requests: i64 = row.get(0)?;
        let total_input: i64 = row.get(1)?;
        let total_output: i64 = row.get(2)?;
        let total_cache: i64 = row.get(3)?;
        let total_cost: f64 = row.get(4)?;
        let all_input = total_input + total_cache;
        let cache_efficiency = if all_input > 0 {
            total_cache as f64 / all_input as f64
        } else {
            0.0
        };
        Ok(UsageSummary {
            total_requests,
            total_input_tokens: total_input,
            total_output_tokens: total_output,
            total_cache_read_tokens: total_cache,
            total_cost_usd: total_cost,
            cache_efficiency,
        })
    })
}

/// 获取每日趋势（从预聚合表查询）
#[allow(dead_code)]
pub fn get_daily_trends(
    conn: &Connection,
    platform: &Option<String>,
    start: &Option<String>,
    end: &Option<String>,
) -> Result<Vec<DailyTrend>, rusqlite::Error> {
    let (where_sql, bind_params) = build_daily_agg_where_clause(platform, start, end);

    let sql = format!(
        "SELECT date, SUM(request_count), SUM(input_tokens), SUM(output_tokens),
                SUM(cache_read_tokens), SUM(cost_usd)
         FROM usage_daily_agg{}
         GROUP BY date ORDER BY date ASC",
        where_sql
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        bind_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(DailyTrend {
                date: row.get(0)?,
                request_count: row.get(1)?,
                input_tokens: row.get::<_, i64>(2).unwrap_or(0),
                output_tokens: row.get::<_, i64>(3).unwrap_or(0),
                cache_read_tokens: row.get::<_, i64>(4).unwrap_or(0),
                cost_usd: row.get::<_, f64>(5).unwrap_or(0.0),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// 获取按平台聚合的汇总（从预聚合表查询）
#[allow(dead_code)]
pub fn get_platform_summaries(
    conn: &Connection,
    start: &Option<String>,
    end: &Option<String>,
) -> Result<Vec<PlatformSummaryStat>, rusqlite::Error> {
    let (where_sql, bind_params) = build_daily_agg_where_clause(&None, start, end);
    let sql = format!(
        "SELECT platform,
                SUM(request_count),
                SUM(input_tokens + output_tokens),
                SUM(cost_usd)
         FROM usage_daily_agg{}
         GROUP BY platform
         ORDER BY platform ASC",
        where_sql
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        bind_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(PlatformSummaryStat {
                platform: row.get(0)?,
                request_count: row.get(1)?,
                total_tokens: row.get::<_, i64>(2).unwrap_or(0),
                total_cost: row.get::<_, f64>(3).unwrap_or(0.0),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// 获取按 (date, platform) 聚合的趋势（从预聚合表查询）
#[allow(dead_code)]
pub fn get_daily_platform_trends(
    conn: &Connection,
    start: &Option<String>,
    end: &Option<String>,
) -> Result<Vec<PlatformDailyTrend>, rusqlite::Error> {
    let (where_sql, bind_params) = build_daily_agg_where_clause(&None, start, end);
    let sql = format!(
        "SELECT date, platform,
                SUM(request_count),
                SUM(input_tokens),
                SUM(output_tokens),
                SUM(cache_read_tokens),
                SUM(cost_usd)
         FROM usage_daily_agg{}
         GROUP BY date, platform
         ORDER BY date ASC, platform ASC",
        where_sql
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        bind_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(PlatformDailyTrend {
                date: row.get(0)?,
                platform: row.get(1)?,
                request_count: row.get(2)?,
                input_tokens: row.get::<_, i64>(3).unwrap_or(0),
                output_tokens: row.get::<_, i64>(4).unwrap_or(0),
                cache_read_tokens: row.get::<_, i64>(5).unwrap_or(0),
                cost_usd: row.get::<_, f64>(6).unwrap_or(0.0),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// 获取模型统计
#[allow(dead_code)]
pub fn get_model_stats(
    conn: &Connection,
    platform: &Option<String>,
    start: &Option<String>,
    end: &Option<String>,
) -> Result<Vec<ModelStat>, rusqlite::Error> {
    let (where_sql, bind_params) = build_where_clause(platform, start, end);
    let sql = format!(
        "SELECT COALESCE(model,'unknown'), COUNT(*),
                SUM(input_tokens + output_tokens + cache_read_tokens),
                SUM(cost_usd)
         FROM usage_records{}
         GROUP BY model ORDER BY COUNT(*) DESC",
        where_sql
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        bind_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(ModelStat {
                model: row.get(0)?,
                request_count: row.get(1)?,
                total_tokens: row.get::<_, i64>(2).unwrap_or(0),
                total_cost: row.get::<_, f64>(3).unwrap_or(0.0),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// 获取项目统计
#[allow(dead_code)]
pub fn get_project_stats(
    conn: &Connection,
    platform: &Option<String>,
    start: &Option<String>,
    end: &Option<String>,
) -> Result<Vec<ProjectStat>, rusqlite::Error> {
    let (where_sql, bind_params) = build_where_clause(platform, start, end);
    let sql = format!(
        "SELECT project_path, COUNT(*),
                SUM(input_tokens + output_tokens + cache_read_tokens),
                SUM(cost_usd)
         FROM usage_records{}
         GROUP BY project_path ORDER BY COUNT(*) DESC",
        where_sql
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        bind_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(params_ref.as_slice(), |row| {
            Ok(ProjectStat {
                project_path: row.get(0)?,
                request_count: row.get(1)?,
                total_tokens: row.get::<_, i64>(2).unwrap_or(0),
                total_cost: row.get::<_, f64>(3).unwrap_or(0.0),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}

/// 获取热力图数据（从预聚合表查询）
#[allow(dead_code)]
pub fn get_heatmap_data(
    conn: &Connection,
    platform: &Option<String>,
    days: i64,
) -> Result<HashMap<String, i64>, rusqlite::Error> {
    let cutoff = Utc::now() - chrono::Duration::days(days);
    let cutoff_str = cutoff.format("%Y-%m-%d").to_string();

    let (where_sql, bind_params) = build_daily_agg_where_clause(platform, &Some(cutoff_str), &None);

    let sql = format!(
        "SELECT date, SUM(request_count) FROM usage_daily_agg{} GROUP BY date",
        where_sql
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        bind_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let mut map = HashMap::new();
    let rows = stmt.query_map(params_ref.as_slice(), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    for row in rows.flatten() {
        map.insert(row.0, row.1);
    }
    Ok(map)
}

/// 获取分页日志
#[allow(dead_code, clippy::too_many_arguments)]
pub fn get_paginated_logs(
    conn: &Connection,
    platform: &Option<String>,
    start: &Option<String>,
    end: &Option<String>,
    page: i64,
    page_size: i64,
    model_filter: &Option<String>,
    include_total: bool,
) -> Result<PaginatedLogs, rusqlite::Error> {
    let (where_sql, mut bind_params) = build_logs_where_clause(platform, model_filter, start, end);

    let total = if include_total {
        let count_sql = format!("SELECT COUNT(*) FROM usage_records{}", where_sql);
        let params_ref: Vec<&dyn rusqlite::types::ToSql> =
            bind_params.iter().map(|p| p.as_ref()).collect();
        Some(conn.query_row(&count_sql, params_ref.as_slice(), |row| row.get(0))?)
    } else {
        None
    };

    // Data
    let offset = (page - 1) * page_size;
    bind_params.push(Box::new(page_size));
    bind_params.push(Box::new(offset));
    let data_sql = format!(
        "SELECT {} FROM usage_records{} ORDER BY recorded_at DESC, id DESC LIMIT ?{} OFFSET ?{}",
        USAGE_RECORD_COLUMNS,
        where_sql,
        bind_params.len() - 1,
        bind_params.len()
    );

    let params_ref2: Vec<&dyn rusqlite::types::ToSql> =
        bind_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&data_sql)?;
    let records = stmt
        .query_map(params_ref2.as_slice(), UsageRecord::from_row)?
        .filter_map(|r| r.ok())
        .collect();

    Ok(PaginatedLogs {
        records,
        total,
        page,
        page_size,
        next_cursor: None,
    })
}

fn parse_cursor(cursor: &str) -> Option<(String, String)> {
    let mut parts = cursor.splitn(2, '|');
    let recorded_at = parts.next()?.trim().to_string();
    let id = parts.next()?.trim().to_string();
    if recorded_at.is_empty() || id.is_empty() {
        return None;
    }
    Some((recorded_at, id))
}

fn format_cursor(record: &UsageRecord) -> String {
    format!("{}|{}", record.recorded_at.to_rfc3339(), record.id)
}

/// 基于游标分页日志（Keyset Pagination）
#[allow(dead_code, clippy::too_many_arguments)]
pub fn get_logs_by_cursor(
    conn: &Connection,
    platform: &Option<String>,
    start: &Option<String>,
    end: &Option<String>,
    page_size: i64,
    model_filter: &Option<String>,
    cursor: &Option<String>,
    include_total: bool,
) -> Result<PaginatedLogs, rusqlite::Error> {
    let (base_where, mut bind_params) = build_logs_where_clause(platform, model_filter, start, end);
    let mut conditions = if base_where.is_empty() {
        Vec::new()
    } else {
        vec![base_where.trim_start_matches(" WHERE ").to_string()]
    };

    if let Some(raw_cursor) = cursor
        && let Some((recorded_at, id)) = parse_cursor(raw_cursor)
    {
        bind_params.push(Box::new(recorded_at.clone()));
        let recorded_idx = bind_params.len();
        bind_params.push(Box::new(recorded_at));
        let recorded_eq_idx = bind_params.len();
        bind_params.push(Box::new(id));
        let id_idx = bind_params.len();
        conditions.push(format!(
            "(recorded_at < ?{} OR (recorded_at = ?{} AND id < ?{}))",
            recorded_idx, recorded_eq_idx, id_idx
        ));
    }

    let where_sql = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

    let total = if include_total {
        let (count_where, count_params) =
            build_logs_where_clause(platform, model_filter, start, end);
        let count_sql = format!("SELECT COUNT(*) FROM usage_records{}", count_where);
        let count_params_ref: Vec<&dyn rusqlite::types::ToSql> =
            count_params.iter().map(|p| p.as_ref()).collect();
        Some(conn.query_row(&count_sql, count_params_ref.as_slice(), |row| row.get(0))?)
    } else {
        None
    };

    bind_params.push(Box::new(page_size + 1));
    let limit_idx = bind_params.len();
    let sql = format!(
        "SELECT {} FROM usage_records{} ORDER BY recorded_at DESC, id DESC LIMIT ?{}",
        USAGE_RECORD_COLUMNS, where_sql, limit_idx
    );

    let params_ref: Vec<&dyn rusqlite::types::ToSql> =
        bind_params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let mut records: Vec<UsageRecord> = stmt
        .query_map(params_ref.as_slice(), UsageRecord::from_row)?
        .filter_map(|r| r.ok())
        .collect();

    let next_cursor = if records.len() as i64 > page_size {
        records.pop();
        records.last().map(format_cursor)
    } else {
        None
    };

    Ok(PaginatedLogs {
        records,
        total,
        page: 1,
        page_size,
        next_cursor,
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
        // 添加 v3/v11 列（CREATE_TABLES_SQL 是早期 schema，不含后续扩展）
        for stmt in &[
            "ALTER TABLE usage_records ADD COLUMN model TEXT",
            "ALTER TABLE usage_records ADD COLUMN input_tokens INTEGER DEFAULT 0",
            "ALTER TABLE usage_records ADD COLUMN output_tokens INTEGER DEFAULT 0",
            "ALTER TABLE usage_records ADD COLUMN cache_read_tokens INTEGER DEFAULT 0",
            "ALTER TABLE usage_records ADD COLUMN cost_usd REAL DEFAULT 0",
            "ALTER TABLE usage_sources ADD COLUMN source_state TEXT NOT NULL DEFAULT 'live'",
            "ALTER TABLE usage_sources ADD COLUMN file_size INTEGER",
            "ALTER TABLE usage_sources ADD COLUMN modified_at TEXT",
            "ALTER TABLE usage_sources ADD COLUMN last_seen_at TEXT",
            "ALTER TABLE usage_sources ADD COLUMN raw_deleted_at TEXT",
            "CREATE TABLE IF NOT EXISTS usage_history_cursor (
                platform TEXT PRIMARY KEY,
                recent_window_days INTEGER NOT NULL DEFAULT 30,
                last_history_file_path TEXT,
                last_history_file_modified_at TEXT,
                last_history_offset INTEGER NOT NULL DEFAULT 0,
                recent_completed_at TEXT,
                history_completed_at TEXT,
                updated_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS usage_codex_checkpoint (
                source_id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                project_path TEXT NOT NULL,
                model TEXT,
                last_line_number INTEGER NOT NULL DEFAULT 0,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                cached_input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                prefers_turn_completed INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS usage_session_archive (
                archive_id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                platform TEXT NOT NULL,
                title TEXT,
                cwd TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_hash TEXT,
                message_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                source_state TEXT NOT NULL DEFAULT 'live',
                last_seen_at TEXT,
                raw_deleted_at TEXT,
                archived_at TEXT NOT NULL
            )",
        ] {
            let _ = conn.execute_batch(stmt);
        }
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
            source_state: UsageSourceState::Live,
            file_size: Some(1024),
            modified_at: Some(Utc::now()),
            last_seen_at: Some(Utc::now()),
            raw_deleted_at: None,
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
            model: None,
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cost_usd: 0.0,
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
                model: None,
                input_tokens: 0,
                output_tokens: 0,
                cache_read_tokens: 0,
                cost_usd: 0.0,
            })
            .collect();

        let inserted = insert_records_batch(&conn, &records).unwrap();
        assert_eq!(inserted, 5);

        let count = count_records_by_platform(&conn, "codex").unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_delete_records_by_source_keeps_daily_agg_in_sync() {
        let conn = setup_test_db();
        let recorded_at = DateTime::parse_from_rfc3339("2026-03-20T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let source_a = "source-a".to_string();
        let source_b = "source-b".to_string();
        let records = vec![
            UsageRecord {
                id: "r-a".to_string(),
                platform: "codex".to_string(),
                project_path: "/project/a".to_string(),
                record_json: "{}".to_string(),
                recorded_at,
                source_id: source_a.clone(),
                model: Some("gpt-5".to_string()),
                input_tokens: 100,
                output_tokens: 20,
                cache_read_tokens: 10,
                cost_usd: 1.0,
            },
            UsageRecord {
                id: "r-b".to_string(),
                platform: "codex".to_string(),
                project_path: "/project/b".to_string(),
                record_json: "{}".to_string(),
                recorded_at,
                source_id: source_b.clone(),
                model: Some("gpt-5".to_string()),
                input_tokens: 200,
                output_tokens: 40,
                cache_read_tokens: 20,
                cost_usd: 2.0,
            },
        ];

        insert_records_batch(&conn, &records).unwrap();

        let before = get_daily_trends(&conn, &Some("codex".to_string()), &None, &None).unwrap();
        assert_eq!(before.len(), 1);
        assert_eq!(before[0].request_count, 2);
        assert_eq!(before[0].input_tokens, 300);
        assert_eq!(before[0].output_tokens, 60);
        assert_eq!(before[0].cache_read_tokens, 30);

        delete_records_by_source(&conn, &source_a).unwrap();

        let after = get_daily_trends(&conn, &Some("codex".to_string()), &None, &None).unwrap();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].request_count, 1);
        assert_eq!(after[0].input_tokens, 200);
        assert_eq!(after[0].output_tokens, 40);
        assert_eq!(after[0].cache_read_tokens, 20);
        assert!((after[0].cost_usd - 2.0).abs() < 0.0001);
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
            source_state: UsageSourceState::Live,
            file_size: Some(512),
            modified_at: Some(Utc::now()),
            last_seen_at: Some(Utc::now()),
            raw_deleted_at: None,
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
            model: None,
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cost_usd: 0.0,
        };
        insert_record(&conn, &record).unwrap();
    }

    #[test]
    fn test_platform_rollups_use_daily_agg() {
        let conn = setup_test_db();
        let source_id = Uuid::new_v4().to_string();
        let records = vec![
            UsageRecord {
                id: "claude-day1".to_string(),
                platform: "claude".to_string(),
                project_path: "/project/claude".to_string(),
                record_json: "{}".to_string(),
                recorded_at: DateTime::parse_from_rfc3339("2026-03-01T10:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                source_id: source_id.clone(),
                model: Some("claude-sonnet-4-6".to_string()),
                input_tokens: 100,
                output_tokens: 50,
                cache_read_tokens: 10,
                cost_usd: 1.0,
            },
            UsageRecord {
                id: "codex-day1".to_string(),
                platform: "codex".to_string(),
                project_path: "/project/codex".to_string(),
                record_json: "{}".to_string(),
                recorded_at: DateTime::parse_from_rfc3339("2026-03-01T12:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                source_id: source_id.clone(),
                model: Some("gpt-5".to_string()),
                input_tokens: 40,
                output_tokens: 20,
                cache_read_tokens: 0,
                cost_usd: 0.4,
            },
            UsageRecord {
                id: "claude-day2".to_string(),
                platform: "claude".to_string(),
                project_path: "/project/claude".to_string(),
                record_json: "{}".to_string(),
                recorded_at: DateTime::parse_from_rfc3339("2026-03-02T09:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                source_id,
                model: Some("claude-sonnet-4-6".to_string()),
                input_tokens: 80,
                output_tokens: 20,
                cache_read_tokens: 5,
                cost_usd: 0.8,
            },
        ];
        insert_records_batch(&conn, &records).unwrap();

        let summaries = get_platform_summaries(
            &conn,
            &Some("2026-03-01".to_string()),
            &Some("2026-03-02".to_string()),
        )
        .unwrap();
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0].platform, "claude");
        assert_eq!(summaries[0].request_count, 2);
        assert_eq!(summaries[0].total_tokens, 250);
        assert!((summaries[0].total_cost - 1.8).abs() < 0.0001);
        assert_eq!(summaries[1].platform, "codex");
        assert_eq!(summaries[1].request_count, 1);
        assert_eq!(summaries[1].total_tokens, 60);

        let daily = get_daily_platform_trends(
            &conn,
            &Some("2026-03-01".to_string()),
            &Some("2026-03-02".to_string()),
        )
        .unwrap();
        assert_eq!(daily.len(), 3);
        assert_eq!(daily[0].date, "2026-03-01");
        assert_eq!(daily[0].platform, "claude");
        assert_eq!(daily[0].request_count, 1);
        assert_eq!(daily[1].platform, "codex");
        assert_eq!(daily[2].date, "2026-03-02");
        assert_eq!(daily[2].platform, "claude");
        assert_eq!(daily[2].input_tokens, 80);
        assert_eq!(daily[2].output_tokens, 20);
    }

    #[test]
    fn test_paginated_logs_respects_date_and_model_filters() {
        let conn = setup_test_db();
        let source_id = Uuid::new_v4().to_string();

        let records = vec![
            UsageRecord {
                id: "r1".to_string(),
                platform: "codex".to_string(),
                project_path: "/p1".to_string(),
                record_json: "{}".to_string(),
                recorded_at: DateTime::parse_from_rfc3339("2026-01-01T10:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                source_id: source_id.clone(),
                model: Some("gpt-3.5".to_string()),
                input_tokens: 10,
                output_tokens: 10,
                cache_read_tokens: 0,
                cost_usd: 0.1,
            },
            UsageRecord {
                id: "r2".to_string(),
                platform: "codex".to_string(),
                project_path: "/p2".to_string(),
                record_json: "{}".to_string(),
                recorded_at: DateTime::parse_from_rfc3339("2026-01-02T12:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                source_id: source_id.clone(),
                model: Some("gpt-4o".to_string()),
                input_tokens: 20,
                output_tokens: 20,
                cache_read_tokens: 0,
                cost_usd: 0.2,
            },
            UsageRecord {
                id: "r3".to_string(),
                platform: "claude".to_string(),
                project_path: "/p3".to_string(),
                record_json: "{}".to_string(),
                recorded_at: DateTime::parse_from_rfc3339("2026-01-02T15:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                source_id,
                model: Some("claude-3.7-sonnet".to_string()),
                input_tokens: 30,
                output_tokens: 30,
                cache_read_tokens: 0,
                cost_usd: 0.3,
            },
        ];

        insert_records_batch(&conn, &records).unwrap();

        let page = get_paginated_logs(
            &conn,
            &Some("codex".to_string()),
            &Some("2026-01-02".to_string()),
            &Some("2026-01-02".to_string()),
            1,
            20,
            &Some("gpt-4o".to_string()),
            true,
        )
        .unwrap();

        assert_eq!(page.records.len(), 1);
        assert_eq!(page.records[0].id, "r2");
        assert_eq!(page.total, Some(1));
    }

    #[test]
    fn test_cursor_and_offset_first_page_consistency() {
        let conn = setup_test_db();
        let source_id = Uuid::new_v4().to_string();

        let mut records = Vec::new();
        for i in 0..30 {
            records.push(UsageRecord {
                id: format!("rid-{i:02}"),
                platform: "codex".to_string(),
                project_path: "/bulk".to_string(),
                record_json: "{}".to_string(),
                recorded_at: Utc::now() + chrono::Duration::seconds(i),
                source_id: source_id.clone(),
                model: Some("gpt-4o".to_string()),
                input_tokens: i,
                output_tokens: i,
                cache_read_tokens: 0,
                cost_usd: i as f64 / 100.0,
            });
        }
        insert_records_batch(&conn, &records).unwrap();

        let offset_page = get_paginated_logs(
            &conn,
            &Some("codex".to_string()),
            &None,
            &None,
            1,
            10,
            &Some("gpt-4o".to_string()),
            true,
        )
        .unwrap();
        let cursor_page = get_logs_by_cursor(
            &conn,
            &Some("codex".to_string()),
            &None,
            &None,
            10,
            &Some("gpt-4o".to_string()),
            &None,
            true,
        )
        .unwrap();

        assert_eq!(offset_page.records.len(), cursor_page.records.len());
        let offset_ids: Vec<&str> = offset_page.records.iter().map(|r| r.id.as_str()).collect();
        let cursor_ids: Vec<&str> = cursor_page.records.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(offset_ids, cursor_ids);
    }
}
