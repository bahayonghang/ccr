//! Crate-free llmusage integration boundary for CCR Desktop.
//!
//! ccr-ui consumes the installed `llmusage` CLI for sync/import and reads the
//! active llmusage SQLite database through read-only, version-gated SQL
//! projections. It must not link the upstream Rust crate, bootstrap/migrate the
//! DB, or parse raw provider logs.
//!
//! The read-only projection itself (paths, source vocabulary, capability
//! gates, SQL) lives in the shared `ccr_usage` crate; this module only keeps
//! CLI sync execution, NDJSON events, and the Tauri-facing DTO/error mapping.

pub mod capabilities;
pub mod cli;
pub mod db;
pub mod error;
pub mod events;
pub mod queries;

pub use capabilities::CapabilityReport;
pub use ccr_usage::{
    AppPaths, SourceKind, canonical_source_id, discover_llmusage_paths, parse_source_filter,
    platform_scope_label,
};
pub use cli::{LlmusageCli, SyncCommandOptions, run_sync_collect};
pub use db::{Dashboard, LogsQuery, QueryFilter, build_filter, open_dashboard};
pub use events::{JobEvent, SourceSyncStats, SyncSummaryEvent, is_optional_source_absent};

#[derive(Debug, Clone)]
pub struct LlmusageRuntime {
    paths: AppPaths,
    cli: LlmusageCli,
}

impl LlmusageRuntime {
    /// Builds a lightweight runtime handle. This never opens, creates,
    /// bootstraps, migrates, or writes the llmusage database.
    pub fn discover() -> Result<Self, String> {
        Ok(Self::from_paths(discover_llmusage_paths()?))
    }

    /// Builds a runtime handle for an explicit path set (tests, remote roots).
    pub fn from_paths(paths: AppPaths) -> Self {
        Self {
            cli: LlmusageCli::new(paths.clone()),
            paths,
        }
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
