// 🎯 CCR 平台管理模块
// 📦 定义跨平台配置管理的核心类型和接口
//
// 核心职责:
// - 🏷️ Platform 枚举 - 支持的平台类型
// - 🔌 PlatformConfig trait - 平台实现接口
// - 📋 ProfileConfig - 通用配置结构
// - 📁 PlatformPaths - 平台路径管理
// - 📁 PlatformPaths - 平台路径管理

use crate::core::error::Result;
use crate::utils::AutoCompletable;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

/// 🏷️ 平台类型枚举
///
/// 表示 CCR 支持的 AI CLI 平台
///
/// ## 支持状态
/// - ✅ **Claude**: 完全支持（Claude Code）
/// - ✅ **Codex**: 完全支持（GitHub Copilot CLI）
/// - ✅ **Gemini**: 完全支持（Gemini CLI）
/// - 🚧 **Qwen**: 计划支持（阿里通义千问 CLI）
/// - 🚧 **IFlow**: 计划支持（iFlow CLI）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// Claude Code - Anthropic 官方 CLI
    Claude,
    /// Codex - GitHub Copilot CLI
    Codex,
    /// Gemini CLI - Google Gemini CLI
    Gemini,
    /// Qwen CLI - 阿里通义千问 CLI (未实现)
    Qwen,
    /// iFlow CLI - iFlow CLI (未实现)
    IFlow,
    /// Factory Droid CLI - Factory AI Droid CLI
    Droid,
}

impl Platform {
    /// 获取平台的显示名称
    pub fn display_name(&self) -> &str {
        match self {
            Platform::Claude => "Claude Code",
            Platform::Codex => "Codex",
            Platform::Gemini => "Gemini CLI",
            Platform::Qwen => "Qwen CLI",
            Platform::IFlow => "iFlow CLI",
            Platform::Droid => "Factory Droid",
        }
    }

    /// 获取平台的简短名称（用于文件系统路径）
    pub fn short_name(&self) -> &str {
        match self {
            Platform::Claude => "claude",
            Platform::Codex => "codex",
            Platform::Gemini => "gemini",
            Platform::Qwen => "qwen",
            Platform::IFlow => "iflow",
            Platform::Droid => "droid",
        }
    }

    /// 获取平台的图标（用于 CLI 显示）
    pub fn icon(&self) -> &str {
        match self {
            Platform::Claude => "🤖",
            Platform::Codex => "💻",
            Platform::Gemini => "✨",
            Platform::Qwen => "🌟",
            Platform::IFlow => "🌊",
            Platform::Droid => "🏭",
        }
    }

    /// 检查平台是否已实现
    pub fn is_implemented(&self) -> bool {
        matches!(
            self,
            Platform::Claude | Platform::Codex | Platform::Gemini | Platform::Droid
        )
    }

    /// 列出所有平台
    pub fn all() -> Vec<Platform> {
        vec![
            Platform::Claude,
            Platform::Codex,
            Platform::Gemini,
            Platform::Qwen,
            Platform::IFlow,
            Platform::Droid,
        ]
    }

    /// 列出已实现的平台
    pub fn implemented() -> Vec<Platform> {
        Self::all()
            .into_iter()
            .filter(|p| p.is_implemented())
            .collect()
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short_name())
    }
}

impl FromStr for Platform {
    type Err = crate::core::error::CcrError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(Platform::Claude),
            "codex" => Ok(Platform::Codex),
            "gemini" => Ok(Platform::Gemini),
            "qwen" => Ok(Platform::Qwen),
            "iflow" => Ok(Platform::IFlow),
            "droid" | "factory" => Ok(Platform::Droid),
            _ => Err(crate::core::error::CcrError::PlatformNotFound(
                s.to_string(),
            )),
        }
    }
}

/// 📋 通用配置结构
///
/// 表示一个平台的配置 profile
///
/// ## 字段说明
/// - 通用字段：所有平台共享
/// - 平台特定字段：通过 `platform_data` 存储
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// 📝 配置描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 🌐 API 基础 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    /// 🔑 认证令牌/密钥
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,

    /// 🤖 默认模型名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// ⚡ 快速小模型名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_fast_model: Option<String>,

    /// 🏢 提供商名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// 🏷️ 提供商类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_type: Option<String>,

    /// 👤 账号标识
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,

    /// 🏷️ 标签列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    // === 🆕 使用统计和状态字段 ===
    /// 📊 使用次数统计
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage_count: Option<u32>,

    /// 🔘 启用/禁用状态
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// 📦 平台特定数据（扁平化存储）
    #[serde(flatten)]
    pub platform_data: IndexMap<String, serde_json::Value>,
}

#[allow(dead_code)]
impl ProfileConfig {
    /// 创建新的空配置
    pub fn new() -> Self {
        Self {
            description: None,
            base_url: None,
            auth_token: None,
            model: None,
            small_fast_model: None,
            provider: None,
            provider_type: None,
            account: None,
            tags: None,
            usage_count: None,
            enabled: None,
            platform_data: IndexMap::new(),
        }
    }

