// Codex process cleanup service.
//
// The matcher only accepts native Codex or the Node Codex wrapper followed by
// the exact `app-server` subcommand. Process ownership and identity are checked
// again immediately before every signal.

use std::collections::{HashMap, HashSet};
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::thread;
use std::time::Duration;

use sysinfo::{
    Pid, Process, ProcessRefreshKind, ProcessesToUpdate, Signal, System, Uid, UpdateKind,
    get_current_pid,
};

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
    }

    // Kill every target that exists at the deadline, including replacement PIDs.
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
    use std::time::Duration;

    use super::{
        CleanupTiming, CodexProcessDiscoveryIssue, CodexSignalStage, ProcessBackend,
        ProcessDiscovery, ProcessIdentity, SignalAttempt, TerminationKind, TrackedProcess,
        cleanup_with_backend, is_codex_app_server,
    };

    #[cfg(unix)]
    struct ChildGuard(std::process::Child);

    #[cfg(unix)]
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
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
        let reused = tracked(301, 2);
        let respawned = tracked(301, 3);
        let mut backend = FakeBackend::new(vec![
            Ok(vec![initial.clone()]),
            Ok(Vec::new()),
            Ok(vec![reused.clone()]),
            Ok(vec![respawned.clone()]),
        ]);

        let report = cleanup_with_backend(&mut backend, false, test_timing());

        assert_eq!(report.discovered_during_cleanup, vec![reused.display]);
        assert_eq!(
            report.cleanup.terminated,
            vec![(301, TerminationKind::Kill)]
        );
        assert_eq!(report.cleanup.respawned, vec![respawned.display]);
        assert_eq!(
            backend.signal_calls,
            vec![
                (initial.identity, CodexSignalStage::Term),
                (reused.identity, CodexSignalStage::Kill),
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
}
