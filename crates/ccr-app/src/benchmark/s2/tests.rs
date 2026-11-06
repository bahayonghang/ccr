#![allow(clippy::unwrap_used)]
use super::*;
fn seed() -> Seed {
    Seed {
        root: std::env::temp_dir(),
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
    }
}
#[test]
fn source_identity_once_and_deterministic_events() {
    let slot = AtomicBool::new(false);
    let owner = Arc::new(Owner::default());
    let mut wrong = seed();
    wrong.scenario = "S1".into();
    assert!(owner.admit_once(&wrong, &slot).is_err());
    assert!(!slot.load(Ordering::SeqCst));
    let run = owner.admit_once(&seed(), &slot).unwrap();
    assert!(
        Arc::new(Owner::default())
            .admit_once(&seed(), &slot)
            .is_err()
    );
    assert!(owner.request_cancel("foreign").is_err());
    let a = entry(seed().as_of, 12);
    let b = entry(seed().as_of, 12);
    assert_eq!(a.id, b.id);
    assert_eq!(a.timestamp, b.timestamp);
    assert!(owns_event(&a.id));
    assert!(!owns_event("benchmark-s2-v1-60000"));
    assert!(owner.complete_joined(None).is_err());
    drop(run);
    assert_eq!(owner.snapshot().phase, Phase::Cleaning);
    owner
        .complete_joined(Some("benchmark_s2_not_started"))
        .unwrap();
    assert_eq!(owner.snapshot().phase, Phase::Failed);
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_short_source_counts_actual_stages_and_joins() {
    let owner = Arc::new(Owner::default());
    let slot = AtomicBool::new(false);
    let run = owner.admit_once(&seed(), &slot).unwrap();
    let entries = Arc::new(Mutex::new(Vec::new()));
    let observed = entries.clone();
    let outcome = run
        .drive(
            CancellationToken::new(),
            move |entry| {
                lock(&observed).push(entry.id);
                async {
                    Ack {
                        ring: Some(true),
                        db_attempted: 1,
                        db_committed: 1,
                        emit: Some(true),
                        ..Default::default()
                    }
                }
            },
            || async { Ack::default() },
        )
        .await;
    owner.complete_joined(outcome).unwrap();
    let out = owner.snapshot();
    assert_eq!(out.phase, Phase::Completed);
    assert_eq!(out.attempted + out.scheduler_skipped, out.scheduled_slots);
    assert_eq!(out.scheduled_slots, ACTIVE_SLOTS);
    assert_eq!(out.attempted, out.queue_accepted + out.queue_dropped);
    assert_eq!(out.ring_accepted, out.db_committed);
    assert_eq!(out.queue_pending, 0);
    assert!(out.source_elapsed_ms.unwrap() > 0.0);
    assert!(out.actual_delivered_per_second.unwrap().is_finite());
    let before = out.revision;
    owner
        .request_cancel(out.run_id.as_deref().unwrap())
        .unwrap();
    assert_eq!(owner.snapshot().revision, before);
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn slow_sink_never_catches_up_and_budget_abandons_unstarted_queue() {
    let owner = Arc::new(Owner::default());
    let slot = AtomicBool::new(false);
    let run = owner.admit_once(&seed(), &slot).unwrap();
    let outcome = run
        .drive(
            CancellationToken::new(),
            |_| async {
                tokio::time::sleep(Duration::from_millis(900)).await;
                Ack {
                    ring: Some(true),
                    ..Default::default()
                }
            },
            || async { Ack::default() },
        )
        .await;
    owner.complete_joined(outcome).unwrap();
    let out = owner.snapshot();
    assert!(out.queue_dropped + out.scheduler_skipped > 0);
    assert!(out.queue_abandoned > 0);
    assert!(out.cleanup_over_budget);
    assert_eq!(out.phase, Phase::Failed);
    assert_eq!(out.queue_pending, 0);
    assert_eq!(out.ring_accepted, 1);
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancellation_and_final_io_remain_nonterminal_until_ack() {
    let owner = Arc::new(Owner::default());
    let slot = AtomicBool::new(false);
    let run = owner.admit_once(&seed(), &slot).unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel();
    let release = Arc::new(Mutex::new(Some(rx)));
    let pending = release.clone();
    let task = tokio::spawn(run.drive(
        CancellationToken::new(),
        |_| async {
            Ack {
                ring: Some(true),
                ..Default::default()
            }
        },
        move || {
            let rx = lock(&pending).take();
            async move {
                if let Some(rx) = rx {
                    let _ = rx.await;
                }
                Ack::default()
            }
        },
    ));
    tokio::time::sleep(Duration::from_millis(8)).await;
    let id = owner.snapshot().run_id.unwrap();
    owner.request_cancel(&id).unwrap();
    tokio::time::sleep(Duration::from_millis(15)).await;
    assert!(!owner.snapshot().phase.terminal());
    assert_eq!(owner.snapshot().phase, Phase::Cleaning);
    tx.send(()).unwrap();
    let outcome = task.await.unwrap();
    assert!(!owner.snapshot().phase.terminal());
    owner.complete_joined(outcome).unwrap();
    assert_eq!(owner.snapshot().phase, Phase::Cancelled);
}

#[test]
fn each_active_projection_has_its_own_revision_and_terminal_is_frozen() {
    let owner = Arc::new(Owner::default());
    let slot = AtomicBool::new(false);
    let run = owner.admit_once(&seed(), &slot).unwrap();
    let first = owner.snapshot();
    owner.apply(Ack {
        ring: Some(true),
        db_attempted: 1,
        db_committed: 1,
        ..Default::default()
    });
    let second = owner.snapshot();
    assert!(second.revision > first.revision);
    assert_eq!(first.db_committed, 0);
    assert_eq!(second.db_committed, 1);
    // No millisecond resolution is required to reject this older reply.
    let newest = if first.revision > second.revision {
        first
    } else {
        second
    };
    assert_eq!(newest.db_committed, 1);
    drop(run);
    owner
        .complete_joined(Some("benchmark_s2_not_started"))
        .unwrap();
    let terminal = owner.snapshot();
    std::thread::sleep(Duration::from_millis(2));
    owner.apply(Ack {
        ring: Some(true),
        ..Default::default()
    });
    let retained = owner.snapshot();
    assert_eq!(terminal.revision, retained.revision);
    assert_eq!(terminal.ring_accepted, retained.ring_accepted);
    assert_eq!(terminal.elapsed_ms, retained.elapsed_ms);
    assert_eq!(terminal.cleanup_over_budget, retained.cleanup_over_budget);
}
