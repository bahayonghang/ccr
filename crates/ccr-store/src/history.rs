// 📚 CCR 历史记录模块
// 🔍 提供完整的操作历史记录和审计功能
//
// 核心功能:
// - 📝 记录所有操作(switch, backup, restore等)
// - 🔐 敏感信息自动掩码(TOKEN, KEY, SECRET)
// - 📊 操作统计和筛选
// - 🆔 UUID 唯一标识每个操作
// - 📅 时间戳和操作者追踪
// - 💾 SQLite 持久化存储(~/.ccr/data.db)

use crate::storage::Database;
use ccr_core::core::error::{CcrError, Result};
use chrono::{DateTime, Local};
use rusqlite::params;
use serde::{Deserialize, Serialize};
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

    fn to_db_str(&self) -> &str {
        match self {
            OperationType::Switch => "Switch",
            OperationType::Backup => "Backup",
            OperationType::Restore => "Restore",
            OperationType::Validate => "Validate",
            OperationType::Update => "Update",
        }
    }

    fn from_db_str(s: &str) -> Self {
        match s {
            "Switch" => OperationType::Switch,
            "Backup" => OperationType::Backup,
            "Restore" => OperationType::Restore,
            "Validate" => OperationType::Validate,
            "Update" => OperationType::Update,
            _ => OperationType::Switch,
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
            actor: whoami::username().unwrap_or_else(|_| "unknown".to_string()),
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
        let old_masked = old_value.map(|v| ccr_core::mask_if_sensitive(&var_name, &v));
        let new_masked = new_value.map(|v| ccr_core::mask_if_sensitive(&var_name, &v));

        self.env_changes.push(EnvChange {
            var_name,
            old_value: old_masked,
            new_value: new_masked,
        });
    }
}

/// 历史记录管理器
pub struct HistoryManager {
    db: Database,
}

impl HistoryManager {
    /// 创建新的历史记录管理器
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// 使用默认路径创建管理器
    pub fn with_default() -> Result<Self> {
        let db = Database::init_default()?;
        Ok(Self::new(db))
    }

    /// 从行数据构建 HistoryEntry
    fn row_to_entry(row: &rusqlite::Row) -> rusqlite::Result<HistoryEntry> {
        let timestamp_str: String = row.get(1)?;
        let operation_str: String = row.get(3)?;
        let result_str: String = row.get(4)?;
        let result_message: Option<String> = row.get(5)?;
        let env_changes_json: Option<String> = row.get(11)?;

        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
            .map(|dt| dt.with_timezone(&Local))
            .unwrap_or_else(|_| Local::now());

        let result = match result_str.as_str() {
            "Failure" => OperationResult::Failure(result_message.unwrap_or_default()),
            "Warning" => OperationResult::Warning(result_message.unwrap_or_default()),
            _ => OperationResult::Success,
        };

        let env_changes: Vec<EnvChange> = env_changes_json
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default();

        Ok(HistoryEntry {
            id: row.get(0)?,
            timestamp,
            actor: row.get(2)?,
            operation: OperationType::from_db_str(&operation_str),
            details: OperationDetails {
                from_config: row.get(6)?,
                to_config: row.get(7)?,
                backup_path: row.get(8)?,
                extra: row.get(9)?,
            },
            env_changes,
            result,
            notes: row.get(10)?,
        })
    }

