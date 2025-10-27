// 📊 统计 API 处理器
// 提供成本和使用统计的 Web API

use crate::executor;
use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 请求/响应类型
// ============================================================

/// 时间范围查询参数
#[derive(Debug, Deserialize)]
pub struct TimeRangeQuery {
    /// 范围: today, week, month
    #[serde(default = "default_range")]
    pub range: String,
}

fn default_range() -> String {
    "today".to_string()
}

/// 成本统计响应
#[derive(Debug, Serialize)]
pub struct CostStatsResponse {
    pub total_cost: f64,
    pub record_count: usize,
    pub token_stats: TokenStatsResponse,
    pub by_model: HashMap<String, f64>,
    pub by_project: HashMap<String, f64>,
    pub trend: Option<Vec<DailyCostResponse>>,
}

/// Token 统计
#[derive(Debug, Serialize)]
pub struct TokenStatsResponse {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_tokens: u64,
    pub cache_efficiency: f64,
}

/// 每日成本
#[derive(Debug, Serialize)]
pub struct DailyCostResponse {
    pub date: String,
    pub cost: f64,
    pub count: usize,
}

/// 顶级会话
#[derive(Debug, Serialize)]
pub struct TopSessionResponse {
    pub session_id: String,
    pub cost: f64,
}

/// Top N 查询参数
#[derive(Debug, Deserialize)]
pub struct TopNQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    10
}

// ============================================================
// API 处理器
// ============================================================

