//! 📍 claude auth current 命令实现

#![allow(clippy::unused_async)]

use crate::commands::common::print_next_steps;
use crate::models::{ClaudeCurrentAuthInfo, ClaudeLoginState, ClaudeRuntimeSummary};
use crate::services::ClaudeAuthService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
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

    ColorOutput::title("Claude 当前认证状态");
    println!();

    ColorOutput::key_value("运行时模式", runtime_summary.mode.label(), 2);
    ColorOutput::key_value("当前 Profile", &runtime_summary.profile_label(), 2);
    if let Some(auth_mode) = runtime_summary.current_profile_auth_mode {
        ColorOutput::key_value("Profile 认证模式", auth_mode.as_str(), 2);
    }
    if let Some(auth_source) = &runtime_summary.current_profile_auth_source {
        ColorOutput::key_value("认证来源", auth_source, 2);
    }
    ColorOutput::key_value("当前官方登录", &runtime_summary.official_login_label(), 2);
    ColorOutput::key_value("当前生效认证", &runtime_summary.auth_label(), 2);

    match &runtime_summary.login_state {
        ClaudeLoginState::NotLoggedIn => {
            println!();
            ColorOutput::warning("未检测到可用的 Claude 官方订阅登录");
            print_next_steps(&[
                ("登录 Claude", "claude login"),
                ("查看 Profile 切换帮助", "ccr claude profile switch --help"),
            ]);
        }
        ClaudeLoginState::LoggedInUnsaved => {
            println!();
            ColorOutput::warning("当前官方订阅登录尚未保存");
        }
        ClaudeLoginState::LoggedInSaved { account_name } => {
            println!();
            ColorOutput::success(&format!(
                "已检测到官方订阅登录（已保存为 '{account_name}'）"
            ));
        }
        ClaudeLoginState::ApiKeyActive => {
            println!();
            if let Some(account_name) = &runtime_summary.current_login_name {
                ColorOutput::info(&format!(
                    "当前由 API key profile 控制；官方账号 '{}' 已登录但未生效",
                    account_name
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

    if matches!(
        runtime_summary.login_state,
        ClaudeLoginState::LoggedInUnsaved
    ) {
        print_next_steps(&[("查看保存帮助", "ccr claude auth save --help")]);
    }
    Ok(())
}

fn display_current_auth_info(service: &ClaudeAuthService, info: &ClaudeCurrentAuthInfo) {
    if let Some(email) = &info.email {
        ColorOutput::key_value("邮箱", &service.mask_email(email), 2);
    }
    if let Some(account_uuid) = &info.account_uuid {
        ColorOutput::key_value("账号 UUID", &mask_uuid(account_uuid), 2);
    }
    if let Some(billing_type) = &info.billing_type {
        ColorOutput::key_value("计费类型", billing_type, 2);
    }
    if let Some(subscription_type) = &info.subscription_type {
        ColorOutput::key_value("订阅类型", subscription_type, 2);
    }
    if let Some(rate_limit_tier) = &info.rate_limit_tier {
        ColorOutput::key_value("速率档位", rate_limit_tier, 2);
    }
    if let Some(expires_at) = info.expires_at {
        let local = expires_at.with_timezone(&chrono::Local);
        ColorOutput::key_value(
            "Access Token 到期时间",
            &local.format("%Y-%m-%d %H:%M:%S").to_string(),
            2,
        );
    }
}

fn mask_uuid(value: &str) -> String {
    if value.len() <= 8 {
        return value.to_string();
    }
    format!("{}...{}", &value[..4], &value[value.len() - 4..])
}
