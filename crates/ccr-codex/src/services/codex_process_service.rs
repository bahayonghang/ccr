// Codex process cleanup service.
//
// The matcher only accepts native Codex or the Node Codex wrapper followed by
// the exact `app-server` subcommand. Process ownership and identity are checked
// again immediately before every signal.

use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

use ccr_core::core::process_gateway::{ManagedProcess, read_bounded_line};
use serde::Deserialize;
use sysinfo::{
    Pid, Process, ProcessRefreshKind, ProcessesToUpdate, Signal, System, Uid, UpdateKind,
    get_current_pid,
};
use tokio::io::BufReader;

use crate::utils::{CodexPaths, which_on_path};

/// A Codex app-server process visible to the caller.
///
/// `cmdline` is a redacted display summary, not the raw process command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexAppServer {
    pub pid: u32,
    pub cmdline: String,
}

/// How a process was confirmed to have stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminationKind {
    /// Unix: exited after SIGTERM.
    Term,
    /// Unix: SIGKILL after the grace period / Windows: terminate.
    Kill,
    /// The process stopped matching before a signal could be delivered.
    AlreadyGone,
}

/// Backward-compatible cleanup result.
#[derive(Debug, Clone, Default)]
pub struct CodexAppServerCleanup {
    /// App-servers found in the initial snapshot.
    pub found: Vec<CodexAppServer>,
    /// Confirmed stopped PIDs and the strongest successful action for each PID.
    pub terminated: Vec<(u32, TerminationKind)>,
    /// App-servers still present after the settle window.
    pub respawned: Vec<CodexAppServer>,
    /// Whether no signals were sent.
    pub dry_run: bool,
}

/// A process-discovery condition that makes cleanup unsafe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexProcessDiscoveryIssue {
    CurrentProcessUnavailable,
    CurrentOwnerUnavailable,
    CommandLineUnavailable,
}

impl CodexProcessDiscoveryIssue {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CurrentProcessUnavailable => "current_process_unavailable",
            Self::CurrentOwnerUnavailable => "current_owner_unavailable",
            Self::CommandLineUnavailable => "command_line_unavailable",
        }
    }
}

/// Signal stage used by a failed delivery attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodexSignalStage {
    Term,
    Kill,
}

impl CodexSignalStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Term => "term",
            Self::Kill => "kill",
        }
    }
}

/// A signal was supported but the operating system rejected its delivery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodexSignalFailure {
    pub pid: u32,
    pub stage: CodexSignalStage,
}

/// Detailed cleanup report used by command surfaces that need failure semantics.
#[derive(Debug, Clone, Default)]
pub struct CodexAppServerCleanupReport {
    pub cleanup: CodexAppServerCleanup,
    pub discovered_during_cleanup: Vec<CodexAppServer>,
    pub signal_failures: Vec<CodexSignalFailure>,
    pub discovery_issue: Option<CodexProcessDiscoveryIssue>,
}

/// The managed Codex app-server daemon detected through `daemon.pid`.
///
/// `cmdline` is a redacted display summary, not the raw process command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexDaemon {
    pub pid: u32,
    pub cmdline: String,
}

/// `daemon.pid` 的最小解析结构：判定只依赖 `pid` 字段，
/// `processStartTime` 与 `executableIdentity` 仅作诊断保留。
#[derive(Debug, Deserialize)]
struct DaemonPidRecord {
    pid: u32,
    #[serde(rename = "processStartTime")]
    process_start_time: Option<String>,
    #[serde(rename = "executableIdentity")]
    _executable_identity: Option<serde_json::Value>,
}

/// SIGTERM polling interval.
const POLL_INTERVAL: Duration = Duration::from_millis(300);
/// Grace-period poll count (about three seconds).
const POLL_ROUNDS: u32 = 10;
/// Delay before the final respawn/remaining snapshot.
const RESPAWN_SETTLE: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy)]
struct CleanupTiming {
    poll_interval: Duration,
    poll_rounds: u32,
    respawn_settle: Duration,
}

