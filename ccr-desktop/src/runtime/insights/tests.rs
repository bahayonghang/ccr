#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use crate::runtime::ConfigSource;
use ccr_app::runtime::{Completion, Platform, RuntimeOwner, TaskEvent};
use ccr_usage::fixtures::{
    SeedBucket, SeedEvent, create_projection_db, seed_bucket, seed_event, seed_run_log,
};
use std::time::Duration;

fn key(environment: EnvironmentId) -> RequestKey {
    RequestKey {
        environment,
        platform: Platform::Claude,
        generation: 1,
        request_id: 1,
    }
}
/// Noon on the local report day, as UTC. `read_workbench` windows by local dates,
/// so a UTC calendar date can fall outside that window.
fn hour_on_local_report_day() -> String {
    let noon = chrono::Local::now()
        .date_naive()
        .and_hms_opt(12, 0, 0)
        .expect("noon");
    noon.and_local_timezone(chrono::Local)
        .earliest()
        .expect("local noon")
        .with_timezone(&chrono::Utc)
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
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
fn insights_paths_accept_either_file_and_create_nothing() {
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
    let usage_only =
        insights_paths_if_present(usage_root.clone(), &usage_db, archive.clone(), home.clone())
            .unwrap();
    assert_eq!(usage_only.usage_root, usage_root);
    assert!(!archive.parent().unwrap().exists());
    std::fs::create_dir_all(archive.parent().unwrap()).unwrap();
    std::fs::write(&archive, b"db").unwrap();
    let paths =
        insights_paths_if_present(usage_root.clone(), &usage_db, archive.clone(), home.clone())
            .unwrap();
    assert_eq!(paths.usage_root, usage_root);
    assert_eq!(paths.archive_db, archive);
    assert_eq!(paths.provider_home, home);
    assert!(!home.exists());
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
            hour_start: hour_on_local_report_day(),
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
    // 没有归档时用量部分照常：活动快照仍在，只是会话未知。
    let activity = report.activity.as_ref().expect("usage-only activity");
    assert!(!activity.sessions_indexed);
    assert_eq!(activity.totals.requests, 2);
    assert_eq!(activity.daily.len(), 1);
    assert_eq!(activity.last_7_days.requests, 2);
    // 没有同步记录：不写时刻，按过期处理。
    assert_eq!(report.last_sync_at, None);
    assert_eq!(report.freshness.local, None);
    assert!(report.freshness.stale);
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
    let empty_archive = database::create_pool(&archive, None).unwrap();
    database::migrations::run_all_migrations(&empty_archive.get().unwrap(), root.path()).unwrap();
    let unindexed = requests.read_workbench(7).unwrap();
    assert_eq!(unindexed.session_scope, SessionScope::Unindexed);
    assert_eq!(unindexed.session_count, None);
    let activity = unindexed.activity.expect("unindexed activity");
    assert!(!activity.sessions_indexed);
    assert_eq!(activity.totals.requests, 2);
    owner.shutdown();
}

#[gpui_kit::test]
async fn unreadable_archive_keeps_activity_and_only_sessions_degrade(
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
            hour_start: hour_on_local_report_day(),
            ..Default::default()
        },
    );
    drop(conn);
    drop(pool);
    // 归档文件存在但不是数据库：无论连接池打不开还是查询失败，都要得到纯用量快照。
    std::fs::write(root.path().join("archive.db"), b"not a database").unwrap();
    let requests = InsightsRequests::new(&services(&owner, root.path()));
    let report = requests.read_workbench(7).unwrap();
    assert_eq!(report.session_scope, SessionScope::Unavailable);
    assert_eq!(report.session_count, None);
    assert!(!report.sessions_indexed);
    let activity = report.activity.expect("activity survives the archive");
    assert!(!activity.sessions_indexed);
    assert_eq!(activity.totals.sessions, 0);
    assert_eq!(activity.totals.requests, 2);
    assert_eq!(activity.last_7_days.requests, 2);
    assert!(activity.agents.iter().any(|agent| agent.source == "codex"));
    owner.shutdown();
}

