//! 桌面壳层 —— tray、hide-to-tray 与紧凑面板窗口。

use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{
    AppHandle, Emitter, Manager, Monitor, PhysicalPosition, PhysicalSize, Runtime, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};

use ccr_codex::CodexAuthService;

use crate::commands::codex::{
    CodexTraySnapshot, compute_codex_tray_snapshot, invalidate_codex_dashboard_overview_cache,
};
use crate::state::{
    AppState, DesktopShellPreferences, TrayAnchor, TrayPanelManualPosition, TrayPanelPlacementMode,
};
use crate::{configure_main_window_chrome, open_devtools_in_debug};

pub const TRAY_ICON_ID: &str = "ccr-tray";
pub const TRAY_PANEL_WINDOW_LABEL: &str = "codex-tray-panel";

const MAIN_WINDOW_LABEL: &str = "main";
const TRAY_PANEL_WIDTH: i32 = 456;
const TRAY_PANEL_HEIGHT: i32 = 620;
const TRAY_PANEL_MARGIN: i32 = 12;

const MENU_OPEN_MAIN: &str = "tray.open-main";
const MENU_OPEN_PANEL: &str = "tray.open-panel";
const MENU_OPEN_USAGE: &str = "tray.open-usage";
const MENU_OPEN_AUTH: &str = "tray.open-auth";
const MENU_REFRESH_CODEX: &str = "tray.refresh-codex";
const MENU_QUIT: &str = "tray.quit";
const MENU_ACCOUNT_STATUS: &str = "tray.account-status";
const MENU_QUOTA_STATUS: &str = "tray.quota-status";
const MENU_SWITCH_AUTH_SUBMENU: &str = "tray.switch-auth";
const MENU_SWITCH_AUTH_PREFIX: &str = "tray.switch-auth.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PanelWorkArea {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainWindowCloseAction {
    AllowExit,
    RequestQuit,
    HideToTray,
    ConfirmExit,
}

pub fn is_tray_panel_window(label: &str) -> bool {
    label == TRAY_PANEL_WINDOW_LABEL
}

pub fn resolve_main_window_close_action(
    preferences: &DesktopShellPreferences,
    exit_confirmed: bool,
    force_exit_requested: bool,
) -> MainWindowCloseAction {
    if exit_confirmed || force_exit_requested {
        return MainWindowCloseAction::AllowExit;
    }

    if preferences.close_to_tray {
        return MainWindowCloseAction::HideToTray;
    }

    if preferences.confirm_before_exit {
        return MainWindowCloseAction::ConfirmExit;
    }

    MainWindowCloseAction::RequestQuit
}

pub fn install_desktop_shell<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    ensure_tray_panel_window(app)?;

    let placeholder_menu = build_loading_tray_menu(app)?;
    let mut tray_builder = TrayIconBuilder::with_id(TRAY_ICON_ID)
        .menu(&placeholder_menu)
        .tooltip("CCR Desktop")
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let app_handle = app.clone();
            let menu_id = event.id().as_ref().to_string();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = handle_tray_menu_event(&app_handle, &menu_id).await {
                    tracing::warn!("[tray] menu event failed ({menu_id}): {error}");
                }
            });
        })
        .on_tray_icon_event(|tray, event| {
            let app_handle = tray.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = handle_tray_icon_event(&app_handle, event).await {
                    tracing::warn!("[tray] icon event failed: {error}");
                }
            });
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray_builder = tray_builder.icon(icon);
    }

    tray_builder
        .build(app)
        .map_err(|e| format!("创建 tray 图标失败: {e}"))?;

    Ok(())
}

pub async fn refresh_codex_tray<R: Runtime>(app: &AppHandle<R>, force: bool) -> Result<(), String> {
    let tray = app
        .tray_by_id(TRAY_ICON_ID)
        .ok_or_else(|| "找不到 tray 图标".to_string())?;

    match compute_codex_tray_snapshot(force).await {
        Ok(snapshot) => {
            app.state::<AppState>().set_tray_switch_accounts(
                snapshot
                    .accounts
                    .iter()
                    .map(|item| item.name.clone())
                    .collect(),
            );
            let menu = build_snapshot_tray_menu(app, &snapshot)?;
            tray.set_menu(Some(menu))
                .map_err(|e| format!("更新 tray 菜单失败: {e}"))?;
            emit_codex_tray_refresh(app, &snapshot)?;
            Ok(())
        }
        Err(error) => {
            app.state::<AppState>().set_tray_switch_accounts(Vec::new());
            let menu = build_error_tray_menu(app, &error)?;
            tray.set_menu(Some(menu))
                .map_err(|e| format!("更新 tray 菜单失败: {e}"))?;
            Err(error)
        }
    }
}

