//! Claude Observer Tauri 命令
//!
//! 这些命令为 Claude Code 页 Usage Insight 面板提供数据。
//!
//! 命令层是薄壳：debug 日志 + State 提取 + spawn_blocking(service) + join 错误
//! 映射。业务编排与 wire DTO 在 `crate::services::claude_observer`（数据源边界
//! 见该模块文档）；订阅 get/set 直接走 `claude_observer::subscription`。

use tauri::State;
use tracing::debug;

use crate::claude_observer::subscription;
use crate::services::claude_observer as service;
use crate::state::AppState;

// wire DTO 随服务层迁移，此处保路径兼容（events.rs 等潜在消费点零改动）
pub use crate::services::claude_observer::{
    BreakdownRow, CacheStatsDto, DailyPoint, HeatmapCell, InsightDto, SessionRow, TopToolRow,
};

// ── 9 个 #[tauri::command] ───────────────────────────────────────────────

/// 一次性返回首屏 Hero 三卡 + 订阅 banner 所需的全部数值。
#[tauri::command]
pub async fn claude_observer_get_insight(
    state: State<'_, AppState>,
    range: Option<String>,
) -> Result<InsightDto, String> {
    debug!(
        "[claude_observer] get_insight: range={:?}",
        range.as_deref()
    );
    let llmusage = state.llmusage.clone();
    let db_pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || service::insight(&llmusage, &db_pool))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
}

/// 最近 `days` 天的每日趋势（按 claude 平台过滤）
#[tauri::command]
pub async fn claude_observer_daily_trend(
    state: State<'_, AppState>,
    days: Option<i64>,
) -> Result<Vec<DailyPoint>, String> {
    let llmusage = state.llmusage.clone();
    tokio::task::spawn_blocking(move || service::daily_trend(&llmusage, days))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
}

/// 按 `dim ∈ {project, model}` 维度的 Top N 拆分（仅 claude 平台）
#[tauri::command]
pub async fn claude_observer_cost_breakdown(
    state: State<'_, AppState>,
    dim: String,
    days: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<BreakdownRow>, String> {
    let llmusage = state.llmusage.clone();
    tokio::task::spawn_blocking(move || service::cost_breakdown(&llmusage, &dim, days, limit))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
}

/// 缓存效率：命中率 + 4 个 token 总量
#[tauri::command]
pub async fn claude_observer_cache_stats(
    state: State<'_, AppState>,
) -> Result<CacheStatsDto, String> {
    let llmusage = state.llmusage.clone();
    tokio::task::spawn_blocking(move || service::cache_stats(&llmusage))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
}

/// 按 cost 或工具调用密度排序的 Top sessions（来自 claude_tool_calls 维度）
///
/// `by` ∈ `cost | calls`，默认 cost。30 天窗口。
#[tauri::command]
pub async fn claude_observer_top_sessions(
    state: State<'_, AppState>,
    limit: Option<i64>,
    by: Option<String>,
) -> Result<Vec<SessionRow>, String> {
    let db_pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || service::top_sessions(&db_pool, limit, by.as_deref()))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
}

/// 周×小时工具调用热力图
#[tauri::command]
pub async fn claude_observer_tool_heatmap(
    state: State<'_, AppState>,
    days: Option<i64>,
) -> Result<Vec<HeatmapCell>, String> {
    let db_pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || service::tool_heatmap(&db_pool, days))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
}

/// Top tools 排行
#[tauri::command]
pub async fn claude_observer_top_tools(
    state: State<'_, AppState>,
    days: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<TopToolRow>, String> {
    let db_pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || service::top_tools(&db_pool, days, limit))
        .await
        .map_err(|e| format!("Task join error: {e}"))?
}

/// 读取订阅设置
#[tauri::command]
pub async fn claude_observer_subscription_get(
    state: State<'_, AppState>,
) -> Result<subscription::SubscriptionDto, String> {
    let db_pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || -> Result<subscription::SubscriptionDto, String> {
        let conn = db_pool.get().map_err(|e| format!("DB pool error: {e}"))?;
        Ok(subscription::get(&conn))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 写入订阅设置
#[tauri::command]
pub async fn claude_observer_subscription_set(
    state: State<'_, AppState>,
    mode: String,
    plan: String,
    monthly_usd: f64,
) -> Result<subscription::SubscriptionDto, String> {
    let db_pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || -> Result<subscription::SubscriptionDto, String> {
        let conn = db_pool.get().map_err(|e| format!("DB pool error: {e}"))?;
        subscription::set(&conn, &mode, &plan, monthly_usd)
            .map_err(|e| format!("Subscription set error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}
