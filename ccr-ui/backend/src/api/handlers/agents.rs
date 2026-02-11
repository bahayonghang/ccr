use axum::{Json, extract::Path};
use serde_json::json;

use crate::api::handlers::response::ApiSuccess;
use crate::cache::GLOBAL_SETTINGS_CACHE;
use crate::core::error::{ApiError, ApiResult};
use crate::managers::markdown_manager::{AgentFrontmatter, MarkdownManager};
use crate::models::api::{Agent, AgentRequest};

fn find_project_root() -> Option<std::path::PathBuf> {
    let start = std::env::current_dir().ok()?;
    let mut current = start.as_path();
    loop {
        if current.join(".git").exists() {
            return Some(current.to_path_buf());
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return Some(start),
        }
    }
}

fn load_agents_from_manager(manager: &MarkdownManager) -> Vec<Agent> {
    let files = match manager.list_files_with_folders() {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let mut agents = Vec::new();
    for (file_name, folder_path) in files {
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
    agents
}

/// GET /api/agents - List all agents
pub async fn list_agents() -> ApiResult<ApiSuccess<serde_json::Value>> {
    let project_root = find_project_root();
    let project_manager = project_root
        .as_ref()
        .and_then(|root| MarkdownManager::from_directory(root.join(".claude").join("agents")).ok());
    let user_manager = MarkdownManager::from_home_subdir("agents").ok();

    let mut merged: std::collections::HashMap<String, Agent> = std::collections::HashMap::new();

    if let Some(m) = project_manager.as_ref() {
        for agent in load_agents_from_manager(m) {
            let key = if agent.folder.is_empty() {
                agent.name.clone()
            } else {
                format!("{}/{}", agent.folder, agent.name)
            };
            merged.insert(key, agent);
        }
    }

    if let Some(m) = user_manager.as_ref() {
        for agent in load_agents_from_manager(m) {
            let key = if agent.folder.is_empty() {
                agent.name.clone()
            } else {
                format!("{}/{}", agent.folder, agent.name)
            };
            merged.entry(key).or_insert(agent);
        }
    }

    let mut agents: Vec<Agent> = merged.into_values().collect();
    agents.sort_by(|a, b| a.name.cmp(&b.name));

    if !agents.is_empty() {
        let folders: Vec<String> = agents
            .iter()
            .filter_map(|a| {
                if a.folder.is_empty() {
                    None
                } else {
                    Some(a.folder.clone())
                }
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        return Ok(ApiSuccess(json!({
            "agents": agents,
            "folders": folders
        })));
    }

    // Fallback to settings.json (使用全局缓存)
    let settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let agents: Vec<Agent> = settings
        .agents
        .into_iter()
        .map(|agent| Agent {
            name: agent.name,
            model: agent.model,
            tools: agent.tools,
            system_prompt: agent.system_prompt,
            disabled: agent.disabled,
            folder: String::new(),
        })
        .collect();

    Ok(ApiSuccess(json!({
        "agents": agents,
        "folders": Vec::<String>::new()
    })))
}

/// POST /api/agents - Add a new agent
pub async fn add_agent(Json(req): Json<AgentRequest>) -> ApiResult<ApiSuccess<&'static str>> {
    // Try settings.json first (使用全局缓存)
    let mut settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let new_agent = crate::managers::settings_manager::Agent {
        name: req.name.clone(),
        model: req.model.clone(),
        tools: req.tools.unwrap_or_default(),
        system_prompt: req.system_prompt.clone(),
        disabled: req.disabled.unwrap_or(false),
        other: std::collections::HashMap::new(),
    };

    settings.agents.push(new_agent);

    GLOBAL_SETTINGS_CACHE
        .save_atomic(&settings)
        .map_err(|e| ApiError::internal(format!("Failed to save settings: {}", e)))?;

    Ok(ApiSuccess("Agent added successfully"))
}

/// PATCH /api/agents/:name - Update an agent
pub async fn update_agent(
    Path(name): Path<String>,
    Json(req): Json<AgentRequest>,
) -> ApiResult<ApiSuccess<&'static str>> {
    let mut settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let agent = settings
        .agents
        .iter_mut()
        .find(|a| a.name == name)
        .ok_or_else(|| ApiError::not_found("Agent not found"))?;

    agent.name = req.name;
    agent.model = req.model;
    if let Some(tools) = req.tools {
        agent.tools = tools;
    }
    agent.system_prompt = req.system_prompt;
    if let Some(disabled) = req.disabled {
        agent.disabled = disabled;
    }

    GLOBAL_SETTINGS_CACHE
        .save_atomic(&settings)
        .map_err(|e| ApiError::internal(format!("Failed to save settings: {}", e)))?;

    Ok(ApiSuccess("Agent updated successfully"))
}

/// GET /api/agents/:name - Get a single agent by name
pub async fn get_agent(Path(name): Path<String>) -> ApiResult<ApiSuccess<serde_json::Value>> {
    let project_root = find_project_root();
    let project_manager = project_root
        .as_ref()
        .and_then(|root| MarkdownManager::from_directory(root.join(".claude").join("agents")).ok());
    let user_manager = MarkdownManager::from_home_subdir("agents").ok();

    for manager in [project_manager.as_ref(), user_manager.as_ref()]
        .into_iter()
        .flatten()
    {
        if let Ok(files) = manager.list_files_with_folders() {
            for (file_name, folder_path) in files {
                if file_name != name {
                    continue;
                }
                let full_name = if folder_path.is_empty() {
                    file_name.clone()
                } else {
                    format!("{}/{}", folder_path, file_name)
                };

                if let Ok(file) = manager.read_file::<AgentFrontmatter>(&full_name) {
                    let tools = file
                        .frontmatter
                        .tools
                        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                        .unwrap_or_default();

                    let agent = Agent {
                        name: file_name,
                        model: String::new(),
                        tools,
                        system_prompt: Some(file.content),
                        disabled: false,
                        folder: folder_path,
                    };

                    return Ok(ApiSuccess(json!(agent)));
                }
            }
        }
    }

    // Fallback to settings.json (使用全局缓存)
    let settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let agent = settings
        .agents
        .iter()
        .find(|a| a.name == name)
        .ok_or_else(|| ApiError::not_found(format!("Agent '{}' not found", name)))?;

    let api_agent = Agent {
        name: agent.name.clone(),
        model: agent.model.clone(),
        tools: agent.tools.clone(),
        system_prompt: agent.system_prompt.clone(),
        disabled: agent.disabled,
        folder: String::new(),
    };

    Ok(ApiSuccess(json!(api_agent)))
}

/// DELETE /api/agents/:name - Delete an agent
pub async fn delete_agent(Path(name): Path<String>) -> ApiResult<ApiSuccess<&'static str>> {
    let mut settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let original_len = settings.agents.len();
    settings.agents.retain(|a| a.name != name);

    if settings.agents.len() >= original_len {
        return Err(ApiError::not_found("Agent not found"));
    }

    GLOBAL_SETTINGS_CACHE
        .save_atomic(&settings)
        .map_err(|e| ApiError::internal(format!("Failed to save settings: {}", e)))?;

    Ok(ApiSuccess("Agent deleted successfully"))
}

/// PATCH /api/agents/:name/toggle - Toggle agent enabled/disabled state
pub async fn toggle_agent(Path(name): Path<String>) -> ApiResult<ApiSuccess<String>> {
    let mut settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let agent = settings
        .agents
        .iter_mut()
        .find(|a| a.name == name)
        .ok_or_else(|| ApiError::not_found("Agent not found"))?;

    agent.disabled = !agent.disabled;
    let new_state = if agent.disabled {
        "disabled"
    } else {
        "enabled"
    };

    GLOBAL_SETTINGS_CACHE
        .save_atomic(&settings)
        .map_err(|e| ApiError::internal(format!("Failed to save settings: {}", e)))?;

    Ok(ApiSuccess(format!("Agent toggled to {}", new_state)))
}
