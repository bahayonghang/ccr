//! Home Insights projection: one bounded read of `usage_bucket_30m`, with every
//! time dimension (local day, hour, weekday, month, week) derived in Rust.
//!
//! SQL only groups rows coarsely (see `Dashboard::insights`). The pure
//! aggregation in this module takes an explicit `as_of` date and zone, so
//! callers and tests never depend on the system clock.

use std::collections::BTreeMap;

use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike, Utc};
use serde::{Deserialize, Serialize};

use crate::{UsageError, source::SourceKind, timezone::ResolvedZone};

/// Number of weekly buckets in the trend and activity windows: 52 full weeks
/// plus the week that contains `as_of`.
pub const INSIGHTS_WEEKS: usize = 53;

/// One local day with its request count.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/usage/")
)]
pub struct InsightsDay {
    /// Local calendar date, `YYYY-MM-DD`.
    pub date: String,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub requests: i64,
}

/// Weekly request counts for one source across the 53-week window. The last
/// item is the week that contains `as_of`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/usage/")
)]
pub struct InsightsTrendSeries {
    /// Canonical llmusage source id.
    pub source: String,
    #[cfg_attr(feature = "ts", ts(as = "Vec<f64>"))]
    pub weekly: Vec<i64>,
}

/// Ranking row for models and projects.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/usage/")
)]
pub struct InsightsTally {
    /// Grouping key: the model id, or the project hash.
    pub key: String,
    /// Display label: the model id, or the llmusage project label.
    pub label: String,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub requests: i64,
    #[cfg_attr(feature = "ts", ts(as = "f64"))]
    pub tokens: i64,
}

/// All-history usage totals for one canonical source.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InsightsSourceTally {
    pub source: String,
    pub requests: i64,
    pub tokens: i64,
}

/// Request activity inside one closed local-date window. `start_utc` and
/// `end_utc` are the half-open UTC bounds of that window in the report zone,
/// so callers can count other records (for example sessions) with the same
/// boundaries.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InsightsUsageWindow {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub start_utc: DateTime<Utc>,
    pub end_utc: DateTime<Utc>,
    pub requests: i64,
    pub active_days: i64,
}

/// Usage-only Insights snapshot. Session counts are merged by the caller.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct InsightsPayload {
    pub as_of: NaiveDate,
    pub first_date: Option<NaiveDate>,
    pub requests: i64,
    pub tokens: i64,
    /// Sources with any requests or tokens.
    pub agents: i64,
    /// Distinct non-empty project hashes; equals `projects.len()`.
    pub project_count: i64,
    /// Local days with at least one request.
    pub active_days: i64,
    pub last_7_days: InsightsUsageWindow,
    pub previous_7_days: InsightsUsageWindow,
    /// Days with requests inside the 53-week window, ascending.
    pub daily: Vec<InsightsDay>,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub busiest_day: Option<InsightsDay>,
    /// Requests per local hour (24 items).
    pub hourly: Vec<i64>,
    /// Requests per weekday, Monday first (7 items).
    pub weekday: Vec<i64>,
    /// Requests per calendar month across years, January first (12 items).
    pub monthly: Vec<i64>,
    /// Monday of the first week in the 53-week window.
    pub trend_start: NaiveDate,
    /// Sources with requests in the window, ordered by window total descending.
    pub trend: Vec<InsightsTrendSeries>,
    pub sources: Vec<InsightsSourceTally>,
    pub projects: Vec<InsightsTally>,
    pub models: Vec<InsightsTally>,
}

/// One coarse SQL row: a 30-minute bucket for one canonical source.
/// `instant` is `None` when the stored timestamp cannot be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TimeRow {
    pub(crate) instant: Option<DateTime<Utc>>,
    pub(crate) source: String,
    pub(crate) requests: i64,
    pub(crate) tokens: i64,
}

/// Time-derived part of the snapshot (everything except model/project ranks).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InsightsTimeAggregate {
    pub(crate) requests: i64,
    pub(crate) tokens: i64,
    pub(crate) first_date: Option<NaiveDate>,
    /// Requests per local day up to `as_of`; only days with requests > 0.
    pub(crate) daily: BTreeMap<NaiveDate, i64>,
    pub(crate) hourly: [i64; 24],
    pub(crate) weekday: [i64; 7],
    pub(crate) monthly: [i64; 12],
    pub(crate) trend: BTreeMap<String, [i64; INSIGHTS_WEEKS]>,
    pub(crate) sources: BTreeMap<String, (i64, i64)>,
}

