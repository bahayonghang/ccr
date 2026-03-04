//! 环境管理命令 — 执行环境列表、切换、刷新。
//!
//! 通过 AppState 中的 EnvironmentRegistry 管理 Local/WSL/SSH 环境。

use std::sync::Arc;

use serde_json::Value;
use tauri::{Emitter, Manager, State};

use ccr_db::database::repositories::ssh_repo;

use crate::events::{AppEvent, EnvironmentEventPayload};
use crate::platform::EnvironmentInfo;
use crate::platform::local::LocalEnvironment;
use crate::platform::ssh::{SshEnvironment, SshHostConfig};
#[cfg(target_os = "windows")]
use crate::platform::wsl::{WslEnvironment, detect_wsl_distros};
use crate::state::AppState;

/// 列出所有已注册的执行环境
#[tauri::command]
pub async fn list_environments(state: State<'_, AppState>) -> Result<Vec<EnvironmentInfo>, String> {
    let registry = state.env_registry.read().await;
    Ok(registry.list())
}

/// 获取当前活跃环境
#[tauri::command]
pub async fn get_current_environment(
    state: State<'_, AppState>,
) -> Result<EnvironmentInfo, String> {
    let registry = state.env_registry.read().await;
    let envs = registry.list();
    envs.into_iter()
        .find(|e| e.is_active)
        .ok_or_else(|| "No active environment".to_string())
}

/// 切换活跃环境（按 ID）
#[tauri::command]
pub async fn switch_environment(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    env_id: String,
) -> Result<EnvironmentInfo, String> {
    let mut registry = state.env_registry.write().await;
    registry
        .switch_by_id(&env_id)
        .map_err(|e| format!("Failed to switch environment: {e}"))?;

    let active_env = registry
        .list()
        .into_iter()
        .find(|e| e.is_active)
        .ok_or_else(|| "Switch succeeded but no active environment found".to_string())?;

    drop(registry);

    // 广播环境切换事件（失败仅记录，不影响主流程）
    let payload = EnvironmentEventPayload {
        env_id: active_env.id.clone(),
        env_type: format!("{:?}", active_env.env_type).to_lowercase(),
        status: "active".to_string(),
    };
    if let Err(e) = app_handle.emit("env:changed", payload.clone()) {
        tracing::debug!("[environment] emit env:changed failed: {e}");
    }
    let state_ref = app_handle.state::<AppState>();
    state_ref
        .event_log
        .push(AppEvent::EnvironmentChanged(payload))
        .await;

    Ok(active_env)
}

/// 刷新环境列表 — 重新检测可用的 WSL/SSH 环境
#[tauri::command]
pub async fn refresh_environments(
    state: State<'_, AppState>,
) -> Result<Vec<EnvironmentInfo>, String> {
    let mut registry = state.env_registry.write().await;
    let current_active_id = registry.active().map(|env| env.env_id());

    registry.clear();
    registry.register(Arc::new(LocalEnvironment::new()));

    #[cfg(target_os = "windows")]
    {
        match tokio::task::spawn_blocking(detect_wsl_distros).await {
            Ok(Ok(distros)) => {
                for distro in distros {
                    registry.register(Arc::new(WslEnvironment::new(distro)));
                }
            }
            Ok(Err(e)) => {
                tracing::debug!("[environment] WSL refresh skipped: {e}");
            }
            Err(e) => {
                tracing::debug!("[environment] WSL refresh task failed: {e}");
            }
        }
    }

    let db_pool = state.db_pool.clone();
    match tokio::task::spawn_blocking(move || {
        let conn = db_pool
            .get()
            .map_err(|e| format!("获取数据库连接失败: {e}"))?;
        ssh_repo::list_hosts(&conn).map_err(|e| format!("读取 SSH 主机失败: {e}"))
    })
    .await
    {
        Ok(Ok(hosts)) => {
            for host in hosts {
                registry.register(Arc::new(SshEnvironment::new(SshHostConfig {
                    id: Some(host.id),
                    name: Some(host.name).filter(|v| !v.trim().is_empty()),
                    host: host.host,
                    port: Some(host.port),
                    user: Some(host.username).filter(|v| !v.trim().is_empty()),
                    identity_file: host.identity_file,
                    remote_home: host.remote_home,
                })));
            }
        }
        Ok(Err(e)) => {
            tracing::warn!("[environment] SSH hosts refresh failed: {e}");
        }
        Err(e) => {
            tracing::warn!("[environment] SSH hosts refresh task failed: {e}");
        }
    }

    if let Some(active_id) = current_active_id {
        let _ = registry.switch_by_id(&active_id);
    }

    Ok(registry.list())
}

/// 通过当前活跃环境列出平台
#[tauri::command]
pub async fn env_list_platforms(state: State<'_, AppState>) -> Result<Value, String> {
    let registry = state.env_registry.read().await;
    let env = registry
        .active()
        .ok_or_else(|| "No active environment".to_string())?;
    drop(registry);

    let platforms = env
        .list_platforms()
        .await
        .map_err(|e| format!("Failed to list platforms: {e}"))?;

    serde_json::to_value(&platforms).map_err(|e| format!("Serialization error: {e}"))
}

/// 通过当前活跃环境检测 CLI 工具状态
#[tauri::command]
pub async fn env_detect_cli(state: State<'_, AppState>) -> Result<Value, String> {
    let registry = state.env_registry.read().await;
    let env = registry
        .active()
        .ok_or_else(|| "No active environment".to_string())?;
    drop(registry);

    let status = env
        .detect_cli_status()
        .await
        .map_err(|e| format!("Failed to detect CLI status: {e}"))?;

    serde_json::to_value(&status).map_err(|e| format!("Serialization error: {e}"))
}
