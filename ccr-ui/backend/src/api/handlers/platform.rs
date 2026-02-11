// Platform Management Handlers
// Manages multi-platform configuration registry (Unified mode)

use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};

use crate::managers::config::platform_manager::PlatformConfigManager;
use crate::models::api::*;
use crate::utils::config_reader;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// GET /api/platforms - List all platforms with registry info
pub async fn list_platforms() -> impl IntoResponse {
    // Check if Unified mode is active
    let mode = config_reader::detect_config_mode();
    if mode != config_reader::ConfigMode::Unified {
        return ApiResponse::<Vec<PlatformListItem>>::error(
            "Platform management requires Unified mode. Current mode: Legacy".to_string(),
        );
    }

    let manager = match PlatformConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return ApiResponse::<Vec<PlatformListItem>>::error(format!(
                "Failed to initialize platform manager: {}",
                e
            ));
        }
    };

    match manager.list_platforms() {
        Ok(platforms) => {
            let current_platform = manager.get_current_platform().ok();

            let items: Vec<PlatformListItem> = platforms
                .into_iter()
                .map(|(name, entry)| PlatformListItem {
                    name: name.clone(),
                    enabled: entry.enabled,
                    current_profile: entry.current_profile,
                    description: entry.description,
                    last_used: entry.last_used,
                    is_current: current_platform
                        .as_ref()
                        .map(|cp| cp == &name)
                        .unwrap_or(false),
                })
                .collect();

            ApiResponse::success(items)
        }
        Err(e) => {
            ApiResponse::<Vec<PlatformListItem>>::error(format!("Failed to list platforms: {}", e))
        }
    }
}

/// GET /api/platforms/current - Get current active platform
pub async fn get_current_platform() -> impl IntoResponse {
    // Check if Unified mode is active
    let mode = config_reader::detect_config_mode();
    if mode != config_reader::ConfigMode::Unified {
        return ApiResponse::<CurrentPlatformResponse>::error(
            "Platform management requires Unified mode. Current mode: Legacy".to_string(),
        );
    }

    let manager = match PlatformConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return ApiResponse::<CurrentPlatformResponse>::error(format!(
                "Failed to initialize platform manager: {}",
                e
            ));
        }
    };

    match manager.get_current_platform() {
        Ok(platform_name) => {
            // Get platform details
            match manager.get_platform(&platform_name) {
                Ok(entry) => {
                    let response = CurrentPlatformResponse {
                        name: platform_name,
                        enabled: entry.enabled,
                        current_profile: entry.current_profile,
                        description: entry.description,
                        last_used: entry.last_used,
                    };
                    ApiResponse::success(response)
                }
                Err(e) => ApiResponse::<CurrentPlatformResponse>::error(format!(
                    "Failed to get platform details: {}",
                    e
                )),
            }
        }
        Err(e) => ApiResponse::<CurrentPlatformResponse>::error(format!(
            "Failed to get current platform: {}",
            e
        )),
    }
}

/// POST /api/platforms/switch - Switch to a different platform
pub async fn switch_platform(Json(req): Json<SwitchPlatformRequest>) -> impl IntoResponse {
    // Check if Unified mode is active
    let mode = config_reader::detect_config_mode();
    if mode != config_reader::ConfigMode::Unified {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String>::error(
                "Platform switching requires Unified mode. Current mode: Legacy".to_string(),
            )),
        );
    }

    let manager = match PlatformConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<String>::error(format!(
                    "Failed to initialize platform manager: {}",
                    e
                ))),
            );
        }
    };

    match manager.set_current_platform(&req.platform_name) {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::<String>::success(format!(
                "Successfully switched to platform '{}'",
                req.platform_name
            ))),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String>::error(format!(
                "Failed to switch platform: {}",
                e
            ))),
        ),
    }
}

