//! Explicit MCP copies keep credentials in backend snapshots, never UI drafts.
use super::{
    McpEditSession, McpEntry, McpError, McpTarget, Services,
    editor::{self, EditDocument},
};
use ccr_app::{
    mcp_document::{self, EntryMutation},
    runtime::{AdmissionError, Completion, EnvironmentId, RequestKey, TaskSubscription},
};
use ccr_skills::McpServerSpec;
use serde_json::{Value, json};
use std::sync::Arc;

pub struct McpCopySession {
    source: McpEditSession,
    target: McpEditSession,
    entry: Value,
}

fn spec_patch(spec: &McpServerSpec, target: &McpTarget) -> Result<Value, McpError> {
    let command = spec
        .command
        .as_deref()
        .filter(|value| !value.trim().is_empty());
    let url = spec.url.as_deref().filter(|value| !value.trim().is_empty());
    if command.is_some() == url.is_some()
        || (url.is_some() && spec.server_type != "http")
        || (command.is_some() && spec.server_type != "stdio")
    {
        return Err(McpError::UnsupportedCopy);
    }
    let mut value = json!({});
    if let Some(command) = command {
        value["command"] = command.into();
        if !spec.args.is_empty() {
            value["args"] = json!(spec.args);
        }
    } else if let Some(url) = url {
        if !spec.args.is_empty() || !spec.env.is_empty() || *target == McpTarget::Antigravity {
            return Err(McpError::UnsupportedCopy);
        }
        value["url"] = url.into();
        if *target != McpTarget::Codex {
            value["type"] = "http".into();
        }
    }
    if !spec.env.is_empty() {
        value["env"] = json!(spec.env);
    }
    Ok(value)
}

/// The existing pure catalog supplies templates; its legacy HOME-bound writer
/// is deliberately not used. Only the native guarded writer commits a draft.
pub fn preset_patch(id: &str, target: &McpTarget) -> Result<Value, McpError> {
    let preset = ccr_skills::get_builtin_presets()
        .into_iter()
        .find(|preset| preset.id == id)
        .ok_or(McpError::Missing)?;
    let mut value = spec_patch(&preset.server, target)?;
    if let Some(key) = preset.api_key_env {
        if !value.get("env").is_some_and(Value::is_object) {
            value["env"] = json!({});
        }
        value["env"][key] = "".into();
    }
    Ok(value)
}

pub(super) fn portable_entry(value: Value, target: &McpTarget) -> Result<Value, McpError> {
    let mut fields = value.as_object().cloned().ok_or(McpError::Invalid)?;
    fields.retain(|_, value| !value.is_null());
    // Unknown options may change authentication, approval or tool access. Never
    // silently drop them or pretend they use an equivalent target schema.
    if fields.keys().any(|key| {
        !matches!(
            key.as_str(),
            "command"
                | "args"
                | "env"
                | "cwd"
                | "url"
                | "type"
                | "headers"
                | "http_headers"
                | "enabled"
                | "disabled"
        )
    }) {
        return Err(McpError::UnsupportedCopy);
    }
    if let Some(headers) = fields.remove("http_headers")
        && fields.insert("headers".into(), headers).is_some()
    {
        return Err(McpError::UnsupportedCopy);
    }
    let http = fields.contains_key("url");
    fields
        .entry("type")
        .or_insert_with(|| if http { "http".into() } else { "stdio".into() });
    let value = Value::Object(fields);
    mcp_document::validate_entry(&value).map_err(|_| McpError::Invalid)?;
    let spec: McpServerSpec =
        serde_json::from_value(value.clone()).map_err(|_| McpError::Invalid)?;
    let mut patch = spec_patch(&spec, target)?;
    if let Some(cwd) = value.get("cwd") {
        if http {
            return Err(McpError::UnsupportedCopy);
        }
        patch["cwd"] = cwd.clone();
    }
    if let Some(headers) = value.get("headers") {
        if !http || *target == McpTarget::Antigravity {
            return Err(McpError::UnsupportedCopy);
        }
        patch[if *target == McpTarget::Codex {
            "http_headers"
        } else {
            "headers"
        }] = headers.clone();
    }
    let enabled = value.get("enabled").and_then(Value::as_bool);
    let disabled = value.get("disabled").and_then(Value::as_bool);
    if matches!((enabled, disabled), (Some(enabled), Some(disabled)) if enabled == disabled) {
        return Err(McpError::UnsupportedCopy);
    }
    if let Some(enabled) = enabled.or(disabled.map(|disabled| !disabled)) {
        if *target == McpTarget::Codex {
            patch["enabled"] = enabled.into();
        } else {
            patch["disabled"] = (!enabled).into();
        }
    }
    Ok(patch)
}

