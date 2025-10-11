// ⚙️ 配置服务
// 封装配置相关的业务逻辑

use crate::config::{CcsConfig, ConfigManager, ConfigSection};
use crate::error::{CcrError, Result};
use crate::utils::Validatable;
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
    pub fn list_configs(&self) -> Result<ConfigList> {
        let config = self.config_manager.load()?;

        let configs: Vec<ConfigInfo> = config
            .list_sections()
            .into_iter()
            .filter_map(|name| {
                config.get_section(&name).ok().map(|section| ConfigInfo {
                    name: name.clone(),
                    description: section.display_description(),
                    base_url: section.base_url.clone(),
                    auth_token: section.auth_token.clone(),
                    model: section.model.clone(),
                    small_fast_model: section.small_fast_model.clone(),
                    is_current: name == config.current_config,
                    is_default: name == config.default_config,
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
            description: section.display_description(),
            base_url: section.base_url.clone(),
            auth_token: section.auth_token.clone(),
            model: section.model.clone(),
            small_fast_model: section.small_fast_model.clone(),
            is_current: true,
            is_default: config.current_config == config.default_config,
        })
    }

    /// 🔍 获取指定配置信息
    pub fn get_config(&self, name: &str) -> Result<ConfigInfo> {
        let config = self.config_manager.load()?;
        let section = config.get_section(name)?;

        Ok(ConfigInfo {
            name: name.to_string(),
            description: section.display_description(),
            base_url: section.base_url.clone(),
            auth_token: section.auth_token.clone(),
            model: section.model.clone(),
            small_fast_model: section.small_fast_model.clone(),
            is_current: name == config.current_config,
            is_default: name == config.default_config,
        })
    }

    /// ➕ 添加新配置
    pub fn add_config(&self, name: String, section: ConfigSection) -> Result<()> {
        // 验证配置
        section.validate()?;

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
    pub fn update_config(
        &self,
        old_name: &str,
        new_name: String,
        section: ConfigSection,
    ) -> Result<()> {
        // 验证配置
        section.validate()?;

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
    pub fn delete_config(&self, name: &str) -> Result<()> {
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
    pub fn set_current(&self, name: &str) -> Result<()> {
        let mut config = self.config_manager.load()?;
        config.set_current(name)?;
        self.config_manager.save(&config)?;
        Ok(())
    }

    /// ✅ 验证所有配置
    pub fn validate_all(&self) -> Result<ValidationReport> {
        let config = self.config_manager.load()?;
        let validation_results = config.validate_all();

        let mut valid_count = 0;
        let mut invalid_count = 0;
        let mut results = Vec::new();

        for (name, result) in validation_results {
            match result {
                Ok(_) => {
                    valid_count += 1;
                    results.push((name, true, None));
                }
                Err(e) => {
                    invalid_count += 1;
                    results.push((name, false, Some(e.to_string())));
                }
            }
        }

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
