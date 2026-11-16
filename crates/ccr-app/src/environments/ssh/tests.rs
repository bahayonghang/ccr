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
