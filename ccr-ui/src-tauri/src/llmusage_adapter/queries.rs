// 投影 DTO 的定义归 ccr-usage（全仓唯一一份 serde 形状），adapter 通过 re-export
// 暴露并保留未来分叉权；本文件只保留 Tauri 表现层转换 DTO 与纯展示工具函数。
// 部分名字暂无 crate 内点名使用（分叉权预留），显式 allow 未使用 re-export。
#[allow(unused_imports)]
pub use ccr_usage::{
    DailyTrendDto, HeatmapPoint, HomeOverviewPayload, HomeOverviewPlatformStats,
    HomeOverviewSeriesItem, HomeOverviewSummary, ModelBreakdown, OverviewPayload, ProjectBreakdown,
    ProviderBreakdownDto, SourceBreakdownDto, TokenSummary, UsageRecordDto, generated_at,
};

use chrono::DateTime;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct UsageSummaryDto {
    // wire 走 serde_json（i64 序列化为 JSON number），ts(as = "f64") 使生成类型为
    // number 而非 ts-rs 默认的 bigint；用量数值 << 2^53，无精度风险。
    #[ts(as = "f64")]
    pub total_requests: i64,
    #[ts(as = "f64")]
    pub total_tokens: i64,
    #[ts(as = "f64")]
    pub total_input_tokens: i64,
    #[ts(as = "f64")]
    pub total_output_tokens: i64,
    #[ts(as = "f64")]
    pub total_cache_read_tokens: i64,
    pub total_cost_usd: f64,
    pub cache_efficiency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct ModelStatDto {
    pub model: String,
    #[ts(as = "f64")]
    pub request_count: i64,
    #[ts(as = "f64")]
    pub total_tokens: i64,
    pub total_cost: f64,
    #[ts(as = "f64")]
    pub input_tokens: i64,
    #[ts(as = "f64")]
    pub output_tokens: i64,
    #[ts(as = "f64")]
    pub cache_read_tokens: i64,
    #[ts(as = "f64")]
    pub cache_creation_tokens: i64,
    pub cost_with_cache: f64,
    pub cost_without_cache: f64,
    pub cache_savings: f64,
    pub pricing_status: String,
    pub pricing_source: Option<String>,
    pub pricing_rate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct ProjectStatDto {
    pub project_path: String,
    #[ts(as = "f64")]
    pub request_count: i64,
    #[ts(as = "f64")]
    pub total_tokens: i64,
    pub total_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct HeatmapResponseDto {
    #[ts(as = "std::collections::HashMap<String, f64>")]
    pub data: std::collections::HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, TS)]
#[ts(export, export_to = "../../src/types/generated/usage/")]
pub struct PaginatedLogsDto {
    pub records: Vec<UsageRecordDto>,
    #[ts(as = "Option<f64>")]
    pub total: Option<i64>,
    #[ts(as = "f64")]
    pub page: i64,
    #[ts(as = "f64")]
    pub page_size: i64,
    pub next_cursor: Option<String>,
    pub mode: String,
}

pub fn to_usage_summary(payload: OverviewPayload) -> UsageSummaryDto {
    UsageSummaryDto {
        total_requests: payload.total_events,
        total_tokens: payload.total.total_tokens,
        total_input_tokens: payload.total.input_tokens,
        total_output_tokens: payload.total.output_tokens_with_reasoning(),
        total_cache_read_tokens: payload.total.cache_read_tokens,
        total_cost_usd: payload.total_cost_usd,
        cache_efficiency: payload.cache_efficiency,
    }
}

pub fn to_daily_trends(points: Vec<DailyTrendDto>) -> Vec<DailyTrendDto> {
    points
}

pub fn to_model_stats(rows: Vec<ModelBreakdown>) -> Vec<ModelStatDto> {
    rows.into_iter()
        .map(|row| ModelStatDto {
            model: row.model,
            request_count: row.event_count,
            total_tokens: row.total_tokens,
            total_cost: row.cost_with_cache_usd,
            input_tokens: row.input_tokens,
            output_tokens: row.output_tokens + row.reasoning_output_tokens,
            cache_read_tokens: row.cache_read_tokens,
            cache_creation_tokens: row.cache_creation_tokens,
            cost_with_cache: row.cost_with_cache_usd,
            cost_without_cache: row.cost_without_cache_usd,
            cache_savings: row.cache_savings_usd,
            pricing_status: row.pricing_status,
            pricing_source: row.pricing_source,
            pricing_rate: row.pricing_rate,
        })
        .collect()
}

pub fn to_project_stats(rows: Vec<ProjectBreakdown>) -> Vec<ProjectStatDto> {
    rows.into_iter()
        .map(|row| ProjectStatDto {
            project_path: row
                .project_path
                .or(row.project_ref)
                .unwrap_or_else(|| non_empty(row.project_label).unwrap_or(row.project_hash)),
            request_count: row.event_count,
            total_tokens: row.total_tokens,
            total_cost: row.total_cost_usd,
        })
        .collect()
}

pub fn to_heatmap_response(points: Vec<HeatmapPoint>) -> HeatmapResponseDto {
    HeatmapResponseDto {
        data: points
            .into_iter()
            .map(|point| (point.date, point.event_count))
            .collect(),
    }
}

pub fn to_paginated_logs(
    page: crate::llmusage_adapter::db::LogsPage,
    page_size: i64,
) -> PaginatedLogsDto {
    PaginatedLogsDto {
        records: page.records,
        total: page.total,
        page: 1,
        page_size,
        next_cursor: page.next_cursor,
        mode: "cursor".to_string(),
    }
}

fn non_empty(value: String) -> Option<String> {
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

pub fn max_rfc3339(left: Option<String>, right: Option<String>) -> Option<String> {
    match (left, right) {
        (Some(left), Some(right)) => {
            let left_key = DateTime::parse_from_rfc3339(&left).ok();
            let right_key = DateTime::parse_from_rfc3339(&right).ok();
            match (left_key, right_key) {
                (Some(left_dt), Some(right_dt)) if right_dt > left_dt => Some(right),
                (Some(_), Some(_)) => Some(left),
                _ => Some(left.max(right)),
            }
        }
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_overview_to_existing_usage_summary_contract() {
        let summary = to_usage_summary(OverviewPayload {
            generated_at: "2026-05-09T00:00:00Z".to_string(),
            total: TokenSummary {
                input_tokens: 10,
                cache_read_tokens: 5,
                output_tokens: 20,
                reasoning_output_tokens: 3,
                total_tokens: 38,
            },
            last_24h: TokenSummary::default(),
            source_count: 1,
            bucket_count: 2,
            total_events: 7,
            last_24h_events: 1,
            total_cost_usd: 0.42,
            cache_efficiency: 0.33,
            last_sync_at: None,
            last_export_at: None,
        });

        assert_eq!(summary.total_requests, 7);
        assert_eq!(summary.total_tokens, 38);
        assert_eq!(summary.total_output_tokens, 23);
        assert_eq!(summary.total_cache_read_tokens, 5);
        assert_eq!(summary.total_cost_usd, 0.42);
    }

    #[test]
    fn project_path_falls_back_to_display_safe_fields() {
        let row = ProjectBreakdown {
            project_hash: "hash".to_string(),
            project_label: "label".to_string(),
            project_ref: None,
            total_tokens: 10,
            event_count: 2,
            total_cost_usd: 1.2,
            project_path: None,
        };

        let stats = to_project_stats(vec![row]);
        assert_eq!(stats[0].project_path, "label");
        assert_eq!(stats[0].request_count, 2);
    }
}
