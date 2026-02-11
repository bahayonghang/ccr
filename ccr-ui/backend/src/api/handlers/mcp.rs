use axum::{Json, extract::Path};
use serde_json::json;

use crate::api::handlers::response::ApiSuccess;
use crate::core::error::{ApiError, ApiResult};
use crate::managers::config::claude_manager::{ClaudeConfigManager, McpServerConfig};
use crate::models::api::{McpServerRequest, McpServerWithName};

/// GET /api/mcp/servers - List all MCP servers
pub async fn list_mcp_servers() -> ApiResult<ApiSuccess<serde_json::Value>> {
    let manager = ClaudeConfigManager::default()?;
    let servers = manager
        .get_mcp_servers()
        .map_err(|e| ApiError::internal(format!("Failed to read MCP servers: {}", e)))?;

    let servers_list: Vec<McpServerWithName> = servers
        .into_iter()
        .map(|(name, config)| McpServerWithName {
            name,
            command: config.command.unwrap_or_default(),
            args: config.args.unwrap_or_default(),
            env: config.env.unwrap_or_default(),
            server_type: config.server_type,
            url: config.url,
            disabled: config.disabled.unwrap_or(false),
        })
        .collect();

    Ok(ApiSuccess(json!({ "servers": servers_list })))
}

/// POST /api/mcp/servers - Add a new MCP server
pub async fn add_mcp_server(
    Json(req): Json<McpServerRequest>,
) -> ApiResult<ApiSuccess<&'static str>> {
    let manager = ClaudeConfigManager::default()?;

    let server_config = McpServerConfig {
        command: Some(req.command.clone()),
        args: Some(req.args.clone()),
        env: req.env.clone(),
        server_type: None,
        url: None,
        disabled: req.disabled,
    };

    manager
        .add_mcp_server(req.name.clone(), server_config)
        .map_err(|e| ApiError::internal(format!("Failed to add MCP server: {}", e)))?;

    Ok(ApiSuccess("MCP server added successfully"))
}

/// PATCH /api/mcp/servers/:name - Update an existing MCP server
pub async fn update_mcp_server(
    Path(name): Path<String>,
    Json(req): Json<McpServerRequest>,
) -> ApiResult<ApiSuccess<&'static str>> {
    let manager = ClaudeConfigManager::default()?;

    let server_config = McpServerConfig {
        command: Some(req.command.clone()),
        args: Some(req.args.clone()),
        env: req.env.clone(),
        server_type: None,
        url: None,
        disabled: req.disabled,
    };

    manager
        .update_mcp_server(&name, server_config)
        .map_err(|e| ApiError::internal(format!("Failed to update MCP server: {}", e)))?;

    Ok(ApiSuccess("MCP server updated successfully"))
}

/// DELETE /api/mcp/servers/:name - Delete an MCP server
pub async fn delete_mcp_server(Path(name): Path<String>) -> ApiResult<ApiSuccess<&'static str>> {
    let manager = ClaudeConfigManager::default()?;

    manager
        .delete_mcp_server(&name)
        .map_err(|e| ApiError::not_found(format!("Failed to delete MCP server: {}", e)))?;

    Ok(ApiSuccess("MCP server deleted successfully"))
}

/// PATCH /api/mcp/servers/:name/toggle - Toggle MCP server enabled/disabled state
pub async fn toggle_mcp_server(Path(name): Path<String>) -> ApiResult<ApiSuccess<String>> {
    let manager = ClaudeConfigManager::default()?;

    let mut servers = manager
        .get_mcp_servers()
        .map_err(|e| ApiError::internal(format!("Failed to get MCP servers: {}", e)))?;

    let server = servers
        .get_mut(&name)
        .ok_or_else(|| ApiError::not_found(format!("MCP server '{}' not found", name)))?;

    // Toggle disabled state
    let new_state = !server.disabled.unwrap_or(false);
    server.disabled = Some(new_state);

    manager
        .update_mcp_server(&name, server.clone())
        .map_err(|e| ApiError::internal(format!("Failed to update MCP server: {}", e)))?;

    let state_str = if new_state { "disabled" } else { "enabled" };
    Ok(ApiSuccess(format!("MCP server {} {}", state_str, name)))
}
