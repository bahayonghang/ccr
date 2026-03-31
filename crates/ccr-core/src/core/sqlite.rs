use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::Path;
use std::time::Duration;

pub type DbPool = Pool<SqliteConnectionManager>;
pub type DbConnection = PooledConnection<SqliteConnectionManager>;

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_size: u32,
    pub min_idle: Option<u32>,
    pub connection_timeout: Duration,
    pub idle_timeout: Option<Duration>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 10,
            min_idle: Some(2),
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Some(Duration::from_secs(600)),
        }
    }
}

pub fn create_sqlite_pool(path: &Path, config: Option<PoolConfig>) -> Result<DbPool, r2d2::Error> {
    let config = config.unwrap_or_default();
    let manager = SqliteConnectionManager::file(path);

    Pool::builder()
        .max_size(config.max_size)
        .min_idle(config.min_idle)
        .connection_timeout(config.connection_timeout)
        .idle_timeout(config.idle_timeout)
        .connection_customizer(Box::new(SqliteCustomizer))
        .build(manager)
}

pub fn create_memory_sqlite_pool() -> Result<DbPool, r2d2::Error> {
    let manager = SqliteConnectionManager::memory();

    Pool::builder()
        .max_size(1)
        .connection_customizer(Box::new(SqliteCustomizer))
        .build(manager)
}

#[derive(Debug)]
struct SqliteCustomizer;

impl r2d2::CustomizeConnection<Connection, rusqlite::Error> for SqliteCustomizer {
    fn on_acquire(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;
             PRAGMA cache_size = -2000;",
        )?;
        Ok(())
    }
}
