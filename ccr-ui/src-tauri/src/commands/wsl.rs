//! WSL Tauri 命令 — 列出发行版、读写配置、检测 CLI、同步配置。
//!
//! 仅在 Windows 目标平台编译。
//! 注意：`#[cfg(target_os = "windows")]` 已在 `mod.rs` 中应用于本模块。

use serde::{Deserialize, Serialize};

use crate::platform::ExecutionEnvironment;
use crate::platform::wsl::{
    SyncDirection, WslCacheStatus, WslDistroInfo, WslEnvironment, clear_wsl_cache,
    detect_wsl_distros_with_cache, get_wsl_cache_status, sync_config_blocking,
};

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
///
/// # 参数
/// - `force_refresh`: 是否强制刷新（跳过缓存），默认 false
#[ccr_tauri_command_macros::command]
pub async fn wsl_list_distros(force_refresh: Option<bool>) -> Result<Vec<WslDistroInfo>, String> {
    let force = force_refresh.unwrap_or(false);
    tokio::task::spawn_blocking(move || detect_wsl_distros_with_cache(force))
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
        .map_err(|e| format!("检测 WSL 发行版失败: {e}"))
}

/// 强制刷新 WSL 发行版列表（清除缓存并重新检测）。
#[ccr_tauri_command_macros::command]
pub async fn wsl_refresh_distros() -> Result<Vec<WslDistroInfo>, String> {
    tokio::task::spawn_blocking(|| detect_wsl_distros_with_cache(true))
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
        .map_err(|e| format!("刷新 WSL 发行版失败: {e}"))
}

/// 清除 WSL 缓存。
#[ccr_tauri_command_macros::command]
pub async fn wsl_clear_cache() -> Result<(), String> {
    tokio::task::spawn_blocking(clear_wsl_cache)
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
        .map_err(|e| format!("清除 WSL 缓存失败: {e}"))
}

/// 获取 WSL 缓存状态。
#[ccr_tauri_command_macros::command]
pub async fn wsl_cache_status() -> Result<WslCacheStatus, String> {
    Ok(get_wsl_cache_status())
}

/// 读取指定 WSL 发行版中某平台的配置文件。
///
/// - `distro`: 发行版名称（如 "Ubuntu-22.04"）
/// - `platform`: 平台名（如 "claude"、"codex"）
/// - `path`: 相对于平台配置目录的路径（如 "settings.json"）
#[ccr_tauri_command_macros::command]
pub async fn wsl_read_config(
    distro: String,
    platform: String,
    path: String,
) -> Result<String, String> {
    let distros = tokio::task::spawn_blocking(|| detect_wsl_distros_with_cache(false))
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
#[ccr_tauri_command_macros::command]
pub async fn wsl_write_config(
    distro: String,
    platform: String,
    path: String,
    content: String,
) -> Result<(), String> {
    let distros = tokio::task::spawn_blocking(|| detect_wsl_distros_with_cache(false))
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
#[ccr_tauri_command_macros::command]
pub async fn wsl_detect_cli(distro: String) -> Result<Vec<WslCliInfo>, String> {
    let distros = tokio::task::spawn_blocking(|| detect_wsl_distros_with_cache(false))
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
#[ccr_tauri_command_macros::command]
pub async fn wsl_sync_config(
    distro: String,
    platform: String,
    direction: SyncDirection,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || sync_config_blocking(&distro, &platform, &direction))
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
