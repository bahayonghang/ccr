//! 📇 Session 索引器
//!
//! 管理 Session 的索引、搜索和增量更新。

use crate::core::error::Result;
use crate::models::Platform;
use crate::sessions::models::{IndexStats, Session, SessionFilter, SessionSummary};
use crate::sessions::parser::SessionParser;
use crate::storage::{Database, SessionStore};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// 📇 Session 索引器
///
/// 管理 Session 的索引操作。
pub struct SessionIndexer {
    db: Arc<Database>,
}

#[allow(dead_code)]
impl SessionIndexer {
    /// 创建新的索引器
    pub fn new() -> Result<Self> {
        let db = Database::init_default()?;
        Ok(Self { db: Arc::new(db) })
    }

    /// 使用现有数据库创建索引器
    pub fn with_database(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 索引所有平台的 sessions
    pub fn index_all(&self) -> Result<IndexStats> {
        let mut total_stats = IndexStats::default();

        for platform in &[Platform::Claude, Platform::Codex, Platform::Gemini] {
            match self.index_platform(*platform) {
                Ok(stats) => {
                    total_stats.merge(&stats);
                }
                Err(e) => {
                    warn!("索引平台 {:?} 失败: {}", platform, e);
                    total_stats.errors += 1;
                }
            }
        }

        info!(
            "索引完成: {} 个文件, {} 个新增, {} 个更新, {} 个错误",
            total_stats.files_scanned,
            total_stats.sessions_added,
            total_stats.sessions_updated,
            total_stats.errors
        );

        Ok(total_stats)
    }

    /// 索引单个平台
    pub fn index_platform(&self, platform: Platform) -> Result<IndexStats> {
        let start = std::time::Instant::now();
        let mut stats = IndexStats::default();

        let session_dir = match SessionParser::get_platform_session_dir(&platform) {
            Some(dir) => dir,
            None => {
                debug!("平台 {:?} 的 session 目录不存在", platform);
                return Ok(stats);
            }
        };

        info!("索引平台 {:?}: {}", platform, session_dir.display());

        // 扫描文件
        let files = SessionParser::scan_directory(&session_dir, platform)?;
        stats.files_scanned = files.len() as u64;

        // 获取存储层
        let store = SessionStore::new(&self.db);

        // 增量索引
        for file_path in &files {
            let file_path_str = file_path.to_string_lossy().to_string();

            // 检查是否已索引且未变化
            let current_hash = match std::fs::read(file_path) {
                Ok(content) => blake3::hash(&content).to_hex().to_string(),
                Err(e) => {
                    warn!("无法读取文件 {}: {}", file_path.display(), e);
                    stats.errors += 1;
                    continue;
                }
            };

            if let Ok(Some(existing_hash)) = store.get_file_hash(&file_path_str)
                && existing_hash == current_hash
            {
                stats.files_skipped += 1;
                continue;
            }

            // 解析并存储
            match SessionParser::parse_file(file_path, platform) {
                Ok(session) => {
                    // 转换为 storage 格式
                    let storage_session = crate::storage::session_store::Session {
                        id: session.id,
                        platform: session.platform,
                        title: session.title,
                        cwd: session.cwd,
                        file_path: session.file_path,
                        file_hash: session.file_hash,
                        created_at: session.created_at,
                        updated_at: session.updated_at,
                        message_count: session.message_count,
                        user_message_count: session.user_message_count,
                        assistant_message_count: session.assistant_message_count,
                        tool_use_count: session.tool_use_count,
                        indexed_at: session.indexed_at,
                    };

                    if let Err(e) = store.upsert_sessions(&[storage_session]) {
                        warn!("存储 session 失败: {}", e);
                        stats.errors += 1;
                    } else {
                        stats.sessions_added += 1;
                    }
                }
                Err(e) => {
                    warn!("解析文件失败 {}: {}", file_path.display(), e);
                    stats.errors += 1;
                }
            }
        }

        stats.duration_ms = start.elapsed().as_millis() as u64;
        Ok(stats)
    }

    /// 列出 sessions
    pub fn list(&self, filter: SessionFilter) -> Result<Vec<SessionSummary>> {
        let store = SessionStore::new(&self.db);

        // 转换过滤器
        let storage_filter = crate::storage::session_store::SessionFilter {
            platform: filter.platform,
            from_date: filter.from_date,
            to_date: filter.to_date,
            cwd_prefix: filter.cwd_prefix,
            limit: filter.limit,
            offset: filter.offset,
        };

        let summaries = store.list(storage_filter)?;

        // 转换为 sessions 模块的类型
        Ok(summaries
            .into_iter()
            .map(|s| SessionSummary {
                id: s.id,
                platform: s.platform,
                title: s.title,
                cwd: s.cwd,
                created_at: s.created_at,
                updated_at: s.updated_at,
                message_count: s.message_count,
            })
            .collect())
    }

    /// 搜索 sessions
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SessionSummary>> {
        let store = SessionStore::new(&self.db);
        let summaries = store.search(query, limit)?;

        Ok(summaries
            .into_iter()
            .map(|s| SessionSummary {
                id: s.id,
                platform: s.platform,
                title: s.title,
                cwd: s.cwd,
                created_at: s.created_at,
                updated_at: s.updated_at,
                message_count: s.message_count,
            })
            .collect())
    }

    /// 获取单个 session
    pub fn get(&self, id: &str) -> Result<Option<Session>> {
        let store = SessionStore::new(&self.db);

        if let Some(s) = store.get(id)? {
            Ok(Some(Session {
                id: s.id,
                platform: s.platform,
                title: s.title,
                cwd: s.cwd,
                file_path: s.file_path,
                file_hash: s.file_hash,
                created_at: s.created_at,
                updated_at: s.updated_at,
                message_count: s.message_count,
                user_message_count: s.user_message_count,
                assistant_message_count: s.assistant_message_count,
                tool_use_count: s.tool_use_count,
                indexed_at: s.indexed_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// 清理过期 sessions（文件已不存在）
    pub fn prune_stale(&self) -> Result<usize> {
        let store = SessionStore::new(&self.db);
        store.prune_stale()
    }

    /// 获取统计信息
    pub fn stats(&self) -> Result<crate::storage::session_store::SessionStats> {
        let store = SessionStore::new(&self.db);
        store.stats()
    }

    /// 强制重建索引
    pub fn rebuild(&self) -> Result<IndexStats> {
        info!("重建索引...");

        // 清空现有数据
        let store = SessionStore::new(&self.db);
        store.clear_all()?;

        // 重新索引
        self.index_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_indexer_creation() {
        // 这个测试需要数据库，可能在 CI 中跳过
        if std::env::var("SKIP_DB_TESTS").is_ok() {
            return;
        }

        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::init(&db_path).unwrap();
        let _indexer = SessionIndexer::with_database(Arc::new(db));
    }
}
