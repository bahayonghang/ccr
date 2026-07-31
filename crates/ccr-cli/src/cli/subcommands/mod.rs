// 子命令枚举模块
//
// 定义所有需要独立出来的子命令枚举

pub mod check;
pub mod claude;
pub mod codex;
pub mod grok;
pub mod opencode;
pub mod platform;
pub mod profile_args;
pub mod project;
pub mod sync;
pub mod ui;

pub use check::CheckAction;
pub use claude::{ClaudeAction, ClaudeAuthAction, ClaudeProfileAction};
pub use codex::{CodexAction, CodexAuthAction, CodexProfileAction};
pub use grok::{GrokAction, GrokProfileAction};
pub use opencode::{OpenCodeAction, OpenCodeAuthAction};
pub use platform::PlatformAction;
pub use project::ProjectAction;
pub use sync::{AllSyncAction, FolderAction, SyncAction};
pub use ui::{TempTokenAction, UiAction};
