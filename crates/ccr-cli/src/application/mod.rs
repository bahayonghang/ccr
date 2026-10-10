pub mod auth_off;
pub mod platform_switch;
#[cfg(any(test, feature = "test-support"))]
pub mod profile_contract;
pub mod profile_lifecycle;
pub mod profile_off;
pub mod profile_switch;
pub mod types;

pub use auth_off::{
    AuthOffPath, AuthOffResult, auth_off_for_platform, codex_local_auth_off, needs_auth_off,
};
pub use platform_switch::switch_platform;
pub use profile_off::{ProfileOffResult, needs_login_prep, profile_off_for_platform};
pub use types::SwitchPlatformRequest;
