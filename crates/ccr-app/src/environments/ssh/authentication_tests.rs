#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
fn input(method: AuthMethod) -> HostInput {
    HostInput {
        name: "Synthetic".into(),
        host: "fixture.invalid".into(),
        port: 22,
        user: "fixture".into(),
        remote_home: "~".into(),
        auth_method: method,
        identity_file: (method == AuthMethod::PrivateKey).then(|| "synthetic.ppk".into()),
        secret: (method != AuthMethod::Agent).then(|| Secret::new("fixture-only-secret".into())),
        ..Default::default()
    }
}
#[tokio::test]
async fn all_methods_use_mock_store_keep_secrets_out_of_sql_and_clear_on_delete() {
    for method in [
        AuthMethod::Agent,
        AuthMethod::PrivateKey,
        AuthMethod::Password,
    ] {
        let root = tempfile::tempdir().unwrap();
        let (owner, transport) =
            Workspace::fixture(root.path().join("ssh.db"), root.path().join("known_hosts"));
        let host = owner.save_host(input(method)).await.unwrap();
        let details = owner.host_details(&host.id).unwrap();
        assert_eq!(details.auth_method, method);
        assert!(details.secret.is_none());
        assert_eq!(
            details.identity_file.is_some(),
            method == AuthMethod::PrivateKey
        );
        let probe = owner.probe(&host.id, Default::default()).await.unwrap();
        owner
            .confirm(&probe.challenge_id, Default::default())
            .await
            .unwrap();
        owner.connect(&host.id, Default::default()).await.unwrap();
        assert!(owner.host(&host.id).unwrap().last_connected_at.is_some());
        assert!(
            transport
                .batches()
                .iter()
                .any(|s| s == &format!("{} authentication", method.as_str()))
        );
        assert!(!format!("{:?}", owner.host(&host.id).unwrap()).contains("fixture-only-secret"));
        // 同时检查 SQLite、迁移备份与信任文件，避免只验证当前主库。
        for entry in std::fs::read_dir(root.path()).unwrap().flatten() {
            if entry.path().is_file() {
                let bytes = std::fs::read(entry.path()).unwrap();
                assert!(
                    !bytes
                        .windows(b"fixture-only-secret".len())
                        .any(|v| v == b"fixture-only-secret")
                );
            }
        }
        if method != AuthMethod::Agent {
            assert_eq!(
                owner
                    .credentials
                    .get(&host.id)
                    .await
                    .unwrap()
                    .unwrap()
                    .expose(),
                "fixture-only-secret"
            );
        }
        owner.delete_host(&host.id).await.unwrap();
        assert!(owner.credentials.get(&host.id).await.unwrap().is_none());
        assert!(!owner.connected(&host.id));
    }
}
#[tokio::test]
async fn database_failure_after_authentication_disconnects_the_unpublished_session() {
    for cleanup_fails in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let database = root.path().join("ssh.db");
        let (owner, transport) =
            Workspace::fixture(database.clone(), root.path().join("known_hosts"));
        let host = owner.save_host(input(AuthMethod::Agent)).await.unwrap();
        let probe = owner.probe(&host.id, Default::default()).await.unwrap();
        owner
            .confirm(&probe.challenge_id, Default::default())
            .await
            .unwrap();
        transport.break_database_after_home(database);
        if cleanup_fails {
            transport.fail("disconnect");
        }
        assert!(
            matches!(owner.connect(&host.id, Default::default()).await, Err(error) if error == if cleanup_fails { "ssh_cleanup_failed" } else { "ssh_database_unavailable" })
        );
        assert!(
            transport
                .batches()
                .iter()
                .any(|batch| batch == "disconnect fixture.invalid")
        );
        assert_eq!(owner.retained_session_count(), 0);
    }
}
#[tokio::test]
async fn missing_credentials_and_rejected_authentication_are_distinct_and_do_not_connect() {
    let root = tempfile::tempdir().unwrap();
    let (owner, transport) =
        Workspace::fixture(root.path().join("ssh.db"), root.path().join("known_hosts"));
    let mut missing = input(AuthMethod::Password);
    missing.secret = None;
    let host = owner.save_host(missing).await.unwrap();
    let probe = owner.probe(&host.id, Default::default()).await.unwrap();
    owner
        .confirm(&probe.challenge_id, Default::default())
        .await
        .unwrap();
    assert!(matches!(
        owner.connect(&host.id, Default::default()).await,
        Err("ssh_credentials_missing")
    ));
    let mut supplied = input(AuthMethod::Password);
    supplied.id = Some(host.id.clone());
    owner.save_host(supplied).await.unwrap();
    transport.fail_with("password", "ssh_auth_failed_password");
    assert!(matches!(
        owner.connect(&host.id, Default::default()).await,
        Err("ssh_auth_failed_password")
    ));
    assert!(!owner.connected(&host.id));
    let mut changed = input(AuthMethod::PrivateKey);
    changed.id = Some(host.id.clone());
    changed.secret = None;
    owner.save_host(changed).await.unwrap();
    assert!(owner.credentials.get(&host.id).await.unwrap().is_none());
    assert_eq!(
        owner.host_details(&host.id).unwrap().auth_method,
        AuthMethod::PrivateKey
    );
}
#[tokio::test]
async fn unsaved_test_requires_explicit_trust_and_never_persists_host_or_secret() {
    let root = tempfile::tempdir().unwrap();
    let (owner, _) =
        Workspace::fixture(root.path().join("ssh.db"), root.path().join("known_hosts"));
    let probe = owner
        .test_connection(input(AuthMethod::Password), Default::default())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(probe.status, HostKeyStatus::New);
    assert!(!root.path().join("known_hosts").exists());
    owner
        .confirm(&probe.challenge_id, Default::default())
        .await
        .unwrap();
    assert!(
        owner
            .test_connection(input(AuthMethod::Password), Default::default())
            .await
            .unwrap()
            .is_none()
    );
    assert!(owner.hosts().unwrap().is_empty());
    assert_eq!(owner.retained_session_count(), 0);
    let mut missing = input(AuthMethod::Password);
    missing.secret = None;
    assert!(matches!(
        owner.test_connection(missing, Default::default()).await,
        Err("ssh_credentials_missing")
    ));
}
#[test]
fn tauri_password_boundary_remains_closed_and_desktop_does_not_weaken_trust() {
    use crate::environments::{ConnectFacts, ConnectKind, boundary_before_connect};
    let facts = ConnectFacts {
        cancel_requested: false,
        password_only: true,
        stored_fingerprint: None,
        observed_fingerprint: None,
        process_trusted: false,
    };
    assert_eq!(
        boundary_before_connect(ConnectKind::Open, facts),
        Err("ssh_auth_unsupported")
    );
    assert_eq!(
        super::super::ssh_session::desktop_boundary_before_connect(ConnectKind::Open, facts),
        Ok(())
    );
    assert_eq!(
        super::super::ssh_session::desktop_boundary_before_connect(ConnectKind::Again, facts),
        Err("ssh_host_key_untrusted")
    );
    let secret = Secret::new("fixture-only-secret".into());
    assert_eq!(format!("{secret:?} {secret}"), "[REDACTED] [REDACTED]");
    for code in [
        "ssh_credentials_missing",
        "ssh_auth_failed",
        "ssh_auth_failed_all",
        "ssh_private_key_invalid",
        "ssh_credential_store_unavailable",
    ] {
        for chinese in [false, true] {
            assert!(authentication_message(code, chinese).is_some());
        }
    }
}
#[tokio::test]
async fn legacy_records_migrate_with_verified_backup_and_can_connect() {
    use ccr_db::database::migrations::{run_initial_migration, run_migration_v6};
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("ssh.db");
    let conn = rusqlite::Connection::open(&path).unwrap();
    run_initial_migration(&conn).unwrap();
    run_migration_v6(&conn).unwrap();
    conn.execute_batch("INSERT INTO ssh_hosts VALUES ('agent','Agent','fixture.invalid',22,'fixture',NULL,'~','2026-01-01','2026-01-01',NULL); INSERT INTO ssh_hosts VALUES ('key','Key','key.invalid',22,'fixture','synthetic.pem','~','2026-01-01','2026-01-01',NULL);").unwrap();
    drop(conn);
    let (owner, _) = Workspace::fixture(path, root.path().join("known_hosts"));
    for (id, method) in [
        ("agent", AuthMethod::Agent),
        ("key", AuthMethod::PrivateKey),
    ] {
        assert_eq!(owner.host_details(id).unwrap().auth_method, method);
        let probe = owner.probe(id, Default::default()).await.unwrap();
        owner
            .confirm(&probe.challenge_id, Default::default())
            .await
            .unwrap();
        owner.connect(id, Default::default()).await.unwrap();
    }
    let backups: Vec<_> = std::fs::read_dir(root.path())
        .unwrap()
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .contains("pre-migration-v18")
        })
        .collect();
    assert_eq!(backups.len(), 1);
    let backup = rusqlite::Connection::open(backups[0].path()).unwrap();
    assert_eq!(
        backup
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert!(backup.prepare("SELECT auth_method FROM ssh_hosts").is_err());
}
