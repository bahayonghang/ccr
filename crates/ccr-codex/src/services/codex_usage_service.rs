//! Local Codex usage metadata. Input/output totals include their cache/reasoning subsets.

use super::codex_usage_estimation::{CodexCostSummary, price_record};
use ccr_core::core::error::Result;
use ccr_core::core::guarded_write::{self, BackupPolicy, WriteOptions};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

const CODEX_USAGE_CACHE_VERSION: u32 = 4;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CodexUsageDetails {
    pub cache_read_tokens: Option<u64>,
    pub cache_write_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
    pub request_input_tokens: Option<u64>,
    pub request_id: Option<String>,
    pub turn_id: Option<String>,
    pub service_tier: Option<String>,
    pub speed: Option<String>,
    pub model_provider: Option<String>,
    pub account_id: Option<String>,
    pub session_started_at: Option<DateTime<Utc>>,
    pub time_basis: String,
    pub measurement_basis: String,
    pub event_identity: String,
    pub parent_session_id: Option<String>,
    pub fork_timestamp: Option<DateTime<Utc>>,
    /// Explicit API/custom-route evidence. Provider labels alone are mutable.
    pub scope_mismatch: bool,
    pub possible_duplicate: bool,
    pub quota_limit_id: Option<String>,
    pub partial: bool,
    pub inconsistent_usage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexUsageRecord {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub model: Option<String>,
    #[serde(default)]
    pub details: CodexUsageDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CodexUsageStatDetails {
    pub cache_read_tokens: Option<u64>,
    pub cache_write_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
    pub cost: CodexCostSummary,
    pub partial: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexUsageStats {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    /// Usage records; only records with request identities represent identified requests.
    pub total_requests: u64,
    pub window_start: Option<DateTime<Utc>>,
    pub window_end: Option<DateTime<Utc>>,
    #[serde(default)]
    pub details: CodexUsageStatDetails,
}
impl CodexUsageStats {
    pub fn total_tokens(&self) -> u64 {
        self.total_input_tokens
            .saturating_add(self.total_output_tokens)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexRollingUsage {
    pub five_hour: CodexUsageStats,
    pub seven_day: CodexUsageStats,
    pub all_time: CodexUsageStats,
    pub by_model: HashMap<String, CodexUsageStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CodexScanDiagnostics {
    pub files: u64,
    pub unreadable_files: u64,
    pub invalid_json: u64,
    pub missing_timestamps: u64,
    pub future_records: u64,
    pub ambiguous_records: u64,
    pub inconsistent_records: u64,
    pub duplicate_records: u64,
    pub cache_warning: bool,
}
impl CodexScanDiagnostics {
    pub fn is_complete(&self) -> bool {
        self.unreadable_files == 0
            && self.invalid_json == 0
            && self.missing_timestamps == 0
            && self.future_records == 0
            && self.ambiguous_records == 0
            && self.inconsistent_records == 0
    }
    fn merge(&mut self, other: &Self) {
        self.unreadable_files += other.unreadable_files;
        self.invalid_json += other.invalid_json;
        self.missing_timestamps += other.missing_timestamps;
        self.ambiguous_records += other.ambiguous_records;
        self.inconsistent_records += other.inconsistent_records;
        self.duplicate_records += other.duplicate_records;
    }
}
#[derive(Debug, Clone, Default)]
pub struct CodexUsageScan {
    pub records: Vec<CodexUsageRecord>,
    pub diagnostics: CodexScanDiagnostics,
    pub completed_at: DateTime<Utc>,
    pub generation: String,
}

#[derive(Debug, Clone, Copy, Default)]
struct TokenUsage {
    input: u64,
    output: u64,
    read: Option<u64>,
    write: Option<u64>,
    reasoning: Option<u64>,
    invalid: bool,
}
impl TokenUsage {
    fn parse(value: &Value) -> Option<Self> {
        let input = value
            .get("input_tokens")
            .or_else(|| value.get("prompt_tokens"))
            .and_then(Value::as_u64);
        let output = value
            .get("output_tokens")
            .or_else(|| value.get("completion_tokens"))
            .and_then(Value::as_u64);
        if input.is_none() && output.is_none() {
            return None;
        }
        let read_value = value
            .pointer("/input_tokens_details/cached_tokens")
            .or_else(|| value.pointer("/prompt_tokens_details/cached_tokens"))
            .or_else(|| value.get("cached_input_tokens"));
        let write_value = value
            .pointer("/input_tokens_details/cache_write_tokens")
            .or_else(|| value.pointer("/prompt_tokens_details/cache_write_tokens"))
            .or_else(|| value.pointer("/input_tokens_details/cache_creation_tokens"))
            .or_else(|| value.pointer("/prompt_tokens_details/cache_creation_tokens"))
            .or_else(|| value.get("cache_write_input_tokens"))
            .or_else(|| value.get("cache_creation_input_tokens"))
            .or_else(|| value.get("cache_write_tokens"))
            .or_else(|| value.get("cache_creation_tokens"));
        let reasoning_value = value
            .pointer("/output_tokens_details/reasoning_tokens")
            .or_else(|| value.pointer("/completion_tokens_details/reasoning_tokens"))
            .or_else(|| value.get("reasoning_output_tokens"));
        let read = read_value.and_then(Value::as_u64);
        let write = write_value.and_then(Value::as_u64);
        let reasoning = reasoning_value.and_then(Value::as_u64);
        let missing = input.is_none() || output.is_none();
        let input = input.unwrap_or(0);
        let output = output.unwrap_or(0);
        let invalid = missing
            || read_value.is_some() && read.is_none()
            || write_value.is_some() && write.is_none()
            || reasoning_value.is_some() && reasoning.is_none()
            || read
                .unwrap_or(0)
                .checked_add(write.unwrap_or(0))
                .is_none_or(|sum| sum > input)
            || reasoning.is_some_and(|q| q > output)
            || value.get("total_tokens").is_some_and(|total| {
                total
                    .as_u64()
                    .is_none_or(|total| input.checked_add(output) != Some(total))
            });
        Some(Self {
            input,
            output,
            read,
            write,
            reasoning,
            invalid,
        })
    }
    fn delta(self, previous: Self) -> Self {
        let diff = |a: Option<u64>, b: Option<u64>| a.zip(b).and_then(|(a, b)| a.checked_sub(b));
        Self {
            input: self.input.saturating_sub(previous.input),
            output: self.output.saturating_sub(previous.output),
            read: diff(self.read, previous.read),
            write: diff(self.write, previous.write),
            reasoning: diff(self.reasoning, previous.reasoning),
            invalid: self.invalid || previous.invalid,
        }
    }
    fn zero_for(self) -> Self {
        Self {
            read: self.read.map(|_| 0),
            write: self.write.map(|_| 0),
            reasoning: self.reasoning.map(|_| 0),
            ..Self::default()
        }
    }
}
pub struct CodexUsageService {
    codex_dir: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CodexUsageCache {
    version: u32,
    files: BTreeMap<String, CachedCodexUsageFile>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CachedCodexUsageFile {
    modified_ns: u128,
    size_bytes: u64,
    records: Vec<CodexUsageRecord>,
    diagnostics: CodexScanDiagnostics,
}

impl CodexUsageService {
    pub fn new(codex_dir: PathBuf) -> Self {
        Self { codex_dir }
    }
    fn usage_cache_path(&self) -> PathBuf {
        self.codex_dir.join(".ccr-codex-usage-cache.json")
    }
    /// No line body or filesystem path enters diagnostic DTOs.
    pub fn scan(&self, as_of: DateTime<Utc>) -> Result<CodexUsageScan> {
        let sessions = self.codex_dir.join("sessions");
        let cache_path = self.usage_cache_path();
        let (cache, cache_warning) = match Self::read_usage_cache(&cache_path) {
            Ok(cache) => (cache, false),
            Err(_) => (CodexUsageCache::default(), true),
        };
        let mut next = CodexUsageCache {
            version: CODEX_USAGE_CACHE_VERSION,
            ..Default::default()
        };
        let mut records = Vec::new();
        let mut diagnostics = CodexScanDiagnostics {
            cache_warning,
            ..Default::default()
        };
        let mut files = Vec::new();
        if sessions.exists() {
            Self::collect_files(&sessions, &mut files, &mut diagnostics);
        }
        files.sort();
        for path in files {
            diagnostics.files += 1;
            let metadata = match std::fs::metadata(&path) {
                Ok(meta) => meta,
                Err(_) => {
                    diagnostics.unreadable_files += 1;
                    continue;
                }
            };
            let modified_ns = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_nanos())
                .unwrap_or(0);
            let size_bytes = metadata.len();
            let relative = path
                .strip_prefix(&sessions)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let parsed = if let Some(cached) = cache.files.get(&relative).filter(|cached| {
                cached.modified_ns == modified_ns && cached.size_bytes == size_bytes
            }) {
                cached.clone()
            } else {
                match self.parse_session_file(&path) {
                    Ok((records, diagnostics)) => CachedCodexUsageFile {
                        modified_ns,
                        size_bytes,
                        records,
                        diagnostics,
                    },
                    Err(_) => {
                        diagnostics.unreadable_files += 1;
                        continue;
                    }
                }
            };
            if std::fs::metadata(&path)
                .map(|meta| {
                    meta.len() != size_bytes || meta.modified().ok() != metadata.modified().ok()
                })
                .unwrap_or(true)
            {
                diagnostics.unreadable_files += 1;
            }
            diagnostics.merge(&parsed.diagnostics);
            records.extend(parsed.records.clone());
            next.files.insert(relative, parsed);
        }
        records.sort_by(|a, b| {
            a.timestamp
                .cmp(&b.timestamp)
                .then(a.details.event_identity.cmp(&b.details.event_identity))
        });
        // A fork timestamp alone does not prove that a child event was copied.
        let parent_records = records.clone();
        let mut unique: BTreeMap<String, CodexUsageRecord> = BTreeMap::new();
        for mut record in records {
            if let Some(parent) = &record.details.parent_session_id {
                let in_prefix = record
                    .details
                    .fork_timestamp
                    .is_some_and(|fork| record.timestamp <= fork);
                let proven_copy = in_prefix
                    && parent_records.iter().any(|candidate| {
                        candidate.session_id == *parent && same_copied_event(candidate, &record)
                    });
                if proven_copy {
                    diagnostics.duplicate_records += 1;
                    continue;
                }
                if in_prefix || record.details.fork_timestamp.is_none() {
                    record.details.partial = true;
                    record.details.possible_duplicate = true;
                    diagnostics.ambiguous_records += 1;
                }
            }
            let key = record.details.event_identity.clone();
            if let Some(existing) = unique.get_mut(&key) {
                diagnostics.duplicate_records += 1;
                if existing.input_tokens != record.input_tokens
                    || existing.output_tokens != record.output_tokens
                    || duplicate_metadata_conflict(existing, &record)
                {
                    existing.details.partial = true;
                    diagnostics.ambiguous_records += 1;
                } else {
                    if existing.model.is_none() {
                        existing.model.clone_from(&record.model);
                    }
                    merge_classification(&mut existing.details, &record.details);
                }
            } else {
                unique.insert(key, record);
            }
        }
        let mut records: Vec<_> = unique.into_values().collect();
        records.sort_by_key(|record| record.timestamp);
        diagnostics.future_records = records
            .iter()
            .filter(|record| record.timestamp > as_of)
            .count() as u64;
        let generation = guarded_write::content_version_token(&serde_json::to_vec(&next)?);
        if Self::write_usage_cache(&cache_path, &next).is_err() {
            diagnostics.cache_warning = true;
        }
        Ok(CodexUsageScan {
            records,
            diagnostics,
            completed_at: Utc::now(),
            generation,
        })
    }
    fn collect_files(dir: &Path, files: &mut Vec<PathBuf>, diagnostics: &mut CodexScanDiagnostics) {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => {
                diagnostics.unreadable_files += 1;
                return;
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    diagnostics.unreadable_files += 1;
                    continue;
                }
            };
            let path = entry.path();
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => Self::collect_files(&path, files, diagnostics),
                Ok(kind)
                    if kind.is_file() && path.extension().is_some_and(|ext| ext == "jsonl") =>
                {
                    files.push(path)
                }
                Ok(kind) if kind.is_symlink() => diagnostics.unreadable_files += 1,
                Err(_) => diagnostics.unreadable_files += 1,
                _ => {}
            }
        }
    }
    pub fn parse_all_logs(&self) -> Result<Vec<CodexUsageRecord>> {
        Ok(self.scan(Utc::now())?.records)
    }

    fn parse_session_file(
        &self,
        path: &Path,
    ) -> Result<(Vec<CodexUsageRecord>, CodexScanDiagnostics)> {
        let reader = BufReader::new(File::open(path)?);
        let mut diagnostics = CodexScanDiagnostics::default();
        let mut records: Vec<CodexUsageRecord> = Vec::new();
        let mut session_id = String::new();
        let mut model = None;
        let mut details = CodexUsageDetails::default();
        let mut previous: Option<TokenUsage> = None;
        let mut high = TokenUsage::default();
        let mut epoch: Option<String> = None;
        let mut ambiguous_epoch = false;
        let mut authoritative: BTreeMap<String, CodexUsageRecord> = BTreeMap::new();
        for (ordinal, line) in reader.lines().enumerate() {
            let line = match line {
                Ok(line) => line,
                Err(_) => {
                    diagnostics.unreadable_files += 1;
                    continue;
                }
            };
            if line.trim().is_empty() {
                continue;
            }
            let json: Value = match serde_json::from_str(&line) {
                Ok(value) => value,
                Err(_) => {
                    diagnostics.invalid_json += 1;
                    continue;
                }
            };
            let top_type = json.get("type").and_then(Value::as_str).unwrap_or("");
            if top_type == "session_meta"
                || (session_id.is_empty() && json.get("session_id").is_some())
            {
                let payload = json.get("payload").unwrap_or(&json);
                session_id = string_field(payload, "id")
                    .or_else(|| string_field(payload, "session_id"))
                    .or_else(|| string_field(&json, "id"))
                    .or_else(|| string_field(&json, "session_id"))
                    .unwrap_or_default();
                model = string_field(payload, "model").or_else(|| string_field(&json, "model"));
                details.session_started_at = parse_time(
                    payload
                        .get("timestamp")
                        .or_else(|| payload.get("created_at"))
                        .or_else(|| json.get("timestamp"))
                        .or_else(|| json.get("created_at")),
                );
                details.parent_session_id = string_field(payload, "forked_from_id")
                    .or_else(|| string_field(payload, "parent_session_id"));
                details.fork_timestamp = parse_time(payload.get("fork_timestamp"));
                details.model_provider = string_field(payload, "model_provider");
                details.account_id = string_field(payload, "account_id");
                details.scope_mismatch = explicit_scope_mismatch(payload);
                details.quota_limit_id = string_field(payload, "limit_id");
                continue;
            }
            let payload = if top_type == "event_msg" {
                json.get("payload").unwrap_or(&json)
            } else {
                json.pointer("/event_msg/payload")
                    .or_else(|| json.get("payload"))
                    .unwrap_or(&json)
            };
            let event_type = payload
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or(top_type);
            if event_type == "turn_context" {
                let next_model = string_field(payload, "model").or(model.clone());
                let next_speed =
                    string_field(payload, "speed").or_else(|| string_field(payload, "mode"));
                if previous.is_some()
                    && (next_model != model
                        || (next_speed.is_some() && next_speed != details.speed))
                {
                    ambiguous_epoch = true;
                }
                model = next_model;
                details.speed = next_speed.or(details.speed);
                details.service_tier =
                    string_field(payload, "service_tier").or(details.service_tier);
                details.turn_id = string_field(payload, "turn_id");
                details.model_provider =
                    string_field(payload, "model_provider").or(details.model_provider);
                details.scope_mismatch |= explicit_scope_mismatch(payload);
                details.quota_limit_id =
                    string_field(payload, "limit_id").or(details.quota_limit_id);
                continue;
            }
            if top_type == "compacted" || event_type == "context_compacted" {
                ambiguous_epoch = true;
                continue;
            }
            let response = payload
                .get("response")
                .or_else(|| json.get("response"))
                .or_else(|| payload.pointer("/data/response"))
                .unwrap_or(payload);
            let is_usage_record = top_type == "token_usage_record";
            let is_completed =
                is_usage_record || matches!(event_type, "turn.completed" | "response.completed");
            let cumulative = payload
                .pointer("/info/total_token_usage")
                .or_else(|| payload.get("total_token_usage"));
            let last_value = payload
                .pointer("/info/last_token_usage")
                .or_else(|| payload.get("last_token_usage"));
            let last_only = !is_completed
                && event_type == "token_count"
                && cumulative.is_none()
                && last_value.is_some()
                && TokenUsage::parse(payload).is_none();
            let usage_value = if is_usage_record {
                // Canonical usage is per response; turn/thread fields are cumulative totals.
                payload.get("usage")
            } else if is_completed {
                response
                    .get("usage")
                    .or_else(|| payload.get("usage"))
                    .or_else(|| payload.pointer("/data/usage"))
            } else if event_type == "token_count" {
                cumulative.or(if last_only { last_value } else { Some(payload) })
            } else {
                None
            };
            let Some(mut usage) = usage_value.and_then(TokenUsage::parse) else {
                let explicit_usage = is_usage_record
                    || (is_completed && usage_value.is_some())
                    || cumulative.is_some()
                    || last_value.is_some()
                    || (event_type == "token_count"
                        && [
                            "input_tokens",
                            "prompt_tokens",
                            "output_tokens",
                            "completion_tokens",
                            "total_tokens",
                            "cached_input_tokens",
                            "cache_write_input_tokens",
                            "cache_creation_input_tokens",
                            "cache_write_tokens",
                            "cache_creation_tokens",
                            "reasoning_output_tokens",
                            "input_tokens_details",
                            "prompt_tokens_details",
                            "output_tokens_details",
                            "completion_tokens_details",
                        ]
                        .iter()
                        .any(|key| payload.get(key).is_some()));
                if explicit_usage {
                    diagnostics.inconsistent_records += 1;
                }
                continue;
            };
            usage.invalid |= last_value
                .is_some_and(|last| TokenUsage::parse(last).is_none_or(|last| last.invalid));
            let record_session_id = if is_usage_record && session_id.is_empty() {
                string_field(payload, "session_id").unwrap_or_default()
            } else {
                session_id.clone()
            };
            let mut record_details = details.clone();
            let timestamp = parse_time(json.get("timestamp").or_else(|| payload.get("timestamp")));
            record_details.time_basis = if timestamp.is_some() {
                "event_timestamp"
            } else {
                "session_timestamp_inferred"
            }
            .into();
            if timestamp.is_none() {
                diagnostics.missing_timestamps += 1;
                record_details.partial = true;
            }
            let timestamp = timestamp
                .or(details.session_started_at)
                .unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
            record_details.request_id = string_field(response, "response_id")
                .or_else(|| string_field(response, "request_id"))
                .or_else(|| {
                    if event_type == "response.completed" {
                        string_field(response, "id")
                    } else {
                        None
                    }
                })
                .or_else(|| string_field(payload, "response_id"))
                .or_else(|| string_field(payload, "request_id"))
                .or_else(|| {
                    payload.get("info").and_then(|info| {
                        string_field(info, "request_id")
                            .or_else(|| string_field(info, "response_id"))
                    })
                });
            record_details.turn_id = string_field(payload, "turn_id")
                .or_else(|| string_field(&json, "turn_id"))
                .or(record_details.turn_id);
            record_details.speed = string_field(response, "speed")
                .or_else(|| string_field(payload, "speed"))
                .or(record_details.speed);
            record_details.service_tier =
                string_field(response, "service_tier").or(record_details.service_tier);
            record_details.account_id =
                string_field(payload, "account_id").or(record_details.account_id);
            record_details.scope_mismatch |=
                explicit_scope_mismatch(payload) || explicit_scope_mismatch(response);
            record_details.quota_limit_id = string_field(response, "limit_id")
                .or_else(|| string_field(payload, "limit_id"))
                .or(record_details.quota_limit_id);
            let record_model = string_field(response, "model")
                .or_else(|| string_field(payload, "model"))
                .or(model.clone());
            if is_completed || last_only {
                record_details.measurement_basis =
                    if is_usage_record && record_details.request_id.is_none() {
                        record_details.partial = true;
                        record_details.possible_duplicate = true;
                        diagnostics.ambiguous_records += 1;
                        "request_unverified"
                    } else if is_completed && record_details.request_id.is_some() {
                        "request"
                    } else if is_completed {
                        "turn_aggregate"
                    } else if record_details.request_id.is_some() {
                        "request_last_snapshot"
                    } else {
                        record_details.partial = true;
                        record_details.possible_duplicate = true;
                        diagnostics.ambiguous_records += 1;
                        "last_snapshot_unverified"
                    }
                    .into();
                record_details.request_input_tokens =
                    record_details.request_id.as_ref().map(|_| usage.input);
            } else {
                let new_epoch = string_field(payload, "epoch_id");
                if new_epoch.is_some() && new_epoch != epoch {
                    previous = None;
                    high = TokenUsage::default();
                    ambiguous_epoch = false;
                    epoch = new_epoch;
                }
                let prev = previous.unwrap_or_else(|| usage.zero_for());
                if usage.input < prev.input
                    || usage.output < prev.output
                    || usage.read.zip(prev.read).is_some_and(|(a, b)| a < b)
                    || usage.write.zip(prev.write).is_some_and(|(a, b)| a < b)
                    || usage
                        .reasoning
                        .zip(prev.reasoning)
                        .is_some_and(|(a, b)| a < b)
                {
                    ambiguous_epoch = true;
                }
                let current = usage;
                if ambiguous_epoch {
                    usage = current.delta(high);
                    high.input = high.input.max(current.input);
                    high.output = high.output.max(current.output);
                    high.read = high.read.zip(current.read).map(|(a, b)| a.max(b));
                    high.write = high.write.zip(current.write).map(|(a, b)| a.max(b));
                    high.reasoning = high.reasoning.zip(current.reasoning).map(|(a, b)| a.max(b));
                    record_details.partial = true;
                    diagnostics.ambiguous_records += 1;
                } else {
                    usage = current.delta(prev);
                    high = current;
                }
                previous = Some(current);
                record_details.measurement_basis = "cumulative_delta".into();
                if let Some(last) = payload
                    .pointer("/info/last_token_usage")
                    .and_then(TokenUsage::parse)
                    && last.input == usage.input
                    && last.output == usage.output
                    && !ambiguous_epoch
                    && !last.invalid
                {
                    usage = last;
                    record_details.request_input_tokens = Some(last.input);
                    record_details.measurement_basis = "request_delta".into();
                }
            }
            if usage.input == 0 && usage.output == 0 {
                if usage.invalid {
                    diagnostics.inconsistent_records += 1;
                }
                continue;
            }
            record_details.cache_read_tokens = usage.read;
            record_details.cache_write_tokens = usage.write;
            record_details.reasoning_tokens = usage.reasoning;
            record_details.inconsistent_usage = usage.invalid
                || usage
                    .read
                    .unwrap_or(0)
                    .checked_add(usage.write.unwrap_or(0))
                    .is_none_or(|sum| sum > usage.input)
                || usage.reasoning.is_some_and(|q| q > usage.output);
            if record_details.inconsistent_usage {
                diagnostics.inconsistent_records += 1;
                record_details.partial = true;
            }
            if record_session_id.is_empty() {
                record_details.partial = true;
                diagnostics.ambiguous_records += 1;
            }
            let source_id = if record_session_id.is_empty() {
                guarded_write::content_version_token(path.to_string_lossy().as_bytes())
            } else {
                record_session_id.clone()
            };
            let identity = if let Some(id) = &record_details.request_id {
                format!("{source_id}:request:{id}")
            } else if is_completed
                && !is_usage_record
                && let Some(turn) = &record_details.turn_id
            {
                format!("{source_id}:turn:{turn}")
            } else {
                format!("{source_id}:ordinal:{ordinal}")
            };
            record_details.event_identity = identity.clone();
            let record = CodexUsageRecord {
                session_id: record_session_id,
                timestamp,
                input_tokens: usage.input,
                output_tokens: usage.output,
                model: record_model,
                details: record_details,
            };
            if (is_completed || last_only)
                && (record.details.request_id.is_some()
                    || (!is_usage_record && record.details.turn_id.is_some()))
            {
                if let Some(previous) = authoritative.get_mut(&identity) {
                    diagnostics.duplicate_records += 1;
                    let metadata_conflict = duplicate_metadata_conflict(previous, &record);
                    let mut record = record;
                    if metadata_conflict {
                        previous.details.partial = true;
                        record.details.partial = true;
                        diagnostics.ambiguous_records += 1;
                    }
                    if is_completed && previous.details.measurement_basis == "request_last_snapshot"
                    {
                        merge_record_metadata(&mut record, previous);
                        *previous = record;
                    } else if last_only
                        && matches!(
                            previous.details.measurement_basis.as_str(),
                            "request" | "turn_aggregate"
                        )
                    {
                        // A completed event remains authoritative over its earlier snapshot.
                        merge_record_metadata(previous, &record);
                    } else if previous.input_tokens == record.input_tokens
                        && previous.output_tokens == record.output_tokens
                    {
                        if previous.model.is_none() {
                            previous.model.clone_from(&record.model);
                        }
                        merge_classification(&mut previous.details, &record.details);
                    } else {
                        previous.details.partial = true;
                        diagnostics.ambiguous_records += u64::from(!metadata_conflict);
                    }
                } else {
                    authoritative.insert(identity, record);
                }
            } else if !is_completed
                && record.details.request_id.is_some()
                && let Some(previous) = records
                    .iter_mut()
                    .find(|previous| previous.details.event_identity == identity)
            {
                diagnostics.ambiguous_records +=
                    u64::from(merge_record_metadata(previous, &record));
                previous.input_tokens = previous.input_tokens.saturating_add(record.input_tokens);
                previous.output_tokens =
                    previous.output_tokens.saturating_add(record.output_tokens);
                add_classification(&mut previous.details, &record.details);
                previous.timestamp = record.timestamp;
                previous.details.request_input_tokens = Some(previous.input_tokens);
            } else {
                records.push(record);
            }
        }
        // An explicit turn aggregate supersedes all measurements for that turn.
        let turn_aggregates: Vec<_> = authoritative
            .values()
            .filter(|record| record.details.measurement_basis == "turn_aggregate")
            .cloned()
            .collect();
        for aggregate in &turn_aggregates {
            authoritative.retain(|_, candidate| {
                candidate.details.turn_id != aggregate.details.turn_id
                    || candidate.details.request_id.is_none()
            });
        }
        for mut record in authoritative.into_values() {
            if record.details.request_id.is_some()
                && records.iter().any(|candidate| {
                    candidate.details.request_id.is_none()
                        && candidate.details.turn_id.is_some()
                        && candidate.details.turn_id == record.details.turn_id
                })
            {
                // No request identity links the aggregate delta to this completion.
                // Retain the local delta once and expose the uncertain coverage.
                for candidate in records.iter_mut().filter(|candidate| {
                    candidate.details.request_id.is_none()
                        && candidate.details.turn_id == record.details.turn_id
                }) {
                    candidate.details.partial = true;
                    candidate.details.possible_duplicate = true;
                }
                diagnostics.ambiguous_records += 1;
                continue;
            }
            for candidate in &records {
                if record.details.request_id.is_some()
                    && record.details.request_id == candidate.details.request_id
                {
                    // Completion owns Token values, but cannot erase request scope evidence.
                    diagnostics.ambiguous_records +=
                        u64::from(merge_record_metadata(&mut record, candidate));
                }
            }
            records.retain(|candidate| {
                let same_request = record.details.request_id.is_some()
                    && record.details.request_id == candidate.details.request_id;
                let same_turn = record.details.turn_id.is_some()
                    && record.details.turn_id == candidate.details.turn_id
                    && record.details.measurement_basis == "turn_aggregate";
                !(same_request || same_turn)
            });
            if record.details.measurement_basis == "request" {
                record.details.request_input_tokens = Some(record.input_tokens);
            }
            records.push(record);
        }
        // Missing identities leave overlap between counters and completions unproven.
        let unproven_overlap = records.iter().any(|completion| {
            matches!(
                completion.details.measurement_basis.as_str(),
                "request" | "turn_aggregate" | "request_last_snapshot" | "request_unverified"
            ) && records.iter().any(|counter| {
                counter.details.request_id.is_none()
                    && matches!(
                        counter.details.measurement_basis.as_str(),
                        "cumulative_delta" | "request_delta" | "last_snapshot_unverified"
                    )
                    && (completion.details.turn_id.is_none()
                        || counter.details.turn_id.is_none()
                        || completion.details.turn_id == counter.details.turn_id)
            })
        });
        if unproven_overlap {
            for record in &mut records {
                record.details.partial = true;
                record.details.possible_duplicate = true;
            }
            diagnostics.ambiguous_records += 1;
        }
        Ok((records, diagnostics))
    }
    fn read_usage_cache(path: &Path) -> Result<CodexUsageCache> {
        if !path.exists() {
            return Ok(CodexUsageCache::default());
        }
        let value: Value = serde_json::from_slice(&std::fs::read(path)?)?;
        if value.get("version").and_then(Value::as_u64)
            != Some(u64::from(CODEX_USAGE_CACHE_VERSION))
        {
            return Ok(CodexUsageCache::default());
        }
        Ok(serde_json::from_value(value)?)
    }
    fn write_usage_cache(path: &Path, cache: &CodexUsageCache) -> Result<()> {
        guarded_write::write_guarded(
            path,
            &serde_json::to_vec(cache)?,
            &WriteOptions {
                secret: true,
                backup: BackupPolicy::None,
                ..Default::default()
            },
        )
    }
    pub fn compute_rolling_usage(&self) -> Result<CodexRollingUsage> {
        Ok(Self::compute_rolling_usage_for_records(
            &self.parse_all_logs()?,
        ))
    }
    pub fn compute_rolling_usage_for_records(records: &[CodexUsageRecord]) -> CodexRollingUsage {
        Self::compute_rolling_usage_at(records, Utc::now())
    }
    pub fn compute_rolling_usage_at(
        records: &[CodexUsageRecord],
        as_of: DateTime<Utc>,
    ) -> CodexRollingUsage {
        let mut rolling = CodexRollingUsage::default();
        let five_start = as_of - Duration::hours(5);
        let seven_start = as_of - Duration::days(7);
        for record in records.iter().filter(|record| {
            record.timestamp <= as_of
                && (record.details.time_basis.is_empty()
                    || record.details.time_basis == "event_timestamp")
        }) {
            Self::add_to_stats(&mut rolling.all_time, record);
            if record.timestamp >= five_start {
                Self::add_to_stats(&mut rolling.five_hour, record);
            }
            if record.timestamp >= seven_start {
                Self::add_to_stats(&mut rolling.seven_day, record);
            }
            if let Some(model) = &record.model {
                Self::add_to_stats(rolling.by_model.entry(model.clone()).or_default(), record);
            }
            rolling.all_time.window_start = Some(
                rolling
                    .all_time
                    .window_start
                    .map_or(record.timestamp, |old| old.min(record.timestamp)),
            );
        }
        rolling.five_hour.window_start = Some(five_start);
        rolling.five_hour.window_end = Some(as_of);
        rolling.seven_day.window_start = Some(seven_start);
        rolling.seven_day.window_end = Some(as_of);
        rolling.all_time.window_end = Some(as_of);
        rolling
    }
    fn add_to_stats(stats: &mut CodexUsageStats, record: &CodexUsageRecord) {
        let first = stats.total_requests == 0;
        stats.total_input_tokens = stats.total_input_tokens.saturating_add(record.input_tokens);
        stats.total_output_tokens = stats
            .total_output_tokens
            .saturating_add(record.output_tokens);
        stats.total_requests += 1;
        let add_optional = |total: &mut Option<u64>, value: Option<u64>| match (*total, value) {
            (Some(current), Some(value)) => *total = current.checked_add(value),
            (None, Some(value)) if first => *total = Some(value),
            _ => *total = None,
        };
        add_optional(
            &mut stats.details.cache_read_tokens,
            record.details.cache_read_tokens,
        );
        add_optional(
            &mut stats.details.cache_write_tokens,
            record.details.cache_write_tokens,
        );
        add_optional(
            &mut stats.details.reasoning_tokens,
            record.details.reasoning_tokens,
        );
        stats.details.partial |= record.details.partial || record.details.inconsistent_usage;
        stats.details.cost.add(record, price_record(record));
    }
    pub fn format_tokens(tokens: u64) -> String {
        if tokens >= 1_000_000 {
            format!("{:.1}M", tokens as f64 / 1_000_000.0)
        } else if tokens >= 1_000 {
            format!("{:.1}K", tokens as f64 / 1_000.0)
        } else {
            tokens.to_string()
        }
    }
    pub fn get_usage_summary(&self) -> Result<String> {
        let rolling = self.compute_rolling_usage()?;
        Ok(format!(
            "Codex Usage Stats\n5h: {} tokens, {} usage records\n7d: {} tokens, {} usage records\nAll time: {} tokens, {} usage records\n",
            Self::format_tokens(rolling.five_hour.total_tokens()),
            rolling.five_hour.total_requests,
            Self::format_tokens(rolling.seven_day.total_tokens()),
            rolling.seven_day.total_requests,
            Self::format_tokens(rolling.all_time.total_tokens()),
            rolling.all_time.total_requests
        ))
    }
}
fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
fn explicit_scope_mismatch(value: &Value) -> bool {
    let mode = value
        .get("auth_mode")
        .or_else(|| value.get("forced_login_method"))
        .and_then(Value::as_str);
    let api_mode = mode.is_some_and(|mode| !matches!(mode, "chatgpt" | "openai_chatgpt" | "oauth"));
    let custom_route = ["api_base_url", "base_url", "endpoint"]
        .into_iter()
        .filter_map(|key| value.get(key).and_then(Value::as_str))
        .any(|url| !subscription_url(url));
    api_mode || custom_route
}
pub(super) fn subscription_url(value: &str) -> bool {
    reqwest::Url::parse(value).is_ok_and(|url| {
        url.scheme() == "https"
            && url.host_str() == Some("chatgpt.com")
            && (url.path() == "/backend-api" || url.path().starts_with("/backend-api/"))
    })
}
fn same_copied_event(parent: &CodexUsageRecord, child: &CodexUsageRecord) -> bool {
    let parent_key = parent
        .details
        .event_identity
        .strip_prefix(&format!("{}:", parent.session_id));
    let child_key = child
        .details
        .event_identity
        .strip_prefix(&format!("{}:", child.session_id));
    parent_key.is_some()
        && parent_key == child_key
        && parent.timestamp == child.timestamp
        && parent.model == child.model
        && parent.input_tokens == child.input_tokens
        && parent.output_tokens == child.output_tokens
        && parent.details.cache_read_tokens == child.details.cache_read_tokens
        && parent.details.cache_write_tokens == child.details.cache_write_tokens
        && parent.details.reasoning_tokens == child.details.reasoning_tokens
}
fn add_classification(target: &mut CodexUsageDetails, source: &CodexUsageDetails) {
    for (a, b) in [
        (&mut target.cache_read_tokens, source.cache_read_tokens),
        (&mut target.cache_write_tokens, source.cache_write_tokens),
        (&mut target.reasoning_tokens, source.reasoning_tokens),
    ] {
        *a = a.zip(b).and_then(|(a, b)| a.checked_add(b));
    }
    target.inconsistent_usage |= source.inconsistent_usage;
    target.scope_mismatch |= source.scope_mismatch;
}
fn duplicate_metadata_conflict(left: &CodexUsageRecord, right: &CodexUsageRecord) -> bool {
    [
        (left.model.as_deref(), right.model.as_deref()),
        (
            left.details.account_id.as_deref(),
            right.details.account_id.as_deref(),
        ),
        (
            left.details.model_provider.as_deref(),
            right.details.model_provider.as_deref(),
        ),
        (
            left.details.speed.as_deref(),
            right.details.speed.as_deref(),
        ),
        (
            left.details.service_tier.as_deref(),
            right.details.service_tier.as_deref(),
        ),
        (
            left.details.quota_limit_id.as_deref(),
            right.details.quota_limit_id.as_deref(),
        ),
        (
            left.details.turn_id.as_deref(),
            right.details.turn_id.as_deref(),
        ),
    ]
    .into_iter()
    .any(|(left, right)| left.zip(right).is_some_and(|(left, right)| left != right))
        || left.details.scope_mismatch != right.details.scope_mismatch
}
fn parse_time(value: Option<&Value>) -> Option<DateTime<Utc>> {
    let value = value?;
    if let Some(text) = value.as_str() {
        return DateTime::parse_from_rfc3339(text)
            .ok()
            .map(|time| time.with_timezone(&Utc));
    }
    let seconds = value
        .as_i64()
        .or_else(|| value.as_f64().map(|value| value as i64))?;
    DateTime::from_timestamp(seconds, 0)
}
fn merge_record_metadata(target: &mut CodexUsageRecord, source: &CodexUsageRecord) -> bool {
    let conflict = duplicate_metadata_conflict(target, source);
    if target.model.is_none() {
        target.model.clone_from(&source.model);
    }
    merge_metadata(&mut target.details, &source.details);
    target.details.partial |= conflict;
    conflict
}
fn merge_metadata(target: &mut CodexUsageDetails, source: &CodexUsageDetails) {
    for (target, source) in [
        (&mut target.account_id, &source.account_id),
        (&mut target.model_provider, &source.model_provider),
        (&mut target.speed, &source.speed),
        (&mut target.service_tier, &source.service_tier),
        (&mut target.quota_limit_id, &source.quota_limit_id),
        (&mut target.turn_id, &source.turn_id),
    ] {
        if target.is_none() {
            target.clone_from(source);
        }
    }
    target.partial |= source.partial;
    target.scope_mismatch |= source.scope_mismatch;
    target.possible_duplicate |= source.possible_duplicate;
    target.inconsistent_usage |= source.inconsistent_usage;
}
fn merge_classification(target: &mut CodexUsageDetails, source: &CodexUsageDetails) {
    merge_metadata(target, source);
    for (a, b) in [
        (&mut target.cache_read_tokens, source.cache_read_tokens),
        (&mut target.cache_write_tokens, source.cache_write_tokens),
        (&mut target.reasoning_tokens, source.reasoning_tokens),
    ] {
        if a.is_some() && b.is_some() && *a != b {
            target.partial = true;
            target.inconsistent_usage = true;
        } else if a.is_none() {
            *a = b;
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use serde_json::json;

    #[test]
    fn parses_cumulative_deltas_and_preserves_explicit_zero() {
        let _env = crate::test_support::TestCodexEnv::new();
        let temp = tempfile::tempdir().unwrap();
        let sessions = temp.path().join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        let first = DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let second = first + Duration::minutes(1);
        let lines = [
            json!({
                "type": "session_meta",
                "payload": {"id": "session-1", "timestamp": first.to_rfc3339(), "model": "gpt-6.1-sol"}
            }),
            json!({
                "type": "event_msg",
                "timestamp": first.to_rfc3339(),
                "payload": {"type": "token_count", "info": {
                    "total_token_usage": {"input_tokens": 100, "output_tokens": 10, "total_tokens": 110, "input_tokens_details": {"cached_tokens": 20}, "cache_write_input_tokens": 10, "output_tokens_details": {"reasoning_tokens": 5}},
                    "last_token_usage": {"input_tokens": 100, "output_tokens": 10, "total_tokens": 110, "input_tokens_details": {"cached_tokens": 20}, "cache_write_input_tokens": 10, "output_tokens_details": {"reasoning_tokens": 5}}
                }}
            }),
            json!({
                "type": "event_msg",
                "timestamp": second.to_rfc3339(),
                "payload": {"type": "token_count", "info": {
                    "total_token_usage": {"input_tokens": 200, "output_tokens": 20, "total_tokens": 220, "input_tokens_details": {"cached_tokens": 40}, "cache_write_input_tokens": 20, "output_tokens_details": {"reasoning_tokens": 10}},
                    "last_token_usage": {"input_tokens": 100, "output_tokens": 10, "total_tokens": 110, "input_tokens_details": {"cached_tokens": 20}, "cache_write_input_tokens": 10, "output_tokens_details": {"reasoning_tokens": 5}}
                }}
            }),
        ];
        let content = lines
            .iter()
            .map(serde_json::Value::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(sessions.join("rollout.jsonl"), content).unwrap();

        let scan = CodexUsageService::new(temp.path().to_path_buf())
            .scan(second + Duration::minutes(1))
            .unwrap();
        assert_eq!(scan.records.len(), 2);
        assert_eq!(scan.records[0].input_tokens, 100);
        assert_eq!(scan.records[1].input_tokens, 100);
        assert_eq!(scan.records[0].details.cache_read_tokens, Some(20));
        assert_eq!(scan.records[1].details.cache_write_tokens, Some(10));
        let usage = CodexUsageService::compute_rolling_usage_at(&scan.records, second);
        assert_eq!(usage.all_time.total_tokens(), 220);
        assert_eq!(usage.all_time.details.cache_read_tokens, Some(40));
        assert_eq!(usage.all_time.details.reasoning_tokens, Some(10));

        let mut explicit_zero = scan.records[0].clone();
        explicit_zero.details.cache_read_tokens = Some(0);
        explicit_zero.details.cache_write_tokens = Some(0);
        explicit_zero.details.reasoning_tokens = Some(0);
        let zero_usage = CodexUsageService::compute_rolling_usage_at(&[explicit_zero], second);
        assert_eq!(zero_usage.all_time.details.cache_read_tokens, Some(0));
        assert_eq!(zero_usage.all_time.details.cache_write_tokens, Some(0));
        assert_eq!(zero_usage.all_time.details.reasoning_tokens, Some(0));
    }

    #[test]
    fn completed_response_identity_deduplicates_without_dropping_other_turns() {
        let _env = crate::test_support::TestCodexEnv::new();
        let temp = tempfile::tempdir().unwrap();
        let sessions = temp.path().join("sessions");
        std::fs::create_dir_all(&sessions).unwrap();
        let time = DateTime::from_timestamp(1_700_000_000, 0).unwrap();
        let completed = |response_id: &str, input: u64| {
            json!({
                "type": "response.completed",
                "timestamp": time.to_rfc3339(),
                "response": {"id": response_id, "usage": {"input_tokens": input, "output_tokens": 1, "total_tokens": input + 1}}
            })
        };
        let lines = [
            completed("response-a", 100),
            completed("response-a", 100),
            completed("response-b", 50),
        ];
        std::fs::write(
            sessions.join("rollout.jsonl"),
            lines
                .iter()
                .map(serde_json::Value::to_string)
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .unwrap();
        let scan = CodexUsageService::new(temp.path().to_path_buf())
            .scan(time)
            .unwrap();
        assert_eq!(scan.records.len(), 2);
        assert_eq!(
            scan.records
                .iter()
                .map(|record| record.input_tokens)
                .sum::<u64>(),
            150
        );
        assert!(scan.diagnostics.duplicate_records >= 1);
    }

    fn time() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).unwrap()
    }
    fn metadata(id: &str) -> Value {
        json!({"type":"session_meta","payload":{"id":id,"timestamp":time().to_rfc3339(),"model":"gpt-6.1-sol"}})
    }
    fn usage(input: u64, output: u64) -> Value {
        json!({"input_tokens":input,"output_tokens":output,"cached_input_tokens":0,"cache_write_input_tokens":0,"reasoning_output_tokens":0,"total_tokens":input+output})
    }
    fn completed(id: &str, input: u64) -> Value {
        json!({"type":"response.completed","timestamp":time().to_rfc3339(),"response":{"id":id,"model":"gpt-6.1-sol","usage":usage(input,10)}})
    }
    fn cumulative(id: Option<&str>, input: u64) -> Value {
        json!({"type":"event_msg","timestamp":time().to_rfc3339(),"payload":{"type":"token_count","request_id":id,"info":{"total_token_usage":usage(input,input/10)}}})
    }
    fn canonical_usage(session_id: &str, response_id: &str, input: u64) -> Value {
        json!({"type":"token_usage_record","timestamp":time().to_rfc3339(),"payload":{"thread_id":"synthetic-thread","turn_id":"synthetic-turn","session_id":session_id,"root_turn_id":"synthetic-root","response_id":response_id,"usage":usage(input,input/10),"turn_token_usage":usage(1_000,100),"thread_token_usage":usage(10_000,1_000)}})
    }
    fn write_log(dir: &Path, name: &str, lines: &[Value]) -> Vec<u8> {
        let sessions = dir.join("sessions").join("2026/01/15");
        std::fs::create_dir_all(&sessions).unwrap();
        let bytes = lines
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            .into_bytes();
        std::fs::write(sessions.join(name), &bytes).unwrap();
        bytes
    }
    fn scan_lines(lines: &[Value]) -> CodexUsageScan {
        let env = crate::test_support::TestCodexEnv::new();
        write_log(env.codex_dir(), "rollout.jsonl", lines);
        CodexUsageService::new(env.codex_dir().to_path_buf())
            .scan(time() + Duration::hours(1))
            .unwrap()
    }
    fn assert_partial_calibration(scan: &CodexUsageScan) {
        use crate::managers::codex_quota_observation::CodexQuotaObservation;
        use crate::services::codex_usage_estimation::{CodexEstimateStatus, estimate_window};
        let observations = [CodexQuotaObservation {
            account_id: "synthetic-account".into(),
            plan: Some("plus".into()),
            source: "network".into(),
            bucket_source: "wham_main".into(),
            duration_minutes: Some(300),
            used_percent: Some(50.0),
            resets_at: Some((scan.completed_at + Duration::hours(1)).timestamp()),
            reset_generation: Some("reset_at:synthetic".into()),
            network_acquired_at: scan.completed_at,
            scope_supported: true,
            ..Default::default()
        }];
        let estimate = estimate_window(
            &observations,
            scan,
            &scan.records,
            &[],
            "synthetic-account",
            300,
            scan.completed_at,
        );
        assert_eq!(estimate.status, CodexEstimateStatus::PartialUsage);
        assert_eq!(estimate.usd_status, CodexEstimateStatus::PartialUsage);
        assert!(estimate.token_total.is_none());
        assert!(estimate.usd_total.is_none());
    }

    #[test]
    fn inclusive_nested_aliases_zero_missing_and_invalid_classification() {
        let parsed=TokenUsage::parse(&json!({"prompt_tokens":100,"completion_tokens":10,"cached_input_tokens":90,"cache_write_input_tokens":80,"prompt_tokens_details":{"cached_tokens":0,"cache_write_tokens":0},"completion_tokens_details":{"reasoning_tokens":0}})).unwrap();
        assert_eq!(parsed.read, Some(0));
        assert_eq!(parsed.write, Some(0));
        assert_eq!(parsed.reasoning, Some(0));
        assert!(!parsed.invalid);
        let missing = TokenUsage::parse(&json!({"input_tokens":100,"output_tokens":10})).unwrap();
        assert_eq!(missing.read, None);
        assert_eq!(missing.write, None);
        assert_eq!(missing.reasoning, None);
        assert!(!missing.invalid);
        for value in [
            json!({"input_tokens":100,"output_tokens":10,"cached_input_tokens":90,"cache_write_input_tokens":11}),
            json!({"input_tokens":100,"output_tokens":10,"reasoning_output_tokens":11}),
            json!({"input_tokens":100,"output_tokens":10,"total_tokens":111}),
            json!({"input_tokens":100,"output_tokens":10,"cached_input_tokens":-1}),
            json!({"input_tokens":100}),
        ] {
            assert!(TokenUsage::parse(&value).unwrap().invalid);
        }
    }

    #[test]
    fn optional_aggregates_require_all_records_and_preserve_explicit_zero() {
        let mut complete = CodexUsageRecord {
            timestamp: time(),
            input_tokens: 100,
            output_tokens: 10,
            details: CodexUsageDetails {
                cache_read_tokens: Some(0),
                cache_write_tokens: Some(0),
                reasoning_tokens: Some(0),
                ..Default::default()
            },
            ..Default::default()
        };
        let missing = CodexUsageRecord {
            timestamp: time(),
            input_tokens: 100,
            output_tokens: 10,
            ..Default::default()
        };
        for records in [
            vec![complete.clone(), missing.clone()],
            vec![missing, complete.clone()],
        ] {
            let stats = CodexUsageService::compute_rolling_usage_at(&records, time()).all_time;
            assert_eq!(stats.details.cache_read_tokens, None);
            assert_eq!(stats.details.cache_write_tokens, None);
            assert_eq!(stats.details.reasoning_tokens, None);
            assert_eq!(stats.total_tokens(), 220);
        }
        complete.details.cache_read_tokens = Some(5);
        let stats = CodexUsageService::compute_rolling_usage_at(&[complete], time()).all_time;
        assert_eq!(stats.details.cache_read_tokens, Some(5));
        assert_eq!(stats.details.cache_write_tokens, Some(0));
    }

    #[test]
    fn mixed_request_authority_nested_response_and_partial_turns() {
        let lines = [
            metadata("mixed"),
            cumulative(Some("r1"), 100),
            json!({"type":"event_msg","timestamp":time().to_rfc3339(),"payload":{"type":"response.completed","response":{"id":"r1","usage":usage(100,10)}}}),
            cumulative(Some("r2"), 150),
        ];
        let scan = scan_lines(&lines);
        assert_eq!(scan.records.len(), 2);
        assert_eq!(
            scan.records
                .iter()
                .map(|record| record.input_tokens + record.output_tokens)
                .sum::<u64>(),
            165
        );
        assert!(scan.diagnostics.is_complete());
        let scan = scan_lines(&[
            metadata("chunks"),
            cumulative(Some("r1"), 100),
            cumulative(Some("r1"), 150),
            completed("r1", 150),
        ]);
        assert_eq!(scan.records.len(), 1);
        assert_eq!(scan.records[0].input_tokens, 150);
        assert_eq!(scan.records[0].output_tokens, 10);
        assert_eq!(scan.records[0].details.request_input_tokens, Some(150));
        let context =
            json!({"type":"turn_context","payload":{"turn_id":"t1","model":"gpt-6.1-sol"}});
        let aggregate = json!({"type":"turn.completed","timestamp":time().to_rfc3339(),"turn_id":"t1","usage":usage(150,15)});
        let scan = scan_lines(&[
            metadata("turn"),
            context,
            cumulative(Some("r1"), 100),
            completed("r1", 100),
            cumulative(Some("r2"), 150),
            aggregate,
        ]);
        assert_eq!(scan.records.len(), 1);
        assert_eq!(scan.records[0].input_tokens, 150);
        assert_eq!(scan.records[0].details.measurement_basis, "turn_aggregate");
    }

    #[test]
    fn repeated_last_snapshots_and_request_chunks_do_not_repeat_tokens() {
        let count = |total, last| json!({"type":"event_msg","timestamp":time().to_rfc3339(),"payload":{"type":"token_count","info":{"total_token_usage":usage(total,total/10),"last_token_usage":usage(last,last/10)}}});
        let scan = scan_lines(&[
            metadata("repeated"),
            count(100, 100),
            count(100, 100),
            count(200, 100),
            count(200, 100),
        ]);
        assert_eq!(scan.records.len(), 2);
        assert_eq!(
            scan.records
                .iter()
                .map(|record| record.input_tokens)
                .sum::<u64>(),
            200
        );
        assert!(
            scan.records
                .iter()
                .all(|record| record.details.request_input_tokens == Some(100))
        );
        let scan = scan_lines(&[
            metadata("chunks"),
            cumulative(Some("r1"), 100),
            cumulative(Some("r1"), 150),
        ]);
        assert_eq!(scan.records.len(), 1);
        assert_eq!(scan.records[0].input_tokens, 150);
        assert_eq!(scan.records[0].details.cache_read_tokens, Some(0));
    }

    #[test]
    fn rollback_restoration_model_change_and_proven_epoch() {
        let scan = scan_lines(&[
            metadata("rollback"),
            cumulative(None, 100),
            cumulative(None, 50),
            cumulative(None, 100),
            cumulative(None, 120),
        ]);
        assert_eq!(
            scan.records
                .iter()
                .map(|record| record.input_tokens)
                .sum::<u64>(),
            120
        );
        assert!(!scan.diagnostics.is_complete());
        assert!(scan.records.last().unwrap().details.partial);
        let context = json!({"type":"turn_context","payload":{"model":"unknown-model"}});
        let scan = scan_lines(&[
            metadata("change"),
            cumulative(None, 100),
            context.clone(),
            cumulative(None, 200),
        ]);
        assert!(scan.records.last().unwrap().details.partial);
        assert_eq!(
            scan.records.last().unwrap().model.as_deref(),
            Some("unknown-model")
        );
        let epoch = json!({"type":"event_msg","timestamp":time().to_rfc3339(),"payload":{"type":"token_count","epoch_id":"new-counter-epoch","info":{"total_token_usage":usage(20,2)}}});
        let scan = scan_lines(&[metadata("epoch"), cumulative(None, 100), context, epoch]);
        assert_eq!(
            scan.records
                .iter()
                .map(|record| record.input_tokens)
                .sum::<u64>(),
            120
        );
        assert!(!scan.records.last().unwrap().details.partial);
    }

    #[test]
    fn same_session_copies_and_equal_independent_requests() {
        let env = crate::test_support::TestCodexEnv::new();
        let lines = [
            metadata("copied"),
            completed("r1", 100),
            completed("r2", 100),
        ];
        write_log(env.codex_dir(), "original.jsonl", &lines);
        write_log(env.codex_dir(), "copy.jsonl", &lines);
        let service = CodexUsageService::new(env.codex_dir().to_path_buf());
        let first = service.scan(time()).unwrap();
        let second = service.scan(time()).unwrap();
        assert_eq!(first.records.len(), 2);
        assert_eq!(first.diagnostics.duplicate_records, 2);
        assert_eq!(first.generation, second.generation);
        assert_eq!(
            first
                .records
                .iter()
                .map(|record| record.input_tokens)
                .sum::<u64>(),
            200
        );
    }

    #[test]
    fn fork_prefix_requires_parent_event_proof() {
        let env = crate::test_support::TestCodexEnv::new();
        let mut child = metadata("child");
        child["payload"]["parent_session_id"] = json!("parent");
        child["payload"]["fork_timestamp"] = json!((time() + Duration::seconds(1)).to_rfc3339());
        write_log(
            env.codex_dir(),
            "child.jsonl",
            &[child.clone(), completed("r1", 100)],
        );
        let service = CodexUsageService::new(env.codex_dir().to_path_buf());
        let scan = service.scan(time()).unwrap();
        assert_eq!(scan.records.len(), 1);
        assert!(scan.records[0].details.possible_duplicate);
        assert!(!scan.diagnostics.is_complete());
        write_log(
            env.codex_dir(),
            "parent.jsonl",
            &[metadata("parent"), completed("r1", 100)],
        );
        let scan = service.scan(time()).unwrap();
        assert_eq!(scan.records.len(), 1);
        assert_eq!(scan.records[0].session_id, "parent");
        assert_eq!(scan.diagnostics.duplicate_records, 1);
        assert!(scan.diagnostics.is_complete());
        write_log(
            env.codex_dir(),
            "child.jsonl",
            &[child, completed("r1", 101)],
        );
        let scan = service.scan(time()).unwrap();
        assert_eq!(scan.records.len(), 2);
        assert!(
            scan.records
                .iter()
                .any(|record| record.details.possible_duplicate)
        );
    }

    #[test]
    fn missing_invalid_future_timestamps_and_utc_rolling_bounds() {
        let mut missing = completed("missing", 100);
        missing.as_object_mut().unwrap().remove("timestamp");
        let mut invalid = completed("invalid", 100);
        invalid["timestamp"] = json!("invalid");
        let mut future = completed("future", 100);
        future["timestamp"] = json!((time() + Duration::hours(2)).to_rfc3339());
        let scan = scan_lines(&[metadata("time"), missing, invalid, future]);
        assert_eq!(scan.diagnostics.missing_timestamps, 2);
        assert_eq!(scan.diagnostics.future_records, 1);
        assert_eq!(
            CodexUsageService::compute_rolling_usage_at(&scan.records, time() + Duration::hours(1))
                .all_time
                .total_requests,
            0
        );
        let records = [
            time() - Duration::days(8),
            time() - Duration::days(7),
            time() - Duration::hours(5),
            time(),
            time() + Duration::seconds(1),
        ]
        .map(|timestamp| CodexUsageRecord {
            timestamp,
            input_tokens: 100,
            output_tokens: 10,
            ..Default::default()
        });
        let rolling = CodexUsageService::compute_rolling_usage_at(&records, time());
        assert_eq!(rolling.five_hour.total_requests, 2);
        assert_eq!(rolling.seven_day.total_requests, 3);
        assert_eq!(rolling.all_time.total_requests, 4);
        assert_eq!(
            parse_time(Some(&json!("2023-11-14T16:13:20-06:00"))),
            Some(time())
        );
    }

    #[test]
    fn old_cache_rebuilds_and_original_bytes_stay_unchanged() {
        let env = crate::test_support::TestCodexEnv::new();
        let bytes = write_log(
            env.codex_dir(),
            "source.jsonl",
            &[metadata("legacy"), completed("r1", 100)],
        );
        let service = CodexUsageService::new(env.codex_dir().to_path_buf());
        let first = service.scan(time()).unwrap();
        assert_eq!(first.records[0].input_tokens, 100);
        let mut cached = CodexUsageService::read_usage_cache(&service.usage_cache_path()).unwrap();
        for version in [2, 3] {
            cached.version = version;
            cached.files.values_mut().next().unwrap().records[0].input_tokens = 999;
            CodexUsageService::write_usage_cache(&service.usage_cache_path(), &cached).unwrap();
            assert_eq!(service.scan(time()).unwrap().records[0].input_tokens, 100);
        }
        assert_eq!(
            std::fs::read(env.codex_dir().join("sessions/2026/01/15/source.jsonl")).unwrap(),
            bytes
        );
        std::fs::write(service.usage_cache_path(), b"invalid-cache").unwrap();
        let scan = service.scan(time()).unwrap();
        assert!(scan.diagnostics.cache_warning);
        assert_eq!(scan.records[0].input_tokens, 100);
    }

    #[test]
    fn legacy_nested_context_empty_sessions_and_invalid_lines() {
        let env = crate::test_support::TestCodexEnv::new();
        let service = CodexUsageService::new(env.codex_dir().to_path_buf());
        assert!(service.parse_all_logs().unwrap().is_empty());
        write_log(
            env.codex_dir(),
            "legacy.jsonl",
            &[
                json!({"session_id":"legacy","model":"codex-mini-latest","created_at":time().to_rfc3339()}),
                json!({"event_msg":{"payload":{"type":"turn_context","model":"o3"}}}),
                json!({"event_msg":{"payload":{"type":"token_count","input_tokens":500,"output_tokens":100}}}),
            ],
        );
        let scan = service.scan(time()).unwrap();
        assert_eq!(scan.records.len(), 1);
        assert_eq!(scan.records[0].model.as_deref(), Some("o3"));
        assert_eq!(scan.records[0].input_tokens, 500);
        assert_eq!(scan.diagnostics.missing_timestamps, 1);
        std::fs::write(
            env.codex_dir().join("sessions/bad.jsonl"),
            b"invalid synthetic JSON\n",
        )
        .unwrap();
        assert_eq!(service.scan(time()).unwrap().diagnostics.invalid_json, 1);
        assert_eq!(CodexUsageService::format_tokens(500), "500");
        assert_eq!(CodexUsageService::format_tokens(1500), "1.5K");
        assert_eq!(CodexUsageService::format_tokens(1_500_000), "1.5M");
    }

    #[test]
    fn explicit_api_route_evidence_is_preserved_without_url_or_key() {
        let mut meta = metadata("api");
        meta["payload"]["auth_mode"] = json!("api_key");
        meta["payload"]["api_base_url"] = json!("https://api.openai.com/v1?synthetic-secret");
        let scan = scan_lines(&[meta, completed("r1", 100)]);
        assert!(scan.records[0].details.scope_mismatch);
        assert!(
            !serde_json::to_string(&scan.records)
                .unwrap()
                .contains("synthetic-secret")
        );
    }

    #[test]
    fn last_only_snapshots_need_identity_and_completed_event_wins() {
        let last = |id: Option<&str>, input| json!({"type":"event_msg","timestamp":time().to_rfc3339(),"payload":{"type":"token_count","request_id":id,"info":{"last_token_usage":usage(input,10)}}});
        let scan = scan_lines(&[
            metadata("last"),
            last(Some("r1"), 100),
            last(Some("r1"), 100),
        ]);
        assert_eq!(scan.records.len(), 1);
        assert_eq!(scan.records[0].input_tokens, 100);
        assert_eq!(
            scan.records[0].details.measurement_basis,
            "request_last_snapshot"
        );
        assert!(scan.diagnostics.is_complete());
        let scan = scan_lines(&[
            metadata("last-completed"),
            last(Some("r1"), 100),
            completed("r1", 150),
            last(Some("r1"), 100),
        ]);
        assert_eq!(scan.records.len(), 1);
        assert_eq!(scan.records[0].input_tokens, 150);
        assert_eq!(scan.records[0].details.measurement_basis, "request");
        assert!(!scan.records[0].details.partial);
        let scan = scan_lines(&[metadata("anonymous-last"), last(None, 100), last(None, 100)]);
        assert_eq!(scan.records.len(), 2);
        assert!(
            scan.records
                .iter()
                .all(|record| record.details.partial && record.details.possible_duplicate)
        );
        assert!(!scan.diagnostics.is_complete());
    }

    #[test]
    fn uncorrelated_counter_and_request_completion_are_partial() {
        let scan = scan_lines(&[
            metadata("unproven"),
            cumulative(None, 100),
            completed("r1", 100),
        ]);
        assert!(!scan.diagnostics.is_complete());
        assert!(
            scan.records
                .iter()
                .all(|record| record.details.partial && record.details.possible_duplicate)
        );
        let scan = scan_lines(&[
            metadata("separate-turns"),
            json!({"type":"turn_context","payload":{"turn_id":"t1"}}),
            cumulative(None, 100),
            json!({"type":"turn_context","payload":{"turn_id":"t2"}}),
            json!({"type":"turn.completed","turn_id":"t2","timestamp":time().to_rfc3339(),"usage":usage(50,10)}),
        ]);
        assert!(scan.diagnostics.is_complete());
        assert_eq!(scan.records.len(), 2);
    }

    #[test]
    fn canonical_usage_records_count_only_requests_and_reuse_session_identity() {
        let scan = scan_lines(&[
            json!({"type":"turn_context","payload":{"model":"gpt-6.1-sol"}}),
            canonical_usage("canonical-session", "r1", 100),
            canonical_usage("canonical-session", "r1", 100),
            canonical_usage("canonical-session", "r2", 50),
        ]);
        assert!(scan.diagnostics.is_complete());
        assert_eq!(scan.records.len(), 2);
        let stats = CodexUsageService::compute_rolling_usage_at(&scan.records, time()).all_time;
        assert_eq!(stats.total_tokens(), 165);
        assert_eq!(stats.details.cache_read_tokens, Some(0));
        for record in &scan.records {
            assert_eq!(record.session_id, "canonical-session");
            assert_eq!(record.model.as_deref(), Some("gpt-6.1-sol"));
            assert_eq!(record.details.turn_id.as_deref(), Some("synthetic-turn"));
            assert_eq!(record.details.measurement_basis, "request");
            assert_eq!(
                record.details.request_input_tokens,
                Some(record.input_tokens)
            );
        }
    }

    #[test]
    fn canonical_usage_mixed_with_legacy_and_copies_uses_request_authority() {
        let env = crate::test_support::TestCodexEnv::new();
        let lines = [
            metadata("canonical-mixed"),
            cumulative(Some("r1"), 100),
            canonical_usage("canonical-mixed", "r1", 100),
            completed("r1", 100),
            cumulative(Some("r2"), 150),
            canonical_usage("canonical-mixed", "r2", 50),
        ];
        write_log(env.codex_dir(), "canonical.jsonl", &lines);
        let mut copy = lines.clone();
        copy[2]["timestamp"] = json!((time() + Duration::seconds(1)).to_rfc3339());
        copy[5]["timestamp"] = json!((time() + Duration::seconds(1)).to_rfc3339());
        write_log(env.codex_dir(), "canonical-copy.jsonl", &copy);
        let service = CodexUsageService::new(env.codex_dir().to_path_buf());
        let scan = service.scan(time() + Duration::hours(1)).unwrap();
        assert!(scan.diagnostics.is_complete());
        assert_eq!(scan.records.len(), 2);
        assert!(scan.diagnostics.duplicate_records >= 2);
        assert_eq!(
            CodexUsageService::compute_rolling_usage_at(&scan.records, time())
                .all_time
                .total_tokens(),
            165
        );
        assert!(scan.records.iter().all(|record| {
            record.details.measurement_basis == "request" && !record.details.partial
        }));
        let second = service.scan(time() + Duration::hours(1)).unwrap();
        assert_eq!(scan.generation, second.generation);
        assert_eq!(
            serde_json::to_value(&scan.records).unwrap(),
            serde_json::to_value(&second.records).unwrap()
        );
    }

    #[test]
    fn canonical_usage_invalid_identity_time_and_overlap_remain_partial() {
        let scan = scan_lines(&[
            metadata("canonical-overlap"),
            cumulative(None, 100),
            canonical_usage("canonical-overlap", "r1", 100),
        ]);
        assert!(!scan.diagnostics.is_complete());
        assert!(
            scan.records
                .iter()
                .all(|record| record.details.partial && record.details.possible_duplicate)
        );
        let mut unidentified = canonical_usage("canonical-unidentified", "r1", 100);
        unidentified["payload"]
            .as_object_mut()
            .unwrap()
            .remove("response_id");
        let scan = scan_lines(&[unidentified.clone(), unidentified]);
        assert_eq!(scan.records.len(), 2);
        assert!(!scan.diagnostics.is_complete());
        assert!(scan.records.iter().all(|record| {
            record.details.partial && record.details.measurement_basis == "request_unverified"
        }));
        let mut missing_time = canonical_usage("canonical-time", "r1", 100);
        missing_time.as_object_mut().unwrap().remove("timestamp");
        let scan = scan_lines(&[missing_time]);
        assert_eq!(scan.diagnostics.missing_timestamps, 1);
        assert!(scan.records[0].details.partial);
        assert_eq!(
            CodexUsageService::compute_rolling_usage_at(&scan.records, time())
                .five_hour
                .total_tokens(),
            0
        );
        let mut invalid = canonical_usage("canonical-invalid", "r1", 100);
        invalid["payload"]["usage"]["reasoning_output_tokens"] = json!(11);
        let scan = scan_lines(&[invalid]);
        assert_eq!(scan.diagnostics.inconsistent_records, 1);
        assert!(scan.records[0].details.inconsistent_usage);
        let mut aggregate_only = canonical_usage("canonical-aggregate", "r1", 100);
        aggregate_only["payload"]
            .as_object_mut()
            .unwrap()
            .remove("usage");
        let scan = scan_lines(&[aggregate_only]);
        assert!(scan.records.is_empty());
        assert_eq!(scan.diagnostics.inconsistent_records, 1);
    }

    #[test]
    fn compacted_rollout_marks_counter_epoch_without_consuming_payload() {
        let scan = scan_lines(&[
            metadata("canonical-compaction"),
            cumulative(None, 100),
            json!({"type":"compacted","timestamp":time().to_rfc3339(),"payload":{"type":"token_usage_record","message":"synthetic-compaction-body","usage":usage(900_000,90_000)}}),
            cumulative(None, 120),
        ]);
        assert_eq!(scan.records.len(), 2);
        assert_eq!(scan.records[1].input_tokens, 20);
        assert!(scan.records[1].details.partial);
        assert!(!scan.diagnostics.is_complete());
        assert_eq!(
            CodexUsageService::compute_rolling_usage_at(&scan.records, time())
                .all_time
                .total_tokens(),
            132
        );
        assert!(
            !serde_json::to_string(&scan.records)
                .unwrap()
                .contains("synthetic-compaction-body")
        );
    }

    #[test]
    fn canonical_usage_turn_aggregate_remains_authoritative() {
        let scan = scan_lines(&[
            metadata("canonical-turn"),
            canonical_usage("canonical-turn", "r1", 100),
            canonical_usage("canonical-turn", "r2", 50),
            json!({"type":"turn.completed","turn_id":"synthetic-turn","timestamp":time().to_rfc3339(),"usage":usage(150,15)}),
        ]);
        assert_eq!(scan.records.len(), 1);
        assert_eq!(scan.records[0].details.measurement_basis, "turn_aggregate");
        assert_eq!(scan.records[0].input_tokens, 150);
        assert!(scan.diagnostics.is_complete());
    }

    #[test]
    fn malformed_usage_objects_keep_scan_partial_without_fake_zero_records() {
        let env = crate::test_support::TestCodexEnv::new();
        let mut lines = vec![metadata("malformed-usage"), completed("valid", 100)];
        for (index, value) in [
            Value::Null,
            json!({}),
            json!({"input_tokens":null,"output_tokens":null}),
            json!({"input_tokens":-1,"output_tokens":-1}),
            json!({"input_tokens":"invalid","output_tokens":"invalid"}),
            json!({"input_tokens":0,"output_tokens":0,"total_tokens":null}),
            json!({"input_tokens":0,"output_tokens":0,"total_tokens":-1}),
            json!({"input_tokens":0,"output_tokens":0,"cached_input_tokens":-1}),
            json!({"input_tokens":0,"output_tokens":0,"reasoning_output_tokens":1}),
        ]
        .into_iter()
        .enumerate()
        {
            let mut record = canonical_usage("malformed-usage", &format!("invalid-{index}"), 0);
            record["payload"]["usage"] = value;
            lines.push(record);
        }
        lines.extend([
            json!({"type":"event_msg","timestamp":time().to_rfc3339(),"payload":{"type":"token_count","info":{"total_token_usage":null}}}),
            json!({"type":"event_msg","timestamp":time().to_rfc3339(),"payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":-1,"output_tokens":null}}}}),
            json!({"type":"event_msg","timestamp":time().to_rfc3339(),"payload":{"type":"token_count","input_tokens":null,"output_tokens":null}}),
        ]);
        let bytes = write_log(env.codex_dir(), "malformed.jsonl", &lines);
        let service = CodexUsageService::new(env.codex_dir().to_path_buf());
        let scan = service.scan(time() + Duration::hours(1)).unwrap();
        assert_eq!(scan.records.len(), 1);
        assert_eq!(scan.records[0].input_tokens, 100);
        assert_eq!(scan.diagnostics.inconsistent_records, 12);
        assert!(!scan.diagnostics.is_complete());
        assert_partial_calibration(&scan);
        assert_eq!(
            std::fs::read(env.codex_dir().join("sessions/2026/01/15/malformed.jsonl")).unwrap(),
            bytes
        );
        drop(env);
        let quota_only = scan_lines(&[
            metadata("quota-only"),
            json!({"type":"event_msg","timestamp":time().to_rfc3339(),"payload":{"type":"token_count","info":null}}),
            json!({"type":"event_msg","timestamp":time().to_rfc3339(),"payload":{"type":"token_count","info":{"rate_limits":{"remaining":50}}}}),
        ]);
        assert!(quota_only.records.is_empty());
        assert!(quota_only.diagnostics.is_complete());
    }

    #[test]
    fn duplicate_request_metadata_conflicts_are_partial_without_recounting() {
        let context = json!({"type":"turn_context","payload":{"model":"gpt-6.1-sol","model_provider":"openai","speed":"standard","turn_id":"synthetic-turn"}});
        for (field, value) in [
            ("model", "unknown-model"),
            ("account_id", "different-account"),
            ("model_provider", "custom-provider"),
            ("speed", "fast"),
        ] {
            let mut first = canonical_usage("metadata-conflict", "r1", 100);
            first["payload"]["account_id"] = json!("synthetic-account");
            let mut next_context = context.clone();
            let mut second = first.clone();
            if field == "account_id" {
                second["payload"][field] = json!(value);
            } else {
                next_context["payload"][field] = json!(value);
            }
            let scan = scan_lines(&[
                metadata("metadata-conflict"),
                context.clone(),
                first.clone(),
                next_context.clone(),
                second.clone(),
            ]);
            assert_eq!(scan.records.len(), 1);
            assert_eq!(scan.records[0].input_tokens, 100);
            assert!(scan.records[0].details.partial);
            assert!(!scan.diagnostics.is_complete());
            assert_partial_calibration(&scan);
            let env = crate::test_support::TestCodexEnv::new();
            write_log(
                env.codex_dir(),
                "original.jsonl",
                &[metadata("metadata-conflict"), context.clone(), first],
            );
            write_log(
                env.codex_dir(),
                "conflicting-copy.jsonl",
                &[metadata("metadata-conflict"), next_context, second],
            );
            let scan = CodexUsageService::new(env.codex_dir().to_path_buf())
                .scan(time() + Duration::hours(1))
                .unwrap();
            assert_eq!(scan.records.len(), 1);
            assert_eq!(scan.records[0].input_tokens, 100);
            assert!(scan.records[0].details.partial);
            assert_partial_calibration(&scan);
        }
    }

    #[test]
    fn duplicate_request_metadata_supplements_preserve_visible_scope_and_speed() {
        let mut supplement = canonical_usage("metadata-supplement", "r1", 100);
        supplement["payload"]["account_id"] = json!("explicit-account");
        supplement["payload"]["speed"] = json!("fast");
        let scan = scan_lines(&[
            metadata("metadata-supplement"),
            canonical_usage("metadata-supplement", "r1", 100),
            supplement,
        ]);
        assert_eq!(scan.records.len(), 1);
        assert!(scan.diagnostics.is_complete());
        assert_eq!(
            scan.records[0].details.account_id.as_deref(),
            Some("explicit-account")
        );
        assert_eq!(scan.records[0].details.speed.as_deref(), Some("fast"));
    }

    #[test]
    fn same_request_counter_metadata_conflicts_remain_partial() {
        for (field, value) in [
            ("account_id", "different-account"),
            ("speed", "fast"),
            ("service_tier", "priority"),
            ("limit_id", "different-bucket"),
        ] {
            let mut first = cumulative(Some("r1"), 100);
            first["payload"]["account_id"] = json!("synthetic-account");
            first["payload"]["speed"] = json!("standard");
            first["payload"]["service_tier"] = json!("default");
            first["payload"]["limit_id"] = json!("synthetic-bucket");
            let mut second = cumulative(Some("r1"), 150);
            for key in ["account_id", "speed", "service_tier", "limit_id"] {
                second["payload"][key] = first["payload"][key].clone();
            }
            second["payload"][field] = json!(value);
            let scan = scan_lines(&[metadata("counter-conflict"), first, second]);
            assert_eq!(scan.records.len(), 1, "{field}");
            assert_eq!(scan.records[0].input_tokens, 150, "{field}");
            assert!(scan.records[0].details.partial, "{field}");
            assert!(!scan.diagnostics.is_complete(), "{field}");
            assert_partial_calibration(&scan);
        }
    }

    #[test]
    fn same_request_counter_metadata_supplements_are_preserved() {
        let mut supplement = cumulative(Some("r1"), 150);
        supplement["payload"]["account_id"] = json!("synthetic-account");
        supplement["payload"]["speed"] = json!("fast");
        supplement["payload"]["service_tier"] = json!("default");
        supplement["payload"]["limit_id"] = json!("synthetic-bucket");
        let scan = scan_lines(&[
            metadata("counter-supplement"),
            cumulative(Some("r1"), 100),
            supplement,
        ]);
        assert_eq!(scan.records.len(), 1);
        assert_eq!(scan.records[0].input_tokens, 150);
        assert!(scan.diagnostics.is_complete());
        let details = &scan.records[0].details;
        assert_eq!(details.account_id.as_deref(), Some("synthetic-account"));
        assert_eq!(details.speed.as_deref(), Some("fast"));
        assert_eq!(details.service_tier.as_deref(), Some("default"));
        assert_eq!(details.quota_limit_id.as_deref(), Some("synthetic-bucket"));
    }

    #[test]
    fn completed_authority_preserves_counter_metadata_and_scope_evidence() {
        for explicit_api_scope in [false, true] {
            let mut counter = cumulative(Some("r1"), 100);
            counter["payload"]["account_id"] = json!("synthetic-account");
            counter["payload"]["speed"] = json!("fast");
            counter["payload"]["info"]["total_token_usage"]["cached_input_tokens"] = json!(20);
            if explicit_api_scope {
                counter["payload"]["auth_mode"] = json!("api_key");
            }
            let scan = scan_lines(&[
                metadata("completed-scope"),
                counter,
                canonical_usage("completed-scope", "r1", 150),
            ]);
            assert_eq!(scan.records.len(), 1);
            let record = &scan.records[0];
            assert_eq!(record.input_tokens, 150);
            assert_eq!(record.output_tokens, 15);
            assert_eq!(record.details.measurement_basis, "request");
            assert_eq!(record.details.cache_read_tokens, Some(0));
            assert_eq!(
                record.details.account_id.as_deref(),
                Some("synthetic-account")
            );
            assert_eq!(record.details.speed.as_deref(), Some("fast"));
            assert_eq!(record.details.scope_mismatch, explicit_api_scope);
            if explicit_api_scope {
                assert!(record.details.partial);
                assert_partial_calibration(&scan);
            } else {
                assert!(scan.diagnostics.is_complete());
            }
        }
    }

    #[test]
    #[ignore]
    fn benchmark_compute_rolling_usage_cache() {
        let env = crate::test_support::TestCodexEnv::new();
        for file_count in [300, 2_000, 10_000] {
            let dir = env.codex_dir().join(file_count.to_string());
            let service = CodexUsageService::new(dir.clone());
            for index in 0..file_count {
                write_log(
                    &dir,
                    &format!("rollout-{index}.jsonl"),
                    &[
                        metadata(&format!("benchmark-{index}")),
                        completed("r1", 100),
                        completed("r2", 200),
                    ],
                );
            }
            let cold_start = std::time::Instant::now();
            let cold = service.compute_rolling_usage().unwrap();
            let cold_elapsed = cold_start.elapsed();
            let warm_start = std::time::Instant::now();
            let warm = service.compute_rolling_usage().unwrap();
            let warm_elapsed = warm_start.elapsed();
            assert_eq!(cold.all_time.total_requests, (file_count * 2) as u64);
            assert_eq!(warm.all_time.total_requests, cold.all_time.total_requests);
            eprintln!(
                "codex_compute_rolling_usage: files={file_count}, cold={cold_elapsed:?}, warm={warm_elapsed:?}"
            );
        }
    }
}
