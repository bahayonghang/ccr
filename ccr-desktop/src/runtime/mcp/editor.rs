//! Source-bound MCP edit sessions. Opaque snapshots never expose stored secrets to UI.
use crate::runtime::remote_resources::{self as remote, Remote};
use ccr_app::environments::EnvironmentPolicy;
use ccr_app::resources::environment::Family;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use ccr_app::{
    claude_mcp::{self, ClaudeMcpContext, ClaudeMcpScope, NativeSnapshot},
    mcp_document::{self, EntryMutation},
    platform_settings::{Document, Format, SaveOutcome, Snapshot},
    runtime::{AdmissionError, Completion, EnvironmentId, RequestKey, TaskSubscription},
};
use ccr_core::core::BackupPolicy;

use super::{ConfigSource, McpEntry, McpError, McpScope, McpSource, Services, roots};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum McpTarget {
    ClaudeUser,
    ClaudeLocal(PathBuf),
    ClaudeProject(PathBuf),
    Codex,
    Antigravity,
}

impl McpTarget {
    pub(super) fn platform(&self) -> ccr_app::runtime::Platform {
        use ccr_app::runtime::Platform;
        match self {
            Self::Codex => Platform::Codex,
            Self::Antigravity => Platform::Gemini,
            _ => Platform::Claude,
        }
    }
    pub fn for_entry(entry: &McpEntry) -> Result<Self, McpError> {
        match (&entry.source, &entry.scope) {
            (McpSource::ClaudeState, McpScope::User) => Ok(Self::ClaudeUser),
            (McpSource::ClaudeState, McpScope::Project(path)) => Ok(Self::ClaudeLocal(path.into())),
            (McpSource::ProjectFile, McpScope::Project(path)) => {
                Ok(Self::ClaudeProject(path.into()))
            }
            (McpSource::CodexConfig, McpScope::User) => Ok(Self::Codex),
            (McpSource::AntigravityConfig, McpScope::User) => Ok(Self::Antigravity),
            _ => Err(McpError::UnsupportedSource),
        }
    }
}

pub(super) enum EditDocument {
    Claude(NativeSnapshot),
    Standard {
        document: Document,
        snapshot: Snapshot,
        collection: &'static str,
        toml: bool,
    },
}

pub struct McpEditSession {
    pub(super) remote: Remote,
    pub(super) platform: ccr_app::runtime::Platform,
    pub(super) environment: EnvironmentId,
    pub(super) document: EditDocument,
    benchmark_root: Option<PathBuf>,
}

pub(super) fn project_path(source: &ConfigSource, path: &Path) -> Result<PathBuf, McpError> {
    if let Some(layout) = source.benchmark() {
        layout.check_path(path).map_err(|_| McpError::Scope)?;
    }
    if !path.is_absolute() {
        return Err(McpError::Scope);
    }
    let path = path.canonicalize().map_err(|_| McpError::Scope)?;
    if !path.is_dir() {
        return Err(McpError::Scope);
    }
    if let ConfigSource::Fixture { registry, .. } = source {
        let base = registry
            .parent()
            .ok_or(McpError::Scope)?
            .canonicalize()
            .map_err(|_| McpError::Scope)?;
        if !path.starts_with(base) {
            return Err(McpError::Scope);
        }
    }
    Ok(path)
}