#[gpui_kit::test]
async fn failed_usage_projection_yields_no_activity(cx: &mut gpui_kit::TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let paths = create_projection_db(&root.path().join("llmusage"));
    let pool = database::create_pool(&paths.db_path, None).unwrap();
    let dashboard = ccr_usage::open_dashboard(paths).unwrap();
    // 打开后再删表：能力探测已通过，只有投影查询失败。
    pool.get()
        .unwrap()
        .execute_batch("DROP TABLE usage_bucket_30m")
        .unwrap();
    let requests = InsightsRequests::new(&services(&owner, root.path()));
    let paths = requests.paths.clone().unwrap();
    let window = requests.activity_window(&paths, &dashboard, requests.report_clock(), 7);
    assert_eq!(window, (None, false, None, SessionScope::Unavailable));
    owner.shutdown();
}

#[test]
fn sync_freshness_uses_the_report_timezone_and_marks_older_days_stale() {
    let clock = ReportClock::fixed_utc(
        DateTime::parse_from_rfc3339("2026-11-06T09:30:00Z")
            .unwrap()
            .with_timezone(&Utc),
    );
    assert_eq!(
        SyncFreshness::at(Some("2026-11-06T02:04:00Z"), clock),
        SyncFreshness {
            local: Some("11-06 02:04".into()),
            date: Some("11-06".into()),
            stale: false,
        }
    );
    assert_eq!(
        SyncFreshness::at(Some("2026-11-05T23:59:00Z"), clock),
        SyncFreshness {
            local: Some("11-05 23:59".into()),
            date: Some("11-05".into()),
            stale: true,
        }
    );
    // 带偏移的时间先换算到报告时区再比日期：+08:00 的 07:00 是 UTC 前一天 23:00。
    let offset = SyncFreshness::at(Some("2026-11-06T07:00:00+08:00"), clock);
    assert_eq!(offset.local.as_deref(), Some("11-05 23:00"));
    assert!(offset.stale);
    // 没有记录与读不出的记录都没有时刻可写，都算过期。
    for missing in [None, Some("yesterday")] {
        let freshness = SyncFreshness::at(missing, clock);
        assert_eq!(freshness.local, None);
        assert_eq!(freshness.date, None);
        assert!(freshness.stale);
    }
}

#[gpui_kit::test]
async fn workbench_carries_the_last_sync_time_and_its_freshness(cx: &mut gpui_kit::TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let paths = create_projection_db(&root.path().join("llmusage"));
    let pool = database::create_pool(&paths.db_path, None).unwrap();
    let conn = pool.get().unwrap();
    seed_bucket(&conn, &SeedBucket::default());
    seed_run_log(&conn, "codex", "recent", "2020-01-02T03:04:00Z");
    drop(conn);
    drop(pool);
    let requests = InsightsRequests::new(&services(&owner, root.path()));
    let report = requests.read_workbench(7).unwrap();
    assert_eq!(report.last_sync_at.as_deref(), Some("2020-01-02T03:04:00Z"));
    assert_eq!(
        report.freshness,
        SyncFreshness::at(report.last_sync_at.as_deref(), requests.report_clock())
    );
    assert!(report.freshness.local.is_some());
    assert!(report.freshness.stale);
    owner.shutdown();
}

#[gpui_kit::test]
async fn workbench_folds_stored_source_aliases_without_inventing_daily_bars(
    cx: &mut gpui_kit::TestAppContext,
) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let paths = create_projection_db(&root.path().join("llmusage"));
    let pool = database::create_pool(&paths.db_path, None).unwrap();
    let conn = pool.get().unwrap();
    let hour = hour_on_local_report_day();
    seed_bucket(
        &conn,
        &SeedBucket {
            source: "antigravity_ide".into(),
            hour_start: hour.clone(),
            event_count: 5,
            total_tokens: 50,
            cost_with_cache_usd: 0.5,
            ..Default::default()
        },
    );
    seed_bucket(
        &conn,
        &SeedBucket {
            source: "omp".into(),
            hour_start: hour,
            event_count: 7,
            total_tokens: 70,
            cost_with_cache_usd: 0.25,
            ..Default::default()
        },
    );
    drop(conn);
    drop(pool);
    let requests = InsightsRequests::new(&services(&owner, root.path()));
    let mut read = requests
        .load_workbench(key(EnvironmentId::Local("fixture".into())), 7)
        .unwrap();
    let Some(TaskEvent::Terminal(result)) = read.next().await else {
        panic!("missing alias workbench terminal");
    };
    let Completion::Success(report) = result.as_ref() else {
        panic!("alias workbench failed");
    };
    assert_eq!(report.requests, 12);
    assert_eq!(report.tokens, 120);
    assert_eq!(report.cost_usd, Some(0.75));
    assert!(!report.sessions_indexed);
    assert!(
        report
            .platforms
            .iter()
            .all(|platform| platform.id != "antigravity_ide" && platform.id != "omp")
    );
    let antigravity = report
        .platforms
        .iter()
        .find(|platform| platform.id == "antigravity")
        .unwrap();
    let pi = report
        .platforms
        .iter()
        .find(|platform| platform.id == "pi")
        .unwrap();
    assert_eq!(antigravity.requests, 5);
    assert_eq!(antigravity.tokens, 50);
    assert!(antigravity.requests_by_day.is_empty());
    assert_eq!(pi.requests, 7);
    assert_eq!(pi.tokens, 70);
    assert!(pi.requests_by_day.is_empty());
    assert_eq!(report.other_requests, 12);
    owner.shutdown();
}

