#[path = "support/env.rs"]
mod env;
pub(crate) type PlatformTestEnv = env::CcrIntegrationTestEnv;

pub(crate) fn setup_platform_test_env() -> PlatformTestEnv {
    env::CcrIntegrationTestEnv::new()
}

#[path = "platforms/auth_profile_surface.rs"]
mod auth_profile_surface;
#[path = "platforms/general.rs"]
mod general;
#[path = "platforms/integration.rs"]
mod integration;
