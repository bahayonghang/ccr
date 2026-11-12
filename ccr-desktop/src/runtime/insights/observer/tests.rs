#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
fn source(root: &Path) -> ConfigSource {
    ConfigSource::Fixture {
        registry: root.join("registry.toml"),
        profiles: root.join("profiles.toml"),
        delay: std::time::Duration::ZERO,
    }
}
#[test]
fn settings_initialize_only_on_explicit_save_and_conflict_without_usage_database() {
    let root = tempfile::tempdir().unwrap();
    let owner = Owner::default();
    let source = source(root.path());
    let report = read(&owner, &source, None, 30, false).unwrap();
    assert!(matches!(report.subscription, Ok(None)));
    assert!(report.usage.is_err());
    assert!(!root.path().join("observer.db").exists());
    let initial = SubscriptionDto {
        mode: "subscription".into(),
        plan: "custom".into(),
        monthly_usd: 17.5,
    };
    assert_eq!(
        save(&owner, &source, None, initial.clone()).unwrap(),
        initial
    );
    let report = read(&owner, &source, None, 30, false).unwrap();
    assert_eq!(report.subscription.unwrap(), Some(initial.clone()));
    assert!(report.usage.is_err());
    let changed = SubscriptionDto {
        monthly_usd: 20.0,
        ..initial.clone()
    };
    save(&owner, &source, Some(initial.clone()), changed.clone()).unwrap();
    assert_eq!(
        save(&owner, &source, Some(initial), SubscriptionDto::default()),
        Err(Error::Conflict)
    );
    assert_eq!(
        read(&owner, &source, None, 30, false)
            .unwrap()
            .subscription
            .unwrap(),
        Some(changed)
    );
    assert!(!root.path().join("archive.db").exists());
}
#[test]
fn broken_existing_storage_is_not_a_default_subscription_or_setup_state() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("observer.db");
    Connection::open(&path).unwrap();
    let owner = Owner::default();
    let report = read(&owner, &source(root.path()), None, 30, false).unwrap();
    assert_eq!(report.subscription, Err(Error::Storage));
    assert!(report.tools.is_err());
    assert_eq!(
        save(
            &owner,
            &source(root.path()),
            None,
            SubscriptionDto::default()
        ),
        Err(Error::Storage)
    );
}
