//! App-owned explicit tool operations. No probes or updates run while rendering.
use super::Services;
use ccr_app::{
    runtime::{Completion, EnvironmentId, RequestKey},
    tools::{self, CliVersion, bridge::SkillportAppStatus},
};
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Versions,
    DetectBridge,
    OpenBridge,
    UpdateCcr,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    LocalOnly,
    FixtureDisabled,
    Busy,
    Confirmation,
    Operation,
    Interrupted,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Idle,
    Queued,
    Running,
    Finished,
    Failed(Error),
}
impl Status {
    pub fn active(self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }
}
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub run: u64,
    pub action: Option<Action>,
    pub status: Status,
    pub versions: Vec<CliVersion>,
    pub bridge: Option<SkillportAppStatus>,
    pub update_output: Option<String>,
}
pub struct Owner {
    state: Mutex<Snapshot>,
    updates: watch::Sender<Arc<Snapshot>>,
    #[cfg(test)]
    synthetic: Mutex<Option<Arc<std::sync::Barrier>>>,
}
impl Default for Owner {
    fn default() -> Self {
        Self {
            state: Mutex::new(Snapshot::default()),
            updates: watch::channel(Arc::new(Snapshot::default())).0,
            #[cfg(test)]
            synthetic: Mutex::new(None),
        }
    }
}
impl Owner {
    fn update(&self, change: impl FnOnce(&mut Snapshot)) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        change(&mut state);
        self.updates.send_replace(Arc::new(state.clone()));
    }
}
struct Pending {
    owner: Arc<Owner>,
    done: bool,
}
impl Drop for Pending {
    fn drop(&mut self) {
        if !self.done {
            self.owner
                .update(|s| s.status = Status::Failed(Error::Interrupted));
        }
    }
}
#[derive(Clone)]
pub struct Requests(Services);
impl Requests {
    pub fn new(services: &Services) -> Self {
        Self(services.clone())
    }
    pub fn is_fixture(&self) -> bool {
        self.0.is_fixture()
    }
    pub fn snapshot(&self) -> Arc<Snapshot> {
        self.0.tools.updates.borrow().clone()
    }
    pub fn observe(&self) -> watch::Receiver<Arc<Snapshot>> {
        self.0.tools.updates.subscribe()
    }
    pub fn perform(&self, key: RequestKey, action: Action, confirmed: bool) -> Result<(), Error> {
        if !matches!(key.environment, EnvironmentId::Local(_)) {
            return Err(Error::LocalOnly);
        }
        #[cfg(test)]
        let synthetic = self
            .0
            .tools
            .synthetic
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        #[cfg(not(test))]
        let synthetic: Option<()> = None;
        if self.0.source.is_benchmark() || (self.is_fixture() && synthetic.is_none()) {
            return Err(Error::FixtureDisabled);
        }
        if action == Action::UpdateCcr && !confirmed {
            return Err(Error::Confirmation);
        }
        let owner = self.0.tools.clone();
        {
            let mut state = owner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.status.active() {
                return Err(Error::Busy);
            }
            state.run += 1;
            state.action = Some(action);
            state.status = Status::Queued;
            match action {
                Action::UpdateCcr => state.update_output = None,
                Action::Versions => state.versions.clear(),
                Action::DetectBridge => state.bridge = None,
                Action::OpenBridge => {}
            }
            owner.updates.send_replace(Arc::new(state.clone()));
        }
        let monitoring = self.0.monitoring.clone();
        let monitor_source = self.0.source.clone();
        let pending = Pending {
            owner: owner.clone(),
            done: false,
        };
        let subscription = self
            .0
            .executor
            .job(key, move |cancel, _| {
                let mut pending = pending;
                owner.update(|s| s.status = Status::Running);
                #[cfg(test)]
                if let Some(barrier) = synthetic {
                    barrier.wait();
                    owner.update(|s| {
                        s.update_output =
                            Some("Controlled fixture: no installed CLI changed".into());
                        s.status = Status::Finished;
                    });
                    pending.done = true;
                    return Completion::<(), Error>::Success(());
                }
                if cancel.is_cancelled() {
                    return Completion::Cancelled;
                }
                let result = run(action);
                let failed = match &result {
                    Ok(Output::Updated(report)) => !report.success,
                    Err(_) => true,
                    _ => false,
                };
                owner.update(|s| {
                    match result {
                        Ok(Output::Versions(versions)) => s.versions = versions,
                        Ok(Output::Bridge(bridge)) => s.bridge = Some(bridge),
                        Ok(Output::Opened) => {}
                        Ok(Output::Updated(report)) => {
                            s.update_output = Some(safe_output(&format!(
                                "{}\n{}",
                                report.stdout, report.stderr
                            )));
                            s.status = if report.success {
                                Status::Finished
                            } else {
                                Status::Failed(Error::Operation)
                            };
                            return;
                        }
                        Err(error) => {
                            s.status = Status::Failed(error);
                            return;
                        }
                    }
                    s.status = Status::Finished;
                });
                let event = match (action, failed) {
                    (Action::UpdateCcr, false) => "cli.update.completed",
                    (Action::UpdateCcr, true) => "cli.update.failed",
                    (_, false) => "tools.operation.completed",
                    (_, true) => "tools.operation.failed",
                };
                monitoring.terminal(&monitor_source, "tools", event, failed);
                pending.done = true;
                if failed {
                    Completion::Failed(Error::Operation)
                } else {
                    Completion::Success(())
                }
            })
            .map_err(|_| Error::Interrupted)?;
        drop(subscription); // The app retains the operation when its page detaches.
        Ok(())
    }
}
enum Output {
    Versions(Vec<CliVersion>),
    Bridge(SkillportAppStatus),
    Opened,
    Updated(tools::UpdateReport),
}
fn run(action: Action) -> Result<Output, Error> {
    match action {
        Action::DetectBridge => Ok(Output::Bridge(
            tokio::runtime::Handle::try_current()
                .map_err(|_| Error::Interrupted)?
                .block_on(tools::bridge::detect()),
        )),
        Action::OpenBridge => tokio::runtime::Handle::try_current()
            .map_err(|_| Error::Interrupted)?
            .block_on(tools::bridge::open())
            .map(|_| Output::Opened)
            .map_err(|_| Error::Operation),
        Action::Versions | Action::UpdateCcr => {
            let handle = tokio::runtime::Handle::try_current().map_err(|_| Error::Interrupted)?;
            handle.block_on(async {
                if action == Action::UpdateCcr {
                    let root = user_install_root()?;
                    return tools::update_ccr_for_root(&root)
                        .await
                        .map(Output::Updated)
                        .map_err(|_| Error::Operation);
                }
                let mut versions = Vec::new();
                for (tool, program) in tools::VERSION_TOOLS {
                    let mut version = tools::probe_version(tool, program, 3500).await;
                    version.version = version.version.map(|s| safe_output(&s));
                    versions.push(version);
                }
                Ok(Output::Versions(versions))
            })
        }
    }
}
fn user_install_root() -> Result<std::path::PathBuf, Error> {
    let home = std::env::var_os("USERPROFILE")
        .filter(|v| !v.is_empty())
        .or_else(|| std::env::var_os("HOME").filter(|v| !v.is_empty()))
        .map(std::path::PathBuf::from)
        .filter(|p| p.is_absolute())
        .ok_or(Error::Operation)?;
    let root = home.join(".cargo");
    let executable = std::env::current_exe().map_err(|_| Error::Operation)?;
    validate_install_root(&root, &executable)?;
    Ok(root)
}
fn validate_install_root(
    root: &std::path::Path,
    executable: &std::path::Path,
) -> Result<(), Error> {
    if !root.is_absolute() {
        return Err(Error::Operation);
    }
    // Cargo configuration and CARGO_INSTALL_ROOT cannot redirect this explicit root.
    // Refuse a Cargo-bin-hosted desktop so its adjacent verified sidecar stays intact.
    if let (Ok(bin), Some(parent)) = (root.join("bin").canonicalize(), executable.parent())
        && parent.canonicalize().is_ok_and(|parent| parent == bin)
    {
        return Err(Error::Operation);
    }
    Ok(())
}
fn safe_output(value: &str) -> String {
    let value = ccr_core::core::log_redact::redact_log_text(value);
    let mut end = value.len().min(8192);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end]
        .chars()
        .filter(|ch| !ch.is_control() || *ch == '\n' || *ch == '\t')
        .collect()
}
#[cfg(test)]
mod tests;
