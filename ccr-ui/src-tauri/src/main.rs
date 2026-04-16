// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod checkin_jobs;
mod commands;
mod desktop_shell;
mod events;
mod monitoring;
mod platform;
mod process;
mod session_index_jobs;
mod skills_watcher;
mod ssh;
mod state;
mod usage_jobs;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tokio::sync::Notify;

use platform::local::LocalEnvironment;
use state::{AppState, DEFAULT_SSH_PASSWORD_TTL_SECS, DEFAULT_SSH_STATE_TTL_SECS};

/// 退出流程标志，避免重复触发清理与关闭逻辑。
static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "macos")]
fn configure_main_window_chrome(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    window.set_decorations(true)?;
    window.set_title_bar_style(tauri::TitleBarStyle::Visible)?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn configure_main_window_chrome(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    window.set_decorations(true)?;
    Ok(())
}

fn main() {
    ccr_core::init_logger();

    tracing::info!(
        "[app] CCR Desktop v{} starting (native Tauri mode)",
        env!("CARGO_PKG_VERSION")
    );

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                if let Err(error) = configure_main_window_chrome(&window) {
                    tracing::warn!("[app] failed to configure main window chrome: {error}");
                } else {
                    #[cfg(target_os = "macos")]
                    tracing::info!("[app] macOS native window chrome enabled for main window");
                    #[cfg(not(target_os = "macos"))]
                    tracing::info!("[app] native window chrome enabled for main window");
                }
            }

            // 初始化全局数据库，并确保后续通过 with_connection() 的调用可用。
            // 这里先启动全局连接池，再为 AppState 创建独立的应用连接池。
            ccr_db::database::initialize().map_err(|e| {
                tracing::error!("[app] failed to initialize global database pool: {e}");
                Box::new(e) as Box<dyn std::error::Error>
            })?;
            // 创建应用级连接池，供 AppState、SSH/环境注册和后台任务复用。
            let db_pool = ccr_db::database::create_app_pool().map_err(|e| {
                tracing::error!("[app] failed to create app database pool: {e}");
                Box::new(e) as Box<dyn std::error::Error>
            })?;
            tracing::info!("[app] database initialized (global + app pool)");

            // 构建并注册全局 AppState。
            let app_state = AppState::new(db_pool);

            // 先注册 Local 环境，其他环境在异步初始化完成后写入 managed state。
            app.manage(app_state);
            app.manage(skills_watcher::SkillsWatcherState::default());
            skills_watcher::reload(app.handle()).map_err(|e| {
                tracing::warn!("[app] skills watcher init failed: {e}");
                std::io::Error::other(e)
            })?;
            desktop_shell::install_desktop_shell(app.handle()).map_err(std::io::Error::other)?;

            let tray_refresh_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) =
                    desktop_shell::refresh_codex_tray(&tray_refresh_handle, false).await
                {
                    tracing::debug!("[tray] initial refresh skipped: {error}");
                }
            });

            crate::commands::codex::restore_pending_oauth_listener(app.handle().clone());

            // 异步初始化环境注册表，避免阻塞启动流程。
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = app_handle.state::<AppState>();

                // Phase C1：在 Windows 上探测 WSL 发行版，失败时仅记录日志。
                #[cfg(target_os = "windows")]
                let distros = {
                    use platform::wsl::detect_wsl_distros_with_cache;
                    match tokio::task::spawn_blocking(|| detect_wsl_distros_with_cache(false)).await
                    {
                        Ok(Ok(distros)) => distros,
                        Ok(Err(e)) => {
                            tracing::debug!("[app] WSL detection skipped: {e}");
                            Vec::new()
                        }
                        Err(e) => {
                            tracing::debug!("[app] WSL detection task failed: {e}");
                            Vec::new()
                        }
                    }
                };

                // Phase C2：从数据库读取 SSH 主机配置并注册到环境列表。
                use ccr_db::database::repositories::ssh_repo;
                let db_pool = state.db_pool.clone();
                let hosts = match tokio::task::spawn_blocking(move || {
                    let conn = db_pool
                        .get()
                        .map_err(|e| format!("获取 SSH 数据库连接失败: {e}"))?;
                    ssh_repo::list_hosts(&conn).map_err(|e| format!("查询 SSH 主机列表失败: {e}"))
                })
                .await
                {
                    Ok(Ok(hosts)) => hosts,
                    Ok(Err(e)) => {
                        tracing::warn!("[app] failed to load SSH hosts: {e}");
                        Vec::new()
                    }
                    Err(e) => {
                        tracing::warn!("[app] SSH hosts loading task failed: {e}");
                        Vec::new()
                    }
                };

                let mut registry = state.env_registry.write().await;

                // 注册本地环境。
                registry.register(Arc::new(LocalEnvironment::new()));
                tracing::info!("[app] local environment registered");

                #[cfg(target_os = "windows")]
                {
                    use platform::wsl::WslEnvironment;
                    for distro in distros {
                        let name = distro.name.clone();
                        registry.register(Arc::new(WslEnvironment::new(distro)));
                        tracing::info!("[app] WSL environment registered: {name}");
                    }
                }

                use platform::ssh::SshEnvironment;
                for host in hosts {
                    let label = if host.name.trim().is_empty() {
                        host.host.clone()
                    } else {
                        host.name.clone()
                    };
                    registry.register(Arc::new(SshEnvironment::new(
                        crate::platform::ssh::SshHostConfig {
                            id: Some(host.id),
                            name: Some(host.name).filter(|v| !v.trim().is_empty()),
                            host: host.host,
                            port: Some(host.port),
                            user: Some(host.username).filter(|v| !v.trim().is_empty()),
                            identity_file: host.identity_file,
                            remote_home: host.remote_home,
                        },
                    )));
                    tracing::info!("[app] SSH environment registered: {label}");
                }

                tracing::info!(
                    "[app] environment registry initialized ({} environments)",
                    registry.len()
                );
            });

            // 启动后台维护任务。
            let app_handle = app.handle().clone();
            let shutdown_notify = Arc::new(Notify::new());
            let shutdown_clone = shutdown_notify.clone();

            tauri::async_runtime::spawn(async move {
                run_background_tasks(app_handle, shutdown_clone).await;
            });

            // 注册 shutdown notify，供退出时通知后台任务收尾。
            app.manage(shutdown_notify);

            tracing::info!("[app] setup complete");
            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                if desktop_shell::is_tray_panel_window(window.label()) {
                    api.prevent_close();
                    let _ = window.hide();
                    return;
                }

                if window.label() != "main" {
                    return;
                }

                let state = window.state::<AppState>();
                let preferences = match state.desktop_shell_preferences() {
                    Ok(settings) => settings,
                    Err(error) => {
                        tracing::warn!(
                            "[app] failed to access shell preferences during exit flow: {error}"
                        );
                        state::DesktopShellPreferences::default()
                    }
                };
                let action = desktop_shell::resolve_main_window_close_action(
                    &preferences,
                    state
                        .exit_confirmed
                        .load(std::sync::atomic::Ordering::SeqCst),
                    state
                        .force_exit_requested
                        .load(std::sync::atomic::Ordering::SeqCst),
                );

                match action {
                    desktop_shell::MainWindowCloseAction::AllowExit => {}
                    desktop_shell::MainWindowCloseAction::HideToTray => {
                        api.prevent_close();
                        if let Err(error) = window.hide() {
                            tracing::warn!("[app] failed to hide main window to tray: {error}");
                        }
                    }
                    desktop_shell::MainWindowCloseAction::ConfirmExit => {
                        api.prevent_close();
                        let window = window.clone();
                        window
                            .dialog()
                            .message("Are you sure you want to close CCR Desktop?")
                            .title("Confirm Exit")
                            .kind(MessageDialogKind::Warning)
                            .buttons(MessageDialogButtons::OkCancelCustom(
                                "Exit".to_string(),
                                "Cancel".to_string(),
                            ))
                            .show(move |confirmed| {
                                if confirmed {
                                    let state = window.state::<AppState>();
                                    state
                                        .exit_confirmed
                                        .store(true, std::sync::atomic::Ordering::SeqCst);
                                    let _ = window.close();
                                }
                            });
                    }
                }
            }
            WindowEvent::Focused(false) if desktop_shell::is_tray_panel_window(window.label()) => {
                if let Err(error) = window.hide() {
                    tracing::debug!("[tray] failed to auto-hide panel on blur: {error}");
                }
            }
            _ => {}
        })
        .invoke_handler(commands::generate_handler())
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app, event| {
        if let RunEvent::ExitRequested { .. } | RunEvent::Exit = event {
            tracing::info!("[app] exit requested, shutting down...");
            EXIT_REQUESTED.store(true, Ordering::SeqCst);

            // 通知后台任务停止等待并尽快结束。
            if let Some(notify) = _app.try_state::<Arc<Notify>>() {
                notify.notify_waiters();
            }

            // 关闭数据库全局资源。
            ccr_db::database::shutdown();
            tracing::info!("[app] cleanup complete");
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::desktop_shell::{MainWindowCloseAction, resolve_main_window_close_action};
    use crate::state::DesktopShellPreferences;

    #[test]
    fn close_action_hides_main_window_to_tray_when_enabled() {
        let action = resolve_main_window_close_action(
            &DesktopShellPreferences {
                confirm_before_exit: true,
                close_to_tray: true,
                open_panel_on_tray_click: true,
            },
            false,
            false,
        );

        assert_eq!(action, MainWindowCloseAction::HideToTray);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_window_chrome_config_enables_native_decorations() {
        let config = super::macos_native_window_chrome_config();

        assert!(config.decorations);
        assert_eq!(config.title_bar_style, tauri::TitleBarStyle::Visible);
    }
}

/// 后台维护任务循环，定期清理缓存并尝试刷新用量数据。
async fn run_background_tasks(app_handle: tauri::AppHandle, shutdown: Arc<Notify>) {
    tracing::info!("[background] starting background tasks");

    loop {
        tokio::select! {
            _ = shutdown.notified() => {
                tracing::info!("[background] shutdown signal received");
                break;
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(300)) => {
                if EXIT_REQUESTED.load(Ordering::SeqCst) {
                    break;
                }

                // 定期执行缓存、SSH 状态与监控日志清理。
                let state = app_handle.state::<AppState>();
                state.cache_cleanup().await;
                state
                    .cleanup_ssh_runtime_states(DEFAULT_SSH_STATE_TTL_SECS)
                    .await;
                state
                    .cleanup_ssh_password_cache(DEFAULT_SSH_PASSWORD_TTL_SECS)
                    .await;
                state.monitoring_logs.cleanup_old_logs().await;

                // 尝试刷新 usage 数据；失败时仅记录调试日志，不影响后台循环。
                if let Err(e) = import_usage_data().await {
                    tracing::debug!("[background] usage import skipped: {e}");
                }

                tracing::debug!("[background] periodic maintenance completed");
            }
        }
    }

    tracing::info!("[background] background tasks stopped");
}

/// 检查并准备 Usage 导入所需的 JSONL 存储目录。
async fn import_usage_data() -> Result<(), String> {
    // CostTracker 默认使用与 CLI 一致的目录 `~/.ccr/costs/`。
    // 这里只校验 storage dir 可访问，真正的导入由其他命令触发。
    tokio::task::spawn_blocking(|| {
        let _storage_dir = ccr_store::CostTracker::default_storage_dir()
            .map_err(|e| format!("Storage dir: {e}"))?;
        Ok(())
    })
    .await
    .map_err(|e| format!("Task join: {e}"))?
}
