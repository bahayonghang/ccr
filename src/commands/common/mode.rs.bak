// 🔍 配置模式检测
// 统一处理 Unified/Legacy 模式检测逻辑

use crate::core::error::{CcrError, Result};
use crate::managers::{PlatformConfigManager, UnifiedConfig};
use crate::models::{Platform, PlatformConfig, PlatformPaths};
use crate::platforms::create_platform;
use std::str::FromStr;
use std::sync::Arc;

/// 配置模式信息
#[derive(Clone)]
pub struct ConfigMode {
    /// 是否为 Unified 模式
    pub is_unified: bool,
    /// Unified 配置（如果在 Unified 模式下）
    unified_config: Option<UnifiedConfig>,
}

impl ConfigMode {
    /// 检测当前配置模式
    pub fn detect() -> Result<Self> {
        let manager = PlatformConfigManager::with_default().ok();
        let unified_config = manager.as_ref().and_then(|m| m.load().ok());
        let is_unified = unified_config.is_some();

        Ok(Self {
            is_unified,
            unified_config,
        })
    }

    /// 获取 Unified 配置，如果不在 Unified 模式则返回错误
    pub fn require_unified(&self) -> Result<&UnifiedConfig> {
        self.unified_config
            .as_ref()
            .ok_or_else(|| CcrError::ConfigError("当前不在 Unified 模式".to_string()))
    }

    /// 获取 Unified 配置（可选）
    pub fn unified_config(&self) -> Option<&UnifiedConfig> {
        self.unified_config.as_ref()
    }

    /// 获取当前平台名称
    pub fn current_platform(&self) -> Option<&str> {
        self.unified_config
            .as_ref()
            .map(|c| c.current_platform.as_str())
    }

    /// 获取当前平台的 Platform 枚举
    pub fn current_platform_enum(&self) -> Result<Platform> {
        let name = self
            .current_platform()
            .ok_or_else(|| CcrError::ConfigError("当前不在 Unified 模式".to_string()))?;
        Platform::from_str(name).map_err(|_| CcrError::PlatformNotFound(name.to_string()))
    }

    /// 获取当前平台路径
    pub fn current_platform_paths(&self) -> Result<PlatformPaths> {
        let platform = self.current_platform_enum()?;
        PlatformPaths::new(platform)
    }

    /// 获取当前平台实现
    pub fn current_platform_impl(&self) -> Result<Arc<dyn PlatformConfig>> {
        let platform = self.current_platform_enum()?;
        create_platform(platform)
    }

    /// 获取模式显示名称
    pub fn mode_display_name(&self) -> &'static str {
        if self.is_unified { "Unified" } else { "Legacy" }
    }
}

/// 快速检测配置模式（返回简单结果）
pub fn detect_config_mode() -> (bool, Option<UnifiedConfig>) {
    let unified_config = PlatformConfigManager::with_default()
        .ok()
        .and_then(|mgr| mgr.load().ok());
    let is_unified = unified_config.is_some();
    (is_unified, unified_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_mode_detect() {
        // 测试模式检测不会 panic
        let result = ConfigMode::detect();
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_config_mode_simple() {
        let (is_unified, _) = detect_config_mode();
        // 只验证函数能正常执行
        let _ = is_unified;
    }
}
