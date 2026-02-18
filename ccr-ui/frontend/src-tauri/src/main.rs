// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Manager, RunEvent, State, WindowEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_shell::{ShellExt, process::CommandChild};

// 🎯 导入 CCR 核心库
use ccr::{ConfigManager, ConfigService, HistoryService};

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppSettings {
    skip_exit_confirm: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            skip_exit_confirm: false,
        }
    }
}

/// 应用状态
struct AppState {
    current_platform: Mutex<String>,
    backend_child: Mutex<Option<CommandChild>>,
    backend_ready: Mutex<bool>,
    settings: Mutex<AppSettings>,
    exit_confirmed: AtomicBool,
}

const BACKEND_HOST: &str = "127.0.0.1";
const BACKEND_PORT: u16 = 38081;
const BACKEND_STARTUP_TIMEOUT_SECS: u64 = 15;

fn backend_port() -> u16 {
    if let Ok(value) =
        std::env::var("CCR_UI_BACKEND_PORT").or_else(|_| std::env::var("BACKEND_PORT"))
    {
        if let Ok(port) = value.parse::<u16>() {
            return port;
        }
    }
    BACKEND_PORT
}

async fn wait_for_backend_ready(port: u16) -> bool {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::time::{Duration, sleep};

    let deadline = std::time::Instant::now() + Duration::from_secs(BACKEND_STARTUP_TIMEOUT_SECS);
    while std::time::Instant::now() < deadline {
        if let Ok(mut stream) = TcpStream::connect((BACKEND_HOST, port)).await {
            let request = format!(
                "GET /health HTTP/1.1\r\nHost: {}:{}\r\nConnection: close\r\n\r\n",
                BACKEND_HOST, port
            );
            if stream.write_all(request.as_bytes()).await.is_ok() {
                let mut buffer = [0u8; 12];
                if stream.read(&mut buffer).await.is_ok() {
                    return true;
                }
            }
        }

        sleep(Duration::from_millis(300)).await;
    }

    false
}

fn try_spawn_backend_sidecar(app: &tauri::AppHandle, port: u16) -> Result<CommandChild, String> {
    let command = app
        .shell()
        .sidecar("ccr-ui-backend")
        .map_err(|e| format!("Sidecar not available: {}", e))?
        .args(["--host", BACKEND_HOST, "--port", &port.to_string()]);

    let (mut rx, child) = command
        .spawn()
        .map_err(|e| format!("Failed to spawn sidecar: {}", e))?;

    tauri::async_runtime::spawn(async move {
        use tauri_plugin_shell::process::CommandEvent;
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    tracing::info!("[backend] {}", String::from_utf8_lossy(&line));
                }
                CommandEvent::Stderr(line) => {
                    tracing::warn!("[backend] {}", String::from_utf8_lossy(&line));
                }
                CommandEvent::Error(error) => {
                    tracing::error!("[backend] {}", error);
                }
                CommandEvent::Terminated(payload) => {
                    tracing::warn!("[backend] exited: {:?}", payload.code);
                }
                _ => {}
            }
        }
    });

    Ok(child)
}

