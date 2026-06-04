#[path = "managers/general.rs"]
mod general;
#[path = "managers/legacy_registry.rs"]
mod legacy_registry;

#[path = "support/env.rs"]
mod env;
pub(crate) fn setup_ccr_test_env() -> env::CcrIntegrationTestEnv {
    env::CcrIntegrationTestEnv::new()
}
