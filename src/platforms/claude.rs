// 🤖 Claude Platform 实现
// 📦 Claude Code 平台配置管理
//
// 核心职责:
// - 🔄 支持双模式运行（Legacy/Unified）
// - 📋 管理 Claude profiles
// - ⚙️ 操作 ~/.claude/settings.json
// - 🔗 兼容现有 ConfigSection 结构

use crate::core::error::{CcrError, Result};
use crate::managers::PlatformConfigManager;
use crate::managers::config::{ConfigManager, ConfigSection};
use crate::managers::settings::{ClaudeSettings, SettingsManager};
use crate::models::{ConfigMode, Platform, PlatformConfig, PlatformPaths, ProfileConfig};
use crate::utils::Validatable;
use indexmap::IndexMap;
use std::fs;
use std::path::PathBuf;

/// 🤖 Claude Platform 实现
///
/// ## 运行模式
///
/// ### Legacy Mode (默认)
/// - 配置文件: `~/.ccs_config.toml`
/// - 设置文件: `~/.claude/settings.json`
/// - 与 CCS Shell 版本兼容
///
/// ### Unified Mode (可选)
/// - 配置文件: `~/.ccr/platforms/claude/profiles.toml`
/// - 设置文件: `~/.claude/settings.json`（路径不变）
/// - 支持多平台配置
///
/// ## 模式检测
///
/// 优先级：
/// 1. 环境变量 `CCR_ROOT` 存在 → Unified
/// 2. `~/.ccr/config.toml` 存在 → Unified
/// 3. 其他情况 → Legacy
pub struct ClaudePlatform {
    mode: ConfigMode,
    paths: PlatformPaths,
    config_manager: Option<ConfigManager>,
    settings_manager: SettingsManager,
}

impl ClaudePlatform {
    /// 🏗️ 创建新的 Claude Platform 实例
    ///
    /// 自动检测运行模式并初始化相应的管理器
    pub fn new() -> Result<Self> {
        let mode = Self::detect_mode()?;
        let paths = PlatformPaths::new(Platform::Claude)?;
        let settings_manager = SettingsManager::default()?;

        let config_manager = if matches!(mode, ConfigMode::Legacy) {
            Some(ConfigManager::default()?)
        } else {
            None
        };

        Ok(Self {
            mode,
            paths,
            config_manager,
            settings_manager,
        })
    }

    /// 🔍 检测配置模式
    ///
    /// 检测规则：
    /// 1. 环境变量 `CCR_ROOT` 存在 → Unified
    /// 2. `~/.ccr/config.toml` 存在 → Unified
    /// 3. 默认 → Legacy
    fn detect_mode() -> Result<ConfigMode> {
        // 检查环境变量
        if std::env::var("CCR_ROOT").is_ok() {
            log::debug!("检测到 CCR_ROOT 环境变量，使用 Unified 模式");
            return Ok(ConfigMode::Unified);
        }

        // 检查 ~/.ccr/config.toml 是否存在
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
        let unified_config = home.join(".ccr").join("config.toml");

        if unified_config.exists() {
            log::debug!("检测到 ~/.ccr/config.toml，使用 Unified 模式");
            Ok(ConfigMode::Unified)
        } else {
            log::debug!("使用 Legacy 模式（默认）");
            Ok(ConfigMode::Legacy)
        }
    }

    /// 📋 从 ConfigSection 转换为 ProfileConfig
    fn section_to_profile(section: &ConfigSection) -> ProfileConfig {
        ProfileConfig {
            description: section.description.clone(),
            base_url: section.base_url.clone(),
            auth_token: section.auth_token.clone(),
            model: section.model.clone(),
            small_fast_model: section.small_fast_model.clone(),
            provider: section.provider.clone(),
            provider_type: section
                .provider_type
                .as_ref()
                .map(|t| t.to_string_value().to_string()),
            account: section.account.clone(),
            tags: section.tags.clone(),
            platform_data: IndexMap::new(),
        }
    }

    /// 📋 从 ProfileConfig 转换为 ConfigSection
    fn profile_to_section(profile: &ProfileConfig) -> Result<ConfigSection> {
        use crate::managers::config::ProviderType;

        let provider_type = profile
            .provider_type
            .as_ref()
            .and_then(|s| match s.as_str() {
                "official_relay" => Some(ProviderType::OfficialRelay),
                "third_party_model" => Some(ProviderType::ThirdPartyModel),
                _ => None,
            });

        Ok(ConfigSection {
            description: profile.description.clone(),
            base_url: profile.base_url.clone(),
            auth_token: profile.auth_token.clone(),
            model: profile.model.clone(),
            small_fast_model: profile.small_fast_model.clone(),
            provider: profile.provider.clone(),
            provider_type,
            account: profile.account.clone(),
            tags: profile.tags.clone(),
        })
    }

