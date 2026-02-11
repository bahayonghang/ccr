use axum::{Json, extract::Path};

use crate::api::handlers::response::ApiSuccess;
use crate::cache::GLOBAL_SETTINGS_CACHE;
use crate::core::error::{ApiError, ApiResult};
use crate::models::api::{Hook, HookRequest};

/// GET /api/hooks - List all hooks
pub async fn list_hooks() -> ApiResult<ApiSuccess<Vec<Hook>>> {
    let settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load hooks: {}", e)))?;

    let hooks: Vec<Hook> = settings
        .hooks
        .into_iter()
        .map(|hook| Hook {
            event: hook.event,
            command: hook.command,
            enabled: hook.enabled,
            description: hook.description,
        })
        .collect();

    Ok(ApiSuccess(hooks))
}

/// POST /api/hooks - Add a new hook
pub async fn add_hook(Json(req): Json<HookRequest>) -> ApiResult<ApiSuccess<&'static str>> {
    let mut settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let new_hook = crate::managers::settings_manager::Hook {
        event: req.event,
        command: req.command,
        enabled: req.enabled.unwrap_or(true),
        description: req.description,
        other: std::collections::HashMap::new(),
    };

    settings.hooks.push(new_hook);

    GLOBAL_SETTINGS_CACHE
        .save_atomic(&settings)
        .map_err(|e| ApiError::internal(format!("Failed to save settings: {}", e)))?;

    Ok(ApiSuccess("Hook added successfully"))
}

/// PATCH /api/hooks/:event - Update a hook
pub async fn update_hook(
    Path(event): Path<String>,
    Json(req): Json<HookRequest>,
) -> ApiResult<ApiSuccess<&'static str>> {
    let mut settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let hook = settings
        .hooks
        .iter_mut()
        .find(|h| h.event == event)
        .ok_or_else(|| ApiError::not_found("Hook not found"))?;

    hook.event = req.event;
    hook.command = req.command;
    if let Some(enabled) = req.enabled {
        hook.enabled = enabled;
    }
    hook.description = req.description;

    GLOBAL_SETTINGS_CACHE
        .save_atomic(&settings)
        .map_err(|e| ApiError::internal(format!("Failed to save settings: {}", e)))?;

    Ok(ApiSuccess("Hook updated successfully"))
}

/// DELETE /api/hooks/:event - Delete a hook
pub async fn delete_hook(Path(event): Path<String>) -> ApiResult<ApiSuccess<&'static str>> {
    let mut settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let original_len = settings.hooks.len();
    settings.hooks.retain(|h| h.event != event);

    if settings.hooks.len() >= original_len {
        return Err(ApiError::not_found("Hook not found"));
    }

    GLOBAL_SETTINGS_CACHE
        .save_atomic(&settings)
        .map_err(|e| ApiError::internal(format!("Failed to save settings: {}", e)))?;

    Ok(ApiSuccess("Hook deleted successfully"))
}

/// PATCH /api/hooks/:event/toggle - Toggle hook enabled/disabled state
pub async fn toggle_hook(Path(event): Path<String>) -> ApiResult<ApiSuccess<String>> {
    let mut settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let hook = settings
        .hooks
        .iter_mut()
        .find(|h| h.event == event)
        .ok_or_else(|| ApiError::not_found("Hook not found"))?;

    hook.enabled = !hook.enabled;
    let new_state = if hook.enabled { "enabled" } else { "disabled" };

    GLOBAL_SETTINGS_CACHE
        .save_atomic(&settings)
        .map_err(|e| ApiError::internal(format!("Failed to save settings: {}", e)))?;

    Ok(ApiSuccess(format!("Hook toggled to {}", new_state)))
}