pub fn emit_codex_tray_refresh<R: Runtime>(
    app: &AppHandle<R>,
    snapshot: &CodexTraySnapshot,
) -> Result<(), String> {
    app.emit_to(TRAY_PANEL_WINDOW_LABEL, "codex-tray:refresh", snapshot)
        .map_err(|e| format!("广播 tray 刷新事件失败: {e}"))
}

pub async fn toggle_tray_panel<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let panel = ensure_tray_panel_window(app)?;
    if panel
        .is_visible()
        .map_err(|e| format!("读取 tray 面板可见状态失败: {e}"))?
    {
        hide_tray_panel(app)?;
        return Ok(());
    }

    show_tray_panel(app).await
}

pub async fn show_tray_panel<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let panel = ensure_tray_panel_window(app)?;
    position_tray_panel_window(app, &panel)?;
    panel
        .show()
        .map_err(|e| format!("显示 tray 面板失败: {e}"))?;
    panel
        .unminimize()
        .map_err(|e| format!("恢复 tray 面板失败: {e}"))?;
    panel
        .set_focus()
        .map_err(|e| format!("聚焦 tray 面板失败: {e}"))?;

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = refresh_codex_tray(&app_handle, false).await {
            tracing::debug!("[tray] refresh while opening panel skipped: {error}");
        }
    });

    Ok(())
}

pub fn hide_tray_panel<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if let Some(panel) = app.get_webview_window(TRAY_PANEL_WINDOW_LABEL) {
        panel
            .hide()
            .map_err(|e| format!("隐藏 tray 面板失败: {e}"))?;
    }

    Ok(())
}

pub async fn show_main_window<R: Runtime>(
    app: &AppHandle<R>,
    target_route: Option<&str>,
) -> Result<(), String> {
    hide_tray_panel(app)?;

    let main_window = ensure_main_window(app)?;
    if main_window
        .is_minimized()
        .map_err(|e| format!("读取主窗口最小化状态失败: {e}"))?
    {
        main_window
            .unminimize()
            .map_err(|e| format!("恢复主窗口失败: {e}"))?;
    }

    main_window
        .show()
        .map_err(|e| format!("显示主窗口失败: {e}"))?;
    main_window
        .set_focus()
        .map_err(|e| format!("聚焦主窗口失败: {e}"))?;

    if let Some(route) = target_route.filter(|route| !route.trim().is_empty()) {
        main_window
            .emit("shell:navigate", route.to_string())
            .map_err(|e| format!("发送主窗口导航事件失败: {e}"))?;
    }

    Ok(())
}

pub fn request_quit<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let state = app.state::<AppState>();
    state
        .force_exit_requested
        .store(true, std::sync::atomic::Ordering::SeqCst);
    state
        .exit_confirmed
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let _ = hide_tray_panel(app);
    app.exit(0);
    Ok(())
}

pub async fn handle_tray_icon_event<R: Runtime>(
    app: &AppHandle<R>,
    event: TrayIconEvent,
) -> Result<(), String> {
    match event {
        TrayIconEvent::Click {
            position,
            button,
            button_state,
            ..
        } => {
            update_tray_anchor(
                app,
                TrayAnchor {
                    x: position.x as i32,
                    y: position.y as i32,
                    width: 0,
                    height: 0,
                },
            )?;

            if button == MouseButton::Left && button_state == MouseButtonState::Up {
                let preferences = app.state::<AppState>().desktop_shell_preferences();
                if preferences.open_panel_on_tray_click {
                    toggle_tray_panel(app).await?;
                } else {
                    app.state::<AppState>().set_tray_anchor(Some(TrayAnchor {
                        x: position.x as i32,
                        y: position.y as i32,
                        width: 0,
                        height: 0,
                    }));
                    show_main_window(app, None).await?;
                }
            }
        }
        TrayIconEvent::Enter { position, .. } | TrayIconEvent::Move { position, .. } => {
            update_tray_anchor(
                app,
                TrayAnchor {
                    x: position.x as i32,
                    y: position.y as i32,
                    width: 0,
                    height: 0,
                },
            )?;
        }
        TrayIconEvent::DoubleClick { .. } | TrayIconEvent::Leave { .. } => {}
        _ => {}
    }

    Ok(())
}

