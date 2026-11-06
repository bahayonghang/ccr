//! Shared verified SQLite snapshot owner, extracted from the migration backup path.
use crate::core::error::DbError;
use rusqlite::{Connection, OpenFlags};
use std::path::Path;

/// Destination must be absent. Snapshot consistency includes committed WAL contents.
pub fn create_verified_backup(conn: &Connection, destination: &Path) -> Result<(), DbError> {
    if !destination.is_absolute() || destination.exists() {
        return Err(DbError::Io(
            "backup destination must be new and absolute".into(),
        ));
    }
    // Reserve an empty private file before SQLite writes any database content.
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(destination)?;
    drop(file);
    let result = (|| -> Result<(), DbError> {
        ccr_core::core::guarded_write::restrict_owner_only(destination)
            .map_err(|e| DbError::Io(e.to_string()))?;
        conn.execute(
            "VACUUM main INTO ?1",
            [destination.to_string_lossy().into_owned()],
        )?;
        let backup = Connection::open_with_flags(destination, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        let integrity: String = backup.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        if integrity != "ok" {
            return Err(DbError::Query("backup integrity check failed".into()));
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(destination);
    }
    result
}
