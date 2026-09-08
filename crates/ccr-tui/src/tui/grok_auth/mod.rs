// Grok Auth TUI — saved OAuth accounts and explicit runtime operations

mod app;
pub(crate) mod ui;

pub use app::GrokAuthApp;

use ccr_core::core::error::Result;

/// Run the main TUI pre-selected to the Grok Auth tab.
pub fn run_grok_auth_tui() -> Result<()> {
    super::run_tui_with_grok_auth()
}
