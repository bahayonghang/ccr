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
    let start = std::env::current_dir().ok();
    let project_root = start.as_ref().map(|start| {
        let mut current = start.as_path();
        loop {
            if current.join(".git").exists() {
                return current.to_path_buf();
            }
            match current.parent() {
                Some(parent) => current = parent,
                None => return start.clone(),
            }
        }
    });

    let project_manager = project_root.as_ref().and_then(|root| {
        MarkdownManager::from_directory(root.join(".claude").join("commands")).ok()
    });
    let user_manager = MarkdownManager::from_home_subdir("commands").ok();

    let mut merged: std::collections::HashMap<String, SlashCommand> =
        std::collections::HashMap::new();

    for manager in [project_manager.as_ref(), user_manager.as_ref()]
        .into_iter()
        .flatten()
    {
        let files = match manager.list_files_with_folders() {
            Ok(f) => f,
            Err(_) => continue,
        };
        for (file_name, folder_path) in files {
            let full_name = if folder_path.is_empty() {
                file_name.clone()
            } else {
                format!("{}/{}", folder_path, file_name)
            };

            let description = match manager.read_file::<CommandFrontmatter>(&full_name) {
                Ok(file) => {
                    file.frontmatter.description.clone().unwrap_or_else(|| {
                        extract_description_from_content(&file.content, &file_name)
                    })
                }
                Err(_) => match manager.read_file_content(&full_name) {
                    Ok(content) => extract_description_from_content(&content, &file_name),
                    Err(e) => {
                        tracing::warn!("Failed to read slash command file {}: {}", full_name, e);
                        continue;
                    }
                },
            };

            let key = full_name.clone();
            merged.entry(key).or_insert(SlashCommand {
                name: file_name.clone(),
                description,
                command: String::new(),
                args: None,
                disabled: false,
                folder: folder_path,
            });
        }
    }

    let mut commands: Vec<SlashCommand> = merged.into_values().collect();
    commands.sort_by(|a, b| a.name.cmp(&b.name));

    if !commands.is_empty() {
        let folders: Vec<String> = commands
            .iter()
            .filter_map(|c| {
                if c.folder.is_empty() {
                    None
                } else {
                    Some(c.folder.clone())
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
            other: std::collections::HashMap::new(),
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
