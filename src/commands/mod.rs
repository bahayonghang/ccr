// CCR 命令模块
// 各个 CLI 子命令的实现

pub mod add;
pub mod clean;
pub mod current;
pub mod delete;
pub mod disable;
pub mod enable;
pub mod export;
pub mod history_cmd;
pub mod import;
pub mod init;
pub mod list;
pub mod migrate;
pub mod optimize;
pub mod platform;
#[cfg(feature = "web")]
pub mod stats;
pub mod switch;
#[cfg(feature = "web")]
pub mod sync_cmd;
#[cfg(feature = "web")]
pub mod sync_content_selector;
pub mod temp_token;
pub mod ui;
pub mod update;
pub mod validate;

pub use add::add_command;
pub use clean::clean_command;
pub use current::current_command;
pub use delete::delete_command;
pub use disable::disable_command;
pub use enable::enable_command;
pub use export::export_command;
pub use history_cmd::history_command;
pub use import::{ImportMode, import_command};
pub use init::init_command;
pub use list::list_command;
pub use migrate::{migrate_check_command, migrate_command};
pub use optimize::optimize_command;
pub use platform::{
    platform_current_command, platform_info_command, platform_init_command, platform_list_command,
    platform_switch_command,
};
#[cfg(feature = "web")]
pub use stats::{StatsArgs, stats_command};
pub use switch::switch_command;
#[cfg(feature = "web")]
pub use sync_content_selector::SyncContentSelector;
pub use temp_token::{temp_token_clear, temp_token_set, temp_token_show};
pub use ui::ui_command;
pub use update::update_command;
pub use validate::validate_command;
