#![allow(clippy::unwrap_used)]
use super::*;
use std::sync::mpsc;

fn key(generation: u64) -> RequestKey {
    RequestKey {
        environment: EnvironmentId::Local("fixture".into()),
        platform: Platform::Claude,
        generation,
        request_id: generation,
    }
}

fn owner() -> RuntimeOwner {
    RuntimeOwner::new().unwrap_or_else(|error| panic!("runtime: {error}"))
}

#[test]
fn slow_blocking_read_does_not_block_another_read() {
    let owner = owner();
    let handle = owner.handle();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let _slow = handle
        .read(key(1), move |_, _| {
            entered_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            Ok::<_, ()>("slow")
        })
        .unwrap();
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let mut quick = handle.read(key(2), |_, _| Ok::<_, ()>("quick")).unwrap();
    owner.runtime.block_on(async {
        let event = tokio::time::timeout(Duration::from_secs(5), quick.next()).await.unwrap();
        assert!(matches!(event, Some(TaskEvent::Terminal(value)) if *value == Completion::Success("quick")));
    });
    release_tx.send(()).unwrap();
    owner.shutdown();
    assert_eq!(handle.active_tasks(), 0);
}

#[test]
fn request_gate_rejects_late_environment_platform_generation_and_request() {
    let mut gate = ResponseGate::default();
    let first = key(1);
    gate.replace(first.clone());
    assert!(gate.accepts(&first));
    let mut changed = first.clone();
    changed.environment = EnvironmentId::Ssh("fixture".into());
    gate.replace(changed.clone());
    assert!(!gate.accepts(&first));
    for field in 0..3 {
        let mut late = changed.clone();
        match field {
            0 => late.platform = Platform::Codex,
            1 => late.generation += 1,
            _ => late.request_id += 1,
        }
        assert!(!gate.accepts(&late));
    }
    gate.clear();
    assert!(!gate.accepts(&changed));
}

#[test]
fn actual_late_worker_result_cannot_overwrite_a_new_environment() {
    let owner = owner();
    let handle = owner.handle();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let old_key = key(1);
    let mut old = handle
        .read(old_key.clone(), move |_, _| {
            entered_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            Ok::<_, ()>("old")
        })
        .unwrap();
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let mut current_key = key(2);
    current_key.environment = EnvironmentId::Local("other-environment".into());
    let mut gate = ResponseGate::default();
    gate.replace(current_key.clone());
    let mut current = handle
        .read(current_key.clone(), |_, _| Ok::<_, ()>("current"))
        .unwrap();
    owner.runtime.block_on(async {
        assert!(matches!(current.next().await, Some(TaskEvent::Terminal(value)) if *value == Completion::Success("current")));
        assert!(gate.accepts(&current_key));
        release_tx.send(()).unwrap();
        assert!(matches!(old.next().await, Some(TaskEvent::Terminal(value)) if *value == Completion::Success("old")));
        assert!(!gate.accepts(&old_key));
    });
    owner.shutdown();
}

#[tokio::test(start_paused = true)]
async fn progress_coalesces_latest_at_most_ten_times_a_second() {
    let reporter = ProgressReporter(Arc::new(PendingProgress {
        value: Mutex::new(None),
        changed: Notify::new(),
    }));
    let (tx, rx) = async_channel::bounded(NOTIFICATION_CAPACITY);
    let task = tokio::spawn(forward_progress(reporter.clone(), tx));
    reporter.report(Progress {
        completed: 1,
        total: Some(20),
    });
    tokio::task::yield_now().await;
    assert!(matches!(
        rx.recv().await.unwrap(),
        Wake::Progress(Progress { completed: 1, .. })
    ));
    for completed in 2..=20 {
        reporter.report(Progress {
            completed,
            total: Some(20),
        });
    }
    tokio::time::advance(Duration::from_millis(99)).await;
    assert!(rx.try_recv().is_err());
    tokio::time::advance(Duration::from_millis(1)).await;
    tokio::task::yield_now().await;
    assert!(matches!(
        rx.recv().await.unwrap(),
        Wake::Progress(Progress { completed: 20, .. })
    ));
    assert!(rx.try_recv().is_err());
    task.abort();
}