/// GET /api/platforms/:name - Get specific platform details
pub async fn get_platform(Path(name): Path<String>) -> impl IntoResponse {
    // Check if Unified mode is active
    let mode = config_reader::detect_config_mode();
    if mode != config_reader::ConfigMode::Unified {
        return ApiResponse::<PlatformDetailResponse>::error(
            "Platform management requires Unified mode. Current mode: Legacy".to_string(),
        );
    }

    let manager = match PlatformConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return ApiResponse::<PlatformDetailResponse>::error(format!(
                "Failed to initialize platform manager: {}",
                e
            ));
        }
    };

    match manager.get_platform(&name) {
        Ok(entry) => {
            let current_platform = manager.get_current_platform().ok();
            let is_current = current_platform
                .as_ref()
                .map(|cp| cp == &name)
                .unwrap_or(false);

            let response = PlatformDetailResponse {
                name: name.clone(),
                enabled: entry.enabled,
                current_profile: entry.current_profile,
                description: entry.description,
                last_used: entry.last_used,
                is_current,
            };
            ApiResponse::success(response)
        }
        Err(e) => ApiResponse::<PlatformDetailResponse>::error(format!(
            "Failed to get platform '{}': {}",
            name, e
        )),
    }
}

/// PUT /api/platforms/:name - Update platform registry entry
pub async fn update_platform(
    Path(name): Path<String>,
    Json(req): Json<UpdatePlatformRequest>,
) -> impl IntoResponse {
    // Check if Unified mode is active
    let mode = config_reader::detect_config_mode();
    if mode != config_reader::ConfigMode::Unified {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String>::error(
                "Platform management requires Unified mode. Current mode: Legacy".to_string(),
            )),
        );
    }

    let manager = match PlatformConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<String>::error(format!(
                    "Failed to initialize platform manager: {}",
                    e
                ))),
            );
        }
    };

    // Get existing entry first
    let mut entry = match manager.get_platform(&name) {
        Ok(e) => e,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<String>::error(format!(
                    "Platform '{}' not found: {}",
                    name, e
                ))),
            );
        }
    };

    // Update fields if provided
    if let Some(enabled) = req.enabled {
        entry.enabled = enabled;
    }
    if let Some(description) = req.description {
        entry.description = Some(description);
    }
    if let Some(current_profile) = req.current_profile {
        entry.current_profile = Some(current_profile);
    }

    match manager.register_platform(name.clone(), entry) {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::<String>::success(format!(
                "Successfully updated platform '{}'",
                name
            ))),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<String>::error(format!(
                "Failed to update platform '{}': {}",
                name, e
            ))),
        ),
    }
}

/// POST /api/platforms/:name/enable - Enable a platform
pub async fn enable_platform(Path(name): Path<String>) -> impl IntoResponse {
    // Check if Unified mode is active
    let mode = config_reader::detect_config_mode();
    if mode != config_reader::ConfigMode::Unified {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String>::error(
                "Platform management requires Unified mode. Current mode: Legacy".to_string(),
            )),
        );
    }

    let manager = match PlatformConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<String>::error(format!(
                    "Failed to initialize platform manager: {}",
                    e
                ))),
            );
        }
    };

    match manager.enable_platform(&name) {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::<String>::success(format!(
                "Successfully enabled platform '{}'",
                name
            ))),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String>::error(format!(
                "Failed to enable platform '{}': {}",
                name, e
            ))),
        ),
    }
}

/// POST /api/platforms/:name/disable - Disable a platform
pub async fn disable_platform(Path(name): Path<String>) -> impl IntoResponse {
    // Check if Unified mode is active
    let mode = config_reader::detect_config_mode();
    if mode != config_reader::ConfigMode::Unified {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String>::error(
                "Platform management requires Unified mode. Current mode: Legacy".to_string(),
            )),
        );
    }

    let manager = match PlatformConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<String>::error(format!(
                    "Failed to initialize platform manager: {}",
                    e
                ))),
            );
        }
    };

    match manager.disable_platform(&name) {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::<String>::success(format!(
                "Successfully disabled platform '{}'",
                name
            ))),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String>::error(format!(
                "Failed to disable platform '{}': {}",
                name, e
            ))),
        ),
    }
}

/// GET /api/platforms/:name/profile - Get platform's current profile
pub async fn get_platform_profile(Path(name): Path<String>) -> impl IntoResponse {
    // Check if Unified mode is active
    let mode = config_reader::detect_config_mode();
    if mode != config_reader::ConfigMode::Unified {
        return ApiResponse::<PlatformProfileResponse>::error(
            "Platform management requires Unified mode. Current mode: Legacy".to_string(),
        );
    }

    let manager = match PlatformConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return ApiResponse::<PlatformProfileResponse>::error(format!(
                "Failed to initialize platform manager: {}",
                e
            ));
        }
    };

    match manager.get_platform_profile(&name) {
        Ok(profile) => {
            let response = PlatformProfileResponse {
                platform_name: name,
                current_profile: profile,
            };
            ApiResponse::success(response)
        }
        Err(e) => ApiResponse::<PlatformProfileResponse>::error(format!(
            "Failed to get platform profile: {}",
            e
        )),
    }
}

