// 🔍 验证服务
// 封装配置和设置的验证业务逻辑
//
// 🎯 设计目的:
// - 从 Web handler 层分离验证逻辑（修复层级违规）
// - 提供统一的验证接口
// - 复用 ConfigService 和 SettingsService 的验证能力

use crate::managers::SettingsManager;
use crate::services::ConfigService;
use crate::services::config_service::ValidationReport;
use ccr_core::Validatable;
use ccr_core::core::error::Result;
use std::sync::Arc;

/// 🔍 完整验证报告
///
/// 包含配置文件和设置文件的验证结果
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FullValidationReport {
    /// 配置文件验证报告
    pub config_report: ValidationReport,
    /// 配置文件路径
    pub config_path: String,
    /// 当前配置名称
    pub current_config: String,
    /// 当前配置是否存在
    pub current_config_exists: bool,
    /// 默认配置名称
    pub default_config: String,
    /// 默认配置是否存在
    pub default_config_exists: bool,
    /// Settings 文件验证结果
    pub settings_valid: bool,
    /// Settings 文件路径
    pub settings_path: Option<String>,
    /// Settings 错误消息（如果有）
    pub settings_error: Option<String>,
    /// 是否有错误
    pub has_errors: bool,
    /// 是否有警告
    pub has_warnings: bool,
}

/// 🔍 验证服务
///
/// 封装所有验证相关的业务逻辑
///
/// # 架构说明
///
/// 此服务遵循分层架构原则:
/// - Web handlers → ValidateService → ConfigService/SettingsService → Managers
/// - 不直接访问 commands 层
pub struct ValidateService {
    config_service: Arc<ConfigService>,
    #[allow(dead_code)]
    settings_manager: Arc<SettingsManager>,
}

#[allow(dead_code)]
impl ValidateService {
    /// 🏗️ 创建新的验证服务
    pub fn new(config_service: Arc<ConfigService>, settings_manager: Arc<SettingsManager>) -> Self {
        Self {
            config_service,
            settings_manager,
        }
    }

    /// 🏠 使用默认配置创建服务
    pub fn with_default() -> Result<Self> {
        let config_service = Arc::new(ConfigService::with_default()?);
        let settings_manager = Arc::new(SettingsManager::with_default()?);
        Ok(Self::new(config_service, settings_manager))
    }

    /// ✅ 验证所有配置节
    ///
    /// 委托给 ConfigService 执行验证
    #[allow(dead_code)]
    pub fn validate_configs(&self) -> Result<ValidationReport> {
        self.config_service.validate_all()
    }

    /// ✅ 验证 Settings 文件
    ///
    /// 检查 settings.json 是否存在且格式正确
    pub fn validate_settings(&self) -> (bool, Option<String>, Option<String>) {
        let settings_path = self.settings_manager.settings_path();

        if !settings_path.exists() {
            return (
                false,
                Some(settings_path.display().to_string()),
                Some("Settings 文件不存在".to_string()),
            );
        }

        match self.settings_manager.load() {
            Ok(settings) => {
                // 验证 settings 内容
                match settings.validate() {
                    Ok(_) => (true, Some(settings_path.display().to_string()), None),
                    Err(e) => (
                        false,
                        Some(settings_path.display().to_string()),
                        Some(e.to_string()),
                    ),
                }
            }
            Err(e) => (
                false,
                Some(settings_path.display().to_string()),
                Some(format!("加载 Settings 失败: {}", e)),
            ),
        }
    }

    /// ✅ 执行完整验证
    ///
    /// 验证配置文件和设置文件
    pub fn validate_all(&self) -> Result<FullValidationReport> {
        let mut has_errors = false;
        let mut has_warnings = false;

        // 1. 验证配置节
        let config_report = self.config_service.validate_all()?;
        if config_report.invalid_count > 0 {
            has_errors = true;
        }

        // 2. 获取配置信息
        let config = self.config_service.load_config()?;
        let config_path = self
            .config_service
            .config_manager()
            .config_path()
            .display()
            .to_string();

        // 3. 检查当前配置和默认配置
        let current_config_exists = config.sections.contains_key(&config.current_config);
        let default_config_exists = config.sections.contains_key(&config.default_config);

        if !current_config_exists {
            has_errors = true;
        }
        if !default_config_exists {
            has_warnings = true;
        }

        // 4. 验证 Settings
        let (settings_valid, settings_path, settings_error) = self.validate_settings();
        if !settings_valid {
            has_warnings = true;
        }

        Ok(FullValidationReport {
            config_report,
            config_path,
            current_config: config.current_config.clone(),
            current_config_exists,
            default_config: config.default_config.clone(),
            default_config_exists,
            settings_valid,
            settings_path,
            settings_error,
            has_errors,
            has_warnings,
        })
    }

    /// 🔍 快速验证（仅验证配置节）
    ///
    /// 用于需要快速响应的场景
    pub fn quick_validate(&self) -> Result<ValidationReport> {
        self.config_service.validate_all()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_service_creation() {
        // 这个测试需要配置文件存在，在实际环境中运行
        // 这里只验证类型定义正确
        let _ = ValidateService::with_default();
    }
}
