// 🔌 CCR 平台实现模块
// 📦 包含各平台的具体实现
//
// 核心职责:
// - 🏭 平台工厂 - 根据平台类型创建实例
// - 📋 平台注册表 - 管理所有可用平台
// - 🔍 平台检测 - 检测系统中可用的平台

use crate::core::error::{CcrError, Result};
use crate::models::{Platform, PlatformConfig};
use std::str::FromStr;
use std::sync::Arc;

// 平台实现模块
pub mod claude;
pub mod codex;
pub mod gemini;
pub mod iflow;
pub mod qwen;

// 重新导出平台实现
pub use claude::ClaudePlatform;
pub use codex::CodexPlatform;
pub use gemini::GeminiPlatform;
pub use iflow::IFlowPlatform;
pub use qwen::QwenPlatform;

/// 🏭 平台工厂函数
///
/// 根据平台类型创建对应的平台实例
///
/// # 参数
/// - `platform`: 平台类型
///
/// # 返回
/// - `Ok(Arc<dyn PlatformConfig>)`: 平台实例（线程安全）
/// - Err: 平台未实现或创建失败
///
/// # 示例
/// ```rust,no_run
/// use ccr::{Platform, create_platform};
///
/// let claude = create_platform(Platform::Claude)?;
/// let profiles = claude.load_profiles()?;
/// # Ok::<(), ccr::CcrError>(())
/// ```
pub fn create_platform(platform: Platform) -> Result<Arc<dyn PlatformConfig>> {
    match platform {
        Platform::Claude => {
            let claude = ClaudePlatform::new()?;
            Ok(Arc::new(claude))
        }
        Platform::Codex => {
            let codex = CodexPlatform::new()?;
            Ok(Arc::new(codex))
        }
        Platform::Gemini => {
            let gemini = GeminiPlatform::new()?;
            Ok(Arc::new(gemini))
        }
        Platform::Qwen => {
            let qwen = QwenPlatform::new()?;
            Ok(Arc::new(qwen))
        }
        Platform::IFlow => {
            let iflow = IFlowPlatform::new()?;
            Ok(Arc::new(iflow))
        }
    }
}

/// 📋 平台注册表
///
/// 管理所有可用平台的信息
pub struct PlatformRegistry {
    /// 所有支持的平台列表
    platforms: Vec<Platform>,
}

impl PlatformRegistry {
    /// 创建新的平台注册表
    pub fn new() -> Self {
        Self {
            platforms: Platform::all(),
        }
    }

    /// 获取所有平台
    #[allow(dead_code)]
    pub fn all_platforms(&self) -> &[Platform] {
        &self.platforms
    }

    /// 获取已实现的平台
    #[allow(dead_code)]
    pub fn implemented_platforms(&self) -> Vec<Platform> {
        Platform::implemented()
    }

    /// 检查平台是否已实现
    #[allow(dead_code)]
    pub fn is_implemented(&self, platform: Platform) -> bool {
        platform.is_implemented()
    }

    /// 获取平台显示信息
    pub fn get_platform_info(&self, platform: Platform) -> PlatformInfo {
        PlatformInfo {
            platform,
            name: platform.display_name().to_string(),
            short_name: platform.short_name().to_string(),
            icon: platform.icon().to_string(),
            is_implemented: platform.is_implemented(),
            status: if platform.is_implemented() {
                PlatformStatus::Implemented
            } else {
                PlatformStatus::NotImplemented
            },
        }
    }

    /// 列出所有平台信息
    pub fn list_platform_info(&self) -> Vec<PlatformInfo> {
        self.platforms
            .iter()
            .map(|&p| self.get_platform_info(p))
            .collect()
    }
}

impl Default for PlatformRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 📊 平台状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformStatus {
    /// ✅ 已实现且可用
    Implemented,
    /// 🚧 未实现（计划中）
    NotImplemented,
    /// ⚙️ 已配置（有 profiles）
    #[allow(dead_code)]
    Configured,
    /// 📭 可用但未配置
    #[allow(dead_code)]
    Available,
}

impl PlatformStatus {
    /// 获取状态的显示文本
    #[allow(dead_code)]
    pub fn display_text(&self) -> &str {
        match self {
            PlatformStatus::Implemented => "已实现",
            PlatformStatus::NotImplemented => "未实现",
            PlatformStatus::Configured => "已配置",
            PlatformStatus::Available => "可用",
        }
    }

    /// 获取状态的英文文本
    #[allow(dead_code)]
    pub fn english_text(&self) -> &str {
        match self {
            PlatformStatus::Implemented => "Implemented",
            PlatformStatus::NotImplemented => "Not Implemented",
            PlatformStatus::Configured => "Configured",
            PlatformStatus::Available => "Available",
        }
    }
}

