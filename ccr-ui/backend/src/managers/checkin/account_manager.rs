// 签到账号管理器
// 负责账号的 CRUD 操作，包括 Cookies JSON 加密存储
// 使用 SQLite 统一存储（替代 JSON 文件）

use crate::core::crypto::CryptoManager;
use crate::database::{self, DatabaseError, repositories::checkin_repo};
use crate::models::checkin::{
    AccountInfo, AccountsResponse, CheckinAccount, CreateAccountRequest, UpdateAccountRequest,
    mask_cookies_json,
};
use chrono::Utc;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AccountError {
    #[error("Account not found: {0}")]
    NotFound(String),
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("Encryption error: {0}")]
    CryptoError(String),
}

pub type Result<T> = std::result::Result<T, AccountError>;

/// 账号管理器
/// 使用 SQLite 统一存储
pub struct AccountManager {
    /// 签到数据目录（用于加密密钥）
    checkin_dir: PathBuf,
}

impl AccountManager {
    /// 创建新的账号管理器
    /// 注意：checkin_dir 仅用于加密密钥存储
    pub fn new(checkin_dir: &Path) -> Self {
        Self {
            checkin_dir: checkin_dir.to_path_buf(),
        }
    }

    /// 获取加密管理器
    fn get_crypto(&self) -> Result<CryptoManager> {
        CryptoManager::new(&self.checkin_dir).map_err(|e| AccountError::CryptoError(e.to_string()))
    }

    /// 获取所有账号 (API Key 已遮罩)
    pub fn list(&self) -> Result<AccountsResponse> {
        let accounts = database::with_connection(checkin_repo::get_all_accounts)?;
        let crypto = self.get_crypto()?;

        let account_infos: Vec<AccountInfo> = accounts
            .iter()
            .map(|a| self.to_account_info(a, &crypto))
            .collect();

        let total = account_infos.len();
        Ok(AccountsResponse {
            accounts: account_infos,
            total,
        })
    }

    /// 根据提供商 ID 获取账号列表
    pub fn list_by_provider(&self, provider_id: &str) -> Result<Vec<AccountInfo>> {
        let accounts = database::with_connection(|conn| {
            checkin_repo::get_accounts_by_provider(conn, provider_id)
        })?;
        let crypto = self.get_crypto()?;

        Ok(accounts
            .iter()
            .map(|a| self.to_account_info(a, &crypto))
            .collect())
    }

    /// 检查提供商是否有关联账号
    pub fn has_accounts_for_provider(&self, provider_id: &str) -> Result<bool> {
        let accounts = database::with_connection(|conn| {
            checkin_repo::get_accounts_by_provider(conn, provider_id)
        })?;
        Ok(!accounts.is_empty())
    }

    /// 转换为账号信息 (遮罩 Cookies)
    fn to_account_info(&self, account: &CheckinAccount, crypto: &CryptoManager) -> AccountInfo {
        // 尝试解密 Cookies JSON 以生成遮罩
        let cookies_masked = match crypto.decrypt(&account.cookies_json_encrypted) {
            Ok(plaintext) => mask_cookies_json(&plaintext),
            Err(_) => "****".to_string(), // 解密失败时显示占位符
        };

        AccountInfo {
            id: account.id.clone(),
            provider_id: account.provider_id.clone(),
            provider_name: None, // 由 Service 层填充
            name: account.name.clone(),
            cookies_masked,
            api_user: account.api_user.clone(),
            enabled: account.enabled,
            created_at: account.created_at,
            last_checkin_at: account.last_checkin_at,
            last_balance_check_at: account.last_balance_check_at,
            latest_balance: None, // 由 Service 层填充
            balance_currency: None,
            total_quota: None,    // 由 Service 层填充
            total_consumed: None, // 由 Service 层填充
        }
    }

    /// 根据 ID 获取账号
    pub fn get(&self, id: &str) -> Result<CheckinAccount> {
        database::with_connection(|conn| checkin_repo::get_account_by_id(conn, id))?
            .ok_or_else(|| AccountError::NotFound(id.to_string()))
    }

