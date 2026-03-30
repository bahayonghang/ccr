//! Qoder 命令 — Settings/MCP/Commands/Subagents/Hooks。
//!
//! - Settings/Hooks: `~/.qoder/settings.json` / `{project}/.qoder/settings.json`
//! - MCP: `~/.qoder.json` / `{project}/.mcp.json`
//! - Commands: `~/.qoder/commands/*.md` / `{project}/.qoder/commands/*.md`
//! - Subagents: `~/.qoder/agents/*.md` / `{project}/.qoder/agents/*.md`

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};
use std::collections::{HashMap, HashSet};
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct QoderCommandFrontmatter {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct QoderAgentFrontmatter {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tools: Vec<String>,
}

fn user_home_dir() -> Result<PathBuf, String> {
    dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())
}

fn find_project_root() -> Option<PathBuf> {
    let start = std::env::current_dir().ok()?;
    let mut current = start.as_path();
    loop {
        if current.join(".git").exists() {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

fn project_qoder_dir() -> Option<PathBuf> {
    find_project_root().map(|root| root.join(".qoder"))
}

fn qoder_user_settings_path() -> Result<PathBuf, String> {
    Ok(user_home_dir()?.join(".qoder").join("settings.json"))
}

fn qoder_project_settings_path() -> Option<PathBuf> {
    project_qoder_dir().map(|dir| dir.join("settings.json"))
}

fn qoder_project_local_settings_path() -> Option<PathBuf> {
    project_qoder_dir().map(|dir| dir.join("settings.local.json"))
}

fn qoder_settings_write_path() -> Result<PathBuf, String> {
    if let Some(local) = qoder_project_local_settings_path()
        && local.exists()
    {
        return Ok(local);
    }
    if let Some(project) = qoder_project_settings_path() {
        return Ok(project);
    }
    qoder_user_settings_path()
}

fn qoder_user_mcp_path() -> Result<PathBuf, String> {
    Ok(user_home_dir()?.join(".qoder.json"))
}

fn qoder_project_mcp_path() -> Option<PathBuf> {
    find_project_root().map(|root| root.join(".mcp.json"))
}

fn qoder_commands_dirs() -> Result<(PathBuf, PathBuf), String> {
    let home = user_home_dir()?.join(".qoder").join("commands");
    let project = project_qoder_dir()
        .unwrap_or_else(|| PathBuf::from(".").join(".qoder"))
        .join("commands");
    Ok((project, home))
}

fn qoder_agents_dirs() -> Result<(PathBuf, PathBuf), String> {
    let home = user_home_dir()?.join(".qoder").join("agents");
    let project = project_qoder_dir()
        .unwrap_or_else(|| PathBuf::from(".").join(".qoder"))
        .join("agents");
    Ok((project, home))
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    Ok(())
}

fn read_json_file_or_empty(path: &Path) -> Result<Value, String> {
    if !path.exists() {
        return Ok(json!({}));
    }
    let content = std::fs::read_to_string(path).map_err(|e| format!("读取 JSON 文件失败: {e}"))?;
    if content.trim().is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_str(&content).map_err(|e| format!("解析 JSON 文件失败: {e}"))
}

fn write_json_atomic(path: &Path, value: &Value) -> Result<(), String> {
    ensure_parent_dir(path)?;
    let parent = path.parent().ok_or_else(|| "无法获取父目录".to_string())?;
    let mut tmp =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| format!("创建临时文件失败: {e}"))?;
    let payload =
        serde_json::to_string_pretty(value).map_err(|e| format!("序列化 JSON 失败: {e}"))?;
    tmp.write_all(payload.as_bytes())
        .map_err(|e| format!("写入临时文件失败: {e}"))?;
    tmp.persist(path)
        .map_err(|e| format!("持久化配置文件失败: {e}"))?;
    Ok(())
}

fn merge_object(base: &mut Map<String, Value>, overlay: &Map<String, Value>) {
    for (key, value) in overlay {
        base.insert(key.clone(), value.clone());
    }
}

fn read_qoder_settings_merged() -> Result<Value, String> {
    let mut merged = Map::new();
    let mut paths = vec![qoder_user_settings_path()?];
    if let Some(project) = qoder_project_settings_path() {
        paths.push(project);
    }
    if let Some(local) = qoder_project_local_settings_path() {
        paths.push(local);
    }

    for path in paths {
        let value = read_json_file_or_empty(&path)?;
        if let Some(obj) = value.as_object() {
            merge_object(&mut merged, obj);
        }
    }

    Ok(Value::Object(merged))
}

fn read_qoder_mcp_map(path: &Path) -> Result<Map<String, Value>, String> {
    let value = read_json_file_or_empty(path)?;
    Ok(value
        .get("mcpServers")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default())
}

fn write_qoder_mcp_map(path: &Path, servers: &Map<String, Value>) -> Result<(), String> {
    let mut value = read_json_file_or_empty(path)?;
    let root = value
        .as_object_mut()
        .ok_or_else(|| "MCP 配置文件必须是 JSON 对象".to_string())?;
    root.insert("mcpServers".to_string(), Value::Object(servers.clone()));
    write_json_atomic(path, &value)
}

fn qoder_mcp_write_path_for_new() -> Result<PathBuf, String> {
    if let Some(project) = qoder_project_mcp_path() {
        return Ok(project);
    }
    qoder_user_mcp_path()
}

fn find_named_entry_path(
    project_path: &Path,
    user_path: &Path,
    name: &str,
) -> Result<Option<PathBuf>, String> {
    let project_map = read_qoder_mcp_map(project_path)?;
    if project_map.contains_key(name) {
        return Ok(Some(project_path.to_path_buf()));
    }
    let user_map = read_qoder_mcp_map(user_path)?;
    if user_map.contains_key(name) {
        return Ok(Some(user_path.to_path_buf()));
    }
    Ok(None)
}

fn split_frontmatter(content: &str) -> (Option<&str>, &str) {
    if let Some(rest) = content.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        let frontmatter = &rest[..end];
        let body = &rest[end + 5..];
        return (Some(frontmatter), body);
    }
    (None, content)
}

fn parse_markdown_frontmatter<T>(content: &str) -> Result<(T, String), String>
where
    T: DeserializeOwned + Default,
{
    let (frontmatter, body) = split_frontmatter(content);
    let metadata = if let Some(frontmatter) = frontmatter {
        serde_yaml::from_str(frontmatter).map_err(|e| format!("解析 frontmatter 失败: {e}"))?
    } else {
        T::default()
    };
    Ok((metadata, body.trim().to_string()))
}

fn render_markdown_with_frontmatter<T>(metadata: &T, body: &str) -> Result<String, String>
where
    T: Serialize,
{
    let mut yaml =
        serde_yaml::to_string(metadata).map_err(|e| format!("序列化 frontmatter 失败: {e}"))?;
    if let Some(stripped) = yaml.strip_prefix("---\n") {
        yaml = stripped.to_string();
    }
    Ok(format!("---\n{}---\n\n{}\n", yaml, body.trim()))
}

fn first_non_empty_line(value: &str) -> Option<String> {
    value
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

fn list_markdown_entities<F>(
    project_dir: &Path,
    user_dir: &Path,
    build_item: F,
) -> Result<Value, String>
where
    F: Fn(&str, &Path, &str) -> Result<Value, String>,
{
    let mut chosen: HashMap<String, PathBuf> = HashMap::new();
    for base in [user_dir, project_dir] {
        if !base.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(base)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let rel = path.strip_prefix(base).map_err(|e| e.to_string())?;
            let rel_no_ext = rel.with_extension("");
            let key = rel_no_ext.to_string_lossy().replace('\\', "/");
            chosen.insert(key, path.to_path_buf());
        }
    }

    let mut items = Vec::new();
    let mut folders = HashSet::new();
    for (key, path) in &chosen {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("读取 Markdown 文件失败: {e}"))?;
        let (folder, name) = match key.rsplit_once('/') {
            Some((folder, name)) => {
                folders.insert(folder.to_string());
                (folder.to_string(), name.to_string())
            }
            None => (String::new(), key.clone()),
        };
        items.push(
            build_item(&name, path, &content)?
                .as_object()
                .cloned()
                .map(|mut obj| {
                    obj.insert("folder".to_string(), json!(folder));
                    Value::Object(obj)
                })
                .unwrap_or(Value::Null),
        );
    }

    items.sort_by(|a, b| {
        a.get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .cmp(b.get("name").and_then(Value::as_str).unwrap_or(""))
    });

    let mut folders_vec: Vec<String> = folders.into_iter().collect();
    folders_vec.sort();

    Ok(json!({
        "items": items,
        "folders": folders_vec,
    }))
}

