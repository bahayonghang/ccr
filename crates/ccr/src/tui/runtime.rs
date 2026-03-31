// TUI unified runtime
// Provides TuiApp trait, RAII TerminalGuard, and generic run_loop

use ccr_core::core::error::{CcrError, Result};
use crossterm::{
    cursor::{MoveTo, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use ratatui::{Frame, Terminal, backend::CrosstermBackend};
use std::{
    env,
    io::{self, IsTerminal, Stdout, Write},
};

use super::event::{Event, EventHandler};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TerminalSessionMode {
    raw_mode: bool,
    alternate_screen: bool,
    mouse_capture: bool,
    clear_on_enter: bool,
    clear_on_exit: bool,
}

impl TerminalSessionMode {
    fn detect() -> Result<Self> {
        let term = env::var("TERM").ok();
        let sty = env::var("STY").ok();
        let tmux = env::var("TMUX").ok();

        Self::from_env(
            term.as_deref(),
            sty.as_deref(),
            tmux.as_deref(),
            io::stdin().is_terminal(),
            io::stdout().is_terminal(),
        )
    }

    fn from_env(
        term: Option<&str>,
        sty: Option<&str>,
        tmux: Option<&str>,
        stdin_is_tty: bool,
        stdout_is_tty: bool,
    ) -> Result<Self> {
        if !stdin_is_tty || !stdout_is_tty {
            return Err(CcrError::UiError(
                "TUI 需要交互式终端。\n建议: 直接在终端中运行 `ccr`，或改用 `ccr list` / `ccr current`。"
                    .to_string(),
            ));
        }

        let term = term.unwrap_or_default();
        if term.eq_ignore_ascii_case("dumb") {
            return Err(CcrError::UiError(
                "当前终端能力不足（TERM=dumb），无法启动 TUI。\n建议: 使用支持 ANSI/光标控制的终端，或改用 `ccr list` / `ccr current`。"
                    .to_string(),
            ));
        }

        // 仅在明确处于 GNU Screen 会话时才降级为 inline 模式。
        // 远端 SSH 会话经常继承 TERM=screen-*，但这并不意味着当前就在 Screen 内。
        if sty.is_some() && tmux.is_none() {
            Ok(Self {
                raw_mode: true,
                alternate_screen: false,
                mouse_capture: false,
                clear_on_enter: true,
                clear_on_exit: true,
            })
        } else {
            Ok(Self {
                raw_mode: true,
                alternate_screen: true,
                mouse_capture: true,
                clear_on_enter: true,
                clear_on_exit: false,
            })
        }
    }
}

#[derive(Debug, Default)]
struct TerminalSetupState {
    raw_mode_enabled: bool,
    alternate_screen_enabled: bool,
    mouse_capture_enabled: bool,
}

/// Contract for any TUI view — implement this to plug into the unified runtime.
pub trait TuiApp {
    /// Handle a key event. Return `true` to exit the TUI.
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Result<bool>;

    /// Handle a mouse event. Return `true` to exit the TUI.
    /// Default implementation is a no-op.
    fn handle_mouse(&mut self, _mouse: crossterm::event::MouseEvent) -> Result<bool> {
        Ok(false)
    }

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
    mode: TerminalSessionMode,
    state: TerminalSetupState,
}

impl TerminalGuard {
    /// Create a new guard using the detected terminal session mode.
    pub fn new() -> Result<Self> {
        let mode = TerminalSessionMode::detect()?;
        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)
            .map_err(|e| ccr_core::core::error::CcrError::IoError(io::Error::other(e)))?;
        let mut state = TerminalSetupState::default();

        if let Err(err) = Self::setup_terminal(&mut terminal, &mut state, mode) {
            Self::restore_terminal(terminal.backend_mut(), &mut state, mode.clear_on_exit);
            return Err(err);
        }

        Ok(Self {
            terminal,
            mode,
            state,
        })
    }

    /// Borrow the inner terminal for drawing.
    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }

    fn setup_terminal(
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        state: &mut TerminalSetupState,
        mode: TerminalSessionMode,
    ) -> Result<()> {
        if mode.raw_mode {
            enable_raw_mode()?;
            state.raw_mode_enabled = true;
        }

        if mode.alternate_screen {
            execute!(terminal.backend_mut(), EnterAlternateScreen)?;
            state.alternate_screen_enabled = true;
        }

        if mode.mouse_capture {
            execute!(terminal.backend_mut(), EnableMouseCapture)?;
            state.mouse_capture_enabled = true;
        }

        if mode.clear_on_enter {
            terminal
                .clear()
                .map_err(|e| ccr_core::core::error::CcrError::IoError(io::Error::other(e)))?;
        }

        Ok(())
    }

    fn restore_terminal<W: Write>(
        writer: &mut W,
        state: &mut TerminalSetupState,
        clear_on_exit: bool,
    ) {
        if state.mouse_capture_enabled {
            let _ = execute!(writer, DisableMouseCapture);
            state.mouse_capture_enabled = false;
        }

        if state.alternate_screen_enabled {
            let _ = execute!(writer, LeaveAlternateScreen);
            state.alternate_screen_enabled = false;
        }

        if clear_on_exit {
            let _ = execute!(writer, Clear(ClearType::All), MoveTo(0, 0));
        }

        if state.raw_mode_enabled {
            let _ = disable_raw_mode();
            state.raw_mode_enabled = false;
        }

        let _ = execute!(writer, Show);
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        Self::restore_terminal(
            self.terminal.backend_mut(),
            &mut self.state,
            self.mode.clear_on_exit,
        );
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
            Event::Mouse(mouse) => {
                // 只处理可操作的鼠标事件，忽略移动/拖拽等高频无效事件
                if matches!(
                    mouse.kind,
                    crossterm::event::MouseEventKind::Down(_)
                        | crossterm::event::MouseEventKind::ScrollUp
                        | crossterm::event::MouseEventKind::ScrollDown
                ) {
                    if app.handle_mouse(mouse)? {
                        return Ok(());
                    }
                    draw_frame(guard, app)?;
                }
            }
            Event::Resize => {
                guard
                    .terminal_mut()
                    .clear()
                    .map_err(|e| ccr_core::core::error::CcrError::IoError(io::Error::other(e)))?;
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
        .map_err(|e| ccr_core::core::error::CcrError::IoError(io::Error::other(e)))?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn detect_defaults_to_fullscreen_when_terminal_is_normal() {
        let mode =
            TerminalSessionMode::from_env(Some("xterm-256color"), None, None, true, true).unwrap();

        assert!(mode.raw_mode);
        assert!(mode.alternate_screen);
        assert!(mode.mouse_capture);
        assert!(mode.clear_on_enter);
        assert!(!mode.clear_on_exit);
    }

    #[test]
    fn detect_uses_inline_mode_inside_screen() {
        let mode = TerminalSessionMode::from_env(
            Some("screen-256color"),
            Some("1234.pts-0.host"),
            None,
            true,
            true,
        )
        .unwrap();

        assert!(mode.raw_mode);
        assert!(!mode.alternate_screen);
        assert!(!mode.mouse_capture);
        assert!(mode.clear_on_enter);
        assert!(mode.clear_on_exit);
    }

    #[test]
    fn detect_does_not_downgrade_tmux_screen_term() {
        let mode = TerminalSessionMode::from_env(
            Some("screen-256color"),
            None,
            Some("/tmp/tmux-1000/default,123,0"),
            true,
            true,
        )
        .unwrap();

        assert!(mode.alternate_screen);
        assert!(mode.mouse_capture);
        assert!(!mode.clear_on_exit);
    }

    #[test]
    fn detect_does_not_assume_screen_from_term_alone() {
        let mode =
            TerminalSessionMode::from_env(Some("screen-256color"), None, None, true, true).unwrap();

        assert!(mode.alternate_screen);
        assert!(mode.mouse_capture);
        assert!(!mode.clear_on_exit);
    }

    #[test]
    fn detect_rejects_non_interactive_terminal() {
        let err = TerminalSessionMode::from_env(Some("xterm-256color"), None, None, false, true)
            .unwrap_err();

        assert!(matches!(err, CcrError::UiError(_)));
    }

    #[test]
    fn restore_terminal_resets_setup_state() {
        let mut buffer = Vec::new();
        let mut state = TerminalSetupState {
            raw_mode_enabled: false,
            alternate_screen_enabled: true,
            mouse_capture_enabled: true,
        };

        TerminalGuard::restore_terminal(&mut buffer, &mut state, false);

        assert!(!state.alternate_screen_enabled);
        assert!(!state.mouse_capture_enabled);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn restore_terminal_inline_mode_is_safe_without_optional_features() {
        let mut buffer = Vec::new();
        let mut state = TerminalSetupState::default();

        TerminalGuard::restore_terminal(&mut buffer, &mut state, true);

        assert!(!state.raw_mode_enabled);
        assert!(!state.alternate_screen_enabled);
        assert!(!state.mouse_capture_enabled);
        assert!(!buffer.is_empty());
    }
}
