//! 平台 profile 非交互式变更命令

#![allow(clippy::unused_async)]

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::models::{Platform, PlatformConfig, ProfileConfig};
use crate::platforms::create_platform;
use serde::Serialize;
use std::str::FromStr;

const DEFAULT_EDITABLE_FIELDS: &[&str] = &[
    "description",
    "base_url",
    "auth_token",
    "model",
    "small_fast_model",
    "provider",
    "provider_type",
    "account",
    "tags",
];

const CODEX_EDITABLE_FIELDS: &[&str] = &[
    "description",
    "model",
    "small_fast_model",
    "provider",
    "provider_type",
    "account",
    "tags",
];

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

fn parse_platform(platform_name: &str) -> Result<Platform> {
    Platform::from_str(platform_name)
        .map_err(|_| CcrError::PlatformNotFound(platform_name.to_string()))
}

fn editable_fields(platform: Platform) -> &'static [&'static str] {
    match platform {
        Platform::Codex => CODEX_EDITABLE_FIELDS,
        _ => DEFAULT_EDITABLE_FIELDS,
    }
}

fn ensure_field_allowed(platform: Platform, field: &str) -> Result<()> {
    if editable_fields(platform).contains(&field) {
        Ok(())
    } else {
        Err(CcrError::ValidationError(format!(
            "平台 '{}' 不允许编辑字段 '{}'",
            platform,
            field
        )))
    }
}

fn load_existing_profile(
    platform_impl: &dyn PlatformConfig,
    name: &str,
) -> Result<ProfileConfig> {
    let mut profiles = platform_impl.load_profiles()?;
    profiles
        .shift_remove(name)
        .ok_or_else(|| CcrError::ProfileNotFound(name.to_string()))
}

fn print_output(output: &PlatformProfileMutationOutput, json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string(output).map_err(|e| CcrError::JsonError(e))?
        );
        return Ok(());
    }

    ColorOutput::success(&output.message);
    Ok(())
}

fn normalize_tags(tags: Vec<String>) -> Option<Vec<String>> {
    let normalized = tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn update_profile_field(
    profile: &mut ProfileConfig,
    field: &str,
    value: Option<String>,
    value_json: Option<String>,
    clear: bool,
) -> Result<()> {
    match field {
        "description" => profile.description = if clear { None } else { value },
        "base_url" => profile.base_url = if clear { None } else { value },
        "auth_token" => profile.auth_token = if clear { None } else { value },
        "model" => profile.model = if clear { None } else { value },
        "small_fast_model" => profile.small_fast_model = if clear { None } else { value },
        "provider" => profile.provider = if clear { None } else { value },
        "provider_type" => profile.provider_type = if clear { None } else { value },
        "account" => profile.account = if clear { None } else { value },
        "tags" => {
            if clear {
                profile.tags = None;
            } else if let Some(raw_json) = value_json {
                let tags: Vec<String> = serde_json::from_str(&raw_json).map_err(|e| {
                    CcrError::ValidationError(format!("tags 的 value-json 不是合法 JSON 数组: {}", e))
                })?;
                profile.tags = normalize_tags(tags);
            } else {
                profile.tags = normalize_tags(
                    value
                        .unwrap_or_default()
                        .split(',')
                        .map(str::to_string)
                        .collect(),
                );
            }
        }
        _ => {
            return Err(CcrError::ValidationError(format!(
                "不支持的字段 '{}'",
                field
            )));
        }
    }

    Ok(())
}

pub async fn platform_profile_create_command(
    platform_name: &str,
    name: &str,
    description: Option<String>,
    base_url: Option<String>,
    auth_token: Option<String>,
    model: Option<String>,
    small_fast_model: Option<String>,
    provider: Option<String>,
    provider_type: Option<String>,
    account: Option<String>,
    tags: Vec<String>,
    disabled: bool,
    json: bool,
) -> Result<()> {
    let platform = parse_platform(platform_name)?;
    if !matches!(platform, Platform::Claude | Platform::Codex) {
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
    profile.auth_token = auth_token.filter(|value| !value.trim().is_empty());
    profile.model = model.filter(|value| !value.trim().is_empty());
    profile.small_fast_model = small_fast_model.filter(|value| !value.trim().is_empty());
    profile.provider = provider.filter(|value| !value.trim().is_empty());
    profile.provider_type = provider_type.filter(|value| !value.trim().is_empty());
    profile.account = account.filter(|value| !value.trim().is_empty());
    profile.tags = normalize_tags(tags);
    profile.enabled = Some(!disabled);

    platform_impl.save_profile(profile_name, &profile)?;

    let output = PlatformProfileMutationOutput {
        ok: true,
        platform: platform_name.to_string(),
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
    if !clear && value.is_none() && value_json.is_none() {
        return Err(CcrError::ValidationError(
            "必须提供 --value / --value-json 或 --clear".into(),
        ));
    }

    let platform = parse_platform(platform_name)?;
    ensure_field_allowed(platform, field)?;
    let platform_impl = create_platform(platform)?;
    let mut profile = load_existing_profile(platform_impl.as_ref(), name)?;
    update_profile_field(&mut profile, field, value, value_json, clear)?;
    platform_impl.save_profile(name, &profile)?;

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

    #[test]
    fn test_update_profile_field_supports_tags_json() {
        let mut profile = ProfileConfig::new();
        update_profile_field(
            &mut profile,
            "tags",
            None,
            Some(r#"[" work ","team",""]"#.to_string()),
            false,
        )
        .unwrap();

        assert_eq!(
            profile.tags,
            Some(vec!["work".to_string(), "team".to_string()])
        );
    }

    #[test]
    fn test_ensure_field_allowed_rejects_codex_base_url() {
        let err = ensure_field_allowed(Platform::Codex, "base_url").unwrap_err();
        assert!(err.to_string().contains("不允许编辑字段"));
    }
}