#[cfg(test)]
pub(super) fn begin(
    source: &ConfigSource,
    environment: &EnvironmentId,
    target: McpTarget,
) -> Result<McpEditSession, McpError> {
    begin_at(source, environment, target, None)
}
fn begin_at(
    source: &ConfigSource,
    environment: &EnvironmentId,
    target: McpTarget,
    remote: Remote,
) -> Result<McpEditSession, McpError> {
    if remote.is_none() && EnvironmentPolicy::LocalOnly.admit(environment).is_err() {
        return Err(McpError::Unsupported);
    }
    let roots = match &remote {
        Some(remote) => crate::runtime::remote_config::roots(&remote.mirror),
        None => roots(source)?,
    };
    let platform = target.platform();
    let remote_project_key = if remote.is_some() {
        match &target {
            McpTarget::ClaudeLocal(path) => Some(path.to_str().ok_or(McpError::Scope)?.to_owned()),
            _ => None,
        }
    } else {
        None
    };
    let document = match target {
        McpTarget::ClaudeUser | McpTarget::ClaudeLocal(_) | McpTarget::ClaudeProject(_) => {
            let (scope, project_root) = match target {
                McpTarget::ClaudeUser => (ClaudeMcpScope::User, roots.home.clone()),
                McpTarget::ClaudeLocal(path) => (
                    ClaudeMcpScope::Local,
                    if remote.is_some() {
                        path
                    } else {
                        project_path(source, &path)?
                    },
                ),
                McpTarget::ClaudeProject(path) => (
                    ClaudeMcpScope::Project,
                    if let Some(remote) = &remote {
                        remote.root().into()
                    } else {
                        project_path(source, &path)?
                    },
                ),
                _ => return Err(McpError::UnsupportedSource),
            };
            EditDocument::Claude(
                claude_mcp::native_snapshot(
                    ClaudeMcpContext {
                        user_state_path: roots.claude.state_file,
                        project_root,
                        lock_directory: Some(roots.locks),
                    },
                    scope,
                )
                .map_err(|_| McpError::Storage)?,
            )
        }
        McpTarget::Codex | McpTarget::Antigravity => {
            let toml = target == McpTarget::Codex;
            let (path, collection, format) = if toml {
                (
                    roots.codex_runtime.join("config.toml"),
                    "mcp_servers",
                    Format::Toml,
                )
            } else {
                (
                    roots.home.join(".gemini/antigravity-cli/mcp_config.json"),
                    "mcpServers",
                    Format::Json,
                )
            };
            let document = Document::new(path, None, BackupPolicy::None, Some(roots.locks), format);
            let snapshot = document.read().map_err(|_| McpError::Storage)?;
            // Parse before presenting an editor; invalid source is never replaced with {}.
            if snapshot.exists {
                mcp_document::parse(&snapshot.content, toml).map_err(|_| McpError::Invalid)?;
            }
            EditDocument::Standard {
                document,
                snapshot,
                collection,
                toml,
            }
        }
    };
    let document = match (document, remote_project_key) {
        (EditDocument::Claude(snapshot), Some(key)) => EditDocument::Claude(
            snapshot
                .with_remote_project_key(key)
                .map_err(|_| McpError::Scope)?,
        ),
        (document, _) => document,
    };
    Ok(McpEditSession {
        remote,
        platform,
        environment: environment.clone(),
        document,
        benchmark_root: source.benchmark().map(|layout| layout.root.clone()),
    })
}

pub(super) fn begin_remote(
    source: &ConfigSource,
    environment: &EnvironmentId,
    target: McpTarget,
    io: Result<Option<ccr_app::environments::io::EnvironmentIo>, ccr_app::environments::io::Error>,
    cancel: tokio_util::sync::CancellationToken,
) -> Result<McpEditSession, McpError> {
    let mut family = match target {
        McpTarget::Codex => Family::McpCodex,
        McpTarget::Antigravity => Family::McpAntigravity,
        _ => Family::McpClaude,
    };
    let io = match (&target, io.map_err(McpError::Remote)?) {
        (McpTarget::ClaudeProject(path), Some(io)) => {
            let root = ccr_app::environments::ssh_security::RemotePosixPath::root(path.to_str())
                .map_err(|_| McpError::Scope)?;
            family = Family::McpProject;
            Some(
                tokio::runtime::Handle::current()
                    .block_on(io.for_project(root, cancel.clone()))
                    .map_err(McpError::Remote)?,
            )
        }
        (McpTarget::ClaudeLocal(path), Some(io)) => {
            let root = ccr_app::environments::ssh_security::RemotePosixPath::root(path.to_str())
                .map_err(|_| McpError::Scope)?;
            tokio::runtime::Handle::current()
                .block_on(io.for_project(root, cancel.clone()))
                .map_err(McpError::Remote)?;
            Some(io)
        }
        (_, io) => io,
    };
    let remote =
        remote::capture(Ok(io), family, Default::default(), cancel).map_err(McpError::Remote)?;
    begin_at(source, environment, target, remote)
}
fn mutation_error(code: &str) -> McpError {
    match code {
        "mcp_name_exists" => McpError::Duplicate,
        "mcp_entry_missing" => McpError::Missing,
        "mcp_write_failed" => McpError::Storage,
        _ => McpError::Invalid,
    }
}

