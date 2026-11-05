#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::runtime::ConfigSource;
use ccr_app::runtime::{Completion, Platform, RuntimeOwner, TaskEvent};
use ccr_usage::fixtures::{SeedBucket, SeedEvent, create_projection_db, seed_bucket, seed_event};
use std::time::Duration;

fn key(environment: EnvironmentId) -> RequestKey {
    RequestKey {
        environment,
        platform: Platform::Claude,
        generation: 1,
        request_id: 1,
    }
}
fn services(owner: &RuntimeOwner, root: &std::path::Path) -> Services {
    Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.join("profiles.toml"),
            registry: root.join("config.toml"),
            delay: Duration::ZERO,
        },
    )
    .with_insights_paths(InsightsPaths {
        usage_root: root.join("llmusage"),
        archive_db: root.join("archive.db"),
        provider_home: root.join("home"),
    })
}
#[gpui_kit::test]
async fn bounded_usage_consumer_preserves_values_and_paginates_without_raw_content(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let paths = create_projection_db(&root.path().join("llmusage"));
    let pool = database::create_pool(&paths.db_path, None).unwrap();
    let conn = pool.get().unwrap();
    seed_bucket(
        &conn,
        &SeedBucket {
            cost_with_cache_usd: 0.123456789,
            ..Default::default()
        },
    );
    for i in 0..101 {
        seed_event(
            &conn,
            &SeedEvent {
                event_key: format!("fixture-{i:03}"),
                ..Default::default()
            },
        );
    }
    drop(conn);
    drop(pool);
    let before = std::fs::read(&paths.db_path).unwrap();
    let requests = InsightsRequests::new(&services(&owner, root.path()));
    let mut read = requests
        .load(
            key(EnvironmentId::Local("fixture".into())),
            InsightsQuery::Usage(QueryFilter::default()),
        )
        .unwrap();
    let Some(TaskEvent::Terminal(result)) = read.next().await else {
        panic!("missing report terminal");
    };
    let Completion::Success(snapshot) = result.as_ref() else {
        panic!("report failed");
    };
    let InsightsSnapshot::Usage(report) = snapshot.as_ref() else {
        panic!("wrong report");
    };
    assert_eq!(report.overview.total.total_tokens, 100);
    assert_eq!(report.models[0].cost_with_cache_usd, 0.123456789);
    assert_eq!(report.providers[0].reasoning_output_tokens, 15);
    let mut first = requests
        .load(
            key(EnvironmentId::Local("fixture".into())),
            InsightsQuery::Logs {
                filter: QueryFilter::default(),
                cursor: None,
            },
        )
        .unwrap();
    let Some(TaskEvent::Terminal(result)) = first.next().await else {
        panic!("missing page terminal");
    };
    let Completion::Success(snapshot) = result.as_ref() else {
        panic!("logs failed");
    };
    let InsightsSnapshot::Logs(page) = snapshot.as_ref() else {
        panic!("wrong logs");
    };
    assert_eq!(page.records.len(), 100);
    assert_eq!(page.total, Some(101));
    assert!(page.records.iter().all(|r| r.record_json.is_empty()));
    let first_ids = page
        .records
        .iter()
        .map(|r| r.id.clone())
        .collect::<Vec<_>>();
    let mut second = requests
        .load(
            key(EnvironmentId::Local("fixture".into())),
            InsightsQuery::Logs {
                filter: QueryFilter::default(),
                cursor: page.next_cursor.clone(),
            },
        )
        .unwrap();
    let Some(TaskEvent::Terminal(result)) = second.next().await else {
        panic!("missing second terminal");
    };
    let Completion::Success(snapshot) = result.as_ref() else {
        panic!("second logs failed");
    };
    let InsightsSnapshot::Logs(page) = snapshot.as_ref() else {
        panic!("wrong second logs");
    };
    assert_eq!(page.records.len(), 1);
    assert!(!first_ids.contains(&page.records[0].id));
    assert!(page.next_cursor.is_none());
    assert!(!root.path().join("archive.db").exists());
    assert!(!root.path().join("home").exists());
    assert_eq!(before, std::fs::read(paths.db_path).unwrap());
    owner.shutdown();
}
#[gpui_kit::test]
async fn remote_and_unconfigured_reads_fail_closed_without_creating_paths(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let requests = InsightsRequests::new(&services(&owner, root.path()));
    let mut read = requests
        .load(
            key(EnvironmentId::Ssh("remote".into())),
            InsightsQuery::Sessions(SessionListRequest::default()),
        )
        .unwrap();
    assert!(
        matches!(read.next().await,Some(TaskEvent::Terminal(result)) if matches!(result.as_ref(),Completion::Failed(InsightsError::LocalOnly)))
    );
    let mut read = requests
        .load(
            key(EnvironmentId::Local("fixture".into())),
            InsightsQuery::Usage(QueryFilter::default()),
        )
        .unwrap();
    assert!(
        matches!(read.next().await,Some(TaskEvent::Terminal(result)) if matches!(result.as_ref(),Completion::Failed(InsightsError::Missing)))
    );
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    let services = Services::new(
        owner.handle(),
        ConfigSource::Fixture {
            profiles: root.path().join("p"),
            registry: root.path().join("r"),
            delay: Duration::ZERO,
        },
    );
    let requests = InsightsRequests::new(&services);
    assert!(!requests.configured());
    let mut read = requests
        .load(
            key(EnvironmentId::Local("fixture".into())),
            InsightsQuery::Usage(QueryFilter::default()),
        )
        .unwrap();
    assert!(
        matches!(read.next().await,Some(TaskEvent::Terminal(result)) if matches!(result.as_ref(),Completion::Failed(InsightsError::Unconfigured)))
    );
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    owner.shutdown();
}
#[test]
fn error_messages_never_expose_storage_or_parser_payloads() {
    for error in [
        UsageError::DbUnreadable("sensitive-path-and-token".into()),
        UsageError::Query("sensitive-path-and-token".into()),
    ] {
        assert!(
            !InsightsError::from(error)
                .message()
                .contains("sensitive-path-and-token")
        );
    }
    for error in [
        SessionError::SourceValidation,
        SessionError::SourceUnavailable,
        SessionError::Database,
    ] {
        assert!(
            !InsightsError::Session(error)
                .message()
                .contains("agent_session_")
        );
    }
}

