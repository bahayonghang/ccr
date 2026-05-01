//! MCP 预设管理命令

use ccr_config::Platform;
use ccr_skills::McpPresetManager;
use serde_json::Value;

/// 将平台名称字符串解析为 Platform 枚举
fn parse_platform(s: &str) -> Option<Platform> {
    match s.to_lowercase().as_str() {
        "claude" => Some(Platform::Claude),
        "codex" => Some(Platform::Codex),
        "gemini" => Some(Platform::Gemini),
        _ => None,
    }
}

fn requested_platform_ids(platforms: Option<Vec<String>>) -> Vec<String> {
    platforms.unwrap_or_else(|| {
        vec![
            "codex".to_string(),
            "gemini".to_string(),
        ]
    })
}

#[tauri::command]
pub async fn list_mcp_presets() -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(|| {
        let manager = McpPresetManager::new(Platform::Claude)
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
        let manager = McpPresetManager::new(Platform::Claude)
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

        let sync_manager = ccr_skills::McpSyncManager::new();
        let requested_platforms = requested_platform_ids(platforms);

        let mut outcomes = Vec::new();
        for platform in requested_platforms {
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

        let target =
            parse_platform(&platform).ok_or_else(|| format!("Unknown platform: {platform}"))?;

        let sync_manager = ccr_skills::McpSyncManager::new();
        sync_manager
            .sync_preset(&preset_id, custom_env, target)
            .map_err(|e| format!("Failed to install preset to {platform}: {e}"))?;

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
        let sync_manager = ccr_skills::McpSyncManager::new();
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
        let sync_manager = ccr_skills::McpSyncManager::new();
        let servers = sync_manager
            .list_source_mcp_servers()
            .map_err(|e| format!("Failed to load source MCP servers: {e}"))?;
        if !servers.contains_key(&name) {
            return Err(format!("MCP server '{name}' not found in source platform"));
        }

        let mut outcomes = Vec::new();
        for platform in requested_platform_ids(platforms) {
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
        let sync_manager = ccr_skills::McpSyncManager::new();
        let server_specs = sync_manager
            .list_source_mcp_servers()
            .map_err(|e| format!("Failed to load source MCP servers: {e}"))?;
        let requested_platforms = requested_platform_ids(platforms);

        let mut servers = Vec::new();
        for (server_name, _spec) in server_specs {
            let mut outcomes = Vec::new();
            for platform in &requested_platforms {
                let target = parse_platform(platform)
                    .ok_or_else(|| format!("Unknown platform: {platform}"))?;
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
