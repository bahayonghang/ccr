// Claude Code 工具调用事实表的仓储
// 数据由 ccr-ui/src-tauri 的 claude_observer 模块从 ~/.claude/projects/**.jsonl
// 增量解析而来；本仓储只关心读写 SQLite，不接触文件系统。
//
// 用途：为 Claude Code 页 Usage Insight 面板的 Behavior tab 提供
//       周-小时热力图、Top tools、$/tool 效率排行的事实来源。
//
// 注意：duration_ms 和 cost_usd 在当前版本可能为 NULL（来源 jsonl 不一定带
//       足够字段去推断），前端必须能容忍。

use chrono::{Datelike, TimeZone, Timelike, Utc};
use rusqlite::{Connection, params, params_from_iter};

/// All-history distinct sessions with recorded Claude tool calls.
pub fn session_count(conn: &Connection) -> Result<i64, rusqlite::Error> {
    conn.query_row(
        "SELECT COUNT(DISTINCT session_id) FROM claude_tool_calls",
        [],
        |row| row.get(0),
    )
}

/// 待落库的单条 tool 调用事件
#[derive(Debug, Clone)]
pub struct ToolCallRow {
    pub session_id: String,
    pub seq: i64,
    pub ts: String,
    pub tool_name: String,
    pub success: Option<bool>,
    pub duration_ms: Option<i64>,
    pub cost_usd: Option<f64>,
    pub project_path: Option<String>,
}

// All callers use an inclusive instant window, independent of timestamp spelling.
fn report_window(days: i64, as_of: Option<chrono::DateTime<Utc>>) -> (&'static str, Vec<String>) {
    let at = as_of.unwrap_or_else(Utc::now);
    if days > 0 {
        (
            "WHERE julianday(ts) BETWEEN julianday(?2, ?1) AND julianday(?2)",
            vec![format!("-{days} days"), at.to_rfc3339()],
        )
    } else {
        (
            "WHERE julianday(ts) <= julianday(?1)",
            vec![at.to_rfc3339()],
        )
    }
}

/// 周-小时热力图单元
#[derive(Debug, Clone, serde::Serialize)]
pub struct HeatmapCell {
    pub dow: i64,  // 0=Sun..6=Sat（SQLite strftime('%w') 语义）
    pub hour: i64, // 0..23
    pub count: i64,
}

/// Top tools 行
#[derive(Debug, Clone, serde::Serialize)]
pub struct TopToolRow {
    pub tool_name: String,
    pub call_count: i64,
    pub cost_usd: Option<f64>,
}

/// $/tool 效率排行行
#[derive(Debug, Clone, serde::Serialize)]
pub struct EfficiencyRow {
    pub session_id: String,
    pub project_path: Option<String>,
    pub call_count: i64,
    pub success_count: i64,
    pub failure_count: i64,
    pub cost_usd: Option<f64>,
    pub cost_per_tool: Option<f64>,
}

/// jsonl 文件解析进度（增量游标）
#[derive(Debug, Clone)]
pub struct IngestStateRow {
    pub file_path: String,
    pub file_mtime_ns: i64,
    pub last_offset: i64,
    pub updated_at: String,
}

/*
 * ========================================================================
 * 步骤A：写入 / 批量写入
 * ========================================================================
 */

/// 批量 upsert tool 调用；用于增量 ingest 一次提交一个文件的所有新行
#[allow(dead_code)]
pub fn upsert_batch(conn: &mut Connection, rows: &[ToolCallRow]) -> Result<usize, rusqlite::Error> {
    if rows.is_empty() {
        return Ok(0);
    }
    let tx = conn.transaction()?;
    let inserted = upsert_rows(&tx, rows)?;
    tx.commit()?;
    Ok(inserted)
}

/// Commit deterministic file rows and their scan cursor atomically.
pub fn record_ingested_file(
    conn: &mut Connection,
    rows: &[ToolCallRow],
    file_path: &str,
    mtime_ns: i64,
    offset: i64,
) -> Result<usize, rusqlite::Error> {
    record_ingested_file_with_skips(conn, rows, &[], file_path, mtime_ns, offset)
}

/// Remove only explicitly rejected event positions in the same file/cursor transaction.
pub fn record_ingested_file_with_skips(
    conn: &mut Connection,
    rows: &[ToolCallRow],
    skipped: &[(String, i64)],
    file_path: &str,
    mtime_ns: i64,
    offset: i64,
) -> Result<usize, rusqlite::Error> {
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    for (session, seq) in skipped {
        tx.execute(
            "DELETE FROM claude_tool_calls WHERE session_id = ?1 AND seq = ?2",
            params![session, seq],
        )?;
    }
    let inserted = upsert_rows(&tx, rows)?;
    upsert_ingest_state(&tx, file_path, mtime_ns, offset)?;
    tx.commit()?;
    Ok(inserted)
}

