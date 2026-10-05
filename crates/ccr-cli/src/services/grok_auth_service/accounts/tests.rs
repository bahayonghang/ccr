//! Static fixture sources; execution is deferred by the task's user constraint.
use super::*;
use crate::test_support::TestHome;
use serde_json::json;
const SCOPE: &str = "https://auth.x.ai::client";
const SENTINEL: &str = "SECRET_SENTINEL_DO_NOT_DISPLAY";
fn value(user: &str, key: &str) -> Value {
    json!({"auth_mode":"oidc","oidc_issuer":"https://auth.x.ai","oidc_client_id":"client",
        "key":key,"user_id":user,"create_time":"2026-01-01T00:00:00Z","future":{"nested":true}})
}
fn fixture() -> (TestHome, Paths, GrokAuthService) {
    let mut home = TestHome::new_with_home_env();
    let native = home.home().join("grok");
    std::fs::create_dir_all(&native).unwrap();
    home.set_env("GROK_HOME", native.as_os_str());
    (home, paths().unwrap(), GrokAuthService::new())
}
fn runtime(p: &Paths, v: Value) {
    std::fs::write(
        &p.runtime,
        json!({(SCOPE):v,"xai::api_key":{"key":"keep"}}).to_string(),
    )
    .unwrap();
}
fn save(s: &GrokAuthService, name: &str) {
    s.save_current(name, SCOPE, &s.read_snapshot().unwrap().revision, false)
        .unwrap();
}
#[test]
fn grok_auth_save_does_not_take_official_lock_or_write_runtime() {
    let (_home, p, s) = fixture();
    runtime(&p, value("a", SENTINEL));
    let before = std::fs::read(&p.runtime).unwrap();
    save(&s, "a");
    assert!(!p.runtime.with_extension("json.lock").exists());
    let _official = FileLock::new(p.runtime.with_extension("json.lock"), Duration::ZERO).unwrap();
    let revision = s.read_snapshot().unwrap().revision;
    s.save_current("a", SCOPE, &revision, true).unwrap();
    assert_eq!(std::fs::read(&p.runtime).unwrap(), before);
    assert!(!format!("{:?}", s.read_snapshot().unwrap()).contains(SENTINEL));
}
#[test]
fn grok_auth_switch_round_trip_preserves_refresh_and_other_scope() {
    let (_home, p, s) = fixture();
    runtime(&p, value("a", "old"));
    save(&s, "a");
    runtime(&p, value("b", "b"));
    save(&s, "b");
    runtime(&p, value("a", "refreshed"));
    s.switch_account("b", &s.read_snapshot().unwrap().revision)
        .unwrap();
    s.switch_account("a", &s.read_snapshot().unwrap().revision)
        .unwrap();
    let doc = document(read(&p.runtime).unwrap().as_deref()).unwrap();
    assert_eq!(doc[SCOPE], value("a", "refreshed"));
    assert_eq!(doc["xai::api_key"], json!({"key":"keep"}));
}
#[test]
fn grok_auth_stale_revision_and_uncertain_identity_do_not_write_runtime() {
    let (_home, p, s) = fixture();
    runtime(&p, value("a", "a"));
    save(&s, "a");
    let stale = s.read_snapshot().unwrap().revision;
    runtime(&p, value("unknown", SENTINEL));
    let before = std::fs::read(&p.runtime).unwrap();
    assert!(s.switch_account("a", &stale).is_err());
    let err = s
        .switch_account("a", &s.read_snapshot().unwrap().revision)
        .unwrap_err();
    assert!(!err.to_string().contains(SENTINEL));
    assert_eq!(std::fs::read(&p.runtime).unwrap(), before);
}
#[test]
fn grok_auth_identity_enrichment_and_duplicate_candidates_are_not_inferred() {
    let mut store = Store::default();
    let a = value("a", "old");
    store.accounts.insert("a".into(), captured(SCOPE, &a));
    let mut enriched = value("a", "new");
    enriched["team_id"] = json!("new-team");
    assert!(matching(&store, SCOPE, &enriched).unwrap().is_none());
    assert_eq!(
        matching(&store, SCOPE, &value("a", "new"))
            .unwrap()
            .as_deref(),
        Some("a")
    );
    store
        .accounts
        .insert("duplicate".into(), captured(SCOPE, &a));
    assert!(matching(&store, SCOPE, &a).unwrap().is_none());
}
#[test]
fn grok_auth_delete_is_store_only_and_off_preserves_accounts() {
    let (_home, p, s) = fixture();
    runtime(&p, value("a", "a"));
    save(&s, "a");
    let before = std::fs::read(&p.runtime).unwrap();
    s.delete_account("a", &s.read_snapshot().unwrap().revision)
        .unwrap();
    assert_eq!(std::fs::read(&p.runtime).unwrap(), before);
    save(&s, "a");
    runtime(&p, value("a", "new"));
    s.off_checked(&s.read_snapshot().unwrap().revision).unwrap();
    assert!(!p.runtime.exists());
    assert_eq!(
        saved_value(&load(&p).unwrap().store.accounts["a"]).unwrap(),
        value("a", "new")
    );
}
#[test]
fn grok_auth_corruption_blocks_overwrite_but_explicit_runtime_off_works() {
    let (_home, p, s) = fixture();
    runtime(&p, value("a", "a"));
    save(&s, "a");
    std::fs::write(&p.runtime, b"{damaged SECRET_SENTINEL_DO_NOT_DISPLAY").unwrap();
    let snapshot = s.read_snapshot().unwrap();
    assert!(snapshot.runtime_error.is_some());
    assert!(s.switch_account("a", &snapshot.revision).is_err());
    s.off_checked(&snapshot.revision).unwrap();
    runtime(&p, value("a", "a"));
    std::fs::write(&p.store, b"{damaged").unwrap();
    assert!(s.off().is_err());
    assert!(p.runtime.exists());
}
#[test]
fn grok_auth_store_cas_failure_preserves_existing_bytes() {
    let (_home, p, s) = fixture();
    runtime(&p, value("a", "a"));
    save(&s, "a");
    let stale = load(&p).unwrap();
    std::fs::write(&p.store, b"external change").unwrap();
    assert!(save_store(&p, &stale).is_err());
    assert_eq!(std::fs::read(&p.store).unwrap(), b"external change");
}

