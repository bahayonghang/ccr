//! 🚀 CCR (Claude Code Configuration Switcher) 库
//!
//! CCR 是一个强大的 Rust CLI 工具和库，用于管理多个 AI CLI 平台的配置。
//!
//! ## 支持的平台
//!
//! - **Claude Code**: Anthropic 官方 CLI ([`platforms::ClaudePlatform`])
//! - **Codex**: GitHub Copilot CLI ([`platforms::CodexPlatform`])
//! - **Gemini**: Google Gemini CLI ([`platforms::GeminiPlatform`])
//! - **Qwen**: 阿里通义千问 CLI (计划中)
//! - **iFlow**: iFlow CLI (计划中)
//!
//! ## 核心特性
//!
//! - 🔄 **多平台支持**: 统一管理多个 AI CLI 平台
//! - 📋 **Profile 管理**: 每个平台支持多个配置 profiles
//! - 🔐 **安全性**: 自动掩码敏感信息，原子文件操作
//! - 📜 **审计日志**: 完整的操作历史记录
//! - 💾 **自动备份**: 修改前自动备份配置
//! - 🔒 **并发安全**: 文件锁防止并发修改冲突
//!
//! ## 架构层次
//!
//! ```text
//! CLI/Web Layer → Services → Managers → Core/Utils
//! ```
//!
//! ### 模块组织
//!
//! - [`core`] - 核心基础设施（错误处理、文件锁、日志）
//! - [`models`] - 数据模型和 trait 定义
//! - [`platforms`] - 各平台的具体实现
//! - [`managers`] - 数据访问和持久化层
//! - [`services`] - 业务逻辑和编排层
//! - [`commands`] - CLI 命令实现
//! - [`web`] - Web API 服务器
//! - [`tui`] - 终端用户界面
//! - [`utils`] - 工具函数和辅助类型
//!
//! ## 快速开始
//!
//! ### 作为库使用
//!
//! ```rust,no_run
//! use ccr::{Platform, create_platform, PlatformConfigManager};
//!
//! // 创建平台实例
//! let claude = create_platform(Platform::Claude)?;
//!
//! // 加载 profiles
//! let profiles = claude.load_profiles()?;
//!
//! // 应用 profile
//! claude.apply_profile("my-profile")?;
//!
//! // 使用统一配置管理器
//! let manager = PlatformConfigManager::default()?;
//! let config = manager.load_or_create_default()?;
//! println!("当前平台: {}", config.current_platform);
//! # Ok::<(), ccr::CcrError>(())
//! ```
//!
//! ### 配置模式
//!
//! CCR 支持两种配置模式：
//!
//! - **Legacy 模式**: 单平台模式，使用 `~/.ccs_config.toml`（兼容 CCS 工具）
//! - **Unified 模式**: 多平台模式，使用 `~/.ccr/` 目录结构
//!
//! ### 平台路径结构
//!
//! ```text
//! ~/.ccr/                         # root
//!   ├── config.toml               # 统一配置文件
//!   ├── platforms/
//!   │   ├── claude/               # Claude 平台
//!   │   │   ├── profiles.toml
//!   │   │   └── settings.json
//!   │   ├── codex/                # Codex 平台
//!   │   │   └── profiles.toml
//!   │   └── gemini/               # Gemini 平台
//!   │       └── profiles.toml
//!   ├── history/                  # 历史记录
//!   │   ├── claude.json
//!   │   ├── codex.json
//!   │   └── gemini.json
//!   └── backups/                  # 自动备份
//!       ├── claude/
//!       ├── codex/
//!       └── gemini/
//! ```
//!
//! ## 错误处理
//!
//! 所有可能失败的操作返回 [`Result<T, CcrError>`]。
//!
//! ```rust,no_run
//! use ccr::{CcrError, Result};
//!
//! fn my_function() -> Result<()> {
//!     // 操作...
//!     Ok(())
//! }
//! ```
//!
//! ## 并发安全
//!
//! CCR 使用文件锁（[`LockManager`]）确保并发安全：
//!
//! ```rust,no_run
//! use ccr::LockManager;
//!
//! let lock_manager = LockManager::default()?;
//! let _lock = lock_manager.lock_settings(std::time::Duration::from_secs(10))?;
//! // 执行操作...
//! // 锁在离开作用域时自动释放
//! # Ok::<(), ccr::CcrError>(())
//! ```
//!
//! ## 日志
//!
//! 使用环境变量控制日志级别：
//!
//! ```bash
//! export CCR_LOG_LEVEL=debug
//! ```
//!
//! ## 示例
//!
//! 完整的配置示例和故障排除指南请参考 `docs/examples/` 目录。

