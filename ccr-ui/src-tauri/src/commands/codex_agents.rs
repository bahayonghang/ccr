use super::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CodexAgentContextRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexAgentContextPayload {
    pub mode: String,
    pub label: String,
    pub agents_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CodexAgentRecord {
    pub name: String,
    pub file_name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub developer_instructions: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub nickname_candidates: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_effort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills_config: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_toml: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexAgentDiagnostic {
    pub file_name: String,
    pub path: String,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone)]
struct ResolvedCodexAgentContext {
    mode: String,
    label: String,
    agents_dir: PathBuf,
    project_root: Option<PathBuf>,
}

impl ResolvedCodexAgentContext {
    fn payload(&self) -> CodexAgentContextPayload {
        CodexAgentContextPayload {
            mode: self.mode.clone(),
            label: self.label.clone(),
            agents_dir: self.agents_dir.to_string_lossy().to_string(),
            project_root: self
                .project_root
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
        }
    }
}

fn resolve_codex_agent_context(
    context: Option<CodexAgentContextRequest>,
) -> Result<ResolvedCodexAgentContext, String> {
    let context = context.unwrap_or_default();
    let mode = context.mode.unwrap_or_else(|| "global".to_string());

    match mode.as_str() {
        "global" => {
            let agents_dir = codex_agents_dir()?;
            Ok(ResolvedCodexAgentContext {
                mode,
                label: "Global".to_string(),
                agents_dir,
                project_root: None,
            })
        }
        "project" => {
            let project_root = context
                .project_root
                .ok_or_else(|| "Project context requires project_root".to_string())?;
            let project_root = PathBuf::from(project_root);
            if !project_root.exists() {
                return Err(format!(
                    "Project root '{}' does not exist",
                    project_root.to_string_lossy()
                ));
            }

            let agents_dir = project_root.join(".codex").join("agents");
            Ok(ResolvedCodexAgentContext {
                mode,
                label: project_root
                    .file_name()
                    .and_then(|value| value.to_str())
                    .map(|value| format!("Project: {value}"))
                    .unwrap_or_else(|| "Project".to_string()),
                agents_dir,
                project_root: Some(project_root),
            })
        }
        other => Err(format!("Unsupported Codex agent context mode: {other}")),
    }
}

pub(crate) fn ensure_agents_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
        .map_err(|e| format!("创建 agents 目录 '{}' 失败: {e}", path.to_string_lossy()))
}

pub(crate) fn agent_file_path(agents_dir: &Path, name: &str) -> PathBuf {
    agents_dir.join(format!("{name}.toml"))
}

pub(crate) fn read_agent_table(path: &Path) -> Result<(String, toml::value::Table), String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("读取 agent 文件 '{}' 失败: {e}", path.to_string_lossy()))?;
    let value: toml::Value = toml::from_str(&raw)
        .map_err(|e| format!("解析 agent 文件 '{}' 失败: {e}", path.to_string_lossy()))?;
    let table = value.as_table().cloned().ok_or_else(|| {
        format!(
            "Agent 文件 '{}' 顶层必须是 TOML table",
            path.to_string_lossy()
        )
    })?;
    Ok((raw, table))
}

pub(crate) fn optional_string(table: &toml::value::Table, key: &str) -> Option<String> {
    table
        .get(key)
        .and_then(toml::Value::as_str)
        .map(str::to_string)
}

fn optional_json(value: Option<&toml::Value>) -> Option<Value> {
    value.and_then(|item| serde_json::to_value(item).ok())
}

fn remove_known_agent_keys(table: &toml::value::Table) -> toml::value::Table {
    let mut other = table.clone();
    other.remove("name");
    other.remove("description");
    other.remove("developer_instructions");
    other.remove("nickname_candidates");
    other.remove("model");
    other.remove("model_reasoning_effort");
    other.remove("sandbox_mode");
    other.remove("mcp_servers");
    other.remove("skills");
    other
}

