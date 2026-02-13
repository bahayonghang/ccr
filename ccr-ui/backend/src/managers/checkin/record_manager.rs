// 签到记录管理器
// 负责签到记录的存储和历史记录管理
// 使用 SQLite 统一存储（替代 JSON 文件）

use crate::database::{self, DatabaseError, repositories::checkin_repo};
use crate::models::checkin::{
    CheckinRecord, CheckinRecordInfo, CheckinRecordsResponse, CheckinStatus,
};
use chrono::Utc;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum RecordError {
    #[error("Record not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
}

pub type Result<T> = std::result::Result<T, RecordError>;

/// 签到记录管理器
/// 使用 SQLite 统一存储
pub struct RecordManager;

impl RecordManager {
    /// 创建新的签到记录管理器
    /// 注意：不再需要 checkin_dir 参数，使用全局 SQLite 数据库
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    /// 添加签到记录
    pub fn add(&self, record: CheckinRecord) -> Result<CheckinRecord> {
        database::with_connection(|conn| checkin_repo::insert_record(conn, &record))?;
        tracing::debug!("Added checkin record for account: {}", record.account_id);
        Ok(record)
    }

    /// 获取所有签到记录（原始数据）
    #[allow(dead_code)]
    pub fn get_all_raw(&self) -> Result<Vec<CheckinRecord>> {
        // 获取最近 1000 条记录
        let records = database::with_connection(|conn| checkin_repo::get_all_records(conn, 1000))?;
        Ok(records)
    }

    /// 获取分页签到记录（SQL 级分页）
    /// 返回 (records, total_count)
    #[allow(dead_code)]
    pub fn get_paginated(
        &self,
        status: Option<&str>,
        account_id: Option<&str>,
        page: usize,
        page_size: usize,
    ) -> Result<(Vec<CheckinRecord>, usize)> {
        let (records, total) = database::with_connection(|conn| {
            checkin_repo::get_records_paginated(conn, status, account_id, page, page_size)
        })?;
        Ok((records, total))
    }

    /// 获取分页签到记录（高级 SQL 过滤）
    /// 支持 status/account_id/provider_id/keyword 组合过滤
    pub fn get_paginated_advanced(
        &self,
        status: Option<&str>,
        account_id: Option<&str>,
        provider_id: Option<&str>,
        keyword: Option<&str>,
        page: usize,
        page_size: usize,
    ) -> Result<(Vec<CheckinRecord>, usize)> {
        let (records, total) = database::with_connection(|conn| {
            checkin_repo::get_records_paginated_advanced(
                conn,
                status,
                account_id,
                provider_id,
                keyword,
                page,
                page_size,
            )
        })?;
        Ok((records, total))
    }

    /// 获取全部签到记录（高级 SQL 过滤，无分页）
    pub fn get_filtered_advanced(
        &self,
        status: Option<&str>,
        account_id: Option<&str>,
        provider_id: Option<&str>,
        keyword: Option<&str>,
    ) -> Result<Vec<CheckinRecord>> {
        let records = database::with_connection(|conn| {
            checkin_repo::get_records_filtered_advanced(
                conn,
                status,
                account_id,
                provider_id,
                keyword,
            )
        })?;
        Ok(records)
    }

    /// 获取账号今日签到记录
    #[allow(dead_code)]
    pub fn get_today_record(&self, account_id: &str) -> Result<Option<CheckinRecord>> {
        let records =
            database::with_connection(|conn| checkin_repo::get_today_records(conn, account_id))?;
        // 返回最新的一条
        Ok(records.into_iter().next())
    }

