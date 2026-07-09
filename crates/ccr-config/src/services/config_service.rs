// ⚙️ 配置服务
// 封装配置相关的业务逻辑

use crate::managers::config::{CcsConfig, ConfigManager, ConfigSection};
use crate::managers::config_validator::ConfigValidator;
use ccr_core::Validatable;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::lock::{CONFIG_LOCK, LockManager};
use std::sync::Arc;
use std::time::Duration;

/// 📋 配置信息(用于展示)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConfigInfo {
    pub name: String,
    pub description: String,
    pub base_url: Option<String>,
    /// 展示 DTO 持有 Secret：显示走 Display（掩码），需要原文的合法点走 expose()
    pub auth_token: Option<ccr_core::Secret>,
    pub model: Option<String>,
    pub small_fast_model: Option<String>,
    pub is_current: bool,
    pub is_default: bool,
    // === 🆕 分类字段 ===
    pub provider: Option<String>,
    pub provider_type: Option<String>,
    pub account: Option<String>,
    pub tags: Option<Vec<String>>,
    // === 🆕 使用统计和状态字段 ===
    pub usage_count: u32,
    pub enabled: bool,
}

/// 📋 配置列表(用于展示)
#[derive(Debug, Clone)]
pub struct ConfigList {
    pub current_config: String,
    pub default_config: String,
    pub configs: Vec<ConfigInfo>,
}

/// 📊 验证报告
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub valid_count: usize,
    pub invalid_count: usize,
    /// 验证结果：(配置名, 是否有效, 错误消息)
    pub results: Vec<(String, bool, Option<String>)>,
}

/// ⚙️ 配置服务
///
/// 封装所有配置相关的业务逻辑
///
/// **🎯 设计模式：组合模式**
/// - 使用 ConfigValidator 处理验证逻辑
pub struct ConfigService {
    config_manager: Arc<ConfigManager>,
    validator: ConfigValidator,
}

