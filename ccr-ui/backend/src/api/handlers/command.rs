// Command Execution Handlers
// Executes CCR CLI commands with various parameters

use crate::core::error::{ApiError, ApiResult};
use crate::core::executor;
use crate::models::api::*;
use axum::{Json, extract::Path, response::IntoResponse};

/// Helper function to get all available commands
fn get_available_commands() -> Vec<CommandInfo> {
    vec![
        CommandInfo {
            name: "list".to_string(),
            description: "List all available configurations".to_string(),
            usage: "ccr list".to_string(),
            examples: vec!["ccr list".to_string()],
        },
        CommandInfo {
            name: "current".to_string(),
            description: "Show current configuration".to_string(),
            usage: "ccr current".to_string(),
            examples: vec!["ccr current".to_string()],
        },
        CommandInfo {
            name: "switch".to_string(),
            description: "Switch to a different configuration".to_string(),
            usage: "ccr switch <config-name>".to_string(),
            examples: vec![
                "ccr switch anthropic".to_string(),
                "ccr switch openai".to_string(),
            ],
        },
        CommandInfo {
            name: "validate".to_string(),
            description: "Validate all configurations".to_string(),
            usage: "ccr validate".to_string(),
            examples: vec!["ccr validate".to_string()],
        },
        CommandInfo {
            name: "optimize".to_string(),
            description: "Optimize and sort configurations".to_string(),
            usage: "ccr optimize".to_string(),
            examples: vec!["ccr optimize".to_string()],
        },
        CommandInfo {
            name: "history".to_string(),
            description: "Show operation history".to_string(),
            usage: "ccr history [--limit <n>]".to_string(),
            examples: vec![
                "ccr history".to_string(),
                "ccr history --limit 10".to_string(),
            ],
        },
        CommandInfo {
            name: "clean".to_string(),
            description: "Clean old backup files".to_string(),
            usage: "ccr clean [--days <n>] [--dry-run]".to_string(),
            examples: vec![
                "ccr clean --days 7".to_string(),
                "ccr clean --dry-run".to_string(),
            ],
        },
        CommandInfo {
            name: "export".to_string(),
            description: "Export configurations".to_string(),
            usage: "ccr export [--no-secrets]".to_string(),
            examples: vec![
                "ccr export".to_string(),
                "ccr export --no-secrets".to_string(),
            ],
        },
        CommandInfo {
            name: "import".to_string(),
            description: "Import configurations".to_string(),
            usage: "ccr import <file> [--mode <mode>]".to_string(),
            examples: vec![
                "ccr import config.toml".to_string(),
                "ccr import config.toml --mode merge".to_string(),
            ],
        },
        CommandInfo {
            name: "init".to_string(),
            description: "Initialize CCR configuration".to_string(),
            usage: "ccr init".to_string(),
            examples: vec!["ccr init".to_string()],
        },
        CommandInfo {
            name: "version".to_string(),
            description: "Show CCR version".to_string(),
            usage: "ccr version".to_string(),
            examples: vec!["ccr version".to_string()],
        },
        CommandInfo {
            name: "update".to_string(),
            description: "Update CCR to the latest version".to_string(),
            usage: "ccr update".to_string(),
            examples: vec!["ccr update".to_string()],
        },
    ]
}

/// POST /api/command/execute - Execute a CCR command
pub async fn execute_command(Json(req): Json<CommandRequest>) -> impl IntoResponse {
    // Validate command to prevent arbitrary command execution
    let commands = get_available_commands();
    let allowed_commands: Vec<&str> = commands.iter().map(|c| c.name.as_str()).collect();

    if !allowed_commands.contains(&req.command.as_str()) {
        return ApiResponse::<CommandResponse>::error(format!(
            "Command '{}' is not allowed",
            req.command
        ));
    }

    // Build command arguments
    let mut args = vec![req.command.clone()];
    args.extend(req.args.clone());

    // Execute command
    match executor::execute_command(args).await {
        Ok(output) => {
            let response = CommandResponse {
                success: output.success,
                output: output.stdout,
                error: output.stderr,
                exit_code: output.exit_code,
                duration_ms: output.duration_ms,
            };
            ApiResponse::success(response)
        }
        Err(e) => ApiResponse::<CommandResponse>::error(e.to_string()),
    }
}

/// GET /api/command/list - List all available commands
pub async fn list_commands() -> impl IntoResponse {
    let commands = get_available_commands();
    let response = CommandListResponse { commands };
    ApiResponse::success(response)
}

/// GET /api/command/help/:command - Get help for a specific command
pub async fn get_command_help(Path(command): Path<String>) -> impl IntoResponse {
    let result = executor::execute_command(vec![command.clone(), "--help".to_string()]).await;

    match result {
        Ok(output) if output.success => ApiResponse::success(output.stdout),
        Ok(output) => ApiResponse::<String>::error(output.stderr),
        Err(e) => ApiResponse::<String>::error(e.to_string()),
    }
}

/// POST /api/command/execute/stream - Execute a command with streaming output (SSE)
/// Note: Currently simplified - real implementation would use CommandService
pub async fn execute_command_stream(
    Json(req): Json<CommandRequest>,
) -> ApiResult<Json<&'static str>> {
    // Validate command
    let commands = get_available_commands();
    let allowed_commands: Vec<&str> = commands.iter().map(|c| c.name.as_str()).collect();

    // Additional commands allowed for streaming/dev
    let extra_allowed = ["build", "test"];

    if !allowed_commands.contains(&req.command.as_str())
        && !extra_allowed.contains(&req.command.as_str())
    {
        return Err(ApiError::bad_request(format!(
            "Command '{}' is not allowed",
            req.command
        )));
    }

    // TODO: Implement actual streaming using CommandService
    // Current limitation: lifecycle issues with SSE streams need resolution
    // For now, return success message
    Ok(Json("Streaming endpoint - implementation pending"))
}
