// OpenCode Auth TUI application state machine
// Manages manual save/switch/delete for OpenCode openai auth snapshots

use crate::models::{
    OpenCodeAuthItem, OpenCodeAuthRegistry, OpenCodeLoginState, OpenCodeReadSnapshot,
};
use crate::services::{
    OpenCodeAuthService, OpenCodeRollingUsage, OpenCodeUsageRecord, OpenCodeUsageService,
};
use crate::tui::overlay::Overlay;
use crate::tui::runtime::TuiApp;
use crate::tui::toast::{Toast, ToastManager};
use ccr_core::core::error::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::Frame;
use ratatui::layout::Rect;
use std::cell::Cell;
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;

/// Maximum accounts per page
pub const PAGE_SIZE: usize = 10;

const OPENAI_PROVIDER_ID: &str = "openai";

#[derive(Debug, Clone)]
pub struct OpenCodeUsageDataset {
    pub provider_id: String,
    pub records: Vec<OpenCodeUsageRecord>,
    pub rolling: OpenCodeRollingUsage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenCodeUsageAttributionState {
    ProviderGlobal,
    CurrentSavedSelection,
    SavedSelectionFallback,
    VirtualCurrentLogin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenCodeUsageTopModel {
    pub model: String,
    pub total_tokens: u64,
    pub total_requests: u64,
}

#[derive(Debug, Clone)]
pub struct OpenCodeAuthUsagePanelData {
    pub provider_label: String,
    pub attribution_state: OpenCodeUsageAttributionState,
    pub rolling: OpenCodeRollingUsage,
    pub record_count: usize,
    pub top_model: Option<OpenCodeUsageTopModel>,
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub enum OpenCodeUsageState {
    Loading,
    Loaded(OpenCodeUsageDataset),
    Error(String),
    NoData,
}

/// OpenCode Auth TUI application
pub struct OpenCodeAuthApp {
    /// Account list
    pub accounts: Vec<OpenCodeAuthItem>,
    /// Currently selected index (within current page)
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
    pub login_state: OpenCodeLoginState,
    /// Auth registry snapshot
    pub auth_registry: OpenCodeAuthRegistry,
    /// Service instance
    service: OpenCodeAuthService,
    /// OpenCode 数据目录（用于本地 usage 统计）
    opencode_dir: PathBuf,
    /// Last action info (action_type, account_name, success, error)
    pub last_action: Option<(String, String, bool, Option<String>)>,
    /// Usage data state
    pub usage_state: OpenCodeUsageState,
    /// Cached account list area for mouse hit-testing
    pub list_area: Cell<Option<Rect>>,
    /// Usage async result receiver
    usage_rx: Option<std::sync::mpsc::Receiver<OpenCodeUsageState>>,
    /// Delayed usage fetch timer
    delayed_usage_ticks: Option<u32>,
}

impl OpenCodeAuthApp {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        Self::from_service(OpenCodeAuthService::new()?)
    }

    pub(crate) fn from_service(service: OpenCodeAuthService) -> Result<Self> {
        let snapshot = service.read_auth_snapshot()?;
        let accounts = service.build_account_items(&snapshot)?;
        let selected_index = accounts
            .iter()
            .position(|account| account.is_current)
            .unwrap_or(0);
        let opencode_dir = service.opencode_dir().to_path_buf();

        Ok(Self {
            accounts,
            selected_index,
            current_page: 0,
            overlay: None,
            toasts: ToastManager::new(),
            should_quit: false,
            login_state: snapshot.login_state.clone(),
            auth_registry: snapshot.registry.clone(),
            service,
            opencode_dir,
            last_action: None,
            usage_state: OpenCodeUsageState::Loading,
            list_area: Cell::new(None),
            usage_rx: None,
            delayed_usage_ticks: Some(1),
        })
    }

    fn apply_snapshot(&mut self, snapshot: OpenCodeReadSnapshot) -> Result<()> {
        self.login_state = snapshot.login_state.clone();
        self.auth_registry = snapshot.registry.clone();
        self.accounts = self.service.build_account_items(&snapshot)?;

        let preferred = self
            .accounts
            .iter()
            .position(|account| account.is_current)
            .unwrap_or(0);
        self.current_page = preferred / PAGE_SIZE;
        let page_len = self.current_page_accounts().len();
        self.selected_index = if page_len == 0 {
            0
        } else {
            preferred % PAGE_SIZE
        };

        Ok(())
    }

    /// Reload account list from disk
    pub fn reload_accounts(&mut self) -> Result<()> {
        let snapshot = self.service.read_auth_snapshot()?;
        self.apply_snapshot(snapshot)
    }

    /// Get current page accounts
    pub fn current_page_accounts(&self) -> &[OpenCodeAuthItem] {
        let start = self.current_page * PAGE_SIZE;
        let end = (start + PAGE_SIZE).min(self.accounts.len());
        if start >= self.accounts.len() {
            &[]
        } else {
            &self.accounts[start..end]
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
    pub fn selected_account(&self) -> Option<&OpenCodeAuthItem> {
        self.current_page_accounts().get(self.selected_index)
    }

    /// Called when this tab becomes active
    pub fn on_activated(&mut self) {
        if matches!(self.usage_state, OpenCodeUsageState::Loading) && self.usage_rx.is_none() {
            self.delayed_usage_ticks = Some(1);
        }
    }

    pub fn usage_panel_data(&self) -> Option<OpenCodeAuthUsagePanelData> {
        let OpenCodeUsageState::Loaded(dataset) = &self.usage_state else {
            return None;
        };

        let selected = self.selected_account();
        let top_model = Self::top_model_from_usage(&dataset.rolling);
        let fallback_reason = match selected {
            None => Some("当前未选中账号，以下展示本机 OpenCode openai provider 的本地 usage 汇总".to_string()),
            Some(account) if account.is_virtual => Some(
                "当前登录尚未保存为 CCR 账号；OpenCode 本地日志仅记录 provider/model，不包含保存账号 id，以下为 openai provider 汇总"
                    .to_string(),
            ),
            Some(account) if account.is_current => Some(
                "当前选中账号就是当前登录，但 OpenCode 本地日志仍只记录 provider/model；以下为 openai provider 汇总，而不是按账号精确归因"
                    .to_string(),
            ),
            Some(_) => Some(
                "所选账号不是当前登录；OpenCode 本地日志不包含保存账号 id，以下为当前机器 openai provider 汇总，不代表该账号独立历史"
                    .to_string(),
            ),
        };
        let attribution_state = match selected {
            None => OpenCodeUsageAttributionState::ProviderGlobal,
            Some(account) if account.is_virtual => {
                OpenCodeUsageAttributionState::VirtualCurrentLogin
            }
            Some(account) if account.is_current => {
                OpenCodeUsageAttributionState::CurrentSavedSelection
            }
            Some(_) => OpenCodeUsageAttributionState::SavedSelectionFallback,
        };

        Some(OpenCodeAuthUsagePanelData {
            provider_label: dataset.provider_id.clone(),
            attribution_state,
            rolling: dataset.rolling.clone(),
            record_count: dataset.records.len(),
            top_model,
            fallback_reason,
        })
    }

    pub fn refresh_usage(&mut self) {
        self.start_usage_fetch();
    }

    fn load_usage_data(usage_service: &OpenCodeUsageService) -> OpenCodeUsageState {
        match usage_service.parse_provider_messages(OPENAI_PROVIDER_ID) {
            Ok(records) => {
                if records.is_empty() {
                    OpenCodeUsageState::NoData
                } else {
                    OpenCodeUsageState::Loaded(OpenCodeUsageDataset {
                        provider_id: OPENAI_PROVIDER_ID.to_string(),
                        rolling: OpenCodeUsageService::compute_rolling_usage_for_records(&records),
                        records,
                    })
                }
            }
            Err(err) => OpenCodeUsageState::Error(err.to_string()),
        }
    }

    fn top_model_from_usage(usage: &OpenCodeRollingUsage) -> Option<OpenCodeUsageTopModel> {
        usage
            .by_model
            .iter()
            .max_by_key(|(_, stats)| stats.total_input_tokens + stats.total_output_tokens)
            .map(|(model, stats)| OpenCodeUsageTopModel {
                model: model.clone(),
                total_tokens: stats.total_input_tokens + stats.total_output_tokens,
                total_requests: stats.total_requests,
            })
    }

    fn start_usage_fetch(&mut self) {
        if self.usage_rx.is_some() {
            return;
        }

        self.usage_state = OpenCodeUsageState::Loading;
        let usage_service = OpenCodeUsageService::new(self.opencode_dir.clone());
        let (tx, rx) = std::sync::mpsc::channel();
        self.usage_rx = Some(rx);

        std::thread::spawn(move || {
            let state = Self::load_usage_data(&usage_service);
            let _ = tx.send(state);
        });
    }

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
                self.usage_state =
                    OpenCodeUsageState::Error("OpenCode usage 加载通道已断开".to_string());
                self.usage_rx = None;
                true
            }
        }
    }

    fn delayed_fetch_ready(counter: &mut Option<u32>) -> bool {
        match counter {
            Some(0) => {
                *counter = None;
                true
            }
            Some(value) => {
                *value = value.saturating_sub(1);
                false
            }
            None => false,
        }
    }
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
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::PageUp | KeyCode::Char('h') => self.prev_page(),
            KeyCode::PageDown | KeyCode::Char('l') => self.next_page(),
            KeyCode::Enter => {
                if self.switch_selected_account()? {
                    return Ok(true);
                }
            }
            KeyCode::Char('s') => match self.login_state {
                OpenCodeLoginState::LoggedInUnsaved => {
                    self.overlay = Some(Overlay::save_input());
                }
                OpenCodeLoginState::LoggedInSaved(_) => {
                    self.toasts.push(Toast::warning("当前登录已保存"));
                }
                OpenCodeLoginState::NotLoggedIn => {
                    self.toasts
                        .push(Toast::warning("当前未检测到 OpenCode openai 登录"));
                }
            },
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some(account) = self.selected_account() {
                    if account.is_virtual {
                        self.toasts.push(Toast::warning("未保存的当前登录无法删除"));
                    } else {
                        self.overlay = Some(Overlay::confirm_delete(account.name.clone()));
                    }
                }
            }
            KeyCode::Char('r') => {
                self.reload_accounts()?;
                self.refresh_usage();
                self.toasts.push(Toast::info("已刷新 OpenCode 账号列表"));
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_overlay_key(&mut self, key: KeyEvent) -> Result<bool> {
        let is_confirm = matches!(self.overlay, Some(Overlay::Confirm { .. }));
        if is_confirm {
            self.handle_confirm_key(key)
        } else {
            self.handle_input_key(key)
        }
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                let subject = match &self.overlay {
                    Some(Overlay::Confirm { subject, .. }) => subject.clone(),
                    _ => return Ok(false),
                };

                match self.service.delete_account(&subject) {
                    Ok(()) => {
                        self.last_action =
                            Some(("已删除".to_string(), subject.clone(), true, None));
                        self.toasts
                            .push(Toast::success(format!("已删除 OpenCode 账号: {subject}")));
                        self.reload_accounts()?;
                    }
                    Err(e) => {
                        self.last_action = Some((
                            "删除失败".to_string(),
                            subject.clone(),
                            false,
                            Some(e.to_string()),
                        ));
                        self.toasts.push(Toast::error(format!("删除失败: {e}")));
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

    fn handle_input_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.overlay = None;
                self.toasts.push(Toast::info("已取消保存"));
            }
            KeyCode::Enter => {
                let mut overlay = match self.overlay.take() {
                    Some(overlay) => overlay,
                    None => return Ok(false),
                };
                let name = overlay.take_input();

                if name.trim().is_empty() {
                    self.overlay = Some(Overlay::save_input());
                    self.toasts.push(Toast::warning("账号名称不能为空"));
                    return Ok(false);
                }

                match self.service.save_current(name.trim(), false) {
                    Ok(()) => {
                        let saved_name = name.trim().to_string();
                        self.last_action =
                            Some(("已保存为".to_string(), saved_name.clone(), true, None));
                        self.toasts.push(Toast::success(format!(
                            "已保存 OpenCode 账号: {saved_name}"
                        )));
                        self.reload_accounts()?;
                    }
                    Err(e) => {
                        self.last_action = Some((
                            "保存失败".to_string(),
                            name.trim().to_string(),
                            false,
                            Some(e.to_string()),
                        ));
                        self.toasts.push(Toast::error(format!("保存失败: {e}")));
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(overlay) = &mut self.overlay {
                    overlay.pop_char();
                }
            }
            KeyCode::Char(c) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && let Some(overlay) = &mut self.overlay
                {
                    overlay.push_char(c);
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if self.current_page > 0 {
            self.current_page -= 1;
            self.selected_index = self.current_page_accounts().len().saturating_sub(1);
        }
    }

    fn move_down(&mut self) {
        let page_accounts = self.current_page_accounts();
        if self.selected_index + 1 < page_accounts.len() {
            self.selected_index += 1;
        } else if self.current_page + 1 < self.total_pages() {
            self.current_page += 1;
            self.selected_index = 0;
        }
    }

    fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.selected_index = 0;
        }
    }

    fn next_page(&mut self) {
        if self.current_page + 1 < self.total_pages() {
            self.current_page += 1;
            self.selected_index = 0;
        }
    }

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

            if OpenCodeAuthService::is_expired(account.expires_at) {
                self.toasts
                    .push(Toast::warning("账号已过期，请重新登录 OpenCode"));
                return Ok(false);
            }

            match self.service.switch_account(&account.name) {
                Ok(()) => {
                    self.last_action =
                        Some(("已切换到".to_string(), account.name.clone(), true, None));
                    self.toasts.push(Toast::success(format!(
                        "已切换到 OpenCode 账号: {}",
                        account.name
                    )));
                    self.should_quit = true;
                    return Ok(true);
                }
                Err(e) => {
                    self.last_action = Some((
                        "切换失败".to_string(),
                        account.name.clone(),
                        false,
                        Some(e.to_string()),
                    ));
                    self.toasts.push(Toast::error(format!("切换失败: {e}")));
                }
            }
        }

        Ok(false)
    }
}

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

impl TuiApp for OpenCodeAuthApp {
    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if self.overlay.is_some() {
            return self.handle_overlay_key(key);
        }
        self.handle_normal_mode(key)
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<bool> {
        if self.overlay.is_some() {
            return Ok(false);
        }

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(area) = self.list_area.get()
                    && let Some(index) =
                        account_list_hit_test(area, mouse.row, self.current_page_accounts().len())
                {
                    self.selected_index = index;
                }
            }
            MouseEventKind::ScrollUp => self.move_up(),
            MouseEventKind::ScrollDown => self.move_down(),
            _ => {}
        }