/// 📋 平台信息结构
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    /// 平台类型
    #[allow(dead_code)]
    pub platform: Platform,
    /// 显示名称
    pub name: String,
    /// 短名称（文件系统用）
    pub short_name: String,
    /// 图标
    #[allow(dead_code)]
    pub icon: String,
    /// 是否已实现
    #[allow(dead_code)]
    pub is_implemented: bool,
    /// 平台状态
    #[allow(dead_code)]
    pub status: PlatformStatus,
}

/// 🔍 平台检测器
///
/// 检测系统中可用的平台配置
#[allow(dead_code)]
pub struct PlatformDetector {
    registry: PlatformRegistry,
}

impl PlatformDetector {
    /// 创建新的平台检测器
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            registry: PlatformRegistry::new(),
        }
    }

    /// 检测所有已配置的平台
    ///
    /// 通过检查配置文件是否存在来判断平台是否已配置
    #[allow(dead_code)]
    pub fn detect_configured_platforms(&self) -> Result<Vec<Platform>> {
        use crate::models::PlatformPaths;

        let mut configured = Vec::new();

        for &platform in &Platform::implemented() {
            // 检查平台的 profiles.toml 是否存在
            let paths = PlatformPaths::new(platform)?;
            if paths.profiles_file.exists() {
                configured.push(platform);
            }
        }

        Ok(configured)
    }

    /// 检测当前平台
    ///
    /// 读取 ~/.ccr/config.toml 中的 current_platform 字段
    #[allow(dead_code)]
    pub fn detect_current_platform(&self) -> Result<Option<Platform>> {
        use crate::models::PlatformPaths;
        use std::fs;

        // 获取注册表路径
        let claude_paths = PlatformPaths::new(Platform::Claude)?;
        let registry_path = claude_paths.registry_file;

        if !registry_path.exists() {
            return Ok(None);
        }

        // 读取注册表文件
        let content = fs::read_to_string(&registry_path)
            .map_err(|e| CcrError::ConfigError(format!("读取平台注册表失败: {}", e)))?;

        // 解析 TOML
        let config: toml::Value = toml::from_str(&content)
            .map_err(|e| CcrError::ConfigFormatInvalid(format!("平台注册表格式错误: {}", e)))?;

        // 提取 current_platform
        if let Some(current) = config.get("current_platform").and_then(|v| v.as_str()) {
            Ok(Some(Platform::from_str(current)?))
        } else {
            Ok(None)
        }
    }

    /// 获取平台注册表
    #[allow(dead_code)]
    pub fn registry(&self) -> &PlatformRegistry {
        &self.registry
    }
}

impl Default for PlatformDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_create_platform_claude() {
        // Claude 平台应该能成功创建
        let result = create_platform(Platform::Claude);
        assert!(result.is_ok(), "Claude 平台创建失败");

        if let Ok(platform) = result {
            assert_eq!(platform.platform_name(), "claude");
            assert_eq!(platform.platform_type(), Platform::Claude);
        }
    }

    #[test]
    fn test_create_platform_not_implemented() {
        // Codex 和 Gemini 已经实现，应该成功创建
        assert!(
            create_platform(Platform::Codex).is_ok(),
            "Codex 平台应该成功创建"
        );
        assert!(
            create_platform(Platform::Gemini).is_ok(),
            "Gemini 平台应该成功创建"
        );

        // Qwen 和 IFlow 是 stub 实现，也能成功创建（但方法会返回 PlatformNotSupported 错误）
        assert!(
            create_platform(Platform::Qwen).is_ok(),
            "Qwen 平台应该成功创建（stub）"
        );
        assert!(
            create_platform(Platform::IFlow).is_ok(),
            "IFlow 平台应该成功创建（stub）"
        );
    }

    #[test]
    fn test_platform_registry() {
        let registry = PlatformRegistry::new();
        assert_eq!(registry.all_platforms().len(), 5);

        let implemented = registry.implemented_platforms();
        assert_eq!(implemented.len(), 3);
        assert!(implemented.contains(&Platform::Claude));
        assert!(implemented.contains(&Platform::Codex));
        assert!(implemented.contains(&Platform::Gemini));
    }

    #[test]
    fn test_platform_info() {
        let registry = PlatformRegistry::new();
        let info = registry.get_platform_info(Platform::Claude);

        assert_eq!(info.platform, Platform::Claude);
        assert_eq!(info.name, "Claude Code");
        assert_eq!(info.short_name, "claude");
        assert!(info.is_implemented);
        assert_eq!(info.status, PlatformStatus::Implemented);
    }

    #[test]
    fn test_platform_status_display() {
        assert_eq!(PlatformStatus::Implemented.display_text(), "已实现");
        assert_eq!(PlatformStatus::NotImplemented.display_text(), "未实现");
        assert_eq!(PlatformStatus::Implemented.english_text(), "Implemented");
    }

    #[test]
    fn test_platform_detector() {
        let detector = PlatformDetector::new();
        assert_eq!(detector.registry().all_platforms().len(), 5);
    }
}
