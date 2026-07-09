//! llmusage presence detection.
//!
//! Pure PATH scan + version probe. No side effects on disk or environment.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::services::install_types::{
    AbsentReason, DataRootWarning, DetectionResult, HostCapabilities, InstallFlowError, Platform,
};

/// Detect whether the `llmusage` binary is available on the host.
///
/// This operation is idempotent and side-effect free.
pub async fn detect() -> Result<DetectionResult, InstallFlowError> {
    let binary_path = which_on_path("llmusage");
    let data_root_warning = probe_data_root();

    let Some(binary) = binary_path else {
        return Ok(DetectionResult::Absent {
            reason: AbsentReason::NotOnPath,
            data_root_warning,
        });
    };

    // Probe version with a 3-second timeout.
    match tokio::time::timeout(
        std::time::Duration::from_millis(3000),
        tokio::process::Command::new(&binary)
            .arg("--version")
            .kill_on_drop(true)
            .output(),
    )
    .await
    {
        Ok(Ok(output)) if output.status.success() => Ok(DetectionResult::Available {
            path: binary,
            version: parse_version(&output.stdout),
            data_root_warning,
        }),
        Ok(Ok(output)) => Ok(DetectionResult::Absent {
            reason: AbsentReason::NotExecutable {
                exit_code: output.status.code(),
                stderr_excerpt: excerpt(&output.stderr, 256),
            },
            data_root_warning,
        }),
        Ok(Err(e)) => Ok(DetectionResult::Absent {
            reason: AbsentReason::NotExecutable {
                exit_code: None,
                stderr_excerpt: format!("spawn error: {e}"),
            },
            data_root_warning,
        }),
        Err(_timeout) => Ok(DetectionResult::Absent {
            reason: AbsentReason::NotExecutable {
                exit_code: None,
                stderr_excerpt: "version probe timed out after 3000ms".to_string(),
            },
            data_root_warning,
        }),
    }
}

/// Probe the host for available package managers and platform info.
pub fn probe_host_capabilities() -> HostCapabilities {
    let platform = Platform::current();
    let cargo_path = resolve_executable("cargo", cargo_hint_dirs());
    let homebrew_path = if matches!(platform, Platform::Macos) {
        resolve_executable(
            "brew",
            vec![
                PathBuf::from("/opt/homebrew/bin"),
                PathBuf::from("/usr/local/bin"),
            ],
        )
    } else {
        None
    };

    HostCapabilities {
        platform,
        has_cargo: cargo_path.is_some(),
        cargo_path,
        has_homebrew: homebrew_path.is_some(),
        homebrew_path,
        has_scoop: which_on_path("scoop").is_some(),
        has_winget: which_on_path("winget").is_some(),
    }
}

/// Resolve an executable path from PATH first, then fallback directories.
fn resolve_executable(name: &str, hint_dirs: Vec<PathBuf>) -> Option<PathBuf> {
    if let Some(path) = which_on_path(name) {
        return Some(path);
    }

    for dir in hint_dirs {
        for exe_name in executable_names(name) {
            let candidate = dir.join(&exe_name);
            if is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

fn cargo_hint_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::<PathBuf>::new();
    let mut unique = BTreeSet::<PathBuf>::new();

    let push_dir = |path: PathBuf, dirs: &mut Vec<PathBuf>, unique: &mut BTreeSet<PathBuf>| {
        if unique.insert(path.clone()) {
            dirs.push(path);
        }
    };

    if let Some(cargo_home) = std::env::var_os("CARGO_HOME") {
        push_dir(
            PathBuf::from(cargo_home).join("bin"),
            &mut dirs,
            &mut unique,
        );
    }
    if let Some(home) = std::env::var_os("HOME") {
        push_dir(
            PathBuf::from(home).join(".cargo").join("bin"),
            &mut dirs,
            &mut unique,
        );
    }

    #[cfg(windows)]
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        push_dir(
            PathBuf::from(profile).join(".cargo").join("bin"),
            &mut dirs,
            &mut unique,
        );
    }

    dirs
}

/// Search for a binary on PATH without spawning a process.
///
/// Uses `std::env::var_os("PATH")` + `std::env::split_paths` to avoid
/// adding the `which` crate as a dependency.
pub fn which_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let exe_names = executable_names(name);

    for dir in std::env::split_paths(&path_var) {
        for exe_name in &exe_names {
            let candidate = dir.join(exe_name);
            if is_executable(&candidate) {
                return Some(candidate);
            }
        }
    }
    None
}

/// Check if `LLMUSAGE_HOME` is set to a non-existent directory.
fn probe_data_root() -> Option<DataRootWarning> {
    let home = std::env::var_os("LLMUSAGE_HOME")?;
    let path = PathBuf::from(home);
    if !path.is_dir() {
        Some(DataRootWarning::DataRootMissing { path })
    } else {
        None
    }
}

/// Parse version string from stdout (e.g. "llmusage 0.5.3\n" → "0.5.3").
fn parse_version(stdout: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(stdout);
    let trimmed = text.trim();
    // Try to extract version after the last space
    trimmed
        .rsplit_once(' ')
        .map(|(_, ver)| ver.trim().to_string())
        .or_else(|| {
            // If no space, the whole thing might be a version
            if trimmed.chars().any(|c| c.is_ascii_digit()) {
                Some(trimmed.to_string())
            } else {
                None
            }
        })
}

/// Truncate stderr bytes to a UTF-8 excerpt of at most `max_len` chars.
fn excerpt(bytes: &[u8], max_len: usize) -> String {
    let text = String::from_utf8_lossy(bytes);
    let trimmed = text.trim();
    if trimmed.len() <= max_len {
        trimmed.to_string()
    } else {
        format!("{}…", &trimmed[..max_len])
    }
}

