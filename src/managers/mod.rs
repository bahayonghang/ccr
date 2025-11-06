// 📁 CCR Manager 层模块
// 负责数据访问和持久化管理
//
// Manager 层职责:
// - 📖 加载和解析配置文件
// - 💾 保存数据到持久化存储
// - 🔍 数据查询和验证
// - 📋 管理文件生命周期

pub mod config;
pub mod cost_tracker;
pub mod history;
pub mod platform_config;
pub mod settings;
pub mod sync_config;
pub mod sync_folder_manager;
pub mod temp_override;

// 重新导出常用类型（供外部使用）
// 注意: 这些导出是为了库的公共 API，即使在模块内未使用也需要保留
#[allow(unused_imports)]
pub use config::{
    CcsConfig, ConfigManager, ConfigSection, GlobalSettings, MigrationStatus, ProviderType,
};
#[allow(unused_imports)]
pub use cost_tracker::CostTracker;
#[allow(unused_imports)]
pub use history::{
    EnvChange, HistoryEntry, HistoryManager, HistoryStats, OperationDetails, OperationResult,
    OperationType,
};
#[allow(unused_imports)]
pub use platform_config::{PlatformConfigEntry, PlatformConfigManager, UnifiedConfig};
#[allow(unused_imports)]
pub use settings::{ClaudeSettings, SettingsManager};
#[allow(unused_imports)]
pub use sync_config::{SyncConfig, SyncConfigManager};
#[allow(unused_imports)]
pub use sync_folder_manager::SyncFolderManager;
#[allow(unused_imports)]
pub use temp_override::{TempOverride, TempOverrideManager};
