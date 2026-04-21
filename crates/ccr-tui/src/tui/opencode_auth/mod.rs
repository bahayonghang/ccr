// OpenCode Auth TUI module — manual multi-account selector for OpenCode openai auth

mod app;
pub(crate) mod ui;

pub use app::OpenCodeAuthApp;

use ccr_core::core::error::Result;

/// 运行 OpenCode Auth TUI 应用。
///
/// 实际委托到主 TUI, 并预选中 OpenCode Auth 页签。
pub fn run_opencode_auth_tui() -> Result<()> {
    super::run_tui_with_opencode_auth()
}
