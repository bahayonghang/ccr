//! 🔐 Codex Auth 子命令模块
//!
//! 管理 Codex 账号的保存、切换、删除等操作。
//!
//! ## 子命令
//!
//! - [`save_command`] - 保存当前登录到指定名称
//! - [`list_command`] - 列出所有已保存的账号
//! - [`switch_command`] - 切换到指定账号
//! - [`delete_command`] - 删除指定账号
//! - [`current_command`] - 显示当前账号信息

mod current;
mod delete;
mod list;
mod save;
mod switch;

pub use current::current_command;
pub use delete::delete_command;
pub use list::list_command;
pub use save::save_command;
pub use switch::switch_command;
