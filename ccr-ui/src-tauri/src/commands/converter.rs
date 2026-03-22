//! 格式转换命令 — 跨平台配置格式转换。

use ccr_db::models::converter::ConverterRequest;
use serde::{Deserialize, Serialize};

/// 转换结果（前端友好格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertResult {
    pub success: bool,
    pub content: String,
    pub warnings: Vec<String>,
    pub format: String,
    pub stats: ConvertStats,
}

/// 转换统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertStats {
    pub mcp_servers: usize,
    pub slash_commands: usize,
    pub agents: usize,
    pub profiles: usize,
    pub base_config: bool,
}

#[tauri::command]
pub async fn convert_config(request: ConverterRequest) -> Result<ConvertResult, String> {
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
