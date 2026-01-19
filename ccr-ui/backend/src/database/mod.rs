// Unified SQLite database module for CCR UI backend
// Provides centralized data persistence replacing multiple JSON files
//
// Architecture:
// - Single SQLite file at ~/.ccr-ui/ccr-ui.db
// - Connection pool with thread-safe access
// - Automatic schema migration on initialization
//
// See: openspec/changes/add-unified-sqlite-storage/proposal.md

pub mod migrations;
pub mod repositories;
pub mod schema;

use once_cell::sync::OnceCell;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing::info;

use migrations::MigrationError;
use schema::DB_RELATIVE_PATH;

/// Global database connection holder
/// Uses Arc<Mutex<Connection>> for thread-safe access
static DB_CONNECTION: OnceCell<Arc<Mutex<Connection>>> = OnceCell::new();

/// Database initialization error
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Failed to create database directory: {0}")]
    DirectoryCreation(std::io::Error),

    #[error("Failed to open database: {0}")]
    ConnectionOpen(rusqlite::Error),

    #[error("Migration failed: {0}")]
    Migration(#[from] MigrationError),

    #[error("Database not initialized")]
    NotInitialized,

    #[error("Lock poisoned")]
    LockPoisoned,

    #[error("Query error: {0}")]
    Query(#[from] rusqlite::Error),
}

/// Get the database file path
/// Returns: ~/.ccr-ui/ccr-ui.db
pub fn get_db_path() -> PathBuf {
    dirs::home_dir()
        .expect("Failed to get home directory")
        .join(DB_RELATIVE_PATH)
}

/// Initialize the database connection and run migrations
/// This should be called once during application startup
pub fn initialize() -> Result<(), DatabaseError> {
    let db_path = get_db_path();

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(DatabaseError::DirectoryCreation)?;
    }

    info!("Initializing database at: {}", db_path.display());

    // Open connection with WAL mode for better concurrency
    let conn = Connection::open(&db_path).map_err(DatabaseError::ConnectionOpen)?;

    // Configure SQLite for performance
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;",
    )
    .map_err(DatabaseError::ConnectionOpen)?;

    // Run migrations
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    migrations::run_all_migrations(&conn, &home_dir)?;

    // Store connection globally
    let result = DB_CONNECTION.set(Arc::new(Mutex::new(conn)));
    if result.is_err() {
        // Already initialized - this is fine
        info!("Database already initialized");
    }

    info!("Database initialization complete");
    Ok(())
}

/// Get a reference to the database connection
/// Returns an Arc<Mutex<Connection>> for thread-safe access
///
/// # Panics
/// Panics if database was not initialized
#[allow(dead_code)]
pub fn get_connection() -> Arc<Mutex<Connection>> {
    DB_CONNECTION
        .get()
        .cloned()
        .expect("Database not initialized - call initialize() first")
}

/// Try to get a reference to the database connection
/// Returns None if not initialized
pub fn try_get_connection() -> Option<Arc<Mutex<Connection>>> {
    DB_CONNECTION.get().cloned()
}

/// Execute a function with the database connection
/// Handles locking and error conversion
pub fn with_connection<F, T>(f: F) -> Result<T, DatabaseError>
where
    F: FnOnce(&Connection) -> Result<T, rusqlite::Error>,
{
    let conn = try_get_connection().ok_or(DatabaseError::NotInitialized)?;
    let guard = conn.lock().map_err(|_| DatabaseError::LockPoisoned)?;
    f(&guard).map_err(DatabaseError::Query)
}

/// Execute a function with mutable access to the database connection
/// Use this for transactions or batch operations
#[allow(dead_code)]
pub fn with_connection_mut<F, T>(f: F) -> Result<T, DatabaseError>
where
    F: FnOnce(&mut Connection) -> Result<T, rusqlite::Error>,
{
    let conn = try_get_connection().ok_or(DatabaseError::NotInitialized)?;
    let mut guard = conn.lock().map_err(|_| DatabaseError::LockPoisoned)?;
    f(&mut guard).map_err(DatabaseError::Query)
}

/// Execute a transaction with automatic commit/rollback
#[allow(dead_code)]
pub fn transaction<F, T>(f: F) -> Result<T, DatabaseError>
where
    F: FnOnce(&rusqlite::Transaction<'_>) -> Result<T, rusqlite::Error>,
{
    with_connection_mut(|conn| {
        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    })
}

/// Shutdown the database (for testing or graceful shutdown)
/// Note: This consumes the global connection
#[allow(dead_code)]
pub fn shutdown() {
    if let Some(conn) = DB_CONNECTION.get()
        && let Ok(guard) = conn.lock()
    {
        // Execute checkpoint to flush WAL
        let _ = guard.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
        info!("Database shutdown complete");
    }
}

/// Check if database is initialized
pub fn is_initialized() -> bool {
    DB_CONNECTION.get().is_some()
}

/// Initialize an in-memory database for testing
/// This allows tests to use the global database connection
#[cfg(test)]
pub fn initialize_for_test() -> Result<(), DatabaseError> {
    // If already initialized, return OK
    if is_initialized() {
        return Ok(());
    }

    // Open in-memory connection
    let conn = Connection::open_in_memory().map_err(DatabaseError::ConnectionOpen)?;

    // Configure SQLite for performance
    conn.execute_batch(
        "PRAGMA journal_mode = MEMORY;
         PRAGMA synchronous = OFF;
         PRAGMA foreign_keys = ON;",
    )
    .map_err(DatabaseError::ConnectionOpen)?;

    // Run schema creation
    conn.execute_batch(schema::CREATE_TABLES_SQL)
        .map_err(DatabaseError::ConnectionOpen)?;

    // Store connection globally
    let result = DB_CONNECTION.set(Arc::new(Mutex::new(conn)));
    if result.is_err() {
        // Already initialized - this is fine
        return Ok(());
    }

    Ok(())
}

/// Reset test database by clearing all tables
/// Call this at the start of each test to ensure isolation
#[cfg(test)]
pub fn reset_for_test() -> Result<(), DatabaseError> {
    with_connection(|conn| {
        // Clear all data tables (keep migrations table)
        conn.execute_batch(
            "DELETE FROM checkin_records;
             DELETE FROM checkin_balances;
             DELETE FROM checkin_waf_cookies;
             DELETE FROM checkin_accounts;
             DELETE FROM checkin_providers;
             DELETE FROM ui_favorites;
             DELETE FROM ui_history;
             DELETE FROM log_entries;
             DELETE FROM usage_records;
             DELETE FROM usage_sources;",
        )?;
        Ok(())
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_db_path() {
        let path = get_db_path();
        assert!(path.ends_with(".ccr-ui/ccr-ui.db"));
    }

    #[test]
    fn test_in_memory_operations() {
        // Test with in-memory database
        let conn = Connection::open_in_memory().unwrap();

        // Run schema creation
        conn.execute_batch(schema::CREATE_TABLES_SQL).unwrap();

        // Test insert
        conn.execute(
            "INSERT INTO ui_favorites (id, command, args_json, module, created_at)
             VALUES ('test-id', 'test-cmd', '[]', 'test', '2024-01-01T00:00:00Z')",
            [],
        )
        .unwrap();

        // Test query
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM ui_favorites", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
