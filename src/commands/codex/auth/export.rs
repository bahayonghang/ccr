//! 📤 codex auth export 命令实现
//!
//! 导出所有账号到 JSON 文件。

#![allow(clippy::unused_async)]

use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use crate::services::CodexAuthService;
use colored::Colorize;
use std::fs;

/// 📤 导出所有账号到 JSON 文件
///
/// 将所有已保存的账号导出为 JSON 格式。
///
/// # 参数
///
/// * `output` - 输出文件路径 (None 则输出到标准输出)
/// * `no_secrets` - 是否不包含敏感信息 (Token 等)
///
/// # 返回
///
/// * `Ok(())` - 导出成功
/// * `Err(CcrError)` - 导出失败
pub async fn export_command(output: Option<String>, no_secrets: bool) -> Result<()> {
    let service = CodexAuthService::new()?;

    // 检查是否有账号
    let accounts = service.list_accounts()?;
    let saved_count = accounts.iter().filter(|a| !a.is_virtual).count();

    if saved_count == 0 {
        ColorOutput::warning("没有已保存的账号可导出");
        println!();
        ColorOutput::info("提示:");
        println!("  • 使用 'ccr codex auth save <名称>' 保存当前登录");
        return Ok(());
    }

    // 执行导出
    let include_secrets = !no_secrets;
    match service.export_accounts(include_secrets) {
        Ok(json) => {
            if let Some(path) = output {
                // 写入文件
                fs::write(&path, &json).map_err(|e| {
                    crate::core::error::CcrError::ConfigError(format!("写入文件失败: {}", e))
                })?;

                println!();
                ColorOutput::success(&format!("已导出到: {}", path.bright_green()));
                ColorOutput::info(&format!("账号数量: {}", saved_count));

                if include_secrets {
                    println!();
                    ColorOutput::warning("⚠️  导出文件包含敏感信息 (Token)，请妥善保管！");
                } else {
                    ColorOutput::info("导出不包含敏感信息 (仅元数据)");
                }
            } else {
                // 输出到标准输出
                println!("{}", json);
            }
        }
        Err(e) => {
            ColorOutput::error(&format!("导出失败: {}", e));
        }
    }

    Ok(())
}
