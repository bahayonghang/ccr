//! Codex 会话服务
//!
//! 提供 Codex CLI session JSONL 的列表、详情、导出、克隆与删除能力。
//! 会话文件默认位于 `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl`。

use crate::core::error::{CcrError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Reverse;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use uuid::Uuid;

const DEFAULT_DETAIL_MESSAGE_LIMIT: usize = 120;
const DEFAULT_EXPORT_MESSAGE_LIMIT: usize = 200;
const PREVIEW_MAX_CHARS: usize = 160;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexSessionSummary {
    pub session_id: String,
    pub file_path: PathBuf,
    pub relative_path: String,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub cli_version: Option<String>,
    pub originator: Option<String>,
    pub source: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub message_count: usize,
    pub preview: Option<String>,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_requests: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexSessionMessage {
    pub role: String,
    pub text: String,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexSessionDetail {
    pub session: CodexSessionSummary,
    pub messages: Vec<CodexSessionMessage>,
    pub clipped: bool,
    pub message_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexSessionExport {
    pub session_id: String,
    pub file_name: String,
    pub content: String,
    pub truncated: bool,
    pub max_messages: usize,
}

#[derive(Debug, Clone, Default)]
struct CodexSessionMeta {
    session_id: String,
    cwd: Option<String>,
    model: Option<String>,
    cli_version: Option<String>,
    originator: Option<String>,
    source: Option<String>,
    created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
struct SessionTokenAccumulator {
    input_tokens: u64,
    output_tokens: u64,
    requests: u64,
}

#[derive(Debug, Clone)]
struct ParsedCodexSession {
    summary: CodexSessionSummary,
    messages: Vec<CodexSessionMessage>,
}

#[derive(Debug, Clone, Default)]
struct CodexSessionState {
    meta: CodexSessionMeta,
    updated_at: Option<DateTime<Utc>>,
    preview: Option<String>,
    message_count: usize,
    messages: Vec<CodexSessionMessage>,
    token_count_usage: SessionTokenAccumulator,
    completed_usage: SessionTokenAccumulator,
    prev_input_tokens: u64,
    prev_output_tokens: u64,
}

pub struct CodexSessionService {
    codex_dir: PathBuf,
}

impl CodexSessionService {
    pub fn new(codex_dir: PathBuf) -> Self {
        Self { codex_dir }
    }

    fn sessions_dir(&self) -> PathBuf {
        self.codex_dir.join("sessions")
    }

    pub fn count_sessions(&self) -> Result<usize> {
        let sessions_dir = self.sessions_dir();
        if !sessions_dir.exists() {
            return Ok(0);
        }

        Ok(Self::collect_jsonl_files(&sessions_dir).len())
    }

    pub fn list_sessions(
        &self,
        limit: usize,
        query: Option<&str>,
    ) -> Result<Vec<CodexSessionSummary>> {
        let sessions_dir = self.sessions_dir();
        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let normalized_query = query
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_ascii_lowercase());

        let mut files = Self::collect_jsonl_files(&sessions_dir);
        files.sort_by_key(|path| Reverse(Self::metadata_time(path)));

        let mut sessions = Vec::new();
        for path in files {
            let parsed = match self.parse_session_file(&path, 0) {
                Ok(parsed) => parsed,
                Err(_) => continue,
            };

            if let Some(query) = normalized_query.as_ref()
                && !Self::matches_query(&parsed.summary, query)
            {
                continue;
            }

            sessions.push(parsed.summary);
            if sessions.len() >= limit {
                break;
            }
        }

        Ok(sessions)
    }

    pub fn get_session_detail(
        &self,
        file_path: &Path,
        message_limit: Option<usize>,
    ) -> Result<CodexSessionDetail> {
        let resolved = self.resolve_session_path(file_path)?;
        let limit = message_limit.unwrap_or(DEFAULT_DETAIL_MESSAGE_LIMIT).max(1);
        let parsed = self.parse_session_file(&resolved, usize::MAX)?;
        let total_messages = parsed.messages.len();
        let start_index = total_messages.saturating_sub(limit);
        let clipped = total_messages > limit;

        Ok(CodexSessionDetail {
            session: parsed.summary,
            messages: parsed.messages.into_iter().skip(start_index).collect(),
            clipped,
            message_limit: limit,
        })
    }

    pub fn export_session_markdown(
        &self,
        file_path: &Path,
        max_messages: Option<usize>,
    ) -> Result<CodexSessionExport> {
        let resolved = self.resolve_session_path(file_path)?;
        let limit = max_messages.unwrap_or(DEFAULT_EXPORT_MESSAGE_LIMIT).max(1);
        let detail = self.get_session_detail(&resolved, Some(limit))?;
        let file_name = format!(
            "codex-session-{}.md",
            sanitize_filename(&detail.session.session_id)
        );
        let content = build_session_markdown(&detail.session, &detail.messages);

        Ok(CodexSessionExport {
            session_id: detail.session.session_id,
            file_name,
            content,
            truncated: detail.clipped,
            max_messages: detail.message_limit,
        })
    }

    pub fn clone_session(&self, file_path: &Path) -> Result<CodexSessionSummary> {
        let resolved = self.resolve_session_path(file_path)?;
        let content = fs::read_to_string(&resolved)?;
        if content.trim().is_empty() {
            return Err(CcrError::FileIoError("会话文件为空".into()));
        }

        let original_session_id = self.parse_session_file(&resolved, 1)?.summary.session_id;
        let new_session_id = format!("clone-{}", Uuid::new_v4());
        let target_dir = resolved
            .parent()
            .ok_or_else(|| CcrError::FileIoError("无法确定会话目录".into()))?;
        let target_path = allocate_clone_target(target_dir)?;

        let clone_timestamp = Utc::now();
        let max_timestamp = content
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter_map(|record| extract_datetime(record.get("timestamp")))
            .max();
        let time_offset = max_timestamp.map(|timestamp| clone_timestamp - timestamp);

        let mut output_lines = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                output_lines.push(String::new());
                continue;
            }

            let mut record = match serde_json::from_str::<Value>(trimmed) {
                Ok(record) => record,
                Err(_) => {
                    output_lines.push(line.to_string());
                    continue;
                }
            };

            rewrite_session_identifiers(&mut record, &original_session_id, &new_session_id);
            rewrite_session_timestamp(&mut record, clone_timestamp, time_offset);
            output_lines.push(serde_json::to_string(&record).map_err(CcrError::JsonError)?);
        }

        fs::write(&target_path, format!("{}\n", output_lines.join("\n")))?;
        self.parse_session_file(&target_path, 0)
            .map(|parsed| parsed.summary)
    }

    pub fn delete_session(&self, file_path: &Path) -> Result<()> {
        let resolved = self.resolve_session_path(file_path)?;
        fs::remove_file(resolved)?;
        Ok(())
    }

    fn parse_session_file(
        &self,
        path: &Path,
        stored_messages_limit: usize,
    ) -> Result<ParsedCodexSession> {
        let resolved = self.resolve_session_path(path)?;
        let file = File::open(&resolved)?;
        let reader = BufReader::new(file);
        let sessions_dir = self.sessions_dir();
        let mut state = CodexSessionState::default();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let record: Value = match serde_json::from_str(trimmed) {
                Ok(record) => record,
                Err(_) => continue,
            };

            if state.meta.session_id.is_empty() {
                let meta = parse_session_meta(&record);
                if !meta.session_id.is_empty()
                    || meta.cwd.is_some()
                    || meta.model.is_some()
                    || meta.created_at.is_some()
                {
                    state.meta = meta;
                }
            }

            if let Some(timestamp) = extract_datetime(record.get("timestamp")) {
                state.updated_at = Some(
                    state
                        .updated_at
                        .map_or(timestamp, |current| current.max(timestamp)),
                );
            }

            self.parse_message_record(&record, &mut state, stored_messages_limit);
            self.parse_usage_record(&record, &mut state);
        }

        let metadata_updated_at = Self::metadata_time(&resolved);
        let updated_at = state.updated_at.or(metadata_updated_at);
        let usage = if state.completed_usage.requests > 0 {
            state.completed_usage
        } else {
            state.token_count_usage
        };

        let session_id = if state.meta.session_id.trim().is_empty() {
            resolved
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown")
                .to_string()
        } else {
            state.meta.session_id.clone()
        };

        let relative_path = resolved
            .strip_prefix(&sessions_dir)
            .ok()
            .map(|value| value.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|| resolved.to_string_lossy().to_string());

        let summary = CodexSessionSummary {
            session_id,
            file_path: resolved,
            relative_path,
            cwd: state.meta.cwd.clone(),
            model: state.meta.model.clone(),
            cli_version: state.meta.cli_version.clone(),
            originator: state.meta.originator.clone(),
            source: state.meta.source.clone(),
            created_at: state.meta.created_at,
            updated_at,
            message_count: state.message_count,
            preview: state.preview.clone(),
            total_input_tokens: usage.input_tokens,
            total_output_tokens: usage.output_tokens,
            total_requests: usage.requests,
        };

        Ok(ParsedCodexSession {
            summary,
            messages: state.messages,
        })
    }

    fn parse_message_record(
        &self,
        record: &Value,
        state: &mut CodexSessionState,
        stored_messages_limit: usize,
    ) {
        let Some(payload) = record.get("payload") else {
            return;
        };
        if record.get("type").and_then(Value::as_str) != Some("response_item") {
            return;
        }
        if payload.get("type").and_then(Value::as_str) != Some("message") {
            return;
        }

        let Some(role) = normalize_message_role(payload.get("role").and_then(Value::as_str)) else {
            return;
        };
        if role != "user" && role != "assistant" {
            return;
        }

        let Some(text) = extract_message_text(payload.get("content")) else {
            return;
        };

        if state.preview.is_none() && role == "user" {
            state.preview = Some(truncate_preview(&text, PREVIEW_MAX_CHARS));
        }
        if state.preview.is_none() {
            state.preview = Some(truncate_preview(&text, PREVIEW_MAX_CHARS));
        }

        state.message_count += 1;

        if stored_messages_limit == 0 || state.messages.len() >= stored_messages_limit {
            return;
        }

        state.messages.push(CodexSessionMessage {
            role: role.to_string(),
            text,
            timestamp: extract_datetime(record.get("timestamp")),
        });
    }

    fn parse_usage_record(&self, record: &Value, state: &mut CodexSessionState) {
        let Some(record_type) = record.get("type").and_then(Value::as_str) else {
            return;
        };

        if record_type == "turn.completed"
            && let Some(usage) = record.get("usage")
        {
            let input_tokens = usage
                .get("input_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0);
            let output_tokens = usage
                .get("output_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0);

            if input_tokens > 0 || output_tokens > 0 {
                state.completed_usage.input_tokens += input_tokens;
                state.completed_usage.output_tokens += output_tokens;
                state.completed_usage.requests += 1;
            }
            return;
        }

        if record_type != "event_msg" {
            return;
        }

        let Some(payload) = record.get("payload") else {
            return;
        };
        if payload.get("type").and_then(Value::as_str) != Some("token_count") {
            return;
        }

        let usage = payload
            .get("info")
            .and_then(|info| info.get("total_token_usage"))
            .unwrap_or(payload);

        let current_input_tokens = usage
            .get("input_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let current_output_tokens = usage
            .get("output_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(0);

        let delta_input_tokens = current_input_tokens.saturating_sub(state.prev_input_tokens);
        let delta_output_tokens = current_output_tokens.saturating_sub(state.prev_output_tokens);

        if delta_input_tokens > 0 || delta_output_tokens > 0 {
            state.token_count_usage.input_tokens += delta_input_tokens;
            state.token_count_usage.output_tokens += delta_output_tokens;
            state.token_count_usage.requests += 1;
        }

        state.prev_input_tokens = current_input_tokens;
        state.prev_output_tokens = current_output_tokens;
    }

    fn resolve_session_path(&self, file_path: &Path) -> Result<PathBuf> {
        let sessions_dir = self.sessions_dir();
        if !sessions_dir.exists() {
            return Err(CcrError::ResourceNotFound(
                "Codex sessions 目录不存在".into(),
            ));
        }

        let root = sessions_dir
            .canonicalize()
            .map_err(|error| CcrError::FileIoError(format!("解析 sessions 目录失败: {error}")))?;
        let candidate = file_path
            .canonicalize()
            .map_err(|error| CcrError::ResourceNotFound(format!("会话文件不存在: {error}")))?;

        if !candidate.starts_with(&root) {
            return Err(CcrError::ValidationError(
                "会话文件必须位于 ~/.codex/sessions 目录内".into(),
            ));
        }
        if candidate.extension().and_then(|value| value.to_str()) != Some("jsonl") {
            return Err(CcrError::ValidationError("仅支持 .jsonl 会话文件".into()));
        }

        Ok(candidate)
    }

    fn matches_query(summary: &CodexSessionSummary, query: &str) -> bool {
        [
            Some(summary.session_id.as_str()),
            summary.cwd.as_deref(),
            summary.model.as_deref(),
            summary.preview.as_deref(),
            Some(summary.relative_path.as_str()),
        ]
        .into_iter()
        .flatten()
        .any(|value| value.to_ascii_lowercase().contains(query))
    }

    fn collect_jsonl_files(dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let Ok(entries) = fs::read_dir(dir) else {
            return files;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(Self::collect_jsonl_files(&path));
            } else if path.extension().and_then(|value| value.to_str()) == Some("jsonl") {
                files.push(path);
            }
        }

        files
    }

    fn metadata_time(path: &Path) -> Option<DateTime<Utc>> {
        let metadata = fs::metadata(path).ok()?;
        let modified = metadata.modified().ok()?;
        Some(DateTime::<Utc>::from(modified))
    }
}

fn parse_session_meta(record: &Value) -> CodexSessionMeta {
    let payload = if record.get("type").and_then(Value::as_str) == Some("session_meta") {
        record.get("payload").unwrap_or(record)
    } else {
        record
    };

    CodexSessionMeta {
        session_id: payload
            .get("id")
            .or_else(|| payload.get("session_id"))
            .or_else(|| record.get("session_id"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        cwd: payload
            .get("cwd")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        model: payload
            .get("model")
            .or_else(|| record.get("model"))
            .and_then(Value::as_str)
            .map(ToString::to_string),
        cli_version: payload
            .get("cli_version")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        originator: payload
            .get("originator")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        source: payload
            .get("source")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        created_at: extract_datetime(
            payload
                .get("timestamp")
                .or_else(|| payload.get("created_at"))
                .or_else(|| record.get("timestamp"))
                .or_else(|| record.get("created_at")),
        ),
    }
}

fn normalize_message_role(value: Option<&str>) -> Option<&'static str> {
    match value?.trim().to_ascii_lowercase().as_str() {
        "user" => Some("user"),
        "assistant" => Some("assistant"),
        "system" | "developer" => Some("system"),
        _ => None,
    }
}

fn extract_datetime(value: Option<&Value>) -> Option<DateTime<Utc>> {
    value
        .and_then(Value::as_str)
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .map(|timestamp| timestamp.with_timezone(&Utc))
}

fn extract_message_text(content: Option<&Value>) -> Option<String> {
    let mut parts = Vec::new();
    collect_text_parts(content?, &mut parts);
    let combined = parts
        .into_iter()
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if combined.trim().is_empty() {
        None
    } else {
        Some(combined)
    }
}

fn collect_text_parts(value: &Value, parts: &mut Vec<String>) {
    match value {
        Value::String(text) => parts.push(text.to_string()),
        Value::Array(items) => {
            for item in items {
                collect_text_parts(item, parts);
            }
        }
        Value::Object(object) => {
            if let Some(text) = object.get("text").and_then(Value::as_str) {
                parts.push(text.to_string());
            }
            if let Some(content) = object.get("content") {
                collect_text_parts(content, parts);
            }
            if let Some(message) = object.get("message") {
                collect_text_parts(message, parts);
            }
        }
        _ => {}
    }
}

fn truncate_preview(text: &str, max_chars: usize) -> String {
    let mut truncated = String::new();
    for (index, ch) in text.chars().enumerate() {
        if index >= max_chars {
            truncated.push('…');
            break;
        }
        truncated.push(ch);
    }
    truncated
}

fn allocate_clone_target(dir_path: &Path) -> Result<PathBuf> {
    for _attempt in 0..6 {
        let file_name = format!("rollout-clone-{}.jsonl", Uuid::new_v4());
        let candidate = dir_path.join(file_name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(CcrError::UpdateError("无法为克隆会话分配目标文件".into()))
}

fn rewrite_session_identifiers(
    record: &mut Value,
    original_session_id: &str,
    new_session_id: &str,
) {
    if let Some(session_id) = record.get_mut("session_id")
        && session_id.as_str() == Some(original_session_id)
    {
        *session_id = Value::String(new_session_id.to_string());
    }

    if let Some(session_id) = record.get_mut("sessionId")
        && session_id.as_str() == Some(original_session_id)
    {
        *session_id = Value::String(new_session_id.to_string());
    }

    if record.get("type").and_then(Value::as_str) == Some("session_meta")
        && let Some(payload) = record.get_mut("payload").and_then(Value::as_object_mut)
    {
        payload.insert("id".into(), Value::String(new_session_id.to_string()));
    }
}

fn rewrite_session_timestamp(
    record: &mut Value,
    clone_timestamp: DateTime<Utc>,
    time_offset: Option<chrono::Duration>,
) {
    if record.get("type").and_then(Value::as_str) == Some("session_meta") {
        if let Some(payload) = record.get_mut("payload").and_then(Value::as_object_mut) {
            payload.insert(
                "timestamp".into(),
                Value::String(clone_timestamp.to_rfc3339()),
            );
        }
        if let Some(timestamp) = record.get_mut("timestamp") {
            *timestamp = Value::String(clone_timestamp.to_rfc3339());
        }
        return;
    }

    let Some(offset) = time_offset else {
        return;
    };

    if let Some(original_timestamp) = extract_datetime(record.get("timestamp"))
        && let Some(timestamp) = record.get_mut("timestamp")
    {
        *timestamp = Value::String((original_timestamp + offset).to_rfc3339());
    }
}

fn sanitize_filename(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn build_session_markdown(
    summary: &CodexSessionSummary,
    messages: &[CodexSessionMessage],
) -> String {
    let mut lines = Vec::new();
    lines.push("# Codex Session Export".to_string());
    lines.push(String::new());
    lines.push(format!("- Session ID: {}", summary.session_id));
    lines.push(format!(
        "- Updated At: {}",
        summary
            .updated_at
            .map(|value| value.to_rfc3339())
            .unwrap_or_else(|| "unknown".into())
    ));
    if let Some(model) = summary.model.as_deref() {
        lines.push(format!("- Model: {model}"));
    }
    if let Some(cwd) = summary.cwd.as_deref() {
        lines.push(format!("- CWD: {cwd}"));
    }
    lines.push(format!("- File: {}", summary.file_path.to_string_lossy()));
    lines.push(String::new());
    lines.push("## Messages".to_string());
    lines.push(String::new());

    for (index, message) in messages.iter().enumerate() {
        let role = if message.role == "assistant" {
            "Assistant"
        } else {
            "User"
        };
        let time_suffix = message
            .timestamp
            .map(|value| format!(" · {}", value.to_rfc3339()))
            .unwrap_or_default();

        lines.push(format!("### {}. {}{}", index + 1, role, time_suffix));
        lines.push(String::new());
        lines.push(message.text.clone());
        lines.push(String::new());
    }

    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }

    lines.join("\n")
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_service() -> (CodexSessionService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let service = CodexSessionService::new(temp_dir.path().to_path_buf());
        (service, temp_dir)
    }

    fn write_session(temp_dir: &TempDir, file_name: &str, content: &str) -> PathBuf {
        let dir = temp_dir.path().join("sessions").join("2026/03/26");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(file_name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn list_sessions_extracts_summary() {
        let (service, temp_dir) = create_service();
        let content = r#"{"timestamp":"2026-03-26T06:37:24.013Z","type":"session_meta","payload":{"id":"sess-1","timestamp":"2026-03-26T06:37:24.013Z","cwd":"D:\\repo","model":"gpt-5","cli_version":"0.116.0","originator":"codex_cli_rs","source":"cli"}}
{"timestamp":"2026-03-26T06:37:40.000Z","type":"response_item","payload":{"type":"message","role":"developer","content":[{"type":"input_text","text":"system prompt"}]}}
{"timestamp":"2026-03-26T06:37:50.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"Implement the plan"}]}}
{"timestamp":"2026-03-26T06:38:00.000Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"Working on it"}]}}
{"timestamp":"2026-03-26T06:38:10.000Z","type":"turn.completed","usage":{"input_tokens":1200,"output_tokens":240}}
"#;
        write_session(&temp_dir, "rollout-test.jsonl", content);

        let sessions = service.list_sessions(10, None).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "sess-1");
        assert_eq!(sessions[0].message_count, 2);
        assert_eq!(sessions[0].preview.as_deref(), Some("Implement the plan"));
        assert_eq!(sessions[0].total_input_tokens, 1200);
        assert_eq!(sessions[0].total_output_tokens, 240);
        assert_eq!(sessions[0].total_requests, 1);
    }

    #[test]
    fn get_session_detail_clips_messages() {
        let (service, temp_dir) = create_service();
        let path = write_session(
            &temp_dir,
            "rollout-detail.jsonl",
            r#"{"timestamp":"2026-03-26T06:37:24.013Z","type":"session_meta","payload":{"id":"sess-detail","timestamp":"2026-03-26T06:37:24.013Z","cwd":"D:\\repo","model":"gpt-5"}}
{"timestamp":"2026-03-26T06:37:50.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"one"}]}}
{"timestamp":"2026-03-26T06:38:00.000Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"two"}]}}
{"timestamp":"2026-03-26T06:38:10.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"three"}]}}
"#,
        );

        let detail = service.get_session_detail(&path, Some(2)).unwrap();
        assert!(detail.clipped);
        assert_eq!(detail.messages.len(), 2);
        assert_eq!(detail.messages[0].text, "two");
        assert_eq!(detail.messages[1].text, "three");
    }

    #[test]
    fn clone_session_generates_new_session_id() {
        let (service, temp_dir) = create_service();
        let path = write_session(
            &temp_dir,
            "rollout-clone-source.jsonl",
            r#"{"timestamp":"2026-03-26T06:37:24.013Z","type":"session_meta","payload":{"id":"sess-clone","timestamp":"2026-03-26T06:37:24.013Z","cwd":"D:\\repo","model":"gpt-5"}}
{"timestamp":"2026-03-26T06:37:50.000Z","type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"input_text","text":"copy me"}]}}
"#,
        );

        let cloned = service.clone_session(&path).unwrap();
        assert_ne!(cloned.session_id, "sess-clone");
        assert!(cloned.session_id.starts_with("clone-"));
        assert!(cloned.file_path.exists());
    }
}
