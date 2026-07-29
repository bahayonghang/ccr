//! 🎯 platform 命令模块
//!
//! 管理和切换不同的 AI 平台 (Claude, Codex, Gemini 等)。
//!
//! ## 子命令
//!
//! - [`platform_list_command`] - 列出所有可用平台
//! - [`platform_current_command`] - 显示当前平台信息
//! - [`platform_info_command`] - 显示指定平台详细信息
//! - [`platform_switch_command`] - 切换当前平台
//! - [`platform_init_command`] - 初始化平台配置
//! - `platform_profile_*_command` - 非交互式 Profile 变更命令

mod current;
mod info;
mod init;
mod list;
mod profile;
mod profile_card;
mod profile_init;
mod switch;

// 公共数据结构
mod types;

pub use current::platform_current_command;
pub use info::platform_info_command;
pub use init::platform_init_command;
pub use list::platform_list_command;
pub use profile::{
    PlatformProfileCreateArgs, platform_profile_create_command, platform_profile_delete_command,
    platform_profile_disable_command, platform_profile_enable_command,
    platform_profile_set_field_command,
};
pub use profile_card::print_status_card;
pub use profile_init::platform_profile_init_command;
pub use switch::platform_switch_command;

// 内部类型（供子模块使用）
#[allow(unused_imports)]
pub(crate) use types::*;
