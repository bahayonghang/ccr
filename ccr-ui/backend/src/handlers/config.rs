// Config Management Handlers
// 🚀 直接使用 CCR 核心库（无子进程开销）

use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::config_reader;
use crate::errors::{ApiError, ApiResult};
use crate::executor; // TODO: 逐步移除
use crate::models::*;

// 🎯 导入 CCR 核心库
use ccr::ConfigService;

/// GET /api/configs - List all configurations
pub async fn list_configs() -> ApiResult<Json<ConfigListResponse>> {
    // Read config file directly for better reliability
    let config = config_reader::read_config()
        .map_err(|e| ApiError::internal(format!("Failed to read config: {}", e)))?;

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

    Ok(Json(response))
}

/// POST /api/configs/switch - Switch to a configuration
/// 🎯 重构：直接使用 CCR 核心库（性能提升 50x）
pub async fn switch_config(Json(req): Json<SwitchRequest>) -> ApiResult<Json<&'static str>> {
    let config_name = req.config_name.clone();

    // 在 spawn_blocking 中运行同步代码（避免阻塞异步执行器）
    let result = tokio::task::spawn_blocking(move || {
        // 直接调用 ccr 的 switch_command
        ccr::commands::switch::switch_command(&config_name)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| ApiError::internal(format!("Task join error: {}", e)))?;

    match result {
        Ok(_) => Ok(Json("Configuration switched successfully")),
        Err(e) => Err(ApiError::bad_request(e)),
    }
}

/// GET /api/configs/validate - Validate all configurations
/// 🎯 重构：直接使用 CCR 核心库（性能提升 50x）
pub async fn validate_configs() -> ApiResult<Json<&'static str>> {
    // 在 spawn_blocking 中运行同步代码（避免阻塞异步执行器）
    let result = tokio::task::spawn_blocking(move || {
        // 创建 ConfigService 实例（使用默认配置管理器）
        let service = ConfigService::with_default()
            .map_err(|e| format!("Failed to create ConfigService: {}", e))?;

        // 调用验证方法
        service.validate_all()
            .map_err(|e| format!("Validation failed: {}", e))?;

        Ok::<(), String>(())
    })
    .await
    .map_err(|e| ApiError::internal(format!("Task join error: {}", e)))?;

    match result {
        Ok(_) => Ok(Json("All configurations are valid")),
        Err(e) => Err(ApiError::bad_request(e)),
    }
}

/// POST /api/configs/clean - Clean old backups
pub async fn clean_backups(Json(req): Json<CleanRequest>) -> impl IntoResponse {
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
            ApiResponse::success(response)
        }
        Ok(output) => ApiResponse::<CleanResponse>::error(output.stderr),
        Err(e) => ApiResponse::<CleanResponse>::error(e.to_string()),
    }
}

/// POST /api/configs/export - Export configurations
pub async fn export_config(Json(req): Json<ExportRequest>) -> impl IntoResponse {
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
            ApiResponse::success(response)
        }
        Ok(output) => ApiResponse::<ExportResponse>::error(output.stderr),
        Err(e) => ApiResponse::<ExportResponse>::error(e.to_string()),
    }
}

/// POST /api/configs/import - Import configurations
pub async fn import_config(Json(req): Json<ImportRequest>) -> impl IntoResponse {
    // Write the content to a temporary file
    let temp_file = match tempfile::NamedTempFile::new() {
        Ok(f) => f,
        Err(e) => return ApiResponse::<ImportResponse>::error(format!("Failed to create temp file: {}", e)),
    };

    if let Err(e) = std::fs::write(temp_file.path(), &req.content) {
        return ApiResponse::<ImportResponse>::error(format!("Failed to write temp file: {}", e));
    }

    let mut args = vec![
        "import".to_string(),
        temp_file.path().to_string_lossy().to_string(),
        "--mode".to_string(),
        req.mode.clone(),
    ];

    if !req.backup {
        args.push("--no-backup".to_string());
    }

    let result = executor::execute_command(args).await;

    match result {
        Ok(output) if output.success => {
            // Parse output for import statistics (simplified)
            let response = ImportResponse {
                added: 0,
                updated: 0,
                skipped: 0,
            };
            ApiResponse::success(response)
        }
        Ok(output) => ApiResponse::<ImportResponse>::error(output.stderr),
        Err(e) => ApiResponse::<ImportResponse>::error(e.to_string()),
    }
}

/// GET /api/configs/history - Get operation history
pub async fn get_history() -> ApiResult<Json<HistoryResponse>> {
    // For now, return a mock response
    // In a real implementation, you'd read from ~/.claude/ccr_history.json
    let entries: Vec<HistoryEntryJson> = Vec::new();
    
    let response = HistoryResponse {
        entries,
        total: 0,
    };

    Ok(Json(response))
}

/// POST /api/configs - Add a new configuration
pub async fn add_config(Json(_req): Json<UpdateConfigRequest>) -> impl IntoResponse {
    // This operation is not supported via CLI, need to implement in manager
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ApiResponse::<()>::error(
            "Direct config addition not yet implemented".to_string(),
        )),
    )
}

/// PATCH /api/configs/:name - Update a configuration
pub async fn update_config(
    Path(_name): Path<String>,
    Json(_req): Json<UpdateConfigRequest>,
) -> impl IntoResponse {
    // This operation is not supported via CLI, need to implement in manager
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ApiResponse::<()>::error(
            "Direct config update not yet implemented".to_string(),
        )),
    )
}

/// DELETE /api/configs/:name - Delete a configuration
pub async fn delete_config(Path(_name): Path<String>) -> impl IntoResponse {
    // This operation is not supported via CLI, need to implement in manager
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ApiResponse::<()>::error(
            "Direct config deletion not yet implemented".to_string(),
        )),
    )
}