    /// 根据 ID 获取账号信息 (遮罩 Cookies)
    pub fn get_info(&self, id: &str) -> Result<AccountInfo> {
        let account = self.get(id)?;
        let crypto = self.get_crypto()?;
        Ok(self.to_account_info(&account, &crypto))
    }

    /// 获取解密后的 Cookies JSON 和 API User
    pub fn get_cookies_json(&self, id: &str) -> Result<(String, String)> {
        let account = self.get(id)?;
        let crypto = self.get_crypto()?;
        let cookies_json = crypto
            .decrypt(&account.cookies_json_encrypted)
            .map_err(|e| AccountError::CryptoError(e.to_string()))?;
        Ok((cookies_json, account.api_user.clone()))
    }

    /// 创建账号
    pub fn create(&self, request: CreateAccountRequest) -> Result<CheckinAccount> {
        let crypto = self.get_crypto()?;

        // 加密 Cookies JSON
        let cookies_json_encrypted = crypto
            .encrypt(&request.cookies_json)
            .map_err(|e| AccountError::CryptoError(e.to_string()))?;

        let account = CheckinAccount::new(
            request.provider_id,
            request.name,
            cookies_json_encrypted,
            request.api_user,
        );

        database::with_connection(|conn| checkin_repo::insert_account(conn, &account))?;

        tracing::info!("Created account: {} ({})", account.name, account.id);
        Ok(account)
    }

    /// 更新账号
    pub fn update(&self, id: &str, request: UpdateAccountRequest) -> Result<CheckinAccount> {
        let crypto = self.get_crypto()?;
        let mut account = self.get(id)?;

        if let Some(name) = request.name {
            account.name = name;
        }

        if let Some(cookies_json) = request.cookies_json {
            account.cookies_json_encrypted = crypto
                .encrypt(&cookies_json)
                .map_err(|e| AccountError::CryptoError(e.to_string()))?;
        }

        if let Some(api_user) = request.api_user {
            account.api_user = api_user;
        }

        if let Some(enabled) = request.enabled {
            account.enabled = enabled;
        }

        account.updated_at = Some(Utc::now());

        database::with_connection(|conn| checkin_repo::update_account(conn, &account))?;

        tracing::info!("Updated account: {} ({})", account.name, account.id);
        Ok(account)
    }

    /// 更新账号签到时间
    pub fn update_checkin_time(&self, id: &str) -> Result<()> {
        let mut account = self.get(id)?;
        account.update_checkin_time();
        database::with_connection(|conn| checkin_repo::update_account(conn, &account))?;
        Ok(())
    }

    /// 更新账号余额检查时间
    pub fn update_balance_time(&self, id: &str) -> Result<()> {
        let mut account = self.get(id)?;
        account.update_balance_check_time();
        database::with_connection(|conn| checkin_repo::update_account(conn, &account))?;
        Ok(())
    }

    /// 删除账号
    pub fn delete(&self, id: &str) -> Result<()> {
        let deleted = database::with_connection(|conn| checkin_repo::delete_account(conn, id))?;

        if !deleted {
            return Err(AccountError::NotFound(id.to_string()));
        }

        tracing::info!("Deleted account: {}", id);
        Ok(())
    }

    /// 获取所有启用的账号
    pub fn get_enabled_accounts(&self) -> Result<Vec<CheckinAccount>> {
        let accounts = database::with_connection(checkin_repo::get_enabled_accounts)?;
        Ok(accounts)
    }

    /// 加载所有账号（兼容旧 API）
    pub fn load_all(&self) -> Result<Vec<CheckinAccount>> {
        let accounts = database::with_connection(checkin_repo::get_all_accounts)?;
        Ok(accounts)
    }