impl McpEditSession {
    pub(super) fn entry_for_copy(&self, name: &str) -> Result<Value, McpError> {
        let value = match &self.document {
            EditDocument::Claude(snapshot) => snapshot
                .entry_for_copy(name)
                .map_err(|_| McpError::Invalid)?,
            EditDocument::Standard {
                snapshot,
                collection,
                toml,
                ..
            } => {
                let value =
                    mcp_document::parse(&snapshot.content, *toml).map_err(|_| McpError::Invalid)?;
                value
                    .get(*collection)
                    .and_then(|entries| entries.get(name))
                    .cloned()
            }
        };
        value.ok_or(McpError::Missing)
    }

    pub(super) fn is_current(&self) -> Result<bool, McpError> {
        match &self.document {
            EditDocument::Claude(snapshot) => snapshot.is_current().map_err(|_| McpError::Storage),
            EditDocument::Standard {
                document, snapshot, ..
            } => Ok(document.read().map_err(|_| McpError::Storage)?.token == snapshot.token),
        }
    }
}

impl McpCopySession {
    fn save(&self, environment: &EnvironmentId, name: &str) -> Result<(), McpError> {
        if self.source.environment != *environment {
            return Err(McpError::Unsupported);
        }
        if !self.source.is_current()? {
            return Err(McpError::Conflict);
        }
        self.target
            .save(environment, name, EntryMutation::Add(self.entry.clone()))
    }
}

impl Services {
    pub fn mcp_copy_prepare_request(
        &self,
        key: RequestKey,
        source: McpEntry,
        target: McpTarget,
    ) -> Result<TaskSubscription<Arc<McpCopySession>, McpError>, AdmissionError> {
        let scope = self.platform_scope;
        let config = self.source.clone();
        let environment = key.environment.clone();
        self.executor.read(key, move |_, _| {
            if scope.is_some_and(|platform| {
                target.platform() != platform
                    || !McpTarget::for_entry(&source)
                        .is_ok_and(|source| source.platform() == platform)
            }) {
                return Err(McpError::UnsupportedSource);
            }
            let source_session =
                editor::begin(&config, &environment, McpTarget::for_entry(&source)?)?;
            let entry = portable_entry(source_session.entry_for_copy(&source.name)?, &target)?;
            let target = editor::begin(&config, &environment, target)?;
            Ok(Arc::new(McpCopySession {
                source: source_session,
                target,
                entry,
            }))
        })
    }

