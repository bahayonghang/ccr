// 中转站提供商管理器
// 负责提供商配置的 CRUD 操作
// 使用 SQLite 统一存储（替代 JSON 文件）

use crate::database::{self, DatabaseError, repositories::checkin_repo};
use crate::models::checkin::{
    CheckinProvider, CreateProviderRequest, ProvidersResponse, UpdateProviderRequest,
};
use chrono::Utc;

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Provider not found: {0}")]
    NotFound(String),
    #[error("Provider already exists: {0}")]
    AlreadyExists(String),
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("Cannot delete provider with associated accounts: {0}")]
    HasAccounts(String),
}

pub type Result<T> = std::result::Result<T, ProviderError>;

/// 提供商管理器
/// 使用 SQLite 统一存储
pub struct ProviderManager;

impl ProviderManager {
    /// 创建新的提供商管理器
    /// 注意：不再需要 checkin_dir 参数，使用全局 SQLite 数据库
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    /// 获取所有提供商
    pub fn list(&self) -> Result<ProvidersResponse> {
        let providers = database::with_connection(checkin_repo::get_all_providers)?;
        let total = providers.len();
        Ok(ProvidersResponse { providers, total })
    }

    /// 根据 ID 获取提供商
    pub fn get(&self, id: &str) -> Result<CheckinProvider> {
        database::with_connection(|conn| checkin_repo::get_provider_by_id(conn, id))?
            .ok_or_else(|| ProviderError::NotFound(id.to_string()))
    }

    /// 根据名称获取提供商
    #[allow(dead_code)]
    pub fn get_by_name(&self, name: &str) -> Result<Option<CheckinProvider>> {
        let providers = database::with_connection(checkin_repo::get_all_providers)?;
        Ok(providers.into_iter().find(|p| p.name == name))
    }

    /// 创建提供商
    pub fn create(&self, request: CreateProviderRequest) -> Result<CheckinProvider> {
        // 检查名称是否已存在
        let existing = database::with_connection(checkin_repo::get_all_providers)?;
        if existing.iter().any(|p| p.name == request.name) {
            return Err(ProviderError::AlreadyExists(request.name));
        }

        let provider = request.into_provider();
        database::with_connection(|conn| checkin_repo::insert_provider(conn, &provider))?;

        tracing::info!("Created provider: {} ({})", provider.name, provider.id);
        Ok(provider)
    }

    /// 更新提供商
    pub fn update(&self, id: &str, request: UpdateProviderRequest) -> Result<CheckinProvider> {
        // 先获取现有提供商
        let mut provider = self.get(id)?;

        // 检查新名称是否与其他提供商冲突
        if let Some(ref new_name) = request.name {
            let existing = database::with_connection(checkin_repo::get_all_providers)?;
            if existing.iter().any(|p| p.id != id && &p.name == new_name) {
                return Err(ProviderError::AlreadyExists(new_name.clone()));
            }
        }

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

        database::with_connection(|conn| checkin_repo::update_provider(conn, &provider))?;

        tracing::info!("Updated provider: {} ({})", provider.name, provider.id);
        Ok(provider)
    }

    /// 删除提供商 (需要检查是否有关联账号)
    pub fn delete(&self, id: &str, has_accounts: bool) -> Result<()> {
        if has_accounts {
            return Err(ProviderError::HasAccounts(id.to_string()));
        }

        let deleted = database::with_connection(|conn| checkin_repo::delete_provider(conn, id))?;

        if !deleted {
            return Err(ProviderError::NotFound(id.to_string()));
        }

        tracing::info!("Deleted provider: {}", id);
        Ok(())
    }

    /// 批量导入提供商
    pub fn import_batch(
        &self,
        providers_to_import: Vec<CheckinProvider>,
        overwrite: bool,
    ) -> Result<(usize, usize)> {
        let existing = database::with_connection(checkin_repo::get_all_providers)?;
        let mut imported = 0;
        let mut skipped = 0;

        for new_provider in providers_to_import {
            let exists = existing
                .iter()
                .any(|p| p.id == new_provider.id || p.name == new_provider.name);

            if exists {
                if overwrite {
                    // 更新现有记录
                    database::with_connection(|conn| {
                        checkin_repo::update_provider(conn, &new_provider)
                    })?;
                    imported += 1;
                } else {
                    skipped += 1;
                }
            } else {
                // 插入新记录
                database::with_connection(|conn| {
                    checkin_repo::insert_provider(conn, &new_provider)
                })?;
                imported += 1;
            }
        }

        Ok((imported, skipped))
    }

    /// 加载所有提供商（兼容旧 API）
    pub fn load_all(&self) -> Result<Vec<CheckinProvider>> {
        let providers = database::with_connection(checkin_repo::get_all_providers)?;
        Ok(providers)
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
        conn.execute("DELETE FROM checkin_providers", []).unwrap();
        f(&conn)
    }

    #[test]
    fn test_create_and_get_provider() {
        with_test_db(|conn| {
            let request = CreateProviderRequest {
                name: "Test Provider".to_string(),
                base_url: "https://api.example.com".to_string(),
                checkin_path: None,
                balance_path: None,
                user_info_path: None,
                auth_header: None,
                auth_prefix: None,
            };

            let provider = request.into_provider();
            checkin_repo::insert_provider(conn, &provider).unwrap();

            let fetched = checkin_repo::get_provider_by_id(conn, &provider.id)
                .unwrap()
                .unwrap();
            assert_eq!(fetched.name, "Test Provider");
            assert!(fetched.enabled);
        });
    }

    #[test]
    fn test_list_providers() {
        with_test_db(|conn| {
            // 初始为空
            let providers = checkin_repo::get_all_providers(conn).unwrap();
            assert_eq!(providers.len(), 0);

            // 创建两个提供商
            let p1 = CheckinProvider::new(
                "Provider 1".to_string(),
                "https://api1.example.com".to_string(),
            );
            let p2 = CheckinProvider::new(
                "Provider 2".to_string(),
                "https://api2.example.com".to_string(),
            );

            checkin_repo::insert_provider(conn, &p1).unwrap();
            checkin_repo::insert_provider(conn, &p2).unwrap();

            let providers = checkin_repo::get_all_providers(conn).unwrap();
            assert_eq!(providers.len(), 2);
        });
    }

    #[test]
    fn test_update_provider() {
        with_test_db(|conn| {
            let mut provider = CheckinProvider::new(
                "Original".to_string(),
                "https://api.example.com".to_string(),
            );
            checkin_repo::insert_provider(conn, &provider).unwrap();

            // Update
            provider.name = "Updated".to_string();
            provider.base_url = "https://new-api.example.com".to_string();
            provider.enabled = false;
            provider.updated_at = Some(Utc::now());

            checkin_repo::update_provider(conn, &provider).unwrap();

            let fetched = checkin_repo::get_provider_by_id(conn, &provider.id)
                .unwrap()
                .unwrap();
            assert_eq!(fetched.name, "Updated");
            assert_eq!(fetched.base_url, "https://new-api.example.com");
            assert!(!fetched.enabled);
        });
    }

    #[test]
    fn test_delete_provider() {
        with_test_db(|conn| {
            let provider = CheckinProvider::new(
                "To Delete".to_string(),
                "https://api.example.com".to_string(),
            );
            checkin_repo::insert_provider(conn, &provider).unwrap();

            let deleted = checkin_repo::delete_provider(conn, &provider.id).unwrap();
            assert!(deleted);

            let fetched = checkin_repo::get_provider_by_id(conn, &provider.id).unwrap();
            assert!(fetched.is_none());
        });
    }
}
