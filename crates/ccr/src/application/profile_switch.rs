use crate::application::types::{SwitchProfileRequest, SwitchProfileResult};
use crate::managers::settings::SettingsManager;
use crate::managers::{
    HistoryEntry, HistoryManager, OperationDetails, OperationResult, OperationType,
    PlatformConfigManager,
};
use crate::models::Platform;
use crate::platforms::create_platform;
use ccr_config::profile_to_section;
use ccr_core::core::error::{CcrError, Result};
use std::collections::HashMap;
use std::str::FromStr;

#[allow(dead_code)]
pub async fn switch_profile(request: SwitchProfileRequest) -> Result<SwitchProfileResult> {
    let platform_name = match request.platform_name {
        Some(name) => name,
        None => {
            let platform_config_mgr = PlatformConfigManager::with_default()?;
            let unified_config = platform_config_mgr.load()?;
            unified_config.current_platform
        }
    };

    switch_profile_for_platform(&request.config_name, &platform_name).await
}

pub async fn switch_profile_for_platform(
    config_name: &str,
    platform_name: &str,
) -> Result<SwitchProfileResult> {
    let platform = Platform::from_str(platform_name)?;

    let platform_config = create_platform(platform)
        .map_err(|e| CcrError::ConfigError(format!("创建平台 {} 失败: {}", platform_name, e)))?;

    let mut profiles = platform_config.load_profiles()?;

    let profile = profiles
        .get(config_name)
        .ok_or_else(|| CcrError::ConfigSectionNotFound(config_name.to_string()))?;

    platform_config.validate_profile(profile)?;
    let target_section = profile_to_section(profile)?;

    let (old_env, new_env): (
        HashMap<String, Option<String>>,
        HashMap<String, Option<String>>,
    ) = if platform == Platform::Claude {
        let settings_manager = SettingsManager::with_default()?;
        let old_settings = settings_manager.load().ok();
        let old = old_settings
            .as_ref()
            .map(|s| s.anthropic_env_status())
            .unwrap_or_default();
        let new = target_section.to_anthropic_env_status();
        (old, new)
    } else {
        (HashMap::new(), HashMap::new())
    };

    let previous_profile = platform_config.get_current_profile()?;

    if let Some(profile) = profiles.get_mut(config_name) {
        profile.usage_count = Some(profile.usage_count.unwrap_or(0) + 1);
    }

    platform_config.save_profile(
        config_name,
        profiles
            .get(config_name)
            .ok_or_else(|| CcrError::ConfigError("配置名称应该存在".into()))?,
    )?;

    platform_config.apply_profile(config_name)?;

    let history_manager = HistoryManager::with_default()?;
    let mut history_entry = HistoryEntry::new(
        OperationType::Switch,
        OperationDetails {
            from_config: previous_profile.clone(),
            to_config: Some(config_name.to_string()),
            backup_path: None,
            extra: None,
        },
        OperationResult::Success,
    );

    if platform == Platform::Claude {
        for (var_name, new_value) in new_env.clone() {
            let old_value = old_env.get(&var_name).and_then(|v| v.clone());
            history_entry.add_env_change(var_name, old_value, new_value);
        }
    }

    history_manager.add_async(history_entry).await?;

    Ok(SwitchProfileResult {
        platform_name: platform_name.to_string(),
        platform,
        previous_profile,
        current_profile: config_name.to_string(),
        target_section,
        old_env,
        new_env,
    })
}