impl McpEditSession {
    pub(super) fn check_source(&self, source: &ConfigSource) -> Result<(), McpError> {
        if let Some(layout) = source.benchmark()
            && self.benchmark_root.as_deref() != Some(layout.root.as_path())
        {
            return Err(McpError::Scope);
        }
        Ok(())
    }

    /// Allowlisted ordinary metadata only. Arguments, URLs and secret maps stay
    /// inside the source-bound snapshot, even while its editor is open.
    pub fn common_fields(&self, name: &str) -> Result<McpFormFields, McpError> {
        let entry = self.entry_for_copy(name)?;
        let command = entry
            .get("command")
            .and_then(serde_json::Value::as_str)
            .filter(|value| {
                value.len() <= 120
                    && !value.is_empty()
                    && value
                        .chars()
                        .all(|c| c.is_alphanumeric() || "-_.:/\\".contains(c))
            })
            .filter(|value| {
                let lower = value.to_ascii_lowercase();
                !lower.starts_with("sk-")
                    && !lower.starts_with("eyj")
                    && !lower.contains("token")
                    && !lower.contains("bearer")
                    && !lower.contains("api_key")
            })
            .map(str::to_owned);
        let url_key = ["url", "serverUrl", "httpUrl"]
            .into_iter()
            .find(|key| entry.get(*key).is_some())
            .unwrap_or("url");
        Ok(McpFormFields { command, url_key })
    }

    #[cfg(test)]
    pub(super) fn save(
        &self,
        environment: &EnvironmentId,
        name: &str,
        mutation: EntryMutation,
    ) -> Result<(), McpError> {
        self.save_batch_with_cancel(
            environment,
            &[(name.to_owned(), mutation)],
            tokio_util::sync::CancellationToken::new(),
        )
    }

