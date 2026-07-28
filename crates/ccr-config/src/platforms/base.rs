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

use crate::managers::PlatformConfigManager;
use crate::managers::config::{CcsConfig, ConfigSection, GlobalSettings, ProviderType};
use crate::models::{PlatformPaths, ProfileConfig};
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::{BackupPolicy, LockManager, WriteOptions, write_guarded};
use ccr_core::utils::toml_json;
use indexmap::IndexMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const PLATFORM_PROFILE_LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const PLATFORM_REGISTRY_LOCK_RESOURCE: &str = "platform_registry";

fn profile_lock_resource(platform_name: &str) -> String {
    format!("platform_profiles_{}", platform_name)
}

fn save_platform_registry_with_paths<F>(
    registry_path: PathBuf,
    lock_dir: PathBuf,
    mutate: F,
) -> Result<()>
where
    F: FnOnce(&mut crate::managers::UnifiedConfig) -> Result<()>,
{
    let manager = PlatformConfigManager::new(registry_path);
    let lock_manager = LockManager::new(lock_dir);
    let _lock = lock_manager.lock_resource(
        PLATFORM_REGISTRY_LOCK_RESOURCE,
        PLATFORM_PROFILE_LOCK_TIMEOUT,
    )?;

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

fn save_platform_registry<F>(mutate: F) -> Result<()>
where
    F: FnOnce(&mut crate::managers::UnifiedConfig) -> Result<()>,
{
    let manager = PlatformConfigManager::with_default()?;
    let lock_manager = LockManager::with_default_path()?;
    save_platform_registry_with_paths(
        manager.config_path().to_path_buf(),
        lock_manager.lock_dir().to_path_buf(),
        mutate,
    )
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
        default_opus_model: section.default_opus_model.clone(),
        default_sonnet_model: section.default_sonnet_model.clone(),
        default_haiku_model: section.default_haiku_model.clone(),
        default_fable_model: section.default_fable_model.clone(),
        default_opus_model_name: section.default_opus_model_name.clone(),
        default_sonnet_model_name: section.default_sonnet_model_name.clone(),
        default_haiku_model_name: section.default_haiku_model_name.clone(),
        default_fable_model_name: section.default_fable_model_name.clone(),
        subagent_model: section.subagent_model.clone(),
        custom_model_option: section.custom_model_option.clone(),
        custom_model_option_name: section.custom_model_option_name.clone(),
        effort_level: section.effort_level.clone(),
        claude_code_auto_compact_window: section.claude_code_auto_compact_window.clone(),
        api_timeout_ms: section.api_timeout_ms.clone(),
        claude_code_disable_nonessential_traffic: section
            .claude_code_disable_nonessential_traffic
            .clone(),
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
        default_opus_model: profile.default_opus_model.clone(),
        default_sonnet_model: profile.default_sonnet_model.clone(),
        default_haiku_model: profile.default_haiku_model.clone(),
        default_fable_model: profile.default_fable_model.clone(),
        default_opus_model_name: profile.default_opus_model_name.clone(),
        default_sonnet_model_name: profile.default_sonnet_model_name.clone(),
        default_haiku_model_name: profile.default_haiku_model_name.clone(),
        default_fable_model_name: profile.default_fable_model_name.clone(),
        subagent_model: profile.subagent_model.clone(),
        custom_model_option: profile.custom_model_option.clone(),
        custom_model_option_name: profile.custom_model_option_name.clone(),
        effort_level: profile.effort_level.clone(),
        claude_code_auto_compact_window: profile.claude_code_auto_compact_window.clone(),
        api_timeout_ms: profile.api_timeout_ms.clone(),
        claude_code_disable_nonessential_traffic: profile
            .claude_code_disable_nonessential_traffic
            .clone(),
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
pub fn parse_profiles_from_str(content: &str) -> Result<IndexMap<String, ProfileConfig>> {
    let sections = match toml::from_str::<CcsConfig>(content) {
        Ok(config) => config.sections,
        Err(_) => toml::from_str::<IndexMap<String, ConfigSection>>(content)
            .map_err(|_| CcrError::ConfigFormatInvalid("TOML 解析失败：配置包含无效语法".into()))?,
    };

    Ok(sections
        .into_iter()
        .map(|(name, section)| (name, section_to_profile(&section)))
        .collect())
}

/// Read and parse profiles from a TOML file.
pub fn load_profiles_from_toml(profiles_path: &Path) -> Result<IndexMap<String, ProfileConfig>> {
    if !profiles_path.exists() {
        return Ok(IndexMap::new());
    }

    let display_path = profiles_path.display().to_string();

    // 读取文件
    let content = fs::read_to_string(profiles_path)
        .map_err(|e| CcrError::ConfigError(format!("读取配置文件失败 {}: {}", display_path, e)))?;

    parse_profiles_from_str(&content).map_err(|error| {
        CcrError::ConfigFormatInvalid(format!("TOML 解析失败 {display_path}: {error}"))
    })
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

    // 🛡️ 单次 guarded write：备份轮换 + 原子替换合并（命名 RMW 锁在函数开头已持有）
    write_guarded(
        profiles_path,
        content.as_bytes(),
        &WriteOptions {
            backup: BackupPolicy::Dir {
                dir: paths.backups_dir.clone(),
                prefix: "profiles".into(),
            },
            secret: true,
            ..Default::default()
        },
    )?;

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
        .and_then(|parent| {
            // 仅在标准目录结构 (<root>/platforms/<name>/profiles.toml) 时向上推算备份目录
            let grandparent = parent.parent()?;
            if grandparent
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.eq_ignore_ascii_case("platforms"))
            {
                let root = grandparent.parent()?;
                Some(root.join("backups").join(platform_name))
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            profiles_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("backups")
        });

    // 🛡️ 单次 guarded write：备份轮换 + 原子替换合并（命名 RMW 锁在函数开头已持有）
    write_guarded(
        profiles_path,
        new_content.as_bytes(),
        &WriteOptions {
            backup: BackupPolicy::Dir {
                dir: backup_dir,
                prefix: "profiles".into(),
            },
            secret: true,
            ..Default::default()
        },
    )?;

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

    // 📈 记录 provider 激活时间线（尽力而为，失败不影响切换）
    if let Some(root) = crate::managers::provider_activation::default_ccr_root() {
        crate::managers::provider_activation::record_activation(&root, platform_name, profile_name);
    }

    tracing::debug!("✅ 已更新注册表 current_profile: {}", profile_name);
    Ok(())
}

pub fn update_registry_current_profile_with_paths(
    registry_path: &Path,
    lock_dir: &Path,
    platform_name: &str,
    profile_name: &str,
) -> Result<()> {
    save_platform_registry_with_paths(
        registry_path.to_path_buf(),
        lock_dir.to_path_buf(),
        |unified_config| {
            if unified_config.get_platform(platform_name).is_err() {
                tracing::info!("📋 平台 '{}' 未在注册表中，自动注册", platform_name);
                unified_config.register_platform(
                    platform_name.to_string(),
                    crate::managers::platform_config::PlatformConfigEntry::default(),
                )?;
            }

            unified_config.set_platform_profile(platform_name, profile_name)
        },
    )?;

    // 📈 记录 provider 激活时间线（根目录取自 registry_path 的父目录，兼容测试隔离）
    if let Some(root) = registry_path.parent() {
        crate::managers::provider_activation::record_activation(root, platform_name, profile_name);
    }

    tracing::debug!("✅ 已更新注册表 current_profile: {}", profile_name);
    Ok(())
}

/// 🔀 删除当前 profile 后的重定向结果（用于记录激活时间线）
enum ReconcileOutcome {
    Activate(String),
    Clear,
}

/// 把 reconcile 结果写入 provider 激活时间线（尽力而为）
fn record_reconcile_outcome(
    root: Option<&Path>,
    platform_name: &str,
    outcome: Option<ReconcileOutcome>,
) {
    let Some(root) = root else {
        return;
    };
    match outcome {
        Some(ReconcileOutcome::Activate(next)) => {
            crate::managers::provider_activation::record_activation(root, platform_name, &next);
        }
        Some(ReconcileOutcome::Clear) => {
            crate::managers::provider_activation::record_clear(root, platform_name);
        }
        None => {}
    }
}

/// 🔍 获取当前 profile (从注册表)
///
/// 如果平台未在注册表中注册，返回 None 而非报错
pub fn reconcile_registry_current_profile_after_delete(
    platform_name: &str,
    deleted_profile_name: &str,
    remaining_profiles: &IndexMap<String, ProfileConfig>,
) -> Result<()> {
    let mut outcome: Option<ReconcileOutcome> = None;
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
            outcome = Some(ReconcileOutcome::Activate(next_profile_name));
        } else {
            let registry = unified_config.get_platform_mut(platform_name)?;
            registry.current_profile = None;
            registry.last_used = Some(chrono::Utc::now().to_rfc3339());
            outcome = Some(ReconcileOutcome::Clear);
        }

        Ok(())
    })?;
    record_reconcile_outcome(
        crate::managers::provider_activation::default_ccr_root().as_deref(),
        platform_name,
        outcome,
    );
    Ok(())
}