impl Default for CleanupTiming {
    fn default() -> Self {
        Self {
            poll_interval: POLL_INTERVAL,
            poll_rounds: POLL_ROUNDS,
            respawn_settle: RESPAWN_SETTLE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ProcessIdentity {
    pid: u32,
    start_time: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrackedProcess {
    identity: ProcessIdentity,
    display: CodexAppServer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProcessDiscovery {
    targets: Vec<TrackedProcess>,
    alive_identities: HashSet<ProcessIdentity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SignalAttempt {
    Sent,
    Unsupported,
    Failed,
    NoLongerTarget,
    DiscoveryUnavailable(CodexProcessDiscoveryIssue),
}

trait ProcessBackend {
    fn discover(&mut self) -> Result<ProcessDiscovery, CodexProcessDiscoveryIssue>;
    fn signal(&mut self, target: &TrackedProcess, stage: CodexSignalStage) -> SignalAttempt;

    fn wait(&mut self, duration: Duration) {
        thread::sleep(duration);
    }
}

struct SysinfoProcessBackend {
    system: System,
}

impl Default for SysinfoProcessBackend {
    fn default() -> Self {
        Self {
            system: System::new(),
        }
    }
}

impl ProcessBackend for SysinfoProcessBackend {
    fn discover(&mut self) -> Result<ProcessDiscovery, CodexProcessDiscoveryIssue> {
        let (current_pid, current_owner) = self.refresh_all()?;
        let mut discovery = collect_process_discovery(&self.system, current_pid, &current_owner);
        discovery
            .targets
            .sort_by_key(|process| (process.identity.pid, process.identity.start_time));
        Ok(discovery)
    }

    fn signal(&mut self, target: &TrackedProcess, stage: CodexSignalStage) -> SignalAttempt {
        let current_pid = match get_current_pid() {
            Ok(pid) => pid,
            Err(_) => {
                return SignalAttempt::DiscoveryUnavailable(
                    CodexProcessDiscoveryIssue::CurrentProcessUnavailable,
                );
            }
        };
        let target_pid = Pid::from_u32(target.identity.pid);
        let pids = [current_pid, target_pid];
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&pids),
            false,
            process_refresh_kind(),
        );

        let current_owner = match current_process_owner(&self.system, current_pid) {
            Ok(owner) => owner,
            Err(issue) => return SignalAttempt::DiscoveryUnavailable(issue),
        };
        let Some(process) = self.system.process(target_pid) else {
            return SignalAttempt::NoLongerTarget;
        };
        if target_pid == current_pid
            || process.start_time() != target.identity.start_time
            || process_owner(process) != Some(&current_owner)
            || !is_codex_app_server(process.cmd())
        {
            return SignalAttempt::NoLongerTarget;
        }

        match stage {
            CodexSignalStage::Term => match process.kill_with(Signal::Term) {
                Some(true) => SignalAttempt::Sent,
                Some(false) => SignalAttempt::Failed,
                None => SignalAttempt::Unsupported,
            },
            CodexSignalStage::Kill => {
                if process.kill() {
                    SignalAttempt::Sent
                } else {
                    SignalAttempt::Failed
                }
            }
        }
    }
}

impl SysinfoProcessBackend {
    fn refresh_all(&mut self) -> Result<(Pid, Uid), CodexProcessDiscoveryIssue> {
        let current_pid =
            get_current_pid().map_err(|_| CodexProcessDiscoveryIssue::CurrentProcessUnavailable)?;
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            process_refresh_kind(),
        );
        let current_owner = current_process_owner(&self.system, current_pid)?;
        Ok((current_pid, current_owner))
    }
}

/// Stateless Codex process cleanup service.
#[derive(Debug, Default)]
pub struct CodexProcessService;

impl CodexProcessService {
    pub fn new() -> Self {
        Self
    }

    /// Enumerate owner-scoped Codex app-server processes.
    ///
    /// This compatibility API returns an empty list when a safe snapshot cannot
    /// be established. Use [`Self::cleanup_report`] when the distinction matters.
    pub fn find_app_servers(&self) -> Vec<CodexAppServer> {
        let mut backend = SysinfoProcessBackend::default();
        backend
            .discover()
            .map(|discovery| public_apps(&discovery.targets))
            .unwrap_or_default()
    }

    /// Run cleanup and project the detailed state machine onto the legacy result.
    pub fn cleanup(&self, dry_run: bool) -> CodexAppServerCleanup {
        self.cleanup_report(dry_run).cleanup
    }

    /// Run owner-scoped cleanup with explicit discovery and signal failure details.
    pub fn cleanup_report(&self, dry_run: bool) -> CodexAppServerCleanupReport {
        let mut backend = SysinfoProcessBackend::default();
        cleanup_with_backend(&mut backend, dry_run, CleanupTiming::default())
    }

    /// Detect the managed app-server daemon from `daemon.pid`.
    ///
    /// Reads `app-server-daemon/daemon.pid` under the resolved Codex home and
    /// cross-checks the recorded PID against the owner-scoped narrow argv match.
    /// A missing file, unparsable JSON, or a PID outside the target set returns
    /// `None`. Detection sends no signals and starts no external processes.
    pub fn find_managed_daemon(&self) -> Option<CodexDaemon> {
        let paths = match CodexPaths::resolve() {
            Ok(paths) => paths,
            Err(error) => {
                tracing::debug!(%error, "解析 Codex 路径失败，跳过守护进程检测");
                return None;
            }
        };
        let pid_path = paths.codex_dir.join("app-server-daemon").join("daemon.pid");
        let raw = match std::fs::read_to_string(&pid_path) {
            Ok(raw) => raw,
            Err(error) => {
                tracing::debug!(%error, path = %pid_path.display(), "未找到守护进程 pid 文件");
                return None;
            }
        };
        let record: DaemonPidRecord = match serde_json::from_str(&raw) {
            Ok(record) => record,
            Err(error) => {
                tracing::debug!(%error, path = %pid_path.display(), "守护进程 pid 文件解析失败");
                return None;
            }
        };
        let mut backend = SysinfoProcessBackend::default();
        let discovery = match backend.discover() {
            Ok(discovery) => discovery,
            Err(issue) => {
                tracing::debug!(issue = issue.as_str(), "进程枚举不可用，跳过守护进程检测");
                return None;
            }
        };
        let target = discovery
            .targets
            .iter()
            .find(|process| process.identity.pid == record.pid)?;
        tracing::debug!(
            pid = record.pid,
            start_time = ?record.process_start_time,
            "已确认托管 app-server 守护进程"
        );
        Some(CodexDaemon {
            pid: target.identity.pid,
            cmdline: target.display.cmdline.clone(),
        })
    }
}

/// Deadline for the official daemon restart command.
const DAEMON_RESTART_TIMEOUT: Duration = Duration::from_secs(30);
/// Grace period for reclaiming the restart process tree after the deadline.
const DAEMON_RESTART_TERMINATE_GRACE: Duration = Duration::from_secs(1);
const DAEMON_RESTART_MAX_LINE_BYTES: usize = 16 * 1024;
const DAEMON_RESTART_MAX_TOTAL_BYTES: usize = 32 * 1024;
/// 等待排空任务的固定上限；超时按已收集内容继续，防止存活后代持有管道写端时挂起调用方。
const DAEMON_RESTART_DRAIN_WAIT: Duration = Duration::from_secs(2);

/// Outcome of a managed app-server daemon restart attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonRestartOutcome {
    /// The official restart command succeeded.
    ///
    /// `pid` is the new daemon PID when it could be confirmed, `None` when the
    /// new process was not visible yet.
    Restarted { pid: Option<u32> },
    /// The restart command failed or could not start.
    Failed { detail: String },
    /// The restart command exceeded its deadline; its process tree was reclaimed.
    Timeout,
    /// The `codex` binary is not available on PATH.
    Unavailable,
}

/// Restart the managed app-server daemon through the official CLI.
///
/// Runs `codex app-server daemon restart` as one detached managed process with
/// a 30-second deadline: the new daemon is the command's product and must
/// survive the command's exit; a timeout still reclaims the spawned tree.
/// Detection sends no signals.
pub async fn restart_codex_daemon() -> DaemonRestartOutcome {
    let Some(bin) = which_on_path("codex") else {
        tracing::debug!("PATH 中找不到 codex，无法重启守护进程");
        return DaemonRestartOutcome::Unavailable;
    };
    restart_codex_daemon_at(&bin, DAEMON_RESTART_TIMEOUT).await
}

/// 带可注入截止时间的重启入口（测试 seam）。
async fn restart_codex_daemon_at(bin: &Path, timeout: Duration) -> DaemonRestartOutcome {
    let mut command = tokio::process::Command::new(bin);
    command
        .args(["app-server", "daemon", "restart"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // 守护进程是本命令的产物：成功路径必须让它比命令活得更久，
    // 使用分离模式（成功等待与 Drop 都不清理后代；超时仍显式回收整棵树）。
    let mut child = match ManagedProcess::spawn_detached(command) {
        Ok(child) => child,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!(%error, "codex 可执行文件不可用，无法重启守护进程");
            return DaemonRestartOutcome::Unavailable;
        }
        Err(error) => {
            return DaemonRestartOutcome::Failed {
                detail: format!("spawn failed: {error}"),
            };
        }
    };

    // 并发排空受限 stdout/stderr，防止管道写满阻塞子进程。
    let stdout_task = tokio::spawn(drain_bounded_pipe(child.take_stdout()));
    let stderr_task = tokio::spawn(drain_bounded_pipe(child.take_stderr()));

    match tokio::time::timeout(timeout, child.wait()).await {
        Ok(Ok(status)) => {
            let _ = drain_within_bound(stdout_task).await;
            let stderr_bytes = drain_within_bound(stderr_task).await;
            if status.success() {
                DaemonRestartOutcome::Restarted {
                    pid: confirmed_daemon_pid(),
                }
            } else {
                DaemonRestartOutcome::Failed {
                    detail: daemon_restart_exit_detail(status, &stderr_bytes),
                }
            }
        }
        Ok(Err(error)) => {
            let _ = drain_within_bound(stdout_task).await;
            let _ = drain_within_bound(stderr_task).await;
            DaemonRestartOutcome::Failed {
                detail: format!("wait failed: {error}"),
            }
        }
        Err(_) => {
            // 超时只回收本次 spawn 的进程树，不触碰守护进程自身。
            if let Err(error) = child.terminate_tree(DAEMON_RESTART_TERMINATE_GRACE).await {
                tracing::warn!(%error, "守护进程重启进程树回收失败");
            }
            let _ = drain_within_bound(stdout_task).await;
            let _ = drain_within_bound(stderr_task).await;
            DaemonRestartOutcome::Timeout
        }
    }
}

/// 尽力确认重启后的新守护进程 PID（未就绪时返回 None）。
fn confirmed_daemon_pid() -> Option<u32> {
    CodexProcessService::new()
        .find_managed_daemon()
        .map(|daemon| daemon.pid)
}

/// 失败摘要：退出码 + 受限的 stderr 首行（不落全量输出）。
fn daemon_restart_exit_detail(status: std::process::ExitStatus, stderr_bytes: &[u8]) -> String {
    let code = status
        .code()
        .map_or_else(|| "unknown".to_string(), |code| code.to_string());
    match stderr_summary_line(stderr_bytes) {
        Some(line) => format!("exit code {code}: {line}"),
        None => format!("exit code {code}"),
    }
}

/// 提取首行非空 stderr 作为受限摘要（去控制字符，最多 160 字符）。
fn stderr_summary_line(bytes: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(bytes);
    let line = text.lines().map(str::trim).find(|line| !line.is_empty())?;
    let summary: String = line
        .chars()
        .filter(|ch| !ch.is_control())
        .take(160)
        .collect();
    (!summary.is_empty()).then_some(summary)
}

/// 受限排空管道：单行与总量都有上限，超限只截断不阻塞。
async fn drain_bounded_pipe<R>(pipe: Option<R>) -> Vec<u8>
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    let Some(pipe) = pipe else {
        return Vec::new();
    };
    let mut reader = BufReader::new(pipe);
    let mut out = Vec::new();
    while let Ok(Some(line)) = read_bounded_line(&mut reader, DAEMON_RESTART_MAX_LINE_BYTES).await {
        if out.len() >= DAEMON_RESTART_MAX_TOTAL_BYTES {
            break;
        }
        let remaining = DAEMON_RESTART_MAX_TOTAL_BYTES - out.len();
        let bytes = line.text.as_bytes();
        let take = bytes.len().min(remaining);
        out.extend_from_slice(&bytes[..take]);
        if out.len() < DAEMON_RESTART_MAX_TOTAL_BYTES {
            out.push(b'\n');
        }
    }
    out
}

/// 在 [`DAEMON_RESTART_DRAIN_WAIT`] 内等待排空任务；超时或任务失败返回已收集内容。
async fn drain_within_bound(task: tokio::task::JoinHandle<Vec<u8>>) -> Vec<u8> {
    match tokio::time::timeout(DAEMON_RESTART_DRAIN_WAIT, task).await {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(_)) | Err(_) => Vec::new(),
    }
}

fn cleanup_with_backend<B: ProcessBackend>(
    backend: &mut B,
    dry_run: bool,
    timing: CleanupTiming,
) -> CodexAppServerCleanupReport {
    let mut report = CodexAppServerCleanupReport {
        cleanup: CodexAppServerCleanup {
            dry_run,
            ..Default::default()
        },
        ..Default::default()
    };

    let initial = match backend.discover() {
        Ok(discovery) => discovery.targets,
        Err(issue) => {
            report.discovery_issue = Some(issue);
            return report;
        }
    };
    report.cleanup.found = public_apps(&initial);
    if dry_run || initial.is_empty() {
        return report;
    }

    let initial_identities: HashSet<_> = initial.iter().map(|process| process.identity).collect();
    let mut seen: HashMap<ProcessIdentity, TrackedProcess> = initial
        .iter()
        .cloned()
        .map(|process| (process.identity, process))
        .collect();
    let mut successful_signals = HashMap::new();

    for process in &initial {
        let attempt = backend.signal(process, CodexSignalStage::Term);
        if attempt == SignalAttempt::Unsupported {
            let fallback = backend.signal(process, CodexSignalStage::Kill);
            if !record_signal_attempt(
                &mut report,
                &mut successful_signals,
                process,
                CodexSignalStage::Kill,
                fallback,
            ) {
                return report;
            }
        } else if !record_signal_attempt(
            &mut report,
            &mut successful_signals,
            process,
            CodexSignalStage::Term,
            attempt,
        ) {
            return report;
        }
    }

    let mut current = initial;
    for _ in 0..timing.poll_rounds {
        backend.wait(timing.poll_interval);
        current = match backend.discover() {
            Ok(discovery) => discovery.targets,
            Err(issue) => {
                report.discovery_issue = Some(issue);
                return report;
            }
        };
        record_new_processes(&mut report, &mut seen, &initial_identities, &current);
        // 匹配目标已空则结束宽限，不再继续 wait / discover。
        if current.is_empty() {
            break;
        }
    }

    // 截止时仍匹配的身份（含宽限内出现的 replacement）发 KILL。
    if !current.is_empty() {
        for process in &current {
            let attempt = backend.signal(process, CodexSignalStage::Kill);
            if !record_signal_attempt(
                &mut report,
                &mut successful_signals,
                process,
                CodexSignalStage::Kill,
                attempt,
            ) {
                return report;
            }
        }
    }

    // 本轮已对真实目标发过信号：始终 settle 后再拍最终快照。
    backend.wait(timing.respawn_settle);
    let final_discovery = match backend.discover() {
        Ok(discovery) => discovery,
        Err(issue) => {
            report.discovery_issue = Some(issue);
            return report;
        }
    };
    report.cleanup.respawned = public_apps(&final_discovery.targets);

    let mut terminated_by_pid = HashMap::new();
    for identity in seen.keys() {
        if final_discovery.alive_identities.contains(identity) {
            continue;
        }
        let kind = successful_signals
            .get(identity)
            .copied()
            .unwrap_or(TerminationKind::AlreadyGone);
        terminated_by_pid
            .entry(identity.pid)
            .and_modify(|existing| {
                if termination_rank(kind) > termination_rank(*existing) {
                    *existing = kind;
                }
            })
            .or_insert(kind);
    }
    report.cleanup.terminated = terminated_by_pid.into_iter().collect();
    report.cleanup.terminated.sort_by_key(|(pid, _)| *pid);
    report
}

fn record_signal_attempt(
    report: &mut CodexAppServerCleanupReport,
    successful_signals: &mut HashMap<ProcessIdentity, TerminationKind>,
    process: &TrackedProcess,
    stage: CodexSignalStage,
    attempt: SignalAttempt,
) -> bool {
    match attempt {
        SignalAttempt::Sent => {
            let kind = match stage {
                CodexSignalStage::Term => TerminationKind::Term,
                CodexSignalStage::Kill => TerminationKind::Kill,
            };
            successful_signals.insert(process.identity, kind);
            true
        }
        SignalAttempt::Failed | SignalAttempt::Unsupported => {
            report.signal_failures.push(CodexSignalFailure {
                pid: process.identity.pid,
                stage,
            });
            true
        }
        SignalAttempt::NoLongerTarget => true,
        SignalAttempt::DiscoveryUnavailable(issue) => {
            report.discovery_issue = Some(issue);
            false
        }
    }
}

fn record_new_processes(
    report: &mut CodexAppServerCleanupReport,
    seen: &mut HashMap<ProcessIdentity, TrackedProcess>,
    initial_identities: &HashSet<ProcessIdentity>,
    current: &[TrackedProcess],
) {
    for process in current {
        if !seen.contains_key(&process.identity) && !initial_identities.contains(&process.identity)
        {
            report
                .discovered_during_cleanup
                .push(process.display.clone());
        }
        seen.entry(process.identity)
            .or_insert_with(|| process.clone());
    }
    report
        .discovered_during_cleanup
        .sort_by_key(|process| process.pid);
}

fn termination_rank(kind: TerminationKind) -> u8 {
    match kind {
        TerminationKind::AlreadyGone => 0,
        TerminationKind::Term => 1,
        TerminationKind::Kill => 2,
    }
}

fn process_refresh_kind() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing()
        .with_cmd(UpdateKind::Always)
        .with_user(UpdateKind::Always)
        .without_tasks()
}

