//! Shared discovery and explicit launch of the existing SkillPort / skills-manage application.
#[cfg(target_os = "windows")]
use crate::jobs::process;
use serde::Serialize;
#[cfg(target_os = "windows")]
use std::process::Stdio;
#[cfg(any(target_os = "macos", test))]
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

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn not_found() -> Self {
        Self {
            supported: cfg!(any(target_os = "macos", target_os = "windows")),
            installed: false,
            platform: current_platform_name().to_string(),
            source: "not_found".to_string(),
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn installed(source: &'static str) -> Self {
        Self {
            supported: true,
            installed: true,
            platform: current_platform_name().to_string(),
            source: source.to_string(),
        }
    }
}

/// Discovery intentionally returns no executable path to UI consumers.
pub async fn detect() -> SkillportAppStatus {
    detect_skillport_app().await.status
}

/// Re-discover at activation time rather than trusting an old UI status.
pub async fn open() -> Result<(), String> {
    let discovery = detect_skillport_app().await;
    if !discovery.status.supported {
        return Err("SkillPort launch is unsupported".into());
    }
    let target = discovery
        .launch_target
        .as_ref()
        .ok_or("SkillPort was not found")?;
    launch_skillport(target).await
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
async fn detect_skillport_app() -> SkillportDiscovery {
    detect_skillport_app_macos().await
}

#[cfg(target_os = "windows")]
async fn detect_skillport_app() -> SkillportDiscovery {
    tokio::task::spawn_blocking(detect_skillport_app_windows)
        .await
        .unwrap_or_else(|_| SkillportDiscovery {
            status: SkillportAppStatus::not_found(),
            launch_target: None,
        })
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
async fn detect_skillport_app() -> SkillportDiscovery {
    SkillportDiscovery {
        status: SkillportAppStatus::unsupported(),
        launch_target: None,
    }
}

#[cfg(target_os = "windows")]
fn spawn_detached(mut command: std::process::Command, error_label: &str) -> Result<(), String> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
        .spawn()
        .map_err(|error| format!("{error_label}: {error}"))?;
    Ok(())
}

#[cfg(target_os = "macos")]
async fn launch_skillport(target: &SkillportLaunchTarget) -> Result<(), String> {
    let SkillportLaunchTarget::BundleId(bundle_id) = target;
    let args = bundle_launch_args(bundle_id)?;
    let output = crate::jobs::process::ProcessGateway::execute(
        &crate::jobs::process::ProcessDescriptor::skillport_helper(true),
        &args,
    )
    .await?;
    if helper_succeeded(&output) {
        Ok(())
    } else {
        Err("skillport_launch_failed".into())
    }
}

#[cfg(target_os = "windows")]
async fn launch_skillport(target: &SkillportLaunchTarget) -> Result<(), String> {
    let SkillportLaunchTarget::Executable(path) = target;
    let path = validated_windows_executable(path).ok_or("skillport_invalid_executable")?;
    // Intentional external GUI handoff: no arguments, no owned process tree.
    let command = process::std_command(path);
    spawn_detached(command, "启动可执行文件失败")
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
async fn launch_skillport(_target: &SkillportLaunchTarget) -> Result<(), String> {
    Err("当前平台没有可用的 skillport 启动方式".to_string())
}

#[cfg(target_os = "macos")]
async fn detect_skillport_app_macos() -> SkillportDiscovery {
    for bundle_id in SKILLPORT_BUNDLE_IDS {
        if macos_bundle_exists(bundle_id).await {
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
async fn macos_bundle_exists(bundle_id: &str) -> bool {
    if !SKILLPORT_BUNDLE_IDS.contains(&bundle_id) {
        return false;
    }
    let args = [std::ffi::OsString::from(format!(
        "kMDItemCFBundleIdentifier == '{bundle_id}'"
    ))];
    crate::jobs::process::ProcessGateway::execute(
        &crate::jobs::process::ProcessDescriptor::skillport_helper(false),
        &args,
    )
    .await
    .is_ok_and(|output| {
        helper_succeeded(&output) && !String::from_utf8_lossy(&output.stdout).trim().is_empty()
    })
}

#[cfg(any(target_os = "macos", test))]
fn bundle_launch_args(bundle_id: &str) -> Result<[std::ffi::OsString; 2], String> {
    if !SKILLPORT_BUNDLE_IDS.contains(&bundle_id) {
        return Err("skillport_unknown_bundle".into());
    }
    Ok(["-b".into(), bundle_id.into()])
}

#[cfg(any(target_os = "macos", test))]
fn helper_succeeded(output: &crate::jobs::process::CappedProcessOutput) -> bool {
    output.status.success()
        && !output.timed_out
        && !output.stdout_truncated
        && !output.stderr_truncated
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
        .find_map(|candidate| validated_windows_executable(&candidate))
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
        && let Some(path) = validated_windows_executable(&path)
    {
        return Some(path);
    }

    if let Ok(install_location) = entry_key.get_value::<String, _>("InstallLocation") {
        let root = trim_wrapped_quotes(install_location.trim());
        if !root.is_empty() {
            for app_name in SKILLPORT_APP_NAMES {
                let candidate = std::path::PathBuf::from(root).join(format!("{app_name}.exe"));
                if let Some(path) = validated_windows_executable(&candidate) {
                    return Some(path);
                }
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
fn validated_windows_executable(path: &std::path::Path) -> Option<std::path::PathBuf> {
    use std::os::windows::fs::MetadataExt;
    if !path.is_absolute()
        || !path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                ["skillport.exe", "skills-manage.exe"]
                    .iter()
                    .any(|expected| name.eq_ignore_ascii_case(expected))
            })
    {
        return None;
    }
    for ancestor in path.ancestors() {
        let metadata = std::fs::symlink_metadata(ancestor).ok()?;
        if metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0 {
            return None;
        }
    }
    if !path.is_file() {
        return None;
    }
    path.canonicalize().ok()
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
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn macos_handoff_accepts_only_fixed_bundles_and_bounded_helpers() {
        use crate::jobs::process::{ProcessDescriptor, ProcessGateway};
        for bundle in SKILLPORT_BUNDLE_IDS {
            assert_eq!(
                bundle_launch_args(bundle).unwrap(),
                [std::ffi::OsString::from("-b"), bundle.into()]
            );
        }
        for bundle in [
            "",
            "com.example.other",
            "com.bahayonghang.skillport;open -a Terminal",
        ] {
            assert!(bundle_launch_args(bundle).is_err());
        }
        for (launch, program) in [(false, "/usr/bin/mdfind"), (true, "/usr/bin/open")] {
            let descriptor = ProcessDescriptor::skillport_helper(launch);
            let command = ProcessGateway::command(&descriptor).unwrap();
            assert_eq!(command.as_std().get_program(), program);
            assert_eq!(descriptor.timeout(), std::time::Duration::from_secs(3));
            assert_eq!(command.as_std().get_args().count(), 0);
        }
    }

    #[test]
    fn helper_failure_timeout_and_truncation_never_confirm_a_handoff() {
        #[cfg(windows)]
        fn exit(code: u32) -> std::process::ExitStatus {
            use std::os::windows::process::ExitStatusExt;
            std::process::ExitStatus::from_raw(code)
        }
        #[cfg(unix)]
        fn exit(code: i32) -> std::process::ExitStatus {
            use std::os::unix::process::ExitStatusExt;
            std::process::ExitStatus::from_raw(code << 8)
        }
        for (code, timed_out, stdout_truncated, stderr_truncated, expected) in [
            (0, false, false, false, true),
            (1, false, false, false, false),
            (0, true, false, false, false),
            (0, false, true, false, false),
            (0, false, false, true, false),
        ] {
            let output = crate::jobs::process::CappedProcessOutput {
                status: exit(code),
                stdout: vec![],
                stderr: vec![],
                stdout_bytes: 0,
                stderr_bytes: 0,
                stdout_truncated,
                stderr_truncated,
                timed_out,
                duration: std::time::Duration::ZERO,
            };
            assert_eq!(helper_succeeded(&output), expected);
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_handoff_requires_absolute_expected_executable_identity() {
        let root = tempfile::tempdir().unwrap();
        let good = root.path().join("skillport.exe");
        std::fs::write(&good, b"fixture-only; never executed").unwrap();
        let other = root.path().join("untrusted.exe");
        std::fs::write(&other, b"fixture-only; never executed").unwrap();
        assert_eq!(
            validated_windows_executable(&good),
            Some(good.canonicalize().unwrap())
        );
        assert!(validated_windows_executable(&other).is_none());
        assert!(validated_windows_executable(std::path::Path::new("skillport.exe")).is_none());
        assert!(validated_windows_executable(&root.path().join("skills-manage.exe")).is_none());
        assert!(validated_windows_executable(root.path()).is_none());
    }

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
