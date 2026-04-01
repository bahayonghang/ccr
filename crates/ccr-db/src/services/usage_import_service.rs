// Usage import service
// Implements incremental import pipeline for usage logs
// Tracks per-file offsets and hashes for efficient import

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::database::{self, repositories::usage_repo};

/// Import configuration
#[derive(Debug, Clone)]
pub struct ImportConfig {
    /// Maximum lines to process per source per request
    pub max_lines_per_source: usize,
    /// Soft time budget in seconds
    pub time_budget_secs: u64,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            max_lines_per_source: 5000,
            time_budget_secs: 2,
        }
    }
}

/// Import result statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub platform: String,
    pub files_processed: usize,
    pub records_imported: usize,
    pub records_skipped: usize,
    pub duration_ms: u64,
    pub completed: bool,
}

/// Usage import service
pub struct UsageImportService {
    config: ImportConfig,
}

#[derive(Debug, Clone, Default)]
struct CodexSessionMeta {
    session_id: String,
    model: Option<String>,
    created_at: Option<DateTime<Utc>>,
    cwd: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
struct CodexTokenUsage {
    input_tokens: u64,
    cached_input_tokens: u64,
    output_tokens: u64,
}

#[derive(Debug, Clone)]
struct CodexAppendCheckpoint {
    session_id: String,
    model: Option<String>,
    project_path: String,
    last_line_number: u64,
    prefers_turn_completed: bool,
    last_cumulative_usage: CodexTokenUsage,
}

#[derive(Debug, Clone, Copy, Default)]
struct GeminiTokenUsage {
    input_tokens: i64,
    cached_input_tokens: i64,
    output_tokens: i64,
}

impl UsageImportService {
    pub fn new(config: ImportConfig) -> Self {
        Self { config }
    }

    pub fn list_usage_files(&self, platform: &str) -> Result<Vec<PathBuf>, String> {
        let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
        let projects_dir = match platform {
            "claude" => home_dir.join(".claude/projects"),
            "codex" => {
                let codex_home = std::env::var("CODEX_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| home_dir.join(".codex"));
                codex_home.join("sessions")
            }
            "gemini" => home_dir.join(".gemini/tmp"),
            _ => return Err(format!("Unsupported platform: {}", platform)),
        };

        if !projects_dir.exists() {
            return Ok(Vec::new());
        }

        Ok(WalkDir::new(&projects_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| match platform {
                "gemini" => Self::is_gemini_session_file(entry.path()),
                _ => entry.path().extension().is_some_and(|ext| ext == "jsonl"),
            })
            .map(|entry| entry.path().to_path_buf())
            .collect())
    }

    pub fn import_file_path(
        &self,
        platform: &str,
        file_path: &Path,
    ) -> Result<(usize, usize), String> {
        self.import_file(platform, file_path)
    }

    /// Import usage data for a platform incrementally
    pub fn import_platform(&self, platform: &str) -> Result<ImportResult, String> {
        let start = Instant::now();
        let time_budget = Duration::from_secs(self.config.time_budget_secs);
        let usage_files = self.list_usage_files(platform)?;

        debug!(
            "Found {} usage files for platform {}",
            usage_files.len(),
            platform
        );

        let mut total_imported = 0;
        let mut total_skipped = 0;
        let mut files_processed = 0;
        let mut completed = true;

        for file_path in &usage_files {
            // Check time budget
            if start.elapsed() > time_budget {
                debug!("Time budget exceeded, stopping import");
                completed = false;
                break;
            }

            match self.import_file(platform, file_path) {
                Ok((imported, skipped)) => {
                    total_imported += imported;
                    total_skipped += skipped;
                    files_processed += 1;
                }
                Err(e) => {
                    warn!("Failed to import file {:?}: {}", file_path, e);
                }
            }
        }

        info!(
            "Import for {} complete: {} files, {} imported, {} skipped",
            platform, files_processed, total_imported, total_skipped
        );

        Ok(ImportResult {
            platform: platform.to_string(),
            files_processed,
            records_imported: total_imported,
            records_skipped: total_skipped,
            duration_ms: start.elapsed().as_millis() as u64,
            completed,
        })
    }

    /// Import a single file incrementally
    fn import_file(&self, platform: &str, file_path: &Path) -> Result<(usize, usize), String> {
        let file_path_str = file_path.to_str().ok_or("Invalid file path")?.to_string();
        let is_codex = platform == "codex";
        let is_gemini_session = platform == "gemini" && Self::is_gemini_session_file(file_path);
        let current_file_size = std::fs::metadata(file_path)
            .map(|metadata| metadata.len() as i64)
            .unwrap_or(0);

        // Calculate current file hash (first 4KB for efficiency)
        let current_hash = self.calculate_file_hash(file_path)?;

        // Check if we have a source record for this file
        let existing_source =
            database::with_connection(|conn| usage_repo::get_source_by_path(conn, &file_path_str))
                .map_err(|e| e.to_string())?;

        let mut codex_append_checkpoint = None;
        let (source_id, start_offset) = match existing_source {
            Some(source) => {
                if is_codex {
                    if current_file_size < source.last_offset {
                        debug!("Codex session shrank, re-importing: {:?}", file_path);
                        database::with_connection(|conn| {
                            usage_repo::delete_records_by_source(conn, &source.id)
                        })
                        .map_err(|e| e.to_string())?;
                        (source.id, 0i64)
                    } else if current_file_size == source.last_offset {
                        if source.file_hash == current_hash {
                            return Ok((0, 0));
                        }
                        debug!(
                            "Codex session changed in-place, re-importing: {:?}",
                            file_path
                        );
                        database::with_connection(|conn| {
                            usage_repo::delete_records_by_source(conn, &source.id)
                        })
                        .map_err(|e| e.to_string())?;
                        (source.id, 0i64)
                    } else {
                        let stable_prefix_len =
                            std::cmp::min(source.last_offset.max(0) as usize, 4096);
                        let prefix_hash =
                            self.calculate_file_hash_with_limit(file_path, stable_prefix_len)?;

                        if source.file_hash == prefix_hash {
                            codex_append_checkpoint = self
                                .load_codex_append_checkpoint(&source.id)
                                .map_err(|e| e.to_string())?;
                            (source.id, source.last_offset)
                        } else {
                            debug!(
                                "Codex session prefix changed, re-importing from start: {:?}",
                                file_path
                            );
                            database::with_connection(|conn| {
                                usage_repo::delete_records_by_source(conn, &source.id)
                            })
                            .map_err(|e| e.to_string())?;
                            (source.id, 0i64)
                        }
                    }
                } else if source.file_hash != current_hash {
                    // File changed, need to re-import from beginning
                    debug!("File hash changed, re-importing: {:?}", file_path);
                    database::with_connection(|conn| {
                        usage_repo::delete_records_by_source(conn, &source.id)
                    })
                    .map_err(|e| e.to_string())?;
                    (source.id, 0i64)
                } else if is_gemini_session {
                    if source.last_offset >= current_file_size {
                        return Ok((0, 0));
                    }
                    debug!("Session file grew, re-importing: {:?}", file_path);
                    database::with_connection(|conn| {
                        usage_repo::delete_records_by_source(conn, &source.id)
                    })
                    .map_err(|e| e.to_string())?;
                    (source.id, 0i64)
                } else {
                    // Claude/Gemini: continue from last offset
                    (source.id, source.last_offset)
                }
            }
            None => {
                // New file, create source record
                (Uuid::new_v4().to_string(), 0i64)
            }
        };

        // Extract project path from file path
        let project_path = codex_append_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.project_path.clone())
            .unwrap_or_else(|| self.extract_project_path(file_path, platform));

        // Read and parse: Codex/Gemini session 使用专用解析器，其他平台用逐行解析
        let (records, new_offset, skipped) =
            if let Some(checkpoint) = codex_append_checkpoint.as_ref() {
                let (records, parsed_offset, skipped) = self.read_codex_session_from_offset(
                    file_path,
                    start_offset,
                    &source_id,
                    checkpoint,
                )?;
                let next_offset = if records.is_empty() {
                    start_offset
                } else {
                    parsed_offset
                };
                (records, next_offset, skipped)
            } else if is_codex {
                self.read_codex_session(file_path, &project_path, &source_id)?
            } else if is_gemini_session {
                self.read_gemini_session(file_path, &project_path, &source_id)?
            } else {
                self.read_lines_from_offset(
                    file_path,
                    start_offset,
                    platform,
                    &project_path,
                    &source_id,
                )?
            };

        let imported = records.len();

        if !records.is_empty() {
            // Insert records into database
            database::with_connection(|conn| usage_repo::insert_records_batch(conn, &records))
                .map_err(|e| e.to_string())?;
        }

        // Update source record
        let source = usage_repo::UsageSource {
            id: source_id,
            platform: platform.to_string(),
            file_path: file_path_str,
            file_hash: current_hash,
            last_offset: new_offset,
            updated_at: Utc::now(),
        };

        database::with_connection(|conn| usage_repo::upsert_source(conn, &source))
            .map_err(|e| e.to_string())?;

        Ok((imported, skipped))
    }

