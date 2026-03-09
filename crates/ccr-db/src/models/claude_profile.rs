// Claude Profile 数据模型 — 存储 Claude Code 配置快照

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Claude Code Profile — 完整配置快照
///
/// 每个 Profile 包含 settings、MCP 服务器、output styles、budget 等的完整快照，
/// 可以在不同配置之间快速切换。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeProfile {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 完整快照 JSON（包含 settings/mcp_servers/output_styles/budget）
    pub snapshot_json: String,
    /// 标签（JSON array string，如 '["work","personal"]'）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    /// 当前激活的 Profile
    pub is_current: bool,
    /// 启用/禁用
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ClaudeProfile {
    /// 创建新 Profile
    pub fn new(name: String, snapshot_json: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            snapshot_json,
            tags: None,
            is_current: false,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// 解析标签 JSON
    #[allow(dead_code)]
    pub fn parsed_tags(&self) -> Vec<String> {
        self.tags
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default()
    }
}
