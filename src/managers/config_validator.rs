// ✅ 配置验证器
// 负责配置数据的验证逻辑
//
// 🎯 设计目的:
// - 从 ConfigManager 中分离验证逻辑
// - 提供统一的验证接口
// - 生成详细的验证报告

use crate::core::error::Result;
use crate::managers::config::{CcsConfig, ConfigSection};
use crate::utils::Validatable;
use std::collections::HashMap;

/// 📊 验证报告
///
/// 包含配置验证的详细结果
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// 总配置节数量
    pub total_sections: usize,

    /// 有效配置节数量
    pub valid_count: usize,

    /// 无效配置节数量
    pub invalid_count: usize,

    /// 无效配置节的详细信息 (配置名 -> 错误消息)
    pub invalid_sections: HashMap<String, String>,

    /// 警告信息
    #[allow(dead_code)]
    pub warnings: Vec<String>,
}

impl ValidationReport {
    /// 🆕 创建空的验证报告
    pub fn new() -> Self {
        Self {
            total_sections: 0,
            valid_count: 0,
            invalid_count: 0,
            invalid_sections: HashMap::new(),
            warnings: Vec::new(),
        }
    }

    /// ✅ 检查是否所有配置都有效
    #[allow(dead_code)]
    pub fn is_all_valid(&self) -> bool {
        self.invalid_count == 0
    }

    /// ⚠️ 检查是否有警告
    #[allow(dead_code)]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// 📊 获取成功率百分比
    pub fn success_rate(&self) -> f64 {
        if self.total_sections == 0 {
            return 100.0;
        }
        (self.valid_count as f64 / self.total_sections as f64) * 100.0
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// ✅ 配置验证器
///
/// 封装所有配置验证相关的逻辑
///
/// # 职责
/// - 🔍 验证单个配置节
/// - 📋 批量验证所有配置节
/// - ✅ 验证配置文件的一致性
/// - 📊 生成详细的验证报告
pub struct ConfigValidator;

impl ConfigValidator {
    /// 🆕 创建新的配置验证器
    pub fn new() -> Self {
        Self
    }

    /// ✅ 验证单个配置节
    ///
    /// 返回验证结果，如果验证失败返回错误消息
    pub fn validate_section(&self, section: &ConfigSection) -> Result<()> {
        section.validate()
    }

    /// 📋 批量验证所有配置节
    ///
    /// 验证配置中的所有配置节，返回详细的验证报告
    pub fn validate_all_sections(&self, config: &CcsConfig) -> ValidationReport {
        let mut report = ValidationReport::new();
        report.total_sections = config.sections.len();

        for (name, section) in &config.sections {
            match self.validate_section(section) {
                Ok(_) => {
                    report.valid_count += 1;
                    tracing::debug!("✅ 配置节 '{}' 验证通过", name);
                }
                Err(e) => {
                    report.invalid_count += 1;
                    let error_msg = e.to_string();
                    report
                        .invalid_sections
                        .insert(name.clone(), error_msg.clone());
                    tracing::warn!("❌ 配置节 '{}' 验证失败: {}", name, error_msg);
                }
            }
        }

        tracing::info!(
            "📊 验证完成: 总数={}, 有效={}, 无效={}, 成功率={:.1}%",
            report.total_sections,
            report.valid_count,
            report.invalid_count,
            report.success_rate()
        );

        report
    }

    /// ✅ 验证配置文件的一致性
    ///
    /// 检查配置文件的基本一致性：
    /// - 当前配置是否存在
    /// - 默认配置是否存在
    /// - 是否有空的配置节列表
    #[allow(dead_code)]
    pub fn validate_consistency(&self, config: &CcsConfig) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // 检查配置节是否为空
        if config.sections.is_empty() {
            warnings.push("配置文件中没有任何配置节".to_string());
        }

        // 检查当前配置是否存在
        if !config.sections.contains_key(&config.current_config) {
            warnings.push(format!(
                "当前配置 '{}' 不存在于配置节列表中",
                config.current_config
            ));
        }

        // 检查默认配置是否存在
        if !config.sections.contains_key(&config.default_config) {
            warnings.push(format!(
                "默认配置 '{}' 不存在于配置节列表中",
                config.default_config
            ));
        }

        // 检查当前配置和默认配置是否相同
        if config.current_config != config.default_config {
            tracing::debug!(
                "当前配置 '{}' 与默认配置 '{}' 不同",
                config.current_config,
                config.default_config
            );
        }

        Ok(warnings)
    }

    /// 🔍 完整验证
    ///
    /// 执行完整的验证流程：
    /// 1. 验证所有配置节
    /// 2. 验证配置一致性
    /// 3. 生成综合验证报告
    #[allow(dead_code)]
    pub fn validate_complete(&self, config: &CcsConfig) -> ValidationReport {
        let mut report = self.validate_all_sections(config);

        // 添加一致性检查警告
        match self.validate_consistency(config) {
            Ok(consistency_warnings) => {
                report.warnings.extend(consistency_warnings);
            }
            Err(e) => {
                report.warnings.push(format!("一致性检查失败: {}", e));
            }
        }

        report
    }

    /// 📊 生成人类可读的验证报告摘要
    ///
    /// 将验证报告格式化为易于阅读的字符串
    #[allow(dead_code)]
    pub fn format_report(&self, report: &ValidationReport) -> String {
        let mut output = String::new();

        output.push_str("📊 验证报告\n");
        output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        output.push_str(&format!("总配置节数: {}\n", report.total_sections));
        output.push_str(&format!("✅ 有效: {}\n", report.valid_count));
        output.push_str(&format!("❌ 无效: {}\n", report.invalid_count));
        output.push_str(&format!("📈 成功率: {:.1}%\n", report.success_rate()));

        if !report.invalid_sections.is_empty() {
            output.push_str("\n❌ 无效配置节:\n");
            for (name, error) in &report.invalid_sections {
                output.push_str(&format!("  - {}: {}\n", name, error));
            }
        }

        if !report.warnings.is_empty() {
            output.push_str("\n⚠️  警告:\n");
            for warning in &report.warnings {
                output.push_str(&format!("  - {}\n", warning));
            }
        }

        if report.is_all_valid() && !report.has_warnings() {
            output.push_str("\n✅ 所有配置均有效，没有警告！\n");
        }

        output
    }
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::managers::config::GlobalSettings;
    use indexmap::IndexMap;

    fn create_valid_section() -> ConfigSection {
        ConfigSection {
            description: Some("Test config".into()),
            base_url: Some("https://api.test.com".into()),
            auth_token: Some("sk-test-token".into()),
            model: Some("test-model".into()),
            small_fast_model: Some("test-small-model".into()),
            provider: None,
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            other: IndexMap::new(),
        }
    }

    fn create_invalid_section() -> ConfigSection {
        ConfigSection {
            description: Some("Invalid config".into()),
            base_url: Some("".into()), // 无效的空 base_url
            auth_token: Some("sk-test-token".into()),
            model: Some("test-model".into()),
            small_fast_model: None,
            provider: None,
            provider_type: None,
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            other: IndexMap::new(),
        }
    }

    #[test]
    fn test_validator_validate_section() {
        let validator = ConfigValidator::new();

        // 测试有效配置节
        let valid_section = create_valid_section();
        assert!(validator.validate_section(&valid_section).is_ok());

        // 测试无效配置节
        let invalid_section = create_invalid_section();
        assert!(validator.validate_section(&invalid_section).is_err());
    }

    #[test]
    fn test_validator_validate_all_sections() {
        let validator = ConfigValidator::new();

        let mut config = CcsConfig {
            default_config: "valid".into(),
            current_config: "valid".into(),
            settings: GlobalSettings::default(),
            sections: IndexMap::new(),
        };

        // 添加有效和无效配置节
        config.set_section("valid".into(), create_valid_section());
        config.set_section("invalid".into(), create_invalid_section());

        let report = validator.validate_all_sections(&config);

        assert_eq!(report.total_sections, 2);
        assert_eq!(report.valid_count, 1);
        assert_eq!(report.invalid_count, 1);
        assert!(report.invalid_sections.contains_key("invalid"));
        assert_eq!(report.success_rate(), 50.0);
    }

    #[test]
    fn test_validator_validate_consistency() {
        let validator = ConfigValidator::new();

        // 测试空配置
        let empty_config = CcsConfig {
            default_config: "default".into(),
            current_config: "current".into(),
            settings: GlobalSettings::default(),
            sections: IndexMap::new(),
        };

        let warnings = validator.validate_consistency(&empty_config).unwrap();
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| w.contains("没有任何配置节")));