    fn load_codex_append_checkpoint(
        &self,
        source_id: &str,
    ) -> Result<Option<CodexAppendCheckpoint>, String> {
        let records =
            database::with_connection(|conn| usage_repo::get_records_by_source(conn, source_id))
                .map_err(|e| e.to_string())?;
        let latest = records
            .into_iter()
            .filter_map(|record| {
                Self::parse_record_line_number(&record.id).map(|line_number| (line_number, record))
            })
            .max_by_key(|(line_number, _)| *line_number);

        let Some((last_line_number, latest_record)) = latest else {
            return Ok(None);
        };

        let session_id = latest_record
            .id
            .rsplit_once(':')
            .map(|(prefix, _)| prefix.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let parsed_json = serde_json::from_str::<Value>(&latest_record.record_json).ok();
        let prefers_turn_completed = parsed_json
            .as_ref()
            .and_then(|json| json.get("type"))
            .and_then(|value| value.as_str())
            == Some("turn.completed");

        let last_cumulative_usage = if prefers_turn_completed {
            CodexTokenUsage::default()
        } else {
            parsed_json
                .as_ref()
                .and_then(Self::extract_codex_event_payload)
                .map(Self::extract_codex_token_usage)
                .unwrap_or_default()
        };

        Ok(Some(CodexAppendCheckpoint {
            session_id,
            model: latest_record.model,
            project_path: latest_record.project_path,
            last_line_number,
            prefers_turn_completed,
            last_cumulative_usage,
        }))
    }

    /// Read lines from a file starting at offset
    fn read_lines_from_offset(
        &self,
        file_path: &Path,
        offset: i64,
        platform: &str,
        project_path: &str,
        source_id: &str,
    ) -> Result<(Vec<usage_repo::UsageRecord>, i64, usize), String> {
        let file = File::open(file_path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);

        // Seek to offset
        reader
            .seek(SeekFrom::Start(offset as u64))
            .map_err(|e| e.to_string())?;

        let mut records = Vec::new();
        let mut current_offset = offset;
        let mut lines_processed = 0;
        let mut skipped = 0;

        loop {
            // Check line limit
            if lines_processed >= self.config.max_lines_per_source {
                debug!("Reached line limit for file {:?}", file_path);
                break;
            }

            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line).map_err(|e| e.to_string())?;

            if bytes_read == 0 {
                // EOF
                break;
            }

            current_offset += bytes_read as i64;
            lines_processed += 1;

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Parse JSON
            match serde_json::from_str::<Value>(line) {
                Ok(json) => {
                    // Check if this is a valid usage record
                    if let Some(record) =
                        self.parse_usage_record(&json, platform, project_path, source_id)
                    {
                        records.push(record);
                    } else {
                        skipped += 1;
                    }
                }
                Err(_) => {
                    skipped += 1;
                }
            }
        }

        Ok((records, current_offset, skipped))
    }

    /// Read and parse a Codex session JSONL file
    ///
    /// Codex session files use an event-stream format:
    ///   Line 1: session metadata (session_id, model, created_at)
    ///   Subsequent: event stream (turn_context, token_count, turn.completed)
    ///
    /// token_count values are cumulative — delta calculation is required.
    /// turn.completed events contain per-turn absolute usage.
    /// When both exist, turn.completed is preferred to avoid double-counting.
    fn read_codex_session(
        &self,
        file_path: &Path,
        project_path: &str,
        source_id: &str,
    ) -> Result<(Vec<usage_repo::UsageRecord>, i64, usize), String> {
        let file = File::open(file_path).map_err(|e| e.to_string())?;
        let file_size = file.metadata().map(|m| m.len() as i64).unwrap_or(0);
        let reader = BufReader::new(file);

        let mut token_count_records = Vec::new();
        let mut turn_completed_records = Vec::new();
        let mut skipped = 0usize;

        // Session state
        let mut session_id = String::from("unknown");
        let mut current_model: Option<String> = None;
        let mut created_at: Option<DateTime<Utc>> = None;
        let mut prev_input_tokens: u64 = 0;
        let mut prev_cached_input_tokens: u64 = 0;
        let mut prev_output_tokens: u64 = 0;
        let mut is_first_line = true;
        let mut line_number = 0u64;
        let mut resolved_project_path = project_path.to_string();

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };

            line_number += 1;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let json: Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };

            if is_first_line {
                is_first_line = false;
                // 第1行: 会话元数据
                let meta = Self::parse_codex_session_meta(&json);
                if !meta.session_id.is_empty() {
                    session_id = meta.session_id;
                }
                current_model = meta.model;
                created_at = meta.created_at;
                if let Some(cwd) = meta.cwd {
                    resolved_project_path = cwd;
                }
                continue;
            }