pub(crate) fn record_from_table(
    file_path: &Path,
    raw_toml: String,
    table: toml::value::Table,
    parse_error: Option<String>,
) -> CodexAgentRecord {
    let file_name = file_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    let fallback_name = file_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    let name = optional_string(&table, "name").unwrap_or(fallback_name);

    let nickname_candidates = table
        .get("nickname_candidates")
        .and_then(toml::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let skills_config = table
        .get("skills")
        .and_then(toml::Value::as_table)
        .and_then(|skills| skills.get("config"))
        .and_then(|value| serde_json::to_value(value).ok());

    let other = {
        let other = remove_known_agent_keys(&table);
        if other.is_empty() {
            None
        } else {
            serde_json::to_value(other).ok()
        }
    };

    CodexAgentRecord {
        name,
        file_name,
        path: file_path.to_string_lossy().to_string(),
        description: optional_string(&table, "description"),
        developer_instructions: optional_string(&table, "developer_instructions"),
        nickname_candidates,
        model: optional_string(&table, "model"),
        model_reasoning_effort: optional_string(&table, "model_reasoning_effort"),
        sandbox_mode: optional_string(&table, "sandbox_mode"),
        mcp_servers: optional_json(table.get("mcp_servers")),
        skills_config,
        other,
        raw_toml: Some(raw_toml),
        parse_error,
    }
}

fn malformed_record(file_path: &Path, raw_toml: String, error: String) -> CodexAgentRecord {
    let file_name = file_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    let name = file_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();

    CodexAgentRecord {
        name,
        file_name,
        path: file_path.to_string_lossy().to_string(),
        raw_toml: Some(raw_toml),
        parse_error: Some(error),
        ..CodexAgentRecord::default()
    }
}

fn list_codex_agents_for_context(
    context: &ResolvedCodexAgentContext,
) -> Result<(Vec<CodexAgentRecord>, Vec<CodexAgentDiagnostic>), String> {
    if !context.agents_dir.exists() {
        return Ok((Vec::new(), Vec::new()));
    }

    let mut agents = Vec::new();
    let mut diagnostics = Vec::new();

    for entry in
        fs::read_dir(&context.agents_dir).map_err(|e| format!("读取 agents 目录失败: {e}"))?
    {
        let entry = entry.map_err(|e| format!("遍历 agents 目录失败: {e}"))?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("toml") {
            continue;
        }

        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("读取 agent 文件 '{}' 失败: {e}", path.to_string_lossy()))?;

        match toml::from_str::<toml::Value>(&raw) {
            Ok(value) => {
                let table = value.as_table().cloned().ok_or_else(|| {
                    format!(
                        "Agent 文件 '{}' 顶层必须是 TOML table",
                        path.to_string_lossy()
                    )
                })?;
                agents.push(record_from_table(&path, raw, table, None));
            }
            Err(error) => {
                let message = format!("解析 agent 文件失败: {error}");
                diagnostics.push(CodexAgentDiagnostic {
                    file_name: path
                        .file_name()
                        .and_then(|value| value.to_str())
                        .unwrap_or_default()
                        .to_string(),
                    path: path.to_string_lossy().to_string(),
                    severity: "error".to_string(),
                    message: message.clone(),
                });
                agents.push(malformed_record(&path, raw, message));
            }
        }
    }

    agents.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));

    Ok((agents, diagnostics))
}

fn set_or_remove_string(
    table: &mut toml::value::Table,
    config: &Map<String, Value>,
    json_key: &str,
    toml_key: &str,
) -> Result<(), String> {
    if let Some(value) = config.get(json_key) {
        match value {
            Value::Null => {
                table.remove(toml_key);
            }
            Value::String(text) => {
                table.insert(toml_key.to_string(), toml::Value::String(text.clone()));
            }
            _ => {
                return Err(format!("Field '{json_key}' must be a string"));
            }
        }
    }
    Ok(())
}

fn set_or_remove_toml_value(
    table: &mut toml::value::Table,
    config: &Map<String, Value>,
    json_key: &str,
    toml_key: &str,
) -> Result<(), String> {
    if let Some(value) = config.get(json_key) {
        if value.is_null() {
            table.remove(toml_key);
        } else {
            let parsed: toml::Value = serde_json::from_value(value.clone())
                .map_err(|e| format!("Field '{json_key}' cannot be converted to TOML: {e}"))?;
            table.insert(toml_key.to_string(), parsed);
        }
    }
    Ok(())
}

fn set_or_remove_skills_config(
    table: &mut toml::value::Table,
    config: &Map<String, Value>,
) -> Result<(), String> {
    if let Some(value) = config.get("skillsConfig") {
        if value.is_null() {
            match table.get_mut("skills").and_then(toml::Value::as_table_mut) {
                Some(skills_table) => {
                    skills_table.remove("config");
                    if skills_table.is_empty() {
                        table.remove("skills");
                    }
                }
                None => {}
            }
        } else {
            let parsed: toml::Value = serde_json::from_value(value.clone())
                .map_err(|e| format!("Field 'skillsConfig' cannot be converted to TOML: {e}"))?;
            let skills_entry = table
                .entry("skills".to_string())
                .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
            let skills_table = skills_entry
                .as_table_mut()
                .ok_or_else(|| "Existing 'skills' field must be a TOML table".to_string())?;
            skills_table.insert("config".to_string(), parsed);
        }
    }
    Ok(())
}

