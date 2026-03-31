//! 📍 codex auth current 命令实现
//!
//! 显示当前账号信息。

#![allow(clippy::unused_async)]

use crate::models::{AuthIntent, AuthState, LoginState, TokenFreshness};
use crate::services::CodexAuthService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
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
    let auth_state = service.get_auth_state();

    println!();
    ColorOutput::title("Codex 当前账号");
    println!();

    if matches!(
        auth_state.status,
        crate::models::AuthStateStatus::Unsupported
    ) {
        ColorOutput::warning("当前凭据存储模式暂不支持 CCR 多账号管理");
        ColorOutput::info(&format!("凭据存储: {}", auth_state.store.as_str()));
        ColorOutput::info(&format!("状态说明: {}", auth_state.reason));
        println!();
        ColorOutput::info("建议:");
        println!("  codex login");
        println!("  codex logout");
        println!("  # 或将 cli_auth_credentials_store 切换为 file");
        return Ok(());
    }

    match login_state {
        LoginState::NotLoggedIn => {
            ColorOutput::warning("未登录 Codex");
            ColorOutput::info(&format!(
                "认证状态: {} / {}",
                render_intent(&auth_state.intent),
                auth_state.store.as_str()
            ));
            ColorOutput::info(&format!("原因: {}", auth_state.reason));
            println!();
            ColorOutput::info("请先运行以下命令登录:");
            println!("  codex login");
        }
        LoginState::LoggedInUnsaved => {
            ColorOutput::info("登录状态: 已登录 (未保存)");

            // 显示详细信息
            if let Ok(info) = service.get_current_auth_info() {
                println!();
                display_auth_info(&service, &info, &auth_state, None); // 未保存的账号没有过期时间

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
                display_auth_info(&service, &info, &auth_state, expires_at);
            }

            println!();
            ColorOutput::info("提示:");
            println!("  • 使用 'ccr codex auth list' 查看所有账号");
            println!("  • 使用 'ccr codex auth switch <名称>' 切换账号");
        }
        LoginState::Unknown { type_name, .. } => {
            ColorOutput::warning(&format!("登录状态: 未知 ({})", type_name));

            if let Ok(info) = service.get_current_auth_info() {
                println!();
                display_auth_info(&service, &info, &auth_state, None);
            }
        }
        LoginState::ApiKeyActive => {
            ColorOutput::info("认证模式: API Key");

            if let Ok(info) = service.get_current_auth_info() {
                println!();
                display_auth_info(&service, &info, &auth_state, None);
            }

            println!();
            ColorOutput::info("提示:");
            println!("  • API Key 模式无需保存账号");
            println!("  • 使用 'ccr codex auth list' 查看已保存的 OAuth 账号");
        }
        LoginState::ProviderKeyActive { env_key } => {
            ColorOutput::info(&format!("认证模式: Provider Key ({})", env_key));

            if let Ok(info) = service.get_current_auth_info() {
                println!();
                display_auth_info(&service, &info, &auth_state, None);
            }

            println!();
            ColorOutput::info("提示:");
            println!("  • Provider Key 模式无需保存账号");
        }
    }

    Ok(())
}

/// 显示账号详细信息
fn display_auth_info(
    service: &CodexAuthService,
    info: &crate::models::CurrentAuthInfo,
    auth_state: &AuthState,
    expires_at: Option<DateTime<Utc>>,
) {
    ColorOutput::info(&format!("认证意图: {}", render_intent(&auth_state.intent)));
    ColorOutput::info(&format!("凭据存储: {}", auth_state.store.as_str()));
    ColorOutput::info(&format!("状态说明: {}", auth_state.reason));

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
    let freshness_str = match &info.freshness {
        TokenFreshness::Fresh => "🟢 新鲜 (< 1 天)".green().to_string(),
        TokenFreshness::Stale => "🟡 陈旧 (1-7 天)".yellow().to_string(),
        TokenFreshness::Old => "🔴 过期 (> 7 天)".red().to_string(),
        TokenFreshness::Unknown(_) => "⚪ 未知".white().to_string(),
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

fn render_intent(intent: &AuthIntent) -> String {
    match intent {
        AuthIntent::OpenAiAuth { method } => match method {
            crate::models::OpenAiAuthMethod::Chatgpt => "OpenAI / ChatGPT".to_string(),
            crate::models::OpenAiAuthMethod::Api => "OpenAI / API Key".to_string(),
        },
        AuthIntent::ProviderEnvKey { env_key } => format!("Provider / {env_key}"),
        AuthIntent::NoAuth => "No Auth".to_string(),
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
