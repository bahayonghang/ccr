use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::cache::GLOBAL_SETTINGS_CACHE;
use crate::models::api::{Hook, HookRequest};

/// GET /api/hooks - List all hooks
pub async fn list_hooks() -> impl IntoResponse {
    let settings_result = GLOBAL_SETTINGS_CACHE.load();

    if let Ok(settings) = settings_result {
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

        return (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": hooks,
                "message": null
            })),
        )
            .into_response();
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "success": false,
            "data": null,
            "message": "Failed to load hooks"
        })),
    )
        .into_response()
}

/// POST /api/hooks - Add a new hook
pub async fn add_hook(Json(req): Json<HookRequest>) -> impl IntoResponse {
    if let Ok(mut settings) = GLOBAL_SETTINGS_CACHE.load() {
        let new_hook = crate::managers::settings_manager::Hook {
            event: req.event,
            command: req.command,
            enabled: req.enabled.unwrap_or(true),
            description: req.description,
        };

        settings.hooks.push(new_hook);

        if GLOBAL_SETTINGS_CACHE.save_atomic(&settings).is_ok() {
            return (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "data": "Hook added successfully",
                    "message": null
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
            "message": "Failed to add hook"
        })),
    )
        .into_response()
}

/// PATCH /api/hooks/:event - Update a hook
pub async fn update_hook(
    Path(event): Path<String>,
    Json(req): Json<HookRequest>,
) -> impl IntoResponse {
    if let Ok(mut settings) = GLOBAL_SETTINGS_CACHE.load() {
        if let Some(hook) = settings.hooks.iter_mut().find(|h| h.event == event) {
            hook.event = req.event;
            hook.command = req.command;
            if let Some(enabled) = req.enabled {
                hook.enabled = enabled;
            }
            hook.description = req.description;

            if GLOBAL_SETTINGS_CACHE.save_atomic(&settings).is_ok() {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "data": "Hook updated successfully",
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
                    "message": "Hook not found"
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
            "message": "Failed to update hook"
        })),
    )
        .into_response()
}

/// DELETE /api/hooks/:event - Delete a hook
pub async fn delete_hook(Path(event): Path<String>) -> impl IntoResponse {
    if let Ok(mut settings) = GLOBAL_SETTINGS_CACHE.load() {
        let original_len = settings.hooks.len();
        settings.hooks.retain(|h| h.event != event);

        if settings.hooks.len() < original_len {
            if GLOBAL_SETTINGS_CACHE.save_atomic(&settings).is_ok() {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "data": "Hook deleted successfully",
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
                    "message": "Hook not found"
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
            "message": "Failed to delete hook"
        })),
    )
        .into_response()
}

/// PATCH /api/hooks/:event/toggle - Toggle hook enabled/disabled state
pub async fn toggle_hook(Path(event): Path<String>) -> impl IntoResponse {
    if let Ok(mut settings) = GLOBAL_SETTINGS_CACHE.load() {
        if let Some(hook) = settings.hooks.iter_mut().find(|h| h.event == event) {
            hook.enabled = !hook.enabled;
            let new_state = if hook.enabled { "enabled" } else { "disabled" };

            if GLOBAL_SETTINGS_CACHE.save_atomic(&settings).is_ok() {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "data": format!("Hook toggled to {}", new_state),
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
                    "message": "Hook not found"
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
            "message": "Failed to toggle hook"
        })),
    )
        .into_response()
}
