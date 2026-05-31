// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod checkin_jobs;
mod claude_observer;
mod commands;
mod desktop_shell;
mod events;
mod llmusage_adapter;
mod monitoring;
mod platform;
mod process;
mod session_index_jobs;
mod ssh;
mod state;
mod stats_snapshot;
#[cfg(test)]
mod test_support;
mod usage_jobs;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{Emitter, Manager, RunEvent, WindowEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tokio::sync::Notify;

use crate::events::channels;
use platform::local::LocalEnvironment;
use state::{AppState, DEFAULT_SSH_PASSWORD_TTL_SECS, DEFAULT_SSH_STATE_TTL_SECS};

/// 退出流程标志，避免重复触发清理与关闭逻辑。
static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);

/// fire-and-forget 后台任务监督封装。
///
/// 目标：
/// 1) 任务 panic 不吞没 —— 通过内嵌 `tokio::spawn` + `JoinError::is_panic()` 捕获；
/// 2) panic / join 失败统一广播 `channels::APP_TASK_PANICKED`，前端监控面板可据此报警；
/// 3) cancel 属于正常关闭路径，仅 debug 日志。
fn spawn_supervised<F>(app: tauri::AppHandle, name: &'static str, fut: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    tauri::async_runtime::spawn(async move {
        let handle = tokio::spawn(fut);
        match handle.await {
            Ok(()) => {}
            Err(error) if error.is_cancelled() => {
                tracing::debug!("[supervised] task '{name}' cancelled");
            }
            Err(error) => {
                if error.is_panic() {
                    tracing::error!("[supervised] task '{name}' panicked: {error}");
                } else {
                    tracing::warn!("[supervised] task '{name}' join error: {error}");
                }
                if let Err(emit_err) = app.emit(
                    channels::APP_TASK_PANICKED,
                    serde_json::json!({ "name": name, "error": error.to_string() }),
                ) {
                    tracing::warn!(
                        "[supervised] failed to emit APP_TASK_PANICKED for '{name}': {emit_err}"
                    );
                }
            }
        }
    });
}

#[cfg(target_os = "macos")]
pub(crate) fn configure_main_window_chrome<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
) -> tauri::Result<()> {
    window.set_decorations(true)?;
    window.set_title_bar_style(tauri::TitleBarStyle::Visible)?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn configure_main_window_chrome<R: tauri::Runtime>(
    window: &tauri::WebviewWindow<R>,
) -> tauri::Result<()> {
    window.set_decorations(true)?;
    Ok(())
}

#[cfg(debug_assertions)]
pub(crate) fn open_devtools_in_debug<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) {
    window.open_devtools();
}

#[cfg(not(debug_assertions))]
pub(crate) fn open_devtools_in_debug<R: tauri::Runtime>(_window: &tauri::WebviewWindow<R>) {}

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
                    open_devtools_in_debug(&window);
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
            let usage_db_pool = ccr_db::database::create_usage_archive_pool().map_err(|e| {
                tracing::error!("[app] failed to create usage archive database pool: {e}");
                Box::new(e) as Box<dyn std::error::Error>
            })?;
            let llmusage = llmusage_adapter::LlmusageRuntime::discover().map_err(|e| {
                tracing::error!("[app] failed to resolve llmusage runtime paths: {e}");
                std::io::Error::other(e)
            })?;
            tracing::info!(
                llmusage_db = %llmusage.paths().db_path.display(),
                "[app] database initialized (global + app pool); llmusage will be read on demand"
            );

            // 构建并注册全局 AppState（启动期错误返 Result，不 panic 跨 FFI）。
            let app_state = AppState::try_new(db_pool, usage_db_pool, llmusage)
                .map_err(std::io::Error::other)?;

            // 先注册 Local 环境，其他环境在异步初始化完成后写入 managed state。
            app.manage(app_state);
            app.manage(std::sync::Arc::new(
                ccr_cli::services::install_service::InstallService::new(),
            ));
            desktop_shell::install_desktop_shell(app.handle()).map_err(std::io::Error::other)?;

            let tray_refresh_handle = app.handle().clone();
            spawn_supervised(
                tray_refresh_handle.clone(),
                "codex-tray:initial-refresh",
                async move {
                    if let Err(error) =
                        desktop_shell::refresh_codex_tray(&tray_refresh_handle, false).await
                    {
                        tracing::debug!("[tray] initial refresh skipped: {error}");
                    }
                },
            );

            crate::commands::codex::restore_pending_oauth_listener(app.handle().clone());

            // 异步初始化环境注册表，避免阻塞启动流程。
            let app_handle = app.handle().clone();
            spawn_supervised(app_handle.clone(), "env-registry:init", async move {
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

            let bg_supervisor_handle = app_handle.clone();
            spawn_supervised(bg_supervisor_handle, "background-maintenance", async move {
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
                let preferences = state.desktop_shell_preferences();
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
                    desktop_shell::MainWindowCloseAction::RequestQuit => {
                        /*
                         * ========================================================================
                         * 步骤1：显式退出桌面进程
                         * ========================================================================
                         * 目标：
                         * 1) 关闭到托盘未启用时，点 X 直接退出桌面进程
                         * 2) 不依赖 last-window 退出，避免隐藏 tray 面板让进程继续驻留
                         */
                        tracing::info!("[app] main window close requests desktop quit");

                        // 1.1 拦截默认 close，统一交给显式退出路径处理
                        api.prevent_close();

                        // 1.2 请求退出进程，并让 tray quit 复用同一条退出链路
                        if let Err(error) = desktop_shell::request_quit(window.app_handle()) {
                            tracing::warn!("[app] failed to request quit from close action: {error}");
                        }
                        tracing::info!("[app] desktop quit request dispatched from close action");
                    }
                    desktop_shell::MainWindowCloseAction::HideToTray => {
                        api.prevent_close();
                        if let Err(error) = window.hide() {
                            tracing::warn!("[app] failed to hide main window to tray: {error}");
                        }
                    }
                    desktop_shell::MainWindowCloseAction::ConfirmExit => {
                        api.prevent_close();
                        let window = window.clone();
                        let app_handle = window.app_handle().clone();
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
                                    /*
                                     * ========================================================================
                                     * 步骤2：确认后退出桌面进程
                                     * ========================================================================
                                     * 目标：
                                     * 1) 用户确认退出后进入显式退出路径
                                     * 2) 避免 window.close 只关闭主窗口而留下隐藏 tray 面板
                                     */
                                    tracing::info!("[app] confirmed close requests desktop quit");

                                    // 2.1 通过桌面壳层退出路径设置退出标志并关闭进程
                                    if let Err(error) = desktop_shell::request_quit(&app_handle) {
                                        tracing::warn!(
                                            "[app] failed to request quit after confirmation: {error}"
                                        );
                                    }
                                    tracing::info!(
                                        "[app] desktop quit request dispatched after confirmation"
                                    );
                                }
                            });
                    }
                }
            }
            WindowEvent::Focused(false) if desktop_shell::is_tray_panel_window(window.label()) => {
                let state = window.state::<AppState>();
                if state.tray_panel_drag_active() {
                    return;
                }
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
                ..DesktopShellPreferences::default()
            },
            false,
            false,
        );

        assert_eq!(action, MainWindowCloseAction::HideToTray);
    }

    #[test]
    fn close_action_requests_quit_when_close_to_tray_and_confirm_are_disabled() {
        let action = resolve_main_window_close_action(
            &DesktopShellPreferences {
                confirm_before_exit: false,
                close_to_tray: false,
                open_panel_on_tray_click: true,
                ..DesktopShellPreferences::default()
            },
            false,
            false,
        );

        assert_eq!(action, MainWindowCloseAction::RequestQuit);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_window_chrome_config_enables_native_decorations() {
        let config = super::macos_native_window_chrome_config();

        assert!(config.decorations);
        assert_eq!(config.title_bar_style, tauri::TitleBarStyle::Visible);
    }
}

