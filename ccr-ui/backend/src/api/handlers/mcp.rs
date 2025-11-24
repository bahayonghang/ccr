use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::managers::config::claude_manager::{ClaudeConfigManager, McpServerConfig};
use crate::models::api::{McpServerRequest, McpServerWithName, McpServersResponse};

/// GET /api/mcp/servers - List all MCP servers
pub async fn list_mcp_servers() -> impl IntoResponse {
    let manager = match ClaudeConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to initialize config manager: {}", e)
                })),
            )
                .into_response();
        }
    };

    match manager.get_mcp_servers() {
        Ok(servers) => {
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

            (
                StatusCode::OK,
                Json(McpServersResponse {
                    success: true,
                    data: json!({ "servers": servers_list }),
                    message: None,
                }),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to read MCP servers: {}", e)
            })),
        )
            .into_response(),
    }
}

/// POST /api/mcp/servers - Add a new MCP server
pub async fn add_mcp_server(Json(req): Json<McpServerRequest>) -> impl IntoResponse {
    let manager = match ClaudeConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to initialize config manager: {}", e)
                })),
            )
                .into_response();
        }
    };

    let server_config = McpServerConfig {
        command: Some(req.command.clone()),
        args: Some(req.args.clone()),
        env: req.env.clone(),
        server_type: None,
        url: None,
        disabled: req.disabled,
    };

    match manager.add_mcp_server(req.name.clone(), server_config) {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": "MCP server added successfully",
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to add MCP server: {}", e)
            })),
        )
            .into_response(),
    }
}

/// PATCH /api/mcp/servers/:name - Update an existing MCP server
pub async fn update_mcp_server(
    Path(name): Path<String>,
    Json(req): Json<McpServerRequest>,
) -> impl IntoResponse {
    let manager = match ClaudeConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to initialize config manager: {}", e)
                })),
            )
                .into_response();
        }
    };

    let server_config = McpServerConfig {
        command: Some(req.command.clone()),
        args: Some(req.args.clone()),
        env: req.env.clone(),
        server_type: None,
        url: None,
        disabled: req.disabled,
    };

    match manager.update_mcp_server(&name, server_config) {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": "MCP server updated successfully",
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to update MCP server: {}", e)
            })),
        )
            .into_response(),
    }
}

/// DELETE /api/mcp/servers/:name - Delete an MCP server
pub async fn delete_mcp_server(Path(name): Path<String>) -> impl IntoResponse {
    let manager = match ClaudeConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to initialize config manager: {}", e)
                })),
            )
                .into_response();
        }
    };

    match manager.delete_mcp_server(&name) {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": "MCP server deleted successfully",
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to delete MCP server: {}", e)
            })),
        )
            .into_response(),
    }
}

/// PATCH /api/mcp/servers/:name/toggle - Toggle MCP server enabled/disabled state
pub async fn toggle_mcp_server(Path(name): Path<String>) -> impl IntoResponse {
    let manager = match ClaudeConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to initialize config manager: {}", e)
                })),
            )
                .into_response();
        }
    };

    // Get current config to find the server
    let mut servers = match manager.get_mcp_servers() {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to get MCP servers: {}", e)
                })),
            )
                .into_response();
        }
    };

    if let Some(server) = servers.get_mut(&name) {
        // Toggle disabled state
        let new_state = !server.disabled.unwrap_or(false);
        server.disabled = Some(new_state);

        // Update server
        match manager.update_mcp_server(&name, server.clone()) {
            Ok(_) => (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "data": format!("MCP server {} {}", if new_state { "disabled" } else { "enabled" }, name),
                    "message": null
                })),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to update MCP server: {}", e)
                })),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("MCP server '{}' not found", name)
            })),
        )
            .into_response()
    }
}
