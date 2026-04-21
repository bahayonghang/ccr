//! ⚙️ CCR 配置管理模块
//!
//! 负责读写和管理配置文件。
//!
//! ## 模块结构
//!
//! - [`types`] - `ProviderType`, `ConfigSection`, `GlobalSettings`
//! - [`ccs_config`] - `CcsConfig` 结构
//! - [`manager`] - `ConfigManager`

mod manager;

// 重新导出所有公共类型
pub use ccr_config::{CcsConfig, ConfigSection, GlobalSettings, ProviderType};
pub use manager::ConfigManager;
