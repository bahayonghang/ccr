//! 环境管理命令模块，负责列出、切换与刷新执行环境。
//!
//! 通过 AppState 中的 EnvironmentRegistry 管理 Local/WSL/SSH 环境。

use std::sync::Arc;

use serde_json::Value;
use tauri::State;

use ccr_db::database::repositories::ssh_repo;

use crate::events::{self, EnvironmentEventPayload};
use crate::monitoring::{
    emit_and_record_monitoring_event, environment_changed_entry, should_persist,
};
use crate::platform::EnvironmentInfo;
use crate::platform::local::LocalEnvironment;
use crate::platform::ssh::{SshEnvironment, SshHostConfig};
#[cfg(target_os = "windows")]
use crate::platform::wsl::{WslEnvironment, detect_wsl_distros_with_cache};
use crate::state::AppState;

/// 列出所有已注册的执行环境。
#[tauri::command]
pub async fn list_environments(state: State<'_, AppState>) -> Result<Vec<EnvironmentInfo>, String> {
    let registry = state.env_registry.read().await;
    Ok(registry.list())
}

/// 获取当前激活的执行环境。
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

/// 按环境 ID 切换当前执行环境。
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

    // 切换完成后广播环境变更事件，并写入监控日志。
    let payload = EnvironmentEventPayload {
        env_id: active_env.id.clone(),
        env_type: format!("{:?}", active_env.env_type).to_lowercase(),
        status: "active".to_string(),
    };
    let entry = environment_changed_entry(&payload);
    let persist = should_persist(entry.level, &entry.event_type);
    emit_and_record_monitoring_event(
        &app_handle,
        events::channels::ENVIRONMENT_CHANGED,
        &payload,
        entry,
        persist,
    )
    .await;

    Ok(active_env)
}

/// 刷新环境列表，并重新发现本地、WSL/SSH 执行环境。
///
/// # 参数
/// - `force_refresh`: 是否强制刷新 WSL 发行版缓存
#[tauri::command]
pub async fn refresh_environments(
    state: State<'_, AppState>,
    force_refresh: Option<bool>,
) -> Result<Vec<EnvironmentInfo>, String> {
    let force = force_refresh.unwrap_or(false);
    #[cfg(not(target_os = "windows"))]
    let _ = force;

    let current_active_id = {
        let registry = state.env_registry.read().await;
        registry.active().map(|env| env.env_id())
    };

    #[cfg(target_os = "windows")]
    let distros =
        match tokio::task::spawn_blocking(move || detect_wsl_distros_with_cache(force)).await {
            Ok(Ok(distros)) => distros,
            Ok(Err(e)) => {
                tracing::debug!("[environment] WSL refresh skipped: {e}");
                Vec::new()
            }
            Err(e) => {
                tracing::debug!("[environment] WSL refresh task failed: {e}");
                Vec::new()
            }
        };

    let db_pool = state.db_pool.clone();
    let hosts = match tokio::task::spawn_blocking(move || {
        let conn = db_pool
            .get()
            .map_err(|e| format!("Failed to get database connection: {e}"))?;
        ssh_repo::list_hosts(&conn).map_err(|e| format!("Failed to list SSH hosts: {e}"))
    })
    .await
    {
        Ok(Ok(hosts)) => hosts,
        Ok(Err(e)) => {
            tracing::warn!("[environment] SSH hosts refresh failed: {e}");
            Vec::new()
        }
        Err(e) => {
            tracing::warn!("[environment] SSH hosts refresh task failed: {e}");
            Vec::new()
        }
    };

    let mut registry = state.env_registry.write().await;
    registry.clear();
    registry.register(Arc::new(LocalEnvironment::new()));

    #[cfg(target_os = "windows")]
    for distro in distros {
        registry.register(Arc::new(WslEnvironment::new(distro)));
    }

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

    if let Some(active_id) = current_active_id {
        let _ = registry.switch_by_id(&active_id);
    }

    Ok(registry.list())
}

/// 获取当前环境支持的平台列表。
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

/// 检测当前环境中的 CLI 状态。
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
