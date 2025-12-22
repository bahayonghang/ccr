// 签到账号管理器
// 负责账号的 CRUD 操作，包括 API Key 加密存储

use crate::core::crypto::{CryptoManager, mask_api_key};
use crate::models::checkin::{
    AccountInfo, AccountsResponse, CheckinAccount, CreateAccountRequest, UpdateAccountRequest,
};
use chrono::Utc;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

const ACCOUNTS_FILE: &str = "accounts.json";

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AccountError {
    #[error("Account not found: {0}")]
    NotFound(String),
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
    #[error("Failed to read accounts: {0}")]
    ReadError(String),
    #[error("Failed to write accounts: {0}")]
    WriteError(String),
    #[error("Failed to parse accounts: {0}")]
    ParseError(String),
    #[error("Encryption error: {0}")]
    CryptoError(String),
}

pub type Result<T> = std::result::Result<T, AccountError>;

/// 账号管理器
pub struct AccountManager {
    /// 数据文件路径
    file_path: PathBuf,
    /// 签到数据目录
    checkin_dir: PathBuf,
}

impl AccountManager {
    /// 创建新的账号管理器
    pub fn new(checkin_dir: &Path) -> Self {
        Self {
            file_path: checkin_dir.join(ACCOUNTS_FILE),
            checkin_dir: checkin_dir.to_path_buf(),
        }
    }

    /// 获取加密管理器
    fn get_crypto(&self) -> Result<CryptoManager> {
        CryptoManager::new(&self.checkin_dir).map_err(|e| AccountError::CryptoError(e.to_string()))
    }