/// Generate possible executable file names for a given binary name.
#[cfg(target_os = "windows")]
fn executable_names(name: &str) -> Vec<String> {
    vec![
        format!("{name}.exe"),
        format!("{name}.cmd"),
        format!("{name}.bat"),
        name.to_string(),
    ]
}

#[cfg(not(target_os = "windows"))]
fn executable_names(name: &str) -> Vec<String> {
    vec![name.to_string()]
}

/// Check if a path exists and is executable.
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.is_file()
        && path
            .metadata()
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TestHostEnv;

    #[test]
    fn which_on_path_finds_common_binary() {
        // `ls` (Unix) or `cmd` (Windows) should always be on PATH in CI/dev.
        #[cfg(unix)]
        let result = which_on_path("ls");
        #[cfg(windows)]
        let result = which_on_path("cmd");

        assert!(result.is_some(), "expected to find a common binary on PATH");
    }

    #[test]
    fn which_on_path_returns_none_for_nonexistent() {
        let result = which_on_path("__nonexistent_binary_xyz_12345__");
        assert!(result.is_none());
    }

    #[test]
    fn parse_version_extracts_semver() {
        assert_eq!(
            parse_version(b"llmusage 0.5.3\n"),
            Some("0.5.3".to_string())
        );
        assert_eq!(
            parse_version(b"llmusage 1.2.3-beta.1\n"),
            Some("1.2.3-beta.1".to_string())
        );
    }

    #[test]
    fn parse_version_handles_empty() {
        assert_eq!(parse_version(b""), None);
        assert_eq!(parse_version(b"   "), None);
    }

    #[test]
    fn excerpt_truncates_long_text() {
        let long = "a".repeat(500);
        let result = excerpt(long.as_bytes(), 100);
        assert!(result.len() <= 104); // 100 + "…" (3 bytes UTF-8)
    }

    #[test]
    fn probe_data_root_returns_none_when_unset() {
        // This test is environment-dependent; if LLMUSAGE_HOME is not set, returns None.
        if std::env::var_os("LLMUSAGE_HOME").is_none() {
            assert!(probe_data_root().is_none());
        }
    }

    #[test]
    fn host_capabilities_detects_platform() {
        let caps = probe_host_capabilities();
        #[cfg(target_os = "macos")]
        assert_eq!(caps.platform, Platform::Macos);
        #[cfg(target_os = "linux")]
        assert_eq!(caps.platform, Platform::Linux);
        #[cfg(target_os = "windows")]
        assert_eq!(caps.platform, Platform::Windows);
    }

    #[test]
    fn resolve_executable_uses_path_before_hints() {
        let mut env = TestHostEnv::new();
        let fixture = tempfile::tempdir().expect("tempdir");
        let bin_dir = fixture.path().join("bin");
        std::fs::create_dir_all(&bin_dir).expect("create bin dir");

        let name = executable_names("brew")[0].clone();
        let path_on_hint = bin_dir.join(name.clone());
        std::fs::write(&path_on_hint, "payload").expect("write fake executable");
        set_executable_mode_if_needed(&path_on_hint);

        env.set_env("PATH", std::ffi::OsStr::new(""));

        let result = resolve_executable("brew", vec![bin_dir]);
        assert_eq!(result, Some(path_on_hint));
    }

    #[test]
    fn resolve_executable_uses_cargo_hints() {
        let mut env = TestHostEnv::new();
        let cwd = env.home();
        let bin_dir = cwd.join("bin");
        std::fs::create_dir_all(&bin_dir).expect("create cargo bin dir");

        let exe_names = executable_names("cargo");
        let path_on_hint = bin_dir.join(&exe_names[0]);
        std::fs::write(&path_on_hint, "payload").expect("write fake executable");
        set_executable_mode_if_needed(&path_on_hint);

        let cargo_home = cwd.as_os_str().to_owned();
        env.set_env("CARGO_HOME", cargo_home.as_os_str());
        env.set_env(
            "PATH",
            std::ffi::OsStr::new("/_ccr_nonexistent_path_for_tests"),
        );
        env.remove_env("HOME");
        let result = probe_host_capabilities();
        assert!(result.has_cargo);
        assert_eq!(result.cargo_path, Some(path_on_hint.clone()));
    }

    #[test]
    fn cargo_path_in_probe_host_capabilities_comes_from_hint_dirs() {
        let mut env = TestHostEnv::new();
        let home = env.home().join("home");
        let cargo_bin = home.join(".cargo").join("bin");
        std::fs::create_dir_all(&cargo_bin).expect("create cargo bin dir");

        let exe_names = executable_names("cargo");
        let exe_path = cargo_bin.join(&exe_names[0]);
        std::fs::write(&exe_path, "payload").expect("write fake executable");
        set_executable_mode_if_needed(&exe_path);

        env.remove_env("CARGO_HOME");
        env.set_env("HOME", home.as_os_str());
        env.set_env(
            "PATH",
            std::ffi::OsStr::new("/_ccr_nonexistent_path_for_tests"),
        );
        let result = probe_host_capabilities();

        assert!(result.has_cargo);
        assert_eq!(result.cargo_path, Some(exe_path));
    }

    #[cfg(unix)]
    fn set_executable_mode_if_needed(path: &Path) {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path).expect("stat").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms).expect("chmod");
    }

    #[cfg(windows)]
    fn set_executable_mode_if_needed(_path: &Path) {}
}
