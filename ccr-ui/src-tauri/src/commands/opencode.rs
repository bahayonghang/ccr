//! OpenCode 命令：Settings / TUI / Agents / Commands / Plugins / Skills。
//!
//! 以官方 OpenCode 文档为准：
//! - `~/.config/opencode/opencode.json` 作为主配置
//! - `~/.config/opencode/tui.json` 作为 TUI 配置
//! - `~/.config/opencode/{agents,commands,plugins,skills}` 作为全局目录
//! - `.opencode/{agents,commands,plugins,skills}` 作为项目目录
//!
//! 兼容读取旧路径 `~/.opencode/*`，但所有写入都会落到新路径。

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use serde_yaml::{Mapping as YamlMapping, Value as YamlValue};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeThemeRecord {
    pub id: String,
    pub name: String,
    pub theme_type: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodePluginFileRecord {
    pub name: String,
    pub path: String,
    pub scope: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodeSkillLocationRecord {
    pub kind: String,
    pub scope: String,
    pub path: String,
    pub exists: bool,
    pub skill_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct OpenCodeScopedRequest {
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    project_root: Option<String>,
}

#[derive(Debug, Clone)]
struct ResolvedScopeDir {
    scope: String,
    dir: PathBuf,
}

#[derive(Debug, Clone)]
struct MarkdownDocument {
    frontmatter: YamlMapping,
    body: String,
}

fn opencode_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())
}

fn opencode_config_dir_from_home(home: &Path) -> PathBuf {
    home.join(".config").join("opencode")
}

fn opencode_legacy_dir_from_home(home: &Path) -> PathBuf {
    home.join(".opencode")
}

fn opencode_config_dir() -> Result<PathBuf, String> {
    Ok(opencode_config_dir_from_home(&opencode_home_dir()?))
}

fn opencode_legacy_dir() -> Result<PathBuf, String> {
    Ok(opencode_legacy_dir_from_home(&opencode_home_dir()?))
}

fn opencode_config_path() -> Result<PathBuf, String> {
    Ok(opencode_config_dir()?.join("opencode.json"))
}

fn opencode_legacy_config_path() -> Result<PathBuf, String> {
    Ok(opencode_legacy_dir()?.join("config.json"))
}

fn opencode_tui_path() -> Result<PathBuf, String> {
    Ok(opencode_config_dir()?.join("tui.json"))
}

fn opencode_legacy_keybindings_path() -> Result<PathBuf, String> {
    Ok(opencode_legacy_dir()?.join("keybindings.json"))
}

fn detect_project_root_from(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join("opencode.json").exists()
            || current.join(".opencode").exists()
            || current.join(".git").exists()
        {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn current_project_root() -> Option<PathBuf> {
    std::env::current_dir()
        .ok()
        .and_then(|path| detect_project_root_from(&path))
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录 '{}' 失败: {e}", parent.to_string_lossy()))?;
    }
    Ok(())
}

fn resolve_scope_dir(
    request: &OpenCodeScopedRequest,
    folder: &str,
) -> Result<ResolvedScopeDir, String> {
    match request.scope.as_deref().unwrap_or("global") {
        "global" => Ok(ResolvedScopeDir {
            scope: "global".to_string(),
            dir: opencode_config_dir()?.join(folder),
        }),
        "project" => {
            let project_root = request
                .project_root
                .as_ref()
                .map(PathBuf::from)
                .or_else(current_project_root)
                .ok_or_else(|| "未检测到项目目录，无法使用 project scope".to_string())?;
            Ok(ResolvedScopeDir {
                scope: "project".to_string(),
                dir: project_root.join(".opencode").join(folder),
            })
        }
        other => Err(format!("不支持的 OpenCode scope: {other}")),
    }
}

fn read_json_file(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(json!({}));
    }
    let content = fs::read_to_string(path)
        .map_err(|e| format!("读取文件 '{}' 失败: {e}", path.to_string_lossy()))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("解析 JSON '{}' 失败: {e}", path.to_string_lossy()))
}

fn read_json_file_with_fallback(primary: &Path, fallback: &Path) -> Result<Value, String> {
    if primary.exists() {
        return read_json_file(primary);
    }
    if fallback.exists() {
        return read_json_file(fallback);
    }
    Ok(json!({}))
}

