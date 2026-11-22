//! App-owned llmusage import, using the same installed-CLI executor as Tauri.
#[cfg(test)]
use super::InsightsPaths;
use super::InsightsRequests;
#[cfg(test)]
use crate::runtime::ConfigSource;
use ccr_app::environments::EnvironmentPolicy;
use ccr_app::{
    runtime::{CancellationHandle, Completion, EnvironmentId, RequestKey},
    usage::import::{
        AppPaths, JobEvent, LlmusageCli, SourceKind, SyncCommandOptions,
        error::LlmusageAdapterError, run_sync_stream,
    },
};
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

mod fixture;
pub use fixture::Scenario;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    LocalOnly,
    SshUnavailable,
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
    pub fn cancel_import_run(&self, environment: &EnvironmentId, run: u64) {
        let owner = &self.services.usage_import;
        let mut state = owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.snapshot.run != run
            || state.snapshot.environment.as_ref() != Some(environment)
            || !matches!(state.snapshot.status, Status::Queued | Status::Running)
        {
            return;
        }
        state.snapshot.status = Status::CancelRequested;
        if let Some(cancel) = &state.cancel {
            cancel.request_cancel();
        }
        owner.updates.send_replace(Arc::new(state.snapshot.clone()));
    }

    #[cfg(test)]
    pub fn cancel_import(&self) {
        let snapshot = self.import_snapshot();
        if let Some(environment) = snapshot.environment.as_ref() {
            self.cancel_import_run(environment, snapshot.run);
        }
    }

    #[cfg(test)]
    fn test_fixture_plan(&self) -> Option<bool> {
        *self
            .services
            .usage_import
            .fixture
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn local_import_paths(&self) -> Result<super::InsightsPaths, Failure> {
        let paths = self
            .paths
            .as_ref()
            .filter(|paths| paths.usage_root.is_absolute() && paths.provider_home.is_absolute())
            .ok_or(Failure::Unconfigured)?;
        if self.services.source.is_fixture() {
            if self.services.import_fixture.is_none() {
                #[cfg(test)]
                if self.test_fixture_plan().is_some() {
                    return Ok(paths.clone());
                }
                return Err(Failure::FixtureDisabled);
            }
        } else if super::local_insights_paths()
            .is_none_or(|local| local.provider_home != paths.provider_home)
        {
            return Err(Failure::SourceRoot);
        }
        Ok(paths.clone())
    }

    pub fn check_import_target(&self, environment: &EnvironmentId) -> Result<(), Failure> {
        if self.services.source.is_benchmark() {
            return Err(Failure::FixtureDisabled);
        }
        match environment {
            EnvironmentId::Local(_) => self.local_import_paths().map(|_| ()),
            EnvironmentId::Ssh(_) => self
                .services
                .remote_configuration_io(environment)
                .map(|_| ())
                .map_err(|_| Failure::SshUnavailable),
            _ => Err(Failure::LocalOnly),
        }
    }

    pub fn import_is_fixture(&self) -> bool {
        self.services.source.is_fixture()
    }

    pub fn import_is_synthetic(&self) -> bool {
        self.services.import_fixture.is_some()
    }

    pub fn start_import(&self, key: RequestKey) -> Result<(), Failure> {
        self.start_import_with_options(key, Options::default())
    }
    pub fn start_import_with_options(
        &self,
        key: RequestKey,
        mut options: Options,
    ) -> Result<(), Failure> {
        if let Some(platform) = self.services.platform_scope {
            let source = SourceKind::parse_id(platform.short_name()).ok_or(Failure::SourceRoot)?;
            if options.source != Some(source) {
                return Err(Failure::SourceRoot);
            }
        }
        if self.services.source.is_benchmark() {
            return Err(Failure::FixtureDisabled);
        }
        // 清理期间 authority 可已撤销，导入准入仍属于旧 runner；先返回 Busy。
        // 准备路径后仍在同一 owner 锁内二次检查，防止并发 start 都取得准入。
        if self
            .services
            .usage_import
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .snapshot
            .status
            .active()
        {
            return Err(Failure::Busy);
        }
        options.recent_days = options.recent_days.max(1);
        let local = EnvironmentPolicy::LocalOnly.admit(&key.environment).is_ok();
        let remote = match &key.environment {
            EnvironmentId::Local(_) => None,
            EnvironmentId::Ssh(_) => Some(
                self.services
                    .remote_configuration_io(&key.environment)
                    .map_err(|_| Failure::SshUnavailable)?,
            ),
            _ => return Err(Failure::LocalOnly),
        };
        let paths = if local {
            self.local_import_paths()?
        } else {
            let root = self.remote_cache().map_err(|_| Failure::Unconfigured)?;
            super::InsightsPaths {
                usage_root: root.clone(),
                provider_home: root.clone(),
                archive_db: root.join("unavailable-archive.db"),
            }
        };
        let native_fixture = self.services.import_fixture;
        #[cfg(test)]
        let fixture_plan = self.test_fixture_plan();
        #[cfg(test)]
        if local && native_fixture.is_none() && fixture_plan.is_some() {
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
                            if let Some(io) = remote.as_ref() {
                                let mut command = SyncCommandOptions {
                                    source: options.source,
                                    recent_days: Some(options.recent_days),
                                    ..Default::default()
                                };
                                if io
                                    .metadata(
                                        ".ccr/analytics/provider_activation.jsonl",
                                        cancel.clone(),
                                    )
                                    .await
                                    .map_err(|_| {
                                        LlmusageAdapterError::Cli("remote_provider_map".into())
                                    })?
                                    .is_some()
                                    && let ccr_app::environments::io::EnvironmentPath::Ssh(path) =
                                        io.roots()
                                            .relative(".ccr/analytics/provider_activation.jsonl")
                                            .map_err(|_| {
                                                LlmusageAdapterError::Cli(
                                                    "remote_provider_map".into(),
                                                )
                                            })?
                                {
                                    command.provider_map = Some(path.as_str().into());
                                }
                                let progress_owner = worker_owner.clone();
                                let summary = ccr_app::usage::import::remote::run_sync(
                                    io,
                                    command,
                                    cancel.clone(),
                                    Arc::new(move |event| progress_owner.event(event)),
                                )
                                .await?;
                                let _snapshot = ccr_app::usage::import::remote::Snapshot::capture(
                                    io,
                                    &requests
                                        .remote_cache()
                                        .map_err(|_| {
                                            LlmusageAdapterError::Cli("remote_cache".into())
                                        })?
                                        .join("usage"),
                                    cancel.clone(),
                                )
                                .await?;
                                return Ok(summary);
                            }
                            if let Some(scenario) = native_fixture {
                                return scenario.run(options, &worker_owner, &cancel).await;
                            }
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
                    let update_tools =
                        local && options.includes_claude() && native_fixture.is_none();
                    worker_owner.update(|state| {
                        state.snapshot.total_seen = Some(summary.total_seen);
                        state.snapshot.records_imported = summary.total_inserted as u64;
                        state.snapshot.tools = if update_tools {
                            ToolUpdate::Running
                        } else {
                            ToolUpdate::NotStarted
                        };
                    });
                    let tools =
                        if update_tools {
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
                if native_fixture.is_none() {
                    let (event, failed) = match &result {
                        Ok(_) => ("usage.import.completed", false),
                        Err(LlmusageAdapterError::Cancelled) => ("usage.import.cancelled", false),
                        Err(_) => ("usage.import.failed", true),
                    };
                    monitoring.terminal(&monitor_source, "usage", event, failed);
                }
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
