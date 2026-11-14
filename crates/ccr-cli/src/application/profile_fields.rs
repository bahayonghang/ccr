//! Shared field allowlists, normalization, and `ProfileConfig` assignment for
//! `profile_create` and `profile_edit`.
//!
//! Validation errors must only describe the field name and expected shape/enum
//! values — they must never echo raw caller values in case a secret was pasted
//! into a regular field.

use crate::models::{Platform, ProfileConfig};
use crate::platforms::GrokPlatform;
use ccr_core::Secret;
use ccr_core::core::error::{CcrError, Result};
use serde_json::Value;

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

pub(crate) fn push_unique(fields: &mut Vec<String>, field: &str) {
    if !fields.iter().any(|existing| existing == field) {
        fields.push(field.to_string());
    }
}

pub(crate) fn is_grok_credential(field: &str) -> bool {
    matches!(field, "api_key" | "env_key" | "auth_token")
}

pub(crate) fn is_secret_field(field: &str) -> bool {
    matches!(field, "auth_token" | "api_key")
}

pub(crate) fn normalize_tags(tags: Vec<String>) -> Option<Vec<String>> {
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

pub(crate) fn ensure_set_field_allowed(platform: Platform, field: &str) -> Result<()> {
    if set_field_allowed(platform, field) {
        Ok(())
    } else {
        Err(CcrError::ValidationError(format!(
            "平台 '{}' 不允许编辑字段 '{field}'",
            platform
        )))
    }
}

pub(crate) fn set_field_allowed(platform: Platform, field: &str) -> bool {
    editable_fields(platform).contains(&field)
}

pub(crate) fn editable_fields(platform: Platform) -> &'static [&'static str] {
    match platform {
        Platform::Codex => ccr_codex::CodexPlatform::editable_fields(),
        Platform::Claude => crate::platforms::ClaudePlatform::editable_fields(),
        Platform::Grok => GrokPlatform::editable_fields(),
        Platform::Gemini | Platform::Qwen | Platform::Droid => DEFAULT_EDITABLE_FIELDS,
    }
}

pub(crate) fn edit_field_allowed(platform: Platform, field: &str) -> bool {
    const COMMON: &[&str] = &[
        "description",
        "base_url",
        "model",
        "provider",
        "tags",
        "enabled",
    ];
    if COMMON.contains(&field) || set_field_allowed(platform, field) {
        return true;
    }
    match platform {
        Platform::Claude => matches!(
            field,
            "default_opus_model"
                | "default_sonnet_model"
                | "default_haiku_model"
                | "default_fable_model"
                | "subagent_model"
                | "effort_level"
                | "api_timeout_ms"
                | "claude_code_auto_compact_window"
        ),
        Platform::Codex => matches!(
            field,
            "model_reasoning_effort" | "wire_api" | "env_key" | "requires_openai_auth"
        ),
        Platform::Grok | Platform::Gemini | Platform::Qwen | Platform::Droid => false,
    }
}

pub(crate) fn create_field_allowed(platform: Platform, field: &str) -> bool {
    if is_secret_field(field) {
        return false;
    }
    match platform {
        Platform::Claude => edit_field_allowed(platform, field),
        Platform::Grok => field != "env_key" && edit_field_allowed(platform, field),
        Platform::Codex => edit_field_allowed(platform, field) || field == "auth_mode",
        Platform::Gemini | Platform::Qwen | Platform::Droid => false,
    }
}

pub(crate) fn enforce_grok_credential_exclusivity(profile: &mut ProfileConfig, field: &str) {
    match field {
        "api_key" => {
            profile.auth_token = None;
            profile.platform_data.shift_remove("auth_token");
            profile.platform_data.shift_remove("env_key");
        }
        "env_key" => {
            profile.auth_token = None;
            profile.platform_data.shift_remove("auth_token");
            profile.platform_data.shift_remove("api_key");
        }
        "auth_token" => {
            profile.platform_data.shift_remove("api_key");
            profile.platform_data.shift_remove("env_key");
            profile.platform_data.shift_remove("auth_token");
        }
        _ => {}
    }
}

fn normalize_effort_level(value: &str) -> Result<String> {
    let normalized = value.trim().to_ascii_lowercase();
    if matches!(
        normalized.as_str(),
        "low" | "medium" | "high" | "xhigh" | "max"
    ) {
        Ok(normalized)
    } else {
        Err(CcrError::ValidationError(
            "effort_level 必须为 low、medium、high、xhigh 或 max".into(),
        ))
    }
}

