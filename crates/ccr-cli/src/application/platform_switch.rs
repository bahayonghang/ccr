use crate::application::types::{SwitchPlatformRequest, SwitchPlatformResult};
use ccr_core::core::error::Result;

pub fn switch_platform(request: SwitchPlatformRequest) -> Result<SwitchPlatformResult> {
    Err(crate::commands::migration::legacy_platform_command_error(
        &format!("switch {}", request.platform_name),
    ))
}