#[gpui_kit::test]
async fn missing_archive_is_unindexed_and_creates_nothing(cx: &mut gpui_kit::TestAppContext) {
    cx.executor().allow_parking();
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let requests = InsightsRequests::new(&services(&owner, root.path()));
    let mut read = requests
        .load(
            key(EnvironmentId::Local("fixture".into())),
            InsightsQuery::Sessions(SessionListRequest::default()),
        )
        .unwrap();
    let Some(TaskEvent::Terminal(result)) = read.next().await else {
        panic!("missing session terminal");
    };
    let Completion::Success(snapshot) = result.as_ref() else {
        panic!("session read failed");
    };
    let InsightsSnapshot::Sessions {
        page,
        statuses,
        indexed,
    } = snapshot.as_ref()
    else {
        panic!("wrong session snapshot");
    };
    assert!(!indexed);
    assert!(page.items.is_empty());
    assert_eq!(statuses.len(), 8);
    assert!(statuses.iter().all(|status| {
        status.availability == ccr_store::sessions::ProviderAvailability::NotInstalled
    }));
    assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    owner.shutdown();
}

#[test]
fn usage_latency_samples_only_successful_opened_report_body_and_keeps_captured_owners_isolated() {
    let owner = RuntimeOwner::new().unwrap();
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    create_projection_db(&first.path().join("llmusage"));
    create_projection_db(&second.path().join("llmusage"));
    let selected = services(&owner, first.path());
    let other = services(&owner, second.path());
    let requests = InsightsRequests::new(&selected);
    let snapshot = |services: &Services| services.usage_report_latency.lock().unwrap().snapshot();
    assert_eq!(snapshot(&selected).p95_ms, None);
    assert_eq!(snapshot(&other).sample_count, 0);
    requests
        .read(InsightsQuery::Usage(QueryFilter::default()))
        .unwrap();
    assert_eq!(snapshot(&selected).sample_count, 1);
    assert!(snapshot(&selected).p95_ms.unwrap().is_finite());
    assert_eq!(snapshot(&other).sample_count, 0);
    // Queries other than the successful report body never enter this cohort.
    requests
        .read(InsightsQuery::Logs {
            filter: QueryFilter::default(),
            cursor: None,
        })
        .unwrap();
    let invalid = QueryFilter {
        since: NaiveDate::from_ymd_opt(2000, 1, 2),
        until: NaiveDate::from_ymd_opt(2000, 1, 1),
        ..Default::default()
    };
    assert!(requests.read(InsightsQuery::Usage(invalid)).is_err());
    assert_eq!(snapshot(&selected).sample_count, 1);
    let unchanged = selected
        .clone()
        .with_insights_paths(selected.insights_paths().unwrap());
    assert!(Arc::ptr_eq(
        &selected.usage_report_latency,
        &unchanged.usage_report_latency
    ));
    let rebound = selected
        .clone()
        .with_insights_paths(other.insights_paths().unwrap());
    assert!(!Arc::ptr_eq(
        &selected.usage_report_latency,
        &rebound.usage_report_latency
    ));
    assert_eq!(snapshot(&rebound).sample_count, 0);
    let absent = tempfile::tempdir().unwrap();
    let missing = services(&owner, absent.path());
    assert!(
        InsightsRequests::new(&missing)
            .read(InsightsQuery::Usage(QueryFilter::default()))
            .is_err()
    );
    assert_eq!(snapshot(&missing).sample_count, 0);
    assert!(!absent.path().join("llmusage").exists());
    assert_eq!(snapshot(&selected.clone()).sample_count, 1);
    owner.shutdown();
}
