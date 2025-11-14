//! Codex 平台配置管理 API 处理器

use crate::core::error::CcrError;
use crate::models::platform::PlatformConfig;
use crate::platforms::codex::CodexPlatform;
use crate::web::{
    error_utils::{
        bad_request, empty_success_response, internal_server_error, spawn_blocking_string,
        success_response,
    },
    models::{CodexProfileItem, CodexProfileRequest, CodexProfilesResponse},
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

        let mut items = Vec::new();
        for (name, profile) in profiles.into_iter() {
            items.push(build_profile_item(name, &profile, &current_name));
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

fn build_profile_config(req: &CodexProfileRequest) -> crate::models::ProfileConfig {
    use crate::models::ProfileConfig;

    let mut platform_data: IndexMap<String, JsonValue> = IndexMap::new();

    insert_string(&mut platform_data, "api_mode", req.api_mode.as_ref());
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
        "organization",
        req.organization.as_ref(),
    );

    ProfileConfig {
        description: req.description.clone(),
        base_url: Some(req.base_url.clone()),
        auth_token: Some(req.auth_token.clone()),
        model: req.model.clone(),
        small_fast_model: req.small_fast_model.clone(),
        provider: req.provider.clone(),
        provider_type: req.provider_type.clone(),
        account: req.account.clone(),
        tags: req.tags.clone(),
        platform_data,
    }
}

fn build_profile_item(
    name: String,
    profile: &crate::models::ProfileConfig,
    current_profile: &str,
) -> CodexProfileItem {
    let data = &profile.platform_data;

    let mut item = CodexProfileItem {
        name,
        description: profile.description.clone(),
        base_url: profile.base_url.clone(),
        auth_token: profile.auth_token.clone(),
        model: profile.model.clone(),
        small_fast_model: profile.small_fast_model.clone(),
        provider: profile.provider.clone(),
        provider_type: profile.provider_type.clone(),
        account: profile.account.clone(),
        tags: profile.tags.clone(),
        api_mode: get_string(data, "api_mode"),
        wire_api: get_string(data, "wire_api"),
        env_key: get_string(data, "env_key"),
        requires_openai_auth: get_bool(data, "requires_openai_auth"),
        approval_policy: get_string(data, "approval_policy"),
        sandbox_mode: get_string(data, "sandbox_mode"),
        model_reasoning_effort: get_string(data, "model_reasoning_effort"),
        organization: get_string(data, "organization"),
        is_current: false,
    };
    item.is_current = item.name == current_profile;
    item
}

fn insert_string(map: &mut IndexMap<String, JsonValue>, key: &str, value: Option<&String>) {
    if let Some(text) = value {
        if !text.trim().is_empty() {
            map.insert(key.to_string(), JsonValue::String(text.trim().to_string()));
        }
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
