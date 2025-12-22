// 余额快照管理器
// 负责余额快照的存储和历史记录管理，包括 90 天数据清理

use crate::models::checkin::{BalanceHistoryItem, BalanceHistoryResponse, BalanceSnapshot};
use chrono::{Duration, Utc};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

const BALANCES_FILE: &str = "balances.json";
const RETENTION_DAYS: i64 = 90;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum BalanceError {
    #[error("Balance not found for account: {0}")]
    NotFound(String),
    #[error("Failed to read balances: {0}")]
    ReadError(String),
    #[error("Failed to write balances: {0}")]
    WriteError(String),
    #[error("Failed to parse balances: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, BalanceError>;

/// 余额管理器
pub struct BalanceManager {
    /// 数据文件路径
    file_path: PathBuf,
}

impl BalanceManager {
    /// 创建新的余额管理器
    pub fn new(checkin_dir: &Path) -> Self {
        Self {
            file_path: checkin_dir.join(BALANCES_FILE),
        }
    }

    /// 加载所有余额快照
    fn load_all(&self) -> Result<Vec<BalanceSnapshot>> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| BalanceError::ReadError(e.to_string()))?;

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let balances: Vec<BalanceSnapshot> =
            serde_json::from_str(&content).map_err(|e| BalanceError::ParseError(e.to_string()))?;

        Ok(balances)
    }

    /// 保存所有余额快照
    fn save_all(&self, balances: &[BalanceSnapshot]) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                BalanceError::WriteError(format!("Failed to create directory: {}", e))
            })?;
        }

        // 序列化为 JSON
        let content = serde_json::to_string_pretty(balances)
            .map_err(|e| BalanceError::WriteError(format!("Failed to serialize: {}", e)))?;

        // 原子写入
        let temp_dir = self.file_path.parent().unwrap_or(std::path::Path::new("."));
        let mut temp_file = NamedTempFile::new_in(temp_dir)
            .map_err(|e| BalanceError::WriteError(format!("Failed to create temp file: {}", e)))?;

        temp_file
            .write_all(content.as_bytes())
            .map_err(|e| BalanceError::WriteError(format!("Failed to write temp file: {}", e)))?;

        temp_file
            .persist(&self.file_path)
            .map_err(|e| BalanceError::WriteError(format!("Failed to persist file: {}", e)))?;

        tracing::debug!(
            "Saved {} balance snapshots to {:?}",
            balances.len(),
            self.file_path
        );
        Ok(())
    }

    /// 添加余额快照 (自动清理旧数据)
    pub fn add(&self, snapshot: BalanceSnapshot) -> Result<BalanceSnapshot> {
        let mut balances = self.load_all()?;

        // 添加新快照
        balances.push(snapshot.clone());

        // 清理超过 90 天的旧数据
        let cutoff = Utc::now() - Duration::days(RETENTION_DAYS);
        let before_cleanup = balances.len();
        balances.retain(|b| b.recorded_at > cutoff);
        let cleaned = before_cleanup - balances.len();

        if cleaned > 0 {
            tracing::info!(
                "Cleaned up {} balance snapshots older than {} days",
                cleaned,
                RETENTION_DAYS
            );
        }

        self.save_all(&balances)?;
        tracing::debug!(
            "Added balance snapshot for account: {}",
            snapshot.account_id
        );

        Ok(snapshot)
    }

    /// 获取账号最新余额
    #[allow(dead_code)]
    pub fn get_latest(&self, account_id: &str) -> Result<Option<BalanceSnapshot>> {
        let balances = self.load_all()?;

        Ok(balances
            .into_iter()
            .filter(|b| b.account_id == account_id)
            .max_by_key(|b| b.recorded_at))
    }

    /// 获取账号余额历史
    pub fn get_history(
        &self,
        account_id: &str,
        limit: Option<usize>,
    ) -> Result<BalanceHistoryResponse> {
        let balances = self.load_all()?;

        let mut account_balances: Vec<_> = balances
            .into_iter()
            .filter(|b| b.account_id == account_id)
            .collect();

        // 按时间倒序排序
        account_balances.sort_by(|a, b| b.recorded_at.cmp(&a.recorded_at));

        // 限制数量
        if let Some(limit) = limit {
            account_balances.truncate(limit);
        }

        let total = account_balances.len();

        // 计算余额变化
        let mut history: Vec<BalanceHistoryItem> = Vec::with_capacity(total);
        for (i, snapshot) in account_balances.iter().enumerate() {
            let mut item: BalanceHistoryItem = snapshot.into();

            // 与下一条记录（时间上更早）比较计算变化
            if i + 1 < account_balances.len() {
                let prev = &account_balances[i + 1];
                item.change = Some(snapshot.remaining_quota - prev.remaining_quota);
            }

            history.push(item);
        }

        Ok(BalanceHistoryResponse {
            account_id: account_id.to_string(),
            history,
            total,
        })
    }

    /// 删除账号的所有余额记录
    pub fn delete_by_account(&self, account_id: &str) -> Result<usize> {
        let mut balances = self.load_all()?;
        let before = balances.len();

        balances.retain(|b| b.account_id != account_id);

        let deleted = before - balances.len();
        if deleted > 0 {
            self.save_all(&balances)?;
            tracing::info!(
                "Deleted {} balance snapshots for account: {}",
                deleted,
                account_id
            );
        }

        Ok(deleted)
    }

    /// 手动触发数据清理
    #[allow(dead_code)]
    pub fn cleanup_old_data(&self) -> Result<usize> {
        let mut balances = self.load_all()?;
        let cutoff = Utc::now() - Duration::days(RETENTION_DAYS);
        let before = balances.len();

        balances.retain(|b| b.recorded_at > cutoff);

        let cleaned = before - balances.len();
        if cleaned > 0 {
            self.save_all(&balances)?;
            tracing::info!("Manual cleanup: removed {} old balance snapshots", cleaned);
        }

        Ok(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, BalanceManager) {
        let temp_dir = TempDir::new().unwrap();
        let manager = BalanceManager::new(temp_dir.path());
        (temp_dir, manager)
    }

    #[test]
    fn test_add_and_get_latest() {
        let (_temp_dir, manager) = setup();

        let snapshot1 = BalanceSnapshot::new(
            "account-1".to_string(),
            100.0,
            30.0,
            70.0,
            "USD".to_string(),
        );
        manager.add(snapshot1).unwrap();

        // 稍后添加第二个快照
        let snapshot2 = BalanceSnapshot::new(
            "account-1".to_string(),
            100.0,
            40.0,
            60.0,
            "USD".to_string(),
        );
        manager.add(snapshot2).unwrap();

        let latest = manager.get_latest("account-1").unwrap().unwrap();
        assert_eq!(latest.remaining_quota, 60.0);
    }

    #[test]
    fn test_get_history() {
        let (_temp_dir, manager) = setup();

        // 添加多个快照
        for i in 0..5 {
            let snapshot = BalanceSnapshot::new(
                "account-1".to_string(),
                100.0,
                (i * 10) as f64,
                (100 - i * 10) as f64,
                "USD".to_string(),
            );
            manager.add(snapshot).unwrap();
        }

        let history = manager.get_history("account-1", None).unwrap();
        assert_eq!(history.total, 5);

        // 验证按时间倒序
        for i in 0..history.history.len() - 1 {
            assert!(history.history[i].recorded_at >= history.history[i + 1].recorded_at);
        }
    }

    #[test]
    fn test_get_history_with_limit() {
        let (_temp_dir, manager) = setup();

        for i in 0..10 {
            let snapshot = BalanceSnapshot::new(
                "account-1".to_string(),
                100.0,
                (i * 5) as f64,
                (100 - i * 5) as f64,
                "USD".to_string(),
            );
            manager.add(snapshot).unwrap();
        }

        let history = manager.get_history("account-1", Some(3)).unwrap();
        assert_eq!(history.history.len(), 3);
    }

    #[test]
    fn test_delete_by_account() {
        let (_temp_dir, manager) = setup();

        // 添加两个账号的快照
        manager
            .add(BalanceSnapshot::new(
                "account-1".to_string(),
                100.0,
                30.0,
                70.0,
                "USD".to_string(),
            ))
            .unwrap();
        manager
            .add(BalanceSnapshot::new(
                "account-2".to_string(),
                200.0,
                50.0,
                150.0,
                "USD".to_string(),
            ))
            .unwrap();

        // 删除 account-1 的记录
        let deleted = manager.delete_by_account("account-1").unwrap();
        assert_eq!(deleted, 1);

        // 验证 account-1 已删除
        assert!(manager.get_latest("account-1").unwrap().is_none());

        // 验证 account-2 仍存在
        assert!(manager.get_latest("account-2").unwrap().is_some());
    }
}
