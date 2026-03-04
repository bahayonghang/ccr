//! MCP 预设管理命令

use serde_json::Value;

/// 将平台名称字符串解析为 Platform 枚举
fn parse_platform(s: &str) -> Option<ccr::Platform> {
    match s.to_lowercase().as_str() {
        "claude" => Some(ccr::Platform::Claude),
        "codex" => Some(ccr::Platform::Codex),
        "gemini" => Some(ccr::Platform::Gemini),
        "qwen" => Some(ccr::Platform::Qwen),
        "iflow" => Some(ccr::Platform::IFlow),
        "droid" => Some(ccr::Platform::Droid),
        _ => None,
    }
}

/// 将可选平台列表转换为 Vec<Platform>，None 时返回所有平台（除 Claude）
fn resolve_platforms(platforms: Option<Vec<String>>) -> Vec<ccr::Platform> {
    match platforms {
        Some(list) => list.iter().filter_map(|s| parse_platform(s)).collect(),
        None => vec![
            ccr::Platform::Codex,
            ccr::Platform::Gemini,
            ccr::Platform::Qwen,
            ccr::Platform::IFlow,
            ccr::Platform::Droid,
        ],
    }
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
        let target_platforms = resolve_platforms(platforms);

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
        let results = sync_manager
            .sync_preset_to_all(&preset_id, custom_env, &target_platforms)
            .map_err(|e| format!("Failed to install preset: {e}"))?;

        let outcomes: Vec<Value> = results
            .into_iter()
            .map(|(platform, res)| {
                serde_json::json!({
                    "platform": format!("{:?}", platform),
                    "success": res.is_ok(),
                    "error": res.err().map(|e| e.to_string()),
                })
            })
            .collect();

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
        let target = parse_platform(&platform)
            .ok_or_else(|| format!("Unknown platform: {platform}"))?;

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
        let target_platforms = resolve_platforms(platforms);

        let sync_manager = ccr::managers::McpSyncManager::new();
        let results = sync_manager
            .sync_mcp_server(&name, &target_platforms)
            .map_err(|e| format!("Failed to sync MCP server: {e}"))?;

        let outcomes: Vec<Value> = results
            .into_iter()
            .map(|(platform, res)| {
                serde_json::json!({
                    "platform": format!("{:?}", platform),
                    "success": res.is_ok(),
                    "error": res.err().map(|e| e.to_string()),
                })
            })
            .collect();

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
        let target_platforms = resolve_platforms(platforms);

        let sync_manager = ccr::managers::McpSyncManager::new();
        let all_results = sync_manager
            .sync_all_mcp_servers(&target_platforms)
            .map_err(|e| format!("Failed to sync all MCP servers: {e}"))?;

        let servers: Vec<Value> = all_results
            .into_iter()
            .map(|(server_name, results)| {
                let outcomes: Vec<Value> = results
                    .into_iter()
                    .map(|(platform, res)| {
                        serde_json::json!({
                            "platform": format!("{:?}", platform),
                            "success": res.is_ok(),
                            "error": res.err().map(|e| e.to_string()),
                        })
                    })
                    .collect();
                serde_json::json!({
                    "server_name": server_name,
                    "results": outcomes,
                })
            })
            .collect();

        Ok::<_, String>(serde_json::json!({ "servers": servers }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(result)
}
