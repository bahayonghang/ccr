// 🔄 Codex Runtime 服务
// 统一管理 runtime 配置、OpenAI 凭据缓存与 CCR profile secret store。

use crate::managers::codex_config::CodexConfigManager;
use crate::models::{
    CodexProfileAuthMode, CodexProfileSecret, CodexProfileSecretStore, CredentialStoreKind,
    Platform, PlatformPaths, ProfileConfig,
};
use ccr_core::core::atomic_writer::AtomicWriter;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::guarded_write::{
    VersionedWriteOutcome, WriteOptions, content_version_token, delete_guarded,
    enforce_owner_only_permissions_versioned, write_guarded, write_guarded_versioned,
};
use chrono::Utc;
use indexmap::IndexMap;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub enum CodexAuthCacheAction {
    Preserve,
    Write(JsonMap<String, JsonValue>),
    Delete,
}

impl std::fmt::Debug for CodexAuthCacheAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Preserve => "Preserve",
            Self::Write(_) => "Write([REDACTED])",
            Self::Delete => "Delete",
        })
    }
}

#[derive(Clone)]
pub struct CodexRuntimeCommitPlan {
    pub config: Option<toml::Value>,
    pub auth_cache: CodexAuthCacheAction,
}

impl std::fmt::Debug for CodexRuntimeCommitPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodexRuntimeCommitPlan")
            .field("config_present", &self.config.is_some())
            .field("auth_cache", &self.auth_cache)
            .finish()
    }
}

impl Default for CodexRuntimeCommitPlan {
    fn default() -> Self {
        Self {
            config: None,
            auth_cache: CodexAuthCacheAction::Preserve,
        }
    }
}

/// Codex 运行时协调服务
pub struct CodexRuntimeService {
    paths: PlatformPaths,
    codex_dir: PathBuf,
    config_manager: CodexConfigManager,
}

impl CodexRuntimeService {
    pub fn new() -> Result<Self> {
        let paths = PlatformPaths::new(Platform::Codex)?;
        let config_manager = CodexConfigManager::with_default()?;
        let codex_dir = config_manager
            .config_path()
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| CcrError::ConfigError("无法获取 Codex 配置目录".into()))?;

