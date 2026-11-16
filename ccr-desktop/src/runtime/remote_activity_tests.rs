#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use ccr_app::{
    environments::ssh::{HostInput, Workspace, fixture::FixtureTransport},
    runtime::{Completion, RuntimeOwner, TaskEvent},
    sessions::SessionListRequest,
};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

struct Fixture {
    root: tempfile::TempDir,
    owner: RuntimeOwner,
    services: Services,
    workspace: Arc<Workspace>,
    transport: Arc<FixtureTransport>,
    id: String,
}
async fn done<T: Send + Sync + 'static, E: Send + Sync + 'static>(
    mut task: TaskSubscription<T, E>,
) -> Arc<Completion<T, E>> {
    tokio::time::timeout(Duration::from_secs(10), async {
        while let Some(event) = task.next().await {
            if let TaskEvent::Terminal(value) = event {
                return value;
            }
        }
        panic!("terminal missing")
    })
    .await
    .unwrap()
}
impl Fixture {
    async fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let owner = RuntimeOwner::new().unwrap();
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("registry.toml"),
                delay: Duration::ZERO,
            },
        )
        .with_insights_paths(insights::InsightsPaths {
            usage_root: root.path().join("poison-usage"),
            archive_db: root.path().join("local-archive.db"),
            provider_home: root.path().join("local-home"),
        });
        let (workspace, transport) =
            Workspace::fixture(root.path().join("ssh.db"), root.path().join("known_hosts"));
        let workspace = Arc::new(workspace);
        services
            .ssh
            .set(workspace.clone())
            .unwrap_or_else(|_| panic!("fixture owner"));
        let host = workspace
            .save_host(HostInput {
                name: "Activity".into(),
                host: "activity.fixture.invalid".into(),
                user: "fixture".into(),
                port: 22,
                remote_home: "/srv/activity".into(),
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
        services.select_environment(EnvironmentId::Ssh(host.id.clone()));
        std::fs::write(root.path().join("local-poison"), "never read or changed").unwrap();
        Self {
            root,
            owner,
            services,
            workspace,
            transport,
            id: host.id,
        }
    }
    fn key(&self, id: u64) -> RequestKey {
        RequestKey {
            environment: EnvironmentId::Ssh(self.id.clone()),
            platform: Platform::Claude,
            generation: 1,
            request_id: id,
        }
    }
    fn set(&self, path: &str, content: &str) {
        self.transport.set_file(
            "activity.fixture.invalid",
            &format!("/srv/activity/{path}"),
            content,
        );
    }
    fn command(&self) {
        self.transport.set_cli_output("ccr", "/usr/bin/ccr");
        self.transport.set_exec_output(
            "'/usr/bin/ccr' 'status' '--json'",
            true,
            "remote-only\nAuthorization: Bearer synthetic-secret\n",
            "",
        );
    }
    async fn finish(self) {
        self.workspace.disconnect().await;
        tokio::task::spawn_blocking(move || self.owner.shutdown())
            .await
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(self.root.path().join("local-poison")).unwrap(),
            "never read or changed"
        );
    }
}

