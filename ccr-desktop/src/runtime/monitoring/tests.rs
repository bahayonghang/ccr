#![allow(clippy::unwrap_used, clippy::expect_used)]
use super::*;
use ccr_app::runtime::{Completion, Platform, RuntimeOwner, TaskEvent};
use ccr_db::database::{create_pool, migrations::run_all_migrations};
use std::time::Duration;

pub(crate) fn source(root: &Path) -> ConfigSource {
    ConfigSource::Fixture {
        profiles: root.join("profiles.toml"),
        registry: root.join("registry.toml"),
        delay: Duration::ZERO,
    }
}
pub(crate) fn seed(root: &Path, name: &str, entry: &MonitoringEntry) {
    let home = root.join("migration-home");
    std::fs::create_dir_all(&home).unwrap();
    let pool = create_pool(&root.join(name), None).unwrap();
    let conn = pool.get().unwrap();
    run_all_migrations(&conn, &home).unwrap();
    LogPersistenceService::append_entry_with_connection(&conn, entry).unwrap();
}
fn key(environment: EnvironmentId) -> RequestKey {
    RequestKey {
        environment,
        platform: Platform::Claude,
        generation: 1,
        request_id: 1,
    }
}
#[test]
fn main_storage_is_distinct_from_archive_and_missing_reads_never_create_it() {
    let root = tempfile::tempdir().unwrap();
    let source = source(root.path());
    seed(
        root.path(),
        "archive.db",
        &MonitoringEntry::new(
            MonitoringLevel::Info,
            "archive",
            "not.main",
            "test",
            "archive-only",
        ),
    );
    let owner = Owner::default();
    let absent = owner.read(&source, &MonitoringFeedQuery::default(), 0);
    assert_eq!(absent.storage, Err(Error::Missing));
    assert!(absent.entries.is_empty());
    assert!(!root.path().join("observer.db").exists());
    owner.terminal(&source, "commands", "command.completed", false);
    let absent = owner.read(&source, &MonitoringFeedQuery::default(), 0);
    assert_eq!(absent.last_write, Some(Err(Error::Missing)));
    assert_eq!(absent.entries.len(), 1);
    assert!(!root.path().join("observer.db").exists());
    seed(
        root.path(),
        "observer.db",
        &MonitoringEntry::new(
            MonitoringLevel::Warn,
            "system",
            "main.legacy",
            "test",
            "legacy record",
        ),
    );
    let present = owner.read(&source, &MonitoringFeedQuery::default(), 7);
    assert_eq!(present.storage, Ok(()));
    assert_eq!(present.entries.len(), 2);
    assert!(
        present
            .entries
            .iter()
            .all(|entry| entry.message != "archive-only")
    );
    owner.terminal(&source, "tools", "cli.update.failed", true);
    let persisted = owner.read(
        &source,
        &MonitoringFeedQuery {
            count: Some(500),
            level: Some(MonitoringLevel::Error),
            channel: Some("tools".into()),
        },
        7,
    );
    assert_eq!(persisted.entries.len(), 1);
    assert_eq!(persisted.metrics.active_tasks, 7);
    assert_eq!(persisted.last_write, Some(Ok(())));
    let reopened = Owner::default().read(&source, &MonitoringFeedQuery::default(), 0);
    assert!(
        reopened
            .entries
            .iter()
            .any(|entry| entry.event_type == "cli.update.failed")
    );
    assert!(
        !reopened
            .entries
            .iter()
            .any(|entry| entry.event_type == "command.completed")
    );
}
#[test]
fn bounded_feed_redacts_before_storage_and_reports_decode_errors() {
    let root = tempfile::tempdir().unwrap();
    let source = source(root.path());
    seed(
        root.path(),
        "observer.db",
        &MonitoringEntry::new(MonitoringLevel::Info, "test", "ready", "test", "ready"),
    );
    let owner = Owner::default();
    let mut entry = MonitoringEntry::new(
        MonitoringLevel::Error,
        "commands",
        "test.failed",
        "test",
        "Authorization: Bearer sk-secret-sentinel",
    );
    entry.fields = Some(serde_json::json!({"api_key":"field-secret-sentinel"}));
    entry.correlation_id = Some("correlation-secret-sentinel".into());
    owner.record(&source, entry);
    let conn = connection(&root.path().join("observer.db"), false).unwrap();
    let stored =
        LogPersistenceService::query_logs_with_connection(&conn, Some("error"), None, 500).unwrap();
    let wire = serde_json::to_string(&stored).unwrap();
    for secret in [
        "sk-secret-sentinel",
        "field-secret-sentinel",
        "correlation-secret-sentinel",
    ] {
        assert!(!wire.contains(secret));
    }
    drop(conn);
    // A missing DB keeps bounded memory events without performing hundreds of writes.
    let absent_root = tempfile::tempdir().unwrap();
    let absent_source = self::source(absent_root.path());
    let memory = Owner::default();
    for _ in 0..ENTRY_LIMIT + 9 {
        memory.record(
            &absent_source,
            MonitoringEntry::new(
                MonitoringLevel::Info,
                "test",
                "event",
                "test",
                "x".repeat(8192),
            ),
        );
    }
    let report = memory.read(
        &absent_source,
        &MonitoringFeedQuery {
            count: Some(usize::MAX),
            ..Default::default()
        },
        0,
    );
    assert!(report.metrics.retained_entries <= ENTRY_LIMIT);
    assert!(report.metrics.retained_payload_bytes <= PAYLOAD_LIMIT);
    assert!(report.metrics.evicted_entries > 0);
    let conn = connection(&root.path().join("observer.db"), true).unwrap();
    conn.execute("UPDATE log_entries SET message = x'01'", [])
        .unwrap();
    let report = owner.read(&source, &MonitoringFeedQuery::default(), 0);
    assert_eq!(report.storage, Err(Error::Storage));
    assert!(!report.entries.is_empty()); // actual in-memory events survive the storage error
    conn.execute(
        "UPDATE log_entries SET message = 'safe', timestamp = 'invalid-time'",
        [],
    )
    .unwrap();
    let report = owner.read(&source, &MonitoringFeedQuery::default(), 0);
    assert_eq!(report.storage, Err(Error::Storage));
    assert!(!report.entries.is_empty());
}
#[test]
fn environment_event_is_retained_after_detach_and_shutdown_remote_reads_refuse() {
    let root = tempfile::tempdir().unwrap();
    let owner = RuntimeOwner::new().unwrap();
    let services = Services::new(owner.handle(), source(root.path()));
    services.record_environment_selection(key(EnvironmentId::Local("fixture".into())));
    owner.shutdown();
    assert_eq!(services.monitoring.state.lock().unwrap().entries.len(), 1);
    let owner = RuntimeOwner::new().unwrap();
    let services = Services::new(owner.handle(), source(root.path()));
    let mut task = Requests::new(&services)
        .refresh(
            key(EnvironmentId::Ssh("fixture".into())),
            MonitoringFeedQuery::default(),
        )
        .unwrap();
    let wait_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    wait_runtime.block_on(async {
        loop {
            if let Some(TaskEvent::Terminal(result)) = task.next().await {
                assert!(matches!(
                    result.as_ref(),
                    Completion::Failed(Error::LocalOnly)
                ));
                break;
            }
        }
    });
    drop(wait_runtime);
    assert!(services.monitoring.path.get().is_none());
    owner.shutdown();
}

#[test]
fn ordinary_push_evictions_are_scoped_and_final_detach_freezes_counter() {
    let root = tempfile::tempdir().unwrap();
    let source = source(root.path());
    let owner = Owner::default();
    owner
        .track_evictions(ccr_app::benchmark::s2::EVENT_PREFIX)
        .unwrap();
    let at = chrono::Utc::now();
    for slot in 0..2 {
        owner.record(&source, ccr_app::benchmark::s2::entry(at, slot));
    }
    for _ in 0..ENTRY_LIMIT {
        owner.terminal(&source, "ordinary", "ordinary", false);
    }
    assert_eq!(owner.eviction_delta(true), 2);
    for _ in 0..ENTRY_LIMIT {
        owner.terminal(&source, "ordinary", "ordinary", false);
    }
    assert_eq!(owner.eviction_delta(false), 0);
}
