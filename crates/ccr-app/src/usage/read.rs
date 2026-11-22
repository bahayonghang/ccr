//! Read-only usage report assembly shared by native and Tauri consumers.
use ccr_usage::{
    DailyTrendDto, Dashboard, HeatmapPoint, ModelBreakdown, OverviewPayload, ProjectBreakdown,
    ProviderBreakdownDto, QueryFilter, RecordedCost, SourceBreakdownDto, UsageError,
    UsagePeriodSummary,
};
use chrono::{DateTime, Days, NaiveDate, Utc};
use std::collections::BTreeMap;

/// Canonical values, without wire-format or presentation rounding.
#[derive(Debug)]
pub struct UsageSnapshot {
    pub overview: OverviewPayload,
    /// Available on the Usage page projection, which reads pricing coverage.
    pub recorded_cost: Option<RecordedCost>,
    /// The Usage page compares only explicit, adjacent calendar periods.
    pub comparison: Option<UsageComparison>,
    pub trends: Vec<DailyTrendDto>,
    /// Usage-page pricing coverage for each present day.
    pub daily_recorded_cost: BTreeMap<String, RecordedCost>,
    pub models: Vec<ModelBreakdown>,
    pub providers: Vec<ProviderBreakdownDto>,
    /// False is unsupported, not a measured empty provider distribution.
    pub providers_supported: bool,
    pub projects: Vec<ProjectBreakdown>,
    pub sources: Vec<SourceBreakdownDto>,
    pub heatmap: Option<Vec<HeatmapPoint>>,
}

#[derive(Debug)]
pub struct UsageComparison {
    pub includes_today: bool,
    pub previous: PreviousPeriod,
}

#[derive(Debug)]
pub enum PreviousPeriod {
    AllHistory,
    IncompleteBounds,
    Bounded {
        since: NaiveDate,
        until: NaiveDate,
        summary: Result<Option<UsagePeriodSummary>, UsageError>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PeriodChange {
    Percent(f64),
    ZeroBaseline,
    UnreliableCost,
}

impl PeriodChange {
    pub fn count(current: i64, previous: i64) -> Self {
        if previous == 0 {
            return Self::ZeroBaseline;
        }
        let difference = i128::from(current) - i128::from(previous);
        Self::Percent(difference as f64 * 100.0 / previous as f64)
    }

    pub fn cost(current: RecordedCost, previous: RecordedCost) -> Self {
        let (RecordedCost::Known(current), RecordedCost::Known(previous)) = (current, previous)
        else {
            return Self::UnreliableCost;
        };
        if !current.is_finite() || !previous.is_finite() {
            return Self::UnreliableCost;
        }
        if previous == 0.0 {
            return Self::ZeroBaseline;
        }
        let percent = (current - previous) / previous * 100.0;
        if percent.is_finite() {
            Self::Percent(percent)
        } else {
            Self::UnreliableCost
        }
    }
}

fn previous_period(dashboard: &Dashboard, filter: &QueryFilter) -> PreviousPeriod {
    let (start, end) = match (filter.since, filter.until) {
        (Some(start), Some(end)) => (start, end),
        (None, None) => return PreviousPeriod::AllHistory,
        _ => return PreviousPeriod::IncompleteBounds,
    };
    let calendar_days = Days::new((end - start).num_days() as u64 + 1);
    let (Some(since), Some(until), Some(_)) = (
        start.checked_sub_days(calendar_days),
        start.pred_opt(),
        end.succ_opt(),
    ) else {
        return PreviousPeriod::IncompleteBounds;
    };
    let previous_filter = QueryFilter {
        since: Some(since),
        until: Some(until),
        ..filter.clone()
    };
    PreviousPeriod::Bounded {
        since,
        until,
        summary: dashboard.usage_period_summary(&previous_filter),
    }
}

/// Assemble all sections from one read-only connection and one complete filter.
/// An absent provider dimension may degrade only an unfiltered report.
pub fn read_dashboard(
    dashboard: &Dashboard,
    filter: &QueryFilter,
    heatmap_days: Option<u32>,
) -> Result<UsageSnapshot, UsageError> {
    read_dashboard_at(dashboard, filter, heatmap_days, None)
}
pub fn read_dashboard_at(
    dashboard: &Dashboard,
    filter: &QueryFilter,
    heatmap_days: Option<u32>,
    as_of: Option<DateTime<Utc>>,
) -> Result<UsageSnapshot, UsageError> {
    read_snapshot(dashboard, filter, heatmap_days, as_of, false)
}

/// The Usage page follows one reporting clock, keeps source comparisons scoped,
/// and reads coverage for the same filters.
pub fn read_usage_page_at(
    dashboard: &Dashboard,
    filter: &QueryFilter,
    as_of: DateTime<Utc>,
) -> Result<UsageSnapshot, UsageError> {
    let filter = filter.clone().for_usage_page();
    dashboard.with_read_snapshot(|dashboard| {
        read_snapshot(dashboard, &filter, Some(366), Some(as_of), true)
    })
}

fn read_snapshot(
    dashboard: &Dashboard,
    filter: &QueryFilter,
    heatmap_days: Option<u32>,
    as_of: Option<DateTime<Utc>>,
    usage_page: bool,
) -> Result<UsageSnapshot, UsageError> {
    if filter
        .since
        .zip(filter.until)
        .is_some_and(|(start, end)| start > end)
    {
        return Err(UsageError::Query("invalid report date range".into()));
    }
    let overview = match as_of {
        Some(as_of) => dashboard.overview_at(filter, as_of)?,
        None => dashboard.overview(filter)?,
    };
    let mut daily_recorded_cost = BTreeMap::new();
    let trends = if usage_page {
        dashboard
            .usage_daily_trends(filter)?
            .into_iter()
            .map(|(day, cost)| {
                daily_recorded_cost.insert(day.date.clone(), cost);
                day
            })
            .collect()
    } else {
        dashboard.trends_daily(filter)?
    };
    let models = dashboard.model_breakdown(filter)?;
    let projects = dashboard.project_breakdown(filter)?;
    let (sources, recorded_cost) = if usage_page {
        (
            dashboard.scoped_source_breakdown(filter)?,
            Some(dashboard.recorded_cost(filter)?),
        )
    } else {
        (dashboard.source_breakdown(filter)?, None)
    };
    let (providers, providers_supported) = match dashboard.provider_breakdown(filter) {
        Ok(rows) => (rows, true),
        Err(UsageError::SchemaUnsupported { .. } | UsageError::FeatureUnavailable { .. })
            if filter.provider.is_none() =>
        {
            (Vec::new(), false)
        }
        Err(error) => return Err(error),
    };
    let heatmap = heatmap_days
        .map(|days| match as_of {
            Some(as_of) => dashboard.heatmap_at(filter, days.clamp(1, 366), as_of),
            None => dashboard.heatmap(filter, days.clamp(1, 366)),
        })
        .transpose()?;
    let comparison = as_of.filter(|_| usage_page).map(|as_of| {
        let today = dashboard.report_date(filter.timezone, as_of);
        UsageComparison {
            includes_today: filter.since.is_none_or(|start| start <= today)
                && filter.until.is_none_or(|end| today <= end),
            previous: previous_period(dashboard, filter),
        }
    });
    Ok(UsageSnapshot {
        overview,
        recorded_cost,
        comparison,
        trends,
        daily_recorded_cost,
        models,
        providers,
        providers_supported,
        projects,
        sources,
        heatmap,
    })
}

#[cfg(test)]
#[path = "read_tests.rs"]
mod tests;
