// Claude Auth TUI application state machine
// 管理 Claude 官方订阅账号快照的保存 / 切换 / 删除

use crate::tui::CompletedAction;
use crate::tui::overlay::Overlay;
use crate::tui::pagination::{
    DEFAULT_PAGE_SIZE, index_in_page, page_for_index, page_slice, total_pages,
};
use crate::tui::runtime::TuiApp;
use crate::tui::toast::{Toast, ToastManager};
use ccr_cli::application::profile_off_for_platform;
use ccr_cli::models::{
    ClaudeAuthActionOutcome, ClaudeAuthRegistry, ClaudeAuthSourceObservation,
    ClaudeCurrentAuthInfo, ClaudeLoginState, ClaudeRuntimeMode, ClaudeRuntimeSummary, Platform,
};
use ccr_cli::services::{ClaudeAuthItem, ClaudeAuthService};
use ccr_core::core::error::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::Frame;
use ratatui::layout::Rect;
use std::cell::Cell;
use std::time::{Duration, Instant};

const AUTH_SNAPSHOT_CACHE_TTL: Duration = Duration::from_secs(5);

pub type ClaudeAuthActionRecord = (CompletedAction, String, bool, Option<String>, Vec<String>);

/// Claude Auth TUI 应用
pub struct ClaudeAuthApp {
    /// 已保存账号列表
    pub accounts: Vec<ClaudeAuthItem>,
    /// 当前页内选中索引
    pub selected_index: usize,
    /// 当前页（0-based）
    pub current_page: usize,
    /// 当前列表可见行数驱动的分页容量
    pub page_size: usize,
    /// 活动中的弹窗
    pub overlay: Option<Overlay>,
    /// Toast 管理器
    pub toasts: ToastManager,
    /// 是否退出
    pub should_quit: bool,
    /// 当前登录状态
    pub login_state: ClaudeLoginState,
    /// 当前官方登录信息
    pub current_info: Option<ClaudeCurrentAuthInfo>,
    /// 当前运行时摘要
    pub runtime_summary: Option<ClaudeRuntimeSummary>,
    /// 注册表快照
    pub auth_registry: ClaudeAuthRegistry,
    /// 服务实例
    service: ClaudeAuthService,
    /// 最近一次操作 (action, account_name, success, error)
    pub last_action: Option<ClaudeAuthActionRecord>,
    /// 账号列表区域（鼠标命中测试）
    pub list_area: Cell<Option<Rect>>,
    /// 最近一次从磁盘刷新快照的时间
    last_refresh_at: Option<Instant>,
}

impl ClaudeAuthApp {
    /// 创建应用实例
    pub fn new() -> Result<Self> {
        Self::from_service(ClaudeAuthService::new()?)
    }

    pub(crate) fn from_service(service: ClaudeAuthService) -> Result<Self> {
        let snapshot = service.read_auth_snapshot()?;
        let runtime_summary = service.get_runtime_summary().ok();
        let accounts = service.build_account_items(&snapshot, runtime_summary.as_ref())?;
        let preferred = accounts
            .iter()
            .position(|account| account.is_current || account.is_logged_in)
            .unwrap_or(0);
        let current_page = page_for_index(preferred, DEFAULT_PAGE_SIZE);
        let page_len = page_slice(&accounts, current_page, DEFAULT_PAGE_SIZE).len();
        let selected_index = if page_len == 0 {
            0
        } else {
            index_in_page(preferred, DEFAULT_PAGE_SIZE)
        };

        let login_state = runtime_summary
            .as_ref()
            .map(|summary| summary.login_state.clone())
            .unwrap_or_else(|| snapshot.login_state.clone());

        Ok(Self {
            accounts,
            selected_index,
            current_page,
            page_size: DEFAULT_PAGE_SIZE,
            overlay: None,
            toasts: ToastManager::new(),
            should_quit: false,
            login_state,
            current_info: snapshot.current_info.clone(),
            runtime_summary,
            auth_registry: snapshot.registry.clone(),
            service,
            last_action: None,
            list_area: Cell::new(None),
            last_refresh_at: Some(Instant::now()),
        })
    }

