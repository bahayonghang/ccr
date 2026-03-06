// UI State Manager - 管理收藏和历史记录的持久化
// Migrated from JSON file storage to unified SQLite database

use crate::database;
use crate::database::repositories::ui_state_repo;
use crate::models::ui_state::{AddFavoriteRequest, CommandHistory, FavoriteCommand};
use chrono::Utc;
use tracing::{debug, error, info};
use uuid::Uuid;

/// UI 状态管理器 - 使用 SQLite 存储
pub struct UiStateManager {
    // No state file needed - using SQLite database
}

impl UiStateManager {
    /// 创建新的管理器实例
    pub fn new() -> Result<Self, String> {
        // Ensure database is initialized
        if !database::is_initialized() {
            return Err("Database not initialized".to_string());
        }
        info!("UiStateManager initialized with SQLite backend");
        Ok(Self {})
    }

    // ===== 收藏命令操作 =====

    /// 获取所有收藏命令
    pub fn get_favorites(&self) -> Vec<FavoriteCommand> {
        match database::with_connection(ui_state_repo::get_all_favorites) {
            Ok(favorites) => favorites,
            Err(e) => {
                error!("Failed to get favorites: {}", e);
                Vec::new()
            }
        }
    }

    /// 添加收藏命令
    pub fn add_favorite(&self, req: AddFavoriteRequest) -> Result<FavoriteCommand, String> {
        // Check for duplicate
        let existing = database::with_connection(|conn| {
            ui_state_repo::find_favorite_by_command_args(conn, &req.command, &req.args)
        })
        .map_err(|e| format!("Database error: {}", e))?;

        if let Some(existing) = existing {
            debug!("Favorite already exists: {}", existing.id);
            return Ok(existing);
        }

        // Create new favorite
        let favorite = FavoriteCommand {
            id: Uuid::new_v4().to_string(),
            command: req.command,
            args: req.args,
            display_name: req.display_name,
            module: req.module,
            created_at: Utc::now(),
        };

        database::with_connection(|conn| ui_state_repo::insert_favorite(conn, &favorite))
            .map_err(|e| format!("Failed to insert favorite: {}", e))?;

        info!("Added favorite: {} ({})", favorite.command, favorite.id);
        Ok(favorite)
    }

    /// 删除收藏命令
    pub fn remove_favorite(&self, id: &str) -> Result<bool, String> {
        let deleted = database::with_connection(|conn| ui_state_repo::delete_favorite(conn, id))
            .map_err(|e| format!("Database error: {}", e))?;

        if deleted {
            info!("Removed favorite: {}", id);
        }
        Ok(deleted)
    }

    // ===== 命令历史操作 =====

    /// 获取命令历史
    pub fn get_history(&self, limit: Option<usize>) -> Vec<CommandHistory> {
        let limit = limit.unwrap_or(50);
        match database::with_connection(|conn| ui_state_repo::get_history(conn, limit)) {
            Ok(history) => history,
            Err(e) => {
                error!("Failed to get history: {}", e);
                Vec::new()
            }
        }
    }

    /// 添加命令历史
    pub fn add_history(
        &self,
        command: String,
        args: Vec<String>,
        success: bool,
        duration_ms: u64,
    ) -> Result<CommandHistory, String> {
        let full_command = if args.is_empty() {
            format!("ccr {}", command)
        } else {
            format!("ccr {} {}", command, args.join(" "))
        };

        let history = CommandHistory {
            id: Uuid::new_v4().to_string(),
            full_command,
            command,
            args,
            success,
            executed_at: Utc::now(),
            duration_ms,
        };

        database::with_connection(|conn| ui_state_repo::insert_history(conn, &history))
            .map_err(|e| format!("Failed to insert history: {}", e))?;

        debug!("Added history: {}", history.full_command);
        Ok(history)
    }

    /// 清空命令历史
    pub fn clear_history(&self) -> Result<(), String> {
        let count = database::with_connection(ui_state_repo::clear_history)
            .map_err(|e| format!("Database error: {}", e))?;

        info!("Cleared {} history entries", count);
        Ok(())
    }
}

// 全局单例
use once_cell::sync::OnceCell;
use std::sync::Arc;

static UI_STATE_MANAGER: OnceCell<Arc<UiStateManager>> = OnceCell::new();

/// 获取全局 UI 状态管理器实例
pub fn get_ui_state_manager() -> Result<Arc<UiStateManager>, String> {
    let manager = UI_STATE_MANAGER.get_or_try_init(|| UiStateManager::new().map(Arc::new))?;
    Ok(Arc::clone(manager))
}
