//! ⚙️ CCR 配置管理模块
//!
//! 负责读写和管理配置文件。
//!
//! ## 模块结构
//!
//! - [`types`] - `ProviderType`, `ConfigSection`, `GlobalSettings`
//! - [`ccs_config`] - `CcsConfig` 结构
//! - [`manager`] - `ConfigManager`

pub(crate) mod catalog;
pub mod ccs_config;
mod manager;
pub mod repository;
pub mod types;

pub use catalog::{
    CatalogTransaction, increment_usage_count, install_profile_catalog, load_applied_operation,
    migrate_catalogs, snapshot_profile_catalog, validate_profile_catalog_file,
};
#[cfg(any(test, feature = "test-support"))]
pub use catalog::{
    fail_next_catalog_checkpoint, fail_next_catalog_commit, fail_next_profile_catalog_install,
    fail_next_usage_increment,
};

// 重新导出所有公共类型
pub use ccs_config::CcsConfig;
pub use manager::ConfigManager;
pub use repository::{ConfigPatch, ConfigSnapshot, FieldPatch};
pub use types::{ConfigSection, GlobalSettings, ProviderType};

#[cfg(test)]
mod repository_tests;
