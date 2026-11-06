//! Typed subset of the existing Tauri Grok settings contract.
use super::*;

pub(super) fn patch(
    session: &PlatformSession,
    patch: &Value,
) -> Result<SaveOutcome, PlatformError> {
    let mut config = if session.snapshot.exists {
        toml::from_str::<toml::Value>(&session.snapshot.content)
            .map_err(|_| PlatformError::Invalid)?
    } else {
        toml::Value::Table(Default::default())
    };
    let root = config.as_table_mut().ok_or(PlatformError::Invalid)?;
    for (section, values) in patch.as_object().ok_or(PlatformError::Invalid)? {
        let values = values.as_object().ok_or(PlatformError::Invalid)?;
        for (key, value) in values {
            let valid = match (section.as_str(), key.as_str()) {
                ("models", "default") => value.as_str().is_some_and(|v| !v.trim().is_empty()),
                ("models", "default_reasoning_effort") => value.as_str().is_some_and(|v| {
                    matches!(
                        v,
                        "none" | "minimal" | "low" | "medium" | "high" | "xhigh" | "max"
                    )
                }),
                ("ui", "theme") => value
                    .as_str()
                    .is_some_and(|v| matches!(v, "system" | "light" | "dark")),
                ("cli", "channel") => value
                    .as_str()
                    .is_some_and(|v| matches!(v, "stable" | "alpha")),
                ("cli", "auto_update" | "show_tips") | ("session", "load_envrc") => {
                    value.is_boolean()
                }
                ("session", "auto_compact_threshold_percent") => {
                    value.as_i64().is_some_and(|v| (0..=100).contains(&v))
                }
                ("hints", "new_session_worktree_mode" | "fork_worktree_mode") => value
                    .as_str()
                    .is_some_and(|v| matches!(v, "ask" | "always" | "never")),
                _ => false,
            };
            if !valid {
                return Err(PlatformError::Invalid);
            }
            let target = root
                .entry(section.clone())
                .or_insert_with(|| toml::Value::Table(Default::default()))
                .as_table_mut()
                .ok_or(PlatformError::Invalid)?;
            target.insert(
                key.clone(),
                match value {
                    Value::String(v) => toml::Value::String(v.clone()),
                    Value::Bool(v) => toml::Value::Boolean(*v),
                    Value::Number(v) => {
                        toml::Value::Integer(v.as_i64().ok_or(PlatformError::Invalid)?)
                    }
                    _ => return Err(PlatformError::Invalid),
                },
            );
        }
    }
    let content = toml::to_string_pretty(&config).map_err(|_| PlatformError::Invalid)?;
    grok_managed_guard(session, &content)?;
    session
        .document
        .save_text(&content, &session.snapshot)
        .map_err(settings_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn session(source: &ConfigSource) -> Arc<PlatformSession> {
        match super::super::read(source, Platform::Grok, Surface::Settings)
            .unwrap_or_else(|e| panic!("read: {e:?}"))
        {
            ResultValue::Loaded(value) => value,
            _ => panic!("loaded session"),
        }
    }
    #[test]
    fn grok_form_patch_validates_types_preserves_unknown_and_enforces_managed_cas() {
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let source = ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: std::time::Duration::ZERO,
        };
        let roots = roots(&source).unwrap_or_else(|e| panic!("roots: {e:?}"));
        std::fs::create_dir_all(&roots.grok_runtime).unwrap_or_else(|e| panic!("mkdir: {e}"));
        let path = roots.grok_runtime.join("config.toml");
        let original = "extension=7\n[models]\ndefault='synthetic'\n[cli]\nauto_update=false\n";
        std::fs::write(&path, original).unwrap_or_else(|e| panic!("write: {e}"));
        let captured = session(&source);
        for value in [
            serde_json::json!({"session":{"auto_compact_threshold_percent":101}}),
            serde_json::json!({"cli":{"auto_update":"true"}}),
            serde_json::json!({"models":{"secret":"forbidden"}}),
        ] {
            assert_eq!(patch(&captured, &value), Err(PlatformError::Invalid));
            assert_eq!(std::fs::read_to_string(&path).unwrap_or_default(), original);
        }
        assert!(matches!(
            patch(&captured, &serde_json::json!({"cli":{"auto_update":true}})),
            Ok(SaveOutcome::Saved { .. })
        ));
        let saved = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(saved.contains("extension = 7"));
        assert!(saved.contains("auto_update = true"));
        assert_eq!(
            patch(&captured, &serde_json::json!({"ui":{"theme":"dark"}})),
            Ok(SaveOutcome::Conflict)
        );
        std::fs::write(&path,"[models]\ndefault='synthetic'\n[model.custom]\nmodel='synthetic'\nbase_url='https://synthetic.invalid'\napi_backend='responses'\napi_key='synthetic-secret'\n").unwrap_or_else(|e|panic!("write: {e}"));
        let captured = session(&source);
        let before = std::fs::read(&path).unwrap_or_default();
        assert_eq!(
            patch(
                &captured,
                &serde_json::json!({"models":{"default":"replacement"}})
            ),
            Err(PlatformError::ManagedLocked)
        );
        assert_eq!(std::fs::read(&path).unwrap_or_default(), before);
        assert!(matches!(
            patch(&captured, &serde_json::json!({"ui":{"theme":"dark"}})),
            Ok(SaveOutcome::Saved { .. })
        ));
    }
}