    /// 批量导入账号
    pub fn import_batch(
        &self,
        accounts_to_import: Vec<CheckinAccount>,
        overwrite: bool,
    ) -> Result<(usize, usize)> {
        let existing = database::with_connection(checkin_repo::get_all_accounts)?;
        let mut imported = 0;
        let mut skipped = 0;

        for new_account in accounts_to_import {
            let exists = existing.iter().any(|a| a.id == new_account.id);

            if exists {
                if overwrite {
                    database::with_connection(|conn| {
                        checkin_repo::update_account(conn, &new_account)
                    })?;
                    imported += 1;
                } else {
                    skipped += 1;
                }
            } else {
                database::with_connection(|conn| checkin_repo::insert_account(conn, &new_account))?;
                imported += 1;
            }
        }

        Ok((imported, skipped))
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
    use tempfile::TempDir;

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
        conn.execute("DELETE FROM checkin_accounts", []).unwrap();
        f(&conn)
    }

    fn create_test_crypto_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    #[test]
    fn test_create_and_get_account() {
        let temp_dir = create_test_crypto_dir();
        let crypto = CryptoManager::new(&temp_dir.path().to_path_buf()).unwrap();

        with_test_db(|conn| {
            let cookies_encrypted = crypto.encrypt(r#"{"session": "abc123"}"#).unwrap();
            let account = CheckinAccount::new(
                "provider-1".to_string(),
                "Test Account".to_string(),
                cookies_encrypted,
                "12345".to_string(),
            );

            checkin_repo::insert_account(conn, &account).unwrap();

            let fetched = checkin_repo::get_account_by_id(conn, &account.id)
                .unwrap()
                .unwrap();
            assert_eq!(fetched.name, "Test Account");
            assert!(fetched.enabled);
            assert_eq!(fetched.api_user, "12345");

            // 验证可以解密
            let decrypted = crypto.decrypt(&fetched.cookies_json_encrypted).unwrap();
            assert_eq!(decrypted, r#"{"session": "abc123"}"#);
        });
    }

    #[test]
    fn test_list_accounts() {
        with_test_db(|conn| {
            // 初始为空
            let accounts = checkin_repo::get_all_accounts(conn).unwrap();
            assert_eq!(accounts.len(), 0);

            // 创建账号
            let account = CheckinAccount::new(
                "provider-1".to_string(),
                "Account 1".to_string(),
                "encrypted".to_string(),
                "".to_string(),
            );
            checkin_repo::insert_account(conn, &account).unwrap();

            let accounts = checkin_repo::get_all_accounts(conn).unwrap();
            assert_eq!(accounts.len(), 1);
        });
    }

    #[test]
    fn test_update_account() {
        with_test_db(|conn| {
            let mut account = CheckinAccount::new(
                "provider-1".to_string(),
                "Original".to_string(),
                "encrypted".to_string(),
                "11111".to_string(),
            );
            checkin_repo::insert_account(conn, &account).unwrap();

            // Update
            account.name = "Updated".to_string();
            account.enabled = false;
            account.updated_at = Some(Utc::now());

            checkin_repo::update_account(conn, &account).unwrap();

            let fetched = checkin_repo::get_account_by_id(conn, &account.id)
                .unwrap()
                .unwrap();
            assert_eq!(fetched.name, "Updated");
            assert!(!fetched.enabled);
        });
    }

    #[test]
    fn test_delete_account() {
        with_test_db(|conn| {
            let account = CheckinAccount::new(
                "provider-1".to_string(),
                "To Delete".to_string(),
                "encrypted".to_string(),
                "".to_string(),
            );
            checkin_repo::insert_account(conn, &account).unwrap();

            let deleted = checkin_repo::delete_account(conn, &account.id).unwrap();
            assert!(deleted);

            let fetched = checkin_repo::get_account_by_id(conn, &account.id).unwrap();
            assert!(fetched.is_none());
        });
    }

    #[test]
    fn test_get_accounts_by_provider() {
        with_test_db(|conn| {
            let a1 = CheckinAccount::new(
                "provider-1".to_string(),
                "Account".to_string(),
                "encrypted".to_string(),
                "".to_string(),
            );
            checkin_repo::insert_account(conn, &a1).unwrap();

            assert!(
                !checkin_repo::get_accounts_by_provider(conn, "provider-1")
                    .unwrap()
                    .is_empty()
            );
            assert!(
                checkin_repo::get_accounts_by_provider(conn, "provider-2")
                    .unwrap()
                    .is_empty()
            );
        });
    }
}
