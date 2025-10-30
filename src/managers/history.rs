// 📚 CCR 历史记录模块
// 🔍 提供完整的操作历史记录和审计功能
//
// 核心功能:
// - 📝 记录所有操作(switch, backup, restore等)
// - 🔐 敏感信息自动掩码(TOKEN, KEY, SECRET)
// - 🔒 文件锁保证并发写入安全
// - 📊 操作统计和筛选
// - 🆔 UUID 唯一标识每个操作
// - 📅 时间戳和操作者追踪

use crate::core::error::{CcrError, Result};
use crate::core::lock::LockManager;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

/// 📋 操作类型枚举
///
/// 定义所有可追踪的操作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationType {
    /// 🔄 切换配置
    Switch,
    /// 💾 备份
    Backup,
    /// 🔄 恢复
    Restore,
    /// ✅ 验证
    Validate,
    /// 🔄 更新
    Update,
}

impl OperationType {
    pub fn as_str(&self) -> &str {
        match self {
            OperationType::Switch => "切换配置",
            OperationType::Backup => "备份",
            OperationType::Restore => "恢复",
            OperationType::Validate => "验证",
            OperationType::Update => "更新",
        }
    }
}

/// 操作结果枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationResult {
    /// 成功
    Success,
    /// 失败
    Failure(String),
    /// 警告(部分成功)
    Warning(String),
}

/// 环境变量变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvChange {
    /// 变量名
    pub var_name: String,
    /// 旧值(已掩码)
    pub old_value: Option<String>,
    /// 新值(已掩码)
    pub new_value: Option<String>,
}

/// 操作详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationDetails {
    /// 源配置(如果适用)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_config: Option<String>,
    /// 目标配置(如果适用)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_config: Option<String>,
    /// 备份路径(如果适用)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_path: Option<String>,
    /// 其他详细信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<String>,
}

/// 历史记录条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// 记录 ID (UUID)
    pub id: String,
    /// 时间戳
    pub timestamp: DateTime<Local>,
    /// 操作者(系统用户名)
    pub actor: String,
    /// 操作类型
    pub operation: OperationType,
    /// 操作详情
    pub details: OperationDetails,
    /// 环境变量变更
    #[serde(default)]
    pub env_changes: Vec<EnvChange>,
    /// 操作结果
    pub result: OperationResult,
    /// 备注
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl HistoryEntry {
    /// 创建新的历史记录条目
    pub fn new(
        operation: OperationType,
        details: OperationDetails,
        result: OperationResult,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Local::now(),
            actor: whoami::username(),
            operation,
            details,
            env_changes: Vec::new(),
            result,
            notes: None,
        }
    }

    /// 添加环境变量变更记录
    pub fn add_env_change(
        &mut self,
        var_name: String,
        old_value: Option<String>,
        new_value: Option<String>,
    ) {
        // 对敏感信息进行掩码处理
        let old_masked = old_value.map(|v| crate::utils::mask_if_sensitive(&var_name, &v));
        let new_masked = new_value.map(|v| crate::utils::mask_if_sensitive(&var_name, &v));

        self.env_changes.push(EnvChange {
            var_name,
            old_value: old_masked,
            new_value: new_masked,
        });
    }
}

/// 历史记录管理器
pub struct HistoryManager {
    history_path: PathBuf,
    lock_manager: LockManager,
}

impl HistoryManager {
    /// 创建新的历史记录管理器
    pub fn new<P: AsRef<Path>>(history_path: P, lock_manager: LockManager) -> Self {
        Self {
            history_path: history_path.as_ref().to_path_buf(),
            lock_manager,
        }
    }

    /// 使用默认路径创建管理器
    ///
    /// ⚙️ **开发者注意**：
    /// 可以通过环境变量 `CCR_HISTORY_PATH` 覆盖默认路径
    pub fn with_default() -> Result<Self> {
        // 🔍 检查环境变量
        let history_path = if let Ok(custom_path) = std::env::var("CCR_HISTORY_PATH") {
            std::path::PathBuf::from(custom_path)
        } else {
            let home = dirs::home_dir()
                .ok_or_else(|| CcrError::HistoryError("无法获取用户主目录".into()))?;
            home.join(".claude").join("ccr_history.json")
        };

        log::debug!("使用历史路径: {:?}", &history_path);
        let lock_manager = LockManager::with_default_path()?;

        Ok(Self::new(history_path, lock_manager))
    }

