// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod events;
mod platform;
mod ssh;
mod state;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tokio::sync::Notify;

use platform::local::LocalEnvironment;
use state::{AppState, DEFAULT_SSH_PASSWORD_TTL_SECS, DEFAULT_SSH_STATE_TTL_SECS};

/// 应用退出信号 — 用于通知后台任务停止
static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);

fn main() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!(
        "[app] CCR Desktop v{} starting (native Tauri mode)",
        env!("CARGO_PKG_VERSION")
    );

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // ── 初始化数据库 ──
            // 先初始化全局连接池（签到管理器等通过 with_connection() 使用）
            ccr_db::database::initialize().map_err(|e| {
                tracing::error!("[app] failed to initialize global database pool: {e}");
                Box::new(e) as Box<dyn std::error::Error>
            })?;
            // 再创建独立连接池给 AppState（SSH/环境管理等使用）
            let db_pool = ccr_db::database::create_app_pool().map_err(|e| {
                tracing::error!("[app] failed to create app database pool: {e}");
                Box::new(e) as Box<dyn std::error::Error>
            })?;
            tracing::info!("[app] database initialized (global + app pool)");

            // ── 构建 AppState ──
            let app_state = AppState::new(db_pool);

            // 注册 Local 环境（始终可用）
            // 注册 managed state
            app.manage(app_state);

            // 异步初始化环境注册表
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = app_handle.state::<AppState>();

                // Phase C1 — 自动检测 WSL 发行版并注册（仅 Windows）
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

                // Phase C2 — 加载已保存 SSH 主机并注册
                use ccr_db::database::repositories::ssh_repo;
                let db_pool = state.db_pool.clone();
                let hosts = match tokio::task::spawn_blocking(move || {
                    let conn = db_pool
                        .get()
                        .map_err(|e| format!("获取数据库连接失败: {e}"))?;
                    ssh_repo::list_hosts(&conn).map_err(|e| format!("读取 SSH 主机失败: {e}"))
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

                // 注册本地环境
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

            // ── 启动后台任务 ──
            let app_handle = app.handle().clone();
            let shutdown_notify = Arc::new(Notify::new());
            let shutdown_clone = shutdown_notify.clone();

            tauri::async_runtime::spawn(async move {
                run_background_tasks(app_handle, shutdown_clone).await;
            });

            // 保存 shutdown notify 以便退出时使用
            app.manage(shutdown_notify);

            tracing::info!("[app] setup complete");
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();

                // 如果已经确认过退出，直接放行（打破循环）
                if state
                    .exit_confirmed
                    .load(std::sync::atomic::Ordering::SeqCst)
                {
                    return;
                }

                // 检查是否跳过确认
                let skip_confirm = {
                    let settings = state.settings.lock().unwrap();
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
                    .message("确定要关闭 CCR Desktop 吗？")
                    .title("确认退出")
                    .kind(MessageDialogKind::Warning)
                    .buttons(MessageDialogButtons::OkCancelCustom(
                        "退出".to_string(),
                        "取消".to_string(),
                    ))
                    .show(move |confirmed| {
                        if confirmed {
                            // 设置标志位，防止 window.close() 再次触发对话框
                            let state = window.state::<AppState>();
                            state
                                .exit_confirmed
                                .store(true, std::sync::atomic::Ordering::SeqCst);
                            let _ = window.close();
                        }
                    });
            }
        })
        .invoke_handler(commands::generate_handler())
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app, event| {
        if let RunEvent::ExitRequested { .. } | RunEvent::Exit = event {
            tracing::info!("[app] exit requested, shutting down...");
            EXIT_REQUESTED.store(true, Ordering::SeqCst);

            // 通知后台任务停止
            if let Some(notify) = _app.try_state::<Arc<Notify>>() {
                notify.notify_waiters();
            }

            // 关闭数据库
            ccr_db::database::shutdown();
            tracing::info!("[app] cleanup complete");
        }
    });
}

/// 后台任务 — 定期执行维护操作
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

                // 定期清理过期缓存
                let state = app_handle.state::<AppState>();
                state.cache_cleanup().await;
                state
                    .cleanup_ssh_runtime_states(DEFAULT_SSH_STATE_TTL_SECS)
                    .await;
                state
                    .cleanup_ssh_password_cache(DEFAULT_SSH_PASSWORD_TTL_SECS)
                    .await;

                // 用量数据导入（静默，失败不阻塞）
                if let Err(e) = import_usage_data().await {
                    tracing::debug!("[background] usage import skipped: {e}");
                }

                tracing::debug!("[background] periodic maintenance completed");
            }
        }
    }

    tracing::info!("[background] background tasks stopped");
}

/// 后台用量数据导入 — 从各平台 JSONL 日志导入 cost 记录
async fn import_usage_data() -> Result<(), String> {
    // CostTracker 的数据由 CLI 工具自行记录到 ~/.ccr/costs/
    // 这里仅验证 storage dir 可用性作为健康检查
    tokio::task::spawn_blocking(|| {
        let _storage_dir =
            ccr::CostTracker::default_storage_dir().map_err(|e| format!("Storage dir: {e}"))?;
        Ok(())
    })
    .await
    .map_err(|e| format!("Task join: {e}"))?
}