/// GET /api/stats/cost - 成本概览
pub async fn cost_overview(
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<CostStatsResponse>, StatusCode> {
    // 执行 CCR CLI 命令获取统计
    let args = vec![
        "stats".to_string(),
        "cost".to_string(),
        "--range".to_string(),
        params.range,
        "--details".to_string(),
        "--export".to_string(),
        "/tmp/ccr_stats.json".to_string(),
    ];

    let output = executor::execute_command(args)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !output.success {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // 读取导出的 JSON 文件
    let stats_json = tokio::fs::read_to_string("/tmp/ccr_stats.json")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stats: serde_json::Value = serde_json::from_str(&stats_json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 转换为响应格式
    let response = CostStatsResponse {
        total_cost: stats["total_cost"].as_f64().unwrap_or(0.0),
        record_count: stats["record_count"].as_u64().unwrap_or(0) as usize,
        token_stats: TokenStatsResponse {
            total_input_tokens: stats["token_stats"]["total_input_tokens"]
                .as_u64()
                .unwrap_or(0),
            total_output_tokens: stats["token_stats"]["total_output_tokens"]
                .as_u64()
                .unwrap_or(0),
            total_cache_tokens: stats["token_stats"]["total_cache_tokens"]
                .as_u64()
                .unwrap_or(0),
            cache_efficiency: stats["token_stats"]["cache_efficiency"]
                .as_f64()
                .unwrap_or(0.0),
        },
        by_model: parse_hashmap(&stats["by_model"]),
        by_project: parse_hashmap(&stats["by_project"]),
        trend: parse_trend(&stats["trend"]),
    };

    // 清理临时文件
    let _ = tokio::fs::remove_file("/tmp/ccr_stats.json").await;

    Ok(Json(response))
}

/// GET /api/stats/cost/today - 今日成本
pub async fn cost_today() -> Result<Json<CostStatsResponse>, StatusCode> {
    cost_overview(Query(TimeRangeQuery {
        range: "today".to_string(),
    }))
    .await
}

/// GET /api/stats/cost/week - 本周成本
pub async fn cost_week() -> Result<Json<CostStatsResponse>, StatusCode> {
    cost_overview(Query(TimeRangeQuery {
        range: "week".to_string(),
    }))
    .await
}

/// GET /api/stats/cost/month - 本月成本
pub async fn cost_month() -> Result<Json<CostStatsResponse>, StatusCode> {
    cost_overview(Query(TimeRangeQuery {
        range: "month".to_string(),
    }))
    .await
}

/// GET /api/stats/cost/trend - 成本趋势
pub async fn cost_trend(
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<Vec<DailyCostResponse>>, StatusCode> {
    let result = cost_overview(Query(params)).await?;
    Ok(Json(result.0.trend.unwrap_or_default()))
}

/// GET /api/stats/cost/by-model - 按模型分组
pub async fn cost_by_model(
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<HashMap<String, f64>>, StatusCode> {
    let result = cost_overview(Query(params)).await?;
    Ok(Json(result.0.by_model))
}

/// GET /api/stats/cost/by-project - 按项目分组
pub async fn cost_by_project(
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<HashMap<String, f64>>, StatusCode> {
    let result = cost_overview(Query(params)).await?;
    Ok(Json(result.0.by_project))
}

/// GET /api/stats/cost/top-sessions - 成本最高的会话
pub async fn cost_top_sessions(
    Query(params): Query<TopNQuery>,
) -> Result<Json<Vec<TopSessionResponse>>, StatusCode> {
    let args = vec![
        "stats".to_string(),
        "cost".to_string(),
        "--top".to_string(),
        params.limit.to_string(),
        "--export".to_string(),
        "/tmp/ccr_stats_top.json".to_string(),
    ];

    let output = executor::execute_command(args)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !output.success {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // 从输出解析 top sessions
    // 临时实现：返回空列表
    // TODO: 改进 CLI 输出格式以便解析
    Ok(Json(vec![]))
}

/// GET /api/stats/summary - 快速摘要
pub async fn stats_summary() -> impl IntoResponse {
    #[derive(Serialize)]
    struct Summary {
        today_cost: f64,
        week_cost: f64,
        month_cost: f64,
        total_sessions: usize,
    }

    // 并行获取今日、本周、本月成本
    let today = cost_today().await;
    let week = cost_week().await;
    let month = cost_month().await;

    if let (Ok(today), Ok(week), Ok(month)) = (today, week, month) {
        let summary = Summary {
            today_cost: today.0.total_cost,
            week_cost: week.0.total_cost,
            month_cost: month.0.total_cost,
            total_sessions: month.0.record_count,
        };
        (StatusCode::OK, Json(summary)).into_response()
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch statistics",
        )
            .into_response()
    }
}

// ============================================================
// 辅助函数
// ============================================================

/// 解析 HashMap 从 JSON
fn parse_hashmap(value: &serde_json::Value) -> HashMap<String, f64> {
    if let Some(obj) = value.as_object() {
        obj.iter()
            .filter_map(|(k, v)| {
                let cost = v.as_f64()?;
                Some((k.clone(), cost))
            })
            .collect()
    } else {
        HashMap::new()
    }
}

/// 解析趋势数据
fn parse_trend(value: &serde_json::Value) -> Option<Vec<DailyCostResponse>> {
    if let Some(arr) = value.as_array() {
        let trend: Vec<DailyCostResponse> = arr
            .iter()
            .filter_map(|item| {
                Some(DailyCostResponse {
                    date: item["date"].as_str()?.to_string(),
                    cost: item["cost"].as_f64()?,
                    count: item["count"].as_u64()? as usize,
                })
            })
            .collect();
        Some(trend)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hashmap() {
        let json: serde_json::Value = serde_json::json!({
            "model1": 10.5,
            "model2": 20.3
        });
        let map = parse_hashmap(&json);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("model1"), Some(&10.5));
    }

    #[test]
    fn test_parse_trend() {
        let json: serde_json::Value = serde_json::json!([
            {"date": "2025-10-27", "cost": 10.5, "count": 5},
            {"date": "2025-10-26", "cost": 8.3, "count": 3}
        ]);
        let trend = parse_trend(&json);
        assert!(trend.is_some());
        let trend = trend.unwrap();
        assert_eq!(trend.len(), 2);
        assert_eq!(trend[0].date, "2025-10-27");
    }
}
