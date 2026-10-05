//! `ccr grok auth` handlers.

#![allow(clippy::unused_async)]

use crate::commands::claude::auth::off::print_auth_off;
use crate::services::GrokAuthService;
use crate::services::grok_auth_service::{GrokAuthMutation, GrokAuthSnapshot};
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use serde::Serialize;
use std::io::{self, Write};

#[derive(Serialize)]
struct AccountJson<'a> {
    name: &'a str,
    scope: &'a str,
    email: &'a Option<String>,
    team: &'a Option<String>,
    saved_at: &'a str,
    expires_at: &'a Option<String>,
    expired: bool,
    local_match: bool,
}

#[derive(Serialize)]
struct SourceJson<'a> {
    scope: &'a str,
    email: &'a Option<String>,
    team: &'a Option<String>,
    matched_account: &'a Option<String>,
}

#[derive(Serialize)]
struct ListJson<'a> {
    accounts: Vec<AccountJson<'a>>,
    sources: Vec<SourceJson<'a>>,
    runtime_present: bool,
    runtime_error: &'a Option<String>,
}

#[derive(Serialize)]
struct MutationJson<'a> {
    name: &'a str,
    cancelled: bool,
    outgoing_saved: bool,
    warnings: Vec<String>,
}

fn print_mutation(
    name: &str,
    action: &str,
    result: GrokAuthMutation,
    json: bool,
    cancelled: bool,
) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&MutationJson {
                name,
                cancelled,
                outgoing_saved: result.outgoing_saved,
                warnings: result.warnings,
            })?
        );
    } else if cancelled {
        ColorOutput::info("已取消删除");
    } else {
        ColorOutput::success(&format!("{action}: {name}"));
        if result.outgoing_saved {
            ColorOutput::info("已回存原账号的最新运行时凭据");
        }
        for warning in result.warnings {
            ColorOutput::warning(&warning);
        }
    }
    Ok(())
}

fn select_scope<'a>(snapshot: &'a GrokAuthSnapshot, requested: Option<&str>) -> Result<&'a str> {
    if let Some(scope) = requested {
        return snapshot
            .sources
            .iter()
            .find(|source| source.scope == scope)
            .map(|source| source.scope.as_str())
            .ok_or_else(|| {
                CcrError::ValidationError(
                    "无效的 OAuth scope；运行 ccr grok auth list 查看来源".into(),
                )
            });
    }
    match snapshot.sources.as_slice() {
        [source] => Ok(&source.scope),
        [] => Err(CcrError::ValidationError(
            snapshot
                .runtime_error
                .clone()
                .unwrap_or_else(|| "没有可保存的官方 OAuth 来源；请先通过 Grok 登录".into()),
        )),
        sources => Err(CcrError::ValidationError(format!(
            "多个 OAuth 来源，请使用 --scope 明确选择: {}",
            sources
                .iter()
                .map(|source| source.scope.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))),
    }
}

pub async fn save_command(name: &str, scope: Option<&str>, force: bool, json: bool) -> Result<()> {
    let service = GrokAuthService::new();
    let snapshot = service.read_snapshot()?;
    let scope = select_scope(&snapshot, scope)?;
    let result = service.save_current(name, scope, &snapshot.revision, force)?;
    print_mutation(name, "已保存 Grok 账号", result, json, false)
}

pub async fn list_command(json: bool) -> Result<()> {
    let snapshot = GrokAuthService::new().read_snapshot()?;
    if json {
        let dto = ListJson {
            accounts: snapshot
                .accounts
                .iter()
                .map(|a| AccountJson {
                    name: &a.name,
                    scope: &a.scope,
                    email: &a.email,
                    team: &a.team,
                    saved_at: &a.saved_at,
                    expires_at: &a.expires_at,
                    expired: a.expired,
                    local_match: a.local_match,
                })
                .collect(),
            sources: snapshot
                .sources
                .iter()
                .map(|s| SourceJson {
                    scope: &s.scope,
                    email: &s.email,
                    team: &s.team,
                    matched_account: &s.matched_account,
                })
                .collect(),
            runtime_present: snapshot.runtime_present,
            runtime_error: &snapshot.runtime_error,
        };
        println!("{}", serde_json::to_string_pretty(&dto)?);
    } else {
        ColorOutput::title("Grok 已保存账号（本地匹配不代表服务端登录有效）");
        let mut table = crate::commands::common::new_table();
        table.set_header(["名称", "Scope", "邮箱", "本地匹配", "已过期"]);
        for account in &snapshot.accounts {
            table.add_row(vec![
                account.name.clone(),
                account.scope.clone(),
                account.email.clone().unwrap_or_default(),
                account.local_match.to_string(),
                account.expired.to_string(),
            ]);
        }
        println!("{table}");
        ColorOutput::info("可保存的 OAuth 来源:");
        for source in &snapshot.sources {
            println!(
                "  {}  {}",
                source.scope,
                source.matched_account.as_deref().unwrap_or("未匹配")
            );
        }
        if let Some(error) = &snapshot.runtime_error {
            ColorOutput::warning(error);
        }
    }
    Ok(())
}

pub async fn switch_command(name: &str, json: bool) -> Result<()> {
    let service = GrokAuthService::new();
    let snapshot = service.read_snapshot()?;
    let result = service.switch_account(name, &snapshot.revision)?;
    print_mutation(
        name,
        "已切换 Grok 账号；请启动新的 Grok 会话",
        result,
        json,
        false,
    )
}

pub async fn delete_command(name: &str, force: bool, json: bool) -> Result<()> {
    let service = GrokAuthService::new();
    let snapshot = service.read_snapshot()?;
    if !snapshot.accounts.iter().any(|account| account.name == name) {
        return Err(CcrError::ValidationError(
            "Grok 保存账号不存在；运行 ccr grok auth list 查看账号".into(),
        ));
    }
    if !force {
        let confirmed = tokio::task::spawn_blocking(|| -> io::Result<bool> {
            // Keep JSON stdout machine-readable even when asking for confirmation.
            eprint!("删除保存账号不会登出运行时。确认删除? [y/N]: ");
            io::stderr().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            Ok(matches!(
                input.trim().to_ascii_lowercase().as_str(),
                "y" | "yes"
            ))
        })
        .await
        .map_err(|e| CcrError::FileIoError(format!("读取确认输入失败: {e}")))??;
        if !confirmed {
            return print_mutation(name, "", GrokAuthMutation::default(), json, true);
        }
    }
    let result = service.delete_account(name, &snapshot.revision)?;
    print_mutation(
        name,
        "已删除 Grok 保存账号（运行时未登出）",
        result,
        json,
        false,
    )
}

#[derive(Debug, Serialize)]
struct GrokAuthCurrentJson {
    logged_in: bool,
}

pub async fn current_command(json: bool) -> Result<()> {
    let current = GrokAuthService::new().current()?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&GrokAuthCurrentJson {
                logged_in: current.logged_in,
            })?
        );
        return Ok(());
    }

    ColorOutput::title("Grok 官方会话");
    if current.logged_in {
        ColorOutput::success("已检测到官方会话文件（auth.json 存在）");
    } else {
        ColorOutput::info("未检测到官方会话；运行时可回退 XAI_API_KEY");
    }
    Ok(())
}

pub async fn off_command(json: bool) -> Result<()> {
    let result = GrokAuthService::new().off()?;
    print_auth_off(
        json,
        result.changed,
        result.path,
        result.profile_pointer,
        result.warnings,
    )
}
