// Codex Auth TUI application state machine
// Manages the Codex multi-account selector state

use crate::core::error::Result;
use crate::models::{CodexAuthItem, LoginState, TokenFreshness};
use crate::services::{CodexAuthService, CodexRollingUsage};
use crate::tui::overlay::Overlay;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use dirs::home_dir;
use ratatui::Frame;

use crate::tui::runtime::TuiApp;
use crate::tui::toast::{Toast, ToastManager};
use std::path::PathBuf;

/// Maximum accounts per page
pub const PAGE_SIZE: usize = 10;

/// Usage data state
#[derive(Debug, Clone)]
pub enum UsageState {
    /// Loading
    #[allow(dead_code)]
    Loading,
    /// Loaded successfully
    Loaded(CodexRollingUsage),
    /// Load failed
    Error(String),
    /// No data
    NoData,
}

/// Codex Auth TUI application
pub struct CodexAuthApp {
    /// Account list
    pub accounts: Vec<CodexAuthItem>,
    /// Currently selected index
    pub selected_index: usize,
    /// Current page (0-based)
    pub current_page: usize,
    /// Active overlay (None = normal mode)
    pub overlay: Option<Overlay>,
    /// Toast notification manager
    pub toasts: ToastManager,
    /// Whether should quit
    pub should_quit: bool,
    /// Login state
    pub login_state: LoginState,
    /// Service instance
    service: CodexAuthService,
    /// Last action info (action_type, account_name, success, error)
    pub last_action: Option<(String, String, bool, Option<String>)>,
    /// Usage data state
    pub usage_state: UsageState,
    /// Codex directory
    #[allow(dead_code)]
    codex_dir: Option<PathBuf>,
}

#[allow(dead_code)]
impl CodexAuthApp {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        let service = CodexAuthService::new()?;
        let login_state = service.get_login_state()?;
        let accounts = service.list_accounts()?;

        // Find the current account index
        let selected_index = accounts.iter().position(|a| a.is_current).unwrap_or(0);

        // Codex directory
        let codex_dir = home_dir().map(|d| d.join(".codex"));

        // Load usage data
        let usage_state = Self::load_usage_data(&codex_dir);

