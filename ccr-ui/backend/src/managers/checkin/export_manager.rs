// 导入/导出管理器
// 负责配置的导入和导出操作

use crate::core::crypto::CryptoManager;
use crate::managers::checkin::{AccountManager, ProviderManager};
use crate::models::checkin::{
    CheckinAccount, EXPORT_VERSION, ExportAccount, ExportData, ExportOptions,
    ImportConflictStrategy, ImportOptions, ImportPreviewItem, ImportPreviewResponse, ImportResult,
};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum ExportError {
    #[error("Failed to export: {0}")]
    ExportFailed(String),
    #[error("Failed to import: {0}")]
    ImportFailed(String),
    #[error("Version incompatible: {0}")]
    VersionIncompatible(String),
    #[error("Crypto error: {0}")]
    CryptoError(String),
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Account error: {0}")]
    AccountError(String),
}

pub type Result<T> = std::result::Result<T, ExportError>;

/// 导入/导出管理器
pub struct ExportManager {
    /// 签到数据目录
    checkin_dir: PathBuf,
}

impl ExportManager {
    /// 创建新的导入/导出管理器
    pub fn new(checkin_dir: &Path) -> Self {
        Self {
            checkin_dir: checkin_dir.to_path_buf(),
        }
    }

    /// 导出配置
    pub fn export(&self, options: &ExportOptions) -> Result<ExportData> {
        let provider_manager = ProviderManager::new(&self.checkin_dir);
        let account_manager = AccountManager::new(&self.checkin_dir);

        // 获取所有提供商
        let providers = provider_manager
            .load_all()
            .map_err(|e| ExportError::ProviderError(e.to_string()))?;

        // 获取所有账号
        let accounts = if options.providers_only {
            Vec::new()
        } else {
            let raw_accounts = account_manager
                .load_all()
                .map_err(|e| ExportError::AccountError(e.to_string()))?;

            // 转换为导出格式
            self.convert_accounts_for_export(&raw_accounts, options)?
        };

        Ok(ExportData::new(providers, accounts))
    }

    /// 转换账号为导出格式
    fn convert_accounts_for_export(
        &self,
        accounts: &[CheckinAccount],
        options: &ExportOptions,
    ) -> Result<Vec<ExportAccount>> {
        let crypto = CryptoManager::new(&self.checkin_dir)
            .map_err(|e| ExportError::CryptoError(e.to_string()))?;

        let mut export_accounts = Vec::with_capacity(accounts.len());

        for account in accounts {
            let (api_key, encrypted) = if options.include_plaintext_keys {
                // 导出明文 API Key
                let plaintext = crypto
                    .decrypt(&account.api_key_encrypted)
                    .map_err(|e| ExportError::CryptoError(e.to_string()))?;
                (plaintext, false)
            } else {
                // 保持加密状态
                (account.api_key_encrypted.clone(), true)
            };

            export_accounts.push(ExportAccount {
                id: account.id.clone(),
                provider_id: account.provider_id.clone(),
                name: account.name.clone(),
                api_key,
                api_key_encrypted: encrypted,
                enabled: account.enabled,
                created_at: account.created_at,
            });
        }

        Ok(export_accounts)
    }

    /// 预览导入
    pub fn preview_import(&self, data: &ExportData) -> Result<ImportPreviewResponse> {
        let provider_manager = ProviderManager::new(&self.checkin_dir);
        let account_manager = AccountManager::new(&self.checkin_dir);

        // 检查版本兼容性
        let version_compatible = data.version == EXPORT_VERSION;

        // 获取现有数据
        let existing_providers = provider_manager
            .load_all()
            .map_err(|e| ExportError::ProviderError(e.to_string()))?;
        let existing_accounts = account_manager
            .load_all()
            .map_err(|e| ExportError::AccountError(e.to_string()))?;

        let mut items = Vec::new();
        let mut new_providers = 0;
        let mut conflicting_providers = 0;
        let mut new_accounts = 0;
        let mut conflicting_accounts = 0;
        let mut warnings = Vec::new();

        // 检查提供商
        for provider in &data.providers {
            let conflict = existing_providers
                .iter()
                .find(|p| p.id == provider.id || p.name == provider.name);

            if conflict.is_some() {
                conflicting_providers += 1;
                items.push(ImportPreviewItem {
                    item_type: "provider".to_string(),
                    name: provider.name.clone(),
                    id: provider.id.clone(),
                    has_conflict: true,
                    conflict_with: conflict.map(|c| c.name.clone()),
                });
            } else {
                new_providers += 1;
                items.push(ImportPreviewItem {
                    item_type: "provider".to_string(),
                    name: provider.name.clone(),
                    id: provider.id.clone(),
                    has_conflict: false,
                    conflict_with: None,
                });
            }
        }

        // 检查账号
        let keys_encrypted = data.accounts.iter().all(|a| a.api_key_encrypted);

        for account in &data.accounts {
            let conflict = existing_accounts.iter().find(|a| a.id == account.id);

            if conflict.is_some() {
                conflicting_accounts += 1;
                items.push(ImportPreviewItem {
                    item_type: "account".to_string(),
                    name: account.name.clone(),
                    id: account.id.clone(),
                    has_conflict: true,
                    conflict_with: conflict.map(|c| c.name.clone()),
                });
            } else {
                new_accounts += 1;
                items.push(ImportPreviewItem {
                    item_type: "account".to_string(),
                    name: account.name.clone(),
                    id: account.id.clone(),
                    has_conflict: false,
                    conflict_with: None,
                });
            }
        }

        // 添加警告
        if keys_encrypted {
            warnings.push(
                "API Key 已加密。如果在不同设备上导入，可能无法解密，需要重新输入 API Key。"
                    .to_string(),
            );
        }

        if !version_compatible {
            warnings.push(format!(
                "导出版本 ({}) 与当前版本 ({}) 不匹配，可能存在兼容性问题。",
                data.version, EXPORT_VERSION
            ));
        }

        Ok(ImportPreviewResponse {
            version_compatible,
            export_version: data.version.clone(),
            items,
            new_providers,
            conflicting_providers,
            new_accounts,
            conflicting_accounts,
            keys_encrypted,
            warnings,
        })
    }

