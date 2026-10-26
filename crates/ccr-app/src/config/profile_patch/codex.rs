//! Codex profile patch composition; normalization stays in CodexPlatform.
use super::fields::*;
use ccr_config::ProfileConfig;
use serde_json::{Map, Value};

pub const EXPLICIT_PLATFORM_STRING_FIELDS: &[&str] = &[
    "api_mode",
    "wire_api",
    "env_key",
    "auth_mode",
    "openai_login_method",
    "approval_policy",
    "sandbox_mode",
    "model_reasoning_effort",
    "model_catalog_json",
    "preferred_auth_method",
    "forced_login_method",
    "network_access",
];
pub const EXPLICIT_PLATFORM_BOOL_FIELDS: &[&str] =
    &["requires_openai_auth", "disable_response_storage"];

/// Accepted fields describe this desktop patch contract, rather than CLI flags.
pub fn validate_fields(value: &Value) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "profile config 必须是对象".to_string())?;
    const CORE: &[&str] = &[
        "name",
        "description",
        "base_url",
        "auth_token",
        "model",
        "small_fast_model",
        "provider",
        "provider_type",
        "account",
        "tags",
        "usage_count",
        "enabled",
        "extra",
        "platform_data",
    ];
    if object.keys().any(|key| {
        !CORE
            .iter()
            .chain(EXPLICIT_PLATFORM_STRING_FIELDS)
            .chain(EXPLICIT_PLATFORM_BOOL_FIELDS)
            .any(|accepted| *accepted == key)
    }) {
        return Err("unknown_profile_patch_field".into());
    }
    Ok(())
}

fn parse_platform_data_update(
    obj: &Map<String, Value>,
) -> Result<Option<Map<String, Value>>, String> {
    let has_extra = obj.contains_key("extra");
    let has_platform_data = obj.contains_key("platform_data");
    let has_explicit_platform_fields = EXPLICIT_PLATFORM_STRING_FIELDS
        .iter()
        .chain(EXPLICIT_PLATFORM_BOOL_FIELDS.iter())
        .any(|field| obj.contains_key(*field));

    if !has_extra && !has_platform_data && !has_explicit_platform_fields {
        return Ok(None);
    }

    let mut platform_data = Map::new();

    merge_explicit_platform_fields(&mut platform_data, obj)?;

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

fn merge_optional_string_field(
    platform_data: &mut Map<String, Value>,
    obj: &Map<String, Value>,
    field_name: &str,
) -> Result<(), String> {
    if let Some(raw) = obj.get(field_name)
        && let Some(value) = parse_string_field(raw, field_name)?
    {
        platform_data.insert(field_name.to_string(), Value::String(value));
    }

    Ok(())
}

fn merge_optional_bool_field(
    platform_data: &mut Map<String, Value>,
    obj: &Map<String, Value>,
    field_name: &str,
) -> Result<(), String> {
    if let Some(raw) = obj.get(field_name)
        && let Some(value) = parse_bool_field(raw, field_name)?
    {
        platform_data.insert(field_name.to_string(), Value::Bool(value));
    }

    Ok(())
}

fn merge_explicit_platform_fields(
    platform_data: &mut Map<String, Value>,
    obj: &Map<String, Value>,
) -> Result<(), String> {
    for field_name in EXPLICIT_PLATFORM_STRING_FIELDS {
        merge_optional_string_field(platform_data, obj, field_name)?;
    }

    for field_name in EXPLICIT_PLATFORM_BOOL_FIELDS {
        merge_optional_bool_field(platform_data, obj, field_name)?;
    }

    Ok(())
}

fn apply_profile_config(
    profile: &mut ProfileConfig,
    obj: &Map<String, Value>,
) -> Result<(), String> {
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

    Ok(())
}

pub fn build(config: &Value) -> Result<ProfileConfig, String> {
    let obj = config
        .as_object()
        .ok_or_else(|| "profile config 必须是对象".to_string())?;

    let mut profile = ProfileConfig::new();
    apply_profile_config(&mut profile, obj)?;
    Ok(profile)
}

pub fn patch(profile: &mut ProfileConfig, config: &Value) -> Result<(), String> {
    let obj = config
        .as_object()
        .ok_or_else(|| "profile config 必须是对象".to_string())?;

    apply_profile_config(profile, obj)
}
