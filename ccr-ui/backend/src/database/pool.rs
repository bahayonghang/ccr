//! 数据库连接池模块
//!
//! 使用 r2d2 + r2d2_sqlite 实现 SQLite 连接池，替代原有的 `Arc<Mutex<Connection>>`。
//! 连接池提供更好的并发性能，避免全局锁竞争。
//!
//! ## 优势
//! - **并发友好**: 多个线程可以同时获取不同的连接
//! - **连接复用**: 避免频繁创建/销毁连接的开销
//! - **自动管理**: 连接自动归还池中，无需手动管理生命周期

use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::Path;
use std::time::Duration;
use tracing::info;

/// 连接池类型别名
pub type DbPool = Pool<SqliteConnectionManager>;

/// 池化连接类型别名
/// NOTE: 当前为 Phase 1 基础设施，Phase 2 会在 Handler 中使用
#[allow(dead_code)]
pub type PooledConn = PooledConnection<SqliteConnectionManager>;

/// 连接池配置
/// NOTE: 当前为 Phase 1 基础设施，Phase 2 会在自定义配置时使用
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// 最大连接数（默认 10）
    pub max_size: u32,
    /// 最小空闲连接数（默认 2）
    pub min_idle: Option<u32>,
    /// 连接超时时间（默认 30 秒）
    pub connection_timeout: Duration,
    /// 空闲连接超时时间（默认 10 分钟）
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

/// 连接池错误
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum PoolError {
    #[error("Failed to create connection pool: {0}")]
    Creation(#[from] r2d2::Error),

    #[error("Failed to get connection from pool: {0}")]
    GetConnection(r2d2::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
}

/// 创建数据库连接池
///
/// # Arguments
/// * `db_path` - 数据库文件路径
/// * `config` - 连接池配置（可选，使用默认配置）
///
/// # Returns
/// 配置好的连接池实例
pub fn create_pool(db_path: &Path, config: Option<PoolConfig>) -> Result<DbPool, PoolError> {
    let config = config.unwrap_or_default();

    info!(
        "Creating database connection pool: path={}, max_size={}, min_idle={:?}",
        db_path.display(),
        config.max_size,
        config.min_idle
    );

    // 创建 SQLite 连接管理器
    let manager = SqliteConnectionManager::file(db_path);

    // 构建连接池
    let pool = Pool::builder()
        .max_size(config.max_size)
        .min_idle(config.min_idle)
        .connection_timeout(config.connection_timeout)
        .idle_timeout(config.idle_timeout)
        .connection_customizer(Box::new(SqliteCustomizer))
        .build(manager)?;

    info!("Database connection pool created successfully");
    Ok(pool)
}

/// 创建内存数据库连接池（用于测试）
#[cfg(test)]
pub fn create_memory_pool() -> Result<DbPool, PoolError> {
    let manager = SqliteConnectionManager::memory();

    let pool = Pool::builder()
        .max_size(1) // 内存数据库只能有一个连接
        .connection_customizer(Box::new(SqliteCustomizer))
        .build(manager)?;

    Ok(pool)
}

/// SQLite 连接自定义器
///
/// 在每个新连接创建时配置 SQLite 参数
#[derive(Debug)]
struct SqliteCustomizer;

impl r2d2::CustomizeConnection<Connection, rusqlite::Error> for SqliteCustomizer {
    fn on_acquire(&self, conn: &mut Connection) -> Result<(), rusqlite::Error> {
        // 配置 SQLite 性能参数
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;
             PRAGMA cache_size = -2000;", // 2MB cache
        )?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_pool() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = create_pool(&db_path, None).unwrap();

        // 验证可以获取连接
        let conn = pool.get().unwrap();
        let result: i32 = conn.query_row("SELECT 1", [], |row| row.get(0)).unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_pool_concurrent_access() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = create_pool(
            &db_path,
            Some(PoolConfig {
                max_size: 5,
                ..Default::default()
            }),
        )
        .unwrap();

        // 创建测试表
        {
            let conn = pool.get().unwrap();
            conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY)", [])
                .unwrap();
        }

        // 并发访问
        let handles: Vec<_> = (0..5)
            .map(|i| {
                let pool = pool.clone();
                std::thread::spawn(move || {
                    let conn = pool.get().unwrap();
                    conn.execute("INSERT INTO test (id) VALUES (?)", [i])
                        .unwrap();
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // 验证结果
        let conn = pool.get().unwrap();
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_memory_pool() {
        let pool = create_memory_pool().unwrap();
        let conn = pool.get().unwrap();
        let result: i32 = conn.query_row("SELECT 1", [], |row| row.get(0)).unwrap();
        assert_eq!(result, 1);
    }
}