fn merge_structured_config(
    mut table: toml::value::Table,
    config: &Map<String, Value>,
    fallback_name: &str,
) -> Result<toml::value::Table, String> {
    let requested_name = config
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| {
            optional_string(&table, "name").unwrap_or_else(|| fallback_name.to_string())
        });
    table.insert("name".to_string(), toml::Value::String(requested_name));

    set_or_remove_string(&mut table, config, "description", "description")?;
    set_or_remove_string(
        &mut table,
        config,
        "developerInstructions",
        "developer_instructions",
    )?;
    set_or_remove_string(&mut table, config, "model", "model")?;
    set_or_remove_string(
        &mut table,
        config,
        "modelReasoningEffort",
        "model_reasoning_effort",
    )?;
    set_or_remove_string(&mut table, config, "sandboxMode", "sandbox_mode")?;
    set_or_remove_toml_value(
        &mut table,
        config,
        "nicknameCandidates",
        "nickname_candidates",
    )?;
    set_or_remove_toml_value(&mut table, config, "mcpServers", "mcp_servers")?;
    set_or_remove_skills_config(&mut table, config)?;

    for (key, value) in config {
        if matches!(
            key.as_str(),
            "name"
                | "description"
                | "developerInstructions"
                | "model"
                | "modelReasoningEffort"
                | "sandboxMode"
                | "nicknameCandidates"
                | "mcpServers"
                | "skillsConfig"
                | "rawToml"
        ) {
            continue;
        }

        if value.is_null() {
            table.remove(key);
            continue;
        }

        let parsed: toml::Value = serde_json::from_value(value.clone())
            .map_err(|e| format!("Field '{key}' cannot be converted to TOML: {e}"))?;
        table.insert(key.clone(), parsed);
    }

    let name = optional_string(&table, "name").unwrap_or_default();
    let description = optional_string(&table, "description");
    let developer_instructions = optional_string(&table, "developer_instructions");
    if name.trim().is_empty() {
        return Err("Codex agent requires a non-empty 'name'".to_string());
    }
    if description.as_deref().unwrap_or("").trim().is_empty() {
        return Err("Codex agent requires a non-empty 'description'".to_string());
    }
    if developer_instructions
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        return Err("Codex agent requires a non-empty 'developer_instructions'".to_string());
    }

    Ok(table)
}

pub(crate) fn table_from_config(
    config: &Value,
    fallback_name: &str,
) -> Result<toml::value::Table, String> {
    let object = config
        .as_object()
        .ok_or_else(|| "Agent config must be a JSON object".to_string())?;

    if let Some(raw_toml) = object.get("rawToml").and_then(Value::as_str) {
        let value: toml::Value =
            toml::from_str(raw_toml).map_err(|e| format!("Raw TOML parse failed: {e}"))?;
        let table = value
            .as_table()
            .cloned()
            .ok_or_else(|| "Raw TOML agent definition must be a top-level table".to_string())?;
        return merge_structured_config(table, object, fallback_name);
    }

    merge_structured_config(toml::value::Table::new(), object, fallback_name)
}

fn serialize_table(table: &toml::value::Table) -> Result<String, String> {
    toml::to_string_pretty(&toml::Value::Table(table.clone()))
        .map_err(|e| format!("序列化 agent TOML 失败: {e}"))
}

pub(crate) fn write_agent_file(path: &Path, table: &toml::value::Table) -> Result<(), String> {
    let serialized = serialize_table(table)?;
    fs::write(path, serialized)
        .map_err(|e| format!("写入 agent 文件 '{}' 失败: {e}", path.to_string_lossy()))
}

