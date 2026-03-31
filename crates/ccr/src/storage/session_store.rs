//! 📚 Session 存储层
//!
//! 提供 Session 的 CRUD 操作和搜索功能。

use ccr_core::core::error::{CcrError, Result};
use crate::models::Platform;
use ccr_store::storage::database::Database;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, info};

/// 📋 Session 摘要（用于列表展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// 唯一标识
    pub id: String,
    /// 所属平台
    pub platform: Platform,
    /// 标题
    pub title: Option<String>,
    /// 工作目录
    pub cwd: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 消息总数
    pub message_count: u32,
}

/// 📄 Session 完整信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// 唯一标识
    pub id: String,
    /// 所属平台
    pub platform: Platform,
    /// 标题
    pub title: Option<String>,
    /// 工作目录
    pub cwd: PathBuf,
    /// 源文件路径
    pub file_path: PathBuf,
    /// 文件哈希（用于增量更新）
    pub file_hash: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 消息总数
    pub message_count: u32,
    /// 用户消息数
    pub user_message_count: u32,
    /// 助手消息数
    pub assistant_message_count: u32,
    /// 工具调用数
    pub tool_use_count: u32,
    /// 索引时间
    pub indexed_at: DateTime<Utc>,
}

/// 🔍 Session 过滤条件
#[derive(Debug, Clone, Default)]
pub struct SessionFilter {
    /// 平台过滤
    pub platform: Option<Platform>,
    /// 日期范围起始
    pub from_date: Option<DateTime<Utc>>,
    /// 日期范围结束
    pub to_date: Option<DateTime<Utc>>,
    /// 工作目录前缀
    pub cwd_prefix: Option<String>,
    /// 限制数量
    pub limit: Option<usize>,
    /// 偏移量
    pub offset: Option<usize>,
}

/// 📚 Session 存储层
pub struct SessionStore<'a> {
    db: &'a Database,
}

impl<'a> SessionStore<'a> {
    /// 创建新的 SessionStore
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// 批量插入/更新 Session
    ///
    /// 使用 UPSERT 语义，根据 file_path 判断是否已存在。
    pub fn upsert_sessions(&self, sessions: &[Session]) -> Result<usize> {
        let conn = self.db.conn()?;
        let mut count = 0;

        for session in sessions {
            let result = conn.execute(
                r#"
                INSERT INTO sessions (
                    id, platform, title, cwd, file_path, file_hash,
                    created_at, updated_at, message_count,
                    user_message_count, assistant_message_count, tool_use_count
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                ON CONFLICT(file_path) DO UPDATE SET
                    title = excluded.title,
                    file_hash = excluded.file_hash,
                    updated_at = excluded.updated_at,
                    message_count = excluded.message_count,
                    user_message_count = excluded.user_message_count,
                    assistant_message_count = excluded.assistant_message_count,
                    tool_use_count = excluded.tool_use_count,
                    indexed_at = datetime('now')
                "#,
                rusqlite::params![
                    session.id,
                    session.platform.to_string(),
                    session.title,
                    session.cwd.to_string_lossy().to_string(),
                    session.file_path.to_string_lossy().to_string(),
                    session.file_hash,
                    session.created_at.to_rfc3339(),
                    session.updated_at.to_rfc3339(),
                    session.message_count,
                    session.user_message_count,
                    session.assistant_message_count,
                    session.tool_use_count,
                ],
            );

            match result {
                Ok(_) => count += 1,
                Err(e) => {
                    debug!("插入 session {} 失败: {}", session.id, e);
                }
            }
        }

        info!("已插入/更新 {} 个 session", count);
        Ok(count)
    }

    /// 查询 Session 列表
    pub fn list(&self, filter: SessionFilter) -> Result<Vec<SessionSummary>> {
        let conn = self.db.conn()?;

        let mut sql = String::from(
            r#"
            SELECT id, platform, title, cwd, created_at, updated_at, message_count
            FROM sessions
            WHERE 1=1
            "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref platform) = filter.platform {
            sql.push_str(" AND platform = ?");
            params.push(Box::new(platform.to_string()));
        }

        if let Some(ref from_date) = filter.from_date {
            sql.push_str(" AND created_at >= ?");
            params.push(Box::new(from_date.to_rfc3339()));
        }

        if let Some(ref to_date) = filter.to_date {
            sql.push_str(" AND created_at <= ?");
            params.push(Box::new(to_date.to_rfc3339()));
        }

        if let Some(ref cwd_prefix) = filter.cwd_prefix {
            sql.push_str(" AND cwd LIKE ?");
            params.push(Box::new(format!("{}%", cwd_prefix)));
        }

        sql.push_str(" ORDER BY updated_at DESC");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = filter.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| CcrError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(SessionSummary {
                    id: row.get(0)?,
                    platform: Platform::from_str_safe(&row.get::<_, String>(1)?),
                    title: row.get(2)?,
                    cwd: row.get(3)?,
                    created_at: parse_datetime(&row.get::<_, String>(4)?),
                    updated_at: parse_datetime(&row.get::<_, String>(5)?),
                    message_count: row.get::<_, i64>(6)? as u32,
                })
            })
            .map_err(|e| CcrError::DatabaseError(format!("执行查询失败: {}", e)))?;

