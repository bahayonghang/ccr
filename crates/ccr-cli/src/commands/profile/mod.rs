//! 📋 profile 命令模块
//!
//! 管理 API 配置 profiles（添加、删除、切换、启用/禁用等）。

mod add;
mod current;
mod delete;
mod disable;
mod enable;
mod list;
mod switch;

pub use add::add_command;
pub use current::current_command;
pub use delete::delete_command;
pub use disable::disable_command;
pub use enable::enable_command;
pub use list::list_command;
pub use switch::{
    switch_command, switch_command_for_platform, switch_command_for_platform_with_options,
};
