//! 桌面壳层命令 —— tray 偏好、主窗口聚焦、显式退出，以及迁移页外部应用桥接。

use serde::Serialize;
use std::process::Stdio;
use tauri::{AppHandle, State};

use crate::desktop_shell;
use crate::process;
use crate::state::{AppState, DesktopShellPreferences, TrayPanelManualPosition};

#[cfg(target_os = "macos")]
const SKILLPORT_BUNDLE_IDS: [&str; 2] =
    ["com.bahayonghang.skillport", "com.iamzhihuix.skillsmanage"];
#[cfg(target_os = "windows")]
const SKILLPORT_APP_NAMES: [&str; 2] = ["skillport", "skills-manage"];

/// `skillport` 检测结果。
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SkillportAppStatus {
    pub supported: bool,
    pub installed: bool,
    pub platform: String,
    pub source: String,
}

pub type SkillsManageAppStatus = SkillportAppStatus;

#[derive(Debug, Clone)]
struct SkillportDiscovery {
    status: SkillportAppStatus,
    launch_target: Option<SkillportLaunchTarget>,
}

#[derive(Debug, Clone)]
enum SkillportLaunchTarget {
    #[cfg(target_os = "macos")]
    BundleId(String),
    #[cfg(target_os = "windows")]
    Executable(std::path::PathBuf),
}

impl SkillportAppStatus {
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn unsupported() -> Self {
        Self {
            supported: false,
            installed: false,
            platform: current_platform_name().to_string(),
            source: "unsupported".to_string(),
        }
    }

    fn not_found() -> Self {
        Self {
            supported: cfg!(any(target_os = "macos", target_os = "windows")),
            installed: false,
            platform: current_platform_name().to_string(),
            source: "not_found".to_string(),
        }
    }

    fn installed(source: &'static str) -> Self {
        Self {
            supported: true,
            installed: true,
            platform: current_platform_name().to_string(),
            source: source.to_string(),
        }
    }
}

/*
 * ========================================================================
 * 步骤1：读取桌面壳层偏好
 * ========================================================================
 * 目标：
 * 1) 返回当前桌面壳层偏好快照
 * 2) 保持前端设置页读取路径稳定
 */
#[tauri::command]
pub async fn shell_get_preferences(
    state: State<'_, AppState>,
) -> Result<DesktopShellPreferences, String> {
    Ok(state.desktop_shell_preferences())
}

/*
 * ========================================================================
 * 步骤2：更新桌面壳层偏好
 * ========================================================================
 * 目标：
 * 1) 写入新的桌面壳层偏好
 * 2) 刷新依赖偏好的 tray 展示
 */
#[tauri::command]
pub async fn shell_set_preferences(
    state: State<'_, AppState>,
    app: AppHandle,
    preferences: DesktopShellPreferences,
) -> Result<DesktopShellPreferences, String> {
    let updated = state.update_desktop_shell_preferences(|current| {
        *current = preferences.clone();
    })?;

    let refresh_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = desktop_shell::refresh_codex_tray(&refresh_handle, false).await {
            tracing::debug!("[shell] tray refresh skipped after preferences update: {error}");
        }
    });

    Ok(updated)
}

#[tauri::command]
pub async fn shell_show_main_window(
    app: AppHandle,
    target_route: Option<String>,
) -> Result<(), String> {
    desktop_shell::show_main_window(&app, target_route.as_deref()).await
}

#[tauri::command]
pub async fn shell_request_quit(app: AppHandle) -> Result<(), String> {
    desktop_shell::request_quit(&app)
}

#[tauri::command]
pub fn shell_begin_tray_panel_drag(state: State<'_, AppState>) -> Result<(), String> {
    state.set_tray_panel_drag_active(true);
    Ok(())
}

#[tauri::command]
pub fn shell_complete_tray_panel_drag(
    state: State<'_, AppState>,
    position: Option<TrayPanelManualPosition>,
) -> Result<(), String> {
    state.set_tray_panel_drag_active(false);

    if let Some(position) = position {
        state.set_tray_panel_manual_position(position)?;
    }

    Ok(())
}

