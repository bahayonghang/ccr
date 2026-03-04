//! Droid 命令 — Settings/MCP/Agents/Plugins/Slash Commands/Models 等。
//!
//! 配置文件路径: ~/.factory/settings.json
//! 结构: {
//!   "customModels": [...],
//!   "mcpServers": { "name": {...} },
//!   "agents": { "name": {...} },
//!   "plugins": { "name": {...} },
//!   "slashCommands": { "name": {...} },
//!   ... (其他设置字段原样保留)
//! }

use serde_json::Value;
use std::path::PathBuf;

// ── 内部工具函数 ──

/// 返回 ~/.factory/settings.json 路径
fn droid_settings_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    Ok(home.join(".factory").join("settings.json"))
}

/// 读取 ~/.factory/settings.json，文件不存在时返回空 Object
async fn read_settings() -> Result<Value, String> {
    let path = droid_settings_path()?;
    tokio::task::spawn_blocking(move || {
        if !path.exists() {
            return Ok(serde_json::json!({}));
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("读取 Droid settings.json 失败: {e}"))?;
        serde_json::from_str(&content).map_err(|e| format!("解析 Droid settings.json 失败: {e}"))
    })
    .await
    .map_err(|e| format!("spawn_blocking 失败: {e}"))?
}

/// 原子写入 ~/.factory/settings.json
async fn write_settings(settings: Value) -> Result<(), String> {
    let path = droid_settings_path()?;
    tokio::task::spawn_blocking(move || {
        // 确保目录存在
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建 ~/.factory 目录失败: {e}"))?;
        }
        // 原子写入：先写临时文件再重命名
        let tmp_path = path.with_extension("json.tmp");
        let content = serde_json::to_string_pretty(&settings)
            .map_err(|e| format!("序列化 Droid settings 失败: {e}"))?;
        std::fs::write(&tmp_path, &content).map_err(|e| format!("写入临时文件失败: {e}"))?;
        std::fs::rename(&tmp_path, &path).map_err(|e| format!("原子重命名失败: {e}"))?;
        Ok(())
    })
    .await
    .map_err(|e| format!("spawn_blocking 失败: {e}"))?
}

// ── Settings ──

#[tauri::command]
pub async fn droid_get_settings() -> Result<Value, String> {
    read_settings().await
}

#[tauri::command]
pub async fn droid_update_settings(settings: Value) -> Result<Value, String> {
    // 读取现有设置，合并传入的字段（浅合并，顶层 key）
    let mut current = read_settings().await?;
    if let (Some(cur_obj), Some(new_obj)) = (current.as_object_mut(), settings.as_object()) {
        for (k, v) in new_obj {
            cur_obj.insert(k.clone(), v.clone());
        }
    } else {
        current = settings;
    }
    write_settings(current.clone()).await?;
    Ok(current)
}

// ── MCP Servers ──
// 存储在 settings["mcpServers"] = { "name": { ...config } }

#[tauri::command]
pub async fn droid_list_mcp_servers() -> Result<Value, String> {
    let settings = read_settings().await?;
    let servers = settings
        .get("mcpServers")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    Ok(servers)
}

#[tauri::command]
pub async fn droid_add_mcp_server(name: String, config: Value) -> Result<Value, String> {
    let mut settings = read_settings().await?;
    let servers = settings
        .as_object_mut()
        .ok_or("settings 不是 object")?
        .entry("mcpServers")
        .or_insert_with(|| serde_json::json!({}));
    servers
        .as_object_mut()
        .ok_or("mcpServers 不是 object")?
        .insert(name, config.clone());
    write_settings(settings).await?;
    Ok(config)
}

#[tauri::command]
pub async fn droid_update_mcp_server(name: String, config: Value) -> Result<Value, String> {
    let mut settings = read_settings().await?;
    let servers = settings
        .as_object_mut()
        .ok_or("settings 不是 object")?
        .entry("mcpServers")
        .or_insert_with(|| serde_json::json!({}));
    let servers_obj = servers.as_object_mut().ok_or("mcpServers 不是 object")?;
    if !servers_obj.contains_key(&name) {
        return Err(format!("MCP 服务器 '{name}' 不存在"));
    }
    servers_obj.insert(name, config.clone());
    write_settings(settings).await?;
    Ok(config)
}

#[tauri::command]
pub async fn droid_delete_mcp_server(name: String) -> Result<String, String> {
    let mut settings = read_settings().await?;
    let removed = settings
        .as_object_mut()
        .and_then(|obj| obj.get_mut("mcpServers"))
        .and_then(|s| s.as_object_mut())
        .map(|servers| servers.remove(&name).is_some())
        .unwrap_or(false);
    if !removed {
        return Err(format!("MCP 服务器 '{name}' 不存在"));
    }
    write_settings(settings).await?;
    Ok(name)
}

// ── Agents ──
// 存储在 settings["agents"] = { "name": { ...config } }

#[tauri::command]
pub async fn droid_list_agents() -> Result<Value, String> {
    let settings = read_settings().await?;
    let agents = settings
        .get("agents")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    Ok(agents)
}

#[tauri::command]
pub async fn droid_add_agent(name: String, config: Value) -> Result<Value, String> {
    let mut settings = read_settings().await?;
    let agents = settings
        .as_object_mut()
        .ok_or("settings 不是 object")?
        .entry("agents")
        .or_insert_with(|| serde_json::json!({}));
    agents
        .as_object_mut()
        .ok_or("agents 不是 object")?
        .insert(name, config.clone());
    write_settings(settings).await?;
    Ok(config)
}

