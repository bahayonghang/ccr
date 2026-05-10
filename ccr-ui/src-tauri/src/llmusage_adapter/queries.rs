use std::collections::HashMap;

use chrono::{DateTime, Utc};
use llmusage::query::HeatmapPoint;
use llmusage::{
    DailyTrendPoint, LogRecord, LogsPage, ModelBreakdown, OverviewPayload, ProjectBreakdown,
};
use serde::Serialize;

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
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub cost_usd: f64,
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
pub struct ProjectStatDto {
    pub project_path: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
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
    pub cost_usd: f64,
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

pub fn to_daily_trends(points: Vec<DailyTrendPoint>) -> Vec<DailyTrendDto> {
    points
        .into_iter()
        .map(|point| DailyTrendDto {
            date: point.date,
            request_count: point.event_count,
            total_tokens: point.total_tokens,
            input_tokens: point.input_tokens,
            output_tokens: point.output_tokens,
            cache_read_tokens: point.cache_read_tokens,
            cache_creation_tokens: point.cache_creation_tokens,
            cost_usd: point.cost_with_cache_usd,
        })
        .collect()
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
            cache_creation_tokens: 0,
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

pub fn to_paginated_logs(page: LogsPage, page_size: i64) -> PaginatedLogsDto {
    PaginatedLogsDto {
        records: page.records.into_iter().map(to_usage_record).collect(),
        total: page.total,
        page: 1,
        page_size,
        next_cursor: page.next_cursor,
        mode: "cursor".to_string(),
    }
}

fn to_usage_record(record: LogRecord) -> UsageRecordDto {
    UsageRecordDto {
        id: record.id,
        platform: record.platform,
        project_path: record
            .project_path
            .or(record.project_ref)
            .or(record.project_label)
            .or(record.project_hash)
            .unwrap_or_default(),
        record_json: record.raw_json.unwrap_or_default(),
        recorded_at: record.event_at,
        source_id: record.source_id,
        model: non_empty(record.model),
        input_tokens: record.input_tokens,
        output_tokens: record.output_tokens + record.reasoning_output_tokens,
        cache_read_tokens: record.cache_read_tokens,
        cache_creation_tokens: record.cache_creation_tokens,
        cost_usd: record.cost_with_cache_usd,
        cost_with_cache_usd: record.cost_with_cache_usd,
        cost_without_cache_usd: record.cost_without_cache_usd,
        pricing_status: record.pricing_status,
        pricing_source: record.pricing_source,
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
    use llmusage::query::TokenSummary;

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
    fn maps_daily_trends_to_token_debug_contract() {
        let trends = to_daily_trends(vec![DailyTrendPoint {
            date: "2026-05-10".to_string(),
            input_tokens: 100,
            cache_read_tokens: 40,
            cache_creation_tokens: 12,
            output_tokens: 30,
            total_tokens: 182,
            event_count: 5,
            cost_with_cache_usd: 0.25,
        }]);

        assert_eq!(trends[0].total_tokens, 182);
        assert_eq!(trends[0].cache_creation_tokens, 12);
        assert_eq!(trends[0].cost_usd, 0.25);
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
