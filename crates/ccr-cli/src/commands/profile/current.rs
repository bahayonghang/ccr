// 🔍 current 命令实现 - 显示当前配置状态
// 📊 显示当前激活的配置详情和 Claude Code 环境变量状态
// 🔄 显示平台信息和路径

#![allow(clippy::unused_async)]

use crate::managers::ConfigManager;
use crate::managers::PlatformConfigManager;
use crate::models::{
    AuthIntent, AuthState, AuthStateStatus, OpenAiAuthMethod, Platform, PlatformPaths,
};
use crate::platforms::{ClaudePlatform, create_platform};
use crate::services::runtime_overview_service::current_profile_from_file;
use crate::services::{
    CodexAuthService, PlatformStatusCard, RuntimeOverview, RuntimeOverviewService, SettingsService,
    StatusAuthKind, StatusHealth,
};
use ccr_config::profile_to_section;
use ccr_core::Validatable;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;
use comfy_table::{
    Attribute, Cell, Color as TableColor, ContentArrangement, Table, presets::UTF8_FULL,
};
use std::str::FromStr;

fn print_runtime_overview(overview: &RuntimeOverview) {
    ColorOutput::title("当前运行状态");
    println!();
    print_status_card(&overview.claude);
    println!();
    print_status_card(&overview.codex);
    println!();
    ColorOutput::info("提示: 使用 `ccr current --verbose` 查看路径、环境变量和完整配置详情");
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

    if let Some(base_url) = card.base_url.as_deref().filter(|value| !value.is_empty()) {
        table.add_row(vec![
            Cell::new("Base URL"),
            Cell::new(base_url).fg(TableColor::Blue),
        ]);
    }

    if let Some(model) = card.model.as_deref().filter(|value| !value.is_empty()) {
        table.add_row(vec![
            Cell::new("主模型"),
            Cell::new(model).fg(TableColor::Magenta),
        ]);
    }

    if let Some(small_model) = card
        .small_fast_model
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        table.add_row(vec![
            Cell::new("快速模型"),
            Cell::new(small_model).fg(TableColor::Magenta),
        ]);
    }

    table.add_row(vec![
        Cell::new("认证").fg(TableColor::Yellow),
        Cell::new(card.auth.as_str()).fg(auth_color(card.auth_kind)),
    ]);

    table.add_row(vec![Cell::new("说明"), Cell::new(card.note.as_str())]);

    println!("{table}");
}

fn render_health(health: StatusHealth) -> &'static str {
    match health {
        StatusHealth::Ready => "✓ 就绪",
        StatusHealth::NeedsLogin => "⚠ 需登录",
        StatusHealth::Invalid => "✗ 无效",
        StatusHealth::Unsupported => "○ 不支持",
        StatusHealth::Error => "✗ 错误",
    }
}

fn health_color(health: StatusHealth) -> TableColor {
    match health {
        StatusHealth::Ready => TableColor::Green,
        StatusHealth::NeedsLogin => TableColor::Yellow,
        StatusHealth::Invalid | StatusHealth::Error => TableColor::Red,
        StatusHealth::Unsupported => TableColor::DarkGrey,
    }
}

fn auth_color(kind: StatusAuthKind) -> TableColor {
    match kind {
        StatusAuthKind::OfficialAuth => TableColor::Green,
        StatusAuthKind::ThirdPartyApi | StatusAuthKind::ProviderKey => TableColor::Cyan,
        StatusAuthKind::NoAuth => TableColor::DarkGrey,
        StatusAuthKind::Missing | StatusAuthKind::Unknown => TableColor::Yellow,
    }
}

/// 🔍 显示当前配置状态
///
/// 显示内容分为三部分:
/// 1. 🔄 平台信息
///    - 当前平台
///    - 平台路径
///
/// 2. 📝 配置文件信息
///    - 当前配置名称
///    - 配置详情(描述、URL、Token、模型等)
///    - 配置验证状态
///
/// 3. 🌍 Claude Code 环境变量状态
///    - ANTHROPIC_* 环境变量当前值
///    - 设置验证状态
pub async fn current_command(verbose: bool, json: bool) -> Result<()> {
    if json {
        let overview = RuntimeOverviewService::load()?;
        println!("{}", serde_json::to_string_pretty(&overview)?);
        return Ok(());
    }

    if !verbose {
        let overview = RuntimeOverviewService::load()?;
        print_runtime_overview(&overview);
        return Ok(());
    }

    current_command_verbose().await
}

