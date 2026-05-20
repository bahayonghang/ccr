//! 🚀 CCR (Claude Code Configuration Switcher) 库
//!
//! CCR 是一个强大的 Rust CLI 工具和库，用于管理多个 AI CLI 平台的配置。
//!
//! ## 支持的平台
//!
//! - **Claude Code**: Anthropic 官方 CLI ([`platforms::ClaudePlatform`])
//! - **Codex**: Codex CLI ([`platforms::CodexPlatform`])
//! - **Gemini**: Antigravity CLI（内部 key 保持 `gemini`）([`platforms::GeminiPlatform`])
//! - **Qwen**: 阿里通义千问 CLI (计划中)
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
//! CLI/TUI Layer → Services → Managers → Core/Utils
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
//! let manager = PlatformConfigManager::with_default()?;
//! let config = manager.load_or_create_default()?;
//! println!("当前平台: {}", config.current_platform);
//! # Ok::<(), ccr::CcrError>(())
//! ```
//!
//! ### 配置模式
//!
//! CCR 使用 Unified 模式，配置存储在 `~/.ccr/` 目录结构中。
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
//! let lock_manager = LockManager::with_default_path()?;
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

pub mod cli;
mod help;

pub use ccr_cli::{application, commands, managers, models, platforms, services, sync};

#[cfg(feature = "tui")]
pub use ccr_tui::tui;

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
/// - [`init_file_only_logger`] - TUI 模式日志初始化函数（仅文件输出）
pub use ccr_core::{
    AutoCompletable, CONFIG_LOCK, CcrError, ColorOutput, FileLock, LockManager, Result,
    Validatable, init_file_only_logger, init_logger, mask_if_sensitive, mask_sensitive,
};

/// 管理器层 - 数据访问和持久化
///
/// **配置管理**:
/// - [`ConfigManager`] - 配置管理器
/// - [`PlatformConfigManager`] - Unified 模式统一配置管理器
/// - [`SettingsManager`] - Claude Code settings.json 管理器
/// - [`HistoryManager`] - 操作历史记录管理器
/// - [`CostTracker`] - 成本追踪管理器
/// - [`BudgetManager`] - 预算控制管理器
/// - [`PricingManager`] - 价格表管理器
///
/// **配置类型**:
/// - [`CcsConfig`] - 配置结构
/// - [`UnifiedConfig`] - Unified 配置结构
/// - [`PlatformConfigEntry`] - 平台注册表条目
/// - [`ConfigSection`] - 配置段
/// - [`ClaudeSettings`] - Claude Code 设置结构
/// - [`GlobalSettings`] - 全局设置
///
/// **临时覆盖**:
/// - [`TempOverrideManager`] - 临时 token 覆盖管理器
/// - [`TempOverride`] - 临时覆盖配置
///
/// - [`ProviderType`] - 提供商类型枚举
pub use managers::{
    BudgetManager, CcsConfig, ClaudeSettings, ConfigManager, ConfigSection, CostTracker,
    GlobalSettings, HistoryManager, PlatformConfigEntry, PlatformConfigManager, PricingManager,
    ProviderType, SettingsManager, TempOverride, TempOverrideManager, UnifiedConfig,
};

pub use models::skills::{
    MarketplaceListResponse, MarketplaceSkill, NpxPlatformSupport, NpxStatus, SkillContent,
    SkillFileContent, SkillFileEntry, SkillInstallMeta, SkillInstallMode, SkillInstallStrategy,
    SkillInstallationRecord, SkillLifecycleSummary, SkillOrigin, SkillPlatformConfig,
    SkillPlatformSummary, SkillRecord, SkillSourceHealth, SkillSourceRecord,
    SkillSourceSkillRecord, SkillSourceType, SkillTargetRecord, SkillTargetStatus,
    SkillsInstallCommandPreview, SkillsInstallRequest, SkillsInstallReviewResponse,
    SkillsInstallReviewSource, SkillsInstallReviewTarget, SkillsInventoryQuery,
    SkillsInventoryResponse, SkillsNpxCapabilities, SkillsOnboardingCandidate,
    SkillsSourceManifest, SkillsSyncRequest,
};
/// 数据模型和平台 trait
///
/// **平台相关**:
/// - [`Platform`] - 平台类型枚举（Claude, Codex, Gemini, Qwen, Droid）
/// - [`PlatformConfig`] - 平台配置接口 trait
/// - [`PlatformPaths`] - 平台路径管理结构
///
/// **配置结构**:
/// - [`ProfileConfig`] - 通用 profile 配置结构
///
/// **成本追踪相关**:
/// - [`Cost`] - 成本信息结构
/// - [`CostRecord`] - 成本记录结构
/// - [`CostStats`] - 成本统计汇总
/// - [`TokenUsage`] - Token 使用情况
/// - [`TokenStats`] - Token 统计信息
/// - [`DailyCost`] - 每日成本记录
/// - [`TimeRange`] - 时间范围枚举
/// - [`ModelPricing`] - 模型定价信息
///
/// **预算控制相关**:
/// - [`BudgetConfig`] - 预算配置结构
/// - [`BudgetStatus`] - 预算状态结构
/// - [`BudgetLimits`] - 预算限制结构
/// - [`BudgetWarning`] - 预算警告结构
/// - [`BudgetPeriod`] - 预算周期枚举（Daily, Weekly, Monthly）
/// - [`LimitAction`] - 超限动作枚举（Warn, Log, None）
/// - [`PeriodCosts`] - 周期成本结构
///
/// **价格表相关**:
/// - [`PricingConfig`] - 价格表配置结构
pub use models::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};

// 成本追踪相关类型 (从 stats 子模块导入)
pub use models::stats::{
    Cost, CostRecord, CostStats, DailyCost, ModelPricing, TimeRange, TokenStats, TokenUsage,
};

// 预算控制相关类型 (从 budget 子模块导入)
pub use models::budget::{
    BudgetConfig, BudgetLimits, BudgetPeriod, BudgetStatus, BudgetWarning, LimitAction, PeriodCosts,
};

// 价格表相关类型 (从 pricing 子模块导入)
pub use models::pricing::PricingConfig;

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
/// - [`ValidateService`] - 验证服务
pub use services::{
    BackupService, ConfigService, HistoryService, SettingsService, SkillsService, ValidateService,
};

pub use ccr_store::sessions;
pub use ccr_store::{
    Database, Session, SessionEvent, SessionFilter, SessionIndexer, SessionStats, SessionStore,
    SessionSummary,
};
pub use sync::{
    FolderStats, SyncConfig, SyncConfigManager, SyncContentSelection, SyncContentSelector,
    SyncContentType, SyncFolder, SyncFolderManager, SyncFoldersConfig, SyncService, WebDavConfig,
    expand_path, get_ccr_sync_path,
};
