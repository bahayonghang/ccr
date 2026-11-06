//! Claude-only dashboard projections and existing CCR tool/settings orchestration.
//! SQL remains in ccr-usage and ccr-db; this module has no framework or filesystem owner.
pub mod jsonl;
pub mod scanner;
pub mod subscription;
use ccr_db::database::repositories::claude_tool_calls_repo;
use ccr_usage::build_filter;
use chrono::{Datelike, Local};
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
    pub cost_usd: f64,
    pub tokens: i64,
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
#[derive(Debug, Clone, Serialize)]
pub struct TopToolRow {
    pub tool_name: String,
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
    llmusage: &ccr_usage::AppPaths,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<ccr_usage::OverviewPayload, String> {
    let filter = build_filter(Some("claude".to_string()), None, start_date, end_date)?;
    let dashboard = ccr_usage::open_dashboard(llmusage.clone())
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
pub fn insight(
    llmusage: &ccr_usage::AppPaths,
    conn: &rusqlite::Connection,
) -> Result<InsightDto, String> {
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
    let dashboard = ccr_usage::open_dashboard(llmusage.clone())
        .map_err(|e| format!("Dashboard open error: {e}"))?;
    let projects = dashboard
        .project_breakdown(&filter)
        .map_err(|e| format!("Project breakdown error: {e}"))?;
    let total_projects = projects.len() as i64;

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
        subscription: sub,
        roi,
    })
}

/// 最近 `days` 天的每日趋势（默认 30，clamp 1..=365，按 claude 平台过滤）。
pub fn daily_trend(
    llmusage: &ccr_usage::AppPaths,
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
    let limit = limit.unwrap_or(10).clamp(1, 200);
    let by_calls = matches!(by, Some("calls"));
    debug!(
        "[claude_observer] top_sessions: limit={}, by={:?}",
        limit, by
    );

    // 用 efficiency_sessions 拉全量，再按需重排
    let mut rows =
        claude_tool_calls_repo::efficiency_sessions(conn, 30, 0.0, 1000).map_err(|e| {
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
pub fn tool_heatmap(
    conn: &rusqlite::Connection,
    days: Option<i64>,
) -> Result<Vec<HeatmapCell>, String> {
    let days = days.unwrap_or(30).clamp(1, 365);
    debug!("[claude_observer] tool_heatmap: days={}", days);

    let result = claude_tool_calls_repo::heatmap(conn, days).map_err(|e| {
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
    conn: &rusqlite::Connection,
    days: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<TopToolRow>, String> {
    let days = days.unwrap_or(30).clamp(1, 365);
    let limit = limit.unwrap_or(15).clamp(1, 200);
    debug!(
        "[claude_observer] top_tools: days={}, limit={}",
        days, limit
    );

    let result = claude_tool_calls_repo::top_tools(conn, days, limit).map_err(|e| {
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
