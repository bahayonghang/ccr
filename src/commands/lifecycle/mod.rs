// 🔄 lifecycle 命令模块 - 生命周期管理
// 初始化、迁移、清理、验证、优化等操作

mod clean;
mod clear;
mod init;
mod migrate;
mod optimize;
mod validate;

pub use clean::clean_command;
pub use clear::clear_command;
pub use init::init_command;
pub use migrate::{migrate_check_command, migrate_command};
pub use optimize::optimize_command;
pub use validate::validate_command;