fn write_json_file(path: &Path, value: &Value) -> Result<(), String> {
    ensure_parent_dir(path)?;
    let content =
        serde_json::to_string_pretty(value).map_err(|e| format!("序列化 JSON 失败: {e}"))?;
    fs::write(path, content).map_err(|e| format!("写入文件 '{}' 失败: {e}", path.to_string_lossy()))
}

fn merge_json_objects(current: Value, incoming: Value) -> Value {
    match (current, incoming) {
        (Value::Object(mut current_map), Value::Object(incoming_map)) => {
            for (key, value) in incoming_map {
                current_map.insert(key, value);
            }
            Value::Object(current_map)
        }
        (_, replacement) => replacement,
    }
}

fn parse_markdown_document(raw: &str) -> Result<MarkdownDocument, String> {
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

fn render_markdown_document(doc: &MarkdownDocument) -> Result<String, String> {
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

fn yaml_string(mapping: &YamlMapping, key: &str) -> Option<String> {
    mapping
        .get(&yaml_key(key))
        .and_then(YamlValue::as_str)
        .map(str::to_string)
}

fn yaml_bool(mapping: &YamlMapping, key: &str) -> Option<bool> {
    mapping.get(&yaml_key(key)).and_then(YamlValue::as_bool)
}

fn yaml_f64(mapping: &YamlMapping, key: &str) -> Option<f64> {
    mapping.get(&yaml_key(key)).and_then(YamlValue::as_f64)
}

fn yaml_u64(mapping: &YamlMapping, key: &str) -> Option<u64> {
    mapping.get(&yaml_key(key)).and_then(YamlValue::as_u64)
}

fn yaml_json(mapping: &YamlMapping, key: &str) -> Option<Value> {
    mapping
        .get(&yaml_key(key))
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

fn validate_markdown_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("名称不能为空".to_string());
    }
    if name.contains(['/', '\\']) {
        return Err("名称不能包含路径分隔符".to_string());
    }
    Ok(())
}

fn read_markdown_file(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("读取文件 '{}' 失败: {e}", path.to_string_lossy()))
}

fn write_markdown_file(path: &Path, doc: &MarkdownDocument) -> Result<(), String> {
    ensure_parent_dir(path)?;
    let rendered = render_markdown_document(doc)?;
    fs::write(path, rendered)
        .map_err(|e| format!("写入文件 '{}' 失败: {e}", path.to_string_lossy()))
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

fn markdown_file_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(format!("{name}.md"))
}

