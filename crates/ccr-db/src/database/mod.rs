//! 数据库层 — SQLite 连接池、Schema、迁移和 Repository。

pub mod migrations;
pub mod pool;
pub mod repositories;
pub mod schema;

use std::path::PathBuf;
use std::sync::OnceLock;

use pool::{DbPool, PoolConfig};
use tracing::info;

use crate::core::error::DbError;

/// 全局连接池单例
static GLOBAL_POOL: OnceLock<DbPool> = OnceLock::new();

/// 获取数据库文件路径: `~/.ccr-ui/ccr-ui.db`
pub fn get_db_path() -> Result<PathBuf, DbError> {
    let home = dirs::home_dir().ok_or(DbError::HomeDirNotFound)?;
    let db_dir = home.join(".ccr-ui");
    if !db_dir.exists() {
        std::fs::create_dir_all(&db_dir).map_err(|e| DbError::DirectoryCreation(e.to_string()))?;
    }
    Ok(db_dir.join("ccr-ui.db"))
}

/// 初始化全局连接池并运行迁移
pub fn initialize() -> Result<(), DbError> {
    let db_path = get_db_path()?;
    let pool = pool::create_pool(&db_path, None)?;

    // 运行迁移
    let conn = pool.get().map_err(|e| DbError::PoolGet(e.to_string()))?;
    let home_dir = dirs::home_dir().ok_or(DbError::HomeDirNotFound)?;
    migrations::run_all_migrations(&conn, &home_dir)
        .map_err(|e| DbError::Migration(e.to_string()))?;

    let _ = GLOBAL_POOL.set(pool);
    info!("[ccr-db] database initialized at {:?}", db_path);
    Ok(())
}

/// 创建独立连接池实例（用于 Tauri AppState）
pub fn create_app_pool() -> Result<DbPool, DbError> {
    let db_path = get_db_path()?;
    let config = PoolConfig {
        max_size: 8,
        ..Default::default()
    };
    let pool = pool::create_pool(&db_path, Some(config))?;

    // 运行迁移
    let conn = pool.get().map_err(|e| DbError::PoolGet(e.to_string()))?;
    let home_dir = dirs::home_dir().ok_or(DbError::HomeDirNotFound)?;
    migrations::run_all_migrations(&conn, &home_dir)
        .map_err(|e| DbError::Migration(e.to_string()))?;

    Ok(pool)
}

/// 获取全局连接池
pub fn get_pool() -> Option<&'static DbPool> {
    GLOBAL_POOL.get()
}

/// 使用连接池执行闭包
pub fn with_connection<F, T>(f: F) -> Result<T, DbError>
where
    F: FnOnce(&rusqlite::Connection) -> Result<T, rusqlite::Error>,
{
    let pool = GLOBAL_POOL.get().ok_or(DbError::NotInitialized)?;
    let conn = pool.get().map_err(|e| DbError::PoolGet(e.to_string()))?;
    f(&conn).map_err(|e| DbError::Query(e.to_string()))
}

/// 使用事务执行闭包
pub fn transaction<F, T>(f: F) -> Result<T, DbError>
where
    F: FnOnce(&rusqlite::Transaction<'_>) -> Result<T, rusqlite::Error>,
{
    let pool = GLOBAL_POOL.get().ok_or(DbError::NotInitialized)?;
    let mut conn = pool.get().map_err(|e| DbError::PoolGet(e.to_string()))?;
    let tx = conn
        .transaction()
        .map_err(|e| DbError::Query(e.to_string()))?;
    let result = f(&tx).map_err(|e| DbError::Query(e.to_string()))?;
    tx.commit().map_err(|e| DbError::Query(e.to_string()))?;
    Ok(result)
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
#[cfg(test)]
pub fn initialize_for_test() -> Result<(), DbError> {
    if is_initialized() {
        return Ok(());
    }

    let pool = pool::create_memory_pool()?;

    {
        let conn = pool.get().map_err(|e| DbError::PoolGet(e.to_string()))?;
        conn.execute_batch(schema::CREATE_TABLES_SQL)
            .map_err(|e| DbError::ConnectionOpen(e.to_string()))?;
    }

    let _ = GLOBAL_POOL.set(pool);
    Ok(())
}
