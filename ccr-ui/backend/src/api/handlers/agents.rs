use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::managers::markdown_manager::{AgentFrontmatter, MarkdownManager};
use crate::managers::settings_manager::SettingsManager;
use crate::models::api::{Agent, AgentRequest};

/// GET /api/agents - List all agents
pub async fn list_agents() -> impl IntoResponse {
    // Try markdown files first (primary source with richer data)
    if let Ok(manager) = MarkdownManager::from_home_subdir("agents")
        && let Ok(files) = manager.list_files_with_folders()
    {
        let mut agents = Vec::new();

        for (file_name, folder_path) in files {
            // Build full path for reading (e.g., "kfc/spec-design" for subfolder files)
            let full_name = if folder_path.is_empty() {
                file_name.clone()
            } else {
                format!("{}/{}", folder_path, file_name)
            };

            match manager.read_file::<AgentFrontmatter>(&full_name) {
                Ok(file) => {
                    let tools = file
                        .frontmatter
                        .tools
                        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                        .unwrap_or_default();

                    agents.push(Agent {
                        name: file_name.clone(),
                        model: String::new(),
                        tools,
                        system_prompt: Some(file.content),
                        disabled: false,
                        folder: folder_path,
                    });
                }
                Err(e) => {
                    tracing::warn!("Failed to read agent {}: {}", full_name, e);
                }
            }
        }

        if !agents.is_empty() {
            // Collect unique folders
            let folders: Vec<String> = agents
                .iter()
                .filter_map(|a| {
                    if !a.folder.is_empty() {
                        Some(a.folder.clone())
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
                        "agents": agents,
                        "folders": folders
                    },
                    "message": null
                })),
            )
                .into_response();
        }
    }

    // Fallback to settings.json
    let settings_result = SettingsManager::default().and_then(|manager| manager.load());

    if let Ok(settings) = settings_result {
        let agents: Vec<Agent> = settings
            .agents
            .into_iter()
            .map(|agent| Agent {
                name: agent.name,
                model: agent.model,
                tools: agent.tools,
                system_prompt: agent.system_prompt,
                disabled: agent.disabled,
                folder: String::new(), // Settings don't have folder concept
            })
            .collect();

        return (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": {
                    "agents": agents,
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
            "message": "Failed to load agents from both markdown files and settings.json"
        })),
    )
        .into_response()
}

/// POST /api/agents - Add a new agent
pub async fn add_agent(Json(req): Json<AgentRequest>) -> impl IntoResponse {
    // Try settings.json first
    if let Ok(settings_manager) = SettingsManager::default()
        && let Ok(mut settings) = settings_manager.load()
    {
        let new_agent = crate::managers::settings_manager::Agent {
            name: req.name.clone(),
            model: req.model.clone(),
            tools: req.tools.unwrap_or_default(),
            system_prompt: req.system_prompt.clone(),
            disabled: req.disabled.unwrap_or(false),
        };

        settings.agents.push(new_agent);

        if settings_manager.save(&settings).is_ok() {
            return (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "data": "Agent added successfully",
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
            "message": "Agent addition via markdown files not implemented"
        })),
    )
        .into_response()
}

/// PATCH /api/agents/:name - Update an agent
pub async fn update_agent(
    Path(name): Path<String>,
    Json(req): Json<AgentRequest>,
) -> impl IntoResponse {
    // Try settings.json first
    if let Ok(settings_manager) = SettingsManager::default()
        && let Ok(mut settings) = settings_manager.load()
        && let Some(agent) = settings.agents.iter_mut().find(|a| a.name == name)
    {
        agent.name = req.name;
        agent.model = req.model;
        if let Some(tools) = req.tools {
            agent.tools = tools;
        }
        agent.system_prompt = req.system_prompt;
        if let Some(disabled) = req.disabled {
            agent.disabled = disabled;
        }

        if settings_manager.save(&settings).is_ok() {
            return (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "data": "Agent updated successfully",
                    "message": null
                })),
            )
                .into_response();
        }
    } else if let Ok(settings_manager) = SettingsManager::default()
        && let Ok(settings) = settings_manager.load()
        && !settings.agents.iter().any(|a| a.name == name)
    {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": "Agent not found"
            })),
        )
            .into_response();
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "success": false,
            "data": null,
            "message": "Failed to update agent"
        })),
    )
        .into_response()
}

/// DELETE /api/agents/:name - Delete an agent
pub async fn delete_agent(Path(name): Path<String>) -> impl IntoResponse {
    // Try settings.json first
    if let Ok(settings_manager) = SettingsManager::default()
        && let Ok(mut settings) = settings_manager.load()
    {
        let original_len = settings.agents.len();
        settings.agents.retain(|a| a.name != name);

        if settings.agents.len() < original_len {
            if settings_manager.save(&settings).is_ok() {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "data": "Agent deleted successfully",
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
                    "message": "Agent not found"
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
            "message": "Failed to delete agent"
        })),
    )
        .into_response()
}

/// PATCH /api/agents/:name/toggle - Toggle agent enabled/disabled state
pub async fn toggle_agent(Path(name): Path<String>) -> impl IntoResponse {
    // Try settings.json first
    if let Ok(settings_manager) = SettingsManager::default()
        && let Ok(mut settings) = settings_manager.load()
    {
        if let Some(agent) = settings.agents.iter_mut().find(|a| a.name == name) {
            agent.disabled = !agent.disabled;
            let new_state = if agent.disabled {
                "disabled"
            } else {
                "enabled"
            };

            if settings_manager.save(&settings).is_ok() {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "data": format!("Agent toggled to {}", new_state),
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
                    "message": "Agent not found"
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
            "message": "Failed to toggle agent"
        })),
    )
        .into_response()
}