#[test]
fn archive_refresh_detaches_and_retains_actual_counters_without_touching_provider() {
    let root = tempfile::tempdir().unwrap();
    let pool = database::create_pool(&root.path().join("archive.db"), None).unwrap();
    database::migrations::run_all_migrations(&pool.get().unwrap(), root.path()).unwrap();
    drop(pool);
    let source = root.path().join("home/.pi/agent/sessions/project/s1.jsonl");
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    let content = "{\"type\":\"message\",\"message\":{\"role\":\"user\",\"content\":\"synthetic refresh\"}}\n";
    std::fs::write(&source, content).unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let requests = InsightsRequests::new(&services(&owner, root.path()));
    let subscription = requests
        .refresh_archive(key(EnvironmentId::Local("fixture".into())))
        .unwrap()
        .unwrap();
    drop(subscription);
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        match requests.refresh_status() {
            RefreshStatus::Finished(_, report) => {
                assert_eq!(report.total.discovered, 1);
                assert_eq!(report.total.parsed, 1);
                assert_eq!(report.total.upserted, 1);
                break;
            }
            RefreshStatus::Running(_) => {
                assert!(
                    std::time::Instant::now() < deadline,
                    "refresh did not finish"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            other => panic!("unexpected refresh status: {other:?}"),
        }
    }
    assert_eq!(std::fs::read_to_string(source).unwrap(), content);
    assert!(!root.path().join("llmusage").exists());
    owner.shutdown();
}

#[test]
fn insights_paths_require_both_files_and_create_nothing() {
    let root = tempfile::tempdir().unwrap();
    let usage_root = root.path().join("llmusage");
    let usage_db = usage_root.join("llmusage.db");
    let archive = root.path().join("missing").join("usage.db");
    let home = root.path().join("home");
    assert!(
        insights_paths_if_present(usage_root.clone(), &usage_db, archive.clone(), home.clone())
            .is_none()
    );
    assert!(!root.path().join("missing").exists());
    std::fs::create_dir_all(&usage_root).unwrap();
    std::fs::write(&usage_db, b"db").unwrap();
    assert!(
        insights_paths_if_present(usage_root.clone(), &usage_db, archive.clone(), home.clone())
            .is_none()
    );
    assert!(!archive.parent().unwrap().exists());
    std::fs::create_dir_all(archive.parent().unwrap()).unwrap();
    std::fs::write(&archive, b"db").unwrap();
    let paths =
        insights_paths_if_present(usage_root.clone(), &usage_db, archive.clone(), home.clone())
            .unwrap();
    assert_eq!(paths.usage_root, usage_root);
    assert_eq!(paths.archive_db, archive);
    assert_eq!(paths.provider_home, home);
}

#[gpui_kit::test]
async fn workbench_reads_seeded_usage_and_does_not_invent_sessions(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let paths = create_projection_db(&root.path().join("llmusage"));
    let pool = database::create_pool(&paths.db_path, None).unwrap();
    let conn = pool.get().unwrap();
    seed_bucket(
        &conn,
        &SeedBucket {
            hour_start: chrono::Utc::now().format("%Y-%m-%dT12:00:00Z").to_string(),
            cost_with_cache_usd: 0.123456789,
            ..Default::default()
        },
    );
    drop(conn);
    drop(pool);
    let requests = InsightsRequests::new(&services(&owner, root.path()));
    let archive = root.path().join("archive.db");
    assert!(!archive.exists());
    let mut read = requests
        .load_workbench(key(EnvironmentId::Local("fixture".into())), 7)
        .unwrap();
    let Some(TaskEvent::Terminal(result)) = read.next().await else {
        panic!("missing workbench terminal");
    };
    let Completion::Success(report) = result.as_ref() else {
        panic!("workbench failed");
    };
    assert_eq!(report.requests, 2);
    assert_eq!(report.tokens, 100);
    assert_eq!(report.cost_usd, Some(0.123456789));
    assert!(!report.sessions_indexed);
    assert_eq!(report.session_count, None);
    assert_eq!(report.session_scope, SessionScope::Unavailable);
    let codex = report
        .platforms
        .iter()
        .find(|platform| platform.id == "codex")
        .unwrap();
    let claude = report
        .platforms
        .iter()
        .find(|platform| platform.id == "claude")
        .unwrap();
    assert_eq!(codex.requests, 2);
    assert_eq!(claude.requests, 0);
    assert!(!archive.exists());
    owner.shutdown();
}