#[tokio::test]
async fn ssh_sessions_use_remote_sources_large_payload_mtime_export_and_fail_closed_actions() {
    let f = Fixture::new().await;
    let mut content = "{\"type\":\"session_meta\",\"payload\":{\"id\":\"remote-codex\",\"timestamp\":\"2026-11-01T00:00:00Z\",\"cwd\":\"/work/remote\"}}\n{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"remote transcript\"}]}}\n".to_string();
    content.push_str(&"\n".repeat(2 * 1024 * 1024));
    f.set(".codex/sessions/rollout.jsonl", &content);
    f.set(".claude/projects/project/session.jsonl","{\"type\":\"user\",\"sessionId\":\"remote-claude\",\"timestamp\":\"2026-11-02T00:00:00Z\",\"cwd\":\"/work/remote\",\"message\":{\"role\":\"user\",\"content\":\"remote Claude\"}}\n");
    f.set(".codex/auth.json", "private decoy must never transfer");
    let requests = insights::InsightsRequests::new(&f.services);
    let result = done(
        requests
            .load(
                f.key(1),
                insights::InsightsQuery::Sessions(SessionListRequest::default()),
            )
            .unwrap(),
    )
    .await;
    let Completion::Success(snapshot) = result.as_ref() else {
        panic!("sessions failed")
    };
    let insights::InsightsSnapshot::Sessions { page, .. } = snapshot.as_ref() else {
        panic!("wrong snapshot")
    };
    assert_eq!(page.items.len(), 2);
    let codex = page
        .items
        .iter()
        .find(|row| row.platform == "codex")
        .unwrap();
    let detail = done(
        requests
            .load(
                f.key(2),
                insights::InsightsQuery::Detail {
                    archive_id: codex.archive_id.clone(),
                    before: None,
                },
            )
            .unwrap(),
    )
    .await;
    let Completion::Success(detail) = detail.as_ref() else {
        panic!("detail failed")
    };
    let insights::InsightsSnapshot::Detail(detail) = detail.as_ref() else {
        panic!("wrong detail")
    };
    assert!(
        detail
            .page
            .messages
            .iter()
            .any(|message| message.content.contains("remote transcript"))
    );
    let export = done(
        requests
            .export_session(f.key(3), codex.archive_id.clone())
            .unwrap(),
    )
    .await;
    assert!(
        matches!(export.as_ref(),Completion::Success(export) if export.content.contains("remote transcript"))
    );
    let Completion::Success(export) = export.as_ref() else {
        panic!("export failed")
    };
    requests
        .start_session_save(
            f.key(6),
            codex.archive_id.clone(),
            export.clone(),
            f.root.path().join("remote-export.md"),
        )
        .unwrap();
    let mut updates = requests.observe_session_operations();
    tokio::time::timeout(Duration::from_secs(5), async {
        while updates.borrow().busy {
            updates.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    assert!(
        requests
            .session_operation_snapshot()
            .result
            .as_ref()
            .unwrap()
            .is_ok()
    );
    assert!(
        std::fs::read_to_string(f.root.path().join("remote-export.md"))
            .unwrap()
            .contains("remote transcript")
    );
    let mut forged = export.as_ref().clone();
    forged.content = "forged preview".into();
    requests
        .start_session_save(
            f.key(7),
            codex.archive_id.clone(),
            Arc::new(forged),
            f.root.path().join("forged-export.md"),
        )
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while updates.borrow().busy {
            updates.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    assert!(matches!(
        requests.session_operation_snapshot().result,
        Some(Err(insights::session_operations::Failure::Conflict))
    ));
    assert!(!f.root.path().join("forged-export.md").exists());
    assert!(matches!(
        done(
            requests
                .prepare_session_operation(f.key(4), codex.archive_id.clone())
                .unwrap()
        )
        .await
        .as_ref(),
        Completion::Failed(insights::session_operations::Failure::LocalOnly)
    ));
    let cache = f.root.path().join("session-cache-v1/ssh");
    let host = std::fs::read_dir(cache)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let snapshot = std::fs::read_dir(host)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let modified = std::fs::metadata(snapshot.join(".codex/sessions/rollout.jsonl"))
        .unwrap()
        .modified()
        .unwrap()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    assert_eq!(modified, 1_700_000_000_000);
    std::fs::write(
        snapshot.join(".codex/sessions/rollout.jsonl"),
        content.replace("remote transcript", "cached transcript"),
    )
    .unwrap();
    assert!(matches!(
        done(
            requests
                .load(
                    f.key(5),
                    insights::InsightsQuery::Detail {
                        archive_id: codex.archive_id.clone(),
                        before: None
                    }
                )
                .unwrap()
        )
        .await
        .as_ref(),
        Completion::Failed(insights::InsightsError::Remote)
    ));
    assert!(
        !f.transport
            .batches()
            .iter()
            .any(|line| line.contains("auth.json")
                || line.starts_with("put ")
                || line.starts_with("remove "))
    );
    f.finish().await;
}

#[tokio::test]
async fn ssh_commands_detect_missing_cli_and_execute_only_captured_remote_authority() {
    let f = Fixture::new().await;
    let requests = jobs::CommandRequests::new(&f.services);
    let started = requests
        .foreground(f.key(1), "status".into(), vec!["--json".into()])
        .unwrap();
    let result = done(started.subscription.unwrap()).await;
    assert!(
        matches!(result.as_ref(),Completion::Success(snapshot) if snapshot.status==ccr_app::jobs::commands::Status::Unavailable && snapshot.error.as_ref().unwrap().contains("Remote CCR CLI is missing"))
    );
    assert!(
        !f.transport
            .batches()
            .iter()
            .any(|line| line.starts_with("exec "))
    );
    f.command();
    let started = requests
        .start(f.key(2), "status".into(), vec!["--json".into()])
        .unwrap();
    let id = started.snapshot.job_id.clone();
    let result = done(started.subscription.unwrap()).await;
    assert!(
        matches!(result.as_ref(),Completion::Success(snapshot) if snapshot.status==ccr_app::jobs::commands::Status::Success && snapshot.stdout_lines.iter().any(|line|line=="remote-only") && !snapshot.stdout_lines.iter().any(|line|line.contains("synthetic-secret")))
    );
    assert_eq!(
        requests.observe(&id).unwrap().0.status,
        ccr_app::jobs::commands::Status::Success
    );
    assert!(
        f.transport
            .batches()
            .iter()
            .any(|line| line == "exec '/usr/bin/ccr' 'status' '--json'")
    );
    f.finish().await;
}

#[tokio::test]
async fn ssh_install_uses_canonical_remote_plan_ack_and_single_consumption() {
    let f = Fixture::new().await;
    f.transport.set_cli_output("cargo", "/usr/bin/cargo");
    f.transport
        .set_exec_output("'uname' '-s'", true, "Linux\n", "");
    f.transport.set_exec_output(
        "'/usr/bin/cargo' 'install' '--locked' 'llmusage'",
        true,
        "installed remote\n",
        "",
    );
    let requests = install::InstallRequests::new(&f.services);
    assert!(
        matches!(done(requests.query(f.key(1),install::Query::Check).unwrap()).await.as_ref(),Completion::Success(snapshot) if matches!(snapshot.detection,Some(ccr_cli::services::install_types::DetectionResult::Absent {..})))
    );
    let plan = done(requests.query(f.key(2), install::Query::Plan).unwrap()).await;
    let Completion::Success(snapshot) = plan.as_ref() else {
        panic!("plan failed")
    };
    let Some(ccr_cli::services::install_types::PlanOutcome::Plan(plan)) = &snapshot.plan else {
        panic!("missing plan")
    };
    assert!(matches!(
        requests.execute(f.key(3), plan.plan_id, false),
        Err(install::Error::Confirmation)
    ));
    assert!(
        matches!(done(requests.execute(f.key(4),plan.plan_id,true).unwrap()).await.as_ref(),Completion::Success(snapshot) if matches!(snapshot.recent.terminal,Some(ccr_cli::services::install_types::InstallEvent::Succeeded {..})))
    );
    assert!(matches!(
        done(requests.execute(f.key(5), plan.plan_id, true).unwrap())
            .await
            .as_ref(),
        Completion::Failed(install::Error::PlanUnavailable)
    ));
    assert_eq!(
        f.transport
            .batches()
            .iter()
            .filter(|line| line.contains("exec '/usr/bin/cargo' 'install'"))
            .count(),
        1
    );
    f.finish().await;
}

#[tokio::test]
async fn ssh_overview_uses_remote_facts_and_never_measures_local_usage_as_zero() {
    let f = Fixture::new().await;
    f.set(".codex/config.toml", "model = 'remote'");
    f.transport.set_cli_output("codex", "/usr/bin/codex");
    let requests = insights::InsightsRequests::new(&f.services);
    let result = done(requests.load_remote_overview(f.key(1), None).unwrap()).await;
    assert!(
        matches!(result.as_ref(),Completion::Success(report) if report.platforms.len()==5 && report.platforms.iter().any(|row|row.platform==Platform::Codex && row.cli && row.configuration))
    );
    assert!(matches!(
        done(requests.load_workbench(f.key(2), 7).unwrap())
            .await
            .as_ref(),
        Completion::Failed(insights::InsightsError::RemoteUsagePending)
    ));
    assert!(!f.root.path().join("poison-usage").exists());
    f.finish().await;
}

#[tokio::test]
async fn ssh_queued_commands_and_session_reads_never_acquire_a_reconnected_session() {
    use std::sync::{
        Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
    };
    let f = Fixture::new().await;
    f.command();
    let entered = Arc::new(AtomicUsize::new(0));
    let release = Arc::new((Mutex::new(false), Condvar::new()));
    let mut blockers = Vec::new();
    for index in 0..4 {
        let entered = entered.clone();
        let release = release.clone();
        blockers.push(
            f.services
                .executor
                .read(
                    RequestKey {
                        environment: EnvironmentId::Local("blocking-fixture".into()),
                        platform: Platform::Claude,
                        generation: 1,
                        request_id: index,
                    },
                    move |_, _| {
                        entered.fetch_add(1, Ordering::SeqCst);
                        let (lock, wake) = &*release;
                        let mut done = lock.lock().unwrap();
                        while !*done {
                            done = wake.wait(done).unwrap();
                        }
                        Ok::<_, ()>(())
                    },
                )
                .unwrap(),
        );
    }
    tokio::time::timeout(Duration::from_secs(5), async {
        while entered.load(Ordering::SeqCst) != 4 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
    let command = jobs::CommandRequests::new(&f.services)
        .start(f.key(10), "status".into(), vec!["--json".into()])
        .unwrap();
    let sessions = insights::InsightsRequests::new(&f.services)
        .load(
            f.key(11),
            insights::InsightsQuery::Sessions(SessionListRequest::default()),
        )
        .unwrap();
    f.workspace.disconnect().await;
    f.workspace
        .connect(&f.id, CancellationToken::new())
        .await
        .unwrap();
    let before = f.transport.batches();
    {
        let (lock, wake) = &*release;
        *lock.lock().unwrap() = true;
        wake.notify_all();
    }
    for blocker in blockers {
        done(blocker).await;
    }
    assert!(
        matches!(done(command.subscription.unwrap()).await.as_ref(),Completion::Success(snapshot) if snapshot.status==ccr_app::jobs::commands::Status::Unavailable)
    );
    assert!(matches!(
        done(sessions).await.as_ref(),
        Completion::Failed(insights::InsightsError::Remote)
    ));
    assert_eq!(f.transport.batches(), before);
    f.finish().await;
}

#[tokio::test]
async fn ssh_command_cancel_after_exec_waits_for_revocation_and_overflow_is_marked() {
    let f = Fixture::new().await;
    f.command();
    f.transport.delay_cli(Duration::from_millis(20));
    let requests = jobs::CommandRequests::new(&f.services);
    let started = requests
        .start(f.key(1), "status".into(), vec!["--json".into()])
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while !f
            .transport
            .batches()
            .iter()
            .any(|line| line == "exec '/usr/bin/ccr' 'status' '--json'")
        {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        requests.cancel(&started.snapshot.job_id).unwrap().status,
        ccr_app::jobs::commands::Status::Running
    );
    assert!(matches!(
        done(started.subscription.unwrap()).await.as_ref(),
        Completion::CleanupFailed(_)
    ));
    assert_eq!(
        requests.status(&started.snapshot.job_id).unwrap().status,
        ccr_app::jobs::commands::Status::CleanupFailed
    );
    assert!(
        f.transport
            .batches()
            .iter()
            .any(|line| line == "disconnect activity.fixture.invalid")
    );
    f.workspace
        .connect(&f.id, CancellationToken::new())
        .await
        .unwrap();
    f.transport.delay_cli(Duration::ZERO);
    f.transport.set_exec_output(
        "'/usr/bin/ccr' 'status' '--json'",
        true,
        &"x".repeat(1024 * 1024 + 1),
        "",
    );
    let started = requests
        .start(f.key(2), "status".into(), vec!["--json".into()])
        .unwrap();
    assert!(matches!(
        done(started.subscription.unwrap()).await.as_ref(),
        Completion::CleanupFailed(_)
    ));
    let snapshot = requests.status(&started.snapshot.job_id).unwrap();
    assert!(snapshot.truncated);
    assert!(snapshot.dropped_lines > 0);
    f.finish().await;
}

#[tokio::test]
async fn ssh_install_plan_cannot_follow_reconnect_and_ccr_install_is_the_existing_remote_cargo_action()
 {
    let f = Fixture::new().await;
    f.transport.set_cli_output("cargo", "/usr/bin/cargo");
    f.transport
        .set_exec_output("'uname' '-s'", true, "Linux\n", "");
    let requests = install::InstallRequests::new(&f.services);
    done(requests.query(f.key(1), install::Query::Check).unwrap()).await;
    let plan = done(requests.query(f.key(2), install::Query::Plan).unwrap()).await;
    let Completion::Success(snapshot) = plan.as_ref() else {
        panic!("plan failed")
    };
    let Some(ccr_cli::services::install_types::PlanOutcome::Plan(plan)) = &snapshot.plan else {
        panic!("missing plan")
    };
    f.workspace.disconnect().await;
    f.workspace
        .connect(&f.id, CancellationToken::new())
        .await
        .unwrap();
    assert!(matches!(
        done(requests.execute(f.key(3), plan.plan_id, true).unwrap())
            .await
            .as_ref(),
        Completion::Failed(install::Error::PlanUnavailable)
    ));
    f.transport.set_exec_output(
        "'/usr/bin/cargo' 'install' 'ccr' '--root' '/srv/activity/.cargo'",
        true,
        "remote CCR installed",
        "",
    );
    let tools = tools::Requests::new(&f.services);
    assert!(matches!(
        tools.perform(f.key(4), tools::Action::UpdateCcr, false),
        Err(tools::Error::Confirmation)
    ));
    tools
        .perform(f.key(5), tools::Action::UpdateCcr, true)
        .unwrap();
    let mut updates = tools.observe();
    tokio::time::timeout(Duration::from_secs(5), async {
        while updates.borrow().status.active() {
            updates.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    assert_eq!(tools.snapshot().status, tools::Status::Finished);
    assert!(f.transport.batches().iter().any(
        |line| line == "exec '/usr/bin/cargo' 'install' 'ccr' '--root' '/srv/activity/.cargo'"
    ));
    f.finish().await;
}

#[tokio::test]
async fn ssh_sqlite_session_snapshot_rejects_same_size_rewrite_and_live_sidecars() {
    let f = Fixture::new().await;
    f.set(".local/share/opencode/opencode.db", "aaaa");
    f.transport
        .replace_after_session_read("/srv/activity/.local/share/opencode/opencode.db", b"bbbb");
    let io = f
        .services
        .remote_configuration_io(&f.key(1).environment)
        .unwrap();
    assert!(matches!(
        ccr_app::sessions::remote::SessionSnapshot::capture(
            io,
            &f.root.path().join("cache"),
            CancellationToken::new()
        )
        .await,
        Err(ccr_app::environments::io::Error::Conflict)
    ));
    f.set(".local/share/opencode/opencode.db-wal", "active");
    let before = f.transport.batches().len();
    let io = f
        .services
        .remote_configuration_io(&f.key(2).environment)
        .unwrap();
    assert!(matches!(
        ccr_app::sessions::remote::SessionSnapshot::capture(
            io,
            &f.root.path().join("cache"),
            CancellationToken::new()
        )
        .await,
        Err(ccr_app::environments::io::Error::Busy)
    ));
    assert!(
        !f.transport.batches()[before..]
            .iter()
            .any(|line| line.starts_with("get-session "))
    );
    f.finish().await;
}

#[tokio::test]
async fn ssh_install_query_cancellation_joins_only_the_sent_probe_and_never_starts_followups() {
    let f = Fixture::new().await;
    f.transport.delay_cli(Duration::from_millis(100));
    let requests = install::InstallRequests::new(&f.services);
    let query = requests.query(f.key(1), install::Query::Check).unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while f.transport.cli_requests().is_empty() {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap();
    query.request_cancel();
    assert!(!matches!(
        done(query).await.as_ref(),
        Completion::Success(_)
    ));
    assert_eq!(f.transport.cli_requests().len(), 1);
    assert!(
        !f.transport
            .batches()
            .iter()
            .any(|line| line.starts_with("exec ") || line.starts_with("disconnect "))
    );
    assert!(f.workspace.connected(&f.id));
    assert!(requests.snapshot().detection.is_none());
    f.finish().await;
}

#[tokio::test]
async fn ssh_opencode_binary_snapshot_reuses_owner_schema_and_transcript_parser() {
    let f = Fixture::new().await;
    let database = f.root.path().join("synthetic-opencode.db");
    let conn = rusqlite::Connection::open(&database).unwrap();
    // 复用 ccr-store::sessions 的 opencode_sqlite_members fixture schema。
    conn.execute_batch("CREATE TABLE session(id TEXT PRIMARY KEY, title TEXT, directory TEXT, time_created INTEGER, time_updated INTEGER); CREATE TABLE message(id TEXT PRIMARY KEY, session_id TEXT, data TEXT, time_created INTEGER); CREATE TABLE part(id TEXT PRIMARY KEY, session_id TEXT, message_id TEXT, data TEXT, time_created INTEGER); INSERT INTO session VALUES('remote-sqlite', 'Remote SQLite', '/work/remote', 1000, 2000); INSERT INTO message VALUES('m1', 'remote-sqlite', '{\"role\":\"user\"}', 1000); INSERT INTO part VALUES('p1', 'remote-sqlite', 'm1', '{\"text\":\"remote OpenCode binary transcript\"}', 1000);").unwrap();
    drop(conn);
    let bytes = std::fs::read(database).unwrap();
    let remote = "/srv/activity/.local/share/opencode/opencode.db";
    f.transport
        .set_bytes("activity.fixture.invalid", remote, &bytes);
    f.set(
        ".local/share/opencode/storage/auth.json",
        "private storage decoy",
    );
    f.set(
        ".local/share/opencode/storage/unrelated.json",
        "unrelated storage decoy",
    );
    let requests = insights::InsightsRequests::new(&f.services);
    let result = done(
        requests
            .load(
                f.key(1),
                insights::InsightsQuery::Sessions(SessionListRequest::default()),
            )
            .unwrap(),
    )
    .await;
    let Completion::Success(snapshot) = result.as_ref() else {
        panic!("SQLite snapshot failed")
    };
    let insights::InsightsSnapshot::Sessions { page, .. } = snapshot.as_ref() else {
        panic!("wrong snapshot")
    };
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].platform, "opencode");
    let result = done(
        requests
            .load(
                f.key(2),
                insights::InsightsQuery::Detail {
                    archive_id: page.items[0].archive_id.clone(),
                    before: None,
                },
            )
            .unwrap(),
    )
    .await;
    assert!(
        matches!(result.as_ref(),Completion::Success(snapshot) if matches!(snapshot.as_ref(), insights::InsightsSnapshot::Detail(detail) if detail.variant=="opencode-sqlite" && detail.page.messages.iter().any(|message|message.content=="remote OpenCode binary transcript")))
    );
    assert_eq!(
        f.transport
            .bytes("activity.fixture.invalid", remote)
            .unwrap(),
        bytes
    );
    assert!(
        !f.transport
            .batches()
            .iter()
            .any(|line| line.starts_with("get-session ")
                && (line.ends_with("/storage/auth.json")
                    || line.ends_with("/storage/unrelated.json")))
    );
    assert_eq!(
        f.transport
            .batches()
            .iter()
            .filter(|line| *line == &format!("get-session {remote}"))
            .count(),
        2
    );
    f.finish().await;
}

#[tokio::test]
async fn ssh_verified_export_without_local_insights_paths_saves_only_inside_the_explicit_fixture() {
    let mut f = Fixture::new().await;
    f.services.insights_paths = None;
    f.set(".codex/sessions/rollout.jsonl", "{\"type\":\"session_meta\",\"payload\":{\"id\":\"remote-delivery\",\"timestamp\":\"2026-11-01T00:00:00Z\",\"cwd\":\"/work/remote\"}}\n{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"verified remote delivery\"}]}}\n");
    let requests = insights::InsightsRequests::new(&f.services);
    assert!(requests.configured());
    assert_eq!(
        requests.session_save_directory().as_deref(),
        Some(f.root.path())
    );
    let result = done(
        requests
            .load(
                f.key(1),
                insights::InsightsQuery::Sessions(SessionListRequest::default()),
            )
            .unwrap(),
    )
    .await;
    let Completion::Success(snapshot) = result.as_ref() else {
        panic!("list failed")
    };
    let insights::InsightsSnapshot::Sessions { page, .. } = snapshot.as_ref() else {
        panic!("wrong list")
    };
    let id = page.items[0].archive_id.clone();
    let result = done(requests.export_session(f.key(2), id.clone()).unwrap()).await;
    let Completion::Success(export) = result.as_ref() else {
        panic!("export failed")
    };
    requests
        .start_session_save(
            f.key(3),
            id.clone(),
            export.clone(),
            f.root.path().join("verified.md"),
        )
        .unwrap();
    let mut updates = requests.observe_session_operations();
    tokio::time::timeout(Duration::from_secs(5), async {
        while updates.borrow().busy {
            updates.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    assert!(
        requests
            .session_operation_snapshot()
            .result
            .as_ref()
            .unwrap()
            .is_ok()
    );
    assert_eq!(
        std::fs::read_to_string(f.root.path().join("verified.md")).unwrap(),
        export.content
    );
    let outside = tempfile::tempdir().unwrap();
    requests
        .start_session_save(
            f.key(4),
            id,
            export.clone(),
            outside.path().join("escape.md"),
        )
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while updates.borrow().busy {
            updates.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    assert!(matches!(
        requests.session_operation_snapshot().result,
        Some(Err(insights::session_operations::Failure::Storage))
    ));
    assert!(!outside.path().join("escape.md").exists());
    assert!(!f.root.path().join("local-archive.db").exists());
    assert!(!f.root.path().join("poison-usage").exists());
    f.finish().await;
}
