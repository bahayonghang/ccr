//! 📦 data 命令模块
//!
//! 导出、导入、历史记录、统计等数据相关操作。

#[cfg(feature = "web")]
mod budget;
mod export;
mod history;
mod import;
#[cfg(feature = "web")]
mod pricing;
#[cfg(feature = "web")]
mod stats;

#[cfg(feature = "web")]
pub use budget::{BudgetArgs, budget_command};
pub use export::export_command;
pub use history::history_command;
pub use import::{ImportMode, import_command};
#[cfg(feature = "web")]
pub use pricing::{PricingArgs, pricing_command};
#[cfg(feature = "web")]
pub use stats::{StatsArgs, stats_command};
