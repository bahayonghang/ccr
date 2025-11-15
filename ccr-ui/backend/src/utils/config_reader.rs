// Config File Reader
// Supports both Legacy mode (~/.ccs_config.toml) and Unified mode (~/.ccr/)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 配置模式枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigMode {
    /// Legacy 模式 - 单一配置文件 ~/.ccs_config.toml
    Legacy,
    /// Unified 模式 - 多平台配置 ~/.ccr/
    Unified,
}

/// 平台配置文件结构（Unified 模式）
#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedConfig {
    pub default_platform: String,
    pub current_platform: String,
    #[serde(flatten)]
    pub platforms: HashMap<String, PlatformEntry>,
}

/// 平台注册表条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformEntry {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<String>,
}

/// 平台 profiles 配置结构（Unified 模式）
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)] // 预留用于多平台配置管理
pub struct PlatformProfiles {
    pub default_config: String,
    pub current_config: String,
    #[serde(flatten)]
    pub sections: HashMap<String, ConfigSection>,
}

/// Legacy 配置文件结构
///
/// 注意：当前 backend 使用 CCR 库的服务层，此结构保留作为参考
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct CcsConfig {
    pub default_config: String,
    pub current_config: String,
    #[serde(flatten)]
    pub sections: HashMap<String, ConfigSection>,
}

/// 配置节结构（通用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_fast_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// 检测配置模式
///
/// 优先级：
/// 1. 检查 CCR_ROOT 环境变量
/// 2. 检查 ~/.ccr/config.toml 是否存在
/// 3. 回退到 Legacy 模式
pub fn detect_config_mode() -> ConfigMode {
    // 1. 检查环境变量
    if std::env::var("CCR_ROOT").is_ok() {
        return ConfigMode::Unified;
    }

    // 2. 检查默认 Unified 配置路径
    if let Ok(home) = dirs::home_dir().ok_or("") {
        let unified_config = home.join(".ccr").join("config.toml");
        if unified_config.exists() {
            return ConfigMode::Unified;
        }
    }

    // 3. 默认为 Legacy 模式
    ConfigMode::Legacy
}

/// 获取 CCR 根目录（Unified 模式）
pub fn get_ccr_root() -> Result<PathBuf, String> {
    if let Ok(custom_root) = std::env::var("CCR_ROOT") {
        Ok(PathBuf::from(custom_root))
    } else {
        let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
        Ok(home.join(".ccr"))
    }
}

/// 读取当前平台（Unified 模式）
///
/// 注意：当前 backend 使用 CCR 库的服务层，此函数保留作为参考
#[allow(dead_code)]
pub fn get_current_platform() -> Result<String, String> {
    let ccr_root = get_ccr_root()?;
    let config_path = ccr_root.join("config.toml");

    if !config_path.exists() {
        return Ok("claude".to_string()); // 默认平台
    }

    let content =
        std::fs::read_to_string(&config_path).map_err(|e| format!("读取平台配置失败: {}", e))?;

    let config: UnifiedConfig =
        toml::from_str(&content).map_err(|e| format!("解析平台配置失败: {}", e))?;

    Ok(config.current_platform)
}

/// 获取平台的 profiles 文件路径（Unified 模式）
pub fn get_platform_profiles_path(platform: &str) -> Result<PathBuf, String> {
    let ccr_root = get_ccr_root()?;
    Ok(ccr_root
        .join("platforms")
        .join(platform)
        .join("profiles.toml"))
}

/// 获取配置文件路径（兼容两种模式）
///
/// 注意：当前 backend 使用 CCR 库的服务层，此函数保留作为参考
#[allow(dead_code)]
pub fn get_config_path() -> Result<PathBuf, String> {
    match detect_config_mode() {
        ConfigMode::Legacy => {
            let home = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
            Ok(home.join(".ccs_config.toml"))
        }
        ConfigMode::Unified => {
            // 获取当前平台的 profiles 路径
            let platform = get_current_platform()?;
            get_platform_profiles_path(&platform)
        }
    }
}

/// 读取配置文件（兼容两种模式）
///
/// 注意：当前 backend 使用 CCR 库的服务层，此函数保留作为参考
#[allow(dead_code)]
pub fn read_config() -> Result<CcsConfig, String> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Err(format!(
            "配置文件不存在: {}. 运行 'ccr init' 或 'ccr platform init <platform>' 创建配置。",
            config_path.display()
        ));
    }

    let content =
        std::fs::read_to_string(&config_path).map_err(|e| format!("读取配置文件失败: {}", e))?;

    let config: CcsConfig =
        toml::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    Ok(config)
}

/// 读取指定平台的配置（仅 Unified 模式）
#[allow(dead_code)] // 预留用于平台特定配置读取
pub fn read_platform_config(platform: &str) -> Result<PlatformProfiles, String> {
    let profiles_path = get_platform_profiles_path(platform)?;

    if !profiles_path.exists() {
        return Err(format!(
            "平台 '{}' 的配置文件不存在: {}",
            platform,
            profiles_path.display()
        ));
    }

    let content =
        std::fs::read_to_string(&profiles_path).map_err(|e| format!("读取平台配置失败: {}", e))?;

    let config: PlatformProfiles =
        toml::from_str(&content).map_err(|e| format!("解析平台配置失败: {}", e))?;

    Ok(config)
}

/// 读取平台注册表（仅 Unified 模式）
#[allow(dead_code)] // 预留用于统一配置读取
pub fn read_unified_config() -> Result<UnifiedConfig, String> {
    let ccr_root = get_ccr_root()?;
    let config_path = ccr_root.join("config.toml");

    if !config_path.exists() {
        return Err(format!("平台注册表不存在: {}", config_path.display()));
    }

    let content =
        std::fs::read_to_string(&config_path).map_err(|e| format!("读取平台注册表失败: {}", e))?;

    let config: UnifiedConfig =
        toml::from_str(&content).map_err(|e| format!("解析平台注册表失败: {}", e))?;

    Ok(config)
}

/// 脱敏 token 用于显示
///
/// 注意：当前 backend 使用 CCR 库的工具函数，此函数保留作为参考
#[allow(dead_code)]
pub fn mask_token(token: &str) -> String {
    if token.len() <= 10 {
        "*".repeat(token.len())
    } else {
        let prefix = &token[..4];
        let suffix = &token[token.len() - 4..];
        format!("{}...{}", prefix, suffix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_config_mode() {
        let mode = detect_config_mode();
        assert!(matches!(mode, ConfigMode::Legacy | ConfigMode::Unified));
    }

    #[test]
    fn test_mask_token() {
        assert_eq!(mask_token("short"), "*****");
        assert_eq!(mask_token("sk-ant-api03-1234567890abcdef"), "sk-a...cdef");
    }
}
