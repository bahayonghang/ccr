// 🌟 Qwen Platform 实现（Stub）
// 📦 阿里通义千问 CLI 平台配置管理
//
// ⚠️ 注意：这是一个占位实现（stub），尚未完全实现
//
// 核心职责:
// - 🚧 返回 NotImplemented 错误
// - 📋 为未来实现预留接口

use crate::models::{Platform, PlatformConfig, ProfileConfig};
use ccr_core::core::error::{CcrError, Result};
use indexmap::IndexMap;
use std::path::PathBuf;

/// 🌟 Qwen Platform 实现（Stub）
///
/// ## 状态
/// 🚧 未实现 - 计划在未来版本中支持
///
/// ## 预期功能
/// - 管理 Qwen CLI 配置 profiles
/// - 支持阿里通义千问 API
/// - 验证 API key 格式
pub struct QwenPlatform;

impl QwenPlatform {
    /// 🏗️ 创建新的 Qwen Platform 实例
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl PlatformConfig for QwenPlatform {
    fn platform_name(&self) -> &str {
        "qwen"
    }

    fn platform_type(&self) -> Platform {
        Platform::Qwen
    }

    fn load_profiles(&self) -> Result<IndexMap<String, ProfileConfig>> {
        Err(CcrError::PlatformNotSupported(
            "Qwen 平台尚未实现".to_string(),
        ))
    }

    fn save_profile(&self, _name: &str, _profile: &ProfileConfig) -> Result<()> {
        Err(CcrError::PlatformNotSupported(
            "Qwen 平台尚未实现".to_string(),
        ))
    }

    fn delete_profile(&self, _name: &str) -> Result<()> {
        Err(CcrError::PlatformNotSupported(
            "Qwen 平台尚未实现".to_string(),
        ))
    }

    fn get_settings_path(&self) -> PathBuf {
        // 返回一个占位路径
        PathBuf::from("~/.ccr/platforms/qwen/settings.json")
    }

    fn apply_profile(&self, _name: &str) -> Result<()> {
        Err(CcrError::PlatformNotSupported(
            "Qwen 平台尚未实现".to_string(),
        ))
    }

    fn validate_profile(&self, _profile: &ProfileConfig) -> Result<()> {
        Err(CcrError::PlatformNotSupported(
            "Qwen 平台尚未实现".to_string(),
        ))
    }

    fn get_current_profile(&self) -> Result<Option<String>> {
        Err(CcrError::PlatformNotSupported(
            "Qwen 平台尚未实现".to_string(),
        ))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_qwen_platform_stub() {
        let platform = QwenPlatform::new().unwrap();

        assert_eq!(platform.platform_name(), "qwen");
        assert_eq!(platform.platform_type(), Platform::Qwen);

        // 所有操作都应该返回 NotImplemented 错误
        assert!(platform.load_profiles().is_err());
        assert!(
            platform
                .save_profile("test", &ProfileConfig::default())
                .is_err()
        );
        assert!(platform.delete_profile("test").is_err());
        assert!(platform.apply_profile("test").is_err());
        assert!(
            platform
                .validate_profile(&ProfileConfig::default())
                .is_err()
        );
        assert!(platform.get_current_profile().is_err());
    }
}