    /// 设置 base_url
    #[allow(dead_code)]
    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = Some(url);
        self
    }

    #[allow(dead_code)]
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = Some(desc);
        self
    }

    /// 设置认证令牌
    pub fn with_auth_token(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }

    /// 设置模型
    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    // === 🆕 使用统计和状态方法 ===

    /// 📊 获取使用次数
    pub fn usage_count(&self) -> u32 {
        self.usage_count.unwrap_or(0)
    }

    /// 🔘 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    /// 📈 递增使用次数
    pub fn increment_usage(&mut self) {
        let count = self.usage_count.unwrap_or(0);
        self.usage_count = Some(count + 1);
    }

    /// ✅ 启用配置
    pub fn enable(&mut self) {
        self.enabled = Some(true);
    }

    /// ❌ 禁用配置
    pub fn disable(&mut self) {
        self.enabled = Some(false);
    }
}

/// 🤖 为 ProfileConfig 实现自动补全功能
impl AutoCompletable for ProfileConfig {
    fn auto_complete(&mut self) -> bool {
        let mut modified = false;

        if self.usage_count.is_none() {
            self.usage_count = Some(0);
            modified = true;
            tracing::debug!("Auto-completed usage_count field for profile");
        }

        if self.enabled.is_none() {
            self.enabled = Some(true);
            modified = true;
            tracing::debug!("Auto-completed enabled field for profile");
        }

        modified
    }
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 📁 平台路径结构
///
/// 管理平台相关的所有目录和文件路径
///
/// ## 路径结构
/// ```text
/// ~/.ccr/                         # root
///   ├── config.toml               # registry_file
///   ├── platforms/
///   │   └── claude/               # platform_dir
///   │       ├── profiles.toml     # profiles_file
///   │       └── settings.json     # settings_file (平台特定)
///   ├── history/
///   │   └── claude.json           # history_file
///   └── backups/
///       └── claude/               # backups_dir
/// ```
#[derive(Debug, Clone)]
pub struct PlatformPaths {
    /// 📁 CCR 根目录 (默认 ~/.ccr/)
    pub root: PathBuf,

    /// 📋 平台注册表文件 (config.toml)
    pub registry_file: PathBuf,

    /// 📂 平台目录 (platforms/{platform}/)
    pub platform_dir: PathBuf,

    /// 📝 平台配置文件 (profiles.toml)
    pub profiles_file: PathBuf,

    /// ⚙️ 平台设置文件 (settings.json，平台特定)
    pub settings_file: PathBuf,

    /// 📜 历史记录文件 (history/{platform}.json)
    pub history_file: PathBuf,

    /// 💾 备份目录 (backups/{platform}/)
    pub backups_dir: PathBuf,
}

impl PlatformPaths {
    /// 🏗️ 创建平台路径结构
    ///
    /// # 参数
    /// - `platform`: 平台类型
    ///
    /// # 返回
    /// - Ok(PlatformPaths): 成功创建的路径结构
    /// - Err: 无法获取用户主目录或自定义根目录无效
    pub fn new(platform: Platform) -> Result<Self> {
        let root = Self::get_ccr_root()?;
        let platform_name = platform.short_name();

        Ok(Self {
            registry_file: root.join("config.toml"),
            platform_dir: root.join("platforms").join(platform_name),
            profiles_file: root
                .join("platforms")
                .join(platform_name)
                .join("profiles.toml"),
            settings_file: root
                .join("platforms")
                .join(platform_name)
                .join("settings.json"),
            history_file: root.join("history").join(format!("{}.json", platform_name)),
            backups_dir: root.join("backups").join(platform_name),
            root,
        })
    }

    /// 🏠 获取 CCR 根目录
    ///
    /// 优先级：
    /// 1. 环境变量 `CCR_ROOT`
    /// 2. 默认路径 `~/.ccr/`
    fn get_ccr_root() -> Result<PathBuf> {
        if let Ok(custom_root) = std::env::var("CCR_ROOT") {
            Ok(PathBuf::from(custom_root))
        } else {
            let home = dirs::home_dir().ok_or_else(|| {
                crate::core::error::CcrError::ConfigError("无法获取用户主目录".into())
            })?;
            Ok(home.join(".ccr"))
        }
    }

    /// 📁 确保所有目录存在
    pub fn ensure_directories(&self) -> Result<()> {
        use std::fs;

        let history_parent = self.history_file.parent().ok_or_else(|| {
            crate::core::error::CcrError::ConfigError(format!(
                "无法获取 history 文件的父目录: {:?}",
                self.history_file
            ))
        })?;

        let dirs = [
            &self.root,
            &self.platform_dir,
            history_parent,
            &self.backups_dir,
        ];

        for dir in &dirs {
            if !dir.exists() {
                fs::create_dir_all(dir).map_err(|e| {
                    crate::core::error::CcrError::ConfigError(format!(
                        "创建目录失败 {:?}: {}",
                        dir, e
                    ))
                })?;
            }
        }

        Ok(())
    }
}

/// 🔌 平台配置接口 Trait
///
/// 定义所有平台实现必须提供的标准接口
///
/// ## 实现要求
/// 每个平台（Claude, Codex, Gemini等）都必须实现此 trait 的所有方法
///
/// ## 示例
/// ```rust,ignore
/// pub struct ClaudePlatform { /* ... */ }
///
/// impl PlatformConfig for ClaudePlatform {
///     fn platform_name(&self) -> &str { "claude" }
///     // ... 其他方法
/// }
/// ```
pub trait PlatformConfig: Send + Sync {
    /// 获取平台名称
    fn platform_name(&self) -> &str;