fn upsert_rows(conn: &Connection, rows: &[ToolCallRow]) -> Result<usize, rusqlite::Error> {
    if rows.is_empty() {
        return Ok(0);
    }
    let mut inserted = 0usize;
    {
        let mut stmt = conn.prepare(
            "INSERT INTO claude_tool_calls
                (session_id, seq, ts, tool_name, success, duration_ms, cost_usd, project_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(session_id, seq) DO UPDATE SET
                ts = excluded.ts,
                tool_name = excluded.tool_name,
                success = COALESCE(excluded.success, claude_tool_calls.success),
                duration_ms = COALESCE(excluded.duration_ms, claude_tool_calls.duration_ms),
                cost_usd = COALESCE(excluded.cost_usd, claude_tool_calls.cost_usd),
                project_path = COALESCE(excluded.project_path, claude_tool_calls.project_path)",
        )?;
        for row in rows {
            let success_int = row.success.map(|b| if b { 1_i64 } else { 0_i64 });
            stmt.execute(params![
                row.session_id,
                row.seq,
                row.ts,
                row.tool_name,
                success_int,
                row.duration_ms,
                row.cost_usd,
                row.project_path,
            ])?;
            inserted += 1;
        }
    }
    Ok(inserted)
}

/// upsert 文件解析进度
#[allow(dead_code)]
pub fn upsert_ingest_state(
    conn: &Connection,
    file_path: &str,
    file_mtime_ns: i64,
    last_offset: i64,
) -> Result<(), rusqlite::Error> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO claude_tool_calls_ingest_state
            (file_path, file_mtime_ns, last_offset, updated_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(file_path) DO UPDATE SET
            file_mtime_ns = excluded.file_mtime_ns,
            last_offset = excluded.last_offset,
            updated_at = excluded.updated_at",
        params![file_path, file_mtime_ns, last_offset, now],
    )?;
    Ok(())
}

#[allow(dead_code)]
pub fn get_ingest_state(
    conn: &Connection,
    file_path: &str,
) -> Result<Option<IngestStateRow>, rusqlite::Error> {
    use rusqlite::OptionalExtension;
    conn.query_row(
        "SELECT file_path, file_mtime_ns, last_offset, updated_at
         FROM claude_tool_calls_ingest_state WHERE file_path = ?1",
        params![file_path],
        |row| {
            Ok(IngestStateRow {
                file_path: row.get(0)?,
                file_mtime_ns: row.get(1)?,
                last_offset: row.get(2)?,
                updated_at: row.get(3)?,
            })
        },
    )
    .optional()
}

/*
 * ========================================================================
 * 步骤B：分析查询
 * ========================================================================
 */

/// 周×小时热力图。`days` 为窗口大小，0 表示不限。
#[allow(dead_code)]
pub fn heatmap(conn: &Connection, days: i64) -> Result<Vec<HeatmapCell>, rusqlite::Error> {
    heatmap_with_clock(conn, days, None)
}
pub fn heatmap_at(
    conn: &Connection,
    days: i64,
    as_of: chrono::DateTime<Utc>,
) -> Result<Vec<HeatmapCell>, rusqlite::Error> {
    heatmap_with_clock(conn, days, Some(as_of))
}
fn heatmap_with_clock(
    conn: &Connection,
    days: i64,
    as_of: Option<chrono::DateTime<Utc>>,
) -> Result<Vec<HeatmapCell>, rusqlite::Error> {
    heatmap_in_timezone_at(conn, days, as_of.unwrap_or_else(Utc::now), &Utc)
}

