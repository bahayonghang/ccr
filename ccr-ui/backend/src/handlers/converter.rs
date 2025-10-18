// 配置转换器 API 处理器

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::config_converter::ConfigConverter;
use crate::converter_models::ConverterRequest;

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