    /// 获取平台类型枚举
    fn platform_type(&self) -> Platform;

    /// 加载所有配置 profiles
    ///
    /// # 返回
    /// - Ok(IndexMap<名称, 配置>): 成功加载的所有 profiles
    /// - Err: 文件读取失败或解析失败
    fn load_profiles(&self) -> Result<IndexMap<String, ProfileConfig>>;

    /// 保存单个 profile
    ///
    /// # 参数
    /// - `name`: profile 名称
    /// - `profile`: profile 配置
    fn save_profile(&self, name: &str, profile: &ProfileConfig) -> Result<()>;

    /// 删除 profile
    ///
    /// # 参数
    /// - `name`: 要删除的 profile 名称
    fn delete_profile(&self, name: &str) -> Result<()>;

    /// 获取设置文件路径
    ///
    /// 返回平台特定的 settings.json 路径
    /// 例如：Claude 使用 ~/.claude/settings.json（硬编码路径）
    #[allow(dead_code)]
    fn get_settings_path(&self) -> PathBuf;

    /// 应用指定的 profile
    ///
    /// 将 profile 配置写入平台的 settings 文件
    ///
    /// # 参数
    /// - `name`: 要应用的 profile 名称
    fn apply_profile(&self, name: &str) -> Result<()>;

    /// 验证 profile 配置
    ///
    /// 检查 profile 是否包含所有必需字段且格式正确
    ///
    /// # 参数
    /// - `profile`: 要验证的配置
    fn validate_profile(&self, profile: &ProfileConfig) -> Result<()>;

    /// 获取当前激活的 profile 名称
    fn get_current_profile(&self) -> Result<Option<String>>;

    /// 列出所有 profile 名称
    fn list_profile_names(&self) -> Result<Vec<String>> {
        Ok(self.load_profiles()?.keys().cloned().collect())
    }

    /// 🌍 获取平台相关的环境变量名称
    ///
    /// 用于 switch 命令显示环境变量变化
    ///
    /// # 返回
    /// 环境变量名称列表
    fn get_env_var_names(&self) -> Vec<&'static str> {
        vec![]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_display() {
        assert_eq!(Platform::Claude.to_string(), "claude");
        assert_eq!(Platform::Codex.to_string(), "codex");
        assert_eq!(Platform::Gemini.to_string(), "gemini");
    }

    #[test]
    fn test_platform_from_str() {
        assert_eq!(Platform::from_str("claude").unwrap(), Platform::Claude);
        assert_eq!(Platform::from_str("CODEX").unwrap(), Platform::Codex);
        assert_eq!(Platform::from_str("Gemini").unwrap(), Platform::Gemini);
        assert!(Platform::from_str("invalid").is_err());
    }

    #[test]
    fn test_platform_is_implemented() {
        assert!(Platform::Claude.is_implemented());
        assert!(Platform::Codex.is_implemented());
        assert!(Platform::Gemini.is_implemented());
        assert!(!Platform::Qwen.is_implemented());
        assert!(!Platform::IFlow.is_implemented());
    }

    #[test]
    fn test_platform_all() {
        let platforms = Platform::all();
        assert_eq!(platforms.len(), 6);
        assert!(platforms.contains(&Platform::Claude));
        assert!(platforms.contains(&Platform::Qwen));
        assert!(platforms.contains(&Platform::Droid));
    }

    #[test]
    fn test_platform_implemented() {
        let implemented = Platform::implemented();
        assert_eq!(implemented.len(), 4);
        assert!(implemented.contains(&Platform::Claude));
        assert!(implemented.contains(&Platform::Droid));
        assert!(!implemented.contains(&Platform::Qwen));
    }

    #[test]
    fn test_profile_config_builder() {
        let profile = ProfileConfig::new()
            .with_description("Test profile".to_string())
            .with_base_url("https://api.example.com".to_string())
            .with_auth_token("test-token".to_string())
            .with_model("test-model".to_string());

        assert_eq!(profile.description, Some("Test profile".to_string()));
        assert_eq!(
            profile.base_url,
            Some("https://api.example.com".to_string())
        );
        assert_eq!(profile.auth_token, Some("test-token".to_string()));
        assert_eq!(profile.model, Some("test-model".to_string()));
    }

    #[test]
    fn test_platform_paths_structure() {
        // 注意：这个测试依赖于环境，在 CI 中可能需要 mock
        if let Ok(paths) = PlatformPaths::new(Platform::Claude) {
            assert!(paths.root.to_str().unwrap().contains(".ccr"));
            assert!(paths.platform_dir.to_str().unwrap().contains("claude"));
            assert!(
                paths
                    .profiles_file
                    .to_str()
                    .unwrap()
                    .ends_with("profiles.toml")
            );
        }
    }
}
