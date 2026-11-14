//! 平台 profile 非交互式变更命令

#![allow(clippy::unused_async)]

use crate::application::profile_create::{
    CreateProfileRequest, CreateSecret, ProfileFieldValue, create_profile,
};
use crate::application::profile_edit::{
    self, EditProfileRequest, FieldEdit, SecretEdit, ensure_set_field_allowed,
};
use crate::cli::subcommands::grok::GrokProfileCreateActionArgs;
use crate::cli::subcommands::profile_args::ProfileCreateActionArgs;
use crate::commands::platform::profile_edit_cmd::{
    parse_set_argument, read_stdin_secret, reject_inline_secret,
};
use crate::models::{ApplyPolicy, Platform, PlatformConfig, ProfileConfig};
use crate::platforms::create_platform;
use ccr_core::Secret;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use serde::Serialize;
use serde_json::Value;
use std::str::FromStr;

#[derive(Debug, Serialize)]
struct PlatformProfileMutationOutput {
    ok: bool,
    platform: String,
    name: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_profile: Option<String>,
}

#[derive(Debug, Serialize)]
struct PlatformProfileCreateOutput {
    ok: bool,
    platform: String,
    name: String,
    message: String,
    enabled: bool,
    current_profile: Option<String>,
    revision: String,
    set_fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PlatformProfileCreateArgs {
    pub platform_name: String,
    pub name: String,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub auth_token: Option<String>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub small_fast_model: Option<String>,
    pub provider: Option<String>,
    pub provider_type: Option<String>,
    pub account: Option<String>,
    pub tags: Vec<String>,
    pub auth_mode: Option<String>,
    pub api_backend: Option<String>,
    pub env_key: Option<String>,
    pub context_window: Option<u64>,
    pub supports_backend_search: Option<bool>,
    pub reasoning_effort: Option<String>,
    pub disabled: bool,
    pub json: bool,
}

#[derive(Debug, Clone)]
struct ProfileCreateCommand {
    base: PlatformProfileCreateArgs,
    auth_token_stdin: bool,
    api_key_stdin: bool,
    expected_revision: Option<String>,
    set: Vec<String>,
}

impl ProfileCreateCommand {
    fn from_platform_args(base: PlatformProfileCreateArgs) -> Self {
        Self {
            base,
            auth_token_stdin: false,
            api_key_stdin: false,
            expected_revision: None,
            set: Vec::new(),
        }
    }

    fn from_common(platform_name: &str, args: ProfileCreateActionArgs) -> Self {
        Self {
            base: PlatformProfileCreateArgs {
                platform_name: platform_name.to_string(),
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
            },
            auth_token_stdin: args.auth_token_stdin,
            api_key_stdin: false,
            expected_revision: args.expected_revision,
            set: args.set,
        }
    }

    fn from_grok(args: GrokProfileCreateActionArgs) -> Self {
        Self {
            base: PlatformProfileCreateArgs {
                platform_name: "grok".to_string(),
                name: args.name,
                description: args.description,
                base_url: args.base_url,
                auth_token: None,
                api_key: args.api_key,
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
            },
            auth_token_stdin: false,
            api_key_stdin: args.api_key_stdin,
            expected_revision: args.expected_revision,
            set: args.set,
        }
    }
}

fn parse_platform(platform_name: &str) -> Result<Platform> {
    let platform = Platform::from_str(platform_name)
        .map_err(|_| CcrError::PlatformNotFound(platform_name.to_string()))?;

    if Platform::auth_profile_supported().contains(&platform) {
        Ok(platform)
    } else {
        Err(CcrError::PlatformNotSupported(format!(
            "{} auth/profile commands support only claude, codex and grok",
            platform
        )))
    }
}

fn load_existing_profile(platform_impl: &dyn PlatformConfig, name: &str) -> Result<ProfileConfig> {
    let mut profiles = platform_impl.load_profiles()?;
    profiles
        .shift_remove(name)
        .ok_or_else(|| CcrError::ProfileNotFound(name.to_string()))
}

fn print_output(output: &PlatformProfileMutationOutput, json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string(output).map_err(CcrError::JsonError)?
        );
        return Ok(());
    }

    ColorOutput::success(&output.message);
    Ok(())
}

pub(crate) fn catalog_revision(platform: Platform) -> Result<Option<String>> {
    let paths = crate::models::PlatformPaths::new(platform)?;
    match ccr_config::ConfigManager::new(&paths.profiles_file).snapshot() {
        Ok(snapshot) => Ok(Some(snapshot.version)),
        Err(CcrError::ConfigMissing(_)) => Ok(None),
        Err(error) => Err(error),
    }
}