    /// 加载所有账号
    pub fn load_all(&self) -> Result<Vec<CheckinAccount>> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| AccountError::ReadError(e.to_string()))?;

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let accounts: Vec<CheckinAccount> =
            serde_json::from_str(&content).map_err(|e| AccountError::ParseError(e.to_string()))?;

        Ok(accounts)
    }

    /// 保存所有账号
    fn save_all(&self, accounts: &[CheckinAccount]) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AccountError::WriteError(format!("Failed to create directory: {}", e))
            })?;
        }

        // 序列化为 JSON
        let content = serde_json::to_string_pretty(accounts)
            .map_err(|e| AccountError::WriteError(format!("Failed to serialize: {}", e)))?;

        // 原子写入
        let temp_dir = self.file_path.parent().unwrap_or(std::path::Path::new("."));
        let mut temp_file = NamedTempFile::new_in(temp_dir)
            .map_err(|e| AccountError::WriteError(format!("Failed to create temp file: {}", e)))?;

        temp_file
            .write_all(content.as_bytes())
            .map_err(|e| AccountError::WriteError(format!("Failed to write temp file: {}", e)))?;

        temp_file
            .persist(&self.file_path)
            .map_err(|e| AccountError::WriteError(format!("Failed to persist file: {}", e)))?;

        tracing::debug!("Saved {} accounts to {:?}", accounts.len(), self.file_path);
        Ok(())
    }

    /// 获取所有账号 (API Key 已遮罩)
    pub fn list(&self) -> Result<AccountsResponse> {
        let accounts = self.load_all()?;
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
        let accounts = self.load_all()?;
        let crypto = self.get_crypto()?;

        Ok(accounts
            .iter()
            .filter(|a| a.provider_id == provider_id)
            .map(|a| self.to_account_info(a, &crypto))
            .collect())
    }

    /// 检查提供商是否有关联账号
    pub fn has_accounts_for_provider(&self, provider_id: &str) -> Result<bool> {
        let accounts = self.load_all()?;
        Ok(accounts.iter().any(|a| a.provider_id == provider_id))
    }

    /// 转换为账号信息 (遮罩 API Key)
    fn to_account_info(&self, account: &CheckinAccount, crypto: &CryptoManager) -> AccountInfo {
        // 尝试解密 API Key 以生成遮罩
        let api_key_masked = match crypto.decrypt(&account.api_key_encrypted) {
            Ok(plaintext) => mask_api_key(&plaintext),
            Err(_) => "****".to_string(), // 解密失败时显示占位符
        };

        AccountInfo {
            id: account.id.clone(),
            provider_id: account.provider_id.clone(),
            provider_name: None, // 由 Service 层填充
            name: account.name.clone(),
            api_key_masked,
            enabled: account.enabled,
            created_at: account.created_at,
            last_checkin_at: account.last_checkin_at,
            last_balance_check_at: account.last_balance_check_at,
            latest_balance: None, // 由 Service 层填充
            balance_currency: None,
        }
    }

    /// 根据 ID 获取账号
    pub fn get(&self, id: &str) -> Result<CheckinAccount> {
        let accounts = self.load_all()?;
        accounts
            .into_iter()
            .find(|a| a.id == id)
            .ok_or_else(|| AccountError::NotFound(id.to_string()))
    }

    /// 根据 ID 获取账号信息 (遮罩 API Key)
    pub fn get_info(&self, id: &str) -> Result<AccountInfo> {
        let account = self.get(id)?;
        let crypto = self.get_crypto()?;
        Ok(self.to_account_info(&account, &crypto))
    }

    /// 获取解密后的 API Key
    #[allow(dead_code)]
    pub fn get_api_key(&self, id: &str) -> Result<String> {
        let account = self.get(id)?;
        let crypto = self.get_crypto()?;
        crypto
            .decrypt(&account.api_key_encrypted)
            .map_err(|e| AccountError::CryptoError(e.to_string()))
    }

    /// 创建账号
    pub fn create(&self, request: CreateAccountRequest) -> Result<CheckinAccount> {
        let crypto = self.get_crypto()?;
        let mut accounts = self.load_all()?;

        // 加密 API Key
        let api_key_encrypted = crypto
            .encrypt(&request.api_key)
            .map_err(|e| AccountError::CryptoError(e.to_string()))?;

        let account = CheckinAccount::new(request.provider_id, request.name, api_key_encrypted);

        accounts.push(account.clone());
        self.save_all(&accounts)?;

        tracing::info!("Created account: {} ({})", account.name, account.id);
        Ok(account)
    }

    /// 更新账号
    pub fn update(&self, id: &str, request: UpdateAccountRequest) -> Result<CheckinAccount> {
        let crypto = self.get_crypto()?;
        let mut accounts = self.load_all()?;

        let account = accounts
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| AccountError::NotFound(id.to_string()))?;

        if let Some(name) = request.name {
            account.name = name;
        }

        if let Some(api_key) = request.api_key {
            account.api_key_encrypted = crypto
                .encrypt(&api_key)
                .map_err(|e| AccountError::CryptoError(e.to_string()))?;
        }

        if let Some(enabled) = request.enabled {
            account.enabled = enabled;
        }

        account.updated_at = Some(Utc::now());

        let updated = account.clone();
        self.save_all(&accounts)?;

        tracing::info!("Updated account: {} ({})", updated.name, updated.id);
        Ok(updated)
    }

    /// 更新账号签到时间
    pub fn update_checkin_time(&self, id: &str) -> Result<()> {
        let mut accounts = self.load_all()?;

        let account = accounts
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| AccountError::NotFound(id.to_string()))?;

        account.update_checkin_time();
        self.save_all(&accounts)?;

        Ok(())
    }

    /// 更新账号余额检查时间
    pub fn update_balance_time(&self, id: &str) -> Result<()> {
        let mut accounts = self.load_all()?;

        let account = accounts
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| AccountError::NotFound(id.to_string()))?;

        account.update_balance_check_time();
        self.save_all(&accounts)?;

        Ok(())
    }

    /// 删除账号
    pub fn delete(&self, id: &str) -> Result<()> {
        let mut accounts = self.load_all()?;
        let original_len = accounts.len();

        accounts.retain(|a| a.id != id);

        if accounts.len() == original_len {
            return Err(AccountError::NotFound(id.to_string()));
        }

        self.save_all(&accounts)?;
        tracing::info!("Deleted account: {}", id);
        Ok(())
    }

    /// 获取所有启用的账号
    pub fn get_enabled_accounts(&self) -> Result<Vec<CheckinAccount>> {
        let accounts = self.load_all()?;
        Ok(accounts.into_iter().filter(|a| a.enabled).collect())
    }

    /// 批量导入账号
    pub fn import_batch(
        &self,
        accounts_to_import: Vec<CheckinAccount>,
        overwrite: bool,
    ) -> Result<(usize, usize)> {
        let mut accounts = self.load_all()?;
        let mut imported = 0;
        let mut skipped = 0;

        for new_account in accounts_to_import {
            if let Some(existing) = accounts.iter_mut().find(|a| a.id == new_account.id) {
                if overwrite {
                    *existing = new_account;
                    imported += 1;
                } else {
                    skipped += 1;
                }
            } else {
                accounts.push(new_account);
                imported += 1;
            }
        }

        self.save_all(&accounts)?;
        Ok((imported, skipped))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, AccountManager) {
        let temp_dir = TempDir::new().unwrap();
        let manager = AccountManager::new(temp_dir.path());
        (temp_dir, manager)
    }

    #[test]
    fn test_create_and_get_account() {
        let (_temp_dir, manager) = setup();

        let request = CreateAccountRequest {
            provider_id: "provider-1".to_string(),
            name: "Test Account".to_string(),
            api_key: "sk-1234567890abcdef".to_string(),
        };

        let account = manager.create(request).unwrap();
        assert_eq!(account.name, "Test Account");
        assert!(account.enabled);

        // 验证 API Key 已加密
        assert_ne!(account.api_key_encrypted, "sk-1234567890abcdef");

        // 验证可以解密
        let api_key = manager.get_api_key(&account.id).unwrap();
        assert_eq!(api_key, "sk-1234567890abcdef");
    }

    #[test]
    fn test_list_accounts() {
        let (_temp_dir, manager) = setup();

        // 初始为空
        let response = manager.list().unwrap();
        assert_eq!(response.total, 0);

        // 创建账号
        manager
            .create(CreateAccountRequest {
                provider_id: "provider-1".to_string(),
                name: "Account 1".to_string(),
                api_key: "sk-key1".to_string(),
            })
            .unwrap();

        let response = manager.list().unwrap();
        assert_eq!(response.total, 1);

        // 验证 API Key 已遮罩
        let account_info = &response.accounts[0];
        assert!(!account_info.api_key_masked.contains("sk-key1"));
    }

    #[test]
    fn test_update_account() {
        let (_temp_dir, manager) = setup();

        let account = manager
            .create(CreateAccountRequest {
                provider_id: "provider-1".to_string(),
                name: "Original".to_string(),
                api_key: "sk-original".to_string(),
            })
            .unwrap();

        let updated = manager
            .update(
                &account.id,
                UpdateAccountRequest {
                    name: Some("Updated".to_string()),
                    api_key: Some("sk-updated".to_string()),
                    enabled: Some(false),
                },
            )
            .unwrap();

        assert_eq!(updated.name, "Updated");
        assert!(!updated.enabled);

        // 验证新 API Key
        let api_key = manager.get_api_key(&account.id).unwrap();
        assert_eq!(api_key, "sk-updated");
    }

    #[test]
    fn test_delete_account() {
        let (_temp_dir, manager) = setup();

        let account = manager
            .create(CreateAccountRequest {
                provider_id: "provider-1".to_string(),
                name: "To Delete".to_string(),
                api_key: "sk-delete".to_string(),
            })
            .unwrap();

        manager.delete(&account.id).unwrap();

        assert!(manager.get(&account.id).is_err());
    }

    #[test]
    fn test_has_accounts_for_provider() {
        let (_temp_dir, manager) = setup();

        assert!(!manager.has_accounts_for_provider("provider-1").unwrap());

        manager
            .create(CreateAccountRequest {
                provider_id: "provider-1".to_string(),
                name: "Account".to_string(),
                api_key: "sk-key".to_string(),
            })
            .unwrap();

        assert!(manager.has_accounts_for_provider("provider-1").unwrap());
        assert!(!manager.has_accounts_for_provider("provider-2").unwrap());
    }
}
