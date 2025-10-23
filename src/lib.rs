// 🚀 CCR 库模块
// 导出公共 API 供测试和外部使用

// 分层模块
pub mod commands;
pub mod core;
pub mod managers;
pub mod services;
pub mod tui;
pub mod utils;
pub mod web;

// 重新导出常用类型
pub use core::{CcrError, ColorOutput, LockManager, Result, init_logger};
pub use managers::{
    CcsConfig, ClaudeSettings, ConfigManager, ConfigSection, GlobalSettings, HistoryManager,
    ProviderType, SettingsManager, TempOverride, TempOverrideManager,
};
pub use services::{BackupService, ConfigService, HistoryService, SettingsService};
pub use utils::{Validatable, mask_if_sensitive, mask_sensitive};
