// 🚀 CCR Tauri 库入口
// 定义 Tauri Commands，集成 CCR 核心服务层

pub mod commands;

// 重新导出 CCR 核心类型供 Commands 使用
pub use ccr::{
    services::{ConfigService, HistoryService, SettingsService, BackupService},
    managers::{CcsConfig, ClaudeSettings, ConfigSection},
    core::{CcrError, Result},
    utils::mask_sensitive,
};
