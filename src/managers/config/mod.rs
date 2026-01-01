//! ⚙️ CCR 配置管理模块
//!
//! 负责读写和管理配置文件。
//!
//! ## 模块结构
//!
//! - [`types`] - `ProviderType`, `ConfigSection`, `GlobalSettings`
//! - [`ccs_config`] - `CcsConfig` 结构
//! - [`manager`] - `ConfigManager`
//! - [`migration`] - `MigrationStatus`

mod ccs_config;
mod manager;
mod migration;
mod types;

// 重新导出所有公共类型
pub use ccs_config::CcsConfig;
pub use manager::ConfigManager;
pub use migration::MigrationStatus;
pub use types::{ConfigSection, GlobalSettings, ProviderType};
