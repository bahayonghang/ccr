use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::cache::GLOBAL_SETTINGS_CACHE;
use crate::models::api::StatuslineConfig;

/// GET /api/statusline - Get current statusline configuration
pub async fn get_statusline() -> impl IntoResponse {
    let settings_result = GLOBAL_SETTINGS_CACHE.load();

    if let Ok(settings) = settings_result {
        let statusline = settings
            .other
            .get("statusline")
            .and_then(|v| serde_json::from_value::<StatuslineConfig>(v.clone()).ok())
            .unwrap_or_default();

        return (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": statusline,
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
            "message": "Failed to load statusline configuration"
        })),
    )
        .into_response()
}

/// PUT /api/statusline - Update statusline configuration
pub async fn update_statusline(Json(config): Json<StatuslineConfig>) -> impl IntoResponse {
    if let Ok(mut settings) = GLOBAL_SETTINGS_CACHE.load() {
        let statusline_value = serde_json::to_value(&config).unwrap_or(serde_json::Value::Null);
        settings
            .other
            .insert("statusline".to_string(), statusline_value);

        if GLOBAL_SETTINGS_CACHE.save_atomic(&settings).is_ok() {
            return (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "data": "Statusline configuration updated successfully",
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
            "message": "Failed to update statusline configuration"
        })),
    )
        .into_response()
}
