// 💰 预算管理 API 处理器
// 提供预算配置和监控的 Web API

use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use ccr::managers::{BudgetManager, CostTracker};
use ccr::models::budget::BudgetPeriod;
use serde::{Deserialize, Serialize};

// ============================================================
// 请求/响应类型
// ============================================================

/// 预算状态响应
#[derive(Debug, Serialize)]
pub struct BudgetStatusResponse {
    pub enabled: bool,
    pub daily_limit: Option<f64>,
    pub weekly_limit: Option<f64>,
    pub monthly_limit: Option<f64>,
    pub warn_threshold: u8,
    pub current_costs: CurrentCosts,
    pub warnings: Vec<BudgetWarning>,
    pub last_updated: String,
}

/// 当前成本
#[derive(Debug, Serialize)]
pub struct CurrentCosts {
    pub today: f64,
    pub this_week: f64,
    pub this_month: f64,
}

/// 预算警告
#[derive(Debug, Serialize)]
pub struct BudgetWarning {
    pub period: String,
    pub current_cost: f64,
    pub limit: f64,
    pub usage_percent: f64,
}

/// 设置预算请求
#[derive(Debug, Deserialize)]
pub struct SetBudgetRequest {
    pub enabled: Option<bool>,
    pub daily_limit: Option<Option<f64>>,
    pub weekly_limit: Option<Option<f64>>,
    pub monthly_limit: Option<Option<f64>>,
    pub warn_threshold: Option<u8>,
}

// ============================================================
// API 处理器
// ============================================================

/// GET /api/budget/status - 获取预算状态
pub async fn get_budget_status() -> Result<Json<BudgetStatusResponse>, Response> {
    // 加载预算管理器和成本追踪器
    let budget_manager = BudgetManager::with_default().map_err(internal_error)?;
    let storage_dir = CostTracker::default_storage_dir().map_err(internal_error)?;
    let tracker = CostTracker::new(storage_dir).map_err(internal_error)?;

    // 获取预算状态
    let status = budget_manager
        .check_status(&tracker)
        .map_err(internal_error)?;
    let config = budget_manager.get_config();

    // 转换警告
    let warnings: Vec<BudgetWarning> = status
        .warnings
        .iter()
        .map(|w| {
            let period_str = match w.period {
                BudgetPeriod::Daily => "daily",
                BudgetPeriod::Weekly => "weekly",
                BudgetPeriod::Monthly => "monthly",
            };

            BudgetWarning {
                period: period_str.to_string(),
                current_cost: w.current_cost,
                limit: w.limit,
                usage_percent: w.usage_percent,
            }
        })
        .collect();

    // 构建响应
    let response = BudgetStatusResponse {
        enabled: status.enabled,
        daily_limit: status.limits.daily,
        weekly_limit: status.limits.weekly,
        monthly_limit: status.limits.monthly,
        warn_threshold: config.warn_at_percent,
        current_costs: CurrentCosts {
            today: status.current_costs.today,
            this_week: status.current_costs.this_week,
            this_month: status.current_costs.this_month,
        },
        warnings,
        last_updated: status.last_updated.to_rfc3339(),
    };

    Ok(Json(response))
}

/// POST /api/budget/set - 设置预算
pub async fn set_budget(Json(req): Json<SetBudgetRequest>) -> Result<StatusCode, Response> {
    let mut manager = BudgetManager::with_default().map_err(internal_error)?;

    // 启用/禁用
    if let Some(enabled) = req.enabled {
        if enabled {
            manager.enable().map_err(internal_error)?;
        } else {
            manager.disable().map_err(internal_error)?;
        }
    }

    // 设置限制
    if let Some(daily) = req.daily_limit {
        manager.set_daily_limit(daily).map_err(internal_error)?;
    }

    if let Some(weekly) = req.weekly_limit {
        manager.set_weekly_limit(weekly).map_err(internal_error)?;
    }

    if let Some(monthly) = req.monthly_limit {
        manager.set_monthly_limit(monthly).map_err(internal_error)?;
    }

    if let Some(threshold) = req.warn_threshold {
        manager
            .set_warn_threshold(threshold)
            .map_err(internal_error)?;
    }

    Ok(StatusCode::OK)
}

/// POST /api/budget/reset - 重置预算限制
pub async fn reset_budget() -> Result<StatusCode, Response> {
    let mut manager = BudgetManager::with_default().map_err(internal_error)?;
    manager.reset_limits().map_err(internal_error)?;

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
    fn test_budget_warning_serialization() {
        let warning = BudgetWarning {
            period: "daily".to_string(),
            current_cost: 10.5,
            limit: 20.0,
            usage_percent: 52.5,
        };

        let json = serde_json::to_string(&warning).unwrap();
        assert!(json.contains("daily"));
        assert!(json.contains("10.5"));
    }

    #[test]
    fn test_set_budget_request_deserialization() {
        let json = r#"{
            "enabled": true,
            "daily_limit": 10.0,
            "warn_threshold": 90
        }"#;

        let req: SetBudgetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.enabled, Some(true));
        assert_eq!(req.daily_limit, Some(Some(10.0)));
        assert_eq!(req.warn_threshold, Some(90));
    }
}