/// Maps a stored llmusage source value to its canonical id. Unknown values
/// stay unchanged so they remain visible as raw labels.
pub(crate) fn canonical_source_key(raw: &str) -> String {
    SourceKind::parse_id(raw)
        .map(|source| source.as_str().to_string())
        .unwrap_or_else(|| raw.to_string())
}

/// Monday of the week that contains `day`.
pub(crate) fn week_start(day: NaiveDate) -> NaiveDate {
    day - Duration::days(i64::from(day.weekday().num_days_from_monday()))
}

/// First Monday of the 53-week window that ends with the week of `as_of`.
pub(crate) fn trend_start(as_of: NaiveDate) -> NaiveDate {
    week_start(as_of) - Duration::weeks((INSIGHTS_WEEKS - 1) as i64)
}

/// Weekly bucket index for `day`: `INSIGHTS_WEEKS - 1` is the week of
/// `as_of`, `0` is the oldest week. Days outside the window return `None`.
pub(crate) fn week_index(day: NaiveDate, as_of: NaiveDate) -> Option<usize> {
    if day > as_of {
        return None;
    }
    let weeks_back = (week_start(as_of) - week_start(day)).num_days() / 7;
    let weeks_back = usize::try_from(weeks_back).ok()?;
    (weeks_back < INSIGHTS_WEEKS).then(|| INSIGHTS_WEEKS - 1 - weeks_back)
}

/// Aggregates coarse bucket rows into every time dimension of the snapshot.
///
/// Rows whose local date is after `as_of` (clock drift) and rows without a
/// parseable timestamp count toward `requests`/`tokens` and source totals only.
pub(crate) fn aggregate_time_rows(
    rows: impl IntoIterator<Item = TimeRow>,
    zone: &ResolvedZone,
    as_of: NaiveDate,
) -> InsightsTimeAggregate {
    let mut aggregate = InsightsTimeAggregate {
        requests: 0,
        tokens: 0,
        first_date: None,
        daily: BTreeMap::new(),
        hourly: [0; 24],
        weekday: [0; 7],
        monthly: [0; 12],
        trend: BTreeMap::new(),
        sources: BTreeMap::new(),
    };
    for row in rows {
        aggregate.requests += row.requests;
        aggregate.tokens += row.tokens;
        let source_total = aggregate.sources.entry(row.source.clone()).or_default();
        source_total.0 += row.requests;
        source_total.1 += row.tokens;

        let Some(instant) = row.instant else {
            continue;
        };
        let local = zone.local_at(instant);
        let day = local.date();
        // 晚于 as_of 的桶是时钟漂移脏数据：只计入总量，不进任何分桶。
        if day > as_of {
            continue;
        }
        if row.requests > 0 || row.tokens > 0 {
            aggregate.first_date = Some(aggregate.first_date.map_or(day, |first| first.min(day)));
        }
        if row.requests <= 0 {
            continue;
        }
        *aggregate.daily.entry(day).or_default() += row.requests;
        // 30 分钟桶在 +05:45 等非整点偏移时区会跨本地整点，小时分布只用于展示，接受该误差。
        aggregate.hourly[local.hour() as usize] += row.requests;
        aggregate.weekday[day.weekday().num_days_from_monday() as usize] += row.requests;
        aggregate.monthly[day.month0() as usize] += row.requests;
        if let Some(index) = week_index(day, as_of) {
            aggregate
                .trend
                .entry(row.source)
                .or_insert([0; INSIGHTS_WEEKS])[index] += row.requests;
        }
    }
    aggregate
}

/// Consecutive active days ending at `as_of`, or at the day before when
/// `as_of` itself has no requests.
pub(crate) fn current_streak(daily: &BTreeMap<NaiveDate, i64>, as_of: NaiveDate) -> u32 {
    let mut day = if daily.contains_key(&as_of) {
        as_of
    } else {
        match as_of.pred_opt() {
            Some(previous) => previous,
            None => return 0,
        }
    };
    let mut streak = 0;
    while daily.contains_key(&day) {
        streak += 1;
        match day.pred_opt() {
            Some(previous) => day = previous,
            None => break,
        }
    }
    streak
}

