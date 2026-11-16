//! Captured local Claude resources; remote requests refuse before resolving paths.
use crate::runtime::remote_resources::{self as remote, Remote};
use crate::runtime::{ConfigSource, Services};
use ccr_app::environments::EnvironmentPolicy;
use ccr_app::resources::environment::Family;
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
    Remote(ccr_app::environments::io::Error),
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
    pub unsupported: bool,
    pub name: String,
    pub enabled: Option<bool>,
}

enum DocumentSession {
    File(files::Session),
    Collection(collections::Session),
}
pub struct Session {
    remote: Remote,
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

#[cfg(test)]
fn execute(source: &ConfigSource, key: &RequestKey, command: Command) -> Result<Outcome, Error> {
    execute_at(source, key, command, None)
}
fn execute_at(
    source: &ConfigSource,
    key: &RequestKey,
    command: Command,
    remote: Remote,
) -> Result<Outcome, Error> {
    if (remote.is_none()
        && EnvironmentPolicy::LocalOnly
            .admit(&key.environment)
            .is_err())
        || key.platform != Platform::Claude
    {
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
    let roots = remote::roots(source, &remote).map_err(|_| Error::Storage)?;
    // 所有 Claude 用户资源共享捕获的配置根；settings 文件覆盖不改变资源根。
    let store = files::Store::new(
        roots.claude.config_dir.clone(),
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
            let mut rows: Vec<Row> = if let Some(file) = kind.file() {
                store
                    .list(file)?
                    .into_iter()
                    .map(|row| Row {
                        unsupported: false,
                        name: row.name,
                        enabled: None,
                    })
                    .collect()
            } else {
                collections::list(&settings, kind.collection().ok_or(Error::Invalid)?)?
                    .into_iter()
                    .map(|row| Row {
                        unsupported: false,
                        name: row.name,
                        enabled: Some(row.enabled),
                    })
                    .collect()
            };
            if let Some(remote) = &remote {
                rows.extend(remote.unsupported.iter().map(|path| Row {
                    name: format!("Unsupported source / 不支持的来源: {path}"),
                    enabled: None,
                    unsupported: true,
                }));
            }
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
                source: remote
                    .as_ref()
                    .map(|r| r.label(&source).map(std::path::PathBuf::from))
                    .transpose()
                    .map_err(Error::Remote)?
                    .unwrap_or(source),
                remote,
                document,
                environment: key.environment.clone(),
                kind,
                name,
                creating,
            })))
        }
        Command::Commit { .. } => Err(Error::Invalid),
    }
}

