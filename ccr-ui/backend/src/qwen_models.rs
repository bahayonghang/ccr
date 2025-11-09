// Qwen CLI 配置数据模型
// 配置文件位置: .qwen/settings.json

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Qwen CLI 完整配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QwenConfig {
    // MCP 服务器配置
    pub mcp_servers: Option<HashMap<String, QwenMcpServer>>,

    // 其他未知字段
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// Qwen MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QwenMcpServer {
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

impl QwenMcpServer {
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
pub struct QwenMcpServerRequest {
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
mod tests {
    use super::*;

    #[test]
    fn test_qwen_config_serialization() {
        let config = QwenConfig {
            mcp_servers: Some(HashMap::from([(
                "github".to_string(),
                QwenMcpServer {
                    command: Some("docker".to_string()),
                    args: Some(vec![
                        "run".to_string(),
                        "-i".to_string(),
                        "--rm".to_string(),
                        "ghcr.io/github/github-mcp-server".to_string(),
                    ]),
                    env: Some(HashMap::from([(
                        "GITHUB_TOKEN".to_string(),
                        "token".to_string(),
                    )])),
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
        assert!(json.contains("mcpServers"));
        assert!(json.contains("github"));
    }

    #[test]
    fn test_transport_type_detection() {
        let stdio_server = QwenMcpServer {
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
        assert!(stdio_server.is_stdio());
        assert!(!stdio_server.is_http());

        let sse_server = QwenMcpServer {
            command: None,
            args: None,
            env: None,
            url: Some("https://example.com/sse".to_string()),
            http_url: None,
            headers: None,
            timeout: None,
            other: HashMap::new(),
        };
        assert_eq!(sse_server.transport_type(), "sse");
        assert!(!sse_server.is_stdio());
        assert!(sse_server.is_http());

        let http_server = QwenMcpServer {
            command: None,
            args: None,
            env: None,
            url: None,
            http_url: Some("https://example.com/http".to_string()),
            headers: None,
            timeout: None,
            other: HashMap::new(),
        };
        assert_eq!(http_server.transport_type(), "http");
        assert!(!http_server.is_stdio());
        assert!(http_server.is_http());
    }
}