    #[cfg(test)]
    pub(super) fn save_batch(
        &self,
        environment: &EnvironmentId,
        mutations: &[(String, EntryMutation)],
    ) -> Result<(), McpError> {
        self.save_batch_with_cancel(
            environment,
            mutations,
            tokio_util::sync::CancellationToken::new(),
        )
    }
    pub(super) fn save_batch_with_cancel(
        &self,
        environment: &EnvironmentId,
        mutations: &[(String, EntryMutation)],
        cancel: tokio_util::sync::CancellationToken,
    ) -> Result<(), McpError> {
        if cancel.is_cancelled() {
            return Err(McpError::Remote(
                ccr_app::environments::io::Error::Cancelled,
            ));
        }
        if *environment != self.environment
            || (self.remote.is_none() && EnvironmentPolicy::LocalOnly.admit(environment).is_err())
        {
            return Err(McpError::Unsupported);
        }
        let saved = match &self.document {
            EditDocument::Claude(snapshot) => snapshot
                .save_batch(mutations)
                .map_err(|error| mutation_error(&error))?,
            EditDocument::Standard {
                document,
                snapshot,
                collection,
                toml,
            } => {
                let mut value = if snapshot.exists {
                    mcp_document::parse(&snapshot.content, *toml).map_err(|_| McpError::Invalid)?
                } else {
                    serde_json::json!({})
                };
                for (name, mutation) in mutations {
                    let mut mutation = mutation.clone();
                    if *toml
                        && let EntryMutation::Patch(patch) = &mut mutation
                        && let Some(incoming) = patch.get("http_headers")
                        && !incoming.is_null()
                    {
                        let incoming = incoming
                            .as_object()
                            .filter(|map| map.values().all(serde_json::Value::is_string))
                            .ok_or(McpError::Invalid)?;
                        let mut headers = value
                            .get(*collection)
                            .and_then(|rows| rows.get(name))
                            .and_then(|entry| entry.get("http_headers"))
                            .map(|stored| stored.as_object().cloned().ok_or(McpError::Invalid))
                            .transpose()?
                            .unwrap_or_default();
                        headers.extend(incoming.clone());
                        patch
                            .as_object_mut()
                            .ok_or(McpError::Invalid)?
                            .insert("http_headers".into(), serde_json::Value::Object(headers));
                    }
                    mcp_document::mutate_entries(&mut value, collection, name, &mutation)
                        .map_err(mutation_error)?;
                }
                let content =
                    mcp_document::serialize(&value, *toml).map_err(|_| McpError::Invalid)?;
                matches!(
                    document
                        .save_validated(&content, snapshot)
                        .map_err(|_| McpError::Storage)?,
                    SaveOutcome::Saved { .. }
                )
            }
        };
        if saved {
            remote::publish(&self.remote, cancel).map_err(McpError::Remote)?;
            Ok(())
        } else {
            Err(McpError::Conflict)
        }
    }
}

pub struct McpFormFields {
    pub command: Option<String>,
    pub url_key: &'static str,
}

impl Services {
    pub fn mcp_edit_request(
        &self,
        key: RequestKey,
        target: McpTarget,
    ) -> Result<TaskSubscription<Arc<McpEditSession>, McpError>, AdmissionError> {
        let scope = self.platform_scope;
        let source = self.source.clone();
        let environment = key.environment.clone();
        let io = self.resource_io(&key.environment);
        self.executor.read(key, move |cancel, _| {
            if scope.is_some_and(|platform| platform != target.platform()) {
                return Err(McpError::UnsupportedSource);
            }
            begin_remote(&source, &environment, target, io, cancel).map(Arc::new)
        })
    }