fn current_process_owner(
    system: &System,
    current_pid: Pid,
) -> Result<Uid, CodexProcessDiscoveryIssue> {
    let process = system
        .process(current_pid)
        .ok_or(CodexProcessDiscoveryIssue::CurrentProcessUnavailable)?;
    if process.cmd().is_empty() {
        return Err(CodexProcessDiscoveryIssue::CommandLineUnavailable);
    }
    process_owner(process)
        .cloned()
        .ok_or(CodexProcessDiscoveryIssue::CurrentOwnerUnavailable)
}

fn process_owner(process: &Process) -> Option<&Uid> {
    process.effective_user_id().or_else(|| process.user_id())
}

fn collect_process_discovery(
    system: &System,
    current_pid: Pid,
    current_owner: &Uid,
) -> ProcessDiscovery {
    let mut targets = Vec::new();
    let mut alive_identities = HashSet::new();
    for (pid, process) in system.processes() {
        let identity = ProcessIdentity {
            pid: pid.as_u32(),
            start_time: process.start_time(),
        };
        alive_identities.insert(identity);
        if *pid == current_pid
            || process_owner(process) != Some(current_owner)
            || !is_codex_app_server(process.cmd())
        {
            continue;
        }
        targets.push(TrackedProcess {
            identity,
            display: CodexAppServer {
                pid: pid.as_u32(),
                cmdline: app_server_display_summary(process.cmd()),
            },
        });
    }
    ProcessDiscovery {
        targets,
        alive_identities,
    }
}

