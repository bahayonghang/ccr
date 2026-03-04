//! SSH 命令 — 主机管理、连接切换、配置读写、CLI 检测。

use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{Emitter, State};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

use ccr_db::database::repositories::ssh_repo;
use ccr_db::models::ssh::{SshHost, SshKnownHost};

use crate::platform::local::LocalEnvironment;
use crate::platform::ssh::{SshEnvironment, SshHostConfig};
use crate::ssh::{auth, auth::SshKeyInfo, connection::SshConnectionManager, sftp};
use crate::ssh::connection::SshConnectResult;
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
    pub host: String,
    pub port: u16,
    pub key_type: String,
    pub fingerprint: String,
    pub status: String,
    pub stored_fingerprint: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SshConfirmFingerprintRequest {
    pub host: String,
    pub port: Option<u16>,
    pub key_type: String,
    pub fingerprint: String,
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
        return Ok(ProbeTarget {
            host,
            port: req.port.unwrap_or(22),
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

    Ok(ProbeTarget {
        host: host.host,
        port: host.port,
    })
}

async fn collect_host_fingerprint(host: &str, port: u16) -> Result<(String, String), String> {
    let mut keyscan_cmd = Command::new("ssh-keyscan");
    keyscan_cmd
        .arg("-T")
        .arg("5")
        .arg("-p")
        .arg(port.to_string())
        .arg("-t")
        .arg("ed25519,ecdsa,rsa")
        .arg(host);

    let keyscan_output = timeout(Duration::from_secs(10), keyscan_cmd.output())
        .await
        .map_err(|_| "ssh-keyscan 超时（10 秒）".to_string())?
        .map_err(|e| format!("执行 ssh-keyscan 失败: {e}"))?;

    if !keyscan_output.status.success() {
        return Err(format!(
            "ssh-keyscan 返回失败: {}",
            String::from_utf8_lossy(&keyscan_output.stderr)
        ));
    }

    let scanned = String::from_utf8_lossy(&keyscan_output.stdout);
    let key_line = scanned
        .lines()
        .find(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .ok_or_else(|| "未获取到主机公钥".to_string())?
        .to_string();

    let key_type = key_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("unknown")
        .to_string();

    let mut keygen_cmd = Command::new("ssh-keygen");
    keygen_cmd.arg("-lf").arg("-");
    keygen_cmd.stdin(std::process::Stdio::piped());
    keygen_cmd.stdout(std::process::Stdio::piped());
    keygen_cmd.stderr(std::process::Stdio::piped());

    let mut child = keygen_cmd
        .spawn()
        .map_err(|e| format!("启动 ssh-keygen 失败: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(key_line.as_bytes())
            .await
            .map_err(|e| format!("写入 ssh-keygen stdin 失败: {e}"))?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(|e| format!("写入 ssh-keygen 换行失败: {e}"))?;
    }

    let keygen_output = timeout(Duration::from_secs(10), child.wait_with_output())
        .await
        .map_err(|_| "ssh-keygen 超时（10 秒）".to_string())?
        .map_err(|e| format!("等待 ssh-keygen 结果失败: {e}"))?;

    if !keygen_output.status.success() {
        return Err(format!(
            "ssh-keygen 返回失败: {}",
            String::from_utf8_lossy(&keygen_output.stderr)
        ));
    }

    let output_line = String::from_utf8_lossy(&keygen_output.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    let fingerprint = output_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| format!("无法解析指纹输出: {output_line}"))?
        .to_string();

    Ok((key_type, fingerprint))
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

    if let Some(pass) = password.filter(|v| !v.trim().is_empty()) {
        SshConnectionManager::cache_password(state, &env_id, pass).await;
    }

    let mut registry = state.env_registry.write().await;
    registry
        .switch_by_id(&env_id)
        .map_err(|e| format!("切换 SSH 环境失败: {e}"))?;
    drop(registry);

    let has_password_cached = SshConnectionManager::has_password(state, &env_id).await;
    let has_password_now = has_password_input || has_password_cached;

    let host_id = env_id.strip_prefix("ssh:").unwrap_or(&env_id).to_string();
    let db_pool = state.db_pool.clone();
    let _ = tokio::task::spawn_blocking(move || {
        let conn = db_pool
            .get()
            .map_err(|e| format!("获取数据库连接失败: {e}"))?;
        ssh_repo::set_last_connected_at(&conn, &host_id, Utc::now())
            .map_err(|e| format!("更新最近连接时间失败: {e}"))
    })
    .await;

    SshConnectionManager::mark_connected(state, env_id.clone(), has_password_now).await;

    Ok(SshConnectionState {
        env_id,
        connected: true,
        has_password: has_password_now,
        last_checked_at: Some(Utc::now().to_rfc3339()),
        last_error: None,
    })
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
            created_at: existing
                .as_ref()
                .map(|v| v.created_at)
                .unwrap_or(now),
            updated_at: now,
            last_connected_at: existing.and_then(|v| v.last_connected_at),
        };

        if ssh_repo::update_host(&conn, &db_host).map_err(|e| format!("更新 SSH 主机失败: {e}"))? {
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
    request: SshProbeFingerprintRequest,
) -> Result<SshFingerprintProbeResult, String> {
    let target = resolve_probe_target(state.inner(), &request).await?;
    let (key_type, fingerprint) = collect_host_fingerprint(&target.host, target.port).await?;

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

    let (status, stored_fingerprint) = if let Some(known_host) = known {
        if known_host.fingerprint == fingerprint {
            ("matched".to_string(), Some(known_host.fingerprint))
        } else {
            ("mismatch".to_string(), Some(known_host.fingerprint))
        }
    } else {
        ("new".to_string(), None)
    };

    Ok(SshFingerprintProbeResult {
        host: target.host,
        port: target.port,
        key_type,
        fingerprint,
        status,
        stored_fingerprint,
    })
}

#[tauri::command]
pub async fn ssh_confirm_host_fingerprint(
    state: State<'_, AppState>,
    request: SshConfirmFingerprintRequest,
) -> Result<(), String> {
    let entry = SshKnownHost {
        host: request.host,
        port: request.port.unwrap_or(22),
        key_type: request.key_type,
        fingerprint: request.fingerprint,
        confirmed_at: Utc::now(),
    };

    let db_pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db_pool
            .get()
            .map_err(|e| format!("获取数据库连接失败: {e}"))?;
        ssh_repo::upsert_known_host(&conn, &entry).map_err(|e| format!("写入 known_hosts 失败: {e}"))
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

    SshConnectionManager::test_connectivity(
        &host.host,
        host.port,
        Some(host.username.as_str()).filter(|v| !v.trim().is_empty()),
        host.identity_file.as_deref(),
    )
    .await
}

/// 列出本机 `~/.ssh/` 目录中发现的所有 SSH 私钥文件信息。
#[tauri::command]
pub async fn ssh_list_keys() -> Result<Vec<SshKeyInfo>, String> {
    Ok(auth::discover_keys().await)
}
