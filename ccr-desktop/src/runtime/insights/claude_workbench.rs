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

    // 以下夹具都用固定 UTC 时钟 2026-11-10T12:00Z，与本机时区和当前时间无关。
    fn fixed_clock() -> ReportClock {
        ReportClock::fixed_utc(
            chrono::DateTime::parse_from_rfc3339("2026-11-10T12:00:00Z")
                .unwrap_or_else(|e| panic!("clock: {e}"))
                .with_timezone(&chrono::Utc),
        )
    }
    fn captured(root: &std::path::Path) -> InsightsPaths {
        InsightsPaths {
            usage_root: root.into(),
            archive_db: root.join("absent.db"),
            provider_home: root.join("home"),
        }
    }

    #[test]
    fn missing_and_unreadable_databases_fail_instead_of_reading_as_zero() {
        // 无库：读取失败，且不为了“读出 0”去创建数据库文件。
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        assert!(usage(&captured(root.path()), fixed_clock()).is_err());
        assert!(!root.path().join("llmusage.db").exists());
        // 不可读库：文件存在但不是 SQLite，同样是失败而不是空报告。
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        std::fs::write(root.path().join("llmusage.db"), b"not a sqlite database")
            .unwrap_or_else(|e| panic!("write: {e}"));
        assert!(usage(&captured(root.path()), fixed_clock()).is_err());
        assert_eq!(
            std::fs::read(root.path().join("llmusage.db")).unwrap_or_default(),
            b"not a sqlite database"
        );
    }

    #[test]
    fn empty_database_keeps_start_latest_and_sync_unknown_and_sync_time_is_global() {
        use ccr_usage::fixtures::{create_projection_db, seed_run_log};
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let db = create_projection_db(root.path());
        let report =
            usage(&captured(root.path()), fixed_clock()).unwrap_or_else(|_| panic!("empty report"));
        // 空库是“已读取、无记录”：起点、最新记录日与同步时间都未知，不补出日期或 0 行。
        assert_eq!(report.first, None);
        assert_eq!(report.latest, None);
        assert_eq!(report.history.last_sync_at, None);
        assert_eq!(report.today.bucket_count, 0);
        assert_eq!(report.month.bucket_count, 0);
        assert_eq!(report.history.bucket_count, 0);
        assert!(report.recent.trends.is_empty());
        // 只有 Codex 记过同步：Claude 报告仍拿到同一个全局时间，它不是 Claude 专属的新鲜度。
        let conn = rusqlite::Connection::open(db.db_path).unwrap_or_else(|e| panic!("db: {e}"));
        seed_run_log(&conn, "codex", "recent", "2026-10-01T00:00:00Z");
        drop(conn);
        let report = usage(&captured(root.path()), fixed_clock())
            .unwrap_or_else(|_| panic!("synced report"));
        assert_eq!(
            report.history.last_sync_at.as_deref(),
            Some("2026-10-01T00:00:00Z")
        );
        assert_eq!(report.first, None, "a sync run is not a usage record");
    }

    #[test]
    fn tokens_and_cache_come_only_from_the_30_day_window() {
        use ccr_usage::fixtures::{SeedBucket, create_projection_db, seed_bucket};
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let db = create_projection_db(root.path());
        let conn = rusqlite::Connection::open(db.db_path).unwrap_or_else(|e| panic!("db: {e}"));
        // 窗口外（45 天前）有大量缓存；窗口内只有输出（含推理），输入与缓存读取都为 0。
        for seed in [
            SeedBucket {
                source: "claude".into(),
                hour_start: "2026-09-26T12:00:00Z".into(),
                input_tokens: 100,
                cache_read_tokens: 500,
                cache_creation_tokens: 7,
                output_tokens: 10,
                reasoning_output_tokens: 0,
                total_tokens: 617,
                cost_with_cache_usd: 8.,
                ..Default::default()
            },
            SeedBucket {
                source: "claude".into(),
                hour_start: "2026-11-05T12:00:00Z".into(),
                input_tokens: 0,
                cache_read_tokens: 0,
                cache_creation_tokens: 0,
                output_tokens: 30,
                reasoning_output_tokens: 15,
                total_tokens: 45,
                cost_with_cache_usd: 0.5,
                ..Default::default()
            },
        ] {
            seed_bucket(&conn, &seed);
        }
        drop(conn);
        let report =
            usage(&captured(root.path()), fixed_clock()).unwrap_or_else(|_| panic!("report"));
        let recent = &report.recent.overview.total;
        assert_eq!((recent.input_tokens, recent.cache_read_tokens), (0, 0));
        // 30 天分母为零：不可计算；不能借全历史的缓存凑出命中率。
        assert_eq!(
            cache_ratio(recent.input_tokens, recent.cache_read_tokens),
            None
        );
        assert_eq!(report.history.total.cache_read_tokens, 500);
        assert!(
            cache_ratio(
                report.history.total.input_tokens,
                report.history.total.cache_read_tokens
            )
            .is_some()
        );
        // 输出取兼容字段（已含推理），不再加一次 reasoning；总量用 owner 的 total。
        let output: i64 = report.recent.trends.iter().map(|r| r.output_tokens).sum();
        assert_eq!(output, 45);
        assert_eq!(recent.total_tokens, 45);
        let creation: i64 = report
            .recent
            .trends
            .iter()
            .map(|r| r.cache_creation_tokens)
            .sum();
        assert_eq!(creation, 0);
        assert_eq!(report.first.as_deref(), Some("2026-09-26"));
        assert_eq!(report.latest.as_deref(), Some("2026-11-05"));
        assert!((report.history.total_cost_usd - 8.5).abs() < 1e-9);
    }
}
