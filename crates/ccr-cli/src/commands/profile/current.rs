// 🔍 current 命令实现 - 显示当前配置状态
// 📊 显示当前激活的配置详情和 Claude Code 环境变量状态
// 🔄 显示平台信息和路径

#![allow(clippy::unused_async)]

use crate::managers::ConfigManager;
use crate::managers::PlatformConfigManager;
use crate::models::{
    AuthIntent, AuthState, AuthStateStatus, ClaudeLoginState, ClaudeProfileAuthMode,
    ClaudeRuntimeMode, ClaudeRuntimeSummary, CodexProfileAuthMode, CodexRuntimeMode,
    CodexRuntimeSummary, LoginState, OpenAiAuthMethod, Platform, PlatformPaths, ProfileConfig,
};
use crate::platforms::{ClaudePlatform, create_platform};
use crate::services::{ClaudeAuthService, CodexAuthService, SettingsService};
use ccr_config::profile_to_section;
use ccr_core::Validatable;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use colored::Colorize;
use comfy_table::{
    Attribute, Cell, Color as TableColor, ContentArrangement, Table, presets::UTF8_FULL,
};
use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, Serialize)]
pub struct StatusOverview {
    current_platform: String,
    claude: PlatformStatusCard,
    codex: PlatformStatusCard,
}

