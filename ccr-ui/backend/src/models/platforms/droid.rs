// Droid CLI 配置数据模型
// 配置文件位置: .factory/settings.json

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Droid CLI 完整配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DroidConfig {
    // 自定义模型列表
    pub custom_models: Option<Vec<DroidCustomModel>>,

    // MCP 服务器配置
    pub mcp_servers: Option<HashMap<String, DroidMcpServer>>,

    // 其他未知字段
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// Droid 自定义模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DroidCustomModel {
    /// 模型标识符
    pub model: String,

    /// 显示名称
    pub display_name: Option<String>,

    /// API 端点 URL
    pub base_url: String,

    /// API 密钥
    pub api_key: String,

    /// 提供商类型: anthropic / openai / generic-chat-completion-api
    pub provider: String,

    /// 最大输出 token 数
    pub max_output_tokens: Option<u32>,
}

/// Droid MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DroidMcpServer {
    // STDIO transport 字段
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,

    // HTTP transport 字段 (SSE)
    pub url: Option<String>,

    // HTTP transport 字段 (streamable HTTP)
    pub http_url: Option<String>,

    // HTTP headers
    pub headers: Option<HashMap<String, String>>,

    // 通用字段
    pub timeout: Option<u64>,

    // 其他未知字段
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

impl DroidMcpServer {
    /// 判断是否为 STDIO 服务器
    #[allow(dead_code)]
    pub fn is_stdio(&self) -> bool {
        self.command.is_some()
    }

    /// 判断是否为 HTTP 服务器
    #[allow(dead_code)]
    pub fn is_http(&self) -> bool {
        self.url.is_some() || self.http_url.is_some()
    }

    /// 获取传输类型
    pub fn transport_type(&self) -> &str {
        if self.http_url.is_some() {
            "http"
        } else if self.url.is_some() {
            "sse"
        } else if self.command.is_some() {
            "stdio"
        } else {
            "unknown"
        }
    }
}

/// MCP 服务器请求（用于 API）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DroidMcpServerRequest {
    pub name: String,
    // STDIO
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    // HTTP
    pub url: Option<String>,
    pub http_url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    // 通用
    pub timeout: Option<u64>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_droid_config_serialization() {
        let config = DroidConfig {
            custom_models: Some(vec![DroidCustomModel {
                model: "claude-sonnet-4-5".to_string(),
                display_name: Some("My Claude".to_string()),
                base_url: "https://api.anthropic.com/v1".to_string(),
                api_key: "sk-test".to_string(),
                provider: "anthropic".to_string(),
                max_output_tokens: Some(8192),
            }]),
            mcp_servers: Some(HashMap::from([(
                "github".to_string(),
                DroidMcpServer {
                    command: Some("docker".to_string()),
                    args: Some(vec!["run".to_string(), "-i".to_string()]),
                    env: None,
                    url: None,
                    http_url: None,
                    headers: None,
                    timeout: Some(30000),
                    other: HashMap::new(),
                },
            )])),
            other: HashMap::new(),
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("customModels"));
        assert!(json.contains("mcpServers"));
    }

    #[test]
    fn test_transport_type_detection() {
        let stdio_server = DroidMcpServer {
            command: Some("npx".to_string()),
            args: None,
            env: None,
            url: None,
            http_url: None,
            headers: None,
            timeout: None,
            other: HashMap::new(),
        };
        assert_eq!(stdio_server.transport_type(), "stdio");
    }
}
