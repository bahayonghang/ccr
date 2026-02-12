// 💰 CCR 价格表配置模型
// 定义模型定价相关的数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::stats::ModelPricing;

/// 💰 价格表配置
///
/// 管理所有模型的定价信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingConfig {
    /// 📌 配置版本
    #[serde(default = "default_version")]
    pub version: String,

    /// 📅 最后更新时间
    pub last_updated: DateTime<Utc>,

    /// 🤖 模型定价表（key: 模型名称）
    #[serde(default)]
    pub models: HashMap<String, ModelPricing>,

    /// 🌍 默认定价（当模型未在表中时使用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_pricing: Option<ModelPricing>,
}

impl Default for PricingConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            last_updated: Utc::now(),
            models: HashMap::new(),
            default_pricing: None,
        }
    }
}

fn default_version() -> String {
    "1.0".to_string()
}

#[allow(dead_code)]
impl PricingConfig {
    /// 创建新的价格表配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加或更新模型定价
    pub fn set_pricing(&mut self, model: String, pricing: ModelPricing) {
        self.models.insert(model, pricing);
        self.last_updated = Utc::now();
    }

    /// 获取模型定价
    pub fn get_pricing(&self, model: &str) -> Option<&ModelPricing> {
        self.models.get(model)
    }

    /// 移除模型定价
    pub fn remove_pricing(&mut self, model: &str) -> Option<ModelPricing> {
        let result = self.models.remove(model);
        if result.is_some() {
            self.last_updated = Utc::now();
        }
        result
    }

    /// 获取或创建默认定价
    pub fn get_or_default_pricing(&self, model: &str) -> Option<&ModelPricing> {
        self.models.get(model).or(self.default_pricing.as_ref())
    }

    /// 设置默认定价
    #[allow(dead_code)]
    pub fn set_default_pricing(&mut self, pricing: ModelPricing) {
        self.default_pricing = Some(pricing);
        self.last_updated = Utc::now();
    }

    /// 清除默认定价
    pub fn clear_default_pricing(&mut self) {
        self.default_pricing = None;
        self.last_updated = Utc::now();
    }

    /// 清空所有定价
    pub fn clear_all(&mut self) {
        self.models.clear();
        self.default_pricing = None;
        self.last_updated = Utc::now();
    }

    /// 检查配置是否为空
    pub fn is_empty(&self) -> bool {
        self.models.is_empty() && self.default_pricing.is_none()
    }

