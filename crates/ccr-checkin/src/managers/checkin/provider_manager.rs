// 中转站提供商管理器
// 负责提供商配置的 CRUD 操作
// 使用 SQLite 统一存储（替代 JSON 文件）

use crate::core::error::DbError;
use ccr_db::database::{self, repositories::checkin_repo};
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
    Database(#[from] DbError),
    #[error("Cannot delete provider with associated accounts: {0}")]
    HasAccounts(String),
}

pub type Result<T> = std::result::Result<T, ProviderError>;

/// 提供商管理器
/// 使用 SQLite 统一存储
pub struct ProviderManager;

impl ProviderManager {
    /// 创建新的提供商管理器
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
                    database::with_connection(|conn| {
                        checkin_repo::update_provider(conn, &new_provider)
                    })?;
                    imported += 1;
                } else {
                    skipped += 1;
                }
            } else {
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

    /// 旧数据回填：为缺少 builtin_id 的提供商按 name/base_url 匹配内置目录并写回。
    /// 幂等操作（仅更新 builtin_id 为 NULL 的行），返回成功回填的数量。
    pub fn backfill_builtin_ids(&self) -> Result<usize> {
        use super::builtin_providers::get_builtin_providers;

        let providers = database::with_connection(checkin_repo::get_all_providers)?;
        let builtins = get_builtin_providers();
        let mut updated = 0;

        for provider in providers.iter().filter(|p| p.builtin_id.is_none()) {
            let matched = builtins.iter().find(|bp| {
                bp.name == provider.name
                    || bp.base_url.trim_end_matches('/') == provider.base_url.trim_end_matches('/')
            });

            if let Some(builtin) = matched {
                let written = database::with_connection(|conn| {
                    checkin_repo::set_provider_builtin_id_if_missing(
                        conn,
                        &provider.id,
                        &builtin.id,
                    )
                })?;
                if written {
                    updated += 1;
                }
            }
        }

        if updated > 0 {
            tracing::info!("Backfilled builtin_id for {} provider(s)", updated);
        }
        Ok(updated)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_db::database::repositories::checkin_repo;
    use ccr_db::database::schema::CREATE_TABLES_SQL;
    use once_cell::sync::Lazy;
    use rusqlite::Connection;
    use std::sync::Mutex;

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
                builtin_id: None,
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

    #[test]
    fn test_backfill_builtin_ids_matches_by_name_and_base_url() {
        // 该测试覆盖「旧数据无 builtin_id 时一次性回填」路径，使用全局测试库
        database::initialize_for_test().unwrap();
        database::with_connection(|conn| {
            conn.execute("DELETE FROM checkin_providers", [])
                .map(|_| ())
        })
        .unwrap();

        // 旧行 1：按 name 匹配（base_url 已被用户改过）
        let by_name = CheckinProvider::new(
            "AnyRouter".to_string(),
            "https://renamed-url.example".to_string(),
        );
        // 旧行 2：按 base_url 匹配（name 已被用户改过，含尾部斜杠）
        let by_url = CheckinProvider::new("我的小站".to_string(), "https://codex.cab/".to_string());
        // 旧行 3：自定义站，不应被回填
        let custom =
            CheckinProvider::new("Custom".to_string(), "https://custom.example".to_string());
        // 旧行 4：已有 builtin_id，不应被覆盖
        let mut already =
            CheckinProvider::new("Hotaru".to_string(), "https://hotaruapi.com".to_string());
        already.builtin_id = Some("builtin-hotaru".to_string());

        for provider in [&by_name, &by_url, &custom, &already] {
            database::with_connection(|conn| checkin_repo::insert_provider(conn, provider))
                .unwrap();
        }

        let manager = ProviderManager::new();
        let updated = manager.backfill_builtin_ids().unwrap();
        assert_eq!(updated, 2);

        assert_eq!(
            manager.get(&by_name.id).unwrap().builtin_id.as_deref(),
            Some("builtin-anyrouter")
        );
        assert_eq!(
            manager.get(&by_url.id).unwrap().builtin_id.as_deref(),
            Some("builtin-codex-cab")
        );
        assert_eq!(manager.get(&custom.id).unwrap().builtin_id, None);
        assert_eq!(
            manager.get(&already.id).unwrap().builtin_id.as_deref(),
            Some("builtin-hotaru")
        );

        // 幂等：再跑一次不应有新的回填
        assert_eq!(manager.backfill_builtin_ids().unwrap(), 0);

        database::with_connection(|conn| {
            conn.execute("DELETE FROM checkin_providers", [])
                .map(|_| ())
        })
        .unwrap();
    }
}