fn find_markdown_entity_file(
    project_dir: &Path,
    user_dir: &Path,
    name: &str,
) -> Result<Option<PathBuf>, String> {
    for base in [project_dir, user_dir] {
        if !base.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(base)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md")
                && path.file_stem().and_then(|s| s.to_str()) == Some(name)
            {
                return Ok(Some(path.to_path_buf()));
            }
        }
    }
    Ok(None)
}

fn build_nested_markdown_path(base_dir: &Path, folder: &str, name: &str) -> PathBuf {
    let mut target = base_dir.to_path_buf();
    if !folder.trim().is_empty() {
        target = target.join(folder.trim());
    }
    target.join(format!("{name}.md"))
}

fn normalize_qoder_hook_item(index: usize, value: &Value) -> Value {
    let command = value
        .get("hooks")
        .and_then(Value::as_array)
        .and_then(|hooks| {
            hooks.iter().find_map(|hook| {
                if hook.get("type").and_then(Value::as_str) == Some("command") {
                    hook.get("command")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                } else {
                    None
                }
            })
        })
        .unwrap_or_default();

    json!({
        "id": index,
        "name": format!("Notification {}", index + 1),
        "event": "Notification",
        "command": command,
    })
}

fn qoder_hooks_list_from_settings(settings: &Value) -> Vec<Value> {
    settings
        .get("hooks")
        .and_then(|value| value.get("Notification"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .enumerate()
                .map(|(index, item)| normalize_qoder_hook_item(index, item))
                .collect()
        })
        .unwrap_or_default()
}

