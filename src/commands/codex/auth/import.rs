//! 📥 codex auth import 命令实现
//!
//! 从 JSON 文件导入账号。

#![allow(clippy::unused_async)]

use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use crate::models::ImportMode;
use crate::services::CodexAuthService;
use colored::Colorize;
use std::fs;

/// 📥 从 JSON 文件导入账号
///
/// 从 JSON 文件导入账号数据。
///
/// # 参数
///
/// * `input` - 输入文件路径
/// * `replace` - 是否使用替换模式 (覆盖同名账号)
/// * `force` - 是否强制覆盖 (在合并模式下覆盖已存在的账号)
///
/// # 返回
///
/// * `Ok(())` - 导入成功
/// * `Err(CcrError)` - 导入失败
pub async fn import_command(input: &str, replace: bool, force: bool) -> Result<()> {
    let service = CodexAuthService::new()?;

    // 读取文件
    let content = fs::read_to_string(input)
        .map_err(|e| crate::core::error::CcrError::ConfigError(format!("读取文件失败: {}", e)))?;

    // 确定导入模式
    let mode = if replace {
        ImportMode::Replace
    } else {
        ImportMode::Merge
    };

    // 执行导入
    match service.import_accounts(&content, mode, force) {
        Ok(result) => {
            println!();
            ColorOutput::success("导入完成！");
            println!();

            // 显示统计信息
            if result.added > 0 {
                ColorOutput::info(&format!(
                    "新增账号: {}",
                    result.added.to_string().bright_green()
                ));
            }
            if result.updated > 0 {
                ColorOutput::info(&format!(
                    "更新账号: {}",
                    result.updated.to_string().bright_yellow()
                ));
            }
            if result.skipped > 0 {
                ColorOutput::info(&format!(
                    "跳过账号: {}",
                    result.skipped.to_string().bright_cyan()
                ));
            }

            let total = result.added + result.updated + result.skipped;
            if total == 0 {
                ColorOutput::warning("没有账号被导入");
            }

            // 显示模式说明
            println!();
            match mode {
                ImportMode::Merge => {
                    if force {
                        ColorOutput::info("模式: 合并 (强制覆盖已存在的账号)");
                    } else {
                        ColorOutput::info("模式: 合并 (跳过已存在的账号)");
                    }
                }
                ImportMode::Replace => {
                    ColorOutput::info("模式: 替换 (覆盖同名账号)");
                }
            }

            println!();
            ColorOutput::info("提示:");
            println!("  • 使用 'ccr codex auth list' 查看所有账号");
        }
        Err(e) => {
            ColorOutput::error(&format!("导入失败: {}", e));

            // 提供帮助信息
            let err_msg = e.to_string();
            if err_msg.contains("解析") {
                println!();
                ColorOutput::info("提示: 请确保文件是有效的 JSON 格式");
            }
        }
    }

    Ok(())
}
