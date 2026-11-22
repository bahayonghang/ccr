#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use ccr_usage::{
    AppPaths, ReportTimezone, SourceKind,
    fixtures::{SeedBucket, create_projection_db, seed_bucket},
};
use chrono::NaiveDate;
use rusqlite::Connection;

#[test]
fn report_preserves_provider_splits_pricing_and_complete_filter_without_writes() {
    let temp = tempfile::tempdir().unwrap();
    let paths = create_projection_db(temp.path());
    let conn = Connection::open(&paths.db_path).unwrap();
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
    // Close the sole fixture writer before taking the no-write baseline.
    drop(conn);
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
        ..Default::default()
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
    assert!(
        before == std::fs::read(&paths.db_path).unwrap(),
        "read-only report changed the fixture database"
    );
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
    let conn = Connection::open(&paths.db_path).unwrap();
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

#[test]
fn usage_page_merges_aliases_within_the_selected_scope_and_keeps_unknown_sources() {
    let temp = tempfile::tempdir().unwrap();
    let paths = create_projection_db(temp.path());
    let conn = Connection::open(&paths.db_path).unwrap();
    for (source, date, project, count) in [
        ("gemini", "2026-07-01T00:00:00Z", "p1", 2),
        ("antigravity", "2026-07-01T00:30:00Z", "p1", 3),
        ("antigravity_ide", "2026-07-03T00:00:00Z", "p1", 5),
        ("antigravity", "2026-07-01T01:00:00Z", "p2", 13),
        ("legacy Project Source", "2026-07-01T00:00:00Z", "p1", 7),
    ] {
        seed_bucket(
            &conn,
            &SeedBucket {
                source: source.into(),
                hour_start: date.into(),
                project_hash: project.into(),
                event_count: count,
                ..Default::default()
            },
        );
    }
    drop(conn);
    let dashboard = ccr_usage::open_dashboard(paths).unwrap();
    let day = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let at = chrono::DateTime::parse_from_rfc3339("2026-07-04T00:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let filter = QueryFilter {
        source: Some(SourceKind::Antigravity),
        since: Some(day),
        until: Some(day),
        project_hash: Some("p1".into()),
        timezone: ReportTimezone::Utc,
        ..Default::default()
    };
    let report = read_usage_page_at(&dashboard, &filter, at).unwrap();
    assert_eq!(report.overview.total_events, 5);
    assert_eq!(report.trends[0].request_count, 5);
    assert_eq!(report.models[0].event_count, 5);
    assert_eq!(report.providers[0].request_count, 5);
    assert_eq!(report.projects[0].event_count, 5);
    assert_eq!(report.sources.len(), 1);
    assert_eq!(report.sources[0].event_count, 5);
    assert_eq!(report.sources[0].active_days, 1);
    assert_eq!(report.sources[0].share_tokens, 1.0);
    let heatmap = report.heatmap.unwrap();
    assert_eq!(heatmap.len(), 366);
    assert_eq!(heatmap[364].date, "2026-07-03");
    assert_eq!(heatmap[364].event_count, 5);

    let all = read_usage_page_at(
        &dashboard,
        &QueryFilter {
            source: None,
            ..filter.clone()
        },
        at,
    )
    .unwrap();
    assert_eq!(all.sources.len(), 2);
    assert_eq!(all.sources[0].source, "antigravity");
    assert_eq!(all.sources[1].source, "legacy Project Source");
    assert!((all.sources[0].share_tokens - 2.0 / 3.0).abs() < f64::EPSILON);
    let legacy = dashboard.source_breakdown(&filter).unwrap();
    assert!(
        legacy
            .iter()
            .any(|row| row.source == "legacy Project Source")
    );
    assert_eq!(dashboard.overview(&filter).unwrap().total_events, 3);
}