/// Longest run of consecutive active days in the full history.
pub(crate) fn longest_streak(daily: &BTreeMap<NaiveDate, i64>) -> u32 {
    let mut longest = 0;
    let mut run = 0;
    let mut previous: Option<NaiveDate> = None;
    for day in daily.keys() {
        run = match previous.and_then(|value| value.succ_opt()) {
            Some(next) if next == *day => run + 1,
            _ => 1,
        };
        longest = longest.max(run);
        previous = Some(*day);
    }
    longest
}

/// Day with the most requests; ties resolve to the earlier day.
pub(crate) fn busiest_day(daily: &BTreeMap<NaiveDate, i64>) -> Option<(NaiveDate, i64)> {
    let mut best: Option<(NaiveDate, i64)> = None;
    for (day, requests) in daily {
        if best.is_none_or(|(_, top)| *requests > top) {
            best = Some((*day, *requests));
        }
    }
    best
}

/// Request activity in the closed local-date window `[start, end]`.
pub(crate) fn usage_window(
    daily: &BTreeMap<NaiveDate, i64>,
    zone: &ResolvedZone,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<InsightsUsageWindow, UsageError> {
    let Some(end_exclusive) = end.succ_opt() else {
        return Err(UsageError::Query(format!(
            "invalid local date boundary: {end}"
        )));
    };
    let mut requests = 0;
    let mut active_days = 0;
    for (_, value) in daily.range(start..=end) {
        requests += value;
        active_days += 1;
    }
    Ok(InsightsUsageWindow {
        start,
        end,
        start_utc: zone.local_date_start_utc(start)?,
        end_utc: zone.local_date_start_utc(end_exclusive)?,
        requests,
        active_days,
    })
}

/// Builds the full snapshot from the time aggregate plus the SQL rankings.
pub(crate) fn build_payload(
    aggregate: InsightsTimeAggregate,
    zone: &ResolvedZone,
    as_of: NaiveDate,
    projects: Vec<InsightsTally>,
    models: Vec<InsightsTally>,
) -> Result<InsightsPayload, UsageError> {
    let start = trend_start(as_of);
    let last_7_days = usage_window(&aggregate.daily, zone, as_of - Duration::days(6), as_of)?;
    let previous_7_days = usage_window(
        &aggregate.daily,
        zone,
        as_of - Duration::days(13),
        as_of - Duration::days(7),
    )?;
    let daily = aggregate
        .daily
        .range(start..=as_of)
        .map(|(day, requests)| InsightsDay {
            date: format_date(*day),
            requests: *requests,
        })
        .collect();
    let busiest = busiest_day(&aggregate.daily).map(|(day, requests)| InsightsDay {
        date: format_date(day),
        requests,
    });

    let mut trend = aggregate
        .trend
        .into_iter()
        .map(|(source, weekly)| InsightsTrendSeries {
            source,
            weekly: weekly.to_vec(),
        })
        .collect::<Vec<_>>();
    // 按窗口内总量降序；同量按来源键升序，保证输出确定。
    trend.sort_by(|left, right| {
        let left_total: i64 = left.weekly.iter().sum();
        let right_total: i64 = right.weekly.iter().sum();
        right_total
            .cmp(&left_total)
            .then_with(|| left.source.cmp(&right.source))
    });

    let mut sources = aggregate
        .sources
        .into_iter()
        .filter(|(_, (requests, tokens))| *requests > 0 || *tokens > 0)
        .map(|(source, (requests, tokens))| InsightsSourceTally {
            source,
            requests,
            tokens,
        })
        .collect::<Vec<_>>();
    sources.sort_by(|left, right| {
        right
            .requests
            .cmp(&left.requests)
            .then_with(|| left.source.cmp(&right.source))
    });

    Ok(InsightsPayload {
        as_of,
        first_date: aggregate.first_date,
        requests: aggregate.requests,
        tokens: aggregate.tokens,
        agents: sources.len() as i64,
        project_count: projects.len() as i64,
        active_days: aggregate.daily.len() as i64,
        last_7_days,
        previous_7_days,
        daily,
        current_streak: current_streak(&aggregate.daily, as_of),
        longest_streak: longest_streak(&aggregate.daily),
        busiest_day: busiest,
        hourly: aggregate.hourly.to_vec(),
        weekday: aggregate.weekday.to_vec(),
        monthly: aggregate.monthly.to_vec(),
        trend_start: start,
        trend,
        sources,
        projects,
        models,
    })
}

pub(crate) fn format_date(day: NaiveDate) -> String {
    day.format("%Y-%m-%d").to_string()
}
#[cfg(test)]
mod tests {
    use chrono::{FixedOffset, TimeZone};

    use super::*;

    fn day(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).expect("fixture date should be valid")
    }

    fn instant(raw: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(raw)
            .expect("fixture timestamp should parse")
            .with_timezone(&Utc)
    }

    fn row(raw: &str, source: &str, requests: i64, tokens: i64) -> TimeRow {
        TimeRow {
            instant: Some(instant(raw)),
            source: source.to_string(),
            requests,
            tokens,
        }
    }

    fn daily(days: &[(NaiveDate, i64)]) -> BTreeMap<NaiveDate, i64> {
        days.iter().copied().collect()
    }

    fn payload(rows: Vec<TimeRow>, zone: &ResolvedZone, as_of: NaiveDate) -> InsightsPayload {
        build_payload(
            aggregate_time_rows(rows, zone, as_of),
            zone,
            as_of,
            Vec::new(),
            Vec::new(),
        )
        .expect("payload should build")
    }

    fn kolkata() -> ResolvedZone {
        ResolvedZone::Fixed(
            FixedOffset::east_opt(5 * 3600 + 30 * 60).expect("+05:30 should be valid"),
        )
    }

    // 2026-09-24 是星期四，所在周的周一是 2026-09-21。
    fn as_of() -> NaiveDate {
        day(2026, 9, 24)
    }

    #[test]
    fn insights_empty_rows_yield_zero_snapshot() {
        let snapshot = payload(Vec::new(), &ResolvedZone::utc(), as_of());

        assert_eq!(snapshot.requests, 0);
        assert_eq!(snapshot.tokens, 0);
        assert_eq!(snapshot.agents, 0);
        assert_eq!(snapshot.project_count, 0);
        assert_eq!(snapshot.active_days, 0);
        assert_eq!(snapshot.first_date, None);
        assert_eq!(snapshot.last_7_days.requests, 0);
        assert_eq!(snapshot.previous_7_days.active_days, 0);
        assert!(snapshot.daily.is_empty());
        assert_eq!(snapshot.current_streak, 0);
        assert_eq!(snapshot.longest_streak, 0);
        assert_eq!(snapshot.busiest_day, None);
        assert_eq!(snapshot.hourly, vec![0; 24]);
        assert_eq!(snapshot.weekday, vec![0; 7]);
        assert_eq!(snapshot.monthly, vec![0; 12]);
        assert_eq!(snapshot.trend_start, day(2025, 9, 22));
        assert!(snapshot.trend.is_empty());
        assert!(snapshot.sources.is_empty());
    }

    #[test]
    fn insights_fixed_offset_zones_shift_local_day_and_hour() {
        let rows = || vec![row("2026-09-23T20:00:00Z", "codex", 3, 30)];

        let utc = payload(rows(), &ResolvedZone::utc(), as_of());
        assert_eq!(utc.daily[0].date, "2026-09-23");
        assert_eq!(utc.hourly[20], 3);
        assert_eq!(utc.weekday[2], 3);

        let shifted = payload(rows(), &kolkata(), as_of());
        assert_eq!(shifted.daily[0].date, "2026-09-24");
        assert_eq!(shifted.hourly[1], 3);
        assert_eq!(shifted.weekday[3], 3);
        assert_eq!(shifted.monthly[8], 3);
        assert_eq!(shifted.first_date, Some(as_of()));
        assert_eq!(shifted.current_streak, 1);

        let minus_five =
            ResolvedZone::Fixed(FixedOffset::west_opt(5 * 3600).expect("-05:00 should be valid"));
        let west = payload(rows(), &minus_five, as_of());
        assert_eq!(west.daily[0].date, "2026-09-23");
        assert_eq!(west.hourly[15], 3);
    }

    #[test]
    fn insights_window_bounds_follow_the_report_zone() {
        let snapshot = payload(Vec::new(), &kolkata(), as_of());

        assert_eq!(snapshot.last_7_days.start, day(2026, 9, 18));
        assert_eq!(snapshot.last_7_days.end, as_of());
        assert_eq!(
            snapshot.last_7_days.start_utc,
            instant("2026-09-17T18:30:00Z")
        );
        assert_eq!(
            snapshot.last_7_days.end_utc,
            instant("2026-09-24T18:30:00Z")
        );
        assert_eq!(snapshot.previous_7_days.start, day(2026, 9, 11));
        assert_eq!(snapshot.previous_7_days.end, day(2026, 9, 17));
        assert_eq!(
            snapshot.previous_7_days.end_utc,
            snapshot.last_7_days.start_utc
        );
    }

    #[test]
    fn insights_weeks_start_on_monday_and_index_53_buckets() {
        let as_of = as_of();
        assert_eq!(week_start(as_of), day(2026, 9, 21));
        assert_eq!(week_start(day(2026, 9, 21)), day(2026, 9, 21));
        assert_eq!(week_start(day(2026, 9, 20)), day(2026, 9, 14));

        assert_eq!(week_index(as_of, as_of), Some(52));
        assert_eq!(week_index(day(2026, 9, 21), as_of), Some(52));
        assert_eq!(week_index(day(2026, 9, 20), as_of), Some(51));
        assert_eq!(trend_start(as_of), day(2025, 9, 22));
        assert_eq!(week_index(day(2025, 9, 22), as_of), Some(0));
        assert_eq!(week_index(day(2025, 9, 28), as_of), Some(0));
        assert_eq!(week_index(day(2025, 9, 21), as_of), None);
        assert_eq!(week_index(day(2026, 9, 25), as_of), None);
    }

    #[test]
    fn insights_trend_and_daily_keep_only_the_53_week_window() {
        let snapshot = payload(
            vec![
                row("2026-09-24T09:00:00Z", "codex", 2, 20),
                row("2025-09-22T09:00:00Z", "codex", 5, 50),
                row("2025-09-21T09:00:00Z", "codex", 7, 70),
            ],
            &ResolvedZone::utc(),
            as_of(),
        );

        assert_eq!(snapshot.trend.len(), 1);
        let weekly = &snapshot.trend[0].weekly;
        assert_eq!(weekly.len(), INSIGHTS_WEEKS);
        assert_eq!(weekly[0], 5);
        assert_eq!(weekly[52], 2);
        assert_eq!(weekly.iter().sum::<i64>(), 7);
        assert_eq!(
            snapshot
                .daily
                .iter()
                .map(|item| item.date.as_str())
                .collect::<Vec<_>>(),
            vec!["2025-09-22", "2026-09-24"]
        );
        // 全历史口径：窗外日期仍计入总量、活跃天数与分布。
        assert_eq!(snapshot.requests, 14);
        assert_eq!(snapshot.active_days, 3);
        assert_eq!(snapshot.first_date, Some(day(2025, 9, 21)));
        assert_eq!(snapshot.hourly[9], 14);
    }

    #[test]
    fn insights_current_streak_counts_from_previous_day_when_as_of_is_idle() {
        let as_of = as_of();
        let idle_today = daily(&[
            (day(2026, 9, 21), 1),
            (day(2026, 9, 22), 1),
            (day(2026, 9, 23), 1),
        ]);
        assert_eq!(current_streak(&idle_today, as_of), 3);

        let active_today = daily(&[(day(2026, 9, 23), 1), (day(2026, 9, 24), 1)]);
        assert_eq!(current_streak(&active_today, as_of), 2);

        let broken = daily(&[(day(2026, 9, 21), 1), (day(2026, 9, 22), 1)]);
        assert_eq!(current_streak(&broken, as_of), 0);
    }

    #[test]
    fn insights_longest_streak_scans_full_history() {
        let history = daily(&[
            (day(2025, 1, 1), 1),
            (day(2025, 1, 2), 1),
            (day(2025, 1, 3), 1),
            (day(2025, 1, 4), 1),
            (day(2026, 9, 23), 1),
            (day(2026, 9, 24), 1),
        ]);
        assert_eq!(longest_streak(&history), 4);
        assert_eq!(longest_streak(&BTreeMap::new()), 0);
    }

    #[test]
    fn insights_busiest_day_ties_resolve_to_the_earlier_day() {
        let history = daily(&[
            (day(2026, 9, 1), 4),
            (day(2026, 9, 2), 9),
            (day(2026, 9, 3), 9),
        ]);
        assert_eq!(busiest_day(&history), Some((day(2026, 9, 2), 9)));
    }

    #[test]
    fn insights_future_rows_count_in_totals_but_no_bucket() {
        let snapshot = payload(
            vec![
                row("2026-09-24T09:00:00Z", "codex", 2, 20),
                row("2026-09-26T09:00:00Z", "codex", 5, 50),
            ],
            &ResolvedZone::utc(),
            as_of(),
        );

        assert_eq!(snapshot.requests, 7);
        assert_eq!(snapshot.tokens, 70);
        assert_eq!(snapshot.sources[0].requests, 7);
        assert_eq!(snapshot.active_days, 1);
        assert_eq!(snapshot.daily.len(), 1);
        assert_eq!(snapshot.hourly.iter().sum::<i64>(), 2);
        assert_eq!(snapshot.weekday.iter().sum::<i64>(), 2);
        assert_eq!(snapshot.monthly.iter().sum::<i64>(), 2);
        assert_eq!(snapshot.trend[0].weekly.iter().sum::<i64>(), 2);
        assert_eq!(snapshot.last_7_days.requests, 2);
        assert_eq!(snapshot.busiest_day.map(|item| item.requests), Some(2));
    }

    #[test]
    fn insights_seven_day_windows_split_at_as_of_minus_seven() {
        let snapshot = payload(
            vec![
                row("2026-09-18T09:00:00Z", "codex", 1, 0),
                row("2026-09-24T09:00:00Z", "codex", 2, 0),
                row("2026-09-17T09:00:00Z", "claude", 4, 0),
                row("2026-09-11T09:00:00Z", "claude", 8, 0),
                row("2026-09-10T09:00:00Z", "claude", 16, 0),
            ],
            &ResolvedZone::utc(),
            as_of(),
        );

        assert_eq!(snapshot.last_7_days.requests, 3);
        assert_eq!(snapshot.last_7_days.active_days, 2);
        assert_eq!(snapshot.previous_7_days.requests, 12);
        assert_eq!(snapshot.previous_7_days.active_days, 2);
    }

    #[test]
    fn insights_trend_orders_sources_by_window_total() {
        let snapshot = payload(
            vec![
                row("2026-09-24T09:00:00Z", "codex", 2, 0),
                row("2026-09-24T09:00:00Z", "claude", 9, 0),
                row("2026-09-24T09:00:00Z", "pi", 2, 0),
            ],
            &ResolvedZone::utc(),
            as_of(),
        );

        assert_eq!(
            snapshot
                .trend
                .iter()
                .map(|series| series.source.as_str())
                .collect::<Vec<_>>(),
            vec!["claude", "codex", "pi"]
        );
        assert_eq!(snapshot.agents, 3);
    }

    #[test]
    fn insights_iana_zone_uses_historical_offsets_for_hours() {
        let zone = ResolvedZone::Iana(chrono_tz::America::New_York);
        let winter = zone.local_at(
            Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0)
                .single()
                .expect("winter instant should exist"),
        );
        let summer = zone.local_at(
            Utc.with_ymd_and_hms(2026, 7, 15, 12, 0, 0)
                .single()
                .expect("summer instant should exist"),
        );
        assert_eq!(winter.hour(), 7);
        assert_eq!(summer.hour(), 8);
    }

    #[cfg(feature = "test-fixtures")]
    mod dashboard {
        use rusqlite::Connection;

        use super::{as_of, day};
        use crate::{
            Dashboard, QueryFilter, ReportTimezone, SourceKind,
            fixtures::{SeedBucket, create_projection_db, seed_bucket},
        };

        fn utc_filter() -> QueryFilter {
            QueryFilter {
                timezone: ReportTimezone::Utc,
                ..QueryFilter::default()
            }
        }

        fn open(temp: &tempfile::TempDir, seeds: &[SeedBucket]) -> Dashboard {
            let paths = create_projection_db(temp.path());
            let conn = Connection::open(&paths.db_path).expect("fixture db should reopen");
            for seed in seeds {
                seed_bucket(&conn, seed);
            }
            drop(conn);
            Dashboard::open(paths).expect("dashboard should open fixture db")
        }

        #[test]
        fn insights_dashboard_emits_a_trend_series_for_every_source() {
            let temp = tempfile::TempDir::new().expect("temp dir should be created");
            let seeds = SourceKind::ALL
                .iter()
                .enumerate()
                .map(|(index, source)| SeedBucket {
                    source: source.as_str().to_string(),
                    hour_start: format!("2026-09-{:02}T09:00:00Z", 10 + index),
                    ..SeedBucket::default()
                })
                .collect::<Vec<_>>();
            let dashboard = open(&temp, &seeds);

            let snapshot = dashboard
                .insights(&utc_filter(), as_of())
                .expect("insights should query");

            let mut sources = snapshot
                .trend
                .iter()
                .map(|series| series.source.clone())
                .collect::<Vec<_>>();
            sources.sort();
            let mut expected = SourceKind::ALL
                .iter()
                .map(|source| source.as_str().to_string())
                .collect::<Vec<_>>();
            expected.sort();
            assert_eq!(sources, expected);
            assert_eq!(snapshot.agents, 9);
            assert_eq!(snapshot.requests, 18);
            assert_eq!(snapshot.first_date, Some(day(2026, 9, 10)));
        }

        #[test]
        fn insights_dashboard_merges_gemini_into_antigravity() {
            let temp = tempfile::TempDir::new().expect("temp dir should be created");
            let dashboard = open(
                &temp,
                &[
                    SeedBucket {
                        source: "gemini".to_string(),
                        ..SeedBucket::default()
                    },
                    SeedBucket {
                        source: "antigravity".to_string(),
                        ..SeedBucket::default()
                    },
                ],
            );

            let snapshot = dashboard
                .insights(&utc_filter(), as_of())
                .expect("insights should query");

            assert_eq!(snapshot.sources.len(), 1);
            assert_eq!(snapshot.sources[0].source, "antigravity");
            assert_eq!(snapshot.sources[0].requests, 4);
            assert!(
                snapshot
                    .trend
                    .iter()
                    .all(|series| series.source != "gemini")
            );
        }

        #[test]
        fn insights_dashboard_canonicalizes_stored_aliases_and_keeps_unknown_labels() {
            let temp = tempfile::TempDir::new().expect("temp dir should be created");
            let dashboard = open(
                &temp,
                &[
                    SeedBucket {
                        source: "omp".to_string(),
                        ..SeedBucket::default()
                    },
                    SeedBucket {
                        source: "pi".to_string(),
                        ..SeedBucket::default()
                    },
                    SeedBucket {
                        source: "antigravity_ide".to_string(),
                        ..SeedBucket::default()
                    },
                    SeedBucket {
                        source: "some_unknown_tool".to_string(),
                        ..SeedBucket::default()
                    },
                ],
            );

            let snapshot = dashboard
                .insights(&utc_filter(), as_of())
                .expect("insights should query");

            let sources = snapshot
                .sources
                .iter()
                .map(|item| (item.source.as_str(), item.requests))
                .collect::<Vec<_>>();
            assert_eq!(
                sources,
                vec![("pi", 4), ("antigravity", 2), ("some_unknown_tool", 2)]
            );
            assert!(
                snapshot
                    .sources
                    .iter()
                    .all(|item| item.source != "antigravity_ide")
            );
            assert_eq!(snapshot.agents, 3);
        }

        #[test]
        fn insights_dashboard_project_total_matches_ranking_and_skips_empty_keys() {
            let temp = tempfile::TempDir::new().expect("temp dir should be created");
            let dashboard = open(
                &temp,
                &[
                    SeedBucket::default(),
                    SeedBucket {
                        project_hash: "p2".to_string(),
                        project_label: String::new(),
                        model: "claude-sonnet".to_string(),
                        ..SeedBucket::default()
                    },
                    SeedBucket {
                        project_hash: String::new(),
                        project_label: String::new(),
                        model: String::new(),
                        event_count: 5,
                        ..SeedBucket::default()
                    },
                ],
            );

            let snapshot = dashboard
                .insights(&utc_filter(), as_of())
                .expect("insights should query");

            assert_eq!(snapshot.project_count, 2);
            assert_eq!(snapshot.projects.len(), 2);
            let p2 = snapshot
                .projects
                .iter()
                .find(|item| item.key == "p2")
                .expect("p2 row should exist");
            assert_eq!(p2.label, "p2");
            assert_eq!(
                snapshot
                    .models
                    .iter()
                    .map(|item| item.key.as_str())
                    .collect::<Vec<_>>(),
                vec!["claude-sonnet", "gpt-5"]
            );
            // 空项目、空模型的桶仍计入总量。
            assert_eq!(snapshot.requests, 9);
        }
    }
}
