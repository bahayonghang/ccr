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
#[derive(Debug, Serialize)]
pub struct CodexProfileListResponse {
    pub profiles: Vec<CodexProfileWithName>,
}

/// 带名称的 Codex Profile
#[derive(Debug, Serialize, Deserialize)]
pub struct CodexProfileWithName {
    pub name: String,
    #[serde(flatten)]
    pub profile: CodexProfile,
}

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

#[cfg(test)]
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
