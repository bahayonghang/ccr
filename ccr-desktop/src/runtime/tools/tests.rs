#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::runtime::ConfigSource;
use ccr_app::runtime::{Platform, RuntimeOwner};
use std::time::Duration;
fn key(environment: EnvironmentId) -> RequestKey {
    RequestKey {
        environment,
        platform: Platform::Claude,
        generation: 1,
        request_id: 1,
    }
}
fn fixture(owner: &RuntimeOwner, root: &std::path::Path) -> Requests {
    Requests::new(&Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.join("profiles.toml"),
            registry: root.join("config.toml"),
            delay: Duration::ZERO,
        },
    ))
}
fn wait(requests: &Requests, status: Status) {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while requests.snapshot().status != status {
        assert!(
            std::time::Instant::now() < deadline,
            "{:?}",
            requests.snapshot()
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
#[test]
fn tool_actions_reject_remote_fixture_and_unconfirmed_updates_before_admission() {
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let requests = fixture(&owner, root.path());
    for action in [
        Action::Versions,
        Action::DetectBridge,
        Action::OpenBridge,
        Action::UpdateCcr,
    ] {
        assert_eq!(
            requests.perform(key(EnvironmentId::Ssh("test".into())), action, true),
            Err(Error::LocalOnly)
        );
        assert_eq!(
            requests.perform(key(EnvironmentId::Local("fixture".into())), action, true),
            Err(Error::FixtureDisabled)
        );
    }
    let barrier = Arc::new(std::sync::Barrier::new(2));
    *requests.0.tools.synthetic.lock().unwrap() = Some(barrier);
    assert_eq!(
        requests.perform(
            key(EnvironmentId::Local("fixture".into())),
            Action::UpdateCcr,
            false
        ),
        Err(Error::Confirmation)
    );
    assert_eq!(owner.handle().active_tasks(), 0);
    owner.shutdown();
}
#[test]
fn confirmed_update_is_singleton_and_retains_terminal_after_detaching() {
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let requests = fixture(&owner, root.path());
    let barrier = Arc::new(std::sync::Barrier::new(2));
    *requests.0.tools.synthetic.lock().unwrap() = Some(barrier.clone());
    requests
        .perform(
            key(EnvironmentId::Local("fixture".into())),
            Action::UpdateCcr,
            true,
        )
        .unwrap();
    wait(&requests, Status::Running);
    assert_eq!(
        requests.perform(
            key(EnvironmentId::Local("fixture".into())),
            Action::UpdateCcr,
            true
        ),
        Err(Error::Busy)
    );
    drop(requests.observe());
    barrier.wait();
    wait(&requests, Status::Finished);
    assert_eq!(
        requests.observe().borrow().update_output.as_deref(),
        Some("Controlled fixture: no installed CLI changed")
    );
    owner.shutdown();
}
#[test]
fn update_scope_cannot_target_the_running_desktops_sidecar_directory() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("bin")).unwrap();
    assert_eq!(
        validate_install_root(root.path(), &root.path().join("bin/ccr-gpui.exe")),
        Err(Error::Operation)
    );
    assert!(validate_install_root(root.path(), &root.path().join("portable/ccr-gpui.exe")).is_ok());
    assert_eq!(
        validate_install_root(
            std::path::Path::new("relative"),
            &root.path().join("portable/ccr-gpui.exe")
        ),
        Err(Error::Operation)
    );
}
