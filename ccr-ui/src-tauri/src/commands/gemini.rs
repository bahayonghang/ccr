//! Antigravity CLI 命令 — Settings/MCP/Slash Commands/Extensions。
//!
//! Persisted platform key and Tauri command names remain `gemini` for CCR
//! compatibility. Antigravity stores its CLI config under
//! `~/.gemini/antigravity-cli/`.
//!
//! Settings file: ~/.gemini/antigravity-cli/settings.json
//! MCP file:      ~/.gemini/antigravity-cli/mcp_config.json
//! Slash commands remain legacy-compatible until Antigravity publishes a stable
//! replacement path: ~/.gemini/commands/*.toml and project .gemini/commands.

use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::Write as IoWrite;
use std::path::PathBuf;

// ── Config file helpers ──

fn antigravity_cli_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    let dir = home.join(".gemini").join("antigravity-cli");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("创建 Antigravity CLI 配置目录失败: {e}"))?;
    }
    Ok(dir)
}

/// 定位 ~/.gemini/antigravity-cli/settings.json
fn gemini_config_path() -> Result<PathBuf, String> {
    Ok(antigravity_cli_dir()?.join("settings.json"))
}

/// 定位 ~/.gemini/antigravity-cli/mcp_config.json
fn gemini_mcp_config_path() -> Result<PathBuf, String> {
    Ok(antigravity_cli_dir()?.join("mcp_config.json"))
}

/// 读取 Antigravity CLI settings.json，不存在时返回空对象
fn read_gemini_config() -> Result<Value, String> {
    let path = gemini_config_path()?;
    if !path.exists() {
        return Ok(json!({}));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取 Antigravity CLI 配置文件失败: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("解析 Antigravity CLI JSON 失败: {e}"))
}

/// 原子写入 Antigravity CLI settings.json
fn write_gemini_config(config: &Value) -> Result<(), String> {
    let path = gemini_config_path()?;
    let parent = path.parent().ok_or_else(|| "无法获取父目录".to_string())?;
    let json_str = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化 Antigravity CLI 配置失败: {e}"))?;
    let mut tmp =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| format!("创建临时文件失败: {e}"))?;
    tmp.write_all(json_str.as_bytes())
        .map_err(|e| format!("写入临时文件失败: {e}"))?;
    tmp.persist(&path)
        .map_err(|e| format!("持久化配置文件失败: {e}"))?;
    Ok(())
}

