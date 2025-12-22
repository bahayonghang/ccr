// 💰 成本追踪相关处理器
// 提供成本统计、预算管理、价格配置的 Web API

use crate::core::error::CcrError;
use crate::managers::{BudgetManager, CostTracker, PricingManager};
use crate::models::stats::{CostRecord, ModelPricing};
use crate::web::error_utils::{
    bad_request, empty_success_response, internal_server_error, spawn_blocking_string,
    success_response,
};
use crate::web::handlers::AppState;
use axum::{
    Json,
    extract::{Path, Query, State},
    response::Response,
};
use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================
// 📊 Stats API Handlers - 成本统计
// ============================================

/// 成本摘要响应
#[derive(Serialize)]
pub struct CostSummaryResponse {
    pub today: f64,
    pub this_week: f64,
    pub this_month: f64,
    pub total_entries: usize,
}

/// GET /api/stats/cost/summary
///
/// 获取成本摘要（今日/本周/本月）
pub async fn handle_get_cost_summary(State(_state): State<AppState>) -> Response {
    match spawn_blocking_string(|| {
        let storage_dir = CostTracker::default_storage_dir()?;
        let tracker = CostTracker::new(storage_dir)?;

        let today = tracker.get_today_cost()?;
        let this_week = tracker.get_week_cost()?;
        let this_month = tracker.get_month_cost()?;
        let total_entries = tracker.read_all()?.len();

        Ok(CostSummaryResponse {
            today,
            this_week,
            this_month,
            total_entries,
        })
    })
    .await
    {
        Ok(summary) => success_response(summary),
        Err(e) => internal_server_error(e),
    }
}

/// 查询参数：期间
#[derive(Deserialize)]
pub struct PeriodQuery {
    /// 期间 (YYYYMM 格式，如 202512)
    pub period: Option<String>,
}

/// GET /api/stats/cost/details?period=YYYYMM
///
/// 获取指定期间的详细成本记录
pub async fn handle_get_cost_details(Query(query): Query<PeriodQuery>) -> Response {
    match spawn_blocking_string(move || {
        let storage_dir = CostTracker::default_storage_dir()?;
        let tracker = CostTracker::new(storage_dir)?;

        let entries: Vec<CostRecord> = if let Some(period_str) = query.period {
            // 解析期间参数
            if period_str.len() != 6 {
                return Err(CcrError::ValidationError(
                    "期间格式错误，应为 YYYYMM (如 202512)".to_string(),
                ));
            }

            let year: i32 = period_str[0..4]
                .parse()
                .map_err(|_| CcrError::ValidationError("无效的年份格式".to_string()))?;
            let month: u32 = period_str[4..6]
                .parse()
                .map_err(|_| CcrError::ValidationError("无效的月份格式".to_string()))?;

            if !(1..=12).contains(&month) {
                return Err(CcrError::ValidationError(
                    "月份必须在 1-12 之间".to_string(),
                ));
            }

            get_entries_for_month(&tracker, year, month)?
        } else {
            // 未指定期间，返回当前月份
            let now = chrono::Local::now();
            get_entries_for_month(&tracker, now.year(), now.month())?
        };

        Ok(entries)
    })
    .await
    {
        Ok(entries) => success_response(entries),
        Err(e) => bad_request(e),
    }
}

/// GET /api/stats/cost/export?period=YYYYMM
///
/// 导出指定期间的成本数据为 CSV 格式
pub async fn handle_export_costs(Query(query): Query<PeriodQuery>) -> Response {
    match spawn_blocking_string(move || {
        let storage_dir = CostTracker::default_storage_dir()?;
        let tracker = CostTracker::new(storage_dir)?;

        if let Some(period_str) = query.period {
            // 解析期间参数
            if period_str.len() != 6 {
                return Err(CcrError::ValidationError(
                    "期间格式错误，应为 YYYYMM (如 202512)".to_string(),
                ));
            }

            let year: i32 = period_str[0..4]
                .parse()
                .map_err(|_| CcrError::ValidationError("无效的年份格式".to_string()))?;
            let month: u32 = period_str[4..6]
                .parse()
                .map_err(|_| CcrError::ValidationError("无效的月份格式".to_string()))?;

            if !(1..=12).contains(&month) {
                return Err(CcrError::ValidationError(
                    "月份必须在 1-12 之间".to_string(),
                ));
            }

            let csv_data = export_month_to_csv(&tracker, year, month)?;
            Ok(csv_data)
        } else {
            Err(CcrError::ValidationError(
                "必须指定 period 参数 (YYYYMM 格式)".to_string(),
            ))
        }
    })
    .await
    {
        Ok(csv) => success_response(serde_json::json!({ "csv_data": csv })),
        Err(e) => bad_request(e),
    }
}

/// 模型使用统计响应
#[derive(Serialize)]
pub struct ModelUsageItem {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub total_cost: f64,
    pub request_count: usize,
}

