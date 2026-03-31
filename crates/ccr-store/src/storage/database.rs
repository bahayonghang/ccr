//! 🗄️ 数据库连接管理
//!
//! 提供 SQLite 数据库连接池和迁移管理。

use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::sqlite::{DbConnection, DbPool, PoolConfig, create_sqlite_pool};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// 🗄️ 数据库管理器
///
/// 管理 SQLite 数据库连接池和迁移。
pub struct Database {
    pool: DbPool,
    #[allow(dead_code)]
    path: PathBuf,
}

#[allow(dead_code)]
impl Database {
    /// 初始化默认位置的数据库
    ///
    /// 默认路径: `~/.ccr/data.db`
    pub fn init_default() -> Result<Self> {
        let ccr_dir = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("无法获取用户目录".to_string()))?
            .join(".ccr");

        std::fs::create_dir_all(&ccr_dir).map_err(|e| {
            CcrError::DatabaseError(format!("无法创建 CCR 目录 {}: {}", ccr_dir.display(), e))
        })?;

        let db_path = ccr_dir.join("data.db");
        Self::init(&db_path)
    }

    /// 初始化指定路径的数据库
    ///
    /// # 参数
    /// - `path`: 数据库文件路径
    ///
    /// # 返回
    /// - `Ok(Database)`: 初始化成功的数据库实例
    /// - `Err`: 初始化失败
    pub fn init(path: &Path) -> Result<Self> {
        info!("初始化数据库: {}", path.display());

        let pool = create_sqlite_pool(
            path,
            Some(PoolConfig {
                max_size: 10,
                min_idle: Some(1),
                ..Default::default()
            }),
        )
        .map_err(|e| CcrError::DatabaseError(format!("无法创建连接池: {}", e)))?;

        let db = Self {
            pool,
            path: path.to_path_buf(),
        };

        // 运行迁移
        db.migrate()?;

        Ok(db)
    }

    /// 获取数据库连接
    pub fn conn(&self) -> Result<DbConnection> {
        self.pool
            .get()
            .map_err(|e| CcrError::DatabaseError(format!("无法获取数据库连接: {}", e)))
    }

    /// 获取数据库路径
    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 运行数据库迁移
    ///
    /// 确保所有必要的表和索引都已创建。
    pub fn migrate(&self) -> Result<()> {
        let conn = self.conn()?;

        // 启用 WAL 模式以提高并发性能
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(|e| CcrError::DatabaseError(format!("无法设置 PRAGMA: {}", e)))?;

        // 创建迁移表
        conn.execute(
            "CREATE TABLE IF NOT EXISTS migrations (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )
        .map_err(|e| CcrError::DatabaseError(format!("无法创建迁移表: {}", e)))?;

        // 运行迁移
        self.run_migration(
            &conn,
            "001_create_sessions",
            Self::migration_001_create_sessions,
        )?;
        self.run_migration(
            &conn,
            "002_create_search_history",
            Self::migration_002_create_search_history,
        )?;
        self.run_migration(
            &conn,
            "003_create_history",
            Self::migration_003_create_history,
        )?;

        info!("数据库迁移完成");
        Ok(())
    }

    /// 运行单个迁移
    fn run_migration(
        &self,
        conn: &Connection,
        name: &str,
        migration_fn: fn(&Connection) -> Result<()>,
    ) -> Result<()> {
        // 检查迁移是否已应用
        let applied: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM migrations WHERE name = ?1)",
                [name],
                |row| row.get(0),
            )
            .map_err(|e| CcrError::DatabaseError(format!("检查迁移状态失败: {}", e)))?;

        if applied {
            debug!("迁移 {} 已应用，跳过", name);
            return Ok(());
        }

        info!("应用迁移: {}", name);

        // 运行迁移
        migration_fn(conn)?;

        // 记录迁移
        conn.execute("INSERT INTO migrations (name) VALUES (?1)", [name])
            .map_err(|e| CcrError::DatabaseError(format!("记录迁移失败: {}", e)))?;

        Ok(())
    }

    /// 迁移 001: 创建 sessions 表
    fn migration_001_create_sessions(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                platform TEXT NOT NULL,
                title TEXT,
                cwd TEXT NOT NULL,
                file_path TEXT NOT NULL UNIQUE,
                file_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                message_count INTEGER DEFAULT 0,
                user_message_count INTEGER DEFAULT 0,
                assistant_message_count INTEGER DEFAULT 0,
                tool_use_count INTEGER DEFAULT 0,
                indexed_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_sessions_platform ON sessions(platform);
            CREATE INDEX IF NOT EXISTS idx_sessions_created_at ON sessions(created_at);
            CREATE INDEX IF NOT EXISTS idx_sessions_updated_at ON sessions(updated_at);
            CREATE INDEX IF NOT EXISTS idx_sessions_cwd ON sessions(cwd);
            "#,
        )
        .map_err(|e| CcrError::DatabaseError(format!("创建 sessions 表失败: {}", e)))?;

        Ok(())
    }

    /// 迁移 002: 创建 search_history 表
    fn migration_002_create_search_history(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS search_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                query TEXT NOT NULL,
                scope TEXT NOT NULL,
                result_count INTEGER DEFAULT 0,
                searched_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_search_history_searched_at 
                ON search_history(searched_at);
            "#,
        )
        .map_err(|e| CcrError::DatabaseError(format!("创建 search_history 表失败: {}", e)))?;

        Ok(())
    }

    /// 迁移 003: 创建 history 表
    fn migration_003_create_history(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                actor TEXT NOT NULL,
                operation TEXT NOT NULL,
                result TEXT NOT NULL,
                result_message TEXT,
                from_config TEXT,
                to_config TEXT,
                backup_path TEXT,
                extra TEXT,
                notes TEXT,
                env_changes TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_history_timestamp ON history(timestamp);
            CREATE INDEX IF NOT EXISTS idx_history_operation ON history(operation);
            "#,
        )
        .map_err(|e| CcrError::DatabaseError(format!("创建 history 表失败: {}", e)))?;

        Ok(())
    }

    /// 获取数据库统计信息
    pub fn stats(&self) -> Result<DatabaseStats> {
        let conn = self.conn()?;

        let session_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
            .unwrap_or_else(|e| {
                debug!("查询 session 数量失败: {}", e);
                0
            });

        let search_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM search_history", [], |row| row.get(0))
            .unwrap_or_else(|e| {
                debug!("查询搜索历史数量失败: {}", e);
                0
            });

        // 获取文件大小
        let file_size = std::fs::metadata(&self.path)
            .map(|m| m.len())
            .unwrap_or_else(|e| {
                debug!("获取数据库文件大小失败: {}", e);
                0
            });

        Ok(DatabaseStats {
            session_count: session_count as u64,
            search_history_count: search_count as u64,
            file_size_bytes: file_size,
        })
    }

    /// 清空所有数据（危险操作）
    pub fn clear_all(&self) -> Result<()> {
        warn!("清空所有数据库数据");
        let conn = self.conn()?;
        conn.execute_batch("DELETE FROM sessions; DELETE FROM search_history;")
            .map_err(|e| CcrError::DatabaseError(format!("清空数据失败: {}", e)))?;
        Ok(())
    }
}

/// 📊 数据库统计信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DatabaseStats {
    /// Session 数量
    pub session_count: u64,
    /// 搜索历史数量
    pub search_history_count: u64,
    /// 数据库文件大小（字节）
    pub file_size_bytes: u64,
}

impl DatabaseStats {
    /// 格式化文件大小
    #[allow(dead_code)]
    pub fn file_size_display(&self) -> String {
        let size = self.file_size_bytes;
        if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", size as f64 / 1024.0 / 1024.0)
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_init() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let db = Database::init(&db_path).unwrap();
        assert!(db_path.exists());

        // 验证迁移已应用
        let conn = db.conn().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM migrations", [], |row| row.get(0))
            .unwrap();
        assert!(count >= 2);
    }

    #[test]
    fn test_database_stats() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let db = Database::init(&db_path).unwrap();
        let stats = db.stats().unwrap();

        assert_eq!(stats.session_count, 0);
        assert_eq!(stats.search_history_count, 0);
        assert!(stats.file_size_bytes > 0);
    }
}
