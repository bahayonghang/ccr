#![allow(clippy::unused_async)]

use crate::application::profile_off_for_platform;
use crate::cli::subcommands::profile_args::{
    ProfileCreateActionArgs, ProfileDisableActionArgs, ProfileNameJsonActionArgs,
    ProfileSetFieldActionArgs,
};
use crate::commands::platform::{
    PlatformProfileCreateArgs, platform_profile_create_command, platform_profile_delete_command,
    platform_profile_disable_command, platform_profile_enable_command,
    platform_profile_set_field_command, print_status_card,
};
use crate::commands::profile::switch_command_for_platform;
use crate::models::Platform;
use crate::platforms::{ClaudePlatform, create_platform};
use crate::services::RuntimeOverviewService;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;
use comfy_table::{
    Attribute, Cell, Color as TableColor, ContentArrangement, Table, presets::UTF8_FULL,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ClaudeProfileSummary {
    name: String,
    is_current: bool,
    enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    auth_source: String,
}

#[derive(Debug, Serialize)]
struct ClaudeProfileListJson {
    current_profile: Option<String>,
    profiles: Vec<ClaudeProfileSummary>,
}

#[derive(Debug, Serialize)]
struct ClaudeProfileOffJson {
    ok: bool,
    changed: bool,
    previous_profile: Option<String>,
    runtime_mode: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<String>,
}

pub async fn init_command(json: bool) -> Result<()> {
    let template = crate::commands::platform::profile_open::template_for(Platform::Claude);
    crate::commands::platform::platform_profile_init_command("claude", template, json).await
}

pub async fn open_command(json: bool) -> Result<()> {
    crate::commands::platform::platform_profile_open_command("claude", json).await
}

pub async fn current_command(json: bool) -> Result<()> {
    let overview = RuntimeOverviewService::load()?;
    if json {
        println!("{}", serde_json::to_string_pretty(&overview.claude)?);
        return Ok(());
    }

    ColorOutput::title("Claude Profile 当前状态");
    println!();
    print_status_card(&overview.claude);
    Ok(())
}

pub async fn list_command(json: bool) -> Result<()> {
    let platform = create_platform(Platform::Claude)?;
    let current_profile = platform.get_current_profile()?;
    let profiles = platform.load_profiles()?;

    let entries = profiles
        .into_iter()
        .map(|(name, profile)| ClaudeProfileSummary {
            auth_source: ClaudePlatform::profile_auth_source(&profile),
            enabled: profile.is_enabled(),
            is_current: current_profile.as_deref() == Some(name.as_str()),
            model: profile.model.clone(),
            name,
            provider: profile.provider.clone(),
        })
        .collect::<Vec<_>>();

    if json {
        let output = ClaudeProfileListJson {
            current_profile,
            profiles: entries,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    ColorOutput::title("Claude Profiles");
    println!();
    if entries.is_empty() {
        ColorOutput::info("未找到 Claude profiles");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(vec![
            Cell::new("状态")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("名称")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("Provider")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("模型")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("认证来源")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("启用")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    for entry in entries {
        let status = if entry.is_current {
            Cell::new(">> 当前")
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else {
            Cell::new("")
        };

        table.add_row(vec![
            status,
            Cell::new(entry.name),
            Cell::new(entry.provider.unwrap_or_else(|| "-".to_string())),
            Cell::new(entry.model.unwrap_or_else(|| "-".to_string())),
            Cell::new(entry.auth_source),
            Cell::new(if entry.enabled { "✓" } else { "✗" }),
        ]);
    }

    println!("{table}");
    Ok(())
}

pub async fn switch_command(name: &str) -> Result<()> {
    switch_command_for_platform(name, "claude").await
}

pub async fn create_command(args: ProfileCreateActionArgs) -> Result<()> {
    platform_profile_create_command(PlatformProfileCreateArgs {
        platform_name: "claude".to_string(),
        name: args.name,
        description: args.description,
        base_url: args.base_url,
        auth_token: args.auth_token,
        api_key: None,
        model: args.model,
        small_fast_model: args.small_fast_model,
        provider: args.provider,
        provider_type: args.provider_type,
        account: args.account,
        tags: args.tags,
        auth_mode: args.auth_mode,
        api_backend: None,
        env_key: None,
        context_window: None,
        supports_backend_search: None,
        reasoning_effort: None,
        disabled: args.disabled,
        json: args.json,
    })
    .await
}

pub async fn set_field_command(args: ProfileSetFieldActionArgs) -> Result<()> {
    platform_profile_set_field_command(
        "claude",
        &args.name,
        &args.field,
        args.value,
        args.value_json,
        args.clear,
        args.json,
    )
    .await
}

pub async fn enable_command(args: ProfileNameJsonActionArgs) -> Result<()> {
    platform_profile_enable_command("claude", &args.name, args.json).await
}

pub async fn disable_command(args: ProfileDisableActionArgs) -> Result<()> {
    platform_profile_disable_command("claude", &args.name, args.force, args.json).await
}

pub async fn delete_command(args: ProfileDisableActionArgs) -> Result<()> {
    platform_profile_delete_command("claude", &args.name, args.force, args.json).await
}

pub async fn off_command(json: bool) -> Result<()> {
    let result = profile_off_for_platform(Platform::Claude)?;

    if json {
        let output = ClaudeProfileOffJson {
            ok: true,
            changed: result.changed,
            previous_profile: result.previous_profile,
            runtime_mode: "official_auth",
            warnings: result
                .auth_outcome
                .as_ref()
                .map(|outcome| outcome.warnings.clone())
                .unwrap_or_default(),
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    if result.changed {
        ColorOutput::success(&format!(
            "已退出 Claude profile '{}'，当前回到 official auth runtime",
            result
                .previous_profile
                .as_deref()
                .unwrap_or("-")
                .bright_yellow()
        ));
    } else {
        ColorOutput::info("当前不在 Claude profile mode；无需执行 profile off");
    }

    if let Some(outcome) = &result.auth_outcome {
        if !outcome.remaining_suppressors.is_empty() {
            ColorOutput::warning(
                "退出 Profile 后仍存在 CCR 不会自动清理的认证来源（请按置信度判断）:",
            );
            for source in &outcome.remaining_suppressors {
                println!(
                    "  • {} @ {} ({}; {}; {})",
                    source.kind.as_str(),
                    source.location.as_str(),
                    source.confidence.as_str(),
                    source.evidence.as_str(),
                    source.ownership.as_str()
                );
            }
        } else if !outcome.warnings.is_empty() {
            ColorOutput::warning("退出 Profile 后认证来源诊断未完成:");
            for warning in &outcome.warnings {
                println!("  • {warning}");
            }
        }
    }

    Ok(())
}
