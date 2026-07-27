//! 格式转换命令 — 跨平台配置格式转换。

use ccr_db::models::converter::{CliType, ConverterRequest};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export, export_to = "../../src/types/generated/converter/")]
pub enum ConverterCliType {
    ClaudeCode,
    Codex,
    Gemini,
    Qwen,
}

impl From<ConverterCliType> for CliType {
    fn from(value: ConverterCliType) -> Self {
        match value {
            ConverterCliType::ClaudeCode => Self::ClaudeCode,
            ConverterCliType::Codex => Self::Codex,
            ConverterCliType::Gemini => Self::Gemini,
            ConverterCliType::Qwen => Self::Qwen,
        }
    }
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/converter/")]
pub struct ConverterRequestDto {
    pub source_format: ConverterCliType,
    pub target_format: ConverterCliType,
    pub config_data: String,
    #[ts(optional)]
    pub convert_mcp: Option<bool>,
    #[ts(optional)]
    pub convert_commands: Option<bool>,
    #[ts(optional)]
    pub convert_agents: Option<bool>,
}

impl From<ConverterRequestDto> for ConverterRequest {
    fn from(value: ConverterRequestDto) -> Self {
        Self {
            source_format: value.source_format.into(),
            target_format: value.target_format.into(),
            config_data: value.config_data,
            convert_mcp: value.convert_mcp.unwrap_or(true),
            convert_commands: value.convert_commands.unwrap_or(true),
            convert_agents: value.convert_agents.unwrap_or(true),
        }
    }
}

/// 转换结果（前端友好格式）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/converter/")]
pub struct ConvertResult {
    pub success: bool,
    pub content: String,
    pub warnings: Vec<String>,
    pub format: String,
    pub stats: ConvertStats,
}

/// 转换统计
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/converter/")]
pub struct ConvertStats {
    pub mcp_servers: usize,
    pub slash_commands: usize,
    pub agents: usize,
    pub profiles: usize,
    pub base_config: bool,
}

#[tauri::command]
pub async fn convert_config(request: ConverterRequestDto) -> Result<ConvertResult, String> {
    let request = ConverterRequest::from(request);
    let result = tokio::task::spawn_blocking(move || {
        ccr_db::services::converter_service::ConfigConverter::convert(request)
            .map_err(|e| format!("Config conversion failed: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    Ok(ConvertResult {
        success: result.success,
        content: result.converted_data,
        warnings: result.warnings,
        format: result.format,
        stats: ConvertStats {
            mcp_servers: result.stats.mcp_servers,
            slash_commands: result.stats.slash_commands,
            agents: result.stats.agents,
            profiles: result.stats.profiles,
            base_config: result.stats.base_config,
        },
    })
}