pub async fn handle_tray_menu_event<R: Runtime>(
    app: &AppHandle<R>,
    menu_id: &str,
) -> Result<(), String> {
    match menu_id {
        MENU_OPEN_MAIN => show_main_window(app, None).await,
        MENU_OPEN_PANEL => show_tray_panel(app).await,
        MENU_OPEN_USAGE => show_main_window(app, Some("/usage")).await,
        MENU_OPEN_AUTH => show_main_window(app, Some("/codex/auth")).await,
        MENU_REFRESH_CODEX => refresh_codex_tray(app, true).await,
        MENU_QUIT => request_quit(app),
        _ if menu_id.starts_with(MENU_SWITCH_AUTH_PREFIX) => {
            switch_auth_from_menu(app, menu_id).await
        }
        _ => Ok(()),
    }
}

async fn switch_auth_from_menu<R: Runtime>(
    app: &AppHandle<R>,
    menu_id: &str,
) -> Result<(), String> {
    let Some(index_text) = menu_id.strip_prefix(MENU_SWITCH_AUTH_PREFIX) else {
        return Ok(());
    };
    let index = index_text
        .parse::<usize>()
        .map_err(|e| format!("解析账户菜单序号失败: {e}"))?;
    let Some(account_name) = app.state::<AppState>().tray_switch_account_name(index) else {
        return Ok(());
    };

    let account_name_for_task = account_name.clone();
    tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;
        service
            .switch_account(&account_name_for_task)
            .map_err(|e| format!("{e}"))
    })
    .await
    .map_err(|e| format!("切换 Codex Auth 任务失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&app.state::<AppState>()).await;
    refresh_codex_tray(app, true).await?;
    Ok(())
}

fn update_tray_anchor<R: Runtime>(app: &AppHandle<R>, anchor: TrayAnchor) -> Result<(), String> {
    app.state::<AppState>().set_tray_anchor(Some(anchor));
    Ok(())
}

fn ensure_tray_panel_window<R: Runtime>(app: &AppHandle<R>) -> Result<WebviewWindow<R>, String> {
    if let Some(window) = app.get_webview_window(TRAY_PANEL_WINDOW_LABEL) {
        return Ok(window);
    }

    #[allow(unused_mut)]
    let mut builder = WebviewWindowBuilder::new(
        app,
        TRAY_PANEL_WINDOW_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .title("CCR Codex Tray")
    .inner_size(TRAY_PANEL_WIDTH as f64, TRAY_PANEL_HEIGHT as f64)
    .resizable(false)
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(false)
    .focused(false);

    #[cfg(target_os = "macos")]
    {
        builder = builder.hidden_title(true);
    }

    let window = builder
        .build()
        .map_err(|e| format!("创建 tray 面板失败: {e}"))?;

    window
        .set_always_on_top(true)
        .map_err(|e| format!("设置 tray 面板置顶失败: {e}"))?;
    window
        .set_skip_taskbar(true)
        .map_err(|e| format!("设置 tray 面板跳过任务栏失败: {e}"))?;

    Ok(window)
}

fn position_tray_panel_window<R: Runtime>(
    app: &AppHandle<R>,
    panel: &WebviewWindow<R>,
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let preferences = state.desktop_shell_preferences();

    if preferences.tray_panel.placement_mode == TrayPanelPlacementMode::Manual {
        let manual_position = preferences.tray_panel.manual_position.as_ref();
        let manual_monitor = match manual_position {
            Some(position) => panel
                .monitor_from_point(position.x as f64, position.y as f64)
                .map_err(|e| format!("定位 tray 手动位置所在显示器失败: {e}"))?,
            None => None,
        };
        let manual_work_area = manual_monitor.as_ref().map(work_area_from_monitor);
        let manual_size = resolve_panel_size_for_work_area(manual_work_area.as_ref());

        if let Some(target) = resolve_manual_panel_position_for_work_area(
            manual_position,
            manual_work_area.as_ref(),
            &manual_size,
        ) {
            if manual_position
                .is_some_and(|position| position.x != target.x || position.y != target.y)
            {
                state.set_tray_panel_manual_position(TrayPanelManualPosition {
                    x: target.x,
                    y: target.y,
                })?;
            }

            panel
                .set_size(manual_size)
                .map_err(|e| format!("调整 tray 面板尺寸失败: {e}"))?;
            panel
                .set_position(target)
                .map_err(|e| format!("定位 tray 面板失败: {e}"))?;
            return Ok(());
        }

        state.reset_tray_panel_manual_position()?;
    }

    let anchor = state.tray_anchor();
    let monitor = monitor_for_anchor(panel, anchor.as_ref())?;
    let work_area = monitor.as_ref().map(work_area_from_monitor);
    let size = resolve_panel_size_for_work_area(work_area.as_ref());
    let target = resolve_panel_position_for_work_area(anchor.as_ref(), work_area.as_ref(), &size);

    panel
        .set_size(size)
        .map_err(|e| format!("调整 tray 面板尺寸失败: {e}"))?;

    panel
        .set_position(target)
        .map_err(|e| format!("定位 tray 面板失败: {e}"))?;
    Ok(())
}

fn monitor_for_anchor<R: Runtime>(
    panel: &WebviewWindow<R>,
    anchor: Option<&TrayAnchor>,
) -> Result<Option<Monitor>, String> {
    // 优先使用 tray 锚点命中的显示器；这样多显示器场景下即便主窗口缺席也能贴对屏。
    if let Some(anchor) = anchor
        && let Some(monitor) = panel
            .monitor_from_point(anchor.x as f64, anchor.y as f64)
            .map_err(|e| format!("定位 tray 所在显示器失败: {e}"))?
    {
        return Ok(Some(monitor));
    }

    // 回落到 panel 窗口当前所在的显示器。
    if let Some(monitor) = panel
        .current_monitor()
        .map_err(|e| format!("读取 tray 面板当前显示器失败: {e}"))?
    {
        return Ok(Some(monitor));
    }

    // 最后兜底主显示器；如果系统无显示器则返回 None。
    panel
        .primary_monitor()
        .map_err(|e| format!("读取主显示器失败: {e}"))
}

fn resolve_panel_position_for_work_area(
    anchor: Option<&TrayAnchor>,
    work_area: Option<&PanelWorkArea>,
    panel_size: &PhysicalSize<u32>,
) -> PhysicalPosition<i32> {
    let panel_width = panel_size.width as i32;
    let panel_height = panel_size.height as i32;
    let fallback = work_area
        .map(|work_area| {
            fallback_panel_position_for_work_area(work_area, panel_width, panel_height)
        })
        .unwrap_or_else(|| PhysicalPosition::new(64, 64));

    let Some(anchor) = anchor else {
        return fallback;
    };

    let Some(work_area) = work_area else {
        // 无显示器信息时优先把面板抬到锚点上方，避免任务栏在底部时面板坠出屏外。
        let y = (anchor.y - panel_height - TRAY_PANEL_MARGIN).max(0);
        return PhysicalPosition::new((anchor.x - (panel_width / 2)).max(0), y);
    };

    let min_x = work_area.x;
    let min_y = work_area.y;
    let max_x = work_area.x + work_area.width - panel_width;
    let max_y = work_area.y + work_area.height - panel_height;

    let mut x = anchor.x + anchor.width as i32 - panel_width;
    x = x.clamp(min_x, max_x.max(min_x));

    let below_y = anchor.y + anchor.height as i32 + TRAY_PANEL_MARGIN;
    let above_y = anchor.y - panel_height - TRAY_PANEL_MARGIN;
    let y = if (min_y..=max_y).contains(&below_y) {
        below_y
    } else if (min_y..=max_y).contains(&above_y) {
        above_y
    } else {
        below_y.clamp(min_y, max_y.max(min_y))
    };

    PhysicalPosition::new(x, y)
}

fn resolve_manual_panel_position_for_work_area(
    manual_position: Option<&TrayPanelManualPosition>,
    work_area: Option<&PanelWorkArea>,
    panel_size: &PhysicalSize<u32>,
) -> Option<PhysicalPosition<i32>> {
    let manual_position = manual_position?;
    let work_area = work_area?;
    let panel_width = panel_size.width as i32;
    let panel_height = panel_size.height as i32;
    let min_x = work_area.x;
    let min_y = work_area.y;
    let max_x = (work_area.x + work_area.width - panel_width).max(min_x);
    let max_y = (work_area.y + work_area.height - panel_height).max(min_y);

    Some(PhysicalPosition::new(
        manual_position.x.clamp(min_x, max_x),
        manual_position.y.clamp(min_y, max_y),
    ))
}

fn work_area_from_monitor(monitor: &Monitor) -> PanelWorkArea {
    let work_area = monitor.work_area();
    PanelWorkArea {
        x: work_area.position.x,
        y: work_area.position.y,
        width: work_area.size.width as i32,
        height: work_area.size.height as i32,
    }
}

fn resolve_panel_size_for_work_area(work_area: Option<&PanelWorkArea>) -> PhysicalSize<u32> {
    let Some(work_area) = work_area else {
        return PhysicalSize::new(TRAY_PANEL_WIDTH as u32, TRAY_PANEL_HEIGHT as u32);
    };

    let width = (work_area.width - (TRAY_PANEL_MARGIN * 2)).max(1);
    let height = (work_area.height - (TRAY_PANEL_MARGIN * 2)).max(1);

    PhysicalSize::new(
        TRAY_PANEL_WIDTH.min(width) as u32,
        TRAY_PANEL_HEIGHT.min(height) as u32,
    )
}

fn fallback_panel_position_for_work_area(
    work_area: &PanelWorkArea,
    panel_width: i32,
    _panel_height: i32,
) -> PhysicalPosition<i32> {
    let x = work_area.x + work_area.width - panel_width - TRAY_PANEL_MARGIN;
    let y = work_area.y + TRAY_PANEL_MARGIN;
    PhysicalPosition::new(x.max(work_area.x), y.max(work_area.y))
}

fn ensure_main_window<R: Runtime>(app: &AppHandle<R>) -> Result<WebviewWindow<R>, String> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        return Ok(window);
    }

    // 主 webview 已被销毁（用户确认退出/关闭），按 tauri.conf.json 的主窗口配置重建，
    // 并复用 main.rs 里的 chrome 装饰逻辑，保持和首次启动一致的标题栏与边框。
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == MAIN_WINDOW_LABEL)
        .cloned()
        .ok_or_else(|| "tauri.conf.json 中缺少主窗口配置".to_string())?;

    let window = WebviewWindowBuilder::from_config(app, &config)
        .map_err(|e| format!("读取主窗口配置失败: {e}"))?
        .build()
        .map_err(|e| format!("重建主窗口失败: {e}"))?;

    configure_main_window_chrome(&window).map_err(|e| format!("初始化主窗口 chrome 失败: {e}"))?;
    open_devtools_in_debug(&window);

    Ok(window)
}

