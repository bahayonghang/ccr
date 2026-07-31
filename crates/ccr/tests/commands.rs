#[path = "commands/clean.rs"]
mod clean;

#[path = "commands/current.rs"]
mod current;

#[path = "commands/codex_profile.rs"]
mod codex_profile;

#[path = "commands/codex_fix.rs"]
mod codex_fix;

#[path = "commands/grok_profile.rs"]
mod grok_profile;

#[path = "commands/claude_profile.rs"]
mod claude_profile;

#[path = "commands/doctor.rs"]
mod doctor;

#[path = "commands/help.rs"]
mod help;

#[path = "commands/legacy_routing.rs"]
mod legacy_routing;

#[path = "commands/platform_profile_surface.rs"]
mod platform_profile_surface;

#[path = "commands/project_init.rs"]
mod project_init;

#[path = "commands/validate.rs"]
mod validate;

#[path = "commands/profile_open.rs"]
mod profile_open;

#[path = "commands/sync_content.rs"]
mod sync_content;

#[path = "support/env.rs"]
mod env;
pub(crate) fn setup_ccr_test_env() -> env::CcrIntegrationTestEnv {
    env::CcrIntegrationTestEnv::new()
}