// 分层模块
pub mod commands;
pub mod core;
pub mod managers;
pub mod models;
pub mod platforms;
pub mod services;
pub mod utils;

// 可选功能模块
#[cfg(feature = "tui")]
pub mod tui;

#[cfg(feature = "web")]
pub mod web;

// ═══════════════════════════════════════════════════════════
// 核心类型导出
// ═══════════════════════════════════════════════════════════

/// 核心基础设施
///
/// - [`CcrError`] - 统一错误类型
/// - [`Result`] - Result<T, CcrError> 类型别名
/// - [`ColorOutput`] - 彩色终端输出
/// - [`LockManager`] - 文件锁管理器
/// - [`init_logger`] - 日志初始化函数
pub use core::{CcrError, ColorOutput, LockManager, Result, init_logger};

/// 管理器层 - 数据访问和持久化
///
/// **配置管理**:
/// - [`ConfigManager`] - Legacy 模式配置管理器（兼容 CCS）
/// - [`PlatformConfigManager`] - Unified 模式统一配置管理器
/// - [`SettingsManager`] - Claude Code settings.json 管理器
/// - [`HistoryManager`] - 操作历史记录管理器
///
/// **配置类型**:
/// - [`CcsConfig`] - Legacy 配置结构
/// - [`UnifiedConfig`] - Unified 配置结构
/// - [`PlatformConfigEntry`] - 平台注册表条目
/// - [`ConfigSection`] - Legacy 配置段
/// - [`ClaudeSettings`] - Claude Code 设置结构
/// - [`GlobalSettings`] - 全局设置
///
/// **临时覆盖**:
/// - [`TempOverrideManager`] - 临时 token 覆盖管理器
/// - [`TempOverride`] - 临时覆盖配置
///
/// **其他**:
/// - [`MigrationStatus`] - 迁移状态
/// - [`ProviderType`] - 提供商类型枚举
pub use managers::{
    CcsConfig, ClaudeSettings, ConfigManager, ConfigSection, GlobalSettings, HistoryManager,
    MigrationStatus, PlatformConfigEntry, PlatformConfigManager, ProviderType, SettingsManager,
    TempOverride, TempOverrideManager, UnifiedConfig,
};

/// 数据模型和平台 trait
///
/// **平台相关**:
/// - [`Platform`] - 平台类型枚举（Claude, Codex, Gemini, Qwen, iFlow）
/// - [`PlatformConfig`] - 平台配置接口 trait
/// - [`PlatformPaths`] - 平台路径管理结构
/// - [`ConfigMode`] - 配置模式枚举（Legacy, Unified）
///
/// **配置结构**:
/// - [`ProfileConfig`] - 通用 profile 配置结构
pub use models::{ConfigMode, Platform, PlatformConfig, PlatformPaths, ProfileConfig};

/// 平台实现和工厂
///
/// **工厂函数**:
/// - [`create_platform`] - 根据平台类型创建实例的工厂函数
///
/// **平台注册表**:
/// - [`PlatformRegistry`] - 平台注册表，管理所有可用平台信息
/// - [`PlatformDetector`] - 平台检测器，检测已配置的平台
/// - [`PlatformInfo`] - 平台显示信息结构
/// - [`PlatformStatus`] - 平台状态枚举
///
/// ## 示例
///
/// ```rust,no_run
/// use ccr::{Platform, create_platform};
///
/// // 创建 Claude 平台实例
/// let claude = create_platform(Platform::Claude)?;
/// let profiles = claude.load_profiles()?;
/// # Ok::<(), ccr::CcrError>(())
/// ```
pub use platforms::{
    PlatformDetector, PlatformInfo, PlatformRegistry, PlatformStatus, create_platform,
};

/// 服务层 - 业务逻辑编排
///
/// - [`ConfigService`] - 配置切换和管理服务
/// - [`SettingsService`] - 设置管理服务
/// - [`HistoryService`] - 历史记录服务
/// - [`BackupService`] - 备份和恢复服务
pub use services::{BackupService, ConfigService, HistoryService, SettingsService};

/// 工具函数和辅助类型
///
/// **验证**:
/// - [`Validatable`] - 可验证 trait
///
/// **敏感信息掩码**:
/// - [`mask_sensitive`] - 掩码敏感字符串
/// - [`mask_if_sensitive`] - 条件掩码（根据键名判断）
pub use utils::{Validatable, mask_if_sensitive, mask_sensitive};
