use axum::{Json, extract::Path};
use serde_json::json;

use crate::api::handlers::response::ApiSuccess;
use crate::cache::GLOBAL_SETTINGS_CACHE;
use crate::core::error::{ApiError, ApiResult};
use crate::managers::markdown_manager::{CommandFrontmatter, MarkdownManager};
use crate::models::api::{SlashCommand, SlashCommandRequest};

/// Extract description from markdown content
fn extract_description_from_content(content: &str, name: &str) -> String {
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
pub async fn list_slash_commands() -> ApiResult<ApiSuccess<serde_json::Value>> {
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

        return Ok(ApiSuccess(json!({
            "commands": commands,
            "folders": folders
        })));
    }

    // Fallback to settings.json (使用全局缓存)
    let settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let commands: Vec<SlashCommand> = settings
        .slash_commands
        .into_iter()
        .map(|cmd| SlashCommand {
            name: cmd.name,
            description: cmd.description,
            command: cmd.command,
            args: cmd.args,
            disabled: cmd.disabled,
            folder: String::new(),
        })
        .collect();

    Ok(ApiSuccess(json!({
        "commands": commands,
        "folders": Vec::<String>::new()
    })))
}

/// POST /api/slash-commands - Add a new slash command
pub async fn add_slash_command(
    Json(req): Json<SlashCommandRequest>,
) -> ApiResult<ApiSuccess<&'static str>> {
    let mut settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let new_command = crate::managers::settings_manager::SlashCommand {
        name: req.name.clone(),
        description: req.description.clone(),
        command: req.command.clone(),
        args: req.args.clone(),
        disabled: req.disabled.unwrap_or(false),
        other: std::collections::HashMap::new(),
    };

    settings.slash_commands.push(new_command);

    GLOBAL_SETTINGS_CACHE
        .save_atomic(&settings)
        .map_err(|e| ApiError::internal(format!("Failed to save settings: {}", e)))?;

    Ok(ApiSuccess("Slash command added successfully"))
}

/// PATCH /api/slash-commands/:name - Update a slash command
pub async fn update_slash_command(
    Path(name): Path<String>,
    Json(req): Json<SlashCommandRequest>,
) -> ApiResult<ApiSuccess<&'static str>> {
    let mut settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let cmd = settings
        .slash_commands
        .iter_mut()
        .find(|c| c.name == name)
        .ok_or_else(|| ApiError::not_found("Slash command not found"))?;

    cmd.name = req.name;
    cmd.description = req.description;
    cmd.command = req.command;
    cmd.args = req.args;
    if let Some(disabled) = req.disabled {
        cmd.disabled = disabled;
    }

    GLOBAL_SETTINGS_CACHE
        .save_atomic(&settings)
        .map_err(|e| ApiError::internal(format!("Failed to save settings: {}", e)))?;

    Ok(ApiSuccess("Slash command updated successfully"))
}

/// DELETE /api/slash-commands/:name - Delete a slash command
pub async fn delete_slash_command(Path(name): Path<String>) -> ApiResult<ApiSuccess<&'static str>> {
    let mut settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let original_len = settings.slash_commands.len();
    settings.slash_commands.retain(|c| c.name != name);

    if settings.slash_commands.len() >= original_len {
        return Err(ApiError::not_found("Slash command not found"));
    }

    GLOBAL_SETTINGS_CACHE
        .save_atomic(&settings)
        .map_err(|e| ApiError::internal(format!("Failed to save settings: {}", e)))?;

    Ok(ApiSuccess("Slash command deleted successfully"))
}

/// PATCH /api/slash-commands/:name/toggle - Toggle slash command enabled/disabled state
pub async fn toggle_slash_command(Path(name): Path<String>) -> ApiResult<ApiSuccess<String>> {
    let mut settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let cmd = settings
        .slash_commands
        .iter_mut()
        .find(|c| c.name == name)
        .ok_or_else(|| ApiError::not_found("Slash command not found"))?;

    cmd.disabled = !cmd.disabled;
    let new_state = if cmd.disabled { "disabled" } else { "enabled" };

    GLOBAL_SETTINGS_CACHE
        .save_atomic(&settings)
        .map_err(|e| ApiError::internal(format!("Failed to save settings: {}", e)))?;

    Ok(ApiSuccess(format!(
        "Slash command toggled to {}",
        new_state
    )))
}
