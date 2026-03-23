//! MCP 预设管理命令

use ccr::models::mcp_preset::McpServerSpec;
use serde_json::{Map, Value, json};
use std::path::{Path, PathBuf};

/// 将平台名称字符串解析为 Platform 枚举
fn parse_platform(s: &str) -> Option<ccr::Platform> {
    match s.to_lowercase().as_str() {
        "claude" => Some(ccr::Platform::Claude),
        "codex" => Some(ccr::Platform::Codex),
        "gemini" => Some(ccr::Platform::Gemini),
        "qwen" => Some(ccr::Platform::Qwen),
        "droid" => Some(ccr::Platform::Droid),
        _ => None,
    }
}

fn requested_platform_ids(platforms: Option<Vec<String>>) -> Vec<String> {
    platforms.unwrap_or_else(|| {
        vec![
            "codex".to_string(),
            "gemini".to_string(),
            "qwen".to_string(),
            "qoder".to_string(),
            "droid".to_string(),
        ]
    })
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

fn qoder_mcp_target_path() -> Result<PathBuf, String> {
    if let Some(root) = find_project_root() {
        return Ok(root.join(".mcp.json"));
    }
    let home = dirs::home_dir().ok_or_else(|| "Cannot find home directory".to_string())?;
    Ok(home.join(".qoder.json"))
}

fn read_qoder_mcp_servers(path: &Path) -> Result<Map<String, Value>, String> {
    if !path.exists() {
        return Ok(Map::new());
    }
    let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read Qoder MCP config: {e}"))?;
    let value: Value = if content.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse Qoder MCP config: {e}"))?
    };
    Ok(value
        .get("mcpServers")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default())
}

fn write_qoder_mcp_servers(path: &Path, servers: &Map<String, Value>) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create Qoder MCP directory: {e}"))?;
    }

    let mut root = if path.exists() {
        let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read Qoder MCP config: {e}"))?;
        if content.trim().is_empty() {
            json!({})
        } else {
            serde_json::from_str::<Value>(&content).map_err(|e| format!("Failed to parse Qoder MCP config: {e}"))?
        }
    } else {
        json!({})
    };

    let obj = root
        .as_object_mut()
        .ok_or_else(|| "Qoder MCP config must be a JSON object".to_string())?;
    obj.insert("mcpServers".to_string(), Value::Object(servers.clone()));

    let parent = path.parent().ok_or_else(|| "Invalid Qoder MCP path".to_string())?;
    let tmp =
        tempfile::NamedTempFile::new_in(parent).map_err(|e| format!("Failed to create temp file: {e}"))?;
    std::fs::write(
        tmp.path(),
        serde_json::to_string_pretty(&root).map_err(|e| format!("Failed to serialize Qoder MCP config: {e}"))?,
    )
    .map_err(|e| format!("Failed to write temp file: {e}"))?;
    tmp.persist(path)
        .map_err(|e| format!("Failed to persist Qoder MCP config: {e}"))?;
    Ok(())
}

fn qoder_spec_to_json(spec: &McpServerSpec) -> Value {
    let mut obj = serde_json::Map::new();
    if let Some(command) = &spec.command {
        obj.insert("command".to_string(), json!(command));
    }
    if !spec.args.is_empty() {
        obj.insert("args".to_string(), json!(spec.args));
    }
    if !spec.env.is_empty() {
        obj.insert("env".to_string(), json!(spec.env));
    }
    if let Some(url) = &spec.url {
        obj.insert("url".to_string(), json!(url));
    }
    Value::Object(obj)
}

fn install_qoder_mcp_server(name: &str, spec: &McpServerSpec) -> Result<(), String> {
    let path = qoder_mcp_target_path()?;
    let mut servers = read_qoder_mcp_servers(&path)?;
    servers.insert(name.to_string(), qoder_spec_to_json(spec));
    write_qoder_mcp_servers(&path, &servers)
}

