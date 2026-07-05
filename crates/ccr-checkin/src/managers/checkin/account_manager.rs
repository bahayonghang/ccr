// 签到账号管理器
// 负责账号的 CRUD 操作，包括 Cookies JSON 加密存储
// 使用 SQLite 统一存储（替代 JSON 文件）

use crate::core::crypto::CryptoManager;
use crate::core::error::DbError;
use crate::database::{self, repositories::checkin_repo};
use crate::models::checkin::{
    AccountInfo, AccountsResponse, CheckinAccount, CreateAccountRequest, UpdateAccountRequest,
};
use ccr_core::Secret;
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
    Database(#[from] DbError),
    #[error("Encryption error: {0}")]
    CryptoError(String),
}

pub type Result<T> = std::result::Result<T, AccountError>;

/// 账号管理器
/// 使用 SQLite 统一存储
pub struct AccountManager {
    /// 签到数据目录（用于加密密钥）
    checkin_dir: PathBuf,
    /// 数据库访问句柄（默认全局池；测试/嵌入场景可注入独立池）
    db: database::DbAccess,
}

impl AccountManager {
    /// 创建新的账号管理器
    /// 注意：checkin_dir 仅用于加密密钥存储
    pub fn new(checkin_dir: &Path) -> Self {
        Self::with_db(checkin_dir, database::DbAccess::Global)
    }

    /// 注入式构造：测试或嵌入场景使用独立连接池
    pub fn with_db(checkin_dir: &Path, db: database::DbAccess) -> Self {
        Self {
            checkin_dir: checkin_dir.to_path_buf(),
            db,
        }
    }

    /// 获取加密管理器
    fn get_crypto(&self) -> Result<CryptoManager> {
        CryptoManager::new(&self.checkin_dir).map_err(|e| AccountError::CryptoError(e.to_string()))
    }

    /// 获取所有账号 (列表路径不解密 Cookies，掩码为占位符)
    pub fn list(&self) -> Result<AccountsResponse> {
        let accounts = self.db.with_connection(checkin_repo::get_all_accounts)?;

        let account_infos: Vec<AccountInfo> = accounts.iter().map(Self::to_account_info).collect();

        let total = account_infos.len();
        Ok(AccountsResponse {
            accounts: account_infos,
            total,
        })
    }

    /// 根据提供商 ID 获取账号列表
    pub fn list_by_provider(&self, provider_id: &str) -> Result<Vec<AccountInfo>> {
        let accounts = self.db.with_connection(|conn| {
            checkin_repo::get_accounts_by_provider(conn, provider_id)
        })?;

        Ok(accounts.iter().map(Self::to_account_info).collect())
    }

    /// 检查提供商是否有关联账号
    pub fn has_accounts_for_provider(&self, provider_id: &str) -> Result<bool> {
        let accounts = self.db.with_connection(|conn| {
            checkin_repo::get_accounts_by_provider(conn, provider_id)
        })?;
        Ok(!accounts.is_empty())
    }

    /// 转换为账号信息（列表路径专用：不解密 Cookies，掩码用占位符；
    /// 真实掩码通过 `get_info` 单账号惰性生成，避免列表逐账号解密）
    fn to_account_info(account: &CheckinAccount) -> AccountInfo {
        Self::build_account_info(account, "****".to_string())
    }

