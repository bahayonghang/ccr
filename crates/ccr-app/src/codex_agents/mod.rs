//! Shared Codex agent TOML and scoped writes. Tauri and GPUI use this single owner.
pub mod sources;
use crate::platform_settings::{Document, Format, SaveOutcome, Snapshot};
use ccr_core::core::{
    BackupPolicy, VersionedWriteOutcome,
    guarded_write::{backup_guarded, delete_guarded_versioned},
    lock::{FileLock, LockManager},
    write_journal::WriteJournal,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

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
pub struct CodexAgentRecord {
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
pub struct CodexAgentDiagnostic {
    pub file_name: String,
    pub path: String,
    pub severity: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedCodexAgentContext {
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

#[derive(Clone)]
pub struct AgentStore {
    pub global_agents: PathBuf,
    pub sources_root: PathBuf,
    pub locks: PathBuf,
    pub backups: PathBuf,
}
impl AgentStore {
    pub fn detect() -> Result<Self, String> {
        let home = dirs::home_dir().ok_or("Cannot resolve home")?;
        Ok(Self::from_home(&home))
    }
    pub fn from_home(home: &Path) -> Self {
        Self {
            global_agents: home.join(".codex/agents"),
            sources_root: home.join(".ccr/platforms/codex/agent-sources"),
            locks: home.join(".ccr/.locks"),
            backups: home.join(".ccr/backups/codex-agents"),
        }
    }
    pub fn context(
        &self,
        request: Option<CodexAgentContextRequest>,
    ) -> Result<ResolvedCodexAgentContext, String> {
        let request = request.unwrap_or_default();
        match request.mode.as_deref().unwrap_or("global") {
            "global" => Ok(ResolvedCodexAgentContext {
                mode: "global".into(),
                label: "Global".into(),
                agents_dir: self.global_agents.clone(),
                project_root: None,
            }),
            "project" => {
                let root = PathBuf::from(
                    request
                        .project_root
                        .ok_or("Project context requires project_root")?,
                );
                if !root.is_absolute() || !root.is_dir() {
                    return Err("Project root must be an existing absolute directory".into());
                }
                reject_links(&root)?;
                let root = root
                    .canonicalize()
                    .map_err(|_| "Cannot resolve project root")?;
                Ok(ResolvedCodexAgentContext {
                    mode: "project".into(),
                    label: root
                        .file_name()
                        .and_then(|v| v.to_str())
                        .map(|v| format!("Project: {v}"))
                        .unwrap_or_else(|| "Project".into()),
                    agents_dir: root.join(".codex/agents"),
                    project_root: Some(root),
                })
            }
            _ => Err("Unsupported Codex agent context".into()),
        }
    }
    fn lock(&self) -> Result<FileLock, String> {
        reject_links(&self.locks)?;
        FileLock::new(
            self.locks.join("codex-agents-operation.lock"),
            Duration::from_secs(10),
        )
        .map_err(|_| "Cannot lock Codex agents".into())
    }
    pub fn document(&self, path: &Path) -> Document {
        Document::new(
            path.into(),
            None,
            BackupPolicy::Dir {
                dir: self.backups.clone(),
                prefix: "agent".into(),
            },
            Some(self.locks.clone()),
            Format::Toml,
        )
    }
    pub fn read(
        &self,
        context: &ResolvedCodexAgentContext,
        name: &str,
    ) -> Result<Snapshot, String> {
        let path = agent_file_path(&context.agents_dir, name)?;
        self.document(&path)
            .read()
            .map_err(|_| "Cannot read agent".into())
    }
    fn commit(&self, changes: Vec<(Snapshot, Option<String>)>) -> Result<(), String> {
        reject_links(&self.backups)?;
        let paths = changes
            .iter()
            .map(|(s, _)| s.source.clone())
            .collect::<Vec<_>>();
        for path in &paths {
            reject_links(path)?;
        }
        LockManager::with_scoped_directory(&self.locks, || {
            Ok((|| -> Result<(), String> {
                let journal = WriteJournal::begin(&paths)
                    .map_err(|_| "Cannot prepare agent write".to_string())?;
                let work = (|| {
                    for (before, after) in &changes {
                        journal
                            .expect_version(&before.source, before.token.clone())
                            .map_err(|_| "Agent changed externally")?;
                        let saved = match after {
                            Some(content) => {
                                self.document(&before.source)
                                    .save_validated(content, before)
                                    .map_err(|_| "Agent write failed")?
                                    != SaveOutcome::Conflict
                            }
                            None => {
                                backup_guarded(
                                    &before.source,
                                    &BackupPolicy::Dir {
                                        dir: self.backups.clone(),
                                        prefix: "agent-deleted".into(),
                                    },
                                )
                                .map_err(|_| "Cannot back up agent before deletion")?;
                                matches!(
                                    delete_guarded_versioned(&before.source, &before.token)
                                        .map_err(|_| "Agent delete failed")?,
                                    VersionedWriteOutcome::Written
                                )
                            }
                        };
                        if !saved {
                            return Err("Agent changed externally".to_string());
                        }
                    }
                    journal
                        .verify()
                        .map_err(|_| "Agent changed externally".to_string())
                })();
                match work {
                    Ok(()) => {
                        journal.commit();
                        Ok(())
                    }
                    Err(error) => {
                        if journal.rollback().is_empty() {
                            Err(error)
                        } else {
                            Err("Agent write failed; compensation requires recovery".into())
                        }
                    }
                }
            })())
        })
        .map_err(|_| "Cannot scope agent locks".to_string())?
    }
    pub fn list(&self, request: Option<CodexAgentContextRequest>) -> Result<Value, String> {
        let context = self.context(request)?;
        let (agents, diagnostics) = list_codex_agents_for_context(&context)?;
        Ok(json!({"context":context.payload(),"agents":agents,"diagnostics":diagnostics}))
    }
    pub fn add(
        &self,
        request: Option<CodexAgentContextRequest>,
        name: String,
        config: Value,
    ) -> Result<Value, String> {
        let _lock = self.lock()?;
        let context = self.context(request)?;
        let name = if name.trim().is_empty() {
            config
                .get("name")
                .and_then(Value::as_str)
                .ok_or("Agent name is required")?
                .to_string()
        } else {
            name
        };
        let before = self.read(&context, &name)?;
        if before.exists {
            return Err("Agent already exists".into());
        }
        let table = table_from_config(&config, &name)?;
        let raw = serialize_table(&table)?;
        self.commit(vec![(before.clone(), Some(raw.clone()))])?;
        Ok(
            json!({"message":format!("Agent '{name}' 已添加"),"context":context.payload(),"agent":record_from_table(&before.source,raw,table,None)}),
        )
    }
    pub fn update(
        &self,
        request: Option<CodexAgentContextRequest>,
        name: String,
        config: Value,
        expected: Option<&str>,
    ) -> Result<Value, String> {
        let _lock = self.lock()?;
        let context = self.context(request)?;
        let before = self.read(&context, &name)?;
        check_expected(&before, expected)?;
        let (_, old) = read_agent_table(&before.source)?;
        let object = config
            .as_object()
            .ok_or("Agent config must be a JSON object")?;
        let table = if object.contains_key("rawToml") {
            table_from_config(&config, &name)?
        } else {
            merge_structured_config(old, object, &name)?
        };
        let next_name = optional_string(&table, "name").unwrap_or_else(|| name.clone());
        let target = agent_file_path(&context.agents_dir, &next_name)?;
        let raw = serialize_table(&table)?;
        let changes = if target == before.source {
            vec![(before, Some(raw.clone()))]
        } else {
            let next = self
                .document(&target)
                .read()
                .map_err(|_| "Cannot read target agent")?;
            if next.exists {
                return Err("Target agent already exists".into());
            }
            vec![(next, Some(raw.clone())), (before, None)]
        };
        self.commit(changes)?;
        Ok(
            json!({"message":format!("Agent '{next_name}' 已更新"),"context":context.payload(),"agent":record_from_table(&target,raw,table,None)}),
        )
    }
    pub fn delete(
        &self,
        request: Option<CodexAgentContextRequest>,
        name: String,
        expected: Option<&str>,
    ) -> Result<String, String> {
        let _lock = self.lock()?;
        let context = self.context(request)?;
        let before = self.read(&context, &name)?;
        check_expected(&before, expected)?;
        self.commit(vec![(before, None)])?;
        Ok(format!("Agent '{name}' 已删除"))
    }
    pub fn rename(
        &self,
        request: Option<CodexAgentContextRequest>,
        name: String,
        new_name: String,
        expected: Option<&str>,
    ) -> Result<Value, String> {
        if name == new_name {
            return Err("Target agent already exists".into());
        }
        let mut result = self.update(request, name.clone(), json!({"name":new_name}), expected)?;
        result["message"] = format!("Agent '{name}' 已重命名为 '{new_name}'").into();
        Ok(result)
    }
    pub fn copy(
        &self,
        source: Option<CodexAgentContextRequest>,
        target: Option<CodexAgentContextRequest>,
        name: String,
        target_name: Option<String>,
        expected: Option<&str>,
    ) -> Result<Value, String> {
        let _lock = self.lock()?;
        let source = self.context(source)?;
        let target = self.context(target)?;
        let before = self.read(&source, &name)?;
        check_expected(&before, expected)?;
        // Parse the same revision checked under the store lock. Re-reading the
        // file could silently copy a later external edit instead of this version.
        let mut table: toml::value::Table =
            toml::from_str(&before.content).map_err(|_| "Cannot parse source agent")?;
        let name = target_name
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| optional_string(&table, "name").unwrap_or(name));
        let after = self.read(&target, &name)?;
        if after.exists {
            return Err("Target agent already exists".into());
        }
        table.insert("name".into(), toml::Value::String(name.clone()));
        let raw = serialize_table(&table)?;
        self.commit(vec![(after.clone(), Some(raw.clone()))])?;
        Ok(
            json!({"message":format!("Agent '{name}' 已复制到 {}",target.label),"sourceContext":source.payload(),"targetContext":target.payload(),"agent":record_from_table(&after.source,raw,table,None),"sourceRawToml":before.content}),
        )
    }
    pub fn validate(
        &self,
        request: Option<CodexAgentContextRequest>,
        name: String,
    ) -> Result<Value, String> {
        let context = self.context(request)?;
        let snapshot = self.read(&context, &name)?;
        check_expected(&snapshot, None)?;
        let (_, table) = read_agent_table(&snapshot.source)?;
        Ok(
            json!({"context":context.payload(),"agent":record_from_table(&snapshot.source,snapshot.content,table,None),"diagnostics":[]}),
        )
    }
}
fn check_expected(snapshot: &Snapshot, expected: Option<&str>) -> Result<(), String> {
    if !snapshot.exists {
        return Err("Agent does not exist".into());
    }
    if expected.is_some_and(|value| value != snapshot.token) {
        return Err("Agent changed externally".into());
    }
    Ok(())
}
pub fn agent_file_path(root: &Path, name: &str) -> Result<PathBuf, String> {
    if name.is_empty()
        || name.trim() != name
        || name == "."
        || name == ".."
        || name.chars().any(|c| {
            c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
        })
        || name.ends_with('.')
    {
        return Err("Invalid agent file name".into());
    }
    let path = root.join(format!("{name}.toml"));
    reject_links(&path)?;
    Ok(path)
}
fn reject_links(path: &Path) -> Result<(), String> {
    if !path.is_absolute() {
        return Err("Agent path must be absolute".into());
    }
    for part in path.ancestors() {
        match fs::symlink_metadata(part) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    return Err("Linked agent paths are not supported".into());
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err("Linked agent paths are not supported".into());
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("Cannot inspect agent path".into()),
        }
    }
    Ok(())
}
pub fn read_agent_table(path: &Path) -> Result<(String, toml::value::Table), String> {
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

pub fn optional_string(table: &toml::value::Table, key: &str) -> Option<String> {
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

pub fn record_from_table(
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
    reject_links(&context.agents_dir)?;
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
        reject_links(&path)?;
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

    agents.sort_by_key(|agent| agent.name.to_lowercase());

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
            let remove_skills = if let Some(skills_table) =
                table.get_mut("skills").and_then(toml::Value::as_table_mut)
            {
                skills_table.remove("config");
                skills_table.is_empty()
            } else {
                false
            };
            if remove_skills {
                table.remove("skills");
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

pub fn table_from_config(
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn resolve_global_context_uses_captured_home() {
        let root = tempfile::tempdir().unwrap();
        let context = AgentStore::from_home(root.path()).context(None).unwrap();
        assert_eq!(context.agents_dir, root.path().join(".codex/agents"));
        assert!(!context.agents_dir.exists());
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

    #[test]
    fn merge_structured_config_preserves_existing_skills_config_when_omitted() {
        let mut table = toml::value::Table::new();
        table.insert("name".into(), toml::Value::String("existing".into()));
        table.insert("description".into(), toml::Value::String("old".into()));
        table.insert(
            "developer_instructions".into(),
            toml::Value::String("do work".into()),
        );

        let mut skills = toml::value::Table::new();
        skills.insert(
            "config".into(),
            toml::Value::Array(vec![toml::Value::Table(toml::value::Table::from_iter([
                (
                    "path".into(),
                    toml::Value::String("/tmp/docs/SKILL.md".into()),
                ),
                ("enabled".into(), toml::Value::Boolean(false)),
            ]))]),
        );
        table.insert("skills".into(), toml::Value::Table(skills));

        let config = serde_json::json!({
            "description": "new",
            "model": "gpt-5.4"
        });

        let merged =
            merge_structured_config(table, config.as_object().unwrap(), "existing").unwrap();
        let skills_table = merged
            .get("skills")
            .and_then(toml::Value::as_table)
            .expect("skills table should remain");
        let skills_config = skills_table
            .get("config")
            .and_then(toml::Value::as_array)
            .expect("skills.config should remain");

        assert_eq!(skills_config.len(), 1);
        assert_eq!(
            optional_string(&merged, "model").as_deref(),
            Some("gpt-5.4")
        );
    }
    #[test]
    fn guarded_agent_edits_preserve_unknown_fields_reject_escape_and_compensate() {
        let root = tempfile::tempdir().unwrap();
        let store = AgentStore::from_home(root.path());
        store.add(None,"reviewer".into(),json!({"name":"reviewer","description":"review","developer_instructions":"inspect","extra":true})).unwrap();
        let context = store.context(None).unwrap();
        let before = store.read(&context, "reviewer").unwrap();
        store
            .update(
                None,
                "reviewer".into(),
                json!({"description":"updated"}),
                Some(&before.token),
            )
            .unwrap();
        assert!(
            store
                .update(
                    None,
                    "reviewer".into(),
                    json!({"description":"stale"}),
                    Some(&before.token)
                )
                .is_err()
        );
        let current = store.read(&context, "reviewer").unwrap();
        assert!(
            store
                .copy(
                    None,
                    None,
                    "reviewer".into(),
                    Some("copy".into()),
                    Some(&before.token),
                )
                .is_err()
        );
        assert!(!store.read(&context, "copy").unwrap().exists);
        store
            .copy(
                None,
                None,
                "reviewer".into(),
                Some("copy".into()),
                Some(&current.token),
            )
            .unwrap();
        let copied: toml::Value =
            toml::from_str(&store.read(&context, "copy").unwrap().content).unwrap();
        assert_eq!(copied["description"].as_str(), Some("updated"));
        let table: toml::Value = toml::from_str(&current.content).unwrap();
        assert_eq!(table["extra"].as_bool(), Some(true));
        assert!(store.add(None, "../escaped".into(), json!({})).is_err());
        let target = store.read(&context, "second").unwrap();
        // Simulate another writer after the operation snapshots were captured.
        fs::write(&target.source, "external").unwrap();
        assert!(
            store
                .commit(vec![
                    (current.clone(), Some("replacement".into())),
                    (target, Some("second".into()))
                ])
                .is_err()
        );
        assert_eq!(
            store.read(&context, "reviewer").unwrap().content,
            current.content
        );
        assert_eq!(
            fs::read_to_string(context.agents_dir.join("second.toml")).unwrap(),
            "external"
        );
        store
            .delete(None, "reviewer".into(), Some(&current.token))
            .unwrap();
        assert!(!store.read(&context, "reviewer").unwrap().exists);
    }
}
