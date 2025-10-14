use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde_json::json;
use crate::plugins_manager::PluginsManager;
use crate::models::{Plugin, PluginRequest, PluginsResponse};

#[get("/api/plugins")]
async fn list_plugins() -> impl Responder {
    let manager = match PluginsManager::default() {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize plugins manager: {}", e)
            }))
        }
    };

    match manager.get_plugins() {
        Ok(repositories) => {
            let plugins: Vec<Plugin> = repositories
                .into_iter()
                .map(|(id, config)| Plugin {
                    id: id.clone(),
                    name: config
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(&id)
                        .to_string(),
                    version: config
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("1.0.0")
                        .to_string(),
                    enabled: config
                        .get("enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true),
                    config: Some(config),
                })
                .collect();

            HttpResponse::Ok().json(PluginsResponse {
                success: true,
                data: json!({ "plugins": plugins }),
                message: None,
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to read plugins: {}", e)
        })),
    }
}

#[post("/api/plugins")]
async fn add_plugin(req: web::Json<PluginRequest>) -> impl Responder {
    let manager = match PluginsManager::default() {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize plugins manager: {}", e)
            }))
        }
    };

    let mut plugin_data = req.config.clone().unwrap_or_else(|| json!({}));
    
    // Ensure required fields are in the config
    if let Some(obj) = plugin_data.as_object_mut() {
        obj.insert("name".to_string(), json!(req.name));
        obj.insert("version".to_string(), json!(req.version));
        obj.insert("enabled".to_string(), json!(req.enabled));
    }

    match manager.add_plugin(req.id.clone(), plugin_data) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": "Plugin added successfully",
            "message": null
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to add plugin: {}", e)
        })),
    }
}

#[put("/api/plugins/{id}")]
async fn update_plugin(
    path: web::Path<String>,
    req: web::Json<PluginRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let manager = match PluginsManager::default() {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize plugins manager: {}", e)
            }))
        }
    };

    let mut plugin_data = req.config.clone().unwrap_or_else(|| json!({}));
    
    // Ensure required fields are in the config
    if let Some(obj) = plugin_data.as_object_mut() {
        obj.insert("name".to_string(), json!(req.name));
        obj.insert("version".to_string(), json!(req.version));
        obj.insert("enabled".to_string(), json!(req.enabled));
    }

    match manager.update_plugin(&id, plugin_data) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": "Plugin updated successfully",
            "message": null
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to update plugin: {}", e)
        })),
    }
}

#[delete("/api/plugins/{id}")]
async fn delete_plugin(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let manager = match PluginsManager::default() {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize plugins manager: {}", e)
            }))
        }
    };

    match manager.delete_plugin(&id) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "data": "Plugin deleted successfully",
            "message": null
        })),
        Err(e) => HttpResponse::NotFound().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to delete plugin: {}", e)
        })),
    }
}

#[put("/api/plugins/{id}/toggle")]
async fn toggle_plugin(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let manager = match PluginsManager::default() {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "data": null,
                "message": format!("Failed to initialize plugins manager: {}", e)
            }))
        }
    };

    // Read current plugin
    match manager.get_plugins() {
        Ok(mut repositories) => {
            if let Some(mut plugin_data) = repositories.remove(&id) {
                // Toggle enabled field
                if let Some(obj) = plugin_data.as_object_mut() {
                    let current_enabled = obj
                        .get("enabled")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true);
                    obj.insert("enabled".to_string(), json!(!current_enabled));
                }

                match manager.update_plugin(&id, plugin_data) {
                    Ok(_) => HttpResponse::Ok().json(json!({
                        "success": true,
                        "data": "Plugin toggled successfully",
                        "message": null
                    })),
                    Err(e) => HttpResponse::InternalServerError().json(json!({
                        "success": false,
                        "data": null,
                        "message": format!("Failed to toggle plugin: {}", e)
                    })),
                }
            } else {
                HttpResponse::NotFound().json(json!({
                    "success": false,
                    "data": null,
                    "message": format!("Plugin '{}' not found", id)
                }))
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "data": null,
            "message": format!("Failed to read plugins: {}", e)
        })),
    }
}