#[tauri::command]
pub async fn codex_list_agents(context: Option<CodexAgentContextRequest>) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let context = resolve_codex_agent_context(context)?;
        let (agents, diagnostics) = list_codex_agents_for_context(&context)?;
        Ok(json!({
            "context": context.payload(),
            "agents": agents,
            "diagnostics": diagnostics,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn codex_add_agent(
    state: State<'_, AppState>,
    context: Option<CodexAgentContextRequest>,
    name: String,
    config: Value,
) -> Result<Value, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let context = resolve_codex_agent_context(context)?;
        ensure_agents_dir(&context.agents_dir)?;

        let target_name = if name.trim().is_empty() {
            config
                .get("name")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string)
                .ok_or_else(|| "Agent name is required".to_string())?
        } else {
            name
        };

        let file_path = agent_file_path(&context.agents_dir, &target_name);
        if file_path.exists() {
            return Err(format!("Agent '{target_name}' 已存在"));
        }

        let table = table_from_config(&config, &target_name)?;
        write_agent_file(&file_path, &table)?;

        let raw = fs::read_to_string(&file_path).map_err(|e| {
            format!(
                "读取新 agent 文件 '{}' 失败: {e}",
                file_path.to_string_lossy()
            )
        })?;
        Ok(json!({
            "message": format!("Agent '{target_name}' 已添加"),
            "context": context.payload(),
            "agent": record_from_table(&file_path, raw, table, None),
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}

#[tauri::command]
pub async fn codex_update_agent(
    state: State<'_, AppState>,
    context: Option<CodexAgentContextRequest>,
    name: String,
    config: Value,
) -> Result<Value, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let context = resolve_codex_agent_context(context)?;
        let file_path = agent_file_path(&context.agents_dir, &name);
        if !file_path.exists() {
            return Err(format!("Agent '{name}' 不存在"));
        }

        let (_raw, existing_table) = read_agent_table(&file_path)?;
        let object = config
            .as_object()
            .ok_or_else(|| "Agent config must be a JSON object".to_string())?;
        let updated = if object.contains_key("rawToml") {
            table_from_config(&config, &name)?
        } else {
            merge_structured_config(existing_table, object, &name)?
        };

        let next_name = optional_string(&updated, "name").unwrap_or_else(|| name.clone());
        let next_path = agent_file_path(&context.agents_dir, &next_name);
        if next_name != name && next_path.exists() {
            return Err(format!("目标 Agent '{next_name}' 已存在"));
        }

        if next_name != name {
            fs::rename(&file_path, &next_path).map_err(|e| {
                format!(
                    "重命名 agent 文件 '{}' -> '{}' 失败: {e}",
                    file_path.to_string_lossy(),
                    next_path.to_string_lossy()
                )
            })?;
        }

        write_agent_file(&next_path, &updated)?;
        let raw = fs::read_to_string(&next_path).map_err(|e| {
            format!(
                "读取更新后的 agent 文件 '{}' 失败: {e}",
                next_path.to_string_lossy()
            )
        })?;

        Ok(json!({
            "message": format!("Agent '{}' 已更新", next_name),
            "context": context.payload(),
            "agent": record_from_table(&next_path, raw, updated, None),
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}

#[tauri::command]
pub async fn codex_delete_agent(
    state: State<'_, AppState>,
    context: Option<CodexAgentContextRequest>,
    name: String,
) -> Result<String, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<String, String> {
        let context = resolve_codex_agent_context(context)?;
        let file_path = agent_file_path(&context.agents_dir, &name);
        if !file_path.exists() {
            return Err(format!("Agent '{name}' 不存在"));
        }
        fs::remove_file(&file_path).map_err(|e| {
            format!(
                "删除 agent 文件 '{}' 失败: {e}",
                file_path.to_string_lossy()
            )
        })?;
        Ok(format!("Agent '{name}' 已删除"))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}

#[tauri::command]
pub async fn codex_rename_agent(
    state: State<'_, AppState>,
    context: Option<CodexAgentContextRequest>,
    name: String,
    new_name: String,
) -> Result<Value, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let context = resolve_codex_agent_context(context)?;
        if new_name.trim().is_empty() {
            return Err("New agent name cannot be empty".to_string());
        }

        let current_path = agent_file_path(&context.agents_dir, &name);
        if !current_path.exists() {
            return Err(format!("Agent '{name}' 不存在"));
        }

        let target_path = agent_file_path(&context.agents_dir, &new_name);
        if target_path.exists() {
            return Err(format!("目标 Agent '{new_name}' 已存在"));
        }

        let (_raw, mut table) = read_agent_table(&current_path)?;
        table.insert("name".to_string(), toml::Value::String(new_name.clone()));
        fs::rename(&current_path, &target_path).map_err(|e| {
            format!(
                "重命名 agent 文件 '{}' -> '{}' 失败: {e}",
                current_path.to_string_lossy(),
                target_path.to_string_lossy()
            )
        })?;
        write_agent_file(&target_path, &table)?;
        let raw = fs::read_to_string(&target_path).map_err(|e| {
            format!(
                "读取重命名后的 agent 文件 '{}' 失败: {e}",
                target_path.to_string_lossy()
            )
        })?;

        Ok(json!({
            "message": format!("Agent '{}' 已重命名为 '{}'", name, new_name),
            "context": context.payload(),
            "agent": record_from_table(&target_path, raw, table, None),
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}

#[tauri::command]
pub async fn codex_copy_agent(
    state: State<'_, AppState>,
    source_context: Option<CodexAgentContextRequest>,
    target_context: Option<CodexAgentContextRequest>,
    name: String,
    target_name: Option<String>,
) -> Result<Value, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let source_context = resolve_codex_agent_context(source_context)?;
        let target_context = resolve_codex_agent_context(target_context)?;
        ensure_agents_dir(&target_context.agents_dir)?;

        let source_path = agent_file_path(&source_context.agents_dir, &name);
        if !source_path.exists() {
            return Err(format!("源 Agent '{name}' 不存在"));
        }

        let (raw, mut table) = read_agent_table(&source_path)?;
        let final_name = target_name
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| optional_string(&table, "name").unwrap_or_else(|| name.clone()));
        let target_path = agent_file_path(&target_context.agents_dir, &final_name);
        if target_path.exists() {
            return Err(format!("目标 Agent '{final_name}' 已存在"));
        }

        table.insert("name".to_string(), toml::Value::String(final_name.clone()));
        write_agent_file(&target_path, &table)?;
        let written = fs::read_to_string(&target_path).map_err(|e| {
            format!(
                "读取复制后的 agent 文件 '{}' 失败: {e}",
                target_path.to_string_lossy()
            )
        })?;

        Ok(json!({
            "message": format!(
                "Agent '{}' 已复制到 {}",
                final_name,
                target_context.payload().label
            ),
            "sourceContext": source_context.payload(),
            "targetContext": target_context.payload(),
            "agent": record_from_table(&target_path, written, table, None),
            "sourceRawToml": raw,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}

#[tauri::command]
pub async fn codex_validate_agent_toml(
    context: Option<CodexAgentContextRequest>,
    name: String,
) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let context = resolve_codex_agent_context(context)?;
        let file_path = agent_file_path(&context.agents_dir, &name);
        if !file_path.exists() {
            return Err(format!("Agent '{name}' 不存在"));
        }

        let raw = fs::read_to_string(&file_path).map_err(|e| {
            format!(
                "读取 agent 文件 '{}' 失败: {e}",
                file_path.to_string_lossy()
            )
        })?;
        let value: toml::Value = toml::from_str(&raw).map_err(|e| {
            format!(
                "Raw TOML parse failed for '{}': {e}",
                file_path.to_string_lossy()
            )
        })?;
        let table = value.as_table().cloned().ok_or_else(|| {
            format!(
                "Agent 文件 '{}' 顶层必须是 TOML table",
                file_path.to_string_lossy()
            )
        })?;
        Ok(json!({
            "context": context.payload(),
            "agent": record_from_table(&file_path, raw, table, None),
            "diagnostics": [],
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn resolve_global_context_defaults_to_codex_home() {
        let context = resolve_codex_agent_context(None).unwrap();
        assert_eq!(context.mode, "global");
        assert!(
            context.agents_dir.ends_with(".codex\\agents")
                || context.agents_dir.ends_with(".codex/agents")
        );
    }

    #[test]
    fn merge_structured_config_preserves_unknown_fields() {
        let mut table = toml::value::Table::new();
        table.insert("name".into(), toml::Value::String("existing".into()));
        table.insert("description".into(), toml::Value::String("old".into()));
        table.insert(
            "developer_instructions".into(),
            toml::Value::String("do work".into()),
        );
        table.insert("custom_flag".into(), toml::Value::Boolean(true));

        let config = serde_json::json!({
            "description": "new",
            "model": "gpt-5.4"
        });

        let merged =
            merge_structured_config(table, config.as_object().unwrap(), "existing").unwrap();
        assert_eq!(
            optional_string(&merged, "description").as_deref(),
            Some("new")
        );
        assert_eq!(
            optional_string(&merged, "model").as_deref(),
            Some("gpt-5.4")
        );
        assert_eq!(
            merged.get("custom_flag").and_then(toml::Value::as_bool),
            Some(true)
        );
    }

    #[test]
    fn raw_toml_can_be_overridden_by_structured_name() {
        let config = serde_json::json!({
            "rawToml": "name = 'legacy'\ndescription = 'desc'\ndeveloper_instructions = 'do work'\n",
            "name": "override"
        });

        let table = table_from_config(&config, "fallback").unwrap();
        assert_eq!(optional_string(&table, "name").as_deref(), Some("override"));
    }
}
