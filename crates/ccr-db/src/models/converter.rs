// 配置转换器数据模型和请求/响应类型

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// CLI 类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CliType {
    ClaudeCode,
    Codex,
    Gemini,
    Qwen,
    Iflow,
}

/// 转换请求
#[derive(Debug, Deserialize)]
pub struct ConverterRequest {
    pub source_format: CliType,
    pub target_format: CliType,
    pub config_data: String, // JSON 或 TOML 字符串
    #[serde(default = "default_true")]
    #[allow(dead_code)]
    pub convert_mcp: bool,
    #[serde(default = "default_true")]
    #[allow(dead_code)]
    pub convert_commands: bool,
    #[serde(default = "default_true")]
    #[allow(dead_code)]
    pub convert_agents: bool,
}

fn default_true() -> bool {
    true
}

/// 转换响应
#[derive(Debug, Serialize)]
pub struct ConverterResponse {
    pub success: bool,
    pub converted_data: String, // JSON 或 TOML 字符串
    pub warnings: Vec<String>,
    pub format: String, // "json" 或 "toml"
    pub stats: ConversionStats,
}

/// 转换统计
#[derive(Debug, Serialize, Default)]
pub struct ConversionStats {
    pub mcp_servers: usize,
    pub slash_commands: usize,
    pub agents: usize,
    pub profiles: usize,
    pub base_config: bool,
}

/// 通用 MCP 服务器（中间格式）
#[derive(Debug, Clone)]
pub struct GenericMcpServer {
    pub name: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub url: Option<String>,
    pub bearer_token: Option<String>,
    pub startup_timeout_ms: Option<u64>,
    pub cwd: Option<String>,
}

/// 通用命令（中间格式）
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GenericCommand {
    pub name: String,
    pub description: String,
    pub content: String,
}

/// 通用 Agent（中间格式）
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GenericAgent {
    pub name: String,
    pub model: String,
    pub tools: Vec<String>,
    pub system_prompt: String,
}

/// 通用 Profile（中间格式）
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GenericProfile {
    pub name: String,
    pub model: Option<String>,
    pub approval_policy: Option<String>,
    pub sandbox_mode: Option<String>,
}

/// 中间配置格式（统一表示）
#[derive(Debug, Default)]
pub struct IntermediateConfig {
    pub mcp_servers: Vec<GenericMcpServer>,
    pub commands: Vec<GenericCommand>,
    pub agents: Vec<GenericAgent>,
    pub profiles: Vec<GenericProfile>,
}
