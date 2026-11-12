//! Reviewed batches keep source credentials opaque and commit each destination
//! once against its captured revision. A destination failure is not a rollback.
use super::{
    McpEditSession, McpError, McpTarget, Services,
    editor::{self, EditDocument},
    transfer,
};
use ccr_app::{
    mcp_document::{self, EntryMutation},
    runtime::{AdmissionError, Completion, EnvironmentId, RequestKey, TaskSubscription},
};
use serde_json::{Map, Value};
use std::sync::{Arc, Mutex};

pub enum BatchInput {
    Preset { id: String, api_key: Option<String> },
    Sync { source: McpTarget },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatchAction {
    Add,
    Replace,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatchOutcome {
    Pending,
    Saved,
    Failed(McpError),
    NotRun,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchRow {
    pub target: McpTarget,
    pub name: String,
    pub action: BatchAction,
    pub outcome: BatchOutcome,
}
#[derive(Clone, Debug)]
pub struct BatchSnapshot {
    pub running: bool,
    pub rows: Vec<BatchRow>,
}
pub struct BatchOperation(Mutex<BatchSnapshot>);
impl BatchOperation {
    pub fn snapshot(&self) -> BatchSnapshot {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
    fn update(&self, update: impl FnOnce(&mut BatchSnapshot)) {
        update(
            &mut self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
    }
}

struct TargetBatch {
    session: Result<McpEditSession, McpError>,
    mutations: Vec<(String, EntryMutation)>,
    rows: std::ops::Range<usize>,
}
/// No Debug/Serialize: source, destination and mutation bytes may hold secrets.
pub struct McpBatchSession {
    environment: EnvironmentId,
    source: Option<McpEditSession>,
    targets: Vec<TargetBatch>,
    preview: Vec<BatchRow>,
}
impl McpBatchSession {
    pub fn preview(&self) -> &[BatchRow] {
        &self.preview
    }
}

impl McpEditSession {
    fn entries_for_copy(&self) -> Result<Map<String, Value>, McpError> {
        match &self.document {
            EditDocument::Claude(snapshot) => {
                snapshot.entries_for_copy().map_err(|_| McpError::Invalid)
            }
            EditDocument::Standard {
                snapshot,
                collection,
                toml,
                ..
            } => {
                if !snapshot.exists {
                    return Ok(Map::new());
                }
                let root =
                    mcp_document::parse(&snapshot.content, *toml).map_err(|_| McpError::Invalid)?;
                match root.get(*collection) {
                    None => Ok(Map::new()),
                    Some(Value::Object(entries)) => Ok(entries.clone()),
                    _ => Err(McpError::Invalid),
                }
            }
        }
    }
}

fn prepare(
    services: &Services,
    environment: EnvironmentId,
    input: BatchInput,
    targets: Vec<McpTarget>,
    replace: bool,
) -> Result<McpBatchSession, McpError> {
    if !matches!(environment, EnvironmentId::Local(_)) {
        return Err(McpError::Unsupported);
    }
    if services.platform_scope.is_some() {
        return Err(McpError::UnsupportedSource);
    }
    if targets.is_empty()
        || targets
            .iter()
            .enumerate()
            .any(|(i, target)| targets[..i].contains(target))
    {
        return Err(McpError::Invalid);
    }
    let (source, entries) = match input {
        BatchInput::Sync { source } => {
            if targets.contains(&source) {
                return Err(McpError::Invalid);
            }
            let source = editor::begin(&services.source, &environment, source)?;
            let entries = source.entries_for_copy()?;
            (Some(source), entries)
        }
        BatchInput::Preset { id, api_key } => {
            let mut value = transfer::preset_patch(&id, &McpTarget::ClaudeUser)?;
            let preset = ccr_skills::get_builtin_presets()
                .into_iter()
                .find(|preset| preset.id == id)
                .ok_or(McpError::Missing)?;
            if let Some(key) = preset.api_key_env {
                let secret = api_key
                    .filter(|value| !value.trim().is_empty())
                    .ok_or(McpError::Invalid)?;
                value["env"][key] = secret.into();
            }
            (None, Map::from_iter([(id, value)]))
        }
    };
    let mut prepared = Vec::new();
    let mut preview = Vec::new();
    for target in targets {
        let session = editor::begin(&services.source, &environment, target.clone());
        let existing = session
            .as_ref()
            .map_err(|e| *e)
            .and_then(McpEditSession::entries_for_copy);
        let start = preview.len();
        let mut mutations = Vec::new();
        for (name, entry) in &entries {
            let replacing = existing
                .as_ref()
                .is_ok_and(|entries| entries.contains_key(name));
            let patch = existing
                .as_ref()
                .map_err(|e| *e)
                .and_then(|_| transfer::portable_entry(entry.clone(), &target))
                .and_then(|value| {
                    if replacing && !replace {
                        Err(McpError::Duplicate)
                    } else {
                        Ok(value)
                    }
                });
            let outcome = match patch {
                Ok(value) => {
                    if replacing {
                        mutations.push((name.clone(), EntryMutation::Delete));
                    }
                    mutations.push((name.clone(), EntryMutation::Add(value)));
                    BatchOutcome::Pending
                }
                Err(error) => BatchOutcome::Failed(error),
            };
            preview.push(BatchRow {
                target: target.clone(),
                name: name.clone(),
                action: if replacing {
                    BatchAction::Replace
                } else {
                    BatchAction::Add
                },
                outcome,
            });
        }
        prepared.push(TargetBatch {
            session,
            mutations,
            rows: start..preview.len(),
        });
    }
    Ok(McpBatchSession {
        environment,
        source,
        targets: prepared,
        preview,
    })
}

impl Services {
    pub fn mcp_batch_prepare_request(
        &self,
        key: RequestKey,
        input: BatchInput,
        targets: Vec<McpTarget>,
        replace: bool,
    ) -> Result<TaskSubscription<Arc<McpBatchSession>, McpError>, AdmissionError> {
        let services = self.clone();
        let environment = key.environment.clone();
        self.executor.read(key, move |_, _| {
            prepare(&services, environment, input, targets, replace).map(Arc::new)
        })
    }

    pub fn mcp_batch_save_request(
        &self,
        key: RequestKey,
        session: Arc<McpBatchSession>,
    ) -> Result<Arc<BatchOperation>, AdmissionError> {
        let operation = Arc::new(BatchOperation(Mutex::new(BatchSnapshot {
            running: true,
            rows: session.preview.clone(),
        })));
        let work = operation.clone();
        let finalizer = operation.clone();
        let environment = key.environment.clone();
        let config = self.source.clone();
        let scope = self.platform_scope;
        let subscription = self.executor.job_with_finalizer(
            key,
            move |cancel, _| {
                for target in &session.targets {
                    if cancel.is_cancelled() {
                        break;
                    }
                    if target.mutations.is_empty() {
                        continue;
                    }
                    let result = (|| {
                        if scope.is_some() || session.environment != environment {
                            return Err(McpError::Unsupported);
                        }
                        if let Some(source) = &session.source {
                            source.check_source(&config)?;
                        }
                        if let Ok(destination) = &target.session {
                            destination.check_source(&config)?;
                        }
                        if let Some(source) = &session.source
                            && !source.is_current()?
                        {
                            return Err(McpError::Conflict);
                        }
                        let destination = target.session.as_ref().map_err(|error| *error)?;
                        destination.save_batch(&environment, &target.mutations)
                    })();
                    work.update(|snapshot| {
                        for row in &mut snapshot.rows[target.rows.clone()] {
                            if row.outcome == BatchOutcome::Pending {
                                row.outcome = match result {
                                    Ok(()) => BatchOutcome::Saved,
                                    Err(error) => BatchOutcome::Failed(error),
                                };
                            }
                        }
                    });
                }
                Completion::<(), McpError>::Success(())
            },
            move || {
                finalizer.update(|snapshot| {
                    snapshot.running = false;
                    for row in &mut snapshot.rows {
                        if row.outcome == BatchOutcome::Pending {
                            row.outcome = BatchOutcome::NotRun;
                        }
                    }
                });
            },
        )?;
        drop(subscription); // app-owned, including when the page detaches
        Ok(operation)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::runtime::ConfigSource;
    use ccr_app::runtime::{Platform, RuntimeOwner};
    use serde_json::json;
    use std::time::Duration;

    fn key() -> RequestKey {
        RequestKey {
            environment: EnvironmentId::Local("fixture".into()),
            platform: Platform::Claude,
            generation: 0,
            request_id: 1,
        }
    }
    fn services(owner: &RuntimeOwner, root: &std::path::Path) -> Services {
        Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.join("profiles.toml"),
                registry: root.join("registry.toml"),
                delay: Duration::ZERO,
            },
        )
    }
    fn edit(services: &Services, target: McpTarget) -> McpEditSession {
        editor::begin(&services.source, &key().environment, target).unwrap()
    }
    async fn finished(operation: Arc<BatchOperation>) -> BatchSnapshot {
        tokio::time::timeout(Duration::from_secs(10), async move {
            loop {
                let value = operation.snapshot();
                if !value.running {
                    return value;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap()
    }
    async fn shutdown(owner: RuntimeOwner) {
        tokio::task::spawn_blocking(move || owner.shutdown())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn platform_authority_rejects_forged_batch_and_global_plan_without_writes() {
        let owner = RuntimeOwner::new().unwrap();
        let root = tempfile::tempdir().unwrap();
        let services = services(&owner, root.path());
        edit(&services, McpTarget::ClaudeUser)
            .save(
                &key().environment,
                "one",
                EntryMutation::Add(json!({"command":"synthetic"})),
            )
            .unwrap();
        let scoped = services.clone().scoped(Some(Platform::Claude));
        assert!(matches!(
            prepare(
                &scoped,
                key().environment.clone(),
                BatchInput::Sync {
                    source: McpTarget::ClaudeUser
                },
                vec![McpTarget::Codex],
                false
            ),
            Err(McpError::UnsupportedSource)
        ));
        let plan = prepare(
            &services,
            key().environment.clone(),
            BatchInput::Sync {
                source: McpTarget::ClaudeUser,
            },
            vec![McpTarget::Codex],
            false,
        )
        .unwrap();
        let mut forged = key();
        forged.platform = Platform::Codex;
        let result = finished(
            scoped
                .mcp_batch_save_request(forged, Arc::new(plan))
                .unwrap(),
        )
        .await;
        assert!(
            result
                .rows
                .iter()
                .all(|row| row.outcome == BatchOutcome::Failed(McpError::Unsupported))
        );
        assert!(
            edit(&services, McpTarget::Codex)
                .entries_for_copy()
                .unwrap()
                .is_empty()
        );
        let plan = prepare(
            &services,
            key().environment.clone(),
            BatchInput::Sync {
                source: McpTarget::ClaudeUser,
            },
            vec![McpTarget::Codex],
            false,
        )
        .unwrap();
        let result = finished(
            services
                .mcp_batch_save_request(key(), Arc::new(plan))
                .unwrap(),
        )
        .await;
        assert!(
            result
                .rows
                .iter()
                .all(|row| row.outcome == BatchOutcome::Saved)
        );
        shutdown(owner).await;
    }

    #[tokio::test]
    async fn source_and_target_revisions_bind_whole_batches_with_truthful_partial_results() {
        let owner = RuntimeOwner::new().unwrap();
        let root = tempfile::tempdir().unwrap();
        let services = services(&owner, root.path());
        edit(&services, McpTarget::ClaudeUser)
            .save_batch(
                &key().environment,
                &[
                    (
                        "one".into(),
                        EntryMutation::Add(
                            json!({"command":"tool-one","env":{"TOKEN":"batch-secret-sentinel"}}),
                        ),
                    ),
                    (
                        "two".into(),
                        EntryMutation::Add(json!({"command":"tool-two"})),
                    ),
                    (
                        "unknown".into(),
                        EntryMutation::Add(json!({"command":"tool-unknown","trust":true})),
                    ),
                ],
            )
            .unwrap();
        let session = prepare(
            &services,
            key().environment.clone(),
            BatchInput::Sync {
                source: McpTarget::ClaudeUser,
            },
            vec![McpTarget::Codex, McpTarget::Antigravity],
            false,
        )
        .unwrap();
        assert!(!format!("{:?}", session.preview()).contains("batch-secret-sentinel"));
        // A concurrent target change must fail the complete reviewed target,
        // rather than refreshing its token between imported entries.
        edit(&services, McpTarget::Codex)
            .save(
                &key().environment,
                "unrelated",
                EntryMutation::Add(json!({"command":"keep"})),
            )
            .unwrap();
        let result = finished(
            services
                .mcp_batch_save_request(key(), Arc::new(session))
                .unwrap(),
        )
        .await;
        assert_eq!(
            result
                .rows
                .iter()
                .filter(|row| row.target == McpTarget::Codex
                    && row.outcome == BatchOutcome::Failed(McpError::Conflict))
                .count(),
            2
        );
        assert_eq!(
            result
                .rows
                .iter()
                .filter(|row| row.target == McpTarget::Antigravity
                    && row.outcome == BatchOutcome::Saved)
                .count(),
            2
        );
        assert_eq!(
            result
                .rows
                .iter()
                .filter(|row| row.outcome == BatchOutcome::Failed(McpError::UnsupportedCopy))
                .count(),
            2
        );
        assert!(!format!("{result:?}").contains("batch-secret-sentinel"));
        let copied = edit(&services, McpTarget::Antigravity)
            .entries_for_copy()
            .unwrap();
        assert_eq!(copied["one"]["env"]["TOKEN"], "batch-secret-sentinel");
        assert_eq!(copied["two"]["command"], "tool-two");
        assert!(!copied.contains_key("unknown"));
        let codex = edit(&services, McpTarget::Codex)
            .entries_for_copy()
            .unwrap();
        assert_eq!(codex.len(), 1);
        assert!(codex.contains_key("unrelated"));
        let session = prepare(
            &services,
            key().environment.clone(),
            BatchInput::Sync {
                source: McpTarget::ClaudeUser,
            },
            vec![McpTarget::Codex],
            false,
        )
        .unwrap();
        edit(&services, McpTarget::ClaudeUser)
            .save(
                &key().environment,
                "one",
                EntryMutation::Patch(json!({"command":"changed"})),
            )
            .unwrap();
        let stale = finished(
            services
                .mcp_batch_save_request(key(), Arc::new(session))
                .unwrap(),
        )
        .await;
        assert!(
            stale
                .rows
                .iter()
                .filter(|row| row.outcome != BatchOutcome::Failed(McpError::UnsupportedCopy))
                .all(|row| row.outcome == BatchOutcome::Failed(McpError::Conflict))
        );
        assert_eq!(
            edit(&services, McpTarget::Codex)
                .entries_for_copy()
                .unwrap()
                .len(),
            1
        );
        shutdown(owner).await;
    }

    #[tokio::test]
    async fn replacement_is_explicit_and_preserves_destination_siblings() {
        let owner = RuntimeOwner::new().unwrap();
        let root = tempfile::tempdir().unwrap();
        let services = services(&owner, root.path());
        edit(&services, McpTarget::ClaudeUser)
            .save(
                &key().environment,
                "same",
                EntryMutation::Add(json!({"command":"new-tool"})),
            )
            .unwrap();
        edit(&services,McpTarget::Codex).save_batch(&key().environment,&[
            ("same".into(),EntryMutation::Add(json!({"command":"old-tool","approval_policy":"keep-unless-explicit-replace"}))),
            ("sibling".into(),EntryMutation::Add(json!({"command":"keep"}))),
        ]).unwrap();
        let session = prepare(
            &services,
            key().environment.clone(),
            BatchInput::Sync {
                source: McpTarget::ClaudeUser,
            },
            vec![McpTarget::Codex],
            false,
        )
        .unwrap();
        assert_eq!(
            session.preview[0].outcome,
            BatchOutcome::Failed(McpError::Duplicate)
        );
        let result = finished(
            services
                .mcp_batch_save_request(key(), Arc::new(session))
                .unwrap(),
        )
        .await;
        assert_eq!(
            result.rows[0].outcome,
            BatchOutcome::Failed(McpError::Duplicate)
        );
        assert!(
            edit(&services, McpTarget::Codex)
                .entries_for_copy()
                .unwrap()["same"]
                .get("approval_policy")
                .is_some()
        );
        let session = prepare(
            &services,
            key().environment.clone(),
            BatchInput::Sync {
                source: McpTarget::ClaudeUser,
            },
            vec![McpTarget::Codex],
            true,
        )
        .unwrap();
        assert_eq!(session.preview[0].action, BatchAction::Replace);
        let result = finished(
            services
                .mcp_batch_save_request(key(), Arc::new(session))
                .unwrap(),
        )
        .await;
        assert_eq!(result.rows[0].outcome, BatchOutcome::Saved);
        let entries = edit(&services, McpTarget::Codex)
            .entries_for_copy()
            .unwrap();
        assert_eq!(entries["same"]["command"], "new-tool");
        assert!(entries["same"].get("approval_policy").is_none());
        assert_eq!(entries["sibling"]["command"], "keep");
        shutdown(owner).await;
    }

    #[tokio::test]
    async fn preset_installs_multiple_targets_with_scoped_secret_and_environment_refusal() {
        let owner = RuntimeOwner::new().unwrap();
        let root = tempfile::tempdir().unwrap();
        let services = services(&owner, root.path());
        let preset = ccr_skills::get_builtin_presets()
            .into_iter()
            .find(|preset| preset.api_key_env.is_some())
            .unwrap();
        let targets = vec![
            McpTarget::ClaudeUser,
            McpTarget::Codex,
            McpTarget::Antigravity,
        ];
        assert!(matches!(
            prepare(
                &services,
                key().environment.clone(),
                BatchInput::Preset {
                    id: preset.id.clone(),
                    api_key: None
                },
                targets.clone(),
                false
            ),
            Err(McpError::Invalid)
        ));
        let session = prepare(
            &services,
            key().environment.clone(),
            BatchInput::Preset {
                id: preset.id.clone(),
                api_key: Some("preset-secret-sentinel".into()),
            },
            targets.clone(),
            false,
        )
        .unwrap();
        let result = finished(
            services
                .mcp_batch_save_request(key(), Arc::new(session))
                .unwrap(),
        )
        .await;
        assert_eq!(result.rows.len(), 3);
        assert!(
            result
                .rows
                .iter()
                .all(|row| row.outcome == BatchOutcome::Saved)
        );
        for target in targets {
            assert_eq!(
                edit(&services, target).entries_for_copy().unwrap()[&preset.id]["env"]
                    [preset.api_key_env.as_ref().unwrap()],
                "preset-secret-sentinel"
            );
        }
        let remote = EnvironmentId::Ssh("remote".into());
        assert!(matches!(
            prepare(
                &services,
                remote,
                BatchInput::Sync {
                    source: McpTarget::ClaudeUser
                },
                vec![McpTarget::Codex],
                false
            ),
            Err(McpError::Unsupported)
        ));
        shutdown(owner).await;
    }
}
