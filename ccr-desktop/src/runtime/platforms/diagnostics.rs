//! Explicit Codex auth diagnostics; never probe the host from an isolated source.
use super::*;
use ccr_app::jobs::process::{CappedProcessOutput, ProcessDescriptor, ProcessGateway};
use std::{collections::BTreeSet, ffi::OsString, net::TcpListener};

pub const CALLBACK_PORT: u16 = 1455;
const MAX_PIDS: usize = 256;
const SHOWN_PIDS: usize = 8;

pub enum Request {
    SaveProcesses(Box<PlatformRoots>),
    CallbackPort,
    PrepareCleanup(Box<PlatformRoots>),
    CancelOwnedListeners(Arc<ccr_cli::application::codex_oauth::NativeCleanupPlan>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PortState {
    Available,
    Occupied,
    Unavailable,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessWarning {
    pub pids: Vec<u32>,
    pub additional: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseReport {
    pub owned: Vec<u32>,
    pub unknown: Vec<u32>,
    pub cancel_requested: usize,
    pub port: PortState,
}
#[derive(Clone)]
pub enum Report {
    Processes(ProcessWarning),
    Port(PortState),
    Release(ReleaseReport),
    CleanupPlan(Arc<ccr_cli::application::codex_oauth::NativeCleanupPlan>),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Unsupported,
    Isolated,
    Unavailable,
    CleanupPending,
}
impl Error {
    pub fn message(self) -> &'static str {
        match self {
            Self::Unsupported => {
                "Codex auth diagnostics require a local Codex environment. / 诊断仅支持本地 Codex"
            }
            Self::Isolated => {
                "Host process and callback-port probes are disabled in fixture/benchmark mode. / fixture/benchmark 已禁用主机进程及端口探测"
            }
            Self::Unavailable => {
                "Diagnostic unavailable; process and port state remain unknown. / 诊断不可用，进程与端口状态未知"
            }
            Self::CleanupPending => {
                "Owned OAuth cleanup is pending or failed. No listener exit or port release is claimed. / 自有 OAuth 清理未完成，不声称监听器已退出或端口已释放"
            }
        }
    }
}

fn admit(services: &Services, key: &RequestKey) -> Result<(), Error> {
    // Reject before resolving roots, reading identity files or touching OS probes.
    if !matches!(key.environment, EnvironmentId::Local(_)) || key.platform != Platform::Codex {
        return Err(Error::Unsupported);
    }
    if services.is_fixture() {
        return Err(Error::Isolated);
    }
    Ok(())
}
fn process_warning(mut pids: Vec<u32>) -> ProcessWarning {
    pids.retain(|pid| *pid > 0);
    pids.sort_unstable();
    pids.dedup();
    let additional = pids.len().saturating_sub(SHOWN_PIDS);
    pids.truncate(SHOWN_PIDS);
    ProcessWarning { pids, additional }
}
fn callback_port() -> PortState {
    port_observation(
        TcpListener::bind(("127.0.0.1", CALLBACK_PORT))
            .map(drop)
            .err()
            .map(|error| error.kind()),
    )
}
fn port_observation(error: Option<std::io::ErrorKind>) -> PortState {
    match error {
        None => PortState::Available,
        Some(std::io::ErrorKind::AddrInUse) => PortState::Occupied,
        Some(_) => PortState::Unavailable,
    }
}

#[cfg(any(windows, test))]
fn parse_netstat(text: &str) -> Result<Vec<u32>, Error> {
    let mut pids = BTreeSet::new();
    for line in text.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.first() != Some(&"TCP") {
            continue;
        }
        if fields.len() != 5 {
            return Err(Error::Unavailable);
        }
        let address = fields[1]
            .parse::<std::net::SocketAddr>()
            .map_err(|_| Error::Unavailable)?;
        if address.port() != CALLBACK_PORT || fields[3] != "LISTENING" {
            continue;
        }
        let pid = fields[4]
            .parse::<u32>()
            .ok()
            .filter(|pid| *pid > 0)
            .ok_or(Error::Unavailable)?;
        pids.insert(pid);
        if pids.len() > MAX_PIDS {
            return Err(Error::Unavailable);
        }
    }
    Ok(pids.into_iter().collect())
}
#[cfg(any(not(windows), test))]
fn parse_lsof(text: &str) -> Result<Vec<u32>, Error> {
    let mut pids = BTreeSet::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let pid = line
            .trim()
            .parse::<u32>()
            .ok()
            .filter(|pid| *pid > 0)
            .ok_or(Error::Unavailable)?;
        pids.insert(pid);
        if pids.len() > MAX_PIDS {
            return Err(Error::Unavailable);
        }
    }
    Ok(pids.into_iter().collect())
}
fn checked_output(output: &CappedProcessOutput) -> Result<&str, Error> {
    if !output.status.success()
        || output.timed_out
        || output.stdout_truncated
        || output.stderr_truncated
        || output.stderr.iter().any(|byte| !byte.is_ascii_whitespace())
    {
        return Err(Error::Unavailable);
    }
    std::str::from_utf8(&output.stdout).map_err(|_| Error::Unavailable)
}
fn discover() -> Result<Vec<u32>, Error> {
    let descriptor = ProcessDescriptor::port_discovery();
    #[cfg(windows)]
    let args = [
        OsString::from("-ano"),
        OsString::from("-p"),
        OsString::from("tcp"),
    ];
    #[cfg(not(windows))]
    let args = [
        OsString::from("-nP"),
        OsString::from("-iTCP:1455"),
        OsString::from("-sTCP:LISTEN"),
        OsString::from("-t"),
    ];
    let handle = tokio::runtime::Handle::try_current().map_err(|_| Error::Unavailable)?;
    let output = handle
        .block_on(ProcessGateway::execute(&descriptor, &args))
        .map_err(|_| Error::Unavailable)?;
    let text = checked_output(&output)?;
    #[cfg(windows)]
    {
        parse_netstat(text)
    }
    #[cfg(not(windows))]
    {
        parse_lsof(text)
    }
}
fn cancel_owned(
    discovered: &[u32],
    owned: &[u32],
    mut cancel: impl FnMut(u32),
    port: PortState,
) -> ReleaseReport {
    let discovered = discovered.iter().copied().collect::<BTreeSet<_>>();
    let owned = owned
        .iter()
        .copied()
        .filter(|pid| discovered.contains(pid))
        .collect::<BTreeSet<_>>();
    for pid in &owned {
        cancel(*pid);
    }
    ReleaseReport {
        cancel_requested: owned.len(),
        unknown: discovered.difference(&owned).copied().collect(),
        owned: owned.into_iter().collect(),
        port,
    }
}
impl Services {
    pub fn codex_auth_diagnostic(
        &self,
        key: RequestKey,
        request: Request,
    ) -> Result<TaskSubscription<Report, Error>, AdmissionError> {
        let services = self.clone();
        let release = matches!(&request, Request::CancelOwnedListeners(_));
        let key_copy = key.clone();
        let run = move || {
            admit(&services, &key)?;
            match request {
                Request::SaveProcesses(roots) => {
                    if roots.fixture {
                        return Err(Error::Isolated);
                    }
                    let service = ccr_codex::CodexAuthService::from_dirs(
                        roots.codex_store,
                        roots.codex_runtime,
                    );
                    Ok(Report::Processes(process_warning(
                        service.detect_codex_process(),
                    )))
                }
                Request::CallbackPort => Ok(Report::Port(callback_port())),
                Request::PrepareCleanup(roots) => {
                    if roots.fixture {
                        return Err(Error::Isolated);
                    }
                    services
                        .platform_oauth
                        .prepare_cleanup(&roots)
                        .map(|plan| Report::CleanupPlan(Arc::new(plan)))
                        .map_err(|_| Error::CleanupPending)
                }
                Request::CancelOwnedListeners(plan) => {
                    services
                        .platform_oauth
                        .cancel_captured(&plan)
                        .map_err(|_| Error::CleanupPending)?;
                    let discovered = discover()?;
                    let owned =
                        ProcessGateway::owned_processes_for_port(&discovered, CALLBACK_PORT);
                    let ids = owned.iter().map(|process| process.pid).collect::<Vec<_>>();
                    let mut report = cancel_owned(
                        &discovered,
                        &ids,
                        |pid| {
                            if let Some(process) = owned.iter().find(|process| process.pid == pid) {
                                process.request_cancel();
                            }
                        },
                        PortState::Unavailable,
                    );
                    report.port = callback_port();
                    Ok(Report::Release(report))
                }
            }
        };
        // Release is app-owned once confirmed; reads cancel when their view leaves.
        if release {
            self.executor.job(key_copy, move |_, _| match run() {
                Ok(value) => Completion::Success(value),
                Err(error) => Completion::Failed(error),
            })
        } else {
            self.executor.read(key_copy, move |_, _| run())
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn parsers_only_accept_exact_local_listener_port_and_valid_pids() {
        assert_eq!(parse_netstat("TCP 127.0.0.1:1455 0.0.0.0:0 LISTENING 41\nTCP [::]:1455 [::]:0 LISTENING 42\nTCP 127.0.0.1:14550 0.0.0.0:0 LISTENING 43\nTCP 127.0.0.1:99 127.0.0.1:1455 ESTABLISHED 44").unwrap(),[41,42]);
        assert!(parse_netstat("TCP 127.0.0.1:1455 0.0.0.0:0 LISTENING invalid").is_err());
        assert!(parse_netstat("TCP invalid row").is_err());
        assert_eq!(parse_lsof("42\n41\n42\n").unwrap(), [41, 42]);
        assert!(parse_lsof("permission denied").is_err());
    }
    #[test]
    fn bind_observations_keep_uncheckable_distinct_from_free_and_busy() {
        assert_eq!(port_observation(None), PortState::Available);
        assert_eq!(
            port_observation(Some(std::io::ErrorKind::AddrInUse)),
            PortState::Occupied
        );
        assert_eq!(
            port_observation(Some(std::io::ErrorKind::PermissionDenied)),
            PortState::Unavailable
        );
    }
    #[test]
    fn report_requests_only_discovered_owned_handles_and_bounds_process_warning() {
        let mut called = Vec::new();
        let report = cancel_owned(
            &[41, 42, 41],
            &[41, 99],
            |pid| called.push(pid),
            PortState::Occupied,
        );
        assert_eq!(called, [41]);
        assert_eq!(report.unknown, [42]);
        assert_eq!(report.cancel_requested, 1);
        assert_eq!(report.port, PortState::Occupied);
        let warning = process_warning((1..40).collect());
        assert_eq!(warning.pids.len(), 8);
        assert_eq!(warning.additional, 31);
    }
    #[test]
    fn fixture_remote_and_wrong_platform_are_rejected_before_any_probe() {
        let root = tempfile::tempdir().unwrap();
        let owner = ccr_app::runtime::RuntimeOwner::new().unwrap();
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("registry.toml"),
                delay: std::time::Duration::ZERO,
            },
        );
        let mut key = RequestKey {
            environment: EnvironmentId::Local("fixture".into()),
            platform: Platform::Codex,
            generation: 0,
            request_id: 0,
        };
        assert_eq!(admit(&services, &key), Err(Error::Isolated));
        key.environment = EnvironmentId::Ssh("controlled".into());
        assert_eq!(admit(&services, &key), Err(Error::Unsupported));
        key.environment = EnvironmentId::Local("fixture".into());
        key.platform = Platform::Claude;
        assert_eq!(admit(&services, &key), Err(Error::Unsupported));
        assert!(root.path().read_dir().unwrap().next().is_none());
        owner.shutdown();
    }
    #[test]
    fn failed_timed_out_truncated_and_invalid_utf8_probe_outputs_are_unknown() {
        #[cfg(unix)]
        use std::os::unix::process::ExitStatusExt;
        #[cfg(windows)]
        use std::os::windows::process::ExitStatusExt;
        let mut output = CappedProcessOutput {
            status: std::process::ExitStatus::from_raw(0),
            stdout: b"41\n".to_vec(),
            stderr: Vec::new(),
            stdout_bytes: 3,
            stderr_bytes: 0,
            stdout_truncated: false,
            stderr_truncated: false,
            timed_out: false,
            duration: std::time::Duration::ZERO,
        };
        assert!(checked_output(&output).is_ok());
        output.timed_out = true;
        assert!(checked_output(&output).is_err());
        output.timed_out = false;
        output.stdout_truncated = true;
        assert!(checked_output(&output).is_err());
        output.stdout_truncated = false;
        output.stderr_truncated = true;
        assert!(checked_output(&output).is_err());
        output.stderr_truncated = false;
        output.stderr = b"permission warning".to_vec();
        assert!(checked_output(&output).is_err());
        output.stderr.clear();
        output.status = std::process::ExitStatus::from_raw(1);
        assert!(checked_output(&output).is_err());
        output.status = std::process::ExitStatus::from_raw(0);
        output.stdout = vec![255];
        assert!(checked_output(&output).is_err());
    }
}
