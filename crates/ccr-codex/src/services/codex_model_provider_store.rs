use crate::models::{CodexModelProviderRecord, CodexModelProviderStore};
use crate::utils::{CodexPaths, ensure_private_permissions};
use ccr_core::core::atomic_writer::AtomicWriter;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::lock::LockManager;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const PROVIDER_LOCK_RESOURCE: &str = "codex_model_provider_store";
const PROVIDER_LOCK_TIMEOUT: Duration = Duration::from_secs(10);

pub struct CodexModelProviderStoreService {
    store_path: PathBuf,
    backup_dir: PathBuf,
    lock_dir: PathBuf,
}

impl CodexModelProviderStoreService {
    pub fn new() -> Result<Self> {
        let paths = CodexPaths::resolve()?;
        Ok(Self::from_codex_dir(&paths.ccr_codex_dir))
    }

    pub fn from_codex_dir(ccr_codex_dir: &Path) -> Self {
        Self {
            store_path: ccr_codex_dir.join("model_providers.json"),
            backup_dir: ccr_codex_dir.join("auth").join("backups"),
            lock_dir: ccr_codex_dir.join(".locks"),
        }
    }

    pub fn load(&self) -> Result<CodexModelProviderStore> {
        if !self.store_path.exists() {
            return Ok(CodexModelProviderStore {
                version: 1,
                ..Default::default()
            });
        }

        let content = fs::read_to_string(&self.store_path).map_err(|e| {
            CcrError::ConfigError(format!("读取 Codex model providers 失败: {}", e))
        })?;
        let mut store: CodexModelProviderStore = serde_json::from_str(&content).map_err(|e| {
            CcrError::ConfigError(format!("解析 Codex model providers 失败: {}", e))
        })?;
        if store.version == 0 {
            store.version = 1;
        }
        Ok(store)
    }

    pub fn save(&self, store: &CodexModelProviderStore) -> Result<()> {
        let lock_manager = LockManager::new(&self.lock_dir);
        let _lock = lock_manager.lock_resource(PROVIDER_LOCK_RESOURCE, PROVIDER_LOCK_TIMEOUT)?;

        if let Some(parent) = self.store_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CcrError::ConfigError(format!("创建 model providers 目录失败: {}", e))
            })?;
        }

        let _ = self.backup();
        let content = serde_json::to_string_pretty(store).map_err(|e| {
            CcrError::ConfigError(format!("序列化 Codex model providers 失败: {}", e))
        })?;
        AtomicWriter::new(&self.store_path)
            .write_string(&content)
            .map_err(|e| {
                CcrError::ConfigError(format!("写入 Codex model providers 失败: {}", e))
            })?;
        ensure_private_permissions(&self.store_path);
        Ok(())
    }

    pub fn backup(&self) -> Result<Option<PathBuf>> {
        if !self.store_path.exists() {
            return Ok(None);
        }

        fs::create_dir_all(&self.backup_dir).map_err(|e| {
            CcrError::ConfigError(format!("创建 model providers 备份目录失败: {}", e))
        })?;
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = self
            .backup_dir
            .join(format!("model_providers_{}.json", timestamp));
        fs::copy(&self.store_path, &backup_path).map_err(|e| {
            CcrError::ConfigError(format!("备份 Codex model providers 失败: {}", e))
        })?;
        Ok(Some(backup_path))
    }

    pub fn upsert_provider(
        &self,
        provider: CodexModelProviderRecord,
    ) -> Result<CodexModelProviderRecord> {
        let mut store = self.load()?;
        if let Some(existing) = store
            .providers
            .iter_mut()
            .find(|item| item.id == provider.id)
        {
            *existing = provider.clone();
        } else {
            store.providers.push(provider.clone());
        }
        store
            .providers
            .sort_by(|left, right| left.name.cmp(&right.name));
        self.save(&store)?;
        Ok(provider)
    }

    pub fn delete_provider(&self, provider_id: &str) -> Result<()> {
        let mut store = self.load()?;
        store.providers.retain(|item| item.id != provider_id);
        self.save(&store)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::CodexModelProviderApiKey;

    #[test]
    fn roundtrip_provider_store() {
        let temp = tempfile::tempdir().unwrap();
        let service = CodexModelProviderStoreService::from_codex_dir(temp.path());
        let now = chrono::Utc::now();
        let provider = CodexModelProviderRecord {
            id: "openai".to_string(),
            name: "OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            website_url: Some("https://platform.openai.com".to_string()),
            api_key_url: None,
            api_keys: vec![CodexModelProviderApiKey {
                id: "key-1".to_string(),
                name: "Primary".to_string(),
                api_key: "sk-test".to_string(),
                created_at: now,
                updated_at: now,
            }],
            created_at: now,
            updated_at: now,
        };

        service.upsert_provider(provider.clone()).unwrap();
        let loaded = service.load().unwrap();
        assert_eq!(loaded.providers, vec![provider]);
    }
}
