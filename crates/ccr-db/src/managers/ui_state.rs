// UI State Manager - 管理收藏和历史记录的持久化
// Migrated from JSON file storage to unified SQLite database

use crate::core::error::DbError;
use crate::database::repositories::ui_state_repo;
use crate::database::{self, DbAccess};
use crate::models::ui_state::{AddFavoriteRequest, CommandHistory, FavoriteCommand};
use chrono::Utc;
use tracing::{debug, error, info};
use uuid::Uuid;

/// UI 状态管理器 - 使用 SQLite 存储
pub struct UiStateManager {
    db: DbAccess,
}

impl UiStateManager {
    /// 创建新的管理器实例
    pub fn new() -> Result<Self, String> {
        // Ensure database is initialized
        if !database::is_initialized() {
            return Err("Database not initialized".to_string());
        }
        info!("UiStateManager initialized with SQLite backend");
        Ok(Self::with_db(DbAccess::Global))
    }

    /// Inject an existing domain pool; does not initialize or access the database.
    pub fn with_db(db: DbAccess) -> Self {
        Self { db }
    }

    /// Native consumers retain query failures instead of treating them as empty data.
    pub fn try_get_favorites(&self) -> Result<Vec<FavoriteCommand>, DbError> {
        self.db.with_connection(ui_state_repo::get_all_favorites)
    }

    pub fn try_get_history(&self, limit: Option<usize>) -> Result<Vec<CommandHistory>, DbError> {
        self.db
            .with_connection(|conn| ui_state_repo::get_history(conn, limit.unwrap_or(50)))
    }

    // ===== 收藏命令操作 =====

    /// 获取所有收藏命令
    pub fn get_favorites(&self) -> Vec<FavoriteCommand> {
        match self.try_get_favorites() {
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
        let existing = self
            .db
            .with_connection(|conn| {
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

        self.db
            .with_connection(|conn| ui_state_repo::insert_favorite(conn, &favorite))
            .map_err(|e| format!("Failed to insert favorite: {}", e))?;

        info!("Added favorite: {}", favorite.id);
        Ok(favorite)
    }

    /// 删除收藏命令
    pub fn remove_favorite(&self, id: &str) -> Result<bool, String> {
        let deleted = self
            .db
            .with_connection(|conn| ui_state_repo::delete_favorite(conn, id))
            .map_err(|e| format!("Database error: {}", e))?;

        if deleted {
            info!("Removed favorite: {}", id);
        }
        Ok(deleted)
    }

    // ===== 命令历史操作 =====

    /// 获取命令历史
    pub fn get_history(&self, limit: Option<usize>) -> Vec<CommandHistory> {
        match self.try_get_history(limit) {
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

        self.db
            .with_connection(|conn| ui_state_repo::insert_history(conn, &history))
            .map_err(|e| format!("Failed to insert history: {}", e))?;

        debug!("Added history: {}", history.id);
        Ok(history)
    }

    /// 清空命令历史
    pub fn clear_history(&self) -> Result<(), String> {
        let count = self
            .db
            .with_connection(ui_state_repo::clear_history)
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::path::Path;

    fn manager(root: &Path) -> UiStateManager {
        let home = root.join("empty-home");
        std::fs::create_dir_all(&home).unwrap();
        let pool = database::create_pool(&root.join("state.db"), None).unwrap();
        {
            let conn = pool.get().unwrap();
            database::migrations::run_all_migrations(&conn, &home).unwrap();
        }
        UiStateManager::with_db(DbAccess::Pool(pool))
    }

    fn request(command: &str) -> AddFavoriteRequest {
        AddFavoriteRequest {
            command: command.into(),
            args: vec!["--verbose".into()],
            display_name: None,
            module: "config".into(),
        }
    }

    #[test]
    fn injected_ui_state_persists_and_isolates_parallel_fixtures() {
        let a = tempfile::TempDir::new().unwrap();
        let b = tempfile::TempDir::new().unwrap();
        let left = manager(a.path());
        let right = manager(b.path());
        assert!(left.try_get_favorites().unwrap().is_empty());
        let first = left.add_favorite(request("list")).unwrap();
        let duplicate = left.add_favorite(request("list")).unwrap();
        assert_eq!(first.id, duplicate.id);
        assert!(duplicate.display_name.is_none());
        assert_eq!(duplicate.args, ["--verbose"]);
        let second = std::thread::scope(|scope| {
            let other = scope.spawn(|| right.add_favorite(request("status")).unwrap());
            let item = left.add_favorite(request("current")).unwrap();
            assert_eq!(other.join().unwrap().command, "status");
            item
        });
        let rows = left.try_get_favorites().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, second.id);
        assert_eq!(right.try_get_favorites().unwrap().len(), 1);
        left.add_history("list".into(), vec![], true, 8).unwrap();
        left.add_history("current".into(), vec!["x".into()], false, 12)
            .unwrap();
        let latest = left.try_get_history(Some(1)).unwrap();
        assert_eq!(latest.len(), 1);
        assert_eq!(latest[0].full_command, "ccr current x");
        assert!(!latest[0].success);
        assert_eq!(latest[0].duration_ms, 12);
        assert!(right.try_get_history(None).unwrap().is_empty());
        drop(left);
        let reopened = manager(a.path());
        assert_eq!(reopened.try_get_favorites().unwrap()[1].id, first.id);
        assert_eq!(reopened.try_get_history(None).unwrap().len(), 2);
        assert!(reopened.remove_favorite(&first.id).unwrap());
        assert!(!reopened.remove_favorite(&first.id).unwrap());
        reopened.clear_history().unwrap();
        assert!(reopened.try_get_history(None).unwrap().is_empty());
        for index in 0..55 {
            reopened
                .add_history(format!("fixture-{index}"), vec![], true, 1)
                .unwrap();
        }
        assert_eq!(reopened.try_get_history(None).unwrap().len(), 50);
        assert_eq!(reopened.try_get_history(Some(75)).unwrap().len(), 55);
        assert!(reopened.try_get_history(Some(0)).unwrap().is_empty());
        reopened.clear_history().unwrap();
    }

    #[test]
    fn injected_ui_state_keeps_query_failure_distinct_from_empty() {
        let pool = database::create_memory_pool().unwrap();
        let manager = UiStateManager::with_db(DbAccess::Pool(pool));
        assert!(matches!(
            manager.try_get_favorites(),
            Err(DbError::Query(_))
        ));
        assert!(matches!(
            manager.try_get_history(None),
            Err(DbError::Query(_))
        ));
        // Compatibility methods keep their existing empty-on-error behavior.
        assert!(manager.get_favorites().is_empty());
        assert!(manager.get_history(None).is_empty());
    }
}
