//! 📍 codex auth current 命令实现
//!
//! 显示当前账号信息。

#![allow(clippy::unused_async)]

use crate::core::error::Result;
use crate::core::logging::ColorOutput;
use crate::models::{LoginState, TokenFreshness};
use crate::services::CodexAuthService;
use chrono::{DateTime, Local, Utc};
use colored::Colorize;

/// 📍 显示当前账号信息
///
/// 显示当前 ~/.codex/auth.json 的账号信息。
///
/// # 返回
///
/// * `Ok(())` - 成功执行
/// * `Err(CcrError)` - 执行失败
pub async fn current_command() -> Result<()> {
    let service = CodexAuthService::new()?;

    // 检查登录状态
    let login_state = service.get_login_state()?;

    println!();
    ColorOutput::title("Codex 当前账号");
    println!();

    match login_state {
        LoginState::NotLoggedIn => {
            ColorOutput::warning("未登录 Codex");
            println!();
            ColorOutput::info("请先运行以下命令登录:");
            println!("  codex login");
        }
        LoginState::LoggedInUnsaved => {
            ColorOutput::info("登录状态: 已登录 (未保存)");

            // 显示详细信息
            if let Ok(info) = service.get_current_auth_info() {
                println!();
                display_auth_info(&service, &info, None); // 未保存的账号没有过期时间

                println!();
                ColorOutput::warning("当前登录尚未保存");
                ColorOutput::info("使用以下命令保存当前登录:");
                println!("  ccr codex auth save <名称>");
            }
        }
        LoginState::LoggedInSaved(name) => {
            ColorOutput::success(&format!(
                "登录状态: 已登录 (已保存为 '{}')",
                name.bright_green().bold()
            ));

            // 显示详细信息
            if let Ok(info) = service.get_current_auth_info() {
                let expires_at = service
                    .load_registry()
                    .ok()
                    .and_then(|reg| reg.accounts.get(&name).and_then(|a| a.expires_at));

                println!();
                display_auth_info(&service, &info, expires_at);
            }

            println!();
            ColorOutput::info("提示:");
            println!("  • 使用 'ccr codex auth list' 查看所有账号");
            println!("  • 使用 'ccr codex auth switch <名称>' 切换账号");
        }
    }

    Ok(())
}

/// 显示账号详细信息
fn display_auth_info(
    service: &CodexAuthService,
    info: &crate::models::CurrentAuthInfo,
    expires_at: Option<DateTime<Utc>>,
) {
    // 邮箱
    if let Some(email) = &info.email {
        ColorOutput::info(&format!("邮箱: {}", service.mask_email(email)));
    } else {
        ColorOutput::info("邮箱: (未知)");
    }

    // Account ID
    ColorOutput::info(&format!(
        "Account ID: {}",
        mask_account_id(&info.account_id)
    ));

    // Token 新鲜度
    let freshness_str = match info.freshness {
        TokenFreshness::Fresh => "🟢 新鲜 (< 1 天)".green().to_string(),
        TokenFreshness::Stale => "🟡 陈旧 (1-7 天)".yellow().to_string(),
        TokenFreshness::Old => "🔴 过期 (> 7 天)".red().to_string(),
        TokenFreshness::Unknown => "⚪ 未知".white().to_string(),
    };
    ColorOutput::info(&format!("Token 状态: {}", freshness_str));

    // 最后刷新时间
    if let Some(last_refresh) = &info.last_refresh {
        let local_time = last_refresh.with_timezone(&chrono::Local);
        ColorOutput::info(&format!(
            "最后刷新: {}",
            local_time.format("%Y-%m-%d %H:%M:%S")
        ));
    }

    // 到期时间
    if let Some(exp_at) = expires_at {
        let expired = CodexAuthService::is_expired(Some(exp_at));
        let local_ts = exp_at.with_timezone(&Local).format("%Y-%m-%d %H:%M");
        let label = if expired {
            format!("🔒 已过期: {}", local_ts)
        } else {
            format!("到期: {}", local_ts)
        };
        if expired {
            ColorOutput::error(&label);
        } else {
            ColorOutput::info(&label);
        }
    }
}

/// 脱敏 Account ID
fn mask_account_id(account_id: &str) -> String {
    if account_id.len() <= 8 {
        return account_id.to_string();
    }

    let prefix = &account_id[..4];
    let suffix = &account_id[account_id.len() - 4..];
    format!("{}...{}", prefix, suffix)
}
