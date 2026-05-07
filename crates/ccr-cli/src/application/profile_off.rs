use crate::managers::SettingsManager;
use crate::models::Platform;
use ccr_config::ConfigManager;
use ccr_config::PlatformConfigManager;
use ccr_core::core::error::{CcrError, Result};

pub struct ProfileOffResult {
    pub platform: Platform,
    pub previous_profile: Option<String>,
    pub changed: bool,
}

pub fn profile_off_for_platform(platform: Platform) -> Result<ProfileOffResult> {
    match platform {
        Platform::Codex => codex_profile_off(),
        Platform::Claude => claude_profile_off(),
        _ => Err(CcrError::PlatformNotSupported(format!(
            "{} 暂不支持 profile off",
            platform
        ))),
    }
}

fn claude_profile_off() -> Result<ProfileOffResult> {
    let previous_profile = platform_previous_profile_hint("claude")?;
    let had_profiles_file_pointer = platform_profiles_file_has_current_config("claude")?;
    let had_profile_routing = previous_profile.is_some() || had_profiles_file_pointer;
    let settings_cleared = if had_profile_routing {
        clear_claude_profile_settings_overrides()?
    } else {
        false
    };
    let changed = had_profile_routing || settings_cleared;

    if changed {
        clear_platform_registry_pointer("claude")?;
        clear_profiles_file_pointer("claude")?;
    }

    Ok(ProfileOffResult {
        platform: Platform::Claude,
        previous_profile,
        changed,
    })
}

fn codex_profile_off() -> Result<ProfileOffResult> {
    let codex_platform = ccr_codex::CodexPlatform::new()?;
    let previous_profile = platform_previous_profile_hint("codex")?;
    let changed = previous_profile.is_some()
        || codex_profiles_file_has_current_config()?
        || codex_platform.has_profile_entry_auth_backup();

    if changed {
        // Re-apply official runtime defaults without mutating auth.json.
        codex_platform.clear_active_profile_runtime()?;
        clear_profiles_file_pointer("codex")?;
    }

    Ok(ProfileOffResult {
        platform: Platform::Codex,
        previous_profile,
        changed,
    })
}

fn clear_claude_profile_settings_overrides() -> Result<bool> {
    let manager = SettingsManager::with_default()?;
    let mut settings = match manager.load() {
        Ok(settings) => settings,
        Err(CcrError::SettingsMissing(_)) => return Ok(false),
        Err(error) => return Err(error),
    };

    if !settings.has_anthropic_overrides() {
        return Ok(false);
    }

    settings.clear_anthropic_vars();
    manager.save_atomic(&settings)?;
    Ok(true)
}

fn clear_platform_registry_pointer(platform_name: &str) -> Result<()> {
    let manager = PlatformConfigManager::with_default()?;
    let mut unified = manager.load_or_create_default()?;
    if let Ok(entry) = unified.get_platform_mut(platform_name) {
        entry.current_profile = None;
        entry.last_used = Some(chrono::Utc::now().to_rfc3339());
    }
    manager.save(&unified)
}

fn clear_profiles_file_pointer(platform_name: &str) -> Result<()> {
    let manager = ConfigManager::for_platform(platform_name)?;
    let mut config = manager.load_with_autofix()?;
    config.current_config.clear();
    manager.save(&config)
}

fn platform_previous_profile_hint(platform_name: &str) -> Result<Option<String>> {
    let manager = PlatformConfigManager::with_default()?;
    let unified = manager.load_or_create_default()?;
    Ok(unified
        .get_platform(platform_name)
        .ok()
        .and_then(|entry| entry.current_profile.clone()))
}

fn codex_profiles_file_has_current_config() -> Result<bool> {
    platform_profiles_file_has_current_config("codex")
}

fn platform_profiles_file_has_current_config(platform_name: &str) -> Result<bool> {
    let manager = ConfigManager::for_platform(platform_name)?;
    let config = manager.load_with_autofix()?;
    Ok(!config.current_config.trim().is_empty())
}
