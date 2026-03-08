//! Codex 平台配置管理 API 处理器

use crate::core::error::CcrError;
use crate::models::platform::PlatformConfig;
use crate::platforms::codex::CodexPlatform;
use crate::services::CodexAuthService;
use crate::web::{
    error_utils::{
        bad_request, empty_success_response, internal_server_error, spawn_blocking_string,
        success_response,
    },
    models::{
        CodexProfileEnvResponse, CodexProfileItem, CodexProfileRequest, CodexProfilesResponse,
    },
};
use axum::{Json, extract::Path, response::Response};
use indexmap::IndexMap;
use serde_json::Value as JsonValue;

/// 列出 Codex profiles
pub async fn handle_list_codex_profiles() -> Response {
    let result = spawn_blocking_string(move || {
        let platform = CodexPlatform::new()?;
        let current_name = platform.get_current_profile()?.unwrap_or_default();
        let current_profile = if current_name.is_empty() {
            None
        } else {
            Some(current_name.clone())
        };
        let profiles = platform.load_profiles()?;
        let auth_state = CodexAuthService::new()
            .ok()
            .map(|service| service.get_auth_state());

        let mut items = Vec::new();
        for (name, profile) in profiles.into_iter() {
            items.push(build_profile_item(
                name,
                &profile,
                &current_name,
                auth_state.as_ref(),
            ));
        }

        Ok::<CodexProfilesResponse, CcrError>(CodexProfilesResponse {
            current_profile,
            profiles: items,
        })
    })
    .await;

    match result {
        Ok(payload) => success_response(payload),
        Err(e) => internal_server_error(e),
    }
}

/// 新增 Codex profile
pub async fn handle_add_codex_profile(Json(req): Json<CodexProfileRequest>) -> Response {
    let result = spawn_blocking_string(move || {
        let platform = CodexPlatform::new()?;
        let profile = build_profile_config(&req);
        platform.save_profile(&req.name, &profile)?;
        Ok::<(), CcrError>(())
    })
    .await;

    match result {
        Ok(_) => empty_success_response(),
        Err(e) => bad_request(e),
    }
}

/// 更新 Codex profile
pub async fn handle_update_codex_profile(
    Path(old_name): Path<String>,
    Json(req): Json<CodexProfileRequest>,
) -> Response {
    let result = spawn_blocking_string(move || {
        let platform = CodexPlatform::new()?;
        if old_name != req.name {
            // 先删除旧配置，再写入新配置
            platform.delete_profile(&old_name)?;
        }
        let profile = build_profile_config(&req);
        platform.save_profile(&req.name, &profile)?;
        Ok::<(), CcrError>(())
    })
    .await;

    match result {
        Ok(_) => empty_success_response(),
        Err(e) => bad_request(e),
    }
}

/// 删除 Codex profile
pub async fn handle_delete_codex_profile(Path(name): Path<String>) -> Response {
    let result = spawn_blocking_string(move || {
        let platform = CodexPlatform::new()?;
        platform.delete_profile(&name)?;
        Ok::<(), CcrError>(())
    })
    .await;

    match result {
        Ok(_) => empty_success_response(),
        Err(e) => bad_request(e),
    }
}

/// 获取指定 Codex profile 的环境变量导出
pub async fn handle_get_codex_profile_env(Path(name): Path<String>) -> Response {
    let result = spawn_blocking_string(move || {
        let platform = CodexPlatform::new()?;
        let env = platform.export_profile_env(&name)?;
        let shell_script = platform.export_profile_shell_script(&name)?;
        let env = env
            .into_iter()
            .collect::<std::collections::BTreeMap<_, _>>();
        Ok::<CodexProfileEnvResponse, CcrError>(CodexProfileEnvResponse {
            profile_name: name,
            env,
            shell_script,
        })
    })
    .await;

    match result {
        Ok(payload) => success_response(payload),
        Err(e) => bad_request(e),
    }
}