/// Convert each event in its historical report zone before weekday/hour grouping.
pub fn heatmap_in_timezone_at<Tz: TimeZone>(
    conn: &Connection,
    days: i64,
    as_of: chrono::DateTime<Utc>,
    timezone: &Tz,
) -> Result<Vec<HeatmapCell>, rusqlite::Error> {
    let (where_sql, params_vec) = report_window(days, Some(as_of));
    let sql = format!(
        "SELECT ts, COUNT(*)
         FROM claude_tool_calls
         {where_sql}
         GROUP BY ts"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(params_vec.iter()), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    let mut bins = std::collections::BTreeMap::<(i64, i64), i64>::new();
    for row in rows {
        let (timestamp, count) = row?;
        if let Ok(instant) = chrono::DateTime::parse_from_rfc3339(&timestamp) {
            let local = instant.with_timezone(timezone);
            *bins
                .entry((
                    i64::from(local.weekday().num_days_from_sunday()),
                    i64::from(local.hour()),
                ))
                .or_default() += count;
        }
    }
    Ok(bins
        .into_iter()
        .map(|((dow, hour), count)| HeatmapCell { dow, hour, count })
        .collect())
}

/// Top tools 按调用次数排序，附带累计成本（如已知）
#[allow(dead_code)]
pub fn top_tools(
    conn: &Connection,
    days: i64,
    limit: i64,
) -> Result<Vec<TopToolRow>, rusqlite::Error> {
    top_tools_with_clock(conn, days, limit, None)
}
pub fn top_tools_at(
    conn: &Connection,
    days: i64,
    limit: i64,
    as_of: chrono::DateTime<Utc>,
) -> Result<Vec<TopToolRow>, rusqlite::Error> {
    top_tools_with_clock(conn, days, limit, Some(as_of))
}
fn top_tools_with_clock(
    conn: &Connection,
    days: i64,
    limit: i64,
    as_of: Option<chrono::DateTime<Utc>>,
) -> Result<Vec<TopToolRow>, rusqlite::Error> {
    let (where_sql, params_vec) = report_window(days, as_of);
    let sql = format!(
        "SELECT tool_name, COUNT(*) AS cnt,
                CASE WHEN COUNT(cost_usd) = COUNT(*) THEN SUM(cost_usd) END AS cost
         FROM claude_tool_calls
         {where_sql}
         GROUP BY tool_name
         ORDER BY cnt DESC, tool_name ASC
         LIMIT {limit}"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(params_vec.iter()), |row| {
        Ok(TopToolRow {
            tool_name: row.get(0)?,
            call_count: row.get(1)?,
            cost_usd: row.get(2)?,
        })
    })?;
    rows.collect()
}

/// $/tool 效率排行：把累计成本除以工具调用数，挑出最贵的 N 个会话。
/// `min_cost_usd` 为入选门槛，过滤掉低成本噪音。
#[allow(dead_code)]
pub fn efficiency_sessions(
    conn: &Connection,
    days: i64,
    min_cost_usd: f64,
    limit: i64,
) -> Result<Vec<EfficiencyRow>, rusqlite::Error> {
    efficiency_sessions_with_clock(conn, days, min_cost_usd, limit, None)
}
pub fn efficiency_sessions_at(
    conn: &Connection,
    days: i64,
    min_cost_usd: f64,
    limit: i64,
    as_of: chrono::DateTime<Utc>,
) -> Result<Vec<EfficiencyRow>, rusqlite::Error> {
    efficiency_sessions_with_clock(conn, days, min_cost_usd, limit, Some(as_of))
}
fn efficiency_sessions_with_clock(
    conn: &Connection,
    days: i64,
    min_cost_usd: f64,
    limit: i64,
    as_of: Option<chrono::DateTime<Utc>>,
) -> Result<Vec<EfficiencyRow>, rusqlite::Error> {
    session_rows(
        conn,
        days,
        Some(min_cost_usd),
        limit,
        as_of,
        "cpt DESC, session_id ASC",
    )
}

/// Observed tool sessions, including unpriced sessions. The limit follows the chosen order.
pub fn observed_sessions_at(
    conn: &Connection,
    days: i64,
    limit: i64,
    by_cost: bool,
    as_of: chrono::DateTime<Utc>,
) -> Result<Vec<EfficiencyRow>, rusqlite::Error> {
    session_rows(
        conn,
        days,
        None,
        limit,
        Some(as_of),
        if by_cost {
            "cost DESC, calls DESC, session_id ASC"
        } else {
            "calls DESC, session_id ASC"
        },
    )
}