fn run(
    source: &ConfigSource,
    key: &RequestKey,
    command: Command,
    io: Result<Option<ccr_app::environments::io::EnvironmentIo>, ccr_app::environments::io::Error>,
    cancel: tokio_util::sync::CancellationToken,
) -> Result<Outcome, Error> {
    if let Command::List(kind) = &command
        && let Some(file) = kind.file()
        && let Some(io) = io.clone().map_err(Error::Remote)?
    {
        let root = format!(".claude/{}", file.directory());
        let mut pending = vec![(root.clone(), 0)];
        let mut rows = Vec::new();
        let mut visited = 0;
        while let Some((directory, depth)) = pending.pop() {
            let entries = match tokio::runtime::Handle::current()
                .block_on(io.entries(&directory, cancel.clone()))
            {
                Ok(entries) => entries.unwrap_or_default(),
                Err(ccr_app::environments::io::Error::Ssh("ssh_invalid_remote_path")) => {
                    rows.push(Row {
                        name: format!("Unsupported source / 不支持的来源: {directory}"),
                        enabled: None,
                        unsupported: true,
                    });
                    continue;
                }
                Err(error) => return Err(Error::Remote(error)),
            };
            for entry in entries {
                visited += 1;
                if visited > 4096 {
                    return Err(Error::TooLarge);
                }
                let path = format!("{directory}/{}", entry.name);
                if entry.kind == ccr_app::environments::io::EntryKind::Unsupported
                    || ccr_app::environments::config_mirror::validate_materialized_paths([
                        path.as_str()
                    ])
                    .is_err()
                {
                    rows.push(Row {
                        name: format!("Unsupported source / 不支持的来源: {path}"),
                        enabled: None,
                        unsupported: true,
                    });
                    continue;
                }
                if entry.kind == ccr_app::environments::io::EntryKind::Directory {
                    if depth >= 16 {
                        return Err(Error::TooLarge);
                    }
                    pending.push((path, depth + 1));
                } else if entry.name.ends_with(".md") {
                    let name = path
                        .strip_prefix(&format!("{root}/"))
                        .ok_or(Error::Invalid)?;
                    if files::validate_name(name).is_err() {
                        rows.push(Row {
                            name: format!("Unsupported source / 不支持的来源: {path}"),
                            enabled: None,
                            unsupported: true,
                        });
                        continue;
                    }
                    if rows.len() >= 1000 {
                        return Err(Error::TooLarge);
                    }
                    rows.push(Row {
                        name: path
                            .strip_prefix(&format!("{root}/"))
                            .ok_or(Error::Invalid)?
                            .into(),
                        enabled: None,
                        unsupported: false,
                    });
                }
            }
        }
        rows.sort_by(|a, b| a.name.cmp(&b.name));
        return Ok(Outcome::Listed(rows));
    }
    let writing = matches!(command, Command::Commit { .. });
    let mut targets = std::collections::BTreeSet::new();
    if let Command::Open { kind, name, .. } = &command
        && let Some(kind) = kind.file()
    {
        targets.insert(format!(".claude/{}/{name}", kind.directory()));
    }
    let remote = match &command {
        Command::Commit { session, .. } => session.remote.clone(),
        _ => remote::capture(
            io,
            match &command {
                Command::List(Kind::Rules)
                | Command::Open {
                    kind: Kind::Rules, ..
                } => Family::ClaudeRules,
                Command::List(Kind::OutputStyles)
                | Command::Open {
                    kind: Kind::OutputStyles,
                    ..
                } => Family::ClaudeStyles,
                _ => Family::ClaudeCollections,
            },
            targets,
            cancel.clone(),
        )
        .map_err(Error::Remote)?,
    };
    let result = execute_at(source, key, command, remote.clone())?;
    if writing {
        remote::publish(&remote, cancel).map_err(Error::Remote)?;
    }
    Ok(result)
}
impl Services {
    pub fn claude_resource_request(
        &self,
        key: RequestKey,
        command: Command,
    ) -> Result<TaskSubscription<Outcome, Error>, AdmissionError> {
        let source = self.source.clone();
        let run_key = key.clone();
        let io = self.resource_io(&key.environment);
        if matches!(command, Command::Commit { .. }) {
            self.executor.job(key, move |cancel, _| {
                if cancel.is_cancelled() {
                    return Completion::Cancelled;
                }
                match run(&source, &run_key, command, io, cancel) {
                    Ok(result) => Completion::Success(result),
                    Err(error) => Completion::Failed(error),
                }
            })
        } else {
            self.executor.read(key, move |cancel, _| {
                run(&source, &run_key, command, io, cancel)
            })
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::super::{ResourceCommand, ResourceResult, execute as prompt_execute};
    use super::*;
    use std::{fs, path::PathBuf};

    // 环境变量是进程级状态：在独立子进程里设置，不污染并行测试；锁、备份和 CCR 根都指向临时目录。
    #[test]
    fn rules_styles_and_memory_follow_config_dir_without_touching_default_home() {
        const TEST: &str = "runtime::resources::claude_files::tests::rules_styles_and_memory_follow_config_dir_without_touching_default_home";
        let Some(root) = std::env::var_os("CCR_TEST_NATIVE_CLAUDE_CONFIG_ROOT") else {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().canonicalize().unwrap();
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", TEST, "--nocapture"])
                .env("CCR_TEST_NATIVE_CLAUDE_CONFIG_ROOT", &root)
                .env("CLAUDE_CONFIG_DIR", root.join("custom-claude"))
                .env("CCR_ROOT", root.join("ccr"))
                .env("CCR_LOCK_DIR", root.join("locks"))
                .env("CODEX_HOME", root.join("codex"))
                .env("GROK_HOME", root.join("grok"))
                .env("HOME", root.join("home"))
                .env("USERPROFILE", root.join("home"))
                .env_remove("CCR_DATA_DIR")
                .env_remove("CCR_CODEX_DIR")
                .env_remove("CCR_SETTINGS_PATH")
                .env_remove("CLAUDE_JSON_PATH")
                .env_remove("CCR_BACKUP_DIR")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "stdout={} stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
            return;
        };
        let root = PathBuf::from(root);
        let custom = root.join("custom-claude");
        let default_claude = root.join("home/.claude");
        let original = b"default home must remain byte-identical";
        let sentinels = ["CLAUDE.md", "rules/ambient.md", "output-styles/ambient.md"];
        for relative in sentinels {
            let path = default_claude.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, original).unwrap();
        }
        let source = ConfigSource::Local;
        let key = RequestKey {
            environment: EnvironmentId::Local("local".into()),
            platform: Platform::Claude,
            generation: 1,
            request_id: 1,
        };
        let open = |kind, creating| {
            let Outcome::Opened(session) = execute(
                &source,
                &key,
                Command::Open {
                    kind,
                    name: "custom.md".into(),
                    creating,
                },
            )
            .unwrap() else {
                panic!("session required");
            };
            session
        };
        for (kind, directory) in [
            (Kind::Rules, "rules"),
            (Kind::OutputStyles, "output-styles"),
        ] {
            let created = open(kind, true);
            assert_eq!(created.source, custom.join(directory).join("custom.md"));
            execute(
                &source,
                &key,
                Command::Commit {
                    session: created,
                    content: Some("first".into()),
                },
            )
            .unwrap();
            let Outcome::Listed(rows) = execute(&source, &key, Command::List(kind)).unwrap() else {
                panic!("rows required");
            };
            let names: Vec<_> = rows.iter().map(|row| row.name.as_str()).collect();
            assert_eq!(names, ["custom.md"]);
            let reopened = open(kind, false);
            assert_eq!(reopened.initial_text(), "first");
            execute(
                &source,
                &key,
                Command::Commit {
                    session: reopened,
                    content: Some("second".into()),
                },
            )
            .unwrap();
            assert_eq!(
                fs::read_to_string(custom.join(directory).join("custom.md")).unwrap(),
                "second"
            );
        }

        let open_memory = || {
            let ResourceResult::Opened(session) = prompt_execute(
                &source,
                &key,
                ResourceCommand::OpenPrompt("claude-user-memory"),
            )
            .unwrap() else {
                panic!("prompt session required");
            };
            session
        };
        let ResourceResult::Prompts(rows) =
            prompt_execute(&source, &key, ResourceCommand::ListPrompts).unwrap()
        else {
            panic!("prompt rows required");
        };
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].path, custom.join("CLAUDE.md"));
        assert!(!rows[0].exists);
        let session = open_memory();
        assert_eq!(session.snapshot.source, custom.join("CLAUDE.md"));
        assert!(matches!(
            prompt_execute(&source, &key, ResourceCommand::CreatePrompt { session }).unwrap(),
            ResourceResult::Saved {
                conflict: false,
                ..
            }
        ));
        assert!(matches!(
            prompt_execute(
                &source,
                &key,
                ResourceCommand::SavePrompt {
                    session: open_memory(),
                    content: "# Memory\n".into(),
                },
            )
            .unwrap(),
            ResourceResult::Saved {
                conflict: false,
                ..
            }
        ));
        assert_eq!(open_memory().snapshot.content, "# Memory\n");
        assert_eq!(
            fs::read_to_string(custom.join("CLAUDE.md")).unwrap(),
            "# Memory\n"
        );

        // 主目录下的 .claude 保持原样；备份与锁仍在 CCR 根和锁目录，不进入配置根。
        for relative in sentinels {
            assert_eq!(fs::read(default_claude.join(relative)).unwrap(), original);
        }
        for relative in ["rules/custom.md", "output-styles/custom.md"] {
            assert!(!default_claude.join(relative).exists());
        }
        for directory in [
            "ccr/backups/claude-resources",
            "ccr/backups/claude",
            "locks",
        ] {
            assert!(fs::read_dir(root.join(directory)).unwrap().next().is_some());
        }
        assert!(!custom.join("backups").exists());
    }

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