        // 测试不存在的当前配置
        let mut config_with_missing_current = empty_config.clone();
        config_with_missing_current.set_section("default".into(), create_valid_section());

        let warnings = validator
            .validate_consistency(&config_with_missing_current)
            .unwrap();
        assert!(
            warnings
                .iter()
                .any(|w| w.contains("当前配置") && w.contains("不存在"))
        );
    }

    #[test]
    fn test_validator_complete() {
        let validator = ConfigValidator::new();

        let mut config = CcsConfig {
            default_config: "valid".into(),
            current_config: "valid".into(),
            settings: GlobalSettings::default(),
            sections: IndexMap::new(),
        };

        config.set_section("valid".into(), create_valid_section());
        config.set_section("invalid".into(), create_invalid_section());

        let report = validator.validate_complete(&config);

        assert_eq!(report.total_sections, 2);
        assert_eq!(report.valid_count, 1);
        assert_eq!(report.invalid_count, 1);
        // 应该没有一致性警告（因为 current_config 和 default_config 都存在）
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        assert_eq!(report.total_sections, 0);
        assert!(report.is_all_valid());
        assert!(!report.has_warnings());
        assert_eq!(report.success_rate(), 100.0);

        report.total_sections = 10;
        report.valid_count = 8;
        report.invalid_count = 2;
        report
            .invalid_sections
            .insert("bad1".into(), "error1".into());
        report
            .invalid_sections
            .insert("bad2".into(), "error2".into());
        report.warnings.push("warning1".into());

        assert!(!report.is_all_valid());
        assert!(report.has_warnings());
        assert_eq!(report.success_rate(), 80.0);
    }

    #[test]
    fn test_format_report() {
        let validator = ConfigValidator::new();

        let mut report = ValidationReport::new();
        report.total_sections = 5;
        report.valid_count = 4;
        report.invalid_count = 1;
        report
            .invalid_sections
            .insert("bad".into(), "invalid base_url".into());
        report.warnings.push("当前配置不存在".into());

        let formatted = validator.format_report(&report);

        assert!(formatted.contains("总配置节数: 5"));
        assert!(formatted.contains("有效: 4"));
        assert!(formatted.contains("无效: 1"));
        assert!(formatted.contains("成功率: 80.0%"));
        assert!(formatted.contains("bad: invalid base_url"));
        assert!(formatted.contains("当前配置不存在"));
    }
}
