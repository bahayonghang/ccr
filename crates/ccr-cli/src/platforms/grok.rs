//! Grok Build profile switching backed by `$GROK_HOME/config.toml`.

use crate::managers::PlatformConfigManager;
use crate::models::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
use ccr_config::platforms::base;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::{
    BackupPolicy, FileLock, LockManager, VersionedWriteOutcome, WriteOptions,
    content_version_token, write_guarded_versioned,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const GROK_MANAGED_MODEL_KEY: &str = "custom";
const PROFILE_ENTRY_CONFIG_STATE_FILE: &str = "profile_entry_config_state.json";
const GROK_PROFILE_OPERATION_LOCK: &str = "grok_profile_operation";
const GROK_PROFILE_OPERATION_LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const GROK_EDITABLE_FIELDS: &[&str] = &[
    "description",
    "base_url",
    "auth_token",
    "api_key",
    "model",
    "provider",
    "provider_type",
    "account",
    "tags",
    "api_backend",
    "env_key",
    "context_window",
    "supports_backend_search",
    "reasoning_effort",
];

/// Credential source selected for one Grok profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrokProfileAuthMode {
    /// Store an inline API key in `[model.custom].api_key`.
    InlineApiKey,
    /// Resolve the key from the named environment variable.
    EnvKey,
    /// Let Grok use its own session or global xAI environment variable.
    Session,
}

impl GrokProfileAuthMode {
    /// Stable identifier used by CLI and TUI presentation.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InlineApiKey => "inline_api_key",
            Self::EnvKey => "env_key",
            Self::Session => "session",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfileEntryConfigState {
    exists: bool,
    content: Option<String>,
    original_custom_model: Option<toml::Value>,
    original_default_model: Option<String>,
    #[serde(default)]
    original_default_reasoning_effort: Option<toml::Value>,
}

/// Grok Build platform implementation.
pub struct GrokPlatform {
    paths: PlatformPaths,
    config_path: PathBuf,
}

impl GrokPlatform {
    /// Fields accepted by the shared profile command surface.
    pub fn editable_fields() -> &'static [&'static str] {
        GROK_EDITABLE_FIELDS
    }

