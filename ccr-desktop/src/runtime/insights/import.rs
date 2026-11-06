//! App-owned llmusage import, using the same installed-CLI executor as Tauri.
#[cfg(test)]
use super::InsightsPaths;
use super::InsightsRequests;
use crate::runtime::ConfigSource;
use ccr_app::{
    runtime::{CancellationHandle, Completion, EnvironmentId, RequestKey},
    usage::import::{
        AppPaths, JobEvent, LlmusageCli, SourceKind, SyncCommandOptions,
        error::LlmusageAdapterError, run_sync_stream,
    },
};
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    LocalOnly,
    Unconfigured,
    FixtureDisabled,
    SourceRoot,
    Busy,
    MissingCli,
    Process,
    TimedOut,
    CleanupFailed,
    Interrupted,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Idle,
    Queued,
    Running,
    CancelRequested,
    Finished,
    Cancelled,
    Failed(Failure),
}
impl Status {
    pub fn active(self) -> bool {
        matches!(self, Self::Queued | Self::Running | Self::CancelRequested)
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub enum ToolUpdate {
    #[default]
    NotStarted,
    Running,
    Finished {
        files: usize,
        calls: usize,
        errors: usize,
    },
    Failed,
}
/// Captured import scope, independent from the report's display filters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    pub source: Option<SourceKind>,
    pub recent_days: u32,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            source: None,
            recent_days: 30,
        }
    }
}
impl Options {
    fn includes_claude(self) -> bool {
        self.source
            .is_none_or(|source| source == SourceKind::Claude)
    }
    fn command(self, paths: &super::InsightsPaths) -> SyncCommandOptions {
        // An explicit missing activation map makes the installed CLI fail strictly.
        let provider_map = paths
            .archive_db
            .parent()
            .and_then(std::path::Path::parent)
            .map(ccr_config::managers::provider_activation::activation_log_path)
            .filter(|path| path.is_file());
        SyncCommandOptions {
            source: self.source,
            recent_days: Some(self.recent_days.max(1)),
            rebuild: false,
            provider_map,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub run: u64,
    pub options: Options,
    pub status: Status,
    pub environment: Option<EnvironmentId>,
    pub files_scanned: u64,
    pub records_imported: u64,
    pub total_seen: Option<usize>,
    pub source_errors: usize,
    pub tools: ToolUpdate,
}
struct State {
    snapshot: Snapshot,
    cancel: Option<CancellationHandle>,
    progress: std::collections::BTreeMap<String, (u64, u64)>,
    published: std::time::Instant,
}
pub struct ImportOwner {
    state: Mutex<State>,
    updates: watch::Sender<Arc<Snapshot>>,
    #[cfg(test)]
    fixture: Mutex<Option<bool>>,
}
impl Default for ImportOwner {
    fn default() -> Self {
        let snapshot = Snapshot::default();
        Self {
            updates: watch::channel(Arc::new(snapshot.clone())).0,
            state: Mutex::new(State {
                snapshot,
                cancel: None,
                progress: Default::default(),
                published: std::time::Instant::now(),
            }),
            #[cfg(test)]
            fixture: Mutex::new(None),
        }
    }
}
impl ImportOwner {
    fn update(&self, apply: impl FnOnce(&mut State)) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        apply(&mut state);
        self.updates.send_replace(Arc::new(state.snapshot.clone()));
    }
    fn event(&self, event: JobEvent) {
        let progress_event = matches!(event, JobEvent::Progress { .. });
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let queued = state.snapshot.status == Status::Queued;
        if state.snapshot.status != Status::CancelRequested {
            state.snapshot.status = Status::Running;
        }
        match event {
            JobEvent::Progress {
                source,
                files_scanned,
                records_imported,
                ..
            } => {
                state
                    .progress
                    .insert(source.as_str().into(), (files_scanned, records_imported));
            }
            JobEvent::SourceFinished { source, stats } => {
                state.progress.insert(
                    source.as_str().into(),
                    (stats.files_processed as u64, stats.events_inserted as u64),
                );
                if !stats.absent && stats.last_error.is_some() {
                    state.snapshot.source_errors += 1;
                }
            }
            _ => {}
        }
        state.snapshot.files_scanned = state.progress.values().map(|v| v.0).sum();
        state.snapshot.records_imported = state.progress.values().map(|v| v.1).sum();
        if queued
            || !progress_event
            || state.published.elapsed() >= ccr_app::runtime::PROGRESS_INTERVAL
        {
            state.published = std::time::Instant::now();
            self.updates.send_replace(Arc::new(state.snapshot.clone()));
        }
    }
}
struct Pending {
    owner: Arc<ImportOwner>,
    done: bool,
}
impl Drop for Pending {
    fn drop(&mut self) {
        if !self.done {
            self.owner.update(|state| {
                state.snapshot.status = if state.snapshot.status == Status::CancelRequested {
                    Status::Cancelled
                } else {
                    Status::Failed(Failure::Interrupted)
                };
                state.cancel = None;
            });
        }
    }
}
impl InsightsRequests {
    pub fn import_snapshot(&self) -> Arc<Snapshot> {
        self.services.usage_import.updates.borrow().clone()
    }
    pub fn observe_import(&self) -> watch::Receiver<Arc<Snapshot>> {
        self.services.usage_import.updates.subscribe()
    }
    #[cfg(test)]
    pub(crate) fn fixture_import(&self, wait: bool) {
        *self
            .services
            .usage_import
            .fixture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(wait);
    }
    pub fn cancel_import(&self) {
        self.services.usage_import.update(|state| {
            if state.snapshot.status.active() {
                state.snapshot.status = Status::CancelRequested;
                if let Some(cancel) = &state.cancel {
                    cancel.request_cancel();
                }
            }
        });
    }
    pub fn start_import(&self, key: RequestKey) -> Result<(), Failure> {
        self.start_import_with_options(key, Options::default())
    }
    pub fn start_import_with_options(
        &self,
        key: RequestKey,
        mut options: Options,
    ) -> Result<(), Failure> {
        options.recent_days = options.recent_days.max(1);
        if !matches!(key.environment, EnvironmentId::Local(_)) {
            return Err(Failure::LocalOnly);
        }
        let paths = self.paths.clone().ok_or(Failure::Unconfigured)?;
        if !paths.usage_root.is_absolute() || !paths.provider_home.is_absolute() {
            return Err(Failure::Unconfigured);
        }
        let fixture = matches!(self.services.source, ConfigSource::Fixture { .. });
        #[cfg(test)]
        let fixture_plan = *self
            .services
            .usage_import
            .fixture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        #[cfg(not(test))]
        let fixture_plan: Option<bool> = None;
        if fixture && fixture_plan.is_none() {
            return Err(Failure::FixtureDisabled);
        }
        if !fixture
            && super::local_insights_paths()
                .is_none_or(|local| local.provider_home != paths.provider_home)
        {
            return Err(Failure::SourceRoot);
        }
        #[cfg(test)]
        if fixture {
            validate_fixture(&self.services.source, &paths)?;
        }
        let owner = self.services.usage_import.clone();
        {
            let mut state = owner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.snapshot.status.active() {
                return Err(Failure::Busy);
            }
            state.snapshot = Snapshot {
                run: state.snapshot.run + 1,
                options,
                status: Status::Queued,
                environment: Some(key.environment.clone()),
                ..Default::default()
            };
            state.cancel = None;
            state.progress.clear();
            owner.updates.send_replace(Arc::new(state.snapshot.clone()));
        }
        let pending = Pending {
            owner: owner.clone(),
            done: false,
        };
        let worker_owner = owner.clone();
        let requests = self.clone();
        let monitoring = self.services.monitoring.clone();
        let monitor_source = self.services.source.clone();
        let subscription = self
            .services
            .executor
            .job(key, move |cancel, _| {
                let mut pending = pending;
                let mut result = tokio::runtime::Handle::try_current()
                    .map_err(|_| LlmusageAdapterError::Cli("runtime unavailable".into()))
                    .and_then(|handle| {
                        handle.block_on(async {
                            let observe = |event| {
                                worker_owner.event(event);
                                std::future::ready(Ok(()))
                            };
                            #[cfg(test)]
                            if let Some(wait) = fixture_plan {
                                return ccr_app::usage::import::cli::run_fixture_sync_stream(
                                    fixture_command(&paths, wait),
                                    cancel.clone(),
                                    observe,
                                )
                                .await;
                            }
                            let cli = LlmusageCli::new(AppPaths::from_root(&paths.usage_root));
                            run_sync_stream(&cli, options.command(&paths), cancel.clone(), observe)
                                .await
                        })
                    });
                if let Ok(summary) = &result {
                    worker_owner.update(|state| {
                        state.snapshot.total_seen = Some(summary.total_seen);
                        state.snapshot.records_imported = summary.total_inserted as u64;
                        state.snapshot.tools = if options.includes_claude() {
                            ToolUpdate::Running
                        } else {
                            ToolUpdate::NotStarted
                        };
                    });
                    let tools =
                        if options.includes_claude() {
                            Some(requests.ingest_observer_tools(&paths.provider_home, || {
                                cancel.is_cancelled()
                            }))
                        } else {
                            None
                        };
                    let tail_cancelled =
                        tools.as_ref().is_some_and(Result::is_err) && cancel.is_cancelled();
                    worker_owner.update(|state| {
                        state.snapshot.tools = match tools {
                            Some(Ok(report)) => ToolUpdate::Finished {
                                files: report.files_scanned,
                                calls: report.calls_inserted,
                                errors: report.errors.len(),
                            },
                            Some(Err(_)) => ToolUpdate::Failed,
                            None => ToolUpdate::NotStarted,
                        };
                    });
                    if tail_cancelled {
                        result = Err(LlmusageAdapterError::Cancelled);
                    }
                }
                worker_owner.update(|state| {
                    state.cancel = None;
                    state.snapshot.status = match &result {
                        Ok(summary) => {
                            state.snapshot.total_seen = Some(summary.total_seen);
                            state.snapshot.records_imported = summary.total_inserted as u64;
                            Status::Finished
                        }
                        Err(LlmusageAdapterError::Cancelled) => Status::Cancelled,
                        Err(LlmusageAdapterError::TimedOut) => Status::Failed(Failure::TimedOut),
                        Err(LlmusageAdapterError::CleanupFailed(_)) => {
                            Status::Failed(Failure::CleanupFailed)
                        }
                        Err(LlmusageAdapterError::CliMissing) => {
                            Status::Failed(Failure::MissingCli)
                        }
                        Err(_) => Status::Failed(Failure::Process),
                    };
                });
                let (event, failed) = match &result {
                    Ok(_) => ("usage.import.completed", false),
                    Err(LlmusageAdapterError::Cancelled) => ("usage.import.cancelled", false),
                    Err(_) => ("usage.import.failed", true),
                };
                monitoring.terminal(&monitor_source, "usage", event, failed);
                pending.done = true;
                match result {
                    Ok(_) => Completion::Success(()),
                    Err(LlmusageAdapterError::Cancelled) => Completion::Cancelled,
                    Err(LlmusageAdapterError::CleanupFailed(_)) => {
                        Completion::CleanupFailed(Failure::CleanupFailed)
                    }
                    Err(_) => Completion::Failed(Failure::Process),
                }
            })
            .map_err(|_| Failure::Interrupted)?;
        let cancel = subscription.cancellation_handle();
        owner.update(|state| {
            if state.snapshot.status.active() {
                if state.snapshot.status == Status::CancelRequested {
                    cancel.request_cancel();
                }
                state.cancel = Some(cancel);
            }
        });
        // Runtime job ownership survives receiver/view detachment.
        drop(subscription);
        Ok(())
    }
}

#[cfg(test)]
fn validate_fixture(source: &ConfigSource, paths: &InsightsPaths) -> Result<(), Failure> {
    let ConfigSource::Fixture { profiles, .. } = source else {
        return Err(Failure::FixtureDisabled);
    };
    let root = profiles
        .parent()
        .and_then(|p| p.canonicalize().ok())
        .ok_or(Failure::Unconfigured)?;
    for path in [&paths.usage_root, &paths.provider_home] {
        if !path
            .canonicalize()
            .map_err(|_| Failure::Unconfigured)?
            .starts_with(&root)
        {
            return Err(Failure::SourceRoot);
        }
    }
    Ok(())
}
#[cfg(test)]
fn fixture_command(paths: &InsightsPaths, wait: bool) -> tokio::process::Command {
    let started = r#"{"event":"progress","source":"codex","files_scanned":1,"records_imported":1,"current_file":null}"#;
    let finished =
        r#"{"event":"finished","summary":{"sources":1,"total_seen":1,"total_inserted":1}}"#;
    #[cfg(windows)]
    let mut command = {
        let mut command = ccr_app::jobs::process::tokio_command("powershell.exe");
        let ending = if wait {
            "Start-Sleep -Seconds 30".into()
        } else {
            format!("[Console]::Out.WriteLine('{finished}')")
        };
        command.args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!("[Console]::Out.WriteLine('{started}'); [Console]::Out.Flush(); {ending}"),
        ]);
        command
    };
    #[cfg(not(windows))]
    let mut command = {
        let mut command = ccr_app::jobs::process::tokio_command("sh");
        let ending = if wait {
            "sleep 30".into()
        } else {
            format!("printf '%s\n' '{finished}'")
        };
        command.args(["-c", &format!("printf '%s\n' '{started}'; {ending}")]);
        command
    };
    command.current_dir(&paths.usage_root);
    command
}
#[cfg(test)]
mod tests;
