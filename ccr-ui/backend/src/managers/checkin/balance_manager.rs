// 余额快照管理器
// 负责余额快照的存储和历史记录管理，包括 90 天数据清理
// 使用 SQLite 统一存储（替代 JSON 文件）

use crate::database::{self, DatabaseError, repositories::checkin_repo};
use crate::models::checkin::{BalanceHistoryItem, BalanceHistoryResponse, BalanceSnapshot};
use std::collections::HashMap;

const RETENTION_DAYS: i64 = 90;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum BalanceError {
    #[error("Balance not found for account: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
}

pub type Result<T> = std::result::Result<T, BalanceError>;

/// 余额管理器
/// 使用 SQLite 统一存储
pub struct BalanceManager;

impl BalanceManager {
    /// 创建新的余额管理器
    /// 注意：不再需要 checkin_dir 参数，使用全局 SQLite 数据库
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    /// 添加余额快照 (自动清理旧数据)
    pub fn add(&self, snapshot: BalanceSnapshot) -> Result<BalanceSnapshot> {
        database::with_connection(|conn| checkin_repo::insert_balance(conn, &snapshot))?;

        // 清理超过 90 天的旧数据
        let cleaned = database::with_connection(|conn| {
            checkin_repo::delete_old_balances(conn, RETENTION_DAYS)
        })?;

        if cleaned > 0 {
            tracing::info!(
                "Cleaned up {} balance snapshots older than {} days",
                cleaned,
                RETENTION_DAYS
            );
        }

        tracing::debug!(
            "Added balance snapshot for account: {}",
            snapshot.account_id
        );

        Ok(snapshot)
    }

    /// 获取账号最新余额
    #[allow(dead_code)]
    pub fn get_latest(&self, account_id: &str) -> Result<Option<BalanceSnapshot>> {
        let balance =
            database::with_connection(|conn| checkin_repo::get_latest_balance(conn, account_id))?;
        Ok(balance)
    }

    /// 获取所有账号的最新余额快照映射
    pub fn get_latest_map(&self) -> Result<HashMap<String, BalanceSnapshot>> {
        let balances = database::with_connection(checkin_repo::get_latest_balances_for_all)?;

        let latest_map: HashMap<String, BalanceSnapshot> = balances
            .into_iter()
            .map(|b| (b.account_id.clone(), b))
            .collect();

        Ok(latest_map)
    }

    /// 获取账号所有余额快照（按时间升序）
    pub fn list_by_account(&self, account_id: &str) -> Result<Vec<BalanceSnapshot>> {
        let mut balances = database::with_connection(|conn| {
            checkin_repo::get_balance_history(conn, account_id, 1000)
        })?;

        // Sort ascending by time (repo returns DESC)
        balances.sort_by(|a, b| a.recorded_at.cmp(&b.recorded_at));
        Ok(balances)
    }

    /// 获取账号余额历史
    pub fn get_history(
        &self,
        account_id: &str,
        limit: Option<usize>,
    ) -> Result<BalanceHistoryResponse> {
        let limit = limit.unwrap_or(1000);
        let account_balances = database::with_connection(|conn| {
            checkin_repo::get_balance_history(conn, account_id, limit)
        })?;

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
        let deleted = database::with_connection(|conn| {
            checkin_repo::delete_balances_by_account(conn, account_id)
        })?;

        if deleted > 0 {
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
        let cleaned = database::with_connection(|conn| {
            checkin_repo::delete_old_balances(conn, RETENTION_DAYS)
        })?;

        if cleaned > 0 {
            tracing::info!("Manual cleanup: removed {} old balance snapshots", cleaned);
        }

        Ok(cleaned)
    }
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
        conn.execute("DELETE FROM checkin_balances", []).unwrap();
        f(&conn)
    }

    #[test]
    fn test_add_and_get_latest() {
        with_test_db(|conn| {
            let snapshot1 = BalanceSnapshot::new(
                "account-1".to_string(),
                100.0,
                30.0,
                70.0,
                "USD".to_string(),
            );
            checkin_repo::insert_balance(conn, &snapshot1).unwrap();

            // 稍后添加第二个快照
            let snapshot2 = BalanceSnapshot::new(
                "account-1".to_string(),
                100.0,
                40.0,
                60.0,
                "USD".to_string(),
            );
            checkin_repo::insert_balance(conn, &snapshot2).unwrap();

            let latest = checkin_repo::get_latest_balance(conn, "account-1")
                .unwrap()
                .unwrap();
            assert_eq!(latest.remaining_quota, 60.0);
        });
    }

    #[test]
    fn test_get_history() {
        with_test_db(|conn| {
            // 添加多个快照
            for i in 0..5 {
                let snapshot = BalanceSnapshot::new(
                    "account-1".to_string(),
                    100.0,
                    (i * 10) as f64,
                    (100 - i * 10) as f64,
                    "USD".to_string(),
                );
                checkin_repo::insert_balance(conn, &snapshot).unwrap();
            }

            let history = checkin_repo::get_balance_history(conn, "account-1", 100).unwrap();
            assert_eq!(history.len(), 5);

            // 验证按时间倒序
            for i in 0..history.len() - 1 {
                assert!(history[i].recorded_at >= history[i + 1].recorded_at);
            }
        });
    }

    #[test]
    fn test_get_history_with_limit() {
        with_test_db(|conn| {
            for i in 0..10 {
                let snapshot = BalanceSnapshot::new(
                    "account-1".to_string(),
                    100.0,
                    (i * 5) as f64,
                    (100 - i * 5) as f64,
                    "USD".to_string(),
                );
                checkin_repo::insert_balance(conn, &snapshot).unwrap();
            }

            let history = checkin_repo::get_balance_history(conn, "account-1", 3).unwrap();
            assert_eq!(history.len(), 3);
        });
    }

    #[test]
    fn test_delete_by_account() {
        with_test_db(|conn| {
            // 添加两个账号的快照
            checkin_repo::insert_balance(
                conn,
                &BalanceSnapshot::new(
                    "account-1".to_string(),
                    100.0,
                    30.0,
                    70.0,
                    "USD".to_string(),
                ),
            )
            .unwrap();
            checkin_repo::insert_balance(
                conn,
                &BalanceSnapshot::new(
                    "account-2".to_string(),
                    200.0,
                    50.0,
                    150.0,
                    "USD".to_string(),
                ),
            )
            .unwrap();

            // 删除 account-1 的记录
            let deleted = checkin_repo::delete_balances_by_account(conn, "account-1").unwrap();
            assert_eq!(deleted, 1);

            // 验证 account-1 已删除
            assert!(
                checkin_repo::get_latest_balance(conn, "account-1")
                    .unwrap()
                    .is_none()
            );

            // 验证 account-2 仍存在
            assert!(
                checkin_repo::get_latest_balance(conn, "account-2")
                    .unwrap()
                    .is_some()
            );
        });
    }
}