    /// 添加历史记录条目
    pub fn add(&self, entry: HistoryEntry) -> Result<()> {
        let conn = self.db.conn()?;

        let (result_str, result_message) = match &entry.result {
            OperationResult::Success => ("Success", None),
            OperationResult::Failure(msg) => ("Failure", Some(msg.clone())),
            OperationResult::Warning(msg) => ("Warning", Some(msg.clone())),
        };

        let env_changes_json =
            if entry.env_changes.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&entry.env_changes).map_err(|e| {
                    CcrError::HistoryError(format!("序列化 env_changes 失败: {}", e))
                })?)
            };

        conn.execute(
            "INSERT INTO history (id, timestamp, actor, operation, result, result_message, from_config, to_config, backup_path, extra, notes, env_changes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                entry.id,
                entry.timestamp.to_rfc3339(),
                entry.actor,
                entry.operation.to_db_str(),
                result_str,
                result_message,
                entry.details.from_config,
                entry.details.to_config,
                entry.details.backup_path,
                entry.details.extra,
                entry.notes,
                env_changes_json,
            ],
        )
        .map_err(|e| CcrError::HistoryError(format!("写入历史记录失败: {}", e)))?;

        tracing::info!("✅ 历史记录已添加 (ID: {})", entry.id);
        Ok(())
    }

    /// 异步添加历史记录条目
    pub async fn add_async(&self, entry: HistoryEntry) -> Result<()> {
        self.add(entry)
    }

    /// 加载历史记录
    #[allow(dead_code)]
    pub fn load(&self) -> Result<Vec<HistoryEntry>> {
        let conn = self.db.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, timestamp, actor, operation, result, result_message, from_config, to_config, backup_path, extra, notes, env_changes
                 FROM history ORDER BY timestamp DESC",
            )
            .map_err(|e| CcrError::HistoryError(format!("查询历史记录失败: {}", e)))?;

        let entries = stmt
            .query_map([], Self::row_to_entry)
            .map_err(|e| CcrError::HistoryError(format!("查询历史记录失败: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }

    /// 异步加载历史记录
    pub async fn load_async(&self) -> Result<Vec<HistoryEntry>> {
        self.load()
    }

    /// 获取最近的 N 条记录
    pub fn get_recent(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let conn = self.db.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, timestamp, actor, operation, result, result_message, from_config, to_config, backup_path, extra, notes, env_changes
                 FROM history ORDER BY timestamp DESC LIMIT ?1",
            )
            .map_err(|e| CcrError::HistoryError(format!("查询历史记录失败: {}", e)))?;

        let entries = stmt
            .query_map(params![limit as i64], Self::row_to_entry)
            .map_err(|e| CcrError::HistoryError(format!("查询历史记录失败: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }

    /// 异步获取最近的 N 条记录
    pub async fn get_recent_async(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        self.get_recent(limit)
    }

    /// 按操作类型筛选
    pub fn filter_by_operation(&self, op_type: OperationType) -> Result<Vec<HistoryEntry>> {
        let conn = self.db.conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, timestamp, actor, operation, result, result_message, from_config, to_config, backup_path, extra, notes, env_changes
                 FROM history WHERE operation = ?1 ORDER BY timestamp DESC",
            )
            .map_err(|e| CcrError::HistoryError(format!("查询历史记录失败: {}", e)))?;

        let entries = stmt
            .query_map(params![op_type.to_db_str()], Self::row_to_entry)
            .map_err(|e| CcrError::HistoryError(format!("查询历史记录失败: {}", e)))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }

    /// 异步按操作类型筛选
    pub async fn filter_by_operation_async(
        &self,
        op_type: OperationType,
    ) -> Result<Vec<HistoryEntry>> {
        self.filter_by_operation(op_type)
    }

    /// 统计操作
    pub fn stats(&self) -> Result<HistoryStats> {
        let conn = self.db.conn()?;

        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM history", [], |row| row.get(0))
            .unwrap_or(0);

        let successful: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM history WHERE result = 'Success'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let failed: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM history WHERE result = 'Failure'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let warning: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM history WHERE result = 'Warning'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // 按类型统计
        let mut operations_by_type = std::collections::HashMap::new();
        {
            let mut stmt = conn
                .prepare("SELECT operation, COUNT(*) FROM history GROUP BY operation")
                .map_err(|e| CcrError::HistoryError(format!("统计查询失败: {}", e)))?;
            let rows = stmt
                .query_map([], |row| {
                    let op: String = row.get(0)?;
                    let count: i64 = row.get(1)?;
                    Ok((op, count))
                })
                .map_err(|e| CcrError::HistoryError(format!("统计查询失败: {}", e)))?;
            for row in rows.flatten() {
                let display_name = OperationType::from_db_str(&row.0).as_str().to_string();
                operations_by_type.insert(display_name, row.1 as usize);
            }
        }

        // 最近一次操作
        let last_operation = self.get_recent(1)?.into_iter().next();

        Ok(HistoryStats {
            total_operations: total as usize,
            successful_operations: successful as usize,
            failed_operations: failed as usize,
            warning_operations: warning as usize,
            operations_by_type,
            last_operation,
        })
    }

    /// 异步统计操作
    pub async fn stats_async(&self) -> Result<HistoryStats> {
        self.stats()
    }

    /// 🗑️ 清空所有历史记录
    pub fn clear(&self) -> Result<()> {
        let conn = self.db.conn()?;
        conn.execute("DELETE FROM history", [])
            .map_err(|e| CcrError::HistoryError(format!("清空历史记录失败: {}", e)))?;
        tracing::info!("✅ 历史记录已清空");
        Ok(())
    }

    /// 🗑️ 异步清空所有历史记录
    pub async fn clear_async(&self) -> Result<()> {
        self.clear()
    }
}

/// 历史统计信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn setup_manager() -> HistoryManager {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::init(&db_path).unwrap();
        // 保持 tempdir 不被 drop
        std::mem::forget(dir);
        HistoryManager::new(db)
    }

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
        let manager = setup_manager();

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

        let entries = manager.load().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].operation, OperationType::Switch);
    }

    #[test]
    fn test_history_filter() {
        let manager = setup_manager();

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

        let switch_ops = manager.filter_by_operation(OperationType::Switch).unwrap();
        assert_eq!(switch_ops.len(), 1);

        let backup_ops = manager.filter_by_operation(OperationType::Backup).unwrap();
        assert_eq!(backup_ops.len(), 1);
    }

    #[test]
    fn test_history_stats() {
        let manager = setup_manager();

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
                OperationResult::Failure("error".into()),
            ))
            .unwrap();

        let stats = manager.stats().unwrap();
        assert_eq!(stats.total_operations, 2);
        assert_eq!(stats.successful_operations, 1);
        assert_eq!(stats.failed_operations, 1);
    }

    #[test]
    fn test_history_no_limit() {
        let manager = setup_manager();

        // 添加 15 条记录，验证不再有 10 条上限
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

        let entries = manager.load().unwrap();
        assert_eq!(entries.len(), 15, "SQLite 不再有条数上限");
    }
}
