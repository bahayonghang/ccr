//! Typed command workbench adapter. Fixture processes never invoke a real CLI.
use super::{ConfigSource, Services};
use ccr_app::{
    jobs::{
        commands::{Admission, CommandJobs, Error, Observation, Snapshot, Status, Ticket},
        policy::{CommandExecutionRequest, CommandInfo, command_catalog},
    },
    runtime::{AdmissionError, Completion, EnvironmentId, RequestKey, TaskSubscription},
};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Clone)]
pub struct CommandRequests(Services);
pub struct Started {
    pub snapshot: Snapshot,
    pub subscription: Option<TaskSubscription<Snapshot, Error>>,
}
struct QueuedTicket {
    owner: Arc<CommandJobs>,
    ticket: Mutex<Option<Ticket>>,
}
impl Drop for QueuedTicket {
    fn drop(&mut self) {
        if let Some(ticket) = self
            .ticket
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            self.owner.abandon(ticket);
        }
    }
}
impl CommandRequests {
    pub fn new(services: &Services) -> Self {
        Self(services.clone())
    }
    pub fn catalog(&self) -> Vec<CommandInfo> {
        command_catalog()
    }
    pub fn is_fixture(&self) -> bool {
        self.0.is_fixture()
    }
    pub fn status(&self, id: &str) -> Result<Snapshot, Error> {
        self.0.command_jobs.status(id)
    }
    pub fn cancel(&self, id: &str) -> Result<Snapshot, Error> {
        self.0.command_jobs.request_cancel(id)
    }
    pub fn observe(&self, id: &str) -> Result<(Snapshot, Observation), Error> {
        self.0.command_jobs.observe(id)
    }
    pub fn start(
        &self,
        key: RequestKey,
        command: String,
        args: Vec<String>,
    ) -> Result<Started, &'static str> {
        self.start_request(
            key,
            CommandExecutionRequest::background(command, Some(args), None),
        )
    }
    pub fn foreground(
        &self,
        key: RequestKey,
        command: String,
        args: Vec<String>,
    ) -> Result<Started, &'static str> {
        self.start_request(
            key,
            CommandExecutionRequest::foreground(command, Some(args), None),
        )
    }
    pub fn help(&self, key: RequestKey, command: &str) -> Result<Started, &'static str> {
        let request =
            CommandExecutionRequest::help(command).map_err(|_| Error::Invalid.message())?;
        self.start_request(key, request)
    }
    fn start_request(
        &self,
        key: RequestKey,
        request: CommandExecutionRequest,
    ) -> Result<Started, &'static str> {
        if !matches!(key.environment, EnvironmentId::Local(_)) {
            return Err(
                "Command execution for this environment is unavailable. / 此环境不可执行命令",
            );
        }
        let owner = self.0.command_jobs.clone();
        let Admission { snapshot, ticket } =
            owner.admit(key.clone(), request).map_err(Error::message)?;
        let Some(ticket) = ticket else {
            return Ok(Started {
                snapshot,
                subscription: None,
            });
        };
        // Retain the ticket outside the closure until the executor has accepted it.
        let ticket = Arc::new(QueuedTicket {
            owner: owner.clone(),
            ticket: Mutex::new(Some(ticket)),
        });
        let worker_ticket = ticket.clone();
        let source = self.0.source.clone();
        let id = snapshot.job_id.clone();
        let work_owner = owner.clone();
        let ui_state = super::ui_state::UiStateRequests::new(&self.0);
        let terminal_owner = owner.clone();
        let terminal = Arc::new(Mutex::new(None));
        let worker_terminal = terminal.clone();
        let finalize_ticket = ticket.clone();
        let monitoring = self.0.monitoring.clone();
        let monitor_source = self.0.source.clone();
        match self.0.executor.job_with_finalizer(
            key,
            move |cancel, progress| {
                let Some(ticket) = worker_ticket
                    .ticket
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take()
                else {
                    return Completion::Failed(Error::Missing);
                };
                work_owner.report_to(&id, progress);
                let result = match source {
                    ConfigSource::Local => {
                        work_owner.run_blocking(ticket, cancel, &sidecar_context())
                    }
                    ConfigSource::Fixture { profiles, .. } => {
                        let root = profiles
                            .parent()
                            .ok_or(Error::Unavailable)
                            .and_then(validate_fixture_root);
                        match (root, std::env::current_exe()) {
                            (Ok(root), Ok(executable)) => {
                                let mut command = ccr_app::jobs::process::tokio_command(executable);
                                command.arg("--native-command-fixture").arg(root);
                                work_owner.run_fixture_blocking(ticket, cancel, command)
                            }
                            _ => {
                                *worker_terminal
                                    .lock()
                                    .unwrap_or_else(std::sync::PoisonError::into_inner) =
                                    work_owner.abandon_snapshot(ticket);
                                Err(Error::Unavailable)
                            }
                        }
                    }
                };
                if let Ok(snapshot) = &result {
                    *worker_terminal
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) =
                        Some(snapshot.clone());
                }
                match result {
                    Ok(snapshot) if snapshot.status == Status::CleanupFailed => {
                        Completion::CleanupFailed(Error::Unavailable)
                    }
                    Ok(snapshot) => Completion::Success(snapshot),
                    Err(error) => Completion::Failed(error),
                }
            },
            move || {
                let queued = finalize_ticket
                    .ticket
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take();
                let terminal = if let Some(ticket) = queued {
                    terminal_owner.abandon_snapshot(ticket)
                } else {
                    terminal
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .take()
                };
                // One finalizer per admitted ticket, retained and joined even when
                // queued work is cancelled before its closure can run. Observers
                // and replayed admissions cannot duplicate this write.
                if let Some(terminal) = terminal {
                    let (event, failed) = match terminal.status {
                        Status::Success => ("command.completed", false),
                        Status::Cancelled => ("command.cancelled", false),
                        _ => ("command.failed", true),
                    };
                    monitoring.terminal(&monitor_source, "commands", event, failed);
                    if ui_state.record_command_terminal(&terminal).is_err() {
                        // Preserve process outcome; ambiguous writes are not replayed.
                        eprintln!("Command Recent history could not be persisted");
                    }
                }
            },
        ) {
            Ok(subscription) => Ok(Started {
                snapshot,
                subscription: Some(subscription),
            }),
            Err(error) => {
                if let Some(ticket) = ticket
                    .ticket
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take()
                {
                    owner.abandon(ticket);
                }
                Err(match error {
                    AdmissionError::Full => "Command queue is full. / 命令队列已满",
                    AdmissionError::Stopped => "Application is shutting down. / 应用正在退出",
                })
            }
        }
    }
}

