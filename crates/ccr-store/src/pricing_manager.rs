// 💰 CCR 价格表管理器
// 负责模型定价配置的管理

use crate::models::{ModelPricing, PricingConfig};
use ccr_core::core::error::{CcrError, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// 💰 价格表管理器
pub struct PricingManager {
    /// 📁 配置文件路径
    config_path: PathBuf,

    /// 💰 价格表配置
    config: PricingConfig,
}

#[allow(dead_code)]
impl PricingManager {
    /// 创建新的价格表管理器
    pub fn new(config_path: PathBuf) -> Result<Self> {
        let config = if config_path.exists() {
            Self::load_config(&config_path)?
        } else {
            // 默认加载 Claude 模型定价
            PricingConfig::with_claude_defaults()
        };

        Ok(Self {
            config_path,
            config,
        })
    }

    /// 获取默认配置路径
    pub fn default_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".to_string()))?;
        Ok(home.join(".claude").join("pricing.toml"))
    }

    /// 从默认路径创建价格表管理器
    pub fn with_default() -> Result<Self> {
        let config_path = Self::default_config_path()?;
        Self::new(config_path)
    }

    /// 从 Claude 默认配置创建
    #[allow(dead_code)]
    pub fn with_claude_defaults() -> Result<Self> {
        let config_path = Self::default_config_path()?;
        let config = PricingConfig::with_claude_defaults();

        let manager = Self {
            config_path,
            config,
        };

        // 自动保存默认配置
        manager.save_config()?;
        Ok(manager)
    }

    /// 加载配置文件
    fn load_config(path: &Path) -> Result<PricingConfig> {
        let content = fs::read_to_string(path)?;
        let config: PricingConfig = toml::from_str(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析价格表配置失败: {}", e)))?;

        config.validate().map_err(CcrError::ValidationError)?;
        Ok(config)
    }

    /// 保存配置文件
    fn save_config(&self) -> Result<()> {
        self.config.validate().map_err(CcrError::ValidationError)?;

        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| CcrError::ConfigError(format!("序列化价格表配置失败: {}", e)))?;

        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let parent = self
            .config_path
            .parent()
            .ok_or_else(|| CcrError::ConfigError("价格表配置路径缺少父目录".to_string()))?;

        let mut temp_file = NamedTempFile::new_in(parent)?;
        temp_file.write_all(content.as_bytes())?;
        temp_file.flush()?;
        temp_file.as_file_mut().sync_all()?;
        temp_file
            .persist(&self.config_path)
            .map_err(|error| CcrError::IoError(error.error))?;
        Ok(())
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &PricingConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: PricingConfig) -> Result<()> {
        config.validate().map_err(CcrError::ValidationError)?;
        self.config = config;
        self.save_config()
    }

    /// 获取模型定价
    pub fn get_pricing(&self, model: &str) -> Option<&ModelPricing> {
        self.config.get_pricing(model)
    }

    /// 获取模型定价或默认定价
    pub fn get_or_default_pricing(&self, model: &str) -> Option<&ModelPricing> {
        self.config.get_or_default_pricing(model)
    }

    /// 设置模型定价
    pub fn set_pricing(&mut self, model: String, pricing: ModelPricing) -> Result<()> {
        self.config.set_pricing(model, pricing);
        self.save_config()
    }

    /// 移除模型定价
    pub fn remove_pricing(&mut self, model: &str) -> Result<Option<ModelPricing>> {
        let result = self.config.remove_pricing(model);
        if result.is_some() {
            self.save_config()?;
        }
        Ok(result)
    }

    /// 设置默认定价
    pub fn set_default_pricing(&mut self, pricing: ModelPricing) -> Result<()> {
        self.config.set_default_pricing(pricing);
        self.save_config()
    }

    /// 清除默认定价
    pub fn clear_default_pricing(&mut self) -> Result<()> {
        self.config.clear_default_pricing();
        self.save_config()
    }

    /// 清空所有定价
    pub fn clear_all(&mut self) -> Result<()> {
        self.config.clear_all();
        self.save_config()
    }

    /// 重置为 Claude 默认定价
    pub fn reset_to_defaults(&mut self) -> Result<()> {
        self.config = PricingConfig::with_claude_defaults();
        self.save_config()
    }

    /// 获取所有模型名称列表
    pub fn model_names(&self) -> Vec<String> {
        self.config.model_names()
    }

    /// 获取已配置的模型数量
    pub fn model_count(&self) -> usize {
        self.config.model_count()
    }

    /// 检查配置是否为空
    pub fn is_empty(&self) -> bool {
        self.config.is_empty()
    }

    /// 批量导入定价
    pub fn import_pricing(&mut self, pricing_list: Vec<(String, ModelPricing)>) -> Result<()> {
        for (model, pricing) in pricing_list {
            self.config.set_pricing(model, pricing);
        }
        self.save_config()
    }

    /// 导出所有定价
    pub fn export_pricing(&self) -> Vec<(String, ModelPricing)> {
        let mut result = Vec::new();
        for model_name in self.config.model_names() {
            if let Some(pricing) = self.config.get_pricing(&model_name) {
                result.push((model_name, pricing.clone()));
            }
        }
        result
    }

    /// 合并另一个价格表配置
    pub fn merge_config(&mut self, other: &PricingConfig) -> Result<()> {
        self.config.merge(other);
        self.save_config()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pricing_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("pricing.toml");

        let manager = PricingManager::new(config_path).unwrap();
        // 默认加载 Claude 定价
        assert!(!manager.is_empty());
        assert!(manager.model_count() >= 4);
    }

    #[test]
    fn test_set_and_get_pricing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("pricing.toml");

        let mut manager = PricingManager::new(config_path).unwrap();

        let pricing = ModelPricing {
            model: "test-model".to_string(),
            input_price: 1.0,
            output_price: 2.0,
            cache_read_price: Some(0.1),
            cache_write_price: Some(0.2),
        };

        manager
            .set_pricing("test-model".to_string(), pricing.clone())
            .unwrap();

        let retrieved = manager.get_pricing("test-model");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().input_price, 1.0);
    }

    #[test]
    fn test_remove_pricing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("pricing.toml");

        let mut manager = PricingManager::new(config_path).unwrap();

        let pricing = ModelPricing {
            model: "test-model".to_string(),
            input_price: 1.0,
            output_price: 2.0,
            cache_read_price: None,
            cache_write_price: None,
        };

        manager
            .set_pricing("test-model".to_string(), pricing)
            .unwrap();

        let removed = manager.remove_pricing("test-model").unwrap();
        assert!(removed.is_some());
        assert!(manager.get_pricing("test-model").is_none());
    }

    #[test]
    fn test_default_pricing() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("pricing.toml");

        let mut manager = PricingManager::new(config_path).unwrap();

        let default_pricing = ModelPricing {
            model: "default".to_string(),
            input_price: 5.0,
            output_price: 10.0,
            cache_read_price: None,
            cache_write_price: None,
        };

        manager.set_default_pricing(default_pricing).unwrap();

        // 对于不存在的模型，应返回默认定价
        let result = manager.get_or_default_pricing("non-existent-model");
        assert!(result.is_some());
        assert_eq!(result.unwrap().input_price, 5.0);

        manager.clear_default_pricing().unwrap();
        assert!(
            manager
                .get_or_default_pricing("non-existent-model")
                .is_none()
        );
    }

    #[test]
    fn test_reset_to_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("pricing.toml");

        let mut manager = PricingManager::new(config_path).unwrap();

        // 清空所有定价
        manager.clear_all().unwrap();
        assert!(manager.is_empty());

        // 重置为 Claude 默认定价
        manager.reset_to_defaults().unwrap();
        assert!(!manager.is_empty());
        assert!(manager.model_count() >= 4);
    }

    #[test]
    fn test_model_names() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("pricing.toml");

        let mut manager = PricingManager::new(config_path).unwrap();

        // 清空并添加测试数据
        manager.clear_all().unwrap();

        manager
            .set_pricing(
                "model-b".to_string(),
                ModelPricing {
                    model: "model-b".to_string(),
                    input_price: 1.0,
                    output_price: 2.0,
                    cache_read_price: None,
                    cache_write_price: None,
                },
            )
            .unwrap();

        manager
            .set_pricing(
                "model-a".to_string(),
                ModelPricing {
                    model: "model-a".to_string(),
                    input_price: 1.0,
                    output_price: 2.0,
                    cache_read_price: None,
                    cache_write_price: None,
                },
            )
            .unwrap();

        let names = manager.model_names();
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "model-a"); // 应该已排序
        assert_eq!(names[1], "model-b");
    }

    #[test]
    fn test_import_export() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("pricing.toml");

        let mut manager = PricingManager::new(config_path).unwrap();
        manager.clear_all().unwrap();

        // 导入定价
        let pricing_list = vec![
            (
                "model-1".to_string(),
                ModelPricing {
                    model: "model-1".to_string(),
                    input_price: 1.0,
                    output_price: 2.0,
                    cache_read_price: None,
                    cache_write_price: None,
                },
            ),
            (
                "model-2".to_string(),
                ModelPricing {
                    model: "model-2".to_string(),
                    input_price: 3.0,
                    output_price: 4.0,
                    cache_read_price: None,
                    cache_write_price: None,
                },
            ),
        ];

        manager.import_pricing(pricing_list).unwrap();
        assert_eq!(manager.model_count(), 2);

        // 导出定价
        let exported = manager.export_pricing();
        assert_eq!(exported.len(), 2);
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("pricing.toml");

        {
            let mut manager = PricingManager::new(config_path.clone()).unwrap();
            manager.clear_all().unwrap();
            manager
                .set_pricing(
                    "test-model".to_string(),
                    ModelPricing {
                        model: "test-model".to_string(),
                        input_price: 1.0,
                        output_price: 2.0,
                        cache_read_price: None,
                        cache_write_price: None,
                    },
                )
                .unwrap();
        }

        // 重新加载配置
        let manager = PricingManager::new(config_path).unwrap();
        assert_eq!(manager.model_count(), 1);
        assert!(manager.get_pricing("test-model").is_some());
    }

    #[test]
    fn test_persistence_rewrites_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("pricing.toml");

        let mut manager = PricingManager::new(config_path.clone()).unwrap();
        manager.clear_all().unwrap();
        manager
            .set_pricing(
                "first-model".to_string(),
                ModelPricing {
                    model: "first-model".to_string(),
                    input_price: 1.0,
                    output_price: 2.0,
                    cache_read_price: None,
                    cache_write_price: None,
                },
            )
            .unwrap();

        assert!(config_path.exists());

        manager
            .set_pricing(
                "second-model".to_string(),
                ModelPricing {
                    model: "second-model".to_string(),
                    input_price: 3.0,
                    output_price: 4.0,
                    cache_read_price: Some(0.3),
                    cache_write_price: Some(0.4),
                },
            )
            .unwrap();

        let reloaded = PricingManager::new(config_path).unwrap();
        assert!(reloaded.get_pricing("first-model").is_some());
        assert!(reloaded.get_pricing("second-model").is_some());
        assert_eq!(
            reloaded.get_pricing("second-model").unwrap().output_price,
            4.0
        );
    }

    #[test]
    fn test_with_claude_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("pricing.toml");

        let manager = PricingManager::new(config_path).unwrap();

        // 验证 Claude 默认定价
        let sonnet = manager.get_pricing("claude-sonnet-4-5-20250929");
        assert!(sonnet.is_some());
        assert_eq!(sonnet.unwrap().input_price, 3.0);
        assert_eq!(sonnet.unwrap().output_price, 15.0);

        let opus = manager.get_pricing("claude-opus-4-5-20251101");
        assert!(opus.is_some());
        assert_eq!(opus.unwrap().input_price, 5.0);
        assert_eq!(opus.unwrap().output_price, 25.0);
    }
}