fn scan_markdown_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files = fs::read_dir(dir)
        .map_err(|e| format!("读取目录 '{}' 失败: {e}", dir.to_string_lossy()))?
        .filter_map(|entry| entry.ok().map(|item| item.path()))
        .filter(|path| path.extension() == Some(OsStr::new("md")))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn agent_record_from_path(path: &Path, scope: &str) -> OpenCodeAgentRecord {
    let name = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    let raw = match read_markdown_file(path) {
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

fn command_record_from_path(path: &Path, scope: &str) -> OpenCodeCommandRecord {
    let name = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    let raw = match read_markdown_file(path) {
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

fn load_scope_records<T>(
    dir: &Path,
    scope: &str,
    factory: fn(&Path, &str) -> T,
) -> Result<Vec<T>, String> {
    scan_markdown_files(dir)?
        .into_iter()
        .map(|path| Ok(factory(&path, scope)))
        .collect()
}

fn list_scope_candidates(folder: &str) -> Result<Vec<ResolvedScopeDir>, String> {
    let mut scopes = vec![ResolvedScopeDir {
        scope: "global".to_string(),
        dir: opencode_config_dir()?.join(folder),
    }];

    if let Some(project_root) = current_project_root() {
        scopes.push(ResolvedScopeDir {
            scope: "project".to_string(),
            dir: project_root.join(".opencode").join(folder),
        });
    }

    Ok(scopes)
}

fn list_agents_internal() -> Result<Vec<OpenCodeAgentRecord>, String> {
    let mut records = Vec::new();
    for scope in list_scope_candidates("agents")? {
        records.extend(load_scope_records(
            &scope.dir,
            &scope.scope,
            agent_record_from_path,
        )?);
    }
    Ok(records)
}

fn list_commands_internal() -> Result<Vec<OpenCodeCommandRecord>, String> {
    let mut records = Vec::new();
    for scope in list_scope_candidates("commands")? {
        records.extend(load_scope_records(
            &scope.dir,
            &scope.scope,
            command_record_from_path,
        )?);
    }
    Ok(records)
}

fn upsert_agent_internal(
    name: &str,
    config: &Map<String, Value>,
    scope: &ResolvedScopeDir,
) -> Result<OpenCodeAgentRecord, String> {
    validate_markdown_name(name)?;
    let path = markdown_file_path(&scope.dir, name);
    let mut document = if path.exists() {
        parse_markdown_document(&read_markdown_file(&path)?)?
    } else {
        MarkdownDocument {
            frontmatter: YamlMapping::new(),
            body: String::new(),
        }
    };

    if let Some(body) = config.get("body").and_then(Value::as_str) {
        document.body = body.to_string();
    }

    apply_frontmatter_updates(
        &mut document.frontmatter,
        config,
        &["name", "scope", "projectRoot", "body"],
    )?;

    if yaml_string(&document.frontmatter, "description").is_none() {
        return Err("Agent description 为必填项".to_string());
    }

    write_markdown_file(&path, &document)?;
    Ok(agent_record_from_path(&path, &scope.scope))
}

fn upsert_command_internal(
    name: &str,
    config: &Map<String, Value>,
    scope: &ResolvedScopeDir,
) -> Result<OpenCodeCommandRecord, String> {
    validate_markdown_name(name)?;
    let path = markdown_file_path(&scope.dir, name);
    let mut document = if path.exists() {
        parse_markdown_document(&read_markdown_file(&path)?)?
    } else {
        MarkdownDocument {
            frontmatter: YamlMapping::new(),
            body: String::new(),
        }
    };

    if let Some(template) = config.get("template").and_then(Value::as_str) {
        document.body = template.to_string();
    }

    apply_frontmatter_updates(
        &mut document.frontmatter,
        config,
        &["name", "scope", "projectRoot", "template"],
    )?;

    if yaml_string(&document.frontmatter, "description").is_none() {
        return Err("Command description 为必填项".to_string());
    }

    write_markdown_file(&path, &document)?;
    Ok(command_record_from_path(&path, &scope.scope))
}

fn delete_markdown_doc(
    name: &str,
    request: &OpenCodeScopedRequest,
    folder: &str,
) -> Result<String, String> {
    validate_markdown_name(name)?;
    let scope = resolve_scope_dir(request, folder)?;
    let path = markdown_file_path(&scope.dir, name);
    if !path.exists() {
        return Err(format!(
            "未找到 OpenCode {folder} 文件: {}",
            path.to_string_lossy()
        ));
    }
    fs::remove_file(&path)
        .map_err(|e| format!("删除文件 '{}' 失败: {e}", path.to_string_lossy()))?;
    Ok(name.to_string())
}

fn scan_plugin_files(dir: &Path, scope: &str) -> Result<Vec<OpenCodePluginFileRecord>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = fs::read_dir(dir)
        .map_err(|e| format!("读取目录 '{}' 失败: {e}", dir.to_string_lossy()))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                return None;
            }
            let extension = path.extension()?.to_str()?.to_ascii_lowercase();
            if !matches!(extension.as_str(), "js" | "cjs" | "mjs" | "ts") {
                return None;
            }
            let metadata = entry.metadata().ok()?;
            Some(OpenCodePluginFileRecord {
                name: path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default()
                    .to_string(),
                path: path.to_string_lossy().to_string(),
                scope: scope.to_string(),
                size: metadata.len(),
            })
        })
        .collect::<Vec<_>>();

    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn count_skills_in_dir(dir: &Path) -> (usize, Vec<String>) {
    if !dir.exists() {
        return (0, Vec::new());
    }

    let mut skills = fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_dir() || !path.join("SKILL.md").exists() {
                return None;
            }
            path.file_name()
                .and_then(|value| value.to_str())
                .map(str::to_string)
        })
        .collect::<Vec<_>>();
    skills.sort();
    (skills.len(), skills)
}

