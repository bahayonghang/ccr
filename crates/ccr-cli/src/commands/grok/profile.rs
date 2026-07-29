#![allow(clippy::unused_async)]

use crate::cli::subcommands::grok::GrokProfileCreateActionArgs;
use crate::cli::subcommands::profile_args::{
    ProfileDisableActionArgs, ProfileNameJsonActionArgs, ProfileSetFieldActionArgs,
};
use crate::commands::platform::{
    PlatformProfileCreateArgs, platform_profile_create_command, platform_profile_delete_command,
    platform_profile_disable_command, platform_profile_enable_command,
    platform_profile_set_field_command,
};
use crate::commands::profile::switch_command_for_platform;
use crate::models::{PlatformConfig, ProfileConfig};
use crate::platforms::GrokPlatform;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;
use comfy_table::{
    Attribute, Cell, Color as TableColor, ContentArrangement, Table, presets::UTF8_FULL,
};
use serde::Serialize;

const PROFILE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/grok/profiles.toml"
));

#[derive(Debug, Serialize)]
struct GrokProfileSummary {
    name: String,
    is_current: bool,
    enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    api_backend: String,
    auth_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    env_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context_window: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    supports_backend_search: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<String>,
}

#[derive(Debug, Serialize)]
struct GrokProfileListJson {
    current_profile: Option<String>,
    profiles: Vec<GrokProfileSummary>,
}

#[derive(Debug, Serialize)]
struct GrokProfileCurrentJson {
    platform: &'static str,
    profile: Option<String>,
    runtime_mode: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<GrokProfileSummary>,
}

#[derive(Debug, Serialize)]
struct GrokProfileOffJson {
    ok: bool,
    changed: bool,
    previous_profile: Option<String>,
    runtime_mode: &'static str,
}

pub async fn init_command(json: bool) -> Result<()> {
    crate::commands::platform::platform_profile_init_command("grok", PROFILE_TEMPLATE, json).await
}

fn profile_string(profile: &ProfileConfig, key: &str) -> Option<String> {
    profile
        .platform_data
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}

fn profile_summary(
    name: String,
    profile: ProfileConfig,
    current_profile: Option<&str>,
) -> Result<GrokProfileSummary> {
    let base_url = profile
        .base_url
        .as_deref()
        .map(GrokPlatform::safe_base_url_for_display);
    let auth_mode = GrokPlatform::profile_auth_mode(&profile)?
        .as_str()
        .to_string();
    let api_backend =
        profile_string(&profile, "api_backend").unwrap_or_else(|| "responses".to_string());
    let env_key = profile_string(&profile, "env_key");
    let context_window = profile
        .platform_data
        .get("context_window")
        .and_then(serde_json::Value::as_u64);
    let supports_backend_search = profile
        .platform_data
        .get("supports_backend_search")
        .and_then(serde_json::Value::as_bool);
    let reasoning_effort = profile_string(&profile, "reasoning_effort");

    Ok(GrokProfileSummary {
        is_current: current_profile == Some(name.as_str()),
        enabled: profile.is_enabled(),
        description: profile.description,
        base_url,
        model: profile.model,
        provider: profile.provider,
        name,
        api_backend,
        auth_mode,
        env_key,
        context_window,
        supports_backend_search,
        reasoning_effort,
    })
}

