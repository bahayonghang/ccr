// 📚 历史记录服务
// 封装历史记录相关的业务逻辑

use crate::core::error::Result;
use crate::managers::history::{HistoryEntry, HistoryManager, HistoryStats, OperationType};
use std::sync::Arc;

/// 📚 历史记录服务
///
/// 封装所有历史记录相关的业务逻辑
pub struct HistoryService {
    history_manager: Arc<HistoryManager>,
}

#[allow(dead_code)]
impl HistoryService {
    /// 🏗️ 创建新的历史记录服务
    pub fn new(history_manager: Arc<HistoryManager>) -> Self {
        Self { history_manager }
    }

    /// 🏠 使用默认历史记录管理器创建服务
    pub fn default() -> Result<Self> {
        let history_manager = Arc::new(HistoryManager::default()?);
        Ok(Self::new(history_manager))
    }

    /// 📝 记录操作
    ///
    /// # Arguments
    /// - `entry` - 历史记录条目
    ///
    /// # Process
    /// 1. 获取文件锁
    /// 2. 加载现有记录
    /// 3. 添加新记录
    /// 4. 保存到文件
    pub fn record_operation(&self, entry: HistoryEntry) -> Result<()> {
        self.history_manager.add(entry)
    }

    /// 📋 获取最近的记录
    ///
    /// # Arguments
    /// - `limit` - 限制返回的记录数量
    ///
    /// # Returns
    /// 按时间倒序的记录列表(最新的在前)
    pub fn get_recent(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        self.history_manager.get_recent(limit)
    }

    /// 🔍 按操作类型筛选
    ///
    /// # Arguments
    /// - `op_type` - 操作类型(Switch, Backup, Restore等)
    ///
    /// # Returns
    /// 匹配指定类型的所有记录
    pub fn filter_by_type(&self, op_type: OperationType) -> Result<Vec<HistoryEntry>> {
        self.history_manager.filter_by_operation(op_type)
    }

    /// 📊 获取统计信息
    ///
    /// # Returns
    /// 历史记录的统计信息(总数、成功数、失败数等)
    pub fn get_stats(&self) -> Result<HistoryStats> {
        self.history_manager.stats()
    }

    /// 📖 加载所有记录
    pub fn load_all(&self) -> Result<Vec<HistoryEntry>> {
        self.history_manager.load()
    }

    /// 📁 获取历史记录管理器
    pub fn history_manager(&self) -> &Arc<HistoryManager> {
        &self.history_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::lock::LockManager;
    use crate::managers::history::{OperationDetails, OperationResult};
    use tempfile::tempdir;

    #[test]
    fn test_history_service_record_and_get() {
        let temp_dir = tempdir().unwrap();
        let history_path = temp_dir.path().join("history.json");
        let lock_dir = temp_dir.path().join("locks");

        let lock_manager = LockManager::new(lock_dir);
        let manager = Arc::new(HistoryManager::new(history_path, lock_manager));
        let service = HistoryService::new(manager);

        // 记录操作
        let entry = HistoryEntry::new(
            OperationType::Switch,
            OperationDetails {
                from_config: Some("old".into()),
                to_config: Some("new".into()),
                backup_path: None,
                extra: None,
            },
            OperationResult::Success,
        );

        service.record_operation(entry).unwrap();

        // 获取记录
        let recent = service.get_recent(10).unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].operation, OperationType::Switch);
    }

    #[test]
    fn test_history_service_filter() {
        let temp_dir = tempdir().unwrap();
        let history_path = temp_dir.path().join("history.json");
        let lock_dir = temp_dir.path().join("locks");

        let lock_manager = LockManager::new(lock_dir);
        let manager = Arc::new(HistoryManager::new(history_path, lock_manager));
        let service = HistoryService::new(manager);

        // 记录不同类型的操作
        service
            .record_operation(HistoryEntry::new(
                OperationType::Switch,
                OperationDetails {
                    from_config: None,
                    to_config: Some("a".into()),
                    backup_path: None,
                    extra: None,
                },
                OperationResult::Success,
            ))
            .unwrap();

        service
            .record_operation(HistoryEntry::new(
                OperationType::Backup,
                OperationDetails {
                    from_config: None,
                    to_config: None,
                    backup_path: Some("/tmp/backup".into()),
                    extra: None,
                },
                OperationResult::Success,
            ))
            .unwrap();

        // 筛选
        let switch_ops = service.filter_by_type(OperationType::Switch).unwrap();
        assert_eq!(switch_ops.len(), 1);

        let backup_ops = service.filter_by_type(OperationType::Backup).unwrap();
        assert_eq!(backup_ops.len(), 1);
    }
}
