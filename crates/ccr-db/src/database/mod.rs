//! 数据库层 — SQLite 连接池、Schema、迁移和 Repository。
//!
//! 连接池工厂与 PRAGMA 统一由 `ccr_core::core::sqlite` 提供（全仓唯一 seam），
//! 本模块负责 ccr-db 侧的 DbError 边界、路径解析、迁移执行与全局池生命周期。

pub mod migrations;
pub mod repositories;
pub mod schema;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use tracing::info;

use crate::core::error::DbError;
use ccr_core::core::sqlite::{create_memory_sqlite_pool, create_sqlite_pool};
pub use ccr_core::core::sqlite::{DbConnection, DbPool, PoolConfig};

/// 全局连接池单例
static GLOBAL_POOL: OnceLock<DbPool> = OnceLock::new();

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

    let pool =
        create_sqlite_pool(db_path, Some(config)).map_err(|e| DbError::Pool(e.to_string()))?;

    info!("Database connection pool created successfully");
    Ok(pool)
}

/// 创建内存数据库连接池（用于测试）
pub fn create_memory_pool() -> Result<DbPool, DbError> {
    let pool = create_memory_sqlite_pool().map_err(|e| DbError::Pool(e.to_string()))?;

    Ok(pool)
}

fn resolve_ccr_root() -> Result<PathBuf, DbError> {
    if let Ok(custom_root) = std::env::var("CCR_DATA_DIR") {
        return Ok(PathBuf::from(custom_root));
    }

    if let Ok(custom_root) = std::env::var("CCR_ROOT") {
        return Ok(PathBuf::from(custom_root));
    }

    let home = dirs::home_dir().ok_or(DbError::HomeDirNotFound)?;
    Ok(home.join(".ccr"))
}

/// 获取数据库文件路径: `~/.ccr-ui/ccr-ui.db`
pub fn get_db_path() -> Result<PathBuf, DbError> {
    let home = dirs::home_dir().ok_or(DbError::HomeDirNotFound)?;
    let db_dir = home.join(".ccr-ui");
    if !db_dir.exists() {
        std::fs::create_dir_all(&db_dir).map_err(|e| DbError::DirectoryCreation(e.to_string()))?;
    }
    Ok(db_dir.join("ccr-ui.db"))
}

/// 获取 usage analytics 数据库文件路径: `~/.ccr/analytics/usage.db`
pub fn get_usage_archive_db_path() -> Result<PathBuf, DbError> {
    let ccr_root = resolve_ccr_root()?;
    let analytics_dir = ccr_root.join("analytics");
    if !analytics_dir.exists() {
        std::fs::create_dir_all(&analytics_dir)
            .map_err(|e| DbError::DirectoryCreation(e.to_string()))?;
    }
    Ok(analytics_dir.join("usage.db"))
}

/// 创建应用连接池并登记为全局池（同一实例）。
///
/// GLOBAL_POOL 与返回值共享同一池（r2d2 Pool 为 Arc 语义）：manager 层
/// `with_connection()` 与 AppState 直取连接看到同一连接上限与迁移状态，
/// 避免同一 DB 文件双池双迁移。
pub fn initialize_app_pool() -> Result<DbPool, DbError> {
    let pool = create_app_pool()?;
    let _ = GLOBAL_POOL.set(pool.clone());
    info!("[ccr-db] database initialized at {:?}", get_db_path()?);
    Ok(pool)
}

/// 创建独立连接池实例（用于 Tauri AppState）
pub fn create_app_pool() -> Result<DbPool, DbError> {
    let db_path = get_db_path()?;
    let config = PoolConfig {
        max_size: 8,
        ..Default::default()
    };
    let pool = create_pool(&db_path, Some(config))?;

    // 运行迁移
    let conn = pool.get().map_err(|e| DbError::PoolGet(e.to_string()))?;
    let home_dir = dirs::home_dir().ok_or(DbError::HomeDirNotFound)?;
    migrations::run_all_migrations(&conn, &home_dir)
        .map_err(|e| DbError::Migration(e.to_string()))?;

    Ok(pool)
}

/// 创建 usage analytics 连接池实例（用于 durable archive / usage 查询）
pub fn create_usage_archive_pool() -> Result<DbPool, DbError> {
    let db_path = get_usage_archive_db_path()?;
    let config = PoolConfig {
        max_size: 8,
        ..Default::default()
    };
    let pool = create_pool(&db_path, Some(config))?;

    let conn = pool.get().map_err(|e| DbError::PoolGet(e.to_string()))?;
    let home_dir = dirs::home_dir().ok_or(DbError::HomeDirNotFound)?;
    migrations::run_all_migrations(&conn, &home_dir)
        .map_err(|e| DbError::Migration(e.to_string()))?;
    migrations::migrate_usage_archive_from_legacy_dbs(&conn, &home_dir, &get_db_path()?)
        .map_err(|e| DbError::Migration(e.to_string()))?;

    Ok(pool)
}