/*
 * ========================================================================
 * 步骤3：探测 skillport 安装状态
 * ========================================================================
 * 目标：
 * 1) 在不暴露本机路径给前端的前提下返回安装状态
 * 2) 为 `/skills` 迁移页提供稳定三态来源
 */
#[tauri::command]
pub async fn shell_detect_skills_manage_app() -> Result<SkillsManageAppStatus, String> {
    shell_detect_skillport_app().await
}

/*
 * ========================================================================
 * 步骤3：探测 skillport 安装状态
 * ========================================================================
 * 目标：
 * 1) 在不暴露本机路径给前端的前提下返回安装状态
 * 2) 为 `/skills` 迁移页提供稳定三态来源
 */
#[tauri::command]
pub async fn shell_detect_skillport_app() -> Result<SkillportAppStatus, String> {
    tracing::info!("[shell] 开始探测 skillport 安装状态");

    let status = tokio::task::spawn_blocking(|| detect_skillport_app().status)
        .await
        .map_err(|error| format!("探测 skillport 失败: {error}"))?;

    tracing::info!(
        supported = status.supported,
        installed = status.installed,
        platform = %status.platform,
        source = %status.source,
        "[shell] skillport 安装状态探测完成"
    );

    Ok(status)
}

/*
 * ========================================================================
 * 步骤4：打开 skillport
 * ========================================================================
 * 目标：
 * 1) 按平台重新探测后拉起独立应用
 * 2) 避免前端状态过期导致的盲开
 */
#[tauri::command]
pub async fn shell_open_skills_manage_app() -> Result<(), String> {
    shell_open_skillport_app().await
}

/*
 * ========================================================================
 * 步骤4：打开 skillport
 * ========================================================================
 * 目标：
 * 1) 按平台重新探测后拉起独立应用
 * 2) 避免前端状态过期导致的盲开
 */
#[tauri::command]
pub async fn shell_open_skillport_app() -> Result<(), String> {
    tracing::info!("[shell] 开始打开 skillport");

    tokio::task::spawn_blocking(|| {
        let discovery = detect_skillport_app();
        if !discovery.status.supported {
            return Err("当前平台暂不支持自动打开 skillport".to_string());
        }

        let Some(target) = discovery.launch_target.as_ref() else {
            return Err("未检测到 skillport，请先前往仓库查看安装说明".to_string());
        };

        launch_skillport(target)
    })
    .await
    .map_err(|error| format!("打开 skillport 失败: {error}"))??;

    tracing::info!("[shell] 打开 skillport 请求已发出");
    Ok(())
}

#[cfg(target_os = "windows")]
fn current_platform_name() -> &'static str {
    "windows"
}

#[cfg(target_os = "macos")]
fn current_platform_name() -> &'static str {
    "macos"
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn current_platform_name() -> &'static str {
    "other"
}

#[cfg(target_os = "macos")]
fn detect_skillport_app() -> SkillportDiscovery {
    detect_skillport_app_macos()
}