fn build_loading_tray_menu<R: Runtime, M: Manager<R>>(
    manager: &M,
) -> Result<tauri::menu::Menu<R>, String> {
    build_tray_menu_with_state(
        manager,
        "Codex tray loading…",
        "Refreshing account status…",
        None,
        false,
        None,
    )
}

fn build_error_tray_menu<R: Runtime, M: Manager<R>>(
    manager: &M,
    error: &str,
) -> Result<tauri::menu::Menu<R>, String> {
    build_tray_menu_with_state(
        manager,
        "Codex tray unavailable",
        &truncate_text(error, 72),
        None,
        false,
        Some("Refresh failed"),
    )
}

fn build_snapshot_tray_menu<R: Runtime, M: Manager<R>>(
    manager: &M,
    snapshot: &CodexTraySnapshot,
) -> Result<tauri::menu::Menu<R>, String> {
    build_tray_menu_with_state(
        manager,
        &snapshot_menu_account_line(snapshot),
        &snapshot_menu_quota_line(snapshot),
        Some(snapshot),
        snapshot.can_manage_accounts,
        None,
    )
}

fn build_tray_menu_with_state<R: Runtime, M: Manager<R>>(
    manager: &M,
    account_line: &str,
    quota_line: &str,
    snapshot: Option<&CodexTraySnapshot>,
    can_manage_accounts: bool,
    switch_hint: Option<&str>,
) -> Result<tauri::menu::Menu<R>, String> {
    let account_item =
        MenuItemBuilder::with_id(MENU_ACCOUNT_STATUS, truncate_text(account_line, 72))
            .enabled(false)
            .build(manager)
            .map_err(|e| format!("构建 tray 账号状态菜单失败: {e}"))?;
    let quota_item = MenuItemBuilder::with_id(MENU_QUOTA_STATUS, truncate_text(quota_line, 72))
        .enabled(false)
        .build(manager)
        .map_err(|e| format!("构建 tray 配额状态菜单失败: {e}"))?;

    let switch_submenu =
        build_switch_auth_submenu(manager, snapshot, can_manage_accounts, switch_hint)?;
    let open_main_item = MenuItemBuilder::with_id(MENU_OPEN_MAIN, "Open CCR")
        .build(manager)
        .map_err(|e| format!("构建 Open CCR 菜单失败: {e}"))?;
    let open_panel_item = MenuItemBuilder::with_id(MENU_OPEN_PANEL, "Open Codex Panel")
        .build(manager)
        .map_err(|e| format!("构建 Open Codex Panel 菜单失败: {e}"))?;
    let open_usage_item = MenuItemBuilder::with_id(MENU_OPEN_USAGE, "Open Usage Dashboard")
        .build(manager)
        .map_err(|e| format!("构建 Open Usage Dashboard 菜单失败: {e}"))?;
    let open_auth_item = MenuItemBuilder::with_id(MENU_OPEN_AUTH, "Open Codex Auth")
        .build(manager)
        .map_err(|e| format!("构建 Open Codex Auth 菜单失败: {e}"))?;
    let refresh_item = MenuItemBuilder::with_id(MENU_REFRESH_CODEX, "Refresh Codex")
        .build(manager)
        .map_err(|e| format!("构建 Refresh Codex 菜单失败: {e}"))?;
    let quit_item = MenuItemBuilder::with_id(MENU_QUIT, "Quit")
        .build(manager)
        .map_err(|e| format!("构建 Quit 菜单失败: {e}"))?;

    MenuBuilder::new(manager)
        .item(&account_item)
        .item(&quota_item)
        .separator()
        .item(&open_main_item)
        .item(&open_panel_item)
        .item(&open_usage_item)
        .item(&open_auth_item)
        .separator()
        .item(&refresh_item)
        .item(&switch_submenu)
        .separator()
        .item(&quit_item)
        .build()
        .map_err(|e| format!("构建 tray 菜单失败: {e}"))
}

