use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use ccr_config::ClaudeRuntimePaths;
use ccr_core::core::{
    BackupPolicy, VersionedWriteOutcome, WriteOptions, content_version_token,
    write_guarded_versioned,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

const MCP_UPDATE_ATTEMPTS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClaudeMcpScope {
    Local,
    Project,
    User,
}

impl ClaudeMcpScope {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Project => "project",
            Self::User => "user",
        }
    }

    fn from_value(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "local" => Some(Self::Local),
            "project" => Some(Self::Project),
            "user" | "global" => Some(Self::User),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ClaudeMcpContext {
    pub user_state_path: PathBuf,
    pub project_root: PathBuf,
}

impl ClaudeMcpContext {
    pub(crate) fn detect() -> Result<Self, String> {
        let user_state_path = ClaudeRuntimePaths::from_env()
            .map_err(|error| format!("Resolve Claude runtime paths: {error}"))?
            .state_file;
        let project_root = detect_project_root()?;
        Ok(Self {
            user_state_path,
            project_root,
        })
    }

    fn claude_json_path(&self) -> PathBuf {
        self.user_state_path.clone()
    }

    fn project_mcp_path(&self) -> PathBuf {
        self.project_root.join(".mcp.json")
    }

    fn project_settings_path(&self) -> PathBuf {
        self.project_root.join(".claude").join("settings.json")
    }

    fn project_local_settings_path(&self) -> PathBuf {
        self.project_root
            .join(".claude")
            .join("settings.local.json")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ClaudeMcpServer {
    pub platform: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_tools: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub server_type: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_state: Option<String>,
    #[serde(default)]
    pub effective: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_config: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ClaudeMcpDiagnostic {
    pub level: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ClaudeMcpList {
    pub servers: Vec<ClaudeMcpServer>,
    pub diagnostics: Vec<ClaudeMcpDiagnostic>,
}

#[derive(Debug, Default)]
struct ProjectApproval {
    enable_all: bool,
    enabled: HashSet<String>,
    disabled: HashSet<String>,
    diagnostics: Vec<ClaudeMcpDiagnostic>,
}

#[derive(Debug)]
struct ScopedConfig {
    scope: ClaudeMcpScope,
    path: PathBuf,
    servers: Map<String, Value>,
}

fn detect_project_root() -> Result<PathBuf, String> {
    for key in ["CCR_PROJECT_DIR", "CLAUDE_PROJECT_DIR"] {
        if let Ok(value) = std::env::var(key)
            && !value.trim().is_empty()
        {
            return normalize_existing_or_raw_path(PathBuf::from(value));
        }
    }

    let current = std::env::current_dir().map_err(|e| format!("current_dir: {e}"))?;
    let mut cursor = current.as_path();
    loop {
        if cursor.join(".git").exists() || cursor.join(".mcp.json").exists() {
            return normalize_existing_or_raw_path(cursor.to_path_buf());
        }
        match cursor.parent() {
            Some(parent) => cursor = parent,
            None => break,
        }
    }

    normalize_existing_or_raw_path(current)
}

fn normalize_existing_or_raw_path(path: PathBuf) -> Result<PathBuf, String> {
    match path.canonicalize() {
        Ok(path) => Ok(path),
        Err(_) => Ok(path),
    }
}

fn read_json_object(path: &Path) -> Result<Map<String, Value>, String> {
    read_versioned_json_object(path).map(|(object, _)| object)
}

fn read_versioned_json_object(path: &Path) -> Result<(Map<String, Value>, String), String> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok((Map::new(), String::new()));
        }
        Err(error) => return Err(format!("Read {}: {error}", path.display())),
    };
    let token = content_version_token(&bytes);
    if bytes.iter().all(|byte| byte.is_ascii_whitespace()) {
        return Ok((Map::new(), token));
    }

    let value: Value =
        serde_json::from_slice(&bytes).map_err(|error| format!("Parse {}: {error}", path.display()))?;
    let object = value
        .as_object()
        .cloned()
        .ok_or_else(|| format!("{} must contain a JSON object", path.display()))?;
    Ok((object, token))
}

fn path_for_display(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn normalize_project_key(raw: &str) -> String {
    let mut value = raw.trim().replace('\\', "/");
    if let Some(stripped) = value.strip_prefix("//?/") {
        value = stripped.to_string();
    }
    while value.ends_with('/') && value.len() > 1 {
        value.pop();
    }
    value.to_ascii_lowercase()
}

fn project_key_variants(project_root: &Path) -> Vec<String> {
    let mut variants = vec![path_for_display(project_root)];
    if let Ok(canonical) = project_root.canonicalize() {
        variants.push(path_for_display(&canonical));
    }
    variants
        .into_iter()
        .flat_map(|value| {
            let slash = value.replace('\\', "/");
            let backslash = value.replace('/', "\\");
            [value, slash, backslash]
        })
        .collect()
}

fn find_project_key(projects: &Map<String, Value>, project_root: &Path) -> Option<String> {
    let normalized_candidates: HashSet<String> = project_key_variants(project_root)
        .into_iter()
        .map(|value| normalize_project_key(&value))
        .collect();

    projects
        .keys()
        .find(|key| normalized_candidates.contains(&normalize_project_key(key)))
        .cloned()
}

fn read_mcp_servers_from_object(object: &Map<String, Value>) -> Map<String, Value> {
    object
        .get("mcpServers")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

fn read_scoped_configs(
    ctx: &ClaudeMcpContext,
) -> Result<(Vec<ScopedConfig>, Vec<ClaudeMcpDiagnostic>), String> {
    let mut configs = Vec::new();
    let mut diagnostics = Vec::new();
    let claude_path = ctx.claude_json_path();
    let claude_root = read_json_object(&claude_path)?;

    let projects = claude_root
        .get("projects")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let matched_project_key = find_project_key(&projects, &ctx.project_root);

    if let Some(project_key) = &matched_project_key {
        let servers = projects
            .get(project_key)
            .and_then(Value::as_object)
            .map(read_mcp_servers_from_object)
            .unwrap_or_default();
        diagnostics.push(ClaudeMcpDiagnostic {
            level: "info".into(),
            message: format!("Matched Claude local project key: {project_key}"),
            source_path: Some(path_for_display(&claude_path)),
            scope: Some("local".into()),
            matched: Some(true),
        });
        configs.push(ScopedConfig {
            scope: ClaudeMcpScope::Local,
            path: claude_path.clone(),
            servers,
        });
    } else {
        diagnostics.push(ClaudeMcpDiagnostic {
            level: "info".into(),
            message: format!(
                "No local MCP entry matched current project {}",
                ctx.project_root.display()
            ),
            source_path: Some(path_for_display(&claude_path)),
            scope: Some("local".into()),
            matched: Some(false),
        });
        configs.push(ScopedConfig {
            scope: ClaudeMcpScope::Local,
            path: claude_path.clone(),
            servers: Map::new(),
        });
    }

    let project_path = ctx.project_mcp_path();
    let project_root = read_json_object(&project_path)?;
    configs.push(ScopedConfig {
        scope: ClaudeMcpScope::Project,
        path: project_path,
        servers: read_mcp_servers_from_object(&project_root),
    });

    configs.push(ScopedConfig {
        scope: ClaudeMcpScope::User,
        path: claude_path.clone(),
        servers: read_mcp_servers_from_object(&claude_root),
    });

    Ok((configs, diagnostics))
}

fn read_string_array(object: &Map<String, Value>, key: &str) -> HashSet<String> {
    object
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn read_project_approval(ctx: &ClaudeMcpContext) -> Result<ProjectApproval, String> {
    let mut approval = ProjectApproval::default();

    for path in [
        ctx.project_settings_path(),
        ctx.project_local_settings_path(),
    ] {
        let object = read_json_object(&path)?;
        if object.is_empty() && !path.exists() {
            approval.diagnostics.push(ClaudeMcpDiagnostic {
                level: "info".into(),
                message: "Claude project settings file not found".into(),
                source_path: Some(path_for_display(&path)),
                scope: Some("project".into()),
                matched: None,
            });
            continue;
        }

        if let Some(flag) = object
            .get("enableAllProjectMcpServers")
            .and_then(Value::as_bool)
        {
            approval.enable_all = flag;
        }

        approval
            .enabled
            .extend(read_string_array(&object, "enabledMcpjsonServers"));
        approval
            .disabled
            .extend(read_string_array(&object, "disabledMcpjsonServers"));

        approval.diagnostics.push(ClaudeMcpDiagnostic {
            level: "info".into(),
            message: "Read Claude project MCP approval settings".into(),
            source_path: Some(path_for_display(&path)),
            scope: Some("project".into()),
            matched: Some(true),
        });
    }

    Ok(approval)
}

fn approval_state(name: &str, approval: &ProjectApproval) -> String {
    if approval.disabled.contains(name) {
        "disabled".into()
    } else if approval.enable_all || approval.enabled.contains(name) {
        "approved".into()
    } else {
        "pending".into()
    }
}

fn as_string_map(value: Option<&Value>) -> HashMap<String, String> {
    value
        .and_then(Value::as_object)
        .map(|object| {
            object
                .iter()
                .map(|(key, value)| {
                    let text = value
                        .as_str()
                        .map(ToString::to_string)
                        .unwrap_or_else(|| value.to_string());
                    (key.clone(), mask_secret(&text))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn as_string_vec(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn mask_secret(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return String::new();
    }

    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 6 {
        "••••••".into()
    } else if chars.len() <= 12 {
        format!(
            "{}••••{}",
            chars.iter().take(2).collect::<String>(),
            chars
                .iter()
                .rev()
                .take(2)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<String>()
        )
    } else {
        format!(
            "{}••••••{}",
            chars.iter().take(4).collect::<String>(),
            chars
                .iter()
                .rev()
                .take(4)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<String>()
        )
    }
}

fn is_masked_preview(value: &str) -> bool {
    value.contains('•')
}

fn sanitized_config(config: &Value) -> Value {
    let Some(object) = config.as_object() else {
        return Value::Object(Map::new());
    };

    let mut sanitized = object.clone();
    for key in ["env", "headers"] {
        if let Some(Value::Object(values)) = sanitized.get_mut(key) {
            for value in values.values_mut() {
                if let Some(text) = value.as_str() {
                    *value = Value::String(mask_secret(text));
                }
            }
        }
    }
    Value::Object(sanitized)
}

fn server_from_entry(
    name: &str,
    raw: &Value,
    scoped: &ScopedConfig,
    approval: &ProjectApproval,
) -> ClaudeMcpServer {
    let approval_state =
        (scoped.scope == ClaudeMcpScope::Project).then(|| approval_state(name, approval));

    ClaudeMcpServer {
        platform: "claude".into(),
        name: name.to_string(),
        command: raw
            .get("command")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        url: raw
            .get("url")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        args: as_string_vec(raw.get("args")),
        env: as_string_map(raw.get("env")),
        headers: as_string_map(raw.get("headers")),
        timeout: raw.get("timeout").and_then(Value::as_i64),
        cwd: raw
            .get("cwd")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        trust: raw.get("trust").and_then(Value::as_bool),
        include_tools: as_string_vec(raw.get("include_tools").or_else(|| raw.get("includeTools"))),
        server_type: raw
            .get("type")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        disabled: raw
            .get("disabled")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        scope: scoped.scope.as_str().into(),
        source_path: Some(path_for_display(&scoped.path)),
        approval_state,
        effective: false,
        hidden_by: None,
        raw_config: Some(sanitized_config(raw)),
    }
}

fn can_be_effective(server: &ClaudeMcpServer) -> bool {
    !server.disabled
        && !matches!(
            server.approval_state.as_deref(),
            Some("pending") | Some("disabled")
        )
}

pub(crate) fn list_claude_mcp(ctx: &ClaudeMcpContext) -> Result<ClaudeMcpList, String> {
    let (configs, mut diagnostics) = read_scoped_configs(ctx)?;
    let mut approval = read_project_approval(ctx)?;
    diagnostics.append(&mut approval.diagnostics);

    let mut servers: Vec<ClaudeMcpServer> = Vec::new();
    let mut selected_by_name: HashMap<String, usize> = HashMap::new();

    for scoped in configs {
        diagnostics.push(ClaudeMcpDiagnostic {
            level: "info".into(),
            message: format!(
                "Read {} MCP scope with {} server(s)",
                scoped.scope.as_str(),
                scoped.servers.len()
            ),
            source_path: Some(path_for_display(&scoped.path)),
            scope: Some(scoped.scope.as_str().into()),
            matched: Some(!scoped.servers.is_empty()),
        });

        for (name, raw) in &scoped.servers {
            let mut server = server_from_entry(name, raw, &scoped, &approval);
            if let Some(selected_index) = selected_by_name.get(name).copied() {
                let selected = &servers[selected_index];
                server.hidden_by = Some(format!("{}:{}", selected.scope, selected.name));
            } else {
                selected_by_name.insert(name.clone(), servers.len());
                server.effective = can_be_effective(&server);
            }
            servers.push(server);
        }
    }

    Ok(ClaudeMcpList {
        servers,
        diagnostics,
    })
}

pub(crate) fn list_claude_mcp_default() -> Result<ClaudeMcpList, String> {
    let ctx = ClaudeMcpContext::detect()?;
    list_claude_mcp(&ctx)
}

fn root_path_for_scope(ctx: &ClaudeMcpContext, scope: ClaudeMcpScope) -> PathBuf {
    match scope {
        ClaudeMcpScope::Local | ClaudeMcpScope::User => ctx.claude_json_path(),
        ClaudeMcpScope::Project => ctx.project_mcp_path(),
    }
}

fn write_options_for_scope(scope: ClaudeMcpScope) -> WriteOptions {
    WriteOptions {
        backup: BackupPolicy::None,
        secret: !matches!(scope, ClaudeMcpScope::Project),
        ..Default::default()
    }
}

fn update_root_for_scope<T, F>(
    ctx: &ClaudeMcpContext,
    scope: ClaudeMcpScope,
    mut update: F,
) -> Result<T, String>
where
    F: FnMut(&mut Map<String, Value>) -> Result<T, String>,
{
    let path = root_path_for_scope(ctx, scope);
    let options = write_options_for_scope(scope);

    for _ in 0..MCP_UPDATE_ATTEMPTS {
        let (mut root, expected_token) = read_versioned_json_object(&path)?;
        let result = update(&mut root)?;
        let bytes = serde_json::to_vec_pretty(&root)
            .map_err(|error| format!("Serialize {}: {error}", path.display()))?;

        match write_guarded_versioned(&path, &bytes, &expected_token, &options)
            .map_err(|error| format!("Write {}: {error}", path.display()))?
        {
            VersionedWriteOutcome::Written => return Ok(result),
            VersionedWriteOutcome::Conflict => continue,
        }
    }

    Err(format!(
        "Claude MCP {} scope changed concurrently after {MCP_UPDATE_ATTEMPTS} attempts; retry the operation",
        scope.as_str()
    ))
}

fn ensure_object_field<'a>(
    object: &'a mut Map<String, Value>,
    key: &str,
) -> Result<&'a mut Map<String, Value>, String> {
    let entry = object
        .entry(key.to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    entry
        .as_object_mut()
        .ok_or_else(|| format!("{key} must be a JSON object"))
}

fn mcp_servers_for_write<'a>(
    ctx: &ClaudeMcpContext,
    scope: ClaudeMcpScope,
    root: &'a mut Map<String, Value>,
) -> Result<&'a mut Map<String, Value>, String> {
    match scope {
        ClaudeMcpScope::User | ClaudeMcpScope::Project => ensure_object_field(root, "mcpServers"),
        ClaudeMcpScope::Local => {
            let projects = ensure_object_field(root, "projects")?;
            let project_key = find_project_key(projects, &ctx.project_root)
                .unwrap_or_else(|| path_for_display(&ctx.project_root));
            let project_entry = projects
                .entry(project_key)
                .or_insert_with(|| Value::Object(Map::new()));
            let project_object = project_entry
                .as_object_mut()
                .ok_or_else(|| "projects entry must be a JSON object".to_string())?;
            ensure_object_field(project_object, "mcpServers")
        }
    }
}

fn remove_nulls(value: &mut Value) {
    if let Value::Object(object) = value {
        object.retain(|_, value| !value.is_null());
    }
}

fn clean_config(config: Value) -> Result<Value, String> {
    clean_config_with_null_policy(config, true)
}

fn clean_config_for_patch(config: Value) -> Result<Value, String> {
    clean_config_with_null_policy(config, false)
}

fn clean_config_with_null_policy(
    mut config: Value,
    remove_null_fields: bool,
) -> Result<Value, String> {
    let object = config
        .as_object_mut()
        .ok_or_else(|| "MCP server config must be a JSON object".to_string())?;

    for key in [
        "platform",
        "name",
        "scope",
        "source_path",
        "approval_state",
        "effective",
        "hidden_by",
        "raw_config",
    ] {
        object.remove(key);
    }
    normalize_config_transport(&mut config);
    if remove_null_fields {
        remove_nulls(&mut config);
    }
    Ok(config)
}

fn normalize_config_transport(config: &mut Value) {
    let Some(object) = config.as_object_mut() else {
        return;
    };

    let has_url = object
        .get("url")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty());
    let has_command = object
        .get("command")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty());

    if has_url {
        object
            .entry("type")
            .or_insert_with(|| Value::String("http".into()));
        object.insert("command".into(), Value::Null);
        object.insert("args".into(), Value::Null);
    } else if has_command {
        object.insert("url".into(), Value::Null);
    }
}

fn scope_from_config(config: &Value) -> Option<ClaudeMcpScope> {
    config
        .get("scope")
        .and_then(Value::as_str)
        .and_then(ClaudeMcpScope::from_value)
}

fn resolve_existing_scope(
    ctx: &ClaudeMcpContext,
    name: &str,
    requested_scope: Option<ClaudeMcpScope>,
) -> Result<ClaudeMcpScope, String> {
    if let Some(scope) = requested_scope {
        return Ok(scope);
    }

    let (configs, _) = read_scoped_configs(ctx)?;
    configs
        .iter()
        .find(|scoped| scoped.servers.contains_key(name))
        .map(|scoped| scoped.scope)
        .ok_or_else(|| format!("MCP server '{name}' not found"))
}

fn merge_secret_object(existing: Option<&Value>, incoming: &mut Value) {
    let (Some(existing_object), Some(incoming_object)) = (
        existing.and_then(Value::as_object),
        incoming.as_object_mut(),
    ) else {
        return;
    };

    for (key, value) in incoming_object.iter_mut() {
        if value.as_str().is_some_and(is_masked_preview)
            && let Some(existing_value) = existing_object.get(key)
        {
            *value = existing_value.clone();
        }
    }
}

fn merge_object_patch(existing: Option<&Value>, incoming: &mut Value) {
    let (Some(existing_object), Some(incoming_object)) = (
        existing.and_then(Value::as_object),
        incoming.as_object_mut(),
    ) else {
        return;
    };

    let mut merged = existing_object.clone();
    merged.extend(incoming_object.clone());
    *incoming_object = merged;
}

fn should_replace_existing_value(key: &str, value: &Value) -> bool {
    match key {
        "args" | "include_tools" | "includeTools" => !value
            .as_array()
            .map(|items| items.is_empty())
            .unwrap_or(false),
        "env" | "headers" => value.as_object().is_some_and(|object| !object.is_empty()),
        "command" | "cwd" | "type" | "url" => value.as_str().map(|s| !s.is_empty()).unwrap_or(true),
        _ => true,
    }
}

fn merge_mcp_config(existing: &mut Value, patch: Value) -> Result<(), String> {
    let existing_object = existing
        .as_object_mut()
        .ok_or_else(|| "Existing MCP server config must be a JSON object".to_string())?;
    let mut patch = clean_config_for_patch(patch)?;
    let patch_object = patch
        .as_object_mut()
        .ok_or_else(|| "MCP server patch must be a JSON object".to_string())?;

    for key in ["env", "headers"] {
        if let Some(value) = patch_object.get_mut(key) {
            merge_object_patch(existing_object.get(key), value);
            merge_secret_object(existing_object.get(key), value);
        }
    }

    for (key, value) in patch_object.iter() {
        if value.is_null() {
            existing_object.remove(key);
        } else if should_replace_existing_value(key, value) {
            existing_object.insert(key.clone(), value.clone());
        }
    }
    Ok(())
}

pub(crate) fn add_claude_mcp_server(
    ctx: &ClaudeMcpContext,
    name: String,
    config: Value,
    requested_scope: Option<ClaudeMcpScope>,
) -> Result<Value, String> {
    let scope = requested_scope
        .or_else(|| scope_from_config(&config))
        .unwrap_or(ClaudeMcpScope::User);
    let clean = clean_config(config)?;
    update_root_for_scope(ctx, scope, |root| {
        let servers = mcp_servers_for_write(ctx, scope, root)?;
        servers.insert(name.clone(), clean.clone());
        Ok(())
    })?;

    Ok(json!({
        "success": true,
        "message": format!("MCP server '{name}' added to Claude {} scope", scope.as_str()),
        "scope": scope.as_str(),
    }))
}

pub(crate) fn add_claude_mcp_server_default(
    name: String,
    config: Value,
    requested_scope: Option<ClaudeMcpScope>,
) -> Result<Value, String> {
    let ctx = ClaudeMcpContext::detect()?;
    add_claude_mcp_server(&ctx, name, config, requested_scope)
}

pub(crate) fn update_claude_mcp_server(
    ctx: &ClaudeMcpContext,
    name: String,
    patch: Value,
    requested_scope: Option<ClaudeMcpScope>,
) -> Result<Value, String> {
    let scope = resolve_existing_scope(
        ctx,
        &name,
        requested_scope.or_else(|| scope_from_config(&patch)),
    )?;
    update_root_for_scope(ctx, scope, |root| {
        let servers = mcp_servers_for_write(ctx, scope, root)?;
        let existing = servers.get_mut(&name).ok_or_else(|| {
            format!(
                "MCP server '{name}' not found in Claude {} scope",
                scope.as_str()
            )
        })?;
        merge_mcp_config(existing, patch.clone())
    })?;

    Ok(json!({
        "success": true,
        "message": format!("MCP server '{name}' updated in Claude {} scope", scope.as_str()),
        "scope": scope.as_str(),
    }))
}

pub(crate) fn update_claude_mcp_server_default(
    name: String,
    patch: Value,
    requested_scope: Option<ClaudeMcpScope>,
) -> Result<Value, String> {
    let ctx = ClaudeMcpContext::detect()?;
    update_claude_mcp_server(&ctx, name, patch, requested_scope)
}

pub(crate) fn delete_claude_mcp_server(
    ctx: &ClaudeMcpContext,
    name: String,
    requested_scope: Option<ClaudeMcpScope>,
) -> Result<String, String> {
    let scope = resolve_existing_scope(ctx, &name, requested_scope)?;
    update_root_for_scope(ctx, scope, |root| {
        let servers = mcp_servers_for_write(ctx, scope, root)?;
        if servers.remove(&name).is_none() {
            return Err(format!(
                "MCP server '{name}' not found in Claude {} scope",
                scope.as_str()
            ));
        }
        Ok(())
    })?;

    Ok(format!(
        "MCP server '{name}' deleted from Claude {} scope",
        scope.as_str()
    ))
}

pub(crate) fn delete_claude_mcp_server_default(
    name: String,
    requested_scope: Option<ClaudeMcpScope>,
) -> Result<String, String> {
    let ctx = ClaudeMcpContext::detect()?;
    delete_claude_mcp_server(&ctx, name, requested_scope)
}

pub(crate) fn parse_scope(value: Option<&str>) -> Option<ClaudeMcpScope> {
    value.and_then(ClaudeMcpScope::from_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TestProcessEnv;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};
    use tempfile::TempDir;

    fn write_json(path: &Path, value: Value) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
    }

    fn test_context() -> (TestProcessEnv, TempDir, TempDir, ClaudeMcpContext) {
        let home = tempfile::tempdir().unwrap();
        let project = tempfile::tempdir().unwrap();
        let mut process_env = TestProcessEnv::new();
        process_env.set("CCR_LOCK_DIR", home.path().join("locks").as_os_str());
        let ctx = ClaudeMcpContext {
            user_state_path: home.path().join(".claude.json"),
            project_root: project.path().to_path_buf(),
        };
        (process_env, home, project, ctx)
    }

    #[test]
    fn custom_config_dir_moves_only_the_user_state_path() {
        let home = tempfile::tempdir().unwrap();
        let project = tempfile::tempdir().unwrap();
        let config_dir = home.path().join("claude-custom");
        let paths = ClaudeRuntimePaths::resolve_with(home.path(), |key| {
            (key == "CLAUDE_CONFIG_DIR").then(|| config_dir.as_os_str().to_owned())
        });
        let ctx = ClaudeMcpContext {
            user_state_path: paths.state_file,
            project_root: project.path().to_path_buf(),
        };

        assert_eq!(ctx.claude_json_path(), config_dir.join(".claude.json"));
        assert_eq!(ctx.project_mcp_path(), project.path().join(".mcp.json"));
        assert_eq!(
            ctx.project_settings_path(),
            project.path().join(".claude/settings.json")
        );
    }

    #[test]
    fn claude_mcp_lists_user_local_project_and_marks_precedence_with_windows_key() {
        let (_process_env, _home, project, ctx) = test_context();
        let forward_project_key = path_for_display(project.path()).replace('\\', "/");

        write_json(
            &ctx.claude_json_path(),
            json!({
                "mcpServers": {
                    "exa": { "type": "http", "url": "https://user.example/mcp" },
                    "user-only": { "command": "uvx", "args": ["user"] }
                },
                "projects": {
                    forward_project_key: {
                        "mcpServers": {
                            "exa": { "type": "http", "url": "https://local.example/mcp" },
                            "local-only": { "command": "node", "args": ["server.js"] }
                        }
                    }
                }
            }),
        );
        write_json(
            &ctx.project_mcp_path(),
            json!({
                "mcpServers": {
                    "exa": { "type": "http", "url": "https://project.example/mcp" },
                    "project-only": { "command": "npx", "args": ["project"] }
                }
            }),
        );
        write_json(
            &ctx.project_settings_path(),
            json!({
                "enableAllProjectMcpServers": true
            }),
        );

        let listed = list_claude_mcp(&ctx).unwrap();
        let exa_entries: Vec<_> = listed
            .servers
            .iter()
            .filter(|server| server.name == "exa")
            .collect();

        assert_eq!(exa_entries.len(), 3);
        assert_eq!(exa_entries[0].scope, "local");
        assert_eq!(
            exa_entries[0].url.as_deref(),
            Some("https://local.example/mcp")
        );
        assert!(exa_entries[0].effective);
        assert_eq!(exa_entries[1].scope, "project");
        assert_eq!(exa_entries[1].hidden_by.as_deref(), Some("local:exa"));
        assert_eq!(exa_entries[2].scope, "user");
        assert_eq!(exa_entries[2].hidden_by.as_deref(), Some("local:exa"));
        assert!(listed.diagnostics.iter().any(|d| d.matched == Some(true)));
    }

    #[test]
    fn claude_mcp_reports_project_approval_states() {
        let (_process_env, _home, _project, ctx) = test_context();
        write_json(
            &ctx.project_mcp_path(),
            json!({
                "mcpServers": {
                    "memory": { "command": "npx", "args": ["memory"] },
                    "filesystem": { "command": "npx", "args": ["filesystem"] },
                    "pending": { "command": "npx", "args": ["pending"] }
                }
            }),
        );
        write_json(
            &ctx.project_local_settings_path(),
            json!({
                "enabledMcpjsonServers": ["memory"],
                "disabledMcpjsonServers": ["filesystem"]
            }),
        );

        let listed = list_claude_mcp(&ctx).unwrap();
        let approval = |name: &str| {
            listed
                .servers
                .iter()
                .find(|server| server.name == name)
                .and_then(|server| server.approval_state.as_deref())
                .map(ToString::to_string)
                .unwrap()
        };

        assert_eq!(approval("memory"), "approved");
        assert_eq!(approval("filesystem"), "disabled");
        assert_eq!(approval("pending"), "pending");
    }

    #[test]
    fn claude_mcp_toggle_update_preserves_command_args_and_env() {
        let (_process_env, _home, _project, ctx) = test_context();
        write_json(
            &ctx.claude_json_path(),
            json!({
                "mcpServers": {
                    "exa": {
                        "command": "npx",
                        "args": ["-y", "exa-mcp-server"],
                        "env": { "EXA_API_KEY": "sk-real-secret" }
                    }
                }
            }),
        );

        update_claude_mcp_server(
            &ctx,
            "exa".into(),
            json!({ "disabled": true }),
            Some(ClaudeMcpScope::User),
        )
        .unwrap();

        let root = read_json_object(&ctx.claude_json_path()).unwrap();
        let server = root["mcpServers"]["exa"].as_object().unwrap();
        assert_eq!(server["command"], json!("npx"));
        assert_eq!(server["args"], json!(["-y", "exa-mcp-server"]));
        assert_eq!(server["env"]["EXA_API_KEY"], json!("sk-real-secret"));
        assert_eq!(server["disabled"], json!(true));
    }

    #[test]
    fn claude_mcp_empty_update_patch_does_not_clear_existing_config() {
        let (_process_env, _home, _project, ctx) = test_context();
        write_json(
            &ctx.claude_json_path(),
            json!({
                "mcpServers": {
                    "exa": {
                        "command": "npx",
                        "args": ["-y", "exa-mcp-server"],
                        "env": { "EXA_API_KEY": "sk-real-secret" }
                    }
                }
            }),
        );

        update_claude_mcp_server(
            &ctx,
            "exa".into(),
            json!({
                "args": [],
                "env": {},
                "headers": {},
                "include_tools": [],
                "disabled": false
            }),
            Some(ClaudeMcpScope::User),
        )
        .unwrap();

        let root = read_json_object(&ctx.claude_json_path()).unwrap();
        assert_eq!(
            root["mcpServers"]["exa"]["args"],
            json!(["-y", "exa-mcp-server"])
        );
        assert_eq!(
            root["mcpServers"]["exa"]["env"]["EXA_API_KEY"],
            json!("sk-real-secret")
        );
        assert_eq!(root["mcpServers"]["exa"]["disabled"], json!(false));
    }

    #[test]
    fn claude_mcp_add_creates_missing_mcp_servers_object() {
        let (_process_env, _home, _project, ctx) = test_context();
        write_json(&ctx.claude_json_path(), json!({ "other": true }));

        add_claude_mcp_server(
            &ctx,
            "exa".into(),
            json!({ "type": "http", "url": "https://mcp.exa.ai/mcp" }),
            Some(ClaudeMcpScope::User),
        )
        .unwrap();

        let root = read_json_object(&ctx.claude_json_path()).unwrap();
        assert_eq!(
            root["mcpServers"]["exa"]["url"],
            json!("https://mcp.exa.ai/mcp")
        );
        assert_eq!(root["other"], json!(true));
    }

    #[test]
    fn claude_mcp_masked_env_is_not_written_back_over_real_secret() {
        let (_process_env, _home, _project, ctx) = test_context();
        write_json(
            &ctx.claude_json_path(),
            json!({
                "mcpServers": {
                    "exa": {
                        "command": "npx",
                        "env": { "EXA_API_KEY": "sk-real-secret" }
                    }
                }
            }),
        );

        update_claude_mcp_server(
            &ctx,
            "exa".into(),
            json!({
                "env": { "EXA_API_KEY": "sk-r••••••cret" },
                "args": ["-y", "exa-mcp-server"]
            }),
            Some(ClaudeMcpScope::User),
        )
        .unwrap();

        let root = read_json_object(&ctx.claude_json_path()).unwrap();
        assert_eq!(
            root["mcpServers"]["exa"]["env"]["EXA_API_KEY"],
            json!("sk-real-secret")
        );
        assert_eq!(
            root["mcpServers"]["exa"]["args"],
            json!(["-y", "exa-mcp-server"])
        );
    }

    #[test]
    fn claude_mcp_env_patch_merges_instead_of_replacing_existing_keys() {
        let (_process_env, _home, _project, ctx) = test_context();
        write_json(
            &ctx.claude_json_path(),
            json!({
                "mcpServers": {
                    "exa": {
                        "command": "npx",
                        "env": {
                            "EXA_API_KEY": "sk-real-secret",
                            "EXA_REGION": "us"
                        }
                    }
                }
            }),
        );

        update_claude_mcp_server(
            &ctx,
            "exa".into(),
            json!({
                "env": { "EXA_TIMEOUT": "30" }
            }),
            Some(ClaudeMcpScope::User),
        )
        .unwrap();

        let root = read_json_object(&ctx.claude_json_path()).unwrap();
        assert_eq!(
            root["mcpServers"]["exa"]["env"]["EXA_API_KEY"],
            json!("sk-real-secret")
        );
        assert_eq!(root["mcpServers"]["exa"]["env"]["EXA_REGION"], json!("us"));
        assert_eq!(root["mcpServers"]["exa"]["env"]["EXA_TIMEOUT"], json!("30"));
    }

    #[test]
    fn claude_mcp_add_normalizes_http_transport_without_empty_command() {
        let (_process_env, _home, _project, ctx) = test_context();

        add_claude_mcp_server(
            &ctx,
            "exa".into(),
            json!({
                "command": "",
                "url": "https://mcp.exa.ai/mcp",
                "args": [],
                "headers": { "Authorization": "Bearer token" }
            }),
            Some(ClaudeMcpScope::User),
        )
        .unwrap();

        let root = read_json_object(&ctx.claude_json_path()).unwrap();
        let server = root["mcpServers"]["exa"].as_object().unwrap();
        assert_eq!(server["type"], json!("http"));
        assert_eq!(server["url"], json!("https://mcp.exa.ai/mcp"));
        assert!(!server.contains_key("command"));
        assert!(!server.contains_key("args"));
        assert_eq!(server["headers"]["Authorization"], json!("Bearer token"));
    }

    #[test]
    fn claude_mcp_update_can_switch_from_stdio_to_http() {
        let (_process_env, _home, _project, ctx) = test_context();
        write_json(
            &ctx.claude_json_path(),
            json!({
                "mcpServers": {
                    "exa": {
                        "command": "npx",
                        "args": ["-y", "exa-mcp-server"],
                        "env": { "EXA_API_KEY": "sk-real-secret" }
                    }
                }
            }),
        );

        update_claude_mcp_server(
            &ctx,
            "exa".into(),
            json!({
                "command": null,
                "url": "https://mcp.exa.ai/mcp",
                "args": null,
                "headers": { "Authorization": "Bearer token" }
            }),
            Some(ClaudeMcpScope::User),
        )
        .unwrap();

        let root = read_json_object(&ctx.claude_json_path()).unwrap();
        let server = root["mcpServers"]["exa"].as_object().unwrap();
        assert_eq!(server["type"], json!("http"));
        assert_eq!(server["url"], json!("https://mcp.exa.ai/mcp"));
        assert_eq!(server["env"]["EXA_API_KEY"], json!("sk-real-secret"));
        assert_eq!(server["headers"]["Authorization"], json!("Bearer token"));
        assert!(!server.contains_key("command"));
        assert!(!server.contains_key("args"));
    }

    #[test]
    fn claude_mcp_user_mutations_preserve_unrelated_state_fields() {
        let (_process_env, _home, _project, ctx) = test_context();
        write_json(
            &ctx.claude_json_path(),
            json!({
                "oauthAccount": { "emailAddress": "user@example.com" },
                "primaryApiKey": "sk-private-state",
                "customApiKeyResponses": { "approved": ["key-id"] },
                "projects": { "other-project": { "allowedTools": ["Read"] } },
                "unknownTopLevel": { "keep": true },
                "mcpServers": {
                    "existing": { "command": "node", "args": ["existing.js"] }
                }
            }),
        );

        add_claude_mcp_server(
            &ctx,
            "exa".into(),
            json!({ "command": "npx", "args": ["exa"] }),
            Some(ClaudeMcpScope::User),
        )
        .unwrap();
        update_claude_mcp_server(
            &ctx,
            "exa".into(),
            json!({ "disabled": true }),
            Some(ClaudeMcpScope::User),
        )
        .unwrap();
        delete_claude_mcp_server(&ctx, "exa".into(), Some(ClaudeMcpScope::User)).unwrap();

        let root = read_json_object(&ctx.claude_json_path()).unwrap();
        assert_eq!(
            root["oauthAccount"]["emailAddress"],
            json!("user@example.com")
        );
        assert_eq!(root["primaryApiKey"], json!("sk-private-state"));
        assert_eq!(
            root["customApiKeyResponses"]["approved"],
            json!(["key-id"])
        );
        assert_eq!(root["unknownTopLevel"]["keep"], json!(true));
        assert_eq!(
            root["projects"]["other-project"]["allowedTools"],
            json!(["Read"])
        );
        assert_eq!(
            root["mcpServers"]["existing"]["command"],
            json!("node")
        );
        assert!(root["mcpServers"].get("exa").is_none());
    }

    #[test]
    fn claude_mcp_local_mutation_preserves_other_project_state() {
        let (_process_env, _home, project, ctx) = test_context();
        let project_key = path_for_display(project.path());
        write_json(
            &ctx.claude_json_path(),
            json!({
                "oauthAccount": { "accountUuid": "account-123" },
                "projects": {
                    project_key.clone(): {
                        "allowedTools": ["Read"],
                        "mcpServers": {
                            "existing": { "command": "node" }
                        }
                    },
                    "other-project": { "keep": true }
                },
                "unknownTopLevel": true
            }),
        );

        add_claude_mcp_server(
            &ctx,
            "local-new".into(),
            json!({ "command": "npx", "args": ["local"] }),
            Some(ClaudeMcpScope::Local),
        )
        .unwrap();

        let root = read_json_object(&ctx.claude_json_path()).unwrap();
        assert_eq!(root["oauthAccount"]["accountUuid"], json!("account-123"));
        assert_eq!(root["unknownTopLevel"], json!(true));
        assert_eq!(root["projects"]["other-project"]["keep"], json!(true));
        assert_eq!(
            root["projects"][&project_key]["allowedTools"],
            json!(["Read"])
        );
        assert_eq!(
            root["projects"][&project_key]["mcpServers"]["existing"]["command"],
            json!("node")
        );
        assert_eq!(
            root["projects"][&project_key]["mcpServers"]["local-new"]["command"],
            json!("npx")
        );
    }

    #[test]
    fn claude_mcp_cas_replays_after_one_external_change() {
        let (_process_env, _home, _project, ctx) = test_context();
        let path = ctx.claude_json_path();
        let calls = AtomicUsize::new(0);

        update_root_for_scope(&ctx, ClaudeMcpScope::User, |root| {
            root.insert("ccrMutation".into(), json!(true));
            if calls.fetch_add(1, Ordering::SeqCst) == 0 {
                write_json(&path, json!({ "externalMutation": { "keep": true } }));
            }
            Ok(())
        })
        .unwrap();

        let root = read_json_object(&path).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(root["ccrMutation"], json!(true));
        assert_eq!(root["externalMutation"]["keep"], json!(true));
    }

    #[test]
    fn claude_mcp_cas_fails_after_three_external_changes() {
        let (_process_env, _home, _project, ctx) = test_context();
        let path = ctx.claude_json_path();
        let mut generation = 0;

        let error = update_root_for_scope(&ctx, ClaudeMcpScope::User, |root| {
            root.insert("ccrMutation".into(), json!(true));
            generation += 1;
            write_json(&path, json!({ "externalGeneration": generation }));
            Ok(())
        })
        .unwrap_err();

        let root = read_json_object(&path).unwrap();
        assert!(error.contains("changed concurrently"));
        assert!(error.contains("retry"));
        assert_eq!(generation, MCP_UPDATE_ATTEMPTS);
        assert_eq!(root["externalGeneration"], json!(MCP_UPDATE_ATTEMPTS));
        assert!(root.get("ccrMutation").is_none());
    }

    #[test]
    fn claude_mcp_concurrent_mutations_preserve_both_fields() {
        let (_process_env, _home, _project, ctx) = test_context();
        let first_mutation_ready = Arc::new(Barrier::new(2));
        let second_write_finished = Arc::new(Barrier::new(2));
        let first_calls = Arc::new(AtomicUsize::new(0));
        let first_ctx = ctx.clone();
        let first_thread = {
            let first_mutation_ready = first_mutation_ready.clone();
            let second_write_finished = second_write_finished.clone();
            let first_calls = first_calls.clone();
            std::thread::spawn(move || {
                update_root_for_scope(&first_ctx, ClaudeMcpScope::User, |root| {
                    root.insert("firstWriter".into(), json!(true));
                    if first_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                        first_mutation_ready.wait();
                        second_write_finished.wait();
                    }
                    Ok(())
                })
            })
        };

        first_mutation_ready.wait();
        update_root_for_scope(&ctx, ClaudeMcpScope::User, |root| {
            root.insert("secondWriter".into(), json!(true));
            Ok(())
        })
        .unwrap();
        second_write_finished.wait();
        first_thread.join().unwrap().unwrap();

        let root = read_json_object(&ctx.claude_json_path()).unwrap();
        assert!(first_calls.load(Ordering::SeqCst) >= 2);
        assert_eq!(root["firstWriter"], json!(true));
        assert_eq!(root["secondWriter"], json!(true));
    }

    #[cfg(unix)]
    #[test]
    fn claude_mcp_user_state_is_owner_only_without_same_dir_backup() {
        use std::os::unix::fs::PermissionsExt;

        let (_process_env, _home, _project, ctx) = test_context();
        add_claude_mcp_server(
            &ctx,
            "exa".into(),
            json!({ "command": "npx" }),
            Some(ClaudeMcpScope::User),
        )
        .unwrap();

        let path = ctx.claude_json_path();
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        assert!(
            fs::read_dir(path.parent().unwrap())
                .unwrap()
                .all(|entry| !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".bak"))
        );
    }
}
