//! 📋 Session 数据模型
//!
//! 定义 Session 及其相关类型。

use crate::models::Platform;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 📋 Session 摘要（用于列表展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// 唯一标识
    pub id: String,
    /// 所属平台
    pub platform: Platform,
    /// 标题
    pub title: Option<String>,
    /// 工作目录
    pub cwd: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 消息总数
    pub message_count: u32,
}

#[allow(dead_code)]
impl SessionSummary {
    /// 获取显示标题
    pub fn display_title(&self) -> &str {
        self.title.as_deref().unwrap_or(self.id.as_str())
    }

    /// 格式化持续时间
    pub fn duration_display(&self) -> String {
        let duration = self.updated_at.signed_duration_since(self.created_at);
        let minutes = duration.num_minutes();

        if minutes < 60 {
            format!("{}m", minutes)
        } else {
            let hours = minutes / 60;
            let mins = minutes % 60;
            format!("{}h {}m", hours, mins)
        }
    }

    /// 格式化相对时间（例如：3小时前）
    pub fn relative_time(&self) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.updated_at);

        if duration.num_minutes() < 1 {
            "刚刚".to_string()
        } else if duration.num_minutes() < 60 {
            format!("{}分钟前", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{}小时前", duration.num_hours())
        } else if duration.num_days() < 7 {
            format!("{}天前", duration.num_days())
        } else {
            self.updated_at.format("%Y-%m-%d").to_string()
        }
    }
}

/// 📄 Session 完整信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// 唯一标识
    pub id: String,
    /// 所属平台
    pub platform: Platform,
    /// 标题
    pub title: Option<String>,
    /// 工作目录
    pub cwd: PathBuf,
    /// 源文件路径
    pub file_path: PathBuf,
    /// 文件哈希（用于增量更新）
    pub file_hash: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 消息总数
    pub message_count: u32,
    /// 用户消息数
    pub user_message_count: u32,
    /// 助手消息数
    pub assistant_message_count: u32,
    /// 工具调用数
    pub tool_use_count: u32,
    /// 索引时间
    pub indexed_at: DateTime<Utc>,
}

#[allow(dead_code)]
impl Session {
    /// 转换为摘要
    pub fn to_summary(&self) -> SessionSummary {
        SessionSummary {
            id: self.id.clone(),
            platform: self.platform,
            title: self.title.clone(),
            cwd: self.cwd.to_string_lossy().to_string(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            message_count: self.message_count,
        }
    }

    /// 生成恢复命令
    pub fn resume_command(&self) -> String {
        match self.platform {
            Platform::Claude => format!("claude --resume {}", self.id),
            Platform::Codex => format!("codex resume {}", self.id),
            Platform::Gemini => format!("gemini --continue {}", self.id),
            Platform::Qwen => format!("qwen --resume {}", self.id),
            Platform::IFlow => format!("iflow --resume {}", self.id),
            Platform::Droid => format!("droid --resume {}", self.id),
        }
    }
}

use serde_json::Value;

/// 📝 Session 事件（JSONL 行）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    /// 事件类型
    #[serde(rename = "type")]
    pub event_type: String,

    /// 角色（user/assistant/system）
    #[serde(default)]
    pub role: Option<String>,

    /// 消息内容（可能是字符串或对象）
    #[serde(default)]
    pub message: Option<Value>,

    /// 时间戳
    #[serde(default)]
    pub timestamp: Option<String>,

    /// 工具名称（如果是工具调用）
    #[serde(default)]
    pub tool_name: Option<String>,

    /// Session ID
    #[serde(default)]
    pub session_id: Option<String>,

    /// 工作目录
    #[serde(default)]
    pub cwd: Option<String>,

    /// 原始 JSON（用于调试）
    #[serde(skip)]
    pub raw_json: Option<String>,
}

#[allow(dead_code)]
impl SessionEvent {
    /// 获取消息文本内容
    pub fn message_text(&self) -> Option<String> {
        match &self.message {
            Some(Value::String(s)) => Some(s.clone()),
            Some(Value::Object(map)) => {
                // 尝试从 content 字段获取
                if let Some(content) = map.get("content").and_then(|v| v.as_str()) {
                    return Some(content.to_string());
                }
                // 某些格式可能在 text 字段
                if let Some(text) = map.get("text").and_then(|v| v.as_str()) {
                    return Some(text.to_string());
                }
                None
            }
            _ => None,
        }
    }

