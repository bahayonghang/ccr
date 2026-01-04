use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::cache::GLOBAL_SETTINGS_CACHE;
use crate::managers::markdown_manager::{CommandFrontmatter, MarkdownManager};
use crate::models::api::{SlashCommand, SlashCommandRequest};

/// Extract description from markdown content
fn extract_description_from_content(content: &str, name: &str) -> String {
    // Try to extract first paragraph or heading
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if !line.is_empty() {
            return line.to_string();
        }
    }
    format!("Slash command: {}", name)
}

/// GET /api/slash-commands - List all slash commands
pub async fn list_slash_commands() -> impl IntoResponse {
    // Try markdown files first (primary source with richer data)
    if let Ok(manager) = MarkdownManager::from_home_subdir("commands")
        && let Ok(files) = manager.list_files_with_folders()
    {
        let mut commands = Vec::new();

        for (file_name, folder_path) in files {
            // Build full path for reading (e.g., "subfolder/command" for subfolder files)
            let full_name = if folder_path.is_empty() {
                file_name.clone()
            } else {
                format!("{}/{}", folder_path, file_name)
            };

            // Try to read with frontmatter first
            let description =
                match manager.read_file::<CommandFrontmatter>(&full_name) {
                    Ok(file) => file.frontmatter.description.clone().unwrap_or_else(|| {
                        extract_description_from_content(&file.content, &file_name)
                    }),
                    Err(_) => {
                        // If frontmatter parsing fails, try to read as plain markdown
                        // Support both Unix (HOME) and Windows (USERPROFILE)
                        let path = std::env::var("HOME")
                            .or_else(|_| std::env::var("USERPROFILE"))
                            .ok()
                            .map(|home| {
                                std::path::Path::new(&home)
                                    .join(".claude")
                                    .join("commands")
                                    .join(format!("{}.md", full_name))
                            });

                        if let Some(path) = path {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                extract_description_from_content(&content, &file_name)
                            } else {
                                tracing::warn!("Failed to read slash command file: {}", full_name);
                                continue;
                            }
                        } else {
                            tracing::warn!("Failed to get HOME directory");
                            continue;
                        }
                    }
                };

            commands.push(SlashCommand {
                name: file_name.clone(),
                description,
                command: String::new(),
                args: None,
                disabled: false,
                folder: folder_path,
            });
        }

        if !commands.is_empty() {
            // Collect unique folders
            let folders: Vec<String> = commands
                .iter()
                .filter_map(|c| {
                    if !c.folder.is_empty() {
                        Some(c.folder.clone())
                    } else {
                        None
                    }
                })
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            return (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "data": {
                        "commands": commands,
                        "folders": folders
                    },
                    "message": null
                })),
            )
                .into_response();
        }
    }

    // Fallback to settings.json (使用全局缓存)
    let settings_result = GLOBAL_SETTINGS_CACHE.load();

    if let Ok(settings) = settings_result {
        let commands: Vec<SlashCommand> = settings
            .slash_commands
            .into_iter()
            .map(|cmd| SlashCommand {
                name: cmd.name,
                description: cmd.description,
                command: cmd.command,
                args: cmd.args,
                disabled: cmd.disabled,
                folder: String::new(), // Settings don't have folder concept
            })
            .collect();

        return (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": {
                    "commands": commands,
                    "folders": Vec::<String>::new()
                },
                "message": null
            })),
        )
            .into_response();
    }

    // Both sources failed
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "success": false,
            "data": null,
            "message": "Failed to load slash commands from both markdown files and settings.json"
        })),
    )
        .into_response()
}

/// POST /api/slash-commands - Add a new slash command
pub async fn add_slash_command(Json(req): Json<SlashCommandRequest>) -> impl IntoResponse {
    // Try settings.json first (使用全局缓存)
    if let Ok(mut settings) = GLOBAL_SETTINGS_CACHE.load() {
        let new_command = crate::managers::settings_manager::SlashCommand {
            name: req.name.clone(),
            description: req.description.clone(),
            command: req.command.clone(),
            args: req.args.clone(),
            disabled: req.disabled.unwrap_or(false),
        };

        settings.slash_commands.push(new_command);

        if GLOBAL_SETTINGS_CACHE.save_atomic(&settings).is_ok() {
            return (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "data": "Slash command added successfully",
                    "message": null
                })),
            )
                .into_response();
        }
    }

    // Fallback to markdown files
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "success": false,
            "data": null,
            "message": "Slash command addition via markdown files not implemented"
        })),
    )
        .into_response()
}

/// PATCH /api/slash-commands/:name - Update a slash command
pub async fn update_slash_command(
    Path(name): Path<String>,
    Json(req): Json<SlashCommandRequest>,
) -> impl IntoResponse {
    // Try settings.json first (使用全局缓存)
    if let Ok(mut settings) = GLOBAL_SETTINGS_CACHE.load() {
        if let Some(cmd) = settings.slash_commands.iter_mut().find(|c| c.name == name) {
            cmd.name = req.name;
            cmd.description = req.description;
            cmd.command = req.command;
            cmd.args = req.args;
            if let Some(disabled) = req.disabled {
                cmd.disabled = disabled;
            }

            if GLOBAL_SETTINGS_CACHE.save_atomic(&settings).is_ok() {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "data": "Slash command updated successfully",
                        "message": null
                    })),
                )
                    .into_response();
            }
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": "Slash command not found"
                })),
            )
                .into_response();
        }
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "success": false,
            "data": null,
            "message": "Failed to update slash command"
        })),
    )
        .into_response()
}

/// DELETE /api/slash-commands/:name - Delete a slash command
pub async fn delete_slash_command(Path(name): Path<String>) -> impl IntoResponse {
    // Try settings.json first (使用全局缓存)
    if let Ok(mut settings) = GLOBAL_SETTINGS_CACHE.load() {
        let original_len = settings.slash_commands.len();
        settings.slash_commands.retain(|c| c.name != name);

        if settings.slash_commands.len() < original_len {
            if GLOBAL_SETTINGS_CACHE.save_atomic(&settings).is_ok() {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "data": "Slash command deleted successfully",
                        "message": null
                    })),
                )
                    .into_response();
            }
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": "Slash command not found"
                })),
            )
                .into_response();
        }
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "success": false,
            "data": null,
            "message": "Failed to delete slash command"
        })),
    )
        .into_response()
}

/// PATCH /api/slash-commands/:name/toggle - Toggle slash command enabled/disabled state
pub async fn toggle_slash_command(Path(name): Path<String>) -> impl IntoResponse {
    // Try settings.json first (使用全局缓存)
    if let Ok(mut settings) = GLOBAL_SETTINGS_CACHE.load() {
        if let Some(cmd) = settings.slash_commands.iter_mut().find(|c| c.name == name) {
            cmd.disabled = !cmd.disabled;
            let new_state = if cmd.disabled { "disabled" } else { "enabled" };

            if GLOBAL_SETTINGS_CACHE.save_atomic(&settings).is_ok() {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "data": format!("Slash command toggled to {}", new_state),
                        "message": null
                    })),
                )
                    .into_response();
            }
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": "Slash command not found"
                })),
            )
                .into_response();
        }
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "success": false,
            "data": null,
            "message": "Failed to toggle slash command"
        })),
    )
        .into_response()
}