fn try_spawn_backend_from_repo(app: &tauri::AppHandle, port: u16) -> Result<CommandChild, String> {
    use std::path::PathBuf;

    let exe_dir = tauri::utils::platform::current_exe()
        .map_err(|e| format!("Failed to resolve current exe: {}", e))?
        .parent()
        .ok_or_else(|| "Current exe has no parent directory".to_string())?
        .to_path_buf();

    let mut search_dir = Some(exe_dir.as_path());
    let mut repo_root = None;

    for _ in 0..6 {
        if let Some(dir) = search_dir {
            if dir.join("backend").exists() && dir.join("frontend").exists() {
                repo_root = Some(dir.to_path_buf());
                break;
            }
            search_dir = dir.parent();
        }
    }

    let repo_root =
        repo_root.ok_or_else(|| "Repo root not found for fallback backend".to_string())?;
    let binary_name = if cfg!(windows) {
        "ccr-ui-backend.exe"
    } else {
        "ccr-ui-backend"
    };

    let mut candidates: Vec<PathBuf> = Vec::new();

    candidates.push(exe_dir.join(binary_name));

    if let Ok(target_triple) = std::env::var("TARGET") {
        candidates.push(
            repo_root
                .join("backend")
                .join("target")
                .join(&target_triple)
                .join("release")
                .join(binary_name),
        );
        candidates.push(
            repo_root
                .join("backend")
                .join("target")
                .join(&target_triple)
                .join("debug")
                .join(binary_name),
        );
    }

    candidates.push(
        repo_root
            .join("backend")
            .join("target")
            .join("release")
            .join(binary_name),
    );
    candidates.push(
        repo_root
            .join("backend")
            .join("target")
            .join("debug")
            .join(binary_name),
    );

    if let Some(workspace_root) = repo_root.parent().and_then(|parent| {
        let workspace = parent.join("Cargo.toml");
        let ui_dir = parent.join("ccr-ui");
        if workspace.exists() && ui_dir.exists() {
            Some(parent.to_path_buf())
        } else {
            None
        }
    }) {
        candidates.push(
            workspace_root
                .join("target")
                .join("release")
                .join(binary_name),
        );
        candidates.push(
            workspace_root
                .join("target")
                .join("debug")
                .join(binary_name),
        );
    }

    let sidecar_bin_dir = repo_root.join("frontend").join("src-tauri").join("bin");
    if let Ok(entries) = std::fs::read_dir(&sidecar_bin_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = path.file_name().and_then(|name| name.to_str());
            if let Some(file_name) = file_name {
                if file_name.starts_with("ccr-ui-backend") {
                    candidates.push(path);
                }
            }
        }
    }

    let backend_bin = candidates
        .into_iter()
        .find(|candidate| candidate.exists())
        .ok_or_else(|| "Fallback backend binary missing".to_string())?;

    let command = app.shell().command(backend_bin).args([
        "--host",
        BACKEND_HOST,
        "--port",
        &port.to_string(),
    ]);
    let (mut rx, child) = command
        .spawn()
        .map_err(|e| format!("Failed to spawn fallback backend: {}", e))?;

    tauri::async_runtime::spawn(async move {
        use tauri_plugin_shell::process::CommandEvent;
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    tracing::info!("[backend] {}", String::from_utf8_lossy(&line));
                }
                CommandEvent::Stderr(line) => {
                    tracing::warn!("[backend] {}", String::from_utf8_lossy(&line));
                }
                CommandEvent::Error(error) => {
                    tracing::error!("[backend] {}", error);
                }
                CommandEvent::Terminated(payload) => {
                    tracing::warn!("[backend] exited: {:?}", payload.code);
                }
                _ => {}
            }
        }
    });

    Ok(child)
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
    ccr::commands::switch_command(&name)
        .await
        .map_err(|e| e.to_string())?;

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

    let service = HistoryService::with_default()
        .map_err(|e| format!("Failed to create HistoryService: {}", e))?;

    let entries = service
        .get_recent_async(limit)
        .await
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

    Ok(json_entries)
}