pub async fn platform_profile_create_command(args: PlatformProfileCreateArgs) -> Result<()> {
    run_profile_create_command(ProfileCreateCommand::from_platform_args(args)).await
}

pub(crate) async fn create_from_common_args(
    platform_name: &str,
    args: ProfileCreateActionArgs,
) -> Result<()> {
    run_profile_create_command(ProfileCreateCommand::from_common(platform_name, args)).await
}

pub(crate) async fn create_from_grok_args(args: GrokProfileCreateActionArgs) -> Result<()> {
    run_profile_create_command(ProfileCreateCommand::from_grok(args)).await
}

fn build_create_request(
    command: ProfileCreateCommand,
    platform: Platform,
    secret: CreateSecret,
) -> Result<(CreateProfileRequest, bool)> {
    let PlatformProfileCreateArgs {
        platform_name: _,
        name,
        description,
        base_url,
        auth_token: _,
        api_key: _,
        model,
        small_fast_model,
        provider,
        provider_type,
        account,
        tags,
        auth_mode,
        api_backend,
        env_key,
        context_window,
        supports_backend_search,
        reasoning_effort,
        disabled,
        json,
    } = command.base;

    let mut fields = Vec::new();
    for (field, value) in [
        ("description", description),
        ("base_url", base_url),
        ("model", model),
        ("small_fast_model", small_fast_model),
        ("provider", provider),
        ("account", account),
    ] {
        if let Some(value) = value.filter(|text| !text.trim().is_empty()) {
            push_create_field(&mut fields, field, Value::String(value))?;
        }
    }
    for (field, value) in [
        ("provider_type", provider_type),
        ("auth_mode", auth_mode),
        ("api_backend", api_backend),
        ("reasoning_effort", reasoning_effort),
    ] {
        if let Some(value) = value {
            push_create_field(&mut fields, field, Value::String(value))?;
        }
    }
    if platform != Platform::Grok
        && let Some(env_key) = env_key
    {
        push_create_field(&mut fields, "env_key", Value::String(env_key))?;
    }
    if !tags.is_empty() {
        push_create_field(
            &mut fields,
            "tags",
            Value::Array(tags.into_iter().map(Value::String).collect()),
        )?;
    }
    if let Some(context_window) = context_window {
        push_create_field(
            &mut fields,
            "context_window",
            Value::Number(context_window.into()),
        )?;
    }
    if let Some(supports_backend_search) = supports_backend_search {
        push_create_field(
            &mut fields,
            "supports_backend_search",
            Value::Bool(supports_backend_search),
        )?;
    }
    if disabled {
        push_create_field(&mut fields, "enabled", Value::Bool(false))?;
    }
    for raw in &command.set {
        let (field, value) = parse_set_argument(raw)?;
        reject_inline_secret(&field)?;
        if platform == Platform::Grok && field == "env_key" {
            return Err(CcrError::ValidationError(
                "Grok 创建 Profile 请使用 --env-key 指定环境变量名，不要使用 --set env_key=..."
                    .into(),
            ));
        }
        push_create_field(&mut fields, &field, Value::String(value))?;
    }

    Ok((
        CreateProfileRequest {
            platform,
            name,
            expected_revision: command.expected_revision,
            fields,
            secret,
        },
        json,
    ))
}

async fn run_profile_create_command(command: ProfileCreateCommand) -> Result<()> {
    let platform = parse_platform(&command.base.platform_name)?;
    let secret = resolve_create_secret(&command, platform)?;
    let (request, json) = build_create_request(command, platform, secret)?;
    let outcome = create_profile(request)?;

    let platform_short = outcome.platform.short_name().to_string();
    let message = format!("已创建 {} 平台 Profile '{}'", platform_short, outcome.name);
    let output = PlatformProfileCreateOutput {
        ok: true,
        platform: platform_short,
        name: outcome.name,
        message: message.clone(),
        enabled: outcome.enabled,
        current_profile: outcome.current_profile,
        revision: outcome.revision,
        set_fields: outcome.set_fields,
    };
    if json {
        println!(
            "{}",
            serde_json::to_string(&output).map_err(CcrError::JsonError)?
        );
    } else {
        ColorOutput::success(&message);
    }
    Ok(())
}

fn push_create_field(fields: &mut Vec<ProfileFieldValue>, field: &str, value: Value) -> Result<()> {
    if fields.iter().any(|existing| existing.field == field) {
        return Err(CcrError::ValidationError(format!("字段 '{field}' 重复")));
    }
    fields.push(ProfileFieldValue {
        field: field.to_string(),
        value,
    });
    Ok(())
}

