//! 统计命令 — 费用概览、热力图、会话统计。
//!
//! WP-H Phase 2a（缓存子目标）：把 stats 命令统一接入 `AppState` singleflight 缓存，
//! TTL 30 秒，避免每次前端访问都触发 JSONL 全量扫描。真正的 SQL 聚合归档查询
//! 留给后续 Phase 2b（usage_db_pool + usage_repo 聚合 SQL）。

use std::{
    collections::{HashMap, HashSet},
    future::Future,
};

use ccr_store::CostTracker;
use chrono::{Duration, Utc};
use tauri::State;

use crate::state::{AppState, CacheFillRegistration};

/// 默认 stats 缓存 TTL（秒）。30s 可以吸收一次页面内的连续刷新抖动，
/// 同时保证数据不至于严重滞后。
const STATS_CACHE_TTL_SECS: u64 = 30;

/// 平台每日统计累加器 (内部使用)
#[derive(Default)]
struct PlatformAccum {
    session_ids: HashSet<String>,
    anonymous_count: usize,
    messages: u64,
    tokens: u64,
    duration_ms: u64,
}

/// 归一化平台名称为前端期望的三种之一: claude / codex / gemini
fn normalize_platform(raw: &str) -> &'static str {
    match raw.to_lowercase().as_str() {
        "claude" | "claude-code" | "claude code" => "claude",
        "codex" | "openai-codex" | "openai codex" => "codex",
        "gemini" | "google-gemini" | "google gemini" => "gemini",
        _ => "claude", // 未知平台默认归入 claude
    }
}

/// 统一创建 CostTracker，避免每个 compute 函数重复展开同一段样板代码。
fn create_cost_tracker() -> Result<CostTracker, String> {
    let storage_dir =
        CostTracker::default_storage_dir().map_err(|e| format!("Failed to get stats dir: {e}"))?;
    CostTracker::new(storage_dir).map_err(|e| format!("Failed to create cost tracker: {e}"))
}

/// 统一封装 stats singleflight 缓存。
async fn run_cached_stats_command<F, Fut>(
    state: &AppState,
    cache_key: String,
    compute: F,
) -> Result<serde_json::Value, String>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<serde_json::Value, String>>,
{
    if let Some(cached) = state.cache_get(&cache_key).await {
        tracing::debug!(cache_key = %cache_key, "stats cache hit");
        return Ok(cached);
    }

    match state.begin_cache_fill(&cache_key).await {
        CacheFillRegistration::Wait(notify) => {
            tracing::debug!(cache_key = %cache_key, "stats cache wait");
            notify.notified().await;
            if let Some(cached) = state.cache_get(&cache_key).await {
                tracing::debug!(cache_key = %cache_key, "stats cache hit after wait");
                return Ok(cached);
            }
            tracing::debug!(cache_key = %cache_key, "stats cache fallback compute");
            compute().await
        }
        CacheFillRegistration::Leader => match compute().await {
            Ok(payload) => {
                state
                    .cache_set(cache_key.clone(), payload.clone(), STATS_CACHE_TTL_SECS)
                    .await;
                state.finish_cache_fill(&cache_key).await;
                tracing::debug!(cache_key = %cache_key, "stats cache fill complete");
                Ok(payload)
            }
            Err(err) => {
                state.finish_cache_fill(&cache_key).await;
                Err(err)
            }
        },
    }
}

