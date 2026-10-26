//! Generic configuration-page policy. Repository and platform owners perform writes.

use ccr_config::managers::config::{ConfigPatch, FieldPatch};
use ccr_config::platforms::base::section_to_profile;
use ccr_config::{ConfigManager, ConfigSection, ConfigService, PlatformConfig};
use ccr_core::core::lock::FileLock;
use ccr_core::{CcrError, Result};

/// The generic configuration page accepts only its historical ten typed fields.
/// Dedicated platform editors have a separate patch contract.
pub fn parse_patch(value: &serde_json::Value) -> Result<ConfigPatch> {
    let fields = value
        .as_object()
        .ok_or_else(|| CcrError::ValidationError("invalid_config_patch".into()))?;
    let mut patch = ConfigPatch::default();
    for (key, value) in fields {
        let valid = match key.as_str() {
            "description" | "base_url" | "auth_token" | "model" | "small_fast_model"
            | "provider" | "provider_type" | "account" => value.is_null() || value.is_string(),
            "tags" => {
                value.is_null()
                    || value
                        .as_array()
                        .is_some_and(|tags| tags.iter().all(serde_json::Value::is_string))
            }
            "enabled" => value.is_boolean(),
            _ => {
                return Err(CcrError::ValidationError(
                    "unknown_config_patch_field".into(),
                ));
            }
        };
        if !valid {
            return Err(CcrError::ValidationError("invalid_config_patch".into()));
        }
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

fn valid_name(name: &str) -> Result<()> {
    if name.trim().is_empty() || matches!(name, "default_config" | "current_config" | "settings") {
        return Err(CcrError::ValidationError("invalid_config_name".into()));
    }
    Ok(())
}

pub fn add(
    manager: &ConfigManager,
    instance: &dyn PlatformConfig,
    name: &str,
    patch: &ConfigPatch,
) -> Result<()> {
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
    })
}

/// Caller holds the matching platform application lock before active resolution.
/// The explicit guard also prevents accidentally calling this policy unlocked.
pub fn update(
    manager: &ConfigManager,
    instance: &dyn PlatformConfig,
    name: &str,
    patch: &ConfigPatch,
    expected: Option<&str>,
    _operation: &FileLock,
) -> Result<()> {
    let active = instance.get_current_profile()?;
    manager.mutate_versioned(expected, false, |config| {
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
    })
}

pub fn duplicate(manager: &ConfigManager, source: &str, target: &str) -> Result<()> {
    manager.mutate(|config| {
        valid_name(target)?;
        if config.sections.contains_key(target) {
            return Err(CcrError::ValidationError("config_already_exists".into()));
        }
        let original = config.get_section(source)?.clone();
        config.sections.insert(target.into(), original);
        Ok(())
    })
}

/// As for update, the caller acquires the application lock before calling.
pub fn delete(
    service: &ConfigService,
    instance: &dyn PlatformConfig,
    name: &str,
    _operation: &FileLock,
) -> Result<()> {
    if instance.get_current_profile()?.as_deref() == Some(name) {
        return Err(CcrError::ValidationError(
            "active_config_cannot_be_deleted".into(),
        ));
    }
    service.delete_config(name)
}
