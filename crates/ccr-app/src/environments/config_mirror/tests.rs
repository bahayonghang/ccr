#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::{
    environments::{
        io::EnvironmentIo,
        ssh::{HostInput, Workspace},
    },
    runtime::EnvironmentId,
};
use std::sync::Arc;

#[tokio::test]
async fn binary_mirror_cas_backup_partial_cleanup_and_cancellation_are_truthful() {
    let root = tempfile::tempdir().unwrap();
    let (workspace, transport) =
        Workspace::fixture(root.path().join("ssh.db"), root.path().join("known_hosts"));
    let workspace = Arc::new(workspace);
    let host = workspace
        .save_host(HostInput {
            name: "mirror".into(),
            host: "mirror.fixture.invalid".into(),
            user: "fixture".into(),
            port: 22,
            remote_home: "/srv/mirror".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    let probe = workspace
        .probe(&host.id, CancellationToken::new())
        .await
        .unwrap();
    workspace
        .confirm(&probe.challenge_id, CancellationToken::new())
        .await
        .unwrap();
    workspace
        .connect(&host.id, CancellationToken::new())
        .await
        .unwrap();
    let io = EnvironmentIo::resolve(
        &EnvironmentId::Ssh(host.id),
        || panic!("local roots must not run"),
        Some(workspace.clone()),
    )
    .unwrap();
    transport.set_bytes(
        "mirror.fixture.invalid",
        "/srv/mirror/one.json",
        &[0, 255, 1],
    );
    transport.set_file("mirror.fixture.invalid", "/srv/mirror/two.json", "old-two");
    let paths = BTreeSet::from(["one.json".into(), "two.json".into()]);
    let mirror = ConfigMirror::capture(io.clone(), paths.clone(), None, CancellationToken::new())
        .await
        .unwrap();
    let private = mirror.root().to_path_buf();
    assert_eq!(
        std::fs::read(private.join("one.json")).unwrap(),
        vec![0, 255, 1]
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&private).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
    std::fs::write(private.join("one.json"), "new-one").unwrap();
    let cancel = CancellationToken::new();
    cancel.cancel();
    assert!(matches!(
        mirror.publish(cancel).await,
        Err(Error::Cancelled | Error::Ssh("ssh_cancelled"))
    ));
    assert_eq!(
        transport.bytes("mirror.fixture.invalid", "/srv/mirror/one.json"),
        Some(vec![0, 255, 1])
    );
    transport.set_file("mirror.fixture.invalid", "/srv/mirror/two.json", "external");
    assert_eq!(
        mirror.publish(CancellationToken::new()).await,
        Err(Error::Conflict)
    );
    assert!(
        !transport
            .batches()
            .iter()
            .any(|line| line.starts_with("put "))
    );
    drop(mirror);
    assert!(!private.exists());

    let mirror = ConfigMirror::capture(io, paths, None, CancellationToken::new())
        .await
        .unwrap();
    std::fs::write(mirror.root().join("one.json"), "new-one").unwrap();
    std::fs::write(mirror.root().join("two.json"), "new-two").unwrap();
    transport.fail_path(
        "rename",
        "/srv/mirror/two.json",
        "ssh_write_outcome_unknown",
    );
    assert_eq!(
        mirror.publish(CancellationToken::new()).await,
        Err(Error::Partial)
    );
    assert_eq!(
        transport
            .file("mirror.fixture.invalid", "/srv/mirror/one.json")
            .as_deref(),
        Some("new-one")
    );
    assert_eq!(
        transport
            .file("mirror.fixture.invalid", "/srv/mirror/two.json")
            .as_deref(),
        Some("external")
    );
    let batches = transport.batches();
    for target in ["one.json", "two.json"] {
        assert!(
            batches
                .iter()
                .any(|line| line.starts_with(&format!("rename /srv/mirror/{target}.bak.")))
        );
    }
    let EnvironmentId::Ssh(id) = mirror.io.environment() else {
        panic!("SSH environment")
    };
    assert!(!workspace.connected(id));
}
