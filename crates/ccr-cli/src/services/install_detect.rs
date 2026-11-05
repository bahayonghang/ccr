//! llmusage presence detection.
//!
//! Pure PATH scan + version probe. No side effects on disk or environment.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use ccr_core::core::process_gateway::ManagedProcess;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;

const PROBE_TIMEOUT: Duration = Duration::from_secs(3);
const PROBE_CLEANUP: Duration = Duration::from_secs(5);
const PROBE_MAX_BYTES: usize = 4096;

struct ProbeOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq)]
enum ProbeError {
    Spawn,
    OutputLimit,
    Failed,
    TimedOut,
    CleanupFailed,
}
impl ProbeError {
    fn message(&self) -> &'static str {
        match self {
            Self::Spawn => "version probe could not start",
            Self::OutputLimit => "version probe output exceeded 4096 bytes",
            Self::Failed => "version probe failed",
            Self::TimedOut => "version probe timed out after 3000ms",
            Self::CleanupFailed => "version probe process tree cleanup could not be confirmed",
        }
    }
}

async fn probe_command(mut command: Command, timeout: Duration) -> Result<ProbeOutput, ProbeError> {
    #[cfg(windows)]
    command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW for background version probes.
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = ManagedProcess::spawn(command).map_err(|_| ProbeError::Spawn)?;
    let stdout = child.take_stdout();
    let stderr = child.take_stderr();
    let result = tokio::time::timeout(timeout, async {
        tokio::try_join!(probe_bytes(stdout), probe_bytes(stderr), child.wait())
    })
    .await;
    let error = match result {
        Ok(Ok((stdout, stderr, status))) => {
            return Ok(ProbeOutput {
                status,
                stdout,
                stderr,
            });
        }
        Ok(Err(error)) if error.kind() == std::io::ErrorKind::InvalidData => {
            ProbeError::OutputLimit
        }
        Ok(Err(_)) => ProbeError::Failed,
        Err(_) => ProbeError::TimedOut,
    };
    // Cancelled read futures own no spawned tasks. Reap and confirm the whole
    // owned tree before returning a timeout/output error to the caller.
    match tokio::time::timeout(PROBE_CLEANUP, child.terminate_tree(PROBE_CLEANUP)).await {
        Ok(Ok(_)) => Err(error),
        _ => Err(ProbeError::CleanupFailed),
    }
}