    fn apply_snapshot(&mut self) -> Result<()> {
        let snapshot = self.service.read_auth_snapshot()?;
        self.current_info = snapshot.current_info.clone();
        self.auth_registry = snapshot.registry.clone();
        self.runtime_summary = self.service.get_runtime_summary().ok();
        self.accounts = self
            .service
            .build_account_items(&snapshot, self.runtime_summary.as_ref())?;

        let preferred = self
            .accounts
            .iter()
            .position(|account| account.is_current || account.is_logged_in)
            .unwrap_or(0);
        self.current_page = page_for_index(preferred, self.page_size);
        let page_len = self.current_page_accounts().len();
        self.selected_index = if page_len == 0 {
            0
        } else {
            index_in_page(preferred, self.page_size)
        };

        self.login_state = self
            .runtime_summary
            .as_ref()
            .map(|summary| summary.login_state.clone())
            .unwrap_or(snapshot.login_state);
        self.last_refresh_at = Some(Instant::now());

        Ok(())
    }

    /// 刷新账号与运行时状态
    pub fn reload_accounts(&mut self) -> Result<()> {
        self.apply_snapshot()
    }

    fn reload_accounts_if_stale(&mut self) -> Result<bool> {
        if !Self::should_refresh(
            self.last_refresh_at,
            AUTH_SNAPSHOT_CACHE_TTL,
            Instant::now(),
        ) {
            return Ok(false);
        }

        self.apply_snapshot()?;
        Ok(true)
    }

    /// 获取当前页账号
    pub fn current_page_accounts(&self) -> &[ClaudeAuthItem] {
        page_slice(&self.accounts, self.current_page, self.page_size)
    }

    /// 获取总页数
    pub fn total_pages(&self) -> usize {
        total_pages(self.accounts.len(), self.page_size)
    }

    /// 获取当前选中账号
    pub fn selected_account(&self) -> Option<&ClaudeAuthItem> {
        self.current_page_accounts().get(self.selected_index)
    }

    pub fn sync_page_size(&mut self, page_size: usize) {
        let page_size = page_size.max(1);
        if self.page_size == page_size {
            return;
        }

        let selected_name = self.selected_account().map(|account| account.name.clone());
        self.page_size = page_size;

        if let Some(name) = selected_name
            && let Some(index) = self
                .accounts
                .iter()
                .position(|account| account.name == name)
        {
            self.current_page = page_for_index(index, self.page_size);
            self.selected_index = index_in_page(index, self.page_size);
            return;
        }

        let index = (self.current_page * self.page_size + self.selected_index)
            .min(self.accounts.len().saturating_sub(1));
        self.current_page = page_for_index(index, self.page_size);
        self.selected_index = index_in_page(index, self.page_size);
    }