    /// 执行导入
    pub fn import(&self, data: ExportData, options: &ImportOptions) -> Result<ImportResult> {
        let provider_manager = ProviderManager::new(&self.checkin_dir);
        let account_manager = AccountManager::new(&self.checkin_dir);

        let mut result = ImportResult::success("导入完成");

        // 导入提供商
        if !options.accounts_only {
            let overwrite = options.conflict_strategy == ImportConflictStrategy::Overwrite;
            let (imported, skipped) = provider_manager
                .import_batch(data.providers, overwrite)
                .map_err(|e| ExportError::ProviderError(e.to_string()))?;

            result.providers_imported = imported;
            result.providers_skipped = skipped;
        }

        // 导入账号
        if !options.providers_only {
            let crypto = CryptoManager::new(&self.checkin_dir)
                .map_err(|e| ExportError::CryptoError(e.to_string()))?;

            let mut accounts_to_import = Vec::new();
            let mut need_reauth = 0;

            for export_account in data.accounts {
                let api_key_encrypted = if export_account.api_key_encrypted {
                    // 尝试解密并重新加密（验证密钥是否匹配）
                    match crypto.decrypt(&export_account.api_key) {
                        Ok(plaintext) => {
                            // 可以解密，重新加密
                            crypto
                                .encrypt(&plaintext)
                                .map_err(|e| ExportError::CryptoError(e.to_string()))?
                        }
                        Err(_) => {
                            // 无法解密，可能是不同设备
                            need_reauth += 1;
                            // 使用空字符串作为占位符，标记需要重新输入
                            crypto
                                .encrypt("")
                                .map_err(|e| ExportError::CryptoError(e.to_string()))?
                        }
                    }
                } else {
                    // 明文 API Key，直接加密
                    crypto
                        .encrypt(&export_account.api_key)
                        .map_err(|e| ExportError::CryptoError(e.to_string()))?
                };

                accounts_to_import.push(CheckinAccount {
                    id: export_account.id,
                    provider_id: export_account.provider_id,
                    name: export_account.name,
                    api_key_encrypted,
                    enabled: export_account.enabled,
                    created_at: export_account.created_at,
                    updated_at: None,
                    last_checkin_at: None,
                    last_balance_check_at: None,
                });
            }

            let overwrite = options.conflict_strategy == ImportConflictStrategy::Overwrite;
            let (imported, skipped) = account_manager
                .import_batch(accounts_to_import, overwrite)
                .map_err(|e| ExportError::AccountError(e.to_string()))?;

            result.accounts_imported = imported;
            result.accounts_skipped = skipped;
            result.accounts_need_reauth = need_reauth;

            if need_reauth > 0 {
                result.add_warning(format!(
                    "{} 个账号的 API Key 无法解密，需要重新输入。",
                    need_reauth
                ));
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::checkin::{CheckinProvider, CreateProviderRequest};
    use tempfile::TempDir;

    fn setup() -> (TempDir, ExportManager) {
        let temp_dir = TempDir::new().unwrap();
        let manager = ExportManager::new(temp_dir.path());
        (temp_dir, manager)
    }

    #[test]
    fn test_export_empty() {
        let (_temp_dir, manager) = setup();

        let options = ExportOptions::default();
        let export_data = manager.export(&options).unwrap();

        assert_eq!(export_data.version, EXPORT_VERSION);
        assert!(export_data.providers.is_empty());
        assert!(export_data.accounts.is_empty());
    }

    #[test]
    fn test_export_with_providers() {
        let (temp_dir, manager) = setup();

        // 创建提供商
        let provider_manager = ProviderManager::new(temp_dir.path());
        provider_manager
            .create(CreateProviderRequest {
                name: "Test Provider".to_string(),
                base_url: "https://api.example.com".to_string(),
                checkin_path: None,
                balance_path: None,
                user_info_path: None,
                auth_header: None,
                auth_prefix: None,
            })
            .unwrap();

        let options = ExportOptions::default();
        let export_data = manager.export(&options).unwrap();

        assert_eq!(export_data.providers.len(), 1);
        assert_eq!(export_data.providers[0].name, "Test Provider");
    }

    #[test]
    fn test_preview_import() {
        let (_temp_dir, manager) = setup();

        let export_data = ExportData::new(
            vec![CheckinProvider::new(
                "New Provider".to_string(),
                "https://api.example.com".to_string(),
            )],
            vec![],
        );

        let preview = manager.preview_import(&export_data).unwrap();

        assert!(preview.version_compatible);
        assert_eq!(preview.new_providers, 1);
        assert_eq!(preview.conflicting_providers, 0);
    }

    #[test]
    fn test_import_providers() {
        let (_temp_dir, manager) = setup();

        let export_data = ExportData::new(
            vec![CheckinProvider::new(
                "Imported Provider".to_string(),
                "https://api.example.com".to_string(),
            )],
            vec![],
        );

        let options = ImportOptions::default();
        let result = manager.import(export_data, &options).unwrap();

        assert!(result.success);
        assert_eq!(result.providers_imported, 1);
    }
}
