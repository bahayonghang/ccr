// TUI module — terminal user interface
// Built on ratatui with unified runtime infrastructure

pub mod action;
mod app;
pub mod codex_auth;
mod event;
pub mod overlay;
pub mod runtime;
pub mod theme;
pub mod toast;
mod ui;

pub use app::App;
pub use event::EventHandler;

use crate::core::error::Result;
use runtime::{TerminalGuard, run_loop};

/// Print exit info for both profile and codex auth actions.
fn print_exit_info(app: &App) {
    // Profile switch result
    if let Some((platform, profile, success, error)) = &app.last_applied {
        if *success {
            println!("✅ [{}] 已切换到配置: {}", platform, profile);
        } else if let Some(err) = error {
            eprintln!("❌ [{}] 切换配置 {} 失败: {}", platform, profile, err);
        }
    }

    // Codex auth action result
    if let Some((action, name, success, error)) = &app.last_codex_action {
        if *success {
            println!("✅ {} 账号: {}", action, name);
        } else if let Some(err) = error {
            eprintln!("❌ {} 账号 {} 失败: {}", action, name, err);
        }
    }
}

/// Run the main profile-switching TUI.
pub fn run_tui() -> Result<()> {
    let mut guard = TerminalGuard::new()?;
    let mut app = App::new()?;
    let mut events = EventHandler::new(250);

    run_loop(&mut guard, &mut app, &mut events)?;

    // Must drop guard BEFORE printing so terminal leaves alternate screen first
    drop(guard);
    print_exit_info(&app);

    Ok(())
}

/// Run the main TUI pre-selected to the Codex tab.
pub fn run_tui_with_codex_auth() -> Result<()> {
    let mut guard = TerminalGuard::new()?;
    let mut app = App::new()?.with_codex_tab();
    let mut events = EventHandler::new(250);

    run_loop(&mut guard, &mut app, &mut events)?;

    // Must drop guard BEFORE printing so terminal leaves alternate screen first
    drop(guard);
    print_exit_info(&app);

    Ok(())
}
