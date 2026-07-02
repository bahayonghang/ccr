use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ProviderBreakdownDto {
    pub provider: Option<String>,
    pub request_count: i64,
    pub input_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub total_tokens: i64,
    pub cost_with_cache_usd: f64,
    pub cost_without_cache_usd: f64,
}
