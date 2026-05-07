//! CCR 命令模块
//!
//! 各个 CLI 子命令的实现。
//!
//! ## 模块结构 (2025 重构版)
//!
//! - `common/` - 公共工具（模式检测、表格构建、交互提示）
//! - `platform/` - 平台管理命令
//! - `profile/` - 配置管理命令
//! - `lifecycle/` - 生命周期命令（初始化、清理等）
//! - `data/` - 数据操作命令（导入、导出、历史等）

// 🔧 公共基础设施
pub mod common;

// 📦 子模块
pub mod claude;
pub mod codex;
pub mod data;
pub mod doctor_cmd;
pub mod lifecycle;
pub mod migration;
pub mod opencode;
pub mod platform;
pub mod profile;

// 🔄 保留的独立命令（暂未迁移到子模块）
pub mod check_cmd;
pub mod prompts_cmd;
pub mod provider_cmd;
pub mod sessions_cmd;
pub mod skills_cmd;
pub mod sync_cmd;
pub mod sync_content_selector;
pub mod temp_cmd;
pub mod temp_token;
pub mod ui;
pub mod update;

// =============================================
// 📤 公共 API 导出（保持向后兼容）
// =============================================

// 🎯 Platform 命令
pub use platform::{
    platform_current_command, platform_info_command, platform_init_command, platform_list_command,
    platform_switch_command,
};

// 📋 Profile 命令
pub use profile::add_command;
pub use profile::current_command;
pub use profile::delete_command;
pub use profile::disable_command;
pub use profile::enable_command;
pub use profile::list_command;
pub use profile::switch_command;
#[allow(unused_imports)]
pub use profile::switch_command_for_platform;

// 🔄 Lifecycle 命令
pub use lifecycle::clear_command;
pub use lifecycle::init_command;
pub use lifecycle::optimize_command;
pub use lifecycle::validate_command;
pub use lifecycle::{clean_backups_command, clean_menu_command, clean_planfiles_command};

// 📦 Data 命令
pub use data::export_command;
pub use data::history_command;
pub use data::{BudgetArgs, budget_command};
pub use data::{ImportMode, import_command};
pub use data::{PricingArgs, pricing_command};
pub use data::{StatsArgs, stats_command};

// 🔧 其他命令
pub use check_cmd::check_conflicts_command;
pub use doctor_cmd::doctor_command;
pub use sync_content_selector::SyncContentSelector;
pub use temp_cmd::temp_command;
pub use temp_token::{temp_token_clear, temp_token_set, temp_token_show};
pub use ui::ui_command;
pub use update::update_command;