fn build_switch_auth_submenu<R: Runtime, M: Manager<R>>(
    manager: &M,
    snapshot: Option<&CodexTraySnapshot>,
    can_manage_accounts: bool,
    switch_hint: Option<&str>,
) -> Result<tauri::menu::Submenu<R>, String> {
    let mut builder =
        SubmenuBuilder::with_id(manager, MENU_SWITCH_AUTH_SUBMENU, "Switch Codex Auth");

    if !can_manage_accounts {
        let hint = MenuItemBuilder::with_id(
            format!("{MENU_SWITCH_AUTH_PREFIX}unsupported"),
            switch_hint.unwrap_or("Current profile does not use OpenAI auth"),
        )
        .enabled(false)
        .build(manager)
        .map_err(|e| format!("构建 Auth 限制提示菜单失败: {e}"))?;
        builder = builder.item(&hint);
        return builder
            .build()
            .map_err(|e| format!("构建 Auth 切换子菜单失败: {e}"));
    }

    let Some(snapshot) = snapshot else {
        let loading_item = MenuItemBuilder::with_id(
            format!("{MENU_SWITCH_AUTH_PREFIX}loading"),
            "Loading accounts…",
        )
        .enabled(false)
        .build(manager)
        .map_err(|e| format!("构建 Auth 加载菜单失败: {e}"))?;
        builder = builder.item(&loading_item);
        return builder
            .build()
            .map_err(|e| format!("构建 Auth 切换子菜单失败: {e}"));
    };

    if snapshot.accounts.is_empty() {
        let empty_item = MenuItemBuilder::with_id(
            format!("{MENU_SWITCH_AUTH_PREFIX}empty"),
            "No saved accounts",
        )
        .enabled(false)
        .build(manager)
        .map_err(|e| format!("构建空账号菜单失败: {e}"))?;
        builder = builder.item(&empty_item);
        return builder
            .build()
            .map_err(|e| format!("构建 Auth 切换子菜单失败: {e}"));
    }

    let switch_items: Vec<_> = snapshot
        .accounts
        .iter()
        .enumerate()
        .map(|(index, account)| {
            let label = build_account_switch_label(account);
            CheckMenuItemBuilder::with_id(format!("{MENU_SWITCH_AUTH_PREFIX}{index}"), label)
                .checked(account.is_current)
                .enabled(account.can_switch)
                .build(manager)
                .map_err(|e| format!("构建账号切换菜单失败: {e}"))
        })
        .collect::<Result<_, _>>()?;

    for item in &switch_items {
        builder = builder.item(item);
    }

    builder
        .build()
        .map_err(|e| format!("构建 Auth 切换子菜单失败: {e}"))
}

