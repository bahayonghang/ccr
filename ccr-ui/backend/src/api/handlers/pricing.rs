// 💲 价格管理 API 处理器
// 提供模型定价配置的 Web API

use axum::{
    extract::{Json, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use ccr::managers::PricingManager;
use ccr::models::stats::ModelPricing;
use serde::{Deserialize, Serialize};

// ============================================================
// 请求/响应类型
// ============================================================

/// 定价列表响应
#[derive(Debug, Serialize)]
pub struct PricingListResponse {
    pub pricings: Vec<ModelPricing>,
    pub default_pricing: ModelPricing,
}

/// 设置定价请求
#[derive(Debug, Deserialize)]
pub struct SetPricingRequest {
    pub model: String,
    pub input_price: f64,
    pub output_price: f64,
    pub cache_read_price: Option<f64>,
    pub cache_write_price: Option<f64>,
}

// ============================================================
// API 处理器
// ============================================================

/// GET /api/pricing/list - 列出所有模型定价
pub async fn get_pricing_list() -> Result<Json<PricingListResponse>, Response> {
    let manager = PricingManager::with_default().map_err(internal_error)?;
    let config = manager.get_config();

    // 收集所有模型的定价
    let mut pricings = Vec::new();
    for model_name in manager.model_names() {
        if let Some(pricing) = manager.get_pricing(&model_name) {
            pricings.push(pricing.clone());
        }
    }

    // 按模型名称排序
    pricings.sort_by(|a, b| a.model.cmp(&b.model));

    let response = PricingListResponse {
        pricings,
        default_pricing: config
            .default_pricing
            .clone()
            .unwrap_or_else(|| ModelPricing {
                model: "default".to_string(),
                input_price: 0.003,
                output_price: 0.015,
                cache_read_price: Some(0.0003),
                cache_write_price: Some(0.00375),
            }),
    };

    Ok(Json(response))
}

/// POST /api/pricing/set - 设置模型定价
pub async fn set_pricing(Json(req): Json<SetPricingRequest>) -> Result<StatusCode, Response> {
    // 验证价格为正数
    if req.input_price < 0.0 || req.output_price < 0.0 {
        return Err((StatusCode::BAD_REQUEST, "价格不能为负数".to_string()).into_response());
    }

    if let Some(cache_read) = req.cache_read_price
        && cache_read < 0.0
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "缓存读取价格不能为负数".to_string(),
        )
            .into_response());
    }

    if let Some(cache_write) = req.cache_write_price
        && cache_write < 0.0
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "缓存写入价格不能为负数".to_string(),
        )
            .into_response());
    }

    let mut manager = PricingManager::with_default().map_err(internal_error)?;

    let pricing = ModelPricing {
        model: req.model.clone(),
        input_price: req.input_price,
        output_price: req.output_price,
        cache_read_price: req.cache_read_price,
        cache_write_price: req.cache_write_price,
    };

    manager
        .set_pricing(req.model, pricing)
        .map_err(internal_error)?;

    Ok(StatusCode::OK)
}

/// DELETE /api/pricing/remove/{model} - 删除模型定价
pub async fn remove_pricing(Path(model): Path<String>) -> Result<StatusCode, Response> {
    let mut manager = PricingManager::with_default().map_err(internal_error)?;
    manager.remove_pricing(&model).map_err(internal_error)?;

    Ok(StatusCode::OK)
}

/// POST /api/pricing/reset - 重置为默认定价
pub async fn reset_pricing() -> Result<StatusCode, Response> {
    let mut manager = PricingManager::with_default().map_err(internal_error)?;
    manager.reset_to_defaults().map_err(internal_error)?;

    Ok(StatusCode::OK)
}

// ============================================================
// 辅助函数
// ============================================================

/// 将错误转换为内部服务器错误响应
fn internal_error<E: std::fmt::Display>(err: E) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal server error: {}", err),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_pricing_request_deserialization() {
        let json = r#"{
            "model": "claude-sonnet-4-5",
            "input_price": 0.003,
            "output_price": 0.015,
            "cache_read_price": 0.0003,
            "cache_write_price": 0.00375
        }"#;

        let req: SetPricingRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.model, "claude-sonnet-4-5");
        assert_eq!(req.input_price, 0.003);
        assert_eq!(req.output_price, 0.015);
        assert_eq!(req.cache_read_price, Some(0.0003));
        assert_eq!(req.cache_write_price, Some(0.00375));
    }

    #[test]
    fn test_pricing_list_response_serialization() {
        let default_pricing = ModelPricing {
            model: "default".to_string(),
            input_price: 0.001,
            output_price: 0.002,
            cache_read_price: None,
            cache_write_price: None,
        };

        let response = PricingListResponse {
            pricings: vec![default_pricing.clone()],
            default_pricing,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("default"));
        assert!(json.contains("0.001"));
    }
}
