// Codex Auth TUI application state machine
// Manages the Codex multi-account selector state

use crate::core::error::Result;
use crate::models::{
    CodexAccountQuota, CodexAuthItem, CodexRuntimeSummary, LoginState, TokenFreshness,
};
use crate::services::codex_auth_service::AuthReadSnapshot;
use crate::services::{CodexAuthService, CodexRollingUsage};
use crate::tui::overlay::Overlay;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use dirs::home_dir;
use ratatui::Frame;
use ratatui::layout::Rect;
use std::cell::Cell;
use std::sync::mpsc::TryRecvError;

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

/// Quota query state
#[derive(Debug, Clone)]
pub enum QuotaState {
    /// 未查询
    Idle,
    /// 查询中
    Loading,
    /// 已加载
    Loaded(Vec<CodexAccountQuota>),
    /// 查询失败
    Error(String),
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
    /// Current runtime interpretation (profile/auth control plane summary)
    pub runtime_summary: Option<CodexRuntimeSummary>,
    /// Service instance
    service: CodexAuthService,
    /// Last action info (action_type, account_name, success, error)
    pub last_action: Option<(String, String, bool, Option<String>)>,
    /// Usage data state
    pub usage_state: UsageState,
    /// Quota query state
    pub quota_state: QuotaState,
    /// Quota async result receiver
    quota_rx:
        Option<std::sync::mpsc::Receiver<std::result::Result<Vec<CodexAccountQuota>, String>>>,
    /// Usage async result receiver
    usage_rx: Option<std::sync::mpsc::Receiver<UsageState>>,
    /// Codex directory
    #[allow(dead_code)]
    codex_dir: Option<PathBuf>,
    /// 🖱️ Cached account list area for mouse hit-testing
    pub list_area: Cell<Option<Rect>>,
    /// Delayed quota fetch timer (tick countdown, None = inactive)
    delayed_quota_ticks: Option<u32>,
    /// Delayed usage fetch timer (tick countdown, None = inactive)
    delayed_usage_ticks: Option<u32>,
    /// Whether a quota refresh confirmation is pending
    pub pending_quota_confirm: bool,
}

#[allow(dead_code)]
impl CodexAuthApp {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        let service = CodexAuthService::new()?;
        let snapshot = service.read_auth_snapshot()?;
        let login_state = snapshot.login_state.clone();
        let accounts = service.build_account_items(&snapshot)?;

        // Find the current account index
        let selected_index = accounts.iter().position(|a| a.is_current).unwrap_or(0);

        // Codex directory
        let codex_dir = home_dir().map(|d| d.join(".codex"));