/// 读取 Antigravity CLI mcp_config.json，不存在时返回空对象
fn read_gemini_mcp_config() -> Result<Value, String> {
    let path = gemini_mcp_config_path()?;
    if !path.exists() {
        return Ok(json!({}));
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("读取 Antigravity MCP 配置失败: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("解析 Antigravity MCP JSON 失败: {e}"))
}

/// 原子写入 Antigravity CLI mcp_config.json
fn write_gemini_mcp_config(config: &Value) -> Result<(), String> {
    let path = gemini_mcp_config_path()?;
    let parent = path.parent().ok_or_else(|| "无法获取父目录".to_string())?;
    let json_str = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化 Antigravity MCP 配置失败: {e}"))?;
    let mut tmp =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| format!("创建临时文件失败: {e}"))?;
    tmp.write_all(json_str.as_bytes())
        .map_err(|e| format!("写入临时文件失败: {e}"))?;
    tmp.persist(&path)
        .map_err(|e| format!("持久化配置文件失败: {e}"))?;
    Ok(())
}

fn normalize_mcp_server_for_write(mut config: Value) -> Value {
    let Some(obj) = config.as_object_mut() else {
        return config;
    };

    obj.remove("name");

    let remote_url = obj
        .remove("serverUrl")
        .or_else(|| obj.remove("url"))
        .or_else(|| obj.remove("httpUrl"));

    if let Some(url) = remote_url {
        obj.insert("serverUrl".to_string(), url);
    }

    config
}

fn normalize_mcp_server_for_read(name: String, server: Value) -> Value {
    let mut obj = server.as_object().cloned().unwrap_or_default();
    if !obj.contains_key("url")
        && let Some(url) = obj.get("serverUrl").or_else(|| obj.get("httpUrl")).cloned()
    {
        obj.insert("url".to_string(), url);
    }
    obj.insert("name".to_string(), json!(name));
    Value::Object(obj)
}

// ── Slash command helpers ──

/// 查找项目根目录（包含 .git 的目录）
fn find_project_root() -> PathBuf {
    let start = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut current = start.as_path();
    loop {
        if current.join(".git").exists() {
            return current.to_path_buf();
        }
        match current.parent() {
            Some(p) => current = p,
            None => return start,
        }
    }
}

/// 返回 (project_commands_dir, user_commands_dir)
fn gemini_commands_dirs() -> Result<(PathBuf, PathBuf), String> {
    let project = find_project_root().join(".gemini").join("commands");
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    let user = home.join(".gemini").join("commands");
    Ok((project, user))
}

/// 从两个目录中收集所有 .toml 命令文件，项目级优先于用户级
fn list_toml_commands(project_dir: &PathBuf, user_dir: &PathBuf) -> Result<Value, String> {
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
            if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }
            let rel = path.strip_prefix(base).map_err(|e| e.to_string())?;
            let rel_no_ext = rel.with_extension("");
            let key = rel_no_ext.to_string_lossy().replace('\\', "/");
            // project_dir entries overwrite user_dir entries
            chosen.insert(key, path.to_path_buf());
        }
    }

    let mut commands = Vec::new();
    let mut folders_set = std::collections::HashSet::new();
    for (key, path) in &chosen {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("读取命令文件失败: {e}"))?;
        let value: toml::Value =
            toml::from_str(&content).map_err(|e| format!("解析 TOML 失败: {e}"))?;
        let prompt = value
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("缺少 prompt 字段: {key}"))?
            .to_string();
        let description = value
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                prompt
                    .lines()
                    .find(|l| !l.trim().is_empty())
                    .unwrap_or("Command")
                    .trim()
                    .to_string()
            });
        let (folder, name) = match key.rsplit_once('/') {
            Some((f, n)) => (f.to_string(), n.to_string()),
            None => (String::new(), key.clone()),
        };
        if !folder.is_empty() {
            folders_set.insert(folder.clone());
        }
        commands.push(json!({
            "name": name,
            "description": description,
            "command": prompt,
            "folder": folder,
        }));
    }
    commands.sort_by(|a, b| {
        a["name"]
            .as_str()
            .unwrap_or("")
            .cmp(b["name"].as_str().unwrap_or(""))
    });
    let folders: Vec<String> = folders_set.into_iter().collect();
    Ok(json!({ "commands": commands, "folders": folders }))
}

/// 按名称查找命令文件
fn find_command_file(
    project_dir: &PathBuf,
    user_dir: &PathBuf,
    name: &str,
) -> Result<Option<PathBuf>, String> {
    let mut matches = Vec::new();
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
            if path.extension().and_then(|s| s.to_str()) == Some("toml")
                && path.file_stem().and_then(|s| s.to_str()) == Some(name)
            {
                matches.push(path.to_path_buf());
            }
        }
    }
    match matches.len() {
        0 => Ok(None),
        1 => Ok(Some(matches.remove(0))),
        _ => Err("存在多个同名命令文件".to_string()),
    }
}

// ── Settings ──