fn build_profile_config(req: &CodexProfileRequest) -> crate::models::ProfileConfig {
    use crate::models::ProfileConfig;

    let mut platform_data: IndexMap<String, JsonValue> = IndexMap::new();

    insert_string(&mut platform_data, "api_mode", req.api_mode.as_ref());
    insert_string(&mut platform_data, "auth_mode", req.auth_mode.as_ref());
    insert_string(
        &mut platform_data,
        "openai_login_method",
        req.openai_login_method.as_ref(),
    );
    insert_string(&mut platform_data, "wire_api", req.wire_api.as_ref());
    insert_string(&mut platform_data, "env_key", req.env_key.as_ref());
    insert_bool(
        &mut platform_data,
        "requires_openai_auth",
        req.requires_openai_auth,
    );
    insert_string(
        &mut platform_data,
        "approval_policy",
        req.approval_policy.as_ref(),
    );
    insert_string(
        &mut platform_data,
        "sandbox_mode",
        req.sandbox_mode.as_ref(),
    );
    insert_string(
        &mut platform_data,
        "model_reasoning_effort",
        req.model_reasoning_effort.as_ref(),
    );
    insert_string(
        &mut platform_data,
        "network_access",
        req.network_access.as_ref(),
    );
    insert_bool(
        &mut platform_data,
        "disable_response_storage",
        req.disable_response_storage,
    );

    ProfileConfig {
        description: req.description.clone(),
        base_url: trim_string(req.base_url.as_ref()),
        auth_token: trim_string(req.auth_token.as_ref()),
        model: req.model.clone(),
        small_fast_model: req.small_fast_model.clone(),
        provider: req.provider.clone(),
        provider_type: normalize_provider_type(req.provider_type.as_deref()).map(str::to_string),
        account: req.account.clone(),
        tags: req.tags.clone(),
        usage_count: Some(0),
        enabled: Some(true),
        platform_data,
    }
}

fn build_profile_item(
    name: String,
    profile: &crate::models::ProfileConfig,
    current_profile: &str,
    auth_state: Option<&crate::models::AuthState>,
) -> CodexProfileItem {
    let data = &profile.platform_data;
    let is_current = name == current_profile;
    let auth_mode = CodexPlatform::profile_auth_mode(profile);
    let openai_login_method =
        CodexPlatform::profile_openai_login_method(profile).map(|method| match method {
            crate::models::OpenAiAuthMethod::Chatgpt => "chatgpt".to_string(),
            crate::models::OpenAiAuthMethod::Api => "api".to_string(),
        });
    let default_auth_source = CodexPlatform::profile_auth_source(profile);
    let auth_source = if is_current {
        auth_state.map(|state| match state.status {
            crate::models::AuthStateStatus::Unsupported => "unsupported".to_string(),
            _ => match &state.intent {
                crate::models::AuthIntent::OpenAiAuth { method } => match method {
                    crate::models::OpenAiAuthMethod::Chatgpt => "openai_chatgpt".to_string(),
                    crate::models::OpenAiAuthMethod::Api => "openai_api_key".to_string(),
                },
                crate::models::AuthIntent::ProviderEnvKey { env_key } => {
                    format!("provider:{env_key}")
                }
                crate::models::AuthIntent::NoAuth => default_auth_source.clone(),
            },
        })
    } else {
        Some(default_auth_source)
    };

    CodexProfileItem {
        name,
        description: profile.description.clone(),
        base_url: profile.base_url.clone(),
        auth_token: profile.auth_token.clone(),
        auth_mode: Some(auth_mode.as_str().to_string()),
        openai_login_method,
        model: profile.model.clone(),
        small_fast_model: profile.small_fast_model.clone(),
        provider: profile.provider.clone(),
        provider_type: normalize_provider_type(profile.provider_type.as_deref())
            .map(str::to_string),
        account: profile.account.clone(),
        tags: profile.tags.clone(),
        wire_api: get_string(data, "wire_api"),
        env_key: get_string(data, "env_key"),
        requires_openai_auth: get_bool(data, "requires_openai_auth"),
        approval_policy: get_string(data, "approval_policy"),
        sandbox_mode: get_string(data, "sandbox_mode"),
        model_reasoning_effort: get_string(data, "model_reasoning_effort"),
        network_access: get_string(data, "network_access"),
        disable_response_storage: get_bool(data, "disable_response_storage"),
        credential_store: if is_current {
            auth_state.map(|state| state.store.as_str().to_string())
        } else {
            None
        },
        auth_source,
        is_current,
    }
}

