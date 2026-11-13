#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use crate::runtime::Services;
use ccr_app::runtime::{Platform, RuntimeOwner};
use std::{path::Path, time::Duration};

fn key(environment: EnvironmentId) -> RequestKey {
    RequestKey {
        environment,
        platform: Platform::Claude,
        generation: 1,
        request_id: 1,
    }
}
fn fixture(owner: &RuntimeOwner, root: &Path) -> InsightsRequests {
    std::fs::create_dir_all(root.join("usage")).unwrap();
    std::fs::create_dir_all(root.join("home")).unwrap();
    InsightsRequests::new(
        &Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.join("profiles.toml"),
                registry: root.join("config.toml"),
                delay: Duration::ZERO,
            },
        )
        .with_insights_paths(InsightsPaths {
            usage_root: root.join("usage"),
            provider_home: root.join("home"),
            archive_db: root.join("archive.db"),
        }),
    )
}
fn wait(requests: &InsightsRequests, predicate: impl Fn(&Snapshot) -> bool) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while !predicate(&requests.import_snapshot()) {
        assert!(
            std::time::Instant::now() < deadline,
            "import status: {:?}",
            requests.import_snapshot()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
#[test]
fn imports_reject_remote_fixture_and_custom_provider_roots_before_admission() {
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let requests = fixture(&owner, root.path());
    assert_eq!(
        requests.start_import(key(EnvironmentId::Ssh("remote".into()))),
        Err(Failure::LocalOnly)
    );
    assert_eq!(
        requests.start_import(key(EnvironmentId::Local("fixture".into()))),
        Err(Failure::FixtureDisabled)
    );
    let local = InsightsRequests::new(
        &Services::new(owner.handle(), ConfigSource::Local)
            .with_insights_paths(requests.paths.clone().unwrap()),
    );
    assert_eq!(
        local.start_import(key(EnvironmentId::Local("local".into()))),
        Err(Failure::SourceRoot)
    );
    assert_eq!(owner.handle().active_tasks(), 0);
    owner.shutdown();
}
#[test]
fn import_survives_view_detachment_and_cancel_waits_for_the_runner() {
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let requests = fixture(&owner, root.path());
    let projects = root.path().join("home/.claude/projects/fixture");
    std::fs::create_dir_all(&projects).unwrap();
    let tool = r#"{"type":"assistant","sessionId":"native-import","timestamp":"2026-11-06T10:00:00Z","message":{"content":[{"type":"tool_use","id":"tool-1","name":"Read","input":{}}]}}"#;
    std::fs::write(projects.join("session.jsonl"), format!("{tool}\n")).unwrap();
    requests.fixture_import(false);
    requests
        .start_import(key(EnvironmentId::Local("fixture".into())))
        .unwrap();
    drop(requests.observe_import());
    wait(&requests, |s| s.status == Status::Finished);
    assert_eq!(requests.import_snapshot().records_imported, 1);
    assert!(matches!(
        requests.import_snapshot().tools,
        ToolUpdate::Finished {
            files: 1,
            calls: 1,
            errors: 0
        }
    ));
    let database = rusqlite::Connection::open_with_flags(
        root.path().join("observer.db"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    assert_eq!(
        ccr_db::database::repositories::claude_tool_calls_repo::total_count(&database).unwrap(),
        1
    );
    requests
        .start_import(key(EnvironmentId::Local("fixture".into())))
        .unwrap();
    wait(&requests, |s| s.status == Status::Finished);
    assert!(matches!(
        requests.import_snapshot().tools,
        ToolUpdate::Finished {
            files: 1,
            calls: 0,
            errors: 0
        }
    ));
    assert_eq!(
        ccr_db::database::repositories::claude_tool_calls_repo::total_count(&database).unwrap(),
        1
    );
    requests.fixture_import(true);
    requests
        .start_import(key(EnvironmentId::Local("fixture".into())))
        .unwrap();
    wait(&requests, |s| s.status == Status::Running);
    assert_eq!(
        requests.start_import(key(EnvironmentId::Local("fixture".into()))),
        Err(Failure::Busy)
    );
    requests.cancel_import();
    assert!(matches!(
        requests.import_snapshot().status,
        Status::CancelRequested | Status::Cancelled
    ));
    wait(&requests, |s| s.status == Status::Cancelled);
    owner.shutdown();
}

#[test]
fn queued_cancel_keeps_a_cancelled_terminal_without_starting_the_importer() {
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let requests = fixture(&owner, root.path());
    requests.fixture_import(true);
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let mut releases = Vec::new();
    let mut blockers = Vec::new();
    for i in 0..ccr_app::runtime::BLOCKING_CONCURRENCY {
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        releases.push(release_tx);
        let entered_tx = entered_tx.clone();
        let mut request = key(EnvironmentId::Local("fixture".into()));
        request.request_id = i as u64 + 100;
        blockers.push(
            owner
                .handle()
                .job(request, move |_, _| {
                    entered_tx.send(()).unwrap();
                    release_rx.recv_timeout(Duration::from_secs(10)).unwrap();
                    Completion::<(), Failure>::Success(())
                })
                .unwrap(),
        );
    }
    for _ in 0..ccr_app::runtime::BLOCKING_CONCURRENCY {
        entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    }
    requests
        .start_import(key(EnvironmentId::Local("fixture".into())))
        .unwrap();
    assert_eq!(requests.import_snapshot().status, Status::Queued);
    requests.cancel_import();
    wait(&requests, |s| s.status == Status::Cancelled);
    assert_eq!(requests.import_snapshot().files_scanned, 0);
    for release in releases {
        release.send(()).unwrap();
    }
    drop(blockers);
    owner.shutdown();
}

#[test]
fn import_options_preserve_each_source_window_and_existing_activation_map() {
    let root = tempfile::tempdir().unwrap();
    let paths = InsightsPaths {
        usage_root: root.path().join("usage"),
        provider_home: root.path().join("home"),
        archive_db: root.path().join("analytics/usage.db"),
    };
    for source in std::iter::once(None).chain(SourceKind::ALL.into_iter().map(Some)) {
        let options = Options {
            source,
            recent_days: 7,
        };
        let command = options.command(&paths);
        assert_eq!(command.source, source);
        assert_eq!(command.recent_days, Some(7));
        assert!(!command.rebuild);
        assert_eq!(command.provider_map, None);
        assert_eq!(
            options.includes_claude(),
            source.is_none() || source == Some(SourceKind::Claude)
        );
    }
    let map = ccr_config::managers::provider_activation::activation_log_path(root.path());
    std::fs::create_dir_all(map.parent().unwrap()).unwrap();
    std::fs::write(&map, "").unwrap();
    let command = Options {
        source: Some(SourceKind::Grok),
        recent_days: 0,
    }
    .command(&paths);
    assert_eq!(command.provider_map, Some(map));
    assert_eq!(command.recent_days, Some(1));
}

#[test]
fn selected_source_is_retained_without_ingesting_another_sources_tools() {
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let requests = fixture(&owner, root.path());
    let projects = root.path().join("home/.claude/projects/fixture");
    std::fs::create_dir_all(&projects).unwrap();
    std::fs::write(projects.join("session.jsonl"),
        r#"{"type":"assistant","sessionId":"excluded","timestamp":"2026-11-06T10:00:00Z","message":{"content":[{"type":"tool_use","id":"tool-1","name":"Read","input":{}}]}}"#).unwrap();
    requests.fixture_import(false);
    let options = Options {
        source: Some(SourceKind::Codex),
        recent_days: 7,
    };
    requests
        .start_import_with_options(key(EnvironmentId::Local("fixture".into())), options)
        .unwrap();
    drop(requests.observe_import());
    wait(&requests, |s| s.status == Status::Finished);
    assert_eq!(requests.import_snapshot().options, options);
    assert!(matches!(
        requests.import_snapshot().tools,
        ToolUpdate::NotStarted
    ));
    assert!(!root.path().join("observer.db").exists());
    requests.cancel_import();
    assert_eq!(requests.import_snapshot().status, Status::Finished);
    assert_eq!(requests.import_snapshot().options, options);
    owner.shutdown();
}
// PRD 8.3「作用域」：平台作用域内的导入来源不能伪造——绕过界面直接传入别的平台或不指定来源，
// 都在准入前拒绝，不启动任何任务。
#[test]
fn scoped_import_rejects_forged_or_missing_source_before_admission() {
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let claude = fixture(&owner, root.path()).scoped(Some(Platform::Claude));
    for source in [Some(SourceKind::Codex), None] {
        assert_eq!(
            claude.start_import_with_options(
                key(EnvironmentId::Local("fixture".into())),
                Options {
                    source,
                    ..Options::default()
                },
            ),
            Err(Failure::SourceRoot)
        );
    }
    // 远程环境同样先按来源拒绝，不会回落到本地导入。
    assert_eq!(
        claude.start_import_with_options(
            key(EnvironmentId::Ssh("remote".into())),
            Options {
                source: Some(SourceKind::Codex),
                ..Options::default()
            },
        ),
        Err(Failure::SourceRoot)
    );
    assert_eq!(owner.handle().active_tasks(), 0);
    assert_eq!(claude.import_snapshot().status, Status::Idle);
    owner.shutdown();
}