    /// 当页签被激活时刷新本地状态
    pub fn on_activated(&mut self) {
        if let Err(err) = self.reload_accounts_if_stale() {
            self.toasts.push(Toast::error(crate::tui_format!(
                "Failed to refresh Claude official account state: {err}",
                "刷新 Claude 官方账号状态失败：{err}"
            )));
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
            KeyCode::Enter if self.switch_selected_account()? => {
                return Ok(true);
            }
            KeyCode::Char('s') => {
                if self.current_info.is_some() {
                    self.overlay = Some(Overlay::save_input());
                } else {
                    self.toasts.push(Toast::warning(crate::tui_text!(
                        "No Claude official login detected; run `claude login` first",
                        "未检测到 Claude 官方登录，请先运行 `claude login`"
                    )));
                }
            }
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some(account) = self.selected_account() {
                    self.overlay = Some(Overlay::confirm_delete(account.name.clone()));
                } else {
                    self.toasts.push(Toast::warning(crate::tui_text!(
                        "No saved account is available to delete",
                        "没有可删除的已保存账号"
                    )));
                }
            }
            KeyCode::Char('r') => {
                self.reload_accounts()?;
                self.toasts.push(Toast::info(crate::tui_text!(
                    "Claude official account list reloaded",
                    "已刷新 Claude 官方账号列表"
                )));
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
                        self.last_action = Some((
                            CompletedAction::Delete,
                            subject.clone(),
                            true,
                            None,
                            Vec::new(),
                        ));
                        self.toasts.push(Toast::success(crate::tui_format!(
                            "Deleted Claude account snapshot: {subject}",
                            "已删除 Claude 账号快照：{subject}"
                        )));
                        self.reload_accounts()?;
                    }
                    Err(err) => {
                        self.last_action = Some((
                            CompletedAction::Delete,
                            subject.clone(),
                            false,
                            Some(err.to_string()),
                            Vec::new(),
                        ));
                        self.toasts.push(Toast::error(crate::tui_format!(
                            "Delete failed: {err}",
                            "删除失败：{err}"
                        )));
                    }
                }
                self.overlay = None;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.overlay = None;
                self.toasts.push(Toast::info(crate::tui_text!(
                    "Deletion cancelled",
                    "已取消删除"
                )));
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.overlay = None;
                self.toasts.push(Toast::info(crate::tui_text!(
                    "Save cancelled",
                    "已取消保存"
                )));
            }
            KeyCode::Enter => {
                let mut overlay = match self.overlay.take() {
                    Some(overlay) => overlay,
                    None => return Ok(false),
                };
                let name = overlay.take_input();
                let name = name.trim().to_string();

                if name.is_empty() {
                    self.overlay = Some(Overlay::save_input());
                    self.toasts.push(Toast::warning(crate::tui_text!(
                        "Account name cannot be empty",
                        "账号名称不能为空"
                    )));
                    return Ok(false);
                }

                match self.service.save_current(&name, None, false) {
                    Ok(_) => {
                        self.last_action =
                            Some((CompletedAction::Save, name.clone(), true, None, Vec::new()));
                        self.toasts.push(Toast::success(crate::tui_format!(
                            "Saved Claude official account: {name}",
                            "已保存 Claude 官方账号：{name}"
                        )));
                        self.reload_accounts()?;
                    }
                    Err(err) => {
                        self.last_action = Some((
                            CompletedAction::Save,
                            name.clone(),
                            false,
                            Some(err.to_string()),
                            Vec::new(),
                        ));
                        self.toasts.push(Toast::error(crate::tui_format!(
                            "Save failed: {err}",
                            "保存失败：{err}"
                        )));
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
        self.selected_index = super::super::selection::previous_index_in_page(
            self.current_page_accounts().len(),
            self.selected_index,
        );
    }

    fn move_down(&mut self) {
        self.selected_index = super::super::selection::next_index_in_page(
            self.current_page_accounts().len(),
            self.selected_index,
        );
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
        let Some(account) = self.selected_account().cloned() else {
            self.toasts.push(Toast::warning(crate::tui_text!(
                "No saved account is available to switch to",
                "没有可切换的已保存账号"
            )));
            return Ok(false);
        };

        if account.is_current {
            self.toasts.push(Toast::info(crate::tui_text!(
                "This official account is already active",
                "已经是当前生效的官方账号"
            )));
            return Ok(false);
        }

        let api_key_override_active = self
            .runtime_summary
            .as_ref()
            .is_some_and(|summary| matches!(summary.login_state, ClaudeLoginState::ApiKeyActive));

        if account.is_logged_in && !api_key_override_active {
            let message = match self.runtime_summary.as_ref() {
                Some(summary) if matches!(summary.mode, ClaudeRuntimeMode::ProfilePendingAuth) => {
                    crate::tui_text!(
                        "The selected official account is already logged in, but subscription credentials are temporarily unavailable",
                        "当前登录就是该官方账号，但订阅凭据暂不可用"
                    )
                    .to_string()
                }
                _ => crate::tui_text!(
                    "The selected official account is already logged in",
                    "当前登录已经是该官方账号"
                )
                .to_string(),
            };
            self.toasts.push(Toast::info(message));
            return Ok(false);
        }

        if let Err(err) = profile_off_for_platform(Platform::Claude) {
            self.toasts.push(Toast::error(crate::tui_format!(
                "Exit profile failed: {}",
                "退出 Profile 失败：{}",
                err
            )));
            return Ok(false);
        }

        match self.service.switch_account(&account.name) {
            Ok(outcome) => {
                let warnings = format_action_warnings(&outcome);
                self.last_action = Some((
                    CompletedAction::Switch,
                    account.name.clone(),
                    true,
                    None,
                    warnings.clone(),
                ));
                let cleared_count = outcome.cleared_managed_sources.len();
                let success_message = if cleared_count > 0 {
                    crate::tui_format!(
                        "Switched Claude official account: {}; cleared {cleared_count} CCR-managed setting(s)",
                        "已切换 Claude 官方账号：{}；已清理 {cleared_count} 个 CCR 托管设置",
                        account.name
                    )
                } else {
                    crate::tui_format!(
                        "Switched Claude official account: {}",
                        "已切换 Claude 官方账号：{}",
                        account.name
                    )
                };
                self.toasts.push(Toast::success(success_message));
                self.reload_accounts()?;
                if !warnings.is_empty() {
                    self.toasts.push(Toast::warning(crate::tui_format!(
                        "{} auth warning(s) remain after the switch",
                        "切换后仍有 {} 条认证警告",
                        warnings.len()
                    )));
                }
                self.should_quit = true;
                Ok(true)
            }
            Err(err) => {
                self.last_action = Some((
                    CompletedAction::Switch,
                    account.name.clone(),
                    false,
                    Some(err.to_string()),
                    Vec::new(),
                ));
                self.toasts.push(Toast::error(crate::tui_format!(
                    "Switch failed: {err}",
                    "切换失败：{err}"
                )));
                Ok(false)
            }
        }
    }

    fn should_refresh(last_refresh_at: Option<Instant>, ttl: Duration, now: Instant) -> bool {
        last_refresh_at.is_none_or(|last_refresh_at| now.duration_since(last_refresh_at) >= ttl)
    }
}

fn format_auth_source_warning(source: &ClaudeAuthSourceObservation) -> String {
    format!(
        "{} @ {} ({}; {}; {})",
        source.kind.as_str(),
        source.location.as_str(),
        source.confidence.as_str(),
        source.evidence.as_str(),
        source.ownership.as_str()
    )
}

fn format_action_warnings(outcome: &ClaudeAuthActionOutcome) -> Vec<String> {
    if outcome.remaining_suppressors.is_empty() {
        outcome.warnings.clone()
    } else {
        outcome
            .remaining_suppressors
            .iter()
            .map(format_auth_source_warning)
            .collect()
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

impl TuiApp for ClaudeAuthApp {
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
                    return Ok(false);
                }
            }
            MouseEventKind::ScrollUp => self.move_up(),
            MouseEventKind::ScrollDown => self.move_down(),
            _ => {}
        }

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
mod tests {
    use super::*;
    use ccr_cli::models::{
        ClaudeAuthConfidence, ClaudeAuthEvidence, ClaudeAuthOwnership, ClaudeAuthSourceKind,
        ClaudeAuthSourceLocation,
    };
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn should_refresh_when_cache_missing() {
        let now = Instant::now();
        assert!(ClaudeAuthApp::should_refresh(
            None,
            AUTH_SNAPSHOT_CACHE_TTL,
            now
        ));
    }

    #[test]
    fn should_refresh_when_cache_expired() {
        let now = Instant::now();
        let last_refresh_at = now - Duration::from_secs(10);
        assert!(ClaudeAuthApp::should_refresh(
            Some(last_refresh_at),
            AUTH_SNAPSHOT_CACHE_TTL,
            now
        ));
    }

    #[test]
    fn skips_refresh_when_cache_is_still_hot() {
        let now = Instant::now();
        let last_refresh_at = now - Duration::from_secs(1);
        assert!(!ClaudeAuthApp::should_refresh(
            Some(last_refresh_at),
            AUTH_SNAPSHOT_CACHE_TTL,
            now
        ));
    }

    fn navigation_account(index: usize) -> ClaudeAuthItem {
        ClaudeAuthItem {
            name: format!("account-{index:02}"),
            description: None,
            email: None,
            billing_type: None,
            subscription_type: None,
            rate_limit_tier: None,
            is_current: false,
            is_logged_in: false,
            saved_at: Utc::now(),
            last_used: None,
            expires_at: None,
        }
    }

    fn navigation_app(
        accounts: Vec<ClaudeAuthItem>,
        selected_index: usize,
        current_page: usize,
    ) -> ClaudeAuthApp {
        ClaudeAuthApp {
            accounts,
            selected_index,
            current_page,
            page_size: DEFAULT_PAGE_SIZE,
            overlay: None,
            toasts: ToastManager::new(),
            should_quit: false,
            login_state: ClaudeLoginState::NotLoggedIn,
            current_info: None,
            runtime_summary: None,
            auth_registry: ClaudeAuthRegistry::default(),
            service: ClaudeAuthService::from_parts(
                PathBuf::from("."),
                PathBuf::from("."),
                PathBuf::from(".claude.json"),
            ),
            last_action: None,
            list_area: Cell::new(None),
            last_refresh_at: None,
        }
    }

    #[test]
    fn navigation_wrap_claude_auth_stays_on_current_page() {
        let accounts = (1..=12).map(navigation_account).collect();
        let mut app = navigation_app(accounts, 0, 1);

        app.move_up();

        assert_eq!(app.current_page, 1);
        assert_eq!(app.selected_index, 1);

        app.move_down();

        assert_eq!(app.current_page, 1);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn current_page_accounts_respects_dynamic_page_size() {
        let accounts = (1..=20).map(navigation_account).collect();
        let mut app = navigation_app(accounts, 0, 0);

        app.sync_page_size(14);

        assert_eq!(app.current_page_accounts().len(), 14);
        assert_eq!(app.total_pages(), 2);

        app.sync_page_size(25);

        assert_eq!(app.current_page_accounts().len(), 20);
        assert_eq!(app.total_pages(), 1);
    }

    #[test]
    fn page_size_growth_preserves_selected_account_identity() {
        let accounts = (1..=28).map(navigation_account).collect();
        let mut app = navigation_app(accounts, 2, 1);

        assert_eq!(
            app.selected_account().map(|account| account.name.as_str()),
            Some("account-13")
        );

        app.sync_page_size(20);

        assert_eq!(app.page_size, 20);
        assert_eq!(app.current_page, 0);
        assert_eq!(app.selected_index, 12);
        assert_eq!(
            app.selected_account().map(|account| account.name.as_str()),
            Some("account-13")
        );
    }

    #[test]
    fn account_list_hit_test_ignores_blank_rows_when_page_has_fewer_items_than_space() {
        let area = Rect::new(4, 7, 60, 14);
        assert_eq!(account_list_hit_test(area, 15, 5), None);
    }

    #[test]
    fn switch_warning_preserves_source_confidence_evidence_and_ownership() {
        let warning = format_auth_source_warning(&ClaudeAuthSourceObservation {
            kind: ClaudeAuthSourceKind::AnthropicApiKey,
            location: ClaudeAuthSourceLocation::SettingsEnv,
            confidence: ClaudeAuthConfidence::Potential,
            evidence: ClaudeAuthEvidence::OfficialContract,
            ownership: ClaudeAuthOwnership::UserOwned,
            suppresses_subscription: true,
        });

        assert_eq!(
            warning,
            "anthropic_api_key @ settings_env (potential; official_contract; user_owned)"
        );
    }

    #[test]
    fn switch_warning_keeps_diagnosis_failure_fallback() {
        let outcome = ClaudeAuthActionOutcome {
            cleared_managed_sources: Vec::new(),
            remaining_suppressors: Vec::new(),
            warnings: vec!["diagnosis failed safely".to_string()],
        };

        assert_eq!(
            format_action_warnings(&outcome),
            vec!["diagnosis failed safely"]
        );
    }
}