            // 事件时间戳（回退到 session created_at）
            let event_ts = json
                .get("timestamp")
                .and_then(|v| v.as_str())
                .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .or(created_at)
                .unwrap_or_else(Utc::now);

            // 检查 event_msg.payload 事件
            if let Some(payload) = Self::extract_codex_event_payload(&json) {
                let event_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("");

                match event_type {
                    "turn_context" => {
                        if let Some(model) = payload.get("model").and_then(|v| v.as_str()) {
                            current_model = Some(model.to_string());
                        }
                    }
                    "token_count" => {
                        let usage = Self::extract_codex_token_usage(payload);
                        let cur_input = usage.input_tokens;
                        let cur_cached = usage.cached_input_tokens;
                        let cur_output = usage.output_tokens;

                        let delta_input = cur_input.saturating_sub(prev_input_tokens);
                        let delta_cached = cur_cached.saturating_sub(prev_cached_input_tokens);
                        let delta_output = cur_output.saturating_sub(prev_output_tokens);

                        if delta_input > 0 || delta_output > 0 {
                            let record_id = format!("{}:{}", session_id, line_number);
                            let model_name = current_model.as_deref().unwrap_or("unknown");
                            let cost = self.calculate_cost(
                                model_name,
                                delta_input as i64,
                                delta_output as i64,
                                delta_cached as i64,
                            );

                            token_count_records.push(usage_repo::UsageRecord {
                                id: record_id,
                                platform: "codex".to_string(),
                                project_path: resolved_project_path.clone(),
                                record_json: json.to_string(),
                                recorded_at: event_ts,
                                source_id: source_id.to_string(),
                                model: current_model.clone(),
                                input_tokens: delta_input as i64,
                                output_tokens: delta_output as i64,
                                cache_read_tokens: delta_cached as i64,
                                cost_usd: cost,
                            });
                        }

                        prev_input_tokens = cur_input;
                        prev_cached_input_tokens = cur_cached;
                        prev_output_tokens = cur_output;
                    }
                    _ => {
                        skipped += 1;
                    }
                }
                continue;
            }

            // 检查 turn.completed 事件（--json 模式）
            if json.get("type").and_then(|v| v.as_str()) == Some("turn.completed") {
                if let Some(usage) = json.get("usage") {
                    let input = usage
                        .get("input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let cached = usage
                        .get("cached_input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let output = usage
                        .get("output_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    if input > 0 || output > 0 {
                        let record_id = format!("{}:{}", session_id, line_number);
                        let model_name = current_model.as_deref().unwrap_or("unknown");
                        let cost = self.calculate_cost(
                            model_name,
                            input as i64,
                            output as i64,
                            cached as i64,
                        );

                        turn_completed_records.push(usage_repo::UsageRecord {
                            id: record_id,
                            platform: "codex".to_string(),
                            project_path: resolved_project_path.clone(),
                            record_json: json.to_string(),
                            recorded_at: event_ts,
                            source_id: source_id.to_string(),
                            model: current_model.clone(),
                            input_tokens: input as i64,
                            output_tokens: output as i64,
                            cache_read_tokens: cached as i64,
                            cost_usd: cost,
                        });
                    }
                } else {
                    skipped += 1;
                }
                continue;
            }

            skipped += 1;
        }

        // 去重策略：优先使用 turn.completed，否则用 token_count 增量
        let records = if !turn_completed_records.is_empty() {
            skipped += token_count_records.len();
            turn_completed_records
        } else {
            token_count_records
        };

        Ok((records, file_size, skipped))
    }

    fn read_codex_session_from_offset(
        &self,
        file_path: &Path,
        offset: i64,
        source_id: &str,
        checkpoint: &CodexAppendCheckpoint,
    ) -> Result<(Vec<usage_repo::UsageRecord>, i64, usize), String> {
        let file = File::open(file_path).map_err(|e| e.to_string())?;
        let file_size = file.metadata().map(|m| m.len() as i64).unwrap_or(offset);
        let mut reader = BufReader::new(file);
        reader
            .seek(SeekFrom::Start(offset as u64))
            .map_err(|e| e.to_string())?;

        let mut current_model = checkpoint.model.clone();
        let mut prev_input_tokens = checkpoint.last_cumulative_usage.input_tokens;
        let mut prev_cached_input_tokens = checkpoint.last_cumulative_usage.cached_input_tokens;
        let mut prev_output_tokens = checkpoint.last_cumulative_usage.output_tokens;
        let mut line_number = checkpoint.last_line_number;
        let mut token_count_records = Vec::new();
        let mut turn_completed_records = Vec::new();
        let mut skipped = 0usize;

        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line).map_err(|e| e.to_string())?;
            if bytes_read == 0 {
                break;
            }

            line_number += 1;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let json: Value = match serde_json::from_str(trimmed) {
                Ok(value) => value,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };

