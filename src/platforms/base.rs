// 🔧 CCR 平台基础操作模块
// 📦 提供平台共享的 ProfileConfig 加载/保存逻辑
//
// 核心职责:
// - 📋 ProfileConfig ↔ ConfigSection 转换
// - 📖 从 TOML 文件加载 profiles
// - 💾 保存 profiles 到 TOML 文件
// - 🔄 更新 current_config 字段
//
// 设计目标: 消除 claude.rs, codex.rs, gemini.rs 中重复的 ~150 行代码

use crate::core::error::{CcrError, Result};
use crate::managers::PlatformConfigManager;
use crate::managers::config::{CcsConfig, ConfigSection, GlobalSettings, ProviderType};
use crate::models::{PlatformPaths, ProfileConfig};
use crate::utils::toml_json;
use indexmap::IndexMap;
use std::fs;
use std::path::Path;

// ═══════════════════════════════════════════════════════════
// 📋 ProfileConfig ↔ ConfigSection 转换
// ═══════════════════════════════════════════════════════════

/// 📋 从 ConfigSection 转换为 ProfileConfig
pub fn section_to_profile(section: &ConfigSection) -> ProfileConfig {
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
pub fn profile_to_section(profile: &ProfileConfig) -> Result<ConfigSection> {
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

// ═══════════════════════════════════════════════════════════
// 📖 从 TOML 文件加载 profiles
// ═══════════════════════════════════════════════════════════

/// 📖 从 TOML 文件加载 profiles (通用实现)
///
/// 支持两种格式:
/// 1. CcsConfig 完整格式 (包含 default_config, current_config, settings)
/// 2. 简化格式 (仅包含 profile sections)
pub fn load_profiles_from_toml(profiles_path: &Path) -> Result<IndexMap<String, ProfileConfig>> {
    if !profiles_path.exists() {
        return Ok(IndexMap::new());
    }

    // 读取文件
    let content = fs::read_to_string(profiles_path)
        .map_err(|e| CcrError::ConfigError(format!("读取配置文件失败: {}", e)))?;

    // 尝试解析为 CcsConfig 或简化格式
    let sections = match toml::from_str::<CcsConfig>(&content) {
        Ok(config) => config.sections,
        Err(_) => toml::from_str::<IndexMap<String, ConfigSection>>(&content)
            .map_err(|e| CcrError::ConfigFormatInvalid(format!("TOML 解析失败: {}", e)))?,
    };

    // 转换为 ProfileConfig
    let profiles = sections
        .into_iter()
        .map(|(name, section)| (name, section_to_profile(&section)))
        .collect();

    Ok(profiles)
}

// ═══════════════════════════════════════════════════════════
// 💾 保存 profiles 到 TOML 文件
// ═══════════════════════════════════════════════════════════

/// 💾 保存 profiles 到 TOML 文件 (通用实现)
///
/// # 参数
/// - `profiles_path`: profiles.toml 文件路径
/// - `profiles`: 要保存的 profiles
/// - `platform_name`: 平台名称 (用于从注册表读取 current_profile)
/// - `paths`: 平台路径结构 (用于确保目录存在)
pub fn save_profiles_to_toml(
    profiles_path: &Path,
    profiles: &IndexMap<String, ProfileConfig>,
    platform_name: &str,
    paths: &PlatformPaths,
) -> Result<()> {
    // 确保目录存在
    paths.ensure_directories()?;

    // 转换为 ConfigSection 格式
    let mut sections = IndexMap::new();
    for (name, profile) in profiles {
        sections.insert(name.clone(), profile_to_section(profile)?);
    }

    // 📖 读取现有配置，保留 current_config 和 default_config
    let (existing_default, existing_current, existing_settings) = if profiles_path.exists() {
        let content = fs::read_to_string(profiles_path)
            .map_err(|e| CcrError::ConfigError(format!("读取配置文件失败: {}", e)))?;
        match toml::from_str::<CcsConfig>(&content) {
            Ok(existing) => (
                existing.default_config,
                existing.current_config,
                existing.settings,
            ),
            Err(_) => get_default_config_values(profiles),
        }
    } else {
        // 尝试从注册表读取 current_profile
        get_default_from_registry(platform_name, profiles)
    };

    // 🔄 验证 current_config 和 default_config 是否仍然存在于 profiles 中
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

    // 构建完整配置
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
    fs::write(profiles_path, content)
        .map_err(|e| CcrError::ConfigError(format!("写入配置文件失败: {}", e)))?;

    tracing::info!("✅ 已保存 {} profiles: {:?}", platform_name, profiles_path);
    Ok(())
}

/// 📐 获取默认配置值 (当解析失败时)
fn get_default_config_values(
    profiles: &IndexMap<String, ProfileConfig>,
) -> (String, String, GlobalSettings) {
    let first_key = profiles
        .keys()
        .next()
        .cloned()
        .unwrap_or_else(|| "default".to_string());
    (first_key.clone(), first_key, GlobalSettings::default())
}

/// 📐 从注册表获取默认配置
fn get_default_from_registry(
    platform_name: &str,
    profiles: &IndexMap<String, ProfileConfig>,
) -> (String, String, GlobalSettings) {
    let platform_config_mgr = match PlatformConfigManager::with_default() {
        Ok(mgr) => mgr,
        Err(_) => return get_default_config_values(profiles),
    };

    let current_profile = match platform_config_mgr.load() {
        Ok(unified_config) => {
            if let Ok(entry) = unified_config.get_platform(platform_name) {
                entry.current_profile.clone()
            } else {
                None
            }
        }
        Err(_) => None,
    }
    .or_else(|| profiles.keys().next().cloned())
    .unwrap_or_else(|| "default".to_string());

    (
        current_profile.clone(),
        current_profile,
        GlobalSettings::default(),
    )
}

// ═══════════════════════════════════════════════════════════
// 🔄 更新 current_config 字段
// ═══════════════════════════════════════════════════════════

/// 🔄 更新 profiles.toml 中的 current_config 字段
///
/// 在配置切换时调用，用于同步更新 profiles.toml 中记录的当前配置名称
pub fn update_current_config(profiles_path: &Path, name: &str) -> Result<()> {
    // 仅在文件存在时更新
    if !profiles_path.exists() {
        return Ok(());
    }

    // 读取现有配置
    let content = fs::read_to_string(profiles_path)
        .map_err(|e| CcrError::ConfigError(format!("读取配置文件失败: {}", e)))?;

    // 解析 TOML
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

    fs::write(profiles_path, new_content)
        .map_err(|e| CcrError::ConfigError(format!("写入配置文件失败: {}", e)))?;

    tracing::debug!("✅ 已更新 profiles.toml 的 current_config: {}", name);
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// 🔄 更新注册表 current_profile
// ═══════════════════════════════════════════════════════════

/// 🔄 更新注册表中的 current_profile
///
/// 在 apply_profile 后调用，同步更新统一配置管理器中的当前 profile
pub fn update_registry_current_profile(platform_name: &str, profile_name: &str) -> Result<()> {
    let platform_config_mgr = PlatformConfigManager::with_default()?;
    let mut unified_config = platform_config_mgr.load()?;

    // 更新平台的 current_profile
    unified_config.set_platform_profile(platform_name, profile_name)?;

    // 保存注册表
    platform_config_mgr.save(&unified_config)?;

    tracing::debug!("✅ 已更新注册表 current_profile: {}", profile_name);
    Ok(())
}

/// 🔍 获取当前 profile (从注册表)
pub fn get_current_profile_from_registry(platform_name: &str) -> Result<Option<String>> {
    let platform_config_mgr = PlatformConfigManager::with_default()?;
    let unified_config = platform_config_mgr.load()?;

    let entry = unified_config.get_platform(platform_name)?;
    Ok(entry.current_profile.clone())
}

// ═══════════════════════════════════════════════════════════
// 🧪 测试
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_section_to_profile_roundtrip() {
        let section = ConfigSection {
            description: Some("Test".to_string()),
            base_url: Some("https://api.test.com".to_string()),
            auth_token: Some("sk-test".to_string()),
            model: Some("test-model".to_string()),
            small_fast_model: None,
            provider: Some("test-provider".to_string()),
            provider_type: None,
            account: None,
            tags: Some(vec!["tag1".to_string()]),
            usage_count: Some(5),
            enabled: Some(true),
            other: indexmap::IndexMap::new(),
        };

        let profile = section_to_profile(&section);
        assert_eq!(profile.description, section.description);
        assert_eq!(profile.base_url, section.base_url);
        assert_eq!(profile.auth_token, section.auth_token);
        assert_eq!(profile.usage_count, section.usage_count);
        assert_eq!(profile.enabled, section.enabled);

        // 反向转换
        let section2 = profile_to_section(&profile).unwrap();
        assert_eq!(section.description, section2.description);
        assert_eq!(section.base_url, section2.base_url);
        assert_eq!(section.tags, section2.tags);
    }

    #[test]
    fn test_profile_to_section_provider_type() {
        let mut profile = ProfileConfig::new();
        profile.provider_type = Some("official_relay".to_string());

        let section = profile_to_section(&profile).unwrap();
        assert_eq!(section.provider_type, Some(ProviderType::OfficialRelay));

        profile.provider_type = Some("third_party_model".to_string());
        let section2 = profile_to_section(&profile).unwrap();
        assert_eq!(section2.provider_type, Some(ProviderType::ThirdPartyModel));

        profile.provider_type = Some("invalid".to_string());
        let section3 = profile_to_section(&profile).unwrap();
        assert_eq!(section3.provider_type, None);
    }
}