/// 获取全局连接池
pub fn get_pool() -> Option<&'static DbPool> {
    GLOBAL_POOL.get()
}

/// 在指定池上执行闭包（全局路径与注入路径共用的实现）
fn with_connection_on<F, T>(pool: &DbPool, f: F) -> Result<T, DbError>
where
    F: FnOnce(&rusqlite::Connection) -> Result<T, rusqlite::Error>,
{
    let conn = pool.get().map_err(|e| DbError::PoolGet(e.to_string()))?;
    f(&conn).map_err(|e| DbError::Query(e.to_string()))
}

/// 在指定池上执行事务闭包（全局路径与注入路径共用的实现）
fn transaction_on<F, T>(pool: &DbPool, f: F) -> Result<T, DbError>
where
    F: FnOnce(&rusqlite::Transaction<'_>) -> Result<T, rusqlite::Error>,
{
    let mut conn = pool.get().map_err(|e| DbError::PoolGet(e.to_string()))?;
    let tx = conn
        .transaction()
        .map_err(|e| DbError::Query(e.to_string()))?;
    let result = f(&tx).map_err(|e| DbError::Query(e.to_string()))?;
    tx.commit().map_err(|e| DbError::Query(e.to_string()))?;
    Ok(result)
}

/// 使用连接池执行闭包
pub fn with_connection<F, T>(f: F) -> Result<T, DbError>
where
    F: FnOnce(&rusqlite::Connection) -> Result<T, rusqlite::Error>,
{
    let pool = GLOBAL_POOL.get().ok_or(DbError::NotInitialized)?;
    with_connection_on(pool, f)
}

/// 使用事务执行闭包
pub fn transaction<F, T>(f: F) -> Result<T, DbError>
where
    F: FnOnce(&rusqlite::Transaction<'_>) -> Result<T, rusqlite::Error>,
{
    let pool = GLOBAL_POOL.get().ok_or(DbError::NotInitialized)?;
    transaction_on(pool, f)
}

/// 注入式数据库访问句柄。
///
/// manager 层默认走进程级全局池（[`DbAccess::Global`]，行为与自由函数
/// [`with_connection`]/[`transaction`] 完全一致）；测试或嵌入场景可注入
/// 独立池（[`DbAccess::Pool`]），使 manager 方法路径可以脱离 GLOBAL_POOL
/// 单测。错误统一说 [`DbError`]。
#[derive(Clone, Default)]
pub enum DbAccess {
    /// 进程级全局池（GLOBAL_POOL）
    #[default]
    Global,
    /// 注入的独立池（如内存池单测、嵌入场景）
    Pool(DbPool),
}

impl DbAccess {
    /// 在句柄指向的池上执行闭包
    pub fn with_connection<F, T>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&rusqlite::Connection) -> Result<T, rusqlite::Error>,
    {
        match self {
            DbAccess::Global => with_connection(f),
            DbAccess::Pool(pool) => with_connection_on(pool, f),
        }
    }

    /// 在句柄指向的池上执行事务闭包
    pub fn transaction<F, T>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce(&rusqlite::Transaction<'_>) -> Result<T, rusqlite::Error>,
    {
        match self {
            DbAccess::Global => transaction(f),
            DbAccess::Pool(pool) => transaction_on(pool, f),
        }
    }
}

/// 关闭数据库（释放全局池引用无法做到，仅做标记日志）
pub fn shutdown() {
    info!("[ccr-db] database shutdown requested");
}

/// 检查是否已初始化
pub fn is_initialized() -> bool {
    GLOBAL_POOL.get().is_some()
}

/// 初始化测试数据库（内存 SQLite）
#[allow(dead_code)]
pub fn initialize_for_test() -> Result<(), DbError> {
    if is_initialized() {
        return Ok(());
    }

    let pool = create_memory_pool()?;

    {
        let conn = pool.get().map_err(|e| DbError::PoolGet(e.to_string()))?;
        conn.execute_batch(schema::CREATE_TABLES_SQL)
            .map_err(|e| DbError::ConnectionOpen(e.to_string()))?;
    }

    let _ = GLOBAL_POOL.set(pool);
    Ok(())
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
