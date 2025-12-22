// Gemini CLI 配置数据模型
// 配置文件位置: ~/.gemini/settings.json

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Gemini CLI 完整配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeminiConfig {
    // MCP 服务器配置
    pub mcp_servers: Option<HashMap<String, GeminiMcpServer>>,

    // 通用设置
    pub general: Option<GeminiGeneralSettings>,

    // 输出设置
    pub output: Option<GeminiOutputSettings>,

    // UI 设置
    pub ui: Option<GeminiUiSettings>,

    // 模型设置
    pub model: Option<GeminiModelSettings>,

    // 上下文设置
    pub context: Option<GeminiContextSettings>,

    // 工具设置
    pub tools: Option<GeminiToolsSettings>,

    // MCP 全局设置
    pub mcp: Option<GeminiMcpSettings>,

    // 安全设置
    pub security: Option<GeminiSecuritySettings>,

    // 高级设置
    pub advanced: Option<GeminiAdvancedSettings>,

    // 实验性功能
    pub experimental: Option<GeminiExperimentalSettings>,

    // 其他未知字段
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// Gemini MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiMcpServer {
    // STDIO transport 字段
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub cwd: Option<String>,

    // 通用字段
    pub timeout: Option<u64>,
    pub trust: Option<bool>,
    pub include_tools: Option<Vec<String>>,

    // 其他未知字段
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

impl GeminiMcpServer {
    /// 判断是否为 STDIO 服务器
    #[allow(dead_code)]
    pub fn is_stdio(&self) -> bool {
        self.command.is_some()
    }
}

/// 通用设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeminiGeneralSettings {
    pub auto_update_check: Option<bool>,
    pub default_model: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// 输出设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeminiOutputSettings {
    pub format: Option<String>, // "text" | "json" | "stream-json"
    pub color: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// UI 设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeminiUiSettings {
    pub theme: Option<String>,
    pub show_token_count: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// 模型设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeminiModelSettings {
    pub name: Option<String>,
    pub max_tokens: Option<u64>,
    pub temperature: Option<f64>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// 上下文设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeminiContextSettings {
    pub max_context_size: Option<u64>,
    pub file_patterns: Option<Vec<String>>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// 工具设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeminiToolsSettings {
    pub sandbox_enabled: Option<bool>,
    pub shell_enabled: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// MCP 全局设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeminiMcpSettings {
    pub auto_discover: Option<bool>,
    pub timeout: Option<u64>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// 安全设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeminiSecuritySettings {
    pub require_confirmation: Option<bool>,
    pub trusted_folders: Option<Vec<String>>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// 高级设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeminiAdvancedSettings {
    pub dns_servers: Option<Vec<String>>,
    pub proxy: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// 实验性功能设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GeminiExperimentalSettings {
    pub feature_flags: Option<HashMap<String, bool>>,
    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

/// MCP 服务器请求（用于 API）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiMcpServerRequest {
    pub name: String,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub cwd: Option<String>,
    pub timeout: Option<u64>,
    pub trust: Option<bool>,
    pub include_tools: Option<Vec<String>>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_config_serialization() {
        let config = GeminiConfig {
            mcp_servers: Some(HashMap::from([(
                "github".to_string(),
                GeminiMcpServer {
                    command: Some("npx".to_string()),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-github".to_string(),
                    ]),
                    env: Some(HashMap::from([(
                        "GITHUB_TOKEN".to_string(),
                        "token".to_string(),
                    )])),
                    cwd: None,
                    timeout: Some(30000),
                    trust: Some(false),
                    include_tools: None,
                    other: HashMap::new(),
                },
            )])),
            ..Default::default()
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("mcpServers"));
        assert!(json.contains("github"));
    }
}
