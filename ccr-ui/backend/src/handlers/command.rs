// Command Execution Handlers
// Execute CCR CLI commands as subprocesses

use axum::{extract::Path, response::IntoResponse, Json};

use crate::executor;
use crate::models::*;

/// POST /api/command/execute - Execute a CCR command
pub async fn execute_command(Json(req): Json<CommandRequest>) -> impl IntoResponse {
    // Validate command to prevent arbitrary command execution
    let allowed_commands = vec![
        "list", "current", "switch", "validate", "optimize", "history", "clean", "export",
        "import", "init", "version", "update",
    ];

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
    let commands = vec![
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
            examples: vec!["ccr history".to_string(), "ccr history --limit 10".to_string()],
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
            examples: vec!["ccr export".to_string(), "ccr export --no-secrets".to_string()],
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
    ];

    let response = CommandListResponse { commands };
    ApiResponse::success(response)
}

/// GET /api/command/:command/help - Get help for a specific command
pub async fn get_command_help(Path(command): Path<String>) -> impl IntoResponse {
    // Execute "ccr <command> --help"
    let args = vec![command.clone(), "--help".to_string()];

    match executor::execute_command(args).await {
        Ok(output) => {
            if output.success {
                ApiResponse::success(output.stdout)
            } else {
                ApiResponse::<String>::error(output.stderr)
            }
        }
        Err(e) => ApiResponse::<String>::error(e.to_string()),
    }
}
