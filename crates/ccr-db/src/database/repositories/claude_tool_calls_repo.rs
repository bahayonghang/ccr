// Claude Code 工具调用事实表的仓储
// 数据由 ccr-ui/src-tauri 的 claude_observer 模块从 ~/.claude/projects/**.jsonl
// 增量解析而来；本仓储只关心读写 SQLite，不接触文件系统。
//
// 用途：为 Claude Code 页 Usage Insight 面板的 Behavior tab 提供
//       周-小时热力图、Top tools、$/tool 效率排行的事实来源。
//
// 注意：duration_ms 和 cost_usd 在当前版本可能为 NULL（来源 jsonl 不一定带
//       足够字段去推断），前端必须能容忍。

use chrono::Utc;
use rusqlite::{Connection, params, params_from_iter};

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
    pub cost_usd: f64,
}

/// $/tool 效率排行行
#[derive(Debug, Clone, serde::Serialize)]
pub struct EfficiencyRow {
    pub session_id: String,
    pub project_path: Option<String>,
    pub call_count: i64,
    pub success_count: i64,
    pub failure_count: i64,
    pub cost_usd: f64,
    pub cost_per_tool: f64,
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
    let mut inserted = 0usize;
    {
        let mut stmt = tx.prepare(
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
    tx.commit()?;
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
    let (where_sql, params_vec): (&str, Vec<String>) = if days > 0 {
        (
            "WHERE ts >= datetime('now', ?1)",
            vec![format!("-{days} days")],
        )
    } else {
        ("", vec![])
    };
    let sql = format!(
        "SELECT CAST(strftime('%w', ts) AS INTEGER) AS dow,
                CAST(strftime('%H', ts) AS INTEGER) AS hour,
                COUNT(*) AS cnt
         FROM claude_tool_calls
         {where_sql}
         GROUP BY dow, hour
         ORDER BY dow ASC, hour ASC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_from_iter(params_vec.iter()), |row| {
        Ok(HeatmapCell {
            dow: row.get(0)?,
            hour: row.get(1)?,
            count: row.get(2)?,
        })
    })?;
    rows.collect()
}

/// Top tools 按调用次数排序，附带累计成本（如已知）
#[allow(dead_code)]
pub fn top_tools(
    conn: &Connection,
    days: i64,
    limit: i64,
) -> Result<Vec<TopToolRow>, rusqlite::Error> {
    let (where_sql, params_vec): (&str, Vec<String>) = if days > 0 {
        (
            "WHERE ts >= datetime('now', ?1)",
            vec![format!("-{days} days")],
        )
    } else {
        ("", vec![])
    };
    let sql = format!(
        "SELECT tool_name, COUNT(*) AS cnt, COALESCE(SUM(cost_usd), 0.0) AS cost
         FROM claude_tool_calls
         {where_sql}
         GROUP BY tool_name
         ORDER BY cnt DESC
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
    let (where_sql, mut params_vec): (&str, Vec<String>) = if days > 0 {
        (
            "WHERE ts >= datetime('now', ?1)",
            vec![format!("-{days} days")],
        )
    } else {
        ("", vec![])
    };
    params_vec.push(format!("{min_cost_usd}"));
    let sql = format!(
        "SELECT session_id,
                MAX(project_path) AS project_path,
                COUNT(*) AS calls,
                SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) AS oks,
                SUM(CASE WHEN success = 0 THEN 1 ELSE 0 END) AS fails,
                COALESCE(SUM(cost_usd), 0.0) AS cost,
                CASE WHEN COUNT(*) > 0 THEN COALESCE(SUM(cost_usd), 0.0) / COUNT(*) ELSE 0.0 END AS cpt
         FROM claude_tool_calls
         {where_sql}
         GROUP BY session_id
         HAVING cost >= CAST(? AS REAL)
         ORDER BY cpt DESC
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
}
