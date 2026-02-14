// Codex CLI 配置数据模型
// 用于读写 ~/.codex/config.toml

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export shared types from ccr-types
pub use ccr_types::{LoginState, TokenFreshness};

/// Codex 完整配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct CodexConfig {
    // ============ 基础配置 ============
    /// 默认模型（如 "gpt-5", "gpt-5-codex"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// 模型提供者（如 "openai", "ollama"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_provider: Option<String>,

    /// 推理深度（"low", "medium", "high"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_effort: Option<String>,

    // ============ 模型与推理（扩展） ============
    /// 推理摘要模式（"auto", "concise", "detailed", "none"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_summary: Option<String>,

    /// 模型详细程度（"low", "medium", "high"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_verbosity: Option<String>,

    /// 上下文窗口大小
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_context_window: Option<i64>,

    /// 自动压缩 token 限制
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_auto_compact_token_limit: Option<i64>,

    /// 个性模式（"none", "friendly", "pragmatic"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personality: Option<String>,

    // ============ 安全策略 ============
    /// 批准策略（"auto", "on-request", "read-only", "full-access"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_policy: Option<String>,

    /// 沙盒模式（如 "workspace-write"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_mode: Option<String>,

    /// 禁用响应存储
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_response_storage: Option<bool>,

    /// 沙盒工作区写入配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_workspace_write: Option<SandboxWorkspaceWrite>,

    // ============ Shell 环境策略 ============
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_environment_policy: Option<ShellEnvironmentPolicy>,

    // ============ 工具与搜索 ============
    /// 网页搜索模式（"disabled", "cached", "live"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search: Option<String>,

    /// 文件打开器（"vscode", "cursor", "windsurf", "none"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_opener: Option<String>,

    /// 开发者指令
    #[serde(skip_serializing_if = "Option::is_none")]
    pub developer_instructions: Option<String>,

    /// 系统指令
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// 工具配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsConfig>,

    // ============ TUI 与界面 ============
    /// TUI 配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tui: Option<TuiConfig>,

    /// 隐藏 Agent 推理过程
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_agent_reasoning: Option<bool>,

    /// 显示原始 Agent 推理
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_raw_agent_reasoning: Option<bool>,

    /// 启动时检查更新
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_for_update_on_startup: Option<bool>,

    /// 抑制不稳定功能警告
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_unstable_features_warning: Option<bool>,

    // ============ MCP 服务器配置 ============
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, CodexMcpServer>>,

    // ============ Profiles 配置 ============
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<HashMap<String, CodexProfile>>,

    // ============ 实验性特性与功能开关 ============
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_use_rmcp_client: Option<bool>,

    /// 历史记录配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<HistoryConfig>,

    /// 分析配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics: Option<AnalyticsConfig>,

    /// 反馈配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<FeedbackConfig>,

    /// 功能开关
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<HashMap<String, bool>>,

    // ============ 其他未知字段 ============
    /// 保留未知字段，避免覆盖用户自定义配置
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
    /// 备用屏幕模式（"auto", "always", "never"）
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
    /// 持久化模式（"save-all", "none"）
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
    /// 仅包含这些环境变量（白名单）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_only: Option<Vec<String>>,
}

/// Codex MCP 服务器配置（支持 STDIO 和 HTTP 两种模式）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CodexMcpServer {
    // ============ STDIO 服务器字段 ============
    /// 启动命令（如 "npx"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// 命令参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,

    /// 环境变量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,

    /// 工作目录
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,

    /// 启动超时（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup_timeout_ms: Option<u64>,

    // ============ HTTP 服务器字段 ============
    /// 服务器 URL（HTTP 模式）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Bearer Token（HTTP 模式）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_token: Option<String>,

    // ============ 其他未知字段 ============
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

impl CodexMcpServer {
    /// 判断是否为 STDIO 服务器
    #[allow(dead_code)]
    pub fn is_stdio(&self) -> bool {
        self.command.is_some()
    }

    /// 判断是否为 HTTP 服务器
    #[allow(dead_code)]
    pub fn is_http(&self) -> bool {
        self.url.is_some()
    }
}

