//! SSH 连接管理器（运行时状态 + 密码缓存）。

use std::time::Instant;

use chrono::Utc;
use tokio::time::{Duration, timeout};

use crate::process::tokio_command;
use crate::state::{AppState, DEFAULT_SSH_PASSWORD_TTL_SECS, SshPasswordEntry, SshRuntimeState};

/// SSH 连接测试结果。
#[derive(Debug, Clone, serde::Serialize)]
pub struct SshConnectResult {
    /// 是否连接成功。
    pub success: bool,
    /// 连接延迟（毫秒）。
    pub latency_ms: u64,
    /// 失败时的错误信息。
    pub error: Option<String>,
}

pub struct SshConnectionManager;

impl SshConnectionManager {
    /// 缓存密码（仅内存，不持久化）。
    pub async fn cache_password(state: &AppState, env_id: &str, password: String) {
        if password.trim().is_empty() {
            return;
        }
        let mut passwords = state.ssh_password_cache.write().await;
        passwords.insert(
            env_id.to_string(),
            SshPasswordEntry {
                value: password,
                cached_at: Instant::now(),
            },
        );
    }

    /// 清理指定环境的密码缓存。
    pub async fn clear_password(state: &AppState, env_id: &str) {
        let mut passwords = state.ssh_password_cache.write().await;
        passwords.remove(env_id);
    }

    /// 检查指定环境是否存在密码缓存。
    pub async fn has_password(state: &AppState, env_id: &str) -> bool {
        let ttl = Duration::from_secs(DEFAULT_SSH_PASSWORD_TTL_SECS);
        let mut passwords = state.ssh_password_cache.write().await;
        match passwords.get(env_id) {
            Some(entry) if entry.cached_at.elapsed() <= ttl => {
                let _is_empty = entry.value.trim().is_empty();
                !_is_empty
            }
            Some(_) => {
                passwords.remove(env_id);
                false
            }
            None => false,
        }
    }

    /// 写入/覆盖连接状态。
    pub async fn set_state(
        state: &AppState,
        env_id: String,
        connected: bool,
        has_password: bool,
        last_error: Option<String>,
    ) {
        let mut map = state.ssh_runtime_states.write().await;
        map.insert(
            env_id.clone(),
            SshRuntimeState {
                env_id,
                connected,
                has_password,
                last_checked_at: Some(Utc::now().to_rfc3339()),
                last_error,
            },
        );
    }

    /// 标记连接成功。
    pub async fn mark_connected(state: &AppState, env_id: String, has_password: bool) {
        Self::set_state(state, env_id, true, has_password, None).await;
    }

    /// 标记连接断开。
    pub async fn mark_disconnected(state: &AppState, env_id: String, last_error: Option<String>) {
        Self::set_state(state, env_id, false, false, last_error).await;
    }

    /// 获取单个环境连接状态。
    pub async fn get_state(state: &AppState, env_id: &str) -> Option<SshRuntimeState> {
        let map = state.ssh_runtime_states.read().await;
        map.get(env_id).cloned()
    }

    /// 获取全部连接状态。
    pub async fn list_states(state: &AppState) -> Vec<SshRuntimeState> {
        let map = state.ssh_runtime_states.read().await;
        map.values().cloned().collect()
    }

    /// 测试 SSH 连接是否可达（10 秒超时）。
    ///
    /// 使用 `ssh -o BatchMode=yes -o ConnectTimeout=10` 执行 `echo __CCR_SSH_OK__`
    /// 来验证连通性并测量延迟。
    pub async fn test_connectivity(
        host: &str,
        port: u16,
        user: Option<&str>,
        identity_file: Option<&str>,
    ) -> Result<SshConnectResult, String> {
        let mut cmd = tokio_command("ssh");
        cmd.arg("-o").arg("BatchMode=yes");
        cmd.arg("-o").arg("ConnectTimeout=10");
        cmd.arg("-o").arg("StrictHostKeyChecking=accept-new");
        cmd.arg("-p").arg(port.to_string());

        if let Some(key) = identity_file
            && !key.trim().is_empty()
        {
            cmd.arg("-i").arg(key);
        }

        let target = if let Some(u) = user.filter(|v| !v.trim().is_empty()) {
            format!("{u}@{host}")
        } else {
            host.to_string()
        };
        cmd.arg(target);
        cmd.arg("echo __CCR_SSH_OK__");

        let start = Instant::now();

        let result = timeout(Duration::from_secs(10), cmd.output()).await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Err(_) => Ok(SshConnectResult {
                success: false,
                latency_ms,
                error: Some("连接超时（10 秒）".to_string()),
            }),
            Ok(Err(e)) => Ok(SshConnectResult {
                success: false,
                latency_ms,
                error: Some(format!("执行 ssh 失败: {e}")),
            }),
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if output.status.success() && stdout.contains("__CCR_SSH_OK__") {
                    Ok(SshConnectResult {
                        success: true,
                        latency_ms,
                        error: None,
                    })
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Ok(SshConnectResult {
                        success: false,
                        latency_ms,
                        error: Some(stderr.trim().to_string()),
                    })
                }
            }
        }
    }
}
