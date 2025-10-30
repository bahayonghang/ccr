// ⚙️ 配置服务
// 封装配置相关的业务逻辑

use crate::core::error::{CcrError, Result};
use crate::managers::config::{CcsConfig, ConfigManager, ConfigSection};
use crate::utils::Validatable;
use rayon::prelude::*;
use std::sync::Arc;

/// 📋 配置信息(用于展示)
#[derive(Debug, Clone)]
pub struct ConfigInfo {
    pub name: String,
    pub description: String,
    pub base_url: Option<String>,
    pub auth_token: Option<String>,
    pub model: Option<String>,
    pub small_fast_model: Option<String>,
    pub is_current: bool,
    pub is_default: bool,
    // === 🆕 分类字段 ===
    pub provider: Option<String>,
    pub provider_type: Option<String>,
    pub account: Option<String>,
    pub tags: Option<Vec<String>>,
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
#[allow(dead_code)]
pub struct ValidationReport {
    pub valid_count: usize,
    pub invalid_count: usize,
    /// 验证结果：(配置名, 是否有效, 错误消息)
    pub results: Vec<(String, bool, Option<String>)>,
}

/// ⚙️ 配置服务
///
/// 封装所有配置相关的业务逻辑
pub struct ConfigService {
    config_manager: Arc<ConfigManager>,
}

#[allow(dead_code)]
impl ConfigService {
    /// 🏗️ 创建新的配置服务
    pub fn new(config_manager: Arc<ConfigManager>) -> Self {
        Self { config_manager }
    }

    /// 🏠 使用默认配置管理器创建服务
    pub fn default() -> Result<Self> {
        let config_manager = Arc::new(ConfigManager::default()?);
        Ok(Self::new(config_manager))
    }

    /// 📋 列出所有配置
    /// 🎯 优化：配合 config.rs 的优化，减少不必要的克隆
    pub fn list_configs(&self) -> Result<ConfigList> {
        let config = self.config_manager.load()?;

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
        let config = self.config_manager.load()?;
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
        })
    }

    /// 🔍 获取指定配置信息
    pub fn get_config(&self, name: &str) -> Result<ConfigInfo> {
        let config = self.config_manager.load()?;
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
        })
    }

    /// ➕ 添加新配置
    ///
    /// 🔐 **并发安全**: 使用 CONFIG_LOCK 保护整个 RMW 序列
    pub fn add_config(&self, name: String, section: ConfigSection) -> Result<()> {
        // 验证配置
        section.validate()?;

        // 🔒 获取进程内配置锁，保护整个 read-modify-write 序列
        let _guard = crate::core::lock::CONFIG_LOCK.lock().expect("配置锁已中毒");

        let mut config = self.config_manager.load()?;

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
    /// 🔐 **并发安全**: 使用 CONFIG_LOCK 保护整个 RMW 序列
    pub fn update_config(
        &self,
        old_name: &str,
        new_name: String,
        section: ConfigSection,
    ) -> Result<()> {
        // 验证配置
        section.validate()?;

        // 🔒 获取进程内配置锁，保护整个 read-modify-write 序列
        let _guard = crate::core::lock::CONFIG_LOCK.lock().expect("配置锁已中毒");

        let mut config = self.config_manager.load()?;

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
    /// 🔐 **并发安全**: 使用 CONFIG_LOCK 保护整个 RMW 序列
    pub fn delete_config(&self, name: &str) -> Result<()> {
        // 🔒 获取进程内配置锁，保护整个 read-modify-write 序列
        let _guard = crate::core::lock::CONFIG_LOCK.lock().expect("配置锁已中毒");

        let mut config = self.config_manager.load()?;

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

    /// 🔄 设置当前配置
    ///
    /// 注意：这只更新配置文件中的 current_config 标记,
    /// 不会修改 settings.json。要完整切换配置,应使用 switch_config。
    ///
    /// 🔐 **并发安全**: 使用 CONFIG_LOCK 保护整个 RMW 序列
    pub fn set_current(&self, name: &str) -> Result<()> {
        // 🔒 获取进程内配置锁，保护整个 read-modify-write 序列
        let _guard = crate::core::lock::CONFIG_LOCK.lock().expect("配置锁已中毒");

        let mut config = self.config_manager.load()?;
        config.set_current(name)?;
        self.config_manager.save(&config)?;
        Ok(())
    }

    /// ✅ 验证所有配置
    /// 🎯 优化：使用 rayon 并行验证，提升性能
    pub fn validate_all(&self) -> Result<ValidationReport> {
        let config = self.config_manager.load()?;

        // 🚀 并行验证所有配置节
        // 收集所有配置节的名称和引用，然后并行验证
        let sections: Vec<(&String, &ConfigSection)> = config.sections.iter().collect();

        let results: Vec<(String, bool, Option<String>)> = sections
            .par_iter()
            .map(|(name, section)| match section.validate() {
                Ok(_) => ((*name).clone(), true, None),
                Err(e) => ((*name).clone(), false, Some(e.to_string())),
            })
            .collect();

        // 统计验证结果
        let valid_count = results.iter().filter(|(_, is_valid, _)| *is_valid).count();
        let invalid_count = results.len() - valid_count;

        Ok(ValidationReport {
            valid_count,
            invalid_count,
            results,
        })
    }

    /// 📁 获取配置管理器
    pub fn config_manager(&self) -> &Arc<ConfigManager> {
        &self.config_manager
    }

    /// 📖 加载原始配置
    pub fn load_config(&self) -> Result<CcsConfig> {
        self.config_manager.load()
    }

    /// 💾 保存配置
    pub fn save_config(&self, config: &CcsConfig) -> Result<()> {
        self.config_manager.save(config)
    }

    /// 📤 导出配置
    ///
    /// 返回配置的 TOML 字符串
    pub fn export_config(&self, include_secrets: bool) -> Result<String> {
        let mut config = self.config_manager.load()?;

        // 🎯 优化：统一使用 utils::mask_sensitive 进行掩码处理
        if !include_secrets {
            for section in config.sections.values_mut() {
                if let Some(ref token) = section.auth_token {
                    section.auth_token = Some(crate::utils::mask_sensitive(token));
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
        // 解析导入的配置
        let import_config: CcsConfig = toml::from_str(content)
            .map_err(|e| CcrError::ConfigFormatInvalid(format!("解析 TOML 失败: {}", e)))?;

        // 备份当前配置（如果需要）
        if backup && self.config_manager.config_path().exists() {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let backup_path = self
                .config_manager
                .config_path()
                .with_extension(format!("toml.import_backup_{}.bak", timestamp));

            std::fs::copy(self.config_manager.config_path(), &backup_path)
                .map_err(|e| CcrError::ConfigError(format!("备份失败: {}", e)))?;
        }

        // 根据模式导入
        let result = match mode {
            ImportMode::Merge => {
                // 合并模式
                if self.config_manager.config_path().exists() {
                    let mut current_config = self.config_manager.load()?;
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
mod tests {
    use super::*;
    use tempfile::tempdir;

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
        }
    }

    #[test]
    fn test_config_service_add_get() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

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
}