fn validate_fixture_root(root: &Path) -> Result<PathBuf, Error> {
    let root = root.canonicalize().map_err(|_| Error::Unavailable)?;
    let allowed = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../outputs/desktop")
        .canonicalize()
        .map_err(|_| Error::Unavailable)?;
    if !root.starts_with(&allowed)
        || std::fs::read_to_string(root.join(".ccr-gpui-profile-fixture"))
            .map_err(|_| Error::Unavailable)?
            != "ccr-gpui-profiles-fixture-v1"
    {
        return Err(Error::Unavailable);
    }
    Ok(root)
}

fn sidecar_context() -> ccr_app::jobs::process::SidecarContext {
    ccr_app::jobs::process::SidecarContext {
        current_exe: std::env::current_exe().ok(),
        repository_root: Path::new(env!("CARGO_MANIFEST_DIR")).join(".."),
        allow_development: cfg!(debug_assertions),
        expected_sha256: option_env!("CCR_SIDECAR_SHA256").map(str::to_owned),
        expected_version: env!("CARGO_PKG_VERSION").into(),
    }
}

/// Closed synthetic child mode for real managed-process fixture acceptance.
/// Returns None for ordinary application arguments; never executes user input.
pub fn fixture_child_mode(args: impl IntoIterator<Item = OsString>) -> Option<i32> {
    let mut args = args.into_iter();
    if args.next()?.to_str() != Some("--native-command-fixture") {
        return None;
    }
    let Some(root) = args.next() else {
        return Some(2);
    };
    if args.next().is_some() || validate_fixture_root(Path::new(&root)).is_err() {
        return Some(2);
    }
    println!("Synthetic owned-process fixture: no CCR command or user configuration is accessed.");
    for index in 0..20 {
        println!("Synthetic process progress {index}/20");
        std::thread::sleep(Duration::from_millis(500));
    }
    eprintln!("Synthetic stderr fixture completed");
    Some(0)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use ccr_app::runtime::{Platform, RuntimeOwner, TaskEvent};

    #[gpui_kit::test]
    async fn recent_persists_owned_terminal_after_registry_eviction(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("config.toml"),
                delay: Duration::ZERO,
            },
        );
        let key = RequestKey {
            environment: EnvironmentId::Local("fixture".into()),
            platform: Platform::Claude,
            generation: 0,
            request_id: 1,
        };
        let admitted = services
            .command_jobs
            .admit(
                key.clone(),
                CommandExecutionRequest::background("status".into(), None, None),
            )
            .unwrap();
        let id = admitted.snapshot.job_id;
        let terminal = services
            .command_jobs
            .abandon_snapshot(admitted.ticket.unwrap())
            .unwrap();
        // Simulate registry pressure while the retained finalizer is waiting.
        // All new entries are terminal, so capacity eviction must remove `id`.
        for index in 0..ccr_app::jobs::commands::MAX_JOBS {
            let mut evict_key = key.clone();
            evict_key.request_id = index as u64 + 100;
            let admitted = services
                .command_jobs
                .admit(
                    evict_key,
                    CommandExecutionRequest::background("version".into(), None, None),
                )
                .unwrap();
            services.command_jobs.abandon(admitted.ticket.unwrap());
        }
        assert_eq!(services.command_jobs.status(&id), Err(Error::Missing));
        let history = super::super::ui_state::UiStateRequests::new(&services);
        let final_history = history.clone();
        let subscription = services
            .executor
            .job_with_finalizer(
                key.clone(),
                |_, _| Completion::<(), ()>::Success(()),
                move || final_history.record_command_terminal(&terminal).unwrap(),
            )
            .unwrap();
        drop(subscription);
        owner.shutdown();
        let reopened_owner = RuntimeOwner::new().unwrap();
        let reopened = super::super::ui_state::UiStateRequests::new(&Services::new(
            reopened_owner.handle(),
            services.source.clone(),
        ));
        let mut read = reopened.load(key).unwrap();
        let Some(TaskEvent::Terminal(value)) = read.next().await else {
            panic!("missing Recent snapshot")
        };
        let Completion::Success(snapshot) = value.as_ref() else {
            panic!("Recent storage failed")
        };
        assert_eq!(snapshot.history.len(), 1);
        assert_eq!(snapshot.history[0].command, "status");
        assert!(!snapshot.history[0].success);
        reopened_owner.shutdown();
    }

    #[gpui_kit::test]
    async fn executed_success_failure_and_cancel_persist_after_view_detaches(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("config.toml"),
                delay: Duration::ZERO,
            },
        );
        let history = super::super::ui_state::UiStateRequests::new(&services);
        for (index, mode, expected) in [
            (1, "success", Status::Success),
            (2, "failed", Status::Failed),
            (3, "wait", Status::Cancelled),
        ] {
            let key = RequestKey {
                environment: EnvironmentId::Local("fixture".into()),
                platform: Platform::Claude,
                generation: 0,
                request_id: index,
            };
            let admitted = services
                .command_jobs
                .admit(
                    key.clone(),
                    CommandExecutionRequest::background("version".into(), None, None),
                )
                .unwrap();
            let id = admitted.snapshot.job_id;
            let jobs = services.command_jobs.clone();
            let worker = jobs.clone();
            let finalizer = jobs.clone();
            let final_id = id.clone();
            let recent = history.clone();
            let ticket = admitted.ticket.unwrap();
            let mut command =
                ccr_app::jobs::process::tokio_command(std::env::current_exe().unwrap());
            command
                .args([
                    "--ignored",
                    "--exact",
                    "runtime::jobs::tests::recent_process_fixture",
                    "--nocapture",
                ])
                .env("CCR_GPUI_RECENT_PROCESS_FIXTURE", mode);
            let subscription = services
                .executor
                .job_with_finalizer(
                    key.clone(),
                    move |cancel, _| match worker.run_fixture_blocking(ticket, cancel, command) {
                        Ok(snapshot) => Completion::Success(snapshot),
                        Err(error) => Completion::Failed(error),
                    },
                    move || {
                        recent
                            .record_command_terminal(&finalizer.status(&final_id).unwrap())
                            .unwrap();
                    },
                )
                .unwrap();
            if mode == "wait" {
                let deadline = std::time::Instant::now() + Duration::from_secs(10);
                while !jobs
                    .status(&id)
                    .unwrap()
                    .stdout_lines
                    .iter()
                    .any(|line| line == "Recent fixture running")
                {
                    assert!(std::time::Instant::now() < deadline);
                    cx.background_executor
                        .timer(Duration::from_millis(10))
                        .await;
                }
                jobs.request_cancel(&id).unwrap();
            }
            // No visible observer remains to perform persistence.
            let terminal = subscription;
            drop(terminal);
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            while services.executor.active_tasks() != 0 {
                assert!(std::time::Instant::now() < deadline);
                cx.background_executor
                    .timer(Duration::from_millis(10))
                    .await;
            }
            assert_eq!(jobs.status(&id).unwrap().status, expected);
            let mut read = history.load(key).unwrap();
            let Some(TaskEvent::Terminal(result)) = read.next().await else {
                panic!("missing Recent snapshot")
            };
            let Completion::Success(snapshot) = result.as_ref() else {
                panic!("Recent read failed")
            };
            assert_eq!(snapshot.history.len(), index as usize);
            assert_eq!(snapshot.history[0].success, expected == Status::Success);
        }
        owner.shutdown();
    }

    #[test]
    #[ignore = "closed subprocess for executed_success_failure_and_cancel_persist_after_view_detaches"]
    fn recent_process_fixture() {
        match std::env::var("CCR_GPUI_RECENT_PROCESS_FIXTURE")
            .unwrap()
            .as_str()
        {
            "success" => (),
            "failed" => panic!("Synthetic process failure"),
            "wait" => {
                println!("Recent fixture running");
                std::thread::sleep(Duration::from_secs(15));
            }
            _ => panic!("unsupported fixture mode"),
        }
    }

    #[gpui_kit::test]
    async fn detached_terminal_records_recent_once_without_observer_or_global_database(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap();
        let source = ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("config.toml"),
            delay: Duration::ZERO,
        };
        let owner = RuntimeOwner::new().unwrap();
        let services = Services::new(owner.handle(), source.clone());
        let requests = CommandRequests::new(&services);
        let key = RequestKey {
            environment: EnvironmentId::Local("fixture".into()),
            platform: Platform::Claude,
            generation: 1,
            request_id: 1,
        };
        // An unowned fixture cannot launch anything; its unavailable terminal is
        // still a real command outcome, independent of page observation.
        let started = requests
            .start(key.clone(), "status".into(), vec![])
            .unwrap();
        let id = started.snapshot.job_id;
        drop(started.subscription);
        owner.shutdown(); // joins the retained worker and its history write
        assert_eq!(requests.status(&id).unwrap().status, Status::Unavailable);
        let replay = requests
            .start(key.clone(), "status".into(), vec![])
            .unwrap();
        assert_eq!(replay.snapshot.job_id, id);
        assert!(replay.subscription.is_none());
        let (snapshot, _) = requests.observe(&id).unwrap();
        assert_eq!(snapshot.status, Status::Unavailable);
        let reopened_owner = RuntimeOwner::new().unwrap();
        let reopened = super::super::ui_state::UiStateRequests::new(&Services::new(
            reopened_owner.handle(),
            source,
        ));
        let mut read = reopened.load(key).unwrap();
        let Some(TaskEvent::Terminal(value)) = read.next().await else {
            panic!("missing Recent snapshot")
        };
        let Completion::Success(snapshot) = value.as_ref() else {
            panic!("Recent storage failed")
        };
        assert_eq!(snapshot.history.len(), 1);
        assert_eq!(snapshot.history[0].command, "status");
        assert!(!snapshot.history[0].success);
        assert!(ccr_db::database::get_pool().is_none());
        reopened_owner.shutdown();
    }

    #[test]
    fn fixture_child_rejects_nonowned_roots_without_spawning_or_writing() {
        let temp = tempfile::tempdir().unwrap();
        assert!(validate_fixture_root(temp.path()).is_err());
        assert!(fixture_child_mode([OsString::from("--probe")]).is_none());
        assert_eq!(
            fixture_child_mode([
                OsString::from("--native-command-fixture"),
                temp.path().as_os_str().to_owned()
            ]),
            Some(2)
        );
        assert!(temp.path().read_dir().unwrap().next().is_none());
    }
}
