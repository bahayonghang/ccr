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
use crate::managers::PlatformConfigManager;
use crate::managers::config::ConfigSection;
use crate::managers::settings::{ClaudeSettings, SettingsManager};
use crate::models::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
use crate::utils::{Validatable, toml_json};
use indexmap::IndexMap;
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
            usage_count: section.usage_count,
            enabled: section.enabled,
            platform_data: toml_json::toml_map_to_json_map(&section.other),
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
            usage_count: profile.usage_count,
            enabled: profile.enabled,
            other: toml_json::json_map_to_toml_map(&profile.platform_data),
        })
    }

    /// 💾 保存 profiles 到 TOML 文件
    fn save_profiles(&self, profiles: &IndexMap<String, ProfileConfig>) -> Result<()> {
        // 确保目录存在
        self.paths.ensure_directories()?;

        // 转换为 ConfigSection 格式
        let mut sections = IndexMap::new();
        for (name, profile) in profiles {
            sections.insert(name.clone(), Self::profile_to_section(profile)?);
        }

        // 📖 先读取现有配置，保留 current_config 和 default_config
        use crate::managers::config::{CcsConfig, GlobalSettings};
        let (existing_default, existing_current, existing_settings) =
            if self.paths.profiles_file.exists() {
                let content = fs::read_to_string(&self.paths.profiles_file)
                    .map_err(|e| CcrError::ConfigError(format!("读取配置文件失败: {}", e)))?;
                match toml::from_str::<CcsConfig>(&content) {
                    Ok(existing) => (
                        existing.default_config,
                        existing.current_config,
                        existing.settings,
                    ),
                    Err(_) => (
                        profiles
                            .keys()
                            .next()
                            .cloned()
                            .unwrap_or_else(|| "default".to_string()),
                        profiles
                            .keys()
                            .next()
                            .cloned()
                            .unwrap_or_else(|| "default".to_string()),
                        GlobalSettings::default(),
                    ),
                }
            } else {
                (
                    profiles
                        .keys()
                        .next()
                        .cloned()
                        .unwrap_or_else(|| "default".to_string()),
                    profiles
                        .keys()
                        .next()
                        .cloned()
                        .unwrap_or_else(|| "default".to_string()),
                    GlobalSettings::default(),
                )
            };

        // 🔄 验证 current_config 和 default_config 是否仍然存在于 profiles 中
        // 如果不存在，回退到第一个 profile
        let default_config = if sections.contains_key(&existing_default) {
            existing_default
        } else {
            profiles
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "default".to_string())
        };

        let current_config = if sections.contains_key(&existing_current) {
            existing_current
        } else {
            profiles
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "default".to_string())
        };

        // 构建完整配置（保留现有的 current_config 和 default_config）
        let config = CcsConfig {
            default_config,
            current_config,
            settings: existing_settings,
            sections,
        };

        // 序列化为 TOML
        let content = toml::to_string_pretty(&config)
            .map_err(|e| CcrError::ConfigError(format!("序列化配置失败: {}", e)))?;

        // 写入文件
        fs::write(&self.paths.profiles_file, content)
            .map_err(|e| CcrError::ConfigError(format!("写入配置文件失败: {}", e)))?;

        tracing::info!("✅ 已保存 Claude profiles: {:?}", self.paths.profiles_file);
        Ok(())
    }

    /// 🔄 更新 profiles.toml 中的 current_config 字段
    ///
    /// 在配置切换时调用，用于同步更新 profiles.toml 中记录的当前配置名称
    fn update_current_config_in_profiles(&self, name: &str) -> Result<()> {
        // 仅在文件存在时更新
        if !self.paths.profiles_file.exists() {
            return Ok(());
        }

        // 读取现有配置
        let content = fs::read_to_string(&self.paths.profiles_file)
            .map_err(|e| CcrError::ConfigError(format!("读取配置文件失败: {}", e)))?;

        // 解析 TOML
        use crate::managers::config::CcsConfig;
        let mut config: CcsConfig = match toml::from_str(&content) {
            Ok(c) => c,
            Err(_) => {
                // 如果解析失败（可能是旧格式），跳过更新
                tracing::warn!("⚠️ 无法解析 profiles.toml，跳过 current_config 更新");
                return Ok(());
            }
        };

        // 验证目标配置存在
        if !config.sections.contains_key(name) {
            return Err(CcrError::ConfigSectionNotFound(name.to_string()));
        }

        // 更新 current_config
        config.current_config = name.to_string();

        // 序列化并写回
        let new_content = toml::to_string_pretty(&config)
            .map_err(|e| CcrError::ConfigError(format!("序列化配置失败: {}", e)))?;

        fs::write(&self.paths.profiles_file, new_content)
            .map_err(|e| CcrError::ConfigError(format!("写入配置文件失败: {}", e)))?;

        tracing::debug!("✅ 已更新 profiles.toml 的 current_config: {}", name);
        Ok(())
    }

    /// 📖 从 TOML 文件加载 profiles
    fn load_profiles_from_file(&self) -> Result<IndexMap<String, ProfileConfig>> {
        if !self.paths.profiles_file.exists() {
            return Ok(IndexMap::new());
        }

        // 读取文件
        let content = fs::read_to_string(&self.paths.profiles_file)
            .map_err(|e| CcrError::ConfigError(format!("读取配置文件失败: {}", e)))?;

        // 解析 TOML
        use crate::managers::config::{CcsConfig, ConfigSection};
        let sections = match toml::from_str::<CcsConfig>(&content) {
            Ok(config) => config.sections,
            Err(_) => toml::from_str::<IndexMap<String, ConfigSection>>(&content)
                .map_err(|e| CcrError::ConfigFormatInvalid(format!("TOML 解析失败: {}", e)))?,
        };

        // 转换为 ProfileConfig
        let mut profiles = IndexMap::new();
        for (name, section) in sections {
            profiles.insert(name, Self::section_to_profile(&section));
        }

        Ok(profiles)
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
        self.save_profiles(&profiles)
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

        // 同步更新注册表中的 current_profile
        let platform_config_mgr = PlatformConfigManager::with_default()?;
        let mut unified_config = platform_config_mgr.load()?;

        // 更新 Claude 平台的 current_profile
        unified_config.set_platform_profile("claude", name)?;

        // 保存注册表
        platform_config_mgr.save(&unified_config)?;

        tracing::debug!("✅ 已更新注册表 current_profile: {}", name);

        tracing::info!("✅ 已应用 Claude profile: {}", name);
        Ok(())
    }

    fn validate_profile(&self, profile: &ProfileConfig) -> Result<()> {
        // 转换为 ConfigSection 并验证
        let section = Self::profile_to_section(profile)?;
        section.validate()
    }

    fn get_current_profile(&self) -> Result<Option<String>> {
        // 从注册表读取 current_profile
        let platform_config_mgr = PlatformConfigManager::with_default()?;
        let unified_config = platform_config_mgr.load()?;

        // 获取 Claude 平台的注册信息
        let claude_entry = unified_config.get_platform("claude")?;
        Ok(claude_entry.current_profile.clone())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

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
}
