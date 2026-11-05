//! Typed command workbench adapter. Fixture processes never invoke a real CLI.
use super::{ConfigSource, Services};
use ccr_app::{
    jobs::{
        commands::{Admission, CommandJobs, Error, Snapshot, Status, Ticket},
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
    pub fn start(
        &self,
        key: RequestKey,
        command: String,
        args: Vec<String>,
    ) -> Result<Started, &'static str> {
        if !matches!(key.environment, EnvironmentId::Local(_)) {
            return Err(
                "Command execution for this environment is unavailable. / 此环境不可执行命令",
            );
        }
        let owner = self.0.command_jobs.clone();
        let Admission { snapshot, ticket } = owner
            .admit(
                key.clone(),
                CommandExecutionRequest::background(command, Some(args), None),
            )
            .map_err(Error::message)?;
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
        match self.0.executor.job(key, move |cancel, progress| {
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
                ConfigSource::Local => work_owner.run_blocking(ticket, cancel, &sidecar_context()),
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
                            work_owner.abandon(ticket);
                            Err(Error::Unavailable)
                        }
                    }
                }
            };
            match result {
                Ok(snapshot) if snapshot.status == Status::CleanupFailed => {
                    Completion::CleanupFailed(Error::Unavailable)
                }
                Ok(snapshot) => Completion::Success(snapshot),
                Err(error) => Completion::Failed(error),
            }
        }) {
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
        std::thread::sleep(Duration::from_millis(100));
    }
    eprintln!("Synthetic stderr fixture completed");
    Some(0)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
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
