//! Read-only usage report assembly shared by native and Tauri consumers.
use ccr_usage::{
    DailyTrendDto, Dashboard, HeatmapPoint, ModelBreakdown, OverviewPayload, ProjectBreakdown,
    ProviderBreakdownDto, QueryFilter, SourceBreakdownDto, UsageError,
};

/// Canonical values, without wire-format or presentation rounding.
#[derive(Debug)]
pub struct UsageSnapshot {
    pub overview: OverviewPayload,
    pub trends: Vec<DailyTrendDto>,
    pub models: Vec<ModelBreakdown>,
    pub providers: Vec<ProviderBreakdownDto>,
    /// False is unsupported, not a measured empty provider distribution.
    pub providers_supported: bool,
    pub projects: Vec<ProjectBreakdown>,
    pub sources: Vec<SourceBreakdownDto>,
    pub heatmap: Option<Vec<HeatmapPoint>>,
}

/// Assemble all sections from one read-only connection and one complete filter.
/// An absent provider dimension may degrade only an unfiltered report.
pub fn read_dashboard(
    dashboard: &Dashboard,
    filter: &QueryFilter,
    heatmap_days: Option<u32>,
) -> Result<UsageSnapshot, UsageError> {
    if filter
        .since
        .zip(filter.until)
        .is_some_and(|(start, end)| start > end)
    {
        return Err(UsageError::Query("invalid report date range".into()));
    }
    let overview = dashboard.overview(filter)?;
    let trends = dashboard.trends_daily(filter)?;
    let models = dashboard.model_breakdown(filter)?;
    let projects = dashboard.project_breakdown(filter)?;
    let sources = dashboard.source_breakdown(filter)?;
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
        .map(|days| dashboard.heatmap(filter, days.clamp(1, 366)))
        .transpose()?;
    Ok(UsageSnapshot {
        overview,
        trends,
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
