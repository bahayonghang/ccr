//! UI 状态命令 — 收藏、最近项目的 CRUD。
//!
//! 使用 ccr-db 的 UiStateManager（SQLite-backed）。

use ccr_db::managers::ui_state::get_ui_state_manager;
use ccr_db::models::ui_state::{AddFavoriteRequest, CommandHistory, FavoriteCommand};

async fn run_blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|e| format!("Blocking task failed: {e}"))?
}

// ── 收藏命令 ──

#[tauri::command]
pub async fn get_favorites() -> Result<Vec<FavoriteCommand>, String> {
    run_blocking(|| {
        let manager =
            get_ui_state_manager().map_err(|e| format!("Failed to get UI state manager: {e}"))?;
        Ok(manager.get_favorites())
    })
    .await
}

#[tauri::command]
pub async fn add_favorite(
    command: String,
    args: Vec<String>,
    display_name: Option<String>,
    module: String,
) -> Result<FavoriteCommand, String> {
    let req = AddFavoriteRequest {
        command,
        args,
        display_name,
        module,
    };

    run_blocking(move || {
        let manager =
            get_ui_state_manager().map_err(|e| format!("Failed to get UI state manager: {e}"))?;
        manager
            .add_favorite(req)
            .map_err(|e| format!("Failed to add favorite: {e}"))
    })
    .await
}

#[tauri::command]
pub async fn remove_favorite(id: String) -> Result<bool, String> {
    run_blocking(move || {
        let manager =
            get_ui_state_manager().map_err(|e| format!("Failed to get UI state manager: {e}"))?;
        manager
            .remove_favorite(&id)
            .map_err(|e| format!("Failed to remove favorite: {e}"))
    })
    .await
}

// ── 命令历史 ──

#[tauri::command]
pub async fn get_recent_items(limit: Option<usize>) -> Result<Vec<CommandHistory>, String> {
    run_blocking(move || {
        let manager =
            get_ui_state_manager().map_err(|e| format!("Failed to get UI state manager: {e}"))?;
        Ok(manager.get_history(limit))
    })
    .await
}

#[tauri::command]
pub async fn add_recent_item(
    command: String,
    args: Vec<String>,
    success: bool,
    duration_ms: u64,
) -> Result<CommandHistory, String> {
    run_blocking(move || {
        let manager =
            get_ui_state_manager().map_err(|e| format!("Failed to get UI state manager: {e}"))?;
        manager
            .add_history(command, args, success, duration_ms)
            .map_err(|e| format!("Failed to add history: {e}"))
    })
    .await
}

#[tauri::command]
pub async fn clear_recent_items() -> Result<String, String> {
    run_blocking(|| {
        let manager =
            get_ui_state_manager().map_err(|e| format!("Failed to get UI state manager: {e}"))?;
        manager
            .clear_history()
            .map_err(|e| format!("Failed to clear history: {e}"))?;
        Ok("History cleared successfully".to_string())
    })
    .await
}
