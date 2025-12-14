// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

// 🎯 导入 CCR 核心库
use ccr::{ConfigManager, ConfigService, HistoryService};

/// 应用状态
struct AppState {
    current_platform: Mutex<String>,
}

/// 配置项响应
#[derive(Debug, Serialize)]
struct ProfileInfo {
    name: String,
    description: String,
    base_url: String,
    model: String,
    is_current: bool,
    is_default: bool,
    provider: Option<String>,
}

// ============================================================================
// 📋 Configuration Management Commands
// ============================================================================

/// 列出所有配置项
#[tauri::command]
async fn list_profiles() -> Result<Vec<ProfileInfo>, String> {
    let result = tokio::task::spawn_blocking(move || {
        let manager = ConfigManager::with_default()
            .map_err(|e| format!("Failed to create ConfigManager: {}", e))?;

        let config = manager
            .load()
            .map_err(|e| format!("Failed to load config: {}", e))?;

        let profiles: Vec<ProfileInfo> = config
            .sections
            .iter()
            .map(|(name, section)| ProfileInfo {
                name: name.clone(),
                description: section.description.clone().unwrap_or_default(),
                base_url: section.base_url.clone().unwrap_or_default(),
                model: section.model.clone().unwrap_or_default(),
                is_current: name == &config.current_config,
                is_default: name == &config.default_config,
                provider: section.provider.clone(),
            })
            .collect();

        Ok::<Vec<ProfileInfo>, String>(profiles)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok(result)
}

/// 切换到指定配置
#[tauri::command]
async fn switch_profile(name: String) -> Result<String, String> {
    let profile_name = name.clone();
    tokio::task::spawn_blocking(move || {
        ccr::commands::switch_command(&profile_name).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok(format!("Successfully switched to profile: {}", name))
}

/// 获取当前配置
#[tauri::command]
async fn get_current_profile() -> Result<String, String> {
    let result = tokio::task::spawn_blocking(move || {
        let manager = ConfigManager::with_default()
            .map_err(|e| format!("Failed to create ConfigManager: {}", e))?;

        let config = manager
            .load()
            .map_err(|e| format!("Failed to load config: {}", e))?;

        Ok::<String, String>(config.current_config)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok(result)
}

/// 验证所有配置
#[tauri::command]
async fn validate_configs() -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let service = ConfigService::with_default()
            .map_err(|e| format!("Failed to create ConfigService: {}", e))?;

        service
            .validate_all()
            .map_err(|e| format!("Validation failed: {}", e))?;

        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok("All configurations are valid".to_string())
}

// ============================================================================
// 📜 History Management Commands
// ============================================================================

/// 历史条目
#[derive(Debug, Serialize)]
struct HistoryEntryJson {
    id: String,
    timestamp: String,
    operation: String,
    actor: String,
}

/// 获取历史记录
#[tauri::command]
async fn get_history(limit: Option<usize>) -> Result<Vec<HistoryEntryJson>, String> {
    let limit = limit.unwrap_or(100);

    let result = tokio::task::spawn_blocking(move || {
        let service = HistoryService::with_default()
            .map_err(|e| format!("Failed to create HistoryService: {}", e))?;

        let entries = service
            .get_recent(limit)
            .map_err(|e| format!("Failed to get history: {}", e))?;

        let json_entries: Vec<HistoryEntryJson> = entries
            .into_iter()
            .map(|e| HistoryEntryJson {
                id: e.id.to_string(),
                timestamp: e.timestamp.to_rfc3339(),
                operation: format!("{:?}", e.operation),
                actor: e.actor,
            })
            .collect();

        Ok::<Vec<HistoryEntryJson>, String>(json_entries)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok(result)
}

/// 清理历史记录
#[tauri::command]
async fn clear_history() -> Result<String, String> {
    let result = tokio::task::spawn_blocking(move || {
        let service = HistoryService::with_default()
            .map_err(|e| format!("Failed to create HistoryService: {}", e))?;

        service
            .clear()
            .map_err(|e| format!("Failed to clear history: {}", e))?;

        Ok::<(), String>(())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok("History cleared successfully".to_string())
}

// ============================================================================
// 🔄 Sync Commands
// ============================================================================

/// 推送配置到云端
#[tauri::command]
async fn sync_push(force: Option<bool>) -> Result<String, String> {
    let force_flag = force.unwrap_or(false);
    let result = tokio::task::spawn_blocking(move || {
        use std::process::Command;

        let mut cmd = Command::new("ccr");
        cmd.arg("sync");
        cmd.arg("push");

        if force_flag {
            cmd.arg("--force");
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute ccr command: {}", e))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok::<String, String>(stdout.to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Sync push failed: {}", stderr))
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok(result)
}

/// 从云端拉取配置
#[tauri::command]
async fn sync_pull(force: Option<bool>) -> Result<String, String> {
    let force_flag = force.unwrap_or(false);
    let result = tokio::task::spawn_blocking(move || {
        use std::process::Command;

        let mut cmd = Command::new("ccr");
        cmd.arg("sync");
        cmd.arg("pull");

        if force_flag {
            cmd.arg("--force");
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute ccr command: {}", e))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok::<String, String>(stdout.to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Sync pull failed: {}", stderr))
        }
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    Ok(result)
}

/// 获取同步状态
#[tauri::command]
async fn sync_status() -> Result<String, String> {
    // TODO: 实现同步状态查询
    Ok("Sync status not yet implemented in Tauri".to_string())
}

// ============================================================================
// 🖥️  Platform Commands
// ============================================================================

/// 列出所有平台
#[tauri::command]
async fn list_platforms() -> Result<Vec<String>, String> {
    // TODO: 实现平台列表
    Ok(vec![
        "claude".to_string(),
        "codex".to_string(),
        "gemini".to_string(),
    ])
}

/// 切换平台
#[tauri::command]
async fn switch_platform(
    platform: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut current = state.current_platform.lock().unwrap();
    *current = platform.clone();

    Ok(format!("Successfully switched to platform: {}", platform))
}

/// 获取当前平台
#[tauri::command]
async fn get_current_platform(state: State<'_, AppState>) -> Result<String, String> {
    let current = state.current_platform.lock().unwrap();
    Ok(current.clone())
}

// ============================================================================
// 🚀 Main Entry Point
// ============================================================================

fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .manage(AppState {
            current_platform: Mutex::new("claude".to_string()),
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // Configuration
            list_profiles,
            switch_profile,
            get_current_profile,
            validate_configs,
            // History
            get_history,
            clear_history,
            // Sync
            sync_push,
            sync_pull,
            sync_status,
            // Platform
            list_platforms,
            switch_platform,
            get_current_platform,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
