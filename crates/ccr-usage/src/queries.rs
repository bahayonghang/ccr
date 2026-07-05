use std::collections::BTreeMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::source::SourceKind;

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

// 上 wire 的投影 DTO（被 ccr-ui Tauri 命令直接返回或嵌入响应体）在 ts feature 下
// 生成 TypeScript 绑定；i64 字段一律 ts(as = "f64")：wire 是 serde_json number，
// 映射为 number 而非 ts-rs 默认的 bigint（用量数值 << 2^53，无精度风险）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/usage/")
)]
pub struct DailyTrendDto {
    pub date: String,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub request_count: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub total_tokens: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub input_tokens: i64,
    /// Compatibility field: assistant output plus reasoning output.
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub output_tokens: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub reasoning_output_tokens: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub cache_read_tokens: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/usage/")
)]
pub struct SourceBreakdownDto {
    pub source: String,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub event_count: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub total_tokens: i64,
    pub total_cost: f64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub active_days: i64,
    pub share_tokens: f64,
    pub share_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/usage/")
)]
pub struct ProviderBreakdownDto {
    pub provider: Option<String>,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub request_count: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub input_tokens: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub cache_read_tokens: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub cache_creation_tokens: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub output_tokens: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub reasoning_output_tokens: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub total_tokens: i64,
    pub cost_with_cache_usd: f64,
    pub cost_without_cache_usd: f64,
}

impl ProviderBreakdownDto {
    /// Display label for the provider; `provider = null` renders as
    /// `unattributed` on every CCR surface.
    pub fn display_provider(&self) -> &str {
        self.provider.as_deref().unwrap_or("unattributed")
    }

    pub fn output_tokens_total(&self) -> i64 {
        self.output_tokens + self.reasoning_output_tokens
    }

    pub fn cache_tokens_total(&self) -> i64 {
        self.cache_read_tokens + self.cache_creation_tokens
    }
}

/// Provider breakdown row tagged with the source it was queried for. Produced
/// by `Dashboard::provider_breakdown_by_source` so presentation layers can mix
/// several sources in one dataset without shadow structs.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TaggedProviderBreakdown {
    pub source: SourceKind,
    pub breakdown: ProviderBreakdownDto,
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
pub struct HeatmapPoint {
    pub date: String,
    pub event_count: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/usage/")
)]
pub struct UsageRecordDto {
    pub id: String,
    pub platform: String,
    pub project_path: String,
    pub record_json: String,
    pub recorded_at: String,
    pub source_id: String,
    pub model: Option<String>,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub input_tokens: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub output_tokens: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub cache_read_tokens: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub cache_creation_tokens: i64,
    pub cost_with_cache_usd: f64,
    pub cost_without_cache_usd: f64,
    pub pricing_status: String,
    pub pricing_source: Option<String>,
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

pub fn generated_at() -> String {
    Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn provider_row_sums_output_and_cache_tokens() {
        let row = ProviderBreakdownDto {
            provider: None,
            request_count: 1,
            input_tokens: 10,
            cache_read_tokens: 2,
            cache_creation_tokens: 3,
            output_tokens: 4,
            reasoning_output_tokens: 5,
            total_tokens: 24,
            cost_with_cache_usd: 0.12,
            cost_without_cache_usd: 0.15,
        };

        assert_eq!(row.display_provider(), "unattributed");
        assert_eq!(row.output_tokens_total(), 9);
        assert_eq!(row.cache_tokens_total(), 5);
    }
}
