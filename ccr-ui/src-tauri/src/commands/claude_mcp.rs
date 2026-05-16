use serde_json::Value;

use crate::commands::claude_mcp_config::{
    add_claude_mcp_server_default, delete_claude_mcp_server_default, list_claude_mcp_default,
    parse_scope, update_claude_mcp_server_default,
};

#[tauri::command]
pub async fn claude_list_mcp_servers() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let list = list_claude_mcp_default()?;
        serde_json::to_value(list).map_err(|e| format!("Serialize Claude MCP list: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_add_mcp_server(
    name: String,
    config: Value,
    scope: Option<String>,
) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        add_claude_mcp_server_default(name, config, parse_scope(scope.as_deref()))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_update_mcp_server(
    name: String,
    config: Value,
    scope: Option<String>,
) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        update_claude_mcp_server_default(name, config, parse_scope(scope.as_deref()))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_delete_mcp_server(
    name: String,
    scope: Option<String>,
) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        delete_claude_mcp_server_default(name, parse_scope(scope.as_deref()))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}
