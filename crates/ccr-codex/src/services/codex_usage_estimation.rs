//! Account-scoped local usage and empirical quota capacity. Values are workload estimates.

use super::codex_usage_service::{
    CodexRollingUsage, CodexScanDiagnostics, CodexUsageRecord, CodexUsageScan, CodexUsageService,
};
use crate::managers::codex_quota_observation::{CodexQuotaObservation, CodexQuotaObservationStore};
use crate::models::{CodexAuthRegistry, CodexUsageActivation};
use ccr_core::Result;
use ccr_types::{ModelRateCatalog, normalize_model_id, official_model_rate_override_for};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;

pub const AUTH_PRICE_VERSION: &str = "codex-api-equivalent-2026-10-06-v1";
pub const AUTH_PRICE_SOURCE: &str = "https://developers.openai.com/api/docs/models/gpt-6.1-sol";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexCostStatus {
    CompletePriced,
    AssumedStandard,
    Partial,
    #[default]
    Unpriced,
}
impl CodexCostStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CompletePriced => "complete_priced",
            Self::AssumedStandard => "assumed_standard",
            Self::Partial => "partial",
            Self::Unpriced => "unpriced",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CodexCostSummary {
    pub subtotal_usd: Option<f64>,
    pub status: CodexCostStatus,
    pub priced_records: u64,
    pub total_records: u64,
    pub priced_tokens: u64,
    pub total_tokens: u64,
    pub assumed_records: u64,
    pub price_version: String,
    pub source_url: Option<String>,
    pub verified_date: Option<String>,
    pub tier_basis: String,
    pub context_assumption: bool,
    pub missing_classification: bool,
    pub partial_usage: bool,
    pub currency: String,
    pub model_matches: BTreeSet<String>,
    pub pricing_sources: BTreeSet<String>,
}
impl CodexCostSummary {
    pub(crate) fn add(&mut self, record: &CodexUsageRecord, price: CodexRecordPrice) {
        self.total_records += 1;
        self.total_tokens = self
            .total_tokens
            .saturating_add(record.input_tokens.saturating_add(record.output_tokens));
        self.price_version = AUTH_PRICE_VERSION.into();
        self.currency = "USD".into();
        self.partial_usage |= record.details.partial || record.details.inconsistent_usage;
        if let Some(value) = price.usd {
            self.model_matches.insert(price.model_match.clone());
            self.pricing_sources.insert(price.source.clone());
            self.subtotal_usd = Some(self.subtotal_usd.unwrap_or(0.0) + value);
            self.priced_records += 1;
            self.priced_tokens = self
                .priced_tokens
                .saturating_add(record.input_tokens.saturating_add(record.output_tokens));
            self.assumed_records += u64::from(price.assumed);
            self.context_assumption |= price.context_assumed;
            self.missing_classification |= !price.classification_complete;
            if self.tier_basis.is_empty() {
                self.tier_basis = price.tier_basis.clone();
            } else if self.tier_basis != price.tier_basis {
                self.tier_basis = "mixed".into();
            }
            if normalize_model_id(record.model.as_deref().unwrap_or("")) == "gpt-6.1-sol" {
                self.source_url = Some(AUTH_PRICE_SOURCE.into());
                self.verified_date = Some("2026-10-06".into());
            }
        }
        self.status = if self.priced_records == 0 {
            CodexCostStatus::Unpriced
        } else if self.priced_records < self.total_records
            || self.partial_usage
            || self.missing_classification
        {
            CodexCostStatus::Partial
        } else if self.assumed_records > 0 {
            CodexCostStatus::AssumedStandard
        } else {
            CodexCostStatus::CompletePriced
        };
    }
}

