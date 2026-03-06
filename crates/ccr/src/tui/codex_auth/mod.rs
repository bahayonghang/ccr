// Codex Auth TUI module — multi-account management terminal UI
// Now embedded in the main TUI as a composite route

mod app;
pub(crate) mod ui;

pub use app::CodexAuthApp;

use crate::core::error::Result;

/// Run the Codex Auth TUI application.
///
/// This now delegates to the main TUI with Codex Auth route pre-selected,
/// providing a unified TUI experience.
pub fn run_codex_auth_tui() -> Result<()> {
    super::run_tui_with_codex_auth()
}
