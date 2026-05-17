//! Codex 使用量服务
//!
//! 解析 Codex Session JSONL 文件，计算滚动窗口使用量统计
//!
//! Codex CLI 会话数据存储在 `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`
//! JSONL 格式为事件流：
//!   - 第1行: 会话元数据 (session_id, model, created_at)
//!   - turn_context 事件: 更新当前 model
//!   - token_count 事件: 累积 token 值（需差值计算）
//!   - turn.completed 事件: 直接包含 usage 数据

use ccr_core::core::error::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

const CODEX_USAGE_CACHE_VERSION: u32 = 1;

/// Codex 使用量记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexUsageRecord {
    /// 会话 ID
    pub session_id: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 输入 tokens
    pub input_tokens: u64,
    /// 输出 tokens
    pub output_tokens: u64,
    /// 模型名称
    pub model: Option<String>,
}

/// Codex 使用量统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexUsageStats {
    /// 总输入 tokens
    pub total_input_tokens: u64,
    /// 总输出 tokens
    pub total_output_tokens: u64,
    /// 总请求数
    pub total_requests: u64,
    /// 窗口开始时间
    pub window_start: Option<DateTime<Utc>>,
    /// 窗口结束时间
    pub window_end: Option<DateTime<Utc>>,
}

impl CodexUsageStats {
    /// 总 tokens
    #[allow(dead_code)]
    pub fn total_tokens(&self) -> u64 {
        self.total_input_tokens + self.total_output_tokens
    }
}

/// 滚动窗口使用量
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexRollingUsage {
    /// 5小时窗口统计
    pub five_hour: CodexUsageStats,
    /// 7天窗口统计
    pub seven_day: CodexUsageStats,
    /// 全部时间统计
    pub all_time: CodexUsageStats,
    /// 按模型分组的统计
    pub by_model: HashMap<String, CodexUsageStats>,
}

/// Codex Session 解析状态（用于累积 token 差值计算）
struct CodexSessionState {
    session_id: String,
    current_model: Option<String>,
    created_at: Option<DateTime<Utc>>,
    // 累积 token 追踪
    prev_input_tokens: u64,
    prev_output_tokens: u64,
}

#[derive(Debug, Clone, Default)]
struct CodexSessionMeta {
    session_id: String,
    model: Option<String>,
    created_at: Option<DateTime<Utc>>,
    #[allow(dead_code)]
    cwd: Option<String>,
}

#[derive(Debug, Clone, Copy, Default)]
struct CodexTokenUsage {
    input_tokens: u64,
    output_tokens: u64,
}

