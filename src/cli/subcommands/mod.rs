// 子命令枚举模块
//
// 定义所有需要独立出来的子命令枚举

pub mod check;
pub mod platform;
pub mod sync;
pub mod ui;

pub use check::CheckAction;
pub use platform::PlatformAction;
pub use sync::{AllSyncAction, FolderAction, SyncAction};
pub use ui::{TempTokenAction, UiAction};
