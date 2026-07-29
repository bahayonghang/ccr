//! 平台 profile 非交互式变更命令

#![allow(clippy::unused_async)]

use crate::models::{Platform, PlatformConfig, ProfileConfig};
use crate::platforms::create_platform;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
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

fn editable_fields(platform: Platform) -> &'static [&'static str] {
    match platform {
        Platform::Codex => ccr_codex::CodexPlatform::editable_fields(),
        Platform::Claude => crate::platforms::ClaudePlatform::editable_fields(),
        Platform::Grok => crate::platforms::GrokPlatform::editable_fields(),
        _ => DEFAULT_EDITABLE_FIELDS,
    }
}

fn ensure_field_allowed(platform: Platform, field: &str) -> Result<()> {
    if editable_fields(platform).contains(&field) {
        Ok(())
    } else {
        Err(CcrError::ValidationError(format!(
            "平台 '{}' 不允许编辑字段 '{}'",
            platform, field
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
        "auth_token" => {
            profile.auth_token = if clear {
                None
            } else {
                value.map(ccr_core::Secret::new)
            }
        }
        "api_key" => {
            if clear {
                profile.platform_data.shift_remove("api_key");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError("api_key 需要非空字符串".into()));
                }
                let api_key = value
                    .ok_or_else(|| CcrError::ValidationError("api_key 需要非空字符串".into()))?
                    .trim()
                    .to_string();
                if api_key.is_empty() {
                    return Err(CcrError::ValidationError("api_key 需要非空字符串".into()));
                }
                profile
                    .platform_data
                    .insert("api_key".to_string(), serde_json::Value::String(api_key));
            }
        }
        "model" => profile.model = if clear { None } else { value },
        "small_fast_model" => profile.small_fast_model = if clear { None } else { value },
        "provider" => profile.provider = if clear { None } else { value },
        "provider_type" => profile.provider_type = if clear { None } else { value },
        "account" => profile.account = if clear { None } else { value },
        "auth_mode" => {
            if clear {
                profile
                    .platform_data
                    .shift_remove(crate::platforms::ClaudePlatform::AUTH_MODE_FIELD);
            } else {
                let value = value
                    .ok_or_else(|| CcrError::ValidationError("auth_mode 需要非空字符串".into()))?;
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return Err(CcrError::ValidationError("auth_mode 不能为空字符串".into()));
                }
                profile.platform_data.insert(
                    crate::platforms::ClaudePlatform::AUTH_MODE_FIELD.to_string(),
                    serde_json::Value::String(trimmed.to_string()),
                );
            }
        }
        "api_backend" => {
            if clear {
                profile.platform_data.shift_remove("api_backend");
            } else {
                let backend = value
                    .ok_or_else(|| CcrError::ValidationError("api_backend 需要非空字符串".into()))?
                    .trim()
                    .to_ascii_lowercase();
                if !matches!(
                    backend.as_str(),
                    "chat_completions" | "responses" | "messages"
                ) {
                    return Err(CcrError::ValidationError(
                        "api_backend 仅支持 chat_completions、responses 或 messages".into(),
                    ));
                }
                profile.platform_data.insert(
                    "api_backend".to_string(),
                    serde_json::Value::String(backend),
                );
            }
        }
        "env_key" => {
            if clear {
                profile.platform_data.shift_remove("env_key");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "env_key MVP 仅支持单个环境变量名，不接受数组".into(),
                    ));
                }
                let env_key = value
                    .ok_or_else(|| CcrError::ValidationError("env_key 需要非空字符串".into()))?
                    .trim()
                    .to_string();
                if env_key.is_empty() || env_key.starts_with('[') || env_key.contains(',') {
                    return Err(CcrError::ValidationError(
                        "env_key MVP 仅支持单个环境变量名，不接受数组".into(),
                    ));
                }
                profile
                    .platform_data
                    .insert("env_key".to_string(), serde_json::Value::String(env_key));
            }
        }
        "context_window" => {
            if clear {
                profile.platform_data.shift_remove("context_window");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "context_window 需为正整数".into(),
                    ));
                }
                let context_window = value
                    .as_deref()
                    .and_then(|raw| raw.trim().parse::<u64>().ok())
                    .filter(|value| *value > 0)
                    .ok_or_else(|| CcrError::ValidationError("context_window 需为正整数".into()))?;
                profile.platform_data.insert(
                    "context_window".to_string(),
                    serde_json::Value::Number(context_window.into()),
                );
            }
        }
        "supports_backend_search" => {
            if clear {
                profile
                    .platform_data
                    .shift_remove("supports_backend_search");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "supports_backend_search 仅支持 true、false、1 或 0".into(),
                    ));
                }
                let enabled = match value.as_deref().map(str::trim) {
                    Some("true" | "1") => true,
                    Some("false" | "0") => false,
                    _ => {
                        return Err(CcrError::ValidationError(
                            "supports_backend_search 仅支持 true、false、1 或 0".into(),
                        ));
                    }
                };
                profile.platform_data.insert(
                    "supports_backend_search".to_string(),
                    serde_json::Value::Bool(enabled),
                );
            }
        }
        "reasoning_effort" => {
            if clear {
                profile.platform_data.shift_remove("reasoning_effort");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "reasoning_effort 需要非空字符串".into(),
                    ));
                }
                let value = value.ok_or_else(|| {
                    CcrError::ValidationError("reasoning_effort 需要非空字符串".into())
                })?;
                let normalized =
                    crate::platforms::GrokPlatform::normalize_reasoning_effort(&value)?;
                profile.platform_data.insert(
                    "reasoning_effort".to_string(),
                    serde_json::Value::String(normalized),
                );
            }
        }
        "tags" => {
            if clear {
                profile.tags = None;
            } else if let Some(raw_json) = value_json {
                let tags: Vec<String> = serde_json::from_str(&raw_json).map_err(|e| {
                    CcrError::ValidationError(format!(
                        "tags 的 value-json 不是合法 JSON 数组: {}",
                        e
                    ))
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
    fn test_parse_platform_rejects_non_auth_profile_platforms() {
        for platform_name in ["gemini", "qwen", "droid"] {
            let err = parse_platform(platform_name).unwrap_err();
            assert!(matches!(err, CcrError::PlatformNotSupported(_)));
            assert!(err.to_string().contains(platform_name));
            assert!(err.to_string().contains("claude, codex and grok"));
        }
    }

    #[test]
    fn test_update_profile_field_parses_grok_specific_values() {
        let mut profile = ProfileConfig::new();

        update_profile_field(
            &mut profile,
            "api_backend",
            Some("MESSAGES".into()),
            None,
            false,
        )
        .unwrap();
        update_profile_field(
            &mut profile,
            "api_key",
            Some("INLINE_SECRET_SENTINEL".into()),
            None,
            false,
        )
        .unwrap();
        update_profile_field(
            &mut profile,
            "env_key",
            Some("GROK_RELAY_KEY".into()),
            None,
            false,
        )
        .unwrap();
        update_profile_field(
            &mut profile,
            "context_window",
            Some("1000000".into()),
            None,
            false,
        )
        .unwrap();
        update_profile_field(
            &mut profile,
            "supports_backend_search",
            Some("1".into()),
            None,
            false,
        )
        .unwrap();
        update_profile_field(
            &mut profile,
            "reasoning_effort",
            Some(" HIGH ".into()),
            None,
            false,
        )
        .unwrap();

        assert_eq!(profile.platform_data["api_backend"], "messages");
        assert_eq!(profile.platform_data["api_key"], "INLINE_SECRET_SENTINEL");
        assert_eq!(profile.platform_data["env_key"], "GROK_RELAY_KEY");
        assert_eq!(profile.platform_data["context_window"], 1_000_000);
        assert_eq!(profile.platform_data["supports_backend_search"], true);
        assert_eq!(profile.platform_data["reasoning_effort"], "high");

        for field in [
            "api_backend",
            "api_key",
            "env_key",
            "context_window",
            "supports_backend_search",
            "reasoning_effort",
        ] {
            update_profile_field(&mut profile, field, None, None, true).unwrap();
            assert!(!profile.platform_data.contains_key(field));
        }
    }

    #[test]
    fn test_update_profile_field_rejects_invalid_grok_specific_values() {
        let invalid = [
            ("api_backend", Some("legacy".to_string()), None),
            ("api_key", Some("  ".to_string()), None),
            ("context_window", Some("0".to_string()), None),
            ("supports_backend_search", Some("maybe".to_string()), None),
            ("env_key", None, Some(r#"["A","B"]"#.to_string())),
            ("reasoning_effort", Some("  ".to_string()), None),
            (
                "reasoning_effort",
                Some("model-specific-option".to_string()),
                None,
            ),
            ("reasoning_effort", None, Some(r#""high""#.to_string())),
        ];

        for (field, value, value_json) in invalid {
            let error =
                update_profile_field(&mut ProfileConfig::new(), field, value, value_json, false)
                    .unwrap_err();
            assert!(matches!(error, CcrError::ValidationError(_)));
        }
    }

    #[test]
    fn test_ensure_field_allowed_rejects_codex_base_url() {
        let err = ensure_field_allowed(Platform::Codex, "base_url").unwrap_err();
        assert!(err.to_string().contains("不允许编辑字段"));
    }
}