    /// 加载历史记录
    pub fn load(&self) -> Result<Vec<HistoryEntry>> {
        if !self.history_path.exists() {
            // 文件不存在时返回空列表
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.history_path)
            .map_err(|e| CcrError::HistoryError(format!("读取历史文件失败: {}", e)))?;

        let entries: Vec<HistoryEntry> = serde_json::from_str(&content)
            .map_err(|e| CcrError::HistoryError(format!("解析历史文件失败: {}", e)))?;

        Ok(entries)
    }

    /// 保存历史记录
    fn save(&self, entries: &[HistoryEntry]) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.history_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CcrError::HistoryError(format!("创建历史目录失败: {}", e)))?;
        }

        // 序列化为 JSON(美化格式)
        let content = serde_json::to_string_pretty(entries)
            .map_err(|e| CcrError::HistoryError(format!("序列化历史记录失败: {}", e)))?;

        // 写入文件
        fs::write(&self.history_path, content)
            .map_err(|e| CcrError::HistoryError(format!("写入历史文件失败: {}", e)))?;

        Ok(())
    }

    /// 添加历史记录条目
    ///
    /// 🔄 **自动清理策略**：
    /// - 保留最近 10 条记录
    /// - 按时间倒序自动排序
    /// - 清理在每次添加时自动触发
    pub fn add(&self, entry: HistoryEntry) -> Result<()> {
        log::debug!(
            "开始添加历史记录: operation={:?}, to_config={:?}",
            entry.operation,
            entry.details.to_config
        );

        // 获取文件锁
        let _lock = self.lock_manager.lock_history(Duration::from_secs(10))?;
        log::debug!("已获取历史记录文件锁");

        // 加载现有记录
        let mut entries = self.load()?;
        let old_count = entries.len();
        log::debug!("已加载 {} 条现有历史记录", old_count);

        // 添加新记录
        entries.push(entry.clone());

        // 🔄 自动清理：按时间倒序排序，只保留最近 10 条
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        const MAX_HISTORY_ENTRIES: usize = 10;
        if entries.len() > MAX_HISTORY_ENTRIES {
            let removed_count = entries.len() - MAX_HISTORY_ENTRIES;
            entries.truncate(MAX_HISTORY_ENTRIES);
            log::debug!("🗑️ 自动清理了 {} 条旧历史记录", removed_count);
        }

        // 保存
        self.save(&entries)?;
        log::info!(
            "✅ 历史记录已添加 (ID: {}, 总数: {} -> {})",
            entry.id,
            old_count,
            entries.len()
        );
        log::debug!("历史记录文件路径: {:?}", self.history_path);

        Ok(())
    }

    /// 按操作类型筛选
    pub fn filter_by_operation(&self, op_type: OperationType) -> Result<Vec<HistoryEntry>> {
        let entries = self.load()?;
        Ok(entries
            .into_iter()
            .filter(|e| e.operation == op_type)
            .collect())
    }

    /// 获取最近的 N 条记录
    pub fn get_recent(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut entries = self.load()?;
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries.truncate(limit);
        Ok(entries)
    }

    /// 统计操作
    pub fn stats(&self) -> Result<HistoryStats> {
        let entries = self.load()?;
        let mut stats = HistoryStats::new();

        for entry in &entries {
            stats.total_operations += 1;

            match &entry.result {
                OperationResult::Success => stats.successful_operations += 1,
                OperationResult::Failure(_) => stats.failed_operations += 1,
                OperationResult::Warning(_) => stats.warning_operations += 1,
            }

            *stats
                .operations_by_type
                .entry(entry.operation.as_str().to_string())
                .or_insert(0) += 1;
        }

        // 获取最近一次操作
        if let Some(latest) = entries.iter().max_by_key(|e| e.timestamp) {
            stats.last_operation = Some(latest.clone());
        }

        Ok(stats)
    }
}

/// 历史统计信息
#[derive(Debug, Clone)]
pub struct HistoryStats {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub warning_operations: usize,
    pub operations_by_type: std::collections::HashMap<String, usize>,
    pub last_operation: Option<HistoryEntry>,
}

impl HistoryStats {
    pub fn new() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            warning_operations: 0,
            operations_by_type: std::collections::HashMap::new(),
            last_operation: None,
        }
    }
}

