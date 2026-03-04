//! 系统命令 — 系统信息、版本检查、健康检查、事件查询。

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

/// 系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub arch: String,
    pub hostname: String,
    pub cpu_count: usize,
    pub total_memory_mb: u64,
    pub ccr_version: String,
}

/// 版本检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub current: String,
    pub latest: Option<String>,
    pub update_available: bool,
}

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_all();

    Ok(SystemInfo {
        os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
        os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
        arch: std::env::consts::ARCH.to_string(),
        hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
        cpu_count: sys.cpus().len(),
        total_memory_mb: sys.total_memory() / 1024 / 1024,
        ccr_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tauri::command]
pub async fn check_version() -> Result<VersionInfo, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    // TODO: Phase B4 — 查询 GitHub releases API
    Ok(VersionInfo {
        current,
        latest: None,
        update_available: false,
    })
}

#[tauri::command]
pub async fn health_check(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    // 检查数据库连接
    let db_ok = state.db_pool.get().map(|_| true).unwrap_or(false);

    Ok(serde_json::json!({
        "status": if db_ok { "healthy" } else { "degraded" },
        "database": db_ok,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

#[tauri::command]
pub async fn get_recent_events(
    state: State<'_, AppState>,
    count: Option<usize>,
) -> Result<Vec<crate::events::EventLogEntry>, String> {
    let count = count.unwrap_or(50);
    Ok(state.event_log.recent(count).await)
}

#[tauri::command]
pub async fn update_ccr() -> Result<serde_json::Value, String> {
    let output = tokio::process::Command::new("cargo")
        .args(["install", "ccr"])
        .output()
        .await
        .map_err(|e| format!("Failed to spawn cargo install: {e}"))?;

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(serde_json::json!({
        "success": success,
        "stdout": stdout,
        "stderr": stderr,
    }))
}

#[tauri::command]
pub async fn get_cli_versions() -> Result<serde_json::Value, String> {
    let tools = ["ccr", "claude", "codex", "gemini"];

    let mut versions = serde_json::Map::new();

    for tool in tools {
        let result = tokio::process::Command::new(tool)
            .arg("--version")
            .output()
            .await;

        let version = match result {
            Ok(output) if output.status.success() => {
                let raw = String::from_utf8_lossy(&output.stdout).to_string();
                raw.lines().next().unwrap_or("").trim().to_string()
            }
            Ok(output) => {
                // 某些工具将版本输出到 stderr
                let raw = String::from_utf8_lossy(&output.stderr).to_string();
                let line = raw.lines().next().unwrap_or("").trim().to_string();
                if line.is_empty() { "not found".to_string() } else { line }
            }
            Err(_) => "not found".to_string(),
        };

        versions.insert(tool.to_string(), serde_json::Value::String(version));
    }

    Ok(serde_json::json!({ "versions": versions }))
}