    /// 用给定掩码组装账号信息
    fn build_account_info(account: &CheckinAccount, cookies_masked: String) -> AccountInfo {
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
            extra_config: account.extra_config.clone(),
        }
    }

    /// 根据 ID 获取账号
    pub fn get(&self, id: &str) -> Result<CheckinAccount> {
        self.db.with_connection(|conn| checkin_repo::get_account_by_id(conn, id))?
            .ok_or_else(|| AccountError::NotFound(id.to_string()))
    }

    /// 根据 ID 获取账号信息（单账号惰性路径：解密生成真实掩码）
    pub fn get_info(&self, id: &str) -> Result<AccountInfo> {
        let account = self.get(id)?;
        let crypto = self.get_crypto()?;
        let cookies_masked = match crypto.decrypt(&account.cookies_json_encrypted) {
            Ok(plaintext) => Self::masked_cookies_display(&plaintext),
            Err(_) => "****".to_string(), // 解密失败时显示占位符
        };
        Ok(Self::build_account_info(&account, cookies_masked))
    }

    /// 生成 Cookies 的掩码展示串（如 "session=sess...alue; token=****"）
    ///
    /// 仅做 JSON map 迭代 + 格式化；掩码算法本身由 [`Secret`] 的 Display
    /// 提供（全仓唯一实现 ccr_core::mask_sensitive），此处不携带第二套规则
    fn masked_cookies_display(plaintext: &Secret) -> String {
        match serde_json::from_str::<std::collections::HashMap<String, String>>(plaintext.expose())
        {
            Ok(cookies) => cookies
                .iter()
                .map(|(k, v)| format!("{}={}", k, Secret::from(v.as_str())))
                .collect::<Vec<_>>()
                .join("; "),
            Err(_) => "****".to_string(),
        }
    }

    /// 获取解密后的 Cookies JSON 和 API User
    pub fn get_cookies_json(&self, id: &str) -> Result<(Secret, String)> {
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

        // 加密 Cookies JSON（加密是明文的合法消费点）
        let cookies_json_encrypted = crypto
            .encrypt(request.cookies_json.expose())
            .map_err(|e| AccountError::CryptoError(e.to_string()))?;

        let mut account = CheckinAccount::new(
            request.provider_id,
            request.name,
            cookies_json_encrypted,
            request.api_user,
        );

        // 设置扩展配置 (CDK 凭证等)
        account.extra_config = request.extra_config;

        self.db.with_connection(|conn| checkin_repo::insert_account(conn, &account))?;

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
                .encrypt(cookies_json.expose())
                .map_err(|e| AccountError::CryptoError(e.to_string()))?;
        }

        if let Some(api_user) = request.api_user {
            account.api_user = api_user;
        }

        if let Some(enabled) = request.enabled {
            account.enabled = enabled;
        }

        if let Some(extra_config) = request.extra_config {
            account.extra_config = extra_config;
        }

        account.updated_at = Some(Utc::now());

        self.db.with_connection(|conn| checkin_repo::update_account(conn, &account))?;

        tracing::info!("Updated account: {} ({})", account.name, account.id);
        Ok(account)
    }

    /// 更新账号签到时间
    pub fn update_checkin_time(&self, id: &str) -> Result<()> {
        let mut account = self.get(id)?;
        account.update_checkin_time();
        self.db.with_connection(|conn| checkin_repo::update_account(conn, &account))?;
        Ok(())
    }

    /// 更新账号余额检查时间
    pub fn update_balance_time(&self, id: &str) -> Result<()> {
        let mut account = self.get(id)?;
        account.update_balance_check_time();
        self.db.with_connection(|conn| checkin_repo::update_account(conn, &account))?;
        Ok(())
    }

    /// 删除账号
    pub fn delete(&self, id: &str) -> Result<()> {
        let deleted = self.db.with_connection(|conn| checkin_repo::delete_account(conn, id))?;

        if !deleted {
            return Err(AccountError::NotFound(id.to_string()));
        }

        tracing::info!("Deleted account: {}", id);
        Ok(())
    }

    /// 获取所有启用的账号
    pub fn get_enabled_accounts(&self) -> Result<Vec<CheckinAccount>> {
        let accounts = self.db.with_connection(checkin_repo::get_enabled_accounts)?;
        Ok(accounts)
    }

    /// 加载所有账号（兼容旧 API）
    pub fn load_all(&self) -> Result<Vec<CheckinAccount>> {
        let accounts = self.db.with_connection(checkin_repo::get_all_accounts)?;
        Ok(accounts)
    }

    /// 批量导入账号
    pub fn import_batch(
        &self,
        accounts_to_import: Vec<CheckinAccount>,
        overwrite: bool,
    ) -> Result<(usize, usize)> {
        let existing = self.db.with_connection(checkin_repo::get_all_accounts)?;
        let mut imported = 0;
        let mut skipped = 0;

        for new_account in accounts_to_import {
            let exists = existing.iter().any(|a| a.id == new_account.id);

            if exists {
                if overwrite {
                    self.db.with_connection(|conn| {
                        checkin_repo::update_account(conn, &new_account)
                    })?;
                    imported += 1;
                } else {
                    skipped += 1;
                }
            } else {
                self.db.with_connection(|conn| checkin_repo::insert_account(conn, &new_account))?;
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
    use tempfile::TempDir;

    /// manager 级测试夹具：注入独立内存池（不触 GLOBAL_POOL），
    /// 加密密钥目录用临时目录。每个测试独立数据库，无跨测试清理。
    fn manager_with_memory_db() -> (TempDir, AccountManager) {
        let pool = database::create_memory_pool().unwrap();
        {
            let conn = pool.get().unwrap();
            conn.execute_batch(CREATE_TABLES_SQL).unwrap();
        }
        let temp_dir = TempDir::new().unwrap();
        let manager = AccountManager::with_db(temp_dir.path(), database::DbAccess::Pool(pool));
        (temp_dir, manager)
    }

    fn create_request(name: &str, cookies_json: &str) -> CreateAccountRequest {
        CreateAccountRequest {
            provider_id: "provider-1".to_string(),
            name: name.to_string(),
            cookies_json: Secret::from(cookies_json),
            api_user: "12345".to_string(),
            extra_config: "{}".to_string(),
        }
    }

    #[test]
    fn test_create_and_get_roundtrip_encrypts_cookies() {
        let (_dir, manager) = manager_with_memory_db();

        let created = manager
            .create(create_request("Test Account", r#"{"session": "abc123"}"#))
            .unwrap();

        // 密文入库：存储字段不含明文片段
        assert!(!created.cookies_json_encrypted.contains("abc123"));

        let fetched = manager.get(&created.id).unwrap();
        assert_eq!(fetched.name, "Test Account");
        assert_eq!(fetched.api_user, "12345");
        assert!(fetched.enabled);

        // manager 解密路径可还原明文
        let (cookies, api_user) = manager.get_cookies_json(&created.id).unwrap();
        assert_eq!(cookies.expose(), r#"{"session": "abc123"}"#);
        assert_eq!(api_user, "12345");
    }

    #[test]
    fn test_get_info_masks_cookie_values() {
        let (_dir, manager) = manager_with_memory_db();
        let created = manager
            .create(create_request("Masked", r#"{"session": "abc123"}"#))
            .unwrap();

        let info = manager.get_info(&created.id).unwrap();

        // 惰性解密生成真实掩码：有键名、无明文值
        assert!(info.cookies_masked.contains("session="));
        assert!(!info.cookies_masked.contains("abc123"));
    }

    #[test]
    fn test_list_uses_placeholder_mask() {
        let (_dir, manager) = manager_with_memory_db();
        assert_eq!(manager.list().unwrap().total, 0);

        manager
            .create(create_request("Account 1", r#"{"session": "abc123"}"#))
            .unwrap();

        let response = manager.list().unwrap();
        assert_eq!(response.total, 1);
        // 列表路径不解密：掩码为占位符
        assert_eq!(response.accounts[0].cookies_masked, "****");
    }

    #[test]
    fn test_list_path_conversion_does_not_decrypt_cookies() {
        // 列表路径转换不接收 CryptoManager（签名级保证不解密），
        // 即使 cookies 密文是任意垃圾内容也能产出占位掩码。
        let account = CheckinAccount::new(
            "provider-1".to_string(),
            "Masked Account".to_string(),
            "not-a-valid-ciphertext".to_string(),
            "12345".to_string(),
        );

        let info = AccountManager::to_account_info(&account);

        assert_eq!(info.cookies_masked, "****");
        assert_eq!(info.name, "Masked Account");
        assert_eq!(info.api_user, "12345");
        // 占位掩码不应泄露任何密文片段
        assert!(!info.cookies_masked.contains("not-a-valid"));
    }

    #[test]
    fn test_update_account_reencrypts_cookies() {
        let (_dir, manager) = manager_with_memory_db();
        let created = manager
            .create(create_request("Original", r#"{"session": "abc123"}"#))
            .unwrap();

        let updated = manager
            .update(
                &created.id,
                UpdateAccountRequest {
                    name: Some("Updated".to_string()),
                    cookies_json: Some(Secret::from(r#"{"session": "new456"}"#)),
                    api_user: None,
                    enabled: Some(false),
                    extra_config: None,
                },
            )
            .unwrap();

        assert_eq!(updated.name, "Updated");
        assert!(!updated.enabled);
        assert!(updated.updated_at.is_some());

        // 新 cookies 重加密后仍可解密
        let (cookies, _) = manager.get_cookies_json(&created.id).unwrap();
        assert_eq!(cookies.expose(), r#"{"session": "new456"}"#);
    }

    #[test]
    fn test_delete_account_then_not_found() {
        let (_dir, manager) = manager_with_memory_db();
        let created = manager
            .create(create_request("To Delete", r#"{"k": "v"}"#))
            .unwrap();

        manager.delete(&created.id).unwrap();

        assert!(matches!(
            manager.get(&created.id),
            Err(AccountError::NotFound(_))
        ));
        assert!(matches!(
            manager.delete(&created.id),
            Err(AccountError::NotFound(_))
        ));
    }

    #[test]
    fn test_list_by_provider_scopes_accounts() {
        let (_dir, manager) = manager_with_memory_db();
        manager
            .create(create_request("Account", r#"{"k": "v"}"#))
            .unwrap();

        assert_eq!(manager.list_by_provider("provider-1").unwrap().len(), 1);
        assert!(manager.list_by_provider("provider-2").unwrap().is_empty());
        assert!(manager.has_accounts_for_provider("provider-1").unwrap());
        assert!(!manager.has_accounts_for_provider("provider-2").unwrap());
    }
}
