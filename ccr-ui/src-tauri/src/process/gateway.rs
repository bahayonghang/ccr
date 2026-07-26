//! Capability-scoped process execution for the desktop backend.

use std::collections::HashMap;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant, SystemTime};

use ccr_core::core::process_gateway::ManagedProcess;
use reqwest::Url;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::{ChildStderr, ChildStdin, ChildStdout, Command};
use tokio_util::sync::CancellationToken;

use super::{std_command, tokio_command};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
const DEFAULT_MAX_BYTES_PER_STREAM: usize = 1024 * 1024;
const TERMINATION_GRACE: Duration = Duration::from_secs(5);
const OAUTH_CALLBACK_PORT: u16 = 1455;
const CCR_SIDECAR_SHA256: Option<&str> = option_env!("CCR_SIDECAR_SHA256");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessCapability {
    CcrCommand,
    CcrVersionProbe,
    CcrUpdate,
    CliProbe,
    PortDiscovery,
    LlmusageVersion,
    LlmusageSync,
    PathLookup,
    OpenSsh,
    Sftp,
    SshKeyscan,
    SshKeygen,
}

impl ProcessCapability {
    pub const fn id(self) -> &'static str {
        match self {
            Self::CcrCommand => "ccr_command",
            Self::CcrVersionProbe => "ccr_version_probe",
            Self::CcrUpdate => "ccr_update",
            Self::CliProbe => "cli_probe",
            Self::PortDiscovery => "port_discovery",
            Self::LlmusageVersion => "llmusage_version",
            Self::LlmusageSync => "llmusage_sync",
            Self::PathLookup => "path_lookup",
            Self::OpenSsh => "openssh",
            Self::Sftp => "sftp",
            Self::SshKeyscan => "ssh_keyscan",
            Self::SshKeygen => "ssh_keygen",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrustedExecutable {
    CcrSidecar,
    SystemCargo,
    DetectedCli { tool: &'static str, path: PathBuf },
    SystemOpenSsh,
    SystemSftp,
    SystemSshKeyscan,
    SystemSshKeygen,
    PathLookup,
    PortDiscovery,
    Llmusage(PathBuf),
}

#[derive(Debug, Clone)]
pub struct ProcessDescriptor {
    capability: ProcessCapability,
    executable: TrustedExecutable,
    timeout: Duration,
    max_bytes_per_stream: usize,
}

impl ProcessDescriptor {
    pub const fn ccr_command() -> Self {
        Self {
            capability: ProcessCapability::CcrCommand,
            executable: TrustedExecutable::CcrSidecar,
            timeout: DEFAULT_TIMEOUT,
            max_bytes_per_stream: DEFAULT_MAX_BYTES_PER_STREAM,
        }
    }

    pub const fn ccr_version_probe() -> Self {
        Self {
            capability: ProcessCapability::CcrVersionProbe,
            executable: TrustedExecutable::CcrSidecar,
            timeout: Duration::from_secs(3),
            max_bytes_per_stream: 64 * 1024,
        }
    }

    pub const fn openssh() -> Self {
        Self {
            capability: ProcessCapability::OpenSsh,
            executable: TrustedExecutable::SystemOpenSsh,
            timeout: Duration::from_secs(15),
            max_bytes_per_stream: DEFAULT_MAX_BYTES_PER_STREAM,
        }
    }

    pub const fn ccr_update() -> Self {
        Self {
            capability: ProcessCapability::CcrUpdate,
            executable: TrustedExecutable::SystemCargo,
            timeout: Duration::from_secs(60),
            max_bytes_per_stream: DEFAULT_MAX_BYTES_PER_STREAM,
        }
    }

    pub fn cli_probe(
        tool: &'static str,
        path: impl Into<PathBuf>,
        timeout: Duration,
    ) -> Result<Self, String> {
        if !matches!(tool, "ccr" | "claude" | "codex" | "gemini") {
            return Err(format!("cli_probe_unsupported: {tool}"));
        }
        Ok(Self {
            capability: ProcessCapability::CliProbe,
            executable: TrustedExecutable::DetectedCli {
                tool,
                path: path.into(),
            },
            timeout,
            max_bytes_per_stream: 64 * 1024,
        })
    }

    pub const fn sftp() -> Self {
        Self {
            capability: ProcessCapability::Sftp,
            executable: TrustedExecutable::SystemSftp,
            timeout: Duration::from_secs(15),
            max_bytes_per_stream: DEFAULT_MAX_BYTES_PER_STREAM,
        }
    }

    pub const fn ssh_keyscan() -> Self {
        Self {
            capability: ProcessCapability::SshKeyscan,
            executable: TrustedExecutable::SystemSshKeyscan,
            timeout: Duration::from_secs(15),
            max_bytes_per_stream: 256 * 1024,
        }
    }

    pub const fn ssh_keygen() -> Self {
        Self {
            capability: ProcessCapability::SshKeygen,
            executable: TrustedExecutable::SystemSshKeygen,
            timeout: Duration::from_secs(15),
            max_bytes_per_stream: 256 * 1024,
        }
    }

    pub const fn path_lookup() -> Self {
        Self {
            capability: ProcessCapability::PathLookup,
            executable: TrustedExecutable::PathLookup,
            timeout: Duration::from_secs(3),
            max_bytes_per_stream: 64 * 1024,
        }
    }

    pub const fn port_discovery() -> Self {
        Self {
            capability: ProcessCapability::PortDiscovery,
            executable: TrustedExecutable::PortDiscovery,
            timeout: Duration::from_secs(3),
            max_bytes_per_stream: 64 * 1024,
        }
    }

    pub fn llmusage_version(binary: impl Into<PathBuf>) -> Result<Self, String> {
        let binary = binary.into();
        if binary.as_os_str().is_empty() {
            return Err("llmusage_binary_empty".to_string());
        }
        Ok(Self {
            capability: ProcessCapability::LlmusageVersion,
            executable: TrustedExecutable::Llmusage(binary),
            timeout: Duration::from_secs(3),
            max_bytes_per_stream: 64 * 1024,
        })
    }

    pub fn llmusage(binary: impl Into<PathBuf>) -> Result<Self, String> {
        let binary = binary.into();
        if binary.as_os_str().is_empty() {
            return Err("llmusage_binary_empty".to_string());
        }
        Ok(Self {
            capability: ProcessCapability::LlmusageSync,
            executable: TrustedExecutable::Llmusage(binary),
            timeout: Duration::from_secs(60 * 60),
            max_bytes_per_stream: DEFAULT_MAX_BYTES_PER_STREAM,
        })
    }

    pub const fn capability(&self) -> ProcessCapability {
        self.capability
    }

    pub const fn timeout(&self) -> Duration {
        self.timeout
    }
}

#[derive(Debug, Clone)]
pub struct OwnedProcessRecord {
    pub pid: u32,
    pub capability: ProcessCapability,
    pub started_at: SystemTime,
    pub ports: Vec<u16>,
    cancellation: CancellationToken,
}

impl OwnedProcessRecord {
    pub fn request_cancel(&self) {
        let age_ms = self
            .started_at
            .elapsed()
            .unwrap_or_default()
            .as_millis()
            .min(u128::from(u64::MAX)) as u64;
        tracing::info!(
            pid = self.pid,
            capability = self.capability.id(),
            ?self.ports,
            age_ms,
            "Requesting cancellation for an owned process"
        );
        self.cancellation.cancel();
    }
}

static OWNED_PROCESSES: LazyLock<Mutex<HashMap<u32, OwnedProcessRecord>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
pub struct CappedProcessOutput {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub timed_out: bool,
    pub duration: Duration,
}

pub struct ProcessGateway;

impl ProcessGateway {
    pub fn command(descriptor: &ProcessDescriptor) -> Result<Command, String> {
        let program = resolve_executable(&descriptor.executable)?;
        Ok(tokio_command(program))
    }

    pub fn spawn(
        command: Command,
        descriptor: &ProcessDescriptor,
        cancellation: CancellationToken,
        ports: Vec<u16>,
    ) -> Result<ManagedChild, String> {
        let child = ManagedProcess::spawn(command)
            .map_err(|error| format!("{}_spawn_failed: {error}", descriptor.capability().id()))?;
        ManagedChild::new(child, descriptor.capability(), cancellation, ports)
    }

    pub async fn execute(
        descriptor: &ProcessDescriptor,
        args: &[OsString],
    ) -> Result<CappedProcessOutput, String> {
        let mut command = Self::command(descriptor)?;
        command.args(args);
        Self::execute_command(command, descriptor, None).await
    }

    pub async fn execute_command(
        mut command: Command,
        descriptor: &ProcessDescriptor,
        stdin: Option<Vec<u8>>,
    ) -> Result<CappedProcessOutput, String> {
        command
            .stdin(if stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let cancellation = CancellationToken::new();
        let mut child = Self::spawn(command, descriptor, cancellation.clone(), Vec::new())?;
        let stdout = child
            .take_stdout()
            .ok_or_else(|| "process_stdout_unavailable".to_string())?;
        let stderr = child
            .take_stderr()
            .ok_or_else(|| "process_stderr_unavailable".to_string())?;
        let stdin_task = match stdin {
            Some(bytes) => {
                let mut child_stdin = child
                    .take_stdin()
                    .ok_or_else(|| "process_stdin_unavailable".to_string())?;
                Some(tokio::spawn(async move {
                    child_stdin.write_all(&bytes).await?;
                    child_stdin.shutdown().await
                }))
            }
            None => None,
        };

        let limit_reached = CancellationToken::new();
        let stdout_task = tokio::spawn(read_capped(
            stdout,
            descriptor.max_bytes_per_stream,
            limit_reached.clone(),
        ));
        let stderr_task = tokio::spawn(read_capped(
            stderr,
            descriptor.max_bytes_per_stream,
            limit_reached.clone(),
        ));

        let started = Instant::now();
        let deadline = tokio::time::sleep(descriptor.timeout);
        tokio::pin!(deadline);
        let mut timed_out = false;
        let status = tokio::select! {
            status = child.wait() => status,
            _ = &mut deadline => {
                timed_out = true;
                child.terminate_tree(TERMINATION_GRACE).await
            }
            _ = limit_reached.cancelled() => {
                child.terminate_tree(TERMINATION_GRACE).await
            }
        }
        .map_err(|error| format!("{}_wait_failed: {error}", descriptor.capability().id()))?;

        let stdout = stdout_task
            .await
            .map_err(|error| format!("stdout_reader_join_failed: {error}"))?
            .map_err(|error| format!("stdout_reader_failed: {error}"))?;
        let stderr = stderr_task
            .await
            .map_err(|error| format!("stderr_reader_join_failed: {error}"))?
            .map_err(|error| format!("stderr_reader_failed: {error}"))?;
        if let Some(stdin_task) = stdin_task {
            stdin_task
                .await
                .map_err(|error| format!("stdin_writer_join_failed: {error}"))?
                .map_err(|error| format!("stdin_writer_failed: {error}"))?;
        }

        Ok(CappedProcessOutput {
            status,
            stdout: stdout.bytes,
            stderr: stderr.bytes,
            stdout_bytes: stdout.total_bytes,
            stderr_bytes: stderr.total_bytes,
            stdout_truncated: stdout.truncated,
            stderr_truncated: stderr.truncated,
            timed_out,
            duration: started.elapsed(),
        })
    }

    pub fn validate_oauth_url(raw_url: &str) -> Result<Url, String> {
        let url = Url::parse(raw_url).map_err(|error| format!("oauth_url_invalid: {error}"))?;
        if !url.username().is_empty() || url.password().is_some() {
            return Err("oauth_url_credentials_forbidden".to_string());
        }

        let host = url
            .host_str()
            .ok_or_else(|| "oauth_url_host_missing".to_string())?;
        let is_authorize = url.scheme() == "https"
            && host.eq_ignore_ascii_case("auth.openai.com")
            && url.port().is_none()
            && url.path() == "/oauth/authorize";
        let is_callback = url.scheme() == "http"
            && matches!(host, "localhost" | "127.0.0.1" | "::1")
            && url.port() == Some(OAUTH_CALLBACK_PORT)
            && url.path() == "/auth/callback";

        if !is_authorize && !is_callback {
            return Err("oauth_url_not_allowed".to_string());
        }
        Ok(url)
    }

    pub fn open_oauth_url(raw_url: &str) -> Result<(), String> {
        let url = Self::validate_oauth_url(raw_url)?;

        #[cfg(target_os = "windows")]
        let mut command = {
            let mut command = std_command("rundll32");
            command.arg("url.dll,FileProtocolHandler").arg(url.as_str());
            command
        };

        #[cfg(target_os = "macos")]
        let mut command = {
            let mut command = std_command("open");
            command.arg(url.as_str());
            command
        };

        #[cfg(all(unix, not(target_os = "macos")))]
        let mut command = {
            let mut command = std_command("xdg-open");
            command.arg(url.as_str());
            command
        };

        command.stdout(Stdio::null()).stderr(Stdio::null());
        command
            .spawn()
            .map_err(|error| format!("oauth_browser_spawn_failed: {error}"))?;
        Ok(())
    }

    pub fn owned_processes_for_port(pids: &[u32], port: u16) -> Vec<OwnedProcessRecord> {
        let registry = OWNED_PROCESSES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        pids.iter()
            .filter_map(|pid| registry.get(pid))
            .filter(|record| record.ports.contains(&port))
            .cloned()
            .collect()
    }
}

#[derive(Debug)]
struct CappedRead {
    bytes: Vec<u8>,
    total_bytes: usize,
    truncated: bool,
}

async fn read_capped<R>(
    mut reader: R,
    max_bytes: usize,
    limit_reached: CancellationToken,
) -> io::Result<CappedRead>
where
    R: AsyncRead + Unpin,
{
    let mut bytes = Vec::with_capacity(max_bytes.min(64 * 1024));
    let mut buffer = [0u8; 8192];
    let mut total_bytes = 0usize;
    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        total_bytes = total_bytes.saturating_add(read);
        let remaining = max_bytes.saturating_sub(bytes.len());
        bytes.extend_from_slice(&buffer[..read.min(remaining)]);
        if read > remaining {
            limit_reached.cancel();
            return Ok(CappedRead {
                bytes,
                total_bytes,
                truncated: true,
            });
        }
    }
    Ok(CappedRead {
        bytes,
        total_bytes,
        truncated: false,
    })
}

fn resolve_executable(executable: &TrustedExecutable) -> Result<PathBuf, String> {
    match executable {
        TrustedExecutable::CcrSidecar => resolve_ccr_sidecar(
            std::env::current_exe().ok().as_deref(),
            Path::new(env!("CARGO_MANIFEST_DIR")),
            cfg!(debug_assertions),
            CCR_SIDECAR_SHA256,
        ),
        TrustedExecutable::SystemCargo => Ok(PathBuf::from("cargo")),
        TrustedExecutable::DetectedCli { tool: _, path } => Ok(path.clone()),
        TrustedExecutable::SystemOpenSsh => Ok(PathBuf::from("ssh")),
        TrustedExecutable::SystemSftp => Ok(PathBuf::from("sftp")),
        TrustedExecutable::SystemSshKeyscan => Ok(PathBuf::from("ssh-keyscan")),
        TrustedExecutable::SystemSshKeygen => Ok(PathBuf::from("ssh-keygen")),
        TrustedExecutable::PathLookup => {
            Ok(PathBuf::from(if cfg!(windows) { "where" } else { "which" }))
        }
        TrustedExecutable::PortDiscovery => Ok(PathBuf::from(if cfg!(windows) {
            "netstat"
        } else {
            "lsof"
        })),
        TrustedExecutable::Llmusage(path) => Ok(path.clone()),
    }
}

fn resolve_ccr_sidecar(
    current_exe: Option<&Path>,
    manifest_dir: &Path,
    allow_development: bool,
    expected_sha256: Option<&str>,
) -> Result<PathBuf, String> {
    let executable_name = if cfg!(windows) { "ccr.exe" } else { "ccr" };
    let adjacent = current_exe
        .and_then(Path::parent)
        .map(|directory| directory.join(executable_name));
    let development = manifest_dir
        .parent()
        .and_then(Path::parent)
        .into_iter()
        .flat_map(|root| {
            [
                root.join("target").join("debug").join(executable_name),
                root.join("target").join("release").join(executable_name),
            ]
        });

    let mut candidates = adjacent.into_iter().collect::<Vec<_>>();
    if allow_development {
        candidates.extend(development);
    }
    let candidate = candidates
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| "ccr_sidecar_not_found: PATH fallback is disabled".to_string())?;

    match expected_sha256 {
        Some(expected) => verify_sha256(&candidate, expected)?,
        None if !allow_development => {
            return Err(
                "ccr_sidecar_hash_missing: release builds require CCR_SIDECAR_SHA256".to_string(),
            );
        }
        None => {}
    }
    Ok(candidate)
}

fn verify_sha256(path: &Path, expected: &str) -> Result<(), String> {
    if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("ccr_sidecar_hash_invalid".to_string());
    }
    let bytes = std::fs::read(path).map_err(|error| format!("ccr_sidecar_read_failed: {error}"))?;
    let actual = hex_lower(&Sha256::digest(bytes));
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(format!(
            "ccr_sidecar_hash_mismatch: expected {expected}, got {actual}"
        ));
    }
    Ok(())
}

fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(encoded, "{byte:02x}");
    }
    encoded
}

pub struct ManagedChild {
    child: ManagedProcess,
    pid: u32,
    registered: bool,
}

impl ManagedChild {
    fn new(
        child: ManagedProcess,
        capability: ProcessCapability,
        cancellation: CancellationToken,
        ports: Vec<u16>,
    ) -> Result<Self, String> {
        let pid = child.pid();
        let record = OwnedProcessRecord {
            pid,
            capability,
            started_at: SystemTime::now(),
            ports,
            cancellation,
        };
        OWNED_PROCESSES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(pid, record);
        Ok(Self {
            child,
            pid,
            registered: true,
        })
    }

    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.take_stdout()
    }

    pub fn take_stderr(&mut self) -> Option<ChildStderr> {
        self.child.take_stderr()
    }

    pub fn take_stdin(&mut self) -> Option<ChildStdin> {
        self.child.take_stdin()
    }

    pub async fn wait(&mut self) -> io::Result<ExitStatus> {
        let status = self.child.wait().await;
        self.unregister();
        status
    }

    pub async fn terminate_tree(&mut self, grace: Duration) -> io::Result<ExitStatus> {
        let status = self.child.terminate_tree(grace).await;
        self.unregister();
        status
    }

    fn unregister(&mut self) {
        if self.registered {
            OWNED_PROCESSES
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(&self.pid);
            self.registered = false;
        }
    }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        if self.registered {
            self.unregister();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oauth_url_policy_rejects_unsafe_schemes_and_host_confusion() {
        for rejected in [
            "file:///tmp/token",
            "custom://auth.openai.com/oauth/authorize",
            "https://auth.openai.com.evil.example/oauth/authorize",
            "https://user:pass@auth.openai.com/oauth/authorize",
            "https://auth.openai.com:444/oauth/authorize",
            "http://localhost:1456/auth/callback",
        ] {
            assert!(
                ProcessGateway::validate_oauth_url(rejected).is_err(),
                "{rejected} must be rejected"
            );
        }
    }

    #[test]
    fn oauth_url_policy_accepts_authorize_and_fixed_loopback_callback() {
        assert!(
            ProcessGateway::validate_oauth_url(
                "https://auth.openai.com/oauth/authorize?client_id=test"
            )
            .is_ok()
        );
        assert!(
            ProcessGateway::validate_oauth_url(
                "http://localhost:1455/auth/callback?code=test&state=test"
            )
            .is_ok()
        );
    }

    #[test]
    fn release_sidecar_resolution_requires_hash_and_never_falls_back_to_path() {
        let temp = tempfile::tempdir().expect("tempdir");
        let app_dir = temp.path().join("app");
        std::fs::create_dir_all(&app_dir).expect("app dir");
        let current_exe = app_dir.join(if cfg!(windows) {
            "ccr-desktop.exe"
        } else {
            "ccr-desktop"
        });

        let missing = resolve_ccr_sidecar(Some(&current_exe), temp.path(), false, None)
            .expect_err("PATH fallback must remain disabled");
        assert!(missing.contains("not_found"));

        let sidecar = app_dir.join(if cfg!(windows) { "ccr.exe" } else { "ccr" });
        std::fs::write(&sidecar, b"trusted sidecar").expect("sidecar");
        let missing_hash = resolve_ccr_sidecar(Some(&current_exe), temp.path(), false, None)
            .expect_err("release hash is required");
        assert!(missing_hash.contains("hash_missing"));

        let expected = hex_lower(&Sha256::digest(b"trusted sidecar"));
        assert_eq!(
            resolve_ccr_sidecar(Some(&current_exe), temp.path(), false, Some(&expected))
                .expect("valid sidecar"),
            sidecar
        );
    }

    #[test]
    fn sidecar_hash_mismatch_is_rejected() {
        let temp = tempfile::tempdir().expect("tempdir");
        let path = temp.path().join("ccr");
        std::fs::write(&path, b"spoofed").expect("fixture");
        let error = verify_sha256(&path, &"0".repeat(64)).expect_err("mismatch");
        assert!(error.contains("hash_mismatch"));
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn foreground_output_flood_is_capped_and_terminated() {
        let descriptor =
            ProcessDescriptor::cli_probe("ccr", "powershell.exe", Duration::from_secs(5))
                .expect("test descriptor");
        let output = ProcessGateway::execute(
            &descriptor,
            &[
                OsString::from("-NoProfile"),
                OsString::from("-NonInteractive"),
                OsString::from("-Command"),
                OsString::from("[Console]::Out.Write('x' * 200000); Start-Sleep -Seconds 10"),
            ],
        )
        .await
        .expect("bounded execution");

        assert!(output.stdout_truncated);
        assert_eq!(output.stdout.len(), 64 * 1024);
        assert!(output.duration < Duration::from_secs(5));
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn foreground_timeout_terminates_and_reaps_process() {
        let descriptor =
            ProcessDescriptor::cli_probe("ccr", "powershell.exe", Duration::from_millis(100))
                .expect("test descriptor");
        let output = ProcessGateway::execute(
            &descriptor,
            &[
                OsString::from("-NoProfile"),
                OsString::from("-NonInteractive"),
                OsString::from("-Command"),
                OsString::from("Start-Sleep -Seconds 30"),
            ],
        )
        .await
        .expect("timed execution");

        assert!(output.timed_out);
        assert!(output.duration < Duration::from_secs(5));
    }
}
