//! Captured OpenCode/Antigravity directory resources; no current-directory fallback.
#[cfg(test)]
use super::roots;
use crate::runtime::remote_resources::{self as remote, Remote};
use crate::runtime::{ConfigSource, Services};
use ccr_app::resources::environment::Family;
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
    Remote(ccr_app::environments::io::Error),
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
    if ccr_app::environments::EnvironmentPolicy::FollowsEnvironment
        .admit(&key.environment)
        .is_ok()
        && key.platform == platform(kind)
    {
        Ok(())
    } else {
        Err(Error::Unsupported)
    }
}
#[cfg(test)]
fn store(source: &ConfigSource, kind: Kind, scope: &Scope) -> Result<files::Store, Error> {
    store_at(source, kind, scope, &None)
}
fn store_at(
    source: &ConfigSource,
    kind: Kind,
    scope: &Scope,
    remote: &Remote,
) -> Result<files::Store, Error> {
    let roots = remote::roots(source, remote).map_err(|_| Error::Storage)?;
    let root = match scope {
        Scope::User => match kind {
            Kind::OpenCodeAgent => roots.home.join(".config/opencode/agents"),
            Kind::OpenCodeCommand => roots.home.join(".config/opencode/commands"),
            Kind::OpenCodePlugin => roots.home.join(".config/opencode/plugins"),
            Kind::AntigravityCommand => roots.home.join(".gemini/commands"),
            Kind::Extension => roots.home.join(".gemini/antigravity-cli/extensions"),
        },
        Scope::Project(project) => {
            let project = if let Some(remote) = remote {
                remote.root().to_path_buf()
            } else {
                project.clone()
            };
            if kind == Kind::Extension {
                return Err(Error::Unsupported);
            }
            if !project.is_absolute() || !project.is_dir() {
                return Err(Error::Invalid);
            }
            let canonical = project.canonicalize().map_err(|_| Error::Invalid)?;
            if let Some(layout) = source.benchmark() {
                layout.check_path(&project).map_err(|_| Error::UnsafePath)?;
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
    pub unsupported: bool,
    pub key: String,
    pub path: String,
    pub bytes: u64,
    pub description: Option<String>,
    pub version: Option<String>,
    pub malformed: bool,
}
pub struct Session {
    remote: Remote,
    file: files::Session,
    environment: EnvironmentId,
    pub kind: Kind,
    pub creating: bool,
}
impl Session {
    pub fn source_label(&self) -> String {
        redact_log_text(
            &self
                .remote
                .as_ref()
                .and_then(|r| r.label(self.file.path()).ok())
                .unwrap_or_else(|| self.file.path().to_string_lossy().into_owned()),
        )
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
#[cfg(test)]
fn read(source: &ConfigSource, key: &RequestKey, request: Read) -> Result<ReadResult, Error> {
    read_at(source, key, request, None)
}
fn read_at(
    source: &ConfigSource,
    key: &RequestKey,
    request: Read,
    remote: Remote,
) -> Result<ReadResult, Error> {
    let (kind, scope) = match &request {
        Read::List { kind, scope } | Read::Open { kind, scope, .. } => (*kind, scope),
    };
    gate(key, kind)?;
    if remote.is_none() && !matches!(key.environment, EnvironmentId::Local(_)) {
        return Err(Error::Unsupported);
    }
    let store = store_at(source, kind, scope, &remote)?;
    match request {
        Read::List { .. } => {
            let mut rows: Vec<Row> = store
                .list()?
                .into_iter()
                .map(|row| Row {
                    unsupported: false,
                    path: redact_log_text(
                        &remote
                            .as_ref()
                            .and_then(|r| r.label(&store.directory().join(&row.key)).ok())
                            .unwrap_or_else(|| {
                                store
                                    .directory()
                                    .join(&row.key)
                                    .to_string_lossy()
                                    .into_owned()
                            }),
                    ),
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
                .collect();
            if let Some(remote) = &remote {
                rows.extend(remote.unsupported.iter().map(|path| Row {
                    key: path.clone(),
                    path: path.clone(),
                    bytes: 0,
                    description: Some("Unsupported source / 不支持的来源".into()),
                    version: None,
                    malformed: false,
                    unsupported: true,
                }));
            }
            Ok(ReadResult::Listed(rows))
        }
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
                remote,
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
fn directory(kind: Kind) -> &'static str {
    match kind {
        Kind::OpenCodeAgent => ".config/opencode/agents",
        Kind::OpenCodeCommand => ".config/opencode/commands",
        Kind::OpenCodePlugin => ".config/opencode/plugins",
        Kind::AntigravityCommand => ".gemini/commands",
        Kind::Extension => ".gemini/antigravity-cli/extensions",
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
        let io = self.resource_io(&key.environment);
        self.executor.read(key, move |cancel, _| {
            let (kind, scope, target) = match &request {
                Read::List { kind, scope } => (*kind, scope, None),
                Read::Open {
                    kind, scope, name, ..
                } => (*kind, scope, Some(name)),
            };
            let project = match scope {
                Scope::User => None,
                Scope::Project(path) => Some(path.as_path()),
            };
            let io = remote::project_io(io, project, cancel.clone()).map_err(Error::Remote)?;
            if kind == Kind::OpenCodePlugin && io.is_some() {
                let io = io.ok_or(Error::Unsupported)?;
                if !matches!(request, Read::List { .. }) {
                    return Err(Error::Unsupported);
                }
                let directory = if project.is_some() {
                    ".opencode/plugins"
                } else {
                    ".config/opencode/plugins"
                };
                let mut rows = Vec::new();
                for entry in tokio::runtime::Handle::current()
                    .block_on(io.entries(directory, cancel.clone()))
                    .map_err(Error::Remote)?
                    .unwrap_or_default()
                {
                    let path = format!("{directory}/{}", entry.name);
                    if rows.len() >= 1000 {
                        return Err(Error::TooLarge);
                    }
                    if entry.kind == ccr_app::environments::io::EntryKind::Unsupported
                        || ccr_app::environments::config_mirror::validate_materialized_paths([
                            path.as_str()
                        ])
                        .is_err()
                    {
                        rows.push(Row {
                            unsupported: true,
                            key: entry.name,
                            path,
                            bytes: 0,
                            description: Some("Unsupported source / 不支持的来源".into()),
                            version: None,
                            malformed: false,
                        });
                        continue;
                    }
                    if entry.kind != ccr_app::environments::io::EntryKind::File
                        || !entry.name.rsplit_once('.').is_some_and(|(_, extension)| {
                            files::opencode::is_plugin_extension(extension)
                        })
                    {
                        continue;
                    }
                    let bytes = tokio::runtime::Handle::current()
                        .block_on(io.metadata(&path, cancel.clone()))
                        .map_err(Error::Remote)?
                        .map(|m| m.0)
                        .unwrap_or_default();
                    let label = match io.roots().relative(&path).map_err(Error::Remote)? {
                        ccr_app::environments::io::EnvironmentPath::Ssh(path) => {
                            path.as_str().to_owned()
                        }
                        _ => return Err(Error::Unsupported),
                    };
                    rows.push(Row {
                        unsupported: false,
                        key: entry.name,
                        path: label,
                        bytes,
                        description: None,
                        version: None,
                        malformed: false,
                    });
                }
                return Ok(ReadResult::Listed(rows));
            }
            let mut targets = std::collections::BTreeSet::new();
            if let Some(name) = target {
                targets.insert(format!(
                    "{}/{name}",
                    if project.is_some() && platform(kind) == Platform::OpenCode {
                        match kind {
                            Kind::OpenCodeAgent => ".opencode/agents",
                            Kind::OpenCodeCommand => ".opencode/commands",
                            _ => return Err(Error::Unsupported),
                        }
                    } else {
                        directory(kind)
                    }
                ));
            }
            let family = Family::PlatformFile(kind, project.is_some());
            let remote = remote::capture(Ok(io), family, targets, cancel).map_err(Error::Remote)?;
            read_at(&source, &run_key, request, remote)
        })
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
            .job(key, move |cancel, _| {
                let mut pending = pending;
                let result = match content {
                    Some(content) => session.file.save(&content).map(|_| Outcome::Saved),
                    None => session.file.delete().map(|_| Outcome::Deleted),
                }
                .map_err(Error::from)
                .and_then(|outcome| {
                    remote::publish(&session.remote, cancel).map_err(Error::Remote)?;
                    Ok(outcome)
                });
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