    /// 💾 保存 profiles 到 TOML 文件（Unified 模式）
    fn save_profiles_unified(&self, profiles: &IndexMap<String, ProfileConfig>) -> Result<()> {
        // 确保目录存在
        self.paths.ensure_directories()?;

        // 转换为 ConfigSection 格式
        let mut sections = IndexMap::new();
        for (name, profile) in profiles {
            sections.insert(name.clone(), Self::profile_to_section(profile)?);
        }

        // 构建完整配置（为了兼容性，包含默认字段）
        use crate::managers::config::{CcsConfig, GlobalSettings};
        let config = CcsConfig {
            default_config: profiles
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "default".to_string()),
            current_config: profiles
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "default".to_string()),
            settings: GlobalSettings::default(),
            sections,
        };

        // 序列化为 TOML
        let content = toml::to_string_pretty(&config)
            .map_err(|e| CcrError::ConfigError(format!("序列化配置失败: {}", e)))?;

        // 写入文件
        fs::write(&self.paths.profiles_file, content)
            .map_err(|e| CcrError::ConfigError(format!("写入配置文件失败: {}", e)))?;

        log::info!("✅ 已保存 Claude profiles: {:?}", self.paths.profiles_file);
        Ok(())
    }

    /// 📖 从 TOML 文件加载 profiles（Unified 模式）
    fn load_profiles_unified(&self) -> Result<IndexMap<String, ProfileConfig>> {
        if !self.paths.profiles_file.exists() {
            return Ok(IndexMap::new());
        }

        // 读取文件
        let content = fs::read_to_string(&self.paths.profiles_file)
            .map_err(|e| CcrError::ConfigError(format!("读取配置文件失败: {}", e)))?;

        // 解析 TOML
        use crate::managers::config::CcsConfig;
        let config: CcsConfig = toml::from_str(&content)
            .map_err(|e| CcrError::ConfigFormatInvalid(format!("TOML 解析失败: {}", e)))?;

        // 转换为 ProfileConfig
        let mut profiles = IndexMap::new();
        for (name, section) in config.sections {
            profiles.insert(name, Self::section_to_profile(&section));
        }

        Ok(profiles)
    }

    /// 📖 从 Legacy 配置加载 profiles
    fn load_profiles_legacy(&self) -> Result<IndexMap<String, ProfileConfig>> {
        let manager = self
            .config_manager
            .as_ref()
            .ok_or_else(|| CcrError::ConfigError("Legacy 配置管理器未初始化".into()))?;

        let config = manager.load()?;
        let mut profiles = IndexMap::new();

        for (name, section) in config.sections {
            profiles.insert(name, Self::section_to_profile(&section));
        }

        Ok(profiles)
    }

    /// 💾 保存 profile 到 Legacy 配置
    fn save_profile_legacy(&self, name: &str, profile: &ProfileConfig) -> Result<()> {
        let manager = self
            .config_manager
            .as_ref()
            .ok_or_else(|| CcrError::ConfigError("Legacy 配置管理器未初始化".into()))?;

        let mut config = manager.load()?;
        let section = Self::profile_to_section(profile)?;

        config.sections.insert(name.to_string(), section);
        manager.save(&config)?;

        Ok(())
    }

    /// 🗑️ 删除 Legacy 配置中的 profile
    #[allow(dead_code)]
    fn delete_profile_legacy(&self, name: &str) -> Result<()> {
        let manager = self
            .config_manager
            .as_ref()
            .ok_or_else(|| CcrError::ConfigError("Legacy 配置管理器未初始化".into()))?;

        let mut config = manager.load()?;
        config.remove_section(name)?;
        manager.save(&config)?;

        Ok(())
    }

    /// 📖 获取当前 profile（Legacy 模式）
    fn get_current_profile_legacy(&self) -> Result<Option<String>> {
        let manager = self
            .config_manager
            .as_ref()
            .ok_or_else(|| CcrError::ConfigError("Legacy 配置管理器未初始化".into()))?;

        let config = manager.load()?;
        Ok(Some(config.current_config))
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
        match self.mode {
            ConfigMode::Legacy => self.load_profiles_legacy(),
            ConfigMode::Unified => self.load_profiles_unified(),
        }
    }

    fn save_profile(&self, name: &str, profile: &ProfileConfig) -> Result<()> {
        // 先验证
        self.validate_profile(profile)?;

        match self.mode {
            ConfigMode::Legacy => self.save_profile_legacy(name, profile),
            ConfigMode::Unified => {
                let mut profiles = self.load_profiles()?;
                profiles.insert(name.to_string(), profile.clone());
                self.save_profiles_unified(&profiles)
            }
        }
    }

    fn delete_profile(&self, name: &str) -> Result<()> {
        match self.mode {
            ConfigMode::Legacy => self.delete_profile_legacy(name),
            ConfigMode::Unified => {
                let mut profiles = self.load_profiles()?;
                if profiles.shift_remove(name).is_none() {
                    return Err(CcrError::ProfileNotFound(name.to_string()));
                }
                self.save_profiles_unified(&profiles)
            }
        }
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

        // 在 Unified 模式下，同步更新注册表中的 current_profile
        if matches!(self.mode, crate::models::ConfigMode::Unified) {
            let platform_config_mgr = PlatformConfigManager::default()?;
            let mut unified_config = platform_config_mgr.load()?;

            // 更新 Claude 平台的 current_profile
            unified_config.set_platform_profile("claude", name)?;

            // 保存注册表
            platform_config_mgr.save(&unified_config)?;

            log::debug!("✅ 已更新注册表 current_profile: {}", name);
        }

        log::info!("✅ 已应用 Claude profile: {}", name);
        Ok(())
    }

    fn validate_profile(&self, profile: &ProfileConfig) -> Result<()> {
        // 转换为 ConfigSection 并验证
        let section = Self::profile_to_section(profile)?;
        section.validate()
    }

    fn get_current_profile(&self) -> Result<Option<String>> {
        match self.mode {
            ConfigMode::Legacy => self.get_current_profile_legacy(),
            ConfigMode::Unified => {
                // 在 Unified 模式下，从注册表读取 current_profile
                let platform_config_mgr = PlatformConfigManager::default()?;
                let unified_config = platform_config_mgr.load()?;

                // 获取 Claude 平台的注册信息
                let claude_entry = unified_config.get_platform("claude")?;
                Ok(claude_entry.current_profile.clone())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_mode() {
        // 注意：这个测试依赖环境，可能在 CI 中需要调整
        let mode = ClaudePlatform::detect_mode();
        assert!(mode.is_ok());
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
}
