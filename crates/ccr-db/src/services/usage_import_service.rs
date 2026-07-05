// Usage import service
// Implements incremental import pipeline for usage logs
// Tracks per-file offsets and hashes for efficient import

use ccr_core::{is_qwen_chat_file, qwen_project_dir_name_from_chat_path, qwen_projects_dir};
use ccr_types::{ModelRateCatalog, PricingComputation};
use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::database::{self, DbPool, repositories::usage_repo};

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
#[deprecated(
    since = "0.7.0",
    note = "ccr-ui usage import now uses llmusage::sync::JobRegistry; retain only for legacy ccr-db data compatibility."
)]
pub struct UsageImportService {
    config: ImportConfig,
    db_pool: DbPool,
    pricing_catalog: ModelRateCatalog,
}

const IMPORT_ALL_PLATFORMS: [&str; 4] = ["claude", "codex", "gemini", "opencode"];

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

#[derive(Debug, Clone)]
struct OpenCodeMessageRow {
    rowid: i64,
    message_id: Option<String>,
    session_id: Option<String>,
    time_updated: Option<i64>,
    data: String,
}

#[allow(deprecated)]
impl UsageImportService {
    pub fn new(config: ImportConfig) -> Self {
        let db_pool = database::get_pool()
            .cloned()
            .expect("UsageImportService requires initialized database pool");
        Self {
            config,
            db_pool,
            pricing_catalog: ModelRateCatalog::official(),
        }
    }

    pub fn with_pool(config: ImportConfig, db_pool: DbPool) -> Self {
        Self {
            config,
            db_pool,
            pricing_catalog: ModelRateCatalog::official(),
        }
    }

    pub fn with_pool_and_catalog(
        config: ImportConfig,
        db_pool: DbPool,
        pricing_catalog: ModelRateCatalog,
    ) -> Self {
        Self {
            config,
            db_pool,
            pricing_catalog,
        }
    }

    fn with_connection<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<T, rusqlite::Error>,
    {
        let conn = self.db_pool.get().map_err(|e| e.to_string())?;
        f(&conn).map_err(|e| e.to_string())
    }

    fn opencode_storage_dir() -> Result<PathBuf, String> {
        if let Ok(custom) = std::env::var("CCR_OPENCODE_DIR") {
            return Ok(PathBuf::from(custom));
        }

        let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
        let official = home_dir.join(".local").join("share").join("opencode");
        if official.exists() {
            return Ok(official);
        }

        if let Some(legacy) = dirs::data_local_dir().map(|dir| dir.join("opencode"))
            && legacy.exists()
        {
            return Ok(legacy);
        }

        Ok(official)
    }

