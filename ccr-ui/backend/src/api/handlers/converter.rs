// 配置转换器 API 处理器

use axum::Json;

use crate::api::handlers::response::ApiSuccess;
use crate::core::error::{ApiError, ApiResult};
use crate::models::converter::{ConverterRequest, ConverterResponse};
use crate::services::converter_service::ConfigConverter;

/// POST /api/converter/convert - 执行配置转换
pub async fn convert_config(
    Json(request): Json<ConverterRequest>,
) -> ApiResult<ApiSuccess<ConverterResponse>> {
    let response = ConfigConverter::convert(request)
        .map_err(|e| ApiError::bad_request(format!("配置转换失败: {}", e)))?;

    Ok(ApiSuccess(response))
}