#[test]
fn grok_auth_unconfirmed_outgoing_durability_blocks_switch_and_off() {
    let (_home, p, s) = fixture();
    runtime(&p, value("a", "old"));
    save(&s, "a");
    runtime(&p, value("b", "b"));
    save(&s, "b");
    runtime(&p, value("a", "latest"));
    let before = std::fs::read(&p.runtime).unwrap();
    STORE_DURABILITY_WARNING.with(|flag| flag.set(true));
    let error = s
        .switch_account("b", &s.read_snapshot().unwrap().revision)
        .unwrap_err();
    assert!(error.to_string().contains("durability"));
    assert_eq!(std::fs::read(&p.runtime).unwrap(), before);
    assert_eq!(
        saved_value(&load(&p).unwrap().store.accounts["a"]).unwrap(),
        value("a", "latest")
    );
    STORE_DURABILITY_WARNING.with(|flag| flag.set(true));
    assert!(s.off().is_err());
    assert_eq!(std::fs::read(&p.runtime).unwrap(), before);
}

#[test]
fn grok_auth_unsupported_outgoing_never_poison_saved_account() {
    let (_home, p, s) = fixture();
    runtime(&p, value("a", "old"));
    save(&s, "a");
    runtime(&p, value("b", "b"));
    save(&s, "b");
    let store_before = std::fs::read(&p.store).unwrap();
    for (field, invalid) in [
        ("auth_mode", "external"),
        ("oidc_issuer", "https://enterprise.invalid"),
        ("oidc_client_id", "other"),
    ] {
        let mut outgoing = value("a", "new");
        outgoing[field] = json!(invalid);
        runtime(&p, outgoing);
        let before = std::fs::read(&p.runtime).unwrap();
        assert!(
            s.switch_account("b", &s.read_snapshot().unwrap().revision)
                .is_err()
        );
        assert_eq!(std::fs::read(&p.runtime).unwrap(), before);
        assert_eq!(std::fs::read(&p.store).unwrap(), store_before);
    }
}

#[test]
fn grok_auth_known_native_types_reject_invalid_but_preserve_unknown_fields() {
    assert!(credential(SCOPE, &value("a", "key")));
    for (field, invalid) in [
        ("expires_at", json!("not-a-date")),
        ("email", json!(42)),
        ("user_id", Value::Null),
        ("team_blocked_reasons", Value::Null),
        ("coding_data_retention_opt_out", Value::Null),
        ("has_grok_code_access", json!("yes")),
    ] {
        let mut entry = value("a", "key");
        entry[field] = invalid;
        assert!(!credential(SCOPE, &entry));
    }
}

#[test]
fn grok_auth_save_captured_snapshot_does_not_revert_native_refresh() {
    let (_home, p, s) = fixture();
    runtime(&p, value("a", "captured"));
    let config = p.runtime.with_file_name("config.toml");
    let mcp = p.runtime.with_file_name("mcp_credentials.json");
    let profile = p
        .store
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("profiles.toml");
    std::fs::create_dir_all(p.store.parent().unwrap()).unwrap();
    for path in [&config, &mcp, &profile] {
        std::fs::write(path, b"unchanged").unwrap();
    }
    BEFORE_STORE_WRITE.with(|slot| slot.set(Some(|p| runtime(p, value("a", "native-refresh")))));
    save(&s, "a");
    assert_eq!(
        saved_value(&load(&p).unwrap().store.accounts["a"]).unwrap(),
        value("a", "captured")
    );
    assert_eq!(
        document(read(&p.runtime).unwrap().as_deref()).unwrap()[SCOPE],
        value("a", "native-refresh")
    );
    for path in [&config, &mcp, &profile] {
        assert_eq!(std::fs::read(path).unwrap(), b"unchanged");
    }
    assert!(!p.runtime.with_extension("json.lock").exists());
}

