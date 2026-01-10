// Codex CLI 配置数据模型
// 用于读写 ~/.codex/config.toml

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    // ============ 安全策略 ============
    /// 批准策略（"auto", "on-request", "read-only", "full-access"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_policy: Option<String>,

    /// 沙盒模式（如 "workspace-write"）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_mode: Option<String>,

    // ============ Shell 环境策略 ============
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_environment_policy: Option<ShellEnvironmentPolicy>,

    // ============ MCP 服务器配置 ============
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, CodexMcpServer>>,

    // ============ Profiles 配置 ============
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<HashMap<String, CodexProfile>>,

    // ============ 实验性特性 ============
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental_use_rmcp_client: Option<bool>,

    // ============ 其他未知字段 ============
    /// 保留未知字段，避免覆盖用户自定义配置
    #[serde(flatten)]
    pub other: HashMap<String, toml::Value>,
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

/// Token 新鲜度状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenFreshness {
    /// 新鲜 (< 1 天)
    Fresh,
    /// 陈旧 (1-7 天)
    Stale,
    /// 过期 (> 7 天)
    Old,
    /// 未知 (无法解析时间)
    Unknown,
}

impl TokenFreshness {
    /// 获取显示图标
    pub fn icon(&self) -> &'static str {
        match self {
            TokenFreshness::Fresh => "🟢",
            TokenFreshness::Stale => "🟡",
            TokenFreshness::Old => "🔴",
            TokenFreshness::Unknown => "⚪",
        }
    }

    /// 获取描述文本
    pub fn description(&self) -> &'static str {
        match self {
            TokenFreshness::Fresh => "Token 状态良好",
            TokenFreshness::Stale => "Token 可能需要刷新",
            TokenFreshness::Old => "Token 可能已过期，建议重新登录",
            TokenFreshness::Unknown => "无法确定 Token 状态",
        }
    }
}

/// 登录状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", content = "account_name")]
pub enum LoginState {
    /// 未登录 (auth.json 不存在)
    NotLoggedIn,
    /// 已登录但未保存
    LoggedInUnsaved,
    /// 已登录且已保存 (账号名)
    LoggedInSaved(String),
}

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
