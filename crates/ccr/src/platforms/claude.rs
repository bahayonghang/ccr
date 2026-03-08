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

use crate::core::error::{CcrError, Result};
use crate::managers::config::ConfigSection;
use crate::managers::settings::{ClaudeSettings, SettingsManager};
use crate::models::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
use crate::platforms::base;
use crate::utils::Validatable;
use indexmap::IndexMap;
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
        profiles.insert(name.to_string(), profile.clone());
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

        // 验证
        section.validate()?;

        // 加载当前设置
        let mut settings = self
            .settings_manager
            .load()
            .unwrap_or_else(|_| ClaudeSettings::new());

        // 更新设置
        settings.update_from_config(&section);

        // 原子保存
        self.settings_manager.save_atomic(&settings)?;

        // 🔧 更新 profiles.toml 中的 current_config
        self.update_current_config_in_profiles(name)?;
        base::update_registry_current_profile("claude", name)?;

        tracing::info!("✅ 已应用 Claude profile: {}", name);
        Ok(())
    }

    fn validate_profile(&self, profile: &ProfileConfig) -> Result<()> {
        // 转换为 ConfigSection 并验证
        let section = Self::profile_to_section(profile)?;
        section.validate()
    }

    fn get_current_profile(&self) -> Result<Option<String>> {
        base::get_current_profile_from_registry("claude")
    }

    fn get_env_var_names(&self) -> Vec<String> {
        vec![
            "ANTHROPIC_BASE_URL".into(),
            "ANTHROPIC_AUTH_TOKEN".into(),
            "ANTHROPIC_MODEL".into(),
            "ANTHROPIC_SMALL_FAST_MODEL".into(),
        ]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::managers::PlatformConfigManager;
    use crate::managers::platform_config::{PlatformConfigEntry, UnifiedConfig};
    use std::sync::{LazyLock, Mutex};

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn restore_env_var(key: &str, previous: Option<String>) {
        unsafe {
            match previous {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
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
        let _guard = ENV_LOCK.lock().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let settings_path = temp_dir.path().join("claude").join("settings.json");
        let backup_dir = temp_dir.path().join("claude").join("backups");
        let lock_dir = temp_dir.path().join("locks");

        let previous_root = std::env::var("CCR_ROOT").ok();
        let previous_settings = std::env::var("CCR_SETTINGS_PATH").ok();
        let previous_backup = std::env::var("CCR_BACKUP_DIR").ok();
        let previous_lock = std::env::var("CCR_LOCK_DIR").ok();

        unsafe {
            std::env::set_var("CCR_ROOT", temp_dir.path());
            std::env::set_var("CCR_SETTINGS_PATH", &settings_path);
            std::env::set_var("CCR_BACKUP_DIR", &backup_dir);
            std::env::set_var("CCR_LOCK_DIR", &lock_dir);
        }

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

        restore_env_var("CCR_ROOT", previous_root);
        restore_env_var("CCR_SETTINGS_PATH", previous_settings);
        restore_env_var("CCR_BACKUP_DIR", previous_backup);
        restore_env_var("CCR_LOCK_DIR", previous_lock);
        result.unwrap();
    }
}
