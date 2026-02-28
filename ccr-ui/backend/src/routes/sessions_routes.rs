//! Sessions 路由
//!
//! 提供 Session 管理的 API 端点

use crate::state::AppState;
use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::info;

struct CachedDailyStats {
    response: DailyStatsResponse,
    timestamp: Instant,
}

lazy_static::lazy_static! {
    static ref DAILY_STATS_CACHE: Arc<Mutex<HashMap<u32, CachedDailyStats>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

const DAILY_STATS_CACHE_TTL: Duration = Duration::from_secs(60);

fn sessions_daily_cache_enabled() -> bool {
    std::env::var("SESSIONS_DAILY_CACHE")
        .ok()
        .map(|v| {
            let normalized = v.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(true)
}

/// 创建 sessions 路由
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/sessions", get(list_sessions))
        .route("/sessions/stats", get(get_stats))
        .route("/sessions/stats/daily", get(get_daily_stats))
        .route("/sessions/reindex", axum::routing::post(reindex))
}

/// Session 摘要
#[derive(Debug, Serialize)]
pub struct SessionSummaryResponse {
    pub id: String,
    pub platform: String,
    pub title: Option<String>,
    pub cwd: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: u32,
}

/// 查询参数
#[derive(Debug, Deserialize)]
pub struct SessionListQuery {
    pub platform: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Sessions 统计
#[derive(Debug, Serialize)]
pub struct SessionStatsResponse {
    pub total: u64,
    pub by_platform: HashMap<String, u64>,
}

/// 索引结果
#[derive(Debug, Serialize)]
pub struct ReindexResponse {
    pub files_scanned: u64,
    pub sessions_added: u64,
    pub sessions_updated: u64,
    pub errors: u64,
    pub duration_ms: u64,
}

/// 每日统计查询参数
#[derive(Debug, Deserialize)]
pub struct DailyStatsQuery {
    /// 查询天数，默认 30
    pub days: Option<u32>,
}

/// 平台统计
#[derive(Debug, Clone, Default, Serialize)]
pub struct PlatformDailyStats {
    pub sessions: u64,
    pub messages: u64,
    pub tokens: u64,
    pub duration_seconds: u64,
}

/// 每日统计项
#[derive(Debug, Clone, Serialize)]
pub struct DailyStatsItem {
    pub date: String,
    pub claude: PlatformDailyStats,
    pub codex: PlatformDailyStats,
    pub gemini: PlatformDailyStats,
}

/// 汇总统计
#[derive(Debug, Clone, Serialize)]
pub struct StatsSummary {
    pub total_sessions: u64,
    pub total_messages: u64,
    pub total_duration_seconds: u64,
    pub by_platform: HashMap<String, PlatformDailyStats>,
}

/// 每日统计响应
#[derive(Debug, Clone, Serialize)]
pub struct DailyStatsResponse {
    pub daily_stats: Vec<DailyStatsItem>,
    pub summary: StatsSummary,
    pub last_updated: String,
}

/// 列出 sessions
async fn list_sessions(
    axum::extract::Query(query): axum::extract::Query<SessionListQuery>,
) -> Json<Vec<SessionSummaryResponse>> {
    use ccr::models::Platform;
    use ccr::sessions::{SessionIndexer, models::SessionFilter};

    let indexer = match SessionIndexer::new() {
        Ok(i) => i,
        Err(_) => return Json(vec![]),
    };

    // 先确保索引是最新的
    let _ = indexer.index_all();

    let platform = query
        .platform
        .as_deref()
        .and_then(|s| match s.to_lowercase().as_str() {
            "claude" => Some(Platform::Claude),
            "codex" => Some(Platform::Codex),
            "gemini" => Some(Platform::Gemini),
            _ => None,
        });

    let filter = SessionFilter {
        platform,
        limit: Some(query.limit.unwrap_or(50)),
        offset: query.offset,
        ..Default::default()
    };

    let sessions = match indexer.list(filter) {
        Ok(s) => s,
        Err(_) => return Json(vec![]),
    };

    let response: Vec<SessionSummaryResponse> = sessions
        .into_iter()
        .map(|s| SessionSummaryResponse {
            id: s.id,
            platform: format!("{:?}", s.platform),
            title: s.title,
            cwd: s.cwd,
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
            message_count: s.message_count,
        })
        .collect();

    Json(response)
}

/// 获取统计
async fn get_stats() -> Json<SessionStatsResponse> {
    use ccr::sessions::SessionIndexer;

    let indexer = match SessionIndexer::new() {
        Ok(i) => i,
        Err(_) => {
            return Json(SessionStatsResponse {
                total: 0,
                by_platform: HashMap::new(),
            });
        }
    };

    // 先确保索引是最新的
    let _ = indexer.index_all();

    let stats = match indexer.stats() {
        Ok(s) => s,
        Err(_) => {
            return Json(SessionStatsResponse {
                total: 0,
                by_platform: HashMap::new(),
            });
        }
    };

    Json(SessionStatsResponse {
        total: stats.total,
        by_platform: stats.by_platform,
    })
}

/// 获取每日统计 - 支持 CodMate 风格的三视图切换
async fn get_daily_stats(
    axum::extract::Query(query): axum::extract::Query<DailyStatsQuery>,
) -> Json<DailyStatsResponse> {
    let timer = Instant::now();
    let days = query.days.unwrap_or(30).clamp(1, 3650);

    if sessions_daily_cache_enabled() {
        let cache = DAILY_STATS_CACHE
            .lock()
            .expect("daily stats cache poisoned");
        if let Some(cached) = cache.get(&days)
            && cached.timestamp.elapsed() < DAILY_STATS_CACHE_TTL
        {
            info!(
                "sessions daily stats cache hit (days={}, elapsed={}ms)",
                days,
                timer.elapsed().as_millis()
            );
            return Json(cached.response.clone());
        }
    }

    let response = build_daily_stats_response(days, !sessions_daily_cache_enabled());

    if sessions_daily_cache_enabled() {
        let mut cache = DAILY_STATS_CACHE
            .lock()
            .expect("daily stats cache poisoned");
        cache.insert(
            days,
            CachedDailyStats {
                response: response.clone(),
                timestamp: Instant::now(),
            },
        );
    }

    info!(
        "sessions daily stats computed (days={}, elapsed={}ms, cache={})",
        days,
        timer.elapsed().as_millis(),
        sessions_daily_cache_enabled()
    );
    Json(response)
}

fn build_daily_stats_response(days: u32, refresh_index: bool) -> DailyStatsResponse {
    use ccr::sessions::{SessionIndexer, models::SessionFilter};
    use chrono::{Duration, Local, NaiveDate};

    let days_i64 = days as i64;
    let today = Local::now().date_naive();
    let start_date = today - Duration::days(days_i64);

    let indexer = match SessionIndexer::new() {
        Ok(i) => i,
        Err(_) => {
            return DailyStatsResponse {
                daily_stats: vec![],
                summary: StatsSummary {
                    total_sessions: 0,
                    total_messages: 0,
                    total_duration_seconds: 0,
                    by_platform: HashMap::new(),
                },
                last_updated: Local::now().to_rfc3339(),
            };
        }
    };

    if refresh_index {
        let _ = indexer.index_all();
    }

    // 获取所有 sessions
    let filter = SessionFilter {
        limit: Some(10000),
        ..Default::default()
    };

    let sessions = indexer.list(filter).unwrap_or_default();

    // 按日期和平台分组统计
    let mut daily_map: HashMap<NaiveDate, HashMap<String, PlatformDailyStats>> = HashMap::new();
    let mut summary_by_platform: HashMap<String, PlatformDailyStats> = HashMap::new();

    // 初始化日期范围
    for i in 0..=days_i64 {
        let date = start_date + Duration::days(i);
        daily_map.insert(date, HashMap::new());
    }

    // 统计每个 session
    for session in &sessions {
        let session_date = session.created_at.date_naive();
        if session_date < start_date || session_date > today {
            continue;
        }

        let platform_name = format!("{:?}", session.platform).to_lowercase();

        // 计算会话时长（秒）
        let duration_secs = (session.updated_at - session.created_at)
            .num_seconds()
            .max(0) as u64;

        // 估算 tokens (每条消息约 500 tokens)
        let estimated_tokens = session.message_count as u64 * 500;

        // 更新每日统计
        if let Some(day_stats) = daily_map.get_mut(&session_date) {
            let platform_stats = day_stats.entry(platform_name.clone()).or_default();
            platform_stats.sessions += 1;
            platform_stats.messages += session.message_count as u64;
            platform_stats.tokens += estimated_tokens;
            platform_stats.duration_seconds += duration_secs;
        }

        // 更新汇总统计
        let summary_stats = summary_by_platform.entry(platform_name).or_default();
        summary_stats.sessions += 1;
        summary_stats.messages += session.message_count as u64;
        summary_stats.tokens += estimated_tokens;
        summary_stats.duration_seconds += duration_secs;
    }

    // 构建响应
    let mut daily_stats: Vec<DailyStatsItem> = daily_map
        .into_iter()
        .map(|(date, platforms)| DailyStatsItem {
            date: date.format("%Y-%m-%d").to_string(),
            claude: platforms.get("claude").cloned().unwrap_or_default(),
            codex: platforms.get("codex").cloned().unwrap_or_default(),
            gemini: platforms.get("gemini").cloned().unwrap_or_default(),
        })
        .collect();

    // 按日期排序
    daily_stats.sort_by(|a, b| a.date.cmp(&b.date));

    // 计算总计
    let total_sessions: u64 = summary_by_platform.values().map(|s| s.sessions).sum();
    let total_messages: u64 = summary_by_platform.values().map(|s| s.messages).sum();
    let total_duration: u64 = summary_by_platform
        .values()
        .map(|s| s.duration_seconds)
        .sum();

    DailyStatsResponse {
        daily_stats,
        summary: StatsSummary {
            total_sessions,
            total_messages,
            total_duration_seconds: total_duration,
            by_platform: summary_by_platform,
        },
        last_updated: Local::now().to_rfc3339(),
    }
}

/// 重建索引
async fn reindex() -> Json<ReindexResponse> {
    use ccr::sessions::SessionIndexer;

    let indexer = match SessionIndexer::new() {
        Ok(i) => i,
        Err(_) => {
            return Json(ReindexResponse {
                files_scanned: 0,
                sessions_added: 0,
                sessions_updated: 0,
                errors: 1,
                duration_ms: 0,
            });
        }
    };

    let stats = match indexer.index_all() {
        Ok(s) => s,
        Err(_) => {
            return Json(ReindexResponse {
                files_scanned: 0,
                sessions_added: 0,
                sessions_updated: 0,
                errors: 1,
                duration_ms: 0,
            });
        }
    };

    if sessions_daily_cache_enabled() {
        let mut cache = DAILY_STATS_CACHE
            .lock()
            .expect("daily stats cache poisoned");
        cache.clear();
    }

    Json(ReindexResponse {
        files_scanned: stats.files_scanned,
        sessions_added: stats.sessions_added,
        sessions_updated: stats.sessions_updated,
        errors: stats.errors,
        duration_ms: stats.duration_ms,
    })
}