#[tauri::command]
pub async fn droid_update_agent(name: String, config: Value) -> Result<Value, String> {
    let mut settings = read_settings().await?;
    let agents = settings
        .as_object_mut()
        .ok_or("settings 不是 object")?
        .entry("agents")
        .or_insert_with(|| serde_json::json!({}));
    let agents_obj = agents.as_object_mut().ok_or("agents 不是 object")?;
    if !agents_obj.contains_key(&name) {
        return Err(format!("Agent '{name}' 不存在"));
    }
    agents_obj.insert(name, config.clone());
    write_settings(settings).await?;
    Ok(config)
}

#[tauri::command]
pub async fn droid_delete_agent(name: String) -> Result<String, String> {
    let mut settings = read_settings().await?;
    let removed = settings
        .as_object_mut()
        .and_then(|obj| obj.get_mut("agents"))
        .and_then(|a| a.as_object_mut())
        .map(|agents| agents.remove(&name).is_some())
        .unwrap_or(false);
    if !removed {
        return Err(format!("Agent '{name}' 不存在"));
    }
    write_settings(settings).await?;
    Ok(name)
}

// ── Plugins ──
// 存储在 settings["plugins"] = { "name": { ...config } }

#[tauri::command]
pub async fn droid_list_plugins() -> Result<Value, String> {
    let settings = read_settings().await?;
    let plugins = settings
        .get("plugins")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    Ok(plugins)
}

#[tauri::command]
pub async fn droid_add_plugin(name: String, config: Value) -> Result<Value, String> {
    let mut settings = read_settings().await?;
    let plugins = settings
        .as_object_mut()
        .ok_or("settings 不是 object")?
        .entry("plugins")
        .or_insert_with(|| serde_json::json!({}));
    plugins
        .as_object_mut()
        .ok_or("plugins 不是 object")?
        .insert(name, config.clone());
    write_settings(settings).await?;
    Ok(config)
}

#[tauri::command]
pub async fn droid_update_plugin(name: String, config: Value) -> Result<Value, String> {
    let mut settings = read_settings().await?;
    let plugins = settings
        .as_object_mut()
        .ok_or("settings 不是 object")?
        .entry("plugins")
        .or_insert_with(|| serde_json::json!({}));
    let plugins_obj = plugins.as_object_mut().ok_or("plugins 不是 object")?;
    if !plugins_obj.contains_key(&name) {
        return Err(format!("插件 '{name}' 不存在"));
    }
    plugins_obj.insert(name, config.clone());
    write_settings(settings).await?;
    Ok(config)
}

#[tauri::command]
pub async fn droid_delete_plugin(name: String) -> Result<String, String> {
    let mut settings = read_settings().await?;
    let removed = settings
        .as_object_mut()
        .and_then(|obj| obj.get_mut("plugins"))
        .and_then(|p| p.as_object_mut())
        .map(|plugins| plugins.remove(&name).is_some())
        .unwrap_or(false);
    if !removed {
        return Err(format!("插件 '{name}' 不存在"));
    }
    write_settings(settings).await?;
    Ok(name)
}

// ── Slash Commands ──
// 存储在 settings["slashCommands"] = { "name": { ...config } }

#[tauri::command]
pub async fn droid_list_slash_commands() -> Result<Value, String> {
    let settings = read_settings().await?;
    let cmds = settings
        .get("slashCommands")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    Ok(cmds)
}

#[tauri::command]
pub async fn droid_add_slash_command(name: String, config: Value) -> Result<Value, String> {
    let mut settings = read_settings().await?;
    let cmds = settings
        .as_object_mut()
        .ok_or("settings 不是 object")?
        .entry("slashCommands")
        .or_insert_with(|| serde_json::json!({}));
    cmds.as_object_mut()
        .ok_or("slashCommands 不是 object")?
        .insert(name, config.clone());
    write_settings(settings).await?;
    Ok(config)
}

#[tauri::command]
pub async fn droid_update_slash_command(name: String, config: Value) -> Result<Value, String> {
    let mut settings = read_settings().await?;
    let cmds = settings
        .as_object_mut()
        .ok_or("settings 不是 object")?
        .entry("slashCommands")
        .or_insert_with(|| serde_json::json!({}));
    let cmds_obj = cmds.as_object_mut().ok_or("slashCommands 不是 object")?;
    if !cmds_obj.contains_key(&name) {
        return Err(format!("斜杠命令 '{name}' 不存在"));
    }
    cmds_obj.insert(name, config.clone());
    write_settings(settings).await?;
    Ok(config)
}

#[tauri::command]
pub async fn droid_delete_slash_command(name: String) -> Result<String, String> {
    let mut settings = read_settings().await?;
    let removed = settings
        .as_object_mut()
        .and_then(|obj| obj.get_mut("slashCommands"))
        .and_then(|c| c.as_object_mut())
        .map(|cmds| cmds.remove(&name).is_some())
        .unwrap_or(false);
    if !removed {
        return Err(format!("斜杠命令 '{name}' 不存在"));
    }
    write_settings(settings).await?;
    Ok(name)
}

// ── Models ──
// 读取 settings["customModels"] 数组

#[tauri::command]
pub async fn droid_list_models() -> Result<Value, String> {
    let settings = read_settings().await?;
    let models = settings
        .get("customModels")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    Ok(models)
}
