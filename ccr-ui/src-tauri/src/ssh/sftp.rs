//! SSH 文件操作抽象（基于当前活跃 SSH 环境，非 russh/sftp 协议实现）。

use std::sync::Arc;

use chrono::Utc;

use crate::platform::ExecutionEnvironment;
use crate::state::AppState;

/// 校验目标环境并返回当前活跃 SSH 环境对象。
async fn ensure_active_ssh_env(
    state: &AppState,
    env_id: &str,
) -> Result<Arc<dyn ExecutionEnvironment>, String> {
    let registry = state.env_registry.read().await;
    let env = registry
        .list()
        .into_iter()
        .find(|e| e.id == env_id)
        .ok_or_else(|| format!("SSH 环境未找到: {env_id}"))?;
    let target_id = env.id;
    drop(registry);

    let registry = state.env_registry.read().await;
    let selected = registry
        .active()
        .ok_or_else(|| "当前无活跃环境".to_string())?;
    if selected.env_id() != target_id {
        return Err("请先连接到目标 SSH 环境".to_string());
    }

    if !selected.env_id().starts_with("ssh:") {
        return Err("当前活跃环境不是 SSH 环境".to_string());
    }

    Ok(selected)
}

/// 读取远端配置文件。
pub async fn read_config(
    state: &AppState,
    env_id: &str,
    platform: &str,
    path: &str,
) -> Result<String, String> {
    let selected = ensure_active_ssh_env(state, env_id).await?;
    selected
        .read_config(platform, path)
        .await
        .map_err(|e| format!("读取 SSH 配置失败: {e}"))
}

/// 写入远端配置文件，可选先做备份。
pub async fn write_config(
    state: &AppState,
    env_id: &str,
    platform: &str,
    path: &str,
    content: &str,
    enable_backup: bool,
) -> Result<(), String> {
    let selected = ensure_active_ssh_env(state, env_id).await?;

    if enable_backup {
        let origin_path = if path.trim().is_empty() {
            "settings.json".to_string()
        } else {
            path.to_string()
        };

        if let Ok(origin_content) = selected.read_config(platform, &origin_path).await {
            let backup_path = format!("{origin_path}.bak.{}", Utc::now().timestamp());
            selected
                .write_config(platform, &backup_path, &origin_content)
                .await
                .map_err(|e| format!("创建 SSH 配置备份失败: {e}"))?;
        }
    }

    selected
        .write_config(platform, path, content)
        .await
        .map_err(|e| format!("写入 SSH 配置失败: {e}"))
}

/// 检测远端 CLI 安装状态。
pub async fn detect_cli(
    state: &AppState,
    env_id: &str,
) -> Result<Vec<crate::platform::CliStatus>, String> {
    let selected = ensure_active_ssh_env(state, env_id).await?;
    let statuses = selected
        .detect_cli_status()
        .await
        .map_err(|e| format!("检测 SSH CLI 失败: {e}"))?;

    Ok(statuses)
}