/// Codex 使用量服务
pub struct CodexUsageService {
    /// Codex 配置目录
    codex_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CodexUsageCache {
    version: u32,
    files: BTreeMap<String, CachedCodexUsageFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CachedCodexUsageFile {
    modified_ms: u64,
    size_bytes: u64,
    records: Vec<CodexUsageRecord>,
}

impl CodexUsageService {
    /// 创建新的服务实例
    pub fn new(codex_dir: PathBuf) -> Self {
        Self { codex_dir }
    }

    /// 获取会话目录（Codex CLI 存储在 sessions/ 下）
    fn sessions_dir(&self) -> PathBuf {
        self.codex_dir.join("sessions")
    }

    fn usage_cache_path(&self) -> PathBuf {
        self.codex_dir.join(".ccr-codex-usage-cache.json")
    }

    /// 递归收集目录下所有 .jsonl 文件
    fn collect_jsonl_files(dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let Ok(entries) = std::fs::read_dir(dir) else {
            return files;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(Self::collect_jsonl_files(&path));
            } else if path.extension().is_some_and(|ext| ext == "jsonl") {
                files.push(path);
            }
        }
        files
    }

    /// 解析所有 Session JSONL 文件
    pub fn parse_all_logs(&self) -> Result<Vec<CodexUsageRecord>> {
        let sessions_dir = self.sessions_dir();
        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let cache_path = self.usage_cache_path();
        let existing_cache = Self::read_usage_cache(&cache_path).unwrap_or_default();
        let mut next_cache = CodexUsageCache {
            version: CODEX_USAGE_CACHE_VERSION,
            files: BTreeMap::new(),
        };
        let mut records = Vec::new();

        // 递归扫描 sessions/YYYY/MM/DD/ 下的 .jsonl 文件
        for path in Self::collect_jsonl_files(&sessions_dir) {
            let relative_path = path
                .strip_prefix(&sessions_dir)
                .ok()
                .map(|value| value.to_string_lossy().replace('\\', "/"))
                .unwrap_or_else(|| path.to_string_lossy().to_string());
            let modified_ms = Self::metadata_modified_ms(&path);
            let size_bytes = Self::metadata_size_bytes(&path);
            let cached_file = existing_cache
                .files
                .get(&relative_path)
                .filter(|file| file.modified_ms == modified_ms && file.size_bytes == size_bytes);

            let file_records = if let Some(cached_file) = cached_file {
                cached_file.records.clone()
            } else {
                match self.parse_session_file(&path) {
                    Ok(file_records) => file_records,
                    Err(_) => continue,
                }
            };

            records.extend(file_records.clone());
            next_cache.files.insert(
                relative_path,
                CachedCodexUsageFile {
                    modified_ms,
                    size_bytes,
                    records: file_records,
                },
            );
        }

        if let Err(error) = Self::write_usage_cache(&cache_path, &next_cache) {
            tracing::debug!("写入 Codex usage cache 失败: {}", error);
        }

        // 按时间排序
        records.sort_by_key(|record| record.timestamp);

        Ok(records)
    }

    /// 基于已解析的 records 计算滚动窗口统计。
    pub fn compute_rolling_usage_for_records(records: &[CodexUsageRecord]) -> CodexRollingUsage {
        Self::compute_rolling_usage_at(records, Utc::now())
    }

    /// 解析单个 Codex Session JSONL 文件
    ///
    /// Codex Session 文件格式：
    ///   第1行: 会话元数据 {"session_id", "model", "created_at", ...}
    ///   后续行: 事件流
    ///     - event_msg.payload.type == "turn_context" -> 更新 model
    ///     - event_msg.payload.type == "token_count" -> 累积 tokens（差值计算）
    ///     - type == "turn.completed" -> 直接 usage 数据
    fn parse_session_file(&self, path: &Path) -> Result<Vec<CodexUsageRecord>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut token_count_records = Vec::new();
        let mut turn_completed_records = Vec::new();
        let mut state: Option<CodexSessionState> = None;
        let mut is_first_line = true;

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };

            if line.trim().is_empty() {
                continue;
            }

            let json: Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if is_first_line {
                is_first_line = false;

                // 第1行: 会话元数据
                let meta = Self::parse_codex_session_meta(&json);

                state = Some(CodexSessionState {
                    session_id: if meta.session_id.is_empty() {
                        "unknown".to_string()
                    } else {
                        meta.session_id
                    },
                    current_model: meta.model,
                    created_at: meta.created_at,
                    prev_input_tokens: 0,
                    prev_output_tokens: 0,
                });
                continue;
            }

            let st = match state.as_mut() {
                Some(s) => s,
                None => continue,
            };

