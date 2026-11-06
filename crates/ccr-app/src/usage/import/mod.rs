//! Installed llmusage CLI protocol and bounded process lifecycle; no upstream Rust dependency.
pub mod cli;
pub mod error;
pub mod events;
pub use ccr_usage::{AppPaths, SourceKind};
pub use cli::{LlmusageCli, SyncCommandOptions, run_sync_collect, run_sync_stream};
pub use events::{JobEvent, SourceSyncStats, SyncSummaryEvent, is_optional_source_absent};
