// 配置转换器 API 处理器

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;

use crate::models::converter::ConverterRequest;
use crate::services::converter_service::ConfigConverter;

/// POST /api/converter/convert - 执行配置转换
pub async fn convert_config(Json(request): Json<ConverterRequest>) -> impl IntoResponse {
    match ConfigConverter::convert(request) {
        Ok(response) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "data": response,
                "message": null
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "data": null,
                "message": format!("配置转换失败: {}", e)
            })),
        )
            .into_response(),
    }
}
