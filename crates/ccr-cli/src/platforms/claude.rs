// 🤖 Claude Platform 实现
// 📦 Claude Code 平台配置管理
//
// 核心职责:
// - 📋 管理 Claude profiles
// - ⚙️ 操作 ~/.claude/settings.json
// - 🔗 兼容现有 ConfigSection 结构
//
// 📁 配置结构 (Unified 模式):
// - 配置文件: `~/.ccr/platforms/claude/profiles.toml`
// - 设置文件: `~/.claude/settings.json`
// - 支持多平台配置

use crate::managers::PlatformConfigManager;
use crate::managers::config::{CcsConfig, ConfigSection};
use crate::managers::settings::{ClaudeSettings, SettingsManager};
use crate::models::{
    ClaudeProfileAuthMode, Platform, PlatformConfig, PlatformPaths, ProfileConfig,
};
use ccr_config::platforms::base;
use ccr_core::Validatable;
use ccr_core::core::error::{CcrError, Result};
use indexmap::IndexMap;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

/// 🤖 Claude Platform 实现
///
/// ## 配置结构 (Unified 模式)
///
/// - 配置文件: `~/.ccr/platforms/claude/profiles.toml`
/// - 设置文件: `~/.claude/settings.json`
/// - 支持多平台配置
pub struct ClaudePlatform {
    paths: PlatformPaths,
    settings_manager: SettingsManager,
}

impl ClaudePlatform {
    pub const AUTH_MODE_FIELD: &str = "auth_mode";
    pub const EDITABLE_FIELDS: &[&str] = &[
        "description",
        "base_url",
        "auth_token",
        "model",
        "small_fast_model",
        "provider",
        "provider_type",
        "account",
        "tags",
        "auth_mode",
    ];

    /// 🏗️ 创建新的 Claude Platform 实例
    pub fn new() -> Result<Self> {
        let paths = PlatformPaths::new(Platform::Claude)?;
        let settings_manager = SettingsManager::with_default()?;

        Ok(Self {
            paths,
            settings_manager,
        })
    }

    /// 📋 从 ConfigSection 转换为 ProfileConfig
    #[allow(dead_code)]
    fn section_to_profile(section: &ConfigSection) -> ProfileConfig {
        base::section_to_profile(section)
    }

    /// 📋 从 ProfileConfig 转换为 ConfigSection
    fn profile_to_section(profile: &ProfileConfig) -> Result<ConfigSection> {
        base::profile_to_section(profile)
    }

    /// 💾 保存 profiles 到 TOML 文件
    fn save_profiles(&self, profiles: &IndexMap<String, ProfileConfig>) -> Result<()> {
        base::save_profiles_to_toml(&self.paths.profiles_file, profiles, "claude", &self.paths)
    }

    /// 🔄 更新 profiles.toml 中的 current_config 字段
    ///
    /// 在配置切换时调用，用于同步更新 profiles.toml 中记录的当前配置名称
    fn update_current_config_in_profiles(&self, name: &str) -> Result<()> {
        base::update_current_config(&self.paths.profiles_file, name)
    }

    /// 📖 从 TOML 文件加载 profiles
    fn load_profiles_from_file(&self) -> Result<IndexMap<String, ProfileConfig>> {
        base::load_profiles_from_toml(&self.paths.profiles_file)
    }

