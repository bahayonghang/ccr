//! Explicit Claude compatibility adapter for the generic configuration page.
//! Platform profile pages retain their dedicated commands.

use ccr_cli::application::profile_lifecycle::{
    ApplyProfileRequest, ProfileOutcome, apply_profile, enable_profile, operation_lock,
    update_profile,
};
use ccr_cli::platforms::create_platform;
use ccr_config::managers::config::ConfigPatch;
use ccr_config::{ConfigManager, ConfigService, Platform};
use ccr_core::{CcrError, Result};
use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/config/")]
pub enum ConfigPlatform {
    Claude,
}

pub(super) fn required_platform(value: Option<&str>) -> Result<Platform> {
    match value {
        Some("claude") => Ok(Platform::Claude),
        None => Err(CcrError::ValidationError("platform_required".into())),
        Some(_) => Err(CcrError::ValidationError(
            "config_platform_unsupported".into(),
        )),
    }
}

/// Wire mapping stays here so the shared application crate never owns TS exports.
pub(super) fn list(service: &ccr_app::config::ConfigReadService) -> Result<Vec<super::ConfigInfo>> {
    service.list().map(|rows| {
        rows.into_iter()
            .map(|row| super::ConfigInfo {
                version: row.version,
                name: row.name,
                description: row.description,
                base_url: row.base_url,
                auth_token: row.auth_token,
                model: row.model,
                small_fast_model: row.small_fast_model,
                is_current: row.is_current,
                is_default: row.is_default,
                provider: row.provider,
                provider_type: row.provider_type,
                account: row.account,
                tags: row.tags,
                usage_count: row.usage_count,
                enabled: row.enabled,
            })
            .collect()
    })
}

/// Missing retains a field; null removes it. Values have no Debug output.
#[derive(Clone, Default, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/config/")]
pub struct ConfigPatchInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub description: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub base_url: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub auth_token: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub model: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub small_fast_model: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub provider: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub provider_type: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub account: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tags: Option<Option<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub enabled: Option<bool>,
}

impl<'de> Deserialize<'de> for ConfigPatchInput {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        use serde::de::Error;
        let mut fields = serde_json::Map::<String, serde_json::Value>::deserialize(deserializer)?;
        ccr_app::config::mutations::parse_patch(&serde_json::Value::Object(fields.clone()))
            .map_err(|error| match error {
                CcrError::ValidationError(message) => D::Error::custom(message),
                _ => D::Error::custom("invalid_config_patch"),
            })?;
        fn take<T: serde::de::DeserializeOwned, E: Error>(
            fields: &mut serde_json::Map<String, serde_json::Value>,
            key: &str,
        ) -> std::result::Result<Option<T>, E> {
            fields
                .remove(key)
                .map(|value| serde_json::from_value(value).map_err(E::custom))
                .transpose()
        }
        let result = Self {
            description: take(&mut fields, "description")?,
            base_url: take(&mut fields, "base_url")?,
            auth_token: take(&mut fields, "auth_token")?,
            model: take(&mut fields, "model")?,
            small_fast_model: take(&mut fields, "small_fast_model")?,
            provider: take(&mut fields, "provider")?,
            provider_type: take(&mut fields, "provider_type")?,
            account: take(&mut fields, "account")?,
            tags: take(&mut fields, "tags")?,
            enabled: take(&mut fields, "enabled")?,
        };
        Ok(result)
    }
}

impl ConfigPatchInput {
    fn into_patch(self) -> Result<ConfigPatch> {
        let value = serde_json::to_value(self)
            .map_err(|_| CcrError::ValidationError("invalid_config_patch".into()))?;
        ccr_app::config::mutations::parse_patch(&value)
    }
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/config/")]
pub struct ConfigMutationResult {
    pub platform: ConfigPlatform,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub outcome: Option<ProfileOutcome>,
}

fn result(name: &str, outcome: Option<ProfileOutcome>) -> ConfigMutationResult {
    ConfigMutationResult {
        platform: ConfigPlatform::Claude,
        name: name.into(),
        outcome,
    }
}

pub(super) fn activate(
    platform: Platform,
    name: String,
    enable: bool,
) -> Result<ConfigMutationResult> {
    let request = ApplyProfileRequest::new(platform, &name);
    let outcome = if enable {
        enable_profile(request)?
    } else {
        apply_profile(request)?
    };
    Ok(result(&name, Some(outcome)))
}

pub(super) fn add(
    platform: Platform,
    name: &str,
    data: ConfigPatchInput,
) -> Result<ConfigMutationResult> {
    let patch = data.into_patch()?;
    let instance = create_platform(platform)?;
    let manager = ConfigManager::for_platform(&platform.to_string())?;
    ccr_app::config::mutations::add(&manager, instance.as_ref(), name, &patch)?;
    Ok(result(name, None))
}

pub(super) fn update(
    platform: Platform,
    name: &str,
    data: ConfigPatchInput,
    expected: Option<&str>,
) -> Result<ConfigMutationResult> {
    let patch = data.into_patch()?;
    let _operation = operation_lock(platform)?;
    let instance = create_platform(platform)?;
    ccr_app::config::mutations::update(
        &ConfigManager::for_platform(&platform.to_string())?,
        instance.as_ref(),
        name,
        &patch,
        expected,
        &_operation,
    )?;
    Ok(result(name, None))
}

pub(super) fn rename(
    platform: Platform,
    source: &str,
    target: &str,
) -> Result<ConfigMutationResult> {
    let outcome = update_profile(platform, source, target, |_| Ok(()))?;
    Ok(result(target, Some(outcome)))
}

pub(super) fn duplicate(
    platform: Platform,
    source: &str,
    target: &str,
) -> Result<ConfigMutationResult> {
    ccr_app::config::mutations::duplicate(
        &ConfigManager::for_platform(&platform.to_string())?,
        source,
        target,
    )?;
    Ok(result(target, None))
}

pub(super) fn delete(platform: Platform, name: &str) -> Result<ConfigMutationResult> {
    let _operation = operation_lock(platform)?;
    ccr_app::config::mutations::delete(
        &ConfigService::for_platform(&platform.to_string())?,
        create_platform(platform)?.as_ref(),
        name,
        &_operation,
    )?;
    Ok(result(name, None))
}