fn build_account_switch_label(account: &crate::commands::codex::CodexTrayAccountRow) -> String {
    let mut parts = vec![account.name.clone()];
    if let Some(email) = account
        .email
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        parts.push(email.to_string());
    }
    if let Some(last_refresh) = account
        .last_refresh
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        parts.push(format!("refreshed {last_refresh}"));
    }
    truncate_text(&parts.join(" · "), 64)
}

fn snapshot_menu_account_line(snapshot: &CodexTraySnapshot) -> String {
    let current = snapshot
        .current_account
        .as_ref()
        .map(|account| {
            account
                .email
                .clone()
                .unwrap_or_else(|| account.name.clone())
        })
        .unwrap_or_else(|| snapshot.auth_label.clone());
    format!("Account: {}", truncate_text(&current, 56))
}

fn snapshot_menu_quota_line(snapshot: &CodexTraySnapshot) -> String {
    let Some(current_account) = snapshot.current_account.as_ref() else {
        return format!("Profile: {}", truncate_text(&snapshot.profile_label, 56));
    };

    if let Some(quota) = current_account.quota.as_ref() {
        let hourly = format!("5h {}%", quota.hourly_percentage);
        let weekly = format!("Week {}%", quota.weekly_percentage);
        let plan = quota
            .plan_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string());

        let mut parts = vec![hourly, weekly];
        if let Some(plan) = plan {
            parts.push(plan);
        }
        return truncate_text(&parts.join(" · "), 64);
    }

    if let Some(error) = current_account.quota_error.as_deref() {
        return truncate_text(error, 64);
    }

    format!("Profile: {}", truncate_text(&snapshot.profile_label, 56))
}

