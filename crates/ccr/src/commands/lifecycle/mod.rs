//! 🔄 lifecycle 命令模块
//!
//! 初始化、清理、验证、优化等生命周期操作。

mod clean;
mod clear;
mod init;
mod optimize;
mod validate;

pub use clean::clean_command;
pub use clear::clear_command;
pub use init::init_command;
pub use optimize::optimize_command;
pub use validate::validate_command;
