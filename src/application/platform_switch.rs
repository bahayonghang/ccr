use crate::application::types::{SwitchPlatformRequest, SwitchPlatformResult};
use crate::core::error::{CcrError, Result};
use crate::managers::{PlatformConfigEntry, PlatformConfigManager};
use crate::models::Platform;
use crate::platforms::create_platform;
use std::str::FromStr;

pub fn switch_platform(request: SwitchPlatformRequest) -> Result<SwitchPlatformResult> {
    let manager = PlatformConfigManager::with_default()?;
    let mut config = manager.load_or_create_default()?;

    let platform = Platform::from_str(&request.platform_name)
        .map_err(|_| CcrError::PlatformNotFound(request.platform_name.clone()))?;

    if !config.platforms.contains_key(&request.platform_name) {
        let platform_impl = create_platform(platform)?;
        let registry = PlatformConfigEntry {
            description: Some(platform_impl.platform_name().to_string()),
            ..Default::default()
        };
        config.register_platform(request.platform_name.clone(), registry)?;
    }

    let old_platform = config.current_platform.clone();
    config.set_current_platform(&request.platform_name)?;
    manager.save(&config)?;

    let current_profile = config
        .platforms
        .get(&request.platform_name)
        .and_then(|entry| entry.current_profile.clone());

    Ok(SwitchPlatformResult {
        old_platform,
        new_platform: request.platform_name,
        current_profile,
    })
}