impl ConfigService {
    /// 🏗️ 创建新的配置服务
    ///
    /// 使用组合模式，内部初始化 ConfigValidator
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        Self {
            config_manager,
            validator: ConfigValidator::new(),
        }
    }

    /// 🏠 使用默认配置管理器创建服务
    pub fn with_default() -> Result<Self> {
        let config_manager = Arc::new(ConfigManager::with_default()?);
        Ok(Self::new(config_manager))
    }

    /// 🎯 为指定平台创建配置服务
    ///
    /// 直接加载指定平台的配置，不依赖 `current_platform`。
    /// 适用于 UI 等需要按平台独立管理配置的场景。
    #[allow(dead_code)]
    pub fn for_platform(platform_name: &str) -> Result<Self> {
        let config_manager = Arc::new(ConfigManager::for_platform(platform_name)?);
        Ok(Self::new(config_manager))
    }

    /// 🔐 获取配置锁（跨进程 + 进程内）
    fn lock_config(
        &self,
    ) -> Result<(
        ccr_core::core::lock::FileLock,
        std::sync::MutexGuard<'static, ()>,
    )> {
        let lock_manager = LockManager::with_default_path()?;
        let file_lock = lock_manager.lock_resource("ccr_config", Duration::from_secs(10))?;
        let guard = CONFIG_LOCK.lock().unwrap_or_else(|poisoned| {
            tracing::warn!("配置锁已中毒，尝试恢复");
            poisoned.into_inner()
        });
        Ok((file_lock, guard))
    }

    /// 📋 列出所有配置
    /// 🎯 优化：配合 config.rs 的优化，减少不必要的克隆
    pub fn list_configs(&self) -> Result<ConfigList> {
        let (_file_lock, _guard) = self.lock_config()?;
        let config = self.config_manager.load_with_autofix()?;

        let configs: Vec<ConfigInfo> = config
            .list_sections()
            .filter_map(|name| {
                config
                    .get_section(name.as_str())
                    .ok()
                    .map(|section| ConfigInfo {
                        name: name.clone(),
                        description: section.display_description().to_string(),
                        base_url: section.base_url.clone(),
                        auth_token: section.auth_token.clone(),
                        model: section.model.clone(),
                        small_fast_model: section.small_fast_model.clone(),
                        is_current: name == &config.current_config,
                        is_default: name == &config.default_config,
                        provider: section.provider.clone(),
                        provider_type: section
                            .provider_type
                            .as_ref()
                            .map(|t| t.to_string_value().to_string()),
                        account: section.account.clone(),
                        tags: section.tags.clone(),
                        usage_count: section.usage_count(),
                        enabled: section.is_enabled(),
                    })
            })
            .collect();

        Ok(ConfigList {
            current_config: config.current_config.clone(),
            default_config: config.default_config.clone(),
            configs,
        })
    }

    /// 🔍 获取当前配置信息
    pub fn get_current(&self) -> Result<ConfigInfo> {
        let (_file_lock, _guard) = self.lock_config()?;
        let config = self.config_manager.load_with_autofix()?;
        let section = config.get_current_section()?;

        Ok(ConfigInfo {
            name: config.current_config.clone(),
            description: section.display_description().to_string(),
            base_url: section.base_url.clone(),
            auth_token: section.auth_token.clone(),
            model: section.model.clone(),
            small_fast_model: section.small_fast_model.clone(),
            is_current: true,
            is_default: config.current_config == config.default_config,
            provider: section.provider.clone(),
            provider_type: section
                .provider_type
                .as_ref()
                .map(|t| t.to_string_value().to_string()),
            account: section.account.clone(),
            tags: section.tags.clone(),
            usage_count: section.usage_count(),
            enabled: section.is_enabled(),
        })
    }

    /// 🔍 获取指定配置信息
    #[allow(dead_code)]
    pub fn get_config(&self, name: &str) -> Result<ConfigInfo> {
        let (_file_lock, _guard) = self.lock_config()?;
        let config = self.config_manager.load_with_autofix()?;
        let section = config.get_section(name)?;

        Ok(ConfigInfo {
            name: name.to_string(),
            description: section.display_description().to_string(),
            base_url: section.base_url.clone(),
            auth_token: section.auth_token.clone(),
            model: section.model.clone(),
            small_fast_model: section.small_fast_model.clone(),
            is_current: name == config.current_config,
            is_default: name == config.default_config,
            provider: section.provider.clone(),
            provider_type: section
                .provider_type
                .as_ref()
                .map(|t| t.to_string_value().to_string()),
            account: section.account.clone(),
            tags: section.tags.clone(),
            usage_count: section.usage_count(),
            enabled: section.is_enabled(),
        })
    }

    /// ➕ 添加新配置
    ///
    /// 🔐 **并发安全**: 使用跨进程锁 + CONFIG_LOCK 保护整个 RMW 序列
    pub fn add_config(&self, name: String, section: ConfigSection) -> Result<()> {
        // 验证配置
        section.validate()?;

        let (_file_lock, _guard) = self.lock_config()?;
        let mut config = self.config_manager.load_with_autofix()?;

        // 检查是否已存在
        if config.sections.contains_key(&name) {
            return Err(CcrError::ConfigError(format!("配置 '{}' 已存在", name)));
        }

        config.set_section(name, section);
        self.config_manager.save(&config)?;

        Ok(())
    }

    /// ✏️ 更新现有配置
    ///
    /// 🔐 **并发安全**: 使用跨进程锁 + CONFIG_LOCK 保护整个 RMW 序列
    #[allow(dead_code)]
    pub fn update_config(
        &self,
        old_name: &str,
        new_name: String,
        section: ConfigSection,
    ) -> Result<()> {
        // 验证配置
        section.validate()?;

        let (_file_lock, _guard) = self.lock_config()?;
        let mut config = self.config_manager.load_with_autofix()?;

        // 如果名称改变,需要删除旧配置
        if old_name != new_name {
            config.remove_section(old_name)?;

            // 更新引用
            if config.current_config == old_name {
                config.current_config = new_name.clone();
            }
            if config.default_config == old_name {
                config.default_config = new_name.clone();
            }
        }

        config.set_section(new_name, section);
        self.config_manager.save(&config)?;

        Ok(())
    }

    /// ➖ 删除配置
    ///
    /// 🔐 **并发安全**: 使用跨进程锁 + CONFIG_LOCK 保护整个 RMW 序列
    pub fn delete_config(&self, name: &str) -> Result<()> {
        let (_file_lock, _guard) = self.lock_config()?;
        let mut config = self.config_manager.load_with_autofix()?;

        // 不允许删除当前或默认配置
        if name == config.current_config {
            return Err(CcrError::ValidationError("不能删除当前配置".into()));
        }
        if name == config.default_config {
            return Err(CcrError::ValidationError("不能删除默认配置".into()));
        }

        config.remove_section(name)?;
        self.config_manager.save(&config)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_current(&self, name: &str) -> Result<()> {
        let (_file_lock, _guard) = self.lock_config()?;
        let mut config = self.config_manager.load_with_autofix()?;

        if let Ok(section) = config.get_section(name)
            && !section.is_enabled()
        {
            return Err(CcrError::ConfigError(format!(
                "配置 '{}' 已被禁用，无法切换到此配置",
                name
            )));
        }

        if let Ok(section) = config.get_section_mut(name) {
            section.increment_usage();
            tracing::debug!(
                "📊 递增配置 '{}' 的使用次数: {}",
                name,
                section.usage_count()
            );
        }

        config.set_current(name)?;
        self.config_manager.save(&config)?;
        Ok(())
    }

    /// ✅ 验证所有配置
    ///
    /// 委托给 ConfigValidator 执行验证，返回统一的验证报告
    pub fn validate_all(&self) -> Result<ValidationReport> {
        let (_file_lock, _guard) = self.lock_config()?;
        let config = self.config_manager.load_with_autofix()?;

        // 🎯 使用 ConfigValidator 执行验证
        let validator_report = self.validator.validate_all_sections(&config);

        // 📊 转换为 ConfigService 的 ValidationReport 格式
        let results: Vec<(String, bool, Option<String>)> = validator_report
            .invalid_sections
            .iter()
            .map(|(name, error)| (name.clone(), false, Some(error.clone())))
            .chain(
                config
                    .sections
                    .keys()
                    .filter(|name| !validator_report.invalid_sections.contains_key(*name))
                    .map(|name| (name.clone(), true, None)),
            )
            .collect();

        Ok(ValidationReport {
            valid_count: validator_report.valid_count,
            invalid_count: validator_report.invalid_count,
            results,
        })
    }

    /// 📁 获取配置管理器
    pub fn config_manager(&self) -> &Arc<ConfigManager> {
        &self.config_manager
    }

    /// 📖 加载配置（含自动补全）
    pub fn load_config(&self) -> Result<CcsConfig> {
        let (_file_lock, _guard) = self.lock_config()?;
        self.config_manager.load_with_autofix()
    }

    /// 💾 保存配置
    pub fn save_config(&self, config: &CcsConfig) -> Result<()> {
        let (_file_lock, _guard) = self.lock_config()?;
        self.config_manager.save(config)
    }

    /// 💾 备份配置文件
    pub fn backup_config(&self, tag: Option<&str>) -> Result<std::path::PathBuf> {
        let (_file_lock, _guard) = self.lock_config()?;
        self.config_manager.backup(tag)
    }

    /// 🔄 从已解析的配置备份恢复
    ///
    /// 🔐 **并发安全**: 在同一个跨进程锁 + CONFIG_LOCK 内完成“备份当前配置 + 保存恢复配置”，
    /// 避免调用方分两次进入 service 造成恢复窗口中配置被其他写入打断。
    pub fn restore_config_from_backup(&self, backup_config: &CcsConfig) -> Result<()> {
        let (_file_lock, _guard) = self.lock_config()?;
        self.config_manager.backup(Some("pre_restore"))?;
        self.config_manager.save(backup_config)
    }

    /// 📤 导出配置
    ///
    /// 返回配置的 TOML 字符串
    pub fn export_config(&self, include_secrets: bool) -> Result<String> {
        let (_file_lock, _guard) = self.lock_config()?;
        let mut config = self.config_manager.load_with_autofix()?;

        // 🎯 掩码处理：Secret 的 Display 即统一掩码；用掩码串重建 Secret，
        // 经 expose_plaintext 注解序列化后导出的就是掩码文本（行为与旧版一致）
        if !include_secrets {
            for section in config.sections.values_mut() {
                if let Some(ref token) = section.auth_token {
                    section.auth_token = Some(ccr_core::Secret::new(token.to_string()));
                }
            }
        }

        // 序列化配置
        let content = toml::to_string_pretty(&config)
            .map_err(|e| CcrError::ConfigError(format!("序列化配置失败: {}", e)))?;

        Ok(content)
    }

    /// 📥 导入配置
    ///
    /// 从 TOML 字符串导入配置
    pub fn import_config(
        &self,
        content: &str,
        mode: ImportMode,
        backup: bool,
    ) -> Result<ImportResult> {
        let (_file_lock, _guard) = self.lock_config()?;

        // 解析导入的配置
        let import_config: CcsConfig = toml::from_str(content)
            .map_err(|e| CcrError::ConfigFormatInvalid(format!("解析 TOML 失败: {}", e)))?;

        // 备份当前配置（如果需要）
        if backup && self.config_manager.config_path().exists() {
            self.config_manager.backup(Some("import_backup"))?;
        }

        // 根据模式导入
        let result = match mode {
            ImportMode::Merge => {
                // 合并模式
                if self.config_manager.config_path().exists() {
                    let mut current_config = self.config_manager.load_with_autofix()?;
                    merge_configs(
                        &mut current_config,
                        import_config,
                        self.config_manager.as_ref(),
                    )?
                } else {
                    // 没有现有配置，直接使用导入的
                    self.config_manager.save(&import_config)?;
                    ImportResult {
                        added: import_config.sections.len(),
                        updated: 0,
                        skipped: 0,
                    }
                }
            }
            ImportMode::Replace => {
                // 替换模式
                let count = import_config.sections.len();
                self.config_manager.save(&import_config)?;
                ImportResult {
                    added: count,
                    updated: 0,
                    skipped: 0,
                }
            }
        };

        Ok(result)
    }

    /// ✅ 启用指定配置
    ///
    /// 将配置的 `enabled` 字段设置为 `true`，使其可以被正常使用。
    ///
    /// # 参数
    /// - `name`: 配置名称
    ///
    /// # 并发安全
    /// 使用跨进程锁 + CONFIG_LOCK 保护整个 read-modify-write 序列
    pub fn enable_config(&self, name: &str) -> Result<()> {
        let (_file_lock, _guard) = self.lock_config()?;

        let mut config = self.config_manager.load_with_autofix()?;
        let section = config.get_section_mut(name)?;
        section.enable();

        tracing::info!("✅ 配置 '{}' 已启用", name);
        self.config_manager.save(&config)?;
        Ok(())
    }

    /// ❌ 禁用指定配置
    ///
    /// 将配置的 `enabled` 字段设置为 `false`，使其不能被使用。
    /// 禁用的配置在列表中会显示为灰色/禁用状态。
    ///
    /// # 参数
    /// - `name`: 配置名称
    ///
    /// # 注意
    /// 禁用当前正在使用的配置不会自动切换到其他配置，
    /// 但会在下次切换时发出警告。
    ///
    /// # 并发安全
    /// 使用跨进程锁 + CONFIG_LOCK 保护整个 read-modify-write 序列
    pub fn disable_config(&self, name: &str) -> Result<()> {
        let (_file_lock, _guard) = self.lock_config()?;

        let mut config = self.config_manager.load_with_autofix()?;
        let section = config.get_section_mut(name)?;
        section.disable();

        tracing::info!("❌ 配置 '{}' 已禁用", name);
        self.config_manager.save(&config)?;
        Ok(())
    }
}

