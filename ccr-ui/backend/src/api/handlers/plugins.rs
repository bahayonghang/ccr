use axum::{Json, extract::Path};
use serde_json::json;

use crate::api::handlers::response::ApiSuccess;
use crate::cache::GLOBAL_SETTINGS_CACHE;
use crate::core::error::{ApiError, ApiResult};
use crate::managers::plugins_manager::PluginsManager;
use crate::models::api::{Plugin, PluginRequest};

/// GET /api/plugins - List all plugins
pub async fn list_plugins() -> ApiResult<ApiSuccess<serde_json::Value>> {
    // Try settings.json first (preferred，使用全局缓存)
    if let Ok(settings) = GLOBAL_SETTINGS_CACHE.load() {
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

        return Ok(ApiSuccess(json!({ "plugins": plugins })));
    }

    // Fallback to plugins config
    let manager = PluginsManager::default()
        .map_err(|e| ApiError::internal(format!("Failed to initialize plugins manager: {}", e)))?;

    let plugins_map = manager
        .get_plugins()
        .map_err(|e| ApiError::internal(format!("Failed to list plugins: {}", e)))?;

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

    Ok(ApiSuccess(json!({ "plugins": plugins })))
}

/// POST /api/plugins - Add a new plugin
pub async fn add_plugin(Json(req): Json<PluginRequest>) -> ApiResult<ApiSuccess<&'static str>> {
    // Try settings.json first (使用全局缓存)
    if let Ok(mut settings) = GLOBAL_SETTINGS_CACHE.load() {
        let new_plugin = crate::managers::settings_manager::Plugin {
            id: req.id.clone(),
            name: req.name.clone(),
            version: req.version.clone(),
            enabled: req.enabled.unwrap_or(true),
            config: req.config.clone(),
            other: std::collections::HashMap::new(),
        };

        settings.plugins.push(new_plugin);

        if GLOBAL_SETTINGS_CACHE.save_atomic(&settings).is_ok() {
            return Ok(ApiSuccess("Plugin added successfully"));
        }
    }

    // Fallback to plugins config
    let manager = PluginsManager::default()
        .map_err(|e| ApiError::internal(format!("Failed to initialize plugins manager: {}", e)))?;

    let mut plugin_data = json!({
        "name": req.name,
        "version": req.version,
        "enabled": req.enabled.unwrap_or(true),
    });

    if let Some(config) = req.config
        && let Some(obj) = plugin_data.as_object_mut()
    {
        obj.insert("config".to_string(), config);
    }

    manager
        .add_plugin(req.id, plugin_data)
        .map_err(|e| ApiError::internal(format!("Failed to add plugin: {}", e)))?;

    Ok(ApiSuccess("Plugin added successfully"))
}

/// PATCH /api/plugins/:id - Update a plugin
pub async fn update_plugin(
    Path(id): Path<String>,
    Json(req): Json<PluginRequest>,
) -> ApiResult<ApiSuccess<&'static str>> {
    // Try settings.json first (使用全局缓存)
    if let Ok(mut settings) = GLOBAL_SETTINGS_CACHE.load() {
        if let Some(plugin) = settings.plugins.iter_mut().find(|p| p.id == id) {
            plugin.id = req.id.clone();
            plugin.name = req.name.clone();
            plugin.version = req.version.clone();
            if let Some(enabled) = req.enabled {
                plugin.enabled = enabled;
            }
            plugin.config = req.config.clone();

            if GLOBAL_SETTINGS_CACHE.save_atomic(&settings).is_ok() {
                return Ok(ApiSuccess("Plugin updated successfully"));
            }
        } else {
            return Err(ApiError::not_found("Plugin not found"));
        }
    }

    // Fallback to plugins config
    let manager = PluginsManager::default()
        .map_err(|e| ApiError::internal(format!("Failed to initialize plugins manager: {}", e)))?;

    let mut plugin_data = json!({
        "name": req.name,
        "version": req.version,
        "enabled": req.enabled.unwrap_or(true),
    });

    if let Some(config) = req.config
        && let Some(obj) = plugin_data.as_object_mut()
    {
        obj.insert("config".to_string(), config);
    }

    manager
        .update_plugin(&id, plugin_data)
        .map_err(|e| ApiError::internal(format!("Failed to update plugin: {}", e)))?;

    Ok(ApiSuccess("Plugin updated successfully"))
}

/// DELETE /api/plugins/:id - Delete a plugin
pub async fn delete_plugin(Path(id): Path<String>) -> ApiResult<ApiSuccess<&'static str>> {
    // Try settings.json first (使用全局缓存)
    if let Ok(mut settings) = GLOBAL_SETTINGS_CACHE.load() {
        let original_len = settings.plugins.len();
        settings.plugins.retain(|p| p.id != id);

        if settings.plugins.len() < original_len {
            if GLOBAL_SETTINGS_CACHE.save_atomic(&settings).is_ok() {
                return Ok(ApiSuccess("Plugin deleted successfully"));
            }
        } else {
            return Err(ApiError::not_found("Plugin not found"));
        }
    }

    // Fallback to plugins config
    let manager = PluginsManager::default()
        .map_err(|e| ApiError::internal(format!("Failed to initialize plugins manager: {}", e)))?;

    manager
        .delete_plugin(&id)
        .map_err(|e| ApiError::not_found(format!("Failed to delete plugin: {}", e)))?;

    Ok(ApiSuccess("Plugin deleted successfully"))
}

/// PATCH /api/plugins/:id/toggle - Toggle plugin enabled/disabled state
pub async fn toggle_plugin(Path(id): Path<String>) -> ApiResult<ApiSuccess<String>> {
    let mut settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let plugin = settings
        .plugins
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| ApiError::not_found("Plugin not found"))?;

    plugin.enabled = !plugin.enabled;
    let new_state = if plugin.enabled {
        "enabled"
    } else {
        "disabled"
    };

    GLOBAL_SETTINGS_CACHE
        .save_atomic(&settings)
        .map_err(|e| ApiError::internal(format!("Failed to save settings: {}", e)))?;

    Ok(ApiSuccess(format!("Plugin toggled to {}", new_state)))
}
