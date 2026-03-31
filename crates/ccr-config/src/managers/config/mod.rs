//! ⚙️ CCR 配置管理模块
//!
//! 负责读写和管理配置文件。
//!
//! ## 模块结构
//!
//! - [`types`] - `ProviderType`, `ConfigSection`, `GlobalSettings`
//! - [`ccs_config`] - `CcsConfig` 结构

pub mod ccs_config;
pub mod types;

// 重新导出所有公共类型
pub use ccs_config::CcsConfig;
pub use types::{ConfigSection, GlobalSettings, ProviderType};
