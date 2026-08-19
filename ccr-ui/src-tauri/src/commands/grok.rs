//! Grok Build profile, settings, and dashboard commands.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use ccr::platforms::{GrokActivationState, GrokPlatform, GrokProfileAuthMode};
use ccr::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
use ccr_cli::application::profile_off_for_platform;
use ccr_core::core::{
    BackupPolicy, VersionedWriteOutcome, WriteOptions, content_version_token,
    write_guarded_versioned,
};
use serde::Serialize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use tauri::State;
use ts_rs::TS;

use crate::commands::settings_raw::{ensure_local_env, toml_error_position};
use crate::commands::wire::OpenJsonValueDto;
use crate::state::AppState;

const SETTINGS_WRITE_ATTEMPTS: usize = 3;
const MANAGED_SETTINGS_KEYS: &[&str] = &["models.default", "models.default_reasoning_effort"];
const SETTINGS_KEYS: &[&str] = &[
    "models.default",
    "models.default_reasoning_effort",
    "ui.theme",
    "session.auto_compact_threshold_percent",
    "session.load_envrc",
    "cli.auto_update",
    "cli.channel",
    "cli.show_tips",
    "hints.new_session_worktree_mode",
    "hints.fork_worktree_mode",
];

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub enum GrokAuthModeDto {
    InlineApiKey,
    EnvKey,
    Session,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub enum GrokActivationDto {
    Inactive,
    Active,
    Drifted,
    UnsafeMissingEntryState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub enum GrokProfileKindDto {
    Official,
    ThirdParty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub enum GrokDeleteBlockedReasonDto {
    Active,
    Drifted,
    UnsafeMissingEntryState,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub struct GrokProfileDto {
    pub name: String,
    pub description: Option<String>,
    pub provider: Option<String>,
    pub profile_kind: GrokProfileKindDto,
    pub base_url_display: Option<String>,
    pub has_base_url: bool,
    pub model: Option<String>,
    pub api_backend: Option<String>,
    #[ts(as = "Option<f64>")]
    pub context_window: Option<u64>,
    pub supports_backend_search: Option<bool>,
    pub reasoning_effort: Option<String>,
    pub auth_mode: GrokAuthModeDto,
    pub env_key: Option<String>,
    pub has_inline_credential: bool,
    pub enabled: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub struct GrokProfilesResponse {
    pub profiles: Vec<GrokProfileDto>,
    pub current_profile: Option<String>,
    pub activation: GrokActivationDto,
    pub activation_name: Option<String>,
    pub default_profile: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub enum GrokProfilesCommandResponse {
    Ok {
        #[serde(flatten)]
        data: GrokProfilesResponse,
    },
    UnsupportedEnvironment {
        env_type: String,
    },
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub enum GrokProfileCommandResponse {
    Ok { profile: Box<GrokProfileDto> },
    UnsupportedEnvironment { env_type: String },
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub enum GrokProfileActionResponse {
    Created {
        profile: GrokProfileDto,
    },
    Updated {
        profile: GrokProfileDto,
    },
    Renamed {
        old_name: String,
        new_name: String,
    },
    RenameApplyFailed {
        old_name: String,
        new_name: String,
        message: String,
    },
    RenameCleanupFailed {
        old_name: String,
        new_name: String,
        message: String,
    },
    Deleted,
    Blocked {
        reason: GrokDeleteBlockedReasonDto,
        message: String,
    },
    Applied {
        profile: String,
    },
    Off {
        previous_profile: Option<String>,
        changed: bool,
    },
    UnsupportedEnvironment {
        env_type: String,
    },
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub struct GrokDashboardOverview {
    pub activation: GrokActivationDto,
    pub activation_name: Option<String>,
    pub current_profile: Option<String>,
    pub auth_mode: Option<GrokAuthModeDto>,
    #[ts(as = "f64")]
    pub profiles_total: u32,
    #[ts(as = "f64")]
    pub profiles_enabled: u32,
    pub config_exists: bool,
    pub config_path_display: String,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub enum GrokDashboardCommandResponse {
    Ok {
        #[serde(flatten)]
        data: GrokDashboardOverview,
    },
    UnsupportedEnvironment {
        env_type: String,
    },
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub struct GrokModelsSettingsDto {
    pub default: Option<String>,
    pub default_reasoning_effort: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub struct GrokUiSettingsDto {
    pub theme: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub struct GrokSessionSettingsDto {
    #[ts(as = "Option<f64>")]
    pub auto_compact_threshold_percent: Option<i64>,
    pub load_envrc: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub struct GrokCliSettingsDto {
    pub auto_update: Option<bool>,
    pub channel: Option<String>,
    pub show_tips: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub struct GrokHintsSettingsDto {
    pub new_session_worktree_mode: Option<String>,
    pub fork_worktree_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub struct GrokCustomModelDto {
    pub id: String,
    pub name: Option<String>,
    pub model: Option<String>,
    pub base_url_display: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub struct GrokSettingsResponse {
    pub exists: bool,
    pub activation: GrokActivationDto,
    pub activation_name: Option<String>,
    pub managed_keys_locked: bool,
    pub models: GrokModelsSettingsDto,
    pub ui: GrokUiSettingsDto,
    pub session: GrokSessionSettingsDto,
    pub cli: GrokCliSettingsDto,
    pub hints: GrokHintsSettingsDto,
    pub custom_models: Vec<GrokCustomModelDto>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub enum GrokSettingsCommandResponse {
    Ok {
        #[serde(flatten)]
        data: Box<GrokSettingsResponse>,
    },
    UnsupportedEnvironment {
        env_type: String,
    },
}

#[derive(Debug, Clone, serde::Deserialize, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub struct GrokSettingsPatchDto {
    pub set: BTreeMap<String, OpenJsonValueDto>,
    pub unset: Vec<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub enum GrokSettingsUpdateResponse {
    Saved,
    Conflict,
    ManagedLocked { message: String },
    UnsupportedEnvironment { env_type: String },
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub enum GrokRawConfigResponse {
    Ok {
        content: String,
        token: String,
        path: String,
        exists: bool,
    },
    UnsupportedEnvironment {
        env_type: String,
    },
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub enum GrokRawSaveResponse {
    Saved {
        token: String,
    },
    Conflict,
    Invalid {
        kind: String,
        message: String,
        #[ts(optional)]
        #[serde(skip_serializing_if = "Option::is_none")]
        line: Option<usize>,
        #[ts(optional)]
        #[serde(skip_serializing_if = "Option::is_none")]
        column: Option<usize>,
    },
    UnsupportedEnvironment {
        env_type: String,
    },
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub struct GrokConfigLayerDto {
    pub id: String,
    pub label: String,
    pub path: String,
    pub exists: bool,
    #[ts(as = "Option<f64>")]
    pub size: Option<u64>,
    #[ts(as = "Option<f64>")]
    pub mtime: Option<u64>,
    pub editable: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "status", rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/grok/")]
pub enum GrokConfigLayersResponse {
    Ok { layers: Vec<GrokConfigLayerDto> },
    UnsupportedEnvironment { env_type: String },
}

fn unsupported_env_type(value: &JsonValue) -> String {
    value
        .get("envType")
        .or_else(|| value.get("env_type"))
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown")
        .to_string()
}

async fn non_local_env(state: &AppState) -> Option<String> {
    ensure_local_env(state)
        .await
        .map(|value| unsupported_env_type(&value))
}

fn activation_parts(state: GrokActivationState) -> (GrokActivationDto, Option<String>) {
    match state {
        GrokActivationState::Inactive => (GrokActivationDto::Inactive, None),
        GrokActivationState::Active { name } => (GrokActivationDto::Active, Some(name)),
        GrokActivationState::Drifted { name } => (GrokActivationDto::Drifted, Some(name)),
        GrokActivationState::UnsafeMissingEntryState { name } => {
            (GrokActivationDto::UnsafeMissingEntryState, name)
        }
    }
}

fn auth_mode(mode: GrokProfileAuthMode) -> GrokAuthModeDto {
    match mode {
        GrokProfileAuthMode::InlineApiKey => GrokAuthModeDto::InlineApiKey,
        GrokProfileAuthMode::EnvKey => GrokAuthModeDto::EnvKey,
        GrokProfileAuthMode::Session => GrokAuthModeDto::Session,
    }
}

fn profile_string(profile: &ProfileConfig, key: &str) -> Option<String> {
    profile
        .platform_data
        .get(key)
        .and_then(JsonValue::as_str)
        .map(str::to_string)
}

fn profile_kind(profile: &ProfileConfig, auth_mode: GrokProfileAuthMode) -> GrokProfileKindDto {
    if profile
        .base_url
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        || auth_mode != GrokProfileAuthMode::Session
    {
        GrokProfileKindDto::ThirdParty
    } else {
        GrokProfileKindDto::Official
    }
}

fn profile_to_dto(name: String, profile: &ProfileConfig) -> Result<GrokProfileDto, String> {
    let resolved_auth = GrokPlatform::profile_auth_mode(profile)
        .map_err(|error| format!("解析 Grok profile '{name}' 认证形态失败: {error}"))?;
    let base_url_display = profile
        .base_url
        .as_deref()
        .map(GrokPlatform::safe_base_url_for_display);

    Ok(GrokProfileDto {
        name,
        description: profile.description.clone(),
        provider: profile.provider.clone(),
        profile_kind: profile_kind(profile, resolved_auth),
        base_url_display,
        has_base_url: profile
            .base_url
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty()),
        model: profile.model.clone(),
        api_backend: profile_string(profile, "api_backend"),
        context_window: profile
            .platform_data
            .get("context_window")
            .and_then(JsonValue::as_u64),
        supports_backend_search: profile
            .platform_data
            .get("supports_backend_search")
            .and_then(JsonValue::as_bool),
        reasoning_effort: profile_string(profile, "reasoning_effort"),
        auth_mode: auth_mode(resolved_auth),
        env_key: profile_string(profile, "env_key"),
        has_inline_credential: resolved_auth == GrokProfileAuthMode::InlineApiKey,
        enabled: profile.is_enabled(),
        tags: profile.tags.clone().unwrap_or_default(),
    })
}

fn default_profile(paths: &PlatformPaths) -> Option<String> {
    let content = fs::read_to_string(&paths.profiles_file).ok()?;
    toml::from_str::<toml::Value>(&content)
        .ok()?
        .get("default_config")?
        .as_str()
        .map(str::to_string)
}

fn profiles_response(platform: &GrokPlatform) -> Result<GrokProfilesResponse, String> {
    let state = platform
        .inspect_activation_state()
        .map_err(|error| format!("检查 Grok profile 激活状态失败: {error}"))?;
    let (activation, activation_name) = activation_parts(state);
    let profiles = platform
        .load_profiles()
        .map_err(|error| format!("读取 Grok profiles 失败: {error}"))?
        .into_iter()
        .map(|(name, profile)| profile_to_dto(name, &profile))
        .collect::<Result<Vec<_>, _>>()?;
    let paths = PlatformPaths::new(Platform::Grok)
        .map_err(|error| format!("解析 Grok profiles 路径失败: {error}"))?;

    Ok(GrokProfilesResponse {
        current_profile: activation_name.clone(),
        activation,
        activation_name,
        default_profile: default_profile(&paths),
        profiles,
    })
}

fn request_object(request: OpenJsonValueDto) -> Result<JsonMap<String, JsonValue>, String> {
    match JsonValue::from(request) {
        JsonValue::Object(object) => Ok(object),
        _ => Err("Grok profile 请求必须是 JSON object".to_string()),
    }
}

fn take_required_name(object: &mut JsonMap<String, JsonValue>) -> Result<String, String> {
    object
        .remove("name")
        .and_then(|value| value.as_str().map(str::trim).map(str::to_string))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Grok profile name 不能为空".to_string())
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

fn apply_credential_action(
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

fn apply_profile_patch(
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

fn load_profile(platform: &GrokPlatform, name: &str) -> Result<ProfileConfig, String> {
    platform
        .load_profiles()
        .map_err(|error| format!("读取 Grok profiles 失败: {error}"))?
        .shift_remove(name)
        .ok_or_else(|| format!("Grok profile '{name}' 不存在"))
}

fn add_profile(request: OpenJsonValueDto) -> Result<GrokProfileActionResponse, String> {
    let mut object = request_object(request)?;
    let name = take_required_name(&mut object)?;
    if !object.contains_key("credential_action") {
        match (
            object.contains_key("api_key"),
            object.contains_key("env_key"),
        ) {
            (true, false) => {
                object.insert(
                    "credential_action".to_string(),
                    JsonValue::String("replace_api_key".to_string()),
                );
            }
            (false, true) => {
                object.insert(
                    "credential_action".to_string(),
                    JsonValue::String("replace_env_key".to_string()),
                );
            }
            (true, true) => return Err("Grok api_key 与 env_key 不能同时设置".to_string()),
            (false, false) => {}
        }
    }
    let platform = GrokPlatform::new().map_err(|error| format!("初始化 Grok 平台失败: {error}"))?;
    if platform
        .load_profiles()
        .map_err(|error| format!("读取 Grok profiles 失败: {error}"))?
        .contains_key(&name)
    {
        return Err(format!("Grok profile '{name}' 已存在"));
    }
    let mut profile = ProfileConfig::new();
    apply_profile_patch(&mut profile, &object)?;
    platform
        .validate_profile(&profile)
        .map_err(|error| format!("Grok profile 校验失败: {error}"))?;
    platform
        .save_profile(&name, &profile)
        .map_err(|error| format!("保存 Grok profile '{name}' 失败: {error}"))?;
    Ok(GrokProfileActionResponse::Created {
        profile: profile_to_dto(name, &profile)?,
    })
}

fn update_profile(
    name: String,
    patch: OpenJsonValueDto,
) -> Result<GrokProfileActionResponse, String> {
    let platform = GrokPlatform::new().map_err(|error| format!("初始化 Grok 平台失败: {error}"))?;
    update_profile_with_platform(&platform, name, patch)
}

fn update_profile_with_platform(
    platform: &GrokPlatform,
    name: String,
    patch: OpenJsonValueDto,
) -> Result<GrokProfileActionResponse, String> {
    let object = request_object(patch)?;
    let activation = platform
        .inspect_activation_state()
        .map_err(|error| format!("检查 Grok profile 激活状态失败: {error}"))?;
    let mut profile = load_profile(platform, &name)?;
    apply_profile_patch(&mut profile, &object)?;
    platform
        .validate_profile(&profile)
        .map_err(|error| format!("Grok profile 校验失败: {error}"))?;

    let new_name = match object.get("name") {
        None => name.clone(),
        Some(JsonValue::String(value)) if !value.trim().is_empty() => value.trim().to_string(),
        Some(JsonValue::Null) => return Err("Grok profile name 不允许设为 null".to_string()),
        Some(_) => return Err("Grok profile name 必须是非空字符串".to_string()),
    };
    if new_name == name {
        platform
            .save_profile(&name, &profile)
            .map_err(|error| format!("更新 Grok profile '{name}' 失败: {error}"))?;
        return Ok(GrokProfileActionResponse::Updated {
            profile: profile_to_dto(name, &profile)?,
        });
    }

    if platform
        .load_profiles()
        .map_err(|error| format!("读取 Grok profiles 失败: {error}"))?
        .contains_key(&new_name)
    {
        return Err(format!("Grok profile '{new_name}' 已存在"));
    }
    platform
        .save_profile(&new_name, &profile)
        .map_err(|error| format!("保存 Grok 新 profile '{new_name}' 失败: {error}"))?;

    let was_active =
        matches!(&activation, GrokActivationState::Active { name: active } if active == &name);
    finish_profile_rename(
        name,
        new_name,
        was_active,
        |new_name| {
            platform
                .apply_profile(new_name)
                .map_err(|error| error.to_string())
        },
        |old_name| {
            platform
                .delete_profile(old_name)
                .map_err(|error| error.to_string())
        },
    )
}

fn finish_profile_rename<A, D>(
    old_name: String,
    new_name: String,
    was_active: bool,
    apply: A,
    delete: D,
) -> Result<GrokProfileActionResponse, String>
where
    A: FnOnce(&str) -> Result<(), String>,
    D: FnOnce(&str) -> Result<(), String>,
{
    if was_active && apply(&new_name).is_err() {
        return Ok(GrokProfileActionResponse::RenameApplyFailed {
            old_name,
            new_name,
            message: "新旧 profile 均已保留，旧 profile 仍在运行；请重试切换到新 profile。"
                .to_string(),
        });
    }
    if delete(&old_name).is_err() {
        return Ok(GrokProfileActionResponse::RenameCleanupFailed {
            old_name,
            new_name,
            message:
                "新 profile 已保存，但旧 profile 未能删除；请确认运行状态后重试删除旧 profile。"
                    .to_string(),
        });
    }
    Ok(GrokProfileActionResponse::Renamed { old_name, new_name })
}

fn blocked_delete_response(
    state: &GrokActivationState,
) -> Result<GrokProfileActionResponse, String> {
    let reason = match state {
        GrokActivationState::Active { .. } => GrokDeleteBlockedReasonDto::Active,
        GrokActivationState::Drifted { .. } => GrokDeleteBlockedReasonDto::Drifted,
        GrokActivationState::UnsafeMissingEntryState { .. } => {
            GrokDeleteBlockedReasonDto::UnsafeMissingEntryState
        }
        GrokActivationState::Inactive => {
            return Err("内部状态错误: inactive profile 不应进入删除阻断分支".to_string());
        }
    };
    let message = if reason == GrokDeleteBlockedReasonDto::UnsafeMissingEntryState {
        "Grok 入口状态缺失，拒绝自动 off 或删除；请先备份并手工恢复 config.toml".to_string()
    } else {
        "该 Grok profile 仍处于激活或漂移状态；请先 off，或确认后执行 force 删除".to_string()
    };
    Ok(GrokProfileActionResponse::Blocked { reason, message })
}

fn delete_profile(name: String, force: bool) -> Result<GrokProfileActionResponse, String> {
    let platform = GrokPlatform::new().map_err(|error| format!("初始化 Grok 平台失败: {error}"))?;
    let state = platform
        .inspect_activation_state()
        .map_err(|error| format!("检查 Grok profile 激活状态失败: {error}"))?;
    execute_delete_with_state(
        &state,
        &name,
        force,
        || {
            platform
                .clear_active_profile_runtime()
                .map_err(|error| format!("退出 Grok profile 模式失败: {error}"))
        },
        || {
            platform
                .delete_profile(&name)
                .map_err(|error| format!("删除 Grok profile '{name}' 失败: {error}"))
        },
    )
}

fn execute_delete_with_state<O, D>(
    state: &GrokActivationState,
    name: &str,
    force: bool,
    off: O,
    delete: D,
) -> Result<GrokProfileActionResponse, String>
where
    O: FnOnce() -> Result<(), String>,
    D: FnOnce() -> Result<(), String>,
{
    let blocked = match state {
        GrokActivationState::Active { name: active }
        | GrokActivationState::Drifted { name: active } => active == name,
        GrokActivationState::UnsafeMissingEntryState { .. } => true,
        GrokActivationState::Inactive => false,
    };
    if blocked {
        if !force || matches!(state, GrokActivationState::UnsafeMissingEntryState { .. }) {
            return blocked_delete_response(state);
        }
        off()?;
    }
    delete()?;
    Ok(GrokProfileActionResponse::Deleted)
}

fn get_toml_string(root: &toml::Value, path: &[&str]) -> Option<String> {
    path.iter()
        .try_fold(root, |value, key| value.get(*key))
        .and_then(toml::Value::as_str)
        .map(str::to_string)
}

fn get_toml_bool(root: &toml::Value, path: &[&str]) -> Option<bool> {
    path.iter()
        .try_fold(root, |value, key| value.get(*key))
        .and_then(toml::Value::as_bool)
}

fn get_toml_integer(root: &toml::Value, path: &[&str]) -> Option<i64> {
    path.iter()
        .try_fold(root, |value, key| value.get(*key))
        .and_then(toml::Value::as_integer)
}

fn read_config(path: &Path) -> Result<(toml::Value, String, bool), String> {
    match fs::read(path) {
        Ok(bytes) => {
            let token = content_version_token(&bytes);
            let content = String::from_utf8(bytes)
                .map_err(|error| format!("Grok config.toml 不是有效 UTF-8: {error}"))?;
            let value = toml::from_str(&content).map_err(|_| {
                format!(
                    "解析 Grok config.toml 失败 {}: 文件包含无效 TOML",
                    path.display()
                )
            })?;
            Ok((value, token, true))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok((toml::Value::Table(Default::default()), String::new(), false))
        }
        Err(error) => Err(format!(
            "读取 Grok config.toml {} 失败: {error}",
            path.display()
        )),
    }
}

fn custom_models(root: &toml::Value) -> Vec<GrokCustomModelDto> {
    root.get("model")
        .and_then(toml::Value::as_table)
        .map(|models| {
            models
                .iter()
                .filter_map(|(id, value)| {
                    let table = value.as_table()?;
                    Some(GrokCustomModelDto {
                        id: id.clone(),
                        name: table
                            .get("name")
                            .and_then(toml::Value::as_str)
                            .map(str::to_string),
                        model: table
                            .get("model")
                            .and_then(toml::Value::as_str)
                            .map(str::to_string),
                        base_url_display: table
                            .get("base_url")
                            .and_then(toml::Value::as_str)
                            .map(GrokPlatform::safe_base_url_for_display),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn settings_response(platform: &GrokPlatform) -> Result<GrokSettingsResponse, String> {
    let path = platform.get_settings_path();
    let (config, _, exists) = read_config(&path)?;
    let state = platform
        .inspect_activation_state()
        .map_err(|error| format!("检查 Grok profile 激活状态失败: {error}"))?;
    let managed_keys_locked = !matches!(state, GrokActivationState::Inactive);
    let (activation, activation_name) = activation_parts(state);
    Ok(GrokSettingsResponse {
        exists,
        activation,
        activation_name,
        managed_keys_locked,
        models: GrokModelsSettingsDto {
            default: get_toml_string(&config, &["models", "default"]),
            default_reasoning_effort: get_toml_string(
                &config,
                &["models", "default_reasoning_effort"],
            ),
        },
        ui: GrokUiSettingsDto {
            theme: get_toml_string(&config, &["ui", "theme"]),
        },
        session: GrokSessionSettingsDto {
            auto_compact_threshold_percent: get_toml_integer(
                &config,
                &["session", "auto_compact_threshold_percent"],
            ),
            load_envrc: get_toml_bool(&config, &["session", "load_envrc"]),
        },
        cli: GrokCliSettingsDto {
            auto_update: get_toml_bool(&config, &["cli", "auto_update"]),
            channel: get_toml_string(&config, &["cli", "channel"]),
            show_tips: get_toml_bool(&config, &["cli", "show_tips"]),
        },
        hints: GrokHintsSettingsDto {
            new_session_worktree_mode: get_toml_string(
                &config,
                &["hints", "new_session_worktree_mode"],
            ),
            fork_worktree_mode: get_toml_string(&config, &["hints", "fork_worktree_mode"]),
        },
        custom_models: custom_models(&config),
    })
}

fn validate_settings_patch(patch: &GrokSettingsPatchDto) -> Result<(), String> {
    let allowed: BTreeSet<&str> = SETTINGS_KEYS.iter().copied().collect();
    let mut seen = BTreeSet::new();
    for key in patch.set.keys().chain(patch.unset.iter()) {
        if !allowed.contains(key.as_str()) {
            return Err(format!("Grok Settings 不支持字段 '{key}'"));
        }
        if !seen.insert(key) {
            return Err(format!("Grok Settings 字段 '{key}' 不能同时 set 与 unset"));
        }
    }

    for (key, value) in &patch.set {
        let value = JsonValue::from(value.clone());
        let valid = match key.as_str() {
            "session.auto_compact_threshold_percent" => value
                .as_i64()
                .is_some_and(|value| (0..=100).contains(&value)),
            "session.load_envrc" | "cli.auto_update" | "cli.show_tips" => value.is_boolean(),
            "models.default_reasoning_effort" => value.as_str().is_some_and(|value| {
                matches!(
                    value,
                    "none" | "minimal" | "low" | "medium" | "high" | "xhigh" | "max"
                )
            }),
            "ui.theme" => value
                .as_str()
                .is_some_and(|value| matches!(value, "system" | "light" | "dark")),
            "cli.channel" => value
                .as_str()
                .is_some_and(|value| matches!(value, "stable" | "alpha")),
            "hints.new_session_worktree_mode" | "hints.fork_worktree_mode" => value
                .as_str()
                .is_some_and(|value| matches!(value, "ask" | "always" | "never")),
            "models.default" => value.as_str().is_some_and(|value| !value.trim().is_empty()),
            _ => false,
        };
        if !valid {
            return Err(format!("Grok Settings 字段 '{key}' 的值无效"));
        }
    }
    Ok(())
}

fn json_to_toml(value: OpenJsonValueDto) -> Result<toml::Value, String> {
    match JsonValue::from(value) {
        JsonValue::String(value) => Ok(toml::Value::String(value)),
        JsonValue::Bool(value) => Ok(toml::Value::Boolean(value)),
        JsonValue::Number(value) => value
            .as_i64()
            .map(toml::Value::Integer)
            .ok_or_else(|| "Grok Settings 数字必须是整数".to_string()),
        _ => Err("Grok Settings set 值仅支持 string、bool 或 integer".to_string()),
    }
}

fn set_toml_path(root: &mut toml::Value, path: &str, value: toml::Value) -> Result<(), String> {
    let (section, key) = path
        .split_once('.')
        .ok_or_else(|| format!("无效 Grok Settings 路径: {path}"))?;
    let table = root
        .as_table_mut()
        .ok_or_else(|| "Grok config.toml 顶层必须是 table".to_string())?;
    let section = table
        .entry(section)
        .or_insert_with(|| toml::Value::Table(Default::default()))
        .as_table_mut()
        .ok_or_else(|| format!("Grok Settings section 与现有非 table 字段冲突: {path}"))?;
    section.insert(key.to_string(), value);
    Ok(())
}

fn unset_toml_path(root: &mut toml::Value, path: &str) -> Result<(), String> {
    let (section_name, key) = path
        .split_once('.')
        .ok_or_else(|| format!("无效 Grok Settings 路径: {path}"))?;
    let Some(root_table) = root.as_table_mut() else {
        return Err("Grok config.toml 顶层必须是 table".to_string());
    };
    let remove_section = if let Some(section) = root_table.get_mut(section_name) {
        let section = section
            .as_table_mut()
            .ok_or_else(|| format!("Grok Settings section 与现有非 table 字段冲突: {path}"))?;
        section.remove(key);
        section.is_empty()
    } else {
        false
    };
    if remove_section {
        root_table.remove(section_name);
    }
    Ok(())
}

fn update_settings(
    platform: &GrokPlatform,
    patch: GrokSettingsPatchDto,
) -> Result<GrokSettingsUpdateResponse, String> {
    update_settings_with_writer(platform, patch, |path, content, token, options| {
        write_guarded_versioned(path, content, token, options)
            .map_err(|error| format!("写入 Grok config.toml {} 失败: {error}", path.display()))
    })
}

fn update_settings_with_writer<W>(
    platform: &GrokPlatform,
    patch: GrokSettingsPatchDto,
    mut write: W,
) -> Result<GrokSettingsUpdateResponse, String>
where
    W: FnMut(&Path, &[u8], &str, &WriteOptions) -> Result<VersionedWriteOutcome, String>,
{
    validate_settings_patch(&patch)?;
    let touches_managed = patch
        .set
        .keys()
        .chain(patch.unset.iter())
        .any(|key| MANAGED_SETTINGS_KEYS.contains(&key.as_str()));
    let path = platform.get_settings_path();

    for _ in 0..SETTINGS_WRITE_ATTEMPTS {
        let (mut config, token, _) = read_config(&path)?;
        let state = platform
            .inspect_activation_state()
            .map_err(|error| format!("检查 Grok profile 激活状态失败: {error}"))?;
        if touches_managed && !matches!(state, GrokActivationState::Inactive) {
            return Ok(GrokSettingsUpdateResponse::ManagedLocked {
                message: "当前 Grok profile 仍处于激活或漂移状态，请先 off 再修改托管模型字段"
                    .to_string(),
            });
        }
        for (key, value) in &patch.set {
            set_toml_path(&mut config, key, json_to_toml(value.clone())?)?;
        }
        for key in &patch.unset {
            unset_toml_path(&mut config, key)?;
        }
        let content = toml::to_string_pretty(&config)
            .map_err(|error| format!("序列化 Grok config.toml 失败: {error}"))?;
        let options = WriteOptions {
            backup: BackupPolicy::None,
            secret: true,
            ..Default::default()
        };
        match write(&path, content.as_bytes(), &token, &options)? {
            VersionedWriteOutcome::Written => return Ok(GrokSettingsUpdateResponse::Saved),
            VersionedWriteOutcome::Conflict => continue,
        }
    }
    Ok(GrokSettingsUpdateResponse::Conflict)
}

fn read_raw_config(path: &Path) -> Result<GrokRawConfigResponse, String> {
    match fs::read(path) {
        Ok(bytes) => {
            let token = content_version_token(&bytes);
            let content = String::from_utf8(bytes)
                .map_err(|error| format!("Grok config.toml 不是有效 UTF-8: {error}"))?;
            Ok(GrokRawConfigResponse::Ok {
                content,
                token,
                path: path.display().to_string(),
                exists: true,
            })
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(GrokRawConfigResponse::Ok {
                content: String::new(),
                token: String::new(),
                path: path.display().to_string(),
                exists: false,
            })
        }
        Err(error) => Err(format!(
            "读取 Grok config.toml {} 失败: {error}",
            path.display()
        )),
    }
}

fn save_raw_config(
    path: &Path,
    content: &str,
    expected_token: &str,
) -> Result<GrokRawSaveResponse, String> {
    if let Err(error) = toml::from_str::<toml::Value>(content) {
        let position = toml_error_position(content, &error);
        return Ok(GrokRawSaveResponse::Invalid {
            kind: "syntax".to_string(),
            message: "Invalid TOML syntax".to_string(),
            line: position.map(|value| value.0),
            column: position.map(|value| value.1),
        });
    }
    let options = WriteOptions {
        backup: BackupPolicy::None,
        secret: true,
        ..Default::default()
    };
    match write_guarded_versioned(path, content.as_bytes(), expected_token, &options)
        .map_err(|error| format!("写入 Grok config.toml {} 失败: {error}", path.display()))?
    {
        VersionedWriteOutcome::Written => Ok(GrokRawSaveResponse::Saved {
            token: content_version_token(content.as_bytes()),
        }),
        VersionedWriteOutcome::Conflict => Ok(GrokRawSaveResponse::Conflict),
    }
}

fn config_layer(id: &str, label: &str, path: PathBuf, editable: bool) -> GrokConfigLayerDto {
    let metadata = fs::metadata(&path).ok();
    let mtime = metadata
        .as_ref()
        .and_then(|value| value.modified().ok())
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map(|value| value.as_millis() as u64);
    GrokConfigLayerDto {
        id: id.to_string(),
        label: label.to_string(),
        path: path.display().to_string(),
        exists: path.exists(),
        size: metadata.as_ref().map(fs::Metadata::len),
        mtime,
        editable,
    }
}

fn config_layers(user_config: PathBuf) -> Result<Vec<GrokConfigLayerDto>, String> {
    let home = dirs::home_dir().ok_or_else(|| "无法获取用户目录".to_string())?;
    let cwd = std::env::current_dir().map_err(|error| format!("读取当前目录失败: {error}"))?;
    Ok(vec![
        config_layer("user", "User", user_config, true),
        config_layer(
            "project",
            "Project",
            cwd.join(".grok").join("config.toml"),
            false,
        ),
        config_layer(
            "managed_user",
            "Managed (user)",
            home.join(".grok").join("managed_config.toml"),
            false,
        ),
        config_layer(
            "managed_system",
            "Managed (system)",
            PathBuf::from("/etc/grok/managed_config.toml"),
            false,
        ),
        config_layer(
            "requirements_user",
            "Requirements (user)",
            home.join(".grok").join("requirements.toml"),
            false,
        ),
        config_layer(
            "requirements_system",
            "Requirements (system)",
            PathBuf::from("/etc/grok/requirements.toml"),
            false,
        ),
    ])
}

#[ccr_tauri_command_macros::command]
pub async fn grok_list_profiles(
    state: State<'_, AppState>,
) -> Result<GrokProfilesCommandResponse, String> {
    if let Some(env_type) = non_local_env(state.inner()).await {
        return Ok(GrokProfilesCommandResponse::UnsupportedEnvironment { env_type });
    }
    tokio::task::spawn_blocking(|| {
        let platform =
            GrokPlatform::new().map_err(|error| format!("初始化 Grok 平台失败: {error}"))?;
        Ok(GrokProfilesCommandResponse::Ok {
            data: profiles_response(&platform)?,
        })
    })
    .await
    .map_err(|error| format!("读取 Grok profiles 后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn grok_get_profile(
    state: State<'_, AppState>,
    name: String,
) -> Result<GrokProfileCommandResponse, String> {
    if let Some(env_type) = non_local_env(state.inner()).await {
        return Ok(GrokProfileCommandResponse::UnsupportedEnvironment { env_type });
    }
    tokio::task::spawn_blocking(move || {
        let platform =
            GrokPlatform::new().map_err(|error| format!("初始化 Grok 平台失败: {error}"))?;
        let profile = load_profile(&platform, &name)?;
        Ok(GrokProfileCommandResponse::Ok {
            profile: Box::new(profile_to_dto(name, &profile)?),
        })
    })
    .await
    .map_err(|error| format!("读取 Grok profile 后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn grok_add_profile(
    state: State<'_, AppState>,
    request: OpenJsonValueDto,
) -> Result<GrokProfileActionResponse, String> {
    if let Some(env_type) = non_local_env(state.inner()).await {
        return Ok(GrokProfileActionResponse::UnsupportedEnvironment { env_type });
    }
    tokio::task::spawn_blocking(move || add_profile(request))
        .await
        .map_err(|error| format!("创建 Grok profile 后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn grok_update_profile(
    state: State<'_, AppState>,
    name: String,
    patch: OpenJsonValueDto,
) -> Result<GrokProfileActionResponse, String> {
    if let Some(env_type) = non_local_env(state.inner()).await {
        return Ok(GrokProfileActionResponse::UnsupportedEnvironment { env_type });
    }
    tokio::task::spawn_blocking(move || update_profile(name, patch))
        .await
        .map_err(|error| format!("更新 Grok profile 后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn grok_delete_profile(
    state: State<'_, AppState>,
    name: String,
    force: Option<bool>,
) -> Result<GrokProfileActionResponse, String> {
    if let Some(env_type) = non_local_env(state.inner()).await {
        return Ok(GrokProfileActionResponse::UnsupportedEnvironment { env_type });
    }
    tokio::task::spawn_blocking(move || delete_profile(name, force.unwrap_or(false)))
        .await
        .map_err(|error| format!("删除 Grok profile 后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn grok_apply_profile(
    state: State<'_, AppState>,
    name: String,
) -> Result<GrokProfileActionResponse, String> {
    if let Some(env_type) = non_local_env(state.inner()).await {
        return Ok(GrokProfileActionResponse::UnsupportedEnvironment { env_type });
    }
    tokio::task::spawn_blocking(move || {
        let platform =
            GrokPlatform::new().map_err(|error| format!("初始化 Grok 平台失败: {error}"))?;
        platform
            .apply_profile(&name)
            .map_err(|error| format!("切换 Grok profile '{name}' 失败: {error}"))?;
        Ok(GrokProfileActionResponse::Applied { profile: name })
    })
    .await
    .map_err(|error| format!("切换 Grok profile 后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn grok_profile_off(
    state: State<'_, AppState>,
) -> Result<GrokProfileActionResponse, String> {
    if let Some(env_type) = non_local_env(state.inner()).await {
        return Ok(GrokProfileActionResponse::UnsupportedEnvironment { env_type });
    }
    tokio::task::spawn_blocking(|| {
        let result = profile_off_for_platform(Platform::Grok)
            .map_err(|error| format!("退出 Grok profile 模式失败: {error}"))?;
        Ok(GrokProfileActionResponse::Off {
            changed: result.changed,
            previous_profile: result.previous_profile,
        })
    })
    .await
    .map_err(|error| format!("退出 Grok profile 模式后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn grok_get_settings(
    state: State<'_, AppState>,
) -> Result<GrokSettingsCommandResponse, String> {
    if let Some(env_type) = non_local_env(state.inner()).await {
        return Ok(GrokSettingsCommandResponse::UnsupportedEnvironment { env_type });
    }
    tokio::task::spawn_blocking(|| {
        let platform =
            GrokPlatform::new().map_err(|error| format!("初始化 Grok 平台失败: {error}"))?;
        Ok(GrokSettingsCommandResponse::Ok {
            data: Box::new(settings_response(&platform)?),
        })
    })
    .await
    .map_err(|error| format!("读取 Grok Settings 后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn grok_update_settings(
    state: State<'_, AppState>,
    patch: GrokSettingsPatchDto,
) -> Result<GrokSettingsUpdateResponse, String> {
    if let Some(env_type) = non_local_env(state.inner()).await {
        return Ok(GrokSettingsUpdateResponse::UnsupportedEnvironment { env_type });
    }
    tokio::task::spawn_blocking(move || {
        let platform =
            GrokPlatform::new().map_err(|error| format!("初始化 Grok 平台失败: {error}"))?;
        update_settings(&platform, patch)
    })
    .await
    .map_err(|error| format!("更新 Grok Settings 后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn grok_get_config_raw_text(
    state: State<'_, AppState>,
) -> Result<GrokRawConfigResponse, String> {
    if let Some(env_type) = non_local_env(state.inner()).await {
        return Ok(GrokRawConfigResponse::UnsupportedEnvironment { env_type });
    }
    let path = GrokPlatform::new()
        .map_err(|error| format!("初始化 Grok 平台失败: {error}"))?
        .get_settings_path();
    tokio::task::spawn_blocking(move || read_raw_config(&path))
        .await
        .map_err(|error| format!("读取 Grok config source 后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn grok_save_config_raw_text(
    state: State<'_, AppState>,
    content: String,
    token: String,
) -> Result<GrokRawSaveResponse, String> {
    if let Some(env_type) = non_local_env(state.inner()).await {
        return Ok(GrokRawSaveResponse::UnsupportedEnvironment { env_type });
    }
    let path = GrokPlatform::new()
        .map_err(|error| format!("初始化 Grok 平台失败: {error}"))?
        .get_settings_path();
    tokio::task::spawn_blocking(move || save_raw_config(&path, &content, &token))
        .await
        .map_err(|error| format!("写入 Grok config source 后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn grok_list_config_layers(
    state: State<'_, AppState>,
) -> Result<GrokConfigLayersResponse, String> {
    if let Some(env_type) = non_local_env(state.inner()).await {
        return Ok(GrokConfigLayersResponse::UnsupportedEnvironment { env_type });
    }
    let path = GrokPlatform::new()
        .map_err(|error| format!("初始化 Grok 平台失败: {error}"))?
        .get_settings_path();
    tokio::task::spawn_blocking(move || {
        Ok(GrokConfigLayersResponse::Ok {
            layers: config_layers(path)?,
        })
    })
    .await
    .map_err(|error| format!("探测 Grok config 层级后台任务失败: {error}"))?
}

#[ccr_tauri_command_macros::command]
pub async fn grok_get_dashboard_overview(
    state: State<'_, AppState>,
) -> Result<GrokDashboardCommandResponse, String> {
    if let Some(env_type) = non_local_env(state.inner()).await {
        return Ok(GrokDashboardCommandResponse::UnsupportedEnvironment { env_type });
    }
    tokio::task::spawn_blocking(|| {
        let platform =
            GrokPlatform::new().map_err(|error| format!("初始化 Grok 平台失败: {error}"))?;
        let profiles = platform
            .load_profiles()
            .map_err(|error| format!("读取 Grok profiles 失败: {error}"))?;
        let state = platform
            .inspect_activation_state()
            .map_err(|error| format!("检查 Grok profile 激活状态失败: {error}"))?;
        let (activation, activation_name) = activation_parts(state);
        let auth_mode = activation_name
            .as_deref()
            .and_then(|name| profiles.get(name))
            .map(GrokPlatform::profile_auth_mode)
            .transpose()
            .map_err(|error| format!("解析 Grok profile 认证形态失败: {error}"))?
            .map(auth_mode);
        let config_path = platform.get_settings_path();
        Ok(GrokDashboardCommandResponse::Ok {
            data: GrokDashboardOverview {
                current_profile: activation_name.clone(),
                activation,
                activation_name,
                auth_mode,
                profiles_total: profiles.len() as u32,
                profiles_enabled: profiles
                    .values()
                    .filter(|profile| profile.is_enabled())
                    .count() as u32,
                config_exists: config_path.exists(),
                config_path_display: config_path.display().to_string(),
            },
        })
    })
    .await
    .map_err(|error| format!("读取 Grok 首页概览后台任务失败: {error}"))?
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_core::Secret;
    use serde_json::json;
    use std::cell::Cell;
    use tempfile::tempdir;

    fn test_platform(config_path: PathBuf, root: PathBuf) -> GrokPlatform {
        let platform_dir = root.join("platforms").join("grok");
        GrokPlatform::from_parts(
            PlatformPaths {
                registry_file: root.join("config.toml"),
                profiles_file: platform_dir.join("profiles.toml"),
                settings_file: platform_dir.join("settings.json"),
                history_file: root.join("history").join("grok.json"),
                backups_dir: root.join("backups").join("grok"),
                platform_dir,
                root,
            },
            config_path,
        )
    }

    fn relay_profile() -> ProfileConfig {
        let mut profile = ProfileConfig::new()
            .with_base_url("https://example.com/v1".to_string())
            .with_model("grok-4.5".to_string());
        profile.provider_type = Some("third_party_model".to_string());
        profile
            .platform_data
            .insert("api_key".to_string(), json!("INLINE_SECRET_SENTINEL"));
        profile
    }

    #[test]
    fn profile_dto_never_serializes_credentials_or_unsafe_url_parts() {
        let mut profile = ProfileConfig::new()
            .with_base_url("https://user:pass@example.com/v1?token=secret#fragment".to_string())
            .with_model("grok-4.5".to_string());
        profile.provider_type = Some("third_party_model".to_string());
        profile.auth_token = Some(Secret::new("INLINE_SECRET_SENTINEL"));
        let dto = profile_to_dto("relay".to_string(), &profile).unwrap();
        let serialized = serde_json::to_string(&dto).unwrap();

        assert!(!serialized.contains("INLINE_SECRET_SENTINEL"));
        assert!(!serialized.contains("pass"));
        assert!(!serialized.contains("token=secret"));
        assert!(serialized.contains("https://example.com/v1"));
        assert!(serialized.contains("has_inline_credential"));

        let payload = serde_json::to_value(&dto).unwrap();
        assert!(payload.get("api_key").is_none());
        assert!(payload.get("auth_token").is_none());
        assert_eq!(payload["profile_kind"], "third_party");
    }

    #[test]
    fn profile_kind_follows_runtime_shape_instead_of_provider_label() {
        let mut inline = ProfileConfig::new();
        inline.provider_type = Some("official_relay".to_string());
        inline.auth_token = Some(Secret::new("INLINE_SECRET_SENTINEL"));
        let inline = profile_to_dto("inline".to_string(), &inline).unwrap();
        assert_eq!(inline.profile_kind, GrokProfileKindDto::ThirdParty);

        let mut session = ProfileConfig::new();
        session.provider_type = Some("third_party_model".to_string());
        let session = profile_to_dto("session".to_string(), &session).unwrap();
        assert_eq!(session.profile_kind, GrokProfileKindDto::Official);
    }

    #[test]
    fn profile_patch_preserves_absent_fields_and_supports_null_and_credentials() {
        let mut profile = ProfileConfig::new()
            .with_description("before".to_string())
            .with_base_url("https://example.com/secret?query=1".to_string())
            .with_model("old".to_string());
        profile
            .platform_data
            .insert("env_key".to_string(), json!("GROK_KEY"));
        let patch = json!({
            "description": null,
            "model": "new",
            "credential_action": "replace_api_key",
            "api_key": "INLINE_SECRET_SENTINEL"
        });
        apply_profile_patch(&mut profile, patch.as_object().unwrap()).unwrap();

        assert_eq!(profile.description, None);
        assert_eq!(profile.model.as_deref(), Some("new"));
        assert_eq!(
            profile.base_url.as_deref(),
            Some("https://example.com/secret?query=1")
        );
        assert!(!profile.platform_data.contains_key("env_key"));
        assert_eq!(
            profile.platform_data["api_key"].as_str(),
            Some("INLINE_SECRET_SENTINEL")
        );
    }

    #[test]
    fn credential_actions_cover_preserve_replace_and_clear() {
        let mut original = relay_profile();
        original.auth_token = Some(Secret::new("COMPAT_SECRET_SENTINEL"));
        original.platform_data.shift_remove("api_key");

        let mut preserved = original.clone();
        apply_credential_action(json!({}).as_object().unwrap(), &mut preserved).unwrap();
        assert!(preserved.auth_token.is_some());

        let mut api_key = original.clone();
        apply_credential_action(
            json!({
                "credential_action": "replace_api_key",
                "api_key": "REPLACEMENT_SECRET_SENTINEL"
            })
            .as_object()
            .unwrap(),
            &mut api_key,
        )
        .unwrap();
        assert!(api_key.auth_token.is_none());
        assert_eq!(
            api_key.platform_data["api_key"].as_str(),
            Some("REPLACEMENT_SECRET_SENTINEL")
        );

        let mut env_key = original.clone();
        apply_credential_action(
            json!({
                "credential_action": "replace_env_key",
                "env_key": "GROK_API_KEY"
            })
            .as_object()
            .unwrap(),
            &mut env_key,
        )
        .unwrap();
        assert!(env_key.auth_token.is_none());
        assert_eq!(env_key.platform_data["env_key"], "GROK_API_KEY");

        let mut cleared = original;
        apply_credential_action(
            json!({ "credential_action": "clear" }).as_object().unwrap(),
            &mut cleared,
        )
        .unwrap();
        assert!(cleared.auth_token.is_none());
        assert!(!cleared.platform_data.contains_key("api_key"));
        assert!(!cleared.platform_data.contains_key("env_key"));

        let error = apply_credential_action(
            json!({ "credential_action": null }).as_object().unwrap(),
            &mut cleared,
        )
        .unwrap_err();
        assert!(error.contains("credential_action"));

        let mut official = ProfileConfig::new().with_model("grok-4".to_string());
        apply_profile_patch(&mut official, json!({ "model": null }).as_object().unwrap()).unwrap();
        assert!(official.model.is_none());
    }

    #[test]
    fn ipc_f64_context_window_is_accepted_as_positive_integer() {
        let mut profile = relay_profile();
        let request = OpenJsonValueDto::try_from(json!({
            "context_window": 500_000.0
        }))
        .unwrap();
        // 模拟 Tauri 反序列化：数字只剩 f64，不再是 JSON 整数。
        let request = match request {
            OpenJsonValueDto::Object(mut object) => {
                object.insert(
                    "context_window".to_string(),
                    OpenJsonValueDto::Number(500_000.0),
                );
                OpenJsonValueDto::Object(object)
            }
            other => other,
        };
        let object = request_object(request).unwrap();
        apply_profile_patch(&mut profile, &object).unwrap();
        assert_eq!(
            profile.platform_data["context_window"].as_u64(),
            Some(500_000)
        );
    }

    #[test]
    fn rename_state_machine_preserves_recoverable_partial_states_without_error_details() {
        let apply_calls = Cell::new(0);
        let delete_calls = Cell::new(0);
        let response = finish_profile_rename(
            "old".to_string(),
            "new".to_string(),
            false,
            |_| {
                apply_calls.set(apply_calls.get() + 1);
                Ok(())
            },
            |_| {
                delete_calls.set(delete_calls.get() + 1);
                Ok(())
            },
        )
        .unwrap();
        assert!(matches!(
            response,
            GrokProfileActionResponse::Renamed { .. }
        ));
        assert_eq!(apply_calls.get(), 0);
        assert_eq!(delete_calls.get(), 1);

        let delete_calls = Cell::new(0);
        let response = finish_profile_rename(
            "old".to_string(),
            "new".to_string(),
            true,
            |_| Err("INLINE_SECRET_SENTINEL https://user:pass@example.com/?token=x".to_string()),
            |_| {
                delete_calls.set(delete_calls.get() + 1);
                Ok(())
            },
        )
        .unwrap();
        let serialized = serde_json::to_string(&response).unwrap();
        assert!(matches!(
            response,
            GrokProfileActionResponse::RenameApplyFailed { .. }
        ));
        assert_eq!(delete_calls.get(), 0);
        assert!(!serialized.contains("INLINE_SECRET_SENTINEL"));
        assert!(!serialized.contains("user:pass"));
        assert!(!serialized.contains("token=x"));

        let response = finish_profile_rename(
            "old".to_string(),
            "new".to_string(),
            true,
            |_| Ok(()),
            |_| Err("CLEANUP_SECRET_SENTINEL".to_string()),
        )
        .unwrap();
        let serialized = serde_json::to_string(&response).unwrap();
        assert!(matches!(
            response,
            GrokProfileActionResponse::RenameCleanupFailed { .. }
        ));
        assert!(!serialized.contains("CLEANUP_SECRET_SENTINEL"));
    }

    #[test]
    fn rename_updates_real_inactive_and_active_profile_state() {
        let inactive_temp = tempdir().unwrap();
        let inactive_platform = test_platform(
            inactive_temp.path().join("config.toml"),
            inactive_temp.path().join("ccr"),
        );
        inactive_platform
            .save_profile("old", &relay_profile())
            .unwrap();
        let response = update_profile_with_platform(
            &inactive_platform,
            "old".to_string(),
            OpenJsonValueDto::try_from(json!({ "name": "new" })).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            response,
            GrokProfileActionResponse::Renamed { .. }
        ));
        let profiles = inactive_platform.load_profiles().unwrap();
        assert!(!profiles.contains_key("old"));
        assert!(profiles.contains_key("new"));
        assert_eq!(
            inactive_platform.inspect_activation_state().unwrap(),
            GrokActivationState::Inactive
        );

        let active_temp = tempdir().unwrap();
        let active_platform = test_platform(
            active_temp.path().join("config.toml"),
            active_temp.path().join("ccr"),
        );
        active_platform
            .save_profile("old", &relay_profile())
            .unwrap();
        active_platform.apply_profile("old").unwrap();
        let response = update_profile_with_platform(
            &active_platform,
            "old".to_string(),
            OpenJsonValueDto::try_from(json!({ "name": "new" })).unwrap(),
        )
        .unwrap();
        assert!(matches!(
            response,
            GrokProfileActionResponse::Renamed { .. }
        ));
        let profiles = active_platform.load_profiles().unwrap();
        assert!(!profiles.contains_key("old"));
        assert!(profiles.contains_key("new"));
        assert_eq!(
            active_platform.inspect_activation_state().unwrap(),
            GrokActivationState::Active {
                name: "new".to_string()
            }
        );
        active_platform.clear_active_profile_runtime().unwrap();
    }

    #[test]
    fn delete_state_machine_blocks_or_forces_only_the_target_activation() {
        let off_calls = Cell::new(0);
        let delete_calls = Cell::new(0);
        let response = execute_delete_with_state(
            &GrokActivationState::Inactive,
            "relay",
            false,
            || {
                off_calls.set(off_calls.get() + 1);
                Ok(())
            },
            || {
                delete_calls.set(delete_calls.get() + 1);
                Ok(())
            },
        )
        .unwrap();
        assert!(matches!(response, GrokProfileActionResponse::Deleted));
        assert_eq!(off_calls.get(), 0);
        assert_eq!(delete_calls.get(), 1);

        let off_calls = Cell::new(0);
        let delete_calls = Cell::new(0);
        let active = GrokActivationState::Active {
            name: "relay".to_string(),
        };
        let response = execute_delete_with_state(
            &active,
            "relay",
            false,
            || {
                off_calls.set(off_calls.get() + 1);
                Ok(())
            },
            || {
                delete_calls.set(delete_calls.get() + 1);
                Ok(())
            },
        )
        .unwrap();
        assert!(matches!(
            response,
            GrokProfileActionResponse::Blocked {
                reason: GrokDeleteBlockedReasonDto::Active,
                ..
            }
        ));
        assert_eq!(off_calls.get(), 0);
        assert_eq!(delete_calls.get(), 0);

        let response = execute_delete_with_state(
            &active,
            "relay",
            true,
            || {
                off_calls.set(off_calls.get() + 1);
                Ok(())
            },
            || {
                delete_calls.set(delete_calls.get() + 1);
                Ok(())
            },
        )
        .unwrap();
        assert!(matches!(response, GrokProfileActionResponse::Deleted));
        assert_eq!(off_calls.get(), 1);
        assert_eq!(delete_calls.get(), 1);

        let unsafe_state = GrokActivationState::UnsafeMissingEntryState {
            name: Some("relay".to_string()),
        };
        let response = execute_delete_with_state(
            &unsafe_state,
            "relay",
            true,
            || panic!("unsafe state must not run off"),
            || panic!("unsafe state must not delete"),
        )
        .unwrap();
        assert!(matches!(
            response,
            GrokProfileActionResponse::Blocked {
                reason: GrokDeleteBlockedReasonDto::UnsafeMissingEntryState,
                ..
            }
        ));

        let response = execute_delete_with_state(
            &GrokActivationState::Drifted {
                name: "other".to_string(),
            },
            "relay",
            true,
            || panic!("force delete of an inactive target must not run off"),
            || Ok(()),
        )
        .unwrap();
        assert!(matches!(response, GrokProfileActionResponse::Deleted));

        let error = execute_delete_with_state(
            &GrokActivationState::Inactive,
            "relay",
            false,
            || Ok(()),
            || Err("concurrent core guard rejection".to_string()),
        )
        .unwrap_err();
        assert_eq!(error, "concurrent core guard rejection");
    }

    #[test]
    fn settings_patch_validates_whitelist_and_value_domains() {
        for (key, value) in [
            ("models.default_reasoning_effort", json!("ultra")),
            ("ui.theme", json!("sepia")),
            ("cli.channel", json!("beta")),
            ("hints.new_session_worktree_mode", json!("sometimes")),
            ("session.auto_compact_threshold_percent", json!(101)),
        ] {
            let patch = GrokSettingsPatchDto {
                set: BTreeMap::from([(
                    key.to_string(),
                    OpenJsonValueDto::try_from(value).unwrap(),
                )]),
                unset: Vec::new(),
            };
            assert!(
                validate_settings_patch(&patch).is_err(),
                "{key} should reject invalid value"
            );
        }

        let unsupported = GrokSettingsPatchDto {
            set: BTreeMap::from([(
                "model.custom.api_key".to_string(),
                OpenJsonValueDto::String("INLINE_SECRET_SENTINEL".to_string()),
            )]),
            unset: Vec::new(),
        };
        let error = validate_settings_patch(&unsupported).unwrap_err();
        assert!(error.contains("不支持字段"));
        assert!(!error.contains("INLINE_SECRET_SENTINEL"));
    }

    #[test]
    fn settings_patch_preserves_unknown_tables_and_writes_no_backup() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("config.toml");
        fs::write(&path, "[ui]\ntheme = 'dark'\n[unknown]\nkeep = 'yes'\n").unwrap();
        let platform = test_platform(path.clone(), temp.path().join("ccr"));
        let patch = GrokSettingsPatchDto {
            set: BTreeMap::from([(
                "ui.theme".to_string(),
                OpenJsonValueDto::String("light".to_string()),
            )]),
            unset: Vec::new(),
        };

        assert!(matches!(
            update_settings(&platform, patch).unwrap(),
            GrokSettingsUpdateResponse::Saved
        ));
        let saved = fs::read_to_string(&path).unwrap();
        assert!(saved.contains("theme = \"light\""));
        assert!(saved.contains("keep = \"yes\""));
        assert!(!temp.path().join("backups").exists());
        assert!(
            fs::read_dir(temp.path())
                .unwrap()
                .all(|entry| { !entry.unwrap().file_name().to_string_lossy().contains("bak") })
        );
    }

    #[test]
    fn settings_patch_retries_after_concurrent_write_and_preserves_external_keys() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("config.toml");
        fs::write(&path, "[ui]\ntheme = 'dark'\n[unknown]\nkeep = 'yes'\n").unwrap();
        let platform = test_platform(path.clone(), temp.path().join("ccr"));
        let patch = GrokSettingsPatchDto {
            set: BTreeMap::from([(
                "ui.theme".to_string(),
                OpenJsonValueDto::String("light".to_string()),
            )]),
            unset: Vec::new(),
        };
        let attempts = Cell::new(0);

        let response = update_settings_with_writer(
            &platform,
            patch,
            |target, content, token, options| {
                attempts.set(attempts.get() + 1);
                if attempts.get() == 1 {
                    let target = target.to_path_buf();
                    std::thread::spawn(move || {
                        fs::write(
                            target,
                            "[ui]\ntheme = 'dark'\n[unknown]\nkeep = 'yes'\n[external]\nadded = true\n",
                        )
                        .unwrap();
                    })
                    .join()
                    .unwrap();
                }
                write_guarded_versioned(target, content, token, options)
                    .map_err(|error| error.to_string())
            },
        )
        .unwrap();

        assert!(matches!(response, GrokSettingsUpdateResponse::Saved));
        assert_eq!(attempts.get(), 2);
        let saved = fs::read_to_string(&path).unwrap();
        assert!(saved.contains("theme = \"light\""));
        assert!(saved.contains("keep = \"yes\""));
        assert!(saved.contains("added = true"));
    }

    #[test]
    fn settings_patch_rechecks_managed_lock_after_concurrent_apply() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("config.toml");
        fs::write(&path, "[ui]\ntheme = 'dark'\n").unwrap();
        let platform = test_platform(path.clone(), temp.path().join("ccr"));
        platform.save_profile("relay", &relay_profile()).unwrap();
        assert_eq!(
            platform.inspect_activation_state().unwrap(),
            GrokActivationState::Inactive
        );
        let patch = GrokSettingsPatchDto {
            set: BTreeMap::from([(
                "models.default".to_string(),
                OpenJsonValueDto::String("manual".to_string()),
            )]),
            unset: Vec::new(),
        };
        let attempts = Cell::new(0);

        let response =
            update_settings_with_writer(&platform, patch, |target, content, token, options| {
                attempts.set(attempts.get() + 1);
                if attempts.get() == 1 {
                    platform.apply_profile("relay").unwrap();
                }
                write_guarded_versioned(target, content, token, options)
                    .map_err(|error| error.to_string())
            })
            .unwrap();

        assert!(matches!(
            response,
            GrokSettingsUpdateResponse::ManagedLocked { .. }
        ));
        assert_eq!(attempts.get(), 1);
        assert!(
            fs::read_to_string(&path)
                .unwrap()
                .contains("default = \"custom\"")
        );
        platform.clear_active_profile_runtime().unwrap();
    }

    #[test]
    fn settings_patch_returns_conflict_after_three_attempts() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("config.toml");
        fs::write(&path, "[ui]\ntheme = 'dark'\n").unwrap();
        let platform = test_platform(path.clone(), temp.path().join("ccr"));
        let attempts = Cell::new(0);
        let response = update_settings_with_writer(
            &platform,
            GrokSettingsPatchDto {
                set: BTreeMap::from([(
                    "ui.theme".to_string(),
                    OpenJsonValueDto::String("light".to_string()),
                )]),
                unset: Vec::new(),
            },
            |_, _, _, _| {
                attempts.set(attempts.get() + 1);
                Ok(VersionedWriteOutcome::Conflict)
            },
        )
        .unwrap();
        assert!(matches!(response, GrokSettingsUpdateResponse::Conflict));
        assert_eq!(attempts.get(), SETTINGS_WRITE_ATTEMPTS);
        assert_eq!(fs::read_to_string(path).unwrap(), "[ui]\ntheme = 'dark'\n");
    }

    #[test]
    fn raw_save_rejects_invalid_or_stale_content_without_backup() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("config.toml");
        fs::write(&path, "[ui]\ntheme = 'dark'\n").unwrap();
        let token = content_version_token(b"[ui]\ntheme = 'dark'\n");
        let invalid = save_raw_config(
            &path,
            "[ui]\napi_key = 'RAW_SECRET_SENTINEL'\nbroken = [",
            &token,
        )
        .unwrap();
        let invalid_json = serde_json::to_string(&invalid).unwrap();
        assert!(matches!(invalid, GrokRawSaveResponse::Invalid { .. }));
        assert!(!invalid_json.contains("RAW_SECRET_SENTINEL"));
        fs::write(&path, "[ui]\ntheme = 'external'\n").unwrap();
        let conflict = save_raw_config(&path, "[ui]\ntheme = 'editor'\n", &token).unwrap();
        assert!(matches!(conflict, GrokRawSaveResponse::Conflict));
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "[ui]\ntheme = 'external'\n"
        );
        assert!(
            fs::read_dir(temp.path())
                .unwrap()
                .all(|entry| { !entry.unwrap().file_name().to_string_lossy().contains("bak") })
        );
    }

    #[test]
    fn every_grok_command_applies_the_local_only_gate_before_work() {
        let source = include_str!("grok.rs");
        let command_sections = source
            .split("#[ccr_tauri_command_macros::command]")
            .skip(1)
            .filter(|section| section.trim_start().starts_with("pub async fn grok_"))
            .collect::<Vec<_>>();

        assert_eq!(command_sections.len(), 13);
        for section in command_sections {
            let signature = section.lines().next().unwrap_or_default();
            assert!(
                section.contains("if let Some(env_type) = non_local_env(state.inner()).await"),
                "{signature} is missing the Local-only gate"
            );
            let gate = section.find("non_local_env").unwrap();
            let blocking_work = section.find("spawn_blocking").unwrap_or(usize::MAX);
            assert!(
                gate < blocking_work,
                "{signature} performs work before the Local-only gate"
            );
        }
    }
}