        Ok(Self {
            accounts,
            selected_index,
            current_page: 0,
            overlay: None,
            toasts: ToastManager::new(),
            should_quit: false,
            login_state,
            runtime_summary: service.get_runtime_summary().ok(),
            service,
            last_action: None,
            usage_state: UsageState::Loading,
            quota_state: QuotaState::Idle,
            quota_rx: None,
            usage_rx: None,
            codex_dir,
            list_area: Cell::new(None),
            delayed_quota_ticks: None,
            delayed_usage_ticks: None,
            pending_quota_confirm: false,
        })
    }

    fn apply_snapshot(&mut self, snapshot: AuthReadSnapshot) -> Result<()> {
        self.login_state = snapshot.login_state.clone();
        self.accounts = self.service.build_account_items(&snapshot)?;
        self.runtime_summary = self.service.get_runtime_summary().ok();

        if self.selected_index >= self.accounts.len() {
            self.selected_index = self.accounts.len().saturating_sub(1);
        }

        Ok(())
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
        self.start_usage_fetch();
    }

    /// Reload account list
    pub fn reload_accounts(&mut self) -> Result<()> {
        let snapshot = self.service.read_auth_snapshot()?;
        self.apply_snapshot(snapshot)
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
        // 配额刷新确认拦截
        if self.pending_quota_confirm {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.pending_quota_confirm = false;
                    self.start_quota_fetch();
                    self.toasts.push(Toast::info("正在查询配额余额..."));
                }
                _ => {
                    self.pending_quota_confirm = false;
                    self.toasts.push(Toast::info("已取消配额查询"));
                }
            }
            return Ok(false);
        }

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
            KeyCode::Char('s') => match &self.login_state {
                LoginState::LoggedInUnsaved => {
                    self.overlay = Some(Overlay::save_input());
                }
                LoginState::ApiKeyActive | LoginState::ProviderKeyActive { .. } => {
                    self.toasts.push(Toast::info("API Key 模式无需保存账号"));
                }
                _ => {
                    self.toasts.push(Toast::warning("当前登录已保存或未登录"));
                }
            },
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
            KeyCode::Char('R') => {
                if let Some(account) = self.selected_account().cloned() {
                    if account.is_virtual {
                        self.toasts.push(Toast::warning("未保存的当前登录无法修复"));
                        return Ok(false);
                    }

                    match crate::services::CodexOAuthTokenService::new() {
                        Ok(oauth) => match oauth.repair_saved_account(&account.name) {
                            Ok(outcome) => {
                                if outcome.updated {
                                    self.toasts.push(Toast::success(outcome.message));
                                    // 修复后刷新列表与配额
                                    self.reload_accounts()?;
                                    self.start_quota_fetch();
                                } else {
                                    self.toasts.push(Toast::info(outcome.message));
                                }
                            }
                            Err(e) => {
                                self.toasts.push(Toast::error(format!("修复失败: {}", e)));
                            }
                        },
                        Err(e) => {
                            self.toasts
                                .push(Toast::error(format!("初始化修复服务失败: {}", e)));
                        }
                    }
                }
            }
            KeyCode::Char('b') => {
                self.pending_quota_confirm = true;
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
            TokenFreshness::Unknown(_) => "⚪ 未知",
        }
    }

    /// Called when this tab becomes active (e.g., tab switch)
    /// Schedules delayed usage/quota fetches after ~1 second (4 ticks at 250ms)
    pub fn on_activated(&mut self) {
        if matches!(self.usage_state, UsageState::Loading) && self.usage_rx.is_none() {
            self.delayed_usage_ticks = Some(1);
        }
        if matches!(self.quota_state, QuotaState::Idle) {
            self.delayed_quota_ticks = Some(4);
        }
    }

    /// Start async usage fetch in background thread
    fn start_usage_fetch(&mut self) {
        if self.usage_rx.is_some() {
            return;
        }

        self.usage_state = UsageState::Loading;
        let codex_dir = self.codex_dir.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        self.usage_rx = Some(rx);

        std::thread::spawn(move || {
            let state = Self::load_usage_data(&codex_dir);
            let _ = tx.send(state);
        });
    }

    /// Start async quota fetch in background thread
    fn start_quota_fetch(&mut self) {
        // 避免重复查询
        if matches!(self.quota_state, QuotaState::Loading) {
            return;
        }

        self.quota_state = QuotaState::Loading;
        let (tx, rx) = std::sync::mpsc::channel();
        self.quota_rx = Some(rx);

        // 在后台线程中执行异步配额查询
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx.send(Err(format!("创建运行时失败: {}", e)));
                    return;
                }
            };
            rt.block_on(async {
                match crate::services::CodexQuotaService::new() {
                    Ok(service) => {
                        let quotas = service.fetch_all_quotas().await;
                        let _ = tx.send(Ok(quotas));
                    }
                    Err(e) => {
                        let _ = tx.send(Err(format!("初始化配额服务失败: {}", e)));
                    }
                }
            });
        });
    }

    /// Run a delayed fetch timer and return whether it is ready to trigger
    fn delayed_fetch_ready(ticks: &mut Option<u32>) -> bool {
        let Some(remaining) = ticks.as_mut() else {
            return false;
        };

        if *remaining == 0 {
            *ticks = None;
            true
        } else {
            *remaining -= 1;
            false
        }
    }

    /// Poll local usage fetch result and return whether it changed visible state
    fn poll_usage_result(&mut self) -> bool {
        let Some(rx) = &self.usage_rx else {
            return false;
        };

        match rx.try_recv() {
            Ok(state) => {
                self.usage_state = state;
                self.usage_rx = None;
                true
            }
            Err(TryRecvError::Empty) => false,
            Err(TryRecvError::Disconnected) => {
                self.usage_state = UsageState::Error("本地统计通道已断开".to_string());
                self.usage_rx = None;
                true
            }
        }
    }

    /// Poll remote quota fetch result and return whether it changed visible state
    fn poll_quota_result(&mut self) -> bool {
        let Some(rx) = &self.quota_rx else {
            return false;
        };

        match rx.try_recv() {
            Ok(Ok(quotas)) => {
                self.quota_state = QuotaState::Loaded(quotas);
                self.quota_rx = None;
                true
            }
            Ok(Err(e)) => {
                self.quota_state = QuotaState::Error(e);
                self.quota_rx = None;
                true
            }
            Err(TryRecvError::Empty) => false,
            Err(TryRecvError::Disconnected) => {
                self.quota_state = QuotaState::Error("配额查询通道已断开".to_string());
                self.quota_rx = None;
                true
            }
        }
    }
}

