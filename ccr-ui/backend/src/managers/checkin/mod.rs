// 签到功能数据管理模块

pub mod account_manager;
pub mod balance_manager;
pub mod export_manager;
pub mod provider_manager;
pub mod record_manager;

pub use account_manager::AccountManager;
pub use balance_manager::BalanceManager;
pub use export_manager::ExportManager;
pub use provider_manager::ProviderManager;
pub use record_manager::RecordManager;

use std::path::PathBuf;

/// 获取签到数据目录路径
/// ~/.ccr/checkin/
#[allow(dead_code)]
pub fn get_checkin_dir() -> Result<PathBuf, std::io::Error> {
    let home = dirs::home_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot get home directory")
    })?;
    let checkin_dir = home.join(".ccr").join("checkin");

    if !checkin_dir.exists() {
        std::fs::create_dir_all(&checkin_dir)?;
    }

    Ok(checkin_dir)
}