#[test]
fn grok_auth_store_write_failure_blocks_destructive_operations() {
    let (_home, p, s) = fixture();
    runtime(&p, value("a", "old"));
    save(&s, "a");
    runtime(&p, value("b", "b"));
    save(&s, "b");
    runtime(&p, value("a", "latest"));
    let before = std::fs::read(&p.runtime).unwrap();
    let original_store = std::fs::read(&p.store).unwrap();
    for off in [false, true] {
        BEFORE_STORE_WRITE.with(|slot| {
            slot.set(Some(|p| {
                std::fs::remove_file(&p.store).unwrap();
                std::fs::create_dir(&p.store).unwrap();
            }))
        });
        let revision = s.read_snapshot().unwrap().revision;
        let failed = if off {
            s.off_checked(&revision).is_err()
        } else {
            s.switch_account("b", &revision).is_err()
        };
        assert!(failed);
        assert_eq!(std::fs::read(&p.runtime).unwrap(), before);
        std::fs::remove_dir(&p.store).unwrap();
        std::fs::write(&p.store, &original_store).unwrap();
    }
}

#[test]
fn grok_auth_runtime_cas_failure_keeps_latest_outgoing_saved() {
    let (_home, p, s) = fixture();
    runtime(&p, value("a", "old"));
    save(&s, "a");
    runtime(&p, value("b", "b"));
    save(&s, "b");
    runtime(&p, value("a", "latest"));
    BEFORE_STORE_WRITE.with(|slot| slot.set(Some(|p| runtime(p, value("external", "external")))));
    assert!(
        s.switch_account("b", &s.read_snapshot().unwrap().revision)
            .is_err()
    );
    assert_eq!(
        saved_value(&load(&p).unwrap().store.accounts["a"]).unwrap(),
        value("a", "latest")
    );
    assert_eq!(
        document(read(&p.runtime).unwrap().as_deref()).unwrap()[SCOPE],
        value("external", "external")
    );
}

#[test]
fn grok_auth_all_mutations_wait_for_the_same_operation_lock() {
    let (_home, p, s) = fixture();
    for operation in 0..4 {
        runtime(&p, value("a", "current"));
        s.save_current("a", SCOPE, &s.read_snapshot().unwrap().revision, true)
            .unwrap();
        let revision = s.read_snapshot().unwrap().revision;
        let held = FileLock::new(&p.operation, Duration::ZERO).unwrap();
        let before = std::fs::read(&p.runtime).unwrap();
        let (sent, received) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            let service = GrokAuthService::new();
            let result = match operation {
                0 => service
                    .save_current("a", SCOPE, &revision, true)
                    .map(|_| ()),
                1 => service.switch_account("a", &revision).map(|_| ()),
                2 => service.delete_account("a", &revision).map(|_| ()),
                _ => service.off_checked(&revision).map(|_| ()),
            };
            sent.send(result).unwrap();
        });
        assert!(received.recv_timeout(Duration::from_millis(25)).is_err());
        assert_eq!(std::fs::read(&p.runtime).unwrap(), before);
        drop(held);
        received
            .recv_timeout(Duration::from_secs(5))
            .unwrap()
            .unwrap();
        worker.join().unwrap();
    }
}

#[test]
fn grok_auth_runtime_postwrite_error_reports_applied_unconfirmed() {
    let (_home, p, s) = fixture();
    runtime(&p, value("a", "a"));
    save(&s, "a");
    runtime(&p, value("b", "b"));
    save(&s, "b");
    WRITE_ERROR_AFTER_WRITE.with(|slot| *slot.borrow_mut() = Some(p.runtime.clone()));
    let result = s
        .switch_account("a", &s.read_snapshot().unwrap().revision)
        .unwrap();
    assert!(result.outgoing_saved);
    assert!(
        result
            .warnings
            .iter()
            .any(|warning| warning.contains("durability"))
    );
    assert_eq!(
        document(read(&p.runtime).unwrap().as_deref()).unwrap()[SCOPE],
        value("a", "a")
    );
}

#[cfg(windows)]
#[test]
fn grok_auth_runtime_replacement_denied_keeps_latest_outgoing_saved() {
    use std::os::windows::fs::OpenOptionsExt;

    let (_home, p, s) = fixture();
    runtime(&p, value("a", "old"));
    save(&s, "a");
    runtime(&p, value("b", "b"));
    save(&s, "b");
    runtime(&p, value("a", "latest"));
    let before = std::fs::read(&p.runtime).unwrap();
    // Permit reads/writes, including the version check, but exclude DELETE
    // sharing. MoveFileExW must fail while this handle remains open.
    let held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0x0000_0001 | 0x0000_0002)
        .open(&p.runtime)
        .unwrap();
    let error = s
        .switch_account("b", &s.read_snapshot().unwrap().revision)
        .unwrap_err();
    assert!(error.to_string().contains("Outgoing account was saved"));
    assert_eq!(std::fs::read(&p.runtime).unwrap(), before);
    assert_eq!(
        saved_value(&load(&p).unwrap().store.accounts["a"]).unwrap(),
        value("a", "latest")
    );
    drop(held);
}
