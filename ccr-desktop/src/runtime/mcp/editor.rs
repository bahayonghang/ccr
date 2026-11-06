//! Source-bound MCP edit sessions. Opaque snapshots never expose stored secrets to UI.
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

enum EditDocument {
    Claude(NativeSnapshot),
    Standard {
        document: Document,
        snapshot: Snapshot,
        collection: &'static str,
        toml: bool,
    },
}

pub struct McpEditSession {
    environment: EnvironmentId,
    document: EditDocument,
}

pub(super) fn project_path(source: &ConfigSource, path: &Path) -> Result<PathBuf, McpError> {
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

fn begin(
    source: &ConfigSource,
    environment: &EnvironmentId,
    target: McpTarget,
) -> Result<McpEditSession, McpError> {
    if !matches!(environment, EnvironmentId::Local(_)) {
        return Err(McpError::Unsupported);
    }
    let roots = roots(source)?;
    let document = match target {
        McpTarget::ClaudeUser | McpTarget::ClaudeLocal(_) | McpTarget::ClaudeProject(_) => {
            let (scope, project_root) = match target {
                McpTarget::ClaudeUser => (ClaudeMcpScope::User, roots.home.clone()),
                McpTarget::ClaudeLocal(path) => {
                    (ClaudeMcpScope::Local, project_path(source, &path)?)
                }
                McpTarget::ClaudeProject(path) => {
                    (ClaudeMcpScope::Project, project_path(source, &path)?)
                }
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
    Ok(McpEditSession {
        environment: environment.clone(),
        document,
    })
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
    fn save(
        &self,
        environment: &EnvironmentId,
        name: &str,
        mutation: EntryMutation,
    ) -> Result<(), McpError> {
        if *environment != self.environment || !matches!(environment, EnvironmentId::Local(_)) {
            return Err(McpError::Unsupported);
        }
        let saved = match &self.document {
            EditDocument::Claude(snapshot) => snapshot
                .save(name, &mutation)
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
                mcp_document::mutate_entries(&mut value, collection, name, &mutation)
                    .map_err(mutation_error)?;
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
            Ok(())
        } else {
            Err(McpError::Conflict)
        }
    }
}

impl Services {
    pub fn mcp_edit_request(
        &self,
        key: RequestKey,
        target: McpTarget,
    ) -> Result<TaskSubscription<Arc<McpEditSession>, McpError>, AdmissionError> {
        let source = self.source.clone();
        let environment = key.environment.clone();
        self.executor.read(key, move |_, _| {
            begin(&source, &environment, target).map(Arc::new)
        })
    }

    pub fn mcp_mutate_request(
        &self,
        key: RequestKey,
        session: Arc<McpEditSession>,
        name: String,
        mutation: EntryMutation,
    ) -> Result<TaskSubscription<(), McpError>, AdmissionError> {
        let environment = key.environment.clone();
        self.executor.job(key, move |cancel, _| {
            if cancel.is_cancelled() {
                return Completion::Cancelled;
            }
            match session.save(&environment, &name, mutation) {
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
