//! Claude profile patch composition extracted from the Tauri adapter.
use super::fields::*;
use ccr_config::ProfileConfig;
use serde_json::{Map, Value};

/// Native advanced editing fails closed; legacy Tauri decoding keeps its policy.
pub fn validate_fields(value: &Value) -> Result<(), String> {
    super::fields::validate_fields(
        value,
        &[
            "name",
            "description",
            "base_url",
            "auth_token",
            "model",
            "small_fast_model",
            "default_opus_model",
            "default_sonnet_model",
            "default_haiku_model",
            "default_fable_model",
            "default_opus_model_name",
            "default_sonnet_model_name",
            "default_haiku_model_name",
            "default_fable_model_name",
            "subagent_model",
            "custom_model_option",
            "custom_model_option_name",
            "effort_level",
            "claude_code_auto_compact_window",
            "api_timeout_ms",
            "claude_code_disable_nonessential_traffic",
            "provider",
            "provider_type",
            "account",
            "tags",
            "usage_count",
            "enabled",
            "extra",
            "platform_data",
            "auth_mode",
        ],
    )
}

fn parse_platform_data_update(
    obj: &Map<String, Value>,
) -> Result<Option<Map<String, Value>>, String> {
    let has_extra = obj.contains_key("extra");
    let has_platform_data = obj.contains_key("platform_data");

    if !has_extra && !has_platform_data {
        return Ok(None);
    }

    let mut platform_data = Map::new();

    if let Some(raw) = obj.get("extra")
        && let Some(extra) = parse_extra_field(raw, "extra")?
    {
        platform_data.extend(extra);
    }

    if let Some(raw) = obj.get("platform_data")
        && let Some(extra) = parse_extra_field(raw, "platform_data")?
    {
        platform_data.extend(extra);
    }

    Ok(Some(platform_data))
}

pub fn build(config: &Value) -> Result<ProfileConfig, String> {
    let obj = config
        .as_object()
        .ok_or_else(|| "profile config 必须是对象".to_string())?;

    let mut profile = ProfileConfig::new();
    patch(&mut profile, config)?;

    if let Some(platform_data) = parse_platform_data_update(obj)? {
        profile.platform_data = platform_data.into_iter().collect();
    }

    if let Some(raw) = obj.get("auth_mode") {
        match raw {
            Value::Null => {
                profile.platform_data.shift_remove("auth_mode");
            }
            Value::String(value) if !value.trim().is_empty() => {
                profile.platform_data.insert(
                    "auth_mode".to_string(),
                    Value::String(value.trim().to_string()),
                );
            }
            Value::String(_) => {
                return Err("字段 'auth_mode' 不能为空字符串".to_string());
            }
            _ => return Err("字段 'auth_mode' 必须是字符串".to_string()),
        }
    }

    Ok(profile)
}

pub fn patch(profile: &mut ProfileConfig, config: &Value) -> Result<(), String> {
    let obj = config
        .as_object()
        .ok_or_else(|| "profile config 必须是对象".to_string())?;

    if let Some(raw) = obj.get("description") {
        profile.description = parse_string_field(raw, "description")?;
    }
    if let Some(raw) = obj.get("base_url") {
        profile.base_url = parse_string_field(raw, "base_url")?;
    }
    if let Some(raw) = obj.get("auth_token") {
        profile.auth_token = parse_string_field(raw, "auth_token")?.map(ccr_core::Secret::new);
    }
    if let Some(raw) = obj.get("model") {
        profile.model = parse_string_field(raw, "model")?;
    }
    if let Some(raw) = obj.get("small_fast_model") {
        profile.small_fast_model = parse_string_field(raw, "small_fast_model")?;
    }
    if let Some(raw) = obj.get("default_opus_model") {
        profile.default_opus_model = parse_string_field(raw, "default_opus_model")?;
    }
    if let Some(raw) = obj.get("default_sonnet_model") {
        profile.default_sonnet_model = parse_string_field(raw, "default_sonnet_model")?;
    }
    if let Some(raw) = obj.get("default_haiku_model") {
        profile.default_haiku_model = parse_string_field(raw, "default_haiku_model")?;
    }
    if let Some(raw) = obj.get("default_fable_model") {
        profile.default_fable_model = parse_string_field(raw, "default_fable_model")?;
    }
    if let Some(raw) = obj.get("default_opus_model_name") {
        profile.default_opus_model_name = parse_string_field(raw, "default_opus_model_name")?;
    }
    if let Some(raw) = obj.get("default_sonnet_model_name") {
        profile.default_sonnet_model_name = parse_string_field(raw, "default_sonnet_model_name")?;
    }
    if let Some(raw) = obj.get("default_haiku_model_name") {
        profile.default_haiku_model_name = parse_string_field(raw, "default_haiku_model_name")?;
    }
    if let Some(raw) = obj.get("default_fable_model_name") {
        profile.default_fable_model_name = parse_string_field(raw, "default_fable_model_name")?;
    }
    if let Some(raw) = obj.get("subagent_model") {
        profile.subagent_model = parse_string_field(raw, "subagent_model")?;
    }
    if let Some(raw) = obj.get("custom_model_option") {
        profile.custom_model_option = parse_string_field(raw, "custom_model_option")?;
    }
    if let Some(raw) = obj.get("custom_model_option_name") {
        profile.custom_model_option_name = parse_string_field(raw, "custom_model_option_name")?;
    }
    if let Some(raw) = obj.get("effort_level") {
        profile.effort_level = parse_string_field(raw, "effort_level")?;
    }
    if let Some(raw) = obj.get("claude_code_auto_compact_window") {
        profile.claude_code_auto_compact_window =
            parse_string_field(raw, "claude_code_auto_compact_window")?;
    }
    if let Some(raw) = obj.get("api_timeout_ms") {
        profile.api_timeout_ms = parse_string_field(raw, "api_timeout_ms")?;
    }
    if let Some(raw) = obj.get("claude_code_disable_nonessential_traffic") {
        profile.claude_code_disable_nonessential_traffic =
            parse_string_field(raw, "claude_code_disable_nonessential_traffic")?;
    }
    if let Some(raw) = obj.get("provider") {
        profile.provider = parse_string_field(raw, "provider")?;
    }
    if let Some(raw) = obj.get("provider_type") {
        profile.provider_type = parse_string_field(raw, "provider_type")?;
    }
    if let Some(raw) = obj.get("account") {
        profile.account = parse_string_field(raw, "account")?;
    }
    if let Some(raw) = obj.get("tags") {
        profile.tags = parse_tags_field(raw)?;
    }
    if let Some(raw) = obj.get("usage_count") {
        profile.usage_count = parse_usage_count_field(raw)?;
    }
    if let Some(raw) = obj.get("enabled") {
        profile.enabled = parse_bool_field(raw, "enabled")?;
    }

    if let Some(platform_data) = parse_platform_data_update(obj)? {
        profile.platform_data = platform_data.into_iter().collect();
    }

    if let Some(raw) = obj.get("auth_mode") {
        match raw {
            Value::Null => {
                profile.platform_data.shift_remove("auth_mode");
            }
            Value::String(value) if !value.trim().is_empty() => {
                profile.platform_data.insert(
                    "auth_mode".to_string(),
                    Value::String(value.trim().to_string()),
                );
            }
            Value::String(_) => {
                return Err("字段 'auth_mode' 不能为空字符串".to_string());
            }
            _ => return Err("字段 'auth_mode' 必须是字符串".to_string()),
        }
    }

    Ok(())
}
