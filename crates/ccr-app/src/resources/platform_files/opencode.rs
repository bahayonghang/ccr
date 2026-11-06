//! Existing OpenCode Markdown/frontmatter codec and wire records.
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_yaml::{Mapping as YamlMapping, Value as YamlValue};
use std::path::Path;

pub struct MarkdownDocument {
    pub frontmatter: YamlMapping,
    pub body: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeAgentRecord {
    pub name: String,
    pub path: String,
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeCommandRecord {
    pub name: String,
    pub path: String,
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtask: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub template: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_error: Option<String>,
}

pub fn parse_markdown_document(raw: &str) -> Result<MarkdownDocument, String> {
    let normalized = raw.replace("\r\n", "\n");
    let mut lines = normalized.lines();
    if lines.next() != Some("---") {
        return Ok(MarkdownDocument {
            frontmatter: YamlMapping::new(),
            body: normalized.trim().to_string(),
        });
    }

    let mut frontmatter_lines = Vec::new();
    let mut body_lines = Vec::new();
    let mut in_frontmatter = true;

    for line in normalized.lines().skip(1) {
        if in_frontmatter && line == "---" {
            in_frontmatter = false;
            continue;
        }
        if in_frontmatter {
            frontmatter_lines.push(line);
        } else {
            body_lines.push(line);
        }
    }

    if in_frontmatter {
        return Err("Markdown frontmatter 缺少结束分隔符 '---'".to_string());
    }

    let frontmatter = if frontmatter_lines.is_empty() {
        YamlMapping::new()
    } else {
        let raw_frontmatter = frontmatter_lines.join("\n");
        let yaml = serde_yaml::from_str::<YamlValue>(&raw_frontmatter)
            .map_err(|e| format!("解析 YAML frontmatter 失败: {e}"))?;
        yaml.as_mapping()
            .cloned()
            .ok_or_else(|| "Markdown frontmatter 顶层必须是 mapping".to_string())?
    };

    Ok(MarkdownDocument {
        frontmatter,
        body: body_lines.join("\n").trim().to_string(),
    })
}

pub fn render_markdown_document(doc: &MarkdownDocument) -> Result<String, String> {
    let yaml = serde_yaml::to_string(&YamlValue::Mapping(doc.frontmatter.clone()))
        .map_err(|e| format!("序列化 YAML frontmatter 失败: {e}"))?;
    let cleaned_yaml = yaml.trim().trim_start_matches("---").trim();
    let mut rendered = String::new();
    rendered.push_str("---\n");
    if !cleaned_yaml.is_empty() {
        rendered.push_str(cleaned_yaml);
        rendered.push('\n');
    }
    rendered.push_str("---\n\n");
    rendered.push_str(doc.body.trim());
    rendered.push('\n');
    Ok(rendered)
}

fn yaml_key(name: &str) -> YamlValue {
    YamlValue::String(name.to_string())
}

pub fn yaml_string(mapping: &YamlMapping, key: &str) -> Option<String> {
    mapping
        .get(yaml_key(key))
        .and_then(YamlValue::as_str)
        .map(str::to_string)
}

pub fn yaml_bool(mapping: &YamlMapping, key: &str) -> Option<bool> {
    mapping.get(yaml_key(key)).and_then(YamlValue::as_bool)
}

fn yaml_f64(mapping: &YamlMapping, key: &str) -> Option<f64> {
    mapping.get(yaml_key(key)).and_then(YamlValue::as_f64)
}

fn yaml_u64(mapping: &YamlMapping, key: &str) -> Option<u64> {
    mapping.get(yaml_key(key)).and_then(YamlValue::as_u64)
}

fn yaml_json(mapping: &YamlMapping, key: &str) -> Option<Value> {
    mapping
        .get(yaml_key(key))
        .and_then(|value| serde_json::to_value(value).ok())
}

fn json_key_to_frontmatter_key(key: &str) -> String {
    match key {
        "topP" => "top_p".to_string(),
        other => other.to_string(),
    }
}

fn frontmatter_key_to_json_key(key: &str) -> String {
    match key {
        "top_p" => "topP".to_string(),
        other => other.to_string(),
    }
}

fn mapping_without_known_keys(mapping: &YamlMapping, known_keys: &[&str]) -> Option<Value> {
    let mut trimmed = YamlMapping::new();
    for (key, value) in mapping {
        let Some(key_str) = key.as_str() else {
            trimmed.insert(key.clone(), value.clone());
            continue;
        };
        if known_keys.contains(&key_str) {
            continue;
        }
        trimmed.insert(
            YamlValue::String(frontmatter_key_to_json_key(key_str)),
            value.clone(),
        );
    }

    if trimmed.is_empty() {
        None
    } else {
        serde_json::to_value(YamlValue::Mapping(trimmed)).ok()
    }
}

fn apply_frontmatter_updates(
    frontmatter: &mut YamlMapping,
    updates: &Map<String, Value>,
    skip_keys: &[&str],
) -> Result<(), String> {
    for (key, value) in updates {
        if skip_keys.contains(&key.as_str()) {
            continue;
        }
        let target_key = json_key_to_frontmatter_key(key);
        let yaml_key = yaml_key(&target_key);
        if value.is_null() {
            frontmatter.remove(&yaml_key);
            continue;
        }
        let yaml_value = serde_yaml::to_value(value)
            .map_err(|e| format!("转换 frontmatter 字段 '{}' 失败: {e}", key))?;
        frontmatter.insert(yaml_key, yaml_value);
    }
    Ok(())
}

pub fn agent_record_from_path(
    path: &Path,
    scope: &str,
    raw: Result<String, String>,
) -> OpenCodeAgentRecord {
    let name = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    let raw = match raw {
        Ok(raw) => raw,
        Err(error) => {
            return OpenCodeAgentRecord {
                name,
                path: path.to_string_lossy().to_string(),
                scope: scope.to_string(),
                description: None,
                mode: None,
                model: None,
                temperature: None,
                top_p: None,
                steps: None,
                hidden: None,
                disable: None,
                color: None,
                permission: None,
                tools: None,
                body: String::new(),
                other: None,
                parse_error: Some(error),
            };
        }
    };

    match parse_markdown_document(&raw) {
        Ok(doc) => OpenCodeAgentRecord {
            name,
            path: path.to_string_lossy().to_string(),
            scope: scope.to_string(),
            description: yaml_string(&doc.frontmatter, "description"),
            mode: yaml_string(&doc.frontmatter, "mode"),
            model: yaml_string(&doc.frontmatter, "model"),
            temperature: yaml_f64(&doc.frontmatter, "temperature"),
            top_p: yaml_f64(&doc.frontmatter, "top_p"),
            steps: yaml_u64(&doc.frontmatter, "steps"),
            hidden: yaml_bool(&doc.frontmatter, "hidden"),
            disable: yaml_bool(&doc.frontmatter, "disable"),
            color: yaml_string(&doc.frontmatter, "color"),
            permission: yaml_json(&doc.frontmatter, "permission"),
            tools: yaml_json(&doc.frontmatter, "tools"),
            body: doc.body,
            other: mapping_without_known_keys(
                &doc.frontmatter,
                &[
                    "description",
                    "mode",
                    "model",
                    "temperature",
                    "top_p",
                    "steps",
                    "hidden",
                    "disable",
                    "color",
                    "permission",
                    "tools",
                ],
            ),
            parse_error: None,
        },
        Err(error) => OpenCodeAgentRecord {
            name,
            path: path.to_string_lossy().to_string(),
            scope: scope.to_string(),
            description: None,
            mode: None,
            model: None,
            temperature: None,
            top_p: None,
            steps: None,
            hidden: None,
            disable: None,
            color: None,
            permission: None,
            tools: None,
            body: raw,
            other: None,
            parse_error: Some(error),
        },
    }
}

pub fn command_record_from_path(
    path: &Path,
    scope: &str,
    raw: Result<String, String>,
) -> OpenCodeCommandRecord {
    let name = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    let raw = match raw {
        Ok(raw) => raw,
        Err(error) => {
            return OpenCodeCommandRecord {
                name,
                path: path.to_string_lossy().to_string(),
                scope: scope.to_string(),
                description: None,
                agent: None,
                subtask: None,
                model: None,
                template: String::new(),
                other: None,
                parse_error: Some(error),
            };
        }
    };

    match parse_markdown_document(&raw) {
        Ok(doc) => OpenCodeCommandRecord {
            name,
            path: path.to_string_lossy().to_string(),
            scope: scope.to_string(),
            description: yaml_string(&doc.frontmatter, "description"),
            agent: yaml_string(&doc.frontmatter, "agent"),
            subtask: yaml_bool(&doc.frontmatter, "subtask"),
            model: yaml_string(&doc.frontmatter, "model"),
            template: doc.body,
            other: mapping_without_known_keys(
                &doc.frontmatter,
                &["description", "agent", "subtask", "model"],
            ),
            parse_error: None,
        },
        Err(error) => OpenCodeCommandRecord {
            name,
            path: path.to_string_lossy().to_string(),
            scope: scope.to_string(),
            description: None,
            agent: None,
            subtask: None,
            model: None,
            template: raw,
            other: None,
            parse_error: Some(error),
        },
    }
}

/// Structured Tauri updates patch existing frontmatter, retaining unknown keys.
pub fn patch(raw: &str, config: &Map<String, Value>, agent: bool) -> Result<String, String> {
    let mut document = parse_markdown_document(raw)?;
    let body_key = if agent { "body" } else { "template" };
    if let Some(body) = config.get(body_key).and_then(Value::as_str) {
        document.body = body.into();
    }
    apply_frontmatter_updates(
        &mut document.frontmatter,
        config,
        &["name", "scope", "projectRoot", body_key],
    )?;
    validate(&document)?;
    render_markdown_document(&document)
}
pub fn validate(document: &MarkdownDocument) -> Result<(), String> {
    if yaml_string(&document.frontmatter, "description").is_none() {
        return Err("Description is required".into());
    }
    Ok(())
}

pub fn list_agents(store: &super::Store, scope: &str) -> Result<Vec<OpenCodeAgentRecord>, String> {
    store
        .list()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|row| {
            let session = store.open(&row.key).map_err(|e| e.to_string())?;
            Ok(agent_record_from_path(
                session.path(),
                scope,
                Ok(session.content().into()),
            ))
        })
        .collect()
}
pub fn list_commands(
    store: &super::Store,
    scope: &str,
) -> Result<Vec<OpenCodeCommandRecord>, String> {
    store
        .list()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|row| {
            let session = store.open(&row.key).map_err(|e| e.to_string())?;
            Ok(command_record_from_path(
                session.path(),
                scope,
                Ok(session.content().into()),
            ))
        })
        .collect()
}
pub fn upsert_agent(
    store: &super::Store,
    name: &str,
    config: &Map<String, Value>,
    scope: &str,
) -> Result<OpenCodeAgentRecord, String> {
    if store.kind() != super::Kind::OpenCodeAgent {
        return Err("Invalid resource kind".into());
    }
    let key = format!("{name}.md");
    let session = store.open(&key).map_err(|e| e.to_string())?;
    let next = patch(session.content(), config, true)?;
    session.save(&next).map_err(|e| e.to_string())?;
    Ok(agent_record_from_path(session.path(), scope, Ok(next)))
}
pub fn upsert_command(
    store: &super::Store,
    name: &str,
    config: &Map<String, Value>,
    scope: &str,
) -> Result<OpenCodeCommandRecord, String> {
    if store.kind() != super::Kind::OpenCodeCommand {
        return Err("Invalid resource kind".into());
    }
    let key = format!("{name}.md");
    let session = store.open(&key).map_err(|e| e.to_string())?;
    let next = patch(session.content(), config, false)?;
    session.save(&next).map_err(|e| e.to_string())?;
    Ok(command_record_from_path(session.path(), scope, Ok(next)))
}
pub fn delete(store: &super::Store, name: &str) -> Result<(), String> {
    store
        .open(&format!("{name}.md"))
        .map_err(|e| e.to_string())?
        .delete()
        .map_err(|e| e.to_string())
}

