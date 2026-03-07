// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod checkin_jobs;
mod events;
mod monitoring;
mod platform;
mod process;
mod ssh;
mod state;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{Manager, RunEvent, WindowEvent};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tokio::sync::Notify;

use platform::local::LocalEnvironment;
use state::{AppState, DEFAULT_SSH_PASSWORD_TTL_SECS, DEFAULT_SSH_STATE_TTL_SECS};

/// 閹煎瓨姊婚弫銈夋焻閳ь剟宕欐潪棰佺箚闁?闁?闁活潿鍔嬬花顒勬焻濮樿京鍙€闁告艾楠歌ぐ瀛樼鐠囨彃顫ら柛瀣矋椤?
static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);

fn main() {
    ccr::init_logger();

    tracing::info!(
        "[app] CCR Desktop v{} starting (native Tauri mode)",
        env!("CARGO_PKG_VERSION")
    );

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 闁冲厜鍋撻柍鍏夊亾 闁告帗绻傞～鎰板礌閺嶃劍娈堕柟璇″枛缁?闁冲厜鍋撻柍鍏夊亾
            // 闁稿繐鐗嗛崹鍨叏鐎ｎ亜顕ч柛蹇嬪妼閻剚娼婚悙鏉戝婵湱濯寸槐娆戠驳閹冪厒缂佺媴绱曢幃濠囧闯閵娧呮惣闂侇偅淇虹换?with_connection() 濞达綀娉曢弫銈夋晬?
            ccr_db::database::initialize().map_err(|e| {
                tracing::error!("[app] failed to initialize global database pool: {e}");
                Box::new(e) as Box<dyn std::error::Error>
            })?;
            // 闁告劕绉撮崹鍗烆嚈閾忕懓顏紒鏂款儓缁绘盯骞掗妷锔炬建缂?AppState闁挎稑婀疭H/闁绘粠鍨伴。銊х不閿涘嫭鍊炵紒娑橆槷婵炲洭鎮介…鎺旂
            let db_pool = ccr_db::database::create_app_pool().map_err(|e| {
                tracing::error!("[app] failed to create app database pool: {e}");
                Box::new(e) as Box<dyn std::error::Error>
            })?;
            tracing::info!("[app] database initialized (global + app pool)");

            // 闁冲厜鍋撻柍鍏夊亾 闁哄瀚紓?AppState 闁冲厜鍋撻柍鍏夊亾
            let app_state = AppState::new(db_pool);

            // 婵炲鍔岄崬?Local 闁绘粠鍨伴。銊╂晬閸繍娼楃紓浣哥墕瑜版煡鎮介…鎺旂
            // 婵炲鍔岄崬?managed state
            app.manage(app_state);

            // 鐎殿喖鍊归鐐哄礆濠靛棭娼楅柛鏍ㄧ墱楠炲棙鏅堕崘鈺傛殘闁告劕鐭侀妴?
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let state = app_handle.state::<AppState>();

                // Phase C1 闁?闁煎浜滄慨鈺佄涢埀顒€霉?WSL 闁告瑦鍨奸、鎴︽偋閸繆瀚欐繛澶堝妼閸炰粙鏁嶉崼婊呯煂 Windows闁?
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

                // Phase C2 闁?闁告梻濮惧ù鍥ь啅闊厾绠介悗?SSH 濞戞挾绮┃鈧鐐跺煐閺佺偤宕?
                use ccr_db::database::repositories::ssh_repo;
                let db_pool = state.db_pool.clone();
                let hosts = match tokio::task::spawn_blocking(move || {
                    let conn = db_pool
                        .get()
                        .map_err(|e| format!("闁兼儳鍢茶ぐ鍥极閻楀牆绁﹂幖瀛樻崄缁绘盯骞掗妷銉ｄ杭閻? {e}"))?;
                    ssh_repo::list_hosts(&conn).map_err(|e| format!("閻犲洩顕цぐ?SSH 濞戞挾绮┃鈧鎯扮簿鐟? {e}"))
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

                // 婵炲鍔岄崬浠嬪嫉椤掆偓濠€鎾偝椤栨凹鏆?
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

            // 闁冲厜鍋撻柍鍏夊亾 闁告凹鍨版慨鈺呭触鎼粹€抽叡濞寸姾顕ф慨?闁冲厜鍋撻柍鍏夊亾
            let app_handle = app.handle().clone();
            let shutdown_notify = Arc::new(Notify::new());
            let shutdown_clone = shutdown_notify.clone();

            tauri::async_runtime::spawn(async move {
                run_background_tasks(app_handle, shutdown_clone).await;
            });

            // 濞ｅ洦绻傞悺?shutdown notify 濞寸姰鍎扮粚鍫曟焻閳ь剟宕欓悜妯活槯濞达綀娉曢弫?
            app.manage(shutdown_notify);

            tracing::info!("[app] setup complete");
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<AppState>();

                // 濠碘€冲€归悘澶婎啅閼碱剛鐥呯痪顓у枦椤撶粯娼婚崶顑藉亾閳ь剟宕欓悮瀵哥闁烩晛鐡ㄧ敮鎾绩閹规劦鏀介柨娑樼墛婢э箓鎯嶉弶鎴炲剷闁绘粠鍨界槐?
                if state
                    .exit_confirmed
                    .load(std::sync::atomic::Ordering::SeqCst)
                {
                    return;
                }

                // 婵☆偀鍋撻柡灞诲劜濡叉悂宕ラ敃浣哄劜閺夆晛娲ㄩ垾妯兼媼?
                let skip_confirm = {
                    let settings = state.settings.lock().unwrap();
                    settings.skip_exit_confirm
                };

                if skip_confirm {
                    return;
                }

                // 闂傚啰绮娑欘渶濡鍚囬柛蹇斿▕濡挳鏁嶇仦鐐枖缂佲偓閾忓厜鈧鎷嬮妶鍜佸殸閻犲洦绻冮、?
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
                            // 閻犱礁澧介悿鍡涘冀閸パ呯濞达絽绋勭槐婵嬫⒓閸欏鍓?window.close() 闁告劕绉甸鑲╂喆閿曗偓瑜板倻鈧數顢婇惁钘夘浖?
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

            // 闂侇偅姘ㄩ悡锟犲触鎼粹€抽叡濞寸姾顕ф慨鐔煎磻濠婂嫷鍓?
            if let Some(notify) = _app.try_state::<Arc<Notify>>() {
                notify.notify_waiters();
            }

            // 闁稿繑濞婂Λ鎾极閻楀牆绁﹂幖?
            ccr_db::database::shutdown();
            tracing::info!("[app] cleanup complete");
        }
    });
}

