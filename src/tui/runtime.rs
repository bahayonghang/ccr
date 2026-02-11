// TUI unified runtime
// Provides TuiApp trait, RAII TerminalGuard, and generic run_loop

use crate::core::error::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Frame, Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};

use super::event::{Event, EventHandler};

/// Contract for any TUI view — implement this to plug into the unified runtime.
pub trait TuiApp {
    /// Handle a key event. Return `true` to exit the TUI.
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Result<bool>;

    /// Handle a tick event (periodic refresh, toast expiry, etc.).
    /// Return `true` if the UI needs a redraw (e.g. toast expired).
    fn on_tick(&mut self) -> bool {
        false
    }

    /// Render the current state to the frame.
    fn render(&self, frame: &mut Frame);
}

/// RAII guard that sets up the terminal on creation and restores it on drop.
///
/// This ensures the terminal is always restored, even on panic.
pub struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalGuard {
    /// Create a new guard: enables raw mode, enters alternate screen, enables mouse capture.
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)
            .map_err(|e| crate::core::error::CcrError::IoError(io::Error::other(e)))?;
        terminal
            .clear()
            .map_err(|e| crate::core::error::CcrError::IoError(io::Error::other(e)))?;

        Ok(Self { terminal })
    }

    /// Borrow the inner terminal for drawing.
    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

/// Generic event loop that drives any `TuiApp` through render → poll → dispatch.
pub fn run_loop<A: TuiApp>(
    guard: &mut TerminalGuard,
    app: &mut A,
    event_handler: &mut EventHandler,
) -> Result<()> {
    draw_frame(guard, app)?;

    loop {
        match event_handler.poll_event()? {
            Event::Key(key) => {
                if app.handle_key(key)? {
                    return Ok(());
                }
                draw_frame(guard, app)?;
            }
            Event::Resize => {
                guard
                    .terminal_mut()
                    .clear()
                    .map_err(|e| crate::core::error::CcrError::IoError(io::Error::other(e)))?;
                draw_frame(guard, app)?;
            }
            Event::Tick => {
                if app.on_tick() {
                    draw_frame(guard, app)?;
                }
            }
        }
    }
}

fn draw_frame<A: TuiApp>(guard: &mut TerminalGuard, app: &A) -> Result<()> {
    guard
        .terminal_mut()
        .draw(|f| app.render(f))
        .map_err(|e| crate::core::error::CcrError::IoError(io::Error::other(e)))?;
    Ok(())
}