    /// 是否是用户消息
    pub fn is_user_message(&self) -> bool {
        // 检查顶层 role
        if self.role.as_deref() == Some("user") {
            return true;
        }

        // 检查 event type
        if self.event_type == "user" || self.event_type == "human" {
            return true;
        }

        // 检查 message 对象里的 role
        if let Some(Value::Object(map)) = &self.message
            && let Some(role) = map.get("role").and_then(|v| v.as_str())
        {
            return role == "user";
        }

        false
    }

    /// 是否是助手消息
    pub fn is_assistant_message(&self) -> bool {
        if self.role.as_deref() == Some("assistant") {
            return true;
        }

        if self.event_type == "assistant" || self.event_type == "text" {
            return true;
        }

        // 检查 message 对象里的 role
        if let Some(Value::Object(map)) = &self.message
            && let Some(role) = map.get("role").and_then(|v| v.as_str())
        {
            return role == "assistant";
        }

        false
    }

    /// 是否是工具调用
    pub fn is_tool_use(&self) -> bool {
        self.event_type == "tool_use" || self.event_type == "tool_call" || self.tool_name.is_some()
    }

    /// 是否是 session 开始事件
    pub fn is_session_start(&self) -> bool {
        self.event_type == "init"
            || self.event_type == "session_start"
            || self.event_type == "start"
    }
}

/// 🔍 Session 过滤条件
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct SessionFilter {
    /// 平台过滤
    pub platform: Option<Platform>,
    /// 日期范围起始
    pub from_date: Option<DateTime<Utc>>,
    /// 日期范围结束
    pub to_date: Option<DateTime<Utc>>,
    /// 工作目录前缀
    pub cwd_prefix: Option<String>,
    /// 限制数量
    pub limit: Option<usize>,
    /// 偏移量
    pub offset: Option<usize>,
    /// 仅今天
    pub today_only: bool,
}

#[allow(dead_code)]
impl SessionFilter {
    /// 创建仅今天的过滤器
    pub fn today() -> Self {
        Self {
            today_only: true,
            ..Default::default()
        }
    }

    /// 创建指定平台的过滤器
    pub fn for_platform(platform: Platform) -> Self {
        Self {
            platform: Some(platform),
            ..Default::default()
        }
    }

    /// 设置限制
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// 📊 索引统计
#[derive(Debug, Clone, Default)]
pub struct IndexStats {
    /// 扫描文件数
    pub files_scanned: u64,
    /// 新增 session 数
    pub sessions_added: u64,
    /// 更新 session 数
    pub sessions_updated: u64,
    /// 跳过文件数（未变化）
    pub files_skipped: u64,
    /// 错误数
    pub errors: u64,
    /// 耗时（毫秒）
    pub duration_ms: u64,
}

impl IndexStats {
    /// 合并统计
    pub fn merge(&mut self, other: &IndexStats) {
        self.files_scanned += other.files_scanned;
        self.sessions_added += other.sessions_added;
        self.sessions_updated += other.sessions_updated;
        self.files_skipped += other.files_skipped;
        self.errors += other.errors;
        self.duration_ms += other.duration_ms;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_summary_display_title() {
        let summary = SessionSummary {
            id: "abc123".to_string(),
            platform: Platform::Claude,
            title: Some("Test Session".to_string()),
            cwd: "/tmp".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            message_count: 10,
        };

        assert_eq!(summary.display_title(), "Test Session");

        let summary_no_title = SessionSummary {
            title: None,
            ..summary
        };

        assert_eq!(summary_no_title.display_title(), "abc123");
    }

    #[test]
    fn test_session_event_types() {
        let user_event = SessionEvent {
            event_type: "user".to_string(),
            role: Some("user".to_string()),
            message: None,
            timestamp: None,
            tool_name: None,
            session_id: None,
            cwd: None,
            raw_json: None,
        };

        assert!(user_event.is_user_message());
        assert!(!user_event.is_assistant_message());
        assert!(!user_event.is_tool_use());

        let tool_event = SessionEvent {
            event_type: "tool_use".to_string(),
            tool_name: Some("read_file".to_string()),
            ..user_event.clone()
        };

        assert!(tool_event.is_tool_use());
    }
}