fn public_apps(processes: &[TrackedProcess]) -> Vec<CodexAppServer> {
    let mut apps: Vec<_> = processes
        .iter()
        .map(|process| process.display.clone())
        .collect();
    apps.sort_by_key(|process| process.pid);
    apps
}

/// Match native Codex or a Node wrapper followed by the exact `app-server` subcommand.
fn is_codex_app_server(args: &[OsString]) -> bool {
    let Some(command_index) = codex_command_index(args) else {
        return false;
    };
    args.iter()
        .skip(command_index + 1)
        .any(|arg| arg.eq_ignore_ascii_case("app-server"))
}

fn codex_command_index(args: &[OsString]) -> Option<usize> {
    if args.first().is_some_and(|arg| is_codex_launcher(arg)) {
        return Some(0);
    }
    let is_node_wrapper = args.first().is_some_and(|arg| {
        executable_stem(arg).is_some_and(|name| name.eq_ignore_ascii_case("node"))
    });
    (is_node_wrapper && args.get(1).is_some_and(|arg| is_codex_launcher(arg))).then_some(1)
}

fn is_codex_launcher(value: &OsStr) -> bool {
    executable_stem(value).is_some_and(|name| name.eq_ignore_ascii_case("codex"))
}

fn executable_stem(value: &OsStr) -> Option<&str> {
    Path::new(value).file_stem()?.to_str()
}