fn session_rows(
    conn: &Connection,
    days: i64,
    min_cost_usd: Option<f64>,
    limit: i64,
    as_of: Option<chrono::DateTime<Utc>>,
    order: &str,
) -> Result<Vec<EfficiencyRow>, rusqlite::Error> {
    let (where_sql, mut params_vec) = report_window(days, as_of);
    let having = if let Some(min_cost) = min_cost_usd {
        params_vec.push(min_cost.to_string());
        "HAVING cost >= CAST(? AS REAL)"
    } else {
        ""
    };
    let sql = format!(
        "SELECT session_id,
                MAX(project_path) AS project_path,
                COUNT(*) AS calls,
                SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) AS oks,
                SUM(CASE WHEN success = 0 THEN 1 ELSE 0 END) AS fails,
                CASE WHEN COUNT(cost_usd) = COUNT(*) THEN SUM(cost_usd) END AS cost,
                CASE WHEN COUNT(cost_usd) = COUNT(*) THEN SUM(cost_usd) / COUNT(*) END AS cpt
         FROM claude_tool_calls
         {where_sql}
         GROUP BY session_id
         {having}
         ORDER BY {order}
         LIMIT {limit}"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(params_vec.iter()), |row| {
        Ok(EfficiencyRow {
            session_id: row.get(0)?,
            project_path: row.get(1)?,
            call_count: row.get(2)?,
            success_count: row.get(3)?,
            failure_count: row.get(4)?,
            cost_usd: row.get(5)?,
            cost_per_tool: row.get(6)?,
        })
    })?;
    rows.collect()
}

