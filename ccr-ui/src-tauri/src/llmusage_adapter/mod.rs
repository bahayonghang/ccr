//! Crate-free llmusage integration boundary for CCR Desktop.
//!
//! ccr-ui consumes the installed `llmusage` CLI for sync/import and reads the
//! active llmusage SQLite database through read-only, version-gated SQL
//! projections. It must not link the upstream Rust crate, bootstrap/migrate the
//! DB, or parse raw provider logs.

pub mod capabilities;
pub mod cli;
pub mod db;
pub mod error;
pub mod events;
pub mod paths;
pub mod queries;
pub mod source;

pub use capabilities::{CapabilityReport, FeatureKey};
pub use cli::{LlmusageCli, SyncCommandOptions, run_sync_collect};
pub use db::{Dashboard, LogsQuery, QueryFilter, build_filter, open_dashboard};
pub use events::{JobEvent, SourceSyncStats, SyncSummaryEvent, is_optional_source_absent};
pub use paths::{AppPaths, discover_llmusage_paths};
pub use source::{SourceKind, canonical_source_id, parse_source_filter, platform_scope_label};

#[derive(Debug, Clone)]
pub struct LlmusageRuntime {
    paths: AppPaths,
    cli: LlmusageCli,
}

impl LlmusageRuntime {
    /// Builds a lightweight runtime handle. This never opens, creates,
    /// bootstraps, migrates, or writes the llmusage database.
    pub fn discover() -> Result<Self, String> {
        let paths = discover_llmusage_paths()?;
        Ok(Self {
            paths: paths.clone(),
            cli: LlmusageCli::new(paths),
        })
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub fn cli(&self) -> &LlmusageCli {
        &self.cli
    }

    pub fn dashboard(&self) -> Result<Dashboard, String> {
        open_dashboard(self.paths.clone()).map_err(|error| error.to_string())
    }

    pub fn capabilities(&self) -> CapabilityReport {
        CapabilityReport::detect(&self.paths)
    }
}
