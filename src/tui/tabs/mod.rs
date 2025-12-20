// 📑 TUI Tabs 模块
// 包含所有 Tab 组件

mod configs;
mod history;
mod sessions;
mod sync;
mod system;

pub use configs::ConfigsTab;
pub use history::HistoryTab;
pub use sessions::SessionsTab;
pub use sync::SyncTab;
pub use system::SystemTab;
