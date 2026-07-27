//! UI 状态命令 — 收藏、最近项目的 CRUD。
//!
//! 使用 ccr-db 的 UiStateManager（SQLite-backed）。

use ccr_db::managers::ui_state::get_ui_state_manager;
use ccr_db::models::ui_state::{AddFavoriteRequest, CommandHistory, FavoriteCommand};
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/ui_state/")]
pub struct FavoriteCommandDto {
    pub id: String,
    pub command: String,
    pub args: Vec<String>,
    pub display_name: Option<String>,
    pub module: String,
    pub created_at: String,
}

impl From<FavoriteCommand> for FavoriteCommandDto {
    fn from(value: FavoriteCommand) -> Self {
        Self {
            id: value.id,
            command: value.command,
            args: value.args,
            display_name: value.display_name,
            module: value.module,
            created_at: value.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/ui_state/")]
pub struct CommandHistoryDto {
    pub id: String,
    pub full_command: String,
    pub command: String,
    pub args: Vec<String>,
    pub success: bool,
    pub executed_at: String,
    #[ts(as = "f64")]
    pub duration_ms: u64,
}

impl From<CommandHistory> for CommandHistoryDto {
    fn from(value: CommandHistory) -> Self {
        Self {
            id: value.id,
            full_command: value.full_command,
            command: value.command,
            args: value.args,
            success: value.success,
            executed_at: value.executed_at.to_rfc3339(),
            duration_ms: value.duration_ms,
        }
    }
}

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

#[ccr_tauri_command_macros::command]
pub async fn get_favorites() -> Result<Vec<FavoriteCommandDto>, String> {
    run_blocking(|| {
        let manager =
            get_ui_state_manager().map_err(|e| format!("Failed to get UI state manager: {e}"))?;
        Ok(manager
            .get_favorites()
            .into_iter()
            .map(FavoriteCommandDto::from)
            .collect())
    })
    .await
}

#[ccr_tauri_command_macros::command]
pub async fn add_favorite(
    command: String,
    args: Vec<String>,
    display_name: Option<String>,
    module: String,
) -> Result<FavoriteCommandDto, String> {
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
            .map(FavoriteCommandDto::from)
    })
    .await
}

#[ccr_tauri_command_macros::command]
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

#[ccr_tauri_command_macros::command]
pub async fn get_recent_items(limit: Option<usize>) -> Result<Vec<CommandHistoryDto>, String> {
    run_blocking(move || {
        let manager =
            get_ui_state_manager().map_err(|e| format!("Failed to get UI state manager: {e}"))?;
        Ok(manager
            .get_history(limit)
            .into_iter()
            .map(CommandHistoryDto::from)
            .collect())
    })
    .await
}

#[ccr_tauri_command_macros::command]
pub async fn add_recent_item(
    command: String,
    args: Vec<String>,
    success: bool,
    duration_ms: u64,
) -> Result<CommandHistoryDto, String> {
    run_blocking(move || {
        let manager =
            get_ui_state_manager().map_err(|e| format!("Failed to get UI state manager: {e}"))?;
        manager
            .add_history(command, args, success, duration_ms)
            .map_err(|e| format!("Failed to add history: {e}"))
            .map(CommandHistoryDto::from)
    })
    .await
}

#[ccr_tauri_command_macros::command]
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
