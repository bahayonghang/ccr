use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::models::{Plugin, PluginRequest};
use crate::plugins_manager::PluginsManager;
use crate::settings_manager::SettingsManager;

/// GET /api/plugins - List all plugins
pub async fn list_plugins() -> impl IntoResponse {
    // Try settings.json first (preferred)
    let settings_result = SettingsManager::default().and_then(|manager| manager.load());

    if let Ok(settings) = settings_result {
        let plugins: Vec<Plugin> = settings
            .plugins
            .into_iter()
            .map(|plugin| Plugin {
                id: plugin.id,
                name: plugin.name,
                version: plugin.version,
                enabled: plugin.enabled,
                config: plugin.config,
            })
            .collect();

        return (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": { "plugins": plugins },
                "message": null
            })),
        )
            .into_response();
    }

    // Fallback to plugins config
    let manager = match PluginsManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to initialize plugins manager: {}", e)
                })),
            )
                .into_response();
        }
    };

    match manager.get_plugins() {
        Ok(plugins_map) => {
            let plugins: Vec<Plugin> = plugins_map
                .into_iter()
                .map(|(id, value)| {
                    let name = value
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&id)
                        .to_string();
                    let version = value
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("1.0.0")
                        .to_string();
                    let enabled = value
                        .get("enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);

                    Plugin {
                        id,
                        name,
                        version,
                        enabled,
                        config: Some(value),
                    }
                })
                .collect();

            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "data": { "plugins": plugins },
                    "message": null
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to list plugins: {}", e)
            })),
        )
            .into_response(),
    }
}

/// POST /api/plugins - Add a new plugin
pub async fn add_plugin(Json(req): Json<PluginRequest>) -> impl IntoResponse {
    // Try settings.json first
    if let Ok(settings_manager) = SettingsManager::default()
        && let Ok(mut settings) = settings_manager.load()
    {
        let new_plugin = crate::settings_manager::Plugin {
            id: req.id.clone(),
            name: req.name.clone(),
            version: req.version.clone(),
            enabled: req.enabled.unwrap_or(true),
            config: req.config.clone(),
        };

        settings.plugins.push(new_plugin);

        if settings_manager.save(&settings).is_ok() {
            return (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "data": "Plugin added successfully",
                    "message": null
                })),
            )
                .into_response();
        }
    }

    // Fallback to plugins config
    let manager = match PluginsManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to initialize plugins manager: {}", e)
                })),
            )
                .into_response();
        }
    };

    let mut plugin_data = json!({
        "name": req.name,
        "version": req.version,
        "enabled": req.enabled.unwrap_or(true),
    });

    if let Some(config) = req.config {
        plugin_data
            .as_object_mut()
            .unwrap()
            .insert("config".to_string(), config);
    }

    match manager.add_plugin(req.id, plugin_data) {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": "Plugin added successfully",
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to add plugin: {}", e)
            })),
        )
            .into_response(),
    }
}

/// PATCH /api/plugins/:id - Update a plugin
pub async fn update_plugin(
    Path(id): Path<String>,
    Json(req): Json<PluginRequest>,
) -> impl IntoResponse {
    // Try settings.json first
    if let Ok(settings_manager) = SettingsManager::default()
        && let Ok(mut settings) = settings_manager.load()
    {
        if let Some(plugin) = settings.plugins.iter_mut().find(|p| p.id == id) {
            plugin.id = req.id.clone();
            plugin.name = req.name.clone();
            plugin.version = req.version.clone();
            if let Some(enabled) = req.enabled {
                plugin.enabled = enabled;
            }
            plugin.config = req.config.clone();

            if settings_manager.save(&settings).is_ok() {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "data": "Plugin updated successfully",
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
                    "message": "Plugin not found"
                })),
            )
                .into_response();
        }
    }

    // Fallback to plugins config
    let manager = match PluginsManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to initialize plugins manager: {}", e)
                })),
            )
                .into_response();
        }
    };

    let mut plugin_data = json!({
        "name": req.name,
        "version": req.version,
        "enabled": req.enabled.unwrap_or(true),
    });

    if let Some(config) = req.config {
        plugin_data
            .as_object_mut()
            .unwrap()
            .insert("config".to_string(), config);
    }

    match manager.update_plugin(&id, plugin_data) {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": "Plugin updated successfully",
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to update plugin: {}", e)
            })),
        )
            .into_response(),
    }
}

/// DELETE /api/plugins/:id - Delete a plugin
pub async fn delete_plugin(Path(id): Path<String>) -> impl IntoResponse {
    // Try settings.json first
    if let Ok(settings_manager) = SettingsManager::default()
        && let Ok(mut settings) = settings_manager.load()
    {
        let original_len = settings.plugins.len();
        settings.plugins.retain(|p| p.id != id);

        if settings.plugins.len() < original_len {
            if settings_manager.save(&settings).is_ok() {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "data": "Plugin deleted successfully",
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
                    "message": "Plugin not found"
                })),
            )
                .into_response();
        }
    }

    // Fallback to plugins config
    let manager = match PluginsManager::default() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Failed to initialize plugins manager: {}", e)
                })),
            )
                .into_response();
        }
    };

    match manager.delete_plugin(&id) {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": "Plugin deleted successfully",
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to delete plugin: {}", e)
            })),
        )
            .into_response(),
    }
}

/// PATCH /api/plugins/:id/toggle - Toggle plugin enabled/disabled state
pub async fn toggle_plugin(Path(id): Path<String>) -> impl IntoResponse {
    // Try settings.json first
    if let Ok(settings_manager) = SettingsManager::default()
        && let Ok(mut settings) = settings_manager.load()
    {
        if let Some(plugin) = settings.plugins.iter_mut().find(|p| p.id == id) {
            plugin.enabled = !plugin.enabled;
            let new_state = if plugin.enabled {
                "enabled"
            } else {
                "disabled"
            };

            if settings_manager.save(&settings).is_ok() {
                return (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "data": format!("Plugin toggled to {}", new_state),
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
                    "message": "Plugin not found"
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
            "message": "Failed to toggle plugin"
        })),
    )
        .into_response()
}