/// POST /api/platforms/:name/profile - Set platform's current profile
pub async fn set_platform_profile(
    Path(name): Path<String>,
    Json(req): Json<SetPlatformProfileRequest>,
) -> impl IntoResponse {
    // Check if Unified mode is active
    let mode = config_reader::detect_config_mode();
    if mode != config_reader::ConfigMode::Unified {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String>::error(
                "Platform management requires Unified mode. Current mode: Legacy".to_string(),
            )),
        );
    }

    let manager = match PlatformConfigManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<String>::error(format!(
                    "Failed to initialize platform manager: {}",
                    e
                ))),
            );
        }
    };

    match manager.set_platform_profile(&name, &req.profile_name) {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiResponse::<String>::success(format!(
                "Successfully set platform '{}' profile to '{}'",
                name, req.profile_name
            ))),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<String>::error(format!(
                "Failed to set platform profile: {}",
                e
            ))),
        ),
    }
}

fn get_home_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(PathBuf::from)
}

fn find_project_root() -> Option<PathBuf> {
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

fn detect_claude_dirs(project_root: &std::path::Path) -> (bool, bool, bool) {
    let home = match get_home_dir() {
        Some(h) => h,
        None => return (false, false, false),
    };

    let user_base = home.join(".claude");
    let project_base = project_root.join(".claude");

    let agents = user_base.join("agents").exists() || project_base.join("agents").exists();
    let slash_commands =
        user_base.join("commands").exists() || project_base.join("commands").exists();
    let skills = user_base.join("skills").exists() || project_base.join("skills").exists();

    (agents, slash_commands, skills)
}

pub async fn get_platform_capabilities() -> impl IntoResponse {
    let project_root = match find_project_root() {
        Some(p) => p,
        None => {
            return ApiResponse::<PlatformCapabilitiesResponse>::error(
                "Failed to locate project root".to_string(),
            );
        }
    };

    let (claude_agents, claude_slash, claude_skills) = detect_claude_dirs(project_root.as_path());

    let mut platforms: BTreeMap<String, PlatformModuleCapabilities> = BTreeMap::new();

    platforms.insert(
        "claude-code".to_string(),
        PlatformModuleCapabilities {
            config: true,
            mcp: true,
            profiles: false,
            auth: false,
            usage: true,
            agents: true,
            slash_commands: true,
            skills: true,
            plugins: true,
            commands: true,
        },
    );

    platforms.insert(
        "codex".to_string(),
        PlatformModuleCapabilities {
            config: true,
            mcp: true,
            profiles: true,
            auth: true,
            usage: true,
            agents: false,
            slash_commands: true,
            skills: false,
            plugins: false,
            commands: false,
        },
    );

    platforms.insert(
        "gemini-cli".to_string(),
        PlatformModuleCapabilities {
            config: true,
            mcp: true,
            profiles: false,
            auth: false,
            usage: false,
            agents: false,
            slash_commands: true,
            skills: false,
            plugins: false,
            commands: false,
        },
    );

    platforms.insert(
        "qwen".to_string(),
        PlatformModuleCapabilities {
            config: true,
            mcp: true,
            profiles: false,
            auth: false,
            usage: false,
            agents: false,
            slash_commands: true,
            skills: false,
            plugins: false,
            commands: false,
        },
    );

    platforms.insert(
        "iflow".to_string(),
        PlatformModuleCapabilities {
            config: false,
            mcp: false,
            profiles: false,
            auth: false,
            usage: false,
            agents: false,
            slash_commands: false,
            skills: false,
            plugins: false,
            commands: false,
        },
    );

    platforms.insert(
        "droid".to_string(),
        PlatformModuleCapabilities {
            config: true,
            mcp: true,
            profiles: true,
            auth: false,
            usage: false,
            agents: true,
            slash_commands: true,
            skills: false,
            plugins: true,
            commands: false,
        },
    );

    if let Some(claude) = platforms.get_mut("claude-code") {
        claude.agents = claude_agents;
        claude.slash_commands = claude_slash;
        claude.skills = claude_skills;
    }

    ApiResponse::success(PlatformCapabilitiesResponse { platforms })
}
