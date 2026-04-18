//! 📍 codex auth current 命令实现
//!
//! 显示当前账号信息。

#![allow(clippy::unused_async)]

use crate::models::{AuthIntent, AuthState, LoginState};
use crate::services::CodexAuthService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct RuntimeSummaryJsonOutput {
    #[serde(flatten)]
    summary: crate::models::CodexRuntimeSummary,
    profile_label: String,
    auth_label: String,
}

#[derive(Debug, Serialize)]
struct CurrentAuthInfoJsonOutput {
    account_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_method: Option<crate::models::OpenAiAuthMethod>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plan_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_refresh: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
struct CodexAuthCurrentJsonOutput {
    runtime_summary: RuntimeSummaryJsonOutput,
    auth_state: AuthState,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_auth_info: Option<CurrentAuthInfoJsonOutput>,
}

fn build_json_output(
    runtime_summary: crate::models::CodexRuntimeSummary,
    current_auth_info: Option<&crate::models::CurrentAuthInfo>,
) -> CodexAuthCurrentJsonOutput {
    let auth_state = runtime_summary.auth_state.clone();

    CodexAuthCurrentJsonOutput {
        runtime_summary: RuntimeSummaryJsonOutput {
            profile_label: runtime_summary.profile_label(),
            auth_label: runtime_summary.auth_label(),
            summary: runtime_summary,
        },
        auth_state,
        current_auth_info: current_auth_info.map(|info| CurrentAuthInfoJsonOutput {
            account_id: info.account_id.clone(),
            auth_method: info.auth_method,
            email: info.email.clone(),
            plan_type: info.plan_type.clone(),
            last_refresh: info.last_refresh,
        }),
    }
}

/// 📍 显示当前账号信息
///
/// 显示当前 ~/.codex/auth.json 的账号信息。
///
/// # 返回
///
/// * `Ok(())` - 成功执行
/// * `Err(CcrError)` - 执行失败
pub async fn current_command(json: bool) -> Result<()> {
    let service = CodexAuthService::new()?;
    let runtime_summary = service.get_runtime_summary()?;

    let current_auth_info =
        if runtime_summary.auth_state.status == crate::models::AuthStateStatus::Valid {
            service.get_current_auth_info().ok()
        } else {
            None
        };

    if json {
        let output = build_json_output(runtime_summary, current_auth_info.as_ref());
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

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
                display_auth_info(&service, &info, &auth_state);

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
                println!();
                display_auth_info(&service, &info, &auth_state);
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
                display_auth_info(&service, &info, &auth_state);
            }
        }
        LoginState::ApiKeyActive => {
            ColorOutput::info("认证模式: API Key");

            if let Ok(info) = service.get_current_auth_info() {
                println!();
                display_auth_info(&service, &info, &auth_state);
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
                display_auth_info(&service, &info, &auth_state);
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
    if let Some(plan_type) = &info.plan_type {
        ColorOutput::info(&format!("套餐: {}", plan_type));
    }

    // 最后刷新时间
    if let Some(last_refresh) = &info.last_refresh {
        let local_time = last_refresh.with_timezone(&chrono::Local);
        ColorOutput::info(&format!(
            "最后刷新: {}",
            local_time.format("%Y-%m-%d %H:%M:%S")
        ));
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::{
        AuthIntent, AuthStateStatus, CodexProfileAuthMode, CodexRuntimeMode, CodexRuntimeSummary,
        CredentialStoreKind, OpenAiAuthMethod,
    };

    fn auth_state() -> AuthState {
        AuthState {
            intent: AuthIntent::OpenAiAuth {
                method: OpenAiAuthMethod::Chatgpt,
            },
            store: CredentialStoreKind::File,
            status: AuthStateStatus::Valid,
            reason: "ok".to_string(),
        }
    }

    fn runtime_summary(mode: CodexRuntimeMode) -> CodexRuntimeSummary {
        let current_auth_name = match mode {
            CodexRuntimeMode::ProfileWithAuth | CodexRuntimeMode::RuntimeOnly => {
                Some("team".to_string())
            }
            _ => None,
        };
        let current_profile_name = match mode {
            CodexRuntimeMode::RuntimeOnly => None,
            _ => Some("official".to_string()),
        };
        let current_profile_auth_mode = match mode {
            CodexRuntimeMode::ProfileOnly => Some(CodexProfileAuthMode::ProviderEnvKey),
            CodexRuntimeMode::ProfileWithAuth | CodexRuntimeMode::ProfilePendingAuth => {
                Some(CodexProfileAuthMode::OpenAiChatgpt)
            }
            CodexRuntimeMode::RuntimeOnly | CodexRuntimeMode::Unresolved => None,
        };
        let current_profile_auth_source = match mode {
            CodexRuntimeMode::ProfileOnly => Some("provider:DUCK_API_KEY".to_string()),
            CodexRuntimeMode::ProfileWithAuth | CodexRuntimeMode::ProfilePendingAuth => {
                Some("openai_chatgpt".to_string())
            }
            CodexRuntimeMode::RuntimeOnly | CodexRuntimeMode::Unresolved => None,
        };
        let login_state = match mode {
            CodexRuntimeMode::ProfileWithAuth | CodexRuntimeMode::RuntimeOnly => {
                LoginState::LoggedInSaved("team".to_string())
            }
            CodexRuntimeMode::ProfileOnly => LoginState::ProviderKeyActive {
                env_key: "DUCK_API_KEY".to_string(),
            },
            CodexRuntimeMode::ProfilePendingAuth | CodexRuntimeMode::Unresolved => {
                LoginState::NotLoggedIn
            }
        };
        let mut state = auth_state();
        if matches!(
            mode,
            CodexRuntimeMode::ProfilePendingAuth | CodexRuntimeMode::Unresolved
        ) {
            state.status = AuthStateStatus::Missing;
            state.reason = "missing".to_string();
        }
        if mode == CodexRuntimeMode::ProfileOnly {
            state.intent = AuthIntent::ProviderEnvKey {
                env_key: "DUCK_API_KEY".to_string(),
            };
        }

        CodexRuntimeSummary {
            mode,
            current_profile_name,
            current_profile_provider: Some("openai".to_string()),
            current_profile_auth_mode,
            current_profile_auth_source,
            current_auth_name,
            login_state,
            auth_state: state,
        }
    }

    #[test]
    fn json_output_includes_required_runtime_fields_for_all_runtime_modes() {
        for mode in [
            CodexRuntimeMode::ProfileWithAuth,
            CodexRuntimeMode::ProfileOnly,
            CodexRuntimeMode::ProfilePendingAuth,
            CodexRuntimeMode::RuntimeOnly,
        ] {
            let output = build_json_output(runtime_summary(mode), None);
            let json = serde_json::to_value(&output).unwrap();
            let runtime = json
                .get("runtime_summary")
                .and_then(|value| value.as_object())
                .unwrap();

            assert_eq!(
                runtime.get("mode").and_then(|value| value.as_str()),
                Some(match mode {
                    CodexRuntimeMode::ProfileWithAuth => "profile_with_auth",
                    CodexRuntimeMode::ProfileOnly => "profile_only",
                    CodexRuntimeMode::ProfilePendingAuth => "profile_pending_auth",
                    CodexRuntimeMode::RuntimeOnly => "runtime_only",
                    CodexRuntimeMode::Unresolved => "unresolved",
                })
            );
            assert!(runtime.contains_key("current_profile_name"));
            assert!(runtime.contains_key("current_profile_provider"));
            assert!(runtime.contains_key("current_profile_auth_mode"));
            assert!(runtime.contains_key("current_profile_auth_source"));
            assert!(runtime.contains_key("current_auth_name"));
            assert!(runtime.contains_key("profile_label"));
            assert!(runtime.contains_key("auth_label"));
            assert!(json.get("auth_state").is_some());
        }
    }

    #[test]
    fn json_output_serializes_current_auth_info_shape() {
        let info = crate::models::CurrentAuthInfo {
            account_id: "acc-123".to_string(),
            auth_method: Some(OpenAiAuthMethod::Chatgpt),
            email: Some("user@example.com".to_string()),
            plan_type: Some("plus".to_string()),
            last_refresh: Some(
                DateTime::parse_from_rfc3339("2026-04-07T12:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
        };

        let output = build_json_output(
            runtime_summary(CodexRuntimeMode::ProfileWithAuth),
            Some(&info),
        );
        let json = serde_json::to_value(&output).unwrap();
        let auth_info = json
            .get("current_auth_info")
            .and_then(|value| value.as_object())
            .unwrap();

        assert_eq!(
            auth_info.get("account_id").and_then(|value| value.as_str()),
            Some("acc-123")
        );
        assert_eq!(
            auth_info
                .get("auth_method")
                .and_then(|value| value.as_str()),
            Some("chatgpt")
        );
        assert_eq!(
            auth_info.get("email").and_then(|value| value.as_str()),
            Some("user@example.com")
        );
        assert_eq!(
            auth_info.get("plan_type").and_then(|value| value.as_str()),
            Some("plus")
        );
    }
}
