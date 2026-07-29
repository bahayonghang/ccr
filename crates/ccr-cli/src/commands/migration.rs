use ccr_core::core::error::CcrError;

pub fn legacy_switch_error(profile_name: &str) -> CcrError {
    CcrError::ConfigError(format!(
        "legacy command retired: `ccr switch {profile_name}` no longer infers a platform from current_platform. Use `ccr claude profile switch {profile_name}`, `ccr codex profile switch {profile_name}`, or `ccr grok profile switch {profile_name}`."
    ))
}

pub fn legacy_shortcut_error(profile_name: &str) -> CcrError {
    CcrError::ConfigError(format!(
        "legacy shortcut retired: `ccr {profile_name}` no longer infers a platform from current_platform. Use `ccr claude profile switch {profile_name}`, `ccr codex profile switch {profile_name}`, or `ccr grok profile switch {profile_name}`."
    ))
}

pub fn legacy_platform_command_error(command: &str) -> CcrError {
    CcrError::ConfigError(format!(
        "legacy command retired: `ccr platform {command}` no longer controls auth/profile routing through global current_platform/default_platform state. Use `ccr current` for runtime status, `ccr claude profile ...` for Claude profiles, `ccr codex profile ...` for Codex profiles, and `ccr grok profile ...` for Grok profiles."
    ))
}

pub fn legacy_platform_init_error() -> CcrError {
    CcrError::ConfigError(
        "legacy command retired: `ccr platform init` has moved to explicit profile commands. Use `ccr claude profile init`, `ccr codex profile init`, or `ccr grok profile init`."
            .to_string(),
    )
}
