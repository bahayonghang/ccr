// ─────────────────────────────────────────────────────────────────────
// 端口自 vibe-observer（MIT License）
//   原始路径: ref/repo/vibe-observer/crates/observer-ingest/src/jsonl.rs
// 仅保留 tool_use / tool_result 提取链路；丢弃 assistant 文本、user prompt、
// summary、compact、session_start 等与「工具调用维度行为分析」无关的事件。
// ─────────────────────────────────────────────────────────────────────

use chrono::{DateTime, Utc};
use serde::Deserialize;
use sha2::{Digest, Sha256};

/// 已解析的单条 tool 事件（tool_use 与 tool_result 共用同一结构）
///
/// `tool_use` 行：`tool_name` 已知；`success` 字段为 `None`（待后续 tool_result 回填）。
/// `tool_result` 行：`tool_name` 为空串；`success` 可被读取，但调用方需通过
///   `dedup_key`（即 tool_use_id）将 success 状态合并回对应 tool_use 行——本模块不
///   做合并，只输出原始事件。
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedToolEvent {
    /// 行类型：`"tool_use"` 或 `"tool_result"`
    pub kind: ToolEventKind,
    /// 上报的 session_id（来自 JSONL 行）
    pub session_id: String,
    /// 上报时间戳（RFC3339）
    pub ts: DateTime<Utc>,
    /// 项目路径提示（cwd 字段或文件路径推断；可能为 None）
    pub project_path: Option<String>,
    /// 工具名（仅 tool_use 行有；tool_result 行为空串）
    pub tool_name: String,
    /// 成功标记（仅 tool_result 行有意义）
    pub success: Option<bool>,
    /// tool_use 的唯一标识。
    /// - tool_use 行：取 `message.content[].id` 字段
    /// - tool_result 行：取 `message.content[].tool_use_id` 字段
    ///
    /// 调用方据此把 tool_result 的 success 回填到对应 tool_use。
    pub dedup_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolEventKind {
    ToolUse,
    ToolResult,
}

#[derive(Debug, Deserialize)]
struct JsonlLineRaw {
    #[serde(rename = "type")]
    type_: Option<String>,
    #[serde(rename = "sessionId", alias = "session_id")]
    session_id: Option<String>,
    #[serde(rename = "timestamp", alias = "ts")]
    timestamp: Option<String>,
    cwd: Option<String>,
    #[serde(flatten)]
    rest: serde_json::Map<String, serde_json::Value>,
}

/// 解析单行 JSONL，返回 0..N 条 tool 事件。
///
/// - 行不是 JSON / 不是 tool 相关 / 缺关键字段时，返回空 Vec。
/// - 多个 tool_use / tool_result 块在同一行时按数组顺序展开。
/// - `project_path_hint` 用于在 JSONL 缺 `cwd` 时回退（例如从文件路径反推）。
pub fn parse_line(line: &str, project_path_hint: Option<&str>) -> Vec<ParsedToolEvent> {
    /* ====================================================================
     * 步骤1：JSON 解析与共享字段抽取
     * ====================================================================
     */
    let Ok(raw) = serde_json::from_str::<JsonlLineRaw>(line) else {
        return Vec::new();
    };
    let session_id = match raw.session_id.clone() {
        Some(sid) if !sid.is_empty() => sid,
        _ => return Vec::new(),
    };
    let ts = raw
        .timestamp
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|t| t.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);
    let project_path = raw
        .cwd
        .clone()
        .filter(|c| !c.is_empty())
        .or_else(|| project_path_hint.map(|s| s.to_string()));

    /* ====================================================================
     * 步骤2：按行类型分发，只提取 tool_use / tool_result
     * ====================================================================
     * - assistant 行携带 tool_use 块（嵌在 message.content[]）
     * - user      行携带 tool_result 块（同样在 message.content[]）
     * - 其他类型一律忽略
     */
    let mut out = Vec::new();
    match raw.type_.as_deref() {
        Some("assistant") => {
            for tool in extract_tool_uses(&raw.rest) {
                let id = tool
                    .get("id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| compute_fallback_id(&session_id, &ts, &tool));
                let name = tool
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                out.push(ParsedToolEvent {
                    kind: ToolEventKind::ToolUse,
                    session_id: session_id.clone(),
                    ts,
                    project_path: project_path.clone(),
                    tool_name: name,
                    success: None,
                    dedup_key: id,
                });
            }
        }
        Some("user") => {
            for result in extract_tool_results(&raw.rest) {
                let Some(tool_use_id) = result
                    .get("tool_use_id")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                else {
                    continue;
                };
                let is_error = result
                    .get("is_error")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                out.push(ParsedToolEvent {
                    kind: ToolEventKind::ToolResult,
                    session_id: session_id.clone(),
                    ts,
                    project_path: project_path.clone(),
                    tool_name: String::new(),
                    success: Some(!is_error),
                    dedup_key: tool_use_id,
                });
            }
        }
        _ => {}
    }
    out
}

/// 从 assistant 行的 `message.content[]` 中筛出 type=tool_use 的块
fn extract_tool_uses(rest: &serde_json::Map<String, serde_json::Value>) -> Vec<serde_json::Value> {
    let Some(msg) = rest.get("message") else {
        return vec![];
    };
    let Some(content) = msg.get("content").and_then(|v| v.as_array()) else {
        return vec![];
    };
    content
        .iter()
        .filter(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_use"))
        .cloned()
        .collect()
}

/// 从 user 行的 `message.content[]` 中筛出 type=tool_result 的块（只取 tool_use_id + is_error）
fn extract_tool_results(
    rest: &serde_json::Map<String, serde_json::Value>,
) -> Vec<serde_json::Value> {
    let Some(msg) = rest.get("message") else {
        return vec![];
    };
    let Some(content) = msg.get("content").and_then(|v| v.as_array()) else {
        return vec![];
    };
    content
        .iter()
        .filter(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_result"))
        .cloned()
        .collect()
}

/// 当 tool_use 块缺 `id` 时退化为 sha256(session_id || ts || payload) 前 16 字符。
/// 实际生产数据里 Claude Code 总是带 id，这里只是兜底以保证去重键非空。
fn compute_fallback_id(
    session_id: &str,
    ts: &DateTime<Utc>,
    payload: &serde_json::Value,
) -> String {
    let mut h = Sha256::new();
    h.update(session_id.as_bytes());
    h.update(b"\0");
    h.update(ts.to_rfc3339().as_bytes());
    h.update(b"\0");
    h.update(payload.to_string().as_bytes());
    let digest = h.finalize();
    hex_encode_short(&digest[..8])
}

fn hex_encode_short(bytes: &[u8]) -> String {
    const HEX: &[u8] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assistant_line_emits_tool_use_events() {
        let line = r#"{
            "type":"assistant",
            "sessionId":"s1",
            "timestamp":"2026-05-15T10:00:00Z",
            "cwd":"/repo/foo",
            "message":{"content":[
                {"type":"text","text":"hi"},
                {"type":"tool_use","id":"tu_1","name":"Read","input":{}},
                {"type":"tool_use","id":"tu_2","name":"Bash","input":{}}
            ]}
        }"#;
        let evs = parse_line(line, None);
        assert_eq!(evs.len(), 2);
        assert_eq!(evs[0].kind, ToolEventKind::ToolUse);
        assert_eq!(evs[0].tool_name, "Read");
        assert_eq!(evs[0].dedup_key, "tu_1");
        assert_eq!(evs[0].project_path.as_deref(), Some("/repo/foo"));
        assert_eq!(evs[1].tool_name, "Bash");
    }

    #[test]
    fn user_line_emits_tool_result_events() {
        let line = r#"{
            "type":"user",
            "sessionId":"s1",
            "timestamp":"2026-05-15T10:00:01Z",
            "message":{"role":"user","content":[
                {"type":"tool_result","tool_use_id":"tu_1","is_error":false,"content":"ok"},
                {"type":"tool_result","tool_use_id":"tu_2","is_error":true,"content":"ENOENT"}
            ]}
        }"#;
        let evs = parse_line(line, None);
        assert_eq!(evs.len(), 2);
        assert_eq!(evs[0].kind, ToolEventKind::ToolResult);
        assert_eq!(evs[0].dedup_key, "tu_1");
        assert_eq!(evs[0].success, Some(true));
        assert_eq!(evs[1].dedup_key, "tu_2");
        assert_eq!(evs[1].success, Some(false));
    }

    #[test]
    fn ignores_summary_and_meta_lines() {
        let lines = [
            r#"{"type":"summary","sessionId":"s1","timestamp":"2026-05-15T10:00:00Z"}"#,
            r#"{"type":"session_start","sessionId":"s1","timestamp":"2026-05-15T10:00:00Z"}"#,
            r#"{"type":"meta","sessionId":"s1","timestamp":"2026-05-15T10:00:00Z"}"#,
        ];
        for l in lines {
            assert!(
                parse_line(l, None).is_empty(),
                "line should be ignored: {l}"
            );
        }
    }

    #[test]
    fn invalid_json_returns_empty() {
        assert!(parse_line("not json", None).is_empty());
    }

    #[test]
    fn missing_session_id_returns_empty() {
        let line = r#"{"type":"assistant","timestamp":"2026-05-15T10:00:00Z","message":{"content":[{"type":"tool_use","id":"tu_x","name":"Read","input":{}}]}}"#;
        assert!(parse_line(line, None).is_empty());
    }

    #[test]
    fn falls_back_to_project_hint_when_cwd_missing() {
        let line = r#"{"type":"assistant","sessionId":"s1","timestamp":"2026-05-15T10:00:00Z","message":{"content":[{"type":"tool_use","id":"tu_x","name":"Read","input":{}}]}}"#;
        let evs = parse_line(line, Some("/repo/from-path"));
        assert_eq!(evs[0].project_path.as_deref(), Some("/repo/from-path"));
    }
}