/// OpenCode's existing built-in theme catalog, shared by both desktop adapters.
pub const THEMES: &[(&str, &str, &str)] = &[
    ("dark", "Dark", "dark"),
    ("light", "Light", "light"),
    ("system", "System", "system"),
    ("catppuccin-mocha", "Catppuccin Mocha", "dark"),
    ("catppuccin-latte", "Catppuccin Latte", "light"),
    ("dracula", "Dracula", "dark"),
    ("nord", "Nord", "dark"),
    ("one-dark", "One Dark", "dark"),
    ("github-dark", "GitHub Dark", "dark"),
    ("github-light", "GitHub Light", "light"),
    ("solarized-dark", "Solarized Dark", "dark"),
    ("solarized-light", "Solarized Light", "light"),
    ("tokyonight", "Tokyo Night", "dark"),
];
pub struct PluginFile {
    pub name: String,
    pub path: std::path::PathBuf,
    pub size: u64,
}
/// Metadata-only inventory: plugin code is never read or executed.
pub fn list_plugins(directory: &std::path::Path) -> Result<Vec<PluginFile>, super::Error> {
    use super::Error;
    super::check_path(directory)?;
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => return Err(Error::Storage),
    };
    let mut files = Vec::new();
    for (index, entry) in entries.enumerate() {
        if index >= 4096 || files.len() >= super::MAX_FILES {
            return Err(Error::TooLarge);
        }
        let entry = entry.map_err(|_| Error::Storage)?;
        let path = entry.path();
        super::check_path(&path)?;
        let metadata = entry.metadata().map_err(|_| Error::Storage)?;
        if !metadata.is_file()
            || !path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    matches!(
                        ext.to_ascii_lowercase().as_str(),
                        "js" | "cjs" | "mjs" | "ts"
                    )
                })
        {
            continue;
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| Error::InvalidName)?;
        files.push(PluginFile {
            name,
            path,
            size: metadata.len(),
        });
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}
