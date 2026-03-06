pub mod platform_switch;
pub mod profile_switch;
pub mod types;

pub use platform_switch::switch_platform;
pub use profile_switch::switch_profile;
pub use types::{SwitchPlatformRequest, SwitchProfileRequest};
