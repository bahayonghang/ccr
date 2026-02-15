// Usage tracking repository
// Handles CRUD operations for usage sources and records

use chrono::{DateTime, Utc};
use rusqlite::{Connection, Row, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
pub fn delete_records_by_source(
    conn: &Connection,
    source_id: &str,
) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "DELETE FROM usage_records WHERE source_id = ?1",
        params![source_id],
    )
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

/// 获取使用量汇总
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
pub fn get_daily_trends(
    conn: &Connection,
    platform: &Option<String>,
    start: &Option<String>,
    end: &Option<String>,
) -> Result<Vec<DailyTrend>, rusqlite::Error> {
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

/// 获取模型统计
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
pub fn get_heatmap_data(
    conn: &Connection,
    platform: &Option<String>,
    days: i64,
) -> Result<HashMap<String, i64>, rusqlite::Error> {
    let cutoff = Utc::now() - chrono::Duration::days(days);
    let cutoff_str = cutoff.format("%Y-%m-%d").to_string();

    let mut conditions = vec![format!("date >= ?1")];
    let mut bind_params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(cutoff_str)];

    if let Some(p) = platform {
        bind_params.push(Box::new(p.clone()));
        conditions.push(format!("platform = ?{}", bind_params.len()));
    }

    let sql = format!(
        "SELECT date, SUM(request_count) FROM usage_daily_agg WHERE {} GROUP BY date",
        conditions.join(" AND ")
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
pub fn get_paginated_logs(
    conn: &Connection,
    platform: &Option<String>,
    page: i64,
    page_size: i64,
    model_filter: &Option<String>,
    include_total: bool,
) -> Result<PaginatedLogs, rusqlite::Error> {
    // 构建 WHERE
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

    let where_sql = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };

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
        "SELECT {} FROM usage_records{} ORDER BY recorded_at DESC LIMIT ?{} OFFSET ?{}",
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
pub fn get_logs_by_cursor(
    conn: &Connection,
    platform: &Option<String>,
    page_size: i64,
    model_filter: &Option<String>,
    cursor: &Option<String>,
    include_total: bool,
) -> Result<PaginatedLogs, rusqlite::Error> {
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
        let mut count_conditions: Vec<String> = Vec::new();
        let mut count_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        if let Some(p) = platform {
            count_params.push(Box::new(p.clone()));
            count_conditions.push(format!("platform = ?{}", count_params.len()));
        }
        if let Some(m) = model_filter {
            count_params.push(Box::new(m.clone()));
            count_conditions.push(format!("model = ?{}", count_params.len()));
        }
        let count_where = if count_conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", count_conditions.join(" AND "))
        };
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
        // 添加 v3 列（CREATE_TABLES_SQL 是 v1 schema，不含新列）
        for stmt in &[
            "ALTER TABLE usage_records ADD COLUMN model TEXT",
            "ALTER TABLE usage_records ADD COLUMN input_tokens INTEGER DEFAULT 0",
            "ALTER TABLE usage_records ADD COLUMN output_tokens INTEGER DEFAULT 0",
            "ALTER TABLE usage_records ADD COLUMN cache_read_tokens INTEGER DEFAULT 0",
            "ALTER TABLE usage_records ADD COLUMN cost_usd REAL DEFAULT 0",
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
            model: None,
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cost_usd: 0.0,
        };
        insert_record(&conn, &record).unwrap();
    }
}