/// 闁告艾楠歌ぐ瀛樼鐠囨彃顫?闁?閻庤纰嶅﹢锟犲箥瑜戦、鎴犵磼鐎涙ê袘闁瑰灝绉崇紞?
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

                // 閻庤纰嶅﹢鈥炽€掗崨顖涘€為弶鈺佹处濠€锛勭磽閹惧磭鎽?
                let state = app_handle.state::<AppState>();
                state.cache_cleanup().await;
                state
                    .cleanup_ssh_runtime_states(DEFAULT_SSH_STATE_TTL_SECS)
                    .await;
                state
                    .cleanup_ssh_password_cache(DEFAULT_SSH_PASSWORD_TTL_SECS)
                    .await;
                state.monitoring_logs.cleanup_old_logs().await;

                // 闁活潿鍔戦崳娲极閻楀牆绁﹂悗鐢靛帶閸欏棝鏁嶉崼銉﹂ク濮掓稒锕槐婵囧緞鏉堫偉袝濞戞挸绉瑰Ο鍡樼箙閻戝洨绀?
                if let Err(e) = import_usage_data().await {
                    tracing::debug!("[background] usage import skipped: {e}");
                }

                tracing::debug!("[background] periodic maintenance completed");
            }
        }
    }

    tracing::info!("[background] background tasks stopped");
}

/// 闁告艾楠歌ぐ鎾偨閵娾晛娅ら柡浣哄瀹撲胶鈧數鍘ч崣?闁?濞寸姴楠搁幃鍥嵁閸愭彃閰?JSONL 闁哄啨鍎辩换鏃傗偓鐢靛帶閸?cost 閻犱焦婢樼紞?
async fn import_usage_data() -> Result<(), String> {
    // CostTracker 闁汇劌瀚弳鐔煎箲椤旂偓鏆?CLI 鐎规悶鍎遍崣鍧楁嚊椤忓浂鏀介悹浣规緲缂嶅秹宕?~/.ccr/costs/
    // 閺夆晜鐟╅崳閿嬬閸涘宕ｉ悹?storage dir 闁告瑯鍨抽弫銈夊箑瑜岀紞鏃€绋夐崫鍕樊閹兼挳鏀遍ˉ鍛村蓟?
    tokio::task::spawn_blocking(|| {
        let _storage_dir =
            ccr::CostTracker::default_storage_dir().map_err(|e| format!("Storage dir: {e}"))?;
        Ok(())
    })
    .await
    .map_err(|e| format!("Task join: {e}"))?
}