#[derive(Debug, Clone, Default)]
pub struct CodexRecordPrice {
    pub usd: Option<f64>,
    pub calibration_usd: Option<f64>,
    pub assumed: bool,
    pub context_assumed: bool,
    pub classification_complete: bool,
    pub tier_basis: String,
    pub model_match: String,
    pub source: String,
    pub context_basis: String,
}
pub fn price_record(record: &CodexUsageRecord) -> CodexRecordPrice {
    let d = &record.details;
    let read = d.cache_read_tokens.unwrap_or(0);
    let write = d.cache_write_tokens.unwrap_or(0);
    let Some(uncached) = read
        .checked_add(write)
        .and_then(|cached| record.input_tokens.checked_sub(cached))
    else {
        return CodexRecordPrice::default();
    };
    if d.inconsistent_usage || d.reasoning_tokens.is_some_and(|q| q > record.output_tokens) {
        return CodexRecordPrice::default();
    }
    let model = record.model.as_deref().unwrap_or("");
    let context_assumed = d.request_input_tokens != Some(record.input_tokens);
    // Aggregate input must never select a request context tier.
    let catalog = if context_assumed {
        let Some(rate) = official_model_rate_override_for(model) else {
            return CodexRecordPrice::default();
        };
        ModelRateCatalog::with_overrides(vec![rate])
    } else {
        ModelRateCatalog::official()
    };
    let (Ok(input), Ok(output), Ok(read), Ok(write)) = (
        i64::try_from(uncached),
        i64::try_from(record.output_tokens),
        i64::try_from(read),
        i64::try_from(write),
    ) else {
        return CodexRecordPrice::default();
    };
    let result = catalog.calculate(model, input, output, read, write);
    if result.pricing_status == "unpriced" {
        return CodexRecordPrice::default();
    }
    let speed = d.speed.as_deref();
    let known_fast = speed == Some("fast") && normalize_model_id(model) == "gpt-6.1-sol";
    let known_standard = speed == Some("standard");
    if speed == Some("fast") && !known_fast {
        return CodexRecordPrice::default();
    }
    let classification_complete = d.cache_read_tokens.is_some()
        && d.cache_write_tokens.is_some()
        && d.reasoning_tokens.is_some();
    let usd = result.cost_with_cache_usd * if known_fast { 2.0 } else { 1.0 };
    let tier_basis = tier_basis(record);
    CodexRecordPrice {
        usd: Some(usd),
        calibration_usd: (classification_complete
            && !context_assumed
            && !d.partial
            && matches!(speed, None | Some("standard" | "fast")))
        .then_some(usd),
        assumed: context_assumed || !classification_complete || !(known_fast || known_standard),
        context_assumed,
        classification_complete,
        tier_basis,
        model_match: result.normalized_model,
        source: if normalize_model_id(model) == "gpt-6.1-sol" {
            "official_verified"
        } else {
            "catalog_estimate"
        }
        .into(),
        context_basis: if context_assumed {
            "short_context_assumed"
        } else if record.input_tokens > 272_000 {
            "request_long_context"
        } else {
            "request_short_context"
        }
        .into(),
    }
}
fn tier_basis(record: &CodexUsageRecord) -> String {
    match record.details.speed.as_deref() {
        Some("fast") => "fast".into(),
        Some("standard") => "standard".into(),
        None => "standard_assumed".into(),
        Some(other) => format!("standard_assumed_speed_unverified:{other}"),
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexUsageScope {
    #[default]
    ActivationIntervalInferred,
    GlobalFallback,
    Unattributed,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexEstimateStatus {
    LocalEstimate,
    #[default]
    InsufficientSamples,
    PartialUsage,
    InvalidScope,
    UnsupportedWindow,
    ResetChanged,
    Stale,
    Unpriced,
    Unstable,
    HistoryError,
    UnexplainedQuotaChange,
}
impl CodexEstimateStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalEstimate => "local_estimate",
            Self::InsufficientSamples => "insufficient_samples",
            Self::PartialUsage => "partial_usage",
            Self::InvalidScope => "invalid_scope",
            Self::UnsupportedWindow => "unsupported_window",
            Self::ResetChanged => "reset_changed",
            Self::Stale => "stale",
            Self::Unpriced => "unpriced",
            Self::Unstable => "unstable",
            Self::HistoryError => "history_error",
            Self::UnexplainedQuotaChange => "unexplained_quota_change",
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct CodexEstimateRange {
    pub median: f64,
    pub min: f64,
    pub max: f64,
}
impl CodexEstimateRange {
    fn scale(self, factor: f64) -> Self {
        Self {
            median: self.median * factor,
            min: self.min * factor,
            max: self.max * factor,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexCapacityEstimate {
    pub status: CodexEstimateStatus,
    pub usd_status: CodexEstimateStatus,
    pub token_total: Option<CodexEstimateRange>,
    pub token_remaining: Option<CodexEstimateRange>,
    pub usd_total: Option<CodexEstimateRange>,
    pub usd_remaining: Option<CodexEstimateRange>,
    pub sample_count: usize,
    pub usd_sample_count: usize,
    pub span_start: Option<DateTime<Utc>>,
    pub span_end: Option<DateTime<Utc>>,
    pub bucket: Option<String>,
    pub workload_basis: Option<String>,
    pub pricing_basis: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub network_acquired_at: Option<DateTime<Utc>>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexJointEstimate {
    pub status: CodexEstimateStatus,
    pub token_remaining: Option<CodexEstimateRange>,
    pub usd_remaining: Option<CodexEstimateRange>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexAuthUsageSnapshot {
    pub account_id: Option<String>,
    pub account_name: String,
    pub as_of: DateTime<Utc>,
    pub rolling: CodexRollingUsage,
    pub scope: CodexUsageScope,
    pub diagnostics: CodexScanDiagnostics,
    pub excluded_records: u64,
    pub unattributed_records: u64,
    pub five_hour: CodexCapacityEstimate,
    pub seven_day: CodexCapacityEstimate,
    pub joint: CodexJointEstimate,
    pub history_warning: Option<String>,
}
impl CodexAuthUsageSnapshot {
    /// Reconcile the displayed quota with the observation used for remaining capacity.
    pub fn with_quota(&self, quota: &crate::models::CodexAccountQuota) -> Self {
        let mut snapshot = self.clone();
        let provenance = quota.observation.as_ref();
        let warning = provenance.and_then(|observation| observation.history_warning.clone());
        if let Some(warning) = warning {
            snapshot.history_warning = Some(warning);
        }
        let windows = quota.quota.as_ref().map(|quota| {
            [
                (quota.hourly_window_present, quota.hourly_window_minutes),
                (quota.weekly_window_present, quota.weekly_window_minutes),
            ]
        });
        for (index, estimate) in [&mut snapshot.five_hour, &mut snapshot.seven_day]
            .into_iter()
            .enumerate()
        {
            let status = if snapshot.history_warning.is_some() {
                Some(CodexEstimateStatus::HistoryError)
            } else if snapshot.scope != CodexUsageScope::ActivationIntervalInferred
                || provenance
                    .is_some_and(|observation| observation.account_id != snapshot.account_id)
            {
                Some(CodexEstimateStatus::InvalidScope)
            } else if windows.is_none_or(|windows| {
                windows[index] != (Some(true), Some(if index == 0 { 300 } else { 10080 }))
            }) {
                Some(CodexEstimateStatus::UnsupportedWindow)
            } else if provenance.is_none_or(|observation| {
                estimate.network_acquired_at != Some(observation.network_acquired_at)
            }) {
                Some(CodexEstimateStatus::Stale)
            } else {
                None
            };
            if let Some(status) = status {
                estimate.status = status;
                estimate.usd_status = status;
                estimate.token_remaining = None;
                estimate.usd_remaining = None;
            }
        }
        snapshot.joint = joint_estimate(&snapshot.five_hour, &snapshot.seven_day);
        snapshot
    }
    /// Applies the same freshness rule when a loaded snapshot stays on screen.
    pub fn for_display(&self, now: DateTime<Utc>) -> Self {
        let mut snapshot = self.clone();
        for estimate in [&mut snapshot.five_hour, &mut snapshot.seven_day] {
            if estimate.expires_at.is_some_and(|expires| now >= expires) {
                estimate.status = CodexEstimateStatus::Stale;
                estimate.usd_status = CodexEstimateStatus::Stale;
                estimate.token_remaining = None;
                estimate.usd_remaining = None;
            }
        }
        snapshot.joint = joint_estimate(&snapshot.five_hour, &snapshot.seven_day);
        snapshot
    }
}
pub struct CodexUsageEstimationService {
    codex_dir: PathBuf,
    ccr_codex_dir: PathBuf,
}
impl CodexUsageEstimationService {
    pub fn new(codex_dir: PathBuf, ccr_codex_dir: PathBuf) -> Self {
        Self {
            codex_dir,
            ccr_codex_dir,
        }
    }
    pub fn load(
        &self,
        registry: &CodexAuthRegistry,
        account_name: &str,
        as_of: DateTime<Utc>,
    ) -> Result<CodexAuthUsageSnapshot> {
        let scan = CodexUsageService::new(self.codex_dir.clone()).scan(as_of)?;
        let mut snapshot = Self::project(&scan, registry, account_name, as_of);
        match CodexQuotaObservationStore::with_path(
            self.ccr_codex_dir.join("quota_observations.json"),
        )
        .load()
        {
            Ok(mut observations) => {
                // Rebind to a completed scan and recompute every interval. Late records change the generation.
                for observation in &mut observations {
                    if observation.network_acquired_at <= scan.completed_at {
                        observation.scan_generation = Some(scan.generation.clone());
                        observation.scan_watermark = Some(scan.completed_at);
                        observation.scan_complete = scan.diagnostics.is_complete();
                    }
                }
                if let Some(id) = &snapshot.account_id
                    && snapshot.scope == CodexUsageScope::ActivationIntervalInferred
                    && selected_account_supported(registry, account_name)
                {
                    let records = records_for_account(&scan.records, id, &registry.usage_ledger);
                    snapshot.five_hour = estimate_window(
                        &observations,
                        &scan,
                        &records,
                        &registry.usage_ledger,
                        id,
                        300,
                        as_of,
                    );
                    snapshot.seven_day = estimate_window(
                        &observations,
                        &scan,
                        &records,
                        &registry.usage_ledger,
                        id,
                        10080,
                        as_of,
                    );
                    snapshot.joint = joint_estimate(&snapshot.five_hour, &snapshot.seven_day);
                    if !runtime_route_supported(&self.codex_dir) {
                        for estimate in [&mut snapshot.five_hour, &mut snapshot.seven_day] {
                            *estimate = CodexCapacityEstimate {
                                status: CodexEstimateStatus::InvalidScope,
                                usd_status: CodexEstimateStatus::InvalidScope,
                                ..Default::default()
                            };
                        }
                        snapshot.joint = joint_estimate(&snapshot.five_hour, &snapshot.seven_day);
                    }
                }
            }
            Err(_) => {
                snapshot.history_warning = Some("quota_history_unavailable".into());
                for estimate in [&mut snapshot.five_hour, &mut snapshot.seven_day] {
                    estimate.status = CodexEstimateStatus::HistoryError;
                    estimate.usd_status = CodexEstimateStatus::HistoryError;
                }
            }
        }
        Ok(snapshot)
    }
    pub fn project(
        scan: &CodexUsageScan,
        registry: &CodexAuthRegistry,
        account_name: &str,
        as_of: DateTime<Utc>,
    ) -> CodexAuthUsageSnapshot {
        let account_id = registry
            .accounts
            .get(account_name)
            .map(|account| account.account_id.clone());
        let scope = if account_id.as_ref().is_none_or(|id| {
            !registry
                .usage_ledger
                .iter()
                .any(|entry| &entry.account_id == id)
        }) {
            CodexUsageScope::GlobalFallback
        } else {
            CodexUsageScope::ActivationIntervalInferred
        };
        let records = if let Some(id) = &account_id
            && scope == CodexUsageScope::ActivationIntervalInferred
        {
            records_for_account(&scan.records, id, &registry.usage_ledger)
        } else {
            scan.records.clone()
        };
        let unattributed_records = scan
            .records
            .iter()
            .filter(|record| activation_at(&registry.usage_ledger, record.timestamp).is_none())
            .count() as u64;
        let mut rolling = CodexUsageService::compute_rolling_usage_at(&records, as_of);
        if !scan.diagnostics.is_complete() {
            for stats in [
                &mut rolling.five_hour,
                &mut rolling.seven_day,
                &mut rolling.all_time,
            ]
            .into_iter()
            .chain(rolling.by_model.values_mut())
            {
                stats.details.partial = true;
                stats.details.cost.partial_usage = true;
                if stats.details.cost.status != CodexCostStatus::Unpriced {
                    stats.details.cost.status = CodexCostStatus::Partial;
                }
            }
        }
        let mut snapshot = CodexAuthUsageSnapshot {
            account_id,
            account_name: account_name.into(),
            as_of,
            rolling,
            scope,
            diagnostics: scan.diagnostics.clone(),
            excluded_records: scan.records.len().saturating_sub(records.len()) as u64,
            unattributed_records,
            ..Default::default()
        };
        if !selected_account_supported(registry, account_name) {
            for estimate in [&mut snapshot.five_hour, &mut snapshot.seven_day] {
                estimate.status = CodexEstimateStatus::InvalidScope;
                estimate.usd_status = CodexEstimateStatus::InvalidScope;
            }
            snapshot.joint = joint_estimate(&snapshot.five_hour, &snapshot.seven_day);
        }
        snapshot
    }
}
fn selected_account_supported(registry: &CodexAuthRegistry, name: &str) -> bool {
    registry.accounts.get(name).is_some_and(|account| {
        !account.account_id.is_empty()
            && account.auth_method != Some(crate::models::OpenAiAuthMethod::Api)
            && account.api_provider_name.is_none()
            && account
                .api_base_url
                .as_deref()
                .is_none_or(super::codex_usage_service::subscription_url)
    })
}
pub(super) fn config_route_supported(text: &str) -> bool {
    let Ok(config) = toml::from_str::<toml::Value>(text) else {
        return false;
    };
    if config
        .get("forced_login_method")
        .and_then(toml::Value::as_str)
        .is_some_and(|mode| mode != "chatgpt")
    {
        return false;
    }
    let provider = config
        .get("model_provider")
        .and_then(toml::Value::as_str)
        .unwrap_or("openai");
    if !matches!(provider, "openai" | "chatgpt") {
        return false;
    }
    let provider = config
        .get("model_providers")
        .and_then(|providers| providers.get(provider));
    provider
        .and_then(|provider| provider.get("base_url"))
        .and_then(toml::Value::as_str)
        .is_none_or(super::codex_usage_service::subscription_url)
}
fn runtime_route_supported(dir: &std::path::Path) -> bool {
    let config_ok = match std::fs::read_to_string(dir.join("config.toml")) {
        Ok(text) => config_route_supported(&text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(_) => false,
    };
    let auth_ok = match std::fs::read(dir.join("auth.json")) {
        Ok(bytes) => serde_json::from_slice::<serde_json::Value>(&bytes).is_ok_and(|auth| {
            auth.get("auth_mode")
                .and_then(serde_json::Value::as_str)
                .is_none_or(|mode| mode == "chatgpt")
                && auth
                    .get("OPENAI_API_KEY")
                    .is_none_or(|key| key.is_null() || key.as_str() == Some(""))
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(_) => false,
    };
    config_ok && auth_ok
}
fn activation_at(
    ledger: &[CodexUsageActivation],
    time: DateTime<Utc>,
) -> Option<&CodexUsageActivation> {
    ledger
        .iter()
        .filter(|entry| entry.started_at <= time)
        .max_by_key(|entry| entry.started_at)
}
pub fn records_for_account(
    records: &[CodexUsageRecord],
    id: &str,
    ledger: &[CodexUsageActivation],
) -> Vec<CodexUsageRecord> {
    records
        .iter()
        .filter(|record| {
            activation_at(ledger, record.timestamp).is_some_and(|entry| entry.account_id == id)
        })
        .cloned()
        .collect()
}

/// Pure estimation. Endpoints must be bound to the supplied completed scan.
pub fn estimate_window(
    observations: &[CodexQuotaObservation],
    scan: &CodexUsageScan,
    records: &[CodexUsageRecord],
    ledger: &[CodexUsageActivation],
    account_id: &str,
    duration: i64,
    as_of: DateTime<Utc>,
) -> CodexCapacityEstimate {
    let mut result = CodexCapacityEstimate::default();
    let mut all: Vec<_> = observations
        .iter()
        .filter(|observation| {
            observation.account_id == account_id && observation.source == "network"
        })
        .collect();
    all.sort_by_key(|observation| observation.network_acquired_at);
    all.dedup_by(|first, second| *first == *second);
    let Some(latest_time) = all
        .last()
        .map(|observation| observation.network_acquired_at)
    else {
        return result;
    };
    let Some(current) = all
        .iter()
        .rev()
        .find(|observation| {
            observation.network_acquired_at == latest_time
                && observation.duration_minutes == Some(duration)
        })
        .copied()
    else {
        result.status = CodexEstimateStatus::UnsupportedWindow;
        result.usd_status = result.status;
        return result;
    };
    result.bucket = Some(current.bucket());
    result.network_acquired_at = Some(current.network_acquired_at);
    if !current.scope_supported
        || !matches!(
            current.bucket_source.as_str(),
            "wham_main" | "wham_limit_id"
        )
        || current.plan.as_ref().is_none_or(|plan| plan.is_empty())
        || current
            .used_percent
            .is_none_or(|percent| !percent.is_finite() || !(0.0..=100.0).contains(&percent))
    {
        result.status = CodexEstimateStatus::InvalidScope;
        result.usd_status = result.status;
        return result;
    }
    let Some(reset) = current
        .resets_at
        .filter(|_| current.reset_generation.is_some())
    else {
        result.status = CodexEstimateStatus::UnsupportedWindow;
        result.usd_status = result.status;
        return result;
    };
    result.expires_at = DateTime::from_timestamp(reset, 0)
        .map(|reset| reset.min(current.network_acquired_at + Duration::minutes(5)));
    let fresh = current.network_acquired_at <= as_of
        && result.expires_at.is_some_and(|expires| as_of < expires);
    if !scan.diagnostics.is_complete() {
        result.status = CodexEstimateStatus::PartialUsage;
        result.usd_status = CodexEstimateStatus::PartialUsage;
        return result;
    }
    let same_series = |candidate: &&CodexQuotaObservation| {
        candidate.duration_minutes == current.duration_minutes
            && candidate.plan == current.plan
            && candidate.bucket() == current.bucket()
            && candidate.reset_generation == current.reset_generation
    };
    let mut samples: Vec<_> = all.iter().copied().filter(same_series).collect();
    samples.sort_by_key(|observation| observation.network_acquired_at);
    samples.dedup_by_key(|observation| observation.network_acquired_at);
    // A plan/bucket/reset/window transition ends the old series. Do not reuse an older matching series.
    let acquisition_times: BTreeSet<_> = all
        .iter()
        .map(|candidate| candidate.network_acquired_at)
        .collect();
    if let Some(boundary) = acquisition_times.iter().rev().find(|time| {
        let group: Vec<_> = all
            .iter()
            .filter(|candidate| {
                candidate.network_acquired_at == **time
                    && candidate.duration_minutes == Some(duration)
            })
            .collect();
        group.len() != 1
            || group.iter().any(|candidate| {
                candidate.plan != current.plan
                    || candidate.bucket() != current.bucket()
                    || candidate.reset_generation != current.reset_generation
            })
    }) {
        samples.retain(|candidate| candidate.network_acquired_at > *boundary);
        result.status = CodexEstimateStatus::ResetChanged;
    }
    let mut token_capacities = Vec::new();
    let mut token_starts = Vec::new();
    let mut usd_capacities = Vec::new();
    let mut workload_basis: Option<String> = None;
    let mut pricing_basis: Option<String> = None;
    let Some(mut baseline) = samples.first().copied() else {
        return result;
    };
    for endpoint in samples.into_iter().skip(1) {
        let valid_endpoint = |observation: &CodexQuotaObservation| {
            observation.scan_complete
                && observation.scan_generation.as_deref() == Some(scan.generation.as_str())
                && observation
                    .scan_watermark
                    .is_some_and(|watermark| watermark >= observation.network_acquired_at)
                && observation.scope_supported
        };
        if !valid_endpoint(baseline) || !valid_endpoint(endpoint) {
            result.status = CodexEstimateStatus::PartialUsage;
            token_capacities.clear();
            token_starts.clear();
            usd_capacities.clear();
            result.span_start = None;
            result.span_end = None;
            baseline = endpoint;
            continue;
        }
        let Some((before, after)) =
            baseline
                .used_percent
                .zip(endpoint.used_percent)
                .filter(|(a, b)| {
                    a.is_finite()
                        && b.is_finite()
                        && (0.0..=100.0).contains(a)
                        && (0.0..=100.0).contains(b)
                })
        else {
            result.status = CodexEstimateStatus::InvalidScope;
            token_capacities.clear();
            token_starts.clear();
            usd_capacities.clear();
            result.span_start = None;
            result.span_end = None;
            baseline = endpoint;
            continue;
        };
        let delta = after - before;
        if delta < 0.0 {
            token_capacities.clear();
            token_starts.clear();
            usd_capacities.clear();
            result.span_start = None;
            result.span_end = None;
            baseline = endpoint;
            result.status = CodexEstimateStatus::ResetChanged;
            continue;
        }
        if delta < 5.0 {
            continue;
        }
        let interval: Vec<_> = records
            .iter()
            .filter(|record| {
                record.timestamp >= baseline.network_acquired_at
                    && record.timestamp < endpoint.network_acquired_at
            })
            .collect();
        let active = activation_at(ledger, baseline.network_acquired_at);
        let scope_valid = active.is_some_and(|activation| activation.account_id == account_id)
            && !ledger.iter().any(|activation| {
                activation.started_at > baseline.network_acquired_at
                    && activation.started_at < endpoint.network_acquired_at
                    && activation.account_id != account_id
            })
            && interval.iter().all(|record| {
                let d = &record.details;
                !d.scope_mismatch
                    && d.account_id.as_deref().is_none_or(|id| id == account_id)
                    && d.model_provider
                        .as_deref()
                        .is_none_or(|provider| matches!(provider, "openai" | "chatgpt"))
                    && d.session_started_at.is_some_and(|start| {
                        active.is_some_and(|activation| start >= activation.started_at)
                    })
                    && record.model.is_some()
                    && if current.bucket_source == "wham_limit_id" {
                        d.quota_limit_id == current.bucket_id && current.bucket_id.is_some()
                    } else {
                        d.quota_limit_id.is_none()
                    }
            });
        let usage_valid = interval.iter().all(|record| {
            !record.details.partial
                && !record.details.inconsistent_usage
                && record.details.time_basis == "event_timestamp"
        });
        let tokens = interval
            .iter()
            .map(|record| record.input_tokens as f64 + record.output_tokens as f64)
            .sum::<f64>();
        if !scope_valid || !usage_valid || tokens <= 0.0 {
            result.status = if !scope_valid {
                CodexEstimateStatus::InvalidScope
            } else if !usage_valid {
                CodexEstimateStatus::PartialUsage
            } else {
                CodexEstimateStatus::UnexplainedQuotaChange
            };
            token_capacities.clear();
            token_starts.clear();
            usd_capacities.clear();
            result.span_start = None;
            result.span_end = None;
            baseline = endpoint;
            continue;
        }
        let models: BTreeSet<_> = interval
            .iter()
            .map(|record| normalize_model_id(record.model.as_deref().unwrap_or("")))
            .collect();
        let tiers: BTreeSet<_> = interval.iter().map(|record| tier_basis(record)).collect();
        let basis = format!(
            "models={};tiers={}",
            models.into_iter().collect::<Vec<_>>().join(","),
            tiers.into_iter().collect::<Vec<_>>().join(",")
        );
        if workload_basis.as_ref().is_some_and(|old| old != &basis) {
            token_capacities.clear();
            token_starts.clear();
            usd_capacities.clear();
            result.span_start = None;
            result.span_end = None;
        }
        workload_basis = Some(basis.clone());
        let price_basis = format!("{AUTH_PRICE_VERSION}:{basis}");
        pricing_basis = Some(price_basis);
        let capacity = tokens / (delta / 100.0);
        token_capacities.push(capacity);
        token_starts.push(baseline.network_acquired_at);
        if token_capacities.len() > 20 {
            token_capacities.remove(0);
            token_starts.remove(0);
        }
        let costs: Option<Vec<f64>> = interval
            .iter()
            .map(|record| price_record(record).calibration_usd)
            .collect();
        // Mixed Fast and assumed Standard cannot describe one USD scenario.
        let mixed_tier = basis.contains("fast") && basis.contains("standard");
        if let Some(costs) = costs.filter(|_| !mixed_tier) {
            usd_capacities.push(costs.into_iter().sum::<f64>() / (delta / 100.0));
            if usd_capacities.len() > 20 {
                usd_capacities.remove(0);
            }
        } else {
            usd_capacities.clear();
        }
        result.span_start = token_starts.first().copied();
        result.span_end = Some(endpoint.network_acquired_at);
        baseline = endpoint;
        result.status = CodexEstimateStatus::InsufficientSamples;
    }
    result.sample_count = token_capacities.len();
    result.usd_sample_count = usd_capacities.len();
    result.workload_basis = workload_basis;
    result.pricing_basis = pricing_basis;
    if result.sample_count >= 3 {
        result.token_total = empirical_range(&token_capacities);
        result.status = if result.token_total.is_some() {
            CodexEstimateStatus::LocalEstimate
        } else {
            CodexEstimateStatus::Unstable
        };
    }
    if result.usd_sample_count >= 3 {
        result.usd_total = empirical_range(&usd_capacities);
        result.usd_status = if result.usd_total.is_some() {
            CodexEstimateStatus::LocalEstimate
        } else {
            CodexEstimateStatus::Unstable
        };
    } else {
        result.usd_status = if result.sample_count >= 3 {
            CodexEstimateStatus::Unpriced
        } else {
            result.status
        };
    }
    if !fresh {
        result.status = CodexEstimateStatus::Stale;
        result.usd_status = CodexEstimateStatus::Stale;
    } else {
        let remaining = (1.0 - current.used_percent.unwrap_or(100.0) / 100.0).clamp(0.0, 1.0);
        result.token_remaining = result.token_total.map(|range| range.scale(remaining));
        result.usd_remaining = result.usd_total.map(|range| range.scale(remaining));
    }
    result
}
fn empirical_range(values: &[f64]) -> Option<CodexEstimateRange> {
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let min = *sorted.first()?;
    let max = *sorted.last()?;
    if !min.is_finite() || !max.is_finite() || min <= 0.0 || max / min > 2.0 {
        return None;
    }
    let median = if sorted.len().is_multiple_of(2) {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };
    Some(CodexEstimateRange { median, min, max })
}
pub fn joint_estimate(
    five: &CodexCapacityEstimate,
    seven: &CodexCapacityEstimate,
) -> CodexJointEstimate {
    if five.bucket.is_none()
        || five.bucket != seven.bucket
        || five.workload_basis.is_none()
        || five.workload_basis != seven.workload_basis
    {
        return CodexJointEstimate {
            status: CodexEstimateStatus::InvalidScope,
            ..Default::default()
        };
    }
    let min = |a: CodexEstimateRange, b: CodexEstimateRange| CodexEstimateRange {
        median: a.median.min(b.median),
        min: a.min.min(b.min),
        max: a.max.min(b.max),
    };
    let token_remaining = if five.status == CodexEstimateStatus::LocalEstimate
        && seven.status == CodexEstimateStatus::LocalEstimate
    {
        five.token_remaining
            .zip(seven.token_remaining)
            .map(|(a, b)| min(a, b))
    } else {
        None
    };
    let usd_remaining = if five.usd_status == CodexEstimateStatus::LocalEstimate
        && seven.usd_status == CodexEstimateStatus::LocalEstimate
        && five.pricing_basis.is_some()
        && five.pricing_basis == seven.pricing_basis
    {
        five.usd_remaining
            .zip(seven.usd_remaining)
            .map(|(a, b)| min(a, b))
    } else {
        None
    };
    CodexJointEstimate {
        status: if token_remaining.is_some() || usd_remaining.is_some() {
            CodexEstimateStatus::LocalEstimate
        } else {
            CodexEstimateStatus::InvalidScope
        },
        token_remaining,
        usd_remaining,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::models::CodexUsageActivation;
    use crate::services::codex_usage_service::CodexUsageDetails;

    fn usage_record(
        timestamp: DateTime<Utc>,
        account_id: &str,
        ordinal: usize,
    ) -> CodexUsageRecord {
        CodexUsageRecord {
            session_id: "synthetic-session".into(),
            timestamp,
            input_tokens: 900_000,
            output_tokens: 100_000,
            model: Some("gpt-6.1-sol".into()),
            details: CodexUsageDetails {
                cache_read_tokens: Some(0),
                cache_write_tokens: Some(0),
                reasoning_tokens: Some(0),
                request_input_tokens: Some(900_000),
                model_provider: Some("openai".into()),
                account_id: Some(account_id.into()),
                session_started_at: Some(timestamp),
                time_basis: "event_timestamp".into(),
                measurement_basis: "request".into(),
                event_identity: format!("synthetic:{ordinal}"),
                ..Default::default()
            },
        }
    }

    fn observation(
        acquired_at: DateTime<Utc>,
        used_percent: f64,
        scan_generation: &str,
    ) -> CodexQuotaObservation {
        CodexQuotaObservation {
            account_id: "stable-account".into(),
            plan: Some("plus".into()),
            source: "network".into(),
            bucket_source: "wham_main".into(),
            window_role: "primary_window".into(),
            duration_minutes: Some(300),
            limit_window_seconds: Some(18_000),
            used_percent: Some(used_percent),
            resets_at: Some(acquired_at.timestamp() + 86_400),
            reset_generation: Some("reset_at:fixed".into()),
            network_acquired_at: acquired_at,
            returned_at: acquired_at,
            scope_supported: true,
            scan_generation: Some(scan_generation.into()),
            scan_watermark: Some(acquired_at),
            scan_complete: true,
            ..Default::default()
        }
    }

    #[test]
    fn gpt_6_1_sol_synthetic_cost_matches_verified_standard_and_fast_examples() {
        let timestamp = DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let mut record = usage_record(timestamp, "stable-account", 1);
        record.input_tokens = 100_000;
        record.output_tokens = 2_000;
        record.details.cache_read_tokens = Some(90_000);
        record.details.cache_write_tokens = Some(5_000);
        record.details.reasoning_tokens = Some(1_000);
        record.details.request_input_tokens = Some(100_000);
        record.details.speed = None;
        let standard = (0..100)
            .map(|_| price_record(&record).usd.unwrap())
            .sum::<f64>();
        record.details.speed = Some("fast".into());
        let fast = (0..100)
            .map(|_| price_record(&record).usd.unwrap())
            .sum::<f64>();
        assert!((standard - 5.15).abs() < 0.000_001);
        assert!((fast - standard * 2.0).abs() < 0.000_001);
    }

    #[test]
    fn three_independent_segments_produce_ten_million_token_capacity() {
        let as_of = DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let points = [
            as_of - Duration::hours(4),
            as_of - Duration::hours(3),
            as_of - Duration::hours(2),
            as_of - Duration::minutes(1),
        ];
        let observations = points
            .iter()
            .zip([10.0, 20.0, 30.0, 40.0])
            .map(|(time, used)| observation(*time, used, "scan-1"))
            .collect::<Vec<_>>();
        let records = points
            .iter()
            .enumerate()
            .take(3)
            .map(|(index, time)| {
                usage_record(*time + Duration::seconds(1), "stable-account", index)
            })
            .collect::<Vec<_>>();
        let scan = CodexUsageScan {
            records: records.clone(),
            diagnostics: CodexScanDiagnostics::default(),
            completed_at: as_of,
            generation: "scan-1".into(),
        };
        let ledger = vec![CodexUsageActivation {
            account_name: "main".into(),
            account_id: "stable-account".into(),
            started_at: as_of - Duration::days(1),
        }];
        let estimate = estimate_window(
            &observations,
            &scan,
            &records,
            &ledger,
            "stable-account",
            300,
            as_of,
        );
        assert_eq!(estimate.status, CodexEstimateStatus::LocalEstimate);
        let total = estimate.token_total.unwrap();
        assert!((total.median - 10_000_000.0).abs() < 0.1);
        assert!((total.min - 10_000_000.0).abs() < 0.1);
        assert!((total.max - 10_000_000.0).abs() < 0.1);
        assert!((estimate.token_remaining.unwrap().median - 6_000_000.0).abs() < 0.1);
    }

    struct Fixture {
        as_of: DateTime<Utc>,
        observations: Vec<CodexQuotaObservation>,
        scan: CodexUsageScan,
        records: Vec<CodexUsageRecord>,
        ledger: Vec<CodexUsageActivation>,
    }
    impl Fixture {
        fn new() -> Self {
            let as_of = DateTime::from_timestamp(1_700_000_000, 0).unwrap();
            let points = [
                as_of - Duration::hours(4),
                as_of - Duration::hours(3),
                as_of - Duration::hours(2),
                as_of - Duration::minutes(1),
            ];
            let observations = points
                .into_iter()
                .zip([20.0, 30.0, 40.0, 50.0])
                .map(|(time, used)| {
                    let mut value = observation(time, used, "scan");
                    value.resets_at = Some((as_of + Duration::days(1)).timestamp());
                    value.reset_generation = Some(format!("reset_at:{}", value.resets_at.unwrap()));
                    value
                })
                .collect();
            let mut records = Vec::new();
            for (segment, count) in [100, 120, 80].into_iter().enumerate() {
                for index in 0..count {
                    let mut record = usage_record(
                        points[segment] + Duration::seconds(index + 1),
                        "stable-account",
                        records.len(),
                    );
                    record.input_tokens = 10_000;
                    record.output_tokens = 0;
                    record.details.request_input_tokens = Some(10_000);
                    record.details.cache_read_tokens = Some(8_000);
                    record.details.cache_write_tokens = Some(400);
                    records.push(record);
                }
            }
            let scan = CodexUsageScan {
                records: records.clone(),
                diagnostics: CodexScanDiagnostics::default(),
                completed_at: as_of,
                generation: "scan".into(),
            };
            let ledger = vec![CodexUsageActivation {
                account_name: "main".into(),
                account_id: "stable-account".into(),
                started_at: as_of - Duration::days(1),
            }];
            Self {
                as_of,
                observations,
                scan,
                records,
                ledger,
            }
        }
        fn estimate(&self) -> CodexCapacityEstimate {
            estimate_window(
                &self.observations,
                &self.scan,
                &self.records,
                &self.ledger,
                "stable-account",
                300,
                self.as_of,
            )
        }
    }
    fn close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 0.000_001,
            "actual={actual}; expected={expected}"
        );
    }

    #[test]
    fn independent_synthetic_requests_produce_full_token_usd_envelope() {
        let fixture = Fixture::new();
        let result = fixture.estimate();
        assert_eq!(result.status, CodexEstimateStatus::LocalEstimate);
        assert_eq!(result.usd_status, CodexEstimateStatus::LocalEstimate);
        assert_eq!(result.sample_count, 3);
        assert_eq!(result.usd_sample_count, 3);
        let tokens = result.token_total.unwrap();
        close(tokens.median, 10_000_000.0);
        close(tokens.min, 8_000_000.0);
        close(tokens.max, 12_000_000.0);
        let usd = result.usd_total.unwrap();
        close(usd.median, 5.0);
        close(usd.min, 4.0);
        close(usd.max, 6.0);
        let remaining = result.token_remaining.unwrap();
        close(remaining.median, 5_000_000.0);
        close(remaining.min, 4_000_000.0);
        close(remaining.max, 6_000_000.0);
        close(result.usd_remaining.unwrap().median, 2.5);
        assert_eq!(
            result.span_start,
            Some(fixture.observations[0].network_acquired_at)
        );
        assert_eq!(
            result.span_end,
            Some(fixture.observations[3].network_acquired_at)
        );
    }

    #[test]
    fn scan_gaps_mark_cost_subtotals_partial_without_inventing_coverage() {
        let fixture = Fixture::new();
        for (speed, model, complete_status) in [
            (
                Some("standard"),
                "gpt-6.1-sol",
                CodexCostStatus::CompletePriced,
            ),
            (None, "gpt-6.1-sol", CodexCostStatus::AssumedStandard),
            (None, "unknown-model", CodexCostStatus::Unpriced),
        ] {
            let mut scan = fixture.scan.clone();
            for record in &mut scan.records {
                record.model = Some(model.into());
                record.details.speed = speed.map(str::to_owned);
            }
            let registry = CodexAuthRegistry::default();
            let complete =
                CodexUsageEstimationService::project(&scan, &registry, "main", fixture.as_of);
            assert_eq!(
                complete.rolling.all_time.details.cost.status,
                complete_status
            );
            for unreadable in [false, true] {
                scan.diagnostics = if unreadable {
                    CodexScanDiagnostics {
                        unreadable_files: 1,
                        ..Default::default()
                    }
                } else {
                    CodexScanDiagnostics {
                        invalid_json: 1,
                        ..Default::default()
                    }
                };
                let projected =
                    CodexUsageEstimationService::project(&scan, &registry, "main", fixture.as_of);
                for stats in [
                    &projected.rolling.five_hour,
                    &projected.rolling.seven_day,
                    &projected.rolling.all_time,
                ]
                .into_iter()
                .chain(projected.rolling.by_model.values())
                {
                    let cost = &stats.details.cost;
                    let complete_cost = &complete.rolling.all_time.details.cost;
                    assert!(stats.details.partial);
                    assert!(cost.partial_usage);
                    assert_eq!(
                        cost.status,
                        if complete_status == CodexCostStatus::Unpriced {
                            CodexCostStatus::Unpriced
                        } else {
                            CodexCostStatus::Partial
                        }
                    );
                    assert_eq!(cost.subtotal_usd, complete_cost.subtotal_usd);
                    assert_eq!(cost.priced_records, complete_cost.priced_records);
                    assert_eq!(cost.total_records, 300);
                    assert_eq!(cost.priced_tokens, complete_cost.priced_tokens);
                    assert_eq!(cost.total_tokens, 3_000_000);
                }
            }
        }
    }

    #[test]
    fn price_quality_context_threshold_cache_write_and_unknown_model() {
        let mut record = Fixture::new().records[0].clone();
        let price = price_record(&record);
        close(price.usd.unwrap(), 0.005);
        assert!(price.calibration_usd.is_some());
        assert_eq!(price.tier_basis, "standard_assumed");
        assert_eq!(price.source, "official_verified");
        assert_eq!(price.model_match, "gpt-6.1-sol");
        record.details.service_tier = Some("default".into());
        assert!(price_record(&record).assumed);
        record.details.speed = Some("standard".into());
        assert!(!price_record(&record).assumed);
        record.input_tokens = 272_000;
        record.output_tokens = 2_000;
        record.details.request_input_tokens = Some(272_000);
        record.details.cache_read_tokens = Some(200_000);
        record.details.cache_write_tokens = Some(72_000);
        record.details.reasoning_tokens = Some(2_000);
        close(price_record(&record).usd.unwrap(), 0.22);
        assert_eq!(price_record(&record).context_basis, "request_short_context");
        record.input_tokens = 272_001;
        record.details.cache_write_tokens = Some(72_001);
        record.details.request_input_tokens = Some(272_001);
        close(price_record(&record).usd.unwrap(), 0.430005);
        assert_eq!(price_record(&record).context_basis, "request_long_context");
        record.details.request_input_tokens = None;
        let price = price_record(&record);
        close(price.usd.unwrap(), 0.2200025);
        assert!(price.context_assumed);
        assert!(price.calibration_usd.is_none());
        record.details.cache_write_tokens = None;
        assert!(price_record(&record).usd.is_some());
        assert!(!price_record(&record).classification_complete);
        record.model = Some("gpt-6.1-sol-unverified-preview".into());
        assert!(price_record(&record).usd.is_none());
        record.model = Some("gpt-5.4".into());
        record.details.speed = None;
        assert_eq!(price_record(&record).source, "catalog_estimate");
    }

    #[test]
    fn costs_keep_partial_coverage_and_known_zero_separate_from_unpriced() {
        let fixture = Fixture::new();
        let priced = fixture.records[0].clone();
        let mut unknown = priced.clone();
        unknown.model = Some("unknown".into());
        let mut summary = CodexCostSummary::default();
        summary.add(&priced, price_record(&priced));
        summary.add(&unknown, price_record(&unknown));
        assert_eq!(summary.status, CodexCostStatus::Partial);
        assert_eq!(summary.priced_records, 1);
        assert_eq!(summary.total_records, 2);
        assert_eq!(summary.priced_tokens, 10_000);
        assert_eq!(summary.total_tokens, 20_000);
        assert_eq!(summary.currency, "USD");
        assert_eq!(summary.verified_date.as_deref(), Some("2026-10-06"));
        let mut all_unknown = CodexCostSummary::default();
        all_unknown.add(&unknown, price_record(&unknown));
        assert_eq!(all_unknown.status, CodexCostStatus::Unpriced);
        assert_eq!(all_unknown.subtotal_usd, None);
        let mut zero = priced;
        zero.input_tokens = 0;
        zero.details.request_input_tokens = Some(0);
        zero.details.cache_read_tokens = Some(0);
        zero.details.cache_write_tokens = Some(0);
        close(price_record(&zero).usd.unwrap(), 0.0);
    }

    #[test]
    fn independent_token_usd_quality_and_instability() {
        for missing in [true, false] {
            let mut fixture = Fixture::new();
            for record in &mut fixture.records {
                if missing {
                    record.details.cache_write_tokens = None;
                } else {
                    record.model = Some("unpriced-model".into());
                }
            }
            let estimate = fixture.estimate();
            assert_eq!(estimate.status, CodexEstimateStatus::LocalEstimate);
            assert_eq!(estimate.usd_status, CodexEstimateStatus::Unpriced);
            assert!(estimate.token_remaining.is_some());
            assert!(estimate.usd_remaining.is_none());
        }
        let mut fixture = Fixture::new();
        let third_start = fixture.observations[2].network_acquired_at;
        fixture
            .records
            .retain(|record| record.timestamp < third_start);
        for index in 0..300 {
            let mut record = fixture.records[0].clone();
            record.timestamp = third_start + Duration::seconds(index + 1);
            record.details.event_identity = format!("third:{index}");
            record.details.cache_read_tokens = Some(9_500);
            record.details.cache_write_tokens = Some(0);
            fixture.records.push(record);
        }
        let estimate = fixture.estimate();
        assert_eq!(estimate.status, CodexEstimateStatus::Unstable);
        assert!(estimate.token_total.is_none());
        assert_eq!(estimate.usd_status, CodexEstimateStatus::LocalEstimate);
        assert!(estimate.usd_remaining.is_some());
        let mut fixture = Fixture::new();
        for record in fixture
            .records
            .iter_mut()
            .filter(|record| record.timestamp < fixture.observations[1].network_acquired_at)
        {
            record.details.cache_read_tokens = Some(10_000);
            record.details.cache_write_tokens = Some(0);
        }
        let estimate = fixture.estimate();
        assert_eq!(estimate.status, CodexEstimateStatus::LocalEstimate);
        assert_eq!(estimate.usd_status, CodexEstimateStatus::Unstable);
        assert!(estimate.token_remaining.is_some());
        assert!(estimate.usd_remaining.is_none());
    }

    #[test]
    fn invalid_sampling_conditions_have_determined_states() {
        let mut fixture = Fixture::new();
        fixture.observations.truncate(3);
        assert!(fixture.estimate().token_total.is_none());
        assert_eq!(fixture.estimate().status, CodexEstimateStatus::Stale);
        for values in [[20.0, 20.0, 20.0, 20.0], [20.0, 24.0, 28.0, 32.0]] {
            let mut fixture = Fixture::new();
            for (sample, value) in fixture.observations.iter_mut().zip(values) {
                sample.used_percent = Some(value);
            }
            let result = fixture.estimate();
            assert_eq!(result.status, CodexEstimateStatus::InsufficientSamples);
            assert!(result.token_total.is_none());
        }
        let mut fixture = Fixture::new();
        fixture.observations[3].used_percent = Some(10.0);
        assert_eq!(fixture.estimate().status, CodexEstimateStatus::ResetChanged);
        let mut fixture = Fixture::new();
        fixture.records.clear();
        assert_eq!(
            fixture.estimate().status,
            CodexEstimateStatus::UnexplainedQuotaChange
        );
        let mut fixture = Fixture::new();
        fixture.scan.diagnostics.invalid_json = 1;
        assert_eq!(fixture.estimate().status, CodexEstimateStatus::PartialUsage);
        let mut fixture = Fixture::new();
        fixture.observations[3].scan_watermark =
            Some(fixture.observations[3].network_acquired_at - Duration::seconds(1));
        assert_eq!(fixture.estimate().status, CodexEstimateStatus::PartialUsage);
        let mut fixture = Fixture::new();
        fixture.observations[3].used_percent = Some(f64::NAN);
        assert_eq!(fixture.estimate().status, CodexEstimateStatus::InvalidScope);
        for change in 0..4 {
            let mut fixture = Fixture::new();
            match change {
                0 => fixture.observations[3].reset_generation = None,
                1 => fixture.observations[3].resets_at = None,
                2 => fixture.observations[3].duration_minutes = None,
                _ => fixture.observations[3].duration_minutes = Some(600),
            };
            assert_eq!(
                fixture.estimate().status,
                CodexEstimateStatus::UnsupportedWindow
            );
            assert!(fixture.estimate().token_remaining.is_none());
        }
    }

    #[test]
    fn reset_plan_window_model_and_speed_boundaries_restart_sampling() {
        for change in 0..3 {
            let mut fixture = Fixture::new();
            match change {
                0 => fixture.observations[3].reset_generation = Some("new-reset".into()),
                1 => fixture.observations[3].plan = Some("pro".into()),
                _ => fixture.observations[1].duration_minutes = Some(600),
            };
            let estimate = fixture.estimate();
            assert!(estimate.token_total.is_none());
            assert!(estimate.sample_count < 3);
        }
        for change in 0..2 {
            let mut fixture = Fixture::new();
            let boundary = fixture.observations[2].network_acquired_at;
            for record in fixture
                .records
                .iter_mut()
                .filter(|record| record.timestamp >= boundary)
            {
                if change == 0 {
                    record.model = Some("gpt-5.4".into());
                } else {
                    record.details.speed = Some("fast".into());
                }
            }
            let result = fixture.estimate();
            assert_eq!(result.sample_count, 1);
            assert_eq!(result.span_start, Some(boundary));
            assert!(result.token_total.is_none());
        }
        let mut fixture = Fixture::new();
        fixture.observations[3].source = "cache".into();
        assert!(fixture.estimate().token_remaining.is_none());
        let mut fixture = Fixture::new();
        fixture.observations.push(fixture.observations[3].clone());
        let estimate = fixture.estimate();
        assert_eq!(estimate.sample_count, 3);
    }

    #[test]
    fn known_scope_mismatches_and_concurrent_old_sessions_are_rejected() {
        for change in 0..5 {
            let mut fixture = Fixture::new();
            for record in &mut fixture.records {
                match change {
                    0 => record.details.scope_mismatch = true,
                    1 => record.details.account_id = Some("other-account".into()),
                    2 => record.details.model_provider = Some("custom".into()),
                    3 => {
                        record.details.session_started_at =
                            Some(fixture.ledger[0].started_at - Duration::seconds(1))
                    }
                    _ => record.details.session_started_at = None,
                };
            }
            assert_eq!(fixture.estimate().status, CodexEstimateStatus::InvalidScope);
        }
        let mut fixture = Fixture::new();
        for sample in &mut fixture.observations {
            sample.bucket_source = "wham_limit_id".into();
            sample.bucket_id = Some("codex-model-bucket".into());
        }
        assert_eq!(fixture.estimate().status, CodexEstimateStatus::InvalidScope);
        for record in &mut fixture.records {
            record.details.quota_limit_id = Some("codex-model-bucket".into());
        }
        assert_eq!(
            fixture.estimate().status,
            CodexEstimateStatus::LocalEstimate
        );
        let mut fixture = Fixture::new();
        fixture.ledger.push(CodexUsageActivation {
            account_name: "other".into(),
            account_id: "other-account".into(),
            started_at: fixture.observations[2].network_acquired_at + Duration::seconds(10),
        });
        assert_eq!(fixture.estimate().status, CodexEstimateStatus::InvalidScope);
        for config in [
            "forced_login_method='api'",
            "model_provider='custom'",
            "model_provider='openai'\n[model_providers.openai]\nbase_url='https://api.openai.com/v1'",
            "invalid=[",
        ] {
            assert!(!config_route_supported(config));
        }
        assert!(config_route_supported("model='gpt-6.1-sol'"));
    }

    #[test]
    fn freshness_reset_and_joint_constraints_are_independent() {
        let fixture = Fixture::new();
        let five = fixture.estimate();
        let mut seven = five.clone();
        seven.token_remaining = Some(CodexEstimateRange {
            median: 2_000_000.0,
            min: 1_000_000.0,
            max: 3_000_000.0,
        });
        seven.usd_remaining = Some(CodexEstimateRange {
            median: 1.0,
            min: 0.5,
            max: 1.5,
        });
        let joint = joint_estimate(&five, &seven);
        close(joint.token_remaining.unwrap().median, 2_000_000.0);
        close(joint.usd_remaining.unwrap().median, 1.0);
        seven.bucket = Some("different-bucket".into());
        assert!(joint_estimate(&five, &seven).token_remaining.is_none());
        seven.bucket = five.bucket.clone();
        seven.pricing_basis = Some("different-price".into());
        let joint = joint_estimate(&five, &seven);
        assert!(joint.token_remaining.is_some());
        assert!(joint.usd_remaining.is_none());
        seven.pricing_basis = five.pricing_basis.clone();
        seven.status = CodexEstimateStatus::Unstable;
        seven.token_remaining = None;
        let joint = joint_estimate(&five, &seven);
        assert!(joint.token_remaining.is_none());
        assert!(joint.usd_remaining.is_some());
        let mut stale = Fixture::new();
        stale.as_of = stale.observations[3].network_acquired_at + Duration::minutes(5);
        assert_eq!(stale.estimate().status, CodexEstimateStatus::Stale);
        assert!(stale.estimate().token_total.is_some());
        assert!(stale.estimate().token_remaining.is_none());
        let mut reset = Fixture::new();
        for sample in &mut reset.observations {
            sample.resets_at = Some(reset.as_of.timestamp());
        }
        assert_eq!(reset.estimate().status, CodexEstimateStatus::Stale);
    }

    #[test]
    fn quota_write_warning_and_new_acquisition_hide_only_current_capacity() {
        let fixture = Fixture::new();
        let estimate = fixture.estimate();
        let snapshot = CodexAuthUsageSnapshot {
            account_id: Some("stable-account".into()),
            five_hour: estimate.clone(),
            seven_day: estimate,
            ..Default::default()
        };
        let provenance = crate::models::CodexQuotaProvenance {
            account_id: snapshot.account_id.clone(),
            network_acquired_at: fixture.observations[3].network_acquired_at,
            ..Default::default()
        };
        let quota = crate::models::CodexQuota {
            hourly_percentage: 50,
            hourly_reset_time: None,
            hourly_window_minutes: Some(300),
            hourly_window_present: Some(true),
            weekly_percentage: 50,
            weekly_reset_time: None,
            weekly_window_minutes: Some(10080),
            weekly_window_present: Some(true),
            plan_type: Some("plus".into()),
            raw_data: None,
        };
        let mut current = crate::models::CodexAccountQuota {
            quota: Some(quota),
            observation: Some(provenance),
            ..Default::default()
        };
        assert!(
            snapshot
                .with_quota(&current)
                .five_hour
                .token_remaining
                .is_some()
        );
        current.observation.as_mut().unwrap().history_warning =
            Some("quota_history_write_failed".into());
        let warned = snapshot.with_quota(&current);
        assert_eq!(warned.five_hour.status, CodexEstimateStatus::HistoryError);
        assert!(warned.five_hour.token_remaining.is_none());
        assert!(warned.five_hour.token_total.is_some());
        assert!(warned.history_warning.is_some());
        current.observation.as_mut().unwrap().history_warning = None;
        current.observation.as_mut().unwrap().network_acquired_at += Duration::seconds(1);
        assert_eq!(
            snapshot.with_quota(&current).five_hour.status,
            CodexEstimateStatus::Stale
        );
        current.observation.as_mut().unwrap().network_acquired_at -= Duration::seconds(1);
        current.quota.as_mut().unwrap().hourly_window_present = None;
        assert_eq!(
            snapshot.with_quota(&current).five_hour.status,
            CodexEstimateStatus::UnsupportedWindow
        );
        let displayed = snapshot
            .for_display(fixture.observations[3].network_acquired_at + Duration::minutes(5));
        assert_eq!(displayed.five_hour.status, CodexEstimateStatus::Stale);
        assert!(displayed.five_hour.token_remaining.is_none());
    }

    #[test]
    fn activation_attribution_preserves_unattributed_and_global_fallback() {
        let fixture = Fixture::new();
        let start = fixture.as_of - Duration::hours(1);
        let ledger = vec![
            CodexUsageActivation {
                account_name: "a".into(),
                account_id: "a".into(),
                started_at: start,
            },
            CodexUsageActivation {
                account_name: "b".into(),
                account_id: "b".into(),
                started_at: start + Duration::minutes(10),
            },
        ];
        let records = [
            start - Duration::seconds(1),
            start,
            start + Duration::minutes(9),
            start + Duration::minutes(10),
            start + Duration::minutes(11),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, time)| usage_record(time, "unknown", index))
        .collect::<Vec<_>>();
        assert_eq!(records_for_account(&records, "a", &ledger).len(), 2);
        assert_eq!(records_for_account(&records, "b", &ledger).len(), 2);
        let scan = CodexUsageScan {
            records,
            completed_at: fixture.as_of,
            ..Default::default()
        };
        let registry = CodexAuthRegistry {
            usage_ledger: ledger,
            ..Default::default()
        };
        let snapshot =
            CodexUsageEstimationService::project(&scan, &registry, "unsaved", fixture.as_of);
        assert_eq!(snapshot.scope, CodexUsageScope::GlobalFallback);
        assert_eq!(snapshot.unattributed_records, 1);
        assert_eq!(snapshot.rolling.all_time.total_requests, 5);
        assert!(snapshot.five_hour.token_remaining.is_none());
    }

    fn registry_account(id: &str) -> crate::models::CodexAuthAccount {
        serde_json::from_value(
            serde_json::json!({"account_id":id,"saved_at":"2023-11-14T00:00:00Z"}),
        )
        .unwrap()
    }

    #[test]
    fn selected_api_account_and_original_registry_remain_separate() {
        let fixture = Fixture::new();
        let mut registry = CodexAuthRegistry {
            usage_ledger: fixture.ledger.clone(),
            ..Default::default()
        };
        registry
            .accounts
            .insert("main".into(), registry_account("stable-account"));
        assert!(selected_account_supported(&registry, "main"));
        let original = toml::to_string(&registry).unwrap();
        for change in 0..3 {
            let mut registry = registry.clone();
            let account = registry.accounts.get_mut("main").unwrap();
            match change {
                0 => account.auth_method = Some(crate::models::OpenAiAuthMethod::Api),
                1 => account.api_base_url = Some("https://api.openai.com/v1".into()),
                _ => account.api_provider_name = Some("custom".into()),
            };
            let snapshot = CodexUsageEstimationService::project(
                &fixture.scan,
                &registry,
                "main",
                fixture.as_of,
            );
            assert_eq!(snapshot.five_hour.status, CodexEstimateStatus::InvalidScope);
            assert_eq!(
                snapshot.seven_day.usd_status,
                CodexEstimateStatus::InvalidScope
            );
            assert_eq!(snapshot.rolling.all_time.total_tokens(), 3_000_000);
        }
        let restored: CodexAuthRegistry = toml::from_str(&original).unwrap();
        assert_eq!(toml::to_string(&restored).unwrap(), original);
        assert_eq!(restored.usage_ledger.len(), 1);
    }

    #[test]
    fn twenty_nonoverlapping_intervals_and_half_open_endpoint() {
        let mut fixture = Fixture::new();
        let start = fixture.as_of - Duration::minutes(21);
        fixture.observations = (0..=20)
            .map(|index| {
                let mut value =
                    observation(start + Duration::minutes(index), index as f64 * 5.0, "scan");
                value.resets_at = Some((fixture.as_of + Duration::days(1)).timestamp());
                value
            })
            .collect();
        fixture.records = (0..20)
            .map(|index| {
                let mut record = fixture.records[0].clone();
                record.timestamp = start + Duration::minutes(index) + Duration::seconds(1);
                record.details.session_started_at = Some(start);
                record.details.event_identity = format!("interval:{index}");
                record
            })
            .collect();
        let first = fixture.estimate();
        assert_eq!(first.status, CodexEstimateStatus::LocalEstimate);
        assert_eq!(first.sample_count, 20);
        assert_eq!(first.span_start, Some(start));
        assert_eq!(first.span_end, Some(start + Duration::minutes(20)));
        close(first.token_total.unwrap().median, 200_000.0);
        close(first.token_remaining.unwrap().median, 0.0);
        let mut at_endpoint = fixture.records[0].clone();
        at_endpoint.timestamp = fixture.observations[20].network_acquired_at;
        fixture.records.push(at_endpoint);
        close(fixture.estimate().token_total.unwrap().median, 200_000.0);
    }

    #[test]
    fn late_log_arrival_recomputes_bound_observation_intervals() {
        let env = crate::test_support::TestCodexEnv::new();
        let fixture = Fixture::new();
        let service = CodexUsageEstimationService::new(
            env.codex_dir().to_path_buf(),
            env.ccr_codex_dir().to_path_buf(),
        );
        let mut registry = CodexAuthRegistry {
            usage_ledger: fixture.ledger.clone(),
            ..Default::default()
        };
        registry
            .accounts
            .insert("main".into(), registry_account("stable-account"));
        let mut lines = vec![
            serde_json::json!({"type":"session_meta","payload":{"id":"synthetic","model":"gpt-6.1-sol","timestamp":fixture.observations[0].network_acquired_at.to_rfc3339()}}),
        ];
        for record in &fixture.records {
            lines.push(serde_json::json!({"type":"response.completed","timestamp":record.timestamp.to_rfc3339(),"response":{"id":record.details.event_identity,"usage":{"input_tokens":10000,"output_tokens":0,"cached_input_tokens":8000,"cache_write_input_tokens":400,"reasoning_output_tokens":0}}}));
        }
        let sessions = env.codex_dir().join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        let path = sessions.join("late.jsonl");
        let encode = |lines: &[serde_json::Value]| {
            lines
                .iter()
                .map(serde_json::Value::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        };
        std::fs::write(&path, encode(&lines)).unwrap();
        CodexQuotaObservationStore::with_path(env.ccr_codex_dir().join("quota_observations.json"))
            .record(fixture.observations.clone(), fixture.as_of)
            .unwrap();
        let first = service.load(&registry, "main", fixture.as_of).unwrap();
        close(first.five_hour.token_total.unwrap().median, 10_000_000.0);
        lines.push(serde_json::json!({"type":"response.completed","timestamp":(fixture.observations[0].network_acquired_at+Duration::minutes(2)).to_rfc3339(),"response":{"id":"late-record","usage":{"input_tokens":10000,"output_tokens":0,"cached_input_tokens":8000,"cache_write_input_tokens":400,"reasoning_output_tokens":0}}}));
        std::fs::write(&path, encode(&lines)).unwrap();
        let updated = service.load(&registry, "main", fixture.as_of).unwrap();
        close(updated.five_hour.token_total.unwrap().median, 10_100_000.0);
        close(updated.five_hour.usd_total.unwrap().median, 5.05);
        assert_eq!(updated.five_hour.sample_count, 3);
    }
}
