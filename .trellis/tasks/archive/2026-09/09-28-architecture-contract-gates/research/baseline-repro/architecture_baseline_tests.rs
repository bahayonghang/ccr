#![allow(clippy::unwrap_used, clippy::expect_used)]
use crate::{models::{Platform, ProfileConfig}, platforms::create_platform, test_support::TestHome};
use ccr_config::{models::PlatformPaths, services::config_service::ConfigService};
fn fixture() -> TestHome {
    let mut home = TestHome::new_with_home_env();
    let data = home.root().as_os_str().to_owned();
    home.set_env("CCR_DATA_DIR", &data);
    let platform = create_platform(Platform::Claude).unwrap();
    for name in ["old", "new"] {
        let mut profile = ProfileConfig::new().with_base_url("https://fixture.invalid".into()).with_auth_token("synthetic-baseline-token".into()).with_model(format!("model-{name}"));
        profile.provider_type = Some("third_party_model".into());
        platform.save_profile(name, &profile).unwrap();
    }
    platform.apply_profile("old").unwrap();
    home
}
#[tokio::test]
async fn a01_desktop_legacy_switch_target_must_succeed() {
    let _home = fixture();
    let result = crate::commands::switch_command("new").await;
    assert!(result.is_ok(), "valid desktop target is permanently rejected: {result:?}");
}
#[test]
fn a02_stale_platform_snapshot_must_not_erase_service_update() {
    let _home = fixture();
    let platform = create_platform(Platform::Claude).unwrap();
    let mut stale = platform.load_profiles().unwrap();
    let service = ConfigService::for_platform("claude").unwrap();
    let mut section = service.load_config().unwrap().sections["old"].clone();
    section.description = Some("service committed change".into());
    service.update_config("old", "old".into(), section).unwrap();
    assert_eq!(service.load_config().unwrap().sections["old"].description.as_deref(), Some("service committed change"));
    stale.get_mut("new").unwrap().description = Some("platform committed change".into());
    let paths = PlatformPaths::new(Platform::Claude).unwrap();
    ccr_config::platforms::base::save_profiles_to_toml(&paths.profiles_file, &stale, "claude", &paths).unwrap();
    let after = service.load_config().unwrap();
    assert_eq!(after.sections["new"].description.as_deref(), Some("platform committed change"));
    assert_eq!(after.sections["old"].description.as_deref(), Some("service committed change"), "stale snapshot erased a successful service update");
}
#[tokio::test]
async fn a03_committed_activation_must_not_return_generic_history_failure() {
    let home = fixture();
    std::fs::create_dir(home.root().join("data.db")).unwrap();
    let result = crate::application::profile_switch::switch_profile_for_platform("new", "claude").await;
    let platform = create_platform(Platform::Claude).unwrap();
    assert_eq!(platform.get_current_profile().unwrap().as_deref(), Some("new"), "fixture must reach committed activation");
    assert!(result.is_ok(), "runtime committed but ancillary history error escaped: {result:?}");
}
#[cfg(windows)]
#[test]
fn a11_permission_failure_must_not_be_reported_as_success() {
    let mut home = TestHome::new_with_home_env();
    home.set_env("USERNAME", std::ffi::OsStr::new("CCR_NO_SUCH_TEST_USER_920134"));
    let path = home.root().join("synthetic-pending.json");
    std::fs::write(&path, b"old synthetic state").unwrap();
    let denied = std::process::Command::new("icacls").arg(&path).args(["/inheritance:r", "/grant:r", "CCR_NO_SUCH_TEST_USER_920134:(F)"]).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status().unwrap();
    assert!(!denied.success(), "fixture must prove actual icacls permission failure");
    // Baseline persistence call sequence with synthetic serialized bytes.
    let result = (|| -> Result<(), String> {
        ccr_core::core::AtomicWriter::new(&path).write_string("synthetic verifier payload").map_err(|e|e.to_string())?;
        ccr_codex::utils::ensure_private_permissions(&path);
        Ok(())
    })();
    assert!(result.is_err(), "failed permission command was swallowed after publishing new bytes");
}