fn build_skill_locations(
    home: &Path,
    project_root: Option<&Path>,
) -> Vec<OpenCodeSkillLocationRecord> {
    let mut locations = Vec::new();

    let mut push_location = |kind: &str, scope: &str, path: PathBuf| {
        let (skill_count, skills) = count_skills_in_dir(&path);
        locations.push(OpenCodeSkillLocationRecord {
            kind: kind.to_string(),
            scope: scope.to_string(),
            path: path.to_string_lossy().to_string(),
            exists: path.exists(),
            skill_count,
            skills,
        });
    };

    push_location(
        "opencode",
        "global",
        opencode_config_dir_from_home(home).join("skills"),
    );
    push_location(
        "claude-compatible",
        "global",
        home.join(".claude").join("skills"),
    );
    push_location(
        "agents-compatible",
        "global",
        home.join(".agents").join("skills"),
    );

    if let Some(project_root) = project_root {
        push_location(
            "opencode",
            "project",
            project_root.join(".opencode").join("skills"),
        );
        push_location(
            "claude-compatible",
            "project",
            project_root.join(".claude").join("skills"),
        );
        push_location(
            "agents-compatible",
            "project",
            project_root.join(".agents").join("skills"),
        );
    }

    locations
}

#[tauri::command]
pub async fn opencode_get_settings() -> Result<Value, String> {
    let primary = opencode_config_path()?;
    let legacy = opencode_legacy_config_path()?;
    read_json_file_with_fallback(&primary, &legacy)
}

#[tauri::command]
pub async fn opencode_update_settings(settings: Value) -> Result<Value, String> {
    let primary = opencode_config_path()?;
    let legacy = opencode_legacy_config_path()?;
    let current = read_json_file_with_fallback(&primary, &legacy)?;
    let merged = merge_json_objects(current, settings);
    write_json_file(&primary, &merged)?;
    Ok(merged)
}

#[tauri::command]
pub async fn opencode_get_tui_settings() -> Result<Value, String> {
    let primary = opencode_tui_path()?;
    let legacy = opencode_legacy_keybindings_path()?;
    if primary.exists() {
        return read_json_file(&primary);
    }
    if legacy.exists() {
        let keybinds = read_json_file(&legacy)?;
        return Ok(json!({ "keybinds": keybinds }));
    }
    Ok(json!({}))
}

#[tauri::command]
pub async fn opencode_update_tui_settings(settings: Value) -> Result<Value, String> {
    let primary = opencode_tui_path()?;
    let current = read_json_file(&primary)?;
    let merged = merge_json_objects(current, settings);
    write_json_file(&primary, &merged)?;
    Ok(merged)
}

#[tauri::command]
pub async fn opencode_get_keybindings() -> Result<Value, String> {
    let tui = opencode_get_tui_settings().await?;
    Ok(tui
        .as_object()
        .and_then(|map| map.get("keybinds"))
        .cloned()
        .unwrap_or_else(|| json!({})))
}

#[tauri::command]
pub async fn opencode_update_keybindings(keybindings: Value) -> Result<Value, String> {
    let updated = opencode_update_tui_settings(json!({ "keybinds": keybindings })).await?;
    Ok(updated
        .as_object()
        .and_then(|map| map.get("keybinds"))
        .cloned()
        .unwrap_or_else(|| json!({})))
}

#[tauri::command]
pub async fn opencode_list_themes() -> Result<Value, String> {
    Ok(serde_json::to_value(vec![
        OpenCodeThemeRecord {
            id: "dark".to_string(),
            name: "Dark".to_string(),
            theme_type: "dark".to_string(),
        },
        OpenCodeThemeRecord {
            id: "light".to_string(),
            name: "Light".to_string(),
            theme_type: "light".to_string(),
        },
        OpenCodeThemeRecord {
            id: "system".to_string(),
            name: "System".to_string(),
            theme_type: "system".to_string(),
        },
        OpenCodeThemeRecord {
            id: "catppuccin-mocha".to_string(),
            name: "Catppuccin Mocha".to_string(),
            theme_type: "dark".to_string(),
        },
        OpenCodeThemeRecord {
            id: "catppuccin-latte".to_string(),
            name: "Catppuccin Latte".to_string(),
            theme_type: "light".to_string(),
        },
        OpenCodeThemeRecord {
            id: "dracula".to_string(),
            name: "Dracula".to_string(),
            theme_type: "dark".to_string(),
        },
        OpenCodeThemeRecord {
            id: "nord".to_string(),
            name: "Nord".to_string(),
            theme_type: "dark".to_string(),
        },
        OpenCodeThemeRecord {
            id: "one-dark".to_string(),
            name: "One Dark".to_string(),
            theme_type: "dark".to_string(),
        },
        OpenCodeThemeRecord {
            id: "github-dark".to_string(),
            name: "GitHub Dark".to_string(),
            theme_type: "dark".to_string(),
        },
        OpenCodeThemeRecord {
            id: "github-light".to_string(),
            name: "GitHub Light".to_string(),
            theme_type: "light".to_string(),
        },
        OpenCodeThemeRecord {
            id: "solarized-dark".to_string(),
            name: "Solarized Dark".to_string(),
            theme_type: "dark".to_string(),
        },
        OpenCodeThemeRecord {
            id: "solarized-light".to_string(),
            name: "Solarized Light".to_string(),
            theme_type: "light".to_string(),
        },
        OpenCodeThemeRecord {
            id: "tokyonight".to_string(),
            name: "Tokyo Night".to_string(),
            theme_type: "dark".to_string(),
        },
    ])
    .map_err(|e| format!("序列化主题列表失败: {e}"))?)
}

