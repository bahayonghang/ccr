// 🚀 CCR 库模块
// 导出公共 API 供测试和外部使用

// 核心模块
pub mod config;
pub mod core;
pub mod error;
pub mod history;
pub mod lock;
pub mod logging;
pub mod services;
pub mod settings;
pub mod utils;

// 重新导出常用类型
pub use config::{CcsConfig, ConfigManager, ConfigSection};
pub use error::{CcrError, Result};
pub use services::{BackupService, ConfigService, HistoryService, SettingsService};
pub use settings::{ClaudeSettings, SettingsManager};
pub use utils::{Validatable, mask_if_sensitive, mask_sensitive};
