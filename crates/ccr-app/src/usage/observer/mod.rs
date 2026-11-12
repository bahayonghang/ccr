//! Claude-only dashboard projections and existing CCR tool/settings orchestration.
//! SQL remains in ccr-usage and ccr-db; this module has no framework or filesystem owner.
pub mod jsonl;
pub mod scanner;
pub mod subscription;
use super::report_clock::ReportClock;
use ccr_db::database::repositories::claude_tool_calls_repo;
use ccr_usage::build_filter;
use chrono::Datelike;
use serde::Serialize;
use tracing::debug;

// Shared projections; framework-specific binding derives stay in the adapters.

#[derive(Debug, Clone, Serialize)]
pub struct InsightDto {
    pub today_value_usd: f64,
    pub month_value_usd: f64,
    pub total_value_usd: f64,
    pub today_tokens: i64,
    pub month_tokens: i64,
    pub total_sessions: i64,
    pub total_projects: i64,
    pub project_attribution_complete: bool,
    pub subscription: subscription::SubscriptionDto,
    /// 当 mode=subscription 时，month_value_usd / monthly_usd；否则 None
    pub roi: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CacheStatsDto {
    pub hit_rate: f64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cache_read_tokens: i64,
    pub total_cache_write_tokens: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyPoint {
    pub date: String,
    pub cost_usd: f64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BreakdownRow {
    pub key: String,
    pub cost_usd: f64,
    pub tokens: i64,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionRow {
    pub session_id: String,
    pub project_path: Option<String>,
    pub model: Option<String>,
    pub cost_usd: Option<f64>,
    pub tokens: Option<i64>,
    pub tool_call_count: i64,
    pub started_at: Option<String>,
}

/// 周×小时热力图单元（wire 版，映射自 `claude_tool_calls_repo::HeatmapCell`）
#[derive(Debug, Clone, Serialize)]
pub struct HeatmapCell {
    /// 0=Sun..6=Sat
    pub dow: i64,
    /// 0..23
    pub hour: i64,
    pub count: i64,
    pub timezone: &'static str,
}

impl From<claude_tool_calls_repo::HeatmapCell> for HeatmapCell {
    fn from(cell: claude_tool_calls_repo::HeatmapCell) -> Self {
        Self {
            dow: cell.dow,
            hour: cell.hour,
            count: cell.count,
            timezone: "utc",
        }
    }
}

/// Top tools 行（wire 版，映射自 `claude_tool_calls_repo::TopToolRow`）
#[derive(Debug, Clone, Serialize)]
pub struct TopToolRow {
    pub tool_name: String,
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

// ── 工具：日期窗口与 overview 拉取 ────────────────────────────────────────

fn overview_in_window(
    llmusage: &ccr_usage::AppPaths,
    start_date: Option<String>,
    end_date: Option<String>,
    clock: ReportClock,
) -> Result<ccr_usage::OverviewPayload, String> {
    let mut filter = build_filter(Some("claude".to_string()), None, start_date, end_date)?;
    filter.timezone = clock.timezone;
    let dashboard = ccr_usage::open_dashboard(llmusage.clone())
        .map_err(|e| format!("Dashboard open error: {e}"))?;
    (if clock.timezone == ccr_usage::ReportTimezone::Utc {
        dashboard.overview_at(&filter, clock.as_of)
    } else {
        dashboard.overview(&filter)
    })
    .map_err(|e| format!("Overview query error: {e}"))
}

/// 计算「今日」/「本月」的日期窗口（local time）。返回 (start_yyyy_mm_dd, end_yyyy_mm_dd)。
fn today_window(clock: ReportClock) -> (String, String) {
    let today = clock.date();
    let s = today.format("%Y-%m-%d").to_string();
    (s.clone(), s)
}
fn month_window(clock: ReportClock) -> (String, String) {
    let today = clock.date();
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
pub fn insight(
    llmusage: &ccr_usage::AppPaths,
    conn: &rusqlite::Connection,
) -> Result<InsightDto, String> {
    insight_with_clock(llmusage, conn, ReportClock::live())
}
pub fn insight_with_clock(
    llmusage: &ccr_usage::AppPaths,
    conn: &rusqlite::Connection,
    clock: ReportClock,
) -> Result<InsightDto, String> {
    /* ====================================================================
     * 步骤1：从 llmusage 拉 total / today / month 三个 overview
     * ====================================================================
     */
    debug!("[claude_observer] get_insight: fetching total overview");
    let total = overview_in_window(llmusage, None, None, clock)?;

    let (today_s, today_e) = today_window(clock);
    debug!(
        "[claude_observer] get_insight: fetching today overview ({} to {})",
        today_s, today_e
    );
    let today = overview_in_window(llmusage, Some(today_s), Some(today_e), clock)?;

    let (month_s, month_e) = month_window(clock);
    debug!(
        "[claude_observer] get_insight: fetching month overview ({} to {})",
        month_s, month_e
    );
    let month = overview_in_window(llmusage, Some(month_s), Some(month_e), clock)?;

    /* 步骤2：从 llmusage 拉项目数（distinct 项目维度）*/
    let mut filter = build_filter(Some("claude".to_string()), None, None, None)?;
    filter.timezone = clock.timezone;
    let dashboard = ccr_usage::open_dashboard(llmusage.clone())
        .map_err(|e| format!("Dashboard open error: {e}"))?;
    let projects = dashboard
        .project_breakdown(&filter)
        .map_err(|e| format!("Project breakdown error: {e}"))?;
    let total_projects = projects.len() as i64;
    let project_attribution_complete = dashboard
        .project_attribution_complete(&filter)
        .map_err(|e| format!("Project attribution error: {e}"))?;

    /* 步骤3：从 ccr-db 取 session 总数（用 claude_tool_calls 的 distinct session_id） */
    let total_sessions = claude_tool_calls_repo::session_count(conn)
        .map_err(|e| format!("Session count query error: {e}"))?;
    let sub = subscription::read(conn).map_err(|e| format!("Subscription read error: {e}"))?;

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
        project_attribution_complete,
        subscription: sub,
        roi,
    })
}

/// 最近 `days` 天的每日趋势（默认 30，clamp 1..=365，按 claude 平台过滤）。
pub fn daily_trend(
    llmusage: &ccr_usage::AppPaths,
    days: Option<i64>,
) -> Result<Vec<DailyPoint>, String> {
    daily_trend_with_clock(llmusage, days, ReportClock::live())
}
pub fn daily_trend_with_clock(
    llmusage: &ccr_usage::AppPaths,
    days: Option<i64>,
    clock: ReportClock,
) -> Result<Vec<DailyPoint>, String> {
    let days = days.unwrap_or(30).clamp(1, 365);
    debug!("[claude_observer] daily_trend: days={}", days);

    let today = clock.date();
    let start = today - chrono::Duration::days(days - 1);
    debug!(
        "[claude_observer] daily_trend: date range {} to {}",
        start, today
    );

    let mut filter = build_filter(
        Some("claude".to_string()),
        None,
        Some(start.format("%Y-%m-%d").to_string()),
        Some(today.format("%Y-%m-%d").to_string()),
    )?;
    filter.timezone = clock.timezone;

    debug!(
        "[claude_observer] daily_trend: DB path {:?}",
        llmusage.db_path
    );
    let dashboard = ccr_usage::open_dashboard(llmusage.clone()).map_err(|e| {
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
    llmusage: &ccr_usage::AppPaths,
    dim: &str,
    days: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<BreakdownRow>, String> {
    cost_breakdown_with_clock(llmusage, dim, days, limit, ReportClock::live())
}
pub fn cost_breakdown_with_clock(
    llmusage: &ccr_usage::AppPaths,
    dim: &str,
    days: Option<i64>,
    limit: Option<i64>,
    clock: ReportClock,
) -> Result<Vec<BreakdownRow>, String> {
    let limit = limit.unwrap_or(10).clamp(1, 200) as usize;
    let days = days.unwrap_or(30).clamp(1, 365);
    debug!(
        "[claude_observer] cost_breakdown: dim={}, days={}, limit={}",
        dim, days, limit
    );

    let today = clock.date();
    let start = today - chrono::Duration::days(days - 1);
    let mut filter = build_filter(
        Some("claude".to_string()),
        None,
        Some(start.format("%Y-%m-%d").to_string()),
        Some(today.format("%Y-%m-%d").to_string()),
    )?;
    filter.timezone = clock.timezone;
    let dashboard = ccr_usage::open_dashboard(llmusage.clone()).map_err(|e| {
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
pub fn cache_stats(llmusage: &ccr_usage::AppPaths) -> Result<CacheStatsDto, String> {
    debug!("[claude_observer] cache_stats: fetching");

    let filter = build_filter(Some("claude".to_string()), None, None, None)?;
    let dashboard = ccr_usage::open_dashboard(llmusage.clone()).map_err(|e| {
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
    conn: &rusqlite::Connection,
    limit: Option<i64>,
    by: Option<&str>,
) -> Result<Vec<SessionRow>, String> {
    top_sessions_with_clock(conn, limit, by, ReportClock::live())
}
pub fn top_sessions_with_clock(
    conn: &rusqlite::Connection,
    limit: Option<i64>,
    by: Option<&str>,
    clock: ReportClock,
) -> Result<Vec<SessionRow>, String> {
    let limit = limit.unwrap_or(10).clamp(1, 200);
    let by_cost = matches!(by, Some("cost"));
    debug!(
        "[claude_observer] top_sessions: limit={}, by={:?}",
        limit, by
    );

    let rows = claude_tool_calls_repo::observed_sessions_at(conn, 30, limit, by_cost, clock.as_of)
        .map_err(|e| {
            let err = format!("Efficiency query error: {e}");
            tracing::error!("[claude_observer] {}", err);
            err
        })?;
    let out: Vec<SessionRow> = rows
        .into_iter()
        .take(limit as usize)
        .map(|r| SessionRow {
            session_id: r.session_id,
            project_path: r.project_path,
            model: None,
            cost_usd: r.cost_usd,
            tokens: None,
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
pub fn tool_heatmap(
    conn: &rusqlite::Connection,
    days: Option<i64>,
) -> Result<Vec<HeatmapCell>, String> {
    tool_heatmap_with_clock(conn, days, ReportClock::live())
}
pub fn tool_heatmap_with_clock(
    conn: &rusqlite::Connection,
    days: Option<i64>,
    clock: ReportClock,
) -> Result<Vec<HeatmapCell>, String> {
    let days = days.unwrap_or(30).clamp(1, 365);
    debug!("[claude_observer] tool_heatmap: days={}", days);

    let result = (if clock.timezone == ccr_usage::ReportTimezone::Utc {
        claude_tool_calls_repo::heatmap_at(conn, days, clock.as_of)
    } else {
        claude_tool_calls_repo::heatmap_in_timezone_at(conn, days, clock.as_of, &chrono::Local)
    })
    .map_err(|e| {
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

    Ok(result
        .into_iter()
        .map(|cell| {
            let mut cell = HeatmapCell::from(cell);
            cell.timezone = if clock.timezone == ccr_usage::ReportTimezone::Utc {
                "utc"
            } else {
                "local"
            };
            cell
        })
        .collect())
}

/// Top tools 排行（默认 30 天 / 15 条）。
pub fn top_tools(
    conn: &rusqlite::Connection,
    days: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<TopToolRow>, String> {
    top_tools_with_clock(conn, days, limit, ReportClock::live())
}
pub fn top_tools_with_clock(
    conn: &rusqlite::Connection,
    days: Option<i64>,
    limit: Option<i64>,
    clock: ReportClock,
) -> Result<Vec<TopToolRow>, String> {
    let days = days.unwrap_or(30).clamp(1, 365);
    let limit = limit.unwrap_or(15).clamp(1, 200);
    debug!(
        "[claude_observer] top_tools: days={}, limit={}",
        days, limit
    );

    let result =
        claude_tool_calls_repo::top_tools_at(conn, days, limit, clock.as_of).map_err(|e| {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod clock_tests {
    use super::*;
    use ccr_usage::fixtures::{SeedBucket, create_projection_db, seed_bucket};
    #[test]
    fn project_attribution_distinguishes_empty_unattributed_partial_and_complete() {
        for (hashes, complete, expected_count) in [
            (vec![], true, 0),
            (vec![""], false, 0),
            (vec!["", "project-a"], false, 1),
            (vec!["project-a", "project-a"], true, 1),
        ] {
            let root = tempfile::tempdir().unwrap();
            let paths = create_projection_db(&root.path().join("usage"));
            let usage = rusqlite::Connection::open(&paths.db_path).unwrap();
            for (index, hash) in hashes.into_iter().enumerate() {
                seed_bucket(
                    &usage,
                    &SeedBucket {
                        source: "claude".into(),
                        project_hash: hash.into(),
                        hour_start: format!("2026-11-10T{index:02}:00:00Z"),
                        ..Default::default()
                    },
                );
            }
            seed_bucket(
                &usage,
                &SeedBucket {
                    source: "codex".into(),
                    project_hash: "".into(),
                    ..Default::default()
                },
            );
            drop(usage);
            let pool = ccr_db::database::create_app_pool_at(
                &root.path().join("observer.db"),
                &root.path().join("empty-home"),
            )
            .unwrap();
            let clock = ReportClock::fixed_utc(
                chrono::DateTime::parse_from_rfc3339("2026-11-10T12:00:00Z")
                    .unwrap()
                    .with_timezone(&chrono::Utc),
            );
            let insight = insight_with_clock(&paths, &pool.get().unwrap(), clock).unwrap();
            assert_eq!(insight.total_projects, expected_count);
            let value = serde_json::to_value(insight).unwrap();
            assert_eq!(value["project_attribution_complete"], complete);
            let dashboard = ccr_usage::open_dashboard(paths).unwrap();
            let snapshot = dashboard
                .insights(
                    &ccr_usage::QueryFilter {
                        source: Some(ccr_usage::SourceKind::Claude),
                        timezone: ccr_usage::ReportTimezone::Utc,
                        ..Default::default()
                    },
                    clock.date(),
                )
                .unwrap();
            assert_eq!(snapshot.project_count, expected_count);
            assert_eq!(
                serde_json::to_value(snapshot).unwrap()["project_attribution_complete"],
                complete
            );
        }
    }
    #[test]
    fn observer_defaults_use_fixed_utc_today_month_and_days() {
        let root = tempfile::tempdir().unwrap();
        let paths = create_projection_db(root.path());
        let pool = ccr_db::database::create_pool(&paths.db_path, None).unwrap();
        let conn = pool.get().unwrap();
        for ts in [
            "1999-12-31T23:59:59Z",
            "2000-01-01T00:00:00Z",
            "2000-01-02T00:00:00Z",
        ] {
            seed_bucket(
                &conn,
                &SeedBucket {
                    source: "claude".into(),
                    hour_start: ts.into(),
                    ..Default::default()
                },
            );
        }
        drop(conn);
        drop(pool);
        let at = chrono::DateTime::parse_from_rfc3339("2000-01-01T00:30:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let clock = ReportClock::fixed_utc(at);
        assert_eq!(
            today_window(clock),
            ("2000-01-01".into(), "2000-01-01".into())
        );
        assert_eq!(month_window(clock), today_window(clock));
        let trend = daily_trend_with_clock(&paths, Some(1), clock).unwrap();
        assert_eq!(trend.len(), 1);
        assert_eq!(trend[0].date, "2000-01-01");
        let rows = cost_breakdown_with_clock(&paths, "model", Some(1), Some(20), clock).unwrap();
        assert_eq!(rows[0].tokens, 100);
    }
}