    /// 检查账号今日是否已成功签到
    pub fn has_checked_in_today(&self, account_id: &str) -> Result<bool> {
        let records =
            database::with_connection(|conn| checkin_repo::get_today_records(conn, account_id))?;

        Ok(records.iter().any(|r| {
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
        let limit = limit.unwrap_or(100);
        let records = database::with_connection(|conn| {
            checkin_repo::get_records_by_account(conn, account_id, limit)
        })?;

        let total = records.len();
        let record_infos: Vec<CheckinRecordInfo> = records.into_iter().map(|r| r.into()).collect();

        Ok(CheckinRecordsResponse {
            records: record_infos,
            total,
        })
    }

    /// 获取所有签到记录
    #[allow(dead_code)]
    pub fn get_all(&self, limit: Option<usize>) -> Result<CheckinRecordsResponse> {
        let limit = limit.unwrap_or(100);
        let records = database::with_connection(|conn| checkin_repo::get_all_records(conn, limit))?;

        let total = records.len();
        let record_infos: Vec<CheckinRecordInfo> = records.into_iter().map(|r| r.into()).collect();

        Ok(CheckinRecordsResponse {
            records: record_infos,
            total,
        })
    }

    /// 获取今日签到统计
    pub fn get_today_stats(&self, account_ids: &[String]) -> Result<TodayStats> {
        let (checked_in, failed) = database::with_connection(|conn| {
            checkin_repo::get_today_status_counts(conn, account_ids)
        })?;

        Ok(TodayStats {
            total: account_ids.len(),
            checked_in,
            not_checked_in: account_ids.len() - checked_in - failed,
            failed,
        })
    }

    /// 删除账号的所有签到记录
    pub fn delete_by_account(&self, account_id: &str) -> Result<usize> {
        let deleted = database::with_connection(|conn| {
            checkin_repo::delete_records_by_account(conn, account_id)
        })?;

        if deleted > 0 {
            tracing::info!(
                "Deleted {} checkin records for account: {}",
                deleted,
                account_id
            );
        }

        Ok(deleted)
    }

    /// 手动触发数据清理（90 天前的记录）
    /// 注意：SQLite 版本可以在后台任务中定期执行清理
    #[allow(dead_code)]
    pub fn cleanup_old_data(&self) -> Result<usize> {
        use chrono::Duration;

        // 计算 90 天前的时间
        let cutoff = Utc::now() - Duration::days(90);
        let cutoff_str = cutoff.to_rfc3339();

        let deleted = database::with_connection(|conn| {
            conn.execute(
                "DELETE FROM checkin_records WHERE checked_in_at < ?1",
                rusqlite::params![cutoff_str],
            )
        })?;

        if deleted > 0 {
            tracing::info!("Manual cleanup: removed {} old checkin records", deleted);
        }

        Ok(deleted)
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
    use crate::database::schema::CREATE_TABLES_SQL;
    use once_cell::sync::Lazy;
    use rusqlite::Connection;
    use std::sync::Mutex;

    // Use a single in-memory database for tests
    static TEST_DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(CREATE_TABLES_SQL).unwrap();
        Mutex::new(conn)
    });

    fn with_test_db<F, R>(f: F) -> R
    where
        F: FnOnce(&Connection) -> R,
    {
        let conn = TEST_DB.lock().unwrap();
        // Clean up before each test
        conn.execute("DELETE FROM checkin_records", []).unwrap();
        f(&conn)
    }

    #[test]
    fn test_add_and_get_today_record() {
        with_test_db(|conn| {
            let record = CheckinRecord::success(
                "account-1".to_string(),
                Some("签到成功".to_string()),
                Some("+10 积分".to_string()),
            );

            checkin_repo::insert_record(conn, &record).unwrap();

            let records = checkin_repo::get_today_records(conn, "account-1").unwrap();
            assert!(!records.is_empty());
            assert_eq!(records[0].status, CheckinStatus::Success);
        });
    }

    #[test]
    fn test_get_by_account() {
        with_test_db(|conn| {
            // 添加多条记录
            let r1 = CheckinRecord::success("account-1".to_string(), None, None);
            let r2 = CheckinRecord::failed("account-1".to_string(), "Error".to_string());
            let r3 = CheckinRecord::success("account-2".to_string(), None, None);

            checkin_repo::insert_record(conn, &r1).unwrap();
            checkin_repo::insert_record(conn, &r2).unwrap();
            checkin_repo::insert_record(conn, &r3).unwrap();

            let records = checkin_repo::get_records_by_account(conn, "account-1", 100).unwrap();
            assert_eq!(records.len(), 2);
        });
    }

    #[test]
    fn test_get_all_with_limit() {
        with_test_db(|conn| {
            for i in 0..10 {
                let r = CheckinRecord::success(format!("account-{}", i), None, None);
                checkin_repo::insert_record(conn, &r).unwrap();
            }

            let records = checkin_repo::get_all_records(conn, 5).unwrap();
            assert_eq!(records.len(), 5);
        });
    }

    #[test]
    fn test_delete_by_account() {
        with_test_db(|conn| {
            let r1 = CheckinRecord::success("account-1".to_string(), None, None);
            let r2 = CheckinRecord::success("account-2".to_string(), None, None);

            checkin_repo::insert_record(conn, &r1).unwrap();
            checkin_repo::insert_record(conn, &r2).unwrap();

            let deleted = checkin_repo::delete_records_by_account(conn, "account-1").unwrap();
            assert_eq!(deleted, 1);

            let records = checkin_repo::get_all_records(conn, 100).unwrap();
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].account_id, "account-2");
        });
    }
}