fn truncate_text(value: &str, max_chars: usize) -> String {
    let mut iter = value.chars();
    let truncated: String = iter.by_ref().take(max_chars).collect();
    if iter.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DesktopShellPreferences, MainWindowCloseAction, PanelWorkArea, TRAY_PANEL_HEIGHT,
        TRAY_PANEL_WIDTH, TrayAnchor, resolve_main_window_close_action,
        resolve_manual_panel_position_for_work_area, resolve_panel_position_for_work_area,
        resolve_panel_size_for_work_area,
    };
    use crate::state::TrayPanelManualPosition;
    use tauri::{PhysicalPosition, PhysicalSize};

    #[test]
    fn close_to_tray_beats_exit_confirmation() {
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
    fn force_exit_allows_close_immediately() {
        let action =
            resolve_main_window_close_action(&DesktopShellPreferences::default(), false, true);

        assert_eq!(action, MainWindowCloseAction::AllowExit);
    }

    #[test]
    fn tray_panel_stays_below_top_taskbar_work_area() {
        let position = resolve_panel_position_for_work_area(
            Some(&TrayAnchor {
                x: 1120,
                y: 20,
                width: 0,
                height: 0,
            }),
            Some(&PanelWorkArea {
                x: 0,
                y: 48,
                width: 1440,
                height: 852,
            }),
            &PhysicalSize::new(TRAY_PANEL_WIDTH as u32, TRAY_PANEL_HEIGHT as u32),
        );

        assert_eq!(position.y, 48);
    }

    #[test]
    fn tray_panel_prefers_above_bottom_taskbar() {
        let position = resolve_panel_position_for_work_area(
            Some(&TrayAnchor {
                x: 1430,
                y: 880,
                width: 0,
                height: 0,
            }),
            Some(&PanelWorkArea {
                x: 0,
                y: 0,
                width: 1600,
                height: 852,
            }),
            &PhysicalSize::new(TRAY_PANEL_WIDTH as u32, TRAY_PANEL_HEIGHT as u32),
        );

        assert!(position.y < 880);
    }

    #[test]
    fn tray_panel_falls_back_above_anchor_when_work_area_missing() {
        // 主窗口销毁且显示器探测失败时，锚点位于任务栏底部，面板必须抬到锚点上方，
        // 而不是坠到锚点下方（那会超出屏幕底端）。
        let position = resolve_panel_position_for_work_area(
            Some(&TrayAnchor {
                x: 1500,
                y: 1040,
                width: 0,
                height: 0,
            }),
            None,
            &PhysicalSize::new(TRAY_PANEL_WIDTH as u32, TRAY_PANEL_HEIGHT as u32),
        );

        assert_eq!(position.y, 1040 - TRAY_PANEL_HEIGHT - 12);
        assert!(position.y > 0);
        assert!(position.x >= 0);
    }

    #[test]
    fn tray_panel_fallback_clamps_to_zero_when_anchor_near_top() {
        // 锚点贴近屏幕顶部时，上移的目标 y 不应变成负数。
        let position = resolve_panel_position_for_work_area(
            Some(&TrayAnchor {
                x: 120,
                y: 10,
                width: 0,
                height: 0,
            }),
            None,
            &PhysicalSize::new(TRAY_PANEL_WIDTH as u32, TRAY_PANEL_HEIGHT as u32),
        );

        assert_eq!(position.y, 0);
        assert_eq!(position.x, 0);
    }

    #[test]
    fn tray_panel_clamps_size_to_small_work_area() {
        let size = resolve_panel_size_for_work_area(Some(&PanelWorkArea {
            x: 0,
            y: 0,
            width: 360,
            height: 520,
        }));

        assert_eq!(size.width, 336);
        assert_eq!(size.height, 496);
    }

    #[test]
    fn manual_tray_panel_position_is_preserved_inside_work_area() {
        let size = PhysicalSize::new(TRAY_PANEL_WIDTH as u32, TRAY_PANEL_HEIGHT as u32);
        let position = resolve_manual_panel_position_for_work_area(
            Some(&TrayPanelManualPosition { x: 728, y: 112 }),
            Some(&PanelWorkArea {
                x: 0,
                y: 0,
                width: 1600,
                height: 900,
            }),
            &size,
        );

        assert_eq!(position, Some(PhysicalPosition::new(728, 112)));
    }

    #[test]
    fn manual_tray_panel_position_clamps_back_into_work_area() {
        let size = PhysicalSize::new(TRAY_PANEL_WIDTH as u32, TRAY_PANEL_HEIGHT as u32);
        let position = resolve_manual_panel_position_for_work_area(
            Some(&TrayPanelManualPosition { x: 1400, y: 420 }),
            Some(&PanelWorkArea {
                x: 0,
                y: 0,
                width: 1440,
                height: 852,
            }),
            &size,
        );

        // 1440x852 的工作区扣掉 456x620 的面板后，右下角合法上限应为 (984, 232)。
        assert_eq!(position, Some(PhysicalPosition::new(984, 232)));
    }
}
