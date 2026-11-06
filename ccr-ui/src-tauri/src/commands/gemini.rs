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

use ccr_app::resources::platform_files::{
    Kind as ResourceKind, Store as ResourceStore, antigravity as resource_codec,
};
use serde_json::{Value, json};
use std::io::Write as IoWrite;
use std::path::PathBuf;

use super::wire::OpenJsonValueDto;

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

fn command_stores() -> Result<(ResourceStore, ResourceStore), String> {
    let (project, user) = gemini_commands_dirs()?;
    let project = ResourceStore::with_default_storage(project, ResourceKind::AntigravityCommand)
        .map_err(|error| error.to_string())?;
    let user = ResourceStore::with_default_storage(user, ResourceKind::AntigravityCommand)
        .map_err(|error| error.to_string())?;
    Ok((project, user))
}

// ── Settings ──

#[ccr_tauri_command_macros::command]
pub async fn gemini_get_settings() -> Result<OpenJsonValueDto, String> {
    tokio::task::spawn_blocking(read_gemini_config)
        .await
        .map_err(|e| format!("任务执行失败: {e}"))??
        .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn gemini_update_settings(
    settings: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let settings = settings.into();
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        write_gemini_config(&settings)?;
        Ok(json!({ "message": "Antigravity CLI 配置更新成功" }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}

// ── MCP Servers ──

#[ccr_tauri_command_macros::command]
pub async fn gemini_list_mcp_servers() -> Result<OpenJsonValueDto, String> {
    tokio::task::spawn_blocking(|| -> Result<Value, String> {
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
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn gemini_add_mcp_server(
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let config = config.into();
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
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
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn gemini_update_mcp_server(
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let config = config.into();
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
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
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn gemini_delete_mcp_server(name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || -> Result<String, String> {
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

#[ccr_tauri_command_macros::command]
pub async fn gemini_list_slash_commands() -> Result<OpenJsonValueDto, String> {
    tokio::task::spawn_blocking(|| {
        let (project, user) = command_stores()?;
        resource_codec::list_commands(&project, &user)
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn gemini_add_slash_command(
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let config: Value = config.into();
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let (project, _) = command_stores()?;
        resource_codec::add_command(&project, &name, &config)?;
        Ok(json!({ "message": format!("斜杠命令 '{name}' 添加成功") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn gemini_update_slash_command(
    name: String,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let config: Value = config.into();
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let (project, user) = command_stores()?;
        resource_codec::update_command(&project, &user, &name, &config)?;
        Ok(json!({ "message": format!("斜杠命令 '{name}' 更新成功") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn gemini_delete_slash_command(name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let (project, user) = command_stores()?;
        resource_codec::delete_command(&project, &user, &name)?;
        Ok(format!("斜杠命令 '{name}' 删除成功"))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

// ── Extensions ──

#[ccr_tauri_command_macros::command]
pub async fn gemini_list_extensions() -> Result<OpenJsonValueDto, String> {
    // Antigravity plugins/extensions 通过 ~/.gemini/antigravity-cli/extensions/ 目录管理
    tokio::task::spawn_blocking(|| -> Result<Value, String> {
        let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
        let store = ResourceStore::with_default_storage(
            home.join(".gemini/antigravity-cli/extensions"),
            ResourceKind::Extension,
        )
        .map_err(|error| error.to_string())?;
        resource_codec::list_extensions(&store)
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}