/// 清理历史记录
#[tauri::command]
async fn clear_history() -> Result<String, String> {
    let service = HistoryService::with_default()
        .map_err(|e| format!("Failed to create HistoryService: {}", e))?;

    service
        .clear_async()
        .await
        .map_err(|e| format!("Failed to clear history: {}", e))?;

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
    use ccr::sync::{SyncConfigManager, SyncService};

    let result = tokio::task::spawn_blocking(move || {
        let manager = SyncConfigManager::with_default()
            .map_err(|e| format!("Failed to create SyncConfigManager: {}", e))?;

        let config = manager
            .load()
            .map_err(|e| format!("Failed to load sync config: {}", e))?;

        if !config.enabled {
            return Ok("Sync is disabled".to_string());
        }

        // Return a formatted status string
        let mut status = String::new();
        status.push_str(&format!("Server: {}\n", config.webdav_url));
        status.push_str(&format!("User: {}\n", config.username));
        status.push_str(&format!("Remote Path: {}\n", config.remote_path));
        status.push_str(&format!(
            "Auto Sync: {}\n",
            if config.auto_sync {
                "Enabled"
            } else {
                "Disabled"
            }
        ));

        // We can't easily do async check in this blocking task if we want to be quick,
        // but SyncService::new is async.
        // So we need to use a runtime handle or return config info and let frontend check connectivity?
        // Or better, do the check in the async block outside spawn_blocking if possible.
        // But SyncConfigManager is blocking (rusqlite/fs).

        Ok::<String, String>(status)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    // Check connectivity in async context
    if result != "Sync is disabled" {
        // We need config again for service. Ideally pass config out.
        // Let's re-load config in async block for connectivity check? No, minimal overhead.
        // Actually best is to implement this properly.

        let check_result = async {
            let manager = SyncConfigManager::with_default()
                .map_err(|e| format!("Failed to create SyncConfigManager: {}", e))?;
            let config = manager
                .load()
                .map_err(|e| format!("Failed to load sync config: {}", e))?;

            let service = SyncService::new(&config)
                .await
                .map_err(|e| format!("Failed to create SyncService: {}", e))?;

            let exists = service
                .remote_exists()
                .await
                .map_err(|e| format!("Check remote failed: {}", e))?;

            Ok::<String, String>(if exists {
                format!("{}\nRemote Status: Connected (Content Exists)", result)
            } else {
                format!("{}\nRemote Status: Connected (Content Missing)", result)
            })
        }
        .await;

        return check_result;
    }

    Ok(result)
}

// ============================================================================
// 🖥️  Platform Commands
// ============================================================================

/// 列出所有平台
#[tauri::command]
async fn list_platforms() -> Result<Vec<String>, String> {
    Ok(vec![
        "claude".to_string(),
        "codex".to_string(),
        "gemini".to_string(),
        "qwen".to_string(),
        "iflow".to_string(),
    ])
}

/// 切换平台
#[tauri::command]
async fn switch_platform(platform: String, state: State<'_, AppState>) -> Result<String, String> {
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
// ⚙️ Settings Commands
// ============================================================================

/// 获取退出确认设置
#[tauri::command]
async fn get_skip_exit_confirm(state: State<'_, AppState>) -> Result<bool, String> {
    let settings = state.settings.lock().unwrap();
    Ok(settings.skip_exit_confirm)
}

/// 设置退出确认选项
#[tauri::command]
async fn set_skip_exit_confirm(skip: bool, state: State<'_, AppState>) -> Result<(), String> {
    let mut settings = state.settings.lock().unwrap();
    settings.skip_exit_confirm = skip;
    Ok(())
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

    let app = tauri::Builder::default()
        .manage(AppState {
            current_platform: Mutex::new("claude".to_string()),
            backend_child: Mutex::new(None),
            backend_ready: Mutex::new(false),
            settings: Mutex::new(AppSettings::default()),
            exit_confirmed: AtomicBool::new(false),
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let app_state = window.state::<AppState>();

                // 如果已确认退出，直接关闭
                if app_state.exit_confirmed.load(Ordering::SeqCst) {
                    return;
                }

                // 检查是否跳过确认
                let skip_confirm = {
                    let settings = app_state.settings.lock().unwrap();
                    settings.skip_exit_confirm
                };

                if skip_confirm {
                    return;
                }

                // 阻止默认关闭，显示确认对话框
                api.prevent_close();
                let window = window.clone();
                window
                    .dialog()
                    .message("确定要关闭 CCR Desktop 吗？\n\n勾选下方选项可跳过此确认")
                    .title("确认退出")
                    .kind(MessageDialogKind::Warning)
                    .buttons(MessageDialogButtons::OkCancelCustom(
                        "退出".to_string(),
                        "取消".to_string(),
                    ))
                    .show(move |confirmed| {
                        if confirmed {
                            let app_state = window.state::<AppState>();
                            app_state.exit_confirmed.store(true, Ordering::SeqCst);
                            let _ = window.close();
                        }
                    });
            }
        })
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let port = backend_port();
                let child = match try_spawn_backend_sidecar(&app_handle, port) {
                    Ok(child) => Ok(child),
                    Err(err) => {
                        tracing::warn!("[backend] sidecar start failed, fallback: {}", err);
                        try_spawn_backend_from_repo(&app_handle, port)
                    }
                };

                match child {
                    Ok(child) => {
                        if wait_for_backend_ready(port).await {
                            tracing::info!("[backend] ready at http://{}:{}", BACKEND_HOST, port);
                            let state = app_handle.state::<AppState>();
                            *state.backend_ready.lock().unwrap() = true;
                        } else {
                            tracing::warn!("[backend] readiness check timed out");
                        }

                        let state = app_handle.state::<AppState>();
                        *state.backend_child.lock().unwrap() = Some(child);
                    }
                    Err(err) => {
                        tracing::error!("[backend] failed to start: {}", err);
                    }
                }
            });
            Ok(())
        })
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
            // Settings
            get_skip_exit_confirm,
            set_skip_exit_confirm,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app, event| {
        if let RunEvent::Exit = event {
            tracing::info!("[app] Application exiting, cleaning up...");
            let child = {
                let app_state = app.state::<AppState>();
                let mut guard = app_state.backend_child.lock().unwrap();
                guard.take()
            };
            if let Some(child) = child {
                tracing::info!("[backend] Terminating backend process...");
                match child.kill() {
                    Ok(()) => tracing::info!("[backend] Backend process terminated successfully"),
                    Err(e) => tracing::error!("[backend] Failed to terminate backend: {}", e),
                }
            } else {
                tracing::warn!("[backend] No backend process to terminate");
            }
        }
    });
}