fn build_qoder_hook_entry(command: &str) -> Value {
    json!({
        "hooks": [
            {
                "type": "command",
                "command": command,
            }
        ]
    })
}

// ── Settings ──

#[tauri::command]
pub async fn qoder_get_settings() -> Result<Value, String> {
    tokio::task::spawn_blocking(read_qoder_settings_merged)
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn qoder_update_settings(settings: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut current = read_qoder_settings_merged()?;
        if let (Some(cur_obj), Some(new_obj)) = (current.as_object_mut(), settings.as_object()) {
            merge_object(cur_obj, new_obj);
        } else {
            current = settings;
        }
        let path = qoder_settings_write_path()?;
        write_json_atomic(&path, &current)?;
        Ok(json!({ "message": "Qoder 配置更新成功" }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

// ── MCP Servers ──

#[tauri::command]
pub async fn qoder_list_mcp_servers() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let user_path = qoder_user_mcp_path()?;
        let project_path = qoder_project_mcp_path().unwrap_or_else(|| PathBuf::from(".mcp.json"));
        let mut merged = read_qoder_mcp_map(&user_path)?;
        merge_object(&mut merged, &read_qoder_mcp_map(&project_path)?);
        let mut list: Vec<Value> = merged
            .into_iter()
            .map(|(name, server)| {
                let mut obj = server.as_object().cloned().unwrap_or_default();
                obj.insert("name".to_string(), json!(name));
                Value::Object(obj)
            })
            .collect();
        list.sort_by(|a, b| {
            a.get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .cmp(b.get("name").and_then(Value::as_str).unwrap_or(""))
        });
        Ok(json!(list))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn qoder_add_mcp_server(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        if name.trim().is_empty() {
            return Err("MCP 服务器名称不能为空".to_string());
        }
        let path = qoder_mcp_write_path_for_new()?;
        let mut servers = read_qoder_mcp_map(&path)?;
        if servers.contains_key(&name) {
            return Err(format!("MCP 服务器 '{name}' 已存在"));
        }
        servers.insert(name.clone(), config);
        write_qoder_mcp_map(&path, &servers)?;
        Ok(json!({ "message": format!("MCP 服务器 '{name}' 添加成功") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn qoder_update_mcp_server(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let user_path = qoder_user_mcp_path()?;
        let project_path = qoder_project_mcp_path().unwrap_or_else(|| PathBuf::from(".mcp.json"));
        let path = find_named_entry_path(&project_path, &user_path, &name)?
            .ok_or_else(|| format!("MCP 服务器 '{name}' 不存在"))?;
        let mut servers = read_qoder_mcp_map(&path)?;
        servers.insert(name.clone(), config);
        write_qoder_mcp_map(&path, &servers)?;
        Ok(json!({ "message": format!("MCP 服务器 '{name}' 更新成功") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn qoder_delete_mcp_server(name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let user_path = qoder_user_mcp_path()?;
        let project_path = qoder_project_mcp_path().unwrap_or_else(|| PathBuf::from(".mcp.json"));
        let path = find_named_entry_path(&project_path, &user_path, &name)?
            .ok_or_else(|| format!("MCP 服务器 '{name}' 不存在"))?;
        let mut servers = read_qoder_mcp_map(&path)?;
        if servers.remove(&name).is_none() {
            return Err(format!("MCP 服务器 '{name}' 不存在"));
        }
        write_qoder_mcp_map(&path, &servers)?;
        Ok(format!("MCP 服务器 '{name}' 删除成功"))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

// ── Commands ──

#[tauri::command]
pub async fn qoder_list_commands() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let (project_dir, user_dir) = qoder_commands_dirs()?;
        let result = list_markdown_entities(&project_dir, &user_dir, |name, _path, content| {
            let (frontmatter, body): (QoderCommandFrontmatter, String) =
                parse_markdown_frontmatter(content)?;
            Ok(json!({
                "name": frontmatter.name.unwrap_or_else(|| name.to_string()),
                "description": frontmatter
                    .description
                    .or_else(|| first_non_empty_line(&body))
                    .unwrap_or_else(|| "Qoder command".to_string()),
                "command": body,
                "enabled": true,
            }))
        })?;

        Ok(json!({
            "commands": result.get("items").cloned().unwrap_or_else(|| json!([])),
            "folders": result.get("folders").cloned().unwrap_or_else(|| json!([])),
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn qoder_add_command(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        if name.trim().is_empty() {
            return Err("命令名称不能为空".to_string());
        }
        let (project_dir, _user_dir) = qoder_commands_dirs()?;
        let folder = config
            .get("folder")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let description = config
            .get("description")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| {
                config
                    .get("command")
                    .and_then(Value::as_str)
                    .and_then(first_non_empty_line)
            });
        let command = config
            .get("command")
            .and_then(Value::as_str)
            .ok_or_else(|| "命令内容不能为空".to_string())?;

        let path = build_nested_markdown_path(&project_dir, &folder, &name);
        if path.exists() {
            return Err(format!("命令 '{name}' 已存在"));
        }
        let content = render_markdown_with_frontmatter(
            &QoderCommandFrontmatter {
                name: Some(name.clone()),
                description,
            },
            command,
        )?;
        ensure_parent_dir(&path)?;
        std::fs::write(&path, content).map_err(|e| format!("写入命令文件失败: {e}"))?;
        Ok(json!({ "message": format!("命令 '{name}' 添加成功") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn qoder_update_command(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let (project_dir, user_dir) = qoder_commands_dirs()?;
        let current_path = find_markdown_entity_file(&project_dir, &user_dir, &name)?
            .ok_or_else(|| format!("命令 '{name}' 不存在"))?;
        let write_root = if current_path.starts_with(&user_dir) {
            user_dir.clone()
        } else {
            project_dir.clone()
        };
        let folder = config
            .get("folder")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let description = config
            .get("description")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| {
                config
                    .get("command")
                    .and_then(Value::as_str)
                    .and_then(first_non_empty_line)
            });
        let command = config
            .get("command")
            .and_then(Value::as_str)
            .ok_or_else(|| "命令内容不能为空".to_string())?;

        let desired_path = build_nested_markdown_path(&write_root, &folder, &name);
        let content = render_markdown_with_frontmatter(
            &QoderCommandFrontmatter {
                name: Some(name.clone()),
                description,
            },
            command,
        )?;
        ensure_parent_dir(&desired_path)?;
        std::fs::write(&desired_path, content).map_err(|e| format!("写入命令文件失败: {e}"))?;
        if desired_path != current_path {
            std::fs::remove_file(&current_path).map_err(|e| format!("删除旧命令文件失败: {e}"))?;
        }
        Ok(json!({ "message": format!("命令 '{name}' 更新成功") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn qoder_delete_command(name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let (project_dir, user_dir) = qoder_commands_dirs()?;
        let path = find_markdown_entity_file(&project_dir, &user_dir, &name)?
            .ok_or_else(|| format!("命令 '{name}' 不存在"))?;
        std::fs::remove_file(&path).map_err(|e| format!("删除命令文件失败: {e}"))?;
        Ok(format!("命令 '{name}' 删除成功"))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn qoder_list_slash_commands() -> Result<Value, String> {
    qoder_list_commands().await
}

#[tauri::command]
pub async fn qoder_add_slash_command(name: String, config: Value) -> Result<Value, String> {
    qoder_add_command(name, config).await
}

#[tauri::command]
pub async fn qoder_update_slash_command(name: String, config: Value) -> Result<Value, String> {
    qoder_update_command(name, config).await
}

#[tauri::command]
pub async fn qoder_delete_slash_command(name: String) -> Result<String, String> {
    qoder_delete_command(name).await
}

// ── Subagents ──

#[tauri::command]
pub async fn qoder_list_agents() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let (project_dir, user_dir) = qoder_agents_dirs()?;
        let result = list_markdown_entities(&project_dir, &user_dir, |name, _path, content| {
            let (frontmatter, body): (QoderAgentFrontmatter, String) =
                parse_markdown_frontmatter(content)?;
            Ok(json!({
                "name": frontmatter.name.unwrap_or_else(|| name.to_string()),
                "description": frontmatter
                    .description
                    .or_else(|| first_non_empty_line(&body))
                    .unwrap_or_else(|| "Qoder subagent".to_string()),
                "model": "qoder-subagent",
                "tools": frontmatter.tools,
                "system_prompt": body,
                "disabled": false,
            }))
        })?;

        Ok(json!({
            "agents": result.get("items").cloned().unwrap_or_else(|| json!([])),
            "folders": result.get("folders").cloned().unwrap_or_else(|| json!([])),
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn qoder_add_agent(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        if name.trim().is_empty() {
            return Err("Subagent 名称不能为空".to_string());
        }
        let (project_dir, _user_dir) = qoder_agents_dirs()?;
        let folder = config
            .get("folder")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let system_prompt = config
            .get("system_prompt")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let tools = config
            .get("tools")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let description = config
            .get("description")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| first_non_empty_line(&system_prompt))
            .unwrap_or_else(|| "Qoder subagent".to_string());

        let path = build_nested_markdown_path(&project_dir, &folder, &name);
        if path.exists() {
            return Err(format!("Subagent '{name}' 已存在"));
        }
        let content = render_markdown_with_frontmatter(
            &QoderAgentFrontmatter {
                name: Some(name.clone()),
                description: Some(description),
                tools,
            },
            &system_prompt,
        )?;
        ensure_parent_dir(&path)?;
        std::fs::write(&path, content).map_err(|e| format!("写入 Subagent 文件失败: {e}"))?;
        Ok(json!({ "message": format!("Subagent '{name}' 添加成功") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn qoder_update_agent(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let (project_dir, user_dir) = qoder_agents_dirs()?;
        let path = find_markdown_entity_file(&project_dir, &user_dir, &name)?
            .ok_or_else(|| format!("Subagent '{name}' 不存在"))?;
        let system_prompt = config
            .get("system_prompt")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
        let tools = config
            .get("tools")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let description = config
            .get("description")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| first_non_empty_line(&system_prompt))
            .unwrap_or_else(|| "Qoder subagent".to_string());
        let content = render_markdown_with_frontmatter(
            &QoderAgentFrontmatter {
                name: Some(name.clone()),
                description: Some(description),
                tools,
            },
            &system_prompt,
        )?;
        std::fs::write(&path, content).map_err(|e| format!("写入 Subagent 文件失败: {e}"))?;
        Ok(json!({ "message": format!("Subagent '{name}' 更新成功") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn qoder_delete_agent(name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let (project_dir, user_dir) = qoder_agents_dirs()?;
        let path = find_markdown_entity_file(&project_dir, &user_dir, &name)?
            .ok_or_else(|| format!("Subagent '{name}' 不存在"))?;
        std::fs::remove_file(&path).map_err(|e| format!("删除 Subagent 文件失败: {e}"))?;
        Ok(format!("Subagent '{name}' 删除成功"))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn qoder_toggle_agent(_name: String, _enabled: bool) -> Result<Value, String> {
    Err("Qoder Subagent 不支持启用/禁用切换".to_string())
}

// ── Hooks ──

#[tauri::command]
pub async fn qoder_list_hooks() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let settings = read_qoder_settings_merged()?;
        Ok(json!({ "hooks": qoder_hooks_list_from_settings(&settings) }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn qoder_add_hook(config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let command = config
            .get("command")
            .and_then(Value::as_str)
            .ok_or_else(|| "Hook command 不能为空".to_string())?;
        let mut settings = read_qoder_settings_merged()?;
        let root = settings
            .as_object_mut()
            .ok_or_else(|| "Qoder settings 必须是 JSON 对象".to_string())?;
        let hooks = root
            .entry("hooks")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .ok_or_else(|| "hooks 字段格式错误".to_string())?;
        let notifications = hooks
            .entry("Notification")
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .ok_or_else(|| "hooks.Notification 字段格式错误".to_string())?;
        notifications.push(build_qoder_hook_entry(command));
        let path = qoder_settings_write_path()?;
        write_json_atomic(&path, &settings)?;
        Ok(json!({ "message": "Qoder Hook 添加成功" }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn qoder_update_hook(index: usize, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let command = config
            .get("command")
            .and_then(Value::as_str)
            .ok_or_else(|| "Hook command 不能为空".to_string())?;
        let mut settings = read_qoder_settings_merged()?;
        let notifications = settings
            .as_object_mut()
            .and_then(|root| root.get_mut("hooks"))
            .and_then(Value::as_object_mut)
            .and_then(|hooks| hooks.get_mut("Notification"))
            .and_then(Value::as_array_mut)
            .ok_or_else(|| "未找到 Qoder Notification hooks".to_string())?;
        if index >= notifications.len() {
            return Err("Hook 索引超出范围".to_string());
        }
        notifications[index] = build_qoder_hook_entry(command);
        let path = qoder_settings_write_path()?;
        write_json_atomic(&path, &settings)?;
        Ok(json!({ "message": "Qoder Hook 更新成功" }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn qoder_delete_hook(index: usize) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut settings = read_qoder_settings_merged()?;
        let notifications = settings
            .as_object_mut()
            .and_then(|root| root.get_mut("hooks"))
            .and_then(Value::as_object_mut)
            .and_then(|hooks| hooks.get_mut("Notification"))
            .and_then(Value::as_array_mut)
            .ok_or_else(|| "未找到 Qoder Notification hooks".to_string())?;
        if index >= notifications.len() {
            return Err("Hook 索引超出范围".to_string());
        }
        notifications.remove(index);
        let path = qoder_settings_write_path()?;
        write_json_atomic(&path, &settings)?;
        Ok("Qoder Hook 删除成功".to_string())
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}