#[derive(Debug, Serialize)]
pub struct PlatformStatusCard {
    platform: String,
    display_name: String,
    profile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    small_fast_model: Option<String>,
    auth: String,
    auth_kind: StatusAuthKind,
    health: StatusHealth,
    note: String,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatusAuthKind {
    OfficialAuth,
    ThirdPartyApi,
    ProviderKey,
    NoAuth,
    Missing,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatusHealth {
    Ready,
    NeedsLogin,
    Invalid,
    Unsupported,
    Error,
}

impl StatusOverview {
    fn load() -> Result<Self> {
        let current_platform = PlatformConfigManager::with_default()?
            .load_or_create_default()
            .map(|config| config.current_platform)
            .unwrap_or_else(|_| "-".to_string());

        Ok(Self {
            current_platform,
            claude: build_claude_status_card(),
            codex: build_codex_status_card(),
        })
    }

    fn print_human(&self) {
        ColorOutput::title("当前运行状态");
        println!();
        ColorOutput::info(&format!(
            "当前平台: {}",
            self.current_platform.bright_yellow()
        ));
        println!();

        print_status_card(&self.claude);
        println!();
        print_status_card(&self.codex);
        println!();
        ColorOutput::info("提示: 使用 `ccr status --verbose` 查看路径、环境变量和完整配置详情");
    }
}

fn build_claude_status_card() -> PlatformStatusCard {
    let Ok(service) = ClaudeAuthService::new() else {
        return PlatformStatusCard::error(
            Platform::Claude,
            "无法访问 Claude 配置或认证文件".to_string(),
        );
    };

    match service.get_runtime_summary() {
        Ok(summary) => {
            let profile = load_profile(Platform::Claude, summary.current_profile_name.as_deref());
            status_card_from_claude_summary(summary, profile.as_ref())
        }
        Err(error) => PlatformStatusCard::error(Platform::Claude, error.to_string()),
    }
}

fn build_codex_status_card() -> PlatformStatusCard {
    let Ok(service) = CodexAuthService::new() else {
        return PlatformStatusCard::error(
            Platform::Codex,
            "无法访问 Codex 配置或认证文件".to_string(),
        );
    };

    match service.get_runtime_summary() {
        Ok(summary) => {
            let (fallback_name, fallback_profile) = current_profile_from_file(Platform::Codex);
            let profile = load_profile(Platform::Codex, summary.current_profile_name.as_deref())
                .or(fallback_profile);
            status_card_from_codex_summary(summary, profile.as_ref(), fallback_name.as_deref())
        }
        Err(error) => PlatformStatusCard::error(Platform::Codex, error.to_string()),
    }
}

fn load_profile(platform: Platform, profile_name: Option<&str>) -> Option<ProfileConfig> {
    let profile_name = profile_name?;
    let platform_config = create_platform(platform).ok()?;
    let profiles = platform_config.load_profiles().ok()?;
    profiles.get(profile_name).cloned()
}

fn current_profile_from_file(platform: Platform) -> (Option<String>, Option<ProfileConfig>) {
    let Ok(manager) = ConfigManager::for_platform(platform.short_name()) else {
        return (None, None);
    };
    let Ok(config) = manager.load_with_autofix() else {
        return (None, None);
    };

    let name = config.current_config.trim();
    if name.is_empty() {
        return (None, None);
    }

    let profile = config
        .sections
        .get(name)
        .map(ccr_config::section_to_profile);

    (Some(name.to_string()), profile)
}

fn status_card_from_claude_summary(
    summary: ClaudeRuntimeSummary,
    profile: Option<&ProfileConfig>,
) -> PlatformStatusCard {
    let (auth, auth_kind) = claude_auth_display(&summary);
    let health = claude_health(&summary);
    let note = claude_note(&summary);

    PlatformStatusCard {
        platform: Platform::Claude.short_name().to_string(),
        display_name: Platform::Claude.display_name().to_string(),
        profile: summary
            .current_profile_name
            .clone()
            .unwrap_or_else(|| "未绑定".to_string()),
        provider: profile
            .and_then(|profile| profile.provider.clone())
            .or(summary.current_profile_provider.clone()),
        base_url: profile.and_then(|profile| profile.base_url.clone()),
        model: profile.and_then(|profile| profile.model.clone()),
        small_fast_model: profile.and_then(|profile| profile.small_fast_model.clone()),
        auth,
        auth_kind,
        health,
        note,
    }
}

fn status_card_from_codex_summary(
    summary: CodexRuntimeSummary,
    profile: Option<&ProfileConfig>,
    fallback_profile_name: Option<&str>,
) -> PlatformStatusCard {
    let (auth, auth_kind) = codex_auth_display(&summary, profile);
    let health = codex_health(&summary);
    let used_profile_fallback =
        summary.current_profile_name.is_none() && fallback_profile_name.is_some();
    let note = codex_note(
        &summary,
        used_profile_fallback,
        profile_has_auth_token(profile),
    );

    PlatformStatusCard {
        platform: Platform::Codex.short_name().to_string(),
        display_name: Platform::Codex.display_name().to_string(),
        profile: summary
            .current_profile_name
            .clone()
            .or_else(|| fallback_profile_name.map(str::to_string))
            .unwrap_or_else(|| "未绑定".to_string()),
        provider: profile
            .and_then(|profile| profile.provider.clone())
            .or(summary.current_profile_provider.clone()),
        base_url: profile.and_then(|profile| profile.base_url.clone()),
        model: profile.and_then(|profile| profile.model.clone()),
        small_fast_model: profile.and_then(|profile| profile.small_fast_model.clone()),
        auth,
        auth_kind,
        health,
        note,
    }
}

impl PlatformStatusCard {
    fn error(platform: Platform, note: String) -> Self {
        Self {
            platform: platform.short_name().to_string(),
            display_name: platform.display_name().to_string(),
            profile: "未解析".to_string(),
            provider: None,
            base_url: None,
            model: None,
            small_fast_model: None,
            auth: "未解析".to_string(),
            auth_kind: StatusAuthKind::Unknown,
            health: StatusHealth::Error,
            note,
        }
    }
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

fn claude_auth_display(summary: &ClaudeRuntimeSummary) -> (String, StatusAuthKind) {
    match summary.mode {
        ClaudeRuntimeMode::ProfileOnly => match summary.current_profile_auth_mode {
            Some(ClaudeProfileAuthMode::ApiKey) => {
                let source = summary
                    .current_profile_auth_source
                    .as_deref()
                    .and_then(provider_suffix)
                    .unwrap_or("ANTHROPIC_AUTH_TOKEN");
                (
                    format!("第三方 API: {source}"),
                    StatusAuthKind::ThirdPartyApi,
                )
            }
            Some(ClaudeProfileAuthMode::Subscription) => {
                ("官方 Auth: 等待登录".to_string(), StatusAuthKind::Missing)
            }
            None => ("未解析".to_string(), StatusAuthKind::Unknown),
        },
        ClaudeRuntimeMode::ProfileWithAuth | ClaudeRuntimeMode::RuntimeOnly => {
            let name = summary
                .current_auth_name
                .as_deref()
                .or(summary.current_login_name.as_deref())
                .unwrap_or("已登录账号");
            (format!("官方 Auth: {name}"), StatusAuthKind::OfficialAuth)
        }
        ClaudeRuntimeMode::ProfilePendingAuth => (
            "官方 Auth: 需要 claude login".to_string(),
            StatusAuthKind::Missing,
        ),
        ClaudeRuntimeMode::Unresolved => ("未解析".to_string(), StatusAuthKind::Unknown),
    }
}

fn codex_auth_display(
    summary: &CodexRuntimeSummary,
    profile: Option<&ProfileConfig>,
) -> (String, StatusAuthKind) {
    let provider = summary
        .current_profile_provider
        .as_deref()
        .or_else(|| profile.and_then(|profile| profile.provider.as_deref()))
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match &summary.login_state {
        LoginState::LoggedInSaved(name) => {
            return (
                format!("官方 Auth: {name}"),
                codex_openai_kind(&summary.auth_state.intent),
            );
        }
        LoginState::LoggedInUnsaved => {
            return (
                "官方 Auth: 已登录未保存".to_string(),
                codex_openai_kind(&summary.auth_state.intent),
            );
        }
        LoginState::ApiKeyActive => {
            let source = provider.unwrap_or("OPENAI_API_KEY");
            return (
                format!("第三方 API: {source}"),
                StatusAuthKind::ThirdPartyApi,
            );
        }
        LoginState::ProviderKeyActive { env_key } => {
            return (
                format!("Provider Key: {env_key}"),
                StatusAuthKind::ProviderKey,
            );
        }
        LoginState::Unknown { type_name, .. } => {
            return (
                format!("未知认证状态: {type_name}"),
                StatusAuthKind::Unknown,
            );
        }
        LoginState::NotLoggedIn => {}
    }

    if profile_has_auth_token(profile) {
        return (
            format!("第三方 API: {}", provider.unwrap_or("OPENAI_API_KEY")),
            StatusAuthKind::ThirdPartyApi,
        );
    }

    match summary.current_profile_auth_mode {
        Some(CodexProfileAuthMode::OpenAiChatgpt) => (
            "官方 Auth: 需要 codex login".to_string(),
            StatusAuthKind::Missing,
        ),
        Some(CodexProfileAuthMode::OpenAiApiKey) => (
            format!("第三方 API: {}", provider.unwrap_or("OPENAI_API_KEY")),
            StatusAuthKind::ThirdPartyApi,
        ),
        Some(CodexProfileAuthMode::ProviderEnvKey) => {
            let source = summary
                .current_profile_auth_source
                .as_deref()
                .and_then(provider_suffix)
                .unwrap_or("provider env");
            (
                format!("Provider Key: {source}"),
                StatusAuthKind::ProviderKey,
            )
        }
        Some(CodexProfileAuthMode::NoAuth) => ("No Auth".to_string(), StatusAuthKind::NoAuth),
        None => ("未解析".to_string(), StatusAuthKind::Unknown),
    }
}

fn codex_openai_kind(intent: &AuthIntent) -> StatusAuthKind {
    match intent {
        AuthIntent::OpenAiAuth {
            method: OpenAiAuthMethod::Chatgpt,
        } => StatusAuthKind::OfficialAuth,
        AuthIntent::OpenAiAuth {
            method: OpenAiAuthMethod::Api,
        } => StatusAuthKind::ThirdPartyApi,
        AuthIntent::ProviderEnvKey { .. } => StatusAuthKind::ProviderKey,
        AuthIntent::NoAuth => StatusAuthKind::NoAuth,
    }
}

fn claude_health(summary: &ClaudeRuntimeSummary) -> StatusHealth {
    match summary.mode {
        ClaudeRuntimeMode::ProfileOnly | ClaudeRuntimeMode::ProfileWithAuth => StatusHealth::Ready,
        ClaudeRuntimeMode::RuntimeOnly => match summary.login_state {
            ClaudeLoginState::NotLoggedIn => StatusHealth::NeedsLogin,
            _ => StatusHealth::Ready,
        },
        ClaudeRuntimeMode::ProfilePendingAuth => StatusHealth::NeedsLogin,
        ClaudeRuntimeMode::Unresolved => StatusHealth::Invalid,
    }
}

fn codex_health(summary: &CodexRuntimeSummary) -> StatusHealth {
    if matches!(summary.auth_state.status, AuthStateStatus::Unsupported) {
        return StatusHealth::Unsupported;
    }

    match summary.mode {
        CodexRuntimeMode::ProfileOnly | CodexRuntimeMode::ProfileWithAuth => StatusHealth::Ready,
        CodexRuntimeMode::RuntimeOnly => match summary.auth_state.status {
            AuthStateStatus::Valid => StatusHealth::Ready,
            AuthStateStatus::Missing => StatusHealth::NeedsLogin,
            AuthStateStatus::Invalid => StatusHealth::Invalid,
            AuthStateStatus::Unsupported => StatusHealth::Unsupported,
        },
        CodexRuntimeMode::ProfilePendingAuth => StatusHealth::NeedsLogin,
        CodexRuntimeMode::Unresolved => StatusHealth::Invalid,
    }
}

fn claude_note(summary: &ClaudeRuntimeSummary) -> String {
    match summary.mode {
        ClaudeRuntimeMode::ProfileOnly => {
            "当前由 profile/API key 控制，不使用官方订阅凭据".to_string()
        }
        ClaudeRuntimeMode::ProfileWithAuth => "Profile 控制路由，官方 Auth 控制身份".to_string(),
        ClaudeRuntimeMode::ProfilePendingAuth => {
            "Profile 需要官方订阅凭据，但当前未检测到登录".to_string()
        }
        ClaudeRuntimeMode::RuntimeOnly => {
            "当前仅检测到官方 Auth 运行时，未绑定 profile".to_string()
        }
        ClaudeRuntimeMode::Unresolved => "无法解析当前 Claude profile 或认证状态".to_string(),
    }
}

fn codex_note(
    summary: &CodexRuntimeSummary,
    used_profile_fallback: bool,
    profile_has_auth_token: bool,
) -> String {
    if profile_has_auth_token {
        return "当前 profile 使用第三方 API key".to_string();
    }

    if used_profile_fallback {
        return "Profile 来自 CCR 当前配置；runtime/auth 已生效".to_string();
    }

    match summary.mode {
        CodexRuntimeMode::ProfileOnly => "当前由 profile/provider key 控制".to_string(),
        CodexRuntimeMode::ProfileWithAuth => "Profile 控制路由，Auth 控制身份".to_string(),
        CodexRuntimeMode::ProfilePendingAuth => {
            "Profile 需要 OpenAI Auth，但当前未检测到有效凭据".to_string()
        }
        CodexRuntimeMode::RuntimeOnly => {
            "当前仅检测到 runtime/auth 生效，未绑定 profile".to_string()
        }
        CodexRuntimeMode::Unresolved => "无法解析当前 Codex profile 或认证状态".to_string(),
    }
}

fn provider_suffix(source: &str) -> Option<&str> {
    source
        .strip_prefix("provider:")
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn profile_has_auth_token(profile: Option<&ProfileConfig>) -> bool {
    profile
        .and_then(|profile| profile.auth_token.as_deref())
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
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
        let overview = StatusOverview::load()?;
        println!("{}", serde_json::to_string_pretty(&overview)?);
        return Ok(());
    }

    if !verbose {
        let overview = StatusOverview::load()?;
        overview.print_human();
        return Ok(());
    }

    current_command_verbose().await
}

async fn current_command_verbose() -> Result<()> {
    ColorOutput::title("当前配置状态");

    // 🔍 加载平台配置
    let platform_config_mgr = PlatformConfigManager::with_default()?;
    let unified_config = platform_config_mgr.load()?;

    println!();

    // === 第零部分：平台信息 ===
    ColorOutput::step("🔄 平台信息");
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
        Cell::new("当前平台").fg(TableColor::Yellow),
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
                    current_section.auth_token = Some(token.to_string());
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
            Cell::new(ColorOutput::mask_sensitive(auth_token)).fg(TableColor::DarkGrey),
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
            Platform::Gemini => "Gemini",
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    fn codex_auth_state(intent: AuthIntent, status: AuthStateStatus) -> AuthState {
        AuthState {
            intent,
            store: crate::models::CredentialStoreKind::File,
            status,
            reason: "test".to_string(),
        }
    }

    fn codex_summary(
        mode: CodexRuntimeMode,
        auth_mode: Option<CodexProfileAuthMode>,
        login_state: LoginState,
        auth_state: AuthState,
    ) -> CodexRuntimeSummary {
        CodexRuntimeSummary {
            mode,
            current_profile_name: Some("codex-work".to_string()),
            current_profile_provider: Some("9m8m".to_string()),
            current_profile_auth_mode: auth_mode,
            current_profile_auth_source: Some("openai_api_key".to_string()),
            current_auth_name: Some("work".to_string()),
            login_state,
            auth_state,
        }
    }

    fn claude_summary(
        mode: ClaudeRuntimeMode,
        auth_mode: Option<ClaudeProfileAuthMode>,
        login_state: ClaudeLoginState,
    ) -> ClaudeRuntimeSummary {
        ClaudeRuntimeSummary {
            mode,
            current_profile_name: Some("claude-work".to_string()),
            current_profile_provider: Some("anthropic".to_string()),
            current_profile_auth_mode: auth_mode,
            current_profile_auth_source: Some("provider:anthropic".to_string()),
            current_login_name: Some("official-work".to_string()),
            official_login_state: login_state.clone(),
            current_auth_name: Some("official-work".to_string()),
            login_state,
        }
    }

    #[test]
    fn codex_chatgpt_login_renders_official_auth() {
        let summary = codex_summary(
            CodexRuntimeMode::ProfileWithAuth,
            Some(CodexProfileAuthMode::OpenAiChatgpt),
            LoginState::LoggedInSaved("work".to_string()),
            codex_auth_state(
                AuthIntent::OpenAiAuth {
                    method: OpenAiAuthMethod::Chatgpt,
                },
                AuthStateStatus::Valid,
            ),
        );

        let (label, kind) = codex_auth_display(&summary, None);

        assert_eq!(label, "官方 Auth: work");
        assert_eq!(kind, StatusAuthKind::OfficialAuth);
    }

    #[test]
    fn codex_api_key_renders_third_party_api() {
        let summary = codex_summary(
            CodexRuntimeMode::ProfileWithAuth,
            Some(CodexProfileAuthMode::OpenAiApiKey),
            LoginState::ApiKeyActive,
            codex_auth_state(
                AuthIntent::OpenAiAuth {
                    method: OpenAiAuthMethod::Api,
                },
                AuthStateStatus::Valid,
            ),
        );

        let (label, kind) = codex_auth_display(&summary, None);

        assert_eq!(label, "第三方 API: 9m8m");
        assert_eq!(kind, StatusAuthKind::ThirdPartyApi);
    }

    #[test]
    fn codex_provider_key_renders_provider_key() {
        let mut summary = codex_summary(
            CodexRuntimeMode::ProfileOnly,
            Some(CodexProfileAuthMode::ProviderEnvKey),
            LoginState::ProviderKeyActive {
                env_key: "MISTRAL_API_KEY".to_string(),
            },
            codex_auth_state(
                AuthIntent::ProviderEnvKey {
                    env_key: "MISTRAL_API_KEY".to_string(),
                },
                AuthStateStatus::Valid,
            ),
        );
        summary.current_profile_auth_source = Some("provider:MISTRAL_API_KEY".to_string());

        let (label, kind) = codex_auth_display(&summary, None);

        assert_eq!(label, "Provider Key: MISTRAL_API_KEY");
        assert_eq!(kind, StatusAuthKind::ProviderKey);
    }

    #[test]
    fn codex_missing_chatgpt_auth_renders_login_hint() {
        let summary = codex_summary(
            CodexRuntimeMode::ProfilePendingAuth,
            Some(CodexProfileAuthMode::OpenAiChatgpt),
            LoginState::NotLoggedIn,
            codex_auth_state(
                AuthIntent::OpenAiAuth {
                    method: OpenAiAuthMethod::Chatgpt,
                },
                AuthStateStatus::Missing,
            ),
        );

        let (label, kind) = codex_auth_display(&summary, None);

        assert_eq!(label, "官方 Auth: 需要 codex login");
        assert_eq!(kind, StatusAuthKind::Missing);
    }

    #[test]
    fn claude_subscription_login_renders_official_auth() {
        let summary = claude_summary(
            ClaudeRuntimeMode::ProfileWithAuth,
            Some(ClaudeProfileAuthMode::Subscription),
            ClaudeLoginState::LoggedInSaved {
                account_name: "official-work".to_string(),
            },
        );

        let (label, kind) = claude_auth_display(&summary);

        assert_eq!(label, "官方 Auth: official-work");
        assert_eq!(kind, StatusAuthKind::OfficialAuth);
    }

    #[test]
    fn claude_api_key_profile_renders_third_party_api() {
        let summary = claude_summary(
            ClaudeRuntimeMode::ProfileOnly,
            Some(ClaudeProfileAuthMode::ApiKey),
            ClaudeLoginState::ApiKeyActive,
        );

        let (label, kind) = claude_auth_display(&summary);

        assert_eq!(label, "第三方 API: anthropic");
        assert_eq!(kind, StatusAuthKind::ThirdPartyApi);
    }

    #[test]
    fn json_overview_contains_both_platforms_without_tokens() {
        let overview = StatusOverview {
            current_platform: "codex".to_string(),
            claude: PlatformStatusCard {
                platform: "claude".to_string(),
                display_name: "Claude Code".to_string(),
                profile: "claude-work".to_string(),
                provider: Some("anthropic".to_string()),
                base_url: None,
                model: Some("claude-sonnet".to_string()),
                small_fast_model: None,
                auth: "官方 Auth: work".to_string(),
                auth_kind: StatusAuthKind::OfficialAuth,
                health: StatusHealth::Ready,
                note: "ok".to_string(),
            },
            codex: PlatformStatusCard {
                platform: "codex".to_string(),
                display_name: "Codex".to_string(),
                profile: "codex-work".to_string(),
                provider: Some("9m8m".to_string()),
                base_url: Some("https://9m8m.com".to_string()),
                model: Some("gpt-5.4".to_string()),
                small_fast_model: None,
                auth: "第三方 API: OPENAI_API_KEY".to_string(),
                auth_kind: StatusAuthKind::ThirdPartyApi,
                health: StatusHealth::Ready,
                note: "ok".to_string(),
            },
        };

        let value = serde_json::to_value(&overview).unwrap();

        assert_eq!(value["claude"]["platform"], json!("claude"));
        assert_eq!(value["codex"]["platform"], json!("codex"));
        let serialized = serde_json::to_string(&value).unwrap();
        assert!(!serialized.contains("sk-"));
        assert!(!serialized.contains("auth_token"));
    }
}