fn normalize_auth_mode(platform: Option<Platform>, value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CcrError::ValidationError("auth_mode 不能为空字符串".into()));
    }
    let normalized = trimmed.to_ascii_lowercase();
    match platform {
        Some(Platform::Claude) => {
            if matches!(normalized.as_str(), "subscription" | "api_key") {
                Ok(normalized)
            } else {
                Err(CcrError::ValidationError(
                    "auth_mode 仅支持 subscription 或 api_key".into(),
                ))
            }
        }
        Some(Platform::Codex) => {
            if matches!(
                normalized.as_str(),
                "openai_chatgpt"
                    | "openai_api_key"
                    | "provider_env_key"
                    | "provider_bearer_token"
                    | "no_auth"
            ) {
                Ok(normalized)
            } else {
                Err(CcrError::ValidationError(
                    "auth_mode 仅支持 openai_chatgpt、openai_api_key、provider_env_key、provider_bearer_token 或 no_auth"
                        .into(),
                ))
            }
        }
        _ => Ok(trimmed.to_string()),
    }
}

fn non_negative_integer_field(
    field: &str,
    value: Option<String>,
    value_json: Option<String>,
    clear: bool,
) -> Result<Option<String>> {
    if clear {
        return Ok(None);
    }
    if value_json.is_some() {
        return Err(CcrError::ValidationError(format!("{field} 需为非负整数")));
    }
    let raw = value.ok_or_else(|| CcrError::ValidationError(format!("{field} 需为非负整数")))?;
    let trimmed = raw.trim();
    if trimmed.is_empty() || !trimmed.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(CcrError::ValidationError(format!("{field} 需为非负整数")));
    }
    let number = trimmed
        .parse::<u64>()
        .map_err(|_| CcrError::ValidationError(format!("{field} 需为非负整数")))?;
    Ok(Some(number.to_string()))
}

