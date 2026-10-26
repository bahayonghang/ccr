//! Grok write-only credential actions and partial field policy.
use ccr_config::ProfileConfig;
use serde_json::{Map as JsonMap, Value as JsonValue};

pub fn validate_fields(value: &JsonValue) -> Result<(), String> {
    super::fields::validate_fields(
        value,
        &[
            "name",
            "description",
            "base_url",
            "model",
            "provider",
            "provider_type",
            "profile_kind",
            "enabled",
            "tags",
            "api_backend",
            "context_window",
            "supports_backend_search",
            "reasoning_effort",
            "credential_action",
            "api_key",
            "env_key",
        ],
    )
}

fn patch_optional_string(
    object: &JsonMap<String, JsonValue>,
    key: &str,
    target: &mut Option<String>,
    allow_clear: bool,
) -> Result<(), String> {
    let Some(value) = object.get(key) else {
        return Ok(());
    };
    match value {
        JsonValue::Null if allow_clear => *target = None,
        JsonValue::Null => return Err(format!("Grok profile 字段 {key} 不允许设为 null")),
        JsonValue::String(value) if !value.trim().is_empty() => *target = Some(value.clone()),
        JsonValue::String(_) => return Err(format!("Grok profile 字段 {key} 不能为空字符串")),
        _ => return Err(format!("Grok profile 字段 {key} 必须是字符串或 null")),
    }
    Ok(())
}

fn patch_platform_field(
    object: &JsonMap<String, JsonValue>,
    key: &str,
    profile: &mut ProfileConfig,
) {
    let Some(value) = object.get(key) else {
        return;
    };
    if value.is_null() {
        profile.platform_data.shift_remove(key);
    } else {
        profile.platform_data.insert(key.to_string(), value.clone());
    }
}

pub fn apply_credential_action(
    object: &JsonMap<String, JsonValue>,
    profile: &mut ProfileConfig,
) -> Result<(), String> {
    let action = match object.get("credential_action") {
        None => "preserve",
        Some(JsonValue::String(value)) => value.as_str(),
        Some(_) => {
            return Err(
                "credential_action 必须是 preserve、replace_api_key、replace_env_key 或 clear"
                    .to_string(),
            );
        }
    };
    match action {
        "preserve" => Ok(()),
        "replace_api_key" => {
            let value = object
                .get("api_key")
                .and_then(JsonValue::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "replace_api_key 需要非空 api_key".to_string())?;
            profile.auth_token = None;
            profile.platform_data.shift_remove("auth_token");
            profile.platform_data.shift_remove("env_key");
            profile
                .platform_data
                .insert("api_key".to_string(), JsonValue::String(value.to_string()));
            Ok(())
        }
        "replace_env_key" => {
            let value = object
                .get("env_key")
                .and_then(JsonValue::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "replace_env_key 需要非空 env_key".to_string())?;
            profile.auth_token = None;
            profile.platform_data.shift_remove("auth_token");
            profile.platform_data.shift_remove("api_key");
            profile
                .platform_data
                .insert("env_key".to_string(), JsonValue::String(value.to_string()));
            Ok(())
        }
        "clear" => {
            profile.auth_token = None;
            profile.platform_data.shift_remove("auth_token");
            profile.platform_data.shift_remove("api_key");
            profile.platform_data.shift_remove("env_key");
            Ok(())
        }
        _ => Err(
            "credential_action 必须是 preserve、replace_api_key、replace_env_key 或 clear"
                .to_string(),
        ),
    }
}

pub fn patch_object(
    profile: &mut ProfileConfig,
    object: &JsonMap<String, JsonValue>,
) -> Result<(), String> {
    patch_optional_string(object, "description", &mut profile.description, true)?;
    patch_optional_string(object, "base_url", &mut profile.base_url, true)?;
    patch_optional_string(object, "model", &mut profile.model, true)?;
    patch_optional_string(object, "provider", &mut profile.provider, true)?;
    patch_optional_string(object, "provider_type", &mut profile.provider_type, true)?;

    if let Some(kind) = object.get("profile_kind") {
        profile.provider_type = match kind.as_str() {
            Some("official") => Some("official".to_string()),
            Some("third_party") => Some("third_party_model".to_string()),
            _ => return Err("profile_kind 必须是 official 或 third_party".to_string()),
        };
    }
    if let Some(value) = object.get("enabled") {
        profile.enabled = match value {
            JsonValue::Null => None,
            JsonValue::Bool(value) => Some(*value),
            _ => return Err("Grok profile 字段 enabled 必须是 bool 或 null".to_string()),
        };
    }
    if let Some(value) = object.get("tags") {
        profile.tags = match value {
            JsonValue::Null => None,
            JsonValue::Array(values) => Some(
                values
                    .iter()
                    .map(|value| {
                        value
                            .as_str()
                            .map(str::to_string)
                            .ok_or_else(|| "Grok profile tags 必须是字符串数组".to_string())
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            _ => return Err("Grok profile 字段 tags 必须是字符串数组或 null".to_string()),
        };
    }

    for key in [
        "api_backend",
        "context_window",
        "supports_backend_search",
        "reasoning_effort",
    ] {
        patch_platform_field(object, key, profile);
    }
    apply_credential_action(object, profile)
}

pub fn patch(profile: &mut ProfileConfig, value: &JsonValue) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| "Grok profile 请求必须是 JSON object".to_string())?;
    patch_object(profile, object)
}

pub fn build(value: &JsonValue) -> Result<ProfileConfig, String> {
    let mut object = value
        .as_object()
        .cloned()
        .ok_or_else(|| "Grok profile 请求必须是 JSON object".to_string())?;
    if !object.contains_key("credential_action") {
        let action = match (
            object.contains_key("api_key"),
            object.contains_key("env_key"),
        ) {
            (true, false) => Some("replace_api_key"),
            (false, true) => Some("replace_env_key"),
            (true, true) => return Err("Grok api_key 与 env_key 不能同时设置".into()),
            (false, false) => None,
        };
        if let Some(action) = action {
            object.insert("credential_action".into(), JsonValue::String(action.into()));
        }
    }
    let mut profile = ProfileConfig::new();
    patch_object(&mut profile, &object)?;
    Ok(profile)
}
