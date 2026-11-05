#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use ccr_db::database::repositories::usage_repo::{
    UsageSessionArchiveEntry, UsageSourceState, agent_session_archive_id,
    upsert_session_archive_entry,
};
use ccr_db::database::{DbPool, create_pool, migrations::run_all_migrations};
use ccr_usage::fixtures::{SeedBucket, create_projection_db, seed_bucket};
use ccr_usage::{AppPaths, QueryFilter, ReportTimezone};
use tempfile::TempDir;

fn timestamp(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .expect("fixture timestamp")
        .with_timezone(&Utc)
}

fn archive_pool(temp: &TempDir) -> DbPool {
    let pool = create_pool(&temp.path().join("archive.db"), None).expect("fixture pool");
    let conn = pool.get().expect("fixture connection");
    run_all_migrations(&conn, temp.path()).expect("fixture migrations");
    drop(conn);
    pool
}

fn seed_session(pool: &DbPool, platform: &str, file: &str, created_at: &str) {
    let created_at = timestamp(created_at);
    let entry = UsageSessionArchiveEntry {
        archive_id: agent_session_archive_id(platform, file, ""),
        session_id: file.into(),
        platform: platform.into(),
        title: None,
        cwd: String::new(),
        file_path: file.into(),
        file_hash: None,
        source_variant: "fixture".into(),
        source_kind: "file".into(),
        source_member_id: String::new(),
        source_size: None,
        source_mtime_ns: None,
        source_stat_hash: None,
        message_count: 1,
        user_message_count: 1,
        assistant_message_count: 0,
        tool_use_count: 0,
        source_fidelity: "full".into(),
        created_at,
        updated_at: created_at,
        source_state: UsageSourceState::Live,
        last_seen_at: Some(created_at),
        raw_deleted_at: None,
        archived_at: created_at,
    };
    upsert_session_archive_entry(&pool.get().expect("fixture connection"), &entry)
        .expect("fixture archive row");
}

fn usage_fixture(temp: &TempDir) -> AppPaths {
    let paths = create_projection_db(&temp.path().join("llmusage"));
    let pool = create_pool(&paths.db_path, None).expect("fixture seed pool");
    let conn = pool.get().expect("fixture seed connection");
    for (source, requests) in [("codex", 4), ("mystery-cli", 5), ("pi", 2)] {
        seed_bucket(
            &conn,
            &SeedBucket {
                source: source.into(),
                event_count: requests,
                hour_start: "2026-09-24T12:00:00Z".into(),
                ..SeedBucket::default()
            },
        );
    }
    drop(conn);
    drop(pool);
    paths
}

fn read(paths: AppPaths, pool: &DbPool) -> HomeInsightsSnapshot {
    read_home_insights(
        paths,
        &QueryFilter {
            timezone: ReportTimezone::Utc,
            ..Default::default()
        },
        pool,
        NaiveDate::from_ymd_opt(2026, 9, 24).expect("fixture date"),
        timestamp("2026-09-24T12:00:00Z"),
    )
    .expect("shared home projection")
}

#[test]
fn shared_home_joins_exact_projection_windows_and_canonical_sources_without_writes() {
    let temp = TempDir::new().expect("fixture root");
    let paths = usage_fixture(&temp);
    let pool = archive_pool(&temp);
    seed_session(&pool, "omp", "pi.jsonl", "2026-09-24T23:59:59Z");
    seed_session(
        &pool,
        "mystery-cli",
        "unknown.jsonl",
        "2026-09-18T00:00:00Z",
    );
    seed_session(&pool, "codex", "previous.jsonl", "2026-09-17T23:59:59Z");
    seed_session(&pool, "codex", "future.jsonl", "2026-09-25T00:00:00Z");
    let before = std::fs::read(&paths.db_path).expect("fixture bytes");

    let snapshot = read(paths.clone(), &pool);

    assert_eq!(snapshot.totals.requests, 11);
    assert_eq!(snapshot.totals.sessions, 4);
    assert_eq!(snapshot.last_7_days.sessions, 2);
    assert_eq!(snapshot.previous_7_days.sessions, 1);
    assert_eq!(snapshot.last_7_days.requests, 11);
    assert!(snapshot.sessions_indexed);
    let pi = snapshot
        .agents
        .iter()
        .find(|row| row.source == "pi")
        .expect("pi row");
    assert_eq!((pi.requests, pi.sessions, pi.unmapped), (2, 1, false));
    let unknown = snapshot
        .agents
        .iter()
        .filter(|row| row.source == "mystery-cli")
        .collect::<Vec<_>>();
    assert_eq!(unknown.len(), 1);
    assert_eq!(
        (
            unknown[0].requests,
            unknown[0].sessions,
            unknown[0].unmapped
        ),
        (5, 1, true)
    );
    assert_eq!(snapshot.generated_at, "2026-09-24T12:00:00+00:00");
    assert_eq!(
        before,
        std::fs::read(&paths.db_path).expect("fixture bytes after query")
    );
}

#[test]
fn shared_home_preserves_unindexed_state_and_all_exact_distribution_values() {
    let temp = TempDir::new().expect("fixture root");
    let paths = usage_fixture(&temp);
    let pool = archive_pool(&temp);
    let snapshot = read(paths.clone(), &pool);
    assert!(!snapshot.sessions_indexed);
    assert_eq!(snapshot.totals.sessions, 0);
    let projected = ccr_usage::open_dashboard(paths)
        .expect("fixture projection")
        .insights(
            &QueryFilter {
                timezone: ReportTimezone::Utc,
                ..Default::default()
            },
            NaiveDate::from_ymd_opt(2026, 9, 24).expect("fixture date"),
        )
        .expect("owner projection");
    assert_eq!(snapshot.hourly, projected.hourly);
    assert_eq!(snapshot.weekday, projected.weekday);
    assert_eq!(snapshot.monthly, projected.monthly);
    assert_eq!(snapshot.trend, projected.trend);
    assert_eq!(snapshot.projects, projected.projects);
    assert_eq!(snapshot.models, projected.models);
}

#[test]
fn shared_home_missing_usage_does_not_create_or_fabricate_a_zero_snapshot() {
    let temp = TempDir::new().expect("fixture root");
    let pool = archive_pool(&temp);
    let missing = temp.path().join("absent-llmusage");
    let result = read_home_insights(
        AppPaths::from_root(&missing),
        &QueryFilter::default(),
        &pool,
        NaiveDate::from_ymd_opt(2026, 9, 24).expect("fixture date"),
        timestamp("2026-09-24T12:00:00Z"),
    );
    assert!(
        result
            .expect_err("missing usage must fail")
            .starts_with("Dashboard open error:")
    );
    assert!(!missing.exists());
}