            // 尝试提取事件时间戳（部分事件行有顶层 timestamp）
            let event_ts = json
                .get("timestamp")
                .and_then(|v| v.as_str())
                .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .or(st.created_at)
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
                        // 更新当前 model
                        if let Some(model) = payload.get("model").and_then(|v| v.as_str()) {
                            st.current_model = Some(model.to_string());
                        }
                    }
                    "token_count" => {
                        // 累积 token 值，计算增量
                        let usage = Self::extract_codex_token_usage(payload);
                        let cur_input = usage.input_tokens;
                        let cur_output = usage.output_tokens;

                        let delta_input = cur_input.saturating_sub(st.prev_input_tokens);
                        let delta_output = cur_output.saturating_sub(st.prev_output_tokens);

                        // 只在有增量时生成记录
                        if delta_input > 0 || delta_output > 0 {
                            token_count_records.push(CodexUsageRecord {
                                session_id: st.session_id.clone(),
                                timestamp: event_ts,
                                input_tokens: delta_input,
                                output_tokens: delta_output,
                                model: st.current_model.clone(),
                            });
                        }

                        // 更新累积值
                        st.prev_input_tokens = cur_input;
                        st.prev_output_tokens = cur_output;
                    }
                    _ => {}
                }
                continue;
            }

            // 检查 turn.completed 事件（--json 模式）
            if json.get("type").and_then(|v| v.as_str()) == Some("turn.completed")
                && let Some(usage) = json.get("usage")
            {
                let input = usage
                    .get("input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let output = usage
                    .get("output_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                if input > 0 || output > 0 {
                    turn_completed_records.push(CodexUsageRecord {
                        session_id: st.session_id.clone(),
                        timestamp: event_ts,
                        input_tokens: input,
                        output_tokens: output,
                        model: st.current_model.clone(),
                    });
                }
            }
        }

        // 去重策略：优先使用 turn.completed，否则用 token_count 增量
        if !turn_completed_records.is_empty() {
            Ok(turn_completed_records)
        } else {
            Ok(token_count_records)
        }
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
                .or_else(|| json.get("id"))
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
            output_tokens: usage
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
        }
    }

    fn metadata_modified_ms(path: &Path) -> u64 {
        std::fs::metadata(path)
            .ok()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or(0)
    }

    fn metadata_size_bytes(path: &Path) -> u64 {
        std::fs::metadata(path)
            .map(|metadata| metadata.len())
            .unwrap_or(0)
    }

    fn read_usage_cache(path: &Path) -> Result<CodexUsageCache> {
        if !path.exists() {
            return Ok(CodexUsageCache::default());
        }

        let content = std::fs::read_to_string(path)?;
        let cache: CodexUsageCache = serde_json::from_str(&content)?;
        if cache.version != CODEX_USAGE_CACHE_VERSION {
            return Ok(CodexUsageCache::default());
        }

        Ok(cache)
    }

    fn write_usage_cache(path: &Path, cache: &CodexUsageCache) -> Result<()> {
        let content = serde_json::to_string(cache)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 计算滚动窗口使用量
    pub fn compute_rolling_usage(&self) -> Result<CodexRollingUsage> {
        let records = self.parse_all_logs()?;
        Ok(Self::compute_rolling_usage_for_records(&records))
    }

    /// 添加记录到统计
    fn add_to_stats(stats: &mut CodexUsageStats, record: &CodexUsageRecord) {
        stats.total_input_tokens += record.input_tokens;
        stats.total_output_tokens += record.output_tokens;
        stats.total_requests += 1;
    }

    fn compute_rolling_usage_at(
        records: &[CodexUsageRecord],
        now: DateTime<Utc>,
    ) -> CodexRollingUsage {
        let five_hours_ago = now - Duration::hours(5);
        let seven_days_ago = now - Duration::days(7);

        let mut rolling = CodexRollingUsage::default();

        for record in records {
            Self::add_to_stats(&mut rolling.all_time, record);

            if record.timestamp >= seven_days_ago {
                Self::add_to_stats(&mut rolling.seven_day, record);
            }

            if record.timestamp >= five_hours_ago {
                Self::add_to_stats(&mut rolling.five_hour, record);
            }

            if let Some(model) = &record.model {
                let model_stats = rolling
                    .by_model
                    .entry(model.clone())
                    .or_insert_with(CodexUsageStats::default);
                Self::add_to_stats(model_stats, record);
            }
        }

        rolling.five_hour.window_start = Some(five_hours_ago);
        rolling.five_hour.window_end = Some(now);
        rolling.seven_day.window_start = Some(seven_days_ago);
        rolling.seven_day.window_end = Some(now);

        if let Some(first) = records.first() {
            rolling.all_time.window_start = Some(first.timestamp);
        }
        rolling.all_time.window_end = Some(now);

        rolling
    }

    /// 格式化 tokens 数量 (带 K/M 后缀)
    pub fn format_tokens(tokens: u64) -> String {
        if tokens >= 1_000_000 {
            format!("{:.1}M", tokens as f64 / 1_000_000.0)
        } else if tokens >= 1_000 {
            format!("{:.1}K", tokens as f64 / 1_000.0)
        } else {
            tokens.to_string()
        }
    }

    /// 获取使用量摘要文本
    #[allow(dead_code)]
    pub fn get_usage_summary(&self) -> Result<String> {
        let rolling = self.compute_rolling_usage()?;

        let mut summary = String::new();
        summary.push_str("Codex Usage Stats\n\n");

        summary.push_str(&format!(
            "5h window: {} tokens ({} in / {} out) - {} requests\n",
            Self::format_tokens(rolling.five_hour.total_tokens()),
            Self::format_tokens(rolling.five_hour.total_input_tokens),
            Self::format_tokens(rolling.five_hour.total_output_tokens),
            rolling.five_hour.total_requests
        ));

        summary.push_str(&format!(
            "7d window: {} tokens ({} in / {} out) - {} requests\n",
            Self::format_tokens(rolling.seven_day.total_tokens()),
            Self::format_tokens(rolling.seven_day.total_input_tokens),
            Self::format_tokens(rolling.seven_day.total_output_tokens),
            rolling.seven_day.total_requests
        ));

        summary.push_str(&format!(
            "All time:  {} tokens ({} in / {} out) - {} requests\n",
            Self::format_tokens(rolling.all_time.total_tokens()),
            Self::format_tokens(rolling.all_time.total_input_tokens),
            Self::format_tokens(rolling.all_time.total_output_tokens),
            rolling.all_time.total_requests
        ));

        if !rolling.by_model.is_empty() {
            summary.push_str("\nBy model:\n");
            for (model, stats) in &rolling.by_model {
                summary.push_str(&format!(
                    "  {}: {} tokens - {} requests\n",
                    model,
                    Self::format_tokens(stats.total_tokens()),
                    stats.total_requests
                ));
            }
        }

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use tempfile::TempDir;

    fn create_test_service() -> (CodexUsageService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let service = CodexUsageService::new(temp_dir.path().to_path_buf());
        (service, temp_dir)
    }

    #[test]
    fn test_empty_sessions() {
        let (service, _temp) = create_test_service();
        let records = service.parse_all_logs().unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn test_format_tokens() {
        assert_eq!(CodexUsageService::format_tokens(500), "500");
        assert_eq!(CodexUsageService::format_tokens(1500), "1.5K");
        assert_eq!(CodexUsageService::format_tokens(1_500_000), "1.5M");
    }

    #[test]
    fn test_parse_session_with_token_count() {
        let (service, temp_dir) = create_test_service();

        // 创建 sessions/2026/01/15/ 目录结构
        let session_dir = temp_dir.path().join("sessions").join("2026/01/15");
        std::fs::create_dir_all(&session_dir).unwrap();

        let now = Utc::now();
        let jsonl_content = format!(
            r#"{{"session_id":"sess-1","model":"codex-mini-latest","created_at":"{}","source":"terminal"}}
{{"event_msg":{{"payload":{{"type":"token_count","input_tokens":1000,"cached_input_tokens":500,"output_tokens":200}}}}}}
{{"event_msg":{{"payload":{{"type":"token_count","input_tokens":2500,"cached_input_tokens":1000,"output_tokens":500}}}}}}
"#,
            now.to_rfc3339()
        );
        std::fs::write(session_dir.join("rollout-abc123.jsonl"), jsonl_content).unwrap();

        let records = service.parse_all_logs().unwrap();
        assert_eq!(records.len(), 2);
        // 第一次 token_count: delta from 0
        assert_eq!(records[0].input_tokens, 1000);
        assert_eq!(records[0].output_tokens, 200);
        assert_eq!(records[0].model.as_deref(), Some("codex-mini-latest"));
        // 第二次 token_count: delta from previous
        assert_eq!(records[1].input_tokens, 1500);
        assert_eq!(records[1].output_tokens, 300);
    }

    #[test]
    fn test_parse_all_logs_reuses_usage_cache_for_unchanged_file() {
        let (service, temp_dir) = create_test_service();

        let session_dir = temp_dir.path().join("sessions").join("2026/01/15");
        std::fs::create_dir_all(&session_dir).unwrap();

        let now = Utc::now();
        let jsonl_content = format!(
            r#"{{"session_id":"sess-cache","model":"gpt-5","created_at":"{}"}}
{{"type":"turn.completed","usage":{{"input_tokens":100,"output_tokens":25}}}}
"#,
            now.to_rfc3339()
        );
        std::fs::write(session_dir.join("rollout-cache.jsonl"), jsonl_content).unwrap();

        let first = service.parse_all_logs().unwrap();
        assert_eq!(first[0].input_tokens, 100);

        let mut cache = CodexUsageService::read_usage_cache(&service.usage_cache_path()).unwrap();
        let cached_file = cache.files.values_mut().next().unwrap();
        cached_file.records[0].input_tokens = 999;
        CodexUsageService::write_usage_cache(&service.usage_cache_path(), &cache).unwrap();

        let second = service.parse_all_logs().unwrap();
        assert_eq!(second[0].input_tokens, 999);
    }

    #[test]
    fn test_parse_session_with_turn_completed() {
        let (service, temp_dir) = create_test_service();

        let session_dir = temp_dir.path().join("sessions").join("2026/03/16");
        std::fs::create_dir_all(&session_dir).unwrap();

        let now = Utc::now();
        let jsonl_content = format!(
            r#"{{"session_id":"sess-2","model":"o4-mini","created_at":"{}"}}
{{"type":"turn.completed","usage":{{"input_tokens":24763,"cached_input_tokens":24448,"output_tokens":122}}}}
"#,
            now.to_rfc3339()
        );
        std::fs::write(session_dir.join("rollout-def456.jsonl"), jsonl_content).unwrap();

        let records = service.parse_all_logs().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].input_tokens, 24763);
        assert_eq!(records[0].output_tokens, 122);
        assert_eq!(records[0].model.as_deref(), Some("o4-mini"));
    }

    #[test]
    fn test_parse_session_with_turn_context() {
        let (service, temp_dir) = create_test_service();

        let session_dir = temp_dir.path().join("sessions").join("2026/03/16");
        std::fs::create_dir_all(&session_dir).unwrap();

        let now = Utc::now();
        // turn_context 更新 model 后再出现 token_count
        let jsonl_content = format!(
            r#"{{"session_id":"sess-3","model":"codex-mini-latest","created_at":"{}"}}
{{"event_msg":{{"payload":{{"type":"turn_context","model":"o3"}}}}}}
{{"event_msg":{{"payload":{{"type":"token_count","input_tokens":500,"output_tokens":100}}}}}}
"#,
            now.to_rfc3339()
        );
        std::fs::write(session_dir.join("rollout-ghi789.jsonl"), jsonl_content).unwrap();

        let records = service.parse_all_logs().unwrap();
        assert_eq!(records.len(), 1);
        // model 应该被 turn_context 更新为 o3
        assert_eq!(records[0].model.as_deref(), Some("o3"));
        assert_eq!(records[0].input_tokens, 500);
        assert_eq!(records[0].output_tokens, 100);
    }

    #[test]
    fn test_dedup_prefers_turn_completed() {
        let (service, temp_dir) = create_test_service();

        let session_dir = temp_dir.path().join("sessions").join("2026/03/16");
        std::fs::create_dir_all(&session_dir).unwrap();

        let now = Utc::now();
        // 文件同时包含 token_count 和 turn.completed
        let jsonl_content = format!(
            r#"{{"session_id":"sess-4","model":"codex-mini-latest","created_at":"{}"}}
{{"event_msg":{{"payload":{{"type":"token_count","input_tokens":1000,"output_tokens":200}}}}}}
{{"type":"turn.completed","usage":{{"input_tokens":1000,"output_tokens":200}}}}
"#,
            now.to_rfc3339()
        );
        std::fs::write(session_dir.join("rollout-jkl012.jsonl"), jsonl_content).unwrap();

        let records = service.parse_all_logs().unwrap();
        // 应该只返回 turn.completed 的记录，不重复
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].input_tokens, 1000);
    }

    #[test]
    fn test_parse_session_with_current_format_token_count() {
        let (service, temp_dir) = create_test_service();

        let session_dir = temp_dir.path().join("sessions").join("2026/03/05");
        std::fs::create_dir_all(&session_dir).unwrap();

        let jsonl_content = r#"{"timestamp":"2026-03-05T09:11:45.366Z","type":"session_meta","payload":{"id":"sess-current","timestamp":"2026-03-05T09:11:45.366Z","model":"gpt-5"}}
{"timestamp":"2026-03-05T09:11:50.406Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":1000,"cached_input_tokens":400,"output_tokens":200}}}}
{"timestamp":"2026-03-05T09:12:01.000Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":1800,"cached_input_tokens":700,"output_tokens":260}}}}
"#;
        std::fs::write(session_dir.join("rollout-current.jsonl"), jsonl_content).unwrap();

        let records = service.parse_all_logs().unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].model.as_deref(), Some("gpt-5"));
        assert_eq!(records[0].input_tokens, 1000);
        assert_eq!(records[0].output_tokens, 200);
        assert_eq!(records[1].input_tokens, 800);
        assert_eq!(records[1].output_tokens, 60);
    }

    #[test]
    fn test_parse_session_with_top_level_turn_context_and_session_id() {
        let (service, temp_dir) = create_test_service();

        let session_dir = temp_dir.path().join("sessions").join("2026/03/05");
        std::fs::create_dir_all(&session_dir).unwrap();

        let jsonl_content = r#"{"timestamp":"2026-03-05T09:11:45.366Z","type":"session_meta","id":"sess-top-level","cwd":"D:\\Documents\\Code\\Github\\ccr","payload":{"timestamp":"2026-03-05T09:11:45.366Z"}}
{"timestamp":"2026-03-05T09:11:46.000Z","type":"turn_context","payload":{"turn_id":"turn-1","model":"gpt-5.4"}}
{"timestamp":"2026-03-05T09:11:50.406Z","type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":1000,"cached_input_tokens":400,"output_tokens":200}}}}
"#;
        std::fs::write(session_dir.join("rollout-top-level.jsonl"), jsonl_content).unwrap();

        let records = service.parse_all_logs().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].session_id, "sess-top-level");
        assert_eq!(records[0].model.as_deref(), Some("gpt-5.4"));
        assert_eq!(records[0].input_tokens, 1000);
        assert_eq!(records[0].output_tokens, 200);
    }

    #[test]
    fn test_compute_rolling_usage() {
        let (service, temp_dir) = create_test_service();

        let session_dir = temp_dir.path().join("sessions").join("2026/03/16");
        std::fs::create_dir_all(&session_dir).unwrap();

        let now = Utc::now();
        let recent = now - Duration::hours(1);
        let jsonl_content = format!(
            r#"{{"session_id":"sess-5","model":"codex-mini-latest","created_at":"{}"}}
{{"event_msg":{{"payload":{{"type":"token_count","input_tokens":100,"output_tokens":50}}}}}}
"#,
            recent.to_rfc3339()
        );
        std::fs::write(session_dir.join("rollout-mno345.jsonl"), jsonl_content).unwrap();

        let rolling = service.compute_rolling_usage().unwrap();
        assert_eq!(rolling.five_hour.total_requests, 1);
        assert_eq!(rolling.five_hour.total_input_tokens, 100);
        assert_eq!(rolling.five_hour.total_output_tokens, 50);
        assert_eq!(rolling.seven_day.total_requests, 1);
        assert_eq!(rolling.all_time.total_requests, 1);
    }

    #[test]
    fn test_usage_stats_total_tokens() {
        let stats = CodexUsageStats {
            total_input_tokens: 100,
            total_output_tokens: 50,
            total_requests: 1,
            window_start: None,
            window_end: None,
        };
        assert_eq!(stats.total_tokens(), 150);
    }

    #[test]
    #[ignore]
    fn benchmark_compute_rolling_usage_cache() {
        for file_count in [300usize, 2_000, 10_000] {
            let (service, temp_dir) = create_test_service();
            let session_dir = temp_dir.path().join("sessions").join("2026/03/16");
            std::fs::create_dir_all(&session_dir).unwrap();

            for index in 0..file_count {
                let jsonl_content = format!(
                    r#"{{"session_id":"usage-bench-{index}","model":"gpt-5","created_at":"2026-03-16T00:00:00Z"}}
{{"timestamp":"2026-03-16T00:00:01Z","type":"turn.completed","usage":{{"input_tokens":100,"output_tokens":25}}}}
{{"timestamp":"2026-03-16T00:00:02Z","type":"turn.completed","usage":{{"input_tokens":200,"output_tokens":50}}}}
"#
                );
                std::fs::write(
                    session_dir.join(format!("rollout-usage-bench-{index}.jsonl")),
                    jsonl_content,
                )
                .unwrap();
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
