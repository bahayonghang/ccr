//! 💾 codex auth save 命令实现
//!
//! 保存当前登录到指定名称。

#![allow(clippy::unused_async)]

use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use crate::services::CodexAuthService;
use colored::Colorize;

/// 💾 保存当前登录到指定名称
///
/// 将当前 ~/.codex/auth.json 保存为命名账号。
///
/// # 参数
///
/// * `name` - 账号名称
/// * `description` - 账号描述 (可选)
/// * `force` - 是否强制覆盖已存在的账号
///
/// # 返回
///
/// * `Ok(())` - 保存成功
/// * `Err(CcrError)` - 保存失败
pub async fn save_command(name: &str, description: Option<String>, force: bool) -> Result<()> {
    let service = CodexAuthService::new()?;

    // 检查是否已登录
    if !service.is_logged_in() {
        ColorOutput::error("未登录 Codex");
        println!();
        ColorOutput::info("请先运行以下命令登录:");
        println!("  codex login");
        return Ok(());
    }

    // 执行保存
    match service.save_current(name, description.clone(), force) {
        Ok(()) => {
            println!();
            ColorOutput::success(&format!("已保存账号: {}", name.bright_green().bold()));

            if let Some(desc) = description {
                ColorOutput::info(&format!("描述: {}", desc));
            }

            // 显示当前账号信息
            if let Ok(info) = service.get_current_auth_info()
                && let Some(email) = &info.email
            {
                ColorOutput::info(&format!("邮箱: {}", service.mask_email(email)));
            }

            println!();
            ColorOutput::info("提示:");
            println!("  • 使用 'ccr codex auth list' 查看所有账号");
            println!("  • 使用 'ccr codex auth switch <名称>' 切换账号");
        }
        Err(e) => {
            ColorOutput::error(&format!("保存失败: {}", e));

            // 如果是因为账号已存在，提示使用 --force
            let err_msg = e.to_string();
            if err_msg.contains("已存在") {
                println!();
                ColorOutput::info("提示: 使用 --force 参数覆盖已存在的账号");
                println!("  ccr codex auth save {} --force", name);
            }
        }
    }

    Ok(())
}
