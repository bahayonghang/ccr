#![allow(clippy::unused_async)]

use crate::application::profile_off_for_platform;
use crate::cli::subcommands::profile_args::{
    ProfileCreateActionArgs, ProfileDisableActionArgs, ProfileNameJsonActionArgs,
    ProfileSetFieldActionArgs,
};
use crate::commands::platform::{
    PlatformProfileCreateArgs, platform_profile_create_command, platform_profile_delete_command,
    platform_profile_disable_command, platform_profile_enable_command,
    platform_profile_set_field_command,
};
use crate::commands::profile::switch_command_for_platform;
use crate::models::Platform;
use crate::platforms::{ClaudePlatform, create_platform};
use crate::services::{PlatformStatusCard, RuntimeOverviewService};
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
        model: args.model,
        small_fast_model: args.small_fast_model,
        provider: args.provider,
        provider_type: args.provider_type,
        account: args.account,
        tags: args.tags,
        auth_mode: args.auth_mode,
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

    Ok(())
}

fn print_status_card(card: &PlatformStatusCard) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(vec![
            Cell::new(card.display_name.as_str())
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new(render_health(card.health))
                .add_attribute(Attribute::Bold)
                .fg(health_color(card.health)),
        ]);

    table.add_row(vec![
        Cell::new("Profile").fg(TableColor::Yellow),
        Cell::new(card.profile.as_str())
            .fg(TableColor::Green)
            .add_attribute(Attribute::Bold),
    ]);
    if let Some(provider) = card.provider.as_deref().filter(|value| !value.is_empty()) {
        table.add_row(vec![Cell::new("Provider"), Cell::new(provider)]);
    }
    if let Some(model) = card.model.as_deref().filter(|value| !value.is_empty()) {
        table.add_row(vec![Cell::new("主模型"), Cell::new(model)]);
    }
    table.add_row(vec![
        Cell::new("认证").fg(TableColor::Yellow),
        Cell::new(card.auth.as_str()).fg(auth_color(card.auth_kind)),
    ]);
    table.add_row(vec![Cell::new("说明"), Cell::new(card.note.as_str())]);

    println!("{table}");
}

fn render_health(health: crate::services::StatusHealth) -> &'static str {
    match health {
        crate::services::StatusHealth::Ready => "✓ 就绪",
        crate::services::StatusHealth::NeedsLogin => "⚠ 需登录",
        crate::services::StatusHealth::Invalid => "✗ 无效",
        crate::services::StatusHealth::Unsupported => "○ 不支持",
        crate::services::StatusHealth::Error => "✗ 错误",
    }
}

fn health_color(health: crate::services::StatusHealth) -> TableColor {
    match health {
        crate::services::StatusHealth::Ready => TableColor::Green,
        crate::services::StatusHealth::NeedsLogin => TableColor::Yellow,
        crate::services::StatusHealth::Invalid | crate::services::StatusHealth::Error => {
            TableColor::Red
        }
        crate::services::StatusHealth::Unsupported => TableColor::DarkGrey,
    }
}

fn auth_color(kind: crate::services::StatusAuthKind) -> TableColor {
    match kind {
        crate::services::StatusAuthKind::OfficialAuth => TableColor::Green,
        crate::services::StatusAuthKind::ThirdPartyApi
        | crate::services::StatusAuthKind::ProviderKey => TableColor::Cyan,
        crate::services::StatusAuthKind::NoAuth => TableColor::DarkGrey,
        crate::services::StatusAuthKind::Missing | crate::services::StatusAuthKind::Unknown => {
            TableColor::Yellow
        }
    }
}