        let mut sessions = Vec::new();
        for session in rows.flatten() {
            sessions.push(session);
        }

        Ok(sessions)
    }

    /// 搜索 Session
    ///
    /// 在标题和工作目录中搜索关键词。
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SessionSummary>> {
        let conn = self.db.conn()?;

        let search_pattern = format!("%{}%", query);

        let mut stmt = conn
            .prepare(
                r#"
                SELECT id, platform, title, cwd, created_at, updated_at, message_count
                FROM sessions
                WHERE title LIKE ?1 OR cwd LIKE ?1
                ORDER BY updated_at DESC
                LIMIT ?2
                "#,
            )
            .map_err(|e| CcrError::DatabaseError(format!("准备搜索查询失败: {}", e)))?;

        let rows = stmt
            .query_map(rusqlite::params![search_pattern, limit as i64], |row| {
                Ok(SessionSummary {
                    id: row.get(0)?,
                    platform: Platform::from_str_safe(&row.get::<_, String>(1)?),
                    title: row.get(2)?,
                    cwd: row.get(3)?,
                    created_at: parse_datetime(&row.get::<_, String>(4)?),
                    updated_at: parse_datetime(&row.get::<_, String>(5)?),
                    message_count: row.get::<_, i64>(6)? as u32,
                })
            })
            .map_err(|e| CcrError::DatabaseError(format!("执行搜索失败: {}", e)))?;

        let mut sessions = Vec::new();
        for session in rows.flatten() {
            sessions.push(session);
        }

        // 记录搜索历史
        let _ = conn.execute(
            "INSERT INTO search_history (query, scope, result_count) VALUES (?1, ?2, ?3)",
            rusqlite::params![query, "sessions", sessions.len() as i64],
        );

        Ok(sessions)
    }

    /// 根据 ID 获取 Session
    pub fn get(&self, id: &str) -> Result<Option<Session>> {
        let conn = self.db.conn()?;

        let result = conn.query_row(
            r#"
            SELECT id, platform, title, cwd, file_path, file_hash,
                   created_at, updated_at, message_count,
                   user_message_count, assistant_message_count, tool_use_count, indexed_at
            FROM sessions
            WHERE id = ?1
            "#,
            [id],
            |row| {
                Ok(Session {
                    id: row.get(0)?,
                    platform: Platform::from_str_safe(&row.get::<_, String>(1)?),
                    title: row.get(2)?,
                    cwd: PathBuf::from(row.get::<_, String>(3)?),
                    file_path: PathBuf::from(row.get::<_, String>(4)?),
                    file_hash: row.get(5)?,
                    created_at: parse_datetime(&row.get::<_, String>(6)?),
                    updated_at: parse_datetime(&row.get::<_, String>(7)?),
                    message_count: row.get::<_, i64>(8)? as u32,
                    user_message_count: row.get::<_, i64>(9)? as u32,
                    assistant_message_count: row.get::<_, i64>(10)? as u32,
                    tool_use_count: row.get::<_, i64>(11)? as u32,
                    indexed_at: parse_datetime(&row.get::<_, String>(12)?),
                })
            },
        );

        match result {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(CcrError::DatabaseError(format!("查询 session 失败: {}", e))),
        }
    }

    /// 获取文件哈希（用于增量更新检查）
    pub fn get_file_hash(&self, file_path: &str) -> Result<Option<String>> {
        let conn = self.db.conn()?;

        let result: std::result::Result<String, _> = conn.query_row(
            "SELECT file_hash FROM sessions WHERE file_path = ?1",
            [file_path],
            |row| row.get(0),
        );

        match result {
            Ok(hash) => Ok(Some(hash)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(CcrError::DatabaseError(format!("查询文件哈希失败: {}", e))),
        }
    }

    /// 删除过期 Session（文件已不存在）
    pub fn prune_stale(&self) -> Result<usize> {
        let conn = self.db.conn()?;

        // 获取所有 session 的文件路径
        let mut stmt = conn
            .prepare("SELECT id, file_path FROM sessions")
            .map_err(|e| CcrError::DatabaseError(format!("准备查询失败: {}", e)))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| CcrError::DatabaseError(format!("执行查询失败: {}", e)))?;

        let mut stale_ids = Vec::new();
        for (id, file_path) in rows.flatten() {
            if !std::path::Path::new(&file_path).exists() {
                stale_ids.push(id);
            }
        }

        // 删除过期记录
        let count = stale_ids.len();
        for id in stale_ids {
            let _ = conn.execute("DELETE FROM sessions WHERE id = ?1", [&id]);
        }

        info!("已删除 {} 个过期 session", count);
        Ok(count)
    }

    /// 获取 Session 统计
    pub fn stats(&self) -> Result<SessionStats> {
        let conn = self.db.conn()?;

        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
            .unwrap_or_else(|e| {
                debug!("查询 session 总数失败: {}", e);
                0
            });

        let by_platform = conn
            .prepare("SELECT platform, COUNT(*) FROM sessions GROUP BY platform")
            .and_then(|mut stmt| {
                let rows = stmt.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
                })?;
                let mut map = std::collections::HashMap::new();
                for row in rows.flatten() {
                    map.insert(row.0, row.1 as u64);
                }
                Ok(map)
            })
            .unwrap_or_else(|e| {
                debug!("按平台统计 session 失败: {}", e);
                std::collections::HashMap::new()
            });

        Ok(SessionStats {
            total: total as u64,
            by_platform,
        })
    }

    /// 删除所有 Session
    pub fn clear_all(&self) -> Result<usize> {
        let conn = self.db.conn()?;
        let count = conn
            .execute("DELETE FROM sessions", [])
            .map_err(|e| CcrError::DatabaseError(format!("清空 sessions 失败: {}", e)))?;
        Ok(count)
    }
}