/// 后台维护任务循环：60s 基础 tick，按 tick 数分频执行不同粒度的清理。
/// - 每 60s  : cache_cleanup
/// - 每 300s : ssh 运行时状态 + 密码缓存 cleanup（tick % 5 == 0）
/// - 每 600s : 监控日志 cleanup + usage import probe（tick % 10 == 0）
///   shutdown 信号到达时，最多等 60s 退出（tick 粒度）。
async fn run_background_tasks(app_handle: tauri::AppHandle, shutdown: Arc<Notify>) {
    tracing::info!("[background] starting background tasks (60s base tick)");

    let mut tick: u64 = 0;
    loop {
        tokio::select! {
            _ = shutdown.notified() => {
                tracing::info!("[background] shutdown signal received");
                break;
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
                if EXIT_REQUESTED.load(Ordering::SeqCst) {
                    break;
                }
                tick += 1;

                let state = app_handle.state::<AppState>();

                // 每 60s：LRU 缓存过期清理
                state.cache_cleanup().await;

                // 每 5min：SSH 运行时状态与密码缓存清理
                if tick.is_multiple_of(5) {
                    state
                        .cleanup_ssh_runtime_states(DEFAULT_SSH_STATE_TTL_SECS)
                        .await;
                    state
                        .cleanup_ssh_password_cache(DEFAULT_SSH_PASSWORD_TTL_SECS)
                        .await;
                }

                // 每 10min：监控日志清理 + usage import probe
                if tick.is_multiple_of(10) {
                    state.monitoring_logs.cleanup_old_logs().await;
                    if let Err(e) = verify_usage_storage_dir().await {
                        tracing::debug!("[background] usage storage dir check skipped: {e}");
                    }
                }

                tracing::debug!("[background] maintenance tick {tick}");
            }
        }
    }

    tracing::info!("[background] background tasks stopped");
}

/// 检查 Usage 导入所需的 JSONL 存储目录可访问。
///
/// 只校验目录可解析，真正的导入由其他命令触发；这里在启动期跑一次，便于
/// 在日志中尽早暴露 storage 配置错误。函数名旧版叫 `import_usage_data`，
/// 但事实上从来不做导入，已改名以反映其真实职责。
async fn verify_usage_storage_dir() -> Result<(), String> {
    // CostTracker 默认使用与 CLI 一致的目录 `~/.ccr/costs/`。
    tokio::task::spawn_blocking(|| {
        let _storage_dir = ccr_store::CostTracker::default_storage_dir()
            .map_err(|e| format!("Storage dir: {e}"))?;
        Ok(())
    })
    .await
    .map_err(|e| format!("Task join: {e}"))?
}