            let event_ts = json
                .get("timestamp")
                .and_then(|v| v.as_str())
                .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);

            if let Some(payload) = Self::extract_codex_event_payload(&json) {
                let event_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("");

                match event_type {
                    "turn_context" => {
                        if let Some(model) = payload.get("model").and_then(|v| v.as_str()) {
                            current_model = Some(model.to_string());
                        }
                    }
                    "token_count" if !checkpoint.prefers_turn_completed => {
                        let usage = Self::extract_codex_token_usage(payload);
                        let delta_input = usage.input_tokens.saturating_sub(prev_input_tokens);
                        let delta_cached = usage
                            .cached_input_tokens
                            .saturating_sub(prev_cached_input_tokens);
                        let delta_output = usage.output_tokens.saturating_sub(prev_output_tokens);

                        if delta_input > 0 || delta_output > 0 {
                            let record_id = format!("{}:{}", checkpoint.session_id, line_number);
                            let model_name = current_model.as_deref().unwrap_or("unknown");
                            let cost = self.calculate_cost(
                                model_name,
                                delta_input as i64,
                                delta_output as i64,
                                delta_cached as i64,
                            );

                            token_count_records.push(usage_repo::UsageRecord {
                                id: record_id,
                                platform: "codex".to_string(),
                                project_path: checkpoint.project_path.clone(),
                                record_json: json.to_string(),
                                recorded_at: event_ts,
                                source_id: source_id.to_string(),
                                model: current_model.clone(),
                                input_tokens: delta_input as i64,
                                output_tokens: delta_output as i64,
                                cache_read_tokens: delta_cached as i64,
                                cost_usd: cost,
                            });
                        }

                        prev_input_tokens = usage.input_tokens;
                        prev_cached_input_tokens = usage.cached_input_tokens;
                        prev_output_tokens = usage.output_tokens;
                    }
                    _ => {
                        skipped += 1;
                    }
                }
                continue;
            }

            if json.get("type").and_then(|v| v.as_str()) == Some("turn.completed") {
                if let Some(usage) = json.get("usage") {
                    let input = usage
                        .get("input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let cached = usage
                        .get("cached_input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let output = usage
                        .get("output_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    if input > 0 || output > 0 {
                        let record_id = format!("{}:{}", checkpoint.session_id, line_number);
                        let model_name = current_model.as_deref().unwrap_or("unknown");
                        let cost = self.calculate_cost(
                            model_name,
                            input as i64,
                            output as i64,
                            cached as i64,
                        );

                        turn_completed_records.push(usage_repo::UsageRecord {
                            id: record_id,
                            platform: "codex".to_string(),
                            project_path: checkpoint.project_path.clone(),
                            record_json: json.to_string(),
                            recorded_at: event_ts,
                            source_id: source_id.to_string(),
                            model: current_model.clone(),
                            input_tokens: input as i64,
                            output_tokens: output as i64,
                            cache_read_tokens: cached as i64,
                            cost_usd: cost,
                        });
                    }
                } else {
                    skipped += 1;
                }
                continue;
            }

            skipped += 1;
        }

        let records = if !turn_completed_records.is_empty() {
            skipped += token_count_records.len();
            turn_completed_records
        } else {
            token_count_records
        };

        Ok((records, file_size, skipped))
    }

    fn parse_codex_session_meta(json: &Value) -> CodexSessionMeta {
        let payload = if json.get("type").and_then(|v| v.as_str()) == Some("session_meta") {
            json.get("payload").unwrap_or(json)
        } else {
            json
        };

        let created_at = payload
            .get("timestamp")
            .or_else(|| payload.get("created_at"))
            .or_else(|| json.get("timestamp"))
            .or_else(|| json.get("created_at"))
            .and_then(|v| v.as_str())
            .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
            .map(|dt| dt.with_timezone(&Utc));

        CodexSessionMeta {
            session_id: payload
                .get("id")
                .or_else(|| payload.get("session_id"))
                .or_else(|| json.get("session_id"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            model: payload
                .get("model")
                .or_else(|| json.get("model"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            created_at,
            cwd: payload
                .get("cwd")
                .or_else(|| json.get("cwd"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }
    }

    fn extract_codex_event_payload(json: &Value) -> Option<&Value> {
        if json.get("type").and_then(|v| v.as_str()) == Some("event_msg") {
            json.get("payload")
        } else {
            json.get("event_msg").and_then(|em| em.get("payload"))
        }
    }

    fn parse_record_line_number(record_id: &str) -> Option<u64> {
        record_id
            .rsplit_once(':')
            .and_then(|(_, line_number)| line_number.parse::<u64>().ok())
    }

    fn extract_codex_token_usage(payload: &Value) -> CodexTokenUsage {
        let usage = payload
            .get("info")
            .and_then(|info| info.get("total_token_usage"))
            .unwrap_or(payload);

        CodexTokenUsage {
            input_tokens: usage
                .get("input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            cached_input_tokens: usage
                .get("cached_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            output_tokens: usage
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        }
    }

    /// Read and parse a Gemini CLI session JSON file.
    ///
    /// Gemini CLI stores session transcripts under ~/.gemini/tmp/*/chats/session-*.json.
    /// The assistant-side message objects may contain either:
    /// - tokens.{input,output,cached,...}
    /// - usageMetadata / usage_metadata from API responses
    fn read_gemini_session(
        &self,
        file_path: &Path,
        project_path: &str,
        source_id: &str,
    ) -> Result<(Vec<usage_repo::UsageRecord>, i64, usize), String> {
        let file_size = std::fs::metadata(file_path)
            .map(|metadata| metadata.len() as i64)
            .unwrap_or(0);
        let content = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
        let json: Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

        let session_id = json
            .get("sessionId")
            .or_else(|| json.get("session_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let fallback_ts = json
            .get("lastUpdated")
            .or_else(|| json.get("startTime"))
            .or_else(|| json.get("timestamp"))
            .and_then(|v| v.as_str())
            .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let resolved_project_path = Self::resolve_gemini_project_path(file_path, &json)
            .unwrap_or_else(|| {
                if project_path.is_empty() {
                    String::from("unknown")
                } else {
                    project_path.to_string()
                }
            });

        let mut records = Vec::new();
        let mut skipped = 0usize;

        for (index, message) in json
            .get("messages")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .enumerate()
        {
            if message.get("type").and_then(|v| v.as_str()) != Some("gemini") {
                continue;
            }

            let usage = Self::extract_gemini_token_usage(message);
            if usage.input_tokens == 0 && usage.output_tokens == 0 && usage.cached_input_tokens == 0
            {
                skipped += 1;
                continue;
            }

            let recorded_at = message
                .get("timestamp")
                .and_then(|v| v.as_str())
                .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .or(fallback_ts)
                .unwrap_or_else(Utc::now);

            let model = message
                .get("model")
                .or_else(|| json.get("model"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let cost = self.calculate_cost(
                model.as_deref().unwrap_or("unknown"),
                usage.input_tokens,
                usage.output_tokens,
                usage.cached_input_tokens,
            );

            let record_id = message
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{}:{}", session_id, index + 1));

            records.push(usage_repo::UsageRecord {
                id: record_id,
                platform: "gemini".to_string(),
                project_path: resolved_project_path.clone(),
                record_json: message.to_string(),
                recorded_at,
                source_id: source_id.to_string(),
                model,
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
                cache_read_tokens: usage.cached_input_tokens,
                cost_usd: cost,
            });
        }

        Ok((records, file_size, skipped))
    }

    fn is_gemini_session_file(path: &Path) -> bool {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            return false;
        };
        let Some(parent_name) = path.parent().and_then(|parent| parent.file_name()) else {
            return false;
        };
        file_name.starts_with("session-")
            && path.extension().is_some_and(|ext| ext == "json")
            && parent_name == "chats"
    }

    fn extract_gemini_token_usage(message: &Value) -> GeminiTokenUsage {
        if let Some(tokens) = message.get("tokens") {
            let input_tokens = tokens
                .get("input")
                .or_else(|| tokens.get("input_tokens"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let cached_input_tokens = tokens
                .get("cached")
                .or_else(|| tokens.get("cached_input_tokens"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let output_tokens = tokens
                .get("output")
                .or_else(|| tokens.get("output_tokens"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            if input_tokens > 0 || cached_input_tokens > 0 || output_tokens > 0 {
                return GeminiTokenUsage {
                    input_tokens,
                    cached_input_tokens,
                    output_tokens,
                };
            }
        }

        let usage = message
            .get("usageMetadata")
            .or_else(|| message.get("usage_metadata"))
            .or_else(|| {
                message
                    .get("response")
                    .and_then(|response| response.get("usageMetadata"))
            })
            .or_else(|| {
                message
                    .get("response")
                    .and_then(|response| response.get("usage_metadata"))
            });

        GeminiTokenUsage {
            input_tokens: usage
                .and_then(|v| v.get("promptTokenCount"))
                .or_else(|| usage.and_then(|v| v.get("inputTokenCount")))
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            cached_input_tokens: usage
                .and_then(|v| v.get("cachedContentTokenCount"))
                .or_else(|| usage.and_then(|v| v.get("cachedInputTokenCount")))
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            output_tokens: usage
                .and_then(|v| v.get("candidatesTokenCount"))
                .or_else(|| usage.and_then(|v| v.get("outputTokenCount")))
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
        }
    }

    fn resolve_gemini_project_path(file_path: &Path, session_json: &Value) -> Option<String> {
        if let Some(project_root) = Self::read_project_root_marker(file_path) {
            return Some(project_root);
        }

        if let Some(project_root) = Self::resolve_gemini_project_from_config(file_path) {
            return Some(project_root);
        }

        Self::infer_gemini_project_from_messages(session_json)
    }

    fn read_project_root_marker(file_path: &Path) -> Option<String> {
        for ancestor in file_path.ancestors() {
            let marker = ancestor.join(".project_root");
            if marker.is_file() {
                let content = fs::read_to_string(marker).ok()?;
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
        None
    }

    fn resolve_gemini_project_from_config(file_path: &Path) -> Option<String> {
        let project_dir_name = file_path
            .parent()
            .and_then(|parent| parent.parent())
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())?;
        let projects_path = dirs::home_dir()?.join(".gemini/projects.json");
        let projects_json: Value =
            serde_json::from_str(&fs::read_to_string(projects_path).ok()?).ok()?;
        let projects = projects_json.get("projects")?.as_object()?;

        projects.iter().find_map(|(path, alias)| {
            let alias = alias.as_str()?;
            if alias.eq_ignore_ascii_case(project_dir_name) {
                Some(path.to_string())
            } else {
                None
            }
        })
    }

    fn infer_gemini_project_from_messages(session_json: &Value) -> Option<String> {
        let mut paths = Vec::new();

        for message in session_json
            .get("messages")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            for tool_call in message
                .get("toolCalls")
                .and_then(|v| v.as_array())
                .into_iter()
                .flatten()
            {
                let Some(args) = tool_call.get("args") else {
                    continue;
                };

                for key in ["file_path", "cwd", "path"] {
                    if let Some(path) = args.get(key).and_then(|v| v.as_str())
                        && Self::looks_like_absolute_path(path)
                    {
                        paths.push(PathBuf::from(path));
                    }
                }

                if let Some(path_array) = args.get("paths").and_then(|v| v.as_array()) {
                    for path in path_array.iter().filter_map(|value| value.as_str()) {
                        if Self::looks_like_absolute_path(path) {
                            paths.push(PathBuf::from(path));
                        }
                    }
                }
            }
        }

        let common = Self::common_path_prefix(&paths)?;
        Some(common.to_string_lossy().to_string())
    }

    fn looks_like_absolute_path(path: &str) -> bool {
        Path::new(path).is_absolute()
            || path.starts_with("\\\\")
            || path
                .chars()
                .nth(1)
                .is_some_and(|separator| separator == ':')
    }

    fn common_path_prefix(paths: &[PathBuf]) -> Option<PathBuf> {
        let mut components = paths.first()?.components().collect::<Vec<_>>();

        for path in &paths[1..] {
            let path_components = path.components().collect::<Vec<_>>();
            let shared_len = components
                .iter()
                .zip(path_components.iter())
                .take_while(|(left, right)| left == right)
                .count();
            components.truncate(shared_len);
            if components.is_empty() {
                return None;
            }
        }

        if components.is_empty() {
            return None;
        }

        let mut prefix = PathBuf::new();
        for component in components {
            prefix.push(component.as_os_str());
        }
        Some(prefix)
    }

    /// Parse a JSON object into a usage record
    fn parse_usage_record(
        &self,
        json: &Value,
        platform: &str,
        project_path: &str,
        source_id: &str,
    ) -> Option<usage_repo::UsageRecord> {
        // Extract uuid
        let uuid = json.get("uuid").and_then(|v| v.as_str())?;

        // Extract timestamp
        let timestamp_str = json.get("timestamp").and_then(|v| v.as_str())?;
        let recorded_at = DateTime::parse_from_rfc3339(timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()?;

        // Extract model
        let model = json
            .get("model")
            .or_else(|| json.get("message").and_then(|m| m.get("model")))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Check if this has valid usage data
        let usage_obj = json
            .get("usage")
            .or_else(|| json.get("message").and_then(|m| m.get("usage")));

        // Must have at least one token field
        let (input_tokens, output_tokens, cache_read_tokens) = if let Some(usage) = usage_obj {
            let input = usage
                .get("input_tokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let output = usage
                .get("output_tokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let cache = usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            if input == 0 && output == 0 && cache == 0 {
                return None;
            }
            (input, output, cache)
        } else {
            return None;
        };

        // Store the original JSON for flexibility
        let record_json = json.to_string();

        // 计算费用（简化：使用内联定价表）
        let cost_usd = self.calculate_cost(
            model.as_deref().unwrap_or("unknown"),
            input_tokens,
            output_tokens,
            cache_read_tokens,
        );

        Some(usage_repo::UsageRecord {
            id: uuid.to_string(),
            platform: platform.to_string(),
            project_path: project_path.to_string(),
            record_json,
            recorded_at,
            source_id: source_id.to_string(),
            model,
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cost_usd,
        })
    }

    /// 根据模型名称计算费用（每百万 token 定价）
    fn calculate_cost(&self, model: &str, input: i64, output: i64, cache: i64) -> f64 {
        // (input_cost, output_cost, cache_read_cost) per million tokens
        let (ic, oc, cc) = if model.contains("opus") {
            (15.0, 75.0, 1.5)
        } else if model.contains("sonnet") {
            (3.0, 15.0, 0.3)
        } else if model.contains("haiku") {
            (0.8, 4.0, 0.08)
        } else if model.contains("codex-mini") {
            // OpenAI Codex Mini
            (0.15, 0.60, 0.0375)
        } else if model.contains("o4-mini") {
            // OpenAI o4-mini
            (0.55, 2.20, 0.1375)
        } else if model == "o3" || model.starts_with("o3-") {
            // OpenAI o3
            (2.0, 8.0, 0.50)
        } else if model.contains("gpt-5") {
            // OpenAI GPT-5
            (2.0, 8.0, 0.50)
        } else if model.contains("gpt-4") {
            // OpenAI GPT-4 variants (gpt-4, gpt-4.1, etc.)
            (2.0, 8.0, 0.5)
        } else if model.contains("gemini") && model.contains("pro") {
            (1.25, 10.0, 0.315)
        } else if model.contains("gemini") && model.contains("flash") {
            (0.15, 0.6, 0.0375)
        } else {
            (0.0, 0.0, 0.0)
        };
        (input as f64 * ic + output as f64 * oc + cache as f64 * cc) / 1_000_000.0
    }

    /// Calculate file hash (first 4KB for efficiency)
    fn calculate_file_hash(&self, file_path: &Path) -> Result<String, String> {
        self.calculate_file_hash_with_limit(file_path, 4096)
    }

    fn calculate_file_hash_with_limit(
        &self,
        file_path: &Path,
        limit: usize,
    ) -> Result<String, String> {
        let file = File::open(file_path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0u8; limit];

        let bytes_read =
            std::io::Read::read(&mut reader, &mut buffer).map_err(|e| e.to_string())?;

        let mut hasher = Sha256::new();
        hasher.update(&buffer[..bytes_read]);
        let hash = hasher.finalize();

        Ok(hash.iter().map(|byte| format!("{byte:02x}")).collect())
    }

    /// Extract project path from file path
    fn extract_project_path(&self, file_path: &Path, platform: &str) -> String {
        // Try to extract project path from the file path
        let path_str = file_path.to_string_lossy();
        let normalized_path = path_str.replace('\\', "/");

        // Codex: sessions 目录结构 (~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl)
        if platform == "codex" {
            // 尝试匹配 /.codex/sessions/ 或 /sessions/ (CODEX_HOME 场景)
            for marker in &["/.codex/sessions/", "/sessions/"] {
                if let Some(pos) = normalized_path.find(marker) {
                    let after = &normalized_path[pos + marker.len()..];
                    // 提取 YYYY/MM/DD 作为 project path
                    if let Some(end) = after.rfind('/') {
                        return after[..end].to_string();
                    }
                }
            }
        }

        if platform == "gemini" {
            if let Some(project_root) = Self::read_project_root_marker(file_path) {
                return project_root;
            }

            let marker = "/.gemini/tmp/";
            if let Some(pos) = normalized_path.find(marker) {
                let after_marker = &normalized_path[pos + marker.len()..];
                if let Some(end_pos) = after_marker.find('/') {
                    return after_marker[..end_pos].to_string();
                }
            }
        }

        // Claude/Gemini: projects 目录结构
        // e.g., ~/.claude/projects/myproject/usage.jsonl -> myproject
        let marker = format!("/.{}/projects/", platform);
        if let Some(pos) = normalized_path.find(&marker) {
            let after_marker = &normalized_path[pos + marker.len()..];
            if let Some(end_pos) = after_marker.find('/') {
                return after_marker[..end_pos].to_string();
            }
        }

        // Fallback: use parent directory name
        file_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Get cached records from database
    #[allow(dead_code)]
    pub fn get_records(
        &self,
        platform: &str,
        limit: usize,
    ) -> Result<Vec<usage_repo::UsageRecord>, String> {
        database::with_connection(|conn| usage_repo::get_recent_records(conn, platform, limit))
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::database;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup() {
        database::initialize_for_test().unwrap();
    }

    #[test]
    fn test_import_config_default() {
        let config = ImportConfig::default();
        assert_eq!(config.max_lines_per_source, 5000);
        assert_eq!(config.time_budget_secs, 2);
    }

    #[test]
    fn test_extract_project_path() {
        let service = UsageImportService::new(ImportConfig::default());

        let path = PathBuf::from("/home/user/.claude/projects/myproject/usage.jsonl");
        let project = service.extract_project_path(&path, "claude");
        assert_eq!(project, "myproject");
    }

    #[test]
    fn test_calculate_file_hash() {
        setup();

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.jsonl");

        let mut file = File::create(&file_path).unwrap();
        writeln!(
            file,
            r#"{{"uuid": "test", "timestamp": "2025-01-01T00:00:00Z"}}"#
        )
        .unwrap();

        let service = UsageImportService::new(ImportConfig::default());
        let hash = service.calculate_file_hash(&file_path).unwrap();

        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA256 hex
    }

    #[test]
    fn test_parse_usage_record() {
        let service = UsageImportService::new(ImportConfig::default());

        let json: Value = serde_json::from_str(
            r#"{
            "uuid": "abc123",
            "timestamp": "2025-01-19T10:30:00Z",
            "model": "claude-sonnet-4-5",
            "usage": {
                "input_tokens": 100,
                "output_tokens": 50
            }
        }"#,
        )
        .unwrap();

        let record = service.parse_usage_record(&json, "claude", "/project", "source-1");
        assert!(record.is_some());

        let record = record.unwrap();
        assert_eq!(record.id, "abc123");
        assert_eq!(record.platform, "claude");
    }

    #[test]
    fn test_parse_usage_record_nested() {
        let service = UsageImportService::new(ImportConfig::default());

        let json: Value = serde_json::from_str(
            r#"{
            "uuid": "def456",
            "timestamp": "2025-01-19T11:00:00Z",
            "message": {
                "model": "claude-opus-4",
                "usage": {
                    "input_tokens": 200,
                    "output_tokens": 100
                }
            }
        }"#,
        )
        .unwrap();

        let record = service.parse_usage_record(&json, "claude", "/project", "source-1");
        assert!(record.is_some());
    }

    #[test]
    fn test_parse_usage_record_invalid() {
        let service = UsageImportService::new(ImportConfig::default());

        // No usage data
        let json: Value = serde_json::from_str(
            r#"{
            "uuid": "invalid",
            "timestamp": "2025-01-19T10:30:00Z"
        }"#,
        )
        .unwrap();

        let record = service.parse_usage_record(&json, "claude", "/project", "source-1");
        assert!(record.is_none());
    }

    #[test]
    fn test_extract_project_path_codex() {
        let service = UsageImportService::new(ImportConfig::default());

        let path = PathBuf::from("/home/user/.codex/sessions/2026/01/15/rollout-abc123.jsonl");
        let project = service.extract_project_path(&path, "codex");
        assert_eq!(project, "2026/01/15");
    }

    #[test]
    fn test_read_codex_session_token_count() {
        setup();

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("rollout-test.jsonl");

        let content = r#"{"session_id":"sess-1","model":"codex-mini-latest","created_at":"2026-01-15T10:00:00Z","source":"terminal"}
{"event_msg":{"payload":{"type":"token_count","input_tokens":1000,"cached_input_tokens":500,"output_tokens":200}}}
{"event_msg":{"payload":{"type":"token_count","input_tokens":2500,"cached_input_tokens":1000,"output_tokens":500}}}
"#;
        std::fs::write(&file_path, content).unwrap();

        let service = UsageImportService::new(ImportConfig::default());
        let (records, offset, _skipped) = service
            .read_codex_session(&file_path, "2026/01/15", "src-1")
            .unwrap();

        assert_eq!(records.len(), 2);
        // 第一次 delta: from 0
        assert_eq!(records[0].input_tokens, 1000);
        assert_eq!(records[0].output_tokens, 200);
        assert_eq!(records[0].cache_read_tokens, 500);
        assert_eq!(records[0].model.as_deref(), Some("codex-mini-latest"));
        assert_eq!(records[0].id, "sess-1:2");
        // 第二次 delta: from previous cumulative
        assert_eq!(records[1].input_tokens, 1500);
        assert_eq!(records[1].output_tokens, 300);
        assert_eq!(records[1].cache_read_tokens, 500);
        assert_eq!(records[1].id, "sess-1:3");
        // offset 应该等于文件大小
        assert_eq!(offset, content.len() as i64);
    }

    #[test]
    fn test_read_codex_session_turn_completed() {
        setup();

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("rollout-tc.jsonl");

        let content = r#"{"session_id":"sess-2","model":"o4-mini","created_at":"2026-03-16T10:00:00Z"}
{"type":"turn.completed","usage":{"input_tokens":24763,"cached_input_tokens":24448,"output_tokens":122}}
"#;
        std::fs::write(&file_path, content).unwrap();

        let service = UsageImportService::new(ImportConfig::default());
        let (records, _offset, _skipped) = service
            .read_codex_session(&file_path, "2026/03/16", "src-2")
            .unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].input_tokens, 24763);
        assert_eq!(records[0].output_tokens, 122);
        assert_eq!(records[0].cache_read_tokens, 24448);
        assert_eq!(records[0].model.as_deref(), Some("o4-mini"));
    }

    #[test]
    fn test_read_codex_session_turn_context_updates_model() {
        setup();

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("rollout-ctx.jsonl");

        let content = r#"{"session_id":"sess-3","model":"codex-mini-latest","created_at":"2026-03-16T10:00:00Z"}
{"event_msg":{"payload":{"type":"turn_context","model":"o3"}}}
{"event_msg":{"payload":{"type":"token_count","input_tokens":500,"output_tokens":100}}}
"#;
        std::fs::write(&file_path, content).unwrap();

        let service = UsageImportService::new(ImportConfig::default());
        let (records, _offset, _skipped) = service
            .read_codex_session(&file_path, "2026/03/16", "src-3")
            .unwrap();

        assert_eq!(records.len(), 1);
        // model 被 turn_context 更新为 o3
        assert_eq!(records[0].model.as_deref(), Some("o3"));
    }

    #[test]
    fn test_read_codex_session_dedup_prefers_turn_completed() {
        setup();

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("rollout-dedup.jsonl");

        // 同时包含 token_count 和 turn.completed
        let content = r#"{"session_id":"sess-4","model":"codex-mini-latest","created_at":"2026-03-16T10:00:00Z"}
{"event_msg":{"payload":{"type":"token_count","input_tokens":1000,"output_tokens":200}}}
{"type":"turn.completed","usage":{"input_tokens":1000,"output_tokens":200}}
"#;
        std::fs::write(&file_path, content).unwrap();

        let service = UsageImportService::new(ImportConfig::default());
        let (records, _offset, _skipped) = service
            .read_codex_session(&file_path, "2026/03/16", "src-4")
            .unwrap();

        // 应该只返回 turn.completed 记录
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].input_tokens, 1000);
    }

    #[test]
    fn test_read_codex_session_current_format_uses_cwd_and_total_token_usage() {
        setup();

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("rollout-current.jsonl");

        let content = r#"{"timestamp":"2026-03-05T09:11:45.366Z","type":"session_meta","payload":{"id":"sess-current","timestamp":"2026-03-05T09:11:45.366Z","cwd":"D:\\Documents\\Code\\Github\\ccr","model":"gpt-5"}}
{"timestamp":"2026-03-05T09:11:50.406Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":1000,"cached_input_tokens":400,"output_tokens":200}}}}
{"timestamp":"2026-03-05T09:12:01.000Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":1800,"cached_input_tokens":700,"output_tokens":260}}}}
"#;
        std::fs::write(&file_path, content).unwrap();

        let service = UsageImportService::new(ImportConfig::default());
        let (records, _offset, _skipped) = service
            .read_codex_session(&file_path, "2026/03/05", "src-current")
            .unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].project_path, r"D:\Documents\Code\Github\ccr");
        assert_eq!(records[0].input_tokens, 1000);
        assert_eq!(records[0].cache_read_tokens, 400);
        assert_eq!(records[0].output_tokens, 200);
        assert_eq!(records[1].input_tokens, 800);
        assert_eq!(records[1].cache_read_tokens, 300);
        assert_eq!(records[1].output_tokens, 60);
    }

    #[test]
    fn test_import_file_persists_current_format_codex_records() {
        setup();

        let temp_dir = TempDir::new().unwrap();
        let file_dir = temp_dir
            .path()
            .join("sessions")
            .join("2026")
            .join("03")
            .join("05");
        std::fs::create_dir_all(&file_dir).unwrap();
        let file_path = file_dir.join("rollout-persist.jsonl");

        let content = r#"{"timestamp":"2026-03-05T09:11:45.366Z","type":"session_meta","payload":{"id":"sess-persist","timestamp":"2026-03-05T09:11:45.366Z","cwd":"D:\\Documents\\Code\\Github\\ccr","model":"gpt-5"}}
{"timestamp":"2026-03-05T09:11:50.406Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":1000,"cached_input_tokens":400,"output_tokens":200}}}}
"#;
        std::fs::write(&file_path, content).unwrap();

        let service = UsageImportService::new(ImportConfig::default());
        let (imported, skipped) = service.import_file("codex", &file_path).unwrap();

        assert_eq!(imported, 1);
        assert_eq!(skipped, 0);

        let file_path_str = file_path.to_string_lossy().to_string();
        let (source, records, project_stats) = database::with_connection(|conn| {
            let source = usage_repo::get_source_by_path(conn, &file_path_str)?
                .expect("source should exist after import");
            let records = usage_repo::get_records_by_source(conn, &source.id)?;
            let project_stats =
                usage_repo::get_project_stats(conn, &Some("codex".to_string()), &None, &None)?;
            Ok((source, records, project_stats))
        })
        .unwrap();

        assert!(!source.id.is_empty());
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].project_path, r"D:\Documents\Code\Github\ccr");
        assert!(
            project_stats
                .iter()
                .any(|stat| stat.project_path == r"D:\Documents\Code\Github\ccr")
        );
    }

    #[test]
    fn test_import_file_appends_codex_session_tail_without_reimporting_history() {
        setup();

        let temp_dir = TempDir::new().unwrap();
        let file_dir = temp_dir
            .path()
            .join("sessions")
            .join("2026")
            .join("03")
            .join("05");
        std::fs::create_dir_all(&file_dir).unwrap();
        let file_path = file_dir.join("rollout-append.jsonl");

        let initial = r#"{"timestamp":"2026-03-05T09:11:45.366Z","type":"session_meta","payload":{"id":"sess-append","timestamp":"2026-03-05T09:11:45.366Z","cwd":"D:\\Documents\\Code\\Github\\ccr","model":"gpt-5"}}
{"timestamp":"2026-03-05T09:11:50.406Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":1000,"cached_input_tokens":400,"output_tokens":200}}}}
"#;
        std::fs::write(&file_path, initial).unwrap();

        let service = UsageImportService::new(ImportConfig::default());
        let (first_imported, first_skipped) = service.import_file("codex", &file_path).unwrap();
        assert_eq!(first_imported, 1);
        assert_eq!(first_skipped, 0);

        let appended = r#"{"timestamp":"2026-03-05T09:12:01.000Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":1800,"cached_input_tokens":700,"output_tokens":260}}}}