pub fn reconcile_registry_current_profile_after_delete_with_paths(
    registry_path: &Path,
    lock_dir: &Path,
    platform_name: &str,
    deleted_profile_name: &str,
    remaining_profiles: &IndexMap<String, ProfileConfig>,
) -> Result<()> {
    let mut outcome: Option<ReconcileOutcome> = None;
    save_platform_registry_with_paths(
        registry_path.to_path_buf(),
        lock_dir.to_path_buf(),
        |unified_config| {
            let current_profile = match unified_config.get_platform(platform_name) {
                Ok(entry) => entry.current_profile.clone(),
                Err(_) => return Ok(()),
            };

            if current_profile.as_deref() != Some(deleted_profile_name) {
                return Ok(());
            }

            if let Some(next_profile_name) = remaining_profiles.keys().next().cloned() {
                unified_config.set_platform_profile(platform_name, &next_profile_name)?;
                outcome = Some(ReconcileOutcome::Activate(next_profile_name));
            } else {
                let registry = unified_config.get_platform_mut(platform_name)?;
                registry.current_profile = None;
                registry.last_used = Some(chrono::Utc::now().to_rfc3339());
                outcome = Some(ReconcileOutcome::Clear);
            }

            Ok(())
        },
    )?;
    record_reconcile_outcome(registry_path.parent(), platform_name, outcome);
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
    use crate::test_support::TestCcrEnv;

    #[test]
    fn test_section_to_profile_roundtrip() {
        let section = ConfigSection {
            description: Some("Test".to_string()),
            base_url: Some("https://api.test.com".to_string()),
            auth_token: Some(ccr_core::Secret::from("sk-test")),
            model: Some("test-model".to_string()),
            small_fast_model: None,
            provider: Some("test-provider".to_string()),
            provider_type: None,
            account: None,
            tags: Some(vec!["tag1".to_string()]),
            usage_count: Some(5),
            enabled: Some(true),
            other: indexmap::IndexMap::new(),
            ..Default::default()
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
        let _env = TestCcrEnv::new();
        let manager = PlatformConfigManager::with_default().unwrap();
        let mut unified_config = UnifiedConfig::default();
        unified_config
            .register_platform("codex".into(), PlatformConfigEntry::default())
            .unwrap();
        unified_config
            .set_platform_profile("codex", "obsolete")
            .unwrap();
        manager.save(&unified_config).unwrap();

        let mut remaining_profiles = IndexMap::new();
        remaining_profiles.insert("replacement".to_string(), ProfileConfig::new());

        reconcile_registry_current_profile_after_delete("codex", "obsolete", &remaining_profiles)
            .unwrap();

        let reloaded = manager.load().unwrap();
        assert_eq!(
            reloaded
                .get_platform("codex")
                .unwrap()
                .current_profile
                .as_deref(),
            Some("replacement")
        );
    }

    #[test]
    fn test_reconcile_registry_current_profile_after_delete_clears_when_empty() {
        let _env = TestCcrEnv::new();
        let manager = PlatformConfigManager::with_default().unwrap();
        let mut unified_config = UnifiedConfig::default();
        unified_config
            .register_platform("gemini".into(), PlatformConfigEntry::default())
            .unwrap();
        unified_config
            .set_platform_profile("gemini", "obsolete")
            .unwrap();
        manager.save(&unified_config).unwrap();

        let remaining_profiles = IndexMap::new();
        reconcile_registry_current_profile_after_delete("gemini", "obsolete", &remaining_profiles)
            .unwrap();

        let reloaded = manager.load().unwrap();
        assert_eq!(
            reloaded.get_platform("gemini").unwrap().current_profile,
            None
        );
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

    #[test]
    fn test_parse_profiles_from_str_supports_full_and_simplified_formats() {
        let full = r#"
default_config = "first"
current_config = "first"

[first]
base_url = "https://example.com"
auth_token = "secret"
"#;
        let simplified = r#"
[second]
model = "example-model"
"#;

        let full_profiles = parse_profiles_from_str(full).unwrap();
        let simplified_profiles = parse_profiles_from_str(simplified).unwrap();

        assert_eq!(full_profiles.len(), 1);
        assert_eq!(
            full_profiles["first"].base_url.as_deref(),
            Some("https://example.com")
        );
        assert_eq!(simplified_profiles.len(), 1);
        assert_eq!(
            simplified_profiles["second"].model.as_deref(),
            Some("example-model")
        );
    }

    #[test]
    fn test_parse_profiles_from_str_preserves_empty_collection_behavior() {
        assert!(parse_profiles_from_str("").unwrap().is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn test_profile_structured_writes_keep_owner_only_permissions() {
        use crate::models::Platform;
        use std::os::unix::fs::PermissionsExt;

        let _env = TestCcrEnv::new();
        let paths = PlatformPaths::new(Platform::Claude).unwrap();
        let mut profiles = IndexMap::new();
        profiles.insert("default".to_string(), ProfileConfig::new());

        save_profiles_to_toml(&paths.profiles_file, &profiles, "claude", &paths).unwrap();
        assert_eq!(
            fs::metadata(&paths.profiles_file)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );

        fs::set_permissions(&paths.profiles_file, fs::Permissions::from_mode(0o644)).unwrap();
        update_current_config(&paths.profiles_file, "default").unwrap();
        assert_eq!(
            fs::metadata(&paths.profiles_file)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}