#[tauri::command]
pub async fn opencode_list_agents() -> Result<Value, String> {
    serde_json::to_value(list_agents_internal()?)
        .map_err(|e| format!("序列化 OpenCode agents 失败: {e}"))
}

#[tauri::command]
pub async fn opencode_add_agent(config: Value) -> Result<Value, String> {
    let object = config
        .as_object()
        .ok_or_else(|| "OpenCode agent 配置必须是 JSON object".to_string())?;
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "OpenCode agent name 为必填项".to_string())?;
    let scoped: OpenCodeScopedRequest =
        serde_json::from_value(config.clone()).map_err(|e| format!("解析 scope 失败: {e}"))?;
    let scope = resolve_scope_dir(&scoped, "agents")?;
    serde_json::to_value(upsert_agent_internal(name, object, &scope)?)
        .map_err(|e| format!("序列化 OpenCode agent 失败: {e}"))
}

#[tauri::command]
pub async fn opencode_update_agent(config: Value) -> Result<Value, String> {
    opencode_add_agent(config).await
}

#[tauri::command]
pub async fn opencode_delete_agent(name: String, context: Option<Value>) -> Result<Value, String> {
    let scoped = context
        .map(serde_json::from_value::<OpenCodeScopedRequest>)
        .transpose()
        .map_err(|e| format!("解析 OpenCode agent scope 失败: {e}"))?
        .unwrap_or_default();
    serde_json::to_value(delete_markdown_doc(&name, &scoped, "agents")?)
        .map_err(|e| format!("序列化删除结果失败: {e}"))
}

#[tauri::command]
pub async fn opencode_list_commands() -> Result<Value, String> {
    serde_json::to_value(list_commands_internal()?)
        .map_err(|e| format!("序列化 OpenCode commands 失败: {e}"))
}

#[tauri::command]
pub async fn opencode_add_command(config: Value) -> Result<Value, String> {
    let object = config
        .as_object()
        .ok_or_else(|| "OpenCode command 配置必须是 JSON object".to_string())?;
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "OpenCode command name 为必填项".to_string())?;
    let scoped: OpenCodeScopedRequest =
        serde_json::from_value(config.clone()).map_err(|e| format!("解析 scope 失败: {e}"))?;
    let scope = resolve_scope_dir(&scoped, "commands")?;
    serde_json::to_value(upsert_command_internal(name, object, &scope)?)
        .map_err(|e| format!("序列化 OpenCode command 失败: {e}"))
}

#[tauri::command]
pub async fn opencode_update_command(config: Value) -> Result<Value, String> {
    opencode_add_command(config).await
}

#[tauri::command]
pub async fn opencode_delete_command(
    name: String,
    context: Option<Value>,
) -> Result<Value, String> {
    let scoped = context
        .map(serde_json::from_value::<OpenCodeScopedRequest>)
        .transpose()
        .map_err(|e| format!("解析 OpenCode command scope 失败: {e}"))?
        .unwrap_or_default();
    serde_json::to_value(delete_markdown_doc(&name, &scoped, "commands")?)
        .map_err(|e| format!("序列化删除结果失败: {e}"))
}

#[tauri::command]
pub async fn opencode_list_local_plugins() -> Result<Value, String> {
    let mut records = Vec::new();
    records.extend(scan_plugin_files(
        &opencode_config_dir()?.join("plugins"),
        "global",
    )?);
    if let Some(project_root) = current_project_root() {
        records.extend(scan_plugin_files(
            &project_root.join(".opencode").join("plugins"),
            "project",
        )?);
    }
    serde_json::to_value(records).map_err(|e| format!("序列化 OpenCode 本地插件失败: {e}"))
}

