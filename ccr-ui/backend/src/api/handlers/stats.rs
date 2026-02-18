// 📊 统计 API 处理器
// 提供成本和使用统计的 Web API

use crate::core::executor;
use axum::{Json, extract::Query, http::StatusCode, response::IntoResponse};
use ccr::managers::CcsConfig;
use dirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

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
    pub by_provider: HashMap<String, u64>,
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

/// 热力图数据响应
#[derive(Debug, Serialize)]
pub struct HeatmapResponse {
    /// 日期 -> token 数量
    pub data: HashMap<String, u64>,
    /// 最大值（用于计算 level）
    pub max_value: u64,
    /// 总 token 数
    pub total_tokens: u64,
    /// 活跃天数
    pub active_days: u32,
}

// ============================================================
// API 处理器
// ============================================================

/// GET /api/stats/cost - 成本概览
pub async fn cost_overview(
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<CostStatsResponse>, StatusCode> {
    // 创建唯一的临时文件（避免并发冲突）
    let temp_file =
        tempfile::NamedTempFile::new().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let temp_path = temp_file
        .path()
        .to_str()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    // 执行 CCR CLI 命令获取统计
    let args = vec![
        "stats".to_string(),
        "cost".to_string(),
        "--range".to_string(),
        params.range,
        "--details".to_string(),
        "--export".to_string(),
        temp_path.clone(),
    ];

    let output = executor::execute_command(args)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !output.success {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // 读取导出的 JSON 文件
    let stats_json = tokio::fs::read_to_string(&temp_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stats: serde_json::Value =
        serde_json::from_str(&stats_json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
        by_provider: parse_hashmap_u64(&stats["by_provider"]),
        by_model: parse_hashmap_f64(&stats["by_model"]),
        by_project: parse_hashmap_f64(&stats["by_project"]),
        trend: parse_trend(&stats["trend"]),
    };

    // 临时文件会在 temp_file drop 时自动删除

    Ok(Json(response))
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

/// GET /api/stats/provider-usage - 按提供商分组的使用次数（从 profiles.toml 读取）
pub async fn provider_usage() -> Result<Json<HashMap<String, u64>>, StatusCode> {
    let path = provider_profiles_path("claude");
    let usage = read_provider_usage(&path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(usage))
}

/// GET /api/stats/cost/top-sessions - 成本最高的会话
pub async fn cost_top_sessions(
    Query(params): Query<TopNQuery>,
) -> Result<Json<Vec<TopSessionResponse>>, StatusCode> {
    use ccr::managers::CostTracker;

    // 获取默认存储目录
    let storage_dir =
        CostTracker::default_storage_dir().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 创建 CostTracker 实例
    let tracker = CostTracker::new(storage_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 获取 Top N 会话
    let top_sessions = tracker
        .get_top_sessions(params.limit)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 转换为响应格式
    let response: Vec<TopSessionResponse> = top_sessions
        .into_iter()
        .map(|(id, cost)| TopSessionResponse {
            session_id: id,
            cost,
        })
        .collect();

    Ok(Json(response))
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
    let (today, week, month) = tokio::join!(
        cost_overview(Query(TimeRangeQuery {
            range: "today".to_string()
        })),
        cost_overview(Query(TimeRangeQuery {
            range: "week".to_string()
        })),
        cost_overview(Query(TimeRangeQuery {
            range: "month".to_string()
        })),
    );

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

/// GET /api/stats/heatmap - 热力图数据（按日期聚合 tokens）
pub async fn get_heatmap_data() -> Result<Json<HeatmapResponse>, StatusCode> {
    use serde_json::Value;
    use walkdir::WalkDir;

    // 获取 Claude 的 projects 目录
    let home = dirs::home_dir().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let projects_dir = home.join(".claude").join("projects");

    // 计算365天前的日期
    let now = chrono::Utc::now();
    let start_date = now - chrono::Duration::days(365);

    let mut data: HashMap<String, u64> = HashMap::new();
    let mut max_value: u64 = 0;
    let mut total_tokens: u64 = 0;

    // 遍历所有 .jsonl 文件
    if projects_dir.exists() {
        for entry in WalkDir::new(&projects_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "jsonl")
                && let Ok(content) = std::fs::read_to_string(path)
            {
                for line in content.lines() {
                    if let Ok(json) = serde_json::from_str::<Value>(line) {
                        // 解析 timestamp
                        let timestamp = json
                            .get("timestamp")
                            .or_else(|| json.get("message").and_then(|m| m.get("timestamp")))
                            .and_then(|t| t.as_str());

                        // 解析 usage
                        let usage = json
                            .get("usage")
                            .or_else(|| json.get("message").and_then(|m| m.get("usage")));

                        if let (Some(ts), Some(usage)) = (timestamp, usage) {
                            // 提取日期部分 (YYYY-MM-DD)
                            if let Some(date_str) = ts.split('T').next() {
                                // 检查是否在365天内
                                if let Ok(date) =
                                    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                                {
                                    let date_utc = date.and_hms_opt(0, 0, 0).map(|dt| {
                                        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                                            dt,
                                            chrono::Utc,
                                        )
                                    });
                                    if date_utc.is_some_and(|d| d >= start_date) {
                                        // 计算 tokens - 使用 map_or 避免 unwrap 系列
                                        let input = usage
                                            .get("input_tokens")
                                            .and_then(|v| v.as_u64())
                                            .map_or(0, |v| v);
                                        let output = usage
                                            .get("output_tokens")
                                            .and_then(|v| v.as_u64())
                                            .map_or(0, |v| v);
                                        let cache = usage
                                            .get("cache_read_input_tokens")
                                            .and_then(|v| v.as_u64())
                                            .map_or(0, |v| v);
                                        let tokens = input + output + cache;

                                        let entry = data.entry(date_str.to_string()).or_insert(0);
                                        *entry += tokens;
                                        total_tokens += tokens;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 计算 max_value 和 active_days
    let active_days = data.len() as u32;
    for count in data.values() {
        if *count > max_value {
            max_value = *count;
        }
    }

    Ok(Json(HeatmapResponse {
        data,
        max_value,
        total_tokens,
        active_days,
    }))
}

// ============================================================
// 辅助函数
// ============================================================

/// 解析 HashMap<u64> 从 JSON
fn parse_hashmap_u64(value: &serde_json::Value) -> HashMap<String, u64> {
    if let Some(obj) = value.as_object() {
        obj.iter()
            .filter_map(|(k, v)| {
                let count = v.as_u64()?;
                Some((k.clone(), count))
            })
            .collect()
    } else {
        HashMap::new()
    }
}

/// 解析 HashMap<f64> 从 JSON
fn parse_hashmap_f64(value: &serde_json::Value) -> HashMap<String, f64> {
    if let Some(obj) = value.as_object() {
        obj.iter()
            .filter_map(|(k, v)| {
                let val = v.as_f64()?;
                Some((k.clone(), val))
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

/// 计算 profiles.toml 的路径
fn provider_profiles_path(platform: &str) -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    home.join(".ccr")
        .join("platforms")
        .join(platform)
        .join("profiles.toml")
}

/// 从 profiles.toml 读取 provider 使用次数（usage_count 聚合）
async fn read_provider_usage(path: &PathBuf) -> Result<HashMap<String, u64>, std::io::Error> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = tokio::fs::read_to_string(path).await?;
    let config: CcsConfig = toml::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let mut map: HashMap<String, u64> = HashMap::new();
    for (_name, section) in config.sections {
        let provider = section.provider.unwrap_or_else(|| "unknown".to_string());
        let count = section.usage_count.unwrap_or(0) as u64;
        map.entry(provider)
            .and_modify(|c| *c += count)
            .or_insert(count);
    }

    Ok(map)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;

    #[test]
    fn test_parse_hashmap() {
        let json: serde_json::Value = serde_json::json!({
            "model1": 10,
            "model2": 20
        });
        let map = parse_hashmap_u64(&json);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("model1"), Some(&10));
    }

    #[test]
    fn test_parse_hashmap_f64() {
        let json: serde_json::Value = serde_json::json!({
            "model1": 10.5,
            "model2": 20.3
        });
        let map = parse_hashmap_f64(&json);
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

    #[tokio::test]
    async fn test_read_provider_usage() {
        let tmp = tempdir().unwrap();
        let profiles_path = tmp.path().join("profiles.toml");

        let toml = r#"
default_config = "a"
current_config = "a"

[a]
provider = "claude"
usage_count = 3

[b]
provider = "codex"
usage_count = 2
"#;

        fs::write(&profiles_path, toml).await.unwrap();

        let usage = read_provider_usage(&profiles_path).await.unwrap();
        assert_eq!(usage.get("claude"), Some(&3));
        assert_eq!(usage.get("codex"), Some(&2));
    }
}
