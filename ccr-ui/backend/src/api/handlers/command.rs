// Command Execution Handlers
// Executes CCR CLI commands with various parameters

use crate::core::executor;
use crate::models::api::*;
use axum::{
    Json,
    extract::Path,
    response::{
        IntoResponse,
        sse::{Event, Sse},
    },
};
use futures::{Stream, stream::StreamExt};
use std::convert::Infallible;

/// Helper function to get all available commands
fn get_available_commands() -> Vec<CommandInfo> {
    vec![
        // ===== 配置管理命令 =====
        CommandInfo {
            name: "init".to_string(),
            description: "Initialize CCR configuration".to_string(),
            usage: "ccr init".to_string(),
            examples: vec!["ccr init".to_string()],
        },
        CommandInfo {
            name: "add".to_string(),
            description: "Add a new configuration".to_string(),
            usage: "ccr add".to_string(),
            examples: vec!["ccr add".to_string()],
        },
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
            examples: vec!["ccr switch anthropic".to_string()],
        },
        CommandInfo {
            name: "enable".to_string(),
            description: "Enable a configuration".to_string(),
            usage: "ccr enable <config-name>".to_string(),
            examples: vec!["ccr enable my-config".to_string()],
        },
        CommandInfo {
            name: "disable".to_string(),
            description: "Disable a configuration".to_string(),
            usage: "ccr disable <config-name>".to_string(),
            examples: vec!["ccr disable my-config".to_string()],
        },
        CommandInfo {
            name: "validate".to_string(),
            description: "Validate all configurations".to_string(),
            usage: "ccr validate".to_string(),
            examples: vec!["ccr validate".to_string()],
        },
        CommandInfo {
            name: "history".to_string(),
            description: "Show operation history".to_string(),
            usage: "ccr history [--limit <n>]".to_string(),
            examples: vec!["ccr history -l 50".to_string()],
        },
        CommandInfo {
            name: "optimize".to_string(),
            description: "Optimize and sort configurations".to_string(),
            usage: "ccr optimize".to_string(),
            examples: vec!["ccr optimize".to_string()],
        },
        // ===== 平台管理命令 =====
        CommandInfo {
            name: "platform".to_string(),
            description: "Platform management commands".to_string(),
            usage: "ccr platform <subcommand>".to_string(),
            examples: vec![
                "ccr platform list".to_string(),
                "ccr platform switch claude".to_string(),
                "ccr platform current".to_string(),
                "ccr platform info claude".to_string(),
                "ccr platform init codex".to_string(),
            ],
        },
        // ===== 导入导出命令 =====
        CommandInfo {
            name: "export".to_string(),
            description: "Export configurations".to_string(),
            usage: "ccr export [--no-secrets]".to_string(),
            examples: vec!["ccr export --no-secrets".to_string()],
        },
        CommandInfo {
            name: "import".to_string(),
            description: "Import configurations".to_string(),
            usage: "ccr import <file> [--merge]".to_string(),
            examples: vec!["ccr import config.toml --merge".to_string()],
        },
        CommandInfo {
            name: "clean".to_string(),
            description: "Clean old backup files".to_string(),
            usage: "ccr clean [--days <n>] [--dry-run]".to_string(),
            examples: vec!["ccr clean --days 30 --dry-run".to_string()],
        },
        // ===== 迁移命令 =====
        CommandInfo {
            name: "migrate".to_string(),
            description: "Migrate configuration from Legacy to Unified mode".to_string(),
            usage: "ccr migrate [--check] [--platform <name>]".to_string(),
            examples: vec![
                "ccr migrate --check".to_string(),
                "ccr migrate".to_string(),
                "ccr migrate --platform claude".to_string(),
            ],
        },
        // ===== 临时凭证命令 =====
        CommandInfo {
            name: "temp-token".to_string(),
            description: "Temporary credential management".to_string(),
            usage: "ccr temp-token <subcommand>".to_string(),
            examples: vec![
                "ccr temp-token set sk-xxx".to_string(),
                "ccr temp-token show".to_string(),
                "ccr temp-token clear".to_string(),
            ],
        },
        // ===== 技能管理命令 =====
        CommandInfo {
            name: "skills".to_string(),
            description: "Skills management".to_string(),
            usage: "ccr skills <subcommand>".to_string(),
            examples: vec![
                "ccr skills list".to_string(),
                "ccr skills scan ~/skills".to_string(),
                "ccr skills install ~/skills/my-skill".to_string(),
            ],
        },
        // ===== 提示词管理命令 =====
        CommandInfo {
            name: "prompts".to_string(),
            description: "Prompts management".to_string(),
            usage: "ccr prompts <subcommand>".to_string(),
            examples: vec![
                "ccr prompts list".to_string(),
                "ccr prompts add".to_string(),
                "ccr prompts apply my-preset".to_string(),
            ],
        },
        // ===== 统计命令 =====
        CommandInfo {
            name: "stats".to_string(),
            description: "Usage statistics".to_string(),
            usage: "ccr stats cost [--today|--by-model|--this-month]".to_string(),
            examples: vec![
                "ccr stats cost --today".to_string(),
                "ccr stats cost --by-model".to_string(),
            ],
        },
        // ===== 系统命令 =====
        CommandInfo {
            name: "version".to_string(),
            description: "Show CCR version".to_string(),
            usage: "ccr version".to_string(),
            examples: vec!["ccr version".to_string()],
        },
        CommandInfo {
            name: "update".to_string(),
            description: "Update CCR to the latest version".to_string(),
            usage: "ccr update [--check] [--branch <name>]".to_string(),
            examples: vec!["ccr update --check".to_string(), "ccr update".to_string()],
        },
        CommandInfo {
            name: "check".to_string(),
            description: "Check for conflicts between platforms".to_string(),
            usage: "ccr check conflicts".to_string(),
            examples: vec!["ccr check conflicts".to_string()],
        },
        // ===== 外部 CLI 命令 =====
        CommandInfo {
            name: "claude".to_string(),
            description: "Execute Claude Code CLI commands".to_string(),
            usage: "claude <args>".to_string(),
            examples: vec!["claude --version".to_string()],
        },
        CommandInfo {
            name: "qwen".to_string(),
            description: "Execute Qwen CLI commands".to_string(),
            usage: "qwen <args>".to_string(),
            examples: vec!["qwen --help".to_string()],
        },
        CommandInfo {
            name: "gemini".to_string(),
            description: "Execute Gemini CLI commands".to_string(),
            usage: "gemini <args>".to_string(),
            examples: vec!["gemini --version".to_string()],
        },
        CommandInfo {
            name: "iflow".to_string(),
            description: "Execute iFlow CLI commands".to_string(),
            usage: "iflow <args>".to_string(),
            examples: vec!["iflow --help".to_string()],
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

    // Determine binary and args based on command type
    let (binary, args) = match req.command.as_str() {
        "claude" | "qwen" | "gemini" | "iflow" => (req.command.clone(), req.args.clone()),
        _ => {
            // Default to CCR for other commands
            let mut args = vec![req.command.clone()];
            args.extend(req.args.clone());
            ("ccr".to_string(), args)
        }
    };

    // Execute command
    match executor::execute_binary(&binary, args).await {
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

/// POST /api/command/execute-stream - Execute a command with streaming output (SSE)
pub async fn execute_command_stream(
    Json(req): Json<CommandRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // Validate command
    let commands = get_available_commands();
    let allowed_commands: Vec<&str> = commands.iter().map(|c| c.name.as_str()).collect();

    // Determine binary and args based on command type
    let (binary, args) = if !allowed_commands.contains(&req.command.as_str()) {
        // Invalid command - will return error in stream
        (String::new(), vec![])
    } else {
        match req.command.as_str() {
            "claude" | "qwen" | "gemini" | "iflow" => (req.command.clone(), req.args.clone()),
            _ => {
                // Default to CCR for other commands
                let mut args = vec![req.command.clone()];
                args.extend(req.args.clone());
                ("ccr".to_string(), args)
            }
        }
    };

    // Create stream
    let stream = if binary.is_empty() {
        // Return error stream
        let error_msg = format!("Command '{}' is not allowed", req.command);
        futures::stream::once(async move { executor::StreamChunk::Error { message: error_msg } })
            .boxed()
    } else {
        // Execute command and stream output
        executor::execute_binary_stream(binary, args).boxed()
    };

    // Convert StreamChunk to SSE Event
    let sse_stream = stream.map(|chunk| {
        Ok(Event::default()
            .json_data(chunk)
            .unwrap_or_else(|_| Event::default().data("error serializing chunk")))
    });

    Sse::new(sse_stream)
}