fn preset_spec_with_env(
    preset_id: &str,
    custom_env: Option<std::collections::HashMap<String, String>>,
) -> Result<McpServerSpec, String> {
    let manager = ccr::managers::McpPresetManager::new(ccr::Platform::Claude)
        .map_err(|e| format!("Failed to create preset manager: {e}"))?;
    let preset = manager
        .get_preset(preset_id)
        .ok_or_else(|| format!("Preset '{preset_id}' not found"))?;
    let mut spec = preset.server.clone();
    if let Some(env) = custom_env {
        for (key, value) in env {
            spec.env.insert(key, value);
        }
    }
    Ok(spec)
}

#[tauri::command]
pub async fn list_mcp_presets() -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(|| {
        let manager = ccr::managers::McpPresetManager::new(ccr::Platform::Claude)
            .map_err(|e| format!("Failed to create preset manager: {e}"))?;

        let presets = manager.list_presets();
        serde_json::to_value(presets).map_err(|e| format!("Serialization error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}

#[tauri::command]
pub async fn get_mcp_preset(id: String) -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(move || {
        let manager = ccr::managers::McpPresetManager::new(ccr::Platform::Claude)
            .map_err(|e| format!("Failed to create preset manager: {e}"))?;

        let preset = manager.get_preset(&id);
        serde_json::to_value(preset).map_err(|e| format!("Serialization error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}

#[tauri::command]
pub async fn install_mcp_preset(
    preset_id: String,
    platforms: Option<Vec<String>>,
    env_vars: Option<Value>,
) -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(move || {
        // 将 env_vars Value 转换为 HashMap<String, String>
        let custom_env: Option<std::collections::HashMap<String, String>> = match env_vars {
            Some(v) => {
                let map = v
                    .as_object()
                    .ok_or_else(|| "env_vars must be an object".to_string())?
                    .iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect();
                Some(map)
            }
            None => None,
        };

        let sync_manager = ccr::managers::McpSyncManager::new();
        let requested_platforms = requested_platform_ids(platforms);
        let qoder_spec = if requested_platforms.iter().any(|platform| platform == "qoder") {
            Some(preset_spec_with_env(&preset_id, custom_env.clone())?)
        } else {
            None
        };

        let mut outcomes = Vec::new();
        for platform in requested_platforms {
            if platform == "qoder" {
                let result = install_qoder_mcp_server(
                    &preset_id,
                    qoder_spec
                        .as_ref()
                        .ok_or_else(|| "Qoder preset spec missing".to_string())?,
                );
                outcomes.push(serde_json::json!({
                    "platform": "Qoder",
                    "success": result.is_ok(),
                    "error": result.err(),
                }));
                continue;
            }

            let target =
                parse_platform(&platform).ok_or_else(|| format!("Unknown platform: {platform}"))?;
            let result = sync_manager.sync_preset(&preset_id, custom_env.clone(), target);
            outcomes.push(serde_json::json!({
                "platform": format!("{:?}", target),
                "success": result.is_ok(),
                "error": result.err().map(|e| e.to_string()),
            }));
        }

        Ok::<_, String>(serde_json::json!({
            "preset_id": preset_id,
            "results": outcomes,
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}

#[tauri::command]
pub async fn install_mcp_preset_single(
    platform: String,
    preset_id: String,
    env_vars: Option<Value>,
) -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(move || {
        let custom_env: Option<std::collections::HashMap<String, String>> = match env_vars {
            Some(v) => {
                let map = v
                    .as_object()
                    .ok_or_else(|| "env_vars must be an object".to_string())?
                    .iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect();
                Some(map)
            }
            None => None,
        };

        if platform.eq_ignore_ascii_case("qoder") {
            let spec = preset_spec_with_env(&preset_id, custom_env)?;
            install_qoder_mcp_server(&preset_id, &spec)
                .map_err(|e| format!("Failed to install preset to {platform}: {e}"))?;
        } else {
            let target =
                parse_platform(&platform).ok_or_else(|| format!("Unknown platform: {platform}"))?;

            let sync_manager = ccr::managers::McpSyncManager::new();
            sync_manager
                .sync_preset(&preset_id, custom_env, target)
                .map_err(|e| format!("Failed to install preset to {platform}: {e}"))?;
        }

        Ok::<_, String>(serde_json::json!({
            "preset_id": preset_id,
            "platform": platform,
            "success": true,
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}

#[tauri::command]
pub async fn list_source_mcp_servers() -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(|| {
        let sync_manager = ccr::managers::McpSyncManager::new();
        let servers = sync_manager
            .list_source_mcp_servers()
            .map_err(|e| format!("Failed to list source MCP servers: {e}"))?;

        let items: Vec<Value> = servers
            .into_iter()
            .map(|(name, spec)| {
                serde_json::json!({
                    "name": name,
                    "spec": spec,
                })
            })
            .collect();

        Ok::<_, String>(serde_json::json!({ "servers": items, "total": items.len() }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}

#[tauri::command]
pub async fn sync_mcp_server(
    name: String,
    platforms: Option<Vec<String>>,
) -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(move || {
        let sync_manager = ccr::managers::McpSyncManager::new();
        let servers = sync_manager
            .list_source_mcp_servers()
            .map_err(|e| format!("Failed to load source MCP servers: {e}"))?;
        let spec = servers
            .get(&name)
            .ok_or_else(|| format!("MCP server '{name}' not found in source platform"))?
            .clone();

        let mut outcomes = Vec::new();
        for platform in requested_platform_ids(platforms) {
            if platform == "qoder" {
                let result = install_qoder_mcp_server(&name, &spec);
                outcomes.push(serde_json::json!({
                    "platform": "Qoder",
                    "success": result.is_ok(),
                    "error": result.err(),
                }));
                continue;
            }

            let target =
                parse_platform(&platform).ok_or_else(|| format!("Unknown platform: {platform}"))?;
            let result = sync_manager.sync_mcp_server(&name, &[target]);
            let (success, error) = match result {
                Ok(items) => {
                    let inner_error = items
                        .into_iter()
                        .next()
                        .and_then(|(_, inner)| inner.err().map(|e| e.to_string()));
                    (inner_error.is_none(), inner_error)
                }
                Err(error) => (false, Some(error.to_string())),
            };
            outcomes.push(serde_json::json!({
                "platform": format!("{:?}", target),
                "success": success,
                "error": error,
            }));
        }

        Ok::<_, String>(serde_json::json!({
            "server_name": name,
            "results": outcomes,
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}

#[tauri::command]
pub async fn sync_all_mcp_servers(platforms: Option<Vec<String>>) -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(move || {
        let sync_manager = ccr::managers::McpSyncManager::new();
        let server_specs = sync_manager
            .list_source_mcp_servers()
            .map_err(|e| format!("Failed to load source MCP servers: {e}"))?;
        let requested_platforms = requested_platform_ids(platforms);

        let mut servers = Vec::new();
        for (server_name, spec) in server_specs {
            let mut outcomes = Vec::new();
            for platform in &requested_platforms {
                if platform == "qoder" {
                    let result = install_qoder_mcp_server(&server_name, &spec);
                    outcomes.push(serde_json::json!({
                        "platform": "Qoder",
                        "success": result.is_ok(),
                        "error": result.err(),
                    }));
                    continue;
                }

                let target =
                    parse_platform(platform).ok_or_else(|| format!("Unknown platform: {platform}"))?;
                let result = sync_manager.sync_mcp_server(&server_name, &[target]);
                let (success, error) = match result {
                    Ok(items) => {
                        let inner_error = items
                            .into_iter()
                            .next()
                            .and_then(|(_, inner)| inner.err().map(|e| e.to_string()));
                        (inner_error.is_none(), inner_error)
                    }
                    Err(error) => (false, Some(error.to_string())),
                };
                outcomes.push(serde_json::json!({
                    "platform": format!("{:?}", target),
                    "success": success,
                    "error": error,
                }));
            }

            servers.push(serde_json::json!({
                "server_name": server_name,
                "results": outcomes,
            }));
        }

        Ok::<_, String>(serde_json::json!({ "servers": servers }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}
