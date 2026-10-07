//! 💾 claude auth save 命令实现

#![allow(clippy::unused_async)]

use crate::commands::common::print_next_steps;
use crate::services::ClaudeAuthService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;

pub async fn save_command(name: &str, description: Option<String>, force: bool) -> Result<()> {
    let service = ClaudeAuthService::new()?;

    match service.save_current(name, description.clone(), force) {
        Ok(account) => {
            ColorOutput::success(&format!("已保存 Claude 官方账号 {name}"));
            if let Some(email) = &account.email
                && !email.is_empty()
            {
                ColorOutput::key_value("邮箱", email, 2);
            }
            if let Some(subscription_type) = &account.subscription_type
                && !subscription_type.is_empty()
            {
                ColorOutput::key_value("订阅类型", subscription_type, 2);
            }
            if let Some(expires_at) = account.expires_at {
                ColorOutput::key_value("Access Token 到期", &expires_at.to_rfc3339(), 2);
            }
            print_next_steps(&[("查看账号", "ccr claude auth list")]);
        }
        Err(e) => {
            ColorOutput::error(&format!("保存失败: {}", e));
            if e.to_string().contains("已存在") {
                ColorOutput::info("覆盖会替换该账号已保存的凭据快照");
                print_next_steps(&[(
                    "覆盖已保存账号",
                    &format!("ccr claude auth save --force -- {name}"),
                )]);
            } else if e.to_string().contains("claude login") {
                print_next_steps(&[("登录 Claude", "claude login")]);
            }
        }
    }

    Ok(())
}
