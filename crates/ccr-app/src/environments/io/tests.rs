#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::super::ssh::{HostInput, fixture::FixtureTransport};
use super::*;

struct Harness {
    root: tempfile::TempDir,
    io: EnvironmentIo,
    remote: Option<(Arc<Workspace>, Arc<FixtureTransport>)>,
}
impl Harness {
    async fn new(remote: bool) -> Self {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("home/.claude")).unwrap();
        std::fs::write(
            root.path().join("home/.claude/settings.json"),
            "local decoy",
        )
        .unwrap();
        let home = root.path().join("home");
        let locks = root.path().join("locks");
        let (environment, remote) = if remote {
            let (workspace, transport) =
                Workspace::fixture(root.path().join("ssh.db"), root.path().join("known_hosts"));
            let workspace = Arc::new(workspace);
            let host = workspace
                .save_host(HostInput {
                    name: "fixture".into(),
                    host: "fixture.invalid".into(),
                    user: "fixture".into(),
                    port: 22,
                    remote_home: "~".into(),
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
            transport.set_file(
                "fixture.invalid",
                "/home/fixture/.claude/settings.json",
                "remote original",
            );
            (EnvironmentId::Ssh(host.id), Some((workspace, transport)))
        } else {
            (EnvironmentId::Local("fixture".into()), None)
        };
        let io = EnvironmentIo::resolve(
            &environment,
            || LocalRoots::new(home, locks),
            remote.as_ref().map(|(w, _)| w.clone()),
        )
        .unwrap();
        Self { root, io, remote }
    }

    fn backups(&self) -> Vec<String> {
        if let Some((_, transport)) = &self.remote {
            transport
                .batches()
                .into_iter()
                .filter_map(|b| {
                    b.strip_prefix("rename ")
                        .filter(|p| p.contains(".bak."))
                        .and_then(|p| transport.file("fixture.invalid", p))
                })
                .collect()
        } else {
            std::fs::read_dir(self.root.path().join("home/.claude"))
                .unwrap()
                .filter_map(|e| {
                    let path = e.unwrap().path();
                    path.extension()
                        .is_some_and(|ext| ext == "bak")
                        .then(|| std::fs::read_to_string(path).unwrap())
                })
                .collect()
        }
    }
}

#[tokio::test]
async fn local_and_ssh_share_backup_atomic_replacement_cas_and_missing_file_contract() {
    for remote in [false, true] {
        let h = Harness::new(remote).await;
        let original =
            h.io.read(".claude/settings.json", CancellationToken::new())
                .await
                .unwrap();
        assert_eq!(
            original.content,
            if remote {
                "remote original"
            } else {
                "local decoy"
            }
        );
        assert!(original.exists);
        let replacement = "new complete document".repeat(4096);
        let saved =
            h.io.write(&original, &replacement, CancellationToken::new())
                .await
                .unwrap();
        assert_eq!(saved.content, replacement);
        assert_eq!(
            h.io.read(".claude/settings.json", CancellationToken::new())
                .await
                .unwrap()
                .content,
            replacement
        );
        assert_eq!(h.backups(), vec![original.content.clone()]);
        assert!(matches!(
            h.io.write(&original, "stale", CancellationToken::new())
                .await,
            Err(Error::Conflict)
        ));
        assert_eq!(h.backups(), vec![original.content]);
        let missing =
            h.io.read(".claude/new.json", CancellationToken::new())
                .await
                .unwrap();
        assert!(!missing.exists);
        assert_eq!(missing.version(), "");
        h.io.write(&missing, "created", CancellationToken::new())
            .await
            .unwrap();
        assert!(matches!(
            h.io.write(&missing, "stale create", CancellationToken::new())
                .await,
            Err(Error::Conflict)
        ));
        if remote {
            assert_eq!(
                std::fs::read_to_string(h.root.path().join("home/.claude/settings.json")).unwrap(),
                "local decoy"
            );
            assert!(!h.root.path().join("home/.claude/new.json").exists());
        }
    }
}

#[tokio::test]
async fn same_snapshot_concurrent_writers_have_one_winner_on_both_adapters() {
    for remote in [false, true] {
        let h = Harness::new(remote).await;
        let original =
            h.io.read(".claude/settings.json", CancellationToken::new())
                .await
                .unwrap();
        let (a, b) = tokio::join!(
            h.io.write(&original, "a", CancellationToken::new()),
            h.io.write(&original, "b", CancellationToken::new())
        );
        assert_eq!(usize::from(a.is_ok()) + usize::from(b.is_ok()), 1);
        assert!(matches!(a, Err(Error::Conflict)) || matches!(b, Err(Error::Conflict)));
        assert_eq!(h.backups(), vec![original.content]);
    }
}

#[tokio::test]
async fn paths_snapshots_cache_and_cancellation_never_retarget() {
    for remote in [false, true] {
        let h = Harness::new(remote).await;
        for bad in [
            "../secret",
            "/etc/passwd",
            "C:/secret",
            "a\\b",
            "a//b",
            "a/./b",
            "~/.claude",
            "a\n",
        ] {
            assert!(matches!(
                h.io.read(bad, CancellationToken::new()).await,
                Err(Error::InvalidPath)
            ));
        }
        let original =
            h.io.read(".claude/settings.json", CancellationToken::new())
                .await
                .unwrap();
        let other = Harness::new(remote).await;
        assert!(matches!(
            other
                .io
                .write(&original, "cross authority", CancellationToken::new())
                .await,
            Err(Error::StaleAuthority)
        ));
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        assert!(matches!(
            h.io.write(&original, "cancelled", cancelled.clone()).await,
            Err(Error::Cancelled)
        ));
        assert!(matches!(
            h.io.read(".claude/settings.json", cancelled).await,
            Err(Error::Cancelled)
        ));
        assert_eq!(
            h.io.read(".claude/settings.json", CancellationToken::new())
                .await
                .unwrap()
                .content,
            original.content
        );
        assert!(h.backups().is_empty());
        let cache = h.io.cache_root(h.root.path()).unwrap();
        assert!(cache.starts_with(h.root.path()));
        assert!(!cache.exists());
        assert_eq!(
            h.io.cache_root(Path::new("relative")),
            Err(Error::InvalidPath)
        );
    }
}

#[tokio::test]
async fn ssh_authority_is_captured_before_queueing_and_never_reads_local_roots() {
    let h = Harness::new(true).await;
    let (workspace, transport) = h.remote.as_ref().unwrap();
    let fresh = || {
        EnvironmentIo::resolve(
            &h.io.environment,
            || panic!("SSH must not inspect local roots"),
            Some(workspace.clone()),
        )
    };
    let captured = fresh().unwrap();
    assert_eq!(
        captured.roots().platform(Platform::OpenCode).unwrap(),
        EnvironmentPath::Ssh(
            RemotePosixPath::root(Some("/home/fixture/.config/opencode")).unwrap()
        )
    );
    let snapshot = captured
        .read(".claude/settings.json", CancellationToken::new())
        .await
        .unwrap();
    workspace.disconnect().await;
    assert!(matches!(
        captured
            .read(".claude/settings.json", CancellationToken::new())
            .await,
        Err(Error::Ssh("ssh_not_connected"))
    ));
    assert!(matches!(fresh(), Err(Error::Ssh("ssh_not_connected"))));
    let EnvironmentId::Ssh(id) = &h.io.environment else {
        panic!()
    };
    workspace
        .connect(id, CancellationToken::new())
        .await
        .unwrap();
    let batches = transport.batches();
    assert!(matches!(
        captured
            .read(".claude/settings.json", CancellationToken::new())
            .await,
        Err(Error::Ssh("ssh_session_changed"))
    ));
    assert!(matches!(
        captured
            .write(&snapshot, "old request", CancellationToken::new())
            .await,
        Err(Error::Ssh("ssh_session_changed"))
    ));
    assert_eq!(transport.batches(), batches);
    assert!(
        fresh()
            .unwrap()
            .read(".claude/settings.json", CancellationToken::new())
            .await
            .is_ok()
    );
    assert_eq!(
        std::fs::read_to_string(h.root.path().join("home/.claude/settings.json")).unwrap(),
        "local decoy"
    );
}

#[tokio::test]
async fn ssh_backup_failure_and_unsupported_atomic_rename_leave_original_untouched() {
    for failure in ["put", "rename", "unsupported"] {
        let h = Harness::new(true).await;
        let (_, transport) = h.remote.as_ref().unwrap();
        let original =
            h.io.read(".claude/settings.json", CancellationToken::new())
                .await
                .unwrap();
        if failure == "unsupported" {
            transport.disable_posix_rename();
        } else {
            transport.fail(failure);
        }
        assert!(
            h.io.write(&original, "not committed", CancellationToken::new())
                .await
                .is_err()
        );
        assert_eq!(
            transport
                .file("fixture.invalid", "/home/fixture/.claude/settings.json")
                .as_deref(),
            Some("remote original")
        );
        assert!(
            !transport
                .batches()
                .iter()
                .any(|s| s == "rename /home/fixture/.claude/settings.json")
        );
    }
}

#[tokio::test]
async fn process_seam_keeps_local_gateway_and_quotes_remote_argv_without_local_lookup() {
    let local = Harness::new(false).await;
    let descriptor = ProcessDescriptor::cli_probe(
        "ccr",
        std::env::current_exe().unwrap(),
        std::time::Duration::from_secs(3),
    )
    .unwrap();
    assert!(
        local
            .io
            .execute(
                &descriptor,
                "remote-only",
                &["--help".into()],
                CancellationToken::new()
            )
            .await
            .unwrap()
            .success
    );
    let remote = Harness::new(true).await;
    let (_, transport) = remote.remote.as_ref().unwrap();
    let descriptor = ProcessDescriptor::cli_probe(
        "ccr",
        remote.root.path().join("missing-local-executable"),
        std::time::Duration::from_secs(3),
    )
    .unwrap();
    let command = "'fixture-tool' 'a'\"'\"'b' '$(touch secret)'";
    transport.set_exec_output(command, true, "remote result", "");
    let output = remote
        .io
        .execute(
            &descriptor,
            "fixture-tool",
            &["a'b".into(), "$(touch secret)".into()],
            CancellationToken::new(),
        )
        .await
        .unwrap();
    assert_eq!(output.stdout, b"remote result");
    assert!(transport.batches().contains(&format!("exec {command}")));
    let cancelled = CancellationToken::new();
    cancelled.cancel();
    assert!(matches!(
        remote
            .io
            .execute(&descriptor, "fixture-tool", &[], cancelled)
            .await,
        Err(Error::Cancelled)
    ));
}

#[test]
fn unavailable_remote_and_wsl_do_not_evaluate_local_roots() {
    assert!(matches!(
        EnvironmentIo::resolve(
            &EnvironmentId::Ssh("../host\\id".into()),
            || panic!("local roots"),
            None
        ),
        Err(Error::Ssh("ssh_not_connected"))
    ));
    assert!(matches!(
        EnvironmentIo::resolve(
            &EnvironmentId::Wsl("fixture".into()),
            || panic!("local roots"),
            None
        ),
        Err(Error::Unsupported)
    ));
}

#[tokio::test]
async fn all_platform_defaults_and_injected_local_override_use_the_same_io() {
    for remote in [false, true] {
        let h = Harness::new(remote).await;
        for (platform, suffix) in [
            (Platform::Claude, ".claude"),
            (Platform::Codex, ".codex"),
            (Platform::Grok, ".grok"),
            (Platform::Gemini, ".gemini/antigravity-cli"),
            (Platform::OpenCode, ".config/opencode"),
        ] {
            let expected = if remote {
                EnvironmentPath::Ssh(
                    RemotePosixPath::root(Some(&format!("/home/fixture/{suffix}"))).unwrap(),
                )
            } else {
                EnvironmentPath::Local(h.root.path().join("home").join(suffix))
            };
            assert_eq!(h.io.roots().platform(platform).unwrap(), expected);
            assert_eq!(
                h.io.for_platform(platform).unwrap().roots().home(),
                &expected
            );
        }
    }
    let h = Harness::new(false).await;
    let override_root = h.root.path().join("custom-codex");
    std::fs::create_dir(&override_root).unwrap();
    std::fs::write(override_root.join("config.toml"), "custom = true").unwrap();
    let io = EnvironmentIo::resolve(
        &h.io.environment,
        || {
            LocalRoots::new(h.root.path().join("home"), h.root.path().join("locks"))?
                .with_platform(Platform::Codex, override_root.clone())
        },
        None,
    )
    .unwrap()
    .for_platform(Platform::Codex)
    .unwrap();
    let original = io
        .read("config.toml", CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(original.content, "custom = true");
    io.write(&original, "custom = false", CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(override_root.join("config.toml")).unwrap(),
        "custom = false"
    );
    assert!(!h.root.path().join("home/.codex").exists());
}

#[tokio::test(start_paused = true)]
async fn remote_process_uses_descriptor_budget_including_beyond_connect_timeout() {
    for (budget, delay, success) in [
        (
            std::time::Duration::from_millis(1),
            std::time::Duration::from_secs(1),
            false,
        ),
        (
            std::time::Duration::from_secs(60),
            std::time::Duration::from_secs(25),
            true,
        ),
    ] {
        let h = Harness::new(true).await;
        let (_, transport) = h.remote.as_ref().unwrap();
        transport.set_exec_output("'fixture-tool'", true, "done", "");
        transport.delay_cli(delay);
        let descriptor =
            ProcessDescriptor::cli_probe("ccr", h.root.path().join("missing"), budget).unwrap();
        let result =
            h.io.execute(&descriptor, "fixture-tool", &[], CancellationToken::new())
                .await;
        assert_eq!(result.is_ok(), success);
        if !success {
            assert!(matches!(result, Err(Error::Ssh("ssh_network_error"))));
        }
    }
}

#[tokio::test(start_paused = true)]
async fn delayed_remote_completion_or_cancellation_cannot_publish_or_revoke_new_session() {
    for cancelled in [false, true] {
        let h = Harness::new(true).await;
        let (workspace, transport) = h.remote.as_ref().unwrap();
        transport.set_exec_output("'fixture-tool'", true, "old connection output", "");
        transport.delay_cli(std::time::Duration::from_secs(1));
        let io = h.io.clone();
        let cancel = CancellationToken::new();
        let worker_cancel = cancel.clone();
        let descriptor = ProcessDescriptor::cli_probe(
            "ccr",
            h.root.path().join("missing"),
            std::time::Duration::from_secs(60),
        )
        .unwrap();
        let task = tokio::spawn(async move {
            io.execute(&descriptor, "fixture-tool", &[], worker_cancel)
                .await
        });
        while !transport
            .batches()
            .iter()
            .any(|s| s == "exec 'fixture-tool'")
        {
            tokio::task::yield_now().await;
        }
        let EnvironmentId::Ssh(id) = &h.io.environment else {
            panic!()
        };
        workspace
            .connect(id, CancellationToken::new())
            .await
            .unwrap();
        if cancelled {
            cancel.cancel();
        }
        let result = task.await.unwrap();
        assert!(
            matches!(result, Err(Error::Ssh(code)) if code == if cancelled { "ssh_cancelled" } else { "ssh_session_changed" })
        );
        assert!(workspace.connected(id));
    }
}

#[tokio::test]
async fn failed_remote_cleanup_takes_priority_and_revokes_captured_authority() {
    for operation in ["get", "put", "exec"] {
        let h = Harness::new(true).await;
        let (workspace, transport) = h.remote.as_ref().unwrap();
        let original =
            h.io.read(".claude/settings.json", CancellationToken::new())
                .await
                .unwrap();
        transport.fail(operation);
        transport.fail("disconnect");
        let result = if operation == "exec" {
            transport.set_exec_output("'fixture-tool'", true, "output", "");
            let descriptor = ProcessDescriptor::cli_probe(
                "ccr",
                h.root.path().join("missing"),
                std::time::Duration::from_secs(3),
            )
            .unwrap();
            h.io.execute(&descriptor, "fixture-tool", &[], CancellationToken::new())
                .await
                .map(|_| ())
        } else if operation == "put" {
            h.io.write(&original, "replacement", CancellationToken::new())
                .await
                .map(|_| ())
        } else {
            h.io.read(".claude/settings.json", CancellationToken::new())
                .await
                .map(|_| ())
        };
        assert_eq!(result, Err(Error::Ssh("ssh_cleanup_failed")));
        assert_eq!(workspace.retained_session_count(), 0);
        assert_eq!(
            transport
                .file("fixture.invalid", "/home/fixture/.claude/settings.json")
                .as_deref(),
            Some("remote original")
        );
    }
}

#[tokio::test(start_paused = true)]
async fn cancelled_remote_process_with_unconfirmed_disconnect_is_cleanup_failed() {
    let h = Harness::new(true).await;
    let (workspace, transport) = h.remote.as_ref().unwrap();
    transport.set_exec_output("'fixture-tool'", true, "output", "");
    transport.delay_cli(std::time::Duration::from_secs(1));
    transport.fail("disconnect");
    let io = h.io.clone();
    let cancel = CancellationToken::new();
    let worker_cancel = cancel.clone();
    let descriptor = ProcessDescriptor::cli_probe(
        "ccr",
        h.root.path().join("missing"),
        std::time::Duration::from_secs(3),
    )
    .unwrap();
    let task = tokio::spawn(async move {
        io.execute(&descriptor, "fixture-tool", &[], worker_cancel)
            .await
    });
    while !transport
        .batches()
        .iter()
        .any(|s| s == "exec 'fixture-tool'")
    {
        tokio::task::yield_now().await;
    }
    cancel.cancel();
    assert!(matches!(
        task.await.unwrap(),
        Err(Error::Ssh("ssh_cleanup_failed"))
    ));
    assert_eq!(workspace.retained_session_count(), 0);
}
#[tokio::test]
async fn cancelled_read_joins_cleanup_without_revoking_the_current_session_or_returning_bytes() {
    for (fail_cleanup, replace_session) in
        [(false, false), (true, false), (false, true), (true, true)]
    {
        let h = Harness::new(true).await;
        let (workspace, transport) = h.remote.as_ref().unwrap();
        let EnvironmentId::Ssh(id) = h.io.environment() else {
            panic!("remote");
        };
        transport.delay_io(std::time::Duration::from_millis(50));
        if fail_cleanup {
            transport.fail_with("get", "ssh_cleanup_failed");
        }
        let started = transport.io_started();
        let cancel = CancellationToken::new();
        let worker_cancel = cancel.clone();
        let io = h.io.clone();
        let work =
            tokio::spawn(async move { io.read(".claude/settings.json", worker_cancel).await });
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            while transport.io_started() == started {
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
        })
        .await
        .unwrap();
        if replace_session {
            workspace.disconnect().await;
            workspace
                .connect(id, CancellationToken::new())
                .await
                .unwrap();
        }
        cancel.cancel();
        let result = work.await.unwrap();
        assert!(
            matches!(result, Err(Error::Ssh(code)) if code == if fail_cleanup { "ssh_cleanup_failed" } else { "ssh_cancelled" })
        );
        assert_eq!(workspace.connected(id), replace_session || !fail_cleanup);
        if !fail_cleanup && !replace_session {
            assert_eq!(
                h.io.read(".claude/settings.json", CancellationToken::new())
                    .await
                    .unwrap()
                    .content,
                "remote original"
            );
        }
    }
}

#[tokio::test]
async fn cancelled_typed_entries_join_cleanup_keep_health_and_never_revoke_a_replacement() {
    for (fail_cleanup, replace_session) in
        [(false, false), (true, false), (false, true), (true, true)]
    {
        let h = Harness::new(true).await;
        let (workspace, transport) = h.remote.as_ref().unwrap();
        let EnvironmentId::Ssh(id) = h.io.environment() else {
            panic!("remote");
        };
        transport.delay_io(std::time::Duration::from_millis(50));
        if fail_cleanup {
            transport.fail_with("list", "ssh_cleanup_failed");
        }
        let started = transport.io_started();
        let cancel = CancellationToken::new();
        let worker_cancel = cancel.clone();
        let io = h.io.clone();
        let work = tokio::spawn(async move { io.entries(".claude", worker_cancel).await });
        tokio::time::timeout(std::time::Duration::from_secs(10), async {
            while transport.io_started() == started {
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            }
        })
        .await
        .unwrap();
        if replace_session {
            workspace.disconnect().await;
            workspace
                .connect(id, CancellationToken::new())
                .await
                .unwrap();
        }
        cancel.cancel();
        let result = work.await.unwrap();
        assert!(
            matches!(result, Err(Error::Ssh(code)) if code == if fail_cleanup { "ssh_cleanup_failed" } else { "ssh_cancelled" })
        );
        assert_eq!(workspace.connected(id), replace_session || !fail_cleanup);
    }
    let h = Harness::new(true).await;
    let (workspace, transport) = h.remote.as_ref().unwrap();
    let EnvironmentId::Ssh(id) = h.io.environment() else {
        panic!("remote");
    };
    transport.set_unsupported_path("/home/fixture/.claude/shared");
    let entries =
        h.io.entries(".claude", CancellationToken::new())
            .await
            .unwrap()
            .unwrap();
    assert!(
        entries
            .iter()
            .any(|entry| entry.name == "shared" && entry.kind == EntryKind::Unsupported)
    );
    assert!(workspace.connected(id));
    assert!(matches!(
        h.io.entries(".claude/shared", CancellationToken::new())
            .await,
        Err(Error::Ssh("ssh_invalid_remote_path"))
    ));
    assert!(workspace.connected(id));
}
