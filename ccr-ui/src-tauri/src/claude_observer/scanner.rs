//! Compatibility entry; Tauri retains default root discovery.
pub use ccr_app::usage::observer::scanner::IngestReport;
pub fn ingest_incremental(conn: &mut rusqlite::Connection) -> IngestReport {
    let Some(home) = dirs::home_dir() else {
        return IngestReport::default();
    };
    ccr_app::usage::observer::scanner::ingest_incremental_at(
        conn,
        &home.join(".claude/projects"),
        || false,
    )
    .unwrap_or_default()
}
