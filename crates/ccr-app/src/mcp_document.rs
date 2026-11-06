//! The existing unified MCP JSON/TOML codec, shared by desktop adapters.
use serde_json::Value;

pub fn parse(content: &str, toml_format: bool) -> Result<Value, String> {
    if toml_format {
        let value: toml::Value = toml::from_str(content).map_err(|e| format!("Parse TOML: {e}"))?;
        serde_json::to_value(value).map_err(|e| format!("Convert TOML: {e}"))
    } else {
        serde_json::from_str(content).map_err(|e| format!("Parse JSON: {e}"))
    }
}

pub fn serialize(config: &Value, toml_format: bool) -> Result<String, String> {
    if toml_format {
        let value: toml::Value =
            serde_json::from_value(config.clone()).map_err(|e| format!("Convert to TOML: {e}"))?;
        toml::to_string_pretty(&value).map_err(|e| format!("Serialize TOML: {e}"))
    } else {
        serde_json::to_string_pretty(config).map_err(|e| format!("Serialize JSON: {e}"))
    }
}

/// Native drafts contain only explicitly entered fields; no secret values are projected.
#[derive(Clone)]
pub enum EntryMutation {
    Add(Value),
    Patch(Value),
    Delete,
}

pub fn validate_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty() || name.trim() != name || name.chars().any(char::is_control) {
        Err("mcp_invalid_name")
    } else {
        Ok(())
    }
}

pub fn validate_entry(value: &Value) -> Result<(), &'static str> {
    let object = value.as_object().ok_or("mcp_invalid_config")?;
    let transport = ["command", "url", "serverUrl", "httpUrl"]
        .iter()
        .any(|key| {
            object
                .get(*key)
                .and_then(Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
        });
    if !transport {
        return Err("mcp_transport_required");
    }
    for key in ["command", "url", "serverUrl", "httpUrl", "cwd", "type"] {
        if object.get(key).is_some_and(|value| !value.is_string()) {
            return Err("mcp_invalid_config");
        }
    }
    if object.get("args").is_some_and(|value| {
        value
            .as_array()
            .is_none_or(|args| args.iter().any(|item| !item.is_string()))
    }) {
        return Err("mcp_invalid_config");
    }
    for key in ["env", "headers"] {
        if object.get(key).is_some_and(|value| {
            value
                .as_object()
                .is_none_or(|map| map.values().any(|value| !value.is_string()))
        }) {
            return Err("mcp_invalid_config");
        }
    }
    for key in ["disabled", "enabled"] {
        if object.get(key).is_some_and(|value| !value.is_boolean()) {
            return Err("mcp_invalid_config");
        }
    }
    Ok(())
}

/// Codex/Antigravity keep their existing collection and transport field names.
/// Updating a native entry preserves untouched keys, including unknown extensions.
pub fn mutate_entries(
    root: &mut Value,
    collection: &str,
    name: &str,
    mutation: &EntryMutation,
) -> Result<(), &'static str> {
    validate_name(name)?;
    let root = root.as_object_mut().ok_or("mcp_invalid_config")?;
    let servers = root
        .entry(collection)
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or("mcp_invalid_config")?;
    match mutation {
        EntryMutation::Add(value) => {
            if servers.contains_key(name) {
                return Err("mcp_name_exists");
            }
            validate_entry(value)?;
            servers.insert(name.into(), value.clone());
        }
        EntryMutation::Patch(patch) => {
            let existing = servers.get_mut(name).ok_or("mcp_entry_missing")?;
            let target = existing.as_object_mut().ok_or("mcp_invalid_config")?;
            for (key, value) in patch.as_object().ok_or("mcp_invalid_config")? {
                if value.is_null() {
                    target.remove(key);
                } else if matches!(key.as_str(), "env" | "headers") {
                    let additions = value.as_object().ok_or("mcp_invalid_config")?;
                    let current = target
                        .entry(key)
                        .or_insert_with(|| serde_json::json!({}))
                        .as_object_mut()
                        .ok_or("mcp_invalid_config")?;
                    current.extend(additions.clone());
                } else {
                    target.insert(key.clone(), value.clone());
                }
            }
            validate_entry(existing)?;
        }
        EntryMutation::Delete => {
            if servers.remove(name).is_none() {
                return Err("mcp_entry_missing");
            }
        }
    }
    Ok(())
}