/// 简单 count（用于空态判定）
#[allow(dead_code)]
pub fn total_count(conn: &Connection) -> Result<i64, rusqlite::Error> {
    conn.query_row("SELECT COUNT(*) FROM claude_tool_calls", [], |row| {
        row.get(0)
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::database::migrations::{run_initial_migration, run_migration_v14};

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_initial_migration(&conn).unwrap();
        run_migration_v14(&conn).unwrap();
        conn
    }

    fn sample_row(session: &str, seq: i64, tool: &str, success: bool, cost: f64) -> ToolCallRow {
        ToolCallRow {
            session_id: session.to_string(),
            seq,
            ts: chrono::Utc::now().to_rfc3339(),
            tool_name: tool.to_string(),
            success: Some(success),
            duration_ms: Some(120),
            cost_usd: Some(cost),
            project_path: Some("/repo".to_string()),
        }
    }

    #[test]
    fn heatmap_uses_the_event_report_zone_across_day_boundaries() {
        let mut conn = fresh_conn();
        let mut row = sample_row("s", 1, "Read", true, 0.0);
        row.ts = "2026-11-08T23:30:00Z".into();
        upsert_batch(&mut conn, &[row]).unwrap();
        let as_of = chrono::DateTime::parse_from_rfc3339("2026-11-09T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let zone = chrono::FixedOffset::east_opt(9 * 3600).unwrap();
        let local = heatmap_in_timezone_at(&conn, 1, as_of, &zone).unwrap();
        assert_eq!((local[0].dow, local[0].hour, local[0].count), (1, 8, 1));
        let utc = heatmap_at(&conn, 1, as_of).unwrap();
        assert_eq!((utc[0].dow, utc[0].hour), (0, 23));
    }

    #[test]
    fn missing_cost_stays_unknown_including_partially_priced_groups() {
        let mut conn = fresh_conn();
        let mut missing = sample_row("unknown", 1, "Read", true, 0.0);
        missing.cost_usd = None;
        let mut partial = missing.clone();
        partial.session_id = "partial".into();
        partial.tool_name = "Edit".into();
        upsert_batch(
            &mut conn,
            &[
                missing,
                partial,
                sample_row("partial", 2, "Edit", true, 0.5),
                sample_row("free", 1, "Bash", true, 0.0),
            ],
        )
        .unwrap();
        let tools = top_tools(&conn, 0, 10).unwrap();
        for name in ["Read", "Edit"] {
            let row = tools.iter().find(|row| row.tool_name == name).unwrap();
            assert!(serde_json::to_value(row).unwrap()["cost_usd"].is_null());
        }
        let free = tools.iter().find(|row| row.tool_name == "Bash").unwrap();
        assert_eq!(serde_json::to_value(free).unwrap()["cost_usd"], 0.0);
        let sessions = observed_sessions_at(&conn, 0, 1, false, Utc::now()).unwrap();
        assert_eq!(sessions[0].session_id, "partial");
        assert_eq!(sessions[0].cost_usd, None);
        assert_eq!(sessions[0].cost_per_tool, None);
        let priced = observed_sessions_at(&conn, 0, 1, true, Utc::now()).unwrap();
        assert_eq!(priced[0].session_id, "free");
        assert_eq!(priced[0].cost_usd, Some(0.0));
    }

    #[test]
    fn live_window_excludes_early_cutoff_day_and_future_rows() {
        let mut conn = fresh_conn();
        let now = Utc::now();
        let cutoff = now - chrono::Duration::days(1);
        let rows = [
            cutoff - chrono::Duration::seconds(1),
            now,
            now + chrono::Duration::days(1),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, ts)| {
            let mut row = sample_row(&format!("s{index}"), 1, "Read", true, 0.5);
            row.ts = ts.to_rfc3339();
            row
        })
        .collect::<Vec<_>>();
        upsert_batch(&mut conn, &rows).unwrap();
        assert_eq!(top_tools(&conn, 1, 10).unwrap()[0].call_count, 1);
    }

    #[test]
    fn upsert_batch_then_top_tools() {
        let mut conn = fresh_conn();
        let rows = vec![
            sample_row("s1", 1, "Bash", true, 0.10),
            sample_row("s1", 2, "Read", true, 0.02),
            sample_row("s1", 3, "Bash", false, 0.05),
            sample_row("s2", 1, "Edit", true, 0.20),
        ];
        let n = upsert_batch(&mut conn, &rows).unwrap();
        assert_eq!(n, 4);
        assert_eq!(total_count(&conn).unwrap(), 4);

        let tops = top_tools(&conn, 0, 10).unwrap();
        assert_eq!(tops[0].tool_name, "Bash");
        assert_eq!(tops[0].call_count, 2);
    }

    #[test]
    fn upsert_is_idempotent_on_same_key() {
        let mut conn = fresh_conn();
        let rows = vec![sample_row("s1", 1, "Bash", true, 0.10)];
        upsert_batch(&mut conn, &rows).unwrap();
        upsert_batch(&mut conn, &rows).unwrap();
        assert_eq!(total_count(&conn).unwrap(), 1);
    }

    #[test]
    fn ingest_state_upsert_roundtrip() {
        let conn = fresh_conn();
        upsert_ingest_state(&conn, "/tmp/a.jsonl", 1234, 999).unwrap();
        let st = get_ingest_state(&conn, "/tmp/a.jsonl").unwrap().unwrap();
        assert_eq!(st.file_mtime_ns, 1234);
        assert_eq!(st.last_offset, 999);

        upsert_ingest_state(&conn, "/tmp/a.jsonl", 5678, 2048).unwrap();
        let st2 = get_ingest_state(&conn, "/tmp/a.jsonl").unwrap().unwrap();
        assert_eq!(st2.file_mtime_ns, 5678);
        assert_eq!(st2.last_offset, 2048);
    }

    #[test]
    fn efficiency_sorts_by_cost_per_tool() {
        let mut conn = fresh_conn();
        let rows = vec![
            sample_row("cheap", 1, "Bash", true, 0.01),
            sample_row("cheap", 2, "Read", true, 0.01),
            sample_row("expensive", 1, "Agent", true, 1.50),
            sample_row("expensive", 2, "Agent", true, 1.50),
        ];
        upsert_batch(&mut conn, &rows).unwrap();
        let top = efficiency_sessions(&conn, 0, 0.0, 5).unwrap();
        assert_eq!(top[0].session_id, "expensive");
        assert!(top[0].cost_per_tool > top[1].cost_per_tool);
    }
    #[test]
    fn fixed_report_tools_window_includes_edges_and_excludes_future() {
        let mut conn = fresh_conn();
        let as_of = chrono::DateTime::parse_from_rfc3339("2000-01-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let rows = [
            "1999-12-31T23:59:59Z",
            "2000-01-01T00:00:00Z",
            "2000-01-02T00:00:00Z",
            "2000-01-02T00:00:00.001Z",
            "2000-01-02T00:00:01Z",
        ]
        .into_iter()
        .enumerate()
        .map(|(i, ts)| {
            let mut row = sample_row(&format!("s{i}"), 0, "Read", true, 0.5);
            row.ts = ts.into();
            row
        })
        .collect::<Vec<_>>();
        upsert_batch(&mut conn, &rows).unwrap();
        assert_eq!(
            heatmap_at(&conn, 1, as_of)
                .unwrap()
                .iter()
                .map(|cell| cell.count)
                .sum::<i64>(),
            2
        );
        assert_eq!(top_tools_at(&conn, 1, 20, as_of).unwrap()[0].call_count, 2);
        assert_eq!(top_tools_at(&conn, 0, 20, as_of).unwrap()[0].call_count, 3);
        assert_eq!(
            efficiency_sessions_at(&conn, 1, 0.0, 20, as_of)
                .unwrap()
                .len(),
            2
        );
        assert_eq!(top_tools(&conn, 0, 20).unwrap()[0].call_count, 5);
    }
}
