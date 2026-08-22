// Grok Auth TUI: session presence and official logout

use crate::tui::runtime::TuiApp;
use crate::tui::toast::{Toast, ToastManager};
use ccr_cli::application::auth_off_for_platform;
use ccr_cli::models::Platform;
use ccr_cli::services::GrokAuthService;
use ccr_core::core::error::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::Frame;

pub struct GrokAuthApp {
    pub logged_in: bool,
    pub toasts: ToastManager,
    pub should_quit: bool,
    pub last_off: Option<(bool, Option<String>)>,
}

impl GrokAuthApp {
    pub fn new() -> Result<Self> {
        let logged_in = GrokAuthService::new()
            .current()
            .map(|current| current.logged_in)
            .unwrap_or(false);
        Ok(Self {
            logged_in,
            toasts: ToastManager::new(),
            should_quit: false,
            last_off: None,
        })
    }

    pub fn on_activated(&mut self) {
        if let Err(error) = self.reload() {
            self.toasts.push(Toast::error(crate::tui_format!(
                "Failed to refresh Grok session: {}",
                "刷新 Grok 会话失败：{}",
                error
            )));
        }
    }

    fn reload(&mut self) -> Result<()> {
        self.logged_in = GrokAuthService::new().current()?.logged_in;
        Ok(())
    }

    fn auth_off(&mut self) {
        match auth_off_for_platform(Platform::Grok) {
            Ok(result) => {
                self.last_off = Some((true, None));
                if result.changed {
                    self.toasts.push(Toast::success(crate::tui_text!(
                        "Logged out of the official Grok session",
                        "已登出 Grok 官方会话"
                    )));
                } else {
                    self.toasts.push(Toast::info(crate::tui_text!(
                        "No official Grok session file to remove",
                        "没有可删除的 Grok 官方会话文件"
                    )));
                }
                let _ = self.reload();
            }
            Err(error) => {
                let message = error.to_string();
                self.last_off = Some((false, Some(message.clone())));
                self.toasts.push(Toast::error(crate::tui_format!(
                    "Grok auth off failed: {}",
                    "Grok auth off 失败：{}",
                    message
                )));
            }
        }
    }
}

impl TuiApp for GrokAuthApp {
    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                return Ok(true);
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return Ok(true);
            }
            KeyCode::Char('o') | KeyCode::Char('O') => self.auth_off(),
            KeyCode::Char('r') => {
                self.reload()?;
                self.toasts.push(Toast::info(crate::tui_text!(
                    "Grok session status reloaded",
                    "已刷新 Grok 会话状态"
                )));
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_mouse(&mut self, _mouse: MouseEvent) -> Result<bool> {
        Ok(false)
    }

    fn on_tick(&mut self) -> bool {
        self.toasts.tick()
    }

    fn render(&mut self, frame: &mut Frame) {
        super::ui::draw(frame, self);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventKind;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        }
    }

    #[test]
    fn q_quits_and_s_is_not_a_save_action() {
        let mut app = GrokAuthApp {
            logged_in: false,
            toasts: ToastManager::new(),
            should_quit: false,
            last_off: None,
        };
        assert!(app.handle_key(key(KeyCode::Char('q'))).unwrap());
        assert!(app.should_quit);
        app.should_quit = false;
        assert!(!app.handle_key(key(KeyCode::Char('s'))).unwrap());
        assert!(app.last_off.is_none());
    }
}