    pub fn list_usage_files(&self, platform: &str) -> Result<Vec<PathBuf>, String> {
        let projects_dir = match platform {
            "claude" => dirs::home_dir()
                .ok_or("Could not find home directory")?
                .join(".claude/projects"),
            "codex" => {
                let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
                let codex_home = std::env::var("CODEX_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| home_dir.join(".codex"));
                codex_home.join("sessions")
            }
            "gemini" => dirs::home_dir()
                .ok_or("Could not find home directory")?
                .join(".gemini/tmp"),
            "qwen" => qwen_projects_dir().ok_or("Could not resolve Qwen runtime directory")?,
            "opencode" => {
                let db_path = Self::opencode_storage_dir()?.join("opencode.db");
                return Ok(db_path.exists().then_some(db_path).into_iter().collect());
            }
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
                "qwen" => Self::is_qwen_session_file(entry.path()),
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

    /// Delete imported records and checkpoints for all sources of a platform.
    pub fn reset_platform_sources(&self, platform: &str) -> Result<(usize, usize), String> {
        let sources =
            self.with_connection(|conn| usage_repo::get_sources_by_platform(conn, platform))?;

        let mut deleted_sources = 0usize;
        let mut deleted_records = 0usize;

        for source in sources {
            let removed_records = self.with_connection(|conn| {
                let removed_records = usage_repo::delete_records_by_source(conn, &source.id)?;
                let _ = usage_repo::delete_codex_checkpoint(conn, &source.id)?;
                let _ = usage_repo::delete_source(conn, &source.id)?;
                Ok::<usize, rusqlite::Error>(removed_records)
            })?;

            deleted_sources += 1;
            deleted_records += removed_records;
        }

        info!(
            "Reset usage import state for {}: {} sources, {} records",
            platform, deleted_sources, deleted_records
        );

        Ok((deleted_sources, deleted_records))
    }

    /// Import usage data for a platform incrementally
    pub fn import_platform(&self, platform: &str) -> Result<ImportResult, String> {
        if platform == "all" {
            return self.import_all_supported_platforms();
        }

        self.import_single_platform(platform)
    }

    fn import_all_supported_platforms(&self) -> Result<ImportResult, String> {
        let start = Instant::now();
        let mut files_processed = 0usize;
        let mut records_imported = 0usize;
        let mut records_skipped = 0usize;
        let mut completed = true;

        for platform in IMPORT_ALL_PLATFORMS {
            match self.import_single_platform(platform) {
                Ok(result) => {
                    files_processed += result.files_processed;
                    records_imported += result.records_imported;
                    records_skipped += result.records_skipped;
                    completed &= result.completed;
                }
                Err(error) => {
                    completed = false;
                    warn!(platform, ?error, "Failed to import usage platform");
                }
            }
        }

        Ok(ImportResult {
            platform: "all".to_string(),
            files_processed,
            records_imported,
            records_skipped,
            duration_ms: start.elapsed().as_millis() as u64,
            completed,
        })
    }

    fn import_single_platform(&self, platform: &str) -> Result<ImportResult, String> {
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
        let is_opencode_db = platform == "opencode";
        let is_codex = platform == "codex";
        let is_gemini_session = platform == "gemini" && Self::is_gemini_session_file(file_path);
        let is_qwen_session = platform == "qwen" && Self::is_qwen_session_file(file_path);
        let is_replay_session = is_gemini_session || is_qwen_session;
        let current_file_size = std::fs::metadata(file_path)
            .map(|metadata| metadata.len() as i64)
            .unwrap_or(0);

        // OpenCode uses a mutable SQLite database, so hash the full file instead of
        // using the append-log prefix optimization.
        let current_hash = if is_opencode_db {
            self.calculate_full_file_hash(file_path)?
        } else {
            self.calculate_file_hash(file_path)?
        };

        // Check if we have a source record for this file
        let existing_source =
            self.with_connection(|conn| usage_repo::get_source_by_path(conn, &file_path_str))?;

        let mut codex_append_checkpoint = None;
        let (source_id, start_offset) = match existing_source {
            Some(source) => {
                if is_opencode_db {
                    if current_file_size == source.last_offset && source.file_hash == current_hash {
                        return Ok((0, 0));
                    }

                    debug!(
                        "OpenCode usage database changed, re-importing: {:?}",
                        file_path
                    );
                    self.with_connection(|conn| {
                        usage_repo::delete_records_by_source(conn, &source.id)
                    })?;
                    (source.id, 0i64)
                } else if is_codex {
                    if current_file_size < source.last_offset {
                        debug!("Codex session shrank, re-importing: {:?}", file_path);
                        self.with_connection(|conn| {
                            usage_repo::delete_records_by_source(conn, &source.id)
                        })?;
                        self.with_connection(|conn| {
                            usage_repo::delete_codex_checkpoint(conn, &source.id).map(|_| ())
                        })?;
                        (source.id, 0i64)
                    } else if current_file_size == source.last_offset {
                        if source.file_hash == current_hash {
                            return Ok((0, 0));
                        }
                        debug!(
                            "Codex session changed in-place, re-importing: {:?}",
                            file_path
                        );
                        self.with_connection(|conn| {
                            usage_repo::delete_records_by_source(conn, &source.id)
                        })?;
                        self.with_connection(|conn| {
                            usage_repo::delete_codex_checkpoint(conn, &source.id).map(|_| ())
                        })?;
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
                            self.with_connection(|conn| {
                                usage_repo::delete_records_by_source(conn, &source.id)
                            })?;
                            self.with_connection(|conn| {
                                usage_repo::delete_codex_checkpoint(conn, &source.id).map(|_| ())
                            })?;
                            (source.id, 0i64)
                        }
                    }
                } else if is_replay_session {
                    if current_file_size == source.last_offset && source.file_hash == current_hash {
                        return Ok((0, 0));
                    }

                    debug!(
                        "Session file changed, re-importing from start: {:?}",
                        file_path
                    );
                    self.with_connection(|conn| {
                        usage_repo::delete_records_by_source(conn, &source.id)
                    })?;
                    (source.id, 0i64)
                } else if source.file_hash != current_hash {
                    // File changed, need to re-import from beginning
                    debug!("File hash changed, re-importing: {:?}", file_path);
                    self.with_connection(|conn| {
                        usage_repo::delete_records_by_source(conn, &source.id)
                    })?;
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
            } else if is_qwen_session {
                self.read_qwen_session(file_path, &project_path, &source_id)?
            } else if is_opencode_db {
                self.read_opencode_db(file_path, &project_path, &source_id)?
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
            self.with_connection(|conn| {
                usage_repo::insert_records_batch(conn, &records).map(|_| ())
            })?;
        }

        // Update source record
        let source = usage_repo::UsageSource {
            id: source_id,
            platform: platform.to_string(),
            file_path: file_path_str,
            file_hash: current_hash,
            last_offset: new_offset,
            source_state: usage_repo::UsageSourceState::Live,
            file_size: Some(current_file_size),
            modified_at: std::fs::metadata(file_path)
                .ok()
                .and_then(|meta| meta.modified().ok())
                .map(DateTime::<Utc>::from),
            last_seen_at: Some(Utc::now()),
            raw_deleted_at: None,
            updated_at: Utc::now(),
        };

        self.with_connection(|conn| usage_repo::upsert_source(conn, &source))?;

        if is_codex {
            self.persist_codex_checkpoint(&source.id)?;
        }

        Ok((imported, skipped))
    }

    fn load_codex_append_checkpoint(
        &self,
        source_id: &str,
    ) -> Result<Option<CodexAppendCheckpoint>, String> {
        if let Some(checkpoint) =
            self.with_connection(|conn| usage_repo::get_codex_checkpoint(conn, source_id))?
        {
            return Ok(Some(CodexAppendCheckpoint {
                session_id: checkpoint.session_id,
                model: checkpoint.model,
                project_path: checkpoint.project_path,
                last_line_number: checkpoint.last_line_number as u64,
                prefers_turn_completed: checkpoint.prefers_turn_completed,
                last_cumulative_usage: CodexTokenUsage {
                    input_tokens: checkpoint.input_tokens.max(0) as u64,
                    cached_input_tokens: checkpoint.cached_input_tokens.max(0) as u64,
                    output_tokens: checkpoint.output_tokens.max(0) as u64,
                },
            }));
        }

        let records =
            self.with_connection(|conn| usage_repo::get_records_by_source(conn, source_id))?;
        Ok(Self::build_codex_append_checkpoint_from_records(records))
    }

    fn build_codex_append_checkpoint_from_records(
        records: Vec<usage_repo::UsageRecord>,
    ) -> Option<CodexAppendCheckpoint> {
        let latest = records
            .into_iter()
            .filter_map(|record| {
                Self::parse_record_line_number(&record.id).map(|line_number| (line_number, record))
            })
            .max_by_key(|(line_number, _)| *line_number);

        let (last_line_number, latest_record) = latest?;

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

        Some(CodexAppendCheckpoint {
            session_id,
            model: latest_record.model,
            project_path: latest_record.project_path,
            last_line_number,
            prefers_turn_completed,
            last_cumulative_usage,
        })
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
                let event_type = payload
                    .get("type")
                    .and_then(|v| v.as_str())
                    .or_else(|| json.get("type").and_then(|v| v.as_str()))
                    .unwrap_or("");

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

                        if delta_input > 0 || delta_output > 0 || delta_cached > 0 {
                            let record_id = format!("{}:{}", session_id, line_number);
                            token_count_records.push(self.build_usage_record(
                                record_id,
                                "codex",
                                resolved_project_path.clone(),
                                json.to_string(),
                                event_ts,
                                source_id,
                                current_model.clone(),
                                delta_input as i64,
                                delta_output as i64,
                                delta_cached as i64,
                                0,
                            ));
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

                    if input > 0 || output > 0 || cached > 0 {
                        let record_id = format!("{}:{}", session_id, line_number);
                        turn_completed_records.push(self.build_usage_record(
                            record_id,
                            "codex",
                            resolved_project_path.clone(),
                            json.to_string(),
                            event_ts,
                            source_id,
                            current_model.clone(),
                            input as i64,
                            output as i64,
                            cached as i64,
                            0,
                        ));
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

    fn read_opencode_db(
        &self,
        file_path: &Path,
        project_path: &str,
        source_id: &str,
    ) -> Result<(Vec<usage_repo::UsageRecord>, i64, usize), String> {
        let file_size = std::fs::metadata(file_path)
            .map(|metadata| metadata.len() as i64)
            .unwrap_or(0);
        let connection = match rusqlite::Connection::open_with_flags(
            file_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) {
            Ok(connection) => connection,
            Err(error) => {
                warn!(?error, path = %file_path.display(), "Failed to open OpenCode usage database");
                return Ok((Vec::new(), file_size, 0));
            }
        };

        let columns = match Self::opencode_message_columns(&connection) {
            Ok(columns) => columns,
            Err(error) => {
                warn!(?error, path = %file_path.display(), "Failed to inspect OpenCode message schema");
                return Ok((Vec::new(), file_size, 0));
            }
        };
        if columns.is_empty() || !Self::has_column(&columns, "data") {
            return Ok((Vec::new(), file_size, 0));
        }

        let id_expr = if Self::has_column(&columns, "id") {
            "id"
        } else {
            "NULL"
        };
        let session_expr = if Self::has_column(&columns, "session_id") {
            "session_id"
        } else {
            "NULL"
        };
        let time_expr = if Self::has_column(&columns, "time_updated") {
            "time_updated"
        } else if Self::has_column(&columns, "time_created") {
            "time_created"
        } else {
            "NULL"
        };
        let order_expr = if Self::has_column(&columns, "time_updated") {
            "time_updated"
        } else if Self::has_column(&columns, "time_created") {
            "time_created"
        } else {
            "rowid"
        };
        let sql = format!(
            "SELECT rowid, {id_expr}, {session_expr}, {time_expr}, data FROM message ORDER BY {order_expr} ASC, rowid ASC"
        );

        let mut statement = match connection.prepare(&sql) {
            Ok(statement) => statement,
            Err(error) => {
                warn!(?error, path = %file_path.display(), "Failed to prepare OpenCode usage query");
                return Ok((Vec::new(), file_size, 0));
            }
        };

        let rows = match statement.query_map([], |row| {
            Ok(OpenCodeMessageRow {
                rowid: row.get(0)?,
                message_id: row.get(1)?,
                session_id: row.get(2)?,
                time_updated: row.get(3)?,
                data: row.get(4)?,
            })
        }) {
            Ok(rows) => rows,
            Err(error) => {
                warn!(?error, path = %file_path.display(), "Failed to read OpenCode usage rows");
                return Ok((Vec::new(), file_size, 0));
            }
        };

        let fallback_project_path = if project_path.trim().is_empty() || project_path == "unknown" {
            file_path
                .parent()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "opencode".to_string())
        } else {
            project_path.to_string()
        };
        let mut records = Vec::new();
        let mut skipped = 0usize;

        for row in rows {
            let row = match row {
                Ok(row) => row,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };

            if let Some(record) =
                self.parse_opencode_message_row(row, source_id, &fallback_project_path)
            {
                records.push(record);
            } else {
                skipped += 1;
            }
        }

        Ok((records, file_size, skipped))
    }

    fn opencode_message_columns(
        connection: &rusqlite::Connection,
    ) -> Result<Vec<String>, rusqlite::Error> {
        let mut statement = connection.prepare("PRAGMA table_info(message)")?;
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))?
            .filter_map(|row| row.ok())
            .collect();
        Ok(columns)
    }

    fn has_column(columns: &[String], column: &str) -> bool {
        columns.iter().any(|value| value == column)
    }

    fn parse_opencode_message_row(
        &self,
        row: OpenCodeMessageRow,
        source_id: &str,
        fallback_project_path: &str,
    ) -> Option<usage_repo::UsageRecord> {
        let json: Value = serde_json::from_str(&row.data).ok()?;
        let role = json
            .get("role")
            .or_else(|| json.get("type"))
            .and_then(Value::as_str);
        if role != Some("assistant") {
            return None;
        }

        let usage = Self::extract_opencode_token_usage(&json)?;
        let provider = Self::extract_opencode_provider(&json);
        let model = Self::extract_opencode_model(&json, provider.as_deref());
        let recorded_at = Self::extract_opencode_recorded_at(&json).or_else(|| {
            row.time_updated
                .and_then(Self::timestamp_from_unix_seconds_or_millis)
        })?;
        let session_id = row
            .session_id
            .or_else(|| Self::find_string_by_keys(&json, &["sessionID", "sessionId", "session_id"]))
            .unwrap_or_else(|| "unknown".to_string());
        let record_id = row
            .message_id
            .filter(|value| !value.trim().is_empty())
            .map(|message_id| format!("opencode:{message_id}"))
            .unwrap_or_else(|| {
                let time_key = row
                    .time_updated
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| row.rowid.to_string());
                format!("opencode:{session_id}:{time_key}")
            });
        let project_path = Self::extract_opencode_project_path(&json).unwrap_or_else(|| {
            if session_id != "unknown" {
                format!("opencode:{session_id}")
            } else {
                fallback_project_path.to_string()
            }
        });

        Some(self.build_usage_record(
            record_id,
            "opencode",
            project_path,
            json.to_string(),
            recorded_at,
            source_id,
            model,
            usage.input_tokens,
            usage.output_tokens,
            usage.cached_input_tokens,
            0,
        ))
    }

    fn extract_opencode_token_usage(message: &Value) -> Option<GeminiTokenUsage> {
        let tokens = message.get("tokens")?;
        let input_tokens =
            Self::direct_i64_by_keys(tokens, &["input", "input_tokens", "inputTokens"])
                .unwrap_or(0);
        let cached_input_tokens = Self::direct_i64_by_keys(
            tokens,
            &[
                "cached",
                "cache",
                "cache_read",
                "cacheRead",
                "cached_input_tokens",
                "cache_read_input_tokens",
            ],
        )
        .unwrap_or(0);
        let output_tokens =
            Self::direct_i64_by_keys(tokens, &["output", "output_tokens", "outputTokens"])
                .unwrap_or(0);

        if input_tokens == 0 && cached_input_tokens == 0 && output_tokens == 0 {
            return None;
        }

        Some(GeminiTokenUsage {
            input_tokens,
            cached_input_tokens,
            output_tokens,
        })
    }

    fn extract_opencode_provider(message: &Value) -> Option<String> {
        message
            .get("providerID")
            .or_else(|| message.get("provider_id"))
            .or_else(|| message.get("provider"))
            .and_then(Value::as_str)
            .or_else(|| {
                message
                    .get("model")
                    .and_then(|model| {
                        model
                            .get("providerID")
                            .or_else(|| model.get("provider_id"))
                            .or_else(|| model.get("provider"))
                    })
                    .and_then(Value::as_str)
            })
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    }

    fn extract_opencode_model(message: &Value, provider: Option<&str>) -> Option<String> {
        let model = message
            .get("modelID")
            .or_else(|| message.get("model_id"))
            .and_then(Value::as_str)
            .or_else(|| {
                message.get("model").and_then(|model| {
                    model.as_str().or_else(|| {
                        model
                            .get("modelID")
                            .or_else(|| model.get("model_id"))
                            .or_else(|| model.get("id"))
                            .and_then(Value::as_str)
                    })
                })
            })
            .map(str::trim)
            .filter(|value| !value.is_empty())?;

        if model.contains('/') {
            return Some(model.to_string());
        }

        provider
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|provider| format!("{provider}/{model}"))
            .or_else(|| Some(model.to_string()))
    }

    fn extract_opencode_project_path(message: &Value) -> Option<String> {
        Self::find_string_by_keys(
            message,
            &[
                "cwd",
                "projectRoot",
                "project_root",
                "workspace",
                "workspacePath",
                "workspace_path",
            ],
        )
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    }

    fn extract_opencode_recorded_at(message: &Value) -> Option<DateTime<Utc>> {
        if let Some(time) = message.get("time") {
            for key in ["completed", "updated", "created"] {
                if let Some(timestamp) = time.get(key).and_then(Self::timestamp_from_json_value) {
                    return Some(timestamp);
                }
            }
        }

        for key in [
            "timeCompleted",
            "time_completed",
            "timeUpdated",
            "time_updated",
            "timestamp",
            "createdAt",
            "created_at",
        ] {
            if let Some(timestamp) = message.get(key).and_then(Self::timestamp_from_json_value) {
                return Some(timestamp);
            }
        }

        None
    }

    fn timestamp_from_json_value(value: &Value) -> Option<DateTime<Utc>> {
        match value {
            Value::Number(number) => number
                .as_i64()
                .or_else(|| number.as_u64().and_then(|value| i64::try_from(value).ok()))
                .and_then(Self::timestamp_from_unix_seconds_or_millis),
            Value::String(text) => DateTime::parse_from_rfc3339(text)
                .map(|datetime| datetime.with_timezone(&Utc))
                .ok()
                .or_else(|| {
                    text.parse::<i64>()
                        .ok()
                        .and_then(Self::timestamp_from_unix_seconds_or_millis)
                }),
            _ => None,
        }
    }

    fn timestamp_from_unix_seconds_or_millis(value: i64) -> Option<DateTime<Utc>> {
        if value.abs() < 100_000_000_000 {
            Utc.timestamp_opt(value, 0).single()
        } else {
            Utc.timestamp_millis_opt(value).single()
        }
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
                let event_type = payload
                    .get("type")
                    .and_then(|v| v.as_str())
                    .or_else(|| json.get("type").and_then(|v| v.as_str()))
                    .unwrap_or("");

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

                        if delta_input > 0 || delta_output > 0 || delta_cached > 0 {
                            let record_id = format!("{}:{}", checkpoint.session_id, line_number);
                            token_count_records.push(self.build_usage_record(
                                record_id,
                                "codex",
                                checkpoint.project_path.clone(),
                                json.to_string(),
                                event_ts,
                                source_id,
                                current_model.clone(),
                                delta_input as i64,
                                delta_output as i64,
                                delta_cached as i64,
                                0,
                            ));
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

                    if input > 0 || output > 0 || cached > 0 {
                        let record_id = format!("{}:{}", checkpoint.session_id, line_number);
                        turn_completed_records.push(self.build_usage_record(
                            record_id,
                            "codex",
                            checkpoint.project_path.clone(),
                            json.to_string(),
                            event_ts,
                            source_id,
                            current_model.clone(),
                            input as i64,
                            output as i64,
                            cached as i64,
                            0,
                        ));
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
        } else if matches!(
            json.get("type").and_then(|v| v.as_str()),
            Some("turn_context") | Some("token_count")
        ) {
            json.get("payload").or(Some(json))
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

    /// Read and parse a legacy Gemini CLI session JSON file.
    ///
    /// Legacy Gemini CLI stores session transcripts under ~/.gemini/tmp/*/chats/session-*.json. Antigravity import waits for confirmed local log format.
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

            let record_id = message
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{}:{}", session_id, index + 1));

            records.push(self.build_usage_record(
                record_id,
                "gemini",
                resolved_project_path.clone(),
                message.to_string(),
                recorded_at,
                source_id,
                model,
                usage.input_tokens,
                usage.output_tokens,
                usage.cached_input_tokens,
                0,
            ));
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

    fn is_qwen_session_file(path: &Path) -> bool {
        is_qwen_chat_file(path)
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

    fn read_qwen_session(
        &self,
        file_path: &Path,
        project_path: &str,
        source_id: &str,
    ) -> Result<(Vec<usage_repo::UsageRecord>, i64, usize), String> {
        let file = File::open(file_path).map_err(|e| e.to_string())?;
        let file_size = file
            .metadata()
            .map(|metadata| metadata.len() as i64)
            .unwrap_or(0);
        let reader = BufReader::new(file);
        let fallback_recorded_at = std::fs::metadata(file_path)
            .and_then(|metadata| metadata.modified())
            .map(DateTime::<Utc>::from)
            .ok();

        let mut records = Vec::new();
        let mut skipped = 0usize;
        let mut session_id = file_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("unknown")
            .to_string();
        let mut current_model: Option<String> = None;
        let mut resolved_project_path = if project_path.is_empty() {
            String::from("unknown")
        } else {
            project_path.to_string()
        };

        for (index, line_result) in reader.lines().enumerate() {
            let line = match line_result {
                Ok(line) => line,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let json: Value = match serde_json::from_str(trimmed) {
                Ok(json) => json,
                Err(_) => {
                    skipped += 1;
                    continue;
                }
            };

            if let Some(parsed_session_id) = Self::extract_qwen_session_id(&json) {
                session_id = parsed_session_id;
            }

            if let Some(project_root) = Self::extract_qwen_project_path(&json) {
                resolved_project_path = project_root;
            }

            if let Some(model) = Self::extract_qwen_model(&json) {
                current_model = Some(model);
            }

            let recorded_at = Self::extract_qwen_recorded_at(&json)
                .or_else(|| fallback_recorded_at.as_ref().cloned())
                .unwrap_or_else(Utc::now);
            let model = Self::extract_qwen_model(&json).or_else(|| current_model.clone());
            let line_number = (index + 1) as u64;
            let mut produced = false;

            if let Some(usage) = Self::extract_qwen_assistant_usage(&json) {
                records.push(self.build_qwen_usage_record(
                    &session_id,
                    line_number,
                    "assistant",
                    source_id,
                    &resolved_project_path,
                    model.clone(),
                    recorded_at,
                    &json,
                    usage,
                ));
                produced = true;
            }

            if let Some(usage) = Self::extract_qwen_task_execution_usage(&json) {
                records.push(self.build_qwen_usage_record(
                    &session_id,
                    line_number,
                    "task",
                    source_id,
                    &resolved_project_path,
                    model.clone(),
                    recorded_at,
                    &json,
                    usage,
                ));
                produced = true;
            }

            if !produced {
                skipped += 1;
            }
        }

        Ok((records, file_size, skipped))
    }

    #[allow(clippy::too_many_arguments)]
    fn build_qwen_usage_record(
        &self,
        session_id: &str,
        line_number: u64,
        record_kind: &str,
        source_id: &str,
        project_path: &str,
        model: Option<String>,
        recorded_at: DateTime<Utc>,
        record_json: &Value,
        usage: GeminiTokenUsage,
    ) -> usage_repo::UsageRecord {
        self.build_usage_record(
            format!("{session_id}:{record_kind}:{line_number}"),
            "qwen",
            project_path.to_string(),
            record_json.to_string(),
            recorded_at,
            source_id,
            model,
            usage.input_tokens,
            usage.output_tokens,
            usage.cached_input_tokens,
            0,
        )
    }

    fn extract_qwen_assistant_usage(record: &Value) -> Option<GeminiTokenUsage> {
        let usage = record
            .get("usageMetadata")
            .or_else(|| record.get("usage_metadata"))
            .or_else(|| Self::find_value_by_keys(record, &["usageMetadata", "usage_metadata"]))?;

        Self::extract_qwen_token_usage(usage)
    }

    fn extract_qwen_task_execution_usage(record: &Value) -> Option<GeminiTokenUsage> {
        let result_display = record
            .get("resultDisplay")
            .or_else(|| Self::find_value_by_keys(record, &["resultDisplay"]))?;

        if result_display.get("type").and_then(|value| value.as_str()) != Some("task_execution") {
            return None;
        }

        let summary = record
            .get("executionSummary")
            .or_else(|| result_display.get("executionSummary"))
            .or_else(|| Self::find_value_by_keys(record, &["executionSummary"]))?;

        Self::extract_qwen_token_usage(summary)
    }

    fn extract_qwen_token_usage(usage: &Value) -> Option<GeminiTokenUsage> {
        let input_tokens = Self::find_i64_by_keys(
            usage,
            &[
                "promptTokenCount",
                "inputTokenCount",
                "inputTokens",
                "input_tokens",
            ],
        )
        .unwrap_or(0);
        let cached_input_tokens = Self::find_i64_by_keys(
            usage,
            &[
                "cachedContentTokenCount",
                "cachedInputTokenCount",
                "cachedTokens",
                "cachedReadTokens",
                "cached_input_tokens",
            ],
        )
        .unwrap_or(0);
        let output_tokens = Self::find_i64_by_keys(
            usage,
            &[
                "candidatesTokenCount",
                "outputTokenCount",
                "outputTokens",
                "output_tokens",
            ],
        )
        .unwrap_or(0)
            + Self::find_i64_by_keys(usage, &["thoughtTokens"]).unwrap_or(0);

        if input_tokens == 0 && cached_input_tokens == 0 && output_tokens == 0 {
            return None;
        }

        Some(GeminiTokenUsage {
            input_tokens,
            cached_input_tokens,
            output_tokens,
        })
    }

    fn extract_qwen_session_id(record: &Value) -> Option<String> {
        Self::find_string_by_keys(record, &["sessionId", "session_id"])
    }

    fn extract_qwen_project_path(record: &Value) -> Option<String> {
        Self::find_string_by_keys(record, &["cwd", "projectRoot", "project_root"])
            .filter(|value| !value.trim().is_empty())
    }

    fn extract_qwen_model(record: &Value) -> Option<String> {
        Self::find_string_by_keys(record, &["model", "modelId", "model_id"])
            .filter(|value| !value.trim().is_empty())
    }

    fn extract_qwen_recorded_at(record: &Value) -> Option<DateTime<Utc>> {
        let timestamp = Self::find_string_by_keys(
            record,
            &[
                "timestamp",
                "recordedAt",
                "recorded_at",
                "createdAt",
                "created_at",
                "updatedAt",
                "updated_at",
            ],
        )?;

        DateTime::parse_from_rfc3339(&timestamp)
            .map(|datetime| datetime.with_timezone(&Utc))
            .ok()
    }

    fn find_string_by_keys(value: &Value, keys: &[&str]) -> Option<String> {
        Self::find_value_by_keys(value, keys).and_then(|value| value.as_str().map(str::to_owned))
    }

    fn find_i64_by_keys(value: &Value, keys: &[&str]) -> Option<i64> {
        Self::find_value_by_keys(value, keys).and_then(Self::value_as_i64)
    }

    fn direct_i64_by_keys(value: &Value, keys: &[&str]) -> Option<i64> {
        keys.iter()
            .find_map(|key| value.get(*key).and_then(Self::value_as_i64))
    }

    fn value_as_i64(value: &Value) -> Option<i64> {
        match value {
            Value::Number(number) => number
                .as_i64()
                .or_else(|| number.as_u64().and_then(|value| i64::try_from(value).ok())),
            Value::String(text) => text.parse::<i64>().ok(),
            _ => None,
        }
    }

    fn find_value_by_keys<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a Value> {
        match value {
            Value::Object(map) => {
                for key in keys {
                    if let Some(found) = map.get(*key) {
                        return Some(found);
                    }
                }

                for nested in map.values() {
                    if let Some(found) = Self::find_value_by_keys(nested, keys) {
                        return Some(found);
                    }
                }

                None
            }
            Value::Array(items) => items
                .iter()
                .find_map(|item| Self::find_value_by_keys(item, keys)),
            _ => None,
        }
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
        let (input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens) =
            if let Some(usage) = usage_obj {
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
                let cache_creation = usage
                    .get("cache_creation_input_tokens")
                    .or_else(|| usage.get("cache_creation_tokens"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                if input == 0 && output == 0 && cache == 0 && cache_creation == 0 {
                    return None;
                }
                (input, output, cache, cache_creation)
            } else {
                return None;
            };

        // Store the original JSON for flexibility
        let record_json = json.to_string();

        // 计算费用（简化：使用内联定价表）
        Some(self.build_usage_record(
            uuid.to_string(),
            platform,
            project_path.to_string(),
            record_json,
            recorded_at,
            source_id,
            model,
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_creation_tokens,
        ))
    }

    /// 根据模型名称计算费用（每百万 token 定价）
    #[allow(clippy::too_many_arguments)]
    fn build_usage_record(
        &self,
        id: String,
        platform: &str,
        project_path: String,
        record_json: String,
        recorded_at: DateTime<Utc>,
        source_id: &str,
        model: Option<String>,
        input_tokens: i64,
        output_tokens: i64,
        cache_read_tokens: i64,
        cache_creation_tokens: i64,
    ) -> usage_repo::UsageRecord {
        let pricing = self.calculate_pricing(
            model.as_deref().unwrap_or("unknown"),
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_creation_tokens,
        );

        usage_repo::UsageRecord {
            id,
            platform: platform.to_string(),
            project_path,
            record_json,
            recorded_at,
            source_id: source_id.to_string(),
            model,
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_creation_tokens,
            cost_usd: pricing.cost_with_cache_usd,
            cost_with_cache_usd: pricing.cost_with_cache_usd,
            cost_without_cache_usd: pricing.cost_without_cache_usd,
            pricing_status: pricing.pricing_status,
            pricing_source: Some(pricing.pricing_source),
        }
    }

    fn calculate_pricing(
        &self,
        model: &str,
        input: i64,
        output: i64,
        cache_read: i64,
        cache_creation: i64,
    ) -> PricingComputation {
        self.pricing_catalog
            .calculate(model, input, output, cache_read, cache_creation)
    }

    #[cfg(test)]
    fn calculate_cost(&self, model: &str, input: i64, output: i64, cache: i64) -> f64 {
        self.calculate_pricing(model, input, output, cache, 0)
            .cost_with_cache_usd
    }

    /// Calculate file hash (first 4KB for efficiency)
    fn calculate_file_hash(&self, file_path: &Path) -> Result<String, String> {
        self.calculate_file_hash_with_limit(file_path, 4096)
    }

    fn calculate_full_file_hash(&self, file_path: &Path) -> Result<String, String> {
        let mut file = File::open(file_path).map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = file.read(&mut buffer).map_err(|e| e.to_string())?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let hash = hasher.finalize();
        Ok(hash.iter().map(|byte| format!("{byte:02x}")).collect())
    }

    fn calculate_file_hash_with_limit(
        &self,
        file_path: &Path,
        limit: usize,
    ) -> Result<String, String> {
        let file = File::open(file_path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0u8; limit];

        let bytes_read = reader.read(&mut buffer).map_err(|e| e.to_string())?;

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

        if platform == "qwen"
            && let Some(project_dir_name) = qwen_project_dir_name_from_chat_path(file_path)
        {
            return project_dir_name;
        }

        if platform == "opencode" {
            return file_path
                .parent()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "opencode".to_string());
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

    fn persist_codex_checkpoint(&self, source_id: &str) -> Result<(), String> {
        let records =
            self.with_connection(|conn| usage_repo::get_records_by_source(conn, source_id))?;
        let Some(checkpoint) = Self::build_codex_append_checkpoint_from_records(records) else {
            return Ok(());
        };

        self.with_connection(|conn| {
            usage_repo::upsert_codex_checkpoint(
                conn,
                &usage_repo::UsageCodexCheckpoint {
                    source_id: source_id.to_string(),
                    session_id: checkpoint.session_id,
                    project_path: checkpoint.project_path,
                    model: checkpoint.model,
                    last_line_number: checkpoint.last_line_number as i64,
                    input_tokens: checkpoint.last_cumulative_usage.input_tokens as i64,
                    cached_input_tokens: checkpoint.last_cumulative_usage.cached_input_tokens
                        as i64,
                    output_tokens: checkpoint.last_cumulative_usage.output_tokens as i64,
                    prefers_turn_completed: checkpoint.prefers_turn_completed,
                    updated_at: Utc::now(),
                },
            )
        })
    }

    /// Get cached records from database
    #[allow(dead_code)]
    pub fn get_records(
        &self,
        platform: &str,
        limit: usize,
    ) -> Result<Vec<usage_repo::UsageRecord>, String> {
        self.with_connection(|conn| usage_repo::get_recent_records(conn, platform, limit))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, deprecated)]
mod tests {
    use super::*;
    use crate::database;
    use crate::test_support::TestOpenCodeEnv;
    use std::io::Write;
    use std::sync::{Mutex, MutexGuard, OnceLock};
    use tempfile::TempDir;

    fn test_db_lock() -> MutexGuard<'static, ()> {
        static TEST_DB_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        TEST_DB_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap()
    }

    fn setup() -> MutexGuard<'static, ()> {
        let guard = test_db_lock();
        database::initialize_for_test().unwrap();
        reset_usage_tables();
        guard
    }

    fn reset_usage_tables() {
        database::with_connection(|conn| {
            conn.execute("DELETE FROM usage_session_archive", [])?;
            conn.execute("DELETE FROM usage_codex_checkpoint", [])?;
            conn.execute("DELETE FROM usage_history_cursor", [])?;
            conn.execute("DELETE FROM usage_records", [])?;
            conn.execute("DELETE FROM usage_sources", [])?;
            conn.execute("DELETE FROM usage_daily_agg", [])?;
            Ok::<(), rusqlite::Error>(())
        })
        .unwrap();
    }

    #[test]
    fn test_import_config_default() {
        let config = ImportConfig::default();
        assert_eq!(config.max_lines_per_source, 5000);
        assert_eq!(config.time_budget_secs, 2);
    }

    #[test]
    fn test_extract_project_path() {
        let _guard = setup();
        let service = UsageImportService::new(ImportConfig::default());

        let path = PathBuf::from("/home/user/.claude/projects/myproject/usage.jsonl");
        let project = service.extract_project_path(&path, "claude");
        assert_eq!(project, "myproject");
    }

    #[test]
    fn test_calculate_file_hash() {
        let _guard = setup();

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
        let _guard = setup();
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
        let _guard = setup();
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
        let _guard = setup();
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
        let _guard = setup();
        let service = UsageImportService::new(ImportConfig::default());

        let path = PathBuf::from("/home/user/.codex/sessions/2026/01/15/rollout-abc123.jsonl");
        let project = service.extract_project_path(&path, "codex");
        assert_eq!(project, "2026/01/15");
    }

    #[test]
    fn test_extract_project_path_qwen() {
        let _guard = setup();
        let service = UsageImportService::new(ImportConfig::default());

        let path =
            PathBuf::from("/home/user/.qwen/projects/workspace___repo/chats/session-1.jsonl");
        let project = service.extract_project_path(&path, "qwen");
        assert_eq!(project, "workspace___repo");
    }

    #[test]
    fn test_read_codex_session_token_count() {
        let _guard = setup();

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
        let _guard = setup();

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
        let _guard = setup();

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
        let _guard = setup();

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
        let _guard = setup();

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
        let _guard = setup();

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
        let _guard = setup();

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
    fn test_reset_platform_sources_clears_records_and_checkpoints() {
        let _guard = setup();
        reset_usage_tables();

        let codex_source = usage_repo::UsageSource {
            id: "src-codex".to_string(),
            platform: "codex".to_string(),
            file_path: "/tmp/codex-rollout.jsonl".to_string(),
            file_hash: "hash-a".to_string(),
            last_offset: 128,
            source_state: usage_repo::UsageSourceState::Live,
            file_size: Some(2048),
            modified_at: Some(Utc::now()),
            last_seen_at: Some(Utc::now()),
            raw_deleted_at: None,
            updated_at: Utc::now(),
        };
        let claude_source = usage_repo::UsageSource {
            id: "src-claude".to_string(),
            platform: "claude".to_string(),
            file_path: "/tmp/claude-usage.jsonl".to_string(),
            file_hash: "hash-b".to_string(),
            last_offset: 128,
            source_state: usage_repo::UsageSourceState::Live,
            file_size: Some(1024),
            modified_at: Some(Utc::now()),
            last_seen_at: Some(Utc::now()),
            raw_deleted_at: None,
            updated_at: Utc::now(),
        };

        let codex_record = usage_repo::UsageRecord {
            id: "codex-record".to_string(),
            platform: "codex".to_string(),
            project_path: "/tmp/project".to_string(),
            record_json: "{}".to_string(),
            recorded_at: Utc::now(),
            source_id: codex_source.id.clone(),
            model: Some("unknown".to_string()),
            input_tokens: 100,
            output_tokens: 40,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            cost_usd: 0.0,
            cost_with_cache_usd: 0.0,
            cost_without_cache_usd: 0.0,
            pricing_status: "unpriced".to_string(),
            pricing_source: Some("unpriced".to_string()),
        };
        let claude_record = usage_repo::UsageRecord {
            id: "claude-record".to_string(),
            platform: "claude".to_string(),
            project_path: "/tmp/project".to_string(),
            record_json: "{}".to_string(),
            recorded_at: Utc::now(),
            source_id: claude_source.id.clone(),
            model: Some("claude-sonnet-4-5".to_string()),
            input_tokens: 120,
            output_tokens: 60,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            cost_usd: 1.2,
            cost_with_cache_usd: 1.2,
            cost_without_cache_usd: 1.2,
            pricing_status: "priced".to_string(),
            pricing_source: Some("test".to_string()),
        };

        database::with_connection(|conn| {
            usage_repo::upsert_source(conn, &codex_source)?;
            usage_repo::upsert_source(conn, &claude_source)?;
            usage_repo::insert_record(conn, &codex_record)?;
            usage_repo::insert_record(conn, &claude_record)?;
            Ok::<(), rusqlite::Error>(())
        })
        .unwrap();

        let service = UsageImportService::new(ImportConfig::default());
        let (deleted_sources, deleted_records) = service.reset_platform_sources("codex").unwrap();

        assert_eq!(deleted_sources, 1);
        assert_eq!(deleted_records, 1);

        let (
            remaining_codex_sources,
            remaining_codex_records,
            remaining_claude_sources,
            remaining_claude_records,
        ) = database::with_connection(|conn| {
            Ok::<_, rusqlite::Error>((
                usage_repo::get_sources_by_platform(conn, "codex")?.len(),
                usage_repo::count_records_by_platform(conn, "codex")?,
                usage_repo::get_sources_by_platform(conn, "claude")?.len(),
                usage_repo::count_records_by_platform(conn, "claude")?,
            ))
        })
        .unwrap();

        assert_eq!(remaining_codex_sources, 0);
        assert_eq!(remaining_codex_records, 0);
        assert_eq!(remaining_claude_sources, 1);
        assert_eq!(remaining_claude_records, 1);
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
        let _guard = setup();

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
        let _guard = setup();
        reset_usage_tables();

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

    fn create_opencode_message_table(conn: &rusqlite::Connection) {
        conn.execute_batch(
            r#"
            CREATE TABLE message (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                time_created INTEGER NOT NULL,
                time_updated INTEGER NOT NULL,
                data TEXT NOT NULL
            );
            "#,
        )
        .unwrap();
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_opencode_message(
        conn: &rusqlite::Connection,
        id: &str,
        session_id: &str,
        time_updated: i64,
        role: &str,
        provider_id: &str,
        model_id: &str,
        input_tokens: i64,
        output_tokens: i64,
    ) {
        conn.execute(
            "INSERT INTO message (id, session_id, time_created, time_updated, data)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                id,
                session_id,
                time_updated,
                time_updated,
                serde_json::json!({
                    "role": role,
                    "providerID": provider_id,
                    "modelID": model_id,
                    "time": { "completed": time_updated },
                    "tokens": { "input": input_tokens, "output": output_tokens }
                })
                .to_string(),
            ),
        )
        .unwrap();
    }

    #[test]
    fn test_read_opencode_db_imports_assistant_token_rows_for_all_providers() {
        let _guard = setup();
        reset_usage_tables();

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("opencode.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        create_opencode_message_table(&conn);
        let now = Utc
            .with_ymd_and_hms(2026, 4, 1, 8, 0, 0)
            .unwrap()
            .timestamp_millis();
        insert_opencode_message(
            &conn,
            "msg-openai",
            "ses-1",
            now,
            "assistant",
            "openai",
            "gpt-5.4",
            1200,
            240,
        );
        insert_opencode_message(
            &conn,
            "msg-copilot",
            "ses-2",
            now + 1,
            "assistant",
            "github-copilot",
            "claude-opus-4.6",
            2000,
            400,
        );
        insert_opencode_message(
            &conn,
            "msg-user",
            "ses-3",
            now + 2,
            "user",
            "openai",
            "gpt-5.4",
            9999,
            9999,
        );
        insert_opencode_message(
            &conn,
            "msg-zero",
            "ses-4",
            now + 3,
            "assistant",
            "openai",
            "gpt-5.4",
            0,
            0,
        );
        drop(conn);

        let service = UsageImportService::new(ImportConfig::default());
        let (records, offset, skipped) = service
            .read_opencode_db(
                &db_path,
                temp_dir.path().to_string_lossy().as_ref(),
                "src-opencode",
            )
            .unwrap();

        assert_eq!(offset, std::fs::metadata(&db_path).unwrap().len() as i64);
        assert_eq!(records.len(), 2);
        assert_eq!(skipped, 2);
        assert_eq!(records[0].id, "opencode:msg-openai");
        assert_eq!(records[0].platform, "opencode");
        assert_eq!(records[0].project_path, "opencode:ses-1");
        assert_eq!(records[0].model.as_deref(), Some("openai/gpt-5.4"));
        assert_eq!(records[0].input_tokens, 1200);
        assert_eq!(records[0].output_tokens, 240);
        assert_eq!(
            records[1].model.as_deref(),
            Some("github-copilot/claude-opus-4.6")
        );
        assert_eq!(records[1].input_tokens, 2000);
        assert_eq!(records[1].output_tokens, 400);
    }

    #[test]
    fn test_opencode_record_id_falls_back_to_session_and_time() {
        let _guard = setup();
        reset_usage_tables();

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("opencode.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE message (
                session_id TEXT NOT NULL,
                time_updated INTEGER NOT NULL,
                data TEXT NOT NULL
            );
            "#,
        )
        .unwrap();
        let now = Utc
            .with_ymd_and_hms(2026, 4, 1, 8, 0, 0)
            .unwrap()
            .timestamp_millis();
        conn.execute(
            "INSERT INTO message (session_id, time_updated, data) VALUES (?1, ?2, ?3)",
            (
                "ses-fallback",
                now,
                serde_json::json!({
                    "role": "assistant",
                    "providerID": "openai",
                    "modelID": "gpt-5.4",
                    "tokens": { "input": 11, "output": 7 }
                })
                .to_string(),
            ),
        )
        .unwrap();
        drop(conn);

        let service = UsageImportService::new(ImportConfig::default());
        let (records, _offset, skipped) = service
            .read_opencode_db(
                &db_path,
                temp_dir.path().to_string_lossy().as_ref(),
                "src-opencode",
            )
            .unwrap();

        assert_eq!(skipped, 0);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, format!("opencode:ses-fallback:{now}"));
    }

    #[test]
    fn test_import_file_reimports_opencode_db_without_duplicate_counts() {
        let _guard = setup();
        reset_usage_tables();

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("opencode.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        create_opencode_message_table(&conn);
        let now = Utc
            .with_ymd_and_hms(2026, 4, 1, 8, 0, 0)
            .unwrap()
            .timestamp_millis();
        insert_opencode_message(
            &conn,
            "msg-1",
            "ses-1",
            now,
            "assistant",
            "openai",
            "gpt-5.4",
            100,
            20,
        );
        drop(conn);

        let service = UsageImportService::new(ImportConfig::default());
        let (first_imported, first_skipped) = service.import_file("opencode", &db_path).unwrap();
        assert_eq!(first_imported, 1);
        assert_eq!(first_skipped, 0);

        let (second_imported, second_skipped) = service.import_file("opencode", &db_path).unwrap();
        assert_eq!(second_imported, 0);
        assert_eq!(second_skipped, 0);

        let conn = rusqlite::Connection::open(&db_path).unwrap();
        insert_opencode_message(
            &conn,
            "msg-2",
            "ses-2",
            now + 1,
            "assistant",
            "github-copilot",
            "claude-opus-4.6",
            200,
            40,
        );
        drop(conn);

        let (third_imported, third_skipped) = service.import_file("opencode", &db_path).unwrap();
        assert_eq!(third_imported, 2);
        assert_eq!(third_skipped, 0);

        let file_path_str = db_path.to_string_lossy().to_string();
        let (records, summary, sources) = database::with_connection(|conn| {
            let source = usage_repo::get_source_by_path(conn, &file_path_str)?
                .expect("source should exist after opencode import");
            let records = usage_repo::get_records_by_source(conn, &source.id)?;
            let summary =
                usage_repo::get_usage_summary(conn, &Some("opencode".to_string()), &None, &None)?;
            let sources = usage_repo::get_sources_by_platform(conn, "opencode")?;
            Ok((records, summary, sources))
        })
        .unwrap();

        assert_eq!(sources.len(), 1);
        assert_eq!(records.len(), 2);
        assert_eq!(summary.total_requests, 2);
        assert_eq!(summary.total_input_tokens, 300);
        assert_eq!(summary.total_output_tokens, 60);
    }

    #[test]
    fn test_import_platform_all_includes_opencode() {
        let _guard = setup();
        reset_usage_tables();

        let env = TestOpenCodeEnv::new();
        let db_path = env.opencode_dir().join("opencode.db");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        create_opencode_message_table(&conn);
        let now = Utc
            .with_ymd_and_hms(2026, 4, 1, 8, 0, 0)
            .unwrap()
            .timestamp_millis();
        insert_opencode_message(
            &conn,
            "msg-all",
            "ses-all",
            now,
            "assistant",
            "openai",
            "gpt-5.4",
            50,
            10,
        );
        drop(conn);

        let result = UsageImportService::new(ImportConfig::default())
            .import_platform("all")
            .unwrap();
        assert_eq!(result.platform, "all");
        assert!(result.records_imported >= 1);

        let summary = database::with_connection(|conn| {
            usage_repo::get_usage_summary(conn, &Some("opencode".to_string()), &None, &None)
        })
        .unwrap();
        assert_eq!(summary.total_requests, 1);
        assert_eq!(summary.total_input_tokens, 50);
        assert_eq!(summary.total_output_tokens, 10);
    }

    #[test]
    fn test_calculate_cost_codex_models() {
        let _guard = setup();
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

    #[test]
    fn test_calculate_cost_current_model_catalog() {
        let _guard = setup();
        let service = UsageImportService::new(ImportConfig::default());

        let cases = [
            ("claude-opus-4-6", 1_000_000, 1_000_000, 1_000_000, 30.5),
            ("claude-opus-4.7", 1_000_000, 1_000_000, 1_000_000, 30.5),
            ("claude-haiku-4-5", 1_000_000, 1_000_000, 1_000_000, 6.1),
            ("gpt-5.4", 100_000, 100_000, 100_000, 1.775),
            ("gpt-5.5", 100_000, 100_000, 100_000, 3.55),
            ("gpt-5.4-mini", 100_000, 100_000, 100_000, 0.5325),
            ("gpt-5.3-codex", 100_000, 100_000, 100_000, 1.5925),
            (
                "gemini-3-flash-preview",
                1_000_000,
                1_000_000,
                1_000_000,
                3.55,
            ),
        ];

        for (model, input, output, cache, expected) in cases {
            let cost = service.calculate_cost(model, input, output, cache);
            assert!((cost - expected).abs() < 0.000_001, "{model}: {cost}");
        }
    }

    #[test]
    fn test_calculate_cost_long_context_tiers() {
        let _guard = setup();
        let service = UsageImportService::new(ImportConfig::default());

        let gpt = service.calculate_cost("gpt-5.4", 273_000, 1_000_000, 0);
        assert!((gpt - 23.865).abs() < 0.000_001);

        let gemini = service.calculate_cost("gemini-3.1-pro-preview", 201_000, 1_000_000, 0);
        assert!((gemini - 18.804).abs() < 0.000_001);
    }

    #[test]
    fn test_parse_usage_record_cache_creation_and_unpriced() {
        let _guard = setup();
        let service = UsageImportService::new(ImportConfig::default());
        let json = serde_json::json!({
            "uuid": "claude-cache-create",
            "timestamp": "2026-04-01T08:00:00Z",
            "model": "claude-sonnet-4-6",
            "usage": {
                "input_tokens": 1_000_000,
                "output_tokens": 1_000_000,
                "cache_read_input_tokens": 1_000_000,
                "cache_creation_input_tokens": 1_000_000
            }
        });

        let record = service
            .parse_usage_record(&json, "claude", "/workspace", "source-cache")
            .unwrap();
        assert_eq!(record.cache_creation_tokens, 1_000_000);
        assert!((record.cost_with_cache_usd - 22.05).abs() < 0.000_001);
        assert_eq!(record.pricing_status, "priced");

        let unknown_json = serde_json::json!({
            "uuid": "unknown-cache-create",
            "timestamp": "2026-04-01T08:00:00Z",
            "model": "coder-model",
            "usage": {
                "input_tokens": 1,
                "output_tokens": 1
            }
        });
        let unknown = service
            .parse_usage_record(&unknown_json, "claude", "/workspace", "source-unknown")
            .unwrap();
        assert_eq!(unknown.pricing_status, "unpriced");
        assert_eq!(unknown.cost_with_cache_usd, 0.0);
    }

    #[test]
    fn test_read_qwen_session_parses_usage_metadata_and_task_execution() {
        let _guard = setup();
        reset_usage_tables();

        let temp_dir = TempDir::new().unwrap();
        let file_dir = temp_dir
            .path()
            .join(".qwen")
            .join("projects")
            .join("workspace___repo")
            .join("chats");
        std::fs::create_dir_all(&file_dir).unwrap();
        let file_path = file_dir.join("session-qwen.jsonl");

        let content = r#"{"type":"session_meta","sessionId":"sess-qwen","cwd":"D:\\Documents\\Code\\Github\\ccr","model":"qwen3-coder-plus"}
{"type":"assistant","timestamp":"2026-04-01T08:00:00Z","usageMetadata":{"promptTokenCount":1200,"candidatesTokenCount":320,"cachedContentTokenCount":450}}
{"type":"tool","timestamp":"2026-04-01T08:01:00Z","resultDisplay":{"type":"task_execution"},"executionSummary":{"inputTokens":200,"outputTokens":50,"cachedTokens":20,"thoughtTokens":30}}
"#;
        std::fs::write(&file_path, content).unwrap();

        let service = UsageImportService::new(ImportConfig::default());
        let (records, offset, skipped) = service
            .read_qwen_session(&file_path, "workspace___repo", "src-qwen")
            .unwrap();

        assert_eq!(offset, content.len() as i64);
        assert_eq!(skipped, 1);
        assert_eq!(records.len(), 2);

        assert_eq!(records[0].id, "sess-qwen:assistant:2");
        assert_eq!(records[0].platform, "qwen");
        assert_eq!(records[0].project_path, r"D:\Documents\Code\Github\ccr");
        assert_eq!(records[0].model.as_deref(), Some("qwen3-coder-plus"));
        assert_eq!(records[0].input_tokens, 1200);
        assert_eq!(records[0].output_tokens, 320);
        assert_eq!(records[0].cache_read_tokens, 450);

        assert_eq!(records[1].id, "sess-qwen:task:3");
        assert_eq!(records[1].input_tokens, 200);
        assert_eq!(records[1].output_tokens, 80);
        assert_eq!(records[1].cache_read_tokens, 20);
    }

    #[test]
    fn test_import_file_persists_qwen_session_records() {
        let _guard = setup();
        reset_usage_tables();

        let temp_dir = TempDir::new().unwrap();
        let file_dir = temp_dir
            .path()
            .join(".qwen")
            .join("projects")
            .join("workspace___repo")
            .join("chats");
        std::fs::create_dir_all(&file_dir).unwrap();
        let file_path = file_dir.join("session-qwen-import.jsonl");

        let content = r#"{"type":"session_meta","sessionId":"sess-qwen-import","cwd":"D:\\Documents\\Code\\Github\\ccr\\ccr-ui","model":"qwen3-coder-plus"}
{"type":"assistant","timestamp":"2026-04-01T08:00:00Z","usageMetadata":{"promptTokenCount":1200,"candidatesTokenCount":320,"cachedContentTokenCount":450}}
{"type":"tool","timestamp":"2026-04-01T08:01:00Z","resultDisplay":{"type":"task_execution"},"executionSummary":{"inputTokens":200,"outputTokens":50,"cachedTokens":20}}
"#;
        std::fs::write(&file_path, content).unwrap();

        let service = UsageImportService::new(ImportConfig::default());
        let (imported, skipped) = service.import_file("qwen", &file_path).unwrap();

        assert_eq!(imported, 2);
        assert_eq!(skipped, 1);

        let file_path_str = file_path.to_string_lossy().to_string();
        let (source, records, model_stats, project_stats) = database::with_connection(|conn| {
            let source = usage_repo::get_source_by_path(conn, &file_path_str)?
                .expect("source should exist after qwen import");
            let records = usage_repo::get_records_by_source(conn, &source.id)?;
            let model_stats =
                usage_repo::get_model_stats(conn, &Some("qwen".to_string()), &None, &None)?;
            let project_stats =
                usage_repo::get_project_stats(conn, &Some("qwen".to_string()), &None, &None)?;
            Ok((source, records, model_stats, project_stats))
        })
        .unwrap();

        assert!(!source.id.is_empty());
        assert_eq!(records.len(), 2);
        assert_eq!(
            records[0].project_path,
            r"D:\Documents\Code\Github\ccr\ccr-ui"
        );
        assert!(
            model_stats
                .iter()
                .any(|stat| stat.model == "qwen3-coder-plus")
        );
        assert!(
            project_stats
                .iter()
                .any(|stat| stat.project_path == r"D:\Documents\Code\Github\ccr\ccr-ui")
        );
    }
}
