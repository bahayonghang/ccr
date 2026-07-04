#![allow(clippy::unwrap_used)]

use ccr::{Platform, ProfileConfig, create_platform};

#[test]
fn auth_profile_supported_is_limited_to_claude_and_codex() {
    assert_eq!(
        Platform::auth_profile_supported(),
        &[Platform::Claude, Platform::Codex]
    );
}

#[test]
fn all_platforms_still_include_usage_and_sync_platforms() {
    let all_platforms = Platform::all();

    assert!(all_platforms.contains(&Platform::Claude));
    assert!(all_platforms.contains(&Platform::Codex));
    assert!(all_platforms.contains(&Platform::Gemini));
    assert!(all_platforms.contains(&Platform::Qwen));
    assert!(all_platforms.contains(&Platform::Droid));
}

#[test]
fn gemini_profile_storage_still_works_outside_auth_profile_command_surface() {
    let _env = crate::setup_platform_test_env();
    let gemini = create_platform(Platform::Gemini).unwrap();

    let mut profile = ProfileConfig::new();
    profile.base_url = Some("https://generativelanguage.googleapis.com/v1".to_string());
    profile.auth_token = Some(ccr_core::Secret::from("AIzaSy1234567890123456789012345678901234"));
    profile.model = Some("gemini-pro".to_string());

    gemini.save_profile("usage-fixture", &profile).unwrap();

    let profiles = gemini.load_profiles().unwrap();
    assert!(profiles.contains_key("usage-fixture"));
}