#[tauri::command]
pub async fn gemini_get_settings() -> Result<Value, String> {
    tokio::task::spawn_blocking(read_gemini_config)
        .await
        .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn gemini_update_settings(settings: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        write_gemini_config(&settings)?;
        Ok(json!({ "message": "Antigravity CLI 配置更新成功" }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

// ── MCP Servers ──

#[tauri::command]
pub async fn gemini_list_mcp_servers() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let config = read_gemini_mcp_config()?;
        let servers = config
            .get("mcpServers")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let list: Vec<Value> = servers
            .into_iter()
            .map(|(name, server)| normalize_mcp_server_for_read(name, server))
            .collect();
        Ok(json!(list))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn gemini_add_mcp_server(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut full = read_gemini_mcp_config()?;
        let servers = full
            .as_object_mut()
            .ok_or_else(|| "配置不是对象".to_string())?
            .entry("mcpServers")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .ok_or_else(|| "mcpServers 格式错误".to_string())?;
        if servers.contains_key(&name) {
            return Err(format!("MCP 服务器 '{name}' 已存在"));
        }
        servers.insert(name.clone(), normalize_mcp_server_for_write(config));
        write_gemini_mcp_config(&full)?;
        Ok(json!({ "message": format!("MCP 服务器 '{name}' 添加成功") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn gemini_update_mcp_server(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut full = read_gemini_mcp_config()?;
        let servers = full
            .as_object_mut()
            .ok_or_else(|| "配置不是对象".to_string())?
            .entry("mcpServers")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .ok_or_else(|| "mcpServers 格式错误".to_string())?;
        if !servers.contains_key(&name) {
            return Err(format!("MCP 服务器 '{name}' 不存在"));
        }
        servers.insert(name.clone(), normalize_mcp_server_for_write(config));
        write_gemini_mcp_config(&full)?;
        Ok(json!({ "message": format!("MCP 服务器 '{name}' 更新成功") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn gemini_delete_mcp_server(name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut full = read_gemini_mcp_config()?;
        let servers = full
            .as_object_mut()
            .ok_or_else(|| "配置不是对象".to_string())?
            .get_mut("mcpServers")
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| "配置中没有 MCP 服务器".to_string())?;
        if servers.remove(&name).is_none() {
            return Err(format!("MCP 服务器 '{name}' 不存在"));
        }
        write_gemini_mcp_config(&full)?;
        Ok(format!("MCP 服务器 '{name}' 删除成功"))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

// ── Slash Commands ──

#[tauri::command]
pub async fn gemini_list_slash_commands() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let (proj, user) = gemini_commands_dirs()?;
        list_toml_commands(&proj, &user)
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn gemini_add_slash_command(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let (proj, _user) = gemini_commands_dirs()?;
        let folder = config
            .get("folder")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let mut target = proj.clone();
        if !folder.is_empty() {
            target = target.join(&folder);
        }
        std::fs::create_dir_all(&target).map_err(|e| format!("创建目录失败: {e}"))?;
        let description = config
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let command = config
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 command 字段".to_string())?
            .to_string();
        let toml_content = format!(
            "description = {}\nprompt = {}\n",
            toml::Value::String(description),
            toml::Value::String(command),
        );
        let file_path = target.join(format!("{name}.toml"));
        std::fs::write(&file_path, toml_content).map_err(|e| format!("写入命令文件失败: {e}"))?;
        Ok(json!({ "message": format!("斜杠命令 '{name}' 添加成功") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn gemini_update_slash_command(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let (proj, user) = gemini_commands_dirs()?;
        let target = find_command_file(&proj, &user, &name)?
            .ok_or_else(|| format!("斜杠命令 '{name}' 不存在"))?;
        let description = config
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let command = config
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 command 字段".to_string())?
            .to_string();
        let toml_content = format!(
            "description = {}\nprompt = {}\n",
            toml::Value::String(description),
            toml::Value::String(command),
        );
        std::fs::write(&target, toml_content).map_err(|e| format!("写入命令文件失败: {e}"))?;
        Ok(json!({ "message": format!("斜杠命令 '{name}' 更新成功") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn gemini_delete_slash_command(name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let (proj, user) = gemini_commands_dirs()?;
        let target = find_command_file(&proj, &user, &name)?
            .ok_or_else(|| format!("斜杠命令 '{name}' 不存在"))?;
        std::fs::remove_file(&target).map_err(|e| format!("删除命令文件失败: {e}"))?;
        Ok(format!("斜杠命令 '{name}' 删除成功"))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

// ── Extensions ──

#[tauri::command]
pub async fn gemini_list_extensions() -> Result<Value, String> {
    // Antigravity plugins/extensions 通过 ~/.gemini/antigravity-cli/extensions/ 目录管理
    tokio::task::spawn_blocking(|| {
        let ext_dir = antigravity_cli_dir()?.join("extensions");
        if !ext_dir.exists() {
            return Ok(json!([]));
        }
        let mut extensions = Vec::new();
        let entries = std::fs::read_dir(&ext_dir)
            .map_err(|e| format!("读取 extensions 目录失败: {e}"))?;
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                // 读取 extension 的 manifest (package.json 或 gemini-extension.json)
                let manifest_path = path.join("gemini-extension.json");
                let pkg_path = path.join("package.json");
                let manifest: Value = if manifest_path.exists() {
                    std::fs::read_to_string(&manifest_path)
                        .ok()
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or(json!({}))
                } else if pkg_path.exists() {
                    std::fs::read_to_string(&pkg_path)
                        .ok()
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or(json!({}))
                } else {
                    json!({})
                };
                extensions.push(json!({
                    "name": name,
                    "description": manifest.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                    "version": manifest.get("version").and_then(|v| v.as_str()).unwrap_or(""),
                    "path": path.to_string_lossy(),
                }));
            }
        }
        Ok(json!(extensions))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}
