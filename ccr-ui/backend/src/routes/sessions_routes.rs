//! Sessions 路由
//!
//! 提供 Session 管理的 API 端点

use axum::{Json, Router, routing::get};
use serde::{Deserialize, Serialize};

/// 创建 sessions 路由
pub fn routes() -> Router {
    Router::new()
        .route("/sessions", get(list_sessions))
        .route("/sessions/stats", get(get_stats))
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
    pub by_platform: std::collections::HashMap<String, u64>,
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
                by_platform: std::collections::HashMap::new(),
            });
        }
    };

    let stats = match indexer.stats() {
        Ok(s) => s,
        Err(_) => {
            return Json(SessionStatsResponse {
                total: 0,
                by_platform: std::collections::HashMap::new(),
            });
        }
    };

    Json(SessionStatsResponse {
        total: stats.total,
        by_platform: stats.by_platform,
    })
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

    Json(ReindexResponse {
        files_scanned: stats.files_scanned,
        sessions_added: stats.sessions_added,
        sessions_updated: stats.sessions_updated,
        errors: stats.errors,
        duration_ms: stats.duration_ms,
    })
}