fn app_server_display_summary(args: &[OsString]) -> String {
    match codex_command_index(args) {
        Some(1) => "node codex app-server".to_string(),
        _ => "codex app-server".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet, VecDeque};
    use std::ffi::OsString;
    use std::path::PathBuf;
    use std::time::Duration;

    use super::{
        CleanupTiming, CodexProcessDiscoveryIssue, CodexSignalStage, DaemonRestartOutcome,
        ProcessBackend, ProcessDiscovery, ProcessIdentity, SignalAttempt, TerminationKind,
        TrackedProcess, cleanup_with_backend, is_codex_app_server, restart_codex_daemon_at,
    };
    #[cfg(windows)]
    use super::{SysinfoProcessBackend, process_refresh_kind};

    #[cfg(unix)]
    struct ChildGuard(std::process::Child);

    #[cfg(unix)]
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    #[cfg(windows)]
    struct ChildGuard(std::process::Child);

    #[cfg(windows)]
    impl ChildGuard {
        fn wait_for_exit(&mut self) -> bool {
            for _ in 0..50 {
                match self.0.try_wait() {
                    Ok(Some(_)) => return true,
                    Ok(None) => std::thread::sleep(Duration::from_millis(20)),
                    Err(_) => return false,
                }
            }
            false
        }
    }

    #[cfg(windows)]
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    #[cfg(windows)]
    #[test]
    fn app_server_fixture_child() {
        if std::env::var_os("CCR_SYSINFO_PROCESS_FIXTURE").is_some() {
            std::thread::sleep(Duration::from_secs(30));
        }
    }

    #[cfg(unix)]
    #[test]
    fn discovers_real_same_user_app_server_process_without_leaking_arguments() {
        use super::CodexProcessService;
        use std::os::unix::process::CommandExt;
        use std::process::Command;
        use std::thread;

        let sentinel = "CODEX_FIX_SENTINEL_SECRET";
        let mut command = Command::new("sh");
        command.arg0("codex").args([
            "-c",
            "trap 'exit 0' TERM; while :; do sleep 1; done",
            "app-server",
            sentinel,
        ]);
        let child = command.spawn().expect("fixture app-server should start");
        let child_pid = child.id();
        let _guard = ChildGuard(child);
        thread::sleep(Duration::from_millis(100));

        let report = CodexProcessService::new().cleanup_report(true);
        let fixture = report
            .cleanup
            .found
            .iter()
            .find(|app| app.pid == child_pid)
            .expect("fixture app-server should be discovered");
        assert_eq!(fixture.cmdline, "codex app-server");
        assert!(!fixture.cmdline.contains(sentinel));
        assert!(report.discovery_issue.is_none());
        assert!(report.cleanup.terminated.is_empty());
    }

    #[test]
    fn matches_native_and_node_app_servers() {
        assert!(matches(&[
            "/opt/codex/bin/codex",
            "--config",
            "profile=x",
            "app-server",
        ]));
        assert!(matches(&[
            "/usr/bin/node",
            "/home/u/.npm/lib/codex/bin/codex",
            "app-server",
        ]));
        assert!(matches(&["/home/u/.ccr/tools/codex", "app-server",]));
    }

    #[test]
    fn never_matches_plain_codex_tasks_or_argument_mentions() {
        for args in [
            vec!["codex"],
            vec!["codex", "exec", "run", "some task"],
            vec!["codex", "resume", "thread-a"],
            vec!["/usr/local/bin/codex", "login"],
            vec!["ccr", "codex", "fix"],
            vec!["python", "tool.py", "codex", "app-server"],
            vec!["some-other-app-server", "--port", "1234"],
        ] {
            assert!(!matches(&args), "unexpected match: {args:?}");
        }
        assert!(!is_codex_app_server(&[]));
    }

    #[cfg(windows)]
    #[test]
    fn matches_windows_native_and_node_paths() {
        assert!(matches(&[
            r"C:\Program Files\Codex\codex.exe",
            "--config",
            "profile=x",
            "app-server",
        ]));
        assert!(matches(&[
            r"C:\Program Files\nodejs\node.exe",
            r"C:\Users\test\AppData\Roaming\npm\codex.cmd",
            "app-server",
        ]));
    }

    #[test]
    fn pid_round_trip_preserves_process_identifier() {
        let native_pid = std::process::id();
        assert_eq!(sysinfo::Pid::from_u32(native_pid).as_u32(), native_pid);
    }

    #[cfg(windows)]
    #[test]
    fn windows_backend_refreshes_and_terminates_only_controlled_child() {
        use std::process::{Command, Stdio};

        use sysinfo::{Pid, ProcessesToUpdate, get_current_pid};

        let fixture_dir = tempfile::tempdir().expect("fixture directory should be created");
        let fixture_exe = fixture_dir.path().join("codex.exe");
        std::fs::copy(
            std::env::current_exe().expect("current test executable should be available"),
            &fixture_exe,
        )
        .expect("controlled Codex fixture should be copied");

        let child = Command::new(&fixture_exe)
            .arg("app_server_fixture_child")
            .arg("--nocapture")
            .arg("--skip")
            .arg("app-server")
            .env("CCR_SYSINFO_PROCESS_FIXTURE", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("controlled Codex fixture should start");
        let child_pid = Pid::from_u32(child.id());
        let mut child = ChildGuard(child);
        let current_pid = get_current_pid().expect("current PID should be available");
        let pids = [current_pid, child_pid];
        let mut backend = SysinfoProcessBackend::default();

        let mut target = None;
        for _ in 0..50 {
            backend.system.refresh_processes_specifics(
                ProcessesToUpdate::Some(&pids),
                true,
                process_refresh_kind(),
            );
            if let Some(process) = backend.system.process(child_pid)
                && is_codex_app_server(process.cmd())
            {
                target = Some(TrackedProcess {
                    identity: ProcessIdentity {
                        pid: process.pid().as_u32(),
                        start_time: process.start_time(),
                    },
                    display: super::CodexAppServer {
                        pid: process.pid().as_u32(),
                        cmdline: super::app_server_display_summary(process.cmd()),
                    },
                });
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }

        let target = target.expect("controlled fixture should be visible through targeted refresh");
        assert_eq!(target.identity.pid, child_pid.as_u32());
        assert_eq!(
            backend.signal(&target, CodexSignalStage::Term),
            SignalAttempt::Unsupported
        );
        assert_eq!(
            backend.signal(&target, CodexSignalStage::Kill),
            SignalAttempt::Sent
        );
        assert!(child.wait_for_exit(), "controlled fixture should exit");

        for _ in 0..50 {
            backend.system.refresh_processes_specifics(
                ProcessesToUpdate::Some(&[child_pid]),
                true,
                process_refresh_kind(),
            );
            if backend.system.process(child_pid).is_none() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(
            backend.system.process(child_pid).is_none(),
            "targeted refresh should remove the exited controlled fixture"
        );
    }

    #[test]
    fn records_term_exit_after_snapshot_confirmation() {
        let process = tracked(101, 1);
        let mut backend = FakeBackend::new(vec![
            Ok(vec![process.clone()]),
            Ok(Vec::new()),
            Ok(Vec::new()),
            Ok(Vec::new()),
        ]);

        let report = cleanup_with_backend(&mut backend, false, test_timing());

        assert_eq!(
            report.cleanup.terminated,
            vec![(101, TerminationKind::Term)]
        );
        assert!(report.cleanup.respawned.is_empty());
        assert_eq!(
            backend.signal_calls,
            vec![(process.identity, CodexSignalStage::Term)]
        );
    }

    #[test]
    fn escalates_every_target_present_at_deadline() {
        let initial = tracked(201, 1);
        let replacement = tracked(202, 1);
        let mut backend = FakeBackend::new(vec![
            Ok(vec![initial.clone()]),
            Ok(vec![initial.clone(), replacement.clone()]),
            Ok(vec![replacement.clone()]),
            Ok(Vec::new()),
        ]);

        let report = cleanup_with_backend(&mut backend, false, test_timing());

        assert_eq!(report.discovered_during_cleanup, vec![replacement.display]);
        assert_eq!(
            report.cleanup.terminated,
            vec![(201, TerminationKind::Term), (202, TerminationKind::Kill)]
        );
        assert_eq!(
            backend.signal_calls,
            vec![
                (initial.identity, CodexSignalStage::Term),
                (replacement.identity, CodexSignalStage::Kill),
            ]
        );
    }

    #[test]
    fn pid_reuse_is_tracked_by_start_time() {
        let initial = tracked(301, 1);
        let respawned = tracked(301, 2);
        let mut backend = FakeBackend::new(vec![
            Ok(vec![initial.clone()]),
            Ok(Vec::new()),
            Ok(vec![respawned.clone()]),
        ]);

        let report = cleanup_with_backend(&mut backend, false, test_timing());

        // 空快照结束宽限：settle 才出现的新身份只记 respawned，不补发 deadline KILL。
        assert!(report.discovered_during_cleanup.is_empty());
        assert_eq!(
            report.cleanup.terminated,
            vec![(301, TerminationKind::Term)]
        );
        assert_eq!(report.cleanup.respawned, vec![respawned.display]);
        assert_eq!(
            backend.signal_calls,
            vec![(initial.identity, CodexSignalStage::Term)]
        );
    }

    #[test]
    fn empty_snapshot_ends_grace_without_further_waits() {
        let process = tracked(701, 1);
        let mut backend = FakeBackend::new(vec![
            Ok(vec![process.clone()]),
            Ok(Vec::new()),
            Ok(Vec::new()),
        ]);
        let timing = CleanupTiming {
            poll_interval: Duration::from_millis(7),
            poll_rounds: 5,
            respawn_settle: Duration::from_millis(11),
        };

        let report = cleanup_with_backend(&mut backend, false, timing);

        assert_eq!(
            report.cleanup.terminated,
            vec![(701, TerminationKind::Term)]
        );
        assert!(report.cleanup.respawned.is_empty());
        assert_eq!(
            backend.waits,
            vec![Duration::from_millis(7), Duration::from_millis(11)]
        );
        assert_eq!(backend.discovery_calls, 3);
        assert_eq!(
            backend.signal_calls,
            vec![(process.identity, CodexSignalStage::Term)]
        );
    }

    #[test]
    fn settle_only_replacement_is_respawned_without_deadline_kill() {
        let initial = tracked(901, 1);
        let replacement = tracked(902, 1);
        let mut backend = FakeBackend::new(vec![
            Ok(vec![initial.clone()]),
            Ok(Vec::new()),
            Ok(vec![replacement.clone()]),
        ]);

        let report = cleanup_with_backend(&mut backend, false, test_timing());

        assert!(report.discovered_during_cleanup.is_empty());
        assert_eq!(
            report.cleanup.terminated,
            vec![(901, TerminationKind::Term)]
        );
        assert_eq!(report.cleanup.respawned, vec![replacement.display]);
        assert_eq!(
            backend.signal_calls,
            vec![(initial.identity, CodexSignalStage::Term)]
        );
    }

    #[test]
    fn remaining_targets_use_full_poll_rounds_then_kill() {
        let process = tracked(801, 1);
        let mut backend = FakeBackend::new(vec![
            Ok(vec![process.clone()]),
            Ok(vec![process.clone()]),
            Ok(vec![process.clone()]),
            Ok(vec![process.clone()]),
            Ok(Vec::new()),
        ]);
        let timing = CleanupTiming {
            poll_interval: Duration::from_millis(3),
            poll_rounds: 3,
            respawn_settle: Duration::from_millis(5),
        };

        let report = cleanup_with_backend(&mut backend, false, timing);

        assert_eq!(
            report.cleanup.terminated,
            vec![(801, TerminationKind::Kill)]
        );
        assert!(report.cleanup.respawned.is_empty());
        assert_eq!(
            backend.waits,
            vec![
                Duration::from_millis(3),
                Duration::from_millis(3),
                Duration::from_millis(3),
                Duration::from_millis(5),
            ]
        );
        assert_eq!(
            backend.signal_calls,
            vec![
                (process.identity, CodexSignalStage::Term),
                (process.identity, CodexSignalStage::Kill),
            ]
        );
    }

    #[test]
    fn failed_signals_are_not_reported_as_termination() {
        let process = tracked(401, 1);
        let mut backend = FakeBackend::new(vec![
            Ok(vec![process.clone()]),
            Ok(vec![process.clone()]),
            Ok(vec![process.clone()]),
            Ok(vec![process.clone()]),
        ]);
        backend.set_signal(
            process.identity,
            CodexSignalStage::Term,
            SignalAttempt::Failed,
        );
        backend.set_signal(
            process.identity,
            CodexSignalStage::Kill,
            SignalAttempt::Failed,
        );

        let report = cleanup_with_backend(&mut backend, false, test_timing());

        assert!(report.cleanup.terminated.is_empty());
        assert_eq!(report.cleanup.respawned, vec![process.display]);
        assert_eq!(report.signal_failures.len(), 2);
        assert_eq!(report.signal_failures[0].stage, CodexSignalStage::Term);
        assert_eq!(report.signal_failures[1].stage, CodexSignalStage::Kill);
    }

    #[test]
    fn living_identity_that_stops_matching_is_not_reported_terminated() {
        let process = tracked(450, 1);
        let no_longer_matching = ProcessDiscovery {
            targets: Vec::new(),
            alive_identities: HashSet::from([process.identity]),
        };
        let mut backend = FakeBackend::new_snapshots(vec![
            Ok(discovery(vec![process.clone()])),
            Ok(no_longer_matching.clone()),
            Ok(no_longer_matching.clone()),
            Ok(no_longer_matching),
        ]);

        let report = cleanup_with_backend(&mut backend, false, test_timing());

        assert!(report.cleanup.terminated.is_empty());
        assert!(report.cleanup.respawned.is_empty());
    }

    #[test]
    fn unsupported_term_uses_kill_and_checks_its_bool() {
        let process = tracked(501, 1);
        let mut backend = FakeBackend::new(vec![
            Ok(vec![process.clone()]),
            Ok(vec![process.clone()]),
            Ok(vec![process.clone()]),
            Ok(vec![process.clone()]),
        ]);
        backend.set_signal(
            process.identity,
            CodexSignalStage::Term,
            SignalAttempt::Unsupported,
        );
        backend.set_signal(
            process.identity,
            CodexSignalStage::Kill,
            SignalAttempt::Failed,
        );
        backend.set_signal(
            process.identity,
            CodexSignalStage::Kill,
            SignalAttempt::Failed,
        );

        let report = cleanup_with_backend(&mut backend, false, test_timing());

        assert!(report.cleanup.terminated.is_empty());
        assert_eq!(report.signal_failures.len(), 2);
        assert!(
            report
                .signal_failures
                .iter()
                .all(|failure| failure.stage == CodexSignalStage::Kill)
        );
    }

    #[test]
    fn discovery_issue_fails_closed_without_signals() {
        let mut backend = FakeBackend::new(vec![Err(
            CodexProcessDiscoveryIssue::CurrentOwnerUnavailable,
        )]);

        let report = cleanup_with_backend(&mut backend, false, test_timing());

        assert_eq!(
            report.discovery_issue,
            Some(CodexProcessDiscoveryIssue::CurrentOwnerUnavailable)
        );
        assert!(backend.signal_calls.is_empty());
        assert!(report.cleanup.terminated.is_empty());
    }

    #[test]
    fn dry_run_only_reads_the_initial_snapshot() {
        let process = tracked(601, 1);
        let mut backend = FakeBackend::new(vec![Ok(vec![process.clone()])]);

        let report = cleanup_with_backend(&mut backend, true, test_timing());

        assert_eq!(report.cleanup.found, vec![process.display]);
        assert!(report.cleanup.dry_run);
        assert!(backend.signal_calls.is_empty());
        assert!(backend.waits.is_empty());
        assert_eq!(backend.discovery_calls, 1);
    }

    #[test]
    fn empty_initial_snapshot_is_a_no_op() {
        let mut backend = FakeBackend::new(vec![Ok(Vec::new())]);

        let report = cleanup_with_backend(&mut backend, false, test_timing());

        assert!(report.cleanup.found.is_empty());
        assert!(report.cleanup.terminated.is_empty());
        assert!(report.cleanup.respawned.is_empty());
        assert!(backend.signal_calls.is_empty());
        assert!(backend.waits.is_empty());
        assert_eq!(backend.discovery_calls, 1);
    }

    fn matches(args: &[&str]) -> bool {
        let args: Vec<OsString> = args.iter().map(OsString::from).collect();
        is_codex_app_server(&args)
    }

    fn tracked(pid: u32, start_time: u64) -> TrackedProcess {
        TrackedProcess {
            identity: ProcessIdentity { pid, start_time },
            display: super::CodexAppServer {
                pid,
                cmdline: "codex app-server".to_string(),
            },
        }
    }

    fn test_timing() -> CleanupTiming {
        CleanupTiming {
            poll_interval: Duration::ZERO,
            poll_rounds: 2,
            respawn_settle: Duration::ZERO,
        }
    }

    struct FakeBackend {
        discoveries: VecDeque<Result<ProcessDiscovery, CodexProcessDiscoveryIssue>>,
        last_discovery: Option<Result<ProcessDiscovery, CodexProcessDiscoveryIssue>>,
        signal_results: HashMap<(ProcessIdentity, CodexSignalStage), VecDeque<SignalAttempt>>,
        signal_calls: Vec<(ProcessIdentity, CodexSignalStage)>,
        waits: Vec<Duration>,
        discovery_calls: usize,
    }

    impl FakeBackend {
        fn new(discoveries: Vec<Result<Vec<TrackedProcess>, CodexProcessDiscoveryIssue>>) -> Self {
            Self::new_snapshots(
                discoveries
                    .into_iter()
                    .map(|result| result.map(discovery))
                    .collect(),
            )
        }

        fn new_snapshots(
            discoveries: Vec<Result<ProcessDiscovery, CodexProcessDiscoveryIssue>>,
        ) -> Self {
            Self {
                discoveries: discoveries.into(),
                last_discovery: None,
                signal_results: HashMap::new(),
                signal_calls: Vec::new(),
                waits: Vec::new(),
                discovery_calls: 0,
            }
        }

        fn set_signal(
            &mut self,
            identity: ProcessIdentity,
            stage: CodexSignalStage,
            result: SignalAttempt,
        ) {
            self.signal_results
                .entry((identity, stage))
                .or_default()
                .push_back(result);
        }
    }

    impl ProcessBackend for FakeBackend {
        fn discover(&mut self) -> Result<ProcessDiscovery, CodexProcessDiscoveryIssue> {
            self.discovery_calls += 1;
            let result = self
                .discoveries
                .pop_front()
                .or_else(|| self.last_discovery.clone())
                .unwrap_or_else(|| Ok(discovery(Vec::new())));
            self.last_discovery = Some(result.clone());
            result
        }

        fn signal(&mut self, target: &TrackedProcess, stage: CodexSignalStage) -> SignalAttempt {
            self.signal_calls.push((target.identity, stage));
            self.signal_results
                .get_mut(&(target.identity, stage))
                .and_then(VecDeque::pop_front)
                .unwrap_or(SignalAttempt::Sent)
        }

        fn wait(&mut self, duration: Duration) {
            self.waits.push(duration);
        }
    }

    fn discovery(targets: Vec<TrackedProcess>) -> ProcessDiscovery {
        let alive_identities = targets.iter().map(|process| process.identity).collect();
        ProcessDiscovery {
            targets,
            alive_identities,
        }
    }

    #[test]
    fn managed_daemon_requires_pid_file_and_matching_target() {
        use super::CodexProcessService;

        let env = crate::test_support::TestCodexEnv::new();
        let service = CodexProcessService::new();

        // 文件缺失 → None
        assert_eq!(service.find_managed_daemon(), None);

        let daemon_dir = env.codex_dir().join("app-server-daemon");
        std::fs::create_dir_all(&daemon_dir).expect("daemon dir should be created");
        let pid_path = daemon_dir.join("daemon.pid");

        // 坏 JSON → None
        std::fs::write(&pid_path, b"{ not json").expect("bad json should be written");
        assert_eq!(service.find_managed_daemon(), None);

        // 缺少 pid 字段 → None
        std::fs::write(&pid_path, b"{\"processStartTime\":\"134358165894314170\"}")
            .expect("missing pid field should be written");
        assert_eq!(service.find_managed_daemon(), None);

        // 存活但不在目标集（当前测试进程）→ None
        std::fs::write(&pid_path, format!("{{\"pid\":{}}}", std::process::id()))
            .expect("current pid should be written");
        assert_eq!(service.find_managed_daemon(), None);

        // 无对应进程的 pid → None
        std::fs::write(&pid_path, format!("{{\"pid\":{}}}", u32::MAX))
            .expect("stale pid should be written");
        assert_eq!(service.find_managed_daemon(), None);
    }

    #[cfg(unix)]
    #[test]
    fn detects_managed_daemon_from_pid_file() {
        use super::CodexProcessService;
        use std::os::unix::process::CommandExt;
        use std::process::Command;
        use std::thread;

        let env = crate::test_support::TestCodexEnv::new();
        let sentinel = "CODEX_DAEMON_SENTINEL_SECRET";
        let mut command = Command::new("sh");
        command.arg0("codex").args([
            "-c",
            "trap 'exit 0' TERM; while :; do sleep 1; done",
            "app-server",
            sentinel,
        ]);
        let child = command.spawn().expect("fixture app-server should start");
        let child_pid = child.id();
        let _guard = ChildGuard(child);
        thread::sleep(Duration::from_millis(100));

        let daemon_dir = env.codex_dir().join("app-server-daemon");
        std::fs::create_dir_all(&daemon_dir).expect("daemon dir should be created");
        std::fs::write(
            daemon_dir.join("daemon.pid"),
            format!(
                "{{\"pid\":{child_pid},\"processStartTime\":\"134358165894314170\",\"executableIdentity\":{{\"digest\":[\"fixture\"]}}}}"
            ),
        )
        .expect("daemon.pid should be written");

        let daemon = CodexProcessService::new()
            .find_managed_daemon()
            .expect("managed daemon fixture should be detected");
        assert_eq!(daemon.pid, child_pid);
        assert_eq!(daemon.cmdline, "codex app-server");
        assert!(!daemon.cmdline.contains(sentinel));
    }

    #[cfg(windows)]
    #[test]
    fn detects_managed_daemon_from_pid_file() {
        use super::CodexProcessService;
        use std::process::{Command, Stdio};

        let env = crate::test_support::TestCodexEnv::new();
        let fixture_dir = tempfile::tempdir().expect("fixture directory should be created");
        let fixture_exe = fixture_dir.path().join("codex.exe");
        std::fs::copy(
            std::env::current_exe().expect("current test executable should be available"),
            &fixture_exe,
        )
        .expect("controlled Codex fixture should be copied");

        let child = Command::new(&fixture_exe)
            .arg("app_server_fixture_child")
            .arg("--nocapture")
            .arg("--skip")
            .arg("app-server")
            .env("CCR_SYSINFO_PROCESS_FIXTURE", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("controlled Codex fixture should start");
        let child_pid = child.id();
        let _guard = ChildGuard(child);

        let daemon_dir = env.codex_dir().join("app-server-daemon");
        std::fs::create_dir_all(&daemon_dir).expect("daemon dir should be created");
        std::fs::write(
            daemon_dir.join("daemon.pid"),
            format!("{{\"pid\":{child_pid}}}"),
        )
        .expect("daemon.pid should be written");

        let mut daemon = None;
        for _ in 0..50 {
            if let Some(found) = CodexProcessService::new().find_managed_daemon() {
                daemon = Some(found);
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let daemon = daemon.expect("managed daemon fixture should be detected");
        assert_eq!(daemon.pid, child_pid);
        assert_eq!(daemon.cmdline, "codex app-server");
    }

    #[tokio::test]
    async fn restart_success_reports_restarted_without_stale_pid() {
        let env = crate::test_support::TestCodexEnv::new();
        let temp = tempfile::tempdir().expect("tempdir");
        let bin = write_fake_codex_exit(temp.path(), 0);

        // 过期 pid 文件不得被当作新守护进程上报
        let daemon_dir = env.codex_dir().join("app-server-daemon");
        std::fs::create_dir_all(&daemon_dir).expect("daemon dir should be created");
        std::fs::write(
            daemon_dir.join("daemon.pid"),
            format!("{{\"pid\":{}}}", u32::MAX),
        )
        .expect("stale daemon.pid should be written");

        let outcome = restart_codex_daemon_at(&bin, Duration::from_secs(10)).await;
        assert_eq!(outcome, DaemonRestartOutcome::Restarted { pid: None });
    }

    #[tokio::test]
    async fn restart_nonzero_exit_reports_bounded_failure_detail() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bin = write_fake_codex_failure(temp.path(), 7);

        let outcome = restart_codex_daemon_at(&bin, Duration::from_secs(10)).await;
        match outcome {
            DaemonRestartOutcome::Failed { detail } => {
                assert!(
                    detail.contains('7'),
                    "failure detail should carry the exit code: {detail}"
                );
            }
            other => panic!("expected Failed outcome, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn restart_timeout_reclaims_process_tree() {
        let temp = tempfile::tempdir().expect("tempdir");
        let parent_pid_file = temp.path().join("parent.pid");
        let child_pid_file = temp.path().join("grandchild.pid");
        let bin = write_hanging_fake_codex(temp.path(), &parent_pid_file, &child_pid_file);

        let outcome = restart_codex_daemon_at(&bin, Duration::from_secs(3)).await;
        assert_eq!(outcome, DaemonRestartOutcome::Timeout);

        let parent = wait_for_pid_file(&parent_pid_file).await;
        let grandchild = wait_for_pid_file(&child_pid_file).await;
        wait_until_process_gone(parent).await;
        wait_until_process_gone(grandchild).await;
    }

    #[tokio::test]
    async fn restart_missing_binary_is_unavailable() {
        let temp = tempfile::tempdir().expect("tempdir");
        let missing = temp.path().join("missing-codex-binary");

        let outcome = restart_codex_daemon_at(&missing, Duration::from_secs(5)).await;
        assert_eq!(outcome, DaemonRestartOutcome::Unavailable);
    }

    #[tokio::test]
    async fn restart_success_is_bounded_when_descendant_holds_pipes() {
        let env = crate::test_support::TestCodexEnv::new();
        let temp = tempfile::tempdir().expect("tempdir");
        let holder_pid_file = temp.path().join("pipe-holder.pid");
        let bin = write_pipe_holding_fake_codex(temp.path(), &holder_pid_file);

        // 过期 pid 文件避免把既有守护进程当作新守护进程上报
        let daemon_dir = env.codex_dir().join("app-server-daemon");
        std::fs::create_dir_all(&daemon_dir).expect("daemon dir should be created");
        std::fs::write(
            daemon_dir.join("daemon.pid"),
            format!("{{\"pid\":{}}}", u32::MAX),
        )
        .expect("stale daemon.pid should be written");

        let started = std::time::Instant::now();
        let outcome = restart_codex_daemon_at(&bin, Duration::from_secs(10)).await;
        let elapsed = started.elapsed();

        assert_eq!(outcome, DaemonRestartOutcome::Restarted { pid: None });
        let holder = wait_for_pid_file(&holder_pid_file).await;
        assert!(
            test_process_is_running(holder),
            "持有管道的后代在结果返回时仍应存活 (pid {holder})"
        );
        assert!(
            elapsed < Duration::from_secs(15),
            "排空等待应有界，实际耗时 {elapsed:?}"
        );

        terminate_holder(holder);
        wait_until_process_gone(holder).await;
    }

    #[cfg(windows)]
    fn write_fake_codex_exit(dir: &std::path::Path, code: i32) -> PathBuf {
        let bin = dir.join("codex.cmd");
        std::fs::write(&bin, format!("@echo off\r\nexit /b {code}\r\n"))
            .expect("fake codex script should be written");
        bin
    }

    #[cfg(unix)]
    fn write_fake_codex_exit(dir: &std::path::Path, code: i32) -> PathBuf {
        write_executable_script(dir, "codex", &format!("#!/bin/sh\nexit {code}\n"))
    }

    #[cfg(windows)]
    fn write_fake_codex_failure(dir: &std::path::Path, code: i32) -> PathBuf {
        let bin = dir.join("codex.cmd");
        std::fs::write(
            &bin,
            format!("@echo off\r\necho restart fixture failure 1>&2\r\nexit /b {code}\r\n"),
        )
        .expect("fake codex script should be written");
        bin
    }

    #[cfg(unix)]
    fn write_fake_codex_failure(dir: &std::path::Path, code: i32) -> PathBuf {
        write_executable_script(
            dir,
            "codex",
            &format!("#!/bin/sh\necho 'restart fixture failure' 1>&2\nexit {code}\n"),
        )
    }

    #[cfg(unix)]
    fn write_executable_script(dir: &std::path::Path, name: &str, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let bin = dir.join(name);
        std::fs::write(&bin, body).expect("fake script should be written");
        let mut permissions = std::fs::metadata(&bin)
            .expect("fake script metadata should be readable")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&bin, permissions).expect("fake script should be executable");
        bin
    }

    #[cfg(unix)]
    fn write_hanging_fake_codex(
        dir: &std::path::Path,
        parent_pid: &std::path::Path,
        child_pid: &std::path::Path,
    ) -> PathBuf {
        write_executable_script(
            dir,
            "codex",
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$$\" > {parent}\n/bin/sleep 30 &\nprintf '%s\\n' \"$!\" > {child}\nwait\n",
                parent = sh_single_quote(parent_pid),
                child = sh_single_quote(child_pid),
            ),
        )
    }

    #[cfg(windows)]
    fn write_hanging_fake_codex(
        dir: &std::path::Path,
        parent_pid: &std::path::Path,
        child_pid: &std::path::Path,
    ) -> PathBuf {
        let powershell = PathBuf::from(std::env::var_os("SystemRoot").expect("Windows SystemRoot"))
            .join(r"System32\WindowsPowerShell\v1.0\powershell.exe");
        assert!(powershell.is_absolute() && powershell.is_file());
        let script_path = dir.join("hanging-codex.ps1");
        let script = format!(
            "$ErrorActionPreference = 'Stop'\n\
             [IO.File]::WriteAllText({parent}, [string]$PID)\n\
             $child = Start-Process -FilePath (Join-Path $PSHOME 'powershell.exe') -ArgumentList '-NoProfile','-NonInteractive','-Command','Start-Sleep -Seconds 30' -PassThru -WindowStyle Hidden\n\
             if (-not $child -or $child.Id -le 0) {{ throw 'failed to start grandchild' }}\n\
             [IO.File]::WriteAllText({child}, [string]$child.Id)\n\
             Start-Sleep -Seconds 30\n",
            parent = ps_single_quote(parent_pid),
            child = ps_single_quote(child_pid),
        );
        std::fs::write(&script_path, script).expect("hanging codex script should be written");
        let bin = dir.join("codex.cmd");
        std::fs::write(
            &bin,
            format!(
                "@echo off\r\n\"{}\" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File \"{}\"\r\n",
                powershell.display(),
                script_path.display()
            ),
        )
        .expect("fake codex launcher should be written");
        bin
    }

    #[cfg(unix)]
    fn write_pipe_holding_fake_codex(
        dir: &std::path::Path,
        holder_pid: &std::path::Path,
    ) -> PathBuf {
        write_executable_script(
            dir,
            "codex",
            &format!(
                "#!/bin/sh\n/bin/sleep 30 &\nprintf '%s\n' \"$!\" > {holder}\nexit 0\n",
                holder = sh_single_quote(holder_pid),
            ),
        )
    }

    #[cfg(windows)]
    fn write_pipe_holding_fake_codex(
        dir: &std::path::Path,
        holder_pid: &std::path::Path,
    ) -> PathBuf {
        let powershell = PathBuf::from(std::env::var_os("SystemRoot").expect("Windows SystemRoot"))
            .join(r"System32\WindowsPowerShell\v1.0\powershell.exe");
        assert!(powershell.is_absolute() && powershell.is_file());
        let script_path = dir.join("pipe-holder-codex.ps1");
        let script = format!(
            "$ErrorActionPreference = 'Stop'\n\
             $holder = Start-Process -FilePath (Join-Path $PSHOME 'powershell.exe') -ArgumentList '-NoProfile','-NonInteractive','-Command','Start-Sleep -Seconds 30' -PassThru -NoNewWindow\n\
             if (-not $holder -or $holder.Id -le 0) {{ throw 'failed to start pipe holder' }}\n\
             [IO.File]::WriteAllText({holder}, [string]$holder.Id)\n",
            holder = ps_single_quote(holder_pid),
        );
        std::fs::write(&script_path, script).expect("pipe holder script should be written");
        let bin = dir.join("codex.cmd");
        std::fs::write(
            &bin,
            format!(
                "@echo off\r\n\"{}\" -NoProfile -NonInteractive -ExecutionPolicy Bypass -File \"{}\"\r\n",
                powershell.display(),
                script_path.display()
            ),
        )
        .expect("fake codex launcher should be written");
        bin
    }

    #[cfg(windows)]
    fn terminate_holder(pid: u32) {
        let pid_arg = pid.to_string();
        let _ = std::process::Command::new("taskkill")
            .args(["/PID", pid_arg.as_str(), "/T", "/F"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }

    #[cfg(unix)]
    fn terminate_holder(pid: u32) {
        let pid_arg = pid.to_string();
        let _ = std::process::Command::new("kill")
            .args(["-9", pid_arg.as_str()])
            .status();
    }

    #[cfg(unix)]
    fn sh_single_quote(path: &std::path::Path) -> String {
        format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
    }

    #[cfg(windows)]
    fn ps_single_quote(path: &std::path::Path) -> String {
        format!("'{}'", path.display().to_string().replace('\'', "''"))
    }

    async fn wait_for_pid_file(path: &std::path::Path) -> u32 {
        for _ in 0..200 {
            if let Ok(raw) = std::fs::read_to_string(path)
                && let Ok(pid) = raw.trim().parse()
            {
                return pid;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        panic!("pid file was not written: {}", path.display());
    }

    async fn wait_until_process_gone(pid: u32) {
        for _ in 0..200 {
            if !test_process_is_running(pid) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        panic!("process {pid} is still running");
    }

    #[cfg(windows)]
    fn test_process_is_running(pid: u32) -> bool {
        const SYNCHRONIZE: u32 = 0x0010_0000;
        const WAIT_TIMEOUT: u32 = 258;
        unsafe extern "system" {
            fn OpenProcess(
                desired_access: u32,
                inherit_handle: i32,
                process_id: u32,
            ) -> *mut std::ffi::c_void;
            fn WaitForSingleObject(handle: *mut std::ffi::c_void, milliseconds: u32) -> u32;
            fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
        }

        // SAFETY: 同步句柄在返回前关闭；pid 来自本测试启动的子进程。
        unsafe {
            let handle = OpenProcess(SYNCHRONIZE, 0, pid);
            if handle.is_null() {
                return false;
            }
            let running = WaitForSingleObject(handle, 0) == WAIT_TIMEOUT;
            CloseHandle(handle);
            running
        }
    }

    #[cfg(unix)]
    fn test_process_is_running(pid: u32) -> bool {
        unsafe extern "C" {
            fn kill(pid: i32, signal: i32) -> i32;
        }
        // SAFETY: signal 0 只检查进程是否存在，不发送信号。
        unsafe { kill(pid as i32, 0) == 0 }
    }
}