/// 📊 Session 统计
#[derive(Debug, Clone)]
pub struct SessionStats {
    /// 总数
    pub total: u64,
    /// 按平台分组
    pub by_platform: std::collections::HashMap<String, u64>,
}

/// 解析 RFC3339 日期时间字符串
fn parse_datetime(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

// 为 Platform 添加辅助方法
impl Platform {
    /// 从字符串安全解析 Platform
    fn from_str_safe(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "claude" => Platform::Claude,
            "codex" => Platform::Codex,
            "gemini" => Platform::Gemini,
            "qwen" => Platform::Qwen,
            "iflow" => Platform::IFlow,
            _ => Platform::Claude, // 默认
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_store::storage::Database;
    use tempfile::tempdir;

    fn create_test_db() -> Database {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        // 保持 dir 存活
        std::mem::forget(dir);
        Database::init(&db_path).unwrap()
    }

    fn create_test_session(id: &str, platform: Platform) -> Session {
        Session {
            id: id.to_string(),
            platform,
            title: Some(format!("Test Session {}", id)),
            cwd: PathBuf::from("/tmp/test"),
            file_path: PathBuf::from(format!("/tmp/test/{}.jsonl", id)),
            file_hash: "abc123".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            message_count: 10,
            user_message_count: 5,
            assistant_message_count: 5,
            tool_use_count: 2,
            indexed_at: Utc::now(),
        }
    }

    #[test]
    fn test_upsert_and_list() {
        let db = create_test_db();
        let store = SessionStore::new(&db);

        let sessions = vec![
            create_test_session("1", Platform::Claude),
            create_test_session("2", Platform::Codex),
        ];

        let count = store.upsert_sessions(&sessions).unwrap();
        assert_eq!(count, 2);

        let list = store.list(SessionFilter::default()).unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_search() {
        let db = create_test_db();
        let store = SessionStore::new(&db);

        let sessions = vec![
            create_test_session("1", Platform::Claude),
            create_test_session("2", Platform::Codex),
        ];

        store.upsert_sessions(&sessions).unwrap();

        let results = store.search("Test Session", 10).unwrap();
        assert_eq!(results.len(), 2);

        let results = store.search("Session 1", 10).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_get_by_id() {
        let db = create_test_db();
        let store = SessionStore::new(&db);

        let sessions = vec![create_test_session("test-123", Platform::Claude)];

        store.upsert_sessions(&sessions).unwrap();

        let session = store.get("test-123").unwrap();
        assert!(session.is_some());
        assert_eq!(session.unwrap().id, "test-123");

        let session = store.get("nonexistent").unwrap();
        assert!(session.is_none());
    }

    #[test]
    fn test_filter_by_platform() {
        let db = create_test_db();
        let store = SessionStore::new(&db);

        let sessions = vec![
            create_test_session("1", Platform::Claude),
            create_test_session("2", Platform::Claude),
            create_test_session("3", Platform::Codex),
        ];

        store.upsert_sessions(&sessions).unwrap();

        let filter = SessionFilter {
            platform: Some(Platform::Claude),
            ..Default::default()
        };

        let results = store.list(filter).unwrap();
        assert_eq!(results.len(), 2);
    }
}