/// Codex Profile 配置（继承主配置的字段）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CodexProfile {
    /// Profile 的模型配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Profile 的批准策略
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_policy: Option<String>,

    /// Profile 的沙盒模式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_mode: Option<String>,

    /// Profile 的推理深度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_effort: Option<String>,

    // ============ 其他未知字段 ============
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
}

// ============ API 请求/响应模型 ============

/// 列出 Codex MCP 服务器的响应
#[derive(Debug, Serialize)]
pub struct CodexMcpListResponse {
    pub servers: Vec<CodexMcpServerWithName>,
}

/// 带名称的 Codex MCP 服务器
#[derive(Debug, Serialize, Deserialize)]
pub struct CodexMcpServerWithName {
    pub name: String,
    #[serde(flatten)]
    pub server: CodexMcpServer,
}

/// 添加/更新 Codex MCP 服务器的请求
#[derive(Debug, Deserialize)]
pub struct CodexMcpServerRequest {
    pub name: Option<String>,
    // STDIO 字段
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub cwd: Option<String>,
    pub startup_timeout_ms: Option<u64>,

    // HTTP 字段
    pub url: Option<String>,
    pub bearer_token: Option<String>,
}

impl From<CodexMcpServerRequest> for CodexMcpServer {
    fn from(req: CodexMcpServerRequest) -> Self {
        CodexMcpServer {
            command: req.command,
            args: req.args,
            env: req.env,
            cwd: req.cwd,
            startup_timeout_ms: req.startup_timeout_ms,
            url: req.url,
            bearer_token: req.bearer_token,
            other: HashMap::new(),
        }
    }
}

/// 列出 Codex Profiles 的响应
/// 添加/更新 Codex Profile 的请求
#[derive(Debug, Deserialize)]
pub struct CodexProfileRequest {
    pub model: Option<String>,
    pub approval_policy: Option<String>,
    pub sandbox_mode: Option<String>,
    pub model_reasoning_effort: Option<String>,
}

impl From<CodexProfileRequest> for CodexProfile {
    fn from(req: CodexProfileRequest) -> Self {
        CodexProfile {
            model: req.model,
            approval_policy: req.approval_policy,
            sandbox_mode: req.sandbox_mode,
            model_reasoning_effort: req.model_reasoning_effort,
            other: HashMap::new(),
        }
    }
}

/// 获取 Codex 配置的响应
#[derive(Debug, Serialize)]
pub struct CodexConfigResponse {
    pub config: CodexConfig,
}

// ============ Auth 管理 API 模型 ============

// TokenFreshness and LoginState are re-exported from ccr_types at the top of this file

/// 账号列表项 (用于 API 响应)
#[derive(Debug, Clone, Serialize)]
pub struct CodexAuthAccountItem {
    /// 账号名称
    pub name: String,
    /// 账号描述
    pub description: Option<String>,
    /// 脱敏后的邮箱
    pub email: Option<String>,
    /// 是否为当前激活账号
    pub is_current: bool,
    /// 是否为虚拟项 (未保存的 default)
    pub is_virtual: bool,
    /// 最后使用时间 (ISO 8601)
    pub last_used: Option<String>,
    /// 最后刷新时间 (ISO 8601)
    pub last_refresh: Option<String>,
    /// Token 新鲜度
    pub freshness: TokenFreshness,
    /// 新鲜度图标
    pub freshness_icon: String,
    /// 新鲜度描述
    pub freshness_description: String,

    /// 到期时间 (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,

    /// 是否已过期
    pub is_expired: bool,
}

/// 当前 auth 信息
#[derive(Debug, Clone, Serialize)]
pub struct CodexAuthCurrentInfo {
    /// 账号 ID
    pub account_id: String,
    /// 邮箱 (脱敏)
    pub email: Option<String>,
    /// 最后刷新时间 (ISO 8601)
    pub last_refresh: Option<String>,
    /// Token 新鲜度
    pub freshness: TokenFreshness,
    /// 新鲜度图标
    pub freshness_icon: String,
    /// 新鲜度描述
    pub freshness_description: String,