        Ok(Self {
            accounts,
            selected_index,
            current_page: 0,
            overlay: None,
            toasts: ToastManager::new(),
            should_quit: false,
            login_state,
            service,
            last_action: None,
            usage_state,
            codex_dir,
        })
    }

    /// Load usage data
    fn load_usage_data(codex_dir: &Option<PathBuf>) -> UsageState {
        let Some(dir) = codex_dir else {
            return UsageState::Error("无法获取用户目录".to_string());
        };

        use crate::services::CodexUsageService;
        let usage_service = CodexUsageService::new(dir.clone());

        match usage_service.compute_rolling_usage() {
            Ok(usage) => {
                if usage.all_time.total_requests == 0 {
                    UsageState::NoData
                } else {
                    UsageState::Loaded(usage)
                }
            }
            Err(e) => UsageState::Error(e.to_string()),
        }
    }

    /// Refresh usage data
    #[allow(dead_code)]
    pub fn refresh_usage(&mut self) {
        self.usage_state = Self::load_usage_data(&self.codex_dir);
    }

    /// Reload account list
    pub fn reload_accounts(&mut self) -> Result<()> {
        self.login_state = self.service.get_login_state()?;
        self.accounts = self.service.list_accounts()?;

        // Ensure selected index is valid
        if self.selected_index >= self.accounts.len() {
            self.selected_index = self.accounts.len().saturating_sub(1);
        }

        Ok(())
    }

    /// Get current page accounts
    pub fn current_page_accounts(&self) -> &[CodexAuthItem] {
        let start = self.current_page * PAGE_SIZE;
        let end = (start + PAGE_SIZE).min(self.accounts.len());
        if start < self.accounts.len() {
            &self.accounts[start..end]
        } else {
            &[]
        }
    }

    /// Get total pages
    pub fn total_pages(&self) -> usize {
        if self.accounts.is_empty() {
            1
        } else {
            self.accounts.len().div_ceil(PAGE_SIZE)
        }
    }

    /// Get currently selected account
    pub fn selected_account(&self) -> Option<&CodexAuthItem> {
        let page_accounts = self.current_page_accounts();
        page_accounts.get(self.selected_index)
    }

    // ═══════════════════════════════════════════════════════════
    // Key handlers
    // ═══════════════════════════════════════════════════════════

    /// Handle normal mode key events
    fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                return Ok(true);
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return Ok(true);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
            }
            KeyCode::PageUp | KeyCode::Char('h') => {
                self.prev_page();
            }
            KeyCode::PageDown | KeyCode::Char('l') => {
                self.next_page();
            }
            KeyCode::Enter => {
                if self.switch_selected_account()? {
                    return Ok(true);
                }
            }
            KeyCode::Char('s') => {
                if matches!(self.login_state, LoginState::LoggedInUnsaved) {
                    self.overlay = Some(Overlay::save_input());
                } else {
                    self.toasts.push(Toast::warning("当前登录已保存或未登录"));
                }
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some(account) = self.selected_account() {
                    if !account.is_virtual {
                        self.overlay = Some(Overlay::confirm_delete(account.name.clone()));
                    } else {
                        self.toasts.push(Toast::warning("无法删除未保存的登录"));
                    }
                }
            }
            KeyCode::Char('r') => {
                self.reload_accounts()?;
                self.toasts.push(Toast::info("已刷新账号列表"));
            }
            _ => {}
        }
        Ok(false)
    }

    /// Dispatch overlay key events by variant
    fn handle_overlay_key(&mut self, key: KeyEvent) -> Result<bool> {
        let is_confirm = matches!(self.overlay, Some(Overlay::Confirm { .. }));
        if is_confirm {
            return self.handle_confirm_key(key);
        }
        self.handle_input_key(key)
    }

    /// Handle confirm overlay keys
    fn handle_confirm_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                // Extract subject before mutable operations
                let subject = match &self.overlay {
                    Some(Overlay::Confirm { subject, .. }) => subject.clone(),
                    _ => return Ok(false),
                };

                match self.service.delete_account(&subject) {
                    Ok(()) => {
                        self.last_action =
                            Some(("已删除".to_string(), subject.clone(), true, None));
                        self.toasts
                            .push(Toast::success(format!("已删除账号: {}", subject)));
                        self.reload_accounts()?;
                    }
                    Err(e) => {
                        self.toasts.push(Toast::error(format!("删除失败: {}", e)));
                    }
                }
                self.overlay = None;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.overlay = None;
                self.toasts.push(Toast::info("已取消删除"));
            }
            _ => {}
        }
        Ok(false)
    }

    /// Handle input overlay keys
    fn handle_input_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Enter => {
                let name = match &mut self.overlay {
                    Some(overlay) => overlay.take_input(),
                    None => String::new(),
                };

                if !name.is_empty() {
                    match self.service.save_current(&name, None, None, false) {
                        Ok(()) => {
                            self.last_action =
                                Some(("已保存".to_string(), name.clone(), true, None));
                            self.toasts
                                .push(Toast::success(format!("已保存账号: {}", name)));
                            self.reload_accounts()?;
                        }
                        Err(e) => {
                            self.toasts.push(Toast::error(format!("保存失败: {}", e)));
                        }
                    }
                }
                self.overlay = None;
            }
            KeyCode::Esc => {
                self.overlay = None;
                self.toasts.push(Toast::info("已取消保存"));
            }
            KeyCode::Backspace => {
                if let Some(overlay) = &mut self.overlay {
                    overlay.pop_char();
                }
            }
            KeyCode::Char(c) => {
                if (c.is_ascii_alphanumeric() || c == '_' || c == '-')
                    && let Some(overlay) = &mut self.overlay
                {
                    overlay.push_char(c);
                }
            }
            _ => {}
        }
        Ok(false)
    }

    // ═══════════════════════════════════════════════════════════
    // Navigation helpers
    // ═══════════════════════════════════════════════════════════

    /// Move selection up
    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if self.current_page > 0 {
            self.current_page -= 1;
            self.selected_index = PAGE_SIZE - 1;
        }
    }

    /// Move selection down
    fn move_down(&mut self) {
        let page_accounts = self.current_page_accounts();
        if self.selected_index < page_accounts.len().saturating_sub(1) {
            self.selected_index += 1;
        } else if self.current_page < self.total_pages() - 1 {
            self.current_page += 1;
            self.selected_index = 0;
        }
    }

    /// Previous page
    fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.selected_index = 0;
        }
    }

    /// Next page
    fn next_page(&mut self) {
        if self.current_page < self.total_pages() - 1 {
            self.current_page += 1;
            self.selected_index = 0;
        }
    }

    /// Switch to selected account
    /// Returns true if switch succeeded and TUI should exit
    fn switch_selected_account(&mut self) -> Result<bool> {
        if let Some(account) = self.selected_account().cloned() {
            if account.is_virtual {
                self.toasts.push(Toast::info("这是当前登录，无需切换"));
                return Ok(false);
            }

            if account.is_current {
                self.toasts.push(Toast::info("已经是当前账号"));
                return Ok(false);
            }

            // Expiry check
            if CodexAuthService::is_expired(account.expires_at) {
                self.toasts.push(Toast::warning("账号已过期，无法切换"));
                return Ok(false);
            }

            // Detect running Codex processes
            let running = self.service.detect_codex_process();
            if !running.is_empty() {
                self.toasts.push(Toast::warning(format!(
                    "警告: 检测到 {} 个 Codex 进程正在运行",
                    running.len()
                )));
            }

            match self.service.switch_account(&account.name) {
                Ok(()) => {
                    self.last_action =
                        Some(("已切换到".to_string(), account.name.clone(), true, None));
                    self.toasts
                        .push(Toast::success(format!("已切换到账号: {}", account.name)));
                    self.should_quit = true;
                    return Ok(true);
                }
                Err(e) => {
                    self.toasts.push(Toast::error(format!("切换失败: {}", e)));
                }
            }
        }
        Ok(false)
    }

    /// Get freshness display text
    pub fn freshness_text(freshness: TokenFreshness) -> &'static str {
        match freshness {
            TokenFreshness::Fresh => "🟢 新鲜",
            TokenFreshness::Stale => "🟡 陈旧",
            TokenFreshness::Old => "🔴 过期",
            TokenFreshness::Unknown => "⚪ 未知",
        }
    }
}

// -- TuiApp trait implementation --

impl TuiApp for CodexAuthApp {
    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if self.overlay.is_some() {
            return self.handle_overlay_key(key);
        }
        self.handle_normal_mode(key)
    }

    fn on_tick(&mut self) -> bool {
        self.toasts.tick()
    }

    fn render(&self, frame: &mut Frame) {
        super::ui::draw(frame, self);
    }
}
