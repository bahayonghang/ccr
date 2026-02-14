// Usage import service
// Implements incremental import pipeline for usage logs
// Tracks per-file offsets and hashes for efficient import

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::File;
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
    /// Retention days for records
    pub retention_days: i64,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            max_lines_per_source: 5000,
            time_budget_secs: 2,
            retention_days: 365,
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

impl UsageImportService {
    pub fn new(config: ImportConfig) -> Self {
        Self { config }
    }

    /// Import usage data for a platform incrementally
    pub fn import_platform(&self, platform: &str) -> Result<ImportResult, String> {
        let start = Instant::now();
        let time_budget = Duration::from_secs(self.config.time_budget_secs);

        // Get home directory
        let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;

        // Determine projects directory based on platform
        let projects_dir = match platform {
            "claude" => home_dir.join(".claude/projects"),
            "codex" => home_dir.join(".codex/projects"),
            "gemini" => home_dir.join(".gemini/projects"),
            _ => return Err(format!("Unsupported platform: {}", platform)),
        };

        if !projects_dir.exists() {
            debug!(
                "Projects directory does not exist for platform {}: {:?}",
                platform, projects_dir
            );
            return Ok(ImportResult {
                platform: platform.to_string(),
                files_processed: 0,
                records_imported: 0,
                records_skipped: 0,
                duration_ms: start.elapsed().as_millis() as u64,
                completed: true,
            });
        }

        // Collect all .jsonl files
        let jsonl_files: Vec<PathBuf> = WalkDir::new(&projects_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "jsonl"))
            .map(|e| e.path().to_path_buf())
            .collect();

        debug!(
            "Found {} JSONL files for platform {}",
            jsonl_files.len(),
            platform
        );

        let mut total_imported = 0;
        let mut total_skipped = 0;
        let mut files_processed = 0;
        let mut completed = true;

        for file_path in &jsonl_files {
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

        // Calculate current file hash (first 4KB for efficiency)
        let current_hash = self.calculate_file_hash(file_path)?;

        // Check if we have a source record for this file
        let existing_source =
            database::with_connection(|conn| usage_repo::get_source_by_path(conn, &file_path_str))
                .map_err(|e| e.to_string())?;

        let (source_id, start_offset) = match existing_source {
            Some(source) => {
                // Check if file has changed
                if source.file_hash != current_hash {
                    // File changed, need to re-import from beginning
                    debug!("File hash changed, re-importing: {:?}", file_path);
                    // Delete old records for this source
                    database::with_connection(|conn| {
                        usage_repo::delete_records_by_source(conn, &source.id)
                    })
                    .map_err(|e| e.to_string())?;
                    (source.id, 0i64)
                } else {
                    // Continue from last offset
                    (source.id, source.last_offset)
                }
            }
            None => {
                // New file, create source record
                (Uuid::new_v4().to_string(), 0i64)
            }
        };

        // Extract project path from file path
        let project_path = self.extract_project_path(file_path, platform);

        // Read and import new lines
        let (records, new_offset, skipped) = self.read_lines_from_offset(
            file_path,
            start_offset,
            platform,
            &project_path,
            &source_id,
        )?;

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
        } else if model.contains("gpt-4") {
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
        let file = File::open(file_path).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(file);
        let mut buffer = [0u8; 4096];

        let bytes_read =
            std::io::Read::read(&mut reader, &mut buffer).map_err(|e| e.to_string())?;

        let mut hasher = Sha256::new();
        hasher.update(&buffer[..bytes_read]);
        let hash = hasher.finalize();

        Ok(format!("{:x}", hash))
    }

    /// Extract project path from file path
    fn extract_project_path(&self, file_path: &Path, platform: &str) -> String {
        // Try to extract project path from the file path
        // e.g., ~/.claude/projects/myproject/usage.jsonl -> myproject
        let path_str = file_path.to_string_lossy();

        let marker = format!("/.{}/projects/", platform);
        if let Some(pos) = path_str.find(&marker) {
            let after_marker = &path_str[pos + marker.len()..];
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

    /// Get import statistics
    pub fn get_stats(&self) -> Result<usage_repo::UsageStats, String> {
        database::with_connection(usage_repo::get_usage_stats).map_err(|e| e.to_string())
    }

    /// Cleanup old records
    pub fn cleanup_old_records(&self) -> Result<usize, String> {
        database::with_connection(|conn| {
            usage_repo::delete_old_records(conn, self.config.retention_days)
        })
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
        assert_eq!(config.retention_days, 365);
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
}
