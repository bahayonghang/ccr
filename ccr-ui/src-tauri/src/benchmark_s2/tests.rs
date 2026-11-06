#![allow(clippy::unwrap_used)]
use super::*;
#[test]
fn local_cannot_obtain_seed_or_start_context() {
    assert!(seed().is_err());
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn production_profile_cancel_joins_real_batched_storage_and_scoped_evictions() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    std::fs::create_dir(&home).unwrap();
    let pool = ccr_db::database::create_app_pool_at(&root.path().join("main.db"), &home).unwrap();
    let logs = ccr_db::services::log_persistence::LogPersistenceService::with_db(
        Default::default(),
        ccr_db::database::DbAccess::Pool(pool.clone()),
    );
    let event_log = crate::events::EventLog::with_limits(4, 10 * 1024, 5 * 1024 * 1024);
    let tracker = event_log
        .register_eviction_tracker(s2::EVENT_PREFIX)
        .await
        .unwrap();
    let core = Arc::new(Owner::default());
    let seed = Seed {
        root: root.path().into(),
        id: "benchmark-s2".into(),
        scenario: "S2".into(),
        logical_sha256: "a".repeat(64),
        as_of: chrono::DateTime::parse_from_rfc3339("2000-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc),
        timezone: ccr_usage::ReportTimezone::Utc,
        profile_count: 10000,
        session_count: 10000,
        usage_count: 100000,
    };
    let run = core.admit(&seed).unwrap();
    let id = core.snapshot().run_id.unwrap();
    assert!(core.request_cancel("foreign").is_err());
    let sink = Arc::new(Sink {
        context: SinkContext::Headless {
            event_log,
            logs: logs.clone(),
        },
        core: core.clone(),
        pending: AsyncMutex::new(Pending {
            tickets: Vec::with_capacity(s2::CAPACITY),
            tracker: Some(tracker),
            evicted: 0,
        }),
    });
    let deliver = sink.clone();
    let flush = sink.clone();
    let runtime = tokio::runtime::Handle::current();
    let driver = tokio::task::spawn_blocking(move || {
        runtime.block_on(run.drive(
            CancellationToken::new(),
            move |entry| {
                let sink = deliver.clone();
                async move { sink.delivery(entry).await }
            },
            move || {
                let sink = flush.clone();
                async move { sink.flush(false).await }
            },
        ))
    });
    for _ in 0..100 {
        if core.snapshot().ring_accepted >= 5 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    // A normal unit caller flushes a mixed batch. Its rows are never attributed
    // to this run; ticket state still reaches the S2 owner when its receipt drops.
    let ordinary = ccr_types::MonitoringEntry::new(
        ccr_types::MonitoringLevel::Info,
        "ordinary",
        "ordinary",
        "test",
        "ordinary",
    );
    for index in 0..20 {
        let mut entry = ordinary.clone();
        entry.id = format!("ordinary-{index}");
        logs.append_entry(&entry).await;
    }
    logs.force_flush().await;
    if let SinkContext::Headless { event_log, .. } = &sink.context {
        for _ in 0..8 {
            event_log
                .push(crate::events::AppEvent::Monitoring(ordinary.clone()))
                .await;
        }
    }
    core.request_cancel(&id).unwrap();
    let outcome = driver.await.unwrap();
    assert!(!core.snapshot().phase.terminal());
    core.apply(sink.flush(true).await);
    let terminal = core.complete_joined(outcome).unwrap();
    assert!(terminal.ring_accepted >= 1);
    assert_eq!(terminal.db_pending, 0);
    assert_eq!(terminal.queue_pending, 0);
    assert_eq!(terminal.db_failed, 0);
    assert_eq!(terminal.db_attempted, terminal.db_committed);
    assert_eq!(terminal.emit_attempted, terminal.ring_accepted);
    assert!(terminal.ring_evicted > 0);
    let stored: i64 = pool
        .get()
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM log_entries WHERE id LIKE 'benchmark-s2-v1-%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stored as u64, terminal.db_committed);
    assert!(core.admit(&seed).is_err());
    if let SinkContext::Headless { event_log, .. } = &sink.context {
        for _ in 0..8 {
            event_log
                .push(crate::events::AppEvent::Monitoring(ordinary.clone()))
                .await;
        }
    }
    assert_eq!(core.snapshot().revision, terminal.revision);
    assert_eq!(core.snapshot().ring_evicted, terminal.ring_evicted);
}
