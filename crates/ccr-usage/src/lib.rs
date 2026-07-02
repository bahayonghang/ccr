//! Read-only llmusage projections shared by CCR surfaces.
//!
//! This crate only discovers and reads the installed llmusage SQLite database.
//! It does not run the llmusage CLI, create/migrate the DB, parse raw provider
//! logs, or write CCR state.

mod capabilities;
mod db;
mod error;
mod paths;
mod queries;
mod source;

pub use capabilities::{
    FeatureKey, MIN_SUPPORTED_SCHEMA_VERSION, PROVIDER_BREAKDOWN_SCHEMA_VERSION,
};
pub use db::{Dashboard, QueryFilter, ReportTimezone, open_dashboard};
pub use error::UsageError;
pub use paths::{AppPaths, discover_llmusage_paths};
pub use queries::ProviderBreakdownDto;
pub use source::{SourceKind, canonical_source_id, parse_source_filter, platform_scope_label};