fn trim_string(value: Option<&String>) -> Option<String> {
    value
        .map(|text| text.trim())
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

fn normalize_provider_type(value: Option<&str>) -> Option<&'static str> {
    match value?.trim().to_ascii_lowercase().as_str() {
        "official_relay" => Some("official_relay"),
        "third_party" | "third_party_model" => Some("third_party_model"),
        _ => None,
    }
}

fn insert_string(map: &mut IndexMap<String, JsonValue>, key: &str, value: Option<&String>) {
    if let Some(text) = value
        && !text.trim().is_empty()
    {
        map.insert(key.to_string(), JsonValue::String(text.trim().to_string()));
    }
}

fn insert_bool(map: &mut IndexMap<String, JsonValue>, key: &str, value: Option<bool>) {
    if let Some(flag) = value {
        map.insert(key.to_string(), JsonValue::Bool(flag));
    }
}

fn get_string(map: &IndexMap<String, JsonValue>, key: &str) -> Option<String> {
    map.get(key).and_then(|value| match value {
        JsonValue::String(text) => Some(text.to_string()),
        JsonValue::Number(num) => Some(num.to_string()),
        JsonValue::Bool(flag) => Some(flag.to_string()),
        _ => None,
    })
}

fn get_bool(map: &IndexMap<String, JsonValue>, key: &str) -> Option<bool> {
    map.get(key).and_then(|value| match value {
        JsonValue::Bool(flag) => Some(*flag),
        JsonValue::String(text) => match text.to_lowercase().as_str() {
            "true" | "1" => Some(true),
            "false" | "0" => Some(false),
            _ => None,
        },
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_profile_config_preserves_api_mode() {
        let req = CodexProfileRequest {
            name: "legacy".to_string(),
            description: Some("legacy github mode".to_string()),
            base_url: Some("https://api.github.com".to_string()),
            auth_token: Some("ghp_test".to_string()),
            auth_mode: None,
            openai_login_method: None,
            model: Some("gpt-4".to_string()),
            small_fast_model: None,
            provider: Some("github".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            api_mode: Some("github".to_string()),
            wire_api: Some("responses".to_string()),
            env_key: None,
            requires_openai_auth: None,
            approval_policy: None,
            sandbox_mode: None,
            model_reasoning_effort: None,
            network_access: None,
            disable_response_storage: None,
        };

        let profile = build_profile_config(&req);
        assert_eq!(
            profile.platform_data.get("api_mode"),
            Some(&JsonValue::String("github".to_string()))
        );
    }

    #[test]
    fn test_build_profile_config_allows_official_empty_credentials() {
        let req = CodexProfileRequest {
            name: "official".to_string(),
            description: Some("official relay".to_string()),
            base_url: None,
            auth_token: None,
            auth_mode: Some("openai_chatgpt".to_string()),
            openai_login_method: Some("chatgpt".to_string()),
            model: Some("gpt-5-codex".to_string()),
            small_fast_model: None,
            provider: Some("openai".to_string()),
            provider_type: Some("official_relay".to_string()),
            account: None,
            tags: None,
            api_mode: None,
            wire_api: None,
            env_key: None,
            requires_openai_auth: None,
            approval_policy: None,
            sandbox_mode: None,
            model_reasoning_effort: None,
            network_access: None,
            disable_response_storage: None,
        };

        let profile = build_profile_config(&req);
        assert!(profile.base_url.is_none());
        assert!(profile.auth_token.is_none());
        assert_eq!(profile.provider_type.as_deref(), Some("official_relay"));
    }

    #[test]
    fn test_build_profile_config_normalizes_legacy_provider_type() {
        let req = CodexProfileRequest {
            name: "legacy-third-party".to_string(),
            description: Some("legacy provider type".to_string()),
            base_url: Some("https://api.example.com/v1".to_string()),
            auth_token: Some("sk-test".to_string()),
            auth_mode: None,
            openai_login_method: None,
            model: None,
            small_fast_model: None,
            provider: Some("mistral".to_string()),
            provider_type: Some("third_party".to_string()),
            account: None,
            tags: None,
            api_mode: None,
            wire_api: Some("responses".to_string()),
            env_key: None,
            requires_openai_auth: None,
            approval_policy: None,
            sandbox_mode: None,
            model_reasoning_effort: None,
            network_access: None,
            disable_response_storage: None,
        };

        let profile = build_profile_config(&req);
        assert_eq!(profile.provider_type.as_deref(), Some("third_party_model"));
    }
}
