#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use ccr_db::database::create_pool;
use ccr_usage::{
    AppPaths, ReportTimezone, SourceKind,
    fixtures::{SeedBucket, create_projection_db, seed_bucket},
};
use chrono::NaiveDate;

#[test]
fn report_preserves_provider_splits_pricing_and_complete_filter_without_writes() {
    let temp = tempfile::tempdir().unwrap();
    let paths = create_projection_db(temp.path());
    let pool = create_pool(&paths.db_path, None).unwrap();
    let conn = pool.get().unwrap();
    for (source, provider, date) in [
        ("codex", "openai", "2026-07-01T23:59:59Z"),
        ("codex", "", "2026-07-01T12:00:00Z"),
        ("claude", "openai", "2026-07-01T12:00:00Z"),
        ("codex", "openai", "2026-07-02T00:00:00Z"),
    ] {
        seed_bucket(
            &conn,
            &SeedBucket {
                source: source.into(),
                provider_label: provider.into(),
                hour_start: date.into(),
                cost_with_cache_usd: 0.123456789,
                cost_without_cache_usd: 0.234567891,
                pricing_status: "static".into(),
                pricing_source: Some("static-v1".into()),
                pricing_rate: Some("10/1/50".into()),
                ..Default::default()
            },
        );
    }
    drop(conn);
    drop(pool);
    let before = std::fs::read(&paths.db_path).unwrap();
    let dashboard = ccr_usage::open_dashboard(paths.clone()).unwrap();
    let day = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let filter = QueryFilter {
        source: Some(SourceKind::Codex),
        provider: Some("openai".into()),
        model: Some("gpt-5".into()),
        project_hash: Some("p1".into()),
        since: Some(day),
        until: Some(day),
        timezone: ReportTimezone::Utc,
    };
    let report = read_dashboard(&dashboard, &filter, None).unwrap();
    assert_eq!(report.overview.total.total_tokens, 100);
    assert_eq!(report.overview.total_events, 2);
    assert_eq!(report.trends, dashboard.trends_daily(&filter).unwrap());
    assert_eq!(report.models, dashboard.model_breakdown(&filter).unwrap());
    assert_eq!(
        report.providers,
        dashboard.provider_breakdown(&filter).unwrap()
    );
    assert_eq!(
        report.projects,
        dashboard.project_breakdown(&filter).unwrap()
    );
    assert_eq!(report.sources, dashboard.source_breakdown(&filter).unwrap());
    assert_eq!(report.providers[0].reasoning_output_tokens, 15);
    assert_eq!(report.providers[0].cache_creation_tokens, 5);
    assert_eq!(report.providers[0].cost_with_cache_usd, 0.123456789);
    assert_eq!(
        report.models[0].pricing_source.as_deref(),
        Some("static-v1")
    );
    assert_eq!(report.models[0].pricing_rate.as_deref(), Some("10/1/50"));
    assert!(report.providers_supported);
    assert!(report.heatmap.is_none());
    let all = read_dashboard(
        &dashboard,
        &QueryFilter {
            timezone: ReportTimezone::Utc,
            ..Default::default()
        },
        None,
    )
    .unwrap();
    assert!(
        all.providers
            .iter()
            .any(|row| row.provider.is_none() && row.display_provider() == "unattributed")
    );
    drop(dashboard);
    assert_eq!(before, std::fs::read(&paths.db_path).unwrap());
}
#[test]
fn empty_is_measured_zero_but_missing_is_error_and_reversed_bounds_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let missing = AppPaths::from_root(temp.path().join("missing"));
    assert!(matches!(
        ccr_usage::open_dashboard(missing),
        Err(UsageError::DbMissing(_))
    ));
    assert!(!temp.path().join("missing").exists());
    let paths = create_projection_db(temp.path());
    let dashboard = ccr_usage::open_dashboard(paths).unwrap();
    let report = read_dashboard(&dashboard, &QueryFilter::default(), None).unwrap();
    assert_eq!(report.overview.total.total_tokens, 0);
    assert!(report.providers_supported && report.providers.is_empty());
    let filter = QueryFilter {
        since: NaiveDate::from_ymd_opt(2026, 7, 2),
        until: NaiveDate::from_ymd_opt(2026, 7, 1),
        ..Default::default()
    };
    assert!(matches!(
        read_dashboard(&dashboard, &filter, None),
        Err(UsageError::Query(_))
    ));
}

#[test]
fn captured_clock_controls_last24h_and_heatmap_without_changing_explicit_filter() {
    let temp = tempfile::tempdir().unwrap();
    let paths = create_projection_db(temp.path());
    let pool = create_pool(&paths.db_path, None).unwrap();
    let conn = pool.get().unwrap();
    for hour in [
        "1999-12-31T23:59:59Z",
        "2000-01-01T00:00:00Z",
        "2000-01-01T23:59:59Z",
        "2000-01-02T00:00:00Z",
    ] {
        seed_bucket(
            &conn,
            &SeedBucket {
                hour_start: hour.into(),
                ..Default::default()
            },
        );
    }
    drop(conn);
    drop(pool);
    let dashboard = ccr_usage::open_dashboard(paths).unwrap();
    let at = chrono::DateTime::parse_from_rfc3339("2000-01-01T23:59:59Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let filter = QueryFilter {
        timezone: ReportTimezone::Utc,
        ..Default::default()
    };
    let report = read_dashboard_at(&dashboard, &filter, Some(1), Some(at)).unwrap();
    assert_eq!(report.overview.generated_at, at.to_rfc3339());
    assert_eq!(report.overview.last_24h.total_tokens, 300);
    assert_eq!(report.heatmap.unwrap().len(), 1);
    let day = NaiveDate::from_ymd_opt(1999, 12, 31).unwrap();
    let explicit = QueryFilter {
        since: Some(day),
        until: Some(day),
        timezone: ReportTimezone::Utc,
        ..Default::default()
    };
    let report = read_dashboard_at(&dashboard, &explicit, None, Some(at)).unwrap();
    assert_eq!(report.overview.total.total_tokens, 100);
    assert_eq!(explicit.since, Some(day));
    assert_eq!(explicit.until, Some(day));
}