    /// 获取所有模型名称列表
    pub fn model_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.models.keys().cloned().collect();
        names.sort();
        names
    }

    /// 统计已配置的模型数量
    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    /// 验证配置是否有效
    pub fn validate(&self) -> Result<(), String> {
        // 检查版本格式
        if self.version.is_empty() {
            return Err("版本号不能为空".to_string());
        }

        // 验证每个模型的定价
        for (model_name, pricing) in &self.models {
            if model_name.is_empty() {
                return Err("模型名称不能为空".to_string());
            }

            if pricing.input_price < 0.0 {
                return Err(format!("模型 {} 的输入价格不能为负数", model_name));
            }

            if pricing.output_price < 0.0 {
                return Err(format!("模型 {} 的输出价格不能为负数", model_name));
            }

            if let Some(cache_read) = pricing.cache_read_price
                && cache_read < 0.0
            {
                return Err(format!("模型 {} 的 Cache 读取价格不能为负数", model_name));
            }

            if let Some(cache_write) = pricing.cache_write_price
                && cache_write < 0.0
            {
                return Err(format!("模型 {} 的 Cache 写入价格不能为负数", model_name));
            }
        }

        // 验证默认定价（如果存在）
        if let Some(default) = &self.default_pricing {
            if default.input_price < 0.0 {
                return Err("默认输入价格不能为负数".to_string());
            }

            if default.output_price < 0.0 {
                return Err("默认输出价格不能为负数".to_string());
            }

            if let Some(cache_read) = default.cache_read_price
                && cache_read < 0.0
            {
                return Err("默认 Cache 读取价格不能为负数".to_string());
            }

            if let Some(cache_write) = default.cache_write_price
                && cache_write < 0.0
            {
                return Err("默认 Cache 写入价格不能为负数".to_string());
            }
        }

        Ok(())
    }

    /// 加载内置的 Claude 模型定价
    pub fn with_claude_defaults() -> Self {
        let mut config = Self::new();

        // Claude Opus 4.5
        config.set_pricing(
            "claude-opus-4-5-20251101".to_string(),
            ModelPricing {
                model: "claude-opus-4-5-20251101".to_string(),
                input_price: 15.0,
                output_price: 75.0,
                cache_read_price: Some(1.5),
                cache_write_price: Some(18.75),
            },
        );

        // Claude Sonnet 4.5
        config.set_pricing(
            "claude-sonnet-4-5-20250929".to_string(),
            ModelPricing {
                model: "claude-sonnet-4-5-20250929".to_string(),
                input_price: 3.0,
                output_price: 15.0,
                cache_read_price: Some(0.3),
                cache_write_price: Some(3.75),
            },
        );

        // Claude 3.5 Sonnet (旧版)
        config.set_pricing(
            "claude-3-5-sonnet-20241022".to_string(),
            ModelPricing {
                model: "claude-3-5-sonnet-20241022".to_string(),
                input_price: 3.0,
                output_price: 15.0,
                cache_read_price: Some(0.3),
                cache_write_price: Some(3.75),
            },
        );

        // Claude 3.5 Haiku
        config.set_pricing(
            "claude-3-5-haiku-20241022".to_string(),
            ModelPricing {
                model: "claude-3-5-haiku-20241022".to_string(),
                input_price: 0.8,
                output_price: 4.0,
                cache_read_price: Some(0.08),
                cache_write_price: Some(1.0),
            },
        );

        config
    }

    /// 合并另一个价格表配置
    pub fn merge(&mut self, other: &PricingConfig) {
        for (model, pricing) in &other.models {
            self.models.insert(model.clone(), pricing.clone());
        }

        if other.default_pricing.is_some() {
            self.default_pricing = other.default_pricing.clone();
        }

        self.last_updated = Utc::now();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_pricing_config_default() {
        let config = PricingConfig::default();
        assert_eq!(config.version, "1.0");
        assert!(config.is_empty());
        assert_eq!(config.model_count(), 0);
    }

    #[test]
    fn test_set_and_get_pricing() {
        let mut config = PricingConfig::new();

        let pricing = ModelPricing {
            model: "test-model".to_string(),
            input_price: 1.0,
            output_price: 2.0,
            cache_read_price: Some(0.1),
            cache_write_price: Some(0.2),
        };

        config.set_pricing("test-model".to_string(), pricing.clone());

        assert_eq!(config.model_count(), 1);
        assert!(!config.is_empty());

        let retrieved = config.get_pricing("test-model");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().input_price, 1.0);
    }

    #[test]
    fn test_remove_pricing() {
        let mut config = PricingConfig::new();

        let pricing = ModelPricing {
            model: "test-model".to_string(),
            input_price: 1.0,
            output_price: 2.0,
            cache_read_price: None,
            cache_write_price: None,
        };

        config.set_pricing("test-model".to_string(), pricing);
        assert_eq!(config.model_count(), 1);

        let removed = config.remove_pricing("test-model");
        assert!(removed.is_some());
        assert_eq!(config.model_count(), 0);
    }

    #[test]
    fn test_default_pricing() {
        let mut config = PricingConfig::new();

        let default_pricing = ModelPricing {
            model: "default".to_string(),
            input_price: 5.0,
            output_price: 10.0,
            cache_read_price: None,
            cache_write_price: None,
        };

        config.set_default_pricing(default_pricing);

        // 对于不存在的模型，应返回默认定价
        let result = config.get_or_default_pricing("non-existent-model");
        assert!(result.is_some());
        assert_eq!(result.unwrap().input_price, 5.0);

        config.clear_default_pricing();
        assert!(config.default_pricing.is_none());
    }

    #[test]
    fn test_validation() {
        let mut config = PricingConfig::new();
        assert!(config.validate().is_ok());

        // 添加有效定价
        config.set_pricing(
            "valid-model".to_string(),
            ModelPricing {
                model: "valid-model".to_string(),
                input_price: 1.0,
                output_price: 2.0,
                cache_read_price: Some(0.1),
                cache_write_price: Some(0.2),
            },
        );
        assert!(config.validate().is_ok());

        // 添加无效定价（负价格）
        config.set_pricing(
            "invalid-model".to_string(),
            ModelPricing {
                model: "invalid-model".to_string(),
                input_price: -1.0,
                output_price: 2.0,
                cache_read_price: None,
                cache_write_price: None,
            },
        );
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_claude_defaults() {
        let config = PricingConfig::with_claude_defaults();

        assert!(!config.is_empty());
        assert!(config.model_count() >= 4);

        // 验证 Opus 4.5 定价
        let opus = config.get_pricing("claude-opus-4-5-20251101");
        assert!(opus.is_some());
        assert_eq!(opus.unwrap().input_price, 15.0);
        assert_eq!(opus.unwrap().output_price, 75.0);

        // 验证 Sonnet 4.5 定价
        let sonnet = config.get_pricing("claude-sonnet-4-5-20250929");
        assert!(sonnet.is_some());
        assert_eq!(sonnet.unwrap().input_price, 3.0);
        assert_eq!(sonnet.unwrap().output_price, 15.0);
    }

    #[test]
    fn test_model_names() {
        let mut config = PricingConfig::new();

        config.set_pricing(
            "model-b".to_string(),
            ModelPricing {
                model: "model-b".to_string(),
                input_price: 1.0,
                output_price: 2.0,
                cache_read_price: None,
                cache_write_price: None,
            },
        );

        config.set_pricing(
            "model-a".to_string(),
            ModelPricing {
                model: "model-a".to_string(),
                input_price: 1.0,
                output_price: 2.0,
                cache_read_price: None,
                cache_write_price: None,
            },
        );

        let names = config.model_names();
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "model-a"); // 应该已排序
        assert_eq!(names[1], "model-b");
    }

    #[test]
    fn test_merge_configs() {
        let mut config1 = PricingConfig::new();
        config1.set_pricing(
            "model-1".to_string(),
            ModelPricing {
                model: "model-1".to_string(),
                input_price: 1.0,
                output_price: 2.0,
                cache_read_price: None,
                cache_write_price: None,
            },
        );

        let mut config2 = PricingConfig::new();
        config2.set_pricing(
            "model-2".to_string(),
            ModelPricing {
                model: "model-2".to_string(),
                input_price: 3.0,
                output_price: 4.0,
                cache_read_price: None,
                cache_write_price: None,
            },
        );

        config1.merge(&config2);

        assert_eq!(config1.model_count(), 2);
        assert!(config1.get_pricing("model-1").is_some());
        assert!(config1.get_pricing("model-2").is_some());
    }

    #[test]
    fn test_clear_all() {
        let mut config = PricingConfig::with_claude_defaults();
        assert!(!config.is_empty());

        config.clear_all();
        assert!(config.is_empty());
        assert_eq!(config.model_count(), 0);
    }
}
