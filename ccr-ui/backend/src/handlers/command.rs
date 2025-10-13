// Command Execution Handlers
// Execute arbitrary CCR commands via subprocess

use actix_web::{get, post, web, HttpResponse, Responder};

use crate::executor;
use crate::models::*;

/// POST /api/command/execute - Execute a CCR command
#[post("/api/command/execute")]
pub async fn execute_command(req: web::Json<CommandRequest>) -> impl Responder {
    log::info!(
        "Executing command: {} with args: {:?}",
        req.command,
        req.args
    );

    // Validate command is one of the allowed CCR commands
    let allowed_commands = [
        "init", "list", "current", "switch", "validate", "optimize", "history", "clean",
        "export", "import", "update", "version",
    ];

    if !allowed_commands.contains(&req.command.as_str()) {
        return HttpResponse::BadRequest().json(ApiResponse::<()>::error(format!(
            "Invalid command: {}. Allowed commands: {:?}",
            req.command, allowed_commands
        )));
    }

    // Build full args vector
    let mut args = vec![req.command.clone()];
    args.extend(req.args.clone());

    // Execute the command
    let result = executor::execute_command(args).await;

    match result {
        Ok(output) => {
            let response = CommandResponse {
                success: output.success,
                output: output.stdout,
                error: output.stderr,
                exit_code: output.exit_code,
                duration_ms: output.duration_ms,
            };
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Err(e) => {
            let response = CommandResponse {
                success: false,
                output: String::new(),
                error: e.to_string(),
                exit_code: -1,
                duration_ms: 0,
            };
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
    }
}

/// GET /api/command/list - List all available CCR commands
#[get("/api/command/list")]
pub async fn list_commands() -> impl Responder {
    let commands = vec![
        CommandInfo {
            name: "init".to_string(),
            description: "Initialize configuration file with examples".to_string(),
            usage: "ccr init [--force]".to_string(),
            examples: vec![
                "ccr init".to_string(),
                "ccr init --force".to_string(),
            ],
        },
        CommandInfo {
            name: "list".to_string(),
            description: "List all available configurations".to_string(),
            usage: "ccr list".to_string(),
            examples: vec!["ccr list".to_string()],
        },
        CommandInfo {
            name: "current".to_string(),
            description: "Show current configuration status".to_string(),
            usage: "ccr current".to_string(),
            examples: vec!["ccr current".to_string()],
        },
        CommandInfo {
            name: "switch".to_string(),
            description: "Switch to a different configuration".to_string(),
            usage: "ccr switch <config-name>".to_string(),
            examples: vec![
                "ccr switch anthropic".to_string(),
                "ccr switch anyrouter".to_string(),
            ],
        },
        CommandInfo {
            name: "validate".to_string(),
            description: "Validate all configurations and settings".to_string(),
            usage: "ccr validate".to_string(),
            examples: vec!["ccr validate".to_string()],
        },
        CommandInfo {
            name: "optimize".to_string(),
            description: "Sort configuration sections alphabetically".to_string(),
            usage: "ccr optimize".to_string(),
            examples: vec!["ccr optimize".to_string()],
        },
        CommandInfo {
            name: "history".to_string(),
            description: "Show operation history".to_string(),
            usage: "ccr history [-l N] [-t TYPE]".to_string(),
            examples: vec![
                "ccr history".to_string(),
                "ccr history -l 10".to_string(),
                "ccr history -t switch".to_string(),
            ],
        },
        CommandInfo {
            name: "clean".to_string(),
            description: "Clean old backup files".to_string(),
            usage: "ccr clean [--days N] [--dry-run]".to_string(),
            examples: vec![
                "ccr clean".to_string(),
                "ccr clean --days 7 --dry-run".to_string(),
            ],
        },
        CommandInfo {
            name: "export".to_string(),
            description: "Export configurations to file".to_string(),
            usage: "ccr export [-o FILE] [--no-secrets]".to_string(),
            examples: vec![
                "ccr export".to_string(),
                "ccr export -o backup.toml".to_string(),
                "ccr export --no-secrets".to_string(),
            ],
        },
        CommandInfo {
            name: "import".to_string(),
            description: "Import configurations from file".to_string(),
            usage: "ccr import FILE [--merge]".to_string(),
            examples: vec![
                "ccr import backup.toml".to_string(),
                "ccr import backup.toml --merge".to_string(),
            ],
        },
        CommandInfo {
            name: "update".to_string(),
            description: "Update CCR to latest version".to_string(),
            usage: "ccr update [--check]".to_string(),
            examples: vec![
                "ccr update".to_string(),
                "ccr update --check".to_string(),
            ],
        },
        CommandInfo {
            name: "version".to_string(),
            description: "Show version information".to_string(),
            usage: "ccr version".to_string(),
            examples: vec!["ccr version".to_string()],
        },
    ];

    let response = CommandListResponse { commands };
    HttpResponse::Ok().json(ApiResponse::success(response))
}

/// GET /api/command/help/{command} - Get help for a specific command
#[get("/api/command/help/{command}")]
pub async fn get_command_help(command: web::Path<String>) -> impl Responder {
    let result = executor::execute_command(vec![command.to_string(), "--help".to_string()]).await;

    match result {
        Ok(output) => {
            let help_text = if !output.stdout.is_empty() {
                output.stdout
            } else {
                output.stderr
            };
            HttpResponse::Ok().json(ApiResponse::success(help_text))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string()))
        }
    }
}

