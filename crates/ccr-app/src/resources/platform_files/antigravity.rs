//! Legacy-compatible Antigravity slash-command TOML codec.
use serde_json::{Value, json};

pub fn validate(raw: &str) -> Result<toml::Value, String> {
    let value: toml::Value = toml::from_str(raw).map_err(|_| "Invalid command TOML")?;
    if value.get("prompt").and_then(toml::Value::as_str).is_none() {
        return Err("Command prompt is required".into());
    }
    Ok(value)
}
/// Existing wire fields retain their meaning; unrelated TOML data survives patching.
pub fn patch(raw: &str, config: &Value) -> Result<String, String> {
    let mut value: toml::Value = if raw.is_empty() {
        toml::Value::Table(Default::default())
    } else {
        toml::from_str(raw).map_err(|_| "Invalid command TOML")?
    };
    let table = value.as_table_mut().ok_or("Command TOML must be a table")?;
    let prompt = config
        .get("command")
        .and_then(Value::as_str)
        .ok_or("Missing command field")?;
    let description = config
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("");
    table.insert("prompt".into(), prompt.into());
    table.insert("description".into(), description.into());
    toml::to_string_pretty(&value).map_err(|_| "Cannot encode command TOML".into())
}
pub fn record(key: &str, raw: &str) -> Result<Value, String> {
    let value = validate(raw)?;
    let prompt = value
        .get("prompt")
        .and_then(toml::Value::as_str)
        .ok_or("Missing prompt")?;
    let description = value
        .get("description")
        .and_then(toml::Value::as_str)
        .unwrap_or_else(|| {
            prompt
                .lines()
                .find(|line| !line.trim().is_empty())
                .unwrap_or("Command")
                .trim()
        });
    let key = key.strip_suffix(".toml").ok_or("Invalid command name")?;
    let (folder, name) = key.rsplit_once('/').unwrap_or(("", key));
    Ok(json!({"name":name,"folder":folder,"description":description,"command":prompt}))
}

/// User entries are overlaid by the same relative project command key, as before.
pub fn list_commands(project: &super::Store, user: &super::Store) -> Result<Value, String> {
    let mut chosen = std::collections::BTreeMap::new();
    for store in [user, project] {
        for row in store.list().map_err(|e| e.to_string())? {
            let session = store.open(&row.key).map_err(|e| e.to_string())?;
            chosen.insert(row.key.clone(), record(&row.key, session.content())?);
        }
    }
    let mut commands: Vec<_> = chosen.into_values().collect();
    commands.sort_by(|a, b| {
        a["name"]
            .as_str()
            .unwrap_or("")
            .cmp(b["name"].as_str().unwrap_or(""))
    });
    let folders: std::collections::BTreeSet<_> = commands
        .iter()
        .filter_map(|v| v["folder"].as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    Ok(json!({"commands":commands,"folders":folders}))
}
fn find_command(
    project: &super::Store,
    user: &super::Store,
    name: &str,
) -> Result<super::Session, String> {
    if name.contains(['/', '\\']) {
        return Err("Invalid command name".into());
    }
    let mut matches = Vec::new();
    for store in [project, user] {
        for row in store.list().map_err(|e| e.to_string())? {
            if std::path::Path::new(&row.key)
                .file_stem()
                .and_then(|v| v.to_str())
                == Some(name)
            {
                matches.push(store.open(&row.key).map_err(|e| e.to_string())?);
            }
        }
    }
    match matches.len() {
        0 => Err(format!("斜杠命令 '{name}' 不存在")),
        1 => matches.pop().ok_or_else(|| "Command disappeared".into()),
        _ => Err("存在多个同名命令文件".into()),
    }
}
pub fn add_command(project: &super::Store, name: &str, config: &Value) -> Result<(), String> {
    if name.contains(['/', '\\']) {
        return Err("Invalid command name".into());
    }
    let folder = config
        .get("folder")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    let key = if folder.is_empty() {
        format!("{name}.toml")
    } else {
        format!("{folder}/{name}.toml")
    };
    let session = project.open(&key).map_err(|e| e.to_string())?;
    let next = patch(session.content(), config)?;
    session.save(&next).map_err(|e| e.to_string())
}
pub fn update_command(
    project: &super::Store,
    user: &super::Store,
    name: &str,
    config: &Value,
) -> Result<(), String> {
    let session = find_command(project, user, name)?;
    let next = patch(session.content(), config)?;
    session.save(&next).map_err(|e| e.to_string())
}
pub fn delete_command(
    project: &super::Store,
    user: &super::Store,
    name: &str,
) -> Result<(), String> {
    find_command(project, user, name)?
        .delete()
        .map_err(|e| e.to_string())
}
pub fn list_extensions(store: &super::Store) -> Result<Value, String> {
    let mut extensions = Vec::new();
    for row in store.list().map_err(|e| e.to_string())? {
        let session = store.open(&row.key).map_err(|e| e.to_string())?;
        let directory = session
            .path()
            .parent()
            .ok_or("Missing extension directory")?;
        extensions.push(json!({"name":directory.file_name().and_then(|v|v.to_str()).unwrap_or(""),"description":row.description.unwrap_or_default(),"version":row.version.unwrap_or_default(),"path":directory.to_string_lossy()}));
    }
    Ok(json!(extensions))
}