"#;
        {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&file_path)
                .unwrap();
            file.write_all(appended.as_bytes()).unwrap();
        }

        let (second_imported, second_skipped) = service.import_file("codex", &file_path).unwrap();
        assert_eq!(second_imported, 1);
        assert_eq!(second_skipped, 0);

        let file_path_str = file_path.to_string_lossy().to_string();
        let records = database::with_connection(|conn| {
            let source = usage_repo::get_source_by_path(conn, &file_path_str)?
                .expect("source should exist after append import");
            usage_repo::get_records_by_source(conn, &source.id)
        })
        .unwrap();

        assert_eq!(records.len(), 2);
        assert_eq!(records[0].input_tokens, 800);
        assert_eq!(records[0].output_tokens, 60);
        assert_eq!(records[0].cache_read_tokens, 300);
        assert_eq!(records[1].input_tokens, 1000);
        assert_eq!(records[1].output_tokens, 200);
        assert_eq!(records[1].cache_read_tokens, 400);
    }

    #[test]
    fn test_extract_gemini_token_usage_from_usage_metadata() {
        let message: Value = serde_json::from_str(
            r#"{
            "type": "gemini",
            "usageMetadata": {
                "promptTokenCount": 1200,
                "candidatesTokenCount": 320,
                "cachedContentTokenCount": 450
            }
        }"#,
        )
        .unwrap();

        let usage = UsageImportService::extract_gemini_token_usage(&message);
        assert_eq!(usage.input_tokens, 1200);
        assert_eq!(usage.output_tokens, 320);
        assert_eq!(usage.cached_input_tokens, 450);
    }

    #[test]
    fn test_read_gemini_session_uses_tokens_and_project_root() {
        setup();

        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("tmp").join("ccr");
        let chats_dir = project_dir.join("chats");
        std::fs::create_dir_all(&chats_dir).unwrap();
        std::fs::write(
            project_dir.join(".project_root"),
            "D:\\Documents\\Code\\Github\\ccr\n",
        )
        .unwrap();
        let file_path = chats_dir.join("session-2026-03-16T08-00-test.json");

        let content = r#"{
            "sessionId": "gem-sess-1",
            "startTime": "2026-03-16T08:00:00Z",
            "lastUpdated": "2026-03-16T08:10:00Z",
            "messages": [
                {
                    "id": "msg-user",
                    "timestamp": "2026-03-16T08:00:01Z",
                    "type": "user",
                    "content": "hi"
                },
                {
                    "id": "msg-gem-1",
                    "timestamp": "2026-03-16T08:00:05Z",
                    "type": "gemini",
                    "model": "gemini-3-pro-preview",
                    "tokens": {
                        "input": 154528,
                        "output": 1969,
                        "cached": 2048,
                        "thoughts": 11158,
                        "total": 168545
                    }
                }
            ]
        }"#;
        std::fs::write(&file_path, content).unwrap();

        let service = UsageImportService::new(ImportConfig::default());
        let (records, _offset, skipped) = service
            .read_gemini_session(&file_path, "ccr", "src-gemini")
            .unwrap();

        assert_eq!(skipped, 0);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, "msg-gem-1");
        assert_eq!(records[0].platform, "gemini");
        assert_eq!(records[0].project_path, r"D:\Documents\Code\Github\ccr");
        assert_eq!(records[0].model.as_deref(), Some("gemini-3-pro-preview"));
        assert_eq!(records[0].input_tokens, 154528);
        assert_eq!(records[0].output_tokens, 1969);
        assert_eq!(records[0].cache_read_tokens, 2048);
    }

    #[test]
    fn test_import_file_persists_gemini_session_records() {
        setup();

        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path().join("tmp").join("backend");
        let chats_dir = project_dir.join("chats");
        std::fs::create_dir_all(&chats_dir).unwrap();
        std::fs::write(
            project_dir.join(".project_root"),
            "D:\\Documents\\Code\\Github\\ccr\\ccr-ui\\backend\n",
        )
        .unwrap();
        let file_path = chats_dir.join("session-2026-03-16T08-00-backend.json");

        let content = r#"{
            "sessionId": "gem-sess-2",
            "startTime": "2026-03-16T08:00:00Z",
            "lastUpdated": "2026-03-16T08:10:00Z",
            "messages": [
                {
                    "id": "msg-gem-2",
                    "timestamp": "2026-03-16T08:01:05Z",
                    "type": "gemini",
                    "model": "gemini-3-flash-preview",
                    "tokens": {
                        "input": 10481,
                        "output": 72,
                        "cached": 0,
                        "thoughts": 538,
                        "total": 11091
                    }
                }
            ]
        }"#;
        std::fs::write(&file_path, content).unwrap();

        let service = UsageImportService::new(ImportConfig::default());
        let (imported, skipped) = service.import_file("gemini", &file_path).unwrap();

        assert_eq!(imported, 1);
        assert_eq!(skipped, 0);

        let file_path_str = file_path.to_string_lossy().to_string();
        let (source, records, model_stats, project_stats) = database::with_connection(|conn| {
            let source = usage_repo::get_source_by_path(conn, &file_path_str)?
                .expect("source should exist after gemini import");
            let records = usage_repo::get_records_by_source(conn, &source.id)?;
            let model_stats =
                usage_repo::get_model_stats(conn, &Some("gemini".to_string()), &None, &None)?;
            let project_stats =
                usage_repo::get_project_stats(conn, &Some("gemini".to_string()), &None, &None)?;
            Ok((source, records, model_stats, project_stats))
        })
        .unwrap();

        assert!(!source.id.is_empty());
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].project_path,
            r"D:\Documents\Code\Github\ccr\ccr-ui\backend"
        );
        assert!(
            model_stats
                .iter()
                .any(|stat| stat.model == "gemini-3-flash-preview")
        );
        assert!(
            project_stats.iter().any(|stat| {
                stat.project_path == r"D:\Documents\Code\Github\ccr\ccr-ui\backend"
            })
        );
    }

    #[test]
    fn test_calculate_cost_codex_models() {
        let service = UsageImportService::new(ImportConfig::default());

        // codex-mini: (0.15, 0.60, 0.0375) per 1M tokens
        let cost = service.calculate_cost("codex-mini-latest", 1_000_000, 1_000_000, 0);
        assert!((cost - 0.75).abs() < 0.001); // 0.15 + 0.60

        // o4-mini: (0.55, 2.20, 0.1375) per 1M tokens
        let cost = service.calculate_cost("o4-mini", 1_000_000, 1_000_000, 0);
        assert!((cost - 2.75).abs() < 0.001); // 0.55 + 2.20

        // o3: (2.0, 8.0, 0.50) per 1M tokens
        let cost = service.calculate_cost("o3", 1_000_000, 1_000_000, 0);
        assert!((cost - 10.0).abs() < 0.001); // 2.0 + 8.0
    }
}