    /// Normalize one of Grok Build's canonical reasoning-effort values.
    pub fn normalize_reasoning_effort(value: &str) -> Result<String> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(CcrError::ValidationError(
                "Grok reasoning_effort 不能为空字符串".into(),
            ));
        }

        let canonical = trimmed.to_ascii_lowercase();
        if matches!(
            canonical.as_str(),
            "none" | "minimal" | "low" | "medium" | "high" | "xhigh" | "max"
        ) {
            Ok(canonical)
        } else {
            Err(CcrError::ValidationError(format!(
                "Grok reasoning_effort 不受支持: {trimmed}; 允许值为 none、minimal、low、medium、high、xhigh、max"
            )))
        }
    }

    /// Create a platform using the process environment.
    pub fn new() -> Result<Self> {
        let paths = PlatformPaths::new(Platform::Grok)?;
        let grok_home = std::env::var_os("GROK_HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|home| home.join(".grok")))
            .ok_or_else(|| CcrError::ConfigError("无法获取 Grok 配置目录".into()))?;
        Ok(Self::from_parts(paths, grok_home.join("config.toml")))
    }

    /// Construct a platform with explicit paths for isolated callers and tests.
    pub fn from_parts(paths: PlatformPaths, config_path: PathBuf) -> Self {
        Self { paths, config_path }
    }

    /// Resolve a profile's credential source without exposing the credential.
    pub fn profile_auth_mode(profile: &ProfileConfig) -> Result<GrokProfileAuthMode> {
        let has_inline = Self::profile_inline_api_key(profile)?.is_some();
        let has_env = Self::profile_env_key(profile)?.is_some();

        match (has_inline, has_env) {
            (true, true) => Err(CcrError::ValidationError(
                "Grok profile 的 api_key/auth_token 与 env_key 不能同时设置".into(),
            )),
            (true, false) => Ok(GrokProfileAuthMode::InlineApiKey),
            (false, true) => Ok(GrokProfileAuthMode::EnvKey),
            (false, false) => Ok(GrokProfileAuthMode::Session),
        }
    }

    /// Remove credentials and query/userinfo components before displaying a URL.
    pub fn safe_base_url_for_display(value: &str) -> String {
        let without_fragment = value.split('#').next().unwrap_or(value);
        let without_query = without_fragment
            .split('?')
            .next()
            .unwrap_or(without_fragment);
        let (scheme, remainder) = without_query
            .split_once("://")
            .map_or((None, without_query), |(scheme, remainder)| {
                (Some(scheme), remainder)
            });
        let host_and_path = remainder
            .rsplit_once('@')
            .map(|(_, host_and_path)| host_and_path)
            .unwrap_or(remainder);
        scheme.map_or_else(
            || host_and_path.to_string(),
            |scheme| format!("{scheme}://{host_and_path}"),
        )
    }

    /// Restore the entry configuration and leave CCR profile mode.
    pub fn clear_active_profile_runtime(&self) -> Result<()> {
        let _operation_lock = self.lock_profile_operation()?;
        let state = self.load_entry_state()?;
        if state.is_none() {
            let has_active_intent = self.current_profile_from_registry()?.is_some()
                || self.fallback_current_profile_from_file()?.is_some();
            if has_active_intent || self.runtime_has_managed_shape()? {
                return Err(CcrError::ConfigError(
                    "Grok 入口配置状态缺失，拒绝执行 off 以避免遗留或误删凭据；请先备份 config.toml 并手工恢复 [model.custom] 与 [models].default"
                        .into(),
                ));
            }
        }
        if let Some(state) = state {
            self.update_runtime_config(|config| Self::restore_entry_state(config, &state))?;
        }
        self.clear_profiles_current_config()?;
        self.clear_current_profile_registry()?;
        self.remove_entry_state()?;
        Ok(())
    }

    fn registry_lock_dir(&self) -> PathBuf {
        self.paths.root.join(".locks")
    }

    fn lock_profile_operation(&self) -> Result<FileLock> {
        self.lock_profile_operation_with_timeout(GROK_PROFILE_OPERATION_LOCK_TIMEOUT)
    }

    fn lock_profile_operation_with_timeout(&self, timeout: Duration) -> Result<FileLock> {
        LockManager::with_default_path()?.lock_resource(GROK_PROFILE_OPERATION_LOCK, timeout)
    }

    fn entry_state_path(&self) -> PathBuf {
        self.paths
            .platform_dir
            .join(PROFILE_ENTRY_CONFIG_STATE_FILE)
    }

    fn load_profiles_from_file(&self) -> Result<IndexMap<String, ProfileConfig>> {
        base::load_profiles_from_toml(&self.paths.profiles_file)
    }

    fn save_profiles_to_file(&self, profiles: &IndexMap<String, ProfileConfig>) -> Result<()> {
        base::save_profiles_to_toml(&self.paths.profiles_file, profiles, "grok", &self.paths)
    }

    fn trimmed(value: Option<&String>) -> Option<String> {
        value
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    }

    fn profile_env_key(profile: &ProfileConfig) -> Result<Option<String>> {
        match profile.platform_data.get("env_key") {
            None | Some(JsonValue::Null) => Ok(None),
            Some(JsonValue::String(value)) if value.trim().is_empty() => Ok(None),
            Some(JsonValue::String(value)) => Ok(Some(value.trim().to_string())),
            Some(JsonValue::Array(_)) => Err(CcrError::ValidationError(
                "Grok env_key MVP 仅支持单个环境变量名，不支持数组".into(),
            )),
            Some(_) => Err(CcrError::ValidationError(
                "Grok env_key 必须是字符串".into(),
            )),
        }
    }

    fn profile_api_key(profile: &ProfileConfig) -> Result<Option<String>> {
        match profile.platform_data.get("api_key") {
            None | Some(JsonValue::Null) => Ok(None),
            Some(JsonValue::String(value)) if value.trim().is_empty() => Err(
                CcrError::ValidationError("Grok api_key 不能为空字符串".into()),
            ),
            Some(JsonValue::String(value)) => Ok(Some(value.trim().to_string())),
            Some(_) => Err(CcrError::ValidationError(
                "Grok api_key 必须是字符串".into(),
            )),
        }
    }

    fn profile_inline_api_key(profile: &ProfileConfig) -> Result<Option<String>> {
        let auth_token = profile
            .auth_token
            .as_ref()
            .map(|secret| secret.expose().trim())
            .filter(|value| !value.is_empty());
        let api_key = Self::profile_api_key(profile)?;

        match (auth_token, api_key) {
            (Some(_), Some(_)) => Err(CcrError::ValidationError(
                "Grok profile 的 api_key 与兼容字段 auth_token 不能同时设置".into(),
            )),
            (Some(value), None) => Ok(Some(value.to_string())),
            (None, api_key) => Ok(api_key),
        }
    }

    fn profile_api_backend(profile: &ProfileConfig) -> Result<String> {
        match profile.platform_data.get("api_backend") {
            None | Some(JsonValue::Null) => Ok("responses".into()),
            Some(JsonValue::String(value)) => {
                let normalized = value.trim().to_ascii_lowercase();
                if matches!(
                    normalized.as_str(),
                    "chat_completions" | "responses" | "messages"
                ) {
                    Ok(normalized)
                } else {
                    Err(CcrError::ValidationError(
                        "Grok api_backend 仅支持 chat_completions、responses 或 messages".into(),
                    ))
                }
            }
            Some(_) => Err(CcrError::ValidationError(
                "Grok api_backend 必须是字符串".into(),
            )),
        }
    }

    fn profile_context_window(profile: &ProfileConfig) -> Result<Option<i64>> {
        match profile.platform_data.get("context_window") {
            None | Some(JsonValue::Null) => Ok(None),
            Some(JsonValue::Number(value)) => value
                .as_u64()
                .filter(|value| *value > 0)
                .and_then(|value| i64::try_from(value).ok())
                .map(Some)
                .ok_or_else(|| {
                    CcrError::ValidationError("Grok context_window 必须是正整数".into())
                }),
            Some(JsonValue::String(value)) => value
                .trim()
                .parse::<i64>()
                .ok()
                .filter(|value| *value > 0)
                .map(Some)
                .ok_or_else(|| {
                    CcrError::ValidationError("Grok context_window 必须是正整数".into())
                }),
            Some(_) => Err(CcrError::ValidationError(
                "Grok context_window 必须是正整数".into(),
            )),
        }
    }

    fn profile_backend_search(profile: &ProfileConfig) -> Result<Option<bool>> {
        match profile.platform_data.get("supports_backend_search") {
            None | Some(JsonValue::Null) => Ok(None),
            Some(JsonValue::Bool(value)) => Ok(Some(*value)),
            Some(_) => Err(CcrError::ValidationError(
                "Grok supports_backend_search 必须是布尔值".into(),
            )),
        }
    }

    fn profile_reasoning_effort(profile: &ProfileConfig) -> Result<Option<String>> {
        match profile.platform_data.get("reasoning_effort") {
            None | Some(JsonValue::Null) => Ok(None),
            Some(JsonValue::String(value)) => Self::normalize_reasoning_effort(value).map(Some),
            Some(_) => Err(CcrError::ValidationError(
                "Grok reasoning_effort 必须是字符串".into(),
            )),
        }
    }

    fn is_valid_env_key(value: &str) -> bool {
        let mut chars = value.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        (first == '_' || first.is_ascii_alphabetic())
            && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
    }

    fn is_official_profile(profile: &ProfileConfig) -> bool {
        match profile
            .provider_type
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("official" | "official_relay") => true,
            Some("third_party" | "third_party_model") => false,
            _ => Self::trimmed(profile.base_url.as_ref()).is_none(),
        }
    }

    fn load_config_value(path: &Path) -> Result<(toml::Value, String)> {
        match fs::read(path) {
            Ok(bytes) => {
                let token = content_version_token(&bytes);
                let content = String::from_utf8(bytes).map_err(|error| {
                    CcrError::ConfigFormatInvalid(format!(
                        "Grok config.toml 不是有效 UTF-8: {error}"
                    ))
                })?;
                let config = toml::from_str(&content).map_err(|_| {
                    CcrError::ConfigFormatInvalid(format!(
                        "解析 Grok config.toml 失败 {}：文件包含无效 TOML",
                        path.display()
                    ))
                })?;
                Ok((config, token))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok((toml::Value::Table(Default::default()), String::new()))
            }
            Err(error) => Err(CcrError::ConfigError(format!(
                "读取 Grok config.toml 失败 {}: {error}",
                path.display()
            ))),
        }
    }

    fn root_table_mut(config: &mut toml::Value) -> Result<&mut toml::Table> {
        config.as_table_mut().ok_or_else(|| {
            CcrError::ConfigFormatInvalid("Grok config.toml 顶层必须是 TOML table".into())
        })
    }

    fn table_mut<'a>(root: &'a mut toml::Table, key: &str) -> Result<&'a mut toml::Table> {
        let value = root
            .entry(key.to_string())
            .or_insert_with(|| toml::Value::Table(Default::default()));
        value.as_table_mut().ok_or_else(|| {
            CcrError::ConfigFormatInvalid(format!("Grok config.toml 的 [{key}] 必须是 table"))
        })
    }

    fn capture_entry_state(&self) -> Result<()> {
        let state_path = self.entry_state_path();
        if state_path.exists() {
            return Ok(());
        }

        self.paths.ensure_directories()?;
        let (exists, content, config) = match fs::read_to_string(&self.config_path) {
            Ok(content) => {
                let config: toml::Value = toml::from_str(&content).map_err(|_| {
                    CcrError::ConfigFormatInvalid(format!(
                        "解析 Grok 入口 config.toml 失败 {}：文件包含无效 TOML",
                        self.config_path.display()
                    ))
                })?;
                (true, Some(content), config)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                (false, None, toml::Value::Table(Default::default()))
            }
            Err(error) => {
                return Err(CcrError::ConfigError(format!(
                    "读取 Grok 入口 config.toml 失败 {}: {error}",
                    self.config_path.display()
                )));
            }
        };

        let original_custom_model = config
            .get("model")
            .and_then(toml::Value::as_table)
            .and_then(|models| models.get(GROK_MANAGED_MODEL_KEY))
            .cloned();
        let original_default_model = config
            .get("models")
            .and_then(toml::Value::as_table)
            .and_then(|models| models.get("default"))
            .and_then(toml::Value::as_str)
            .map(str::to_string);
        let original_default_reasoning_effort = config
            .get("models")
            .and_then(toml::Value::as_table)
            .and_then(|models| models.get("default_reasoning_effort"))
            .cloned();
        let state = ProfileEntryConfigState {
            exists,
            content,
            original_custom_model,
            original_default_model,
            original_default_reasoning_effort,
        };
        let serialized = serde_json::to_string_pretty(&state).map_err(|error| {
            CcrError::ConfigError(format!("序列化 Grok 入口配置状态失败: {error}"))
        })?;
        match self.write_entry_state_if_absent(serialized.as_bytes())? {
            VersionedWriteOutcome::Written | VersionedWriteOutcome::Conflict => Ok(()),
        }
    }

    fn write_entry_state_if_absent(&self, serialized: &[u8]) -> Result<VersionedWriteOutcome> {
        write_guarded_versioned(
            &self.entry_state_path(),
            serialized,
            "",
            &WriteOptions {
                backup: BackupPolicy::None,
                secret: true,
                ..Default::default()
            },
        )
    }

    fn load_entry_state(&self) -> Result<Option<ProfileEntryConfigState>> {
        let state_path = self.entry_state_path();
        let raw = match fs::read_to_string(&state_path) {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(CcrError::ConfigError(format!(
                    "读取 Grok 入口配置状态失败 {}: {error}",
                    state_path.display()
                )));
            }
        };
        serde_json::from_str(&raw).map(Some).map_err(|error| {
            CcrError::ConfigFormatInvalid(format!(
                "解析 Grok 入口配置状态失败 {}: {error}",
                state_path.display()
            ))
        })
    }

    fn remove_entry_state(&self) -> Result<()> {
        let path = self.entry_state_path();
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(CcrError::ConfigError(format!(
                "删除 Grok 入口配置状态失败 {}: {error}",
                path.display()
            ))),
        }
    }

    fn restore_entry_state(
        config: &mut toml::Value,
        state: &ProfileEntryConfigState,
    ) -> Result<()> {
        let root = Self::root_table_mut(config)?;

        match state.original_custom_model.as_ref() {
            Some(original) => {
                Self::table_mut(root, "model")?
                    .insert(GROK_MANAGED_MODEL_KEY.into(), original.clone());
            }
            None => {
                if let Some(models) = root.get_mut("model") {
                    let models = models.as_table_mut().ok_or_else(|| {
                        CcrError::ConfigFormatInvalid(
                            "Grok config.toml 的 [model] 必须是 table".into(),
                        )
                    })?;
                    models.remove(GROK_MANAGED_MODEL_KEY);
                    if models.is_empty() {
                        root.remove("model");
                    }
                }
            }
        }

        match state.original_default_model.as_ref() {
            Some(original) => {
                Self::table_mut(root, "models")?
                    .insert("default".into(), toml::Value::String(original.clone()));
            }
            None => Self::remove_default_model(root)?,
        }
        Self::restore_default_reasoning_effort(root, state)?;
        Ok(())
    }

    fn remove_default_model(root: &mut toml::Table) -> Result<()> {
        if let Some(models) = root.get_mut("models") {
            let models = models.as_table_mut().ok_or_else(|| {
                CcrError::ConfigFormatInvalid("Grok config.toml 的 [models] 必须是 table".into())
            })?;
            models.remove("default");
            if models.is_empty() {
                root.remove("models");
            }
        }
        Ok(())
    }

    fn restore_default_reasoning_effort(
        root: &mut toml::Table,
        state: &ProfileEntryConfigState,
    ) -> Result<()> {
        let original = state
            .original_default_reasoning_effort
            .clone()
            .or_else(|| Self::legacy_default_reasoning_effort(state));
        match original {
            Some(original) => {
                Self::table_mut(root, "models")?
                    .insert("default_reasoning_effort".into(), original);
            }
            None => {
                if let Some(models) = root.get_mut("models") {
                    let models = models.as_table_mut().ok_or_else(|| {
                        CcrError::ConfigFormatInvalid(
                            "Grok config.toml 的 [models] 必须是 table".into(),
                        )
                    })?;
                    models.remove("default_reasoning_effort");
                    if models.is_empty() {
                        root.remove("models");
                    }
                }
            }
        }
        Ok(())
    }

    fn legacy_default_reasoning_effort(state: &ProfileEntryConfigState) -> Option<toml::Value> {
        let content = state.content.as_deref()?;
        let config = toml::from_str::<toml::Value>(content).ok()?;
        config
            .get("models")?
            .as_table()?
            .get("default_reasoning_effort")
            .cloned()
    }

    fn apply_default_reasoning_effort(
        root: &mut toml::Table,
        effort: Option<&str>,
        state: &ProfileEntryConfigState,
    ) -> Result<()> {
        match effort {
            Some(effort) => {
                Self::table_mut(root, "models")?.insert(
                    "default_reasoning_effort".into(),
                    toml::Value::String(effort.to_string()),
                );
                Ok(())
            }
            None => Self::restore_default_reasoning_effort(root, state),
        }
    }

    fn apply_profile_to_config(
        config: &mut toml::Value,
        name: &str,
        profile: &ProfileConfig,
        state: &ProfileEntryConfigState,
    ) -> Result<()> {
        let reasoning_effort = Self::profile_reasoning_effort(profile)?;
        if Self::is_official_profile(profile) {
            Self::restore_entry_state(config, state)?;
            let root = Self::root_table_mut(config)?;
            if let Some(model) = Self::trimmed(profile.model.as_ref()) {
                Self::table_mut(root, "models")?
                    .insert("default".into(), toml::Value::String(model));
            } else {
                Self::remove_default_model(root)?;
            }
            Self::apply_default_reasoning_effort(root, reasoning_effort.as_deref(), state)?;
            return Ok(());
        }

        let auth_mode = Self::profile_auth_mode(profile)?;
        let base_url = Self::trimmed(profile.base_url.as_ref())
            .ok_or_else(|| CcrError::ValidationError("Grok 第三方 profile 缺少 base_url".into()))?;
        let model = Self::trimmed(profile.model.as_ref())
            .ok_or_else(|| CcrError::ValidationError("Grok 第三方 profile 缺少 model".into()))?;
        let mut managed = toml::Table::new();
        managed.insert("model".into(), toml::Value::String(model));
        managed.insert("base_url".into(), toml::Value::String(base_url));
        managed.insert(
            "name".into(),
            toml::Value::String(
                Self::trimmed(profile.description.as_ref()).unwrap_or_else(|| name.to_string()),
            ),
        );
        managed.insert(
            "api_backend".into(),
            toml::Value::String(Self::profile_api_backend(profile)?),
        );
        match auth_mode {
            GrokProfileAuthMode::InlineApiKey => {
                let api_key = Self::profile_inline_api_key(profile)?.ok_or_else(|| {
                    CcrError::ValidationError("Grok inline profile 缺少 api_key".into())
                })?;
                managed.insert("api_key".into(), toml::Value::String(api_key));
            }
            GrokProfileAuthMode::EnvKey => {
                let env_key = Self::profile_env_key(profile)?.ok_or_else(|| {
                    CcrError::ValidationError("Grok env_key profile 缺少 env_key".into())
                })?;
                managed.insert("env_key".into(), toml::Value::String(env_key));
            }
            GrokProfileAuthMode::Session => {
                return Err(CcrError::ValidationError(
                    "Grok 第三方 profile 必须设置 api_key、auth_token 或 env_key".into(),
                ));
            }
        }
        if let Some(context_window) = Self::profile_context_window(profile)? {
            managed.insert(
                "context_window".into(),
                toml::Value::Integer(context_window),
            );
        }
        if let Some(search) = Self::profile_backend_search(profile)? {
            managed.insert(
                "supports_backend_search".into(),
                toml::Value::Boolean(search),
            );
        }
        if let Some(reasoning_effort) = reasoning_effort.as_deref() {
            managed.insert(
                "supports_reasoning_effort".into(),
                toml::Value::Boolean(true),
            );
            managed.insert(
                "reasoning_effort".into(),
                toml::Value::String(reasoning_effort.to_string()),
            );
        }

        let root = Self::root_table_mut(config)?;
        Self::table_mut(root, "model")?
            .insert(GROK_MANAGED_MODEL_KEY.into(), toml::Value::Table(managed));
        Self::table_mut(root, "models")?.insert(
            "default".into(),
            toml::Value::String(GROK_MANAGED_MODEL_KEY.into()),
        );
        Self::apply_default_reasoning_effort(root, reasoning_effort.as_deref(), state)?;
        Ok(())
    }

    fn runtime_has_managed_shape(&self) -> Result<bool> {
        if !self.config_path.exists() {
            return Ok(false);
        }
        let (config, _) = Self::load_config_value(&self.config_path)?;
        let Some(custom) = config
            .get("model")
            .and_then(toml::Value::as_table)
            .and_then(|models| models.get(GROK_MANAGED_MODEL_KEY))
            .and_then(toml::Value::as_table)
        else {
            return Ok(false);
        };
        Ok(custom.contains_key("model")
            && custom.contains_key("base_url")
            && custom.contains_key("api_backend")
            && (custom.contains_key("api_key") || custom.contains_key("env_key")))
    }

    fn update_runtime_config<M>(&self, mutate: M) -> Result<()>
    where
        M: FnMut(&mut toml::Value) -> Result<()>,
    {
        self.update_runtime_config_with_hook(mutate, |_, _| Ok(()))
    }

    fn update_runtime_config_with_hook<M, H>(
        &self,
        mut mutate: M,
        mut before_write: H,
    ) -> Result<()>
    where
        M: FnMut(&mut toml::Value) -> Result<()>,
        H: FnMut(usize, &Path) -> Result<()>,
    {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                CcrError::ConfigError(format!(
                    "创建 Grok 配置目录失败 {}: {error}",
                    parent.display()
                ))
            })?;
        }

        for attempt in 0..2 {
            let (mut config, token) = Self::load_config_value(&self.config_path)?;
            mutate(&mut config)?;
            let content = toml::to_string_pretty(&config).map_err(|error| {
                CcrError::ConfigError(format!("序列化 Grok config.toml 失败: {error}"))
            })?;
            before_write(attempt, &self.config_path)?;
            let outcome = write_guarded_versioned(
                &self.config_path,
                content.as_bytes(),
                &token,
                &WriteOptions {
                    secret: true,
                    ..Default::default()
                },
            )?;
            match outcome {
                VersionedWriteOutcome::Written => return Ok(()),
                VersionedWriteOutcome::Conflict if attempt == 0 => continue,
                VersionedWriteOutcome::Conflict => {
                    return Err(CcrError::ValidationError(
                        "Grok config.toml 被并发修改，请重试".into(),
                    ));
                }
            }
        }
        Err(CcrError::ValidationError(
            "Grok config.toml 被并发修改，请重试".into(),
        ))
    }

    fn clear_profiles_current_config(&self) -> Result<()> {
        if !self.paths.profiles_file.exists() {
            return Ok(());
        }
        for attempt in 0..2 {
            let (mut value, token) = Self::load_config_value(&self.paths.profiles_file)?;
            Self::root_table_mut(&mut value)?
                .insert("current_config".into(), toml::Value::String(String::new()));
            let content = toml::to_string_pretty(&value).map_err(|error| {
                CcrError::ConfigError(format!("序列化 Grok profiles.toml 失败: {error}"))
            })?;
            let outcome = write_guarded_versioned(
                &self.paths.profiles_file,
                content.as_bytes(),
                &token,
                &WriteOptions {
                    backup: BackupPolicy::Dir {
                        dir: self.paths.backups_dir.clone(),
                        prefix: "profiles".into(),
                    },
                    secret: true,
                    ..Default::default()
                },
            )?;
            match outcome {
                VersionedWriteOutcome::Written => return Ok(()),
                VersionedWriteOutcome::Conflict if attempt == 0 => continue,
                VersionedWriteOutcome::Conflict => {
                    return Err(CcrError::ValidationError(
                        "Grok profiles.toml 被并发修改，请重试".into(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn clear_current_profile_registry(&self) -> Result<()> {
        if !self.paths.registry_file.exists() {
            return Ok(());
        }
        base::clear_registry_current_profile_with_paths(
            &self.paths.registry_file,
            &self.registry_lock_dir(),
            "grok",
        )
    }

    fn current_profile_from_registry(&self) -> Result<Option<String>> {
        if !self.paths.registry_file.exists() {
            return Ok(None);
        }
        let manager = PlatformConfigManager::new(&self.paths.registry_file);
        let config = manager.load()?;
        Ok(config
            .get_platform("grok")
            .ok()
            .and_then(|entry| entry.current_profile.clone()))
    }

    fn fallback_current_profile_from_file(&self) -> Result<Option<String>> {
        if !self.paths.profiles_file.exists() {
            return Ok(None);
        }
        let (value, _) = Self::load_config_value(&self.paths.profiles_file)?;
        let current = value
            .get("current_config")
            .and_then(toml::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        Ok(current.map(str::to_string))
    }

    fn runtime_matches_profile(&self, name: &str, profile: &ProfileConfig) -> Result<bool> {
        self.validate_profile(profile)?;
        let (mut expected, _) = Self::load_config_value(&self.config_path)?;
        let current = expected.clone();
        let Some(state) = self.load_entry_state()? else {
            return Ok(false);
        };
        Self::apply_profile_to_config(&mut expected, name, profile, &state)?;
        Ok(expected == current)
    }

    fn stable_current_profile(&self) -> Result<Option<String>> {
        let profiles = self.load_profiles()?;
        if let Some(current) = self.current_profile_from_registry()? {
            let Some(profile) = profiles.get(&current) else {
                self.clear_current_profile_registry()?;
                return Ok(None);
            };
            if self.runtime_matches_profile(&current, profile)? {
                return Ok(Some(current));
            }
            self.clear_current_profile_registry()?;
            return Ok(None);
        }

        let Some(current) = self.fallback_current_profile_from_file()? else {
            return Ok(None);
        };
        let Some(profile) = profiles.get(&current) else {
            return Ok(None);
        };
        Ok(self
            .runtime_matches_profile(&current, profile)?
            .then_some(current))
    }
}

impl PlatformConfig for GrokPlatform {
    fn platform_name(&self) -> &str {
        "grok"
    }

    fn platform_type(&self) -> Platform {
        Platform::Grok
    }

    fn load_profiles(&self) -> Result<IndexMap<String, ProfileConfig>> {
        self.load_profiles_from_file()
    }

    fn save_profile(&self, name: &str, profile: &ProfileConfig) -> Result<()> {
        let mut normalized = profile.clone();
        if let Some(reasoning_effort) = Self::profile_reasoning_effort(profile)? {
            normalized.platform_data.insert(
                "reasoning_effort".into(),
                JsonValue::String(reasoning_effort),
            );
        }
        self.validate_profile(&normalized)?;
        let mut profiles = self.load_profiles()?;
        profiles.insert(name.to_string(), normalized);
        self.save_profiles_to_file(&profiles)
    }

    fn delete_profile(&self, name: &str) -> Result<()> {
        let _operation_lock = self.lock_profile_operation()?;
        let profiles = self.load_profiles()?;
        let fallback_current = self.fallback_current_profile_from_file()?;
        let active_by_intent = self.current_profile_from_registry()?.as_deref() == Some(name)
            || fallback_current.as_deref() == Some(name);
        if self.load_entry_state()?.is_none()
            && (active_by_intent || self.runtime_has_managed_shape()?)
        {
            return Err(CcrError::ConfigError(
                "Grok 入口配置状态缺失，拒绝删除 profile 以避免遗留凭据；请先备份 config.toml 并手工恢复 [model.custom] 与 [models].default"
                    .into(),
            ));
        }
        let active_by_runtime = match profiles.get(name) {
            Some(profile) => self.runtime_matches_profile(name, profile)?,
            None => false,
        };
        if active_by_intent || active_by_runtime {
            return Err(CcrError::ValidationError(format!(
                "Grok profile '{name}' 当前处于激活状态，请先执行 off 或切换到其他 profile"
            )));
        }
        let preserve_fallback_pointer = match fallback_current.as_deref() {
            Some(current) => match profiles.get(current) {
                Some(profile) => self.runtime_matches_profile(current, profile)?,
                None => false,
            },
            None => false,
        };
        let mut profiles = profiles;
        if profiles.shift_remove(name).is_none() {
            return Err(CcrError::ProfileNotFound(name.to_string()));
        }
        self.save_profiles_to_file(&profiles)?;
        base::reconcile_registry_current_profile_after_delete_with_paths(
            &self.paths.registry_file,
            &self.registry_lock_dir(),
            "grok",
            name,
            &profiles,
        )?;
        if !preserve_fallback_pointer && self.current_profile_from_registry()?.is_none() {
            self.clear_profiles_current_config()?;
        }
        Ok(())
    }

    fn get_settings_path(&self) -> PathBuf {
        self.config_path.clone()
    }

    fn apply_profile(&self, name: &str) -> Result<()> {
        let _operation_lock = self.lock_profile_operation()?;
        let profiles = self.load_profiles()?;
        let profile = profiles
            .get(name)
            .ok_or_else(|| CcrError::ProfileNotFound(name.to_string()))?;
        self.validate_profile(profile)?;
        self.capture_entry_state()?;
        let state = self
            .load_entry_state()?
            .ok_or_else(|| CcrError::ConfigError("Grok 入口配置状态未能创建".into()))?;
        self.update_runtime_config(|config| {
            Self::apply_profile_to_config(config, name, profile, &state)
        })?;
        base::update_current_config(&self.paths.profiles_file, name)?;
        base::update_registry_current_profile_with_paths(
            &self.paths.registry_file,
            &self.registry_lock_dir(),
            "grok",
            name,
        )?;
        tracing::info!(profile = name, "已应用 Grok profile");
        Ok(())
    }

    fn validate_profile(&self, profile: &ProfileConfig) -> Result<()> {
        let official = Self::is_official_profile(profile);
        if let Some(base_url) = Self::trimmed(profile.base_url.as_ref())
            && !base_url.starts_with("http://")
            && !base_url.starts_with("https://")
        {
            return Err(CcrError::ValidationError(
                "Grok base_url 必须以 http:// 或 https:// 开头".into(),
            ));
        }
        if profile
            .auth_token
            .as_ref()
            .is_some_and(|secret| secret.expose().trim().is_empty())
        {
            return Err(CcrError::ValidationError(
                "Grok auth_token 不能为空字符串".into(),
            ));
        }

        let auth_mode = Self::profile_auth_mode(profile)?;
        let env_key = Self::profile_env_key(profile)?;
        if let Some(env_key) = env_key.as_deref()
            && !Self::is_valid_env_key(env_key)
        {
            return Err(CcrError::ValidationError(
                "Grok env_key 不是合法的环境变量名".into(),
            ));
        }
        Self::profile_api_backend(profile)?;
        Self::profile_context_window(profile)?;
        Self::profile_backend_search(profile)?;
        Self::profile_reasoning_effort(profile)?;

        if official {
            if Self::trimmed(profile.base_url.as_ref()).is_some() {
                return Err(CcrError::ValidationError(
                    "Grok 官方 profile 不允许设置 base_url".into(),
                ));
            }
            if auth_mode != GrokProfileAuthMode::Session {
                return Err(CcrError::ValidationError(
                    "Grok 官方 profile 不允许设置 api_key、auth_token 或 env_key".into(),
                ));
            }
        } else {
            if Self::trimmed(profile.base_url.as_ref()).is_none() {
                return Err(CcrError::ValidationError(
                    "Grok 第三方 profile 缺少 base_url".into(),
                ));
            }
            if Self::trimmed(profile.model.as_ref()).is_none() {
                return Err(CcrError::ValidationError(
                    "Grok 第三方 profile 缺少 model".into(),
                ));
            }
            if auth_mode == GrokProfileAuthMode::Session {
                return Err(CcrError::ValidationError(
                    "Grok 第三方 profile 必须设置 api_key、auth_token 或 env_key".into(),
                ));
            }
        }
        Ok(())
    }

    fn get_current_profile(&self) -> Result<Option<String>> {
        self.stable_current_profile()
    }

    fn get_env_var_names(&self) -> Vec<String> {
        vec!["XAI_API_KEY".into(), "GROK_CODE_XAI_API_KEY".into()]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestHome;
    use serde_json::json;

    fn platform() -> (TestHome, GrokPlatform) {
        let mut home = TestHome::new();
        let grok_home = home.home().join(".grok");
        fs::create_dir_all(&grok_home).unwrap();
        home.set_env("GROK_HOME", grok_home.as_os_str());
        let platform = GrokPlatform::new().unwrap();
        (home, platform)
    }

    fn third_party_profile() -> ProfileConfig {
        let mut profile = ProfileConfig::new()
            .with_description("Example relay".into())
            .with_base_url("https://api.example.com/v1".into())
            .with_model("grok-4.5".into());
        profile
            .platform_data
            .insert("env_key".into(), json!("EXAMPLE_GROK_API_KEY"));
        profile
            .platform_data
            .insert("api_backend".into(), json!("responses"));
        profile
            .platform_data
            .insert("context_window".into(), json!(1_000_000));
        profile
            .platform_data
            .insert("supports_backend_search".into(), json!(true));
        profile
            .platform_data
            .insert("reasoning_effort".into(), json!("high"));
        profile
    }

    fn inline_profile() -> ProfileConfig {
        let mut profile = third_party_profile();
        profile.platform_data.shift_remove("env_key");
        profile.auth_token = Some(ccr_core::Secret::new("INLINE_SECRET_SENTINEL"));
        profile
    }

    fn read_config(platform: &GrokPlatform) -> toml::Value {
        toml::from_str(&fs::read_to_string(&platform.config_path).unwrap()).unwrap()
    }

    #[test]
    fn safe_url_removes_userinfo_query_and_fragment() {
        assert_eq!(
            GrokPlatform::safe_base_url_for_display(
                "https://user:secret@api.example.com/v1?token=secret#part"
            ),
            "https://api.example.com/v1"
        );
        assert_eq!(
            GrokPlatform::safe_base_url_for_display(
                "user:secret@api.example.com/v1?token=secret#part"
            ),
            "api.example.com/v1"
        );
    }

    #[test]
    fn accepts_api_key_profile_and_writes_official_runtime_field() {
        let (_home, platform) = platform();
        let mut profile = third_party_profile();
        profile.platform_data.shift_remove("env_key");
        profile
            .platform_data
            .insert("api_key".into(), json!("INLINE_SECRET_SENTINEL"));

        platform.validate_profile(&profile).unwrap();
        platform.save_profile("relay", &profile).unwrap();
        platform.apply_profile("relay").unwrap();

        assert_eq!(
            read_config(&platform)["model"]["custom"]["api_key"].as_str(),
            Some("INLINE_SECRET_SENTINEL")
        );
    }

    #[test]
    fn validates_credential_and_platform_specific_fields() {
        let (_home, platform) = platform();
        let mut profile = third_party_profile();
        assert!(platform.validate_profile(&profile).is_ok());

        profile.auth_token = Some(ccr_core::Secret::new("sk-secret"));
        assert!(platform.validate_profile(&profile).is_err());
        profile.auth_token = None;
        profile
            .platform_data
            .insert("env_key".into(), json!(["ONE", "TWO"]));
        assert!(platform.validate_profile(&profile).is_err());
        profile
            .platform_data
            .insert("env_key".into(), json!("1INVALID"));
        assert!(platform.validate_profile(&profile).is_err());
        profile
            .platform_data
            .insert("env_key".into(), json!("VALID_KEY"));
        profile
            .platform_data
            .insert("context_window".into(), json!(0));
        assert!(platform.validate_profile(&profile).is_err());

        let mut profile = third_party_profile();
        profile
            .platform_data
            .insert("api_backend".into(), json!("invalid"));
        assert!(platform.validate_profile(&profile).is_err());

        let mut profile = third_party_profile();
        profile.model = None;
        assert!(platform.validate_profile(&profile).is_err());

        let mut profile = third_party_profile();
        profile.base_url = Some("ftp://api.example.com".into());
        assert!(platform.validate_profile(&profile).is_err());

        let mut official = ProfileConfig::new();
        official.auth_token = Some(ccr_core::Secret::new("sk-secret"));
        assert!(platform.validate_profile(&official).is_err());

        let mut official = ProfileConfig::new();
        official
            .platform_data
            .insert("env_key".into(), json!("XAI_API_KEY"));
        assert!(platform.validate_profile(&official).is_err());

        let mut missing_credential = third_party_profile();
        missing_credential.platform_data.shift_remove("env_key");
        assert!(platform.validate_profile(&missing_credential).is_err());

        let mut invalid_backend_search = third_party_profile();
        invalid_backend_search
            .platform_data
            .insert("supports_backend_search".into(), json!("yes"));
        assert!(platform.validate_profile(&invalid_backend_search).is_err());

        let mut invalid_reasoning_effort = third_party_profile();
        invalid_reasoning_effort
            .platform_data
            .insert("reasoning_effort".into(), json!(true));
        assert!(
            platform
                .validate_profile(&invalid_reasoning_effort)
                .is_err()
        );

        invalid_reasoning_effort
            .platform_data
            .insert("reasoning_effort".into(), json!("  "));
        assert!(
            platform
                .validate_profile(&invalid_reasoning_effort)
                .is_err()
        );

        invalid_reasoning_effort
            .platform_data
            .insert("reasoning_effort".into(), json!("model-option"));
        assert!(
            platform
                .validate_profile(&invalid_reasoning_effort)
                .is_err()
        );

        let mut official_with_base_url = ProfileConfig::new();
        official_with_base_url.provider_type = Some("official".into());
        official_with_base_url.base_url = Some("https://api.example.com/v1".into());
        assert!(platform.validate_profile(&official_with_base_url).is_err());
    }

    #[test]
    fn reasoning_effort_levels_round_trip_through_profile_storage() {
        let (_home, platform) = platform();
        let levels = ["none", "minimal", "low", "medium", "high", "xhigh", "max"];

        for (index, level) in levels.iter().enumerate() {
            let mut profile = third_party_profile();
            profile.platform_data.insert(
                "reasoning_effort".into(),
                json!(format!(" {} ", level.to_ascii_uppercase())),
            );
            platform
                .save_profile(&format!("relay-{index}"), &profile)
                .unwrap();
        }

        let profiles = platform.load_profiles().unwrap();
        for (index, level) in levels.iter().enumerate() {
            assert_eq!(
                profiles[&format!("relay-{index}")].platform_data["reasoning_effort"],
                *level
            );
        }
    }

    #[test]
    fn third_party_apply_preserves_unmanaged_tables_and_restores_entry_state() {
        let (_home, platform) = platform();
        fs::write(
            &platform.config_path,
            r#"[model.custom]
model = "original-model"
base_url = "https://original.example.com/v1"

[model.other]
model = "keep-me"

[models]
default = "original"
default_reasoning_effort = "low"

[session]
auto_compact_threshold_percent = 85

[ui]
fork_secondary_model = "custom"

[unknown]
nested = "keep-me-too"
"#,
        )
        .unwrap();
        platform
            .save_profile("relay", &third_party_profile())
            .unwrap();
        platform.apply_profile("relay").unwrap();

        let state = platform.load_entry_state().unwrap().unwrap();
        assert!(state.exists);
        assert!(
            state
                .content
                .as_deref()
                .is_some_and(|content| { content.contains("auto_compact_threshold_percent = 85") })
        );
        assert_eq!(
            state
                .original_custom_model
                .as_ref()
                .and_then(|model| model.get("model"))
                .and_then(toml::Value::as_str),
            Some("original-model")
        );
        assert_eq!(state.original_default_model.as_deref(), Some("original"));
        assert_eq!(
            state.original_default_reasoning_effort.as_ref(),
            Some(&toml::Value::String("low".into()))
        );
        assert!(
            !fs::read_dir(platform.config_path.parent().unwrap())
                .unwrap()
                .filter_map(std::result::Result::ok)
                .any(|entry| entry.file_name().to_string_lossy().ends_with(".bak"))
        );

        let applied = read_config(&platform);
        assert_eq!(applied["models"]["default"].as_str(), Some("custom"));
        assert_eq!(
            applied["model"]["custom"]["env_key"].as_str(),
            Some("EXAMPLE_GROK_API_KEY")
        );
        assert_eq!(applied["model"]["other"]["model"].as_str(), Some("keep-me"));
        assert_eq!(
            applied["session"]["auto_compact_threshold_percent"].as_integer(),
            Some(85)
        );
        assert_eq!(
            applied["ui"]["fork_secondary_model"].as_str(),
            Some("custom")
        );
        assert_eq!(applied["unknown"]["nested"].as_str(), Some("keep-me-too"));
        assert_eq!(
            applied["model"]["custom"]["supports_backend_search"].as_bool(),
            Some(true)
        );
        assert_eq!(
            applied["model"]["custom"]["supports_reasoning_effort"].as_bool(),
            Some(true)
        );
        assert_eq!(
            applied["model"]["custom"]["reasoning_effort"].as_str(),
            Some("high")
        );
        assert_eq!(
            applied["models"]["default_reasoning_effort"].as_str(),
            Some("high")
        );

        platform.clear_active_profile_runtime().unwrap();
        let restored = read_config(&platform);
        assert_eq!(
            restored["model"]["custom"]["model"].as_str(),
            Some("original-model")
        );
        assert_eq!(restored["models"]["default"].as_str(), Some("original"));
        assert_eq!(
            restored["models"]["default_reasoning_effort"].as_str(),
            Some("low")
        );
        assert!(!platform.entry_state_path().exists());
        assert_eq!(platform.get_current_profile().unwrap(), None);
    }

    #[test]
    fn official_profile_restores_custom_but_removes_unspecified_default() {
        let (_home, platform) = platform();
        fs::write(
            &platform.config_path,
            "[model.custom]\nmodel = \"original\"\n\n[models]\ndefault = \"original\"\n",
        )
        .unwrap();
        platform
            .save_profile("relay", &third_party_profile())
            .unwrap();
        platform.apply_profile("relay").unwrap();
        platform
            .save_profile("official", &ProfileConfig::new())
            .unwrap();
        platform.apply_profile("official").unwrap();

        let config = read_config(&platform);
        assert_eq!(
            config["model"]["custom"]["model"].as_str(),
            Some("original")
        );
        assert!(config.get("models").is_none());
        assert_eq!(
            platform.get_current_profile().unwrap().as_deref(),
            Some("official")
        );
    }

    #[test]
    fn official_reasoning_effort_only_updates_global_default() {
        let (_home, platform) = platform();
        fs::write(
            &platform.config_path,
            "[model.custom]\nmodel = \"entry\"\n\n[models]\ndefault = \"entry\"\ndefault_reasoning_effort = \"low\"\n",
        )
        .unwrap();
        let mut official = ProfileConfig::new().with_model("grok-example".into());
        official
            .platform_data
            .insert("reasoning_effort".into(), json!("HIGH"));
        platform.save_profile("official", &official).unwrap();
        platform.apply_profile("official").unwrap();

        let config = read_config(&platform);
        assert_eq!(config["model"]["custom"]["model"].as_str(), Some("entry"));
        assert!(config["model"]["custom"].get("reasoning_effort").is_none());
        assert!(
            config["model"]["custom"]
                .get("supports_reasoning_effort")
                .is_none()
        );
        assert_eq!(
            config["models"]["default_reasoning_effort"].as_str(),
            Some("high")
        );
        assert_eq!(
            platform.load_profiles().unwrap()["official"].platform_data["reasoning_effort"],
            "high"
        );
    }

    #[test]
    fn third_party_official_third_party_round_trip_keeps_entry_state() {
        let (_home, platform) = platform();
        fs::write(
            &platform.config_path,
            "[model.custom]\nmodel = \"entry\"\n\n[models]\ndefault = \"entry\"\ndefault_reasoning_effort = \"minimal\"\n",
        )
        .unwrap();
        platform
            .save_profile("relay", &third_party_profile())
            .unwrap();
        platform
            .save_profile("official", &ProfileConfig::new())
            .unwrap();

        platform.apply_profile("relay").unwrap();
        let state_before = fs::read(platform.entry_state_path()).unwrap();
        platform.apply_profile("official").unwrap();
        let official_config = read_config(&platform);
        assert_eq!(
            official_config["model"]["custom"]["model"].as_str(),
            Some("entry")
        );
        assert_eq!(
            official_config["models"]["default_reasoning_effort"].as_str(),
            Some("minimal")
        );
        platform.apply_profile("relay").unwrap();
        let relay_config = read_config(&platform);
        assert_eq!(
            relay_config["model"]["custom"]["model"].as_str(),
            Some("grok-4.5")
        );
        assert_eq!(
            relay_config["models"]["default_reasoning_effort"].as_str(),
            Some("high")
        );
        assert_eq!(fs::read(platform.entry_state_path()).unwrap(), state_before);

        platform.clear_active_profile_runtime().unwrap();
        let restored = read_config(&platform);
        assert_eq!(restored["model"]["custom"]["model"].as_str(), Some("entry"));
        assert_eq!(restored["models"]["default"].as_str(), Some("entry"));
        assert_eq!(
            restored["models"]["default_reasoning_effort"].as_str(),
            Some("minimal")
        );
    }

    #[test]
    fn legacy_entry_state_recovers_reasoning_effort_from_content() {
        let state: ProfileEntryConfigState = serde_json::from_str(
            r#"{
                "exists": true,
                "content": "[models]\ndefault = \"entry\"\ndefault_reasoning_effort = \"low\"\n",
                "original_custom_model": null,
                "original_default_model": "entry"
            }"#,
        )
        .unwrap();

        assert_eq!(state.original_default_reasoning_effort, None);
        let mut root = toml::Table::new();
        root.insert(
            "models".into(),
            toml::toml! {
                default = "custom"
                default_reasoning_effort = "high"
            }
            .into(),
        );

        GrokPlatform::restore_default_reasoning_effort(&mut root, &state).unwrap();
        assert_eq!(
            root["models"]["default_reasoning_effort"].as_str(),
            Some("low")
        );
    }

    #[test]
    fn entry_state_create_if_absent_cannot_replace_the_original_baseline() {
        let (_home, platform) = platform();
        fs::write(
            &platform.config_path,
            "[model.custom]\nmodel = \"entry\"\n\n[models]\ndefault = \"entry\"\n",
        )
        .unwrap();
        platform.capture_entry_state().unwrap();
        let original_state = fs::read(platform.entry_state_path()).unwrap();

        let replacement = ProfileEntryConfigState {
            exists: true,
            content: Some("api_key = \"RACE_SECRET_SENTINEL\"".into()),
            original_custom_model: None,
            original_default_model: Some("custom".into()),
            original_default_reasoning_effort: Some(toml::Value::String("max".into())),
        };
        let replacement = serde_json::to_vec(&replacement).unwrap();
        assert_eq!(
            platform.write_entry_state_if_absent(&replacement).unwrap(),
            VersionedWriteOutcome::Conflict
        );
        assert_eq!(
            fs::read(platform.entry_state_path()).unwrap(),
            original_state
        );
    }

    #[test]
    fn profile_operation_lock_serializes_multi_file_mutations() {
        let (_home, platform) = platform();
        let _first = platform.lock_profile_operation().unwrap();
        assert!(
            platform
                .lock_profile_operation_with_timeout(Duration::from_millis(20))
                .is_err()
        );
    }

    #[test]
    fn active_profile_delete_is_rejected_until_off() {
        let (_home, platform) = platform();
        platform
            .save_profile("relay", &third_party_profile())
            .unwrap();
        platform.apply_profile("relay").unwrap();
        assert!(platform.delete_profile("relay").is_err());
        platform.clear_active_profile_runtime().unwrap();
        let restored = read_config(&platform);
        assert!(restored.get("model").is_none());
        assert!(restored.get("models").is_none());
        platform.delete_profile("relay").unwrap();
        assert!(platform.load_profiles().unwrap().is_empty());
    }

    #[test]
    fn off_rejects_missing_entry_state_while_profile_is_active() {
        let (_home, platform) = platform();
        platform
            .save_profile("relay", &third_party_profile())
            .unwrap();
        platform.apply_profile("relay").unwrap();
        fs::remove_file(platform.entry_state_path()).unwrap();

        let result = platform.clear_active_profile_runtime();
        assert!(result.is_err());
        assert_eq!(
            read_config(&platform)["models"]["default"].as_str(),
            Some("custom")
        );
        assert_eq!(
            platform.current_profile_from_registry().unwrap().as_deref(),
            Some("relay")
        );
    }

    #[test]
    fn off_rejects_managed_runtime_when_state_and_pointers_are_missing() {
        let (_home, platform) = platform();
        platform.save_profile("relay", &inline_profile()).unwrap();
        platform.apply_profile("relay").unwrap();
        fs::remove_file(platform.entry_state_path()).unwrap();
        platform.clear_profiles_current_config().unwrap();
        platform.clear_current_profile_registry().unwrap();

        let result = platform.clear_active_profile_runtime();
        assert!(result.is_err());
        assert_eq!(
            read_config(&platform)["model"]["custom"]["api_key"].as_str(),
            Some("INLINE_SECRET_SENTINEL")
        );
    }

    #[test]
    fn delete_rejects_managed_runtime_when_state_and_pointers_are_missing() {
        let (_home, platform) = platform();
        platform.save_profile("relay", &inline_profile()).unwrap();
        platform.apply_profile("relay").unwrap();
        fs::remove_file(platform.entry_state_path()).unwrap();
        platform.clear_profiles_current_config().unwrap();
        platform.clear_current_profile_registry().unwrap();

        let result = platform.delete_profile("relay");
        assert!(matches!(result, Err(CcrError::ConfigError(_))));
        assert!(platform.load_profiles().unwrap().contains_key("relay"));
        assert_eq!(
            read_config(&platform)["model"]["custom"]["api_key"].as_str(),
            Some("INLINE_SECRET_SENTINEL")
        );
    }

    #[test]
    fn runtime_drift_clears_registry_pointer() {
        let (_home, platform) = platform();
        platform
            .save_profile("relay", &third_party_profile())
            .unwrap();
        platform.apply_profile("relay").unwrap();
        let mut config = read_config(&platform);
        config["models"]["default_reasoning_effort"] = toml::Value::String("medium".into());
        fs::write(
            &platform.config_path,
            toml::to_string_pretty(&config).unwrap(),
        )
        .unwrap();
        assert_eq!(platform.get_current_profile().unwrap(), None);
        assert_eq!(platform.current_profile_from_registry().unwrap(), None);
    }

    #[test]
    fn drifted_inline_profile_cannot_be_deleted_before_off() {
        let (_home, platform) = platform();
        platform.save_profile("relay", &inline_profile()).unwrap();
        platform.apply_profile("relay").unwrap();
        let mut config = read_config(&platform);
        config["models"]["default"] = toml::Value::String("other".into());
        fs::write(
            &platform.config_path,
            toml::to_string_pretty(&config).unwrap(),
        )
        .unwrap();

        assert_eq!(platform.get_current_profile().unwrap(), None);
        assert!(platform.delete_profile("relay").is_err());
        assert!(platform.load_profiles().unwrap().contains_key("relay"));
        platform.clear_active_profile_runtime().unwrap();
        platform.delete_profile("relay").unwrap();
        assert!(read_config(&platform).get("model").is_none());
    }

    #[test]
    fn secret_bearing_toml_parse_errors_do_not_echo_source_lines() {
        let (_home, platform) = platform();
        fs::write(
            &platform.config_path,
            "[model.custom]\napi_key = \"RUNTIME_SECRET_SENTINEL\n",
        )
        .unwrap();
        let runtime_error = GrokPlatform::load_config_value(&platform.config_path)
            .unwrap_err()
            .to_string();
        assert!(!runtime_error.contains("RUNTIME_SECRET_SENTINEL"));

        platform.paths.ensure_directories().unwrap();
        fs::write(
            &platform.paths.profiles_file,
            "[relay]\nauth_token = \"PROFILE_SECRET_SENTINEL\n",
        )
        .unwrap();
        let profiles_error = platform.load_profiles().unwrap_err().to_string();
        assert!(!profiles_error.contains("PROFILE_SECRET_SENTINEL"));
    }

    #[test]
    fn cas_retry_preserves_external_change() {
        let (_home, platform) = platform();
        fs::write(&platform.config_path, "[ui]\ntheme = \"dark\"\n").unwrap();
        platform.capture_entry_state().unwrap();
        let state = platform.load_entry_state().unwrap().unwrap();
        let profile = third_party_profile();
        let mut injected = false;
        platform
            .update_runtime_config_with_hook(
                |config| GrokPlatform::apply_profile_to_config(config, "relay", &profile, &state),
                |_, path| {
                    if !injected {
                        injected = true;
                        fs::write(
                            path,
                            "[ui]\ntheme = \"dark\"\n\n[session]\nexternal = true\n",
                        )?;
                    }
                    Ok(())
                },
            )
            .unwrap();
        let config = read_config(&platform);
        assert_eq!(config["session"]["external"].as_bool(), Some(true));
        assert_eq!(config["models"]["default"].as_str(), Some("custom"));
    }

    #[test]
    fn repeated_cas_conflict_does_not_overwrite_external_content() {
        let (_home, platform) = platform();
        fs::write(&platform.config_path, "[session]\nrevision = 0\n").unwrap();
        platform.capture_entry_state().unwrap();
        let state = platform.load_entry_state().unwrap().unwrap();
        let profile = third_party_profile();
        let result = platform.update_runtime_config_with_hook(
            |config| GrokPlatform::apply_profile_to_config(config, "relay", &profile, &state),
            |attempt, path| {
                fs::write(path, format!("[session]\nrevision = {}\n", attempt + 1))?;
                Ok(())
            },
        );
        assert!(result.is_err());
        let config = read_config(&platform);
        assert_eq!(config["session"]["revision"].as_integer(), Some(2));
        assert!(config.get("model").is_none());
    }

    #[test]
    fn registry_failure_leaves_runtime_truth_recoverable_and_retry_converges() {
        let (_home, platform) = platform();
        platform
            .save_profile("relay", &third_party_profile())
            .unwrap();
        platform
            .save_profile("official", &ProfileConfig::new())
            .unwrap();

        let registry_lock_dir = platform.registry_lock_dir();
        fs::write(&registry_lock_dir, "not a directory").unwrap();
        assert!(platform.apply_profile("relay").is_err());

        let config = read_config(&platform);
        assert_eq!(config["models"]["default"].as_str(), Some("custom"));
        assert_eq!(
            platform
                .fallback_current_profile_from_file()
                .unwrap()
                .as_deref(),
            Some("relay")
        );

        fs::remove_file(&registry_lock_dir).unwrap();
        platform.delete_profile("official").unwrap();
        assert_eq!(
            platform
                .fallback_current_profile_from_file()
                .unwrap()
                .as_deref(),
            Some("relay")
        );
        assert_eq!(
            platform.get_current_profile().unwrap().as_deref(),
            Some("relay")
        );
        platform.apply_profile("relay").unwrap();
        assert_eq!(
            platform.current_profile_from_registry().unwrap().as_deref(),
            Some("relay")
        );
    }
}