impl Default for HistoryStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_type() {
        assert_eq!(OperationType::Switch.as_str(), "切换配置");
        assert_eq!(OperationType::Backup.as_str(), "备份");
    }

    #[test]
    fn test_history_entry() {
        let details = OperationDetails {
            from_config: Some("old".into()),
            to_config: Some("new".into()),
            backup_path: None,
            extra: None,
        };

        let mut entry = HistoryEntry::new(OperationType::Switch, details, OperationResult::Success);

        entry.add_env_change(
            "ANTHROPIC_AUTH_TOKEN".into(),
            Some("old-token".into()),
            Some("new-token".into()),
        );

        assert_eq!(entry.env_changes.len(), 1);
        assert_eq!(entry.operation, OperationType::Switch);
    }

    #[test]
    fn test_history_manager() {
        let temp_dir = tempfile::tempdir().unwrap();
        let history_path = temp_dir.path().join("history.json");
        let lock_dir = temp_dir.path().join("locks");

        let lock_manager = LockManager::new(lock_dir);
        let manager = HistoryManager::new(history_path, lock_manager);

        // 添加记录
        let entry = HistoryEntry::new(
            OperationType::Switch,
            OperationDetails {
                from_config: Some("a".into()),
                to_config: Some("b".into()),
                backup_path: None,
                extra: None,
            },
            OperationResult::Success,
        );

        manager.add(entry).unwrap();

        // 加载并验证
        let entries = manager.load().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].operation, OperationType::Switch);
    }

    #[test]
    fn test_history_filter() {
        let temp_dir = tempfile::tempdir().unwrap();
        let history_path = temp_dir.path().join("history.json");
        let lock_dir = temp_dir.path().join("locks");

        let lock_manager = LockManager::new(lock_dir);
        let manager = HistoryManager::new(history_path, lock_manager);

        // 添加不同类型的记录
        let details = OperationDetails {
            from_config: None,
            to_config: None,
            backup_path: None,
            extra: None,
        };

        manager
            .add(HistoryEntry::new(
                OperationType::Switch,
                details.clone(),
                OperationResult::Success,
            ))
            .unwrap();

        manager
            .add(HistoryEntry::new(
                OperationType::Backup,
                details,
                OperationResult::Success,
            ))
            .unwrap();

        // 筛选
        let switch_ops = manager.filter_by_operation(OperationType::Switch).unwrap();
        assert_eq!(switch_ops.len(), 1);

        let backup_ops = manager.filter_by_operation(OperationType::Backup).unwrap();
        assert_eq!(backup_ops.len(), 1);
    }

    #[test]
    fn test_history_stats() {
        let temp_dir = tempfile::tempdir().unwrap();
        let history_path = temp_dir.path().join("history.json");
        let lock_dir = temp_dir.path().join("locks");

        let lock_manager = LockManager::new(lock_dir);
        let manager = HistoryManager::new(history_path, lock_manager);

        // 添加多条记录
        let details = OperationDetails {
            from_config: None,
            to_config: None,
            backup_path: None,
            extra: None,
        };

        manager
            .add(HistoryEntry::new(
                OperationType::Switch,
                details.clone(),
                OperationResult::Success,
            ))
            .unwrap();

        manager
            .add(HistoryEntry::new(
                OperationType::Backup,
                details.clone(),
                OperationResult::Failure("error".into()),
            ))
            .unwrap();

        // 获取统计
        let stats = manager.stats().unwrap();
        assert_eq!(stats.total_operations, 2);
        assert_eq!(stats.successful_operations, 1);
        assert_eq!(stats.failed_operations, 1);
    }

    #[test]
    fn test_history_auto_cleanup() {
        let temp_dir = tempfile::tempdir().unwrap();
        let history_path = temp_dir.path().join("history.json");
        let lock_dir = temp_dir.path().join("locks");

        let lock_manager = LockManager::new(lock_dir);
        let manager = HistoryManager::new(history_path, lock_manager);

        // 添加 15 条记录（超过限制的 10 条）
        for i in 0..15 {
            manager
                .add(HistoryEntry::new(
                    OperationType::Switch,
                    OperationDetails {
                        from_config: None,
                        to_config: Some(format!("config-{}", i)),
                        backup_path: None,
                        extra: None,
                    },
                    OperationResult::Success,
                ))
                .unwrap();
        }

        // 验证只保留了最近 10 条
        let entries = manager.load().unwrap();
        assert_eq!(entries.len(), 10, "历史记录应该自动清理为最近 10 条");

        // 验证记录是按时间倒序排列的（最新的在前）
        for i in 0..entries.len() - 1 {
            assert!(
                entries[i].timestamp >= entries[i + 1].timestamp,
                "历史记录应按时间倒序排列"
            );
        }

        // 验证保留的是最新的 10 条（config-5 到 config-14）
        let config_names: Vec<_> = entries
            .iter()
            .filter_map(|e| e.details.to_config.as_ref())
            .collect();

        // 最新的配置应该是 config-14
        assert_eq!(config_names[0], "config-14");
        // 最旧的保留配置应该是 config-5
        assert_eq!(config_names[9], "config-5");
    }
}
