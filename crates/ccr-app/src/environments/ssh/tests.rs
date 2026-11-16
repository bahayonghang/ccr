#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
fn fixture() -> (tempfile::TempDir, Workspace, Arc<fixture::FixtureTransport>) {
    let root = tempfile::tempdir().unwrap();
    let db = root.path().join("ssh.db");
    let conn = ssh_repo::open_host_store(&db, true).unwrap();
    for id in ["one", "two"] {
        ssh_repo::insert_host(
            &conn,
            &SshHost {
                id: id.into(),
                name: id.into(),
                host: format!("{id}.example"),
                port: 22,
                username: "fixture".into(),
                identity_file: None,
                remote_home: Some("~".into()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                last_connected_at: None,
            },
        )
        .unwrap();
    }
    let (owner, transport) = Workspace::fixture(db, root.path().join("known_hosts"));
    transport.set_file(
        "one.example",
        "/home/fixture/.claude/settings.json",
        "{\"one\":1}",
    );
    (root, owner, transport)
}
async fn connect(owner: &Workspace) {
    let probe = owner.probe("one", CancellationToken::new()).await.unwrap();
    owner
        .confirm(&probe.challenge_id, CancellationToken::new())
        .await
        .unwrap();
    owner
        .connect("one", CancellationToken::new())
        .await
        .unwrap();
}

#[tokio::test]
async fn temporary_test_trust_preserves_the_current_session_and_one_use_rules() {
    let (root, owner, transport) = fixture();
    connect(&owner).await;
    let input = owner.host_details("two").unwrap();
    let challenge = owner
        .test_connection(input.clone(), CancellationToken::new())
        .await
        .unwrap()
        .unwrap();
    assert!(owner.connected("one"));
    owner
        .confirm_test(&challenge.challenge_id, CancellationToken::new())
        .await
        .unwrap();
    assert!(owner.connected("one"));
    assert_eq!(
        owner
            .confirm_test(&challenge.challenge_id, CancellationToken::new())
            .await,
        Err("ssh_challenge_invalid")
    );
    assert!(
        owner
            .test_connection(input.clone(), CancellationToken::new())
            .await
            .unwrap()
            .is_none()
    );
    assert!(owner.connected("one"));
    assert_eq!(transport.handshakes(), 2);
    let before = std::fs::read(root.path().join("known_hosts")).unwrap();
    transport.change_key();
    assert!(matches!(
        owner.test_connection(input, CancellationToken::new()).await,
        Err("ssh_host_key_mismatch")
    ));
    assert_eq!(
        std::fs::read(root.path().join("known_hosts")).unwrap(),
        before
    );
    assert!(owner.connected("one"));
}

#[tokio::test]
async fn saving_host_does_not_read_an_unrelated_unavailable_credential() {
    let (_root, owner, _transport) = fixture();
    let mut other = owner.host_details("two").unwrap();
    other.auth_method = AuthMethod::Password;
    owner.save_host(other).await.unwrap();
    let store = owner.credentials.0.as_ref().unwrap();
    let entry = store.build("CCR SSH", "two:ssh-secret", None).unwrap();
    entry
        .as_any()
        .downcast_ref::<keyring_core::mock::Cred>()
        .unwrap()
        .set_error(keyring_core::Error::Invalid(
            "synthetic".into(),
            "fixture unavailable".into(),
        ));
    let mut first = owner.host_details("one").unwrap();
    first.name = "Updated fixture".into();
    let saved = owner.save_host(first).await.unwrap();
    assert_eq!(saved.name, "Updated fixture");
    assert_eq!(owner.host_details("one").unwrap().name, "Updated fixture");
    assert!(
        matches!(
            entry.get_password(),
            Err(keyring_core::Error::Invalid(_, _))
        ),
        "unrelated credential failure must remain unconsumed"
    );
}

#[tokio::test]
async fn legacy_host_metadata_projects_authentication_without_mutating_the_database() {
    let root = tempfile::tempdir().unwrap();
    let database = root.path().join("legacy.db");
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection.execute_batch("CREATE TABLE ssh_hosts (id TEXT PRIMARY KEY, name TEXT, host TEXT, port INTEGER, username TEXT, identity_file TEXT, remote_home TEXT, created_at TEXT, updated_at TEXT, last_connected_at TEXT);
        INSERT INTO ssh_hosts VALUES ('agent','Agent fixture','agent.fixture.invalid',22,'fixture',NULL,'~','2026-11-16T00:00:00Z','2026-11-16T00:00:00Z',NULL);
        INSERT INTO ssh_hosts VALUES ('key','Key fixture','key.fixture.invalid',2222,'fixture','C:/synthetic/key','~','2026-11-16T00:00:00Z','2026-11-16T00:00:00Z',NULL);").unwrap();
    drop(connection);
    let before = std::fs::read(&database).unwrap();
    let (owner, _) = Workspace::fixture(database.clone(), root.path().join("known_hosts"));
    let hosts = owner.management_hosts().await.unwrap();
    assert_eq!(hosts.len(), 2);
    assert_eq!(
        hosts
            .iter()
            .find(|host| host.id == "agent")
            .unwrap()
            .auth_method,
        AuthMethod::Agent
    );
    assert_eq!(
        hosts
            .iter()
            .find(|host| host.id == "key")
            .unwrap()
            .auth_method,
        AuthMethod::PrivateKey
    );
    assert_eq!(std::fs::read(&database).unwrap(), before);
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection.execute_batch("ALTER TABLE ssh_hosts ADD COLUMN auth_method TEXT; UPDATE ssh_hosts SET auth_method='invalid';").unwrap();
    drop(connection);
    assert!(matches!(owner.hosts(), Err("ssh_invalid_target")));
}

#[tokio::test]
async fn home_is_validated_cached_per_session_and_explicit_override_has_priority() {
    let (_root, owner, transport) = fixture();
    connect(&owner).await;
    let before = transport.batches();
    assert_eq!(
        owner.capture_io("one").unwrap().home().as_str(),
        "/home/fixture"
    );
    assert_eq!(
        owner.capture_io("one").unwrap().home().as_str(),
        "/home/fixture"
    );
    assert_eq!(transport.batches(), before);
    for home in [
        "~",
        "C:/home",
        "/home/a/../b",
        "/home//user",
        "//home/user",
        "/home/with space",
        "/home/user\n",
    ] {
        transport.set_home(home);
        assert_eq!(
            owner.connect("one", CancellationToken::new()).await,
            Err("ssh_invalid_remote_path")
        );
        assert!(!owner.connected("one"));
    }
    let mut host = owner.host("one").unwrap();
    host.remote_home = Some("/custom/home".into());
    ssh_repo::update_host(&owner.connection(true).unwrap(), &host).unwrap();
    let home_probes = transport
        .batches()
        .iter()
        .filter(|s| s.starts_with("home "))
        .count();
    owner
        .connect("one", CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(
        owner.capture_io("one").unwrap().home().as_str(),
        "/custom/home"
    );
    assert_eq!(
        transport
            .batches()
            .iter()
            .filter(|s| s.starts_with("home "))
            .count(),
        home_probes
    );
    transport.set_file("one.example", "/custom/home/.claude/settings.json", "{}");
    assert_eq!(
        owner
            .read("one", ConfigFile::Claude, CancellationToken::new())
            .await
            .unwrap()
            .content,
        "{}"
    );
}

#[tokio::test]
async fn home_rejection_reports_failed_disconnect_before_business_error() {
    let (_root, owner, transport) = fixture();
    connect(&owner).await;
    owner.disconnect().await;
    transport.set_home("//invalid/home");
    transport.fail("disconnect");
    assert_eq!(
        owner.connect("one", CancellationToken::new()).await,
        Err("ssh_cleanup_failed")
    );
    assert_eq!(owner.retained_session_count(), 0);
}
#[tokio::test]
async fn trust_three_states_no_implicit_writes_and_changed_confirmation_rejected() {
    let (root, owner, transport) = fixture();
    assert_eq!(
        owner.connect("one", CancellationToken::new()).await,
        Err("ssh_host_key_untrusted")
    );
    let probe = owner.probe("one", CancellationToken::new()).await.unwrap();
    assert_eq!(probe.status, HostKeyStatus::New);
    assert!(!root.path().join("known_hosts").exists());
    connect(&owner).await;
    let persisted = std::fs::read(root.path().join("known_hosts")).unwrap();
    assert_eq!(
        owner
            .probe("one", CancellationToken::new())
            .await
            .unwrap()
            .status,
        HostKeyStatus::Matched
    );
    transport.change_key();
    let probe = owner.probe("one", CancellationToken::new()).await.unwrap();
    assert_eq!(probe.status, HostKeyStatus::Mismatch);
    assert_eq!(
        owner
            .confirm(&probe.challenge_id, CancellationToken::new())
            .await,
        Err("ssh_host_key_mismatch")
    );
    assert_eq!(
        owner.connect("one", CancellationToken::new()).await,
        Err("ssh_host_key_mismatch")
    );
    assert_eq!(
        std::fs::read(root.path().join("known_hosts")).unwrap(),
        persisted
    );
    assert!(!owner.connected("one"));
}
#[tokio::test]
async fn one_handshake_multiple_channels_backup_conflict_and_session_authority() {
    let (_root, owner, transport) = fixture();
    connect(&owner).await;
    let original = owner
        .read("one", ConfigFile::Claude, CancellationToken::new())
        .await
        .unwrap();
    let saved = owner
        .save(&original, "{}", CancellationToken::new())
        .await
        .unwrap();
    owner
        .read("one", ConfigFile::Claude, CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(transport.handshakes(), 1);
    assert!(
        transport
            .batches()
            .iter()
            .any(|b| b.starts_with("rename ") && b.contains(".bak."))
    );
    assert!(matches!(
        owner.save(&original, "{}", CancellationToken::new()).await,
        Err("ssh_conflict")
    ));
    owner
        .connect("one", CancellationToken::new())
        .await
        .unwrap();
    assert!(matches!(
        owner.save(&saved, "{}", CancellationToken::new()).await,
        Err("ssh_session_changed")
    ));
    owner.disconnect().await;
    assert_eq!(owner.retained_session_count(), 0);
}
#[tokio::test]
async fn unsupported_atomic_rename_and_failed_upload_leave_original_intact() {
    for unsupported in [true, false] {
        let (_root, owner, transport) = fixture();
        connect(&owner).await;
        let document = owner
            .read("one", ConfigFile::Claude, CancellationToken::new())
            .await
            .unwrap();
        if unsupported {
            transport.disable_posix_rename();
        } else {
            transport.fail("rename");
        }
        assert!(matches!(
            owner.save(&document, "{}", CancellationToken::new()).await,
            Err("ssh_write_outcome_unknown")
        ));
        assert_eq!(
            transport.file("one.example", "/home/fixture/.claude/settings.json"),
            Some(document.content)
        );
        assert!(!owner.connected("one"));
        assert!(
            transport
                .file(
                    "one.example",
                    "/home/fixture/.claude/settings.json.ccr-tmp-fixture"
                )
                .is_none()
        );
    }
}
#[tokio::test]
async fn cleanup_failure_is_not_hidden_by_cancel() {
    let (_root, owner, transport) = fixture();
    connect(&owner).await;
    let document = owner
        .read("one", ConfigFile::Claude, CancellationToken::new())
        .await
        .unwrap();
    transport.fail_with("put", "ssh_cancelled");
    transport.fail_with("-rm", "ssh_cleanup_failed");
    assert!(matches!(
        owner.save(&document, "{}", CancellationToken::new()).await,
        Err("ssh_cleanup_failed")
    ));
}
#[tokio::test]
async fn closed_connection_has_no_liveness_and_never_reconnects_implicitly() {
    let (_root, owner, transport) = fixture();
    connect(&owner).await;
    owner
        .current("one")
        .unwrap()
        .connection
        .disconnect()
        .await
        .unwrap();
    assert!(!owner.connected("one"));
    assert!(matches!(
        owner
            .read("one", ConfigFile::Claude, CancellationToken::new())
            .await,
        Err("ssh_not_connected")
    ));
    assert_eq!(transport.handshakes(), 1);
    owner.disconnect().await;
    assert_eq!(owner.retained_session_count(), 0);
}
#[tokio::test]
async fn malformed_cli_and_cancelled_connect_do_not_publish_authority() {
    let (_root, owner, transport) = fixture();
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert_eq!(owner.connect("one", cancel).await, Err("ssh_cancelled"));
    assert_eq!(transport.handshakes(), 0);
    connect(&owner).await;
    transport.set_cli_output("claude", "banner\n/usr/bin/claude");
    let authority = owner.cli_session("one").unwrap();
    assert_eq!(
        owner.detect_cli(&authority, CancellationToken::new()).await,
        Err("ssh_cli_output_invalid")
    );
    assert!(!owner.connected("one"));
    connect(&owner).await;
    assert_eq!(
        owner.detect_cli(&authority, CancellationToken::new()).await,
        Err("ssh_session_changed")
    );
}
#[test]
fn runtime_cleanup_disconnects_before_runtime_is_dropped() {
    let (_root, workspace, _) = fixture();
    let runtime = crate::runtime::RuntimeOwner::new().unwrap();
    runtime.shutdown_after(async {
        connect(&workspace).await;
        assert!(workspace.connected("one"));
        workspace.disconnect().await;
        assert_eq!(workspace.retained_session_count(), 0);
    });
    assert!(!workspace.connected("one"));
}
#[tokio::test]
async fn concurrent_same_path_writers_only_one_version_wins() {
    let (_root, owner, transport) = fixture();
    connect(&owner).await;
    let document = owner
        .read("one", ConfigFile::Claude, CancellationToken::new())
        .await
        .unwrap();
    let (a, b) = tokio::join!(
        owner.save(&document, "{}", CancellationToken::new()),
        owner.save(&document, "{}", CancellationToken::new())
    );
    assert!(a.is_ok());
    assert!(matches!(b, Err("ssh_conflict")));
    assert_eq!(transport.handshakes(), 1);
}
#[tokio::test]
async fn concurrent_exec_does_not_block_read_and_cancel_revokes_only_old_session() {
    let (_root, owner, transport) = fixture();
    connect(&owner).await;
    transport.delay_cli(std::time::Duration::from_secs(1));
    let authority = owner.cli_session("one").unwrap();
    let cancel = CancellationToken::new();
    let (result, ()) = tokio::join!(owner.detect_cli(&authority, cancel.clone()), async {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        tokio::time::timeout(
            std::time::Duration::from_millis(100),
            owner.read("one", ConfigFile::Claude, CancellationToken::new()),
        )
        .await
        .unwrap()
        .unwrap();
        owner
            .connect("one", CancellationToken::new())
            .await
            .unwrap();
        cancel.cancel();
    });
    assert_eq!(result, Err("ssh_cancelled"));
    assert!(owner.connected("one"));
}