fn resolve_create_secret(
    command: &ProfileCreateCommand,
    platform: Platform,
) -> Result<CreateSecret> {
    let auth_token = &command.base.auth_token;
    let api_key = &command.base.api_key;
    let env_key = &command.base.env_key;

    if command.auth_token_stdin || command.api_key_stdin {
        if auth_token.is_some()
            || api_key.is_some()
            || (platform == Platform::Grok && env_key.is_some())
        {
            return Err(CcrError::ValidationError(
                "标准输入密钥参数不能与明文密钥或 --env-key 同时使用".into(),
            ));
        }
        return Ok(CreateSecret::Secret(read_stdin_secret()?));
    }
    if platform == Platform::Grok
        && env_key.is_some()
        && (auth_token.is_some() || api_key.is_some())
    {
        return Err(CcrError::ValidationError(
            "Grok 的 api_key 与 env_key 互斥，不能同时指定".into(),
        ));
    }
    if let Some(raw_token) = auth_token {
        ColorOutput::deprecation_warning(
            "命令行参数 --auth-token 已软弃用（可能暴露在进程列表或命令历史中），请改用 --auth-token-stdin",
        );
        return Ok(CreateSecret::Secret(Secret::new(raw_token.clone())));
    }
    if let Some(raw_key) = api_key {
        ColorOutput::deprecation_warning(
            "命令行参数 --api-key / --auth-token 已软弃用（可能暴露在进程列表或命令历史中），请改用 --api-key-stdin",
        );
        return Ok(CreateSecret::Secret(Secret::new(raw_key.clone())));
    }
    if platform == Platform::Grok
        && let Some(env_name) = env_key
    {
        return Ok(CreateSecret::UseEnvKey(env_name.clone()));
    }
    Ok(CreateSecret::None)
}

