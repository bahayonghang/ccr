//! Provider Health 路由
//!
//! 提供 Provider 健康检查的 API 端点

use crate::state::AppState;
use axum::{Json, Router, routing::get, routing::post};
use serde::{Deserialize, Serialize};

/// 创建 provider-health 路由
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/provider-health/test", post(test_provider))
        .route("/provider-health/test-all", get(test_all_providers))
}

/// 健康检查请求
#[derive(Debug, Deserialize)]
pub struct HealthCheckRequest {
    pub name: String,
}

/// 健康检查结果
#[derive(Debug, Serialize)]
pub struct HealthCheckResponse {
    pub provider_name: String,
    pub base_url: String,
    pub status: String,
    pub status_color: String,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
    pub model_available: bool,
    pub available_models: Vec<String>,
}

/// 批量检查结果
#[derive(Debug, Serialize)]
pub struct BatchHealthCheckResponse {
    pub results: Vec<HealthCheckResponse>,
    pub summary: HealthSummary,
}

/// 健康状态摘要
#[derive(Debug, Serialize)]
pub struct HealthSummary {
    pub total: usize,
    pub healthy: usize,
    pub degraded: usize,
    pub unhealthy: usize,
}

/// 测试单个 Provider
async fn test_provider(Json(request): Json<HealthCheckRequest>) -> Json<HealthCheckResponse> {
    use ccr::managers::config::ConfigSection;
    use ccr::services::ConfigService;
    use ccr::services::health_check::{HealthCheckService, HealthStatus};

    let config_service = match ConfigService::with_default() {
        Ok(s) => s,
        Err(e) => {
            return Json(HealthCheckResponse {
                provider_name: request.name,
                base_url: String::new(),
                status: "unknown".to_string(),
                status_color: "gray".to_string(),
                latency_ms: None,
                error: Some(format!("无法加载配置: {}", e)),
                model_available: false,
                available_models: vec![],
            });
        }
    };

    let config_list = match config_service.list_configs() {
        Ok(l) => l,
        Err(e) => {
            return Json(HealthCheckResponse {
                provider_name: request.name,
                base_url: String::new(),
                status: "unknown".to_string(),
                status_color: "gray".to_string(),
                latency_ms: None,
                error: Some(format!("无法获取配置列表: {}", e)),
                model_available: false,
                available_models: vec![],
            });
        }
    };

    let config = config_list.configs.iter().find(|c| c.name == request.name);

    match config {
        Some(c) => {
            let service = HealthCheckService::new();

            let section = ConfigSection {
                auth_token: c.auth_token.clone(),
                base_url: c.base_url.clone(),
                model: c.model.clone(),
                ..Default::default()
            };

            let result = service.check(&request.name, &section).await;

            Json(HealthCheckResponse {
                provider_name: result.provider_name,
                base_url: result.base_url,
                status: match result.status {
                    HealthStatus::Healthy => "healthy".to_string(),
                    HealthStatus::Degraded => "degraded".to_string(),
                    HealthStatus::Unhealthy => "unhealthy".to_string(),
                    HealthStatus::Unknown => "unknown".to_string(),
                },
                status_color: result.status.color().to_string(),
                latency_ms: result.latency_ms,
                error: result.error,
                model_available: result.model_available,
                available_models: result.available_models,
            })
        }
        None => Json(HealthCheckResponse {
            provider_name: request.name,
            base_url: String::new(),
            status: "unknown".to_string(),
            status_color: "gray".to_string(),
            latency_ms: None,
            error: Some("配置不存在".to_string()),
            model_available: false,
            available_models: vec![],
        }),
    }
}

/// 测试所有 Provider
async fn test_all_providers() -> Json<BatchHealthCheckResponse> {
    use ccr::managers::config::ConfigSection;
    use ccr::services::ConfigService;
    use ccr::services::health_check::{HealthCheckService, HealthStatus};

    let config_service = match ConfigService::with_default() {
        Ok(s) => s,
        Err(_) => {
            return Json(BatchHealthCheckResponse {
                results: vec![],
                summary: HealthSummary {
                    total: 0,
                    healthy: 0,
                    degraded: 0,
                    unhealthy: 0,
                },
            });
        }
    };

    let config_list = match config_service.list_configs() {
        Ok(l) => l,
        Err(_) => {
            return Json(BatchHealthCheckResponse {
                results: vec![],
                summary: HealthSummary {
                    total: 0,
                    healthy: 0,
                    degraded: 0,
                    unhealthy: 0,
                },
            });
        }
    };

    let service = HealthCheckService::new();
    let mut results = Vec::new();
    let mut healthy = 0;
    let mut degraded = 0;
    let mut unhealthy = 0;

    for config in &config_list.configs {
        let section = ConfigSection {
            auth_token: config.auth_token.clone(),
            base_url: config.base_url.clone(),
            model: config.model.clone(),
            ..Default::default()
        };

        let result = service.check(&config.name, &section).await;

        match result.status {
            HealthStatus::Healthy => healthy += 1,
            HealthStatus::Degraded => degraded += 1,
            HealthStatus::Unhealthy => unhealthy += 1,
            _ => {}
        }

        results.push(HealthCheckResponse {
            provider_name: result.provider_name,
            base_url: result.base_url,
            status: match result.status {
                HealthStatus::Healthy => "healthy".to_string(),
                HealthStatus::Degraded => "degraded".to_string(),
                HealthStatus::Unhealthy => "unhealthy".to_string(),
                HealthStatus::Unknown => "unknown".to_string(),
            },
            status_color: result.status.color().to_string(),
            latency_ms: result.latency_ms,
            error: result.error,
            model_available: result.model_available,
            available_models: result.available_models,
        });
    }

    Json(BatchHealthCheckResponse {
        summary: HealthSummary {
            total: results.len(),
            healthy,
            degraded,
            unhealthy,
        },
        results,
    })
}
