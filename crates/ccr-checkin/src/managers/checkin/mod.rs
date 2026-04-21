//! CheckIn 管理器子模块。

pub mod account_manager;
pub mod balance_manager;
pub mod builtin_providers;
pub mod export_manager;
pub mod provider_manager;
pub mod record_manager;
pub mod waf_cookie_manager;

pub use account_manager::AccountManager;
pub use balance_manager::BalanceManager;
pub use export_manager::ExportManager;
pub use provider_manager::ProviderManager;
pub use record_manager::{RecordManager, TodayStats};
pub use waf_cookie_manager::WafCookieManager;

use std::path::PathBuf;

/// 获取签到数据目录: `~/.ccr/checkin/`
pub fn get_checkin_dir() -> Result<PathBuf, crate::core::error::DbError> {
    let home = dirs::home_dir().ok_or(crate::core::error::DbError::HomeDirNotFound)?;
    let dir = home.join(".ccr").join("checkin");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| crate::core::error::DbError::DirectoryCreation(e.to_string()))?;
    }
    Ok(dir)
}
