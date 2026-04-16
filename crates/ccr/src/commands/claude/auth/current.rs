//! 📍 claude auth current 命令实现

#![allow(clippy::unused_async)]

use crate::models::{
    ClaudeCurrentAuthInfo, ClaudeLoginState, ClaudeRuntimeSummary, TokenFreshness,
};
use crate::services::ClaudeAuthService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ClaudeAuthCurrentJsonOutput {
    runtime_summary: ClaudeRuntimeSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_auth_info: Option<ClaudeCurrentAuthInfo>,
}

pub async fn current_command(json: bool) -> Result<()> {
    let service = ClaudeAuthService::new()?;
    let runtime_summary = service.get_runtime_summary()?;
    let current_auth_info = service.get_current_auth_info().ok();

    if json {
        let output = ClaudeAuthCurrentJsonOutput {
            runtime_summary,
            current_auth_info,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    println!();
    ColorOutput::title("Claude 当前认证状态");
    println!();

    ColorOutput::info(&format!("运行时模式: {}", runtime_summary.mode.label()));
    ColorOutput::info(&format!(
        "当前 Profile: {}",
        runtime_summary.profile_label().bright_green().bold()
    ));
    if let Some(auth_mode) = runtime_summary.current_profile_auth_mode {
        ColorOutput::info(&format!("Profile 认证模式: {}", auth_mode.as_str()));
    }
    if let Some(auth_source) = &runtime_summary.current_profile_auth_source {
        ColorOutput::info(&format!("认证来源: {}", auth_source));
    }
    ColorOutput::info(&format!(
        "当前官方登录: {}",
        runtime_summary.official_login_label()
    ));
    ColorOutput::info(&format!("当前生效认证: {}", runtime_summary.auth_label()));

    match &runtime_summary.login_state {
        ClaudeLoginState::NotLoggedIn => {
            println!();
            ColorOutput::warning("未检测到可用的 Claude 官方订阅登录");
            ColorOutput::info("请先运行 `claude login`，或切换到 API key profile");
        }
        ClaudeLoginState::LoggedInUnsaved => {
            println!();
            ColorOutput::success("已检测到官方订阅登录（未保存）");
            ColorOutput::info("使用 `ccr claude auth save <名称>` 可保存当前账号快照");
        }
        ClaudeLoginState::LoggedInSaved { account_name } => {
            println!();
            ColorOutput::success(&format!(
                "已检测到官方订阅登录（已保存为 '{}'）",
                account_name.bright_green().bold()
            ));
        }
        ClaudeLoginState::ApiKeyActive => {
            println!();
            if let Some(account_name) = &runtime_summary.current_login_name {
                ColorOutput::info(&format!(
                    "当前由 API key profile 控制；官方账号 '{}' 已登录但未生效",
                    account_name.bright_green().bold()
                ));
            } else {
                ColorOutput::info("当前由 API key profile 控制，不使用官方订阅凭据");
            }
        }
    }

    if let Some(info) = current_auth_info {
        println!();
        display_current_auth_info(&service, &info);
    }

    Ok(())
}

fn display_current_auth_info(service: &ClaudeAuthService, info: &ClaudeCurrentAuthInfo) {
    if let Some(email) = &info.email {
        ColorOutput::info(&format!("邮箱: {}", service.mask_email(email)));
    }
    if let Some(account_uuid) = &info.account_uuid {
        ColorOutput::info(&format!("账号 UUID: {}", mask_uuid(account_uuid)));
    }
    if let Some(billing_type) = &info.billing_type {
        ColorOutput::info(&format!("计费类型: {}", billing_type));
    }
    if let Some(subscription_type) = &info.subscription_type {
        ColorOutput::info(&format!("订阅类型: {}", subscription_type));
    }
    if let Some(rate_limit_tier) = &info.rate_limit_tier {
        ColorOutput::info(&format!("速率档位: {}", rate_limit_tier));
    }
    if let Some(expires_at) = info.expires_at {
        let local = expires_at.with_timezone(&chrono::Local);
        let expired = ClaudeAuthService::is_expired(Some(expires_at));
        if expired {
            ColorOutput::warning(&format!(
                "Access Token 已过期: {}",
                local.format("%Y-%m-%d %H:%M:%S")
            ));
        } else {
            ColorOutput::info(&format!(
                "Access Token 到期时间: {}",
                local.format("%Y-%m-%d %H:%M:%S")
            ));
        }
    }

    let freshness = match &info.freshness {
        TokenFreshness::Fresh => "🟢 Fresh".green(),
        TokenFreshness::Stale => "🟡 Stale".yellow(),
        TokenFreshness::Old => "🔴 Old".red(),
        TokenFreshness::Unknown(_) => "⚪ Unknown".white(),
    };
    ColorOutput::info(&format!("Token 状态: {}", freshness));
}

fn mask_uuid(value: &str) -> String {
    if value.len() <= 8 {
        return value.to_string();
    }
    format!("{}...{}", &value[..4], &value[value.len() - 4..])
}
