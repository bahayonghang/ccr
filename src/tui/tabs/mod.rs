// 📑 TUI Tabs 模块
// 包含所有 Tab 组件

mod configs;
mod history;
mod sync;
mod system;

pub use configs::ConfigsTab;
pub use history::HistoryTab;
pub use sync::SyncTab;
pub use system::SystemTab;