/// 📊 导入结果
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub added: usize,
    pub updated: usize,
    pub skipped: usize,
}

/// 📋 导入模式
#[derive(Debug, Clone, Copy)]
pub enum ImportMode {
    /// 🔗 合并模式：保留现有配置，只添加新的
    Merge,
    /// 🔄 覆盖模式：完全替换现有配置
    Replace,
}

/// 合并配置
fn merge_configs(
    current: &mut CcsConfig,
    import: CcsConfig,
    config_manager: &ConfigManager,
) -> Result<ImportResult> {
    let mut result = ImportResult {
        added: 0,
        updated: 0,
        skipped: 0,
    };

    for (name, section) in import.sections {
        if current.sections.contains_key(&name) {
            // 已存在，更新
            current.sections.insert(name, section);
            result.updated += 1;
        } else {
            // 不存在，添加
            current.sections.insert(name, section);
            result.added += 1;
        }
    }

    // 如果导入配置中有 default_config，也更新它
    // 但保持 current_config 不变
    current.default_config = import.default_config;

    config_manager.save(current)?;

    Ok(result)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestCcrEnv;
    use indexmap::IndexMap;

    fn create_test_section() -> ConfigSection {
        ConfigSection {
            description: Some("Test config".into()),
            base_url: Some("https://api.test.com".into()),
            auth_token: Some("sk-test-token".into()),
            model: Some("test-model".into()),
            small_fast_model: Some("test-small".into()),
            provider: None,
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            other: IndexMap::new(),
            ..Default::default()
        }
    }

    #[test]
    fn test_config_service_add_get() {
        let env = TestCcrEnv::new();
        let config_path = env.root().join("config.toml");

        // 创建初始配置
        let mut config = CcsConfig {
            default_config: "test".into(),
            current_config: "test".into(),
            settings: crate::managers::config::GlobalSettings::default(),
            sections: indexmap::IndexMap::new(),
        };
        config.set_section("test".into(), create_test_section());

        let manager = Arc::new(ConfigManager::new(&config_path));
        manager.save(&config).unwrap();

        // 测试服务
        let service = ConfigService::new(manager);

        // 添加新配置
        service
            .add_config("new_config".into(), create_test_section())
            .unwrap();

        // 获取配置
        let info = service.get_config("new_config").unwrap();
        assert_eq!(info.name, "new_config");
        assert_eq!(info.description, "Test config");
    }

    #[test]
    fn restore_config_from_backup_replaces_config_and_creates_pre_restore_backup() {
        let env = TestCcrEnv::new();
        let config_path = env.root().join("config.toml");

        let mut current = CcsConfig {
            default_config: "current".into(),
            current_config: "current".into(),
            settings: crate::managers::config::GlobalSettings::default(),
            sections: indexmap::IndexMap::new(),
        };
        current.set_section("current".into(), create_test_section());

        let mut backup = CcsConfig {
            default_config: "restored".into(),
            current_config: "restored".into(),
            settings: crate::managers::config::GlobalSettings::default(),
            sections: indexmap::IndexMap::new(),
        };
        backup.set_section("restored".into(), create_test_section());

        let manager = Arc::new(ConfigManager::new(&config_path));
        manager.save(&current).unwrap();

        let service = ConfigService::new(manager.clone());
        service.restore_config_from_backup(&backup).unwrap();

        let restored = manager.load().unwrap();
        assert_eq!(restored.current_config, "restored");
        assert!(restored.sections.contains_key("restored"));

        let backups = manager.list_backups().unwrap();
        assert_eq!(backups.len(), 1);
        assert!(
            backups[0]
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap()
                .contains("pre_restore")
        );
    }
}
