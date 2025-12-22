// 中转站提供商管理器
// 负责提供商配置的 CRUD 操作

use crate::models::checkin::{
    CheckinProvider, CreateProviderRequest, ProvidersResponse, UpdateProviderRequest,
};
use chrono::Utc;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

const PROVIDERS_FILE: &str = "providers.json";

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Provider not found: {0}")]
    NotFound(String),
    #[error("Provider already exists: {0}")]
    AlreadyExists(String),
    #[error("Failed to read providers: {0}")]
    ReadError(String),
    #[error("Failed to write providers: {0}")]
    WriteError(String),
    #[error("Failed to parse providers: {0}")]
    ParseError(String),
    #[error("Cannot delete provider with associated accounts: {0}")]
    HasAccounts(String),
}

pub type Result<T> = std::result::Result<T, ProviderError>;

/// 提供商管理器
pub struct ProviderManager {
    /// 数据文件路径
    file_path: PathBuf,
}

impl ProviderManager {
    /// 创建新的提供商管理器
    pub fn new(checkin_dir: &Path) -> Self {
        Self {
            file_path: checkin_dir.join(PROVIDERS_FILE),
        }
    }

    /// 加载所有提供商
    pub fn load_all(&self) -> Result<Vec<CheckinProvider>> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| ProviderError::ReadError(e.to_string()))?;

        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        let providers: Vec<CheckinProvider> =
            serde_json::from_str(&content).map_err(|e| ProviderError::ParseError(e.to_string()))?;

        Ok(providers)
    }

    /// 保存所有提供商
    fn save_all(&self, providers: &[CheckinProvider]) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ProviderError::WriteError(format!("Failed to create directory: {}", e))
            })?;
        }

        // 序列化为 JSON
        let content = serde_json::to_string_pretty(providers)
            .map_err(|e| ProviderError::WriteError(format!("Failed to serialize: {}", e)))?;

        // 原子写入
        let temp_dir = self.file_path.parent().unwrap_or(std::path::Path::new("."));
        let mut temp_file = NamedTempFile::new_in(temp_dir)
            .map_err(|e| ProviderError::WriteError(format!("Failed to create temp file: {}", e)))?;

        temp_file
            .write_all(content.as_bytes())
            .map_err(|e| ProviderError::WriteError(format!("Failed to write temp file: {}", e)))?;

        temp_file
            .persist(&self.file_path)
            .map_err(|e| ProviderError::WriteError(format!("Failed to persist file: {}", e)))?;

        tracing::debug!(
            "Saved {} providers to {:?}",
            providers.len(),
            self.file_path
        );
        Ok(())
    }

    /// 获取所有提供商
    pub fn list(&self) -> Result<ProvidersResponse> {
        let providers = self.load_all()?;
        let total = providers.len();
        Ok(ProvidersResponse { providers, total })
    }

    /// 根据 ID 获取提供商
    pub fn get(&self, id: &str) -> Result<CheckinProvider> {
        let providers = self.load_all()?;
        providers
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| ProviderError::NotFound(id.to_string()))
    }

    /// 根据名称获取提供商
    #[allow(dead_code)]
    pub fn get_by_name(&self, name: &str) -> Result<Option<CheckinProvider>> {
        let providers = self.load_all()?;
        Ok(providers.into_iter().find(|p| p.name == name))
    }

    /// 创建提供商
    pub fn create(&self, request: CreateProviderRequest) -> Result<CheckinProvider> {
        let mut providers = self.load_all()?;

        // 检查名称是否已存在
        if providers.iter().any(|p| p.name == request.name) {
            return Err(ProviderError::AlreadyExists(request.name));
        }

        let provider = request.into_provider();
        providers.push(provider.clone());
        self.save_all(&providers)?;

        tracing::info!("Created provider: {} ({})", provider.name, provider.id);
        Ok(provider)
    }

    /// 更新提供商
    pub fn update(&self, id: &str, request: UpdateProviderRequest) -> Result<CheckinProvider> {
        let mut providers = self.load_all()?;

        // 先检查新名称是否与其他提供商冲突
        if let Some(new_name) = &request.name
            && providers.iter().any(|p| p.id != id && &p.name == new_name)
        {
            return Err(ProviderError::AlreadyExists(new_name.clone()));
        }

        let provider = providers
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| ProviderError::NotFound(id.to_string()))?;

        // 应用更新
        if let Some(name) = request.name {
            provider.name = name;
        }

        if let Some(base_url) = request.base_url {
            provider.base_url = base_url;
        }
        if let Some(checkin_path) = request.checkin_path {
            provider.checkin_path = checkin_path;
        }
        if let Some(balance_path) = request.balance_path {
            provider.balance_path = balance_path;
        }
        if let Some(user_info_path) = request.user_info_path {
            provider.user_info_path = user_info_path;
        }
        if let Some(auth_header) = request.auth_header {
            provider.auth_header = auth_header;
        }
        if let Some(auth_prefix) = request.auth_prefix {
            provider.auth_prefix = auth_prefix;
        }
        if let Some(enabled) = request.enabled {
            provider.enabled = enabled;
        }

        provider.updated_at = Some(Utc::now());

        let updated = provider.clone();
        self.save_all(&providers)?;

        tracing::info!("Updated provider: {} ({})", updated.name, updated.id);
        Ok(updated)
    }

    /// 删除提供商 (需要检查是否有关联账号)
    pub fn delete(&self, id: &str, has_accounts: bool) -> Result<()> {
        if has_accounts {
            return Err(ProviderError::HasAccounts(id.to_string()));
        }

        let mut providers = self.load_all()?;
        let original_len = providers.len();

        providers.retain(|p| p.id != id);

        if providers.len() == original_len {
            return Err(ProviderError::NotFound(id.to_string()));
        }

        self.save_all(&providers)?;
        tracing::info!("Deleted provider: {}", id);
        Ok(())
    }

    /// 批量导入提供商
    pub fn import_batch(
        &self,
        providers_to_import: Vec<CheckinProvider>,
        overwrite: bool,
    ) -> Result<(usize, usize)> {
        let mut providers = self.load_all()?;
        let mut imported = 0;
        let mut skipped = 0;

        for new_provider in providers_to_import {
            if let Some(existing) = providers
                .iter_mut()
                .find(|p| p.id == new_provider.id || p.name == new_provider.name)
            {
                if overwrite {
                    *existing = new_provider;
                    imported += 1;
                } else {
                    skipped += 1;
                }
            } else {
                providers.push(new_provider);
                imported += 1;
            }
        }

        self.save_all(&providers)?;
        Ok((imported, skipped))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (TempDir, ProviderManager) {
        let temp_dir = TempDir::new().unwrap();
        let manager = ProviderManager::new(temp_dir.path());
        (temp_dir, manager)
    }

    #[test]
    fn test_create_and_get_provider() {
        let (_temp_dir, manager) = setup();

        let request = CreateProviderRequest {
            name: "Test Provider".to_string(),
            base_url: "https://api.example.com".to_string(),
            checkin_path: None,
            balance_path: None,
            user_info_path: None,
            auth_header: None,
            auth_prefix: None,
        };

        let provider = manager.create(request).unwrap();
        assert_eq!(provider.name, "Test Provider");
        assert!(provider.enabled);

        let fetched = manager.get(&provider.id).unwrap();
        assert_eq!(fetched.name, "Test Provider");
    }

    #[test]
    fn test_list_providers() {
        let (_temp_dir, manager) = setup();

        // 初始为空
        let response = manager.list().unwrap();
        assert_eq!(response.total, 0);

        // 创建两个提供商
        manager
            .create(CreateProviderRequest {
                name: "Provider 1".to_string(),
                base_url: "https://api1.example.com".to_string(),
                checkin_path: None,
                balance_path: None,
                user_info_path: None,
                auth_header: None,
                auth_prefix: None,
            })
            .unwrap();

        manager
            .create(CreateProviderRequest {
                name: "Provider 2".to_string(),
                base_url: "https://api2.example.com".to_string(),
                checkin_path: None,
                balance_path: None,
                user_info_path: None,
                auth_header: None,
                auth_prefix: None,
            })
            .unwrap();

        let response = manager.list().unwrap();
        assert_eq!(response.total, 2);
    }

    #[test]
    fn test_update_provider() {
        let (_temp_dir, manager) = setup();

        let provider = manager
            .create(CreateProviderRequest {
                name: "Original".to_string(),
                base_url: "https://api.example.com".to_string(),
                checkin_path: None,
                balance_path: None,
                user_info_path: None,
                auth_header: None,
                auth_prefix: None,
            })
            .unwrap();

        let updated = manager
            .update(
                &provider.id,
                UpdateProviderRequest {
                    name: Some("Updated".to_string()),
                    base_url: Some("https://new-api.example.com".to_string()),
                    checkin_path: None,
                    balance_path: None,
                    user_info_path: None,
                    auth_header: None,
                    auth_prefix: None,
                    enabled: Some(false),
                },
            )
            .unwrap();

        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.base_url, "https://new-api.example.com");
        assert!(!updated.enabled);
    }

    #[test]
    fn test_delete_provider() {
        let (_temp_dir, manager) = setup();

        let provider = manager
            .create(CreateProviderRequest {
                name: "To Delete".to_string(),
                base_url: "https://api.example.com".to_string(),
                checkin_path: None,
                balance_path: None,
                user_info_path: None,
                auth_header: None,
                auth_prefix: None,
            })
            .unwrap();

        manager.delete(&provider.id, false).unwrap();

        assert!(manager.get(&provider.id).is_err());
    }

    #[test]
    fn test_cannot_delete_provider_with_accounts() {
        let (_temp_dir, manager) = setup();

        let provider = manager
            .create(CreateProviderRequest {
                name: "Has Accounts".to_string(),
                base_url: "https://api.example.com".to_string(),
                checkin_path: None,
                balance_path: None,
                user_info_path: None,
                auth_header: None,
                auth_prefix: None,
            })
            .unwrap();

        let result = manager.delete(&provider.id, true);
        assert!(matches!(result, Err(ProviderError::HasAccounts(_))));
    }
}
