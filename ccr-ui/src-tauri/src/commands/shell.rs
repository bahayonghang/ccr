//! 桌面壳层命令 —— tray 偏好、主窗口聚焦与显式退出。

use tauri::{AppHandle, State};

use crate::desktop_shell;
use crate::state::{AppState, DesktopShellPreferences, TrayPanelManualPosition};

#[tauri::command]
pub async fn shell_get_preferences(
    state: State<'_, AppState>,
) -> Result<DesktopShellPreferences, String> {
    Ok(state.desktop_shell_preferences())
}

#[tauri::command]
pub async fn shell_set_preferences(
    state: State<'_, AppState>,
    app: AppHandle,
    preferences: DesktopShellPreferences,
) -> Result<DesktopShellPreferences, String> {
    let updated = state.update_desktop_shell_preferences(|current| {
        *current = preferences.clone();
    })?;

    let refresh_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = desktop_shell::refresh_codex_tray(&refresh_handle, false).await {
            tracing::debug!("[shell] tray refresh skipped after preferences update: {error}");
        }
    });

    Ok(updated)
}

#[tauri::command]
pub async fn shell_show_main_window(
    app: AppHandle,
    target_route: Option<String>,
) -> Result<(), String> {
    desktop_shell::show_main_window(&app, target_route.as_deref()).await
}

#[tauri::command]
pub async fn shell_request_quit(app: AppHandle) -> Result<(), String> {
    desktop_shell::request_quit(&app)
}

#[tauri::command]
pub fn shell_begin_tray_panel_drag(state: State<'_, AppState>) -> Result<(), String> {
    state.set_tray_panel_drag_active(true);
    Ok(())
}

#[tauri::command]
pub fn shell_complete_tray_panel_drag(
    state: State<'_, AppState>,
    position: Option<TrayPanelManualPosition>,
) -> Result<(), String> {
    state.set_tray_panel_drag_active(false);

    if let Some(position) = position {
        state.set_tray_panel_manual_position(position)?;
    }

    Ok(())
}
