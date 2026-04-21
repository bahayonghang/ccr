// 🔍 配置模式检测
// Unified 模式配置加载

use crate::managers::{PlatformConfigManager, UnifiedConfig};
use crate::models::{Platform, PlatformConfig, PlatformPaths};
use crate::platforms::create_platform;
use ccr_core::core::error::{CcrError, Result};
use std::str::FromStr;
use std::sync::Arc;

/// 配置模式信息
#[derive(Clone)]
pub struct ConfigMode {
    /// Unified 配置
    unified_config: UnifiedConfig,
}

#[allow(dead_code)]
impl ConfigMode {
    /// 加载配置
    pub fn load() -> Result<Self> {
        let manager = PlatformConfigManager::with_default()?;
        let unified_config = manager.load()?;

        Ok(Self { unified_config })
    }

    /// 检测当前配置模式（兼容旧 API）
    pub fn detect() -> Result<Self> {
        Self::load()
    }

    /// 获取 Unified 配置
    #[expect(dead_code)]
    pub fn unified_config(&self) -> &UnifiedConfig {
        &self.unified_config
    }

    /// 获取当前平台名称
    pub fn current_platform(&self) -> &str {
        &self.unified_config.current_platform
    }

    /// 获取当前平台的 Platform 枚举
    pub fn current_platform_enum(&self) -> Result<Platform> {
        let name = self.current_platform();
        Platform::from_str(name).map_err(|_| CcrError::PlatformNotFound(name.to_string()))
    }

    /// 获取当前平台路径
    #[expect(dead_code)]
    pub fn current_platform_paths(&self) -> Result<PlatformPaths> {
        let platform = self.current_platform_enum()?;
        PlatformPaths::new(platform)
    }

    /// 获取当前平台实现
    #[expect(dead_code)]
    pub fn current_platform_impl(&self) -> Result<Arc<dyn PlatformConfig>> {
        let platform = self.current_platform_enum()?;
        create_platform(platform)
    }
}

/// 快速加载配置
#[allow(dead_code)]
pub fn detect_config_mode() -> Result<UnifiedConfig> {
    let manager = PlatformConfigManager::with_default()?;
    manager.load()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_config_mode_detect() {
        // 测试模式检测不会 panic
        let result = ConfigMode::detect();
        // 如果 ~/.ccr/config.toml 不存在，可能返回 Err
        let _ = result;
    }

    #[test]
    fn test_detect_config_mode_simple() {
        let result = detect_config_mode();
        // 只验证函数能正常执行
        let _ = result;
    }
}