        Ok(Self {
            paths,
            codex_dir,
            config_manager,
        })
    }

    pub fn from_parts(
        paths: PlatformPaths,
        codex_dir: PathBuf,
        config_manager: CodexConfigManager,
    ) -> Self {
        Self {
            paths,
            codex_dir,
            config_manager,
        }
    }

    pub fn secret_store_path(&self) -> PathBuf {
        self.paths.platform_dir.join("profile_secrets.json")
    }

    pub fn load_secret_store(&self) -> Result<CodexProfileSecretStore> {
        let path = self.secret_store_path();
        if !path.exists() {
            return Ok(CodexProfileSecretStore::default());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| CcrError::ConfigError(format!("读取 Codex secret store 失败: {}", e)))?;
        serde_json::from_str(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析 Codex secret store 失败: {}", e)))
    }

    pub fn overlay_profile_secrets(
        &self,
        profiles: &mut IndexMap<String, ProfileConfig>,
    ) -> Result<()> {
        let store = self.load_secret_store()?;

        for (name, profile) in profiles.iter_mut() {
            let Some(secret) = store.profiles.get(name) else {
                continue;
            };

            if matches!(
                secret.auth_mode,
                CodexProfileAuthMode::OpenAiApiKey
                    | CodexProfileAuthMode::ProviderEnvKey
                    | CodexProfileAuthMode::ProviderBearerToken
            ) {
                profile.auth_token = Some(ccr_core::Secret::new(secret.secret.clone()));
            }

            if let Some(env_key) = &secret.env_key
                && !profile.platform_data.contains_key("env_key")
            {
                profile
                    .platform_data
                    .insert("env_key".to_string(), JsonValue::String(env_key.clone()));
            }
        }

        Ok(())
    }

    pub fn persist_profile_secret(
        &self,
        profile_name: &str,
        auth_mode: CodexProfileAuthMode,
        env_key: Option<String>,
        secret: Option<String>,
    ) -> Result<()> {
        let mut store = self.load_secret_store()?;

        match auth_mode {
            CodexProfileAuthMode::OpenAiApiKey
            | CodexProfileAuthMode::ProviderEnvKey
            | CodexProfileAuthMode::ProviderBearerToken => {
                let secret = secret.ok_or_else(|| {
                    CcrError::ValidationError("当前认证模式需要 auth_token / secret".into())
                })?;
                store.profiles.insert(
                    profile_name.to_string(),
                    CodexProfileSecret {
                        auth_mode,
                        env_key,
                        secret,
                        updated_at: Utc::now(),
                    },
                );
            }
            CodexProfileAuthMode::OpenAiChatgpt | CodexProfileAuthMode::NoAuth => {
                store.profiles.shift_remove(profile_name);
            }
        }

        self.save_secret_store(&store)
    }

    pub fn delete_profile_secret(&self, profile_name: &str) -> Result<()> {
        let mut store = self.load_secret_store()?;
        if store.profiles.shift_remove(profile_name).is_some() {
            self.save_secret_store(&store)?;
        }
        Ok(())
    }

    pub fn build_env_export(
        &self,
        profile_name: &str,
        auth_mode: CodexProfileAuthMode,
        env_key: Option<&str>,
    ) -> Result<IndexMap<String, String>> {
        let store = self.load_secret_store()?;
        let Some(secret) = store.profiles.get(profile_name) else {
            return Ok(IndexMap::new());
        };

        let mut env = IndexMap::new();
        match auth_mode {
            CodexProfileAuthMode::OpenAiApiKey => {
                env.insert("OPENAI_API_KEY".to_string(), secret.secret.clone());
            }
            CodexProfileAuthMode::ProviderEnvKey => {
                let key = env_key
                    .map(str::to_string)
                    .or_else(|| secret.env_key.clone())
                    .ok_or_else(|| {
                        CcrError::ValidationError("provider_env_key 模式缺少 env_key".into())
                    })?;
                env.insert(key, secret.secret.clone());
            }
            CodexProfileAuthMode::ProviderBearerToken
            | CodexProfileAuthMode::OpenAiChatgpt
            | CodexProfileAuthMode::NoAuth => {}
        }

        Ok(env)
    }

    pub fn shell_export_script(
        &self,
        profile_name: &str,
        auth_mode: CodexProfileAuthMode,
        env_key: Option<&str>,
    ) -> Result<String> {
        let env = self.build_env_export(profile_name, auth_mode, env_key)?;
        let script = env
            .into_iter()
            .map(|(key, value)| format!("export {}={}", key, shell_quote(&value)))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(script)
    }

    pub fn scrub_profile_secret_fields(
        profile: &mut ProfileConfig,
        auth_mode: CodexProfileAuthMode,
    ) {
        if matches!(
            auth_mode,
            CodexProfileAuthMode::OpenAiApiKey
                | CodexProfileAuthMode::ProviderEnvKey
                | CodexProfileAuthMode::ProviderBearerToken
        ) {
            profile.auth_token = None;
        }
    }

    pub fn commit_plan(&self, plan: CodexRuntimeCommitPlan) -> Result<()> {
        let target_store = match &plan.config {
            Some(config) => detect_auth_store(config),
            None => detect_auth_store(&self.config_manager.load_config()?),
        };

        // 非 file 凭据存储时，仅允许 Delete（清理旧 tokens）和 Preserve，
        // 阻止 Write（避免干扰系统钥匙链等外部凭据管理）
        if matches!(plan.auth_cache, CodexAuthCacheAction::Write(_))
            && !matches!(target_store, CredentialStoreKind::File)
        {
            return Err(CcrError::ValidationError(format!(
                "当前 Codex 凭据存储为 {}，CCR 暂不支持写入 auth.json；请先执行 `codex login` / `codex logout`，或将 cli_auth_credentials_store 切换为 file",
                target_store.as_str()
            )));
        }

        let config_backup = if plan.config.is_some() {
            self.config_manager.backup_config("runtime_switch")?
        } else {
            None
        };
        let auth_backup = if !matches!(plan.auth_cache, CodexAuthCacheAction::Preserve) {
            self.config_manager.backup_auth("runtime_switch")?
        } else {
            None
        };

        let config_existed = self.config_manager.config_path().exists();
        let auth_existed = self.config_manager.auth_path().exists();

        if let Some(config) = &plan.config
            && let Err(err) = self.config_manager.save_config_atomic(config)
        {
            restore_optional_backup(
                self.config_manager.config_path(),
                config_backup.as_deref(),
                config_existed,
            )?;
            return Err(err);
        }

        match &plan.auth_cache {
            CodexAuthCacheAction::Preserve => {}
            CodexAuthCacheAction::Write(auth) => {
                let result = if auth.is_empty() {
                    remove_if_exists(self.config_manager.auth_path())
                } else {
                    (|| {
                        if self.config_manager.auth_path().try_exists()? {
                            crate::utils::ensure_private_permissions(
                                self.config_manager.auth_path(),
                            )?;
                        }
                        self.config_manager.save_auth_atomic(auth)
                    })()
                };

                if let Err(err) = result {
                    if plan.config.is_some() {
                        restore_optional_backup(
                            self.config_manager.config_path(),
                            config_backup.as_deref(),
                            config_existed,
                        )?;
                    }
                    restore_optional_backup(
                        self.config_manager.auth_path(),
                        auth_backup.as_deref(),
                        auth_existed,
                    )?;
                    return Err(err);
                }
            }
            CodexAuthCacheAction::Delete => {
                if let Err(err) = remove_if_exists(self.config_manager.auth_path()) {
                    if plan.config.is_some() {
                        restore_optional_backup(
                            self.config_manager.config_path(),
                            config_backup.as_deref(),
                            config_existed,
                        )?;
                    }
                    restore_optional_backup(
                        self.config_manager.auth_path(),
                        auth_backup.as_deref(),
                        auth_existed,
                    )?;
                    return Err(err);
                }
            }
        }

        Ok(())
    }

    /// Write an auth-only synchronization plan if the runtime version still matches.
    /// A conflict keeps the current runtime; no compensation writes an older backup.
    pub(crate) fn commit_synced_auth_versioned(
        &self,
        auth: &JsonMap<String, JsonValue>,
        expected_version: &str,
    ) -> Result<VersionedWriteOutcome> {
        let target_store = detect_auth_store(&self.config_manager.load_config()?);
        if !matches!(target_store, CredentialStoreKind::File) {
            return Err(CcrError::ValidationError(format!(
                "当前 Codex 凭据存储为 {}，CCR 暂不支持写入 auth.json；请先执行 `codex login` / `codex logout`，或将 cli_auth_credentials_store 切换为 file",
                target_store.as_str()
            )));
        }
        let current = fs::read(self.config_manager.auth_path())
            .map_err(|e| CcrError::ConfigError(format!("读取 runtime auth.json 失败: {}", e)))?;
        if content_version_token(&current) != expected_version {
            return Ok(VersionedWriteOutcome::Conflict);
        }
        let content = serde_json::to_vec_pretty(auth)
            .map_err(|e| CcrError::ConfigError(format!("序列化 auth.json 失败: {}", e)))?;
        self.config_manager.backup_auth("runtime_switch")?;
        if !enforce_owner_only_permissions_versioned(
            self.config_manager.auth_path(),
            expected_version,
            std::time::Duration::from_secs(10),
        )? {
            return Ok(VersionedWriteOutcome::Conflict);
        }
        write_guarded_versioned(
            self.config_manager.auth_path(),
            &content,
            expected_version,
            &WriteOptions {
                secret: true,
                ..Default::default()
            },
        )
    }

    #[allow(dead_code)]
    pub fn update_runtime_settings(&self, config: toml::Value) -> Result<()> {
        self.commit_plan(CodexRuntimeCommitPlan {
            config: Some(config),
            auth_cache: CodexAuthCacheAction::Preserve,
        })
    }

    fn save_secret_store(&self, store: &CodexProfileSecretStore) -> Result<()> {
        let path = self.secret_store_path();
        if store.profiles.is_empty() {
            remove_if_exists(&path)?;
            return Ok(());
        }

        let content = serde_json::to_string_pretty(store)
            .map_err(|e| CcrError::ConfigError(format!("序列化 secret store 失败: {}", e)))?;
        if path.try_exists()? {
            crate::utils::ensure_private_permissions(&path)?;
        }
        write_guarded(
            &path,
            content.as_bytes(),
            &WriteOptions {
                secret: true,
                ..Default::default()
            },
        )
    }

    #[allow(dead_code)]
    pub fn codex_dir(&self) -> &Path {
        &self.codex_dir
    }
}

