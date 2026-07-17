//! Claude Observer State-free 服务层。
//!
//! 从 `commands::claude_observer` 平移的 wire DTO 与同步查询编排：命令层只保留
//! State 提取 / spawn_blocking / join 错误映射，业务编排在此以具名 DTO 出入，
//! 便于无 Tauri app 的单元测试。
//!
//! 数据源边界（与命令层模块文档一致）：
//! - insight / daily_trend / cost_breakdown / cache_stats 来自 llmusage，
//!   platform 固定 `claude`，cost 用 `cost_with_cache_usd`。
//! - tool_heatmap / top_tools / top_sessions 来自 ccr-db 表 `claude_tool_calls`。
//! - 仓储层类型（`claude_tool_calls_repo::{HeatmapCell,TopToolRow}`）不直接上
//!   wire：本层持有同形 wire DTO 并做映射，ccr-db 不携带前端绑定 concern。

use ccr_db::database::DbPool;
use ccr_db::database::repositories::claude_tool_calls_repo;
use chrono::{Datelike, Local};
use serde::Serialize;
use tracing::debug;
use ts_rs::TS;

use crate::claude_observer::{pricing, subscription};
use crate::llmusage_adapter::{LlmusageRuntime, build_filter, queries};

// ── wire DTO 集合 ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_observer/")]
pub struct InsightDto {
    pub today_value_usd: f64,
    pub month_value_usd: f64,
    pub total_value_usd: f64,
    #[ts(as = "f64")]
    pub today_tokens: i64,
    #[ts(as = "f64")]
    pub month_tokens: i64,
    #[ts(as = "f64")]
    pub total_sessions: i64,
    #[ts(as = "f64")]
    pub total_projects: i64,
    pub subscription: subscription::SubscriptionDto,
    /// 当 mode=subscription 时，month_value_usd / monthly_usd；否则 None
    pub roi: Option<f64>,
    /// 嵌入式价目表版本（来自 resources/claude_pricing.json 的 version 字段）
    pub pricing_version: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_observer/")]
pub struct CacheStatsDto {
    pub hit_rate: f64,
    #[ts(as = "f64")]
    pub total_input_tokens: i64,
    #[ts(as = "f64")]
    pub total_output_tokens: i64,
    #[ts(as = "f64")]
    pub total_cache_read_tokens: i64,
    #[ts(as = "f64")]
    pub total_cache_write_tokens: i64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_observer/")]
pub struct DailyPoint {
    pub date: String,
    pub cost_usd: f64,
    #[ts(as = "f64")]
    pub input_tokens: i64,
    #[ts(as = "f64")]
    pub output_tokens: i64,
    #[ts(as = "f64")]
    pub cache_read_tokens: i64,
    #[ts(as = "f64")]
    pub cache_write_tokens: i64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_observer/")]
pub struct BreakdownRow {
    pub key: String,
    pub cost_usd: f64,
    #[ts(as = "f64")]
    pub tokens: i64,
    #[ts(as = "f64")]
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_observer/")]
pub struct SessionRow {
    pub session_id: String,
    pub project_path: Option<String>,
    pub model: Option<String>,
    pub cost_usd: f64,
    #[ts(as = "f64")]
    pub tokens: i64,
    #[ts(as = "f64")]
    pub tool_call_count: i64,
    pub started_at: Option<String>,
}

/// 周×小时热力图单元（wire 版，映射自 `claude_tool_calls_repo::HeatmapCell`）
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_observer/")]
pub struct HeatmapCell {
    /// 0=Sun..6=Sat
    #[ts(as = "f64")]
    pub dow: i64,
    /// 0..23
    #[ts(as = "f64")]
    pub hour: i64,
    #[ts(as = "f64")]
    pub count: i64,
}

impl From<claude_tool_calls_repo::HeatmapCell> for HeatmapCell {
    fn from(cell: claude_tool_calls_repo::HeatmapCell) -> Self {
        Self {
            dow: cell.dow,
            hour: cell.hour,
            count: cell.count,
        }
    }
}

/// Top tools 行（wire 版，映射自 `claude_tool_calls_repo::TopToolRow`）
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/claude_observer/")]
pub struct TopToolRow {
    pub tool_name: String,
    #[ts(as = "f64")]
    pub call_count: i64,
    pub cost_usd: f64,
}

impl From<claude_tool_calls_repo::TopToolRow> for TopToolRow {
    fn from(row: claude_tool_calls_repo::TopToolRow) -> Self {
        Self {
            tool_name: row.tool_name,
            call_count: row.call_count,
            cost_usd: row.cost_usd,
        }
    }
}

// ── 工具：日期窗口与 overview 拉取 ────────────────────────────────────────

fn overview_in_window(
    llmusage: &LlmusageRuntime,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<queries::OverviewPayload, String> {
    let filter = build_filter(Some("claude".to_string()), None, start_date, end_date)?;
    let dashboard = llmusage
        .dashboard()
        .map_err(|e| format!("Dashboard open error: {e}"))?;
    dashboard
        .overview(&filter)
        .map_err(|e| format!("Overview query error: {e}"))
}

/// 计算「今日」/「本月」的日期窗口（local time）。返回 (start_yyyy_mm_dd, end_yyyy_mm_dd)。
fn today_window() -> (String, String) {
    let today = Local::now().date_naive();
    let s = today.format("%Y-%m-%d").to_string();
    (s.clone(), s)
}
fn month_window() -> (String, String) {
    let today = Local::now().date_naive();
    let first = today
        .with_day(1)
        .unwrap_or(today)
        .format("%Y-%m-%d")
        .to_string();
    let last = today.format("%Y-%m-%d").to_string();
    (first, last)
}

// ── 7 个查询服务函数（订阅 get/set 直接走 subscription 模块，不再包一层）──

/// 首屏 Hero 三卡 + 订阅 banner 聚合（llmusage + ccr-db 双源）。
pub fn insight(llmusage: &LlmusageRuntime, db_pool: &DbPool) -> Result<InsightDto, String> {
    /* ====================================================================
     * 步骤1：从 llmusage 拉 total / today / month 三个 overview
     * ====================================================================
     */
    debug!("[claude_observer] get_insight: fetching total overview");
    let total = overview_in_window(llmusage, None, None)?;

    let (today_s, today_e) = today_window();
    debug!(
        "[claude_observer] get_insight: fetching today overview ({} to {})",
        today_s, today_e
    );
    let today = overview_in_window(llmusage, Some(today_s), Some(today_e))?;

    let (month_s, month_e) = month_window();
    debug!(
        "[claude_observer] get_insight: fetching month overview ({} to {})",
        month_s, month_e
    );
    let month = overview_in_window(llmusage, Some(month_s), Some(month_e))?;

    /* 步骤2：从 llmusage 拉项目数（distinct 项目维度）*/
    let filter = build_filter(Some("claude".to_string()), None, None, None)?;
    let dashboard = llmusage
        .dashboard()
        .map_err(|e| format!("Dashboard open error: {e}"))?;
    let projects = dashboard
        .project_breakdown(&filter)
        .map_err(|e| format!("Project breakdown error: {e}"))?;
    let total_projects = projects.len() as i64;

    /* 步骤3：从 ccr-db 取 session 总数（用 claude_tool_calls 的 distinct session_id） */
    let conn = db_pool.get().map_err(|e| format!("DB pool error: {e}"))?;
    let total_sessions: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT session_id) FROM claude_tool_calls",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let sub = subscription::get(&conn);

    /* 步骤4：组装 InsightDto */
    let total_tokens_today = today.total.total_tokens;
    let total_tokens_month = month.total.total_tokens;
    let today_value = today.total_cost_usd;
    let month_value = month.total_cost_usd;
    let total_value = total.total_cost_usd;
    let roi = if sub.mode == "subscription" && sub.monthly_usd > 0.0 {
        Some(month_value / sub.monthly_usd)
    } else {
        None
    };

    debug!(
        "[claude_observer] get_insight result: today=${:.2}, month=${:.2}, total=${:.2}, projects={}, sessions={}",
        today_value, month_value, total_value, total_projects, total_sessions
    );

    Ok(InsightDto {
        today_value_usd: today_value,
        month_value_usd: month_value,
        total_value_usd: total_value,
        today_tokens: total_tokens_today,
        month_tokens: total_tokens_month,
        total_sessions,
        total_projects,
        subscription: sub,
        roi,
        pricing_version: pricing::pricing_version().to_string(),
    })
}

/// 最近 `days` 天的每日趋势（默认 30，clamp 1..=365，按 claude 平台过滤）。
pub fn daily_trend(
    llmusage: &LlmusageRuntime,
    days: Option<i64>,
) -> Result<Vec<DailyPoint>, String> {
    let days = days.unwrap_or(30).clamp(1, 365);
    debug!("[claude_observer] daily_trend: days={}", days);

    let today = Local::now().date_naive();
    let start = today - chrono::Duration::days(days - 1);
    debug!(
        "[claude_observer] daily_trend: date range {} to {}",
        start, today
    );

    let filter = build_filter(
        Some("claude".to_string()),
        None,
        Some(start.format("%Y-%m-%d").to_string()),
        Some(today.format("%Y-%m-%d").to_string()),
    )?;

    debug!(
        "[claude_observer] daily_trend: DB path {:?}",
        llmusage.paths().db_path
    );
    let dashboard = llmusage.dashboard().map_err(|e| {
        let err = format!("Dashboard open error: {e}");
        tracing::error!("[claude_observer] {}", err);
        err
    })?;

    let trends = dashboard.trends_daily(&filter).map_err(|e| {
        let err = format!("Trends query error: {e}");
        tracing::error!("[claude_observer] {}", err);
        err
    })?;

    debug!(
        "[claude_observer] daily_trend result: {} points",
        trends.len()
    );
    if trends.is_empty() {
        tracing::warn!("[claude_observer] daily_trend returned empty array");
    }

    Ok(trends
        .into_iter()
        .map(|t| DailyPoint {
            date: t.date,
            cost_usd: t.cost_usd,
            input_tokens: t.input_tokens,
            output_tokens: t.output_tokens,
            cache_read_tokens: t.cache_read_tokens,
            cache_write_tokens: t.cache_creation_tokens,
        })
        .collect())
}

/// 按 `dim ∈ {project, model}` 维度的 Top N 拆分（仅 claude 平台）。
pub fn cost_breakdown(
    llmusage: &LlmusageRuntime,
    dim: &str,
    days: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<BreakdownRow>, String> {
    let limit = limit.unwrap_or(10).clamp(1, 200) as usize;
    let days = days.unwrap_or(30).clamp(1, 365);
    debug!(
        "[claude_observer] cost_breakdown: dim={}, days={}, limit={}",
        dim, days, limit
    );

    let today = Local::now().date_naive();
    let start = today - chrono::Duration::days(days - 1);
    let filter = build_filter(
        Some("claude".to_string()),
        None,
        Some(start.format("%Y-%m-%d").to_string()),
        Some(today.format("%Y-%m-%d").to_string()),
    )?;
    let dashboard = llmusage.dashboard().map_err(|e| {
        let err = format!("Dashboard open error: {e}");
        tracing::error!("[claude_observer] {}", err);
        err
    })?;

    let rows: Vec<BreakdownRow> = match dim {
        "project" => {
            let mut list = dashboard.project_breakdown(&filter).map_err(|e| {
                let err = format!("Project breakdown error: {e}");
                tracing::error!("[claude_observer] {}", err);
                err
            })?;
            list.sort_by(|a, b| b.total_cost_usd.total_cmp(&a.total_cost_usd));
            list.into_iter()
                .take(limit)
                .map(|p| BreakdownRow {
                    key: p
                        .project_path
                        .clone()
                        .or(p.project_ref.clone())
                        .unwrap_or_else(|| {
                            if !p.project_label.trim().is_empty() {
                                p.project_label.clone()
                            } else {
                                p.project_hash.clone()
                            }
                        }),
                    cost_usd: p.total_cost_usd,
                    tokens: p.total_tokens,
                    count: p.event_count,
                })
                .collect()
        }
        "model" => {
            let mut list = dashboard.model_breakdown(&filter).map_err(|e| {
                let err = format!("Model breakdown error: {e}");
                tracing::error!("[claude_observer] {}", err);
                err
            })?;
            list.sort_by(|a, b| b.cost_with_cache_usd.total_cmp(&a.cost_with_cache_usd));
            list.into_iter()
                .take(limit)
                .map(|m| BreakdownRow {
                    key: m.model,
                    cost_usd: m.cost_with_cache_usd,
                    tokens: m.total_tokens,
                    count: m.event_count,
                })
                .collect()
        }
        other => return Err(format!("Unsupported breakdown dim: {other}")),
    };

    debug!(
        "[claude_observer] cost_breakdown result: {} rows",
        rows.len()
    );
    if rows.is_empty() {
        tracing::warn!(
            "[claude_observer] cost_breakdown returned empty array for dim={}",
            dim
        );
    }

    Ok(rows)
}

/// 缓存效率：命中率 + 4 个 token 总量。
pub fn cache_stats(llmusage: &LlmusageRuntime) -> Result<CacheStatsDto, String> {
    debug!("[claude_observer] cache_stats: fetching");

    let filter = build_filter(Some("claude".to_string()), None, None, None)?;
    let dashboard = llmusage.dashboard().map_err(|e| {
        let err = format!("Dashboard open error: {e}");
        tracing::error!("[claude_observer] {}", err);
        err
    })?;
    let overview = dashboard.overview(&filter).map_err(|e| {
        let err = format!("Overview query error: {e}");
        tracing::error!("[claude_observer] {}", err);
        err
    })?;

    // cache_write_tokens 来自 trends_daily 的 cache_creation_tokens 维度汇总
    let trends = dashboard.trends_daily(&filter).map_err(|e| {
        let err = format!("Trends query error: {e}");
        tracing::error!("[claude_observer] {}", err);
        err
    })?;
    let total_cache_write_tokens: i64 = trends.iter().map(|t| t.cache_creation_tokens).sum();

    debug!(
        "[claude_observer] cache_stats result: hit_rate={:.2}%, write_tokens={}",
        overview.cache_efficiency * 100.0,
        total_cache_write_tokens
    );

    Ok(CacheStatsDto {
        hit_rate: overview.cache_efficiency,
        total_input_tokens: overview.total.input_tokens,
        total_output_tokens: overview.total.output_tokens_with_reasoning(),
        total_cache_read_tokens: overview.total.cache_read_tokens,
        total_cache_write_tokens,
    })
}

/// 按 cost 或工具调用密度排序的 Top sessions（`by ∈ cost | calls`，默认 cost，30 天窗口）。
pub fn top_sessions(
    db_pool: &DbPool,
    limit: Option<i64>,
    by: Option<&str>,
) -> Result<Vec<SessionRow>, String> {
    let limit = limit.unwrap_or(10).clamp(1, 200);
    let by_calls = matches!(by, Some("calls"));
    debug!(
        "[claude_observer] top_sessions: limit={}, by={:?}",
        limit, by
    );

    let conn = db_pool.get().map_err(|e| {
        let err = format!("DB pool error: {e}");
        tracing::error!("[claude_observer] {}", err);
        err
    })?;
    // 用 efficiency_sessions 拉全量，再按需重排
    let mut rows =
        claude_tool_calls_repo::efficiency_sessions(&conn, 30, 0.0, 1000).map_err(|e| {
            let err = format!("Efficiency query error: {e}");
            tracing::error!("[claude_observer] {}", err);
            err
        })?;
    if by_calls {
        // 用 sort_by_key 替换 sort_by 的 (a,b) 反向比较，clippy 偏好
        rows.sort_by_key(|r| std::cmp::Reverse(r.call_count));
    } else {
        // f64 不实现 Ord，必须沿用 sort_by + total_cmp
        rows.sort_by(|a, b| b.cost_usd.total_cmp(&a.cost_usd));
    }
    let out: Vec<SessionRow> = rows
        .into_iter()
        .take(limit as usize)
        .map(|r| SessionRow {
            session_id: r.session_id,
            project_path: r.project_path,
            model: None,
            cost_usd: r.cost_usd,
            tokens: 0,
            tool_call_count: r.call_count,
            started_at: None,
        })
        .collect();

    debug!(
        "[claude_observer] top_sessions result: {} sessions",
        out.len()
    );
    if out.is_empty() {
        tracing::warn!("[claude_observer] top_sessions returned empty (ccr-db may be empty)");
    }

    Ok(out)
}

/// 周×小时工具调用热力图（默认 30 天，clamp 1..=365）。
pub fn tool_heatmap(db_pool: &DbPool, days: Option<i64>) -> Result<Vec<HeatmapCell>, String> {
    let days = days.unwrap_or(30).clamp(1, 365);
    debug!("[claude_observer] tool_heatmap: days={}", days);

    let conn = db_pool.get().map_err(|e| {
        let err = format!("DB pool error: {e}");
        tracing::error!("[claude_observer] {}", err);
        err
    })?;
    let result = claude_tool_calls_repo::heatmap(&conn, days).map_err(|e| {
        let err = format!("Heatmap query error: {e}");
        tracing::error!("[claude_observer] {}", err);
        err
    })?;

    debug!(
        "[claude_observer] tool_heatmap result: {} cells",
        result.len()
    );
    if result.is_empty() {
        tracing::warn!("[claude_observer] tool_heatmap returned empty (ccr-db may be empty)");
    }

    Ok(result.into_iter().map(HeatmapCell::from).collect())
}

/// Top tools 排行（默认 30 天 / 15 条）。
pub fn top_tools(
    db_pool: &DbPool,
    days: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<TopToolRow>, String> {
    let days = days.unwrap_or(30).clamp(1, 365);
    let limit = limit.unwrap_or(15).clamp(1, 200);
    debug!(
        "[claude_observer] top_tools: days={}, limit={}",
        days, limit
    );

    let conn = db_pool.get().map_err(|e| {
        let err = format!("DB pool error: {e}");
        tracing::error!("[claude_observer] {}", err);
        err
    })?;
    let result = claude_tool_calls_repo::top_tools(&conn, days, limit).map_err(|e| {
        let err = format!("Top tools query error: {e}");
        tracing::error!("[claude_observer] {}", err);
        err
    })?;

    debug!("[claude_observer] top_tools result: {} tools", result.len());
    if result.is_empty() {
        tracing::warn!("[claude_observer] top_tools returned empty (ccr-db may be empty)");
    }

    Ok(result.into_iter().map(TopToolRow::from).collect())
}

/// 查询编排 service 的封闭单测：llmusage 侧走 `ccr_usage::fixtures` 投影库，
/// tool 调用侧走 temp ccr-db pool，全程不读写真实用户目录。
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod service_tests {
    use super::*;
    use ccr_usage::fixtures::{SeedBucket, create_projection_db, seed_bucket};
    use chrono::Utc;
    use rusqlite::{Connection, params};
    use tempfile::TempDir;

    /// 打开 fixture 投影库，返回指向它的 runtime 与可继续 seed 的连接。
    fn open_fixture(temp: &TempDir) -> (LlmusageRuntime, Connection) {
        let paths = create_projection_db(temp.path());
        let conn = Connection::open(&paths.db_path).expect("fixture db should reopen");
        (LlmusageRuntime::from_paths(paths), conn)
    }

    /// 临时 ccr-db pool（home_dir 传 temp，防止迁移逻辑碰真实家目录）。
    fn temp_pool(temp: &TempDir) -> DbPool {
        let pool = ccr_db::database::create_pool(&temp.path().join("observer.db"), None)
            .expect("pool should be created");
        let conn = pool.get().expect("pool should hand out a connection");
        ccr_db::database::migrations::run_all_migrations(&conn, temp.path())
            .expect("migrations should run");
        drop(conn);
        pool
    }

    /// 给定本地日期的正午 UTC 时刻：date(hour_start,'localtime') 在 ±11h 时区内
    /// 稳定落在该日期，规避测试机时区导致的日期漂移。
    fn local_noon_utc(date: chrono::NaiveDate) -> String {
        date.and_hms_opt(12, 0, 0)
            .expect("noon should be a valid time")
            .and_local_timezone(Local)
            .earliest()
            .expect("local noon should exist")
            .with_timezone(&Utc)
            .to_rfc3339()
    }

    /// 今日（local）正午的 claude bucket：40/10/5/30(+15) tokens，cost 0.10。
    fn claude_bucket_today() -> SeedBucket {
        SeedBucket {
            source: "claude".to_string(),
            provider_label: "anthropic".to_string(),
            model: "claude-opus".to_string(),
            hour_start: local_noon_utc(Local::now().date_naive()),
            project_path: Some("/repo/a".to_string()),
            ..SeedBucket::default()
        }
    }

    /// 插入一条 claude_tool_calls 种子行（ts=now，稳定落在任何天数窗口内）。
    fn seed_tool_call(pool: &DbPool, session: &str, seq: i64, tool: &str, cost: f64) {
        let conn = pool.get().expect("pool should hand out a connection");
        conn.execute(
            "INSERT INTO claude_tool_calls
                (session_id, seq, ts, tool_name, success, duration_ms, cost_usd, project_path)
             VALUES (?1, ?2, ?3, ?4, 1, NULL, ?5, '/repo/a')",
            params![session, seq, Utc::now().to_rfc3339(), tool, cost],
        )
        .expect("tool call row should insert");
    }

    #[test]
    fn daily_trend_filters_platform_and_maps_fields() {
        let temp = TempDir::new().unwrap();
        let (runtime, conn) = open_fixture(&temp);
        seed_bucket(&conn, &claude_bucket_today());
        // codex 种子应被 platform=claude 过滤掉
        seed_bucket(&conn, &SeedBucket::default());

        let points = daily_trend(&runtime, None).expect("daily_trend should succeed");
        assert_eq!(points.len(), 1);
        let p = &points[0];
        assert_eq!(
            p.date,
            Local::now().date_naive().format("%Y-%m-%d").to_string()
        );
        assert_eq!(p.input_tokens, 40);
        assert_eq!(p.cache_read_tokens, 10);
        // cache_write_tokens 映射自 cache_creation_tokens
        assert_eq!(p.cache_write_tokens, 5);
        // trends_daily 的 output 已含 reasoning（30+15）
        assert_eq!(p.output_tokens, 45);
        assert!((p.cost_usd - 0.10).abs() < 1e-9);
    }

    #[test]
    fn cost_breakdown_sorts_limits_and_rejects_unknown_dim() {
        let temp = TempDir::new().unwrap();
        let (runtime, conn) = open_fixture(&temp);
        seed_bucket(&conn, &claude_bucket_today());
        seed_bucket(
            &conn,
            &SeedBucket {
                model: "claude-haiku".to_string(),
                project_hash: "p2".to_string(),
                project_label: "Project 2".to_string(),
                project_path: Some("/repo/b".to_string()),
                cost_with_cache_usd: 0.30,
                ..claude_bucket_today()
            },
        );

        let by_model = cost_breakdown(&runtime, "model", None, None).unwrap();
        assert_eq!(by_model.len(), 2);
        // cost 降序：0.30 的 haiku 在前
        assert_eq!(by_model[0].key, "claude-haiku");
        assert!((by_model[0].cost_usd - 0.30).abs() < 1e-9);

        let by_project = cost_breakdown(&runtime, "project", None, None).unwrap();
        assert_eq!(by_project[0].key, "/repo/b");

        let limited = cost_breakdown(&runtime, "model", None, Some(1)).unwrap();
        assert_eq!(limited.len(), 1);

        let err = cost_breakdown(&runtime, "tool", None, None).unwrap_err();
        assert!(err.contains("Unsupported breakdown dim"));
    }

    #[test]
    fn cache_stats_aggregates_tokens() {
        let temp = TempDir::new().unwrap();
        let (runtime, conn) = open_fixture(&temp);
        seed_bucket(&conn, &claude_bucket_today());

        let stats = cache_stats(&runtime).expect("cache_stats should succeed");
        assert_eq!(stats.total_input_tokens, 40);
        assert_eq!(stats.total_cache_read_tokens, 10);
        assert_eq!(stats.total_cache_write_tokens, 5);
        // output_tokens_with_reasoning = 30 + 15
        assert_eq!(stats.total_output_tokens, 45);
        assert!((0.0..=1.0).contains(&stats.hit_rate));
    }

    #[test]
    fn tool_heatmap_and_top_tools_read_tool_calls() {
        let temp = TempDir::new().unwrap();
        let pool = temp_pool(&temp);
        seed_tool_call(&pool, "s1", 1, "Bash", 0.20);
        seed_tool_call(&pool, "s1", 2, "Bash", 0.10);
        seed_tool_call(&pool, "s2", 1, "Read", 0.05);

        let cells = tool_heatmap(&pool, None).expect("heatmap should succeed");
        let total: i64 = cells.iter().map(|c| c.count).sum();
        assert_eq!(total, 3);
        assert!(
            cells
                .iter()
                .all(|c| (0..=6).contains(&c.dow) && (0..=23).contains(&c.hour))
        );

        let tools = top_tools(&pool, None, None).expect("top_tools should succeed");
        assert_eq!(tools[0].tool_name, "Bash");
        assert_eq!(tools[0].call_count, 2);
        assert!((tools[0].cost_usd - 0.30).abs() < 1e-9);

        let limited = top_tools(&pool, None, Some(1)).unwrap();
        assert_eq!(limited.len(), 1);
    }

    #[test]
    fn top_sessions_orders_by_cost_or_calls() {
        let temp = TempDir::new().unwrap();
        let pool = temp_pool(&temp);
        // s1：1 次调用高成本；s2：3 次调用低成本
        seed_tool_call(&pool, "s1", 1, "Bash", 0.50);
        seed_tool_call(&pool, "s2", 1, "Read", 0.10);
        seed_tool_call(&pool, "s2", 2, "Read", 0.10);
        seed_tool_call(&pool, "s2", 3, "Read", 0.10);

        let by_cost = top_sessions(&pool, None, None).expect("top_sessions should succeed");
        assert_eq!(by_cost[0].session_id, "s1");
        assert!((by_cost[0].cost_usd - 0.50).abs() < 1e-9);
        // wire 契约：model/started_at 现阶段恒为空，tokens 恒 0
        assert_eq!(by_cost[0].tokens, 0);
        assert!(by_cost[0].model.is_none());

        let by_calls = top_sessions(&pool, None, Some("calls")).unwrap();
        assert_eq!(by_calls[0].session_id, "s2");
        assert_eq!(by_calls[0].tool_call_count, 3);

        let limited = top_sessions(&pool, Some(1), None).unwrap();
        assert_eq!(limited.len(), 1);
    }

    #[test]
    fn insight_assembles_dual_sources_and_roi_branches() {
        let temp = TempDir::new().unwrap();
        let (runtime, conn) = open_fixture(&temp);
        seed_bucket(&conn, &claude_bucket_today());
        let pool = temp_pool(&temp);
        seed_tool_call(&pool, "s1", 1, "Bash", 0.20);
        seed_tool_call(&pool, "s2", 1, "Read", 0.05);

        // 分支1：默认订阅（mode=auto）→ roi None
        let dto = insight(&runtime, &pool).expect("insight should succeed");
        assert!((dto.today_value_usd - 0.10).abs() < 1e-9);
        assert!((dto.month_value_usd - 0.10).abs() < 1e-9);
        assert!((dto.total_value_usd - 0.10).abs() < 1e-9);
        assert_eq!(dto.today_tokens, 100);
        assert_eq!(dto.total_sessions, 2);
        assert_eq!(dto.total_projects, 1);
        assert_eq!(dto.subscription.mode, "auto");
        assert!(dto.roi.is_none());
        assert!(!dto.pricing_version.is_empty());

        // 分支2：订阅模式 → roi = month / monthly_usd
        {
            let sub_conn = pool.get().unwrap();
            subscription::set(&sub_conn, "subscription", "max20x", 100.0).unwrap();
        }
        let dto = insight(&runtime, &pool).expect("insight should succeed");
        let roi = dto.roi.expect("roi should be present in subscription mode");
        assert!((roi - dto.month_value_usd / 100.0).abs() < 1e-9);
    }
}
