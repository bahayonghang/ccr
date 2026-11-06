//! Captured OpenCode/Antigravity directory resources; no current-directory fallback.
use super::roots;
use crate::runtime::{ConfigSource, Services};
use ccr_app::{
    resources::platform_files as files,
    runtime::{AdmissionError, Completion, EnvironmentId, Platform, RequestKey, TaskSubscription},
};
use ccr_core::core::log_redact::redact_log_text;
pub use files::Kind;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::watch;
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Scope {
    #[default]
    User,
    Project(PathBuf),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Unsupported,
    Invalid,
    Storage,
    Conflict,
    TooLarge,
    UnsafePath,
    Missing,
    Duplicate,
    Busy,
    Interrupted,
}
impl From<files::Error> for Error {
    fn from(error: files::Error) -> Self {
        match error {
            files::Error::InvalidName | files::Error::InvalidContent => Self::Invalid,
            files::Error::UnsafePath => Self::UnsafePath,
            files::Error::Storage => Self::Storage,
            files::Error::TooLarge => Self::TooLarge,
            files::Error::Conflict => Self::Conflict,
            files::Error::Missing => Self::Missing,
            files::Error::ReadOnly => Self::Unsupported,
        }
    }
}
pub fn platform(kind: Kind) -> Platform {
    match kind {
        Kind::OpenCodeAgent | Kind::OpenCodeCommand | Kind::OpenCodePlugin => Platform::OpenCode,
        Kind::AntigravityCommand | Kind::Extension => Platform::Gemini,
    }
}
fn gate(key: &RequestKey, kind: Kind) -> Result<(), Error> {
    if matches!(key.environment, EnvironmentId::Local(_)) && key.platform == platform(kind) {
        Ok(())
    } else {
        Err(Error::Unsupported)
    }
}
fn store(source: &ConfigSource, kind: Kind, scope: &Scope) -> Result<files::Store, Error> {
    let roots = roots(source).map_err(|_| Error::Storage)?;
    let root = match scope {
        Scope::User => match kind {
            Kind::OpenCodeAgent => roots.home.join(".config/opencode/agents"),
            Kind::OpenCodeCommand => roots.home.join(".config/opencode/commands"),
            Kind::OpenCodePlugin => roots.home.join(".config/opencode/plugins"),
            Kind::AntigravityCommand => roots.home.join(".gemini/commands"),
            Kind::Extension => roots.home.join(".gemini/antigravity-cli/extensions"),
        },
        Scope::Project(project) => {
            if kind == Kind::Extension {
                return Err(Error::Unsupported);
            }
            if !project.is_absolute() || !project.is_dir() {
                return Err(Error::Invalid);
            }
            let canonical = project.canonicalize().map_err(|_| Error::Invalid)?;
            if let Some(layout) = source.benchmark() {
                layout.check_path(project).map_err(|_| Error::UnsafePath)?;
            }
            if source.is_fixture()
                && !source.is_benchmark()
                && !canonical.starts_with(roots.home.canonicalize().map_err(|_| Error::Invalid)?)
            {
                return Err(Error::UnsafePath);
            }
            // Retain the supplied absolute path so the domain rejects linked ancestors.
            match kind {
                Kind::OpenCodeAgent => project.join(".opencode/agents"),
                Kind::OpenCodeCommand => project.join(".opencode/commands"),
                Kind::OpenCodePlugin => project.join(".opencode/plugins"),
                Kind::AntigravityCommand => project.join(".gemini/commands"),
                Kind::Extension => return Err(Error::Unsupported),
            }
        }
    };
    Ok(files::Store::new(
        root,
        roots.ccr.join("backups/platform-resources"),
        roots.locks,
        kind,
    ))
}
#[derive(Clone)]
pub struct Row {
    pub key: String,
    pub path: String,
    pub bytes: u64,
    pub description: Option<String>,
    pub version: Option<String>,
    pub malformed: bool,
}
pub struct Session {
    file: files::Session,
    environment: EnvironmentId,
    pub kind: Kind,
    pub creating: bool,
}
impl Session {
    pub fn source_label(&self) -> String {
        redact_log_text(&self.file.path().to_string_lossy())
    }
    pub fn initial_text(&self) -> String {
        if self.creating {
            self.kind.initial_text().into()
        } else {
            self.file.content().into()
        }
    }
}
pub enum Read {
    List {
        kind: Kind,
        scope: Scope,
    },
    Open {
        kind: Kind,
        scope: Scope,
        name: String,
        creating: bool,
    },
}
pub enum ReadResult {
    Listed(Vec<Row>),
    Opened(Arc<Session>),
}
fn read(source: &ConfigSource, key: &RequestKey, request: Read) -> Result<ReadResult, Error> {
    let (kind, scope) = match &request {
        Read::List { kind, scope } | Read::Open { kind, scope, .. } => (*kind, scope),
    };
    gate(key, kind)?;
    let store = store(source, kind, scope)?;
    match request {
        Read::List { .. } => Ok(ReadResult::Listed(
            store
                .list()?
                .into_iter()
                .map(|row| Row {
                    path: redact_log_text(&store.directory().join(&row.key).to_string_lossy()),
                    bytes: row.bytes,
                    key: row.key,
                    description: row
                        .description
                        .map(|v| redact_log_text(&v).chars().take(240).collect()),
                    version: row
                        .version
                        .map(|v| redact_log_text(&v).chars().take(80).collect()),
                    malformed: row.malformed,
                })
                .collect(),
        )),
        Read::Open {
            kind,
            scope: _,
            name,
            creating,
        } => {
            if creating && !kind.writable() {
                return Err(Error::Unsupported);
            }
            let file = store.open(&name)?;
            if creating && file.exists() {
                return Err(Error::Duplicate);
            }
            if !creating && !file.exists() {
                return Err(Error::Missing);
            }
            Ok(ReadResult::Opened(Arc::new(Session {
                file,
                environment: key.environment.clone(),
                kind,
                creating,
            })))
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Saved,
    Deleted,
}
#[derive(Clone, Default)]
pub struct OperationState {
    pub busy: bool,
    pub platform: Option<Platform>,
    pub environment: Option<EnvironmentId>,
    pub target: Option<String>,
    pub result: Option<Result<Outcome, Error>>,
}
pub struct FileOperations {
    state: Mutex<OperationState>,
    updates: watch::Sender<Arc<OperationState>>,
}
impl Default for FileOperations {
    fn default() -> Self {
        Self {
            state: Mutex::new(OperationState::default()),
            updates: watch::channel(Arc::new(OperationState::default())).0,
        }
    }
}
impl FileOperations {
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
    owner: Arc<FileOperations>,
    finished: bool,
}
impl Drop for Pending {
    fn drop(&mut self) {
        if !self.finished {
            self.owner.finish(Err(Error::Interrupted));
        }
    }
}
impl Services {
    pub fn platform_files_read(
        &self,
        key: RequestKey,
        request: Read,
    ) -> Result<TaskSubscription<ReadResult, Error>, AdmissionError> {
        let source = self.source.clone();
        let run_key = key.clone();
        self.executor
            .read(key, move |_, _| read(&source, &run_key, request))
    }
    pub fn platform_files_state(&self) -> Arc<OperationState> {
        self.platform_file_operations.updates.borrow().clone()
    }
    pub fn observe_platform_files(&self) -> watch::Receiver<Arc<OperationState>> {
        self.platform_file_operations.updates.subscribe()
    }
    pub fn commit_platform_file(
        &self,
        key: RequestKey,
        session: Arc<Session>,
        content: Option<String>,
    ) -> Result<(), Error> {
        gate(&key, session.kind)?;
        if session.environment != key.environment {
            return Err(Error::Unsupported);
        }
        if !session.kind.writable() {
            return Err(Error::Unsupported);
        }
        if let Some(layout) = self.source.benchmark() {
            layout
                .check_path(session.file.path())
                .map_err(|_| Error::UnsafePath)?;
        }
        let owner = self.platform_file_operations.clone();
        {
            let mut state = owner.state.lock().map_err(|_| Error::Storage)?;
            if state.busy {
                return Err(Error::Busy);
            }
            *state = OperationState {
                busy: true,
                platform: Some(key.platform),
                environment: Some(key.environment.clone()),
                target: Some(session.source_label()),
                result: None,
            };
            owner.updates.send_replace(Arc::new(state.clone()));
        }
        let pending = Pending {
            owner: owner.clone(),
            finished: false,
        };
        self.executor
            .job(key, move |_, _| {
                let mut pending = pending;
                let result = match content {
                    Some(content) => session.file.save(&content).map(|_| Outcome::Saved),
                    None => session.file.delete().map(|_| Outcome::Deleted),
                }
                .map_err(Error::from);
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
#[cfg(test)]
mod tests;
