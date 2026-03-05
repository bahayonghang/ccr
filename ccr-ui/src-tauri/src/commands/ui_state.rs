//! UI 状态命令 — 收藏、最近项目的 CRUD。
//!
//! 使用 ccr-db 的 UiStateManager（SQLite-backed）。

use ccr_db::managers::ui_state::get_ui_state_manager;
use ccr_db::models::ui_state::{AddFavoriteRequest, CommandHistory, FavoriteCommand};

// ── 收藏命令 ──

#[tauri::command]
pub async fn get_favorites() -> Result<Vec<FavoriteCommand>, String> {
    let manager = get_ui_state_manager()
        .await
        .map_err(|e| format!("Failed to get UI state manager: {e}"))?;
    Ok(manager.get_favorites().await)
}

#[tauri::command]
pub async fn add_favorite(
    command: String,
    args: Vec<String>,
    display_name: Option<String>,
    module: String,
) -> Result<FavoriteCommand, String> {
    let manager = get_ui_state_manager()
        .await
        .map_err(|e| format!("Failed to get UI state manager: {e}"))?;

    let req = AddFavoriteRequest {
        command,
        args,
        display_name,
        module,
    };

    manager
        .add_favorite(req)
        .await
        .map_err(|e| format!("Failed to add favorite: {e}"))
}

#[tauri::command]
pub async fn remove_favorite(id: String) -> Result<bool, String> {
    let manager = get_ui_state_manager()
        .await
        .map_err(|e| format!("Failed to get UI state manager: {e}"))?;

    manager
        .remove_favorite(&id)
        .await
        .map_err(|e| format!("Failed to remove favorite: {e}"))
}

// ── 命令历史 ──

#[tauri::command]
pub async fn get_recent_items(limit: Option<usize>) -> Result<Vec<CommandHistory>, String> {
    let manager = get_ui_state_manager()
        .await
        .map_err(|e| format!("Failed to get UI state manager: {e}"))?;
    Ok(manager.get_history(limit).await)
}

#[tauri::command]
pub async fn add_recent_item(
    command: String,
    args: Vec<String>,
    success: bool,
    duration_ms: u64,
) -> Result<CommandHistory, String> {
    let manager = get_ui_state_manager()
        .await
        .map_err(|e| format!("Failed to get UI state manager: {e}"))?;

    manager
        .add_history(command, args, success, duration_ms)
        .await
        .map_err(|e| format!("Failed to add history: {e}"))
}

#[tauri::command]
pub async fn clear_recent_items() -> Result<String, String> {
    let manager = get_ui_state_manager()
        .await
        .map_err(|e| format!("Failed to get UI state manager: {e}"))?;

    manager
        .clear_history()
        .await
        .map_err(|e| format!("Failed to clear history: {e}"))?;

    Ok("History cleared successfully".to_string())
}
