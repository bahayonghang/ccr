// 🎯 platform 命令模块 - 多平台管理
// 📋 管理和切换不同的 AI 平台 (Claude, Codex, Gemini 等)

mod current;
mod info;
mod init;
mod list;
mod switch;

// 公共数据结构
mod types;

pub use current::platform_current_command;
pub use info::platform_info_command;
pub use init::platform_init_command;
pub use list::platform_list_command;
pub use switch::platform_switch_command;

// 内部类型（供子模块使用）
#[allow(unused_imports)]
pub(crate) use types::*;