    fn current_profile_from_file(
        &self,
        profiles: &IndexMap<String, ProfileConfig>,
    ) -> Result<Option<String>> {
        if !self.paths.profiles_file.exists() {
            return Ok(None);
        }

        let content = match fs::read_to_string(&self.paths.profiles_file) {
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

    fn clear_current_profile_registry(&self) -> Result<()> {
        let manager = PlatformConfigManager::with_default()?;
        let mut unified = manager.load()?;
        if let Ok(entry) = unified.get_platform_mut("claude") {
            entry.current_profile = None;
            entry.last_used = Some(chrono::Utc::now().to_rfc3339());
        }
        manager.save(&unified)
    }

    fn stable_current_profile(&self) -> Result<Option<String>> {
        let profiles = self.load_profiles()?;
        let registry_current = base::get_current_profile_from_registry("claude")?;

        if let Some(file_current) = self.current_profile_from_file(&profiles)? {
            if registry_current.as_deref() != Some(file_current.as_str()) {
                base::update_registry_current_profile("claude", &file_current)?;
            }
            return Ok(Some(file_current));
        }

        match registry_current {
            Some(current) if profiles.contains_key(&current) => {
                self.update_current_config_in_profiles(&current)?;
                Ok(Some(current))
            }
            Some(_) => {
                self.clear_current_profile_registry()?;
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn editable_fields() -> &'static [&'static str] {
        Self::EDITABLE_FIELDS
    }

    pub fn profile_auth_mode(profile: &ProfileConfig) -> ClaudeProfileAuthMode {
        crate::services::ClaudeAuthService::resolve_profile_auth_mode(profile)
    }

    pub fn profile_auth_source(profile: &ProfileConfig) -> String {
        match Self::profile_auth_mode(profile) {
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

    fn normalize_profile(profile: &mut ProfileConfig) {
        let auth_mode = Self::profile_auth_mode(profile);
        profile
            .platform_data
            .insert(Self::AUTH_MODE_FIELD.to_string(), json!(auth_mode.as_str()));
    }

    fn validate_optional_subscription_fields(section: &ConfigSection) -> Result<()> {
        if let Some(base_url) = &section.base_url {
            let base_url = base_url.trim();
            if base_url.is_empty() {
                return Err(CcrError::ValidationError("base_url 不能为空字符串".into()));
            }
            if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
                return Err(CcrError::ValidationError(
                    "base_url 必须以 http:// 或 https:// 开头".into(),
                ));
            }
        }

        if let Some(auth_token) = &section.auth_token
            && auth_token.trim().is_empty()
        {
            return Err(CcrError::ValidationError(
                "auth_token 不能为空字符串".into(),
            ));
        }

        if let Some(model) = &section.model
            && model.trim().is_empty()
        {
            return Err(CcrError::ValidationError("model 不能为空字符串".into()));
        }

        if let Some(model) = &section.small_fast_model
            && model.trim().is_empty()
        {
            return Err(CcrError::ValidationError(
                "small_fast_model 不能为空字符串".into(),
            ));
        }

        Ok(())
    }
}

impl PlatformConfig for ClaudePlatform {
    fn platform_name(&self) -> &str {
        "claude"
    }

    fn platform_type(&self) -> Platform {
        Platform::Claude
    }

    fn load_profiles(&self) -> Result<IndexMap<String, ProfileConfig>> {
        self.load_profiles_from_file()
    }

    fn save_profile(&self, name: &str, profile: &ProfileConfig) -> Result<()> {
        // 先验证
        self.validate_profile(profile)?;

        let mut profiles = self.load_profiles()?;
        let mut normalized = profile.clone();
        Self::normalize_profile(&mut normalized);
        profiles.insert(name.to_string(), normalized);
        self.save_profiles(&profiles)
    }

    fn delete_profile(&self, name: &str) -> Result<()> {
        let mut profiles = self.load_profiles()?;
        if profiles.shift_remove(name).is_none() {
            return Err(CcrError::ProfileNotFound(name.to_string()));
        }
        self.save_profiles(&profiles)?;
        base::reconcile_registry_current_profile_after_delete("claude", name, &profiles)
    }

    fn get_settings_path(&self) -> PathBuf {
        // Claude Code 硬编码路径：~/.claude/settings.json
        self.settings_manager.settings_path().to_path_buf()
    }

    fn apply_profile(&self, name: &str) -> Result<()> {
        // 加载 profile
        let profiles = self.load_profiles()?;
        let profile = profiles
            .get(name)
            .ok_or_else(|| CcrError::ProfileNotFound(name.to_string()))?;

        // 转换为 ConfigSection
        let section = Self::profile_to_section(profile)?;

        let auth_mode = Self::profile_auth_mode(profile);
        self.validate_profile(profile)?;

        // 加载当前设置
        let mut settings = self
            .settings_manager
            .load()
            .unwrap_or_else(|_| ClaudeSettings::new());

        match auth_mode {
            ClaudeProfileAuthMode::Subscription => {
                settings.clear_managed_vars();
            }
            ClaudeProfileAuthMode::ApiKey => {
                settings.update_from_config(&section);
            }
        }

        // 原子保存
        self.settings_manager.save_atomic(&settings)?;

        // 🔧 更新 profiles.toml 中的 current_config
        self.update_current_config_in_profiles(name)?;
        base::update_registry_current_profile("claude", name)?;

        tracing::info!("✅ 已应用 Claude profile: {}", name);
        Ok(())
    }

    fn validate_profile(&self, profile: &ProfileConfig) -> Result<()> {
        let section = Self::profile_to_section(profile)?;
        match Self::profile_auth_mode(profile) {
            ClaudeProfileAuthMode::Subscription => {
                Self::validate_optional_subscription_fields(&section)
            }
            ClaudeProfileAuthMode::ApiKey => section.validate(),
        }
    }

    fn get_current_profile(&self) -> Result<Option<String>> {
        self.stable_current_profile()
    }

    fn get_env_var_names(&self) -> Vec<String> {
        vec![
            "ANTHROPIC_BASE_URL".into(),
            "ANTHROPIC_AUTH_TOKEN".into(),
            "ANTHROPIC_MODEL".into(),
            "ANTHROPIC_SMALL_FAST_MODEL".into(),
            "ANTHROPIC_DEFAULT_OPUS_MODEL".into(),
            "ANTHROPIC_DEFAULT_SONNET_MODEL".into(),
            "ANTHROPIC_DEFAULT_HAIKU_MODEL".into(),
            "CLAUDE_CODE_SUBAGENT_MODEL".into(),
            "CLAUDE_CODE_EFFORT_LEVEL".into(),
        ]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::managers::PlatformConfigManager;
    use crate::managers::{PlatformConfigEntry, UnifiedConfig};
    use crate::test_support::TestHome;
    use std::fs;

    struct TestEnv {
        home: TestHome,
    }

    impl TestEnv {
        fn new() -> Self {
            Self {
                home: TestHome::new(),
            }
        }

        fn root_path(&self) -> &std::path::Path {
            self.home.root()
        }
    }

    fn make_profile(name: &str) -> ProfileConfig {
        ProfileConfig::new()
            .with_base_url(format!("https://{name}.example.com"))
            .with_auth_token(format!("sk-{name}"))
            .with_model(format!("claude-{name}"))
    }

    fn make_subscription_profile(name: &str) -> ProfileConfig {
        let mut profile = ProfileConfig::new().with_model(format!("claude-{name}"));
        profile.platform_data.insert(
            ClaudePlatform::AUTH_MODE_FIELD.to_string(),
            json!("subscription"),
        );
        profile
    }

    fn read_profiles_config(root: &std::path::Path) -> CcsConfig {
        let profiles_path = root.join("platforms").join("claude").join("profiles.toml");
        toml::from_str(&fs::read_to_string(profiles_path).unwrap()).unwrap()
    }

    fn write_profiles_config(root: &std::path::Path, config: &CcsConfig) {
        let profiles_path = root.join("platforms").join("claude").join("profiles.toml");
        fs::create_dir_all(profiles_path.parent().unwrap()).unwrap();
        fs::write(profiles_path, toml::to_string_pretty(config).unwrap()).unwrap();
    }

    #[test]
    fn test_section_to_profile_conversion() {
        use crate::managers::config::ConfigSection;

        let section = ConfigSection {
            description: Some("Test".to_string()),
            base_url: Some("https://api.test.com".to_string()),
            auth_token: Some("sk-test".to_string()),
            model: Some("test-model".to_string()),
            small_fast_model: Some("test-small".to_string()),
            provider: Some("test-provider".to_string()),
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            other: indexmap::IndexMap::new(),
            ..Default::default()
        };

        let profile = ClaudePlatform::section_to_profile(&section);
        assert_eq!(profile.description, Some("Test".to_string()));
        assert_eq!(profile.base_url, Some("https://api.test.com".to_string()));
        assert_eq!(profile.auth_token, Some("sk-test".to_string()));

        // 反向转换
        let section2 = ClaudePlatform::profile_to_section(&profile).unwrap();
        assert_eq!(section.description, section2.description);
        assert_eq!(section.base_url, section2.base_url);
    }

    #[test]
    fn test_platform_trait_impl() {
        if let Ok(platform) = ClaudePlatform::new() {
            assert_eq!(platform.platform_name(), "claude");
            assert_eq!(platform.platform_type(), Platform::Claude);
            assert!(
                platform
                    .get_settings_path()
                    .to_str()
                    .unwrap()
                    .contains("claude")
            );
        }
    }

    #[test]
    fn test_apply_profile_auto_registers_missing_claude_platform() {
        let _env = TestEnv::new();

        let result = (|| -> Result<()> {
            let manager = PlatformConfigManager::with_default()?;
            let mut unified_config = UnifiedConfig::default();
            unified_config.register_platform("codex".into(), PlatformConfigEntry::default())?;
            unified_config.platforms.shift_remove("claude");
            unified_config.current_platform = "codex".to_string();
            manager.save(&unified_config)?;

            let platform = ClaudePlatform::new()?;
            let profile = ProfileConfig::new()
                .with_base_url("https://api.example.com".to_string())
                .with_auth_token("sk-test".to_string())
                .with_model("claude-test".to_string());

            platform.save_profile("repair-me", &profile)?;
            platform.apply_profile("repair-me")?;

            let reloaded = manager.load()?;
            assert_eq!(
                reloaded.get_platform("claude")?.current_profile.as_deref(),
                Some("repair-me")
            );

            let settings = SettingsManager::with_default()?.load()?;
            assert_eq!(
                settings.env.get("ANTHROPIC_BASE_URL").map(String::as_str),
                Some("https://api.example.com")
            );
            assert_eq!(
                settings.env.get("ANTHROPIC_AUTH_TOKEN").map(String::as_str),
                Some("sk-test")
            );

            Ok(())
        })();

        result.unwrap();
    }

    #[test]
    fn test_get_current_profile_prefers_profiles_file_and_repairs_registry() {
        let env = TestEnv::new();

        let result = (|| -> Result<()> {
            let platform = ClaudePlatform::new()?;
            platform.save_profile("alpha", &make_profile("alpha"))?;
            platform.save_profile("beta", &make_profile("beta"))?;
            platform.update_current_config_in_profiles("beta")?;
            base::update_registry_current_profile("claude", "alpha")?;

            assert_eq!(platform.get_current_profile()?, Some("beta".to_string()));

            let manager = PlatformConfigManager::with_default()?;
            let reloaded = manager.load()?;
            assert_eq!(
                reloaded.get_platform("claude")?.current_profile.as_deref(),
                Some("beta")
            );

            Ok(())
        })();

        drop(env);
        result.unwrap();
    }

    #[test]
    fn test_get_current_profile_repairs_profiles_file_from_valid_registry() {
        let env = TestEnv::new();

        let result = (|| -> Result<()> {
            let platform = ClaudePlatform::new()?;
            platform.save_profile("alpha", &make_profile("alpha"))?;
            platform.save_profile("beta", &make_profile("beta"))?;

            let mut config = read_profiles_config(env.root_path());
            config.current_config = "ghost".to_string();
            write_profiles_config(env.root_path(), &config);
            base::update_registry_current_profile("claude", "beta")?;

            assert_eq!(platform.get_current_profile()?, Some("beta".to_string()));

            let repaired = read_profiles_config(env.root_path());
            assert_eq!(repaired.current_config, "beta");

            Ok(())
        })();

        drop(env);
        result.unwrap();
    }

    #[test]
    fn test_get_current_profile_returns_none_when_registry_and_file_are_invalid() {
        let env = TestEnv::new();

        let result = (|| -> Result<()> {
            let platform = ClaudePlatform::new()?;
            platform.save_profile("alpha", &make_profile("alpha"))?;
            platform.save_profile("beta", &make_profile("beta"))?;

            let mut config = read_profiles_config(env.root_path());
            config.current_config = "ghost".to_string();
            write_profiles_config(env.root_path(), &config);
            base::update_registry_current_profile("claude", "phantom")?;

            assert_eq!(platform.get_current_profile()?, None);

            let manager = PlatformConfigManager::with_default()?;
            let reloaded = manager.load()?;
            assert_eq!(reloaded.get_platform("claude")?.current_profile, None);

            Ok(())
        })();

        drop(env);
        result.unwrap();
    }

    #[test]
    fn test_delete_current_profile_keeps_registry_and_file_on_same_fallback() {
        let env = TestEnv::new();

        let result = (|| -> Result<()> {
            let platform = ClaudePlatform::new()?;
            platform.save_profile("alpha", &make_profile("alpha"))?;
            platform.save_profile("beta", &make_profile("beta"))?;
            platform.apply_profile("alpha")?;

            platform.delete_profile("alpha")?;

            assert_eq!(platform.get_current_profile()?, Some("beta".to_string()));

            let config = read_profiles_config(env.root_path());
            assert_eq!(config.current_config, "beta");

            let manager = PlatformConfigManager::with_default()?;
            let reloaded = manager.load()?;
            assert_eq!(
                reloaded.get_platform("claude")?.current_profile.as_deref(),
                Some("beta")
            );

            Ok(())
        })();

        drop(env);
        result.unwrap();
    }

    #[test]
    fn test_subscription_profile_can_save_and_apply_without_api_key() {
        let _env = TestEnv::new();

        let result = (|| -> Result<()> {
            let platform = ClaudePlatform::new()?;
            let profile = make_subscription_profile("official");
            platform.save_profile("official", &profile)?;
            platform.apply_profile("official")?;

            let settings = SettingsManager::with_default()?.load()?;
            assert!(!settings.has_anthropic_overrides());

            Ok(())
        })();

        result.unwrap();
    }

    #[test]
    fn test_subscription_profile_apply_clears_only_anthropic_overrides() {
        let _env = TestEnv::new();

        let result = (|| -> Result<()> {
            let platform = ClaudePlatform::new()?;
            let profile = make_subscription_profile("official");
            platform.save_profile("official", &profile)?;

            let mut settings = ClaudeSettings::new();
            settings.env.insert(
                "ANTHROPIC_BASE_URL".into(),
                "https://old.example.com".into(),
            );
            settings
                .env
                .insert("ANTHROPIC_AUTH_TOKEN".into(), "sk-old".into());
            settings.env.insert("KEEP_ME".into(), "value".into());
            SettingsManager::with_default()?.save_atomic(&settings)?;

            platform.apply_profile("official")?;

            let settings = SettingsManager::with_default()?.load()?;
            assert!(!settings.env.contains_key("ANTHROPIC_BASE_URL"));
            assert!(!settings.env.contains_key("ANTHROPIC_AUTH_TOKEN"));
            assert_eq!(
                settings.env.get("KEEP_ME").map(String::as_str),
                Some("value")
            );

            Ok(())
        })();

        result.unwrap();
    }
}
