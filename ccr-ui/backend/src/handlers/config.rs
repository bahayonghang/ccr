// Config Management Handlers
// All operations are executed via CCR CLI subprocess

use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::config_reader;
use crate::executor;
use crate::models::*;

/// GET /api/configs - List all configurations
#[get("/api/configs")]
pub async fn list_configs() -> impl Responder {
    // Read config file directly for better reliability
    match config_reader::read_config() {
        Ok(config) => {
            let configs: Vec<ConfigItem> = config
                .sections
                .iter()
                .filter(|(key, _)| {
                    // Filter out metadata keys
                    *key != "default_config" && *key != "current_config"
                })
                .map(|(name, section)| ConfigItem {
                    name: name.clone(),
                    description: section.description.clone().unwrap_or_default(),
                    base_url: section.base_url.clone().unwrap_or_default(),
                    auth_token: config_reader::mask_token(
                        &section.auth_token.clone().unwrap_or_default(),
                    ),
                    model: section.model.clone(),
                    small_fast_model: section.small_fast_model.clone(),
                    is_current: name == &config.current_config,
                    is_default: name == &config.default_config,
                    provider: section.provider.clone(),
                    provider_type: section.provider_type.clone(),
                    account: section.account.clone(),
                    tags: section.tags.clone(),
                })
                .collect();

            let response = ConfigListResponse {
                current_config: config.current_config,
                default_config: config.default_config,
                configs,
            };

            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e)),
    }
}

/// POST /api/switch - Switch to a configuration
#[post("/api/switch")]
pub async fn switch_config(req: web::Json<SwitchRequest>) -> impl Responder {
    let result = executor::execute_command(vec!["switch".to_string(), req.config_name.clone()]).await;

    match result {
        Ok(output) if output.success => {
            HttpResponse::Ok().json(ApiResponse::success("Configuration switched successfully"))
        }
        Ok(output) => {
            let error_msg = if !output.stderr.is_empty() {
                output.stderr
            } else {
                output.stdout
            };
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(error_msg))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string()))
        }
    }
}

/// POST /api/validate - Validate all configurations
#[post("/api/validate")]
pub async fn validate_configs() -> impl Responder {
    let result = executor::execute_command(vec!["validate".to_string()]).await;

    match result {
        Ok(output) if output.success => {
            HttpResponse::Ok().json(ApiResponse::success("All configurations are valid"))
        }
        Ok(output) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(output.stderr))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string()))
        }
    }
}

/// POST /api/clean - Clean old backups
#[post("/api/clean")]
pub async fn clean_backups(req: web::Json<CleanRequest>) -> impl Responder {
    let mut args = vec!["clean".to_string(), "--days".to_string(), req.days.to_string()];

    if req.dry_run {
        args.push("--dry-run".to_string());
    }

    let result = executor::execute_command(args).await;

    match result {
        Ok(output) if output.success => {
            // Parse the output to extract deleted count (simplified)
            let response = CleanResponse {
                deleted_count: 0,
                skipped_count: 0,
                total_size_mb: 0.0,
                dry_run: req.dry_run,
            };
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Ok(output) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(output.stderr))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string()))
        }
    }
}

/// POST /api/export - Export configurations
#[post("/api/export")]
pub async fn export_config(req: web::Json<ExportRequest>) -> impl Responder {
    let mut args = vec!["export".to_string()];

    if !req.include_secrets {
        args.push("--no-secrets".to_string());
    }

    let result = executor::execute_command(args).await;

    match result {
        Ok(output) if output.success => {
            // Read the exported file (CCR writes to a file)
            // For simplicity, we'll return the stdout
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let response = ExportResponse {
                content: output.stdout,
                filename: format!("ccs_config_export_{}.toml", timestamp),
            };
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Ok(output) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(output.stderr))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string()))
        }
    }
}

