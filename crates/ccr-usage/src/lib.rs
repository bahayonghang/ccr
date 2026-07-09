//! Read-only llmusage projections shared by CCR surfaces.
//!
//! This crate only discovers and reads the installed llmusage SQLite database.
//! It does not run the llmusage CLI, create/migrate the DB, parse raw provider
//! logs, or write CCR state.

mod capabilities;
mod db;
mod error;
#[cfg(feature = "test-fixtures")]
pub mod fixtures;
mod paths;
mod queries;
mod source;

pub use capabilities::{
    DB_BACKED_FEATURES, DbCapabilities, FeatureCapability, FeatureKey,
    MIN_SUPPORTED_SCHEMA_VERSION, PROVIDER_BREAKDOWN_SCHEMA_VERSION, UnsupportedReason,
    required_columns,
};
pub use db::{
    Dashboard, DiagnosticsPayload, LogsPage, LogsQuery, QueryFilter, ReportTimezone,
    SourceDiagnostics, build_filter, open_dashboard,
};
pub use error::UsageError;
pub use paths::{AppPaths, discover_llmusage_paths};
pub use queries::{
    DailyTrendDto, HeatmapPoint, HomeOverviewPayload, HomeOverviewPlatformStats,
    HomeOverviewSeriesItem, HomeOverviewSummary, ModelBreakdown, OverviewPayload, ProjectBreakdown,
    ProviderBreakdownDto, SourceBreakdownDto, TaggedProviderBreakdown, TokenSummary,
    UsageRecordDto, generated_at,
};
pub use source::{SourceKind, canonical_source_id, parse_source_filter, platform_scope_label};
