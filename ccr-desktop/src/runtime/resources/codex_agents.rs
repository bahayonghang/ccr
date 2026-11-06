//! Captured-root Codex agent adapter. Writes and source jobs survive view detachment.
use super::{ConfigSource, Services, roots};
use ccr_app::{
    codex_agents::{
        AgentStore, CodexAgentContextRequest, CodexAgentRecord,
        sources::{
            CodexAgentSourceCatalogResponse, CodexAgentSourceInstallActionRequest,
            CodexAgentSourceInstallRequest, CodexAgentSourceRecord, CodexAgentSourceRequest,
            CodexAgentSourceSyncRequest, CodexAgentSourcesResponse, SourceStore,
        },
    },
    platform_settings::Snapshot,
    runtime::{AdmissionError, Completion, EnvironmentId, Platform, RequestKey, TaskSubscription},
};
use ccr_core::core::log_redact::redact_log_text;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::watch;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Scope {
    #[default]
    Global,
    Project(PathBuf),
}
impl Scope {
    fn request(&self) -> Option<CodexAgentContextRequest> {
        match self {
            Self::Global => None,
            Self::Project(path) => Some(CodexAgentContextRequest {
                mode: Some("project".into()),
                project_root: Some(path.to_string_lossy().into_owned()),
            }),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Unsupported,
    Invalid,
    Storage,
    Conflict,
    Busy,
    Interrupted,
    FixtureNetwork,
    Source,
}
fn error(value: String) -> Error {
    if value.contains("changed externally")
        || value.contains("本地文件")
        || value.contains("已存在")
        || value.contains("already exists")
    {
        Error::Conflict
    } else if value.contains("disabled for fixture") {
        Error::FixtureNetwork
    } else if value.contains("GitHub") || value.contains("扫描") || value.contains("cached catalog")
    {
        Error::Source
    } else if value.contains("Invalid") || value.contains("required") || value.contains("requires")
    {
        Error::Invalid
    } else {
        Error::Storage
    }
}
fn store(source: &ConfigSource, scope: &Scope) -> Result<AgentStore, Error> {
    let roots = roots(source).map_err(|_| Error::Storage)?;
    if let Scope::Project(path) = scope {
        if !path.is_absolute() {
            return Err(Error::Invalid);
        }
        if matches!(source, ConfigSource::Fixture { .. }) {
            let project = path.canonicalize().map_err(|_| Error::Invalid)?;
            let root = roots.home.canonicalize().map_err(|_| Error::Invalid)?;
            if !project.starts_with(root) {
                return Err(Error::Invalid);
            }
        }
    }
    Ok(AgentStore {
        global_agents: roots.codex_runtime.join("agents"),
        sources_root: roots.ccr.join("platforms/codex/agent-sources"),
        locks: roots.locks,
        backups: roots.ccr.join("backups/codex-agents"),
    })
}
fn sources(source: &ConfigSource) -> Result<SourceStore, Error> {
    let agents = store(source, &Scope::Global)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|_| Error::Storage)?;
    Ok(if matches!(source, ConfigSource::Fixture { .. }) {
        SourceStore::cached(agents, client)
    } else {
        SourceStore::new(agents, client)
    })
}
#[derive(Clone)]
pub struct AgentRow {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub malformed: bool,
}
pub struct Session {
    environment: EnvironmentId,
    pub scope: Scope,
    pub name: String,
    pub snapshot: Snapshot,
}
pub enum Read {
    List(Scope),
    Open(Scope, String),
    Sources,
    Catalog(String),
}
pub enum ReadResult {
    Agents(Vec<AgentRow>),
    Opened(Arc<Session>),
    Sources(Vec<CodexAgentSourceRecord>),
    Catalog(Box<CodexAgentSourceCatalogResponse>),
}
pub enum SourceAction {
    Add(String),
    Remove(String),
    Refresh(String),
    Install(CodexAgentSourceInstallRequest),
    Sync(CodexAgentSourceSyncRequest),
    AcceptLocal(String),
    Untrack(String),
}
pub enum Mutation {
    Add {
        scope: Scope,
        name: String,
        content: String,
    },
    Save {
        session: Arc<Session>,
        content: String,
    },
    Delete(Arc<Session>),
    Rename {
        session: Arc<Session>,
        name: String,
    },
    Copy {
        session: Arc<Session>,
        target: Scope,
        name: String,
    },
    Source(SourceAction),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Saved,
    Deleted,
    Renamed,
    Copied,
    SourceUpdated,
}
#[derive(Clone, Default)]
pub struct OperationState {
    pub busy: bool,
    pub environment: Option<EnvironmentId>,
    pub result: Option<Result<Outcome, Error>>,
}
pub struct AgentOperations {
    state: Mutex<OperationState>,
    updates: watch::Sender<Arc<OperationState>>,
}
impl Default for AgentOperations {
    fn default() -> Self {
        Self {
            state: Mutex::new(OperationState::default()),
            updates: watch::channel(Arc::new(OperationState::default())).0,
        }
    }
}
impl AgentOperations {
    fn finish(&self, result: Result<Outcome, Error>) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.busy = false;
        state.result = Some(result);
        self.updates.send_replace(Arc::new(state.clone()));
    }
}
struct Pending {
    owner: Arc<AgentOperations>,
    finished: bool,
}
impl Drop for Pending {
    fn drop(&mut self) {
        if !self.finished {
            self.owner.finish(Err(Error::Interrupted));
        }
    }
}
fn local(key: &RequestKey) -> Result<(), Error> {
    if matches!(key.environment, EnvironmentId::Local(_)) && key.platform == Platform::Codex {
        Ok(())
    } else {
        Err(Error::Unsupported)
    }
}
async fn read(source: &ConfigSource, key: &RequestKey, request: Read) -> Result<ReadResult, Error> {
    local(key)?;
    match request {
        Read::List(scope) => {
            let value = store(source, &scope)?
                .list(scope.request())
                .map_err(error)?;
            let rows: Vec<CodexAgentRecord> =
                serde_json::from_value(value["agents"].clone()).map_err(|_| Error::Storage)?;
            Ok(ReadResult::Agents(
                rows.into_iter()
                    .map(|r| AgentRow {
                        key: r
                            .file_name
                            .strip_suffix(".toml")
                            .unwrap_or(&r.file_name)
                            .into(),
                        name: redact_log_text(&r.name),
                        description: r.description.map(|s| redact_log_text(&s)),
                        malformed: r.parse_error.is_some(),
                    })
                    .collect(),
            ))
        }
        Read::Open(scope, name) => {
            let store = store(source, &scope)?;
            let context = store.context(scope.request()).map_err(error)?;
            let snapshot = store.read(&context, &name).map_err(error)?;
            if !snapshot.exists {
                return Err(Error::Invalid);
            }
            Ok(ReadResult::Opened(Arc::new(Session {
                environment: key.environment.clone(),
                scope,
                name,
                snapshot,
            })))
        }
        Read::Sources => {
            let value = sources(source)?
                .codex_list_agent_sources()
                .await
                .map_err(error)?;
            let response: CodexAgentSourcesResponse =
                serde_json::from_value(value).map_err(|_| Error::Storage)?;
            Ok(ReadResult::Sources(response.sources))
        }
        Read::Catalog(id) => {
            let value = sources(source)?.cached_catalog(&id).map_err(error)?;
            let response = serde_json::from_value(value).map_err(|_| Error::Storage)?;
            Ok(ReadResult::Catalog(Box::new(response)))
        }
    }
}
fn validate_session(
    source: &ConfigSource,
    key: &RequestKey,
    session: &Session,
) -> Result<AgentStore, Error> {
    if session.environment != key.environment {
        return Err(Error::Invalid);
    }
    let store = store(source, &session.scope)?;
    let context = store.context(session.scope.request()).map_err(error)?;
    let current = store.read(&context, &session.name).map_err(error)?;
    if current.source != session.snapshot.source {
        return Err(Error::Invalid);
    }
    if current.token != session.snapshot.token {
        return Err(Error::Conflict);
    }
    Ok(store)
}
async fn mutate(
    source: &ConfigSource,
    key: &RequestKey,
    command: Mutation,
) -> Result<Outcome, Error> {
    local(key)?;
    match command {
        Mutation::Add {
            scope,
            name,
            content,
        } => {
            store(source, &scope)?
                .add(
                    scope.request(),
                    name.clone(),
                    serde_json::json!({"rawToml":content,"name":name}),
                )
                .map_err(error)?;
            Ok(Outcome::Saved)
        }
        Mutation::Save { session, content } => {
            validate_session(source, key, &session)?
                .update(
                    session.scope.request(),
                    session.name.clone(),
                    serde_json::json!({"rawToml":content}),
                    Some(&session.snapshot.token),
                )
                .map_err(error)?;
            Ok(Outcome::Saved)
        }
        Mutation::Delete(session) => {
            validate_session(source, key, &session)?
                .delete(
                    session.scope.request(),
                    session.name.clone(),
                    Some(&session.snapshot.token),
                )
                .map_err(error)?;
            Ok(Outcome::Deleted)
        }
        Mutation::Rename { session, name } => {
            validate_session(source, key, &session)?
                .rename(
                    session.scope.request(),
                    session.name.clone(),
                    name,
                    Some(&session.snapshot.token),
                )
                .map_err(error)?;
            Ok(Outcome::Renamed)
        }
        Mutation::Copy {
            session,
            target,
            name,
        } => {
            let store = validate_session(source, key, &session)?;
            let _ = self::store(source, &target)?;
            store
                .copy(
                    session.scope.request(),
                    target.request(),
                    session.name.clone(),
                    Some(name),
                    Some(&session.snapshot.token),
                )
                .map_err(error)?;
            Ok(Outcome::Copied)
        }
        Mutation::Source(action) => {
            let mut store = sources(source)?;
            match action {
                SourceAction::Add(url) => {
                    store
                        .codex_add_agent_source(CodexAgentSourceRequest { url })
                        .await
                        .map_err(error)?;
                }
                SourceAction::Remove(id) => {
                    store.codex_remove_agent_source(id).await.map_err(error)?
                }
                SourceAction::Refresh(id) => {
                    store.codex_sync_agent_source(id).await.map_err(error)?;
                }
                SourceAction::Install(request) => {
                    store
                        .codex_install_source_agent(request)
                        .await
                        .map_err(error)?;
                }
                SourceAction::Sync(request) => {
                    store
                        .codex_sync_source_install(request)
                        .await
                        .map_err(error)?;
                }
                SourceAction::AcceptLocal(install_id) => {
                    store
                        .codex_accept_local_source_install(CodexAgentSourceInstallActionRequest {
                            install_id,
                        })
                        .await
                        .map_err(error)?;
                }
                SourceAction::Untrack(install_id) => {
                    store
                        .codex_untrack_source_install(CodexAgentSourceInstallActionRequest {
                            install_id,
                        })
                        .await
                        .map_err(error)?;
                }
            }
            Ok(Outcome::SourceUpdated)
        }
    }
}
impl Services {
    pub fn codex_agents_read(
        &self,
        key: RequestKey,
        request: Read,
    ) -> Result<TaskSubscription<ReadResult, Error>, AdmissionError> {
        let source = self.source.clone();
        let run_key = key.clone();
        self.executor.read(key, move |_, _| {
            tokio::runtime::Handle::current().block_on(read(&source, &run_key, request))
        })
    }
    pub fn codex_agents_state(&self) -> Arc<OperationState> {
        self.codex_agent_operations.updates.borrow().clone()
    }
    pub fn observe_codex_agents(&self) -> watch::Receiver<Arc<OperationState>> {
        self.codex_agent_operations.updates.subscribe()
    }
    pub fn start_codex_agents(&self, key: RequestKey, command: Mutation) -> Result<(), Error> {
        local(&key)?;
        let owner = self.codex_agent_operations.clone();
        {
            let mut state = owner.state.lock().map_err(|_| Error::Storage)?;
            if state.busy {
                return Err(Error::Busy);
            }
            state.busy = true;
            state.environment = Some(key.environment.clone());
            state.result = None;
            owner.updates.send_replace(Arc::new(state.clone()));
        }
        let pending = Pending {
            owner: owner.clone(),
            finished: false,
        };
        let source = self.source.clone();
        let run_key = key.clone();
        self.executor
            .job(key, move |_, _| {
                let mut pending = pending;
                let result =
                    tokio::runtime::Handle::current().block_on(mutate(&source, &run_key, command));
                owner.finish(result);
                pending.finished = true;
                match result {
                    Ok(value) => Completion::Success(value),
                    Err(error) => Completion::Failed(error),
                }
            })
            .map(|_| ())
            .map_err(|_| Error::Interrupted)
    }
}