pub async fn platform_profile_set_field_command(
    platform_name: &str,
    name: &str,
    field: &str,
    value: Option<String>,
    value_json: Option<String>,
    clear: bool,
    json: bool,
) -> Result<()> {
    set_field_command_with_revision(
        platform_name,
        name,
        field,
        value,
        value_json,
        clear,
        json,
        None,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn set_field_command_with_revision(
    platform_name: &str,
    name: &str,
    field: &str,
    value: Option<String>,
    value_json: Option<String>,
    clear: bool,
    json: bool,
    expected_revision: Option<String>,
) -> Result<()> {
    if !clear && value.is_none() && value_json.is_none() {
        return Err(CcrError::ValidationError(
            "必须提供 --value / --value-json 或 --clear".into(),
        ));
    }

    let platform = parse_platform(platform_name)?;
    ensure_set_field_allowed(platform, field)?;
    let outcome = profile_edit::edit_profile(EditProfileRequest {
        platform,
        name: name.to_string(),
        rename: None,
        expected_revision,
        fields: vec![FieldEdit::Legacy {
            field: field.to_string(),
            value,
            value_json,
            clear,
        }],
        secret: SecretEdit::Keep,
        apply: ApplyPolicy::IfCurrent,
    })?;
    if !outcome.is_success() {
        return Err(CcrError::ConfigError(outcome.failure_message().into()));
    }

    let platform_impl = create_platform(platform)?;
    let profile = load_existing_profile(platform_impl.as_ref(), name)?;
    let output = PlatformProfileMutationOutput {
        ok: true,
        platform: platform_name.to_string(),
        name: name.to_string(),
        message: format!("已更新 {}.{}", name, field),
        enabled: Some(profile.is_enabled()),
        current_profile: platform_impl.get_current_profile()?,
    };
    print_output(&output, json)
}

pub async fn platform_profile_enable_command(
    platform_name: &str,
    name: &str,
    json: bool,
) -> Result<()> {
    let platform = parse_platform(platform_name)?;
    if matches!(
        platform,
        Platform::Claude | Platform::Codex | Platform::Grok
    ) {
        let context = crate::application::ProfileLifecycleContext::new(platform)?;
        let current_profile = context.run(|| {
            let _lock = context.operation_lock()?;
            context.with_instance(|instance| {
                let mut profile = load_existing_profile(instance, name)?;
                profile.enable();
                if let Err(err) = instance.validate_profile(&profile) {
                    return Err(crate::application::enrich_missing_secret_enable_error(
                        platform, name, &profile, instance, err,
                    ));
                }
                context.manager().mutate(|config| {
                    config.get_section_mut(name)?.enabled = Some(true);
                    Ok(())
                })?;
                instance.get_current_profile()
            })
        })?;
        let output = PlatformProfileMutationOutput {
            ok: true,
            platform: platform_name.to_string(),
            name: name.to_string(),
            message: format!("已启用 {} 平台 Profile '{}'", platform_name, name),
            enabled: Some(true),
            current_profile,
        };
        return print_output(&output, json);
    }

    let platform_impl = create_platform(platform)?;
    let mut profile = load_existing_profile(platform_impl.as_ref(), name)?;
    profile.enable();
    platform_impl.save_profile(name, &profile)?;

    let output = PlatformProfileMutationOutput {
        ok: true,
        platform: platform_name.to_string(),
        name: name.to_string(),
        message: format!("已启用 {} 平台 Profile '{}'", platform_name, name),
        enabled: Some(true),
        current_profile: platform_impl.get_current_profile()?,
    };
    print_output(&output, json)
}

pub async fn platform_profile_disable_command(
    platform_name: &str,
    name: &str,
    force: bool,
    json: bool,
) -> Result<()> {
    let platform = parse_platform(platform_name)?;
    let platform_impl = create_platform(platform)?;
    let current_profile = platform_impl.get_current_profile()?;
    if current_profile.as_deref() == Some(name) && !force {
        return Err(CcrError::ValidationError(format!(
            "当前 Profile '{}' 正在使用，禁用请添加 --force",
            name
        )));
    }

    let mut profile = load_existing_profile(platform_impl.as_ref(), name)?;
    profile.disable();
    platform_impl.save_profile(name, &profile)?;

    let output = PlatformProfileMutationOutput {
        ok: true,
        platform: platform_name.to_string(),
        name: name.to_string(),
        message: format!("已禁用 {} 平台 Profile '{}'", platform_name, name),
        enabled: Some(false),
        current_profile,
    };
    print_output(&output, json)
}

pub async fn platform_profile_delete_command(
    platform_name: &str,
    name: &str,
    force: bool,
    json: bool,
) -> Result<()> {
    let platform = parse_platform(platform_name)?;
    let current_profile =
        crate::application::profile_lifecycle::ProfileLifecycleContext::new(platform)?
            .delete(name, force)?;

    let output = PlatformProfileMutationOutput {
        ok: true,
        platform: platform_name.to_string(),
        name: name.to_string(),
        message: format!("已删除 {} 平台 Profile '{}'", platform_name, name),
        enabled: None,
        current_profile,
    };
    print_output(&output, json)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::ProfileConfig;

    #[test]
    fn test_parse_platform_rejects_non_auth_profile_platforms() {
        for platform_name in ["gemini", "qwen", "droid"] {
            let err = parse_platform(platform_name).unwrap_err();
            assert!(matches!(err, CcrError::PlatformNotSupported(_)));
            assert!(err.to_string().contains(platform_name));
            assert!(err.to_string().contains("claude, codex and grok"));
        }
    }

    #[test]
    fn grok_create_rejects_set_env_key_without_echoing_value() {
        let mut command = ProfileCreateCommand::from_platform_args(PlatformProfileCreateArgs {
            platform_name: "grok".into(),
            name: "relay".into(),
            description: None,
            base_url: Some("https://api.x.ai/v1".into()),
            auth_token: None,
            api_key: None,
            model: Some("grok-4".into()),
            small_fast_model: None,
            provider: None,
            provider_type: None,
            account: None,
            tags: Vec::new(),
            auth_mode: None,
            api_backend: None,
            env_key: None,
            context_window: None,
            supports_backend_search: None,
            reasoning_effort: None,
            disabled: false,
            json: false,
        });
        command.set.push("env_key=XAI_SENTINEL_ENV_999".into());
        let err = build_create_request(command, Platform::Grok, CreateSecret::None).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("--env-key"), "{msg}");
        assert!(!msg.contains("XAI_SENTINEL_ENV_999"), "{msg}");
    }

    #[tokio::test]
    async fn set_field_rejects_stale_revision_without_writing() {
        let _home = crate::test_support::TestHome::new();
        let platform = create_platform(Platform::Claude).unwrap();
        let mut profile = ProfileConfig::new().with_model("model-old".into());
        profile.description = Some("before".into());
        platform.save_profile("work", &profile).unwrap();
        let paths = crate::models::PlatformPaths::new(Platform::Claude).unwrap();
        let manager = ccr_config::ConfigManager::new(&paths.profiles_file);
        let revision = manager.snapshot().unwrap().version;

        set_field_command_with_revision(
            "claude",
            "work",
            "description",
            Some("after".into()),
            None,
            false,
            false,
            None,
        )
        .await
        .unwrap();

        let error = set_field_command_with_revision(
            "claude",
            "work",
            "description",
            Some("stale".into()),
            None,
            false,
            false,
            Some(revision),
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("配置版本冲突"));
        let stored = create_platform(Platform::Claude)
            .unwrap()
            .load_profiles()
            .unwrap();
        assert_eq!(stored["work"].description.as_deref(), Some("after"));
    }
}
