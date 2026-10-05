use crate::managers::config::ConfigSection;
use crate::models::Platform;
use std::collections::HashMap;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SwitchProfileRequest {
    pub config_name: String,
    pub platform_name: Option<String>,
}

#[derive(Clone)]
pub struct SwitchProfileResult {
    pub outcome: super::profile_lifecycle::ProfileOutcome,
    pub platform_name: String,
    pub platform: Platform,
    pub previous_profile: Option<String>,
    pub current_profile: String,
    pub target_section: ConfigSection,
    pub old_env: HashMap<String, Option<String>>,
    pub new_env: HashMap<String, Option<String>>,
}

impl std::fmt::Debug for SwitchProfileResult {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SwitchProfileResult")
            .field("outcome", &self.outcome)
            .field("platform", &self.platform)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub struct SwitchPlatformRequest {
    pub platform_name: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SwitchPlatformResult {
    pub old_platform: String,
    pub new_platform: String,
    pub current_profile: Option<String>,
}