/// 计算费用概览（compute 函数，供 cached 命令与 Wait 降级路径共用）。
///
/// 提取为独立 async 函数的目的：
/// 1) 让 command 层能在 Leader 和 Wait 两条路径上都调用它，无需 FnOnce Clone；
/// 2) `spawn_blocking` 内部是同步 CostTracker 读取，隔离在 blocking 池上。
async fn compute_cost_overview(period: String) -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let tracker = create_cost_tracker()?;

        let now = Utc::now();
        let start = match period.as_str() {
            "today" => now
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .map(|t| t.and_utc())
                .unwrap_or(now - Duration::hours(24)),
            "week" => now - Duration::days(7),
            "month" => now - Duration::days(30),
            _ => now - Duration::hours(24),
        };

        // generate_stats() 已返回完整 CostStats (含 token_stats, by_model, by_project, trend)
        let stats = tracker
            .generate_stats(start, now)
            .map_err(|e| format!("Failed to generate stats: {e}"))?;

        serde_json::to_value(&stats).map_err(|e| format!("Serialize error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

async fn compute_heatmap_data(
    platform: Option<String>,
    days: usize,
) -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let tracker = create_cost_tracker()?;

        let now = Utc::now();
        let start = now - Duration::days(days as i64);

        let records = tracker
            .read_by_time_range(start, now)
            .map_err(|e| format!("Failed to read records: {e}"))?;

        // 按日期汇总 token 数
        let mut daily_tokens: HashMap<String, u64> = HashMap::new();
        for record in &records {
            // 可选平台过滤（通过 model 名称前缀匹配）
            if let Some(ref p) = platform
                && !record.model.to_lowercase().contains(&p.to_lowercase())
            {
                continue;
            }
            let date = record.timestamp.format("%Y-%m-%d").to_string();
            let tokens =
                record.token_usage.input_tokens as u64 + record.token_usage.output_tokens as u64;
            *daily_tokens.entry(date).or_insert(0) += tokens;
        }

        let max_value = daily_tokens.values().copied().max().unwrap_or(0);
        let total_tokens: u64 = daily_tokens.values().sum();
        let active_days = daily_tokens.len() as u32;

        Ok::<_, String>(serde_json::json!({
            "data": daily_tokens,
            "max_value": max_value,
            "total_tokens": total_tokens,
            "active_days": active_days,
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

async fn compute_session_stats(platform: Option<String>) -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let tracker = create_cost_tracker()?;

        let today_cost = tracker
            .get_today_cost()
            .map_err(|e| format!("Failed to get today cost: {e}"))?;
        let week_cost = tracker
            .get_week_cost()
            .map_err(|e| format!("Failed to get week cost: {e}"))?;
        let month_cost = tracker
            .get_month_cost()
            .map_err(|e| format!("Failed to get month cost: {e}"))?;

        let top_sessions = tracker
            .get_top_sessions(10)
            .map_err(|e| format!("Failed to get top sessions: {e}"))?;

        let _ = platform; // TODO: filter by platform

        Ok::<_, String>(serde_json::json!({
            "today_cost": today_cost,
            "week_cost": week_cost,
            "month_cost": month_cost,
            "top_sessions": top_sessions.into_iter().map(|(id, cost)| {
                serde_json::json!({"session_id": id, "cost": cost})
            }).collect::<Vec<_>>(),
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

async fn compute_cost_trend(period: String) -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let tracker = create_cost_tracker()?;

        let now = Utc::now();
        let start = match period.as_str() {
            "week" => now - Duration::days(7),
            "month" => now - Duration::days(30),
            "year" => now - Duration::days(365),
            _ => now - Duration::days(30),
        };

        let records = tracker
            .read_by_time_range(start, now)
            .map_err(|e| format!("Failed to read records: {e}"))?;

        // 按日期聚合
        let mut daily: HashMap<String, (f64, u64)> = HashMap::new();
        for r in &records {
            let date = r.timestamp.format("%Y-%m-%d").to_string();
            let entry = daily.entry(date).or_insert((0.0, 0));
            entry.0 += r.cost.total_cost;
            entry.1 += r.token_usage.input_tokens as u64 + r.token_usage.output_tokens as u64;
        }

        let mut trend: Vec<serde_json::Value> = daily
            .into_iter()
            .map(|(date, (cost, tokens))| {
                serde_json::json!({
                    "date": date,
                    "cost": cost,
                    "tokens": tokens,
                })
            })
            .collect();
        trend.sort_by(|a, b| {
            a["date"]
                .as_str()
                .unwrap_or("")
                .cmp(b["date"].as_str().unwrap_or(""))
        });

        Ok::<_, String>(serde_json::json!({ "trend": trend }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

async fn compute_cost_by_model() -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let tracker = create_cost_tracker()?;

        let now = Utc::now();
        let start = now - Duration::days(30);
        let stats = tracker
            .generate_stats(start, now)
            .map_err(|e| format!("Failed to generate stats: {e}"))?;

        serde_json::to_value(&stats.by_model).map_err(|e| format!("Serialize error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

async fn compute_cost_by_project() -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let tracker = create_cost_tracker()?;

        let now = Utc::now();
        let start = now - Duration::days(30);
        let stats = tracker
            .generate_stats(start, now)
            .map_err(|e| format!("Failed to generate stats: {e}"))?;

        serde_json::to_value(&stats.by_project).map_err(|e| format!("Serialize error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

async fn compute_provider_usage() -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let tracker = create_cost_tracker()?;

        let now = Utc::now();
        let start = now - Duration::days(30);
        let stats = tracker
            .generate_stats(start, now)
            .map_err(|e| format!("Failed to generate stats: {e}"))?;

        serde_json::to_value(&stats.by_provider).map_err(|e| format!("Serialize error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

async fn compute_top_sessions(limit: usize) -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let tracker = create_cost_tracker()?;

        let sessions = tracker
            .get_top_sessions(limit)
            .map_err(|e| format!("Failed to get top sessions: {e}"))?;

        let result: Vec<serde_json::Value> = sessions
            .into_iter()
            .map(|(id, cost)| serde_json::json!({ "session_id": id, "cost": cost }))
            .collect();

        Ok::<_, String>(serde_json::json!({ "sessions": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

async fn compute_stats_summary() -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let tracker = create_cost_tracker()?;

        let today = tracker
            .get_today_cost()
            .map_err(|e| format!("Failed to get today cost: {e}"))?;
        let week = tracker
            .get_week_cost()
            .map_err(|e| format!("Failed to get week cost: {e}"))?;
        let month = tracker
            .get_month_cost()
            .map_err(|e| format!("Failed to get month cost: {e}"))?;

        Ok::<_, String>(serde_json::json!({
            "today": today,
            "week": week,
                "month": month,
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

async fn compute_daily_stats(days: usize) -> Result<serde_json::Value, String> {
    tokio::task::spawn_blocking(move || {
        let tracker = create_cost_tracker()?;

        let now = Utc::now();
        let start = now - Duration::days(days as i64);

        let records = tracker
            .read_by_time_range(start, now)
            .map_err(|e| format!("Failed to read records: {e}"))?;

        // 按日期 + 平台聚合
        let mut daily_platform: HashMap<String, HashMap<String, PlatformAccum>> = HashMap::new();

        for r in &records {
            let date = r.timestamp.format("%Y-%m-%d").to_string();
            let raw_platform = r
                .platform
                .as_deref()
                .filter(|p| !p.is_empty())
                .unwrap_or("claude");
            let platform = normalize_platform(raw_platform).to_string();

            let day_entry = daily_platform.entry(date).or_default();
            let plat_entry = day_entry.entry(platform).or_default();

            if let Some(ref sid) = r.session_id {
                plat_entry.session_ids.insert(sid.clone());
            } else {
                plat_entry.anonymous_count += 1;
            }
            plat_entry.messages += 1;
            plat_entry.tokens +=
                r.token_usage.input_tokens as u64 + r.token_usage.output_tokens as u64;
            plat_entry.duration_ms += r.duration_ms;
        }

        // 构建 DailyStatsItem 列表和汇总
        let mut daily_items: Vec<serde_json::Value> = Vec::new();
        let mut total_sessions: usize = 0;
        let mut total_messages: u64 = 0;
        let mut total_duration_seconds: u64 = 0;
        // (sessions, messages, tokens, duration_s)
        let mut summary_by_platform: HashMap<String, (usize, u64, u64, u64)> = HashMap::new();

        let mut dates: Vec<String> = daily_platform.keys().cloned().collect();
        dates.sort();

        for date in &dates {
            let platforms = &daily_platform[date];
            let mut day_item = serde_json::Map::new();
            day_item.insert("date".to_string(), serde_json::Value::String(date.clone()));

            for platform_name in &["claude", "codex", "gemini"] {
                let stats = if let Some(acc) = platforms.get(*platform_name) {
                    let sessions = acc.session_ids.len() + acc.anonymous_count;
                    let duration_s = acc.duration_ms / 1000;

                    let summary = summary_by_platform
                        .entry(platform_name.to_string())
                        .or_insert((0, 0, 0, 0));
                    summary.0 += sessions;
                    summary.1 += acc.messages;
                    summary.2 += acc.tokens;
                    summary.3 += duration_s;

                    total_sessions += sessions;
                    total_messages += acc.messages;
                    total_duration_seconds += duration_s;

                    serde_json::json!({
                        "sessions": sessions,
                        "messages": acc.messages,
                        "tokens": acc.tokens,
                        "duration_seconds": duration_s,
                    })
                } else {
                    serde_json::json!({
                        "sessions": 0,
                        "messages": 0,
                        "tokens": 0,
                        "duration_seconds": 0,
                    })
                };
                day_item.insert(platform_name.to_string(), stats);
            }

            daily_items.push(serde_json::Value::Object(day_item));
        }

        // 构建 summary.by_platform
        let mut by_platform = serde_json::Map::new();
        for (platform, (sessions, messages, tokens, duration_s)) in &summary_by_platform {
            by_platform.insert(
                platform.clone(),
                serde_json::json!({
                    "sessions": sessions,
                    "messages": messages,
                    "tokens": tokens,
                    "duration_seconds": duration_s,
                }),
            );
        }

        Ok::<_, String>(serde_json::json!({
            "daily_stats": daily_items,
            "summary": {
                "total_sessions": total_sessions,
                "total_messages": total_messages,
                "total_duration_seconds": total_duration_seconds,
                "by_platform": serde_json::Value::Object(by_platform),
            },
            "last_updated": Utc::now().to_rfc3339(),
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 费用概览 — 直接返回核心库的完整 CostStats
///
/// 前端 StatsView.vue 期望: CostStats { total_cost, record_count, token_stats, by_provider, by_model, by_project, trend? }
#[tauri::command]
pub async fn get_cost_overview(
    state: State<'_, AppState>,
    period: Option<String>,
) -> Result<serde_json::Value, String> {
    let period = period.unwrap_or_else(|| "today".to_string());
    let cache_key = format!("stats:cost_overview:{period}");
    run_cached_stats_command(state.inner(), cache_key, move || {
        compute_cost_overview(period.clone())
    })
    .await
}

#[tauri::command]
pub async fn get_heatmap_data(
    state: State<'_, AppState>,
    platform: Option<String>,
    days: Option<usize>,
) -> Result<serde_json::Value, String> {
    let days = days.unwrap_or(365);
    let platform_key = platform.as_deref().unwrap_or("all");
    let cache_key = format!("stats:heatmap:platform={platform_key}:days={days}");
    run_cached_stats_command(state.inner(), cache_key, move || {
        compute_heatmap_data(platform.clone(), days)
    })
    .await
}

#[tauri::command]
pub async fn get_session_stats(
    state: State<'_, AppState>,
    platform: Option<String>,
) -> Result<serde_json::Value, String> {
    let platform_key = platform.as_deref().unwrap_or("all");
    let cache_key = format!("stats:session_stats:platform={platform_key}");
    run_cached_stats_command(state.inner(), cache_key, move || {
        compute_session_stats(platform.clone())
    })
    .await
}

#[tauri::command]
pub async fn get_cost_trend(
    state: State<'_, AppState>,
    period: Option<String>,
) -> Result<serde_json::Value, String> {
    let period = period.unwrap_or_else(|| "month".to_string());
    let cache_key = format!("stats:cost_trend:{period}");
    run_cached_stats_command(state.inner(), cache_key, move || {
        compute_cost_trend(period.clone())
    })
    .await
}

/// 按模型分组成本 — 直接返回扁平 map
#[tauri::command]
pub async fn get_cost_by_model(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    run_cached_stats_command(state.inner(), "stats:cost_by_model".to_string(), || {
        compute_cost_by_model()
    })
    .await
}

/// 按项目分组成本 — 直接返回扁平 map
#[tauri::command]
pub async fn get_cost_by_project(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    run_cached_stats_command(state.inner(), "stats:cost_by_project".to_string(), || {
        compute_cost_by_project()
    })
    .await
}

/// Provider 使用统计 — 直接返回扁平 Record<string, number>
///
/// 前端 StatsView.vue 期望: Record<string, number> (provider -> count)
#[tauri::command]
pub async fn get_provider_usage(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    run_cached_stats_command(state.inner(), "stats:provider_usage".to_string(), || {
        compute_provider_usage()
    })
    .await
}

#[tauri::command]
pub async fn get_top_sessions(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<serde_json::Value, String> {
    let limit = limit.unwrap_or(10);
    let cache_key = format!("stats:top_sessions:limit={limit}");
    run_cached_stats_command(state.inner(), cache_key, move || compute_top_sessions(limit)).await
}

#[tauri::command]
pub async fn get_stats_summary(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    run_cached_stats_command(state.inner(), "stats:summary".to_string(), || {
        compute_stats_summary()
    })
    .await
}

/// 每日统计 — 返回 DailyStatsResponse 格式 (含 per-platform 分平台数据)
///
/// 前端 UsageStatsDashboard.vue 期望:
/// { daily_stats: DailyStatsItem[], summary: UsageStatsSummary, last_updated: string }
#[tauri::command]
pub async fn get_daily_stats(
    state: State<'_, AppState>,
    days: Option<usize>,
) -> Result<serde_json::Value, String> {
    let days = days.unwrap_or(30);
    let cache_key = format!("stats:daily_stats:days={days}");
    run_cached_stats_command(state.inner(), cache_key, move || compute_daily_stats(days)).await
}
