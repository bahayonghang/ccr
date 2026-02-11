use axum::Json;

use crate::api::handlers::response::ApiSuccess;
use crate::cache::GLOBAL_SETTINGS_CACHE;
use crate::core::error::{ApiError, ApiResult};
use crate::models::api::StatuslineConfig;

/// GET /api/statusline - Get current statusline configuration
pub async fn get_statusline() -> ApiResult<ApiSuccess<StatuslineConfig>> {
    let settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let statusline = settings
        .other
        .get("statusline")
        .and_then(|v| serde_json::from_value::<StatuslineConfig>(v.clone()).ok())
        .unwrap_or_default();

    Ok(ApiSuccess(statusline))
}

/// PUT /api/statusline - Update statusline configuration
pub async fn update_statusline(
    Json(config): Json<StatuslineConfig>,
) -> ApiResult<ApiSuccess<&'static str>> {
    let mut settings = GLOBAL_SETTINGS_CACHE
        .load()
        .map_err(|e| ApiError::internal(format!("Failed to load settings: {}", e)))?;

    let statusline_value = serde_json::to_value(&config)
        .map_err(|e| ApiError::internal(format!("Failed to serialize config: {}", e)))?;

    settings
        .other
        .insert("statusline".to_string(), statusline_value);

    GLOBAL_SETTINGS_CACHE
        .save_atomic(&settings)
        .map_err(|e| ApiError::internal(format!("Failed to save settings: {}", e)))?;

    Ok(ApiSuccess("Statusline configuration updated successfully"))
}
