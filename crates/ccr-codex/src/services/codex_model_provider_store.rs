use crate::models::{CodexModelProviderRecord, CodexModelProviderStore};
use crate::utils::CodexPaths;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::guarded_write::{self, BackupPolicy, WriteOptions};
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
    leaf_locks: PathBuf,
}

impl CodexModelProviderStoreService {
    pub fn new() -> Result<Self> {
        let paths = CodexPaths::resolve()?;
        Ok(Self::from_parts(
            &paths.ccr_codex_dir,
            LockManager::with_default_path()?.lock_dir(),
        ))
    }

    pub fn from_codex_dir(ccr_codex_dir: &Path) -> Self {
        Self::from_parts(ccr_codex_dir, &ccr_codex_dir.join(".locks"))
    }

    /// Pure captured-path construction; all nested leaf writes use these locks.
    pub fn from_parts(ccr_codex_dir: &Path, leaf_locks: &Path) -> Self {
        Self {
            store_path: ccr_codex_dir.join("model_providers.json"),
            backup_dir: ccr_codex_dir.join("auth").join("backups"),
            lock_dir: ccr_codex_dir.join(".locks"),
            leaf_locks: leaf_locks.to_path_buf(),
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
        let mut store: CodexModelProviderStore =
            serde_json::from_str(&content).map_err(|error| {
                CcrError::ConfigError(format!(
                    "Codex model providers 格式无效 (line {}, column {})",
                    error.line(),
                    error.column()
                ))
            })?;
        if store.version == 0 {
            store.version = 1;
        }
        Ok(store)
    }

    fn locked<T>(&self, work: impl FnOnce() -> Result<T>) -> Result<T> {
        LockManager::with_scoped_directory(&self.leaf_locks, || {
            let manager = LockManager::new(&self.lock_dir);
            let _lock = manager.lock_resource(PROVIDER_LOCK_RESOURCE, PROVIDER_LOCK_TIMEOUT)?;
            work()
        })
    }

    fn backup_policy(&self) -> BackupPolicy {
        BackupPolicy::Dir {
            dir: self.backup_dir.clone(),
            prefix: "model_providers".into(),
        }
    }

    fn harden_existing(&self) -> Result<()> {
        let bytes = match fs::read(&self.store_path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(CcrError::ConfigError(format!(
                    "读取 provider store 失败: {error}"
                )));
            }
        };
        if !guarded_write::enforce_owner_only_permissions_versioned(
            &self.store_path,
            &guarded_write::content_version_token(&bytes),
            PROVIDER_LOCK_TIMEOUT,
        )? {
            return Err(CcrError::ConfigError(
                "provider_store_changed_reload_required".into(),
            ));
        }
        Ok(())
    }

    fn save_locked(&self, store: &CodexModelProviderStore) -> Result<()> {
        let content = serde_json::to_vec_pretty(store)
            .map_err(|_| CcrError::ConfigError("provider_store_serialization_failed".into()))?;
        self.harden_existing()?;
        guarded_write::write_guarded(
            &self.store_path,
            &content,
            &WriteOptions {
                backup: self.backup_policy(),
                secret: true,
                lock_timeout: PROVIDER_LOCK_TIMEOUT,
            },
        )
    }

    pub fn save(&self, store: &CodexModelProviderStore) -> Result<()> {
        self.locked(|| self.save_locked(store))
    }

    pub fn backup(&self) -> Result<Option<PathBuf>> {
        self.locked(|| {
            self.harden_existing()?;
            guarded_write::backup_guarded(&self.store_path, &self.backup_policy())
        })
    }

    pub fn upsert_provider(
        &self,
        provider: CodexModelProviderRecord,
    ) -> Result<CodexModelProviderRecord> {
        self.locked(|| {
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
            self.save_locked(&store)?;
            Ok(provider)
        })
    }

    pub fn delete_provider(&self, provider_id: &str) -> Result<()> {
        self.locked(|| {
            let mut store = self.load()?;
            store.providers.retain(|item| item.id != provider_id);
            self.save_locked(&store)
        })
    }

