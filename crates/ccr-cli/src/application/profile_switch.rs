use crate::application::profile_lifecycle::{ApplyProfileRequest, apply_profile};
use crate::application::types::{SwitchProfileRequest, SwitchProfileResult};
use crate::managers::settings::SettingsManager;
use crate::models::Platform;
use crate::platforms::create_platform;
use ccr_config::profile_to_section;
use ccr_core::core::error::{CcrError, Result};
use std::collections::HashMap;
use std::str::FromStr;

pub async fn switch_profile(request: SwitchProfileRequest) -> Result<SwitchProfileResult> {
    let platform_name = request
        .platform_name
        .ok_or_else(|| CcrError::ValidationError("platform_name is required".into()))?;
    switch_profile_for_platform(&request.config_name, &platform_name).await
}

pub async fn switch_profile_for_platform(
    config_name: &str,
    platform_name: &str,
) -> Result<SwitchProfileResult> {
    let request = ApplyProfileRequest::new(Platform::from_str(platform_name)?, config_name);
    tokio::task::spawn_blocking(move || switch_profile_sync(request))
        .await
        .map_err(|_| {
            CcrError::ConfigError(
                "Profile operation worker failed; inspect operation record".into(),
            )
        })?
}

/// CLI adapter retains the existing detail result without exposing it in the DTO.
pub fn switch_profile_sync(request: ApplyProfileRequest) -> Result<SwitchProfileResult> {
    let platform = request.platform;
    let instance = create_platform(platform)?;
    // Details are presentation-only. A replay remains available after deletion.
    let target_section = instance
        .load_profiles()
        .ok()
        .and_then(|profiles| profiles.get(&request.name).cloned())
        .and_then(|profile| profile_to_section(&profile).ok())
        .unwrap_or_default();
    let old_env = if platform == Platform::Claude {
        SettingsManager::with_default()
            .and_then(|manager| manager.load())
            .map(|s| s.anthropic_env_status())
            .unwrap_or_default()
    } else {
        HashMap::new()
    };
    let new_env = if platform == Platform::Claude {
        target_section.to_anthropic_env_status()
    } else {
        HashMap::new()
    };
    let outcome = apply_profile(request)?;
    Ok(SwitchProfileResult {
        platform_name: platform.to_string(),
        platform,
        previous_profile: outcome.previous_profile.clone(),
        current_profile: outcome.profile.clone(),
        target_section,
        old_env,
        new_env,
        outcome,
    })
}
