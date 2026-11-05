//! Dirty-leaf JSON settings and OpenCode provider composition.
use super::{Document, Error, SaveOutcome, Snapshot};
use serde_json::Value;

fn invalid() -> Error {
    Error::Invalid {
        line: None,
        column: None,
    }
}

pub fn parse(snapshot: &Snapshot) -> Result<Value, Error> {
    if !snapshot.exists {
        return Ok(serde_json::json!({}));
    }
    let value: Value = serde_json::from_str(&snapshot.content).map_err(|_| invalid())?;
    if !value.is_object() {
        return Err(invalid());
    }
    Ok(value)
}

/// Only submitted leaves change; omitted siblings and extension fields survive.
/// JSON settings have no implicit null/removal protocol.
pub fn save_patch(
    document: &Document,
    snapshot: &Snapshot,
    patch: &Value,
) -> Result<SaveOutcome, Error> {
    if !patch.is_object() {
        return Err(invalid());
    }
    let mut current = parse(snapshot)?;
    merge(&mut current, patch)?;
    save(document, snapshot, &current)
}

fn merge(current: &mut Value, patch: &Value) -> Result<(), Error> {
    let incoming = patch.as_object().ok_or_else(invalid)?;
    let target = current.as_object_mut().ok_or_else(invalid)?;
    for (key, value) in incoming {
        if value.is_null() {
            return Err(invalid());
        }
        if value.is_object() && !value.as_object().is_none_or(|value| value.is_empty()) {
            let child = target
                .entry(key.clone())
                .or_insert_with(|| serde_json::json!({}));
            merge(child, value)?;
        } else {
            target.insert(key.clone(), value.clone());
        }
    }
    Ok(())
}

fn save(document: &Document, snapshot: &Snapshot, value: &Value) -> Result<SaveOutcome, Error> {
    let content = serde_json::to_string_pretty(value).map_err(|_| invalid())?;
    document.save_validated(&content, snapshot)
}

/// OpenCode canonical `provider` wins over legacy `providers`, matching its existing client.
pub fn providers(snapshot: &Snapshot) -> Result<Value, Error> {
    let settings = parse(snapshot)?;
    let value = settings
        .get("provider")
        .or_else(|| settings.get("providers"));
    match value {
        None => Ok(serde_json::json!({})),
        Some(value) if value.is_object() => Ok(value.clone()),
        _ => Err(invalid()),
    }
}

/// Provider edits merge supplied leaves; deletion removes one ID, not the provider collection.
pub fn save_provider(
    document: &Document,
    snapshot: &Snapshot,
    id: &str,
    patch: Option<&Value>,
) -> Result<SaveOutcome, Error> {
    if id.trim().is_empty() {
        return Err(invalid());
    }
    let mut settings = parse(snapshot)?;
    let mut collection = providers(snapshot)?;
    let map = collection.as_object_mut().ok_or_else(invalid)?;
    match patch {
        Some(patch) => {
            let provider = map
                .entry(id.to_owned())
                .or_insert_with(|| serde_json::json!({}));
            merge(provider, patch)?;
        }
        None => {
            map.remove(id);
        }
    }
    settings
        .as_object_mut()
        .ok_or_else(invalid)?
        .insert("provider".into(), collection);
    save(document, snapshot, &settings)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::platform_settings::Format;
    use ccr_core::core::BackupPolicy;
    #[test]
    fn dirty_nested_leaves_provider_crud_and_original_revision_are_lossless() {
        let temp = tempfile::tempdir().unwrap();
        let document = Document::new(
            temp.path().join("opencode.json"),
            None,
            BackupPolicy::None,
            Some(temp.path().join("locks")),
            Format::Json,
        );
        std::fs::write(document.path(), r#"{"unknown":{"retain":7},"tui":{"theme":"dark","extension":true},"providers":{"fixture":{"options":{"apiKey":"secret-sentinel","extension":9},"models":{"kept":{}}}}}"#).unwrap();
        let original = document.read().unwrap();
        save_patch(
            &document,
            &original,
            &serde_json::json!({"tui":{"theme":"light"}}),
        )
        .unwrap();
        assert_eq!(
            parse(&document.read().unwrap()).unwrap()["tui"]["extension"],
            true
        );
        assert_eq!(
            save_provider(&document, &original, "stale", Some(&serde_json::json!({}))).unwrap(),
            SaveOutcome::Conflict
        );
        let snapshot = document.read().unwrap();
        save_provider(
            &document,
            &snapshot,
            "fixture",
            Some(&serde_json::json!({"options":{"baseURL":"https://fixture.invalid"}})),
        )
        .unwrap();
        let snapshot = document.read().unwrap();
        let collection = providers(&snapshot).unwrap();
        assert_eq!(
            collection["fixture"]["options"]["apiKey"],
            "secret-sentinel"
        );
        assert_eq!(collection["fixture"]["options"]["extension"], 9);
        assert!(collection["fixture"]["models"]["kept"].is_object());
        save_provider(&document, &snapshot, "fixture", None).unwrap();
        let snapshot = document.read().unwrap();
        assert_eq!(providers(&snapshot).unwrap(), serde_json::json!({}));
        assert_eq!(parse(&snapshot).unwrap()["unknown"]["retain"], 7);
        assert!(save_patch(&document, &snapshot, &serde_json::json!({"tui":null})).is_err());
    }
}
