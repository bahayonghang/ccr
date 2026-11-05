use super::*;
use ccr_app::runtime::{Platform, RuntimeOwner};
use ccr_cli::services::install_types::{
    DurationClass, InstallActionKind, InstallPlanView, LogStream, PackageManager,
    Platform as HostPlatform,
};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::time::Instant;

#[derive(Default)]
pub(crate) struct Controls {
    pub started: AtomicBool,
    pub cancelled: AtomicBool,
    pub complete: tokio::sync::Notify,
    pub cleanup: tokio::sync::Notify,
    pub starts: AtomicUsize,
    pub hold_after_terminal: AtomicBool,
    pub terminal_recorded: AtomicBool,
    pub after_terminal: tokio::sync::Notify,
}
pub(crate) fn plan_view(id: PlanId) -> InstallPlanView {
    InstallPlanView {
        plan_id: id,
        platform: HostPlatform::Linux,
        package_manager: PackageManager::Cargo,
        action: InstallActionKind::Cargo,
        expected_effects: vec!["Synthetic installation only".into()],
        elevation_required: false,
        duration_class: DurationClass::Fast,
        expires_at_ms: 4_000_000_000_000,
    }
}
pub(crate) fn fixture(owner: &RuntimeOwner) -> (Services, PlanId, Arc<Controls>) {
    let controls = Arc::new(Controls::default());
    let runner = controls.clone();
    let (service, id) = InstallService::test_with_runner(move |id, token, ring| {
        let controls = runner.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        let task = tokio::spawn(async move {
            controls.starts.fetch_add(1, Ordering::SeqCst);
            let start = InstallEvent::Started { attempt_id: id, plan: plan_view(PlanId::new()) };
            ring.record(&start);
            tx.send(start).await.unwrap();
            let line = InstallEvent::Log { attempt_id: id, stream: LogStream::Stdout, line: "fixture token=private-sentinel".into(), seq: 0 };
            ring.record(&line);
            tx.send(line).await.unwrap();
            controls.started.store(true, Ordering::SeqCst);
            let terminal = tokio::select! {
                _ = token.cancelled() => {
                    controls.cancelled.store(true, Ordering::SeqCst);
                    controls.cleanup.notified().await;
                    InstallEvent::Cancelled { attempt_id: id, requested_at_ms: 1 }
                }
                _ = controls.complete.notified() => InstallEvent::Succeeded { attempt_id: id, duration_ms: 1, installed_version: Some("synthetic".into()) },
            };
            ring.record(&terminal);
            tx.send(terminal).await.unwrap();
            controls.terminal_recorded.store(true, Ordering::SeqCst);
            if controls.hold_after_terminal.load(Ordering::SeqCst) {
                controls.after_terminal.notified().await;
            }
        });
        (rx, task)
    }).unwrap();
    let mut install = InstallOwner::with_service(service);
    install.synthetic = true;
    install.publish(|state| state.plan = Some(PlanOutcome::Plan(plan_view(id))));
    let mut services = Services::new(owner.handle(), super::super::ConfigSource::Local);
    services.install = Arc::new(install);
    (services, id, controls)
}
pub(crate) fn key(id: u64) -> RequestKey {
    RequestKey {
        environment: EnvironmentId::Local("synthetic".into()),
        platform: Platform::Claude,
        generation: 0,
        request_id: id,
    }
}
pub(crate) fn wait_until(predicate: impl Fn() -> bool) {
    let end = Instant::now() + Duration::from_secs(5);
    while !predicate() {
        assert!(Instant::now() < end, "fixture deadline");
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn fixture_and_remote_paths_fail_closed_before_host_detection() {
    let owner = RuntimeOwner::new().unwrap();
    let root = tempfile::tempdir().unwrap();
    let services = Services::new(
        owner.handle(),
        super::super::ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("config.toml"),
            delay: Duration::ZERO,
        },
    );
    let requests = InstallRequests::new(&services);
    let query = requests.query(key(1), Query::Check).unwrap();
    wait_until(|| query.terminal().is_some());
    assert!(matches!(
        query.terminal().as_deref(),
        Some(Completion::Failed(Error::Unsupported))
    ));
    assert!(matches!(
        requests.execute(key(2), PlanId::new(), true),
        Err(Error::Unsupported)
    ));
    let mut remote = key(3);
    remote.environment = EnvironmentId::Ssh("unavailable".into());
    assert!(matches!(
        requests.execute(remote, PlanId::new(), true),
        Err(Error::Unsupported)
    ));
    assert!(requests.snapshot().detection.is_none());
    assert_eq!(root.path().read_dir().unwrap().count(), 0);
    owner.shutdown();
}

#[test]
fn confirmation_canonical_plan_and_queued_cleanup_failures_are_retained() {
    let owner = RuntimeOwner::new().unwrap();
    let (services, id, controls) = fixture(&owner);
    let requests = InstallRequests::new(&services);
    assert!(matches!(
        requests.execute(key(1), id, false),
        Err(Error::Confirmation)
    ));
    assert_eq!(controls.starts.load(Ordering::SeqCst), 0);
    let failed = requests.execute(key(2), PlanId::new(), true).unwrap();
    wait_until(|| failed.terminal().is_some());
    assert!(matches!(
        failed.terminal().as_deref(),
        Some(Completion::Failed(Error::PlanUnavailable))
    ));
    assert_eq!(requests.snapshot().error, Some(Error::PlanUnavailable));
    assert!(!requests.snapshot().busy());
    assert_eq!(controls.starts.load(Ordering::SeqCst), 0);
    owner.shutdown();
}

#[test]
fn detach_revisit_cancellation_waits_for_cleanup_and_preserves_redacted_logs() {
    let owner = RuntimeOwner::new().unwrap();
    let (services, id, controls) = fixture(&owner);
    let requests = InstallRequests::new(&services);
    let subscription = requests.execute(key(1), id, true).unwrap();
    wait_until(|| controls.started.load(Ordering::SeqCst));
    assert!(matches!(
        requests.execute(key(2), id, true),
        Err(Error::Busy)
    ));
    drop(subscription);
    let observer = requests.observe();
    wait_until(|| !observer.borrow().recent.logs.is_empty());
    assert!(observer.borrow().running);
    assert!(!format!("{:?}", observer.borrow().recent.logs).contains("private-sentinel"));
    let _ = requests.cancel(key(3)).unwrap();
    wait_until(|| controls.cancelled.load(Ordering::SeqCst));
    assert!(requests.snapshot().cancel_requested);
    assert!(requests.snapshot().running);
    assert!(requests.snapshot().recent.terminal.is_none());
    assert!(matches!(
        requests.execute(key(4), id, true),
        Err(Error::Busy)
    ));
    controls.cleanup.notify_one();
    wait_until(|| !requests.snapshot().busy());
    assert!(matches!(
        requests.snapshot().recent.terminal,
        Some(InstallEvent::Cancelled { .. })
    ));
    assert_eq!(controls.starts.load(Ordering::SeqCst), 1);
    owner.shutdown();
}

#[test]
fn shutdown_cancels_detached_install_and_joins_cleanup() {
    let owner = RuntimeOwner::new().unwrap();
    let (services, id, controls) = fixture(&owner);
    let requests = InstallRequests::new(&services);
    let subscription = requests.execute(key(1), id, true).unwrap();
    wait_until(|| controls.started.load(Ordering::SeqCst));
    drop(subscription);
    let shutdown = std::thread::spawn(move || owner.shutdown());
    wait_until(|| controls.cancelled.load(Ordering::SeqCst));
    assert!(!shutdown.is_finished());
    assert!(requests.snapshot().running);
    controls.cleanup.notify_one();
    shutdown.join().unwrap();
    assert!(!requests.snapshot().busy());
    assert!(matches!(
        requests.snapshot().recent.terminal,
        Some(InstallEvent::Cancelled { .. })
    ));
}

#[test]
fn old_query_generation_cannot_replace_new_plan_state() {
    let owner = InstallOwner::default();
    let old = owner.query_generation.fetch_add(1, Ordering::SeqCst) + 1;
    let current = owner.query_generation.fetch_add(1, Ordering::SeqCst) + 1;
    owner
        .publish_query(current, |state| state.error = Some(Error::HostChanged))
        .unwrap();
    assert!(matches!(
        owner.publish_query(old, |state| state.error = None),
        Err(Error::Interrupted)
    ));
    assert_eq!(owner.snapshot().error, Some(Error::HostChanged));
}

#[test]
fn queued_cancellation_prevents_domain_execution_and_retains_failure_after_detach() {
    let owner = RuntimeOwner::new().unwrap();
    let barrier = Arc::new(std::sync::Barrier::new(
        ccr_app::runtime::BLOCKING_CONCURRENCY + 1,
    ));
    let release = Arc::new(std::sync::Barrier::new(
        ccr_app::runtime::BLOCKING_CONCURRENCY + 1,
    ));
    let mut blockers = Vec::new();
    for index in 0..ccr_app::runtime::BLOCKING_CONCURRENCY {
        let barrier = barrier.clone();
        let release = release.clone();
        blockers.push(
            owner
                .handle()
                .job(key(100 + index as u64), move |_, _| {
                    barrier.wait();
                    release.wait();
                    Completion::<(), Error>::Success(())
                })
                .unwrap(),
        );
    }
    barrier.wait();
    let (services, id, controls) = fixture(&owner);
    let requests = InstallRequests::new(&services);
    let subscription = requests.execute(key(1), id, true).unwrap();
    assert!(requests.snapshot().queued);
    requests.cancel(key(2)).unwrap();
    drop(subscription);
    release.wait();
    wait_until(|| !requests.snapshot().busy());
    assert_eq!(controls.starts.load(Ordering::SeqCst), 0);
    assert_eq!(requests.snapshot().error, Some(Error::Interrupted));
    owner.shutdown();
}

#[test]
fn provisional_terminal_is_hidden_until_executor_completion_releases_domain_slot() {
    let owner = RuntimeOwner::new().unwrap();
    let (services, id, controls) = fixture(&owner);
    controls.hold_after_terminal.store(true, Ordering::SeqCst);
    let requests = InstallRequests::new(&services);
    let subscription = requests.execute(key(1), id, true).unwrap();
    wait_until(|| controls.started.load(Ordering::SeqCst));
    controls.complete.notify_one();
    wait_until(|| controls.terminal_recorded.load(Ordering::SeqCst));
    assert!(services.install.service.recent_events().terminal.is_some());
    assert!(requests.snapshot().running);
    assert!(requests.snapshot().recent.terminal.is_none());
    assert!(subscription.terminal().is_none());
    assert!(matches!(
        requests.execute(key(2), id, true),
        Err(Error::Busy)
    ));
    controls.after_terminal.notify_one();
    wait_until(|| subscription.terminal().is_some());
    assert!(matches!(
        requests.snapshot().recent.terminal,
        Some(InstallEvent::Succeeded { .. })
    ));
    owner.shutdown();
}
