// 🔄 Codex Runtime 服务
// 统一管理 runtime 配置、OpenAI 凭据缓存与 CCR profile secret store。

use crate::managers::codex_config::CodexConfigManager;
use crate::models::{
    CodexProfileAuthMode, CodexProfileSecret, CodexProfileSecretStore, CredentialStoreKind,
    Platform, PlatformPaths, ProfileConfig,
};
use ccr_core::core::error::{CcrError, Result};
use chrono::Utc;
use indexmap::IndexMap;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

#[derive(Debug, Clone)]
pub enum CodexAuthCacheAction {
    Preserve,
    Write(JsonMap<String, JsonValue>),
    Delete,
}

#[derive(Debug, Clone)]
pub struct CodexRuntimeCommitPlan {
    pub config: Option<toml::Value>,
    pub auth_cache: CodexAuthCacheAction,
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
                CodexProfileAuthMode::OpenAiApiKey | CodexProfileAuthMode::ProviderEnvKey
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
            CodexProfileAuthMode::OpenAiApiKey | CodexProfileAuthMode::ProviderEnvKey => {
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
            CodexProfileAuthMode::OpenAiChatgpt | CodexProfileAuthMode::NoAuth => {}
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
            CodexProfileAuthMode::OpenAiApiKey | CodexProfileAuthMode::ProviderEnvKey
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
                    self.config_manager.save_auth_atomic(auth)
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

    #[allow(dead_code)]
    pub fn update_runtime_settings(&self, config: toml::Value) -> Result<()> {
        self.commit_plan(CodexRuntimeCommitPlan {
            config: Some(config),
            auth_cache: CodexAuthCacheAction::Preserve,
        })
    }

    fn save_secret_store(&self, store: &CodexProfileSecretStore) -> Result<()> {
        let path = self.secret_store_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CcrError::ConfigError(format!("创建 secret store 目录失败: {}", e)))?;
        }

        if store.profiles.is_empty() {
            remove_if_exists(&path)?;
            return Ok(());
        }

        let content = serde_json::to_string_pretty(store)
            .map_err(|e| CcrError::ConfigError(format!("序列化 secret store 失败: {}", e)))?;
        atomic_write(&path, content.as_bytes())
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

fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| CcrError::ConfigError("无效的目标路径".into()))?;
    fs::create_dir_all(parent)
        .map_err(|e| CcrError::ConfigError(format!("创建目录失败: {}", e)))?;

    let temp = NamedTempFile::new_in(parent)
        .map_err(|e| CcrError::ConfigError(format!("创建临时文件失败: {}", e)))?;
    fs::write(temp.path(), data)
        .map_err(|e| CcrError::ConfigError(format!("写入临时文件失败: {}", e)))?;
    temp.persist(path)
        .map_err(|e| CcrError::ConfigError(format!("原子写入失败: {}", e)))?;

    crate::utils::ensure_private_permissions(path);

    Ok(())
}

fn remove_if_exists(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)
            .map_err(|e| CcrError::ConfigError(format!("删除文件失败 {:?}: {}", path, e)))?;
    }
    Ok(())
}

fn restore_optional_backup(
    target: &Path,
    backup: Option<&Path>,
    existed_before: bool,
) -> Result<()> {
    match backup {
        Some(backup) if backup.exists() => {
            fs::copy(backup, target).map_err(|e| {
                CcrError::ConfigError(format!("回滚文件失败 {:?} <- {:?}: {}", target, backup, e))
            })?;
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