fn detect_auth_store(config: &toml::Value) -> CredentialStoreKind {
    let store = config
        .as_table()
        .and_then(|t| t.get("cli_auth_credentials_store"))
        .and_then(|v| v.as_str());
    CredentialStoreKind::from_config_value(store)
}

fn remove_if_exists(path: &Path) -> Result<()> {
    delete_guarded(path)
}

fn restore_optional_backup(
    target: &Path,
    backup: Option<&Path>,
    existed_before: bool,
) -> Result<()> {
    // The application journal owns compensation for this operation file.
    if ccr_core::core::write_journal::contains(target) {
        return Ok(());
    }
    match backup {
        Some(backup) if backup.exists() => {
            // 原子替换并在写入内容前设置私有权限（替代非原子的 fs::copy）
            let rollback_error = |e: &dyn std::fmt::Display| {
                CcrError::ConfigError(format!("回滚文件失败 {:?} <- {:?}: {}", target, backup, e))
            };
            let content = fs::read(backup).map_err(|e| rollback_error(&e))?;
            if target.try_exists()? {
                crate::utils::ensure_private_permissions(target)?;
            }
            AtomicWriter::new(target)
                .secret(true)
                .write(&content)
                .map_err(|e| rollback_error(&e))?;
        }
        _ if !existed_before => {
            remove_if_exists(target)?;
        }
        _ => {}
    }
    Ok(())
}

