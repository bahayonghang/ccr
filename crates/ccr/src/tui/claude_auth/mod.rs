// Claude Auth TUI 模块 — 官方订阅账号管理终端视图
// 现在嵌入主 TUI, 也可通过 `ccr claude` 直接进入

mod app;
pub(crate) mod ui;

pub use app::ClaudeAuthApp;

use ccr_core::core::error::Result;

/// 运行 Claude Auth TUI 应用。
///
/// 实际委托到主 TUI, 并预选中 Claude Auth 页签。
pub fn run_claude_auth_tui() -> Result<()> {
    super::run_tui_with_claude_auth()
}
