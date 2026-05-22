use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct TokenSummary {
    pub input_tokens: i64,
    pub cache_read_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub total_tokens: i64,
}

impl TokenSummary {
    pub fn output_tokens_with_reasoning(&self) -> i64 {
        self.output_tokens + self.reasoning_output_tokens
    }

    pub fn cache_efficiency(&self) -> f64 {
        let denominator = self.input_tokens + self.cache_read_tokens;
        if denominator == 0 {
            0.0
        } else {
            self.cache_read_tokens as f64 / denominator as f64
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OverviewPayload {
    pub generated_at: String,
    pub total: TokenSummary,
    pub last_24h: TokenSummary,
    pub source_count: i64,
    pub bucket_count: i64,
    pub total_events: i64,
    pub last_24h_events: i64,
    pub total_cost_usd: f64,
    pub cache_efficiency: f64,
    pub last_sync_at: Option<String>,
    pub last_export_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct UsageSummaryDto {
    pub total_requests: i64,
    pub total_tokens: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cache_read_tokens: i64,
    pub total_cost_usd: f64,
    pub cache_efficiency: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DailyTrendDto {
    pub date: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub input_tokens: i64,
    /// Compatibility field: assistant output plus reasoning output.
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ModelBreakdown {
    pub model: String,
    pub input_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub total_tokens: i64,
    pub event_count: i64,
    pub cost_with_cache_usd: f64,
    pub cost_without_cache_usd: f64,
    pub cache_savings_usd: f64,
    pub pricing_status: String,
    pub pricing_source: Option<String>,
    pub pricing_rate: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ModelStatDto {
    pub model: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cost_with_cache: f64,
    pub cost_without_cache: f64,
    pub cache_savings: f64,
    pub pricing_status: String,
    pub pricing_source: Option<String>,
    pub pricing_rate: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SourceBreakdownDto {
    pub source: String,
    pub event_count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
    pub active_days: i64,
    pub share_tokens: f64,
    pub share_cost: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ProjectBreakdown {
    pub project_hash: String,
    pub project_label: String,
    pub project_ref: Option<String>,
    pub total_tokens: i64,
    pub event_count: i64,
    pub total_cost_usd: f64,
    pub project_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ProjectStatDto {
    pub project_path: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HeatmapPoint {
    pub date: String,
    pub event_count: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HeatmapResponseDto {
    pub data: HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct UsageRecordDto {
    pub id: String,
    pub platform: String,
    pub project_path: String,
    pub record_json: String,
    pub recorded_at: String,
    pub source_id: String,
    pub model: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cost_with_cache_usd: f64,
    pub cost_without_cache_usd: f64,
    pub pricing_status: String,
    pub pricing_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PaginatedLogsDto {
    pub records: Vec<UsageRecordDto>,
    pub total: Option<i64>,
    pub page: i64,
    pub page_size: i64,
    pub next_cursor: Option<String>,
    pub mode: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct HomeOverviewPlatformStats {
    pub sessions: i64,
    pub requests: i64,
    pub tokens: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct HomeOverviewSummary {
    pub total_sessions: i64,
    pub total_requests: i64,
    pub total_tokens: i64,
    pub active_days: i64,
    pub platforms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HomeOverviewSeriesItem {
    pub date: String,
    pub claude: HomeOverviewPlatformStats,
    pub codex: HomeOverviewPlatformStats,
    pub gemini: HomeOverviewPlatformStats,
    pub opencode: HomeOverviewPlatformStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HomeOverviewPayload {
    pub summary: HomeOverviewSummary,
    pub by_platform: BTreeMap<String, HomeOverviewPlatformStats>,
    pub series: Vec<HomeOverviewSeriesItem>,
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

pub fn generated_at() -> String {
    Utc::now().to_rfc3339()
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
    fn serializes_daily_trend_reasoning_without_changing_output_contract() {
        let trend = DailyTrendDto {
            date: "2026-05-21".to_string(),
            request_count: 2,
            total_tokens: 100,
            input_tokens: 40,
            output_tokens: 30,
            reasoning_output_tokens: 8,
            cache_read_tokens: 20,
            cache_creation_tokens: 10,
            cost_usd: 0.42,
        };

        let value = serde_json::to_value(&trend).expect("daily trend must serialize");

        assert_eq!(value["output_tokens"], 30);
        assert_eq!(value["reasoning_output_tokens"], 8);
        assert_eq!(value["total_tokens"], 100);
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