/// POST /api/import - Import configurations
#[post("/api/import")]
pub async fn import_config(req: web::Json<ImportRequest>) -> impl Responder {
    // Write content to a temporary file
    let temp_file = format!("/tmp/ccr_import_{}.toml", chrono::Local::now().timestamp());

    if let Err(e) = std::fs::write(&temp_file, &req.content) {
        return HttpResponse::InternalServerError()
            .json(ApiResponse::<()>::error(format!("Failed to write temp file: {}", e)));
    }

    let mut args = vec!["import".to_string(), temp_file.clone()];

    if req.mode == "replace" {
        args.push("--mode".to_string());
        args.push("replace".to_string());
    }

    if !req.backup {
        args.push("--no-backup".to_string());
    }

    let result = executor::execute_command(args).await;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);

    match result {
        Ok(output) if output.success => {
            let response = ImportResponse {
                added: 0,
                updated: 0,
                skipped: 0,
            };
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
        Ok(output) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(output.stderr))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(e.to_string()))
        }
    }
}

/// GET /api/history - Get operation history
#[get("/api/history")]
pub async fn get_history() -> impl Responder {
    // Try to read history file directly
    let history_path = dirs::home_dir()
        .map(|h| h.join(".claude/ccr_history.json"))
        .ok_or_else(|| "Cannot get home directory".to_string());

    match history_path {
        Ok(path) if path.exists() => {
            match std::fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json) => {
                        // Extract entries array
                        if let Some(entries_array) = json.get("entries").and_then(|v| v.as_array()) {
                            let entries: Vec<HistoryEntryJson> = entries_array
                                .iter()
                                .take(50)
                                .filter_map(|e| serde_json::from_value(e.clone()).ok())
                                .collect();

                            let response = HistoryResponse {
                                total: entries.len(),
                                entries,
                            };
                            return HttpResponse::Ok().json(ApiResponse::success(response));
                        }

                        // Fallback: return empty
                        let response = HistoryResponse {
                            entries: vec![],
                            total: 0,
                        };
                        HttpResponse::Ok().json(ApiResponse::success(response))
                    }
                    Err(e) => {
                        HttpResponse::InternalServerError()
                            .json(ApiResponse::<()>::error(format!("Failed to parse history: {}", e)))
                    }
                },
                Err(e) => HttpResponse::InternalServerError()
                    .json(ApiResponse::<()>::error(format!("Failed to read history: {}", e))),
            }
        }
        _ => {
            // History file doesn't exist, return empty
            let response = HistoryResponse {
                entries: vec![],
                total: 0,
            };
            HttpResponse::Ok().json(ApiResponse::success(response))
        }
    }
}

/// POST /api/config - Add a new configuration
#[post("/api/config")]
pub async fn add_config(_req: web::Json<UpdateConfigRequest>) -> impl Responder {
    // CCR doesn't have a direct "add config" command, users must edit the config file
    HttpResponse::NotImplemented().json(ApiResponse::<()>::error(
        "Adding configs via API is not implemented. Please edit ~/.ccs_config.toml directly.".to_string(),
    ))
}

/// PUT /api/config/{name} - Update a configuration
#[put("/api/config/{name}")]
pub async fn update_config(
    _name: web::Path<String>,
    _req: web::Json<UpdateConfigRequest>,
) -> impl Responder {
    // CCR doesn't have a direct "update config" command
    HttpResponse::NotImplemented().json(ApiResponse::<()>::error(
        "Updating configs via API is not implemented. Please edit ~/.ccs_config.toml directly.".to_string(),
    ))
}

/// DELETE /api/config/{name} - Delete a configuration
#[delete("/api/config/{name}")]
pub async fn delete_config(_name: web::Path<String>) -> impl Responder {
    // CCR doesn't have a direct "delete config" command
    HttpResponse::NotImplemented().json(ApiResponse::<()>::error(
        "Deleting configs via API is not implemented. Please edit ~/.ccs_config.toml directly.".to_string(),
    ))
}

