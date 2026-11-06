//! Desktop shell preferences and the shared external application bridge.
use crate::desktop_shell;
use crate::state::{AppState, DesktopShellPreferences, TrayPanelManualPosition};
use serde::Serialize;
use tauri::{AppHandle, State};
use ts_rs::TS;

/// `skillport` 检测结果。
#[derive(Debug, Clone, Serialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/shell/")]
pub struct SkillportAppStatus {
    pub supported: bool,
    pub installed: bool,
    pub platform: String,
    pub source: String,
}

pub type SkillsManageAppStatus = SkillportAppStatus;

/*
 * ========================================================================
 * 步骤1：读取桌面壳层偏好
 * ========================================================================
 * 目标：
 * 1) 返回当前桌面壳层偏好快照
 * 2) 保持前端设置页读取路径稳定
 */
#[ccr_tauri_command_macros::command]
pub async fn shell_get_preferences(
    state: State<'_, AppState>,
) -> Result<DesktopShellPreferences, String> {
    Ok(state.desktop_shell_preferences())
}

/*
 * ========================================================================
 * 步骤2：更新桌面壳层偏好
 * ========================================================================
 * 目标：
 * 1) 写入新的桌面壳层偏好
 * 2) 刷新依赖偏好的 tray 展示
 */
#[ccr_tauri_command_macros::command]
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

#[ccr_tauri_command_macros::command]
pub async fn shell_show_main_window(
    app: AppHandle,
    target_route: Option<String>,
) -> Result<(), String> {
    desktop_shell::show_main_window(&app, target_route.as_deref()).await
}

#[ccr_tauri_command_macros::command]
pub async fn shell_request_quit(app: AppHandle) -> Result<(), String> {
    desktop_shell::request_quit(&app)
}

#[ccr_tauri_command_macros::command]
pub async fn shell_begin_tray_panel_drag(state: State<'_, AppState>) -> Result<(), String> {
    state.set_tray_panel_drag_active(true);
    Ok(())
}

#[ccr_tauri_command_macros::command]
pub async fn shell_complete_tray_panel_drag(
    state: State<'_, AppState>,
    position: Option<TrayPanelManualPosition>,
) -> Result<(), String> {
    state.set_tray_panel_drag_active(false);

    if let Some(position) = position {
        state.set_tray_panel_manual_position(position)?;
    }

    Ok(())
}

/*
 * ========================================================================
 * 步骤3：探测 skillport 安装状态
 * ========================================================================
 * 目标：
 * 1) 在不暴露本机路径给前端的前提下返回安装状态
 * 2) 为 `/skills` 迁移页提供稳定三态来源
 */
#[ccr_tauri_command_macros::command]
pub async fn shell_detect_skills_manage_app() -> Result<SkillsManageAppStatus, String> {
    shell_detect_skillport_app().await
}

/*
 * ========================================================================
 * 步骤3：探测 skillport 安装状态
 * ========================================================================
 * 目标：
 * 1) 在不暴露本机路径给前端的前提下返回安装状态
 * 2) 为 `/skills` 迁移页提供稳定三态来源
 */
#[ccr_tauri_command_macros::command]
pub async fn shell_detect_skillport_app() -> Result<SkillportAppStatus, String> {
    tracing::info!("[shell] 开始探测 skillport 安装状态");

    let status = ccr_app::tools::bridge::detect().await;

    tracing::info!(
        supported = status.supported,
        installed = status.installed,
        platform = %status.platform,
        source = %status.source,
        "[shell] skillport 安装状态探测完成"
    );

    Ok(SkillportAppStatus {
        supported: status.supported,
        installed: status.installed,
        platform: status.platform,
        source: status.source,
    })
}

/*
 * ========================================================================
 * 步骤4：打开 skillport
 * ========================================================================
 * 目标：
 * 1) 按平台重新探测后拉起独立应用
 * 2) 避免前端状态过期导致的盲开
 */
#[ccr_tauri_command_macros::command]
pub async fn shell_open_skills_manage_app() -> Result<(), String> {
    shell_open_skillport_app().await
}

/*
 * ========================================================================
 * 步骤4：打开 skillport
 * ========================================================================
 * 目标：
 * 1) 按平台重新探测后拉起独立应用
 * 2) 避免前端状态过期导致的盲开
 */
#[ccr_tauri_command_macros::command]
pub async fn shell_open_skillport_app() -> Result<(), String> {
    tracing::info!("[shell] 开始打开 skillport");

    ccr_app::tools::bridge::open().await?;

    tracing::info!("[shell] 打开 skillport 请求已发出");
    Ok(())
}