#[tauri::command]
pub async fn opencode_list_skill_locations() -> Result<Value, String> {
    let home = opencode_home_dir()?;
    let project_root = current_project_root();
    serde_json::to_value(build_skill_locations(&home, project_root.as_deref()))
        .map_err(|e| format!("序列化 OpenCode skills 目录失败: {e}"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn config_dir_uses_xdg_layout() {
        let dir = opencode_config_dir_from_home(Path::new("/tmp/test-home"));
        assert_eq!(dir, PathBuf::from("/tmp/test-home/.config/opencode"));
    }

    #[test]
    fn merge_json_objects_preserves_unrelated_keys() {
        let current = json!({
            "model": "anthropic/claude-sonnet-4-5",
            "permission": {
                "bash": "ask"
            }
        });
        let merged = merge_json_objects(
            current,
            json!({
                "server": {
                    "port": 4096
                }
            }),
        );

        assert_eq!(merged["model"], json!("anthropic/claude-sonnet-4-5"));
        assert_eq!(merged["permission"]["bash"], json!("ask"));
        assert_eq!(merged["server"]["port"], json!(4096));
    }

    #[test]
    fn parse_markdown_frontmatter_round_trip() {
        let raw = "---\ndescription: Review code\nmode: subagent\nhidden: true\n---\n\nYou are a reviewer.\n";
        let parsed = parse_markdown_document(raw).unwrap();
        assert_eq!(
            yaml_string(&parsed.frontmatter, "description").as_deref(),
            Some("Review code")
        );
        assert_eq!(yaml_bool(&parsed.frontmatter, "hidden"), Some(true));
        assert_eq!(parsed.body, "You are a reviewer.");

        let rendered = render_markdown_document(&parsed).unwrap();
        assert!(rendered.contains("description: Review code"));
        assert!(rendered.contains("You are a reviewer."));
    }

    #[test]
    fn agent_upsert_preserves_unknown_frontmatter_and_filename() {
        let temp = tempdir().unwrap();
        let agents_dir = temp.path().join("agents");
        fs::create_dir_all(&agents_dir).unwrap();
        let path = agents_dir.join("review.md");
        fs::write(
            &path,
            "---\ndescription: Review code\nmode: subagent\ncustom_flag: true\n---\n\nOld body\n",
        )
        .unwrap();

        let scope = ResolvedScopeDir {
            scope: "global".to_string(),
            dir: agents_dir,
        };

        let config = json!({
            "description": "Review code carefully",
            "body": "New body",
            "temperature": 0.1
        });

        let record = upsert_agent_internal("review", config.as_object().unwrap(), &scope).unwrap();
        assert_eq!(record.name, "review");
        assert_eq!(record.body, "New body");
        assert_eq!(record.temperature, Some(0.1));
        assert_eq!(record.other.unwrap()["custom_flag"], json!(true));
        assert!(record.path.ends_with("review.md"));
    }

    #[test]
    fn skill_location_inventory_scans_global_and_project_dirs() {
        let temp = tempdir().unwrap();
        let home = temp.path().join("home");
        let project = temp.path().join("project");
        fs::create_dir_all(home.join(".config/opencode/skills/release")).unwrap();
        fs::create_dir_all(project.join(".opencode/skills/debug")).unwrap();
        fs::write(
            home.join(".config/opencode/skills/release/SKILL.md"),
            "# release",
        )
        .unwrap();
        fs::write(project.join(".opencode/skills/debug/SKILL.md"), "# debug").unwrap();

        let locations = build_skill_locations(&home, Some(project.as_path()));
        let global_opencode = locations
            .iter()
            .find(|item| item.scope == "global" && item.kind == "opencode")
            .unwrap();
        let project_opencode = locations
            .iter()
            .find(|item| item.scope == "project" && item.kind == "opencode")
            .unwrap();

        assert_eq!(global_opencode.skill_count, 1);
        assert_eq!(project_opencode.skill_count, 1);
        assert_eq!(global_opencode.skills, vec!["release".to_string()]);
        assert_eq!(project_opencode.skills, vec!["debug".to_string()]);
    }

    #[test]
    fn plugin_scan_only_includes_script_files() {
        let temp = tempdir().unwrap();
        let plugins_dir = temp.path().join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();
        fs::write(plugins_dir.join("notify.ts"), "export default {}").unwrap();
        fs::write(plugins_dir.join("README.md"), "ignored").unwrap();

        let plugins = scan_plugin_files(&plugins_dir, "global").unwrap();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "notify.ts");
    }
}
