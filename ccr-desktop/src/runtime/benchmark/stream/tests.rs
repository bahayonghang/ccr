#![allow(clippy::unwrap_used)]
use super::*;
use crate::runtime::ConfigSource;
#[test]
fn ordinary_sources_reject_without_allocating_owner() {
    let runtime = ccr_app::runtime::RuntimeOwner::new().unwrap();
    for source in [
        ConfigSource::Local,
        ConfigSource::Fixture {
            registry: std::path::PathBuf::from("synthetic-registry"),
            profiles: std::path::PathBuf::from("synthetic-profiles"),
            delay: std::time::Duration::ZERO,
        },
    ] {
        let services = Services::new(runtime.handle(), source);
        assert!(services.benchmark_s2_status().is_err());
        assert!(services.benchmark_s2_cancel("foreign").is_err());
        assert!(services.benchmark_s2.get().is_none());
    }
}

#[test]
fn captured_native_run_cancels_and_joins_actual_persistence() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    std::fs::create_dir(&home).unwrap();
    std::fs::create_dir(root.path().join("ui")).unwrap();
    let pool =
        ccr_db::database::create_app_pool_at(&root.path().join("ui/ccr-ui.db"), &home).unwrap();
    let layout = Arc::new(Layout {
        root: root.path().into(),
        roots: ccr_cli::application::platform_roots::PlatformRoots::fixture(root.path()).unwrap(),
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
        main: Default::default(),
    });
    let runtime = ccr_app::runtime::RuntimeOwner::new().unwrap();
    let services = Services::new(runtime.handle(), ConfigSource::Benchmark(layout));
    let key = ccr_app::runtime::RequestKey {
        environment: ccr_app::runtime::EnvironmentId::Local("s2-test".into()),
        platform: ccr_app::runtime::Platform::Claude,
        generation: 1,
        request_id: 1,
    };
    let snapshot = services.benchmark_s2_start(key.clone()).unwrap();
    let id = snapshot.run_id.unwrap();
    for _ in 0..100 {
        if services.benchmark_s2_status().unwrap().ring_accepted >= 3 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(services.benchmark_s2_cancel("foreign").is_err());
    services.benchmark_s2_cancel(&id).unwrap();
    runtime.shutdown();
    let terminal = services.benchmark_s2_status().unwrap();
    assert!(terminal.phase.terminal());
    assert!(terminal.ring_accepted > 0);
    assert_eq!(terminal.db_failed, 0);
    assert_eq!(terminal.db_committed, terminal.ring_accepted);
    assert_eq!(terminal.db_pending, 0);
    assert_eq!(terminal.queue_pending, 0);
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
    assert!(services.benchmark_s2_start(key).is_err());
}

#[test]
fn smaller_benchmark_scenarios_reject_before_stream_owner_allocation() {
    let root = tempfile::tempdir().unwrap();
    let runtime = ccr_app::runtime::RuntimeOwner::new().unwrap();
    for scenario in ["S0", "S1"] {
        let layout = Arc::new(Layout {
            root: root.path().into(),
            roots: ccr_cli::application::platform_roots::PlatformRoots::fixture(root.path())
                .unwrap(),
            id: format!("benchmark-{}", scenario.to_ascii_lowercase()),
            scenario: scenario.into(),
            logical_sha256: "a".repeat(64),
            as_of: chrono::Utc::now(),
            timezone: ccr_usage::ReportTimezone::Utc,
            profile_count: 0,
            session_count: 0,
            usage_count: 0,
            main: Default::default(),
        });
        let services = Services::new(runtime.handle(), ConfigSource::Benchmark(layout));
        assert!(!services.has_benchmark_s2());
        assert!(services.benchmark_s2_status().is_err());
        assert!(services.benchmark_s2.get().is_none());
    }
    runtime.shutdown();
}

/// Captured synthetic source for headless controls; does not claim paired-data admission.
pub(crate) fn source(root: &std::path::Path) -> ConfigSource {
    ConfigSource::Benchmark(Arc::new(Layout {
        root: root.into(),
        roots: ccr_cli::application::platform_roots::PlatformRoots::fixture(root).unwrap(),
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
        main: Default::default(),
    }))
}