#[tokio::test]
async fn full_notification_queue_cannot_drop_terminal_or_rewrite_it() {
    let (tx, rx) = async_channel::bounded(NOTIFICATION_CAPACITY);
    let state = Arc::new(State {
        terminal: Mutex::new(None),
        cancel: CancellationToken::new(),
    });
    for completed in 0..NOTIFICATION_CAPACITY {
        tx.try_send(Wake::Progress(Progress {
            completed: completed as u64,
            total: None,
        }))
        .unwrap();
    }
    assert_eq!(rx.len(), 128);
    assert!(tx.try_send(Wake::Terminal).is_err());
    let mut subscription = TaskSubscription {
        key: key(1),
        state: state.clone(),
        notifications: rx,
        cancel_on_drop: false,
        terminal_delivered: false,
    };
    publish_terminal(&state, &tx, Completion::<_, ()>::Success(42));
    publish_terminal(&state, &tx, Completion::Cancelled);
    subscription.request_cancel();
    assert!(
        matches!(subscription.next().await, Some(TaskEvent::Terminal(value)) if *value == Completion::Success(42))
    );
    assert!(subscription.next().await.is_none());
}

#[test]
fn read_drop_cancels_and_reclaims_subscription_but_waits_for_worker() {
    let owner = owner();
    let handle = owner.handle();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let read = handle
        .read(key(1), move |cancel, _| {
            entered_tx.send(cancel).unwrap();
            release_rx.recv().unwrap();
            Ok::<_, ()>(())
        })
        .unwrap();
    let cancel = entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let state = read.state.clone();
    let receiver = read.notifications.clone();
    drop(read);
    assert!(cancel.is_cancelled());
    assert!(receiver.is_closed());
    assert!(lock(&state.terminal).is_none());
    release_tx.send(()).unwrap();
    owner.shutdown();
    assert!(matches!(
        lock(&state.terminal).as_deref(),
        Some(Completion::Cancelled)
    ));
    assert_eq!(handle.active_tasks(), 0);
}

#[test]
fn job_page_drop_only_detaches_and_cleanup_failure_has_terminal_ownership() {
    let owner = owner();
    let handle = owner.handle();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let job = handle
        .job(key(1), move |cancel, _| {
            entered_tx.send(cancel).unwrap();
            release_rx.recv().unwrap();
            Completion::<(), _>::CleanupFailed("cleanup proof absent")
        })
        .unwrap();
    let cancel = entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let state = job.state.clone();
    drop(job);
    assert!(!cancel.is_cancelled());
    assert!(lock(&state.terminal).is_none());
    release_tx.send(()).unwrap();
    owner.shutdown();
    assert!(matches!(
        lock(&state.terminal).as_deref(),
        Some(Completion::CleanupFailed("cleanup proof absent"))
    ));
}

#[test]
fn admission_is_bounded_and_queued_cancel_does_not_start_work() {
    let owner = owner();
    let handle = owner.handle();
    let barrier = Arc::new(std::sync::Barrier::new(BLOCKING_CONCURRENCY + 1));
    let mut releases = Vec::new();
    let mut reads = Vec::new();
    for _ in 0..BLOCKING_CONCURRENCY {
        let entered = barrier.clone();
        let (release_tx, release_rx) = mpsc::channel();
        releases.push(release_tx);
        reads.push(
            handle
                .read(key(1), move |_, _| {
                    entered.wait();
                    release_rx.recv().unwrap();
                    Ok::<_, ()>(())
                })
                .unwrap(),
        );
    }
    barrier.wait();
    let ran = Arc::new(std::sync::atomic::AtomicBool::new(false));
    for _ in BLOCKING_CONCURRENCY..MAX_OPERATIONS {
        let ran = ran.clone();
        reads.push(
            handle
                .read(key(1), move |_, _| {
                    ran.store(true, std::sync::atomic::Ordering::SeqCst);
                    Ok::<_, ()>(())
                })
                .unwrap(),
        );
    }
    assert!(matches!(
        handle.read(key(2), |_, _| Ok::<_, ()>(())),
        Err(AdmissionError::Full)
    ));
    for read in &reads {
        read.request_cancel();
    }
    for release in releases {
        release.send(()).unwrap();
    }
    owner.shutdown();
    assert!(!ran.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(handle.active_tasks(), 0);
    assert!(matches!(
        handle.read(key(2), |_, _| Ok::<_, ()>(())),
        Err(AdmissionError::Stopped)
    ));
}

#[test]
fn cancel_does_not_publish_terminal_until_runner_returns_and_shutdown_joins() {
    let owner = owner();
    let handle = owner.handle();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let read = handle
        .read(key(1), move |_, _| {
            entered_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            Ok::<_, ()>(())
        })
        .unwrap();
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    read.request_cancel();
    assert!(read.terminal().is_none());
    let (done_tx, done_rx) = mpsc::channel();
    let shutdown = std::thread::spawn(move || {
        owner.shutdown();
        done_tx.send(()).unwrap();
    });
    assert!(done_rx.try_recv().is_err());
    release_tx.send(()).unwrap();
    done_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    shutdown.join().unwrap();
    assert!(matches!(
        read.terminal().as_deref(),
        Some(Completion::Cancelled)
    ));
    assert_eq!(handle.active_tasks(), 0);
}
