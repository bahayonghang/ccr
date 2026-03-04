//! WSL Tauri 命令 — 列出发行版、读写配置、检测 CLI、同步配置。
//!
//! 仅在 Windows 目标平台编译。
//! 注意：`#[cfg(target_os = "windows")]` 已在 `mod.rs` 中应用于本模块。

use serde::{Deserialize, Serialize};

use crate::platform::wsl::{
    detect_wsl_distros, sync_config_blocking, SyncDirection, WslDistroInfo, WslEnvironment,
};
use crate::platform::ExecutionEnvironment;

// ── 响应类型 ────────────────────────────────────────────────────────────────

/// WSL CLI 检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WslCliInfo {
    pub name: String,
    pub installed: bool,
    pub path: Option<String>,
}

// ── Tauri 命令 ───────────────────────────────────────────────────────────────

/// 列出所有已安装的 WSL 发行版。
#[tauri::command]
pub async fn wsl_list_distros() -> Result<Vec<WslDistroInfo>, String> {
    tokio::task::spawn_blocking(detect_wsl_distros)
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
        .map_err(|e| format!("检测 WSL 发行版失败: {e}"))
}

/// 读取指定 WSL 发行版中某平台的配置文件。
///
/// - `distro`: 发行版名称（如 "Ubuntu-22.04"）
/// - `platform`: 平台名（如 "claude"、"codex"）
/// - `path`: 相对于平台配置目录的路径（如 "settings.json"）
#[tauri::command]
pub async fn wsl_read_config(
    distro: String,
    platform: String,
    path: String,
) -> Result<String, String> {
    // 先获取发行版信息
    let distros = tokio::task::spawn_blocking(detect_wsl_distros)
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
        .map_err(|e| format!("检测 WSL 发行版失败: {e}"))?;

    let distro_info = find_distro(&distros, &distro)?;
    let env = WslEnvironment::new(distro_info);
    env.read_config(&platform, &path)
        .await
        .map_err(|e| format!("读取 WSL 配置失败: {e}"))
}

/// 写入指定 WSL 发行版中某平台的配置文件。
///
/// - `distro`: 发行版名称
/// - `platform`: 平台名
/// - `path`: 相对路径
/// - `content`: 文件内容
#[tauri::command]
pub async fn wsl_write_config(
    distro: String,
    platform: String,
    path: String,
    content: String,
) -> Result<(), String> {
    let distros = tokio::task::spawn_blocking(detect_wsl_distros)
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
        .map_err(|e| format!("检测 WSL 发行版失败: {e}"))?;

    let distro_info = find_distro(&distros, &distro)?;
    let env = WslEnvironment::new(distro_info);
    env.write_config(&platform, &path, &content)
        .await
        .map_err(|e| format!("写入 WSL 配置失败: {e}"))
}

/// 检测指定 WSL 发行版中已安装的 AI CLI 工具。
#[tauri::command]
pub async fn wsl_detect_cli(distro: String) -> Result<Vec<WslCliInfo>, String> {
    let distros = tokio::task::spawn_blocking(detect_wsl_distros)
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
        .map_err(|e| format!("检测 WSL 发行版失败: {e}"))?;

    let distro_info = find_distro(&distros, &distro)?;
    let env = WslEnvironment::new(distro_info);

    let statuses = env
        .detect_cli_status()
        .await
        .map_err(|e| format!("检测 CLI 失败: {e}"))?;

    Ok(statuses
        .into_iter()
        .map(|s| WslCliInfo {
            name: s.name,
            installed: s.installed,
            path: s.path,
        })
        .collect())
}

/// 在本地（Windows）和 WSL 之间同步指定平台的配置文件。
///
/// - `distro`: 发行版名称
/// - `platform`: 平台名
/// - `direction`: 同步方向（"localToWsl" 或 "wslToLocal"）
#[tauri::command]
pub async fn wsl_sync_config(
    distro: String,
    platform: String,
    direction: SyncDirection,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        sync_config_blocking(&distro, &platform, &direction)
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
    .map_err(|e| format!("同步配置失败: {e}"))
}

// ── 内部辅助 ─────────────────────────────────────────────────────────────────

/// 在发行版列表中按名称查找，返回克隆。
fn find_distro(distros: &[WslDistroInfo], name: &str) -> Result<WslDistroInfo, String> {
    distros
        .iter()
        .find(|d| d.name == name)
        .cloned()
        .ok_or_else(|| format!("WSL 发行版未找到: {name}"))
}
