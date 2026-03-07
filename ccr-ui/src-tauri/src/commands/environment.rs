//! 閻滎垰顣ㄧ粻锛勬倞閸涙垝鎶?閳?閹笛嗩攽閻滎垰顣ㄩ崚妤勩€冮妴浣稿瀼閹诡潿鈧礁鍩涢弬鑸偓?
//!
//! 闁俺绻?AppState 娑擃厾娈?EnvironmentRegistry 缁狅紕鎮?Local/WSL/SSH 閻滎垰顣ㄩ妴?

use std::sync::Arc;

use serde_json::Value;
use tauri::State;

use ccr_db::database::repositories::ssh_repo;

use crate::events::{self, EnvironmentEventPayload};
use crate::monitoring::{emit_and_record_monitoring_event, environment_changed_entry, should_persist};
use crate::platform::EnvironmentInfo;
use crate::platform::local::LocalEnvironment;
use crate::platform::ssh::{SshEnvironment, SshHostConfig};
#[cfg(target_os = "windows")]
use crate::platform::wsl::{WslEnvironment, detect_wsl_distros_with_cache};
use crate::state::AppState;

/// 閸掓鍤幍鈧張澶婂嚒濞夈劌鍞介惃鍕⒔鐞涘瞼骞嗘晶?
#[tauri::command]
pub async fn list_environments(state: State<'_, AppState>) -> Result<Vec<EnvironmentInfo>, String> {
    let registry = state.env_registry.read().await;
    Ok(registry.list())
}

/// 閼惧嘲褰囪ぐ鎾冲濞叉槒绌悳顖氼暔
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

/// 閸掑洦宕插ú鏄忕┈閻滎垰顣ㄩ敍鍫熷瘻 ID閿?
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

    // 楠炴寧鎸遍悳顖氼暔閸掑洦宕叉禍瀣╂閿涘牆銇戠拹銉ょ矌鐠佹澘缍嶉敍灞肩瑝瑜板崬鎼锋稉缁樼ウ缁嬪绱?
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

/// 閸掗攱鏌婇悳顖氼暔閸掓銆?閳?闁插秵鏌婂Λ鈧ù瀣讲閻劎娈?WSL/SSH 閻滎垰顣?
///
/// # 閸欏倹鏆?
/// - `force_refresh`: 閺勵垰鎯佸鍝勫煑閸掗攱鏌?WSL 缂傛挸鐡?
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
            .map_err(|e| format!("閼惧嘲褰囬弫鐗堝祦鎼存捁绻涢幒銉ャ亼鐠? {e}"))?;
        ssh_repo::list_hosts(&conn).map_err(|e| format!("鐠囪褰?SSH 娑撶粯婧€婢惰精瑙? {e}"))
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

/// 闁俺绻冭ぐ鎾冲濞叉槒绌悳顖氼暔閸掓鍤獮鍐插酱
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

/// 闁俺绻冭ぐ鎾冲濞叉槒绌悳顖氼暔濡偓濞?CLI 瀹搞儱鍙块悩鑸碘偓?
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

