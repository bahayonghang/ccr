// 🔐 Claude Auth 服务层
// 管理 Claude 官方订阅凭据与 CCR 自管账号快照

use crate::managers::{ClaudeSettings, SettingsManager};
use crate::models::{
    ClaudeAuthAccount, ClaudeAuthRegistry, ClaudeCurrentAuthInfo, ClaudeLoginState,
    ClaudeProfileAuthMode, ClaudeRuntimeMode, ClaudeRuntimeSummary, PlatformConfig, ProfileConfig,
};
use crate::platforms::ClaudePlatform;
use ccr_config::ClaudeRuntimePaths;
use ccr_config::managers::config::CcsConfig;
use ccr_config::platforms::base as platform_base;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::{
    BackupPolicy, LockManager, WriteOptions, content_version_token, write_guarded,
};
use chrono::{DateTime, TimeZone, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};

/// Claude auth 读取快照
pub struct ClaudeAuthReadSnapshot {
    pub login_state: ClaudeLoginState,
    pub current_info: Option<ClaudeCurrentAuthInfo>,
    pub registry: ClaudeAuthRegistry,
    pub current_account_name: Option<String>,
    pub runtime_usable: bool,
}

/// Claude auth 列表项
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClaudeAuthItem {
    pub name: String,
    pub description: Option<String>,
    pub email: Option<String>,
    pub billing_type: Option<String>,
    pub subscription_type: Option<String>,
    pub rate_limit_tier: Option<String>,
    pub is_current: bool,
    pub is_logged_in: bool,
    pub saved_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

struct RuntimeAuthRead {
    info: Option<ClaudeCurrentAuthInfo>,
    usable: bool,
    matched_account_name: Option<String>,
}

/// Claude auth 服务
pub struct ClaudeAuthService {
    ccr_claude_dir: PathBuf,
    runtime_paths: ClaudeRuntimePaths,
    lock_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeCredentialsDocument {
    #[serde(default, rename = "claudeAiOauth")]
    claude_ai_oauth: Option<ClaudeAiOauth>,
    #[serde(default, flatten)]
    other: JsonMap<String, JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeAiOauth {
    #[serde(rename = "accessToken", serialize_with = "ccr_core::expose_plaintext")]
    access_token: ccr_core::Secret,
    #[serde(rename = "refreshToken", serialize_with = "ccr_core::expose_plaintext")]
    refresh_token: ccr_core::Secret,
    #[serde(rename = "expiresAt")]
    expires_at: ClaudeExpiryValue,
    #[serde(default, rename = "subscriptionType")]
    subscription_type: Option<String>,
    #[serde(default, rename = "rateLimitTier")]
    rate_limit_tier: Option<String>,
    #[serde(default)]
    scopes: Option<Vec<String>>,
    #[serde(default, flatten)]
    other: JsonMap<String, JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum ClaudeExpiryValue {
    Text(String),
    Epoch(i64),
}

impl ClaudeExpiryValue {
    fn to_datetime(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::Text(value) => DateTime::parse_from_rfc3339(value)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|| value.parse::<i64>().ok().and_then(Self::epoch_to_datetime)),
            Self::Epoch(value) => Self::epoch_to_datetime(*value),
        }
    }

    fn epoch_to_datetime(value: i64) -> Option<DateTime<Utc>> {
        if value.abs() >= 100_000_000_000 {
            Utc.timestamp_millis_opt(value).single()
        } else {
            Utc.timestamp_opt(value, 0).single()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeMetadataDocument {
    #[serde(default, rename = "oauthAccount")]
    oauth_account: Option<ClaudeOauthAccount>,
    #[serde(default, flatten)]
    other: JsonMap<String, JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeOauthAccount {
    #[serde(default, rename = "accountUuid")]
    account_uuid: Option<String>,
    #[serde(default, rename = "emailAddress")]
    email_address: Option<String>,
    #[serde(default, rename = "billingType")]
    billing_type: Option<String>,
    #[serde(default, rename = "displayName")]
    display_name: Option<String>,
    #[serde(default, rename = "organizationUuid")]
    organization_uuid: Option<String>,
    #[serde(default, flatten)]
    other: JsonMap<String, JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeAuthSnapshotFile {
    #[serde(default = "default_snapshot_version")]
    version: String,
    saved_at: DateTime<Utc>,
    credentials: ClaudeCredentialsDocument,
    #[serde(skip_serializing_if = "Option::is_none")]
    oauth_account: Option<ClaudeOauthAccount>,
}

fn default_snapshot_version() -> String {
    "1.0".to_string()
}

impl ClaudeAuthService {
    /// 创建服务实例
    pub fn new() -> Result<Self> {
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
        let ccr_root = std::env::var("CCR_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(".ccr"));
        let runtime_paths = ClaudeRuntimePaths::from_env()?;
        let lock_dir = LockManager::with_default_path()?.lock_dir().to_path_buf();

        Ok(Self {
            ccr_claude_dir: ccr_root.join("platforms").join("claude"),
            runtime_paths,
            lock_dir,
        })
    }

    #[allow(dead_code)]
    pub fn from_parts(
        ccr_claude_dir: PathBuf,
        claude_dir: PathBuf,
        claude_json_path: PathBuf,
    ) -> Self {
        let lock_dir = claude_dir.join(".locks");
        let runtime_paths = ClaudeRuntimePaths {
            settings_file: claude_dir.join("settings.json"),
            credentials_file: claude_dir.join(".credentials.json"),
            state_file: claude_json_path,
            backups_dir: claude_dir.join("backups"),
            config_dir: claude_dir,
        };
        Self {
            ccr_claude_dir,
            runtime_paths,
            lock_dir,
        }
    }

    fn credentials_path(&self) -> PathBuf {
        self.runtime_paths.credentials_file.clone()
    }

    fn registry_path(&self) -> PathBuf {
        self.ccr_claude_dir.join("auth_registry.toml")
    }

    fn auth_storage_dir(&self) -> PathBuf {
        self.ccr_claude_dir.join("auth")
    }

    fn settings_path(&self) -> PathBuf {
        self.runtime_paths.settings_file.clone()
    }

    fn settings_manager(&self) -> SettingsManager {
        SettingsManager::new(
            self.settings_path(),
            &self.runtime_paths.backups_dir,
            LockManager::new(&self.lock_dir),
        )
    }

    fn load_settings(&self) -> Result<ClaudeSettings> {
        match self.settings_manager().load() {
            Ok(settings) => Ok(settings),
            Err(CcrError::SettingsMissing(_)) => Ok(ClaudeSettings::new()),
            Err(err) => Err(err),
        }
    }

    fn profiles_path(&self) -> PathBuf {
        self.ccr_claude_dir.join("profiles.toml")
    }

    fn load_profiles(&self) -> Result<IndexMap<String, ProfileConfig>> {
        platform_base::load_profiles_from_toml(&self.profiles_path())
    }

    fn current_profile_from_file(
        &self,
        profiles: &IndexMap<String, ProfileConfig>,
    ) -> Result<Option<String>> {
        let path = self.profiles_path();
        if !path.exists() {
            return Ok(None);
        }

        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => return Ok(None),
        };

        let parsed = match toml::from_str::<CcsConfig>(&content) {
            Ok(parsed) => parsed,
            Err(_) => return Ok(None),
        };

        let current = parsed.current_config.trim();
        if current.is_empty() || !profiles.contains_key(current) {
            return Ok(None);
        }

        Ok(Some(current.to_string()))
    }

    fn current_profile(&self) -> Result<Option<(String, ProfileConfig)>> {
        let profiles = self.load_profiles()?;
        let Some(current_name) = self.current_profile_from_file(&profiles)? else {
            return Ok(None);
        };

        Ok(profiles
            .get(&current_name)
            .cloned()
            .map(|profile| (current_name, profile)))
    }

    fn clear_profile_api_key_overrides_if_needed(&self) -> Result<bool> {
        let Some((_, profile)) = self.current_profile()? else {
            return Ok(false);
        };
        if !matches!(
            Self::effective_auth_mode(&profile),
            ClaudeProfileAuthMode::ApiKey
        ) {
            return Ok(false);
        }

        let manager = self.settings_manager();
        let settings = match manager.load() {
            Ok(settings) => settings,
            Err(CcrError::SettingsMissing(_)) => return Ok(false),
            Err(error) => return Err(error),
        };
        if !settings.has_managed_overrides() {
            return Ok(false);
        }

        manager.update_atomic(|settings| {
            let changed = settings.has_managed_overrides();
            settings.clear_ccr_managed_vars();
            Ok(changed)
        })
    }

    fn account_snapshot_path(&self, name: &str) -> PathBuf {
        self.auth_storage_dir().join(format!("{name}.json"))
    }

    fn ensure_storage_dirs(&self) -> Result<()> {
        fs::create_dir_all(self.auth_storage_dir())?;
        Ok(())
    }

    pub fn load_registry(&self) -> Result<ClaudeAuthRegistry> {
        let path = self.registry_path();
        if !path.exists() {
            return Ok(ClaudeAuthRegistry::default());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| CcrError::ConfigError(format!("读取 Claude auth registry 失败: {e}")))?;
        toml::from_str(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析 Claude auth registry 失败: {e}")))
    }

    fn save_registry(&self, registry: &ClaudeAuthRegistry) -> Result<()> {
        self.ensure_storage_dirs()?;
        let path = self.registry_path();
        let content = toml::to_string_pretty(registry)
            .map_err(|e| CcrError::ConfigError(format!("序列化 Claude auth registry 失败: {e}")))?;
        self.write_secret(&path, content.as_bytes())
    }

    fn write_secret(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        write_guarded(
            path,
            bytes,
            &WriteOptions {
                backup: BackupPolicy::None,
                secret: true,
                ..Default::default()
            },
        )
    }

    fn ensure_file_credentials_supported() -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            return Err(CcrError::ValidationError(
                "macOS 上 Claude Code 官方凭据由 Keychain 管理，暂不支持 auth save/switch".into(),
            ));
        }

        #[cfg(not(target_os = "macos"))]
        Ok(())
    }

    fn validate_account_name(&self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(CcrError::ValidationError("账号名称不能为空".into()));
        }

        if name == "default" {
            return Err(CcrError::ValidationError(
                "'default' 是保留名称，请使用其他名称".into(),
            ));
        }

        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(CcrError::ValidationError(
                "账号名称只能包含字母、数字、下划线和连字符".into(),
            ));
        }

        if name.len() > 32 {
            return Err(CcrError::ValidationError(
                "账号名称不能超过 32 个字符".into(),
            ));
        }

        Ok(())
    }

    pub fn mask_email(&self, email: &str) -> String {
        if let Some(at_pos) = email.find('@') {
            let local = &email[..at_pos];
            let domain = &email[at_pos..];

            if local.len() <= 3 {
                format!("{local}***{domain}")
            } else {
                format!("{}***{domain}", &local[..3])
            }
        } else {
            email.to_string()
        }
    }

    fn read_optional_json<T: for<'de> Deserialize<'de>>(&self, path: &Path) -> Option<T> {
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn load_current_credentials_strict(&self) -> Result<ClaudeCredentialsDocument> {
        let path = self.credentials_path();
        if !path.exists() {
            return Err(CcrError::SettingsMissing(path.display().to_string()));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| CcrError::SettingsError(format!("读取 Claude 凭据失败: {e}")))?;
        serde_json::from_str::<ClaudeCredentialsDocument>(&content)
            .map_err(|e| CcrError::SettingsError(format!("解析 Claude 凭据失败: {e}")))
    }

    fn credentials_identity_token(credentials: &ClaudeCredentialsDocument) -> Result<String> {
        let bytes = serde_json::to_vec(credentials)
            .map_err(|error| CcrError::SettingsError(format!("序列化凭据身份失败: {error}")))?;
        Ok(content_version_token(&bytes))
    }

    fn find_matching_snapshot(
        &self,
        registry: &ClaudeAuthRegistry,
        credentials: &ClaudeCredentialsDocument,
    ) -> Option<(String, ClaudeAuthSnapshotFile)> {
        let current_token = Self::credentials_identity_token(credentials).ok()?;
        registry.accounts.keys().find_map(|name| {
            let snapshot = self.load_snapshot(name).ok()?;
            let snapshot_token = Self::credentials_identity_token(&snapshot.credentials).ok()?;
            (snapshot_token == current_token).then(|| (name.clone(), snapshot))
        })
    }

    fn load_current_runtime_auth(&self, registry: &ClaudeAuthRegistry) -> RuntimeAuthRead {
        let credentials = self
            .read_optional_json::<ClaudeCredentialsDocument>(&self.credentials_path())
            .filter(|doc| doc.claude_ai_oauth.is_some());
        let matched = credentials
            .as_ref()
            .and_then(|document| self.find_matching_snapshot(registry, document));
        let matched_account_name = matched.as_ref().map(|(name, _)| name.clone());
        let fallback_metadata = matched.is_none().then(|| {
            self.read_optional_json::<ClaudeMetadataDocument>(&self.runtime_paths.state_file)
        });
        let oauth_account = matched
            .as_ref()
            .and_then(|(_, snapshot)| snapshot.oauth_account.as_ref())
            .or_else(|| {
                fallback_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.as_ref())
                    .and_then(|metadata| metadata.oauth_account.as_ref())
            });
        let info = credentials
            .as_ref()
            .and_then(|document| self.build_current_info(document, oauth_account));
        let usable = info.is_some();

        RuntimeAuthRead {
            info,
            usable,
            matched_account_name,
        }
    }

    fn build_current_info(
        &self,
        credentials: &ClaudeCredentialsDocument,
        oauth_account: Option<&ClaudeOauthAccount>,
    ) -> Option<ClaudeCurrentAuthInfo> {
        let oauth = credentials.claude_ai_oauth.as_ref()?;
        let expires_at = oauth.expires_at.to_datetime();

        Some(ClaudeCurrentAuthInfo {
            account_uuid: oauth_account.and_then(|account| account.account_uuid.clone()),
            email: oauth_account.and_then(|account| account.email_address.clone()),
            billing_type: oauth_account.and_then(|account| account.billing_type.clone()),
            subscription_type: oauth.subscription_type.clone(),
            rate_limit_tier: oauth.rate_limit_tier.clone(),
            expires_at,
        })
    }

    fn build_registry_account(
        &self,
        description: Option<String>,
        info: &ClaudeCurrentAuthInfo,
    ) -> ClaudeAuthAccount {
        ClaudeAuthAccount {
            description,
            account_uuid: info.account_uuid.clone(),
            email: info.email.as_deref().map(|email| self.mask_email(email)),
            billing_type: info.billing_type.clone(),
            subscription_type: info.subscription_type.clone(),
            rate_limit_tier: info.rate_limit_tier.clone(),
            saved_at: Utc::now(),
            last_used: None,
            expires_at: info.expires_at,
        }
    }

    fn load_snapshot(&self, name: &str) -> Result<ClaudeAuthSnapshotFile> {
        let path = self.account_snapshot_path(name);
        if !path.exists() {
            return Err(CcrError::ResourceNotFound(format!(
                "Claude auth account '{}'",
                name
            )));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| CcrError::SettingsError(format!("读取账号快照失败: {e}")))?;
        serde_json::from_str(&content)
            .map_err(|e| CcrError::SettingsError(format!("解析账号快照失败: {e}")))
    }

    pub fn read_auth_snapshot(&self) -> Result<ClaudeAuthReadSnapshot> {
        let registry = self.load_registry()?;
        let runtime = self.load_current_runtime_auth(&registry);
        let current_account_name = runtime.matched_account_name;

        let login_state = if runtime.info.is_none() {
            ClaudeLoginState::NotLoggedIn
        } else if let Some(account_name) = current_account_name.clone() {
            ClaudeLoginState::LoggedInSaved { account_name }
        } else {
            ClaudeLoginState::LoggedInUnsaved
        };

        Ok(ClaudeAuthReadSnapshot {
            login_state,
            current_info: runtime.info,
            registry,
            current_account_name,
            runtime_usable: runtime.usable,
        })
    }

    pub fn get_current_auth_info(&self) -> Result<ClaudeCurrentAuthInfo> {
        self.read_auth_snapshot()?
            .current_info
            .ok_or_else(|| CcrError::SettingsError("未检测到 Claude 官方订阅登录".into()))
    }

    pub fn build_account_items(
        &self,
        snapshot: &ClaudeAuthReadSnapshot,
        runtime_summary: Option<&ClaudeRuntimeSummary>,
    ) -> Result<Vec<ClaudeAuthItem>> {
        let current_login_name = runtime_summary
            .and_then(|summary| summary.current_login_name.as_deref())
            .or(snapshot.current_account_name.as_deref());
        let current_auth_name = if let Some(summary) = runtime_summary {
            summary.current_auth_name.as_deref()
        } else {
            snapshot.current_account_name.as_deref()
        };

        let mut items = snapshot
            .registry
            .accounts
            .iter()
            .map(|(name, account)| ClaudeAuthItem {
                name: name.clone(),
                description: account.description.clone(),
                email: account.email.clone(),
                billing_type: account.billing_type.clone(),
                subscription_type: account.subscription_type.clone(),
                rate_limit_tier: account.rate_limit_tier.clone(),
                is_current: current_auth_name == Some(name.as_str()),
                is_logged_in: current_login_name == Some(name.as_str()),
                saved_at: account.saved_at,
                last_used: account.last_used,
                expires_at: account.expires_at,
            })
            .collect::<Vec<_>>();

        items.sort_by_key(|item| std::cmp::Reverse(item.saved_at));
        Ok(items)
    }

    pub fn list_accounts(&self) -> Result<Vec<ClaudeAuthItem>> {
        let snapshot = self.read_auth_snapshot()?;
        let runtime_summary = self.get_runtime_summary().ok();
        self.build_account_items(&snapshot, runtime_summary.as_ref())
    }

    pub fn save_current(
        &self,
        name: &str,
        description: Option<String>,
        force: bool,
    ) -> Result<ClaudeAuthAccount> {
        Self::ensure_file_credentials_supported()?;
        self.validate_account_name(name)?;

        let credentials = self.load_current_credentials_strict()?;
        if credentials.claude_ai_oauth.is_none() {
            return Err(CcrError::ValidationError(
                "未检测到 Claude 官方订阅登录，请先运行 `claude login`".into(),
            ));
        }

        let metadata =
            self.read_optional_json::<ClaudeMetadataDocument>(&self.runtime_paths.state_file);
        let info = self
            .build_current_info(
                &credentials,
                metadata
                    .as_ref()
                    .and_then(|metadata| metadata.oauth_account.as_ref()),
            )
            .ok_or_else(|| {
                CcrError::ValidationError(
                    "未检测到 Claude 官方订阅登录，请先运行 `claude login`".into(),
                )
            })?;

        let mut registry = self.load_registry()?;
        if registry.accounts.contains_key(name) && !force {
            return Err(CcrError::ResourceAlreadyExists(format!(
                "Claude auth account '{}' 已存在",
                name
            )));
        }

        self.ensure_storage_dirs()?;
        let snapshot = ClaudeAuthSnapshotFile {
            version: default_snapshot_version(),
            saved_at: Utc::now(),
            credentials,
            oauth_account: metadata.and_then(|meta| meta.oauth_account),
        };

        let snapshot_content = serde_json::to_vec_pretty(&snapshot)
            .map_err(|e| CcrError::SettingsError(format!("序列化账号快照失败: {e}")))?;
        self.write_secret(&self.account_snapshot_path(name), &snapshot_content)?;

        let account = self.build_registry_account(description, &info);
        registry.accounts.insert(name.to_string(), account.clone());

        registry.current_auth = Some(name.to_string());

        self.save_registry(&registry)?;
        Ok(account)
    }

    pub fn switch_account(&self, name: &str) -> Result<()> {
        Self::ensure_file_credentials_supported()?;
        let snapshot = self.load_snapshot(name)?;
        let mut registry = self.load_registry()?;
        if !registry.accounts.contains_key(name) {
            return Err(CcrError::ResourceNotFound(format!(
                "Claude auth account '{}'",
                name
            )));
        }

        let previous_snapshot = if self.credentials_path().exists() {
            let current_credentials = self.load_current_credentials_strict()?;
            Some(
                self.find_matching_snapshot(&registry, &current_credentials)
                    .map(|(_, snapshot)| snapshot)
                    .ok_or_else(|| {
                        CcrError::ValidationError(
                            "当前 Claude 登录尚未保存；请先运行 `ccr claude auth save <name>` 再切换"
                                .into(),
                        )
                    })?,
            )
        } else {
            None
        };

        let runtime_content = serde_json::to_vec_pretty(&snapshot.credentials)
            .map_err(|e| CcrError::SettingsError(format!("序列化 Claude 凭据失败: {e}")))?;
        self.write_secret(&self.credentials_path(), &runtime_content)?;
        if let Err(update_error) = self.clear_profile_api_key_overrides_if_needed() {
            if let Some(previous_snapshot) = previous_snapshot {
                let previous_content = serde_json::to_vec_pretty(&previous_snapshot.credentials)
                    .map_err(|error| {
                        CcrError::SettingsError(format!("序列化原 Claude 凭据失败: {error}"))
                    })?;
                if let Err(restore_error) =
                    self.write_secret(&self.credentials_path(), &previous_content)
                {
                    return Err(CcrError::SettingsError(format!(
                        "切换账号后更新 settings 失败: {update_error}；恢复原凭据也失败: {restore_error}"
                    )));
                }
            }
            return Err(update_error);
        }

        if let Some(account) = registry.accounts.get_mut(name) {
            account.last_used = Some(Utc::now());
        }
        registry.current_auth = Some(name.to_string());
        self.save_registry(&registry)
    }

    pub fn delete_account(&self, name: &str) -> Result<()> {
        let mut registry = self.load_registry()?;
        if registry.accounts.remove(name).is_none() {
            return Err(CcrError::ResourceNotFound(format!(
                "Claude auth account '{}'",
                name
            )));
        }

        if registry.current_auth.as_deref() == Some(name) {
            registry.current_auth = None;
        }

        let snapshot_path = self.account_snapshot_path(name);
        if snapshot_path.exists() {
            fs::remove_file(snapshot_path)
                .map_err(|e| CcrError::FileIoError(format!("删除账号快照失败: {e}")))?;
        }

        self.save_registry(&registry)
    }

    pub fn get_runtime_summary(&self) -> Result<ClaudeRuntimeSummary> {
        let snapshot = self.read_auth_snapshot()?;
        let platform = ClaudePlatform::new()?;
        let current_profile_name = platform.get_current_profile()?;
        let settings_has_managed_overrides = self.load_settings()?.has_managed_overrides();

        let mut current_profile_provider = None;
        let mut current_profile_auth_mode = None;
        let mut current_profile_auth_source = None;
        let mut api_key_profile_override_active = false;

        if let Some(profile_name) = current_profile_name.as_ref() {
            let profiles = platform.load_profiles()?;
            if let Some(profile) = profiles.get(profile_name) {
                current_profile_provider = profile.provider.clone();
                let auth_mode = Self::effective_auth_mode(profile);
                current_profile_auth_source = Some(Self::profile_auth_source(profile, auth_mode));
                api_key_profile_override_active =
                    matches!(auth_mode, ClaudeProfileAuthMode::ApiKey)
                        && settings_has_managed_overrides;
                current_profile_auth_mode = Some(auth_mode);
            }
        }

        let mode = match current_profile_auth_mode {
            Some(ClaudeProfileAuthMode::Subscription) => {
                if snapshot.runtime_usable {
                    ClaudeRuntimeMode::ProfileWithAuth
                } else {
                    ClaudeRuntimeMode::ProfilePendingAuth
                }
            }
            Some(ClaudeProfileAuthMode::ApiKey) => {
                if api_key_profile_override_active {
                    ClaudeRuntimeMode::ProfileOnly
                } else if snapshot.runtime_usable {
                    ClaudeRuntimeMode::ProfileWithAuth
                } else {
                    ClaudeRuntimeMode::ProfilePendingAuth
                }
            }
            None if snapshot.runtime_usable => ClaudeRuntimeMode::RuntimeOnly,
            None => ClaudeRuntimeMode::Unresolved,
        };

        let official_login_state = snapshot.login_state.clone();
        let login_state = if api_key_profile_override_active {
            ClaudeLoginState::ApiKeyActive
        } else {
            official_login_state.clone()
        };
        let current_login_name = snapshot.current_account_name.clone();

        let current_auth_name = match mode {
            ClaudeRuntimeMode::ProfileWithAuth | ClaudeRuntimeMode::RuntimeOnly => {
                snapshot.current_account_name
            }
            ClaudeRuntimeMode::ProfileOnly
            | ClaudeRuntimeMode::ProfilePendingAuth
            | ClaudeRuntimeMode::Unresolved => None,
        };

        Ok(ClaudeRuntimeSummary {
            mode,
            current_profile_name,
            current_profile_provider,
            current_profile_auth_mode,
            current_profile_auth_source,
            current_login_name,
            official_login_state,
            current_auth_name,
            login_state,
        })
    }

    pub fn resolve_profile_auth_mode(profile: &ProfileConfig) -> ClaudeProfileAuthMode {
        if let Some(raw) = profile
            .platform_data
            .get("auth_mode")
            .and_then(JsonValue::as_str)
        {
            return match raw {
                "subscription" => ClaudeProfileAuthMode::Subscription,
                "api_key" => ClaudeProfileAuthMode::ApiKey,
                _ => Self::infer_profile_auth_mode(profile),
            };
        }

        Self::infer_profile_auth_mode(profile)
    }

    fn infer_profile_auth_mode(profile: &ProfileConfig) -> ClaudeProfileAuthMode {
        if profile
            .provider_type
            .as_deref()
            .is_some_and(|value| value == "third_party_model")
        {
            return ClaudeProfileAuthMode::ApiKey;
        }

        if profile
            .base_url
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
            || profile
                .auth_token
                .as_ref()
                .is_some_and(|value| !value.expose().trim().is_empty())
        {
            ClaudeProfileAuthMode::ApiKey
        } else {
            ClaudeProfileAuthMode::Subscription
        }
    }

    /// 🔎 判定 profile 是否为「API-key 形态」(第三方/中转必然形态)
    ///
    /// 保守规则: `provider_type == third_party_model`, 或 `base_url` 与
    /// `auth_token` 同时非空。
    ///
    /// 刻意**不**把「模型映射字段非空」纳入判定: `ANTHROPIC_DEFAULT_*_MODEL`
    /// 在官方订阅下也可用于钉某个快照, 以此判 api_key 会误伤合法的
    /// 「订阅 + 快照钉选」profile, 并触发 `section.validate()` 失败。
    /// 真实第三方必然带 base_url + auth_token, 已被覆盖。
    pub fn is_api_key_shaped(profile: &ProfileConfig) -> bool {
        fn filled(value: &Option<String>) -> bool {
            value.as_deref().is_some_and(|raw| !raw.trim().is_empty())
        }

        let token_filled = profile
            .auth_token
            .as_ref()
            .is_some_and(|raw| !raw.expose().trim().is_empty());

        profile.provider_type.as_deref() == Some("third_party_model")
            || (filled(&profile.base_url) && token_filled)
    }

    /// 🩹 在 `resolve_profile_auth_mode` 之上叠加自愈
    ///
    /// 当 profile 解析为 subscription 但实为 API-key 形态时, 纠正为 api_key。
    /// 这避免「base_url + auth_token 齐全却被标成 subscription」的 profile 在
    /// apply 时被 `clear_managed_vars()` 静默清空。
    ///
    /// 纯函数、不打日志: warn 由实际纠正点 (`apply_profile` / `normalize_profile`)
    /// 发出, 以免只读渲染路径 (如 `profile_to_json`) 刷屏。
    pub fn effective_auth_mode(profile: &ProfileConfig) -> ClaudeProfileAuthMode {
        let resolved = Self::resolve_profile_auth_mode(profile);
        if matches!(resolved, ClaudeProfileAuthMode::Subscription)
            && Self::is_api_key_shaped(profile)
        {
            return ClaudeProfileAuthMode::ApiKey;
        }
        resolved
    }

    fn profile_auth_source(profile: &ProfileConfig, auth_mode: ClaudeProfileAuthMode) -> String {
        match auth_mode {
            ClaudeProfileAuthMode::Subscription => "subscription".to_string(),
            ClaudeProfileAuthMode::ApiKey => profile
                .provider
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|provider| format!("provider:{provider}"))
                .unwrap_or_else(|| "settings:anthropic_env".to_string()),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestHome;
    use chrono::Duration;
    use serde_json::json;

    struct TestEnv {
        _home: TestHome,
        service: ClaudeAuthService,
    }

    impl TestEnv {
        fn new() -> Self {
            let home = TestHome::new();
            let ccr_claude_dir = home.root().join("platforms").join("claude");
            let claude_dir = home.home().join(".claude");
            let claude_json_path = home.home().join(".claude.json");
            fs::create_dir_all(&ccr_claude_dir).unwrap();
            fs::create_dir_all(&claude_dir).unwrap();

            Self {
                service: ClaudeAuthService::from_parts(
                    ccr_claude_dir,
                    claude_dir,
                    claude_json_path,
                ),
                _home: home,
            }
        }

        fn write_settings(&self, value: JsonValue) {
            fs::write(
                self.service.settings_path(),
                serde_json::to_string_pretty(&value).unwrap(),
            )
            .unwrap();
        }

        fn write_profiles(&self, content: &str) {
            fs::write(self.service.profiles_path(), content).unwrap();
        }

        fn write_credentials(&self, expires_at: DateTime<Utc>) {
            self.write_credentials_for("access-token", expires_at);
        }

        fn write_credentials_for(&self, access_token: &str, expires_at: DateTime<Utc>) {
            let value = json!({
                "claudeAiOauth": {
                    "accessToken": access_token,
                    "refreshToken": format!("refresh-{access_token}"),
                    "expiresAt": expires_at.to_rfc3339(),
                    "subscriptionType": "pro",
                    "rateLimitTier": "default_claude_ai",
                    "scopes": ["user:profile"]
                }
            });
            fs::write(
                self.service.credentials_path(),
                serde_json::to_string_pretty(&value).unwrap(),
            )
            .unwrap();
        }

        fn write_credentials_epoch_millis(&self, expires_at_ms: i64) {
            let value = json!({
                "claudeAiOauth": {
                    "accessToken": "access-token",
                    "refreshToken": "refresh-token",
                    "expiresAt": expires_at_ms,
                    "subscriptionType": "pro",
                    "rateLimitTier": "default_claude_ai"
                }
            });

            fs::write(
                self.service.credentials_path(),
                serde_json::to_string_pretty(&value).unwrap(),
            )
            .unwrap();
        }

        fn write_metadata(&self) {
            self.write_metadata_for("account-123", "user@example.com");
        }

        fn write_metadata_for(&self, account_uuid: &str, email: &str) {
            let value = json!({
                "oauthAccount": {
                    "accountUuid": account_uuid,
                    "emailAddress": email,
                    "billingType": "apple_subscription"
                }
            });
            fs::write(
                &self.service.runtime_paths.state_file,
                serde_json::to_string_pretty(&value).unwrap(),
            )
            .unwrap();
        }
    }

    #[test]
    fn claude_config_dir_controls_auth_runtime_paths() {
        let mut home = TestHome::new_with_home_env();
        let config_dir = home.home().join("claude-custom");
        home.set_env("CLAUDE_CONFIG_DIR", config_dir.as_os_str());
        home.remove_env("CCR_SETTINGS_PATH");
        home.remove_env("CCR_BACKUP_DIR");
        home.remove_env("CLAUDE_JSON_PATH");

        let service = ClaudeAuthService::new().unwrap();
        assert_eq!(service.settings_path(), config_dir.join("settings.json"));
        assert_eq!(
            service.credentials_path(),
            config_dir.join(".credentials.json")
        );
        assert_eq!(
            service.runtime_paths.state_file,
            config_dir.join(".claude.json")
        );
        assert_eq!(
            service.runtime_paths.backups_dir,
            config_dir.join("backups")
        );
    }

    #[test]
    fn test_read_snapshot_handles_missing_files() {
        let env = TestEnv::new();
        let snapshot = env.service.read_auth_snapshot().unwrap();

        assert_eq!(snapshot.login_state, ClaudeLoginState::NotLoggedIn);
        assert!(snapshot.current_info.is_none());
        assert!(!snapshot.runtime_usable);
    }

    #[test]
    fn test_save_switch_delete_account_roundtrip() {
        let env = TestEnv::new();
        env.write_credentials(Utc::now() + Duration::days(14));
        env.write_metadata();

        let saved = env
            .service
            .save_current("work", Some("Work account".to_string()), false)
            .unwrap();
        assert_eq!(saved.account_uuid.as_deref(), Some("account-123"));

        let snapshot_bytes = fs::read(env.service.account_snapshot_path("work")).unwrap();
        let snapshot_text = String::from_utf8(snapshot_bytes).unwrap();
        assert!(snapshot_text.contains("access-token"));
        assert!(snapshot_text.contains("refresh-access-token"));
        let snapshot: ClaudeAuthSnapshotFile = serde_json::from_str(&snapshot_text).unwrap();
        assert!(!format!("{snapshot:?}").contains("access-token"));

        let accounts = env.service.list_accounts().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].name, "work");

        env.service.switch_account("work").unwrap();
        let registry = env.service.load_registry().unwrap();
        assert_eq!(registry.current_auth.as_deref(), Some("work"));

        env.service.delete_account("work").unwrap();
        let registry = env.service.load_registry().unwrap();
        assert!(registry.accounts.is_empty());
        assert!(registry.current_auth.is_none());
    }

    #[test]
    fn test_switch_account_rejects_unsaved_current_credentials_without_leaking_token() {
        let env = TestEnv::new();
        let expires_at = Utc::now() + Duration::days(14);
        env.write_credentials_for("saved-a-token", expires_at);
        env.write_metadata_for("account-a", "a@example.com");
        env.service.save_current("account_a", None, false).unwrap();

        env.write_credentials_for("unsaved-private-token", expires_at);
        let before = fs::read(env.service.credentials_path()).unwrap();
        let error = env.service.switch_account("account_a").unwrap_err();
        let message = error.to_string();

        assert!(message.contains("ccr claude auth save"));
        assert!(!message.contains("unsaved-private-token"));
        assert_eq!(fs::read(env.service.credentials_path()).unwrap(), before);
        assert_eq!(
            env.service.load_registry().unwrap().current_auth.as_deref(),
            Some("account_a")
        );
    }

    #[test]
    fn test_switch_account_rejects_corrupt_current_credentials_without_overwrite() {
        let env = TestEnv::new();
        let expires_at = Utc::now() + Duration::days(14);
        env.write_credentials_for("saved-a-token", expires_at);
        env.write_metadata_for("account-a", "a@example.com");
        env.service.save_current("account_a", None, false).unwrap();

        let corrupt = br#"{"claudeAiOauth": "#.to_vec();
        fs::write(env.service.credentials_path(), &corrupt).unwrap();
        let error = env.service.switch_account("account_a").unwrap_err();

        assert!(error.to_string().contains("解析 Claude 凭据失败"));
        assert_eq!(fs::read(env.service.credentials_path()).unwrap(), corrupt);
    }

    #[test]
    fn test_switch_account_allows_missing_current_credentials() {
        let env = TestEnv::new();
        let expires_at = Utc::now() + Duration::days(14);
        env.write_credentials_for("saved-a-token", expires_at);
        env.write_metadata_for("account-a", "a@example.com");
        env.service.save_current("account_a", None, false).unwrap();
        fs::remove_file(env.service.credentials_path()).unwrap();

        env.service.switch_account("account_a").unwrap();

        let current = env.service.load_current_credentials_strict().unwrap();
        assert_eq!(
            current.claude_ai_oauth.unwrap().access_token.expose(),
            "saved-a-token"
        );
    }

    #[test]
    fn test_switch_account_uses_matching_snapshot_metadata_instead_of_stale_state_file() {
        let env = TestEnv::new();
        let expires_at = Utc::now() + Duration::days(14);

        env.write_credentials_for("access-a", expires_at);
        env.write_metadata_for("account-a", "a@example.com");
        env.service.save_current("account_a", None, false).unwrap();

        env.write_credentials_for("access-b", expires_at);
        env.write_metadata_for("account-b", "b@example.com");
        env.service.save_current("account_b", None, false).unwrap();

        env.service.switch_account("account_a").unwrap();
        env.write_metadata_for("account-a", "a@example.com");
        env.service.switch_account("account_b").unwrap();

        let current_b = env.service.read_auth_snapshot().unwrap();
        assert_eq!(
            current_b.login_state,
            ClaudeLoginState::LoggedInSaved {
                account_name: "account_b".to_string()
            }
        );
        assert_eq!(
            current_b
                .current_info
                .as_ref()
                .and_then(|info| info.email.as_deref()),
            Some("b@example.com")
        );
        assert_eq!(
            current_b
                .current_info
                .as_ref()
                .and_then(|info| info.account_uuid.as_deref()),
            Some("account-b")
        );
        assert_eq!(
            env.service
                .load_current_credentials_strict()
                .unwrap()
                .claude_ai_oauth
                .unwrap()
                .access_token
                .expose(),
            "access-b"
        );

        env.service.switch_account("account_a").unwrap();
        let current_a = env.service.read_auth_snapshot().unwrap();
        assert_eq!(
            current_a
                .current_info
                .as_ref()
                .and_then(|info| info.email.as_deref()),
            Some("a@example.com")
        );
        assert_eq!(
            current_a
                .current_info
                .as_ref()
                .and_then(|info| info.account_uuid.as_deref()),
            Some("account-a")
        );
    }

    #[test]
    fn test_switch_account_restores_previous_credentials_when_settings_update_fails() {
        let env = TestEnv::new();
        let expires_at = Utc::now() + Duration::days(14);

        env.write_credentials_for("restore-a-token", expires_at);
        env.write_metadata_for("account-a", "a@example.com");
        env.service.save_current("account_a", None, false).unwrap();
        env.write_credentials_for("target-b-token", expires_at);
        env.write_metadata_for("account-b", "b@example.com");
        env.service.save_current("account_b", None, false).unwrap();
        env.service.switch_account("account_a").unwrap();

        env.write_profiles(
            r#"
default_config = "api"
current_config = "api"

[api]
base_url = "https://example.com"
auth_token = "profile-token"
provider = "test"
auth_mode = "api_key"
"#,
        );
        fs::write(env.service.settings_path(), b"{ invalid settings").unwrap();
        let before = fs::read(env.service.credentials_path()).unwrap();

        let error = env.service.switch_account("account_b").unwrap_err();
        let message = error.to_string();

        assert!(message.contains("解析设置文件失败"));
        assert!(!message.contains("restore-a-token"));
        assert!(!message.contains("target-b-token"));
        assert_eq!(fs::read(env.service.credentials_path()).unwrap(), before);
        assert_eq!(
            env.service.load_registry().unwrap().current_auth.as_deref(),
            Some("account_a")
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_auth_durable_files_are_owner_only_without_same_dir_backups() {
        use std::os::unix::fs::PermissionsExt;

        let env = TestEnv::new();
        env.write_credentials(Utc::now() + Duration::days(14));
        env.write_metadata();
        env.service.save_current("work", None, false).unwrap();
        env.service.switch_account("work").unwrap();

        for path in [
            env.service.credentials_path(),
            env.service.account_snapshot_path("work"),
            env.service.registry_path(),
        ] {
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        assert!(
            fs::read_dir(&env.service.runtime_paths.config_dir)
                .unwrap()
                .all(|entry| !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".bak"))
        );
        assert!(
            fs::read_dir(env.service.auth_storage_dir())
                .unwrap()
                .all(|entry| !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".bak"))
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_save_and_switch_reject_file_credentials() {
        let env = TestEnv::new();
        let save_error = env.service.save_current("work", None, false).unwrap_err();
        let switch_error = env.service.switch_account("work").unwrap_err();

        assert!(save_error.to_string().contains("Keychain"));
        assert!(switch_error.to_string().contains("Keychain"));
        assert!(!env.service.credentials_path().exists());
    }

    #[test]
    fn test_build_account_items_distinguishes_logged_in_from_effective_auth() {
        let env = TestEnv::new();
        env.write_credentials(Utc::now() + Duration::days(14));
        env.write_metadata();
        env.service
            .save_current("work", Some("Work account".to_string()), false)
            .unwrap();

        let snapshot = env.service.read_auth_snapshot().unwrap();
        let runtime_summary = ClaudeRuntimeSummary {
            mode: ClaudeRuntimeMode::ProfileOnly,
            current_profile_name: Some("main_pro".to_string()),
            current_profile_provider: Some("pro".to_string()),
            current_profile_auth_mode: Some(ClaudeProfileAuthMode::ApiKey),
            current_profile_auth_source: Some("provider:pro".to_string()),
            current_login_name: Some("work".to_string()),
            official_login_state: snapshot.login_state.clone(),
            current_auth_name: None,
            login_state: ClaudeLoginState::ApiKeyActive,
        };

        let accounts = env
            .service
            .build_account_items(&snapshot, Some(&runtime_summary))
            .unwrap();

        assert_eq!(accounts.len(), 1);
        assert!(accounts[0].is_logged_in);
        assert!(!accounts[0].is_current);
    }

    #[test]
    fn test_read_snapshot_supports_epoch_millis_expiry() {
        let env = TestEnv::new();
        let expires_at = Utc::now() + Duration::days(14);
        env.write_credentials_epoch_millis(expires_at.timestamp_millis());
        env.write_metadata();

        let snapshot = env.service.read_auth_snapshot().unwrap();
        let info = snapshot.current_info.unwrap();

        assert_eq!(info.subscription_type.as_deref(), Some("pro"));
        assert_eq!(info.rate_limit_tier.as_deref(), Some("default_claude_ai"));
        assert_eq!(
            info.expires_at.map(|dt| dt.timestamp_millis()),
            Some(expires_at.timestamp_millis())
        );
        assert!(snapshot.runtime_usable);
    }

    #[test]
    fn test_resolve_profile_auth_mode_prefers_explicit_platform_data() {
        let mut profile = ProfileConfig::new();
        profile
            .platform_data
            .insert("auth_mode".into(), json!("subscription"));
        profile.base_url = Some("https://example.com".to_string());
        profile.auth_token = Some(ccr_core::Secret::from("sk-test"));

        assert_eq!(
            ClaudeAuthService::resolve_profile_auth_mode(&profile),
            ClaudeProfileAuthMode::Subscription
        );
    }

    #[test]
    fn test_resolve_profile_auth_mode_infers_history_profiles() {
        let profile = ProfileConfig::new();
        assert_eq!(
            ClaudeAuthService::resolve_profile_auth_mode(&profile),
            ClaudeProfileAuthMode::Subscription
        );

        let profile = ProfileConfig::new()
            .with_base_url("https://example.com".to_string())
            .with_auth_token("sk-test".to_string());
        assert_eq!(
            ClaudeAuthService::resolve_profile_auth_mode(&profile),
            ClaudeProfileAuthMode::ApiKey
        );
    }

    #[test]
    fn test_switch_account_clears_mismarked_profile_overrides_and_reactivates_official_auth() {
        let env = TestEnv::new();
        env.write_credentials(Utc::now() + Duration::days(14));
        env.write_metadata();
        env.service.save_current("work", None, false).unwrap();
        env.write_profiles(
            r#"
default_config = "anyrouter"
current_config = "anyrouter"

[anyrouter]
base_url = "https://example.com"
auth_token = "sk-profile"
provider = "anyrouter"
auth_mode = "subscription"
"#,
        );
        env.write_settings(json!({
            "env": {
                "ANTHROPIC_BASE_URL": "https://example.com",
                "ANTHROPIC_AUTH_TOKEN": "sk-profile",
                "ANTHROPIC_MODEL": "claude-3-7-sonnet",
                "CLAUDE_CODE_SUBAGENT_MODEL": "claude-haiku",
                "CLAUDE_CODE_EFFORT_LEVEL": "max",
                "CLAUDE_CODE_AUTO_COMPACT_WINDOW": "1000000",
                "API_TIMEOUT_MS": "3000000",
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
                "ANTHROPIC_API_KEY": "user-api-key",
                "ANTHROPIC_CUSTOM_HEADERS": "X-User: keep"
            }
        }));

        let summary = env.service.get_runtime_summary().unwrap();
        assert_eq!(summary.mode, ClaudeRuntimeMode::ProfileOnly);
        assert_eq!(
            summary.current_profile_auth_mode,
            Some(ClaudeProfileAuthMode::ApiKey)
        );

        env.service.switch_account("work").unwrap();

        let settings = env.service.load_settings().unwrap();
        assert!(!settings.has_managed_overrides());
        assert_eq!(
            settings.env.get("ANTHROPIC_API_KEY").map(String::as_str),
            Some("user-api-key")
        );
        assert_eq!(
            settings
                .env
                .get("ANTHROPIC_CUSTOM_HEADERS")
                .map(String::as_str),
            Some("X-User: keep")
        );

        let summary = env.service.get_runtime_summary().unwrap();
        assert_eq!(summary.mode, ClaudeRuntimeMode::ProfileWithAuth);
        assert_eq!(
            summary.login_state,
            ClaudeLoginState::LoggedInSaved {
                account_name: "work".to_string()
            }
        );
        assert_eq!(
            summary.current_profile_auth_mode,
            Some(ClaudeProfileAuthMode::ApiKey)
        );
        assert_eq!(summary.current_auth_name.as_deref(), Some("work"));
        assert_eq!(summary.auth_label(), "Official / work");
    }

    #[test]
    fn test_effective_auth_mode_corrects_mismarked_third_party() {
        let mut profile = ProfileConfig::new()
            .with_base_url("https://chy.example.com".to_string())
            .with_auth_token("sk-chy".to_string());
        profile
            .platform_data
            .insert("auth_mode".into(), json!("subscription"));

        // 字面解析仍是 subscription, 但 effective 纠正为 api_key
        assert_eq!(
            ClaudeAuthService::resolve_profile_auth_mode(&profile),
            ClaudeProfileAuthMode::Subscription
        );
        assert_eq!(
            ClaudeAuthService::effective_auth_mode(&profile),
            ClaudeProfileAuthMode::ApiKey
        );
    }

    #[test]
    fn test_effective_auth_mode_keeps_subscription_snapshot_pin() {
        // 官方订阅 + 仅模型映射 (无 base_url/token): 不应被误纠正为 api_key
        let mut profile = ProfileConfig::new();
        profile.default_opus_model = Some("claude-opus-4-5-20251101".to_string());
        profile
            .platform_data
            .insert("auth_mode".into(), json!("subscription"));

        assert!(!ClaudeAuthService::is_api_key_shaped(&profile));
        assert_eq!(
            ClaudeAuthService::effective_auth_mode(&profile),
            ClaudeProfileAuthMode::Subscription
        );
    }

    #[test]
    fn test_is_api_key_shaped_detects_third_party_provider_type() {
        let mut profile = ProfileConfig::new();
        profile.provider_type = Some("third_party_model".to_string());
        assert!(ClaudeAuthService::is_api_key_shaped(&profile));
    }
}
