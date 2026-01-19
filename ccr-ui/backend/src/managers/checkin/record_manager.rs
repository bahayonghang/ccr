// 签到记录管理器
// 负责签到记录的存储和历史记录管理，包括 90 天数据清理

use crate::models::checkin::{
    CheckinRecord, CheckinRecordInfo, CheckinRecordsResponse, CheckinStatus,
};
use chrono::{Duration, Utc};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

const RECORDS_FILE: &str = "records.json";
const RETENTION_DAYS: i64 = 90;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum RecordError {
    #[error("Record not found: {0}")]
    NotFound(String),
    #[error("Failed to read records: {0}")]
    ReadError(String),
    #[error("Failed to write records: {0}")]
    WriteError(String),
    #[error("Failed to parse records: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, RecordError>;

/// 签到记录管理器
pub struct RecordManager {
    /// 数据文件路径
    file_path: PathBuf,
}

impl RecordManager {
    /// 创建新的签到记录管理器
    pub fn new(checkin_dir: &Path) -> Self {
        Self {
            file_path: checkin_dir.join(RECORDS_FILE),
        }
    }

    /// 加载所有签到记录
    fn load_all(&self) -> Result<Vec<CheckinRecord>> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| RecordError::ReadError(e.to_string()))?;

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let records: Vec<CheckinRecord> =
            serde_json::from_str(&content).map_err(|e| RecordError::ParseError(e.to_string()))?;

        Ok(records)
    }

    /// 获取所有签到记录（原始数据）
    pub fn get_all_raw(&self) -> Result<Vec<CheckinRecord>> {
        self.load_all()
    }

    /// 保存所有签到记录
    fn save_all(&self, records: &[CheckinRecord]) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                RecordError::WriteError(format!("Failed to create directory: {}", e))
            })?;
        }

        // 序列化为 JSON
        let content = serde_json::to_string_pretty(records)
            .map_err(|e| RecordError::WriteError(format!("Failed to serialize: {}", e)))?;

        // 原子写入
        let temp_dir = self.file_path.parent().unwrap_or(std::path::Path::new("."));
        let mut temp_file = NamedTempFile::new_in(temp_dir)
            .map_err(|e| RecordError::WriteError(format!("Failed to create temp file: {}", e)))?;

        temp_file
            .write_all(content.as_bytes())
            .map_err(|e| RecordError::WriteError(format!("Failed to write temp file: {}", e)))?;

        temp_file
            .persist(&self.file_path)
            .map_err(|e| RecordError::WriteError(format!("Failed to persist file: {}", e)))?;

        tracing::debug!(
            "Saved {} checkin records to {:?}",
            records.len(),
            self.file_path
        );
        Ok(())
    }

    /// 添加签到记录 (自动清理旧数据)
    pub fn add(&self, record: CheckinRecord) -> Result<CheckinRecord> {
        let mut records = self.load_all()?;

        // 添加新记录
        records.push(record.clone());

        // 清理超过 90 天的旧数据
        let cutoff = Utc::now() - Duration::days(RETENTION_DAYS);
        let before_cleanup = records.len();
        records.retain(|r| r.checked_in_at > cutoff);
        let cleaned = before_cleanup - records.len();

        if cleaned > 0 {
            tracing::info!(
                "Cleaned up {} checkin records older than {} days",
                cleaned,
                RETENTION_DAYS
            );
        }

        self.save_all(&records)?;
        tracing::debug!("Added checkin record for account: {}", record.account_id);

        Ok(record)
    }

    /// 获取账号今日签到记录
    #[allow(dead_code)]
    pub fn get_today_record(&self, account_id: &str) -> Result<Option<CheckinRecord>> {
        let records = self.load_all()?;
        let today = Utc::now().date_naive();

        Ok(records
            .into_iter()
            .filter(|r| r.account_id == account_id)
            .filter(|r| r.checked_in_at.date_naive() == today)
            .max_by_key(|r| r.checked_in_at))
    }

    /// 检查账号今日是否已成功签到
    pub fn has_checked_in_today(&self, account_id: &str) -> Result<bool> {
        let records = self.load_all()?;
        let today = Utc::now().date_naive();

        Ok(records
            .iter()
            .filter(|r| r.account_id == account_id)
            .filter(|r| r.checked_in_at.date_naive() == today)
            .any(|r| {
                r.status == CheckinStatus::Success || r.status == CheckinStatus::AlreadyCheckedIn
            }))
    }

    /// 获取账号的签到记录
    #[allow(dead_code)]
    pub fn get_by_account(
        &self,
        account_id: &str,
        limit: Option<usize>,
    ) -> Result<CheckinRecordsResponse> {
        let records = self.load_all()?;

        let mut account_records: Vec<_> = records
            .into_iter()
            .filter(|r| r.account_id == account_id)
            .collect();

        // 按时间倒序排序
        account_records.sort_by(|a, b| b.checked_in_at.cmp(&a.checked_in_at));

        // 限制数量
        if let Some(limit) = limit {
            account_records.truncate(limit);
        }

        let total = account_records.len();
        let record_infos: Vec<CheckinRecordInfo> =
            account_records.into_iter().map(|r| r.into()).collect();

        Ok(CheckinRecordsResponse {
            records: record_infos,
            total,
        })
    }

    /// 获取所有签到记录
    #[allow(dead_code)]
    pub fn get_all(&self, limit: Option<usize>) -> Result<CheckinRecordsResponse> {
        let mut records = self.load_all()?;

        // 按时间倒序排序
        records.sort_by(|a, b| b.checked_in_at.cmp(&a.checked_in_at));

        // 限制数量
        if let Some(limit) = limit {
            records.truncate(limit);
        }

        let total = records.len();
        let record_infos: Vec<CheckinRecordInfo> = records.into_iter().map(|r| r.into()).collect();

        Ok(CheckinRecordsResponse {
            records: record_infos,
            total,
        })
    }

    /// 获取今日签到统计
    pub fn get_today_stats(&self, account_ids: &[String]) -> Result<TodayStats> {
        let records = self.load_all()?;
        let today = Utc::now().date_naive();

        let today_records: Vec<_> = records
            .into_iter()
            .filter(|r| r.checked_in_at.date_naive() == today)
            .collect();

        let mut checked_in = 0;
        let mut failed = 0;

        for account_id in account_ids {
            let account_today: Vec<_> = today_records
                .iter()
                .filter(|r| &r.account_id == account_id)
                .collect();

            if account_today.iter().any(|r| {
                r.status == CheckinStatus::Success || r.status == CheckinStatus::AlreadyCheckedIn
            }) {
                checked_in += 1;
            } else if account_today
                .iter()
                .any(|r| r.status == CheckinStatus::Failed)
            {
                failed += 1;
            }
        }

        Ok(TodayStats {
            total: account_ids.len(),
            checked_in,
            not_checked_in: account_ids.len() - checked_in - failed,
            failed,
        })
    }

    /// 删除账号的所有签到记录
    pub fn delete_by_account(&self, account_id: &str) -> Result<usize> {
        let mut records = self.load_all()?;
        let before = records.len();

        records.retain(|r| r.account_id != account_id);

        let deleted = before - records.len();
        if deleted > 0 {
            self.save_all(&records)?;
            tracing::info!(
                "Deleted {} checkin records for account: {}",
                deleted,
                account_id
            );
        }

        Ok(deleted)
    }

    /// 手动触发数据清理
    #[allow(dead_code)]
    pub fn cleanup_old_data(&self) -> Result<usize> {
        let mut records = self.load_all()?;
        let cutoff = Utc::now() - Duration::days(RETENTION_DAYS);
        let before = records.len();

        records.retain(|r| r.checked_in_at > cutoff);

        let cleaned = before - records.len();
        if cleaned > 0 {
            self.save_all(&records)?;
            tracing::info!("Manual cleanup: removed {} old checkin records", cleaned);
        }

        Ok(cleaned)
    }
}

