//! Opaque archived Codex identity to the existing session/export/trash owners.
use super::*;
use ccr_codex::{CodexSessionExport, CodexSessionService, CodexSessionTrashService};
use ccr_core::core::LockManager;
use ccr_store::sessions::{AgentSessionAgentId, AgentSessionSourceKind};
use tokio::sync::watch;
mod delivery;
pub use delivery::{PreparedRestore, TrashItem};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Clone,
    Trash,
    Save,
    Restore,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    LocalOnly,
    Source,
    Unsupported,
    Conflict,
    Storage,
    Busy,
    Interrupted,
    Partial,
}
#[derive(Clone, Debug)]
pub struct Outcome {
    pub archive_id: String,
    pub action: Action,
    pub session_id: String,
    pub archive_updated: bool,
    pub codex_home: PathBuf,
}
#[derive(Clone, Debug, Default)]
pub struct State {
    pub busy: bool,
    pub result: Option<Result<Outcome, Failure>>,
}
pub struct Owner {
    state: Mutex<State>,
    updates: watch::Sender<Arc<State>>,
}
impl Default for Owner {
    fn default() -> Self {
        Self {
            state: Default::default(),
            updates: watch::channel(Arc::new(State::default())).0,
        }
    }
}
impl Owner {
    fn update(&self, apply: impl FnOnce(&mut State)) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        apply(&mut state);
        self.updates.send_replace(Arc::new(state.clone()));
    }
}
/// No source bytes in the confirmation or retained operation state.
pub struct Prepared {
    archive_id: String,
    path: PathBuf,
    codex_home: PathBuf,
    version: String,
}
struct Pending {
    owner: Arc<Owner>,
    done: bool,
}
impl Drop for Pending {
    fn drop(&mut self) {
        if !self.done {
            self.owner.update(|s| {
                s.busy = false;
                s.result = Some(Err(Failure::Interrupted));
            });
        }
    }
}
impl InsightsRequests {
    fn prepare_session(&self, archive_id: &str) -> Result<Arc<Prepared>, Failure> {
        if !self.codex_actions_allowed() {
            return Err(Failure::Unsupported);
        }
        let paths = self.paths.as_ref().ok_or(Failure::Storage)?;
        let pool = self.archive_pool(paths).map_err(|_| Failure::Storage)?;
        let registry = AgentSessionProviderRegistry::new(paths.provider_home.clone())
            .without_environment_overrides();
        let (source, _) = sessions::resolve_source_with_registry(&pool, &registry, archive_id)
            .map_err(|_| Failure::Source)?;
        if source.agent != AgentSessionAgentId::Codex
            || source.kind != AgentSessionSourceKind::File
            || source.variant != "codex-live"
        {
            return Err(Failure::Unsupported);
        }
        let path = source.physical_path;
        let codex_home = paths.provider_home.join(".codex");
        let version = CodexSessionService::new(codex_home.clone())
            .source_version(&path)
            .map_err(|_| Failure::Source)?;
        Ok(Arc::new(Prepared {
            archive_id: archive_id.into(),
            path,
            codex_home,
            version,
        }))
    }
    pub fn prepare_session_operation(
        &self,
        key: RequestKey,
        archive_id: String,
    ) -> Result<TaskSubscription<Arc<Prepared>, Failure>, AdmissionError> {
        let requests = self.clone();
        let local = matches!(key.environment, EnvironmentId::Local(_));
        self.services.executor.read(key, move |_, _| {
            if !local {
                return Err(Failure::LocalOnly);
            }
            requests.prepare_session(&archive_id)
        })
    }
    pub fn export_session(
        &self,
        key: RequestKey,
        archive_id: String,
    ) -> Result<TaskSubscription<Arc<CodexSessionExport>, Failure>, AdmissionError> {
        let requests = self.clone();
        let local = matches!(key.environment, EnvironmentId::Local(_));
        self.services.executor.read(key, move |_, _| {
            if !local {
                return Err(Failure::LocalOnly);
            }
            let prepared = requests.prepare_session(&archive_id)?;
            CodexSessionService::new(prepared.codex_home.clone())
                .export_session_markdown(&prepared.path, None)
                .map(Arc::new)
                .map_err(|_| Failure::Source)
        })
    }
    pub fn session_operation_snapshot(&self) -> Arc<State> {
        self.session_operations.updates.borrow().clone()
    }
    pub fn observe_session_operations(&self) -> watch::Receiver<Arc<State>> {
        self.session_operations.updates.subscribe()
    }
    pub fn start_session_operation(
        &self,
        key: RequestKey,
        prepared: Arc<Prepared>,
        action: Action,
    ) -> Result<(), Failure> {
        let requests = self.clone();
        self.start_session_work(key, move || {
            requests.run_session_operation(&prepared, action)
        })
    }
    fn start_session_work(
        &self,
        key: RequestKey,
        work: impl FnOnce() -> Result<Outcome, Failure> + Send + 'static,
    ) -> Result<(), Failure> {
        if !self.codex_actions_allowed() {
            return Err(Failure::Unsupported);
        }
        if !matches!(key.environment, EnvironmentId::Local(_)) {
            return Err(Failure::LocalOnly);
        }
        let owner = self.session_operations.clone();
        {
            let mut state = owner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.busy {
                return Err(Failure::Busy);
            }
            *state = State {
                busy: true,
                result: None,
            };
            owner.updates.send_replace(Arc::new(state.clone()));
        }
        let pending = Pending {
            owner: owner.clone(),
            done: false,
        };
        self.services
            .executor
            .job(key, move |_, _| {
                let mut pending = pending;
                let result = work();
                owner.update(|state| {
                    state.busy = false;
                    state.result = Some(result.clone());
                });
                pending.done = true;
                match result {
                    Ok(value) => Completion::Success(value),
                    Err(error) => Completion::Failed(error),
                }
            })
            .map(|_| ())
            .map_err(|_| Failure::Interrupted)
    }
    fn run_session_operation(
        &self,
        prepared: &Prepared,
        action: Action,
    ) -> Result<Outcome, Failure> {
        if !self.codex_actions_allowed() {
            return Err(Failure::Unsupported);
        }
        let current = self.prepare_session(&prepared.archive_id)?;
        if current.path != prepared.path
            || current.codex_home != prepared.codex_home
            || current.version != prepared.version
        {
            return Err(Failure::Conflict);
        }
        let paths = self.paths.as_ref().ok_or(Failure::Storage)?;
        let run = || -> Result<String, Failure> {
            let service = CodexSessionService::new(prepared.codex_home.clone());
            match action {
                Action::Clone => service
                    .clone_session_versioned(&prepared.path, &prepared.version)
                    .map_err(|_| Failure::Partial)?
                    .map(|s| s.session_id)
                    .ok_or(Failure::Conflict),
                Action::Trash => {
                    let summary = CodexSessionTrashService::with_codex_home(&prepared.codex_home)
                        .trash_session_versioned(&prepared.path, &prepared.version)
                        .map_err(|_| Failure::Partial)?
                        .ok_or(Failure::Conflict)?;
                    service
                        .invalidate_inventory_cache()
                        .map_err(|_| Failure::Partial)?;
                    summary
                        .trashed_sessions
                        .first()
                        .map(|s| s.session_id.clone())
                        .ok_or(Failure::Partial)
                }
                Action::Save | Action::Restore => Err(Failure::Unsupported),
            }
        };
        let session_id = if let Some(layout) = self.services.source.benchmark() {
            LockManager::with_scoped_directory(&layout.roots.locks, || Ok(run()))
                .map_err(|_| Failure::Storage)??
        } else if self.services.source.is_fixture() {
            let root = paths.archive_db.parent().ok_or(Failure::Storage)?;
            LockManager::with_scoped_directory(&root.join("locks"), || Ok(run()))
                .map_err(|_| Failure::Storage)??
        } else {
            run()?
        };
        let archive_updated = (|| {
            let pool = self.archive_pool(paths).map_err(|_| ())?;
            if action == Action::Trash {
                let conn = pool.get().map_err(|_| ())?;
                let path = prepared.path.to_string_lossy();
                database::repositories::usage_repo::mark_source_deleted_by_path(
                    &conn, "codex", &path,
                )
                .map_err(|_| ())?;
                database::repositories::usage_repo::mark_session_archive_deleted_by_path(
                    &conn, "codex", &path,
                )
                .map_err(|_| ())?;
            } else {
                let registry = AgentSessionProviderRegistry::new(paths.provider_home.clone())
                    .without_environment_overrides();
                sessions::refresh::refresh_archive_with_registry(&pool, &registry)
                    .map_err(|_| ())?;
            }
            Ok::<_, ()>(())
        })()
        .is_ok();
        Ok(Outcome {
            archive_id: prepared.archive_id.clone(),
            action,
            session_id,
            archive_updated,
            codex_home: prepared.codex_home.clone(),
        })
    }
}

#[cfg(test)]
pub(crate) mod tests;
