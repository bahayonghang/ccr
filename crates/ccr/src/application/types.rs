use crate::managers::config::ConfigSection;
use crate::models::Platform;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SwitchProfileRequest {
    pub config_name: String,
    pub platform_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SwitchProfileResult {
    pub platform_name: String,
    pub platform: Platform,
    pub previous_profile: Option<String>,
    pub current_profile: String,
    pub target_section: ConfigSection,
    pub old_env: HashMap<String, Option<String>>,
    pub new_env: HashMap<String, Option<String>>,
}

#[derive(Debug, Clone)]
pub struct SwitchPlatformRequest {
    pub platform_name: String,
}

#[derive(Debug, Clone)]
pub struct SwitchPlatformResult {
    pub old_platform: String,
    pub new_platform: String,
    pub current_profile: Option<String>,
}