    pub fn mcp_copy_save_request(
        &self,
        key: RequestKey,
        session: Arc<McpCopySession>,
        name: String,
    ) -> Result<TaskSubscription<(), McpError>, AdmissionError> {
        let scope = self.platform_scope;
        let environment = key.environment.clone();
        let source = self.source.clone();
        self.executor.job(key, move |cancel, _| {
            if cancel.is_cancelled() {
                return Completion::Cancelled;
            }
            if scope.is_some_and(|platform| {
                session.source.platform != platform || session.target.platform != platform
            }) {
                return Completion::Failed(McpError::UnsupportedSource);
            }
            match session
                .source
                .check_source(&source)
                .and_then(|()| session.target.check_source(&source))
                .and_then(|()| session.save(&environment, &name))
            {
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
    use crate::runtime::{
        ConfigSource,
        mcp::{McpScope, McpSource},
    };
    use ccr_app::runtime::{Platform, RuntimeOwner, TaskEvent};
    use std::time::Duration;

    async fn terminal<T, E>(mut task: TaskSubscription<T, E>) -> Arc<Completion<T, E>> {
        loop {
            if let Some(TaskEvent::Terminal(result)) = task.next().await {
                return result;
            }
        }
    }

    #[tokio::test]
    async fn copy_keeps_credentials_backend_only_and_checks_both_source_and_target_revisions() {
        let root = tempfile::tempdir().unwrap();
        let source = ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        };
        let owner = RuntimeOwner::new().unwrap();
        let environment = EnvironmentId::Local("fixture".into());
        let services = Services::new(owner.handle(), source.clone());
        let key = RequestKey {
            environment: environment.clone(),
            platform: Platform::Claude,
            generation: 0,
            request_id: 0,
        };
        editor::begin(&source,&environment,McpTarget::ClaudeUser).unwrap().save(&environment,"source",EntryMutation::Add(json!({"command":"tool","args":["--flag"],"env":{"TOKEN":"secret-sentinel"},"disabled":true}))).unwrap();
        let row = McpEntry {
            source: McpSource::ClaudeState,
            name: "source".into(),
            scope: McpScope::User,
            command: Some("tool".into()),
            has_url: false,
            has_args: true,
            disabled: Some(true),
            enabled: None,
        };
        let prepared = terminal(
            services
                .mcp_copy_prepare_request(key.clone(), row.clone(), McpTarget::Codex)
                .unwrap(),
        )
        .await;
        let Completion::Success(copy) = prepared.as_ref() else {
            panic!("copy preparation failed");
        };
        let copied = terminal(
            services
                .mcp_copy_save_request(key.clone(), copy.clone(), "copied".into())
                .unwrap(),
        )
        .await;
        assert!(matches!(copied.as_ref(), Completion::Success(())));
        let codex = editor::begin(&source, &environment, McpTarget::Codex).unwrap();
        let value = codex.entry_for_copy("copied").unwrap();
        assert_eq!(value["env"]["TOKEN"], "secret-sentinel");
        assert_eq!(value["enabled"], false);
        assert!(value.get("disabled").is_none());
        let stale_source = terminal(
            services
                .mcp_copy_prepare_request(key.clone(), row.clone(), McpTarget::Codex)
                .unwrap(),
        )
        .await;
        let Completion::Success(copy) = stale_source.as_ref() else {
            panic!("copy preparation failed");
        };
        editor::begin(&source, &environment, McpTarget::ClaudeUser)
            .unwrap()
            .save(
                &environment,
                "source",
                EntryMutation::Patch(json!({"command":"changed"})),
            )
            .unwrap();
        assert!(matches!(
            terminal(
                services
                    .mcp_copy_save_request(key.clone(), copy.clone(), "stale-source".into())
                    .unwrap()
            )
            .await
            .as_ref(),
            Completion::Failed(McpError::Conflict)
        ));
        let stale_target = terminal(
            services
                .mcp_copy_prepare_request(key.clone(), row, McpTarget::Codex)
                .unwrap(),
        )
        .await;
        let Completion::Success(copy) = stale_target.as_ref() else {
            panic!("copy preparation failed");
        };
        editor::begin(&source, &environment, McpTarget::Codex)
            .unwrap()
            .save(
                &environment,
                "unrelated",
                EntryMutation::Add(json!({"command":"keep"})),
            )
            .unwrap();
        assert!(matches!(
            terminal(
                services
                    .mcp_copy_save_request(key, copy.clone(), "stale-target".into())
                    .unwrap()
            )
            .await
            .as_ref(),
            Completion::Failed(McpError::Conflict)
        ));
        let current = editor::begin(&source, &environment, McpTarget::Codex).unwrap();
        assert!(matches!(
            current.entry_for_copy("stale-source"),
            Err(McpError::Missing)
        ));
        assert!(matches!(
            current.entry_for_copy("stale-target"),
            Err(McpError::Missing)
        ));
        assert_eq!(
            current.entry_for_copy("unrelated").unwrap()["command"],
            "keep"
        );
        tokio::task::spawn_blocking(move || owner.shutdown())
            .await
            .unwrap();
    }

    #[test]
    fn transport_mapping_preserves_http_headers_and_refuses_unmapped_options() {
        let value=portable_entry(json!({"url":"https://fixture.invalid/mcp","type":"http","headers":{"Authorization":"secret-sentinel"}}),&McpTarget::Codex).unwrap();
        assert_eq!(value["http_headers"]["Authorization"], "secret-sentinel");
        assert!(value.get("headers").is_none());
        assert!(value.get("type").is_none());
        assert_eq!(
            portable_entry(value, &McpTarget::ClaudeUser).unwrap()["headers"]["Authorization"],
            "secret-sentinel"
        );
        for value in [
            json!({"url":"https://fixture.invalid","type":"sse"}),
            json!({"serverUrl":"https://fixture.invalid"}),
            json!({"command":"tool","trust":true}),
            json!({"url":"https://fixture.invalid","bearer_token_env_var":"TOKEN"}),
        ] {
            assert!(matches!(
                portable_entry(value, &McpTarget::Codex),
                Err(McpError::UnsupportedCopy)
            ));
        }
        assert!(matches!(
            portable_entry(
                json!({"url":"https://fixture.invalid","type":"http"}),
                &McpTarget::Antigravity
            ),
            Err(McpError::UnsupportedCopy)
        ));
    }
}
