use crate::commands::wire::OpenJsonValueDto;

use crate::commands::claude_mcp_config::{
    add_claude_mcp_server_default, delete_claude_mcp_server_default, list_claude_mcp_default,
    parse_scope, update_claude_mcp_server_default,
};

#[ccr_tauri_command_macros::command]
pub async fn claude_list_mcp_servers() -> Result<OpenJsonValueDto, String> {
    tokio::task::spawn_blocking(|| {
        let list = list_claude_mcp_default()?;
        serde_json::to_value(list).map_err(|e| format!("Serialize Claude MCP list: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??
    .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn claude_add_mcp_server(
    name: String,
    config: OpenJsonValueDto,
    scope: Option<String>,
) -> Result<OpenJsonValueDto, String> {
    let config = config.into();
    tokio::task::spawn_blocking(move || {
        add_claude_mcp_server_default(name, config, parse_scope(scope.as_deref()))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??
    .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn claude_update_mcp_server(
    name: String,
    config: OpenJsonValueDto,
    scope: Option<String>,
) -> Result<OpenJsonValueDto, String> {
    let config = config.into();
    tokio::task::spawn_blocking(move || {
        update_claude_mcp_server_default(name, config, parse_scope(scope.as_deref()))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??
    .try_into()
}

#[ccr_tauri_command_macros::command]
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