// -- TuiApp trait implementation --

fn account_list_hit_test(area: Rect, mouse_row: u16, page_len: usize) -> Option<usize> {
    if mouse_row < area.y || mouse_row >= area.y + area.height {
        return None;
    }

    let clicked_row = (mouse_row - area.y) as usize;
    if clicked_row < page_len {
        Some(clicked_row)
    } else {
        None
    }
}

impl TuiApp for CodexAuthApp {
    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if self.overlay.is_some() {
            return self.handle_overlay_key(key);
        }
        self.handle_normal_mode(key)
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<bool> {
        // Overlay 激活时忽略鼠标事件
        if self.overlay.is_some() {
            return Ok(false);
        }

        match mouse.kind {
            // 🖱️ 左键点击列表项
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(area) = self.list_area.get()
                    && let Some(idx) =
                        account_list_hit_test(area, mouse.row, self.current_page_accounts().len())
                {
                    self.selected_index = idx;
                }
            }
            // 🖱️ 滚轮上
            MouseEventKind::ScrollUp => {
                self.move_up();
            }
            // 🖱️ 滚轮下
            MouseEventKind::ScrollDown => {
                self.move_down();
            }
            _ => {}
        }
        Ok(false)
    }

    fn on_tick(&mut self) -> bool {
        let mut needs_redraw = self.toasts.tick();

        if Self::delayed_fetch_ready(&mut self.delayed_usage_ticks) {
            self.start_usage_fetch();
            needs_redraw = true;
        }
        if Self::delayed_fetch_ready(&mut self.delayed_quota_ticks) {
            self.start_quota_fetch();
            needs_redraw = true;
        }
        needs_redraw |= self.poll_usage_result();
        needs_redraw |= self.poll_quota_result();

        needs_redraw
    }

    fn render(&self, frame: &mut Frame) {
        super::ui::draw(frame, self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_list_hit_test_hits_first_row_of_body_area() {
        let area = Rect::new(4, 7, 60, 6);
        assert_eq!(account_list_hit_test(area, 7, 4), Some(0));
    }

    #[test]
    fn account_list_hit_test_ignores_header_row_above_body_area() {
        let area = Rect::new(4, 7, 60, 6);
        assert_eq!(account_list_hit_test(area, 6, 4), None);
    }

    #[test]
    fn account_list_hit_test_ignores_rows_beyond_visible_items() {
        let area = Rect::new(4, 7, 60, 6);
        assert_eq!(account_list_hit_test(area, 10, 2), None);
    }
}
