// Codex CLI 配置数据模型（ccr-db 精简版）
// 仅包含 converter_service 所需的配置类型，不含 API 响应类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Codex 完整配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct CodexConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_provider: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_effort: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_summary: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_verbosity: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_context_window: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_auto_compact_token_limit: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub personality: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_policy: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_mode: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_response_storage: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_workspace_write: Option<SandboxWorkspaceWrite>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_environment_policy: Option<ShellEnvironmentPolicy>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_opener: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub developer_instructions: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tui: Option<TuiConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_agent_reasoning: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_raw_agent_reasoning: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_for_update_on_startup: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_unstable_features_warning: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, CodexMcpServer>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<HashMap<String, CodexProfile>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_use_rmcp_client: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<HistoryConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics: Option<AnalyticsConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<FeedbackConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<HashMap<String, bool>>,

    /// 保留未知字段
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

/// 沙盒工作区写入配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct SandboxWorkspaceWrite {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writable_roots: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_access: Option<bool>,
}

/// TUI 界面配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct TuiConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate_screen: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub animations: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifications: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_tooltips: Option<bool>,
}

/// 历史记录配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct HistoryConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bytes: Option<u64>,
}

/// 分析配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct AnalyticsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// 反馈配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct FeedbackConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

/// 工具配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ToolsConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_image: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<bool>,
}

/// Shell 环境策略
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ShellEnvironmentPolicy {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_only: Option<Vec<String>>,
}

/// Codex MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CodexMcpServer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup_timeout_ms: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_token: Option<String>,

    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

/// Codex Profile 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CodexProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_policy: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_mode: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_effort: Option<String>,

    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}