/// GET /api/stats/cost/by-model
///
/// 按模型统计使用情况（当前月份）
pub async fn handle_get_model_usage(State(_state): State<AppState>) -> Response {
    match spawn_blocking_string(|| {
        let storage_dir = CostTracker::default_storage_dir()?;
        let tracker = CostTracker::new(storage_dir)?;

        let now = chrono::Local::now();
        let entries = get_entries_for_month(&tracker, now.year(), now.month())?;

        // 按模型聚合统计
        let mut model_stats: HashMap<String, ModelUsageItem> = HashMap::new();

        for entry in entries {
            let stats = model_stats
                .entry(entry.model.clone())
                .or_insert_with(|| ModelUsageItem {
                    model: entry.model.clone(),
                    input_tokens: 0,
                    output_tokens: 0,
                    cache_read_tokens: 0,
                    cache_write_tokens: 0,
                    total_cost: 0.0,
                    request_count: 0,
                });

            stats.input_tokens += entry.token_usage.input_tokens as u64;
            stats.output_tokens += entry.token_usage.output_tokens as u64;
            stats.cache_read_tokens += entry.token_usage.cache_read_tokens.unwrap_or(0) as u64;
            stats.cache_write_tokens += entry.token_usage.cache_creation_tokens.unwrap_or(0) as u64;
            stats.total_cost += entry.cost.total_cost;
            stats.request_count += 1;
        }

        let mut result: Vec<ModelUsageItem> = model_stats.into_values().collect();
        // 按总成本降序排列
        result.sort_by(|a, b| {
            b.total_cost
                .partial_cmp(&a.total_cost)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(result)
    })
    .await
    {
        Ok(usage) => success_response(usage),
        Err(e) => internal_server_error(e),
    }
}

// ============================================
// 💰 Budget API Handlers - 预算管理
// ============================================

/// 预算状态响应（简化版）
#[derive(Serialize)]
pub struct BudgetStatusResponse {
    pub enabled: bool,
    pub daily_limit: Option<f64>,
    pub weekly_limit: Option<f64>,
    pub monthly_limit: Option<f64>,
    pub warn_threshold: u8,
    pub current_costs: CurrentCosts,
    pub warnings: Vec<BudgetWarning>,
}

#[derive(Serialize)]
pub struct CurrentCosts {
    pub today: f64,
    pub this_week: f64,
    pub this_month: f64,
}

#[derive(Serialize)]
pub struct BudgetWarning {
    pub period: String,
    pub current_cost: f64,
    pub limit: f64,
    pub usage_percent: f64,
}

/// GET /api/budget/status
///
/// 获取当前预算状态
pub async fn handle_get_budget_status(State(_state): State<AppState>) -> Response {
    match spawn_blocking_string(|| {
        let manager = BudgetManager::with_default()?;
        let storage_dir = CostTracker::default_storage_dir()?;
        let tracker = CostTracker::new(storage_dir)?;

        let status = manager.check_status(&tracker)?;
        let config = manager.get_config();

        // 转换为 Web API 响应格式
        let warnings: Vec<BudgetWarning> = status
            .warnings
            .iter()
            .map(|w| BudgetWarning {
                period: format!("{:?}", w.period),
                current_cost: w.current_cost,
                limit: w.limit,
                usage_percent: w.usage_percent,
            })
            .collect();

        Ok(BudgetStatusResponse {
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
        })
    })
    .await
    {
        Ok(status) => success_response(status),
        Err(e) => internal_server_error(e),
    }
}

/// 设置预算请求
#[derive(Deserialize)]
pub struct SetBudgetRequest {
    pub enabled: Option<bool>,
    pub daily_limit: Option<Option<f64>>,
    pub weekly_limit: Option<Option<f64>>,
    pub monthly_limit: Option<Option<f64>>,
    pub warn_threshold: Option<u8>,
}

/// POST /api/budget/set
///
/// 设置预算限制
pub async fn handle_set_budget(
    State(_state): State<AppState>,
    Json(req): Json<SetBudgetRequest>,
) -> Response {
    match spawn_blocking_string(move || {
        let mut manager = BudgetManager::with_default()?;

        if let Some(enabled) = req.enabled {
            if enabled {
                manager.enable()?;
            } else {
                manager.disable()?;
            }
        }

        if let Some(daily) = req.daily_limit {
            manager.set_daily_limit(daily)?;
        }

        if let Some(weekly) = req.weekly_limit {
            manager.set_weekly_limit(weekly)?;
        }

        if let Some(monthly) = req.monthly_limit {
            manager.set_monthly_limit(monthly)?;
        }

        if let Some(threshold) = req.warn_threshold {
            manager.set_warn_threshold(threshold)?;
        }

        Ok(())
    })
    .await
    {
        Ok(()) => empty_success_response(),
        Err(e) => internal_server_error(e),
    }
}

/// POST /api/budget/reset
///
/// 重置所有预算限制（保留启用状态）
pub async fn handle_reset_budget(State(_state): State<AppState>) -> Response {
    match spawn_blocking_string(|| {
        let mut manager = BudgetManager::with_default()?;
        manager.reset_limits()?;
        Ok(())
    })
    .await
    {
        Ok(()) => empty_success_response(),
        Err(e) => internal_server_error(e),
    }
}

// ============================================
// 💲 Pricing API Handlers - 价格管理
// ============================================

/// GET /api/pricing/list
///
/// 列出所有模型定价配置
pub async fn handle_list_pricing(State(_state): State<AppState>) -> Response {
    match spawn_blocking_string(|| {
        let manager = PricingManager::with_default()?;
        let config = manager.get_config();

        let mut pricings = Vec::new();
        for model_name in manager.model_names() {
            if let Some(pricing) = manager.get_pricing(&model_name) {
                pricings.push(pricing.clone());
            }
        }

        Ok(serde_json::json!({
            "pricings": pricings,
            "default_pricing": config.default_pricing,
        }))
    })
    .await
    {
        Ok(data) => success_response(data),
        Err(e) => internal_server_error(e),
    }
}

/// 设置定价请求
#[derive(Deserialize)]
pub struct SetPricingRequest {
    pub model: String,
    pub input_price: f64,
    pub output_price: f64,
    pub cache_read_price: Option<f64>,
    pub cache_write_price: Option<f64>,
}

/// POST /api/pricing/set
///
/// 设置模型定价
pub async fn handle_set_pricing(
    State(_state): State<AppState>,
    Json(req): Json<SetPricingRequest>,
) -> Response {
    // 验证价格为正数
    if req.input_price < 0.0 || req.output_price < 0.0 {
        return bad_request("定价不能为负数");
    }

    if let Some(cache_read) = req.cache_read_price
        && cache_read < 0.0
    {
        return bad_request("缓存读取价格不能为负数");
    }

    if let Some(cache_write) = req.cache_write_price
        && cache_write < 0.0
    {
        return bad_request("缓存写入价格不能为负数");
    }

    match spawn_blocking_string(move || {
        let mut manager = PricingManager::with_default()?;

        let pricing = ModelPricing {
            model: req.model.clone(),
            input_price: req.input_price,
            output_price: req.output_price,
            cache_read_price: req.cache_read_price,
            cache_write_price: req.cache_write_price,
        };

        manager.set_pricing(req.model, pricing)?;
        Ok(())
    })
    .await
    {
        Ok(()) => empty_success_response(),
        Err(e) => internal_server_error(e),
    }
}

/// DELETE /api/pricing/remove/{model}
///
/// 移除模型定价配置
pub async fn handle_remove_pricing(
    State(_state): State<AppState>,
    Path(model): Path<String>,
) -> Response {
    match spawn_blocking_string(move || {
        let mut manager = PricingManager::with_default()?;
        manager.remove_pricing(&model)?;
        Ok(())
    })
    .await
    {
        Ok(()) => empty_success_response(),
        Err(e) => internal_server_error(e),
    }
}

/// POST /api/pricing/reset
///
/// 重置为默认定价配置
pub async fn handle_reset_pricing(State(_state): State<AppState>) -> Response {
    match spawn_blocking_string(|| {
        let mut manager = PricingManager::with_default()?;
        manager.reset_to_defaults()?;
        Ok(())
    })
    .await
    {
        Ok(()) => empty_success_response(),
        Err(e) => internal_server_error(e),
    }
}

// ============================================
// 🛠️ Helper Functions - 辅助函数
// ============================================

/// 获取指定月份的所有成本记录
fn get_entries_for_month(
    tracker: &CostTracker,
    year: i32,
    month: u32,
) -> Result<Vec<CostRecord>, CcrError> {
    // 计算月份的起始和结束时间
    let start_date = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| CcrError::ValidationError(format!("无效的日期: {}-{}", year, month)))?;

    let end_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(|| CcrError::ValidationError(format!("无法计算月末日期: {}-{}", year, month)))?;

    let start = Utc
        .from_local_datetime(&start_date.and_hms_opt(0, 0, 0).expect("无效的日期时间"))
        .single()
        .ok_or_else(|| CcrError::ValidationError("无效的起始时间".to_string()))?;

    let end = Utc
        .from_local_datetime(&end_date.and_hms_opt(0, 0, 0).expect("无效的日期时间"))
        .single()
        .ok_or_else(|| CcrError::ValidationError("无效的结束时间".to_string()))?;

    tracker.read_by_time_range(start, end)
}

/// 导出月份成本数据为 CSV 字符串
fn export_month_to_csv(tracker: &CostTracker, year: i32, month: u32) -> Result<String, CcrError> {
    let entries = get_entries_for_month(tracker, year, month)?;

    // 生成 CSV 字符串
    let mut csv = String::from(
        "timestamp,session_id,model,input_tokens,output_tokens,cache_read_tokens,cache_write_tokens,cost\n",
    );

    for entry in entries {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{:.6}\n",
            entry.timestamp.to_rfc3339(),
            entry.session_id.as_deref().unwrap_or(""),
            entry.model,
            entry.token_usage.input_tokens,
            entry.token_usage.output_tokens,
            entry.token_usage.cache_read_tokens.unwrap_or(0),
            entry.token_usage.cache_creation_tokens.unwrap_or(0),
            entry.cost.total_cost
        ));
    }

    Ok(csv)
}