async fn probe_bytes(reader: Option<impl AsyncRead + Unpin>) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    if let Some(reader) = reader {
        reader
            .take((PROBE_MAX_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .await?;
    }
    if bytes.len() > PROBE_MAX_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "version output limit",
        ));
    }
    Ok(bytes)
}

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

    let mut command = Command::new(&binary);
    command.arg("--version");
    match probe_command(command, PROBE_TIMEOUT).await {
        Ok(output) if output.status.success() => Ok(DetectionResult::Available {
            path: binary,
            version: parse_version(&output.stdout),
            data_root_warning,
        }),
        Ok(output) => Ok(DetectionResult::Absent {
            reason: AbsentReason::NotExecutable {
                exit_code: output.status.code(),
                stderr_excerpt: excerpt(&output.stderr, 256),
            },
            data_root_warning,
        }),
        Err(error) => Ok(DetectionResult::Absent {
            reason: AbsentReason::NotExecutable {
                exit_code: None,
                stderr_excerpt: error.message().to_owned(),
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
    let text = super::install_ring_buffer::redact(&String::from_utf8_lossy(stdout));
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

/// Truncate stderr to at most `max_len` UTF-8 bytes, plus a clipping marker.
fn excerpt(bytes: &[u8], max_len: usize) -> String {
    let text = super::install_ring_buffer::redact(&String::from_utf8_lossy(bytes));
    let trimmed = text.trim();
    if trimmed.len() <= max_len {
        trimmed.to_string()
    } else {
        let mut end = max_len;
        while !trimmed.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &trimmed[..end])
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
        let _env = TestHostEnv::new();
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
    fn parse_version_redacts_credential_output() {
        for output in [
            b"llmusage token=version-secret-123".as_slice(),
            b"api_key=version-secret-456".as_slice(),
            b"llmusage Authorization: Bearer version-secret-789".as_slice(),
            b"llmusage token=\"version secret 123\"".as_slice(),
            b"llmusage key=\"visible private123\"".as_slice(),
            br#"llmusage key="visible \"escaped\" private123""#.as_slice(),
        ] {
            // Redaction may remove the only digit hint, leaving no version.
            let version = parse_version(output).unwrap_or_default();
            assert!(!version.contains("version-secret"));
            assert!(!version.contains("private123"));
            assert!(version.is_empty() || version.contains("***") || version.contains("..."));
        }
    }

    #[test]
    fn excerpt_truncates_long_text() {
        let long = "a".repeat(500);
        let result = excerpt(long.as_bytes(), 100);
        assert!(result.len() <= 104); // 100 + "…" (3 bytes UTF-8)
    }

    #[test]
    fn regression_excerpt_multibyte_boundary() {
        assert_eq!(
            excerpt("密".repeat(100).as_bytes(), 256),
            format!("{}…", "密".repeat(85))
        );
        assert_eq!(excerpt("😀你好".as_bytes(), 1), "…");
        assert_eq!(excerpt("😀你好".as_bytes(), 4), "😀…");
    }

    fn fixture_probe(mode: &str, root: &Path) -> Command {
        let mut command = Command::new(std::env::current_exe().expect("test executable"));
        command
            .args([
                "--exact",
                "services::install_detect::tests::probe_fixture_child",
                "--ignored",
                "--nocapture",
            ])
            .env("CCR_INSTALL_PROBE_FIXTURE", mode)
            .env("CCR_INSTALL_PROBE_ROOT", root);
        command
    }

    #[test]
    #[ignore = "closed synthetic subprocess entry, invoked by probe fixtures"]
    fn probe_fixture_child() {
        use std::io::Write;
        let mode = std::env::var("CCR_INSTALL_PROBE_FIXTURE").expect("fixture mode");
        let root = PathBuf::from(std::env::var_os("CCR_INSTALL_PROBE_ROOT").expect("fixture root"));
        match mode.as_str() {
            "stdout" => std::io::stdout()
                .write_all(&vec![b'x'; PROBE_MAX_BYTES * 2])
                .expect("fixture stdout"),
            "stderr" => std::io::stderr()
                .write_all(&vec![b'x'; PROBE_MAX_BYTES * 2])
                .expect("fixture stderr"),
            "tree" => {
                let mut command = fixture_probe("descendant", &root);
                command
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null());
                let child = command
                    .as_std_mut()
                    .spawn()
                    .expect("spawn fixture descendant");
                std::fs::write(root.join("pid"), child.id().to_string())
                    .expect("record descendant");
                // The managed parent owns this child through the OS process tree.
                drop(child);
            }
            "descendant" => {
                std::fs::write(root.join("ready"), b"owned descendant").expect("fixture ready")
            }
            "success" => {
                println!("llmusage 0.0.0");
                return;
            }
            _ => panic!("unknown fixture mode"),
        }
        std::thread::sleep(Duration::from_secs(30));
    }

    #[tokio::test]
    async fn bounded_probe_rejects_stdout_and_stderr_floods_and_accepts_short_output() {
        let root = tempfile::tempdir().expect("fixture root");
        let output = probe_command(
            fixture_probe("success", root.path()),
            Duration::from_secs(3),
        )
        .await
        .expect("short probe");
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("llmusage 0.0.0"));
        for mode in ["stdout", "stderr"] {
            let error = probe_command(fixture_probe(mode, root.path()), Duration::from_secs(3))
                .await
                .err()
                .expect("bounded output failure");
            assert_eq!(error, ProbeError::OutputLimit);
        }
    }

    #[tokio::test]
    async fn bounded_probe_timeout_confirms_owned_descendant_exit() {
        let root = tempfile::tempdir().expect("fixture root");
        let error = probe_command(fixture_probe("tree", root.path()), Duration::from_secs(2))
            .await
            .err()
            .expect("timeout");
        assert_eq!(error, ProbeError::TimedOut);
        assert!(
            root.path().join("ready").is_file(),
            "descendant actually started"
        );
        let pid: u32 = std::fs::read_to_string(root.path().join("pid"))
            .expect("descendant pid")
            .parse()
            .expect("pid number");
        #[cfg(unix)]
        assert_eq!(
            unsafe { libc::kill(pid as libc::pid_t, 0) },
            -1,
            "descendant must have exited"
        );
        #[cfg(windows)]
        {
            unsafe extern "system" {
                fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
                fn WaitForSingleObject(handle: *mut std::ffi::c_void, milliseconds: u32) -> u32;
                fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
            }
            // SAFETY: query only the fixture PID and close any acquired handle.
            unsafe {
                let handle = OpenProcess(0x0010_0000, 0, pid);
                if handle.is_null() {
                    assert_eq!(
                        std::io::Error::last_os_error().raw_os_error(),
                        Some(87),
                        "only absent PID is accepted"
                    );
                } else {
                    let state = WaitForSingleObject(handle, 0);
                    CloseHandle(handle);
                    assert_eq!(state, 0, "fixture descendant must be signaled/exited");
                }
            }
        }
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
        if !crate::home_process::is_child() {
            crate::home_process::run_case("without-home", |command, _| {
                command.env_remove("HOME");
            });
            return;
        }
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
        let result = probe_host_capabilities();
        assert!(result.has_cargo);
        assert_eq!(result.cargo_path, Some(path_on_hint.clone()));
    }

    #[test]
    fn cargo_path_in_probe_host_capabilities_comes_from_hint_dirs() {
        if crate::home_process::run_current_test() {
            return;
        }
        let mut env = TestHostEnv::new();
        let home = env.home().to_path_buf();
        let cargo_bin = home.join(".cargo").join("bin");
        std::fs::create_dir_all(&cargo_bin).expect("create cargo bin dir");

        let exe_names = executable_names("cargo");
        let exe_path = cargo_bin.join(&exe_names[0]);
        std::fs::write(&exe_path, "payload").expect("write fake executable");
        set_executable_mode_if_needed(&exe_path);

        env.remove_env("CARGO_HOME");
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
