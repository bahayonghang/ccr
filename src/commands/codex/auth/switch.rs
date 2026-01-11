//! 🔄 codex auth switch 命令实现
//!
//! 切换到指定账号。

#![allow(clippy::unused_async)]

use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use crate::services::CodexAuthService;
use colored::Colorize;

/// 🔄 切换到指定账号
///
/// 将 ~/.codex/auth.json 切换为指定账号的登录状态。
///
/// # 参数
///
/// * `name` - 要切换到的账号名称
///
/// # 返回
///
/// * `Ok(())` - 切换成功
/// * `Err(CcrError)` - 切换失败
pub async fn switch_command(name: &str) -> Result<()> {
    let service = CodexAuthService::new()?;

    // 检测 Codex 进程
    let running_processes = service.detect_codex_process();
    if !running_processes.is_empty() {
        println!();
        ColorOutput::warning("检测到 Codex 进程正在运行");
        ColorOutput::info(&format!("进程 ID: {:?}", running_processes));
        println!();
        ColorOutput::warning("切换账号可能导致正在运行的 Codex 会话出现问题");
        ColorOutput::info("建议先关闭所有 Codex 进程后再切换账号");
        println!();
    }

    // 执行切换
    match service.switch_account(name) {
        Ok(()) => {
            println!();
            ColorOutput::success(&format!("已切换到账号: {}", name.bright_green().bold()));

            // 显示切换后的账号信息
            if let Ok(info) = service.get_current_auth_info() {
                if let Some(email) = &info.email {
                    ColorOutput::info(&format!("邮箱: {}", service.mask_email(email)));
                }

                // 显示 Token 新鲜度
                let freshness_str = match info.freshness {
                    crate::models::TokenFreshness::Fresh => "🟢 新鲜 (< 1 天)".green(),
                    crate::models::TokenFreshness::Stale => "🟡 陈旧 (1-7 天)".yellow(),
                    crate::models::TokenFreshness::Old => "🔴 过期 (> 7 天)".red(),
                    crate::models::TokenFreshness::Unknown => "⚪ 未知".white(),
                };
                ColorOutput::info(&format!("Token 状态: {}", freshness_str));
            }

            println!();
            ColorOutput::info("提示:");
            println!("  • 使用 'ccr codex auth list' 查看所有账号");
            println!("  • 使用 'ccr codex auth current' 查看当前账号详情");
        }
        Err(e) => {
            ColorOutput::error(&format!("切换失败: {}", e));

            // 如果是账号不存在，显示可用账号
            let err_msg = e.to_string();
            if err_msg.contains("不存在") {
                println!();
                ColorOutput::info("使用以下命令查看可用账号:");
                println!("  ccr codex auth list");
            }
        }
    }

    Ok(())
}
