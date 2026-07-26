//! SSH 命令 — 主机管理、连接切换、配置读写、CLI 检测。

use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{Emitter, State};

use ccr_db::database::repositories::ssh_repo;
use ccr_db::models::ssh::{SshHost, SshKnownHost};

use crate::platform::EnvironmentRegistry;
use crate::platform::local::LocalEnvironment;
use crate::platform::ssh::{SshEnvironment, SshHostConfig};
use crate::ssh::connection::SshConnectResult;
use crate::ssh::security::{
    HostKeyStatus, ScannedHostKey, SshTarget, SshTrustService, app_known_hosts_path,
    classify_host_key, parse_keyscan_output, persist_known_host, run_openssh_command,
};
use crate::ssh::{auth, auth::SshKeyInfo, connection::SshConnectionManager, sftp};
use crate::state::AppState;

fn db_host_to_config(host: SshHost) -> SshHostConfig {
    SshHostConfig {
        id: Some(host.id),
        name: Some(host.name).filter(|v| !v.trim().is_empty()),
        host: host.host,
        port: Some(host.port),
        user: Some(host.username).filter(|v| !v.trim().is_empty()),
        identity_file: host.identity_file,
        remote_home: host.remote_home,
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddSshHostRequest {
    pub id: Option<String>,
    pub name: Option<String>,
    pub host: String,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub identity_file: Option<String>,
    pub remote_home: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SshConnectionState {
    pub env_id: String,
    pub connected: bool,
    pub has_password: bool,
    pub last_checked_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SshProbeFingerprintRequest {
    pub env_id: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SshFingerprintProbeResult {
    pub challenge_id: String,
    pub host: String,
    pub port: u16,
    pub key_type: String,
    pub public_key: String,
    pub fingerprint: String,
    pub status: HostKeyStatus,
    pub stored_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SshConfirmFingerprintRequest {
    pub challenge_id: String,
}

#[derive(Debug)]
struct ProbeTarget {
    host: String,
    port: u16,
}

async fn resolve_probe_target(
    state: &AppState,
    req: &SshProbeFingerprintRequest,
) -> Result<ProbeTarget, String> {
    if let Some(host) = req.host.clone().filter(|v| !v.trim().is_empty()) {
        let target = SshTarget::new(&host, req.port.unwrap_or(22), None, None)?;
        return Ok(ProbeTarget {
            host: target.host().to_string(),
            port: target.port(),
        });
    }

    let env_id = req
        .env_id
        .clone()
        .ok_or_else(|| "请提供 env_id 或 host".to_string())?;
    let host_id = env_id
        .strip_prefix("ssh:")
        .ok_or_else(|| format!("无效 SSH 环境 ID: {env_id}"))?
        .to_string();

    let db_pool = state.db_pool.clone();
    let host = tokio::task::spawn_blocking(move || {
        let conn = db_pool
            .get()
            .map_err(|e| format!("获取数据库连接失败: {e}"))?;
        ssh_repo::get_host(&conn, &host_id).map_err(|e| format!("读取 SSH 主机失败: {e}"))
    })
    .await
    .map_err(|e| format!("读取 SSH 主机任务失败: {e}"))??
    .ok_or_else(|| format!("SSH 主机不存在: {env_id}"))?;

    let target = SshTarget::new(&host.host, host.port, None, None)?;
    Ok(ProbeTarget {
        host: target.host().to_string(),
        port: target.port(),
    })
}

async fn collect_host_key(host: &str, port: u16) -> Result<ScannedHostKey, String> {
    let target = SshTarget::new(host, port, None, None)?;
    let mut keyscan_cmd = crate::process::tokio_command("ssh-keyscan");
    keyscan_cmd
        .arg("-T")
        .arg("5")
        .arg("-p")
        .arg(port.to_string())
        .arg("-t")
        .arg("ed25519,ecdsa,rsa")
        .arg(target.host());

    let keyscan_output = run_openssh_command(keyscan_cmd, None).await?;

    if !keyscan_output.status.success() {
        return Err(format!(
            "ssh-keyscan 返回失败: {}",
            String::from_utf8_lossy(&keyscan_output.stderr)
        ));
    }

    parse_keyscan_output(
        &String::from_utf8_lossy(&keyscan_output.stdout),
        target.host(),
        target.port(),
    )
}

async fn connect_internal(
    state: &AppState,
    env_id: String,
    password: Option<String>,
) -> Result<SshConnectionState, String> {
    let has_password_input = password
        .as_ref()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);

    let host_id = env_id
        .strip_prefix("ssh:")
        .ok_or_else(|| format!("无效 SSH 环境 ID: {env_id}"))?
        .to_string();
    let db_pool = state.db_pool.clone();
    let host_id_for_load = host_id.clone();
    let host = tokio::task::spawn_blocking(move || {
        let conn = db_pool
            .get()
            .map_err(|e| format!("获取数据库连接失败: {e}"))?;
        ssh_repo::get_host(&conn, &host_id_for_load).map_err(|e| format!("读取 SSH 主机失败: {e}"))
    })
    .await
    .map_err(|e| format!("读取 SSH 主机任务失败: {e}"))??
    .ok_or_else(|| format!("SSH 主机不存在: {env_id}"))?;

    let config = db_host_to_config(host);
    if let Err(error) = config.validate() {
        deactivate_failed_connection(state, &env_id, &error).await?;
        return Err(error);
    }
    let connectivity = match SshConnectionManager::test_connectivity(&config).await {
        Ok(connectivity) => connectivity,
        Err(error) => {
            deactivate_failed_connection(state, &env_id, &error).await?;
            return Err(error);
        }
    };
    if !connectivity.success {
        let code = connectivity
            .error_code
            .as_deref()
            .unwrap_or("ssh_network_error");
        let detail = connectivity.error.as_deref().unwrap_or(code);
        let error = if detail.starts_with(code) {
            detail.to_string()
        } else {
            format!("{code}: {detail}")
        };
        deactivate_failed_connection(state, &env_id, &error).await?;
        return Err(error);
    }

    let db_pool = state.db_pool.clone();
    let update_result = match tokio::task::spawn_blocking(move || {
        let conn = db_pool
            .get()
            .map_err(|e| format!("获取数据库连接失败: {e}"))?;
        ssh_repo::set_last_connected_at(&conn, &host_id, Utc::now())
            .map_err(|e| format!("更新最近连接时间失败: {e}"))
    })
    .await
    {
        Ok(result) => result,
        Err(error) => Err(format!("更新最近连接时间任务失败: {error}")),
    };
    if let Err(error) = update_result {
        deactivate_failed_connection(state, &env_id, &error).await?;
        return Err(error);
    }

    let switch_result = {
        let mut registry = state.env_registry.write().await;
        registry
            .switch_by_id(&env_id)
            .map_err(|e| format!("切换 SSH 环境失败: {e}"))
    };
    if let Err(error) = switch_result {
        deactivate_failed_connection(state, &env_id, &error).await?;
        return Err(error);
    }

    if let Some(pass) = password.filter(|value| !value.trim().is_empty()) {
        SshConnectionManager::cache_password(state, &env_id, pass).await;
    }
    let has_password_cached = SshConnectionManager::has_password(state, &env_id).await;
    let has_password_now = has_password_input || has_password_cached;

    SshConnectionManager::mark_connected(state, env_id.clone(), has_password_now).await;

    Ok(SshConnectionState {
        env_id,
        connected: true,
        has_password: has_password_now,
        last_checked_at: Some(Utc::now().to_rfc3339()),
        last_error: None,
    })
}

async fn deactivate_failed_connection(
    state: &AppState,
    env_id: &str,
    error: &str,
) -> Result<(), String> {
    {
        let mut registry = state.env_registry.write().await;
        deactivate_active_target(&mut registry, env_id)?;
    }

    SshConnectionManager::clear_password(state, env_id).await;
    SshConnectionManager::mark_disconnected(state, env_id.to_string(), Some(error.to_string()))
        .await;
    Ok(())
}

fn deactivate_active_target(
    registry: &mut EnvironmentRegistry,
    env_id: &str,
) -> Result<(), String> {
    let target_is_active = registry
        .active()
        .is_some_and(|environment| environment.env_id() == env_id);
    if !target_is_active {
        return Ok(());
    }

    if !registry
        .list()
        .iter()
        .any(|environment| environment.id == "local")
    {
        registry.register(Arc::new(LocalEnvironment::new()));
    }
    registry
        .switch_by_id("local")
        .map_err(|cause| format!("SSH 连接失败且无法切换到本地环境: {cause}"))
}

#[tauri::command]
pub async fn ssh_list_hosts(state: State<'_, AppState>) -> Result<Vec<SshHostConfig>, String> {
    let db_pool = state.db_pool.clone();

    let hosts = tokio::task::spawn_blocking(move || {
        let conn = db_pool
            .get()
            .map_err(|e| format!("获取数据库连接失败: {e}"))?;
        ssh_repo::list_hosts(&conn).map_err(|e| format!("读取 SSH 主机列表失败: {e}"))
    })
    .await
    .map_err(|e| format!("读取 SSH 主机列表任务失败: {e}"))??;

    Ok(hosts.into_iter().map(db_host_to_config).collect())
}

#[tauri::command]
pub async fn ssh_add_host(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    host: AddSshHostRequest,
) -> Result<SshHostConfig, String> {
    let config = SshHostConfig {
        id: host.id.or_else(|| Some(host.host.clone())),
        name: host.name,
        host: host.host,
        port: host.port.or(Some(22)),
        user: host.user,
        identity_file: host.identity_file,
        remote_home: host.remote_home,
    };
    config.validate()?;

    let host_id = config
        .id
        .clone()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| config.host.clone());
    let env_id = format!("ssh:{host_id}");

    let db_pool = state.db_pool.clone();
    let config_for_db = config.clone();
    let host_id_for_db = host_id.clone();

    tokio::task::spawn_blocking(move || {
        let conn = db_pool
            .get()
            .map_err(|e| format!("获取数据库连接失败: {e}"))?;

        let existing = ssh_repo::get_host(&conn, &host_id_for_db)
            .map_err(|e| format!("查询 SSH 主机失败: {e}"))?;

        let now = Utc::now();
        let db_host = SshHost {
            id: host_id_for_db.clone(),
            name: config_for_db
                .name
                .clone()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| config_for_db.host.clone()),
            host: config_for_db.host.clone(),
            port: config_for_db.port.unwrap_or(22),
            username: config_for_db
                .user
                .clone()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_default(),
            identity_file: config_for_db.identity_file.clone(),
            remote_home: config_for_db.remote_home.clone(),
            created_at: existing.as_ref().map(|v| v.created_at).unwrap_or(now),
            updated_at: now,
            last_connected_at: existing.and_then(|v| v.last_connected_at),
        };

        if ssh_repo::update_host(&conn, &db_host).map_err(|e| format!("更新 SSH 主机失败: {e}"))?
        {
            Ok(())
        } else {
            ssh_repo::insert_host(&conn, &db_host).map_err(|e| format!("新增 SSH 主机失败: {e}"))
        }
    })
    .await
    .map_err(|e| format!("保存 SSH 主机任务失败: {e}"))??;

    {
        let mut registry = state.env_registry.write().await;
        if let Err(e) = registry.switch_by_id(&env_id) {
            tracing::debug!("[ssh] no existing env to replace before register: {e}");
        }
        registry.register(Arc::new(SshEnvironment::new(config.clone())));
    }

    let _ = app_handle.emit("env:refresh-requested", true);

    Ok(config)
}

#[tauri::command]
pub async fn ssh_connect(
    state: State<'_, AppState>,
    env_id: String,
    password: Option<String>,
) -> Result<SshConnectionState, String> {
    connect_internal(state.inner(), env_id, password).await
}

#[tauri::command]
pub async fn ssh_reconnect(
    state: State<'_, AppState>,
    env_id: String,
    password: Option<String>,
) -> Result<SshConnectionState, String> {
    connect_internal(state.inner(), env_id, password).await
}

#[tauri::command]
pub async fn ssh_disconnect(state: State<'_, AppState>) -> Result<SshConnectionState, String> {
    let mut registry = state.env_registry.write().await;
    let previous = registry.active().map(|env| env.env_id());

    if registry.list().iter().any(|env| env.id == "local") {
        registry
            .switch_by_id("local")
            .map_err(|e| format!("切换到本地环境失败: {e}"))?;
    } else {
        registry.register(Arc::new(LocalEnvironment::new()));
        registry
            .switch_by_id("local")
            .map_err(|e| format!("激活本地环境失败: {e}"))?;
    }

    drop(registry);

    if let Some(prev_env_id) = previous
        && prev_env_id.starts_with("ssh:")
    {
        SshConnectionManager::clear_password(state.inner(), &prev_env_id).await;
        SshConnectionManager::mark_disconnected(state.inner(), prev_env_id, None).await;
    }

    Ok(SshConnectionState {
        env_id: "local".to_string(),
        connected: false,
        has_password: false,
        last_checked_at: Some(Utc::now().to_rfc3339()),
        last_error: None,
    })
}

#[tauri::command]
pub async fn ssh_get_connection_state(
    state: State<'_, AppState>,
    env_id: Option<String>,
) -> Result<Value, String> {
    if let Some(id) = env_id {
        if let Some(item) = SshConnectionManager::get_state(state.inner(), &id).await {
            return serde_json::to_value(item).map_err(|e| format!("序列化连接状态失败: {e}"));
        }

        let fallback = crate::state::SshRuntimeState {
            env_id: id,
            connected: false,
            has_password: false,
            last_checked_at: None,
            last_error: None,
        };
        return serde_json::to_value(fallback).map_err(|e| format!("序列化连接状态失败: {e}"));
    }

    let items = SshConnectionManager::list_states(state.inner()).await;
    serde_json::to_value(items).map_err(|e| format!("序列化连接状态失败: {e}"))
}

#[tauri::command]
pub async fn ssh_probe_host_fingerprint(
    state: State<'_, AppState>,
    trust: State<'_, Arc<SshTrustService>>,
    request: SshProbeFingerprintRequest,
) -> Result<SshFingerprintProbeResult, String> {
    let target = resolve_probe_target(state.inner(), &request).await?;
    let scanned = collect_host_key(&target.host, target.port).await?;

    let host_for_query = target.host.clone();
    let port_for_query = target.port;
    let db_pool = state.db_pool.clone();

    let known = tokio::task::spawn_blocking(move || {
        let conn = db_pool
            .get()
            .map_err(|e| format!("获取数据库连接失败: {e}"))?;
        ssh_repo::get_known_host(&conn, &host_for_query, port_for_query)
            .map_err(|e| format!("读取 known_hosts 失败: {e}"))
    })
    .await
    .map_err(|e| format!("读取 known_hosts 任务失败: {e}"))??;

    let stored_fingerprint = known.map(|known_host| known_host.fingerprint);
    let status = classify_host_key(stored_fingerprint.as_deref(), &scanned.fingerprint);
    let challenge_id = trust.register(target.host.clone(), target.port, scanned.clone());

    if status == HostKeyStatus::Matched {
        persist_known_host(
            app_known_hosts_path()?,
            target.host.clone(),
            target.port,
            scanned.key_type.clone(),
            scanned.key_data.clone(),
        )
        .await?;
    }

    Ok(SshFingerprintProbeResult {
        challenge_id,
        host: target.host,
        port: target.port,
        key_type: scanned.key_type,
        public_key: scanned.key_data,
        fingerprint: scanned.fingerprint,
        status,
        stored_fingerprint,
    })
}

#[tauri::command]
pub async fn ssh_confirm_host_fingerprint(
    state: State<'_, AppState>,
    trust: State<'_, Arc<SshTrustService>>,
    request: SshConfirmFingerprintRequest,
) -> Result<(), String> {
    let challenge = trust.consume(&request.challenge_id)?;
    persist_known_host(
        app_known_hosts_path()?,
        challenge.host.clone(),
        challenge.port,
        challenge.key_type.clone(),
        challenge.key_data.clone(),
    )
    .await?;

    let entry = SshKnownHost {
        host: challenge.host,
        port: challenge.port,
        key_type: challenge.key_type,
        fingerprint: challenge.fingerprint,
        confirmed_at: Utc::now(),
    };

    let db_pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db_pool
            .get()
            .map_err(|e| format!("获取数据库连接失败: {e}"))?;
        ssh_repo::upsert_known_host(&conn, &entry)
            .map_err(|e| format!("写入 known_hosts 失败: {e}"))
    })
    .await
    .map_err(|e| format!("写入 known_hosts 任务失败: {e}"))??;

    Ok(())
}

#[tauri::command]
pub async fn ssh_read_config(
    state: State<'_, AppState>,
    env_id: String,
    platform: String,
    path: String,
) -> Result<String, String> {
    sftp::read_config(state.inner(), &env_id, &platform, &path).await
}

#[tauri::command]
pub async fn ssh_write_config(
    state: State<'_, AppState>,
    env_id: String,
    platform: String,
    path: String,
    content: String,
    enable_backup: Option<bool>,
) -> Result<(), String> {
    sftp::write_config(
        state.inner(),
        &env_id,
        &platform,
        &path,
        &content,
        enable_backup.unwrap_or(true),
    )
    .await
}

#[tauri::command]
pub async fn ssh_detect_cli(state: State<'_, AppState>, env_id: String) -> Result<Value, String> {
    sftp::detect_cli(state.inner(), &env_id).await
}

/// 测试 SSH 连接连通性（从数据库解析主机配置）。
#[tauri::command]
pub async fn ssh_test_connection(
    state: State<'_, AppState>,
    env_id: String,
) -> Result<SshConnectResult, String> {
    let host_id = env_id
        .strip_prefix("ssh:")
        .ok_or_else(|| format!("无效 SSH 环境 ID: {env_id}"))?
        .to_string();

    let db_pool = state.db_pool.clone();
    let host = tokio::task::spawn_blocking(move || {
        let conn = db_pool
            .get()
            .map_err(|e| format!("获取数据库连接失败: {e}"))?;
        ssh_repo::get_host(&conn, &host_id).map_err(|e| format!("读取 SSH 主机失败: {e}"))
    })
    .await
    .map_err(|e| format!("读取 SSH 主机任务失败: {e}"))??
    .ok_or_else(|| format!("SSH 主机不存在: {env_id}"))?;

    SshConnectionManager::test_connectivity(&db_host_to_config(host)).await
}

/// 列出本机 `~/.ssh/` 目录中发现的所有 SSH 私钥文件信息。
#[tauri::command]
pub async fn ssh_list_keys() -> Result<Vec<SshKeyInfo>, String> {
    Ok(auth::discover_keys().await)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ssh_environment(id: &str) -> Arc<SshEnvironment> {
        Arc::new(SshEnvironment::new(SshHostConfig {
            id: Some(id.to_string()),
            name: None,
            host: "example.com".to_string(),
            port: Some(22),
            user: Some("deploy".to_string()),
            identity_file: None,
            remote_home: None,
        }))
    }

    #[test]
    fn failed_active_target_falls_back_to_local() {
        let mut registry = EnvironmentRegistry::new();
        registry.register(Arc::new(LocalEnvironment::new()));
        registry.register(ssh_environment("target"));
        registry.switch_by_id("ssh:target").unwrap();

        deactivate_active_target(&mut registry, "ssh:target").unwrap();

        assert_eq!(registry.active().unwrap().env_id(), "local");
    }

    #[test]
    fn failed_inactive_target_does_not_replace_active_environment() {
        let mut registry = EnvironmentRegistry::new();
        registry.register(Arc::new(LocalEnvironment::new()));
        registry.register(ssh_environment("active"));
        registry.register(ssh_environment("target"));
        registry.switch_by_id("ssh:active").unwrap();

        deactivate_active_target(&mut registry, "ssh:target").unwrap();

        assert_eq!(registry.active().unwrap().env_id(), "ssh:active");
    }
}