    /// Native edit sessions retain the original revision through confirmation.
    /// Compare under the same operation lock as legacy callers' mutations.
    pub fn mutate_versioned(
        &self,
        expected_token: &str,
        provider_id: &str,
        provider: Option<CodexModelProviderRecord>,
    ) -> Result<bool> {
        self.locked(|| {
            let token = match fs::read(&self.store_path) {
                Ok(bytes) => guarded_write::content_version_token(&bytes),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
                Err(error) => return Err(error.into()),
            };
            if token != expected_token {
                return Ok(false);
            }
            let mut store = self.load()?;
            if let Some(mut provider) = provider {
                if provider.id != provider_id {
                    return Err(CcrError::ValidationError("provider_id_mismatch".into()));
                }
                if let Some(existing) = store.providers.iter_mut().find(|p| p.id == provider_id) {
                    provider.created_at = existing.created_at;
                    if provider.api_keys.is_empty() {
                        provider.api_keys = existing.api_keys.clone();
                    }
                    *existing = provider;
                } else {
                    store.providers.push(provider);
                }
                store
                    .providers
                    .sort_by(|left, right| left.name.cmp(&right.name));
            } else {
                store.providers.retain(|p| p.id != provider_id);
            }
            let content = serde_json::to_vec_pretty(&store)
                .map_err(|_| CcrError::ConfigError("provider_store_serialization_failed".into()))?;
            self.harden_existing()?;
            let outcome = guarded_write::write_guarded_versioned(
                &self.store_path,
                &content,
                expected_token,
                &WriteOptions {
                    backup: self.backup_policy(),
                    secret: true,
                    lock_timeout: PROVIDER_LOCK_TIMEOUT,
                },
            )?;
            Ok(matches!(
                outcome,
                guarded_write::VersionedWriteOutcome::Written
            ))
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::CodexModelProviderApiKey;

    #[test]
    fn versioned_mutation_checks_original_token_under_the_owner_lock() {
        let temp = tempfile::tempdir().unwrap();
        let service = CodexModelProviderStoreService::from_codex_dir(temp.path());
        assert!(service.mutate_versioned("", "missing", None).unwrap());
        let original = fs::read(&service.store_path).unwrap();
        let token = guarded_write::content_version_token(&original);
        assert!(!service.mutate_versioned("", "missing", None).unwrap());
        service
            .save(&CodexModelProviderStore {
                version: 2,
                ..Default::default()
            })
            .unwrap();
        let changed = fs::read(&service.store_path).unwrap();
        assert!(!service.mutate_versioned(&token, "missing", None).unwrap());
        assert_eq!(fs::read(&service.store_path).unwrap(), changed);
        let token = guarded_write::content_version_token(&changed);
        assert!(service.mutate_versioned(&token, "missing", None).unwrap());
    }

    #[test]
    fn failed_provider_backup_preserves_previous_store() {
        let temp = tempfile::tempdir().unwrap();
        let service = CodexModelProviderStoreService::from_codex_dir(temp.path());
        service
            .save(&CodexModelProviderStore {
                version: 1,
                providers: Vec::new(),
            })
            .unwrap();
        let original = fs::read(&service.store_path).unwrap();
        fs::create_dir_all(service.backup_dir.parent().unwrap()).unwrap();
        fs::write(&service.backup_dir, b"not-a-directory").unwrap();
        assert!(
            service
                .save(&CodexModelProviderStore {
                    version: 2,
                    providers: Vec::new()
                })
                .is_err()
        );
        assert_eq!(fs::read(&service.store_path).unwrap(), original);
    }

    #[test]
    fn malformed_provider_diagnostic_does_not_echo_secret_values() {
        let temp = tempfile::tempdir().unwrap();
        let service = CodexModelProviderStoreService::from_codex_dir(temp.path());
        let secret = "provider-secret-sentinel";
        fs::write(
            &service.store_path,
            format!(r#"{{"providers":"{secret}"}}"#),
        )
        .unwrap();
        let error = service.load().unwrap_err().to_string();
        assert!(!error.contains(secret));
    }

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