    /// 到期时间 (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,

    /// 是否已过期
    pub is_expired: bool,
}

/// 列出账号的响应
#[derive(Debug, Serialize)]
pub struct CodexAuthListResponse {
    pub accounts: Vec<CodexAuthAccountItem>,
    pub login_state: LoginState,
}

/// 获取当前 auth 信息的响应
#[derive(Debug, Serialize)]
pub struct CodexAuthCurrentResponse {
    pub logged_in: bool,
    pub info: Option<CodexAuthCurrentInfo>,
    pub login_state: LoginState,
}

/// 保存当前登录的请求
#[derive(Debug, Deserialize)]
pub struct CodexAuthSaveRequest {
    /// 账号名称
    pub name: String,
    /// 账号描述 (可选)
    pub description: Option<String>,
    /// 到期时间 (ISO 8601，可选)
    pub expires_at: Option<String>,
    /// 是否强制覆盖
    #[serde(default)]
    pub force: bool,
}

/// 进程检测响应
#[derive(Debug, Serialize)]
pub struct CodexAuthProcessResponse {
    /// 是否有运行中的 Codex 进程
    pub has_running_process: bool,
    /// 运行中的进程 PID 列表
    pub pids: Vec<u32>,
    /// 警告消息
    pub warning: Option<String>,
}

// ============ Usage 统计 API 模型 ============

/// 使用量统计响应
#[derive(Debug, Clone, Serialize)]
pub struct CodexUsageStatsResponse {
    /// 总输入 tokens
    pub total_input_tokens: u64,
    /// 总输出 tokens
    pub total_output_tokens: u64,
    /// 总请求数
    pub total_requests: u64,
    /// 窗口开始时间 (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_start: Option<String>,
    /// 窗口结束时间 (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_end: Option<String>,
}

/// 滚动窗口使用量响应
#[derive(Debug, Serialize)]
pub struct CodexUsageResponse {
    /// 5小时窗口统计
    pub five_hour: CodexUsageStatsResponse,
    /// 7天窗口统计
    pub seven_day: CodexUsageStatsResponse,
    /// 全部时间统计
    pub all_time: CodexUsageStatsResponse,
    /// 按模型分组的统计
    pub by_model: std::collections::HashMap<String, CodexUsageStatsResponse>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_codex_mcp_server_stdio() {
        let server = CodexMcpServer {
            command: Some("npx".to_string()),
            args: Some(vec!["-y".to_string(), "@upstash/context7-mcp".to_string()]),
            env: Some(HashMap::from([("KEY".to_string(), "value".to_string())])),
            cwd: None,
            startup_timeout_ms: Some(20000),
            url: None,
            bearer_token: None,
            other: HashMap::new(),
        };

        assert!(server.is_stdio());
        assert!(!server.is_http());
    }

    #[test]
    fn test_codex_mcp_server_http() {
        let server = CodexMcpServer {
            command: None,
            args: None,
            env: None,
            cwd: None,
            startup_timeout_ms: None,
            url: Some("https://mcp.figma.com/mcp".to_string()),
            bearer_token: Some("token123".to_string()),
            other: HashMap::new(),
        };

        assert!(!server.is_stdio());
        assert!(server.is_http());
    }

    #[test]
    fn test_serialize_codex_config() {
        let config = CodexConfig {
            model: Some("gpt-5".to_string()),
            model_provider: Some("openai".to_string()),
            approval_policy: Some("on-request".to_string()),
            mcp_servers: Some(HashMap::from([(
                "context7".to_string(),
                CodexMcpServer {
                    command: Some("npx".to_string()),
                    args: Some(vec!["-y".to_string(), "@upstash/context7-mcp".to_string()]),
                    env: None,
                    cwd: None,
                    startup_timeout_ms: Some(20000),
                    url: None,
                    bearer_token: None,
                    other: HashMap::new(),
                },
            )])),
            ..Default::default()
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("model = \"gpt-5\""));
        assert!(toml_str.contains("[mcp_servers.context7]"));
    }
}
