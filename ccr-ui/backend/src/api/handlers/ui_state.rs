// UI State API Handlers - 收藏和历史记录 REST API

use axum::{
    Json,
    extract::{Path, Query},
};
use serde::Deserialize;
use tracing::{error, info};

use crate::managers::ui_state_manager::get_ui_state_manager;
use crate::models::api::ApiResponse;
use crate::models::ui_state::{
    AddFavoriteRequest, FavoriteCommand, FavoritesResponse, HistoryResponse,
};

// ===== 收藏命令 API =====

/// 获取所有收藏命令
/// GET /api/ui-state/favorites
pub async fn get_favorites() -> ApiResponse<FavoritesResponse> {
    info!("获取收藏命令列表");

    match get_ui_state_manager().await {
        Ok(manager) => {
            let favorites = manager.get_favorites().await;
            ApiResponse::success(FavoritesResponse { favorites })
        }
        Err(e) => {
            error!("获取 UI 状态管理器失败: {}", e);
            ApiResponse::error(e)
        }
    }
}

/// 添加收藏命令
/// POST /api/ui-state/favorites
pub async fn add_favorite(Json(req): Json<AddFavoriteRequest>) -> ApiResponse<FavoriteCommand> {
    info!("添加收藏命令: {} {:?}", req.command, req.args);

    match get_ui_state_manager().await {
        Ok(manager) => match manager.add_favorite(req).await {
            Ok(favorite) => ApiResponse::success(favorite),
            Err(e) => {
                error!("添加收藏失败: {}", e);
                ApiResponse::error(e)
            }
        },
        Err(e) => {
            error!("获取 UI 状态管理器失败: {}", e);
            ApiResponse::error(e)
        }
    }
}

/// 删除收藏命令
/// DELETE /api/ui-state/favorites/:id
pub async fn remove_favorite(Path(id): Path<String>) -> ApiResponse<bool> {
    info!("删除收藏命令: {}", id);

    match get_ui_state_manager().await {
        Ok(manager) => match manager.remove_favorite(&id).await {
            Ok(removed) => {
                if removed {
                    ApiResponse::success(true)
                } else {
                    ApiResponse::error("收藏命令不存在".to_string())
                }
            }
            Err(e) => {
                error!("删除收藏失败: {}", e);
                ApiResponse::error(e)
            }
        },
        Err(e) => {
            error!("获取 UI 状态管理器失败: {}", e);
            ApiResponse::error(e)
        }
    }
}

// ===== 命令历史 API =====

/// 历史查询参数
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<usize>,
}

/// 获取命令历史
/// GET /api/ui-state/history
pub async fn get_history(Query(query): Query<HistoryQuery>) -> ApiResponse<HistoryResponse> {
    info!("获取命令历史, limit: {:?}", query.limit);

    match get_ui_state_manager().await {
        Ok(manager) => {
            let history = manager.get_history(query.limit).await;
            let total = history.len();
            ApiResponse::success(HistoryResponse { history, total })
        }
        Err(e) => {
            error!("获取 UI 状态管理器失败: {}", e);
            ApiResponse::error(e)
        }
    }
}

/// 添加命令历史请求
#[derive(Debug, Deserialize)]
pub struct AddHistoryRequest {
    pub command: String,
    pub args: Vec<String>,
    pub success: bool,
    pub duration_ms: u64,
}

/// 添加命令历史
/// POST /api/ui-state/history
pub async fn add_history(
    Json(req): Json<AddHistoryRequest>,
) -> ApiResponse<crate::models::ui_state::CommandHistory> {
    info!("添加命令历史: {} {:?}", req.command, req.args);

    match get_ui_state_manager().await {
        Ok(manager) => {
            match manager
                .add_history(req.command, req.args, req.success, req.duration_ms)
                .await
            {
                Ok(history) => ApiResponse::success(history),
                Err(e) => {
                    error!("添加历史失败: {}", e);
                    ApiResponse::error(e)
                }
            }
        }
        Err(e) => {
            error!("获取 UI 状态管理器失败: {}", e);
            ApiResponse::error(e)
        }
    }
}

/// 清空命令历史
/// DELETE /api/ui-state/history
pub async fn clear_history() -> ApiResponse<bool> {
    info!("清空命令历史");

    match get_ui_state_manager().await {
        Ok(manager) => match manager.clear_history().await {
            Ok(()) => ApiResponse::success(true),
            Err(e) => {
                error!("清空历史失败: {}", e);
                ApiResponse::error(e)
            }
        },
        Err(e) => {
            error!("获取 UI 状态管理器失败: {}", e);
            ApiResponse::error(e)
        }
    }
}