    pub fn mcp_mutate_request(
        &self,
        key: RequestKey,
        session: Arc<McpEditSession>,
        name: String,
        mutation: EntryMutation,
    ) -> Result<TaskSubscription<(), McpError>, AdmissionError> {
        let scope = self.platform_scope;
        let environment = key.environment.clone();
        let source = self.source.clone();
        self.executor.job(key, move |cancel, _| {
            if cancel.is_cancelled() {
                return Completion::Cancelled;
            }
            if scope.is_some_and(|platform| platform != session.platform) {
                return Completion::Failed(McpError::UnsupportedSource);
            }
            match session.check_source(&source).and_then(|()| {
                session.save_batch_with_cancel(&environment, &[(name, mutation)], cancel)
            }) {
                Ok(()) => Completion::Success(()),
                Err(error) => Completion::Failed(error),
            }
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;

    #[test]
    fn http_forms_keep_target_keys_omitted_credentials_and_original_revision() {
        let root = tempfile::tempdir().unwrap();
        let source = ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        };
        let environment = EnvironmentId::Local("fixture".into());
        for target in [
            McpTarget::ClaudeUser,
            McpTarget::Codex,
            McpTarget::Antigravity,
        ] {
            let url_key = if target == McpTarget::Antigravity {
                "serverUrl"
            } else {
                "url"
            };
            let header_key = if target == McpTarget::Codex {
                "http_headers"
            } else {
                "headers"
            };
            let mut entry = json!({"extension":7});
            entry[url_key] = json!("https://synthetic.invalid/mcp?token=hidden");
            entry[header_key] = json!({"Authorization":"stored-secret","KEEP":"retained"});
            begin(&source, &environment, target.clone())
                .unwrap()
                .save(&environment, "http-form", EntryMutation::Add(entry))
                .unwrap();
            let edit = begin(&source, &environment, target.clone()).unwrap();
            let fields = edit.common_fields("http-form").unwrap();
            assert!(fields.command.is_none());
            assert_eq!(fields.url_key, url_key);
            let mut patch = json!({});
            patch[header_key] = json!({"NEW":"replacement"});
            edit.save(&environment, "http-form", EntryMutation::Patch(patch))
                .unwrap();
            let latest = begin(&source, &environment, target.clone()).unwrap();
            let stored = latest.entry_for_copy("http-form").unwrap();
            assert_eq!(stored[header_key]["Authorization"], "stored-secret");
            assert_eq!(stored[header_key]["KEEP"], "retained");
            assert_eq!(stored[header_key]["NEW"], "replacement");
            assert_eq!(stored["extension"], 7);
            assert!(stored.get(url_key).is_some());
            assert_eq!(
                edit.save(
                    &environment,
                    "http-form",
                    EntryMutation::Patch(json!({"extension":8}))
                ),
                Err(McpError::Conflict)
            );
            if target == McpTarget::Codex {
                assert_eq!(
                    latest.save(
                        &environment,
                        "http-form",
                        EntryMutation::Patch(json!({"http_headers":{"BAD":9}}))
                    ),
                    Err(McpError::Invalid)
                );
                assert_eq!(
                    begin(&source, &environment, target)
                        .unwrap()
                        .entry_for_copy("http-form")
                        .unwrap(),
                    stored
                );
            }
        }
    }

    #[test]
    fn actual_mcp_writes_preserve_extensions_secrets_scopes_and_original_revision() {
        let root = tempfile::tempdir().unwrap();
        let source = ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        };
        let environment = EnvironmentId::Local("fixture".into());
        let project = root.path().join("project");
        std::fs::create_dir(&project).unwrap();
        let targets = [
            McpTarget::ClaudeUser,
            McpTarget::ClaudeLocal(project.clone()),
            McpTarget::ClaudeProject(project),
            McpTarget::Codex,
            McpTarget::Antigravity,
        ];
        for target in targets {
            let first = begin(&source, &environment, target.clone()).unwrap();
            first.save(&environment, "fixture", EntryMutation::Add(json!({"command":"tool","env":{"TOKEN":"secret-sentinel","KEEP":"yes"},"extension":{"keep":7}}))).unwrap();
            assert!(matches!(
                first.save(
                    &environment,
                    "second",
                    EntryMutation::Add(json!({"command":"other"}))
                ),
                Err(McpError::Conflict)
            ));
            let edit = begin(&source, &environment, target.clone()).unwrap();
            edit.save(
                &environment,
                "fixture",
                EntryMutation::Patch(json!({"disabled":true,"env":{"NEW":"value"}})),
            )
            .unwrap();
            assert!(matches!(
                edit.save(
                    &EnvironmentId::Ssh("other".into()),
                    "fixture",
                    EntryMutation::Delete
                ),
                Err(McpError::Unsupported)
            ));
            let current = begin(&source, &environment, target.clone()).unwrap();
            let bytes = match &current.document {
                EditDocument::Claude(snapshot) => {
                    assert!(snapshot.contains("fixture").unwrap());
                    let roots = roots(&source).unwrap();
                    let path = match &target {
                        McpTarget::ClaudeProject(path) => path.join(".mcp.json"),
                        _ => roots.claude.state_file,
                    };
                    std::fs::read_to_string(path).unwrap()
                }
                EditDocument::Standard { snapshot, .. } => snapshot.content.clone(),
            };
            for preserved in ["secret-sentinel", "KEEP", "extension", "NEW"] {
                assert!(bytes.contains(preserved), "{target:?}: missing {preserved}");
            }
            current
                .save(&environment, "fixture", EntryMutation::Delete)
                .unwrap();
        }
    }
}
