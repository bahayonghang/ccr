//! Explicit Claude compatibility adapter for the generic configuration page.
//! Platform profile pages retain their dedicated commands.

use ccr_cli::application::profile_lifecycle::{
    ApplyProfileRequest, ProfileOutcome, apply_profile, enable_profile, operation_lock,
    update_profile,
};
use ccr_cli::platforms::create_platform;
use ccr_config::managers::config::{ConfigPatch, FieldPatch};
use ccr_config::platforms::base::section_to_profile;
use ccr_config::{ConfigManager, ConfigSection, ConfigService, Platform};
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
        if !fields.is_empty() {
            return Err(D::Error::custom("unknown_config_patch_field"));
        }
        Ok(result)
    }
}

impl ConfigPatchInput {
    fn into_patch(self) -> Result<ConfigPatch> {
        let value = serde_json::to_value(self)
            .map_err(|_| CcrError::ValidationError("invalid_config_patch".into()))?;
        let fields = value
            .as_object()
            .ok_or_else(|| CcrError::ValidationError("invalid_config_patch".into()))?;
        let mut patch = ConfigPatch::default();
        for (key, value) in fields {
            let edit = if value.is_null() {
                FieldPatch::Remove
            } else {
                FieldPatch::Set(
                    serde_json::from_value(value.clone())
                        .map_err(|_| CcrError::ValidationError("invalid_config_patch".into()))?,
                )
            };
            patch.fields.insert(key.clone(), edit);
        }
        Ok(patch)
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

fn valid_name(name: &str) -> Result<()> {
    if name.trim().is_empty() || matches!(name, "default_config" | "current_config" | "settings") {
        return Err(CcrError::ValidationError("invalid_config_name".into()));
    }
    Ok(())
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
    manager.mutate_or_create(|config| {
        valid_name(name)?;
        if config.sections.contains_key(name) {
            return Err(CcrError::ValidationError("config_already_exists".into()));
        }
        let section = patch.apply(&ConfigSection {
            enabled: Some(true),
            usage_count: Some(0),
            ..Default::default()
        })?;
        instance.validate_profile(&section_to_profile(&section))?;
        if config.sections.is_empty() {
            config.default_config = name.into();
            if !config.current_config.is_empty() {
                config.current_config = name.into();
            }
        }
        config.sections.insert(name.into(), section);
        Ok(())
    })?;
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
    let active = instance.get_current_profile()?;
    ConfigManager::for_platform(&platform.to_string())?.mutate_versioned(
        expected,
        false,
        |config| {
            valid_name(name)?;
            let section = patch.apply(config.get_section(name)?)?;
            if (config.current_config == name || active.as_deref() == Some(name))
                && !section.is_enabled()
            {
                return Err(CcrError::ValidationError(
                    "active_config_cannot_be_disabled".into(),
                ));
            }
            instance.validate_profile(&section_to_profile(&section))?;
            config.sections.insert(name.into(), section);
            Ok(())
        },
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
    ConfigManager::for_platform(&platform.to_string())?.mutate(|config| {
        valid_name(target)?;
        if config.sections.contains_key(target) {
            return Err(CcrError::ValidationError("config_already_exists".into()));
        }
        let original = config.get_section(source)?.clone();
        config.sections.insert(target.into(), original);
        Ok(())
    })?;
    Ok(result(target, None))
}

pub(super) fn delete(platform: Platform, name: &str) -> Result<ConfigMutationResult> {
    let _operation = operation_lock(platform)?;
    if create_platform(platform)?.get_current_profile()?.as_deref() == Some(name) {
        return Err(CcrError::ValidationError(
            "active_config_cannot_be_deleted".into(),
        ));
    }
    ConfigService::for_platform(&platform.to_string())?.delete_config(name)?;
    Ok(result(name, None))
}
