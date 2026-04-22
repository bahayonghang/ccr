use super::*;

#[tauri::command]
pub async fn claude_list_mcp_servers() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let config =
            read_claude_config().map_err(|e| format!("Failed to read .claude.json: {}", e))?;

        let servers: Vec<Value> = config
            .mcp_servers
            .into_iter()
            .map(|(name, server)| {
                serde_json::json!({
                    "name": name,
                    "command": server.command.unwrap_or_default(),
                    "args": server.args,
                    "env": server.env.unwrap_or_default(),
                    "type": server.server_type,
                    "url": server.url,
                    "disabled": server.disabled.unwrap_or(false),
                })
            })
            .collect();

        Ok(serde_json::json!({ "servers": servers }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_add_mcp_server(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut claude_config =
            read_claude_config().map_err(|e| format!("Failed to read .claude.json: {}", e))?;

        let entry: McpServerEntry = serde_json::from_value(config)
            .map_err(|e| format!("Invalid MCP server config: {}", e))?;

        claude_config.mcp_servers.insert(name, entry);
        write_claude_config(&claude_config)
            .map_err(|e| format!("Failed to write .claude.json: {}", e))?;

        Ok(serde_json::json!({ "success": true }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_update_mcp_server(name: String, config: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut claude_config =
            read_claude_config().map_err(|e| format!("Failed to read .claude.json: {}", e))?;

        if !claude_config.mcp_servers.contains_key(&name) {
            return Err(format!("MCP server '{}' not found", name));
        }

        let entry: McpServerEntry = serde_json::from_value(config)
            .map_err(|e| format!("Invalid MCP server config: {}", e))?;

        claude_config.mcp_servers.insert(name, entry);
        write_claude_config(&claude_config)
            .map_err(|e| format!("Failed to write .claude.json: {}", e))?;

        Ok(serde_json::json!({ "success": true }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_delete_mcp_server(name: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut claude_config =
            read_claude_config().map_err(|e| format!("Failed to read .claude.json: {}", e))?;

        if claude_config.mcp_servers.remove(&name).is_none() {
            return Err(format!("MCP server '{}' not found", name));
        }

        write_claude_config(&claude_config)
            .map_err(|e| format!("Failed to write .claude.json: {}", e))?;

        Ok(format!("MCP server '{}' deleted", name))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
