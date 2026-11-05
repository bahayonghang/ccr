//! Fixed `wsl.exe` probes. Arguments are not taken from the renderer.
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const LIST_TIMEOUT: Duration = Duration::from_secs(20);
const EXEC_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_OUTPUT: usize = 64 * 1024;

#[derive(Debug)]
pub enum WslListError {
    Spawn(std::io::Error),
    TimedOut,
    Cancelled,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WslVersion {
    Wsl1,
    Wsl2,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WslState {
    Running,
    Stopped,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WslDistro {
    pub name: String,
    pub is_default: bool,
    pub version: WslVersion,
    pub state: WslState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WslExecProbe {
    Completed,
    PathNotFound,
    Failed,
    TimedOut,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WslProbe {
    Unsupported,
    NotQueried,
    Unavailable,
    None,
    Cancelled,
    Listed {
        names: Vec<String>,
        exec: WslExecProbe,
    },
}

struct CommandOutput {
    success: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

/// Decode `wsl -l -v` bytes. Windows prints UTF-16LE, sometimes with a BOM.
pub fn decode_wsl_output(bytes: &[u8]) -> String {
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        let utf16: Vec<u16> = bytes[2..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|chunk| u16::from_le_bytes(*chunk))
            .collect();
        String::from_utf16_lossy(&utf16)
    } else if bytes.contains(&0u8) && bytes.len().is_multiple_of(2) {
        let utf16: Vec<u16> = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|chunk| u16::from_le_bytes(*chunk))
            .collect();
        String::from_utf16_lossy(&utf16)
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

/// Parse one `wsl -l -v` transcript. The name token is data, never a command.
pub fn parse_wsl_list(text: &str) -> Vec<WslDistro> {
    let mut distros = Vec::new();
    for line in text.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let is_default = line.starts_with('*');
        let line = line.trim_start_matches('*').trim();
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        let name = parts[0].to_string();
        if name.is_empty() || name == "NAME" {
            continue;
        }
        let state = match parts.get(1).copied() {
            Some("Running") => WslState::Running,
            Some("Stopped") => WslState::Stopped,
            _ => WslState::Unknown,
        };
        let version = match parts.get(2).copied() {
            Some("1") => WslVersion::Wsl1,
            Some("2") => WslVersion::Wsl2,
            _ => WslVersion::Unknown,
        };
        distros.push(WslDistro {
            name,
            is_default,
            version,
            state,
        });
    }
    distros
}

/// Where a WSL config file would be opened.
///
/// `Unavailable` opens nothing. A remote target keeps the POSIX path and the
/// distribution name; it is never a Windows drive or UNC path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WslFileTarget {
    Unavailable,
    Remote { distro: String, path: String },
}

/// Choose a WSL file target without spawning `wsl.exe`.
///
/// No distribution, a blank distribution, or a Windows path yields
/// `Unavailable`. The local Windows path is not returned and is not opened.
pub fn resolve_wsl_file(distro: Option<&str>, linux_path: &str) -> WslFileTarget {
    let Some(distro) = usable_distro(distro) else {
        return WslFileTarget::Unavailable;
    };
    if is_windows_local_path(linux_path) || !is_absolute_posix(linux_path) {
        return WslFileTarget::Unavailable;
    }
    WslFileTarget::Remote {
        distro: distro.to_string(),
        path: linux_path.to_string(),
    }
}

fn usable_distro(distro: Option<&str>) -> Option<&str> {
    let distro = distro?.trim();
    if distro.is_empty()
        || is_windows_local_path(distro)
        || distro.contains(['/', '\\', ':', ' ', '\n', '\r', '\t'])
    {
        None
    } else {
        Some(distro)
    }
}

fn is_windows_local_path(path: &str) -> bool {
    let path = path.trim();
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        return true;
    }
    path.starts_with(r"\\") || path.starts_with("//")
}

fn is_absolute_posix(path: &str) -> bool {
    path.starts_with('/')
        && !path.contains(['\\', '\n', '\r'])
        && !path.split('/').any(|segment| segment == "..")
}

pub fn classify_exec(success: bool, stdout: &[u8], stderr: &[u8]) -> WslExecProbe {
    if success {
        return WslExecProbe::Completed;
    }
    let text = format!("{}{}", decode_wsl_output(stderr), decode_wsl_output(stdout));
    let lower = text.to_ascii_lowercase();
    if lower.contains("error_path_not_found")
        || lower.contains("path not found")
        || lower.contains("cannot find the path")
        || lower.contains("0x80070003")
        || text.contains("系统找不到指定的路径")
    {
        WslExecProbe::PathNotFound
    } else {
        WslExecProbe::Failed
    }
}

/// Run the fixed `wsl -l -v` list. A failed command with empty stdout is an empty list.
pub fn list_installed_distros() -> Result<Vec<WslDistro>, WslListError> {
    #[cfg(windows)]
    {
        list_installed_distros_windows(&|| false)
    }
    #[cfg(not(windows))]
    {
        Err(WslListError::Unsupported)
    }
}

pub fn probe_wsl(is_cancelled: &impl Fn() -> bool) -> WslProbe {
    #[cfg(windows)]
    {
        probe_wsl_windows(is_cancelled)
    }
    #[cfg(not(windows))]
    {
        let _ = is_cancelled;
        WslProbe::Unsupported
    }
}

#[cfg(windows)]
fn probe_wsl_windows(is_cancelled: &impl Fn() -> bool) -> WslProbe {
    if is_cancelled() {
        return WslProbe::Cancelled;
    }
    match list_installed_distros_windows(is_cancelled) {
        Err(WslListError::Cancelled) => WslProbe::Cancelled,
        Err(_) => WslProbe::Unavailable,
        Ok(distros) if distros.is_empty() => WslProbe::None,
        Ok(distros) => {
            if is_cancelled() {
                return WslProbe::Cancelled;
            }
            let names = distros.into_iter().map(|distro| distro.name).collect();
            let exec = probe_default_exec(is_cancelled);
            if matches!(exec, WslExecProbe::Cancelled) || is_cancelled() {
                return WslProbe::Cancelled;
            }
            WslProbe::Listed { names, exec }
        }
    }
}

#[cfg(windows)]
fn list_installed_distros_windows(
    is_cancelled: &impl Fn() -> bool,
) -> Result<Vec<WslDistro>, WslListError> {
    let output = run_wsl(&["-l", "-v"], LIST_TIMEOUT, is_cancelled)?;
    if !output.success && output.stdout.is_empty() {
        return Ok(Vec::new());
    }
    Ok(parse_wsl_list(&decode_wsl_output(&output.stdout)))
}

#[cfg(windows)]
fn probe_default_exec(is_cancelled: &impl Fn() -> bool) -> WslExecProbe {
    match run_wsl(&["-e", "true"], EXEC_TIMEOUT, is_cancelled) {
        Ok(output) => classify_exec(output.success, &output.stdout, &output.stderr),
        Err(WslListError::TimedOut) => WslExecProbe::TimedOut,
        Err(WslListError::Cancelled) => WslExecProbe::Cancelled,
        Err(_) => WslExecProbe::Failed,
    }
}

#[cfg(windows)]
fn run_wsl(
    args: &[&str],
    timeout: Duration,
    is_cancelled: &impl Fn() -> bool,
) -> Result<CommandOutput, WslListError> {
    let mut command = Command::new("wsl");
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_console(&mut command);
    let mut child = command.spawn().map_err(WslListError::Spawn)?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_thread = std::thread::spawn(move || read_limited(stdout));
    let stderr_thread = std::thread::spawn(move || read_limited(stderr));
    let started = Instant::now();
    let status = loop {
        if is_cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_thread.join();
            let _ = stderr_thread.join();
            return Err(WslListError::Cancelled);
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_thread.join();
                let _ = stderr_thread.join();
                return Err(WslListError::TimedOut);
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(30)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_thread.join();
                let _ = stderr_thread.join();
                return Err(WslListError::Spawn(error));
            }
        }
    };
    let stdout = stdout_thread.join().unwrap_or_default();
    let stderr = stderr_thread.join().unwrap_or_default();
    Ok(CommandOutput {
        success: status.success(),
        stdout,
        stderr,
    })
}

#[cfg(windows)]
fn hide_console(command: &mut Command) {
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(windows)]
fn read_limited(reader: Option<impl Read>) -> Vec<u8> {
    let Some(mut reader) = reader else {
        return Vec::new();
    };
    let mut output = Vec::new();
    let mut buffer = [0u8; 4096];
    while let Ok(read) = reader.read(&mut buffer) {
        if read == 0 {
            break;
        }
        let room = MAX_OUTPUT.saturating_sub(output.len());
        if room == 0 {
            let _ = std::io::copy(&mut reader, &mut std::io::sink());
            break;
        }
        let take = read.min(room);
        output.extend_from_slice(&buffer[..take]);
        if take < read {
            let _ = std::io::copy(&mut reader, &mut std::io::sink());
            break;
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{
        WslExecProbe, WslFileTarget, WslState, WslVersion, classify_exec, decode_wsl_output,
        parse_wsl_list, resolve_wsl_file,
    };

    #[test]
    fn decode_preserves_utf8_and_utf16le() {
        assert_eq!(decode_wsl_output(b"Ubuntu\r\n"), "Ubuntu\r\n");
        assert_eq!(
            decode_wsl_output(&[0xFF, 0xFE, b'U', 0, b'b', 0, 0x7F]),
            "Ub"
        );
        assert_eq!(decode_wsl_output(&[b'U', 0, b'b', 0]), "Ub");
    }

    #[test]
    fn parse_keeps_listed_names_and_does_not_invent_one() {
        let parsed = parse_wsl_list(
            "  NAME      STATE           VERSION\n* docker-desktop    Running         2\n",
        );
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "docker-desktop");
        assert!(parsed[0].is_default);
        assert_eq!(parsed[0].state, WslState::Running);
        assert_eq!(parsed[0].version, WslVersion::Wsl2);
        assert!(parsed.iter().all(|distro| distro.name != "Ubuntu"));
    }

    #[test]
    fn parse_keeps_hostile_name_as_text() {
        let parsed = parse_wsl_list(" NAME STATE VERSION\n* $(whoami) Stopped 1\n");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "$(whoami)");
        assert_eq!(parsed[0].state, WslState::Stopped);
        assert_eq!(parsed[0].version, WslVersion::Wsl1);
    }

    #[test]
    fn missing_distro_does_not_fall_back_to_a_local_windows_path() {
        let dir = tempfile::tempdir().unwrap_or_else(|error| panic!("{error}"));
        let local = dir.path().join("settings.json");
        std::fs::write(&local, b"local-secret").unwrap_or_else(|error| panic!("{error}"));
        let windows_home = r"C:\Users\example\.claude\settings.json";

        for distro in [
            None,
            Some(""),
            Some("   "),
            Some(r"C:\Users\example"),
            Some("docker-desktop"),
        ] {
            let target = resolve_wsl_file(distro, windows_home);
            assert_eq!(target, WslFileTarget::Unavailable);
            let rendered = format!("{target:?}");
            assert!(!rendered.contains("settings.json"));
            assert!(!rendered.contains("local-secret"));
            assert!(!rendered.contains("Users"));
            assert!(!rendered.contains(':'));
        }

        let local_target = resolve_wsl_file(None, &local.display().to_string());
        assert_eq!(local_target, WslFileTarget::Unavailable);
        assert!(!format!("{local_target:?}").contains("local-secret"));
        assert!(!format!("{local_target:?}").contains("settings.json"));

        let remote = resolve_wsl_file(Some("docker-desktop"), "/home/user/.claude/settings.json");
        match remote {
            WslFileTarget::Remote { distro, path } => {
                assert_eq!(distro, "docker-desktop");
                assert_eq!(path, "/home/user/.claude/settings.json");
                assert!(!path.contains('\\'));
                assert!(!path.contains(':'));
            }
            WslFileTarget::Unavailable => panic!("posix path should stay on the named distro"),
        }
        assert_eq!(
            resolve_wsl_file(Some("docker-desktop"), "/home/../mnt/c/Users/example"),
            WslFileTarget::Unavailable
        );

        let body = std::fs::read_to_string(&local).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(body, "local-secret");
    }

    #[test]
    fn path_failure_does_not_retain_the_stderr_path() {
        let stderr = "ERROR_PATH_NOT_FOUND D:\\Docker\\wsl\\main\\ext4.vhdx";
        let probe = classify_exec(false, b"", stderr.as_bytes());
        assert_eq!(probe, WslExecProbe::PathNotFound);
        let rendered = format!("{probe:?}");
        assert!(!rendered.contains("vhdx"));
        assert!(!rendered.contains("Docker"));
    }
}