fn shell_quote(value: &str) -> String {
    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{}'", escaped)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestCodexEnv;
    use ccr_core::core::lock::LockManager;

    #[test]
    fn p4_runtime_commit_debug_redacts_auth_and_provider_config() {
        let auth =
            serde_json::from_str(r#"{"tokens":{"refresh_token":"p4-private-runtime"}}"#).unwrap();
        let plan = CodexRuntimeCommitPlan {
            config: Some(toml::from_str("provider_key = 'p4-private-config'").unwrap()),
            auth_cache: CodexAuthCacheAction::Write(auth),
        };
        assert!(!format!("{plan:?}").contains("p4-private-"));
        assert!(!format!("{:?}", plan.auth_cache).contains("p4-private-"));
    }

    #[test]
    fn synced_auth_cas_preserves_changed_runtime_and_config_without_rollback() {
        let env = TestCodexEnv::new();
        let codex_dir = env.codex_dir().to_path_buf();
        let manager = CodexConfigManager::new(
            codex_dir.join("config.toml"),
            codex_dir.join("auth.json"),
            codex_dir.join("backups"),
            LockManager::new(env.lock_dir()),
        );
        let config = b"cli_auth_credentials_store = 'file'\nmodel = 'unchanged'\n";
        fs::write(manager.config_path(), config).unwrap();
        let first = b"{\"synthetic\":\"first\"}";
        fs::write(manager.auth_path(), first).unwrap();
        let service = CodexRuntimeService::from_parts(
            PlatformPaths::new(Platform::Codex).unwrap(),
            codex_dir.clone(),
            manager,
        );
        let expected = content_version_token(first);
        let newer = b"{\"synthetic\":\"newer-runtime\"}";
        fs::write(codex_dir.join("auth.json"), newer).unwrap();
        let planned = serde_json::from_str("{\"synthetic\":\"planned\"}").unwrap();
        assert_eq!(
            service
                .commit_synced_auth_versioned(&planned, &expected)
                .unwrap(),
            VersionedWriteOutcome::Conflict
        );
        assert_eq!(fs::read(codex_dir.join("auth.json")).unwrap(), newer);
        assert_eq!(fs::read(codex_dir.join("config.toml")).unwrap(), config);
        assert!(!codex_dir.join("backups").exists());
        assert_eq!(
            service
                .commit_synced_auth_versioned(&planned, &content_version_token(newer))
                .unwrap(),
            VersionedWriteOutcome::Written
        );
        assert_eq!(fs::read(codex_dir.join("config.toml")).unwrap(), config);
        let backups: Vec<_> = fs::read_dir(codex_dir.join("backups"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0]).unwrap(), newer);
        assert!(
            backups[0]
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("auth.runtime_switch.")
        );
    }

    #[test]
    fn synced_auth_cas_conflict_after_backup_does_not_restore_old_runtime() {
        let env = TestCodexEnv::new();
        let codex_dir = env.codex_dir().to_path_buf();
        let auth_path = codex_dir.join("auth.json");
        let backup_dir = codex_dir.join("backups");
        let manager = CodexConfigManager::new(
            codex_dir.join("config.toml"),
            auth_path.clone(),
            &backup_dir,
            LockManager::new(env.lock_dir()),
        );
        fs::write(
            manager.config_path(),
            b"cli_auth_credentials_store = 'file'\n",
        )
        .unwrap();
        let original = b"{\"synthetic\":\"before\"}";
        write_guarded(
            &auth_path,
            original,
            &WriteOptions {
                secret: true,
                ..Default::default()
            },
        )
        .unwrap();
        let service = CodexRuntimeService::from_parts(
            PlatformPaths::new(Platform::Codex).unwrap(),
            codex_dir,
            manager,
        );
        let expected = content_version_token(original);
        let lock_manager = LockManager::with_default_path().unwrap();
        let resource = fs::read_dir(env.lock_dir())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.file_stem()
                    .is_some_and(|name| name.to_string_lossy().starts_with("gw_auth_"))
            })
            .unwrap();
        let lock = lock_manager
            .lock_resource(
                resource.file_stem().unwrap().to_str().unwrap(),
                std::time::Duration::from_secs(5),
            )
            .unwrap();
        let worker = std::thread::spawn(move || {
            service.commit_synced_auth_versioned(
                &serde_json::from_str("{\"synthetic\":\"planned\"}").unwrap(),
                &expected,
            )
        });
        let started = std::time::Instant::now();
        let backups = loop {
            let backups: Vec<_> = fs::read_dir(&backup_dir)
                .into_iter()
                .flatten()
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| {
                    path.file_name().is_some_and(|name| {
                        name.to_string_lossy().starts_with("auth.runtime_switch.")
                            && name.to_string_lossy().ends_with(".json.bak")
                    })
                })
                .collect();
            if !backups.is_empty() {
                break backups;
            }
            assert!(
                started.elapsed() < std::time::Duration::from_secs(5),
                "runtime backup did not complete"
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
        };
        let newer = b"{\"synthetic\":\"replacement\"}";
        fs::write(&auth_path, newer).unwrap();
        drop(lock);
        assert_eq!(
            worker.join().unwrap().unwrap(),
            VersionedWriteOutcome::Conflict
        );
        assert_eq!(fs::read(auth_path).unwrap(), newer);
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0]).unwrap(), original);
    }

    #[test]
    fn commit_plan_rollback_restores_config_from_deduplicated_backup() {
        let env = TestCodexEnv::new();
        let codex_dir = env.codex_dir().to_path_buf();
        let backup_dir = codex_dir.join("backups");
        // auth.json 的父路径是普通文件，使 auth 写入在 config 写入成功后失败
        let blocker = env.home().join("blocker");
        fs::write(&blocker, b"not a directory").unwrap();
        let manager = CodexConfigManager::new(
            codex_dir.join("config.toml"),
            blocker.join("auth.json"),
            &backup_dir,
            LockManager::new(env.lock_dir()),
        );
        let original = "cli_auth_credentials_store = \"file\"\nmodel = \"before\"\n";
        fs::write(manager.config_path(), original).unwrap();
        let seeded = manager.backup_config("seed").unwrap().unwrap();

        let service = CodexRuntimeService::from_parts(
            PlatformPaths::new(Platform::Codex).unwrap(),
            codex_dir.clone(),
            manager,
        );
        let next: toml::Value =
            toml::from_str("cli_auth_credentials_store = \"file\"\nmodel = \"after\"\n").unwrap();
        let mut auth = JsonMap::new();
        auth.insert(
            "OPENAI_API_KEY".into(),
            JsonValue::String("sk-synthetic".into()),
        );

        let result = service.commit_plan(CodexRuntimeCommitPlan {
            config: Some(next),
            auth_cache: CodexAuthCacheAction::Write(auth),
        });

        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(codex_dir.join("config.toml")).unwrap(),
            original
        );
        let config_backups: Vec<PathBuf> = fs::read_dir(&backup_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("config"))
            })
            .collect();
        assert_eq!(config_backups, vec![seeded.clone()]);
        assert_eq!(fs::read_to_string(&seeded).unwrap(), original);
    }
}
