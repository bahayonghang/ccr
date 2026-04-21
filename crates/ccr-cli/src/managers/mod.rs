//! 📁 CCR Manager 层模块
//!
//! 负责数据访问和持久化管理。
//!
//! ## 职责
//!
//! - 📖 加载和解析配置文件
//! - 💾 保存数据到持久化存储
//! - 🔍 数据查询和验证
//! - 📋 管理文件生命周期

pub mod budget_manager;
pub mod builtin_prompts;
pub mod config;
pub mod config_editor;
pub mod config_file_handler;
pub mod config_validator;
pub mod conflict_checker;
pub mod cost_tracker;
pub mod mcp_preset_manager;
pub mod pricing_manager;
pub mod prompts_manager;
pub mod settings;
pub mod skills_manager;
pub mod sync_folder_manager;
pub mod temp_override;

// 重新导出常用类型（供外部使用）
// 注意: 这些导出是为了库的公共 API，即使在模块内未使用也需要保留
#[allow(unused_imports)]
pub use budget_manager::BudgetManager;
#[allow(unused_imports)]
pub use ccr_codex::managers::codex_config::{CachedCodexConfigManager, CodexConfigManager};
#[allow(unused_imports)]
pub use ccr_config::managers::platform_config::{
    PlatformConfigEntry, PlatformConfigManager, UnifiedConfig,
};
#[allow(unused_imports)]
pub use ccr_store::history::{
    EnvChange, HistoryEntry, HistoryManager, HistoryStats, OperationDetails, OperationResult,
    OperationType,
};
#[allow(unused_imports)]
pub use ccr_sync::{SyncConfig, SyncConfigManager};
#[allow(unused_imports)]
pub use config::{CcsConfig, ConfigManager, ConfigSection, GlobalSettings, ProviderType};
#[allow(unused_imports)]
pub use config_file_handler::ConfigFileHandler;
#[allow(unused_imports)]
pub use config_validator::{ConfigValidator, ValidationReport};
#[allow(unused_imports)]
pub use cost_tracker::CostTracker;
#[allow(unused_imports)]
pub use mcp_preset_manager::{McpPresetManager, McpSyncManager, get_builtin_presets};
#[allow(unused_imports)]
pub use pricing_manager::PricingManager;
#[allow(unused_imports)]
pub use settings::{CachedSettingsManager, ClaudeSettings, SettingsManager};
#[allow(unused_imports)]
pub use sync_folder_manager::SyncFolderManager;
#[allow(unused_imports)]
pub use temp_override::{TempOverride, TempOverrideManager};