pub async fn current_command(json: bool) -> Result<()> {
    let platform = GrokPlatform::new()?;
    let current_profile = platform.get_current_profile()?;
    let details = match current_profile.as_deref() {
        Some(name) => platform
            .load_profiles()?
            .shift_remove(name)
            .map(|profile| profile_summary(name.to_string(), profile, Some(name)))
            .transpose()?,
        None => None,
    };

    if json {
        let output = GrokProfileCurrentJson {
            platform: "grok",
            profile: current_profile,
            runtime_mode: if details.is_some() {
                "profile"
            } else {
                "grok_native"
            },
            details,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    ColorOutput::title("Grok Profile 当前状态");
    println!();
    if let Some(details) = details {
        println!("  Profile: {}", details.name.bright_yellow());
        println!("  Model: {}", details.model.as_deref().unwrap_or("-"));
        println!("  Auth: {}", details.auth_mode);
        println!(
            "  Base URL: {}",
            details.base_url.as_deref().unwrap_or("Grok native")
        );
    } else {
        ColorOutput::info("当前不在 Grok profile mode；运行时由 Grok 自身配置管理");
    }
    Ok(())
}

pub async fn list_command(json: bool) -> Result<()> {
    let platform = GrokPlatform::new()?;
    let current_profile = platform.get_current_profile()?;
    let entries = platform
        .load_profiles()?
        .into_iter()
        .map(|(name, profile)| profile_summary(name, profile, current_profile.as_deref()))
        .collect::<Result<Vec<_>>>()?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&GrokProfileListJson {
                current_profile,
                profiles: entries,
            })?
        );
        return Ok(());
    }

    ColorOutput::title("Grok Profiles");
    println!();
    if entries.is_empty() {
        ColorOutput::info("未找到 Grok profiles");
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
            Cell::new("模型")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("认证")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("API Backend")
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
            Cell::new(entry.model.unwrap_or_else(|| "-".to_string())),
            Cell::new(entry.auth_mode),
            Cell::new(entry.api_backend),
            Cell::new(if entry.enabled { "yes" } else { "no" }),
        ]);
    }

    println!("{table}");
    Ok(())
}

pub async fn switch_command(name: &str) -> Result<()> {
    switch_command_for_platform(name, "grok").await
}

pub async fn create_command(args: GrokProfileCreateActionArgs) -> Result<()> {
    platform_profile_create_command(PlatformProfileCreateArgs {
        platform_name: "grok".to_string(),
        name: args.name,
        description: args.description,
        base_url: args.base_url,
        auth_token: args.auth_token,
        model: args.model,
        small_fast_model: None,
        provider: args.provider,
        provider_type: args.provider_type,
        account: args.account,
        tags: args.tags,
        auth_mode: None,
        api_backend: args.api_backend,
        env_key: args.env_key,
        context_window: args.context_window,
        supports_backend_search: args.supports_backend_search,
        reasoning_effort: args.reasoning_effort,
        disabled: args.disabled,
        json: args.json,
    })
    .await
}

pub async fn set_field_command(args: ProfileSetFieldActionArgs) -> Result<()> {
    platform_profile_set_field_command(
        "grok",
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
    platform_profile_enable_command("grok", &args.name, args.json).await
}

pub async fn disable_command(args: ProfileDisableActionArgs) -> Result<()> {
    platform_profile_disable_command("grok", &args.name, args.force, args.json).await
}

pub async fn delete_command(args: ProfileDisableActionArgs) -> Result<()> {
    if !args.force {
        return platform_profile_delete_command("grok", &args.name, false, args.json).await;
    }

    let platform = GrokPlatform::new()?;
    match platform.delete_profile(&args.name) {
        Ok(()) => {}
        Err(CcrError::ValidationError(message)) if message.contains("当前处于激活状态") => {
            platform.clear_active_profile_runtime()?;
            platform.delete_profile(&args.name)?;
        }
        Err(error) => return Err(error),
    }

    let message = format!("已删除 grok 平台 Profile '{}'", args.name);
    if args.json {
        println!(
            "{}",
            serde_json::to_string(&serde_json::json!({
                "ok": true,
                "platform": "grok",
                "name": args.name,
                "message": message,
                "current_profile": platform.get_current_profile()?,
            }))?
        );
    } else {
        ColorOutput::success(&message);
    }
    Ok(())
}

pub async fn off_command(json: bool) -> Result<()> {
    let platform = GrokPlatform::new()?;
    let previous_profile = platform.get_current_profile()?;
    platform.clear_active_profile_runtime()?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&GrokProfileOffJson {
                ok: true,
                changed: previous_profile.is_some(),
                previous_profile,
                runtime_mode: "grok_native",
            })?
        );
        return Ok(());
    }

    if let Some(previous_profile) = previous_profile {
        ColorOutput::success(&format!(
            "已退出 Grok profile '{}'，并恢复进入 profile mode 前的 config.toml",
            previous_profile.bright_yellow()
        ));
    } else {
        ColorOutput::info("当前不在 Grok profile mode；无需执行 profile off");
    }
    Ok(())
}