/// 今日签到统计
#[derive(Debug, Clone)]
pub struct TodayStats {
    /// 总账号数
    pub total: usize,
    /// 今日已签到数
    pub checked_in: usize,
    /// 今日未签到数
    pub not_checked_in: usize,
    /// 今日签到失败数
    pub failed: usize,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, RecordManager) {
        let temp_dir = TempDir::new().unwrap();
        let manager = RecordManager::new(temp_dir.path());
        (temp_dir, manager)
    }

    #[test]
    fn test_add_and_get_today_record() {
        let (_temp_dir, manager) = setup();

        let record = CheckinRecord::success(
            "account-1".to_string(),
            Some("签到成功".to_string()),
            Some("+10 积分".to_string()),
        );

        manager.add(record).unwrap();

        let today_record = manager.get_today_record("account-1").unwrap();
        assert!(today_record.is_some());
        assert_eq!(today_record.unwrap().status, CheckinStatus::Success);
    }

    #[test]
    fn test_has_checked_in_today() {
        let (_temp_dir, manager) = setup();

        // 初始未签到
        assert!(!manager.has_checked_in_today("account-1").unwrap());

        // 添加成功签到记录
        manager
            .add(CheckinRecord::success("account-1".to_string(), None, None))
            .unwrap();

        assert!(manager.has_checked_in_today("account-1").unwrap());
    }

    #[test]
    fn test_get_by_account() {
        let (_temp_dir, manager) = setup();

        // 添加多条记录
        manager
            .add(CheckinRecord::success("account-1".to_string(), None, None))
            .unwrap();
        manager
            .add(CheckinRecord::failed(
                "account-1".to_string(),
                "Error".to_string(),
            ))
            .unwrap();
        manager
            .add(CheckinRecord::success("account-2".to_string(), None, None))
            .unwrap();

        let response = manager.get_by_account("account-1", None).unwrap();
        assert_eq!(response.total, 2);
    }

    #[test]
    fn test_get_all_with_limit() {
        let (_temp_dir, manager) = setup();

        for i in 0..10 {
            manager
                .add(CheckinRecord::success(format!("account-{}", i), None, None))
                .unwrap();
        }

        let response = manager.get_all(Some(5)).unwrap();
        assert_eq!(response.records.len(), 5);
    }

    #[test]
    fn test_today_stats() {
        let (_temp_dir, manager) = setup();

        manager
            .add(CheckinRecord::success("account-1".to_string(), None, None))
            .unwrap();
        manager
            .add(CheckinRecord::already_checked_in(
                "account-2".to_string(),
                None,
            ))
            .unwrap();
        manager
            .add(CheckinRecord::failed(
                "account-3".to_string(),
                "Error".to_string(),
            ))
            .unwrap();

        let account_ids: Vec<String> = (1..=4).map(|i| format!("account-{}", i)).collect();
        let stats = manager.get_today_stats(&account_ids).unwrap();

        assert_eq!(stats.total, 4);
        assert_eq!(stats.checked_in, 2); // success + already_checked_in
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.not_checked_in, 1); // account-4 未签到
    }
}