#[cfg(target_os = "windows")]
fn detect_skillport_app() -> SkillportDiscovery {
    detect_skillport_app_windows()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn detect_skillport_app() -> SkillportDiscovery {
    SkillportDiscovery {
        status: SkillportAppStatus::unsupported(),
        launch_target: None,
    }
}

fn spawn_detached(mut command: std::process::Command, error_label: &str) -> Result<(), String> {
    command.stdout(Stdio::null()).stderr(Stdio::null());
    command
        .spawn()
        .map_err(|error| format!("{error_label}: {error}"))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_skillport(target: &SkillportLaunchTarget) -> Result<(), String> {
    let SkillportLaunchTarget::BundleId(bundle_id) = target;
    let mut command = process::std_command("open");
    command.args(["-b", bundle_id.as_str()]);
    spawn_detached(command, "调用 open 失败")
}

#[cfg(target_os = "windows")]
fn launch_skillport(target: &SkillportLaunchTarget) -> Result<(), String> {
    let SkillportLaunchTarget::Executable(path) = target;
    let command = process::std_command(path);
    spawn_detached(command, "启动可执行文件失败")
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn launch_skillport(_target: &SkillportLaunchTarget) -> Result<(), String> {
    Err("当前平台没有可用的 skillport 启动方式".to_string())
}

#[cfg(target_os = "macos")]
fn detect_skillport_app_macos() -> SkillportDiscovery {
    for bundle_id in SKILLPORT_BUNDLE_IDS {
        if macos_bundle_exists(bundle_id) {
            return SkillportDiscovery {
                status: SkillportAppStatus::installed("bundle_id"),
                launch_target: Some(SkillportLaunchTarget::BundleId(bundle_id.to_string())),
            };
        }
    }

    SkillportDiscovery {
        status: SkillportAppStatus::not_found(),
        launch_target: None,
    }
}

#[cfg(target_os = "macos")]
fn macos_bundle_exists(bundle_id: &str) -> bool {
    let mut command = process::std_command("mdfind");
    command.arg(format!("kMDItemCFBundleIdentifier == '{bundle_id}'"));

    command
        .output()
        .map(|output| !String::from_utf8_lossy(&output.stdout).trim().is_empty())
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn detect_skillport_app_windows() -> SkillportDiscovery {
    if let Some(path) = find_skillport_from_windows_registry() {
        return SkillportDiscovery {
            status: SkillportAppStatus::installed("registry"),
            launch_target: Some(SkillportLaunchTarget::Executable(path)),
        };
    }

    if let Some(path) = known_windows_candidate_paths()
        .into_iter()
        .find(|candidate| candidate.is_file())
    {
        return SkillportDiscovery {
            status: SkillportAppStatus::installed("known_path"),
            launch_target: Some(SkillportLaunchTarget::Executable(path)),
        };
    }

    SkillportDiscovery {
        status: SkillportAppStatus::not_found(),
        launch_target: None,
    }
}

#[cfg(target_os = "windows")]
fn find_skillport_from_windows_registry() -> Option<std::path::PathBuf> {
    use winreg::{
        RegKey,
        enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    };

    let uninstall_roots = [
        (
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
    ];

    for (hive, subkey_path) in uninstall_roots {
        let uninstall_root = RegKey::predef(hive);
        let Ok(uninstall_key) = uninstall_root.open_subkey(subkey_path) else {
            continue;
        };

        for entry_name in uninstall_key.enum_keys().flatten() {
            let Ok(entry_key) = uninstall_key.open_subkey(&entry_name) else {
                continue;
            };

            let Ok(display_name) = entry_key.get_value::<String, _>("DisplayName") else {
                continue;
            };

            if !is_skillport_display_name(&display_name) {
                continue;
            }

            if let Some(path) = resolve_skillport_path_from_registry(&entry_key) {
                return Some(path);
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn is_skillport_display_name(value: &str) -> bool {
    let trimmed = value.trim();
    SKILLPORT_APP_NAMES
        .iter()
        .any(|candidate| trimmed.eq_ignore_ascii_case(candidate))
}

#[cfg(target_os = "windows")]
fn resolve_skillport_path_from_registry(entry_key: &winreg::RegKey) -> Option<std::path::PathBuf> {
    if let Ok(display_icon) = entry_key.get_value::<String, _>("DisplayIcon")
        && let Some(path) = normalize_windows_display_icon_path(&display_icon)
        && path.is_file()
    {
        return Some(path);
    }

    if let Ok(install_location) = entry_key.get_value::<String, _>("InstallLocation") {
        let root = trim_wrapped_quotes(install_location.trim());
        if !root.is_empty() {
            for app_name in SKILLPORT_APP_NAMES {
                let candidate = std::path::PathBuf::from(root).join(format!("{app_name}.exe"));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn normalize_windows_display_icon_path(value: &str) -> Option<std::path::PathBuf> {
    let path_text = value.split(',').next().map(str::trim).unwrap_or_default();
    let normalized = trim_wrapped_quotes(path_text);
    if normalized.is_empty() {
        return None;
    }
    Some(std::path::PathBuf::from(normalized))
}

#[cfg(target_os = "windows")]
fn known_windows_candidate_paths() -> Vec<std::path::PathBuf> {
    use std::{env, path::PathBuf};

    let local_app_data = env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let program_files = env::var_os("ProgramFiles").map(PathBuf::from);
    let program_files_x86 = env::var_os("ProgramFiles(x86)").map(PathBuf::from);

    windows_candidate_paths_from_roots(
        local_app_data.as_deref(),
        program_files.as_deref(),
        program_files_x86.as_deref(),
    )
}

#[cfg(target_os = "windows")]
fn windows_candidate_paths_from_roots(
    local_app_data: Option<&std::path::Path>,
    program_files: Option<&std::path::Path>,
    program_files_x86: Option<&std::path::Path>,
) -> Vec<std::path::PathBuf> {
    use std::{collections::HashSet, path::PathBuf};

    let mut seen = HashSet::new();
    let mut candidates = Vec::new();

    if let Some(root) = local_app_data {
        for app_name in SKILLPORT_APP_NAMES {
            candidates.push(
                root.join("Programs")
                    .join(app_name)
                    .join(format!("{app_name}.exe")),
            );
        }
    }

    for root in [program_files, program_files_x86].into_iter().flatten() {
        for app_name in SKILLPORT_APP_NAMES {
            candidates.push(root.join(app_name).join(format!("{app_name}.exe")));
        }
    }

    candidates
        .into_iter()
        .filter(|candidate| seen.insert(candidate.clone()))
        .collect::<Vec<PathBuf>>()
}

#[cfg(target_os = "windows")]
fn trim_wrapped_quotes(value: &str) -> &str {
    value.trim().trim_matches('"')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "windows")]
    #[test]
    fn trim_wrapped_quotes_removes_outer_quotes() {
        assert_eq!(
            trim_wrapped_quotes(r#""C:\Apps\skills-manage.exe""#),
            r#"C:\Apps\skills-manage.exe"#
        );
        assert_eq!(trim_wrapped_quotes(" skillport "), "skillport");
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn normalize_windows_display_icon_path_strips_resource_suffix() {
        let normalized = normalize_windows_display_icon_path(r#""C:\Apps\skillport.exe",0"#)
            .expect("path should parse");

        assert_eq!(
            normalized,
            std::path::PathBuf::from(r#"C:\Apps\skillport.exe"#)
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_candidate_paths_cover_local_and_program_files_roots() {
        let candidates = windows_candidate_paths_from_roots(
            Some(std::path::Path::new(r"C:\Users\demo\AppData\Local")),
            Some(std::path::Path::new(r"C:\Program Files")),
            Some(std::path::Path::new(r"C:\Program Files (x86)")),
        );

        assert_eq!(
            candidates,
            vec![
                std::path::PathBuf::from(
                    r"C:\Users\demo\AppData\Local\Programs\skillport\skillport.exe"
                ),
                std::path::PathBuf::from(
                    r"C:\Users\demo\AppData\Local\Programs\skills-manage\skills-manage.exe"
                ),
                std::path::PathBuf::from(r"C:\Program Files\skillport\skillport.exe"),
                std::path::PathBuf::from(r"C:\Program Files\skills-manage\skills-manage.exe"),
                std::path::PathBuf::from(r"C:\Program Files (x86)\skillport\skillport.exe"),
                std::path::PathBuf::from(r"C:\Program Files (x86)\skills-manage\skills-manage.exe"),
            ]
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn exact_display_name_match_is_required() {
        assert!(is_skillport_display_name("skillport"));
        assert!(is_skillport_display_name("skills-manage"));
        assert!(!is_skillport_display_name("skillport beta"));
    }
}
