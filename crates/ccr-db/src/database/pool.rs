//! 数据库连接池模块
//!
//! 使用 r2d2 + r2d2_sqlite 实现 SQLite 连接池，替代原有的 `Arc<Mutex<Connection>>`。
//! 连接池提供更好的并发性能，避免全局锁竞争。
//!
//! ## 优势
//! - **并发友好**: 多个线程可以同时获取不同的连接
//! - **连接复用**: 避免频繁创建/销毁连接的开销
//! - **自动管理**: 连接自动归还池中，无需手动管理生命周期

use std::path::Path;
use tracing::info;

use crate::core::error::DbError;
use ccr_core::core::sqlite::{
    DbConnection as CoreDbConnection, DbPool as CoreDbPool, PoolConfig as CorePoolConfig,
    create_sqlite_pool,
};

/// 连接池类型别名
pub type DbPool = CoreDbPool;

/// 池化连接类型别名
/// NOTE: 当前为 Phase 1 基础设施，Phase 2 会在 Handler 中使用
#[allow(dead_code)]
pub type PooledConn = CoreDbConnection;

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
    pub connection_timeout: std::time::Duration,
    /// 空闲连接超时时间（默认 10 分钟）
    pub idle_timeout: Option<std::time::Duration>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 10,
            min_idle: Some(2),
            connection_timeout: std::time::Duration::from_secs(30),
            idle_timeout: Some(std::time::Duration::from_secs(600)),
        }
    }
}

/// 创建数据库连接池
///
/// # Arguments
/// * `db_path` - 数据库文件路径
/// * `config` - 连接池配置（可选，使用默认配置）
///
/// # Returns
/// 配置好的连接池实例
pub fn create_pool(db_path: &Path, config: Option<PoolConfig>) -> Result<DbPool, DbError> {
    let config = config.unwrap_or_default();

    info!(
        "Creating database connection pool: path={}, max_size={}, min_idle={:?}",
        db_path.display(),
        config.max_size,
        config.min_idle
    );

    let pool = create_sqlite_pool(
        db_path,
        Some(CorePoolConfig {
            max_size: config.max_size,
            min_idle: config.min_idle,
            connection_timeout: config.connection_timeout,
            idle_timeout: config.idle_timeout,
        }),
    )
    .map_err(|e| DbError::Pool(e.to_string()))?;

    info!("Database connection pool created successfully");
    Ok(pool)
}

/// 创建内存数据库连接池（用于测试）
#[cfg(test)]
pub fn create_memory_pool() -> Result<DbPool, DbError> {
    let pool = ccr_core::core::sqlite::create_memory_sqlite_pool()
        .map_err(|e| DbError::Pool(e.to_string()))?;

    Ok(pool)
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
