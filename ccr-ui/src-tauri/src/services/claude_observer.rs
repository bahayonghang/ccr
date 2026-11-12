//! Tauri wire compatibility for shared Claude Observer orchestration.
use crate::claude_observer::{pricing, subscription};
use crate::llmusage_adapter::LlmusageRuntime;
use ccr_app::usage::observer as shared;
use ccr_db::database::DbPool;
use ccr_db::database::repositories::claude_tool_calls_repo;
use serde::Serialize;
use ts_rs::TS;
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
    pub project_attribution_complete: bool,
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
    pub cost_usd: Option<f64>,
    #[ts(as = "Option<f64>")]
    pub tokens: Option<i64>,
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
    pub timezone: String,
}

impl From<claude_tool_calls_repo::HeatmapCell> for HeatmapCell {
    fn from(cell: claude_tool_calls_repo::HeatmapCell) -> Self {
        Self {
            dow: cell.dow,
            hour: cell.hour,
            count: cell.count,
            timezone: "utc".into(),
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
    pub cost_usd: Option<f64>,
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

pub fn insight(llmusage: &LlmusageRuntime, db_pool: &DbPool) -> Result<InsightDto, String> {
    let conn = db_pool.get().map_err(|e| format!("DB pool error: {e}"))?;
    shared::insight_with_clock(llmusage.paths(), &conn, crate::fixture::report_clock())
        .map(Into::into)
}
pub fn daily_trend(
    llmusage: &LlmusageRuntime,
    days: Option<i64>,
) -> Result<Vec<DailyPoint>, String> {
    shared::daily_trend_with_clock(llmusage.paths(), days, crate::fixture::report_clock())
        .map(|rows| rows.into_iter().map(Into::into).collect())
}
pub fn cost_breakdown(
    llmusage: &LlmusageRuntime,
    dim: &str,
    days: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<BreakdownRow>, String> {
    shared::cost_breakdown_with_clock(
        llmusage.paths(),
        dim,
        days,
        limit,
        crate::fixture::report_clock(),
    )
    .map(|rows| rows.into_iter().map(Into::into).collect())
}
pub fn cache_stats(llmusage: &LlmusageRuntime) -> Result<CacheStatsDto, String> {
    shared::cache_stats(llmusage.paths()).map(Into::into)
}
pub fn top_sessions(
    db_pool: &DbPool,
    limit: Option<i64>,
    by: Option<&str>,
) -> Result<Vec<SessionRow>, String> {
    let conn = db_pool.get().map_err(|e| format!("DB pool error: {e}"))?;
    shared::top_sessions_with_clock(&conn, limit, by, crate::fixture::report_clock())
        .map(|rows| rows.into_iter().map(Into::into).collect())
}
pub fn tool_heatmap(db_pool: &DbPool, days: Option<i64>) -> Result<Vec<HeatmapCell>, String> {
    let conn = db_pool.get().map_err(|e| format!("DB pool error: {e}"))?;
    shared::tool_heatmap_with_clock(&conn, days, crate::fixture::report_clock())
        .map(|rows| rows.into_iter().map(Into::into).collect())
}
pub fn top_tools(
    db_pool: &DbPool,
    days: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<TopToolRow>, String> {
    let conn = db_pool.get().map_err(|e| format!("DB pool error: {e}"))?;
    shared::top_tools_with_clock(&conn, days, limit, crate::fixture::report_clock())
        .map(|rows| rows.into_iter().map(Into::into).collect())
}
impl From<shared::InsightDto> for InsightDto {
    fn from(value: shared::InsightDto) -> Self {
        Self {
            today_value_usd: value.today_value_usd,
            month_value_usd: value.month_value_usd,
            total_value_usd: value.total_value_usd,
            today_tokens: value.today_tokens,
            month_tokens: value.month_tokens,
            total_sessions: value.total_sessions,
            total_projects: value.total_projects,
            project_attribution_complete: value.project_attribution_complete,
            subscription: value.subscription.into(),
            roi: value.roi,
            pricing_version: pricing::pricing_version().to_string(),
        }
    }
}
impl From<shared::CacheStatsDto> for CacheStatsDto {
    fn from(value: shared::CacheStatsDto) -> Self {
        Self {
            hit_rate: value.hit_rate,
            total_input_tokens: value.total_input_tokens,
            total_output_tokens: value.total_output_tokens,
            total_cache_read_tokens: value.total_cache_read_tokens,
            total_cache_write_tokens: value.total_cache_write_tokens,
        }
    }
}
impl From<shared::DailyPoint> for DailyPoint {
    fn from(value: shared::DailyPoint) -> Self {
        Self {
            date: value.date,
            cost_usd: value.cost_usd,
            input_tokens: value.input_tokens,
            output_tokens: value.output_tokens,
            cache_read_tokens: value.cache_read_tokens,
            cache_write_tokens: value.cache_write_tokens,
        }
    }
}
impl From<shared::BreakdownRow> for BreakdownRow {
    fn from(value: shared::BreakdownRow) -> Self {
        Self {
            key: value.key,
            cost_usd: value.cost_usd,
            tokens: value.tokens,
            count: value.count,
        }
    }
}
impl From<shared::SessionRow> for SessionRow {
    fn from(value: shared::SessionRow) -> Self {
        Self {
            session_id: value.session_id,
            project_path: value.project_path,
            model: value.model,
            cost_usd: value.cost_usd,
            tokens: value.tokens,
            tool_call_count: value.tool_call_count,
            started_at: value.started_at,
        }
    }
}
impl From<shared::HeatmapCell> for HeatmapCell {
    fn from(value: shared::HeatmapCell) -> Self {
        Self {
            dow: value.dow,
            hour: value.hour,
            count: value.count,
            timezone: value.timezone.into(),
        }
    }
}
impl From<shared::TopToolRow> for TopToolRow {
    fn from(value: shared::TopToolRow) -> Self {
        Self {
            tool_name: value.tool_name,
            call_count: value.call_count,
            cost_usd: value.cost_usd,
        }
    }
}
/// 查询编排 service 的封闭单测：llmusage 侧走 `ccr_usage::fixtures` 投影库，
/// tool 调用侧走 temp ccr-db pool，全程不读写真实用户目录。
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod service_tests {
    use super::*;
    use ccr_usage::fixtures::{SeedBucket, create_projection_db, seed_bucket};
    use chrono::Local;
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
            project_ref: Some("/repo/a".to_string()),
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
                project_ref: Some("/repo/b".to_string()),
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
        assert!((tools[0].cost_usd.unwrap() - 0.30).abs() < 1e-9);

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

        let by_cost = top_sessions(&pool, None, Some("cost")).expect("top_sessions should succeed");
        assert_eq!(by_cost[0].session_id, "s1");
        assert!((by_cost[0].cost_usd.unwrap() - 0.50).abs() < 1e-9);
        // Tool events do not provide session model, token totals or start time.
        assert_eq!(by_cost[0].tokens, None);
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