async fn current_command_verbose() -> Result<()> {
    ColorOutput::title("当前配置状态");
    println!();

    ColorOutput::step("📊 Runtime 总览");
    println!();
    let overview = RuntimeOverviewService::load()?;
    print_status_card(&overview.claude);
    println!();
    print_status_card(&overview.codex);

    println!();
    ColorOutput::separator();
    println!();

    // 🔍 加载平台配置
    let platform_config_mgr = PlatformConfigManager::with_default()?;
    let unified_config = platform_config_mgr.load()?;

    // === 第零部分：平台信息 ===
    ColorOutput::step("🔄 Registry 目标信息");
    println!();

    let platform_name = &unified_config.current_platform;
    let platform = Platform::from_str(platform_name)?;
    let paths = PlatformPaths::new(platform)?;

    let mut platform_table = Table::new();
    platform_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(vec![
            Cell::new("属性")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("值")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    platform_table.add_row(vec![
        Cell::new("Registry 目标平台").fg(TableColor::Yellow),
        Cell::new(platform_name)
            .fg(TableColor::Green)
            .add_attribute(Attribute::Bold),
    ]);

    platform_table.add_row(vec![
        Cell::new("平台目录"),
        Cell::new(paths.platform_dir.display().to_string()).fg(TableColor::Blue),
    ]);

    platform_table.add_row(vec![
        Cell::new("配置文件"),
        Cell::new(paths.profiles_file.display().to_string()).fg(TableColor::Blue),
    ]);

    platform_table.add_row(vec![
        Cell::new("历史文件"),
        Cell::new(paths.history_file.display().to_string()).fg(TableColor::Blue),
    ]);

    platform_table.add_row(vec![
        Cell::new("备份目录"),
        Cell::new(paths.backups_dir.display().to_string()).fg(TableColor::Blue),
    ]);

    println!("{}", platform_table);
    println!();
    ColorOutput::separator();
    println!();

    // 从平台配置读取
    let platform_config = create_platform(platform)?;

    // 获取当前 profile
    // 加载 profiles
    let profiles = platform_config.load_profiles()?;
    let current_profile = match platform_config.get_current_profile()? {
        Some(current) => current,
        None if platform == Platform::Codex => {
            let (fallback_name, _) = current_profile_from_file(Platform::Codex);
            fallback_name.ok_or_else(|| {
                ccr_core::core::error::CcrError::ConfigError("未设置当前 profile".to_string())
            })?
        }
        None => {
            return Err(ccr_core::core::error::CcrError::ConfigError(
                "未设置当前 profile".to_string(),
            ));
        }
    };
    let profile = profiles.get(&current_profile).ok_or_else(|| {
        ccr_core::core::error::CcrError::ConfigSectionNotFound(current_profile.clone())
    })?;

    // 转换为 ConfigSection（统一复用平台公共转换逻辑）
    let mut current_section = profile_to_section(profile)?;

    let mut codex_auth_state = None;
    if platform == Platform::Codex
        && let Ok(service) = CodexAuthService::new()
    {
        let state = service.get_auth_state();
        if matches!(state.store, crate::models::CredentialStoreKind::File) {
            use crate::managers::CodexConfigManager;
            if let Ok(mgr) = CodexConfigManager::with_default()
                && let Ok(auth) = mgr.load_auth()
            {
                let auth_key_name = match &state.intent {
                    AuthIntent::OpenAiAuth {
                        method: OpenAiAuthMethod::Api,
                    } => Some("OPENAI_API_KEY"),
                    AuthIntent::ProviderEnvKey { env_key } => Some(env_key.as_str()),
                    _ => None,
                };

                if let Some(auth_key_name) = auth_key_name
                    && let Some(token) = auth.get(auth_key_name).and_then(|v| v.as_str())
                {
                    current_section.auth_token = Some(ccr_core::Secret::from(token));
                }
            }
        }
        codex_auth_state = Some(state);
    }

    let current_name = current_profile;
    let config_file_path = paths.profiles_file.clone();
    let default_name = ConfigManager::for_platform(platform_name)
        .and_then(|m| m.load_with_autofix())
        .map(|cfg| cfg.default_config)
        .unwrap_or_else(|_| "-".to_string());

    println!();
    ColorOutput::info(&format!("配置文件: {}", config_file_path.display()));
    ColorOutput::info(&format!("默认 Profile: {}", default_name.bright_yellow()));
    println!();

    // === 第一部分：配置详情表格 ===
    ColorOutput::step("📋 配置详情");
    println!();

    let mut config_table = Table::new();
    config_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(vec![
            Cell::new("属性")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("值")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    // 配置名称
    config_table.add_row(vec![
        Cell::new("配置名称").fg(TableColor::Yellow),
        Cell::new(&current_name)
            .fg(TableColor::Green)
            .add_attribute(Attribute::Bold),
    ]);

    // 描述
    config_table.add_row(vec![
        Cell::new("描述"),
        Cell::new(current_section.display_description()),
    ]);

    // 提供商类型
    if let Some(provider_type) = &current_section.provider_type {
        let type_display = match provider_type.to_string_value() {
            "official_relay" => "🔄 官方中转",
            "third_party_model" => "🤖 第三方模型",
            _ => provider_type.to_string_value(),
        };
        config_table.add_row(vec![
            Cell::new("提供商类型").fg(TableColor::Yellow),
            Cell::new(type_display).fg(TableColor::Cyan),
        ]);
    }

    // 提供商
    if let Some(provider) = &current_section.provider {
        config_table.add_row(vec![
            Cell::new("提供商").fg(TableColor::Yellow),
            Cell::new(provider).fg(TableColor::Cyan),
        ]);
    }

    if platform == Platform::Claude {
        config_table.add_row(vec![
            Cell::new("认证模式").fg(TableColor::Yellow),
            Cell::new(ClaudePlatform::profile_auth_mode(profile).as_str()).fg(TableColor::Cyan),
        ]);
        config_table.add_row(vec![
            Cell::new("认证来源").fg(TableColor::Yellow),
            Cell::new(ClaudePlatform::profile_auth_source(profile)).fg(TableColor::Cyan),
        ]);
    }

    if let Some(auth_state) = &codex_auth_state {
        config_table.add_row(vec![
            Cell::new("凭据存储").fg(TableColor::Yellow),
            Cell::new(auth_state.store.as_str()).fg(TableColor::Cyan),
        ]);
        config_table.add_row(vec![
            Cell::new("认证来源").fg(TableColor::Yellow),
            Cell::new(render_auth_source(auth_state, profile)).fg(TableColor::Cyan),
        ]);
    }

    // Base URL
    if let Some(base_url) = &current_section.base_url {
        config_table.add_row(vec![
            Cell::new("Base URL")
                .fg(TableColor::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new(base_url).fg(TableColor::Blue),
        ]);
    }

    // Auth Token (脱敏)
    if let Some(auth_token) = &current_section.auth_token {
        config_table.add_row(vec![
            Cell::new("Auth Token")
                .fg(TableColor::Yellow)
                .add_attribute(Attribute::Bold),
            Cell::new(auth_token.to_string()).fg(TableColor::DarkGrey),
        ]);
    }

    // Model
    if let Some(model) = &current_section.model {
        config_table.add_row(vec![
            Cell::new("主模型"),
            Cell::new(model).fg(TableColor::Magenta),
        ]);
    }

    // Small Fast Model
    if let Some(small_model) = &current_section.small_fast_model {
        config_table.add_row(vec![
            Cell::new("快速小模型"),
            Cell::new(small_model).fg(TableColor::Magenta),
        ]);
    }

    // 账号
    if let Some(account) = &current_section.account {
        config_table.add_row(vec![
            Cell::new("账号标识"),
            Cell::new(format!("👤 {}", account)).fg(TableColor::Yellow),
        ]);
    }

    // 标签
    if let Some(tags) = &current_section.tags
        && !tags.is_empty()
    {
        config_table.add_row(vec![
            Cell::new("标签"),
            Cell::new(format!("🏷️  {}", tags.join(", "))).fg(TableColor::Magenta),
        ]);
    }

    // 验证状态
    let validation_status = match platform_config.validate_profile(profile) {
        Ok(_) => Cell::new("✓ 配置完整")
            .fg(TableColor::Green)
            .add_attribute(Attribute::Bold),
        Err(e) => Cell::new(format!("✗ 配置不完整: {}", e))
            .fg(TableColor::Red)
            .add_attribute(Attribute::Bold),
    };
    config_table.add_row(vec![
        Cell::new("验证状态").fg(TableColor::Yellow),
        validation_status,
    ]);

    println!("{}", config_table);
    println!();

    // === 第二部分：平台环境变量表格（根据平台动态显示）===
    ColorOutput::separator();
    println!();

    // 根据当前平台获取环境变量名称
    let env_vars = platform_config.get_env_var_names();

    // 仅在有环境变量时显示
    if !env_vars.is_empty() {
        let platform_display = match platform {
            Platform::Claude => "Claude Code",
            Platform::Codex => "Codex",
            Platform::Gemini => "Antigravity CLI",
            Platform::Qwen => "Qwen",
            Platform::Droid => "Factory Droid",
        };
        ColorOutput::step(&format!("🌍 {} 环境变量状态", platform_display));
        println!();

        // 对于 Claude 平台，从 settings.json 读取环境变量
        // 对于其他平台，从系统环境变量读取
        let settings_env = if platform == Platform::Claude {
            match SettingsService::with_default() {
                Ok(service) => service
                    .get_current_settings_async()
                    .await
                    .ok()
                    .map(|s| s.env),
                Err(_) => None,
            }
        } else {
            None
        };

        let mut env_table = Table::new();
        env_table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::DynamicFullWidth)
            .set_header(vec![
                Cell::new("环境变量")
                    .add_attribute(Attribute::Bold)
                    .fg(TableColor::Cyan),
                Cell::new("当前值")
                    .add_attribute(Attribute::Bold)
                    .fg(TableColor::Cyan),
                Cell::new("状态")
                    .add_attribute(Attribute::Bold)
                    .fg(TableColor::Cyan),
            ]);

        for var_name in &env_vars {
            // 优先从 settings.json 读取，如果没有则从系统环境变量读取
            let value = if let Some(ref env_map) = settings_env {
                env_map.get(var_name.as_str()).cloned()
            } else {
                std::env::var(var_name).ok()
            };
            let is_sensitive = var_name.contains("TOKEN") || var_name.contains("KEY");

            let var_cell = Cell::new(format!("{} *", var_name)).fg(TableColor::Yellow);

            let (value_cell, status_cell) = match value {
                Some(v) => {
                    let display_value = if is_sensitive {
                        ColorOutput::mask_sensitive(&v)
                    } else if v.len() > 40 {
                        format!("{}...", &v[..37])
                    } else {
                        v
                    };
                    (
                        Cell::new(display_value).fg(TableColor::Blue),
                        Cell::new("✓")
                            .fg(TableColor::Green)
                            .add_attribute(Attribute::Bold),
                    )
                }
                None => (
                    Cell::new("(未设置)").fg(TableColor::DarkGrey),
                    Cell::new("○").fg(TableColor::DarkGrey),
                ),
            };

            env_table.add_row(vec![var_cell, value_cell, status_cell]);
        }

        println!("{}", env_table);
        println!();
        ColorOutput::info("提示: * 标记的为必需环境变量");
    }

    // 验证设置（仅对 Claude 平台）
    if platform == Platform::Claude {
        match SettingsService::with_default() {
            Ok(settings_service) => match settings_service.get_current_settings_async().await {
                Ok(settings) => match settings.validate() {
                    Ok(_) => ColorOutput::success("✓ Claude Code 设置验证通过"),
                    Err(e) => ColorOutput::warning(&format!("⚠ 设置验证警告: {}", e)),
                },
                Err(e) => {
                    ColorOutput::warning(&format!("无法加载 Claude Code 设置: {}", e));
                }
            },
            Err(e) => {
                ColorOutput::warning(&format!("无法访问 Claude Code 设置: {}", e));
            }
        }
    }

    Ok(())
}

fn render_auth_source(auth_state: &AuthState, profile: &crate::models::ProfileConfig) -> String {
    if matches!(auth_state.status, AuthStateStatus::Unsupported) {
        return "unsupported".to_string();
    }

    match &auth_state.intent {
        AuthIntent::OpenAiAuth { method } => match method {
            OpenAiAuthMethod::Chatgpt => "openai_chatgpt".to_string(),
            OpenAiAuthMethod::Api => "openai_api_key".to_string(),
        },
        AuthIntent::ProviderEnvKey { env_key } => format!("provider:{env_key}"),
        AuthIntent::NoAuth => {
            if ccr_codex::CodexPlatform::profile_auth_mode(profile)
                == crate::models::CodexProfileAuthMode::ProviderEnvKey
            {
                ccr_codex::CodexPlatform::profile_auth_source(profile)
            } else {
                "none".to_string()
            }
        }
    }
}
