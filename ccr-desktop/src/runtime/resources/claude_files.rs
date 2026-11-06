//! Captured local Claude resources; remote requests refuse before resolving paths.
use super::roots;
use crate::runtime::{ConfigSource, Services};
use ccr_app::{
    platform_settings::{Document, Format},
    resources::{claude_collections as collections, claude_files as files},
    runtime::{AdmissionError, Completion, EnvironmentId, Platform, RequestKey, TaskSubscription},
};
use ccr_core::core::BackupPolicy;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Rules,
    OutputStyles,
    Agents,
    SlashCommands,
    Plugins,
}
impl Kind {
    fn file(self) -> Option<files::Kind> {
        match self {
            Self::Rules => Some(files::Kind::Rule),
            Self::OutputStyles => Some(files::Kind::OutputStyle),
            _ => None,
        }
    }
    fn collection(self) -> Option<collections::Collection> {
        match self {
            Self::Agents => Some(collections::Collection::Agents),
            Self::SlashCommands => Some(collections::Collection::SlashCommands),
            Self::Plugins => Some(collections::Collection::Plugins),
            _ => None,
        }
    }
    pub fn is_file(self) -> bool {
        self.file().is_some()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Unsupported,
    Invalid,
    UnsafePath,
    Storage,
    TooLarge,
    Conflict,
    Missing,
    Duplicate,
}
impl From<files::Error> for Error {
    fn from(error: files::Error) -> Self {
        match error {
            files::Error::InvalidName => Self::Invalid,
            files::Error::UnsafePath => Self::UnsafePath,
            files::Error::Storage => Self::Storage,
            files::Error::TooLarge => Self::TooLarge,
            files::Error::Conflict => Self::Conflict,
            files::Error::Missing => Self::Missing,
        }
    }
}
impl From<collections::Error> for Error {
    fn from(error: collections::Error) -> Self {
        match error {
            collections::Error::Invalid => Self::Invalid,
            collections::Error::Missing => Self::Missing,
            collections::Error::Duplicate => Self::Duplicate,
            collections::Error::Conflict => Self::Conflict,
            collections::Error::Storage => Self::Storage,
        }
    }
}

#[derive(Clone)]
pub struct Row {
    pub name: String,
    pub enabled: Option<bool>,
}

enum DocumentSession {
    File(files::Session),
    Collection(collections::Session),
}
pub struct Session {
    document: DocumentSession,
    environment: EnvironmentId,
    pub kind: Kind,
    pub name: String,
    pub creating: bool,
    pub source: std::path::PathBuf,
}
impl Session {
    /// Collection entries can contain secrets. Edit their submitted patch only.
    pub fn initial_text(&self) -> String {
        match &self.document {
            DocumentSession::File(session) => session.content().to_owned(),
            DocumentSession::Collection(_) if self.creating => match self.kind {
                Kind::Agents => "{\n  \"model\": \"\",\n  \"tools\": []\n}".into(),
                Kind::SlashCommands => "{\n  \"description\": \"\",\n  \"command\": \"\"\n}".into(),
                Kind::Plugins => {
                    "{\n  \"id\": \"\",\n  \"version\": \"\",\n  \"enabled\": true\n}".into()
                }
                _ => "{}".into(),
            },
            DocumentSession::Collection(_) => "{}".into(),
        }
    }
}
pub enum Command {
    List(Kind),
    Open {
        kind: Kind,
        name: String,
        creating: bool,
    },
    Commit {
        session: Arc<Session>,
        content: Option<String>,
    },
}
pub enum Outcome {
    Listed(Vec<Row>),
    Opened(Arc<Session>),
    Committed,
}

fn execute(source: &ConfigSource, key: &RequestKey, command: Command) -> Result<Outcome, Error> {
    if !matches!(key.environment, EnvironmentId::Local(_)) || key.platform != Platform::Claude {
        return Err(Error::Unsupported);
    }
    if let Command::Commit { session, content } = command {
        if let Some(layout) = source.benchmark() {
            let path = match &session.document {
                DocumentSession::File(file) => file.path(),
                DocumentSession::Collection(entry) => entry.path(),
            };
            layout.check_path(path).map_err(|_| Error::UnsafePath)?;
        }
        if session.environment != key.environment {
            return Err(Error::Unsupported);
        }
        match &session.document {
            DocumentSession::File(file) => match content {
                Some(content) => file.save(&content)?,
                None => file.delete()?,
            },
            DocumentSession::Collection(entry) => {
                let mutation = match content {
                    Some(content) => {
                        let patch = serde_json::from_str(&content).map_err(|_| Error::Invalid)?;
                        if session.creating {
                            collections::Mutation::Add(patch)
                        } else {
                            collections::Mutation::Patch(patch)
                        }
                    }
                    None => collections::Mutation::Delete,
                };
                entry.commit(mutation)?;
            }
        }
        return Ok(Outcome::Committed);
    }
    let roots = roots(source).map_err(|_| Error::Storage)?;
    // Rules and output styles retain their current Tauri home-relative source;
    // settings collections honor the existing runtime settings path overrides.
    let store = files::Store::new(
        roots.home.join(".claude"),
        roots.ccr.join("backups/claude-resources"),
        roots.locks.clone(),
    );
    let settings = Document::new(
        roots.claude.settings_file,
        None,
        BackupPolicy::Dir {
            dir: roots.ccr.join("backups/claude"),
            prefix: "settings".into(),
        },
        Some(roots.locks),
        Format::Json,
    );
    match command {
        Command::List(kind) => {
            let rows = if let Some(file) = kind.file() {
                store
                    .list(file)?
                    .into_iter()
                    .map(|row| Row {
                        name: row.name,
                        enabled: None,
                    })
                    .collect()
            } else {
                collections::list(&settings, kind.collection().ok_or(Error::Invalid)?)?
                    .into_iter()
                    .map(|row| Row {
                        name: row.name,
                        enabled: Some(row.enabled),
                    })
                    .collect()
            };
            Ok(Outcome::Listed(rows))
        }
        Command::Open {
            kind,
            name,
            creating,
        } => {
            let (document, exists) = if let Some(file) = kind.file() {
                let session = store.open(file, &name)?;
                let exists = session.exists();
                (DocumentSession::File(session), exists)
            } else {
                let session = collections::Session::open(
                    settings,
                    kind.collection().ok_or(Error::Invalid)?,
                    &name,
                )?;
                let exists = session.exists();
                (DocumentSession::Collection(session), exists)
            };
            if creating && exists {
                return Err(Error::Duplicate);
            }
            if !creating && !exists {
                return Err(Error::Missing);
            }
            let source = match &document {
                DocumentSession::File(session) => session.path().to_path_buf(),
                DocumentSession::Collection(session) => session.path().to_path_buf(),
            };
            Ok(Outcome::Opened(Arc::new(Session {
                document,
                environment: key.environment.clone(),
                kind,
                name,
                creating,
                source,
            })))
        }
        Command::Commit { .. } => Err(Error::Invalid),
    }
}

impl Services {
    pub fn claude_resource_request(
        &self,
        key: RequestKey,
        command: Command,
    ) -> Result<TaskSubscription<Outcome, Error>, AdmissionError> {
        let source = self.source.clone();
        let run_key = key.clone();
        if matches!(command, Command::Commit { .. }) {
            self.executor.job(key, move |cancel, _| {
                if cancel.is_cancelled() {
                    return Completion::Cancelled;
                }
                match execute(&source, &run_key, command) {
                    Ok(result) => Completion::Success(result),
                    Err(error) => Completion::Failed(error),
                }
            })
        } else {
            self.executor
                .read(key, move |_, _| execute(&source, &run_key, command))
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn source_bound_file_and_collection_sessions_refuse_other_environments() {
        let root = tempfile::tempdir().unwrap();
        let source = ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Default::default(),
        };
        let mut key = RequestKey {
            environment: EnvironmentId::Local("fixture".into()),
            platform: Platform::Claude,
            generation: 1,
            request_id: 1,
        };
        for (kind, name, content) in [
            (Kind::Rules, "fixture.md", "instruction"),
            (Kind::Agents, "fixture", r#"{"model":"fixture"}"#),
        ] {
            let Outcome::Opened(session) = execute(
                &source,
                &key,
                Command::Open {
                    kind,
                    name: name.into(),
                    creating: true,
                },
            )
            .unwrap() else {
                panic!("session required");
            };
            key.environment = EnvironmentId::Ssh("fixture".into());
            assert!(matches!(
                execute(
                    &source,
                    &key,
                    Command::Commit {
                        session: session.clone(),
                        content: Some(content.into())
                    }
                ),
                Err(Error::Unsupported)
            ));
            key.environment = EnvironmentId::Local("other".into());
            assert!(matches!(
                execute(
                    &source,
                    &key,
                    Command::Commit {
                        session: session.clone(),
                        content: Some(content.into())
                    }
                ),
                Err(Error::Unsupported)
            ));
            key.environment = EnvironmentId::Local("fixture".into());
            execute(
                &source,
                &key,
                Command::Commit {
                    session,
                    content: Some(content.into()),
                },
            )
            .unwrap();
        }
    }
}