pub(crate) fn apply_json_value(
    platform: Option<Platform>,
    profile: &mut ProfileConfig,
    field: &str,
    value: &Value,
) -> Result<()> {
    match value {
        Value::Null => {
            update_profile_field_for_platform(platform, profile, field, None, None, true)
        }
        Value::String(text) => update_profile_field_for_platform(
            platform,
            profile,
            field,
            Some(text.clone()),
            None,
            false,
        ),
        Value::Bool(flag) => update_profile_field_for_platform(
            platform,
            profile,
            field,
            Some(if *flag { "true" } else { "false" }.to_string()),
            None,
            false,
        ),
        Value::Number(number) => update_profile_field_for_platform(
            platform,
            profile,
            field,
            Some(number.to_string()),
            None,
            false,
        ),
        Value::Array(items) if field == "tags" => {
            let mut tags = Vec::with_capacity(items.len());
            for item in items {
                let Some(text) = item.as_str() else {
                    return Err(CcrError::ValidationError("tags 必须是字符串数组".into()));
                };
                tags.push(text.to_string());
            }
            profile.tags = normalize_tags(tags);
            Ok(())
        }
        _ => Err(CcrError::ValidationError(format!(
            "字段 '{field}' 的值类型不受支持"
        ))),
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn update_profile_field(
    profile: &mut ProfileConfig,
    field: &str,
    value: Option<String>,
    value_json: Option<String>,
    clear: bool,
) -> Result<()> {
    update_profile_field_for_platform(None, profile, field, value, value_json, clear)
}

pub(crate) fn update_profile_field_for_platform(
    platform: Option<Platform>,
    profile: &mut ProfileConfig,
    field: &str,
    value: Option<String>,
    value_json: Option<String>,
    clear: bool,
) -> Result<()> {
    match field {
        "description" => profile.description = if clear { None } else { value },
        "base_url" => profile.base_url = if clear { None } else { value },
        "auth_token" => profile.auth_token = if clear { None } else { value.map(Secret::new) },
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
                    .insert("api_key".to_string(), Value::String(api_key));
            }
        }
        "model" => profile.model = if clear { None } else { value },
        "small_fast_model" => profile.small_fast_model = if clear { None } else { value },
        "default_opus_model" => profile.default_opus_model = if clear { None } else { value },
        "default_sonnet_model" => profile.default_sonnet_model = if clear { None } else { value },
        "default_haiku_model" => profile.default_haiku_model = if clear { None } else { value },
        "default_fable_model" => profile.default_fable_model = if clear { None } else { value },
        "subagent_model" => profile.subagent_model = if clear { None } else { value },
        "effort_level" => {
            if clear {
                profile.effort_level = None;
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "effort_level 需要非空字符串".into(),
                    ));
                }
                let value = value.ok_or_else(|| {
                    CcrError::ValidationError("effort_level 需要非空字符串".into())
                })?;
                profile.effort_level = Some(normalize_effort_level(&value)?);
            }
        }
        "api_timeout_ms" => {
            profile.api_timeout_ms =
                non_negative_integer_field("api_timeout_ms", value, value_json, clear)?;
        }
        "claude_code_auto_compact_window" => {
            profile.claude_code_auto_compact_window = non_negative_integer_field(
                "claude_code_auto_compact_window",
                value,
                value_json,
                clear,
            )?;
        }
        "provider" => profile.provider = if clear { None } else { value },
        "provider_type" => {
            if clear {
                profile.provider_type = None;
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "provider_type 仅支持 official_relay 或 third_party_model".into(),
                    ));
                }
                let raw = value.ok_or_else(|| {
                    CcrError::ValidationError(
                        "provider_type 仅支持 official_relay 或 third_party_model".into(),
                    )
                })?;
                let canonical = match raw.trim().to_ascii_lowercase().as_str() {
                    "official_relay" | "official" => "official_relay",
                    "third_party" | "third_party_model" => "third_party_model",
                    _ => {
                        return Err(CcrError::ValidationError(
                            "provider_type 仅支持 official_relay 或 third_party_model".into(),
                        ));
                    }
                };
                profile.provider_type = Some(canonical.to_string());
            }
        }
        "account" => profile.account = if clear { None } else { value },
        "auth_mode" => {
            if clear {
                profile
                    .platform_data
                    .shift_remove(crate::platforms::ClaudePlatform::AUTH_MODE_FIELD);
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError("auth_mode 需要非空字符串".into()));
                }
                let value = value
                    .ok_or_else(|| CcrError::ValidationError("auth_mode 需要非空字符串".into()))?;
                let normalized = normalize_auth_mode(platform, &value)?;
                profile.platform_data.insert(
                    crate::platforms::ClaudePlatform::AUTH_MODE_FIELD.to_string(),
                    Value::String(normalized),
                );
            }
        }
        "api_backend" => {
            if clear {
                profile.platform_data.shift_remove("api_backend");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "api_backend 仅支持 chat_completions、responses 或 messages".into(),
                    ));
                }
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
                profile
                    .platform_data
                    .insert("api_backend".to_string(), Value::String(backend));
            }
        }
        "wire_api" => {
            if clear {
                profile.platform_data.shift_remove("wire_api");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "wire_api 仅支持 responses".into(),
                    ));
                }
                let value = value
                    .ok_or_else(|| CcrError::ValidationError("wire_api 仅支持 responses".into()))?;
                let normalized = value.trim().to_ascii_lowercase();
                if normalized != "responses" {
                    return Err(CcrError::ValidationError(
                        "wire_api 仅支持 responses".into(),
                    ));
                }
                profile
                    .platform_data
                    .insert("wire_api".to_string(), Value::String(normalized));
            }
        }
        "requires_openai_auth" => {
            if clear {
                profile.platform_data.shift_remove("requires_openai_auth");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "requires_openai_auth 仅支持 true、false、1 或 0".into(),
                    ));
                }
                let enabled = match value.as_deref().map(str::trim) {
                    Some("true" | "1") => true,
                    Some("false" | "0") => false,
                    _ => {
                        return Err(CcrError::ValidationError(
                            "requires_openai_auth 仅支持 true、false、1 或 0".into(),
                        ));
                    }
                };
                profile
                    .platform_data
                    .insert("requires_openai_auth".to_string(), Value::Bool(enabled));
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
                    .insert("env_key".to_string(), Value::String(env_key));
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
                    Value::Number(context_window.into()),
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
                profile
                    .platform_data
                    .insert("supports_backend_search".to_string(), Value::Bool(enabled));
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
                let normalized = GrokPlatform::normalize_reasoning_effort(&value)?;
                profile
                    .platform_data
                    .insert("reasoning_effort".to_string(), Value::String(normalized));
            }
        }
        "model_reasoning_effort" => {
            if clear {
                profile.platform_data.shift_remove("model_reasoning_effort");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "model_reasoning_effort 需要非空字符串".into(),
                    ));
                }
                let value = value.ok_or_else(|| {
                    CcrError::ValidationError("model_reasoning_effort 需要非空字符串".into())
                })?;
                let normalized = value.trim().to_ascii_lowercase();
                if !matches!(
                    normalized.as_str(),
                    "minimal" | "low" | "medium" | "high" | "xhigh"
                ) {
                    return Err(CcrError::ValidationError(
                        "model_reasoning_effort 必须为 minimal/low/medium/high/xhigh".into(),
                    ));
                }
                profile.platform_data.insert(
                    "model_reasoning_effort".to_string(),
                    Value::String(normalized),
                );
            }
        }
        "enabled" => {
            if clear {
                return Err(CcrError::ValidationError(
                    "enabled 请使用启用或禁用，不能清空".into(),
                ));
            }
            let enabled = match value.as_deref().map(str::trim) {
                Some("true" | "1") => true,
                Some("false" | "0") => false,
                _ => {
                    return Err(CcrError::ValidationError(
                        "enabled 仅支持 true 或 false".into(),
                    ));
                }
            };
            profile.enabled = Some(enabled);
        }
        "tags" => {
            if clear {
                profile.tags = None;
            } else if let Some(raw_json) = value_json {
                let tags: Vec<String> = serde_json::from_str(&raw_json).map_err(|_| {
                    CcrError::ValidationError("tags 的 value-json 必须是字符串 JSON 数组".into())
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
            return Err(CcrError::ValidationError(format!("不支持的字段 '{field}'")));
        }
    }
    Ok(())
}
