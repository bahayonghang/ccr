//! 平台 profile 非交互式变更命令

#![allow(clippy::unused_async)]

use crate::application::profile_edit::{
    self, EditProfileRequest, FieldEdit, SecretEdit, ensure_set_field_allowed, normalize_tags,
};
use crate::models::{ApplyPolicy, Platform, PlatformConfig, ProfileConfig};
use crate::platforms::create_platform;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use serde::Serialize;
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

pub async fn platform_profile_create_command(args: PlatformProfileCreateArgs) -> Result<()> {
    let PlatformProfileCreateArgs {
        platform_name,
        name,
        description,
        base_url,
        auth_token,
        api_key,
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
    } = args;

    let platform = parse_platform(&platform_name)?;
    if !matches!(
        platform,
        Platform::Claude | Platform::Codex | Platform::Grok
    ) {
        return Err(CcrError::PlatformNotSupported(format!(
            "{} 目前不支持通过 CLI 创建平台 Profile",
            platform
        )));
    }

    let profile_name = name.trim();
    if profile_name.is_empty() {
        return Err(CcrError::ValidationError("Profile 名称不能为空".into()));
    }

    let platform_impl = create_platform(platform)?;
    let existing = platform_impl.load_profiles()?;
    if existing.contains_key(profile_name) {
        return Err(CcrError::ResourceAlreadyExists(format!(
            "Profile '{}' 已存在",
            profile_name
        )));
    }

    let mut profile = ProfileConfig::new();
    profile.description = description.filter(|value| !value.trim().is_empty());
    profile.base_url = base_url.filter(|value| !value.trim().is_empty());
    profile.auth_token = auth_token
        .filter(|value| !value.trim().is_empty())
        .map(ccr_core::Secret::new);
    profile.model = model.filter(|value| !value.trim().is_empty());
    profile.small_fast_model = small_fast_model.filter(|value| !value.trim().is_empty());
    profile.provider = provider.filter(|value| !value.trim().is_empty());
    profile.provider_type = provider_type.filter(|value| !value.trim().is_empty());
    profile.account = account.filter(|value| !value.trim().is_empty());
    profile.tags = normalize_tags(tags);
    profile.enabled = Some(!disabled);
    if let Some(auth_mode) = auth_mode.filter(|value| !value.trim().is_empty()) {
        profile.platform_data.insert(
            crate::platforms::ClaudePlatform::AUTH_MODE_FIELD.to_string(),
            serde_json::Value::String(auth_mode.trim().to_string()),
        );
    }
    for (key, value) in [
        ("api_backend", api_backend.map(serde_json::Value::String)),
        ("api_key", api_key.map(serde_json::Value::String)),
        ("env_key", env_key.map(serde_json::Value::String)),
        (
            "context_window",
            context_window.map(|value| serde_json::Value::Number(value.into())),
        ),
        (
            "supports_backend_search",
            supports_backend_search.map(serde_json::Value::Bool),
        ),
        (
            "reasoning_effort",
            reasoning_effort
                .map(|value| crate::platforms::GrokPlatform::normalize_reasoning_effort(&value))
                .transpose()?
                .map(serde_json::Value::String),
        ),
    ] {
        if let Some(value) = value {
            profile.platform_data.insert(key.to_string(), value);
        }
    }

    platform_impl.save_profile(profile_name, &profile)?;

    let output = PlatformProfileMutationOutput {
        ok: true,
        platform: platform_name.clone(),
        name: profile_name.to_string(),
        message: format!("已创建 {} 平台 Profile '{}'", platform_name, profile_name),
        enabled: Some(profile.is_enabled()),
        current_profile: platform_impl.get_current_profile()?,
    };
    print_output(&output, json)
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
    _force: bool,
    json: bool,
) -> Result<()> {
    let platform = parse_platform(platform_name)?;
    let platform_impl = create_platform(platform)?;
    platform_impl.delete_profile(name)?;

    let output = PlatformProfileMutationOutput {
        ok: true,
        platform: platform_name.to_string(),
        name: name.to_string(),
        message: format!("已删除 {} 平台 Profile '{}'", platform_name, name),
        enabled: None,
        current_profile: platform_impl.get_current_profile()?,
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
