// 📁 CCR Manager 层模块
// 负责数据访问和持久化管理
//
// Manager 层职责:
// - 📖 加载和解析配置文件
// - 💾 保存数据到持久化存储
// - 🔍 数据查询和验证
// - 📋 管理文件生命周期

pub mod config;
pub mod history;
pub mod settings;

// 重新导出常用类型（供外部使用）
// 注意: 这些导出是为了库的公共 API，即使在模块内未使用也需要保留
#[allow(unused_imports)]
pub use config::{CcsConfig, ConfigManager, ConfigSection, ProviderType};
#[allow(unused_imports)]
pub use history::{
    EnvChange, HistoryEntry, HistoryManager, HistoryStats, OperationDetails, OperationResult,
    OperationType,
};
#[allow(unused_imports)]
pub use settings::{ClaudeSettings, SettingsManager};