        Ok(false)
    }

    fn on_tick(&mut self) -> bool {
        let mut needs_redraw = self.toasts.tick();
        needs_redraw |= self.poll_usage_result();

        if Self::delayed_fetch_ready(&mut self.delayed_usage_ticks) {
            self.start_usage_fetch();
            needs_redraw = true;
        }

        needs_redraw
    }

    fn render(&self, frame: &mut Frame) {
        super::ui::draw(frame, self);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use chrono::{Duration, Utc};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use serde_json::{Map as JsonMap, json};
    use tempfile::tempdir;

    fn create_test_service() -> (OpenCodeAuthService, tempfile::TempDir, tempfile::TempDir) {
        let ccr = tempdir().unwrap();
        let opencode = tempdir().unwrap();
        (
            OpenCodeAuthService::from_dirs(
                ccr.path().join("platforms").join("opencode"),
                opencode.path().to_path_buf(),
            ),
            ccr,
            opencode,
        )
    }

    fn fake_access_token(email: &str, account_id: &str, plan: &str) -> String {
        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
        let payload = URL_SAFE_NO_PAD.encode(
            json!({
                "email": email,
                "chatgpt_account_id": account_id,
                "chatgpt_plan_type": plan
            })
            .to_string(),
        );
        format!("{header}.{payload}.signature")
    }

    fn write_auth_json(
        auth_json_path: &std::path::Path,
        account_id: &str,
        email: &str,
        plan: &str,
    ) {
        let expires = (Utc::now() + Duration::days(3)).timestamp_millis();
        let mut root = JsonMap::new();
        root.insert(
            "openai".to_string(),
            json!({
                "type": "oauth",
                "access": fake_access_token(email, account_id, plan),
                "refresh": format!("rt_{account_id}"),
                "expires": expires,
                "accountId": account_id
            }),
        );
        let content = serde_json::to_string_pretty(&root).unwrap();
        if let Some(parent) = auth_json_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(auth_json_path, content).unwrap();
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn save_overlay_persists_current_login() {
        let (service, ccr, opencode) = create_test_service();
        write_auth_json(
            &opencode.path().join("auth.json"),
            "acc-1",
            "user@example.com",
            "plus",
        );

        let mut app = OpenCodeAuthApp::from_service(OpenCodeAuthService::from_dirs(
            ccr.path().join("platforms").join("opencode"),
            opencode.path().to_path_buf(),
        ))
        .unwrap();
        app.overlay = Some(Overlay::save_input());

        app.handle_key(key(KeyCode::Char('w'))).unwrap();
        app.handle_key(key(KeyCode::Char('o'))).unwrap();
        app.handle_key(key(KeyCode::Char('r'))).unwrap();
        app.handle_key(key(KeyCode::Char('k'))).unwrap();
        app.handle_key(key(KeyCode::Enter)).unwrap();

        let registry = service.load_registry().unwrap();
        assert!(registry.accounts.contains_key("work"));
        assert_eq!(
            app.login_state,
            OpenCodeLoginState::LoggedInSaved("work".to_string())
        );
    }

    #[test]
    fn confirm_delete_removes_saved_account() {
        let (service, ccr, opencode) = create_test_service();
        write_auth_json(
            &opencode.path().join("auth.json"),
            "acc-1",
            "user@example.com",
            "plus",
        );
        service.save_current("work", false).unwrap();

        let mut app = OpenCodeAuthApp::from_service(OpenCodeAuthService::from_dirs(
            ccr.path().join("platforms").join("opencode"),
            opencode.path().to_path_buf(),
        ))
        .unwrap();
        app.overlay = Some(Overlay::confirm_delete("work"));
        app.handle_key(key(KeyCode::Char('y'))).unwrap();

        let registry = service.load_registry().unwrap();
        assert!(!registry.accounts.contains_key("work"));
    }
}
