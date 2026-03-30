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
use crate::core::{AtomicWriter, LockManager};
use crate::managers::PlatformConfigManager;
use crate::managers::config::{CcsConfig, ConfigSection, GlobalSettings, ProviderType};
use crate::models::{PlatformPaths, ProfileConfig};
use crate::utils::toml_json;
use chrono::Local;
use indexmap::IndexMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

const PLATFORM_PROFILE_LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const PLATFORM_PROFILE_BACKUP_KEEP: usize = 10;
const PLATFORM_REGISTRY_LOCK_RESOURCE: &str = "platform_registry";

fn profile_lock_resource(platform_name: &str) -> String {
    format!("platform_profiles_{}", platform_name)
}

fn backup_with_rotation(source: &Path, backup_dir: &Path, prefix: &str) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }

    fs::create_dir_all(backup_dir)
        .map_err(|e| CcrError::ConfigError(format!("创建备份目录失败 {:?}: {}", backup_dir, e)))?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let extension = source
        .extension()
        .and_then(|ext| ext.to_str())
        .filter(|ext| !ext.is_empty())
        .unwrap_or("bak");
    let backup_path = backup_dir.join(format!("{prefix}.{timestamp}.{extension}.bak"));

    fs::copy(source, &backup_path).map_err(|e| {
        CcrError::ConfigError(format!(
            "备份文件失败 {:?} -> {:?}: {}",
            source, backup_path, e
        ))
    })?;

    let mut backups: Vec<_> = fs::read_dir(backup_dir)
        .map_err(|e| CcrError::ConfigError(format!("读取备份目录失败 {:?}: {}", backup_dir, e)))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(prefix) && name.ends_with(".bak"))
        })
        .collect();

    backups.sort_by(|a, b| {
        let a_time = fs::metadata(a).and_then(|m| m.modified()).ok();
        let b_time = fs::metadata(b).and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    if backups.len() > PLATFORM_PROFILE_BACKUP_KEEP {
        for old in &backups[PLATFORM_PROFILE_BACKUP_KEEP..] {
            if let Err(err) = fs::remove_file(old) {
                tracing::warn!("清理旧备份失败 {:?}: {}", old, err);
            }
        }
    }

    Ok(())
}

fn save_platform_registry<F>(mutate: F) -> Result<()>
where
    F: FnOnce(&mut crate::managers::UnifiedConfig) -> Result<()>,
{
    let manager = PlatformConfigManager::with_default()?;
    let lock_manager = LockManager::with_default_path()?;
    let _lock =
        lock_manager.lock_resource(PLATFORM_REGISTRY_LOCK_RESOURCE, PLATFORM_PROFILE_LOCK_TIMEOUT)?;

    let mut unified_config = manager.load_or_create_default()?;
    let config_exists = manager.config_path().exists();
    mutate(&mut unified_config)?;

    if config_exists {
        let tag = "platform_profile_mutation".to_string();
        manager.backup(Some(&tag))?;
    }

    manager.save(&unified_config)?;
    Ok(())
}

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
            "third_party" => Some(ProviderType::ThirdPartyModel),
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

    let display_path = profiles_path.display().to_string();

    // 读取文件
    let content = fs::read_to_string(profiles_path)
        .map_err(|e| CcrError::ConfigError(format!("读取配置文件失败 {}: {}", display_path, e)))?;

    // 尝试解析为 CcsConfig 或简化格式
    let sections = match toml::from_str::<CcsConfig>(&content) {
        Ok(config) => config.sections,
        Err(_) => toml::from_str::<IndexMap<String, ConfigSection>>(&content).map_err(|e| {
            CcrError::ConfigFormatInvalid(format!("TOML 解析失败 {}: {}", display_path, e))
        })?,
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
    let lock_manager = LockManager::with_default_path()?;
    let lock_name = profile_lock_resource(platform_name);
    let _lock = lock_manager.lock_resource(&lock_name, PLATFORM_PROFILE_LOCK_TIMEOUT)?;

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

    backup_with_rotation(profiles_path, &paths.backups_dir, "profiles")?;

    AtomicWriter::new(profiles_path)
        .write_string(&content)
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

    let platform_name = profiles_path
        .parent()
        .and_then(|parent| parent.file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");
    let lock_manager = LockManager::with_default_path()?;
    let lock_name = profile_lock_resource(platform_name);
    let _lock = lock_manager.lock_resource(&lock_name, PLATFORM_PROFILE_LOCK_TIMEOUT)?;

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

    let backup_dir = profiles_path
        .parent()
        .and_then(|parent| parent.parent())
        .and_then(|platforms_dir| platforms_dir.parent())
        .map(|root| root.join("backups").join(platform_name))
        .unwrap_or_else(|| {
            profiles_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("backups")
        });
    backup_with_rotation(profiles_path, &backup_dir, "profiles")?;

    AtomicWriter::new(profiles_path)
        .write_string(&new_content)
        .map_err(|e| CcrError::ConfigError(format!("写入配置文件失败: {}", e)))?;

    tracing::debug!("✅ 已更新 profiles.toml 的 current_config: {}", name);
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// 🔄 更新注册表 current_profile
// ═══════════════════════════════════════════════════════════

/// 🔄 更新注册表中的 current_profile
///
/// 在 apply_profile 后调用，同步更新统一配置管理器中的当前 profile。
/// 如果平台尚未在注册表中注册，会自动补全条目。
pub fn update_registry_current_profile(platform_name: &str, profile_name: &str) -> Result<()> {
    save_platform_registry(|unified_config| {
        // 如果平台不存在于注册表中，自动注册
        if unified_config.get_platform(platform_name).is_err() {
            tracing::info!("📋 平台 '{}' 未在注册表中，自动注册", platform_name);
            unified_config.register_platform(
                platform_name.to_string(),
                crate::managers::platform_config::PlatformConfigEntry::default(),
            )?;
        }

        unified_config.set_platform_profile(platform_name, profile_name)
    })?;

    tracing::debug!("✅ 已更新注册表 current_profile: {}", profile_name);
    Ok(())
}

/// 🔍 获取当前 profile (从注册表)
///
/// 如果平台未在注册表中注册，返回 None 而非报错
pub fn reconcile_registry_current_profile_after_delete(
    platform_name: &str,
    deleted_profile_name: &str,
    remaining_profiles: &IndexMap<String, ProfileConfig>,
) -> Result<()> {
    save_platform_registry(|unified_config| {
        let current_profile = match unified_config.get_platform(platform_name) {
            Ok(entry) => entry.current_profile.clone(),
            Err(_) => return Ok(()),
        };

        if current_profile.as_deref() != Some(deleted_profile_name) {
            return Ok(());
        }

        if let Some(next_profile_name) = remaining_profiles.keys().next().cloned() {
            unified_config.set_platform_profile(platform_name, &next_profile_name)?;
        } else {
            let registry = unified_config.get_platform_mut(platform_name)?;
            registry.current_profile = None;
            registry.last_used = Some(chrono::Utc::now().to_rfc3339());
        }

        Ok(())
    })?;
    Ok(())
}

pub fn get_current_profile_from_registry(platform_name: &str) -> Result<Option<String>> {
    let platform_config_mgr = PlatformConfigManager::with_default()?;
    let unified_config = platform_config_mgr.load()?;

    match unified_config.get_platform(platform_name) {
        Ok(entry) => Ok(entry.current_profile.clone()),
        Err(_) => Ok(None),
    }
}

// ═══════════════════════════════════════════════════════════
// 🧪 测试
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::managers::{PlatformConfigEntry, UnifiedConfig};
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

    #[test]
    fn test_reconcile_registry_current_profile_after_delete_repoints_to_first_remaining() {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let previous_root = std::env::var("CCR_ROOT").ok();

        unsafe {
            std::env::set_var("CCR_ROOT", temp_dir.path());
        }

        let result = (|| -> Result<()> {
            let manager = PlatformConfigManager::with_default()?;
            let mut unified_config = UnifiedConfig::default();
            unified_config.register_platform("codex".into(), PlatformConfigEntry::default())?;
            unified_config.set_platform_profile("codex", "obsolete")?;
            manager.save(&unified_config)?;

            let mut remaining_profiles = IndexMap::new();
            remaining_profiles.insert("replacement".to_string(), ProfileConfig::new());

            reconcile_registry_current_profile_after_delete(
                "codex",
                "obsolete",
                &remaining_profiles,
            )?;

            let reloaded = manager.load()?;
            assert_eq!(
                reloaded.get_platform("codex")?.current_profile.as_deref(),
                Some("replacement")
            );
            Ok(())
        })();

        restore_env_var("CCR_ROOT", previous_root);
        result.unwrap();
    }

    #[test]
    fn test_reconcile_registry_current_profile_after_delete_clears_when_empty() {
        let _guard = ENV_LOCK.lock().unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        let previous_root = std::env::var("CCR_ROOT").ok();

        unsafe {
            std::env::set_var("CCR_ROOT", temp_dir.path());
        }

        let result = (|| -> Result<()> {
            let manager = PlatformConfigManager::with_default()?;
            let mut unified_config = UnifiedConfig::default();
            unified_config.register_platform("gemini".into(), PlatformConfigEntry::default())?;
            unified_config.set_platform_profile("gemini", "obsolete")?;
            manager.save(&unified_config)?;

            let remaining_profiles = IndexMap::new();
            reconcile_registry_current_profile_after_delete(
                "gemini",
                "obsolete",
                &remaining_profiles,
            )?;

            let reloaded = manager.load()?;
            assert_eq!(reloaded.get_platform("gemini")?.current_profile, None);
            Ok(())
        })();

        restore_env_var("CCR_ROOT", previous_root);
        result.unwrap();
    }

    #[test]
    fn test_load_profiles_from_toml_reports_path_on_parse_error() {
        let temp_dir = tempfile::tempdir().unwrap();
        let profiles_path = temp_dir.path().join("profiles.toml");
        fs::write(&profiles_path, "not = [valid toml").unwrap();

        let err = load_profiles_from_toml(&profiles_path).unwrap_err();
        let err_text = err.to_string();

        assert!(err_text.contains("profiles.toml"));
        assert!(err_text.contains("TOML 解析失败"));
    }
}
