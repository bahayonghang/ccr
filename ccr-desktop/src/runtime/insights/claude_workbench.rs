//! One captured Claude calendar/report context; SQL remains in ccr-usage.
use super::*;
use ccr_app::usage::observer::subscription::SubscriptionDto;
use chrono::Datelike;

pub struct Usage {
    pub recent: UsageSnapshot,
    pub today: ccr_usage::OverviewPayload,
    pub month: ccr_usage::OverviewPayload,
    pub history: ccr_usage::OverviewPayload,
    pub first: Option<String>,
    pub latest: Option<String>,
}
pub struct Report {
    pub clock: ReportClock,
    pub usage: Result<Usage, InsightsError>,
    pub subscription: Result<Option<SubscriptionDto>, observer::Error>,
}

fn usage(paths: &InsightsPaths, clock: ReportClock) -> Result<Usage, UsageError> {
    let dashboard = ccr_usage::open_dashboard(AppPaths::from_root(&paths.usage_root))?;
    let (since, until) = clock.window(30);
    let mut filter = QueryFilter {
        source: Some(SourceKind::Claude),
        since: Some(since),
        until: Some(until),
        timezone: clock.timezone,
        ..Default::default()
    };
    let recent = read_dashboard_at(&dashboard, &filter, None, Some(clock.as_of))?;
    filter.since = Some(until);
    let today = dashboard.overview_at(&filter, clock.as_of)?;
    filter.since = until.with_day(1);
    let month = dashboard.overview_at(&filter, clock.as_of)?;
    filter.since = None;
    let history = dashboard.overview_at(&filter, clock.as_of)?;
    let days = dashboard.trends_daily(&filter)?;
    let first = days.iter().map(|row| row.date.clone()).min();
    let latest = days.iter().map(|row| row.date.clone()).max();
    Ok(Usage {
        recent,
        today,
        month,
        history,
        first,
        latest,
    })
}

impl InsightsRequests {
    pub fn claude_workbench(
        &self,
        key: RequestKey,
        clock: ReportClock,
    ) -> Result<TaskSubscription<Arc<Report>, &'static str>, AdmissionError> {
        let requests = self.clone();
        self.services.executor.read(key.clone(), move |_, _| {
            if !matches!(key.environment, EnvironmentId::Local(_))
                || key.platform != ccr_app::runtime::Platform::Claude
            {
                return Err("Unsupported Claude workbench scope / 不支持此 Claude 工作台作用域");
            }
            Ok(Arc::new(Report {
                clock,
                usage: requests
                    .paths
                    .as_ref()
                    .ok_or(InsightsError::Unconfigured)
                    .and_then(|paths| usage(paths, clock).map_err(InsightsError::from)),
                subscription: observer::read_subscription(
                    &requests.services.observer,
                    &requests.services.source,
                ),
            }))
        })
    }
}

pub fn cache_ratio(input: i64, read: i64) -> Option<f64> {
    let denominator = input.checked_add(read)?;
    (denominator > 0).then(|| read as f64 / denominator as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[allow(clippy::unwrap_used)]
    fn captured_calendar_distinguishes_missing_today_zero_and_same_window_tokens() {
        use ccr_usage::fixtures::{SeedBucket, create_projection_db, seed_bucket};
        let root = tempfile::tempdir().unwrap();
        let paths = create_projection_db(root.path());
        let conn = rusqlite::Connection::open(paths.db_path).unwrap();
        seed_bucket(
            &conn,
            &SeedBucket {
                source: "claude".into(),
                hour_start: "2026-11-01T00:00:00Z".into(),
                input_tokens: 100,
                cache_read_tokens: 50,
                output_tokens: 30,
                reasoning_output_tokens: 15,
                total_tokens: 200,
                cost_with_cache_usd: 0.004,
                ..Default::default()
            },
        );
        seed_bucket(
            &conn,
            &SeedBucket {
                source: "codex".into(),
                hour_start: "2026-11-10T00:00:00Z".into(),
                cost_with_cache_usd: 999.,
                ..Default::default()
            },
        );
        drop(conn);
        let captured = InsightsPaths {
            usage_root: root.path().into(),
            archive_db: root.path().join("absent.db"),
            provider_home: root.path().join("home"),
        };
        let clock = ReportClock::fixed_utc(
            chrono::DateTime::parse_from_rfc3339("2026-11-10T12:00:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
        );
        let report = usage(&captured, clock).unwrap();
        assert_eq!(report.today.total_events, 0);
        assert_eq!(report.month.total_cost_usd, 0.004);
        assert_eq!(report.recent.overview.total.total_tokens, 200);
        assert_eq!(report.recent.trends[0].output_tokens, 45);
        assert_eq!(report.first.as_deref(), Some("2026-11-01"));
        let conn = rusqlite::Connection::open(root.path().join("llmusage.db")).unwrap();
        seed_bucket(
            &conn,
            &SeedBucket {
                source: "claude".into(),
                hour_start: "2026-11-10T00:00:00Z".into(),
                cost_with_cache_usd: 0.,
                ..Default::default()
            },
        );
        drop(conn);
        let report = usage(&captured, clock).unwrap();
        assert!(report.today.total_events > 0);
        assert_eq!(report.today.total_cost_usd, 0.);
    }
    #[test]
    fn cache_zero_denominator_is_unknown_and_real_zero_is_measured() {
        assert_eq!(cache_ratio(0, 0), None);
        assert_eq!(cache_ratio(100, 0), Some(0.));
        assert_eq!(cache_ratio(100, 100), Some(0.5));
    }
}
