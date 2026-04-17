// Codex Auth TUI application state machine
// Manages the Codex multi-account selector state

use crate::models::{
    CodexAccountQuota, CodexAuthItem, CodexAuthRegistry, CodexRuntimeSummary, LoginState,
    TokenFreshness,
};
use crate::services::AuthReadSnapshot;
use crate::services::{
    CodexAuthService, CodexQuotaService, CodexRollingUsage, CodexUsageRecord, CodexUsageService,
};
use crate::tui::auth_refresh::{RefreshReason, RefreshSchedulerState, RefreshTask, RefreshTier};
use crate::tui::overlay::Overlay;
use ccr_core::core::error::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use dirs::home_dir;
use ratatui::Frame;
use ratatui::layout::Rect;
use std::cell::Cell;
use std::sync::mpsc::TryRecvError;

use crate::tui::runtime::TuiApp;
use crate::tui::toast::{Toast, ToastManager};
use indexmap::IndexMap;
use std::path::PathBuf;

/// Maximum accounts per page
pub const PAGE_SIZE: usize = 10;
const QUOTA_REFRESH_INTERVAL_TICKS: u32 = 4;
const CURRENT_RUNTIME_ACCOUNT_KEY: &str = "default";

#[derive(Debug, Clone)]
pub struct CodexUsageDataset {
    pub records: Vec<CodexUsageRecord>,
    pub global: CodexRollingUsage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodexUsageScope {
    GlobalRuntime,
    AccountAttributed { account_name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodexUsageAttributionState {
    GlobalOnly,
    AccountAttributed,
    VirtualAccount,
    UnattributedFallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexUsageTopModel {
    pub model: String,
    pub total_tokens: u64,
    pub total_requests: u64,
}

#[derive(Debug, Clone)]
pub struct CodexAuthUsagePanelData {
    pub scope: CodexUsageScope,
    pub attribution_state: CodexUsageAttributionState,
    pub rolling: CodexRollingUsage,
    pub top_model: Option<CodexUsageTopModel>,
    pub fallback_reason: Option<String>,
}

/// Usage data state
#[derive(Debug, Clone)]
pub enum UsageState {
    /// Loading
    #[allow(dead_code)]
    Loading,
    /// Loaded successfully
    Loaded(CodexUsageDataset),
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
    Loading {
        account_name: String,
        cache: IndexMap<String, CodexAccountQuota>,
    },
    /// 已加载
    Loaded {
        cache: IndexMap<String, CodexAccountQuota>,
    },
    /// 查询失败
    Error {
        account_name: String,
        message: String,
        cache: IndexMap<String, CodexAccountQuota>,
    },
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
    /// Auth registry snapshot (contains CCR usage attribution ledger)
    pub auth_registry: CodexAuthRegistry,
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
        Option<std::sync::mpsc::Receiver<std::result::Result<CodexAccountQuota, (String, String)>>>,
    /// Usage async result receiver
    usage_rx: Option<std::sync::mpsc::Receiver<UsageState>>,
    /// Codex directory
    #[allow(dead_code)]
    codex_dir: Option<PathBuf>,
    /// 🖱️ Cached account list area for mouse hit-testing
    pub list_area: Cell<Option<Rect>>,
    /// 账号级 quota 刷新调度器
    quota_refresh: RefreshSchedulerState<String>,
    /// Whether a quota refresh confirmation is pending
    pub pending_quota_confirm: bool,
}

#[allow(dead_code)]
impl CodexAuthApp {
    /// Create a new application instance
    pub fn new() -> Result<Self> {
        Self::from_service(CodexAuthService::new()?)
    }

    pub(crate) fn from_service(service: CodexAuthService) -> Result<Self> {
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
            auth_registry: snapshot.registry.clone(),
            runtime_summary: service.get_runtime_summary().ok(),
            service,
            last_action: None,
            usage_state: UsageState::Loading,
            quota_state: QuotaState::Idle,
            quota_rx: None,
            usage_rx: None,
            codex_dir,
            list_area: Cell::new(None),
            quota_refresh: RefreshSchedulerState::new(QUOTA_REFRESH_INTERVAL_TICKS),
            pending_quota_confirm: false,
        })
    }

    fn apply_snapshot(&mut self, snapshot: AuthReadSnapshot) -> Result<()> {
        self.login_state = snapshot.login_state.clone();
        self.auth_registry = snapshot.registry.clone();
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

        let usage_service = CodexUsageService::new(dir.clone());

        match usage_service.parse_all_logs() {
            Ok(records) => {
                if records.is_empty() {
                    UsageState::NoData
                } else {
                    UsageState::Loaded(CodexUsageDataset {
                        global: CodexUsageService::compute_rolling_usage_for_records(&records),
                        records,
                    })
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

    pub fn selected_quota(&self) -> Option<&CodexAccountQuota> {
        let selected_name = self.selected_account()?.name.as_str();
        self.quota_cache().get(selected_name)
    }

    pub fn is_selected_quota_loading(&self) -> bool {
        match &self.quota_state {
            QuotaState::Loading { account_name, .. } => self
                .selected_account()
                .is_some_and(|account| account.name == *account_name),
            _ => false,
        }
    }

    pub fn selected_quota_error(&self) -> Option<&str> {
        match &self.quota_state {
            QuotaState::Error {
                account_name,
                message,
                ..
            } if self
                .selected_account()
                .is_some_and(|account| account.name == *account_name) =>
            {
                Some(message.as_str())
            }
            _ => None,
        }
    }

    fn quota_cache(&self) -> &IndexMap<String, CodexAccountQuota> {
        match &self.quota_state {
            QuotaState::Idle => {
                static EMPTY: std::sync::OnceLock<IndexMap<String, CodexAccountQuota>> =
                    std::sync::OnceLock::new();
                EMPTY.get_or_init(IndexMap::new)
            }
            QuotaState::Loading { cache, .. }
            | QuotaState::Loaded { cache }
            | QuotaState::Error { cache, .. } => cache,
        }
    }

    pub fn usage_panel_data(&self) -> Option<CodexAuthUsagePanelData> {
        let UsageState::Loaded(dataset) = &self.usage_state else {
            return None;
        };

        Some(Self::build_usage_panel_data(
            dataset,
            self.selected_account(),
            &self.auth_registry,
        ))
    }

    fn build_usage_panel_data(
        dataset: &CodexUsageDataset,
        selected_account: Option<&CodexAuthItem>,
        registry: &CodexAuthRegistry,
    ) -> CodexAuthUsagePanelData {
        let global_top_model = Self::top_model_from_usage(&dataset.global);

        let Some(selected_account) = selected_account else {
            return CodexAuthUsagePanelData {
                scope: CodexUsageScope::GlobalRuntime,
                attribution_state: CodexUsageAttributionState::GlobalOnly,
                rolling: dataset.global.clone(),
                top_model: global_top_model,
                fallback_reason: None,
            };
        };

        if selected_account.is_virtual {
            return CodexAuthUsagePanelData {
                scope: CodexUsageScope::GlobalRuntime,
                attribution_state: CodexUsageAttributionState::VirtualAccount,
                rolling: dataset.global.clone(),
                top_model: global_top_model,
                fallback_reason: Some(
                    "当前登录尚未保存为 CCR 账号，本地 usage 暂时只能按全局 runtime 展示"
                        .to_string(),
                ),
            };
        }

        let Some(account_id) = registry
            .accounts
            .get(&selected_account.name)
            .map(|account| account.account_id.as_str())
        else {
            return CodexAuthUsagePanelData {
                scope: CodexUsageScope::GlobalRuntime,
                attribution_state: CodexUsageAttributionState::UnattributedFallback,
                rolling: dataset.global.clone(),
                top_model: global_top_model,
                fallback_reason: Some(
                    "CCR 未找到该账号的归因元数据，以下为全局 runtime 统计".to_string(),
                ),
            };
        };

        let attributed_records =
            Self::records_for_account(&dataset.records, account_id, &registry.usage_ledger);

        if attributed_records.is_empty() {
            return CodexAuthUsagePanelData {
                scope: CodexUsageScope::GlobalRuntime,
                attribution_state: CodexUsageAttributionState::UnattributedFallback,
                rolling: dataset.global.clone(),
                top_model: global_top_model,
                fallback_reason: Some(
                    "本地 Codex 日志不包含账号字段；该账号尚无可置信的 CCR 归因记录，以下为全局 runtime 统计"
                        .to_string(),
                ),
            };
        }

        let rolling = CodexUsageService::compute_rolling_usage_for_records(&attributed_records);
        let top_model = Self::top_model_from_usage(&rolling);

        CodexAuthUsagePanelData {
            scope: CodexUsageScope::AccountAttributed {
                account_name: selected_account.name.clone(),
            },
            attribution_state: CodexUsageAttributionState::AccountAttributed,
            rolling,
            top_model,
            fallback_reason: if dataset.global.all_time.total_requests
                > attributed_records.len() as u64
            {
                Some(
                    "仅展示已归因到该账号的本地记录；更早或未归因历史不会混入该账号统计"
                        .to_string(),
                )
            } else {
                None
            },
        }
    }

    fn top_model_from_usage(usage: &CodexRollingUsage) -> Option<CodexUsageTopModel> {
        usage
            .by_model
            .iter()
            .max_by_key(|(_, stats)| stats.total_input_tokens + stats.total_output_tokens)
            .map(|(model, stats)| CodexUsageTopModel {
                model: model.clone(),
                total_tokens: stats.total_input_tokens + stats.total_output_tokens,
                total_requests: stats.total_requests,
            })
    }

    fn records_for_account(
        records: &[CodexUsageRecord],
        account_id: &str,
        ledger: &[crate::models::CodexUsageActivation],
    ) -> Vec<CodexUsageRecord> {
        if ledger.is_empty() {
            return Vec::new();
        }

        let mut sorted_ledger = ledger.to_vec();
        sorted_ledger.sort_by_key(|entry| entry.started_at);

        records
            .iter()
            .filter(|record| {
                let Some((index, entry)) = sorted_ledger
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, entry)| record.timestamp >= entry.started_at)
                else {
                    return false;
                };

                if entry.account_id != account_id {
                    return false;
                }

                let next_started_at = sorted_ledger.get(index + 1).map(|next| next.started_at);
                next_started_at.is_none_or(|next_start| record.timestamp < next_start)
            })
            .cloned()
            .collect()
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
                    self.queue_selected_quota_refresh(
                        RefreshTier::Current,
                        RefreshReason::ManualRefresh,
                        true,
                    );
                    self.try_start_quota_refresh(true);
                    let label = self
                        .selected_account()
                        .map(|account| account.name.as_str())
                        .unwrap_or("当前账号");
                    self.toasts
                        .push(Toast::info(format!("正在查询账号配额: {}", label)));
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
            KeyCode::Char('n') => {
                if let Some(account) = self.selected_account() {
                    if account.is_virtual {
                        self.toasts.push(Toast::warning("未保存的登录无法重命名"));
                    } else {
                        self.overlay = Some(Overlay::rename_input(account.name.clone()));
                    }
                }
            }
            KeyCode::Char('r') => {
                self.reload_accounts()?;
                self.rebuild_quota_refresh_plan(false);
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
                                    self.queue_selected_quota_refresh(
                                        RefreshTier::Current,
                                        RefreshReason::ManualRefresh,
                                        true,
                                    );
                                    self.try_start_quota_refresh(true);
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
        match &self.overlay {
            Some(Overlay::Confirm { .. }) => self.handle_confirm_key(key),
            Some(Overlay::RenameInput { .. }) => self.handle_rename_input_key(key),
            _ => self.handle_input_key(key),
        }
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

    /// Handle rename input overlay keys
    /// - Enter: 尝试普通重命名（已存在时提示使用 Ctrl+F 强制覆盖）
    /// - Ctrl+F: 强制覆盖同名账号
    /// - Esc: 取消
    fn handle_rename_input_key(&mut self, key: KeyEvent) -> Result<bool> {
        let is_ctrl_f = matches!(key.code, KeyCode::Char('f') | KeyCode::Char('F'))
            && key.modifiers.contains(KeyModifiers::CONTROL);

        match key.code {
            KeyCode::Enter => {
                self.commit_rename(false)?;
            }
            KeyCode::Char(_) if is_ctrl_f => {
                self.commit_rename(true)?;
            }
            KeyCode::Esc => {
                self.overlay = None;
                self.toasts.push(Toast::info("已取消重命名"));
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

    /// Execute rename using the current RenameInput overlay
    fn commit_rename(&mut self, force: bool) -> Result<()> {
        let (source, new_name) = match &mut self.overlay {
            Some(Overlay::RenameInput { source, buffer, .. }) => {
                (source.clone(), std::mem::take(buffer))
            }
            _ => return Ok(()),
        };

        let trimmed = new_name.trim().to_string();
        if trimmed.is_empty() {
            self.toasts.push(Toast::warning("新名称不能为空"));
            if let Some(Overlay::RenameInput { buffer, .. }) = &mut self.overlay {
                *buffer = source.clone();
            }
            return Ok(());
        }

        if trimmed == source {
            self.overlay = None;
            self.toasts.push(Toast::info("名称未变化"));
            return Ok(());
        }

        match self.service.rename_account(&source, &trimmed, force) {
            Ok(_) => {
                self.last_action = Some((
                    "已重命名".to_string(),
                    format!("{} -> {}", source, trimmed),
                    true,
                    None,
                ));
                self.toasts.push(Toast::success(format!(
                    "已重命名: {} -> {}",
                    source, trimmed
                )));
                self.overlay = None;
                self.reload_accounts()?;
            }
            Err(e) => {
                let msg = e.to_string();
                // 把输入还原到 buffer，方便继续编辑
                if let Some(Overlay::RenameInput { buffer, .. }) = &mut self.overlay {
                    *buffer = trimmed.clone();
                }
                if !force && msg.contains("已存在") {
                    self.toasts
                        .push(Toast::warning(format!("{} · 按 Ctrl+F 强制覆盖", msg)));
                } else {
                    self.toasts.push(Toast::error(format!("重命名失败: {}", e)));
                }
            }
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════
    // Navigation helpers
    // ═══════════════════════════════════════════════════════════

    /// Move selection up
    fn move_up(&mut self) {
        let before = self.selected_account().map(|account| account.name.clone());
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if self.current_page > 0 {
            self.current_page -= 1;
            self.selected_index = PAGE_SIZE - 1;
        }
        self.maybe_request_quota_for_selection_change(before);
    }

    /// Move selection down
    fn move_down(&mut self) {
        let before = self.selected_account().map(|account| account.name.clone());
        let page_accounts = self.current_page_accounts();
        if self.selected_index < page_accounts.len().saturating_sub(1) {
            self.selected_index += 1;
        } else if self.current_page < self.total_pages() - 1 {
            self.current_page += 1;
            self.selected_index = 0;
        }
        self.maybe_request_quota_for_selection_change(before);
    }

    /// Previous page
    fn prev_page(&mut self) {
        let before = self.selected_account().map(|account| account.name.clone());
        if self.current_page > 0 {
            self.current_page -= 1;
            self.selected_index = 0;
        }
        self.maybe_request_quota_for_selection_change(before);
    }

    /// Next page
    fn next_page(&mut self) {
        let before = self.selected_account().map(|account| account.name.clone());
        if self.current_page < self.total_pages() - 1 {
            self.current_page += 1;
            self.selected_index = 0;
        }
        self.maybe_request_quota_for_selection_change(before);
    }

    fn maybe_request_quota_for_selection_change(&mut self, previous: Option<String>) {
        let current = self.selected_account().map(|account| account.name.clone());
        if current != previous {
            self.queue_selected_quota_refresh(
                RefreshTier::HoverOnly,
                RefreshReason::SelectionChanged,
                false,
            );
            self.try_start_quota_refresh(false);
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
    pub fn on_activated(&mut self) {
        self.start_usage_fetch();
        self.rebuild_quota_refresh_plan(false);
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

    fn selected_quota_key(&self) -> Option<String> {
        self.selected_account().map(|account| account.name.clone())
    }

    fn warm_quota_account_names(&self) -> Vec<String> {
        let mut candidates: Vec<_> = self
            .accounts
            .iter()
            .filter(|account| {
                !account.is_virtual && !CodexAuthService::is_expired(account.expires_at)
            })
            .map(|account| (account.name.clone(), account.last_used, account.saved_at))
            .collect();

        candidates.sort_by(|left, right| {
            right
                .1
                .cmp(&left.1)
                .then_with(|| right.2.cmp(&left.2))
                .then_with(|| left.0.cmp(&right.0))
        });

        candidates
            .into_iter()
            .take(3)
            .map(|(name, _, _)| name)
            .collect()
    }

    fn rebuild_quota_refresh_plan(&mut self, force_selected_refresh: bool) {
        self.quota_refresh.clear_pending();

        if let Some(selected_key) = self.selected_quota_key() {
            self.quota_refresh.push(RefreshTask::new(
                selected_key.clone(),
                RefreshTier::Current,
                if force_selected_refresh {
                    RefreshReason::ManualRefresh
                } else {
                    RefreshReason::TabActivated
                },
                force_selected_refresh,
            ));
        }

        let selected_key = self.selected_quota_key();
        for account_name in self.warm_quota_account_names() {
            if selected_key.as_deref() == Some(account_name.as_str()) {
                continue;
            }
            self.quota_refresh.push(RefreshTask::new(
                account_name,
                RefreshTier::WarmTop3,
                RefreshReason::TabActivated,
                false,
            ));
        }

        self.quota_refresh
            .set_cooldown(QUOTA_REFRESH_INTERVAL_TICKS);
    }

    fn queue_selected_quota_refresh(
        &mut self,
        tier: RefreshTier,
        reason: RefreshReason,
        force_refresh: bool,
    ) {
        if let Some(selected_key) = self.selected_quota_key() {
            self.quota_refresh
                .push(RefreshTask::new(selected_key, tier, reason, force_refresh));
        }
    }

    fn try_start_quota_refresh(&mut self, bypass_cooldown: bool) -> bool {
        let Some(task) = self.quota_refresh.next_ready(bypass_cooldown) else {
            return false;
        };

        if !self.start_quota_fetch_for_key(task.key.clone(), task.force_refresh) {
            return false;
        }

        self.quota_refresh.mark_dispatched(&task);
        true
    }

    /// Start async quota fetch in background thread
    fn start_quota_fetch_for_key(&mut self, account_key: String, force_refresh: bool) -> bool {
        if self.quota_rx.is_some() {
            return false;
        }

        let cache = self.quota_cache().clone();
        self.quota_state = QuotaState::Loading {
            account_name: account_key.clone(),
            cache,
        };
        let (tx, rx) = std::sync::mpsc::channel();
        self.quota_rx = Some(rx);

        // 在后台线程中执行异步配额查询
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx.send(Err((account_key.clone(), format!("创建运行时失败: {}", e))));
                    return;
                }
            };
            rt.block_on(async {
                match CodexQuotaService::new() {
                    Ok(service) => {
                        let quota = if account_key == CURRENT_RUNTIME_ACCOUNT_KEY {
                            if force_refresh {
                                service.fetch_current_quota_force_refresh().await
                            } else {
                                service.fetch_current_quota().await
                            }
                        } else {
                            if force_refresh {
                                service
                                    .fetch_account_quota_force_refresh(&account_key)
                                    .await
                            } else {
                                service.fetch_account_quota(&account_key).await
                            }
                        };
                        let _ = tx.send(Ok(quota));
                    }
                    Err(e) => {
                        let _ = tx.send(Err((account_key, format!("初始化配额服务失败: {}", e))));
                    }
                }
            });
        });

        true
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
            Ok(Ok(quota)) => {
                let mut cache = self.quota_cache().clone();
                let account_name = quota.account_name.clone();
                cache.insert(account_name.clone(), quota);
                self.quota_state = QuotaState::Loaded { cache };
                self.quota_refresh.finish(&account_name);
                self.quota_rx = None;
                true
            }
            Ok(Err((account_name, message))) => {
                let cache = self.quota_cache().clone();
                self.quota_state = QuotaState::Error {
                    account_name,
                    message,
                    cache,
                };
                if let QuotaState::Error { account_name, .. } = &self.quota_state {
                    self.quota_refresh.finish(account_name);
                }
                self.quota_rx = None;
                true
            }
            Err(TryRecvError::Empty) => false,
            Err(TryRecvError::Disconnected) => {
                let account_name = self
                    .quota_refresh
                    .in_flight_key()
                    .cloned()
                    .or_else(|| self.selected_account().map(|account| account.name.clone()))
                    .unwrap_or_else(|| "当前账号".to_string());
                let cache = self.quota_cache().clone();
                self.quota_state = QuotaState::Error {
                    account_name: account_name.clone(),
                    message: "配额查询通道已断开".to_string(),
                    cache,
                };
                self.quota_refresh.finish(&account_name);
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
                    let before = self.selected_account().map(|account| account.name.clone());
                    self.selected_index = idx;
                    self.maybe_request_quota_for_selection_change(before);
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

        needs_redraw |= self.poll_usage_result();
        needs_redraw |= self.poll_quota_result();
        self.quota_refresh.tick();

        if self.try_start_quota_refresh(false) {
            needs_redraw = true;
        }

        needs_redraw
    }

    fn render(&self, frame: &mut Frame) {
        super::ui::draw(frame, self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};

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

    fn sample_saved_account(name: &str, _account_id: &str) -> CodexAuthItem {
        CodexAuthItem {
            name: name.to_string(),
            description: None,
            email: None,
            plan_type: None,
            is_current: true,
            is_virtual: false,
            saved_at: Some(Utc.with_ymd_and_hms(2026, 4, 13, 10, 0, 0).unwrap()),
            last_used: None,
            last_refresh: None,
            freshness: TokenFreshness::Fresh,
            expires_at: None,
        }
    }

    fn registry_with_account(name: &str, account_id: &str) -> CodexAuthRegistry {
        let mut registry = CodexAuthRegistry {
            current_auth: Some(name.to_string()),
            ..Default::default()
        };
        registry.accounts.insert(
            name.to_string(),
            crate::models::CodexAuthAccount {
                description: None,
                account_id: account_id.to_string(),
                auth_method: None,
                api_base_url: None,
                api_provider_name: None,
                email: None,
                plan_type: None,
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: None,
            },
        );
        registry
    }

    #[test]
    fn records_for_account_only_keeps_records_inside_matching_activation_window() {
        let mut registry = registry_with_account("qq_pro", "acc-qq");
        registry.record_usage_activation(
            "qq_pro",
            "acc-qq",
            Utc.with_ymd_and_hms(2026, 4, 13, 10, 0, 0).unwrap(),
        );
        registry.record_usage_activation(
            "other",
            "acc-other",
            Utc.with_ymd_and_hms(2026, 4, 13, 12, 0, 0).unwrap(),
        );

        let records = vec![
            crate::services::CodexUsageRecord {
                session_id: "before".to_string(),
                timestamp: Utc.with_ymd_and_hms(2026, 4, 13, 9, 30, 0).unwrap(),
                input_tokens: 10,
                output_tokens: 5,
                model: Some("gpt-5.4".to_string()),
            },
            crate::services::CodexUsageRecord {
                session_id: "during".to_string(),
                timestamp: Utc.with_ymd_and_hms(2026, 4, 13, 11, 0, 0).unwrap(),
                input_tokens: 20,
                output_tokens: 10,
                model: Some("gpt-5.4".to_string()),
            },
            crate::services::CodexUsageRecord {
                session_id: "after".to_string(),
                timestamp: Utc.with_ymd_and_hms(2026, 4, 13, 12, 30, 0).unwrap(),
                input_tokens: 30,
                output_tokens: 15,
                model: Some("gpt-5.4-mini".to_string()),
            },
        ];

        let filtered =
            CodexAuthApp::records_for_account(&records, "acc-qq", &registry.usage_ledger);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].session_id, "during");
    }

    #[test]
    fn build_usage_panel_data_falls_back_to_global_runtime_when_selected_account_has_no_attribution()
     {
        let selected = sample_saved_account("qq_pro", "acc-qq");
        let registry = registry_with_account("qq_pro", "acc-qq");
        let now = Utc::now();
        let records = vec![crate::services::CodexUsageRecord {
            session_id: "global-only".to_string(),
            timestamp: now - Duration::minutes(30),
            input_tokens: 120,
            output_tokens: 30,
            model: Some("gpt-5.4".to_string()),
        }];
        let dataset = CodexUsageDataset {
            global: crate::services::CodexUsageService::compute_rolling_usage_for_records(&records),
            records,
        };

        let panel = CodexAuthApp::build_usage_panel_data(&dataset, Some(&selected), &registry);
        assert_eq!(panel.scope, CodexUsageScope::GlobalRuntime);
        assert_eq!(
            panel.attribution_state,
            CodexUsageAttributionState::UnattributedFallback
        );
        assert!(panel.fallback_reason.is_some());
        assert_eq!(panel.rolling.all_time.total_requests, 1);
    }

    #[test]
    fn build_usage_panel_data_prefers_attributed_records_for_selected_account() {
        let selected = sample_saved_account("qq_pro", "acc-qq");
        let mut registry = registry_with_account("qq_pro", "acc-qq");
        registry.accounts.insert(
            "other".to_string(),
            crate::models::CodexAuthAccount {
                description: None,
                account_id: "acc-other".to_string(),
                auth_method: None,
                api_base_url: None,
                api_provider_name: None,
                email: None,
                plan_type: None,
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: None,
            },
        );
        registry.record_usage_activation(
            "qq_pro",
            "acc-qq",
            Utc.with_ymd_and_hms(2026, 4, 13, 10, 0, 0).unwrap(),
        );
        registry.record_usage_activation(
            "other",
            "acc-other",
            Utc.with_ymd_and_hms(2026, 4, 13, 12, 0, 0).unwrap(),
        );

        let records = vec![
            crate::services::CodexUsageRecord {
                session_id: "qq-pro".to_string(),
                timestamp: Utc.with_ymd_and_hms(2026, 4, 13, 11, 0, 0).unwrap(),
                input_tokens: 200,
                output_tokens: 40,
                model: Some("gpt-5.4".to_string()),
            },
            crate::services::CodexUsageRecord {
                session_id: "other".to_string(),
                timestamp: Utc.with_ymd_and_hms(2026, 4, 13, 12, 30, 0).unwrap(),
                input_tokens: 500,
                output_tokens: 80,
                model: Some("gpt-5.4-mini".to_string()),
            },
        ];
        let dataset = CodexUsageDataset {
            global: crate::services::CodexUsageService::compute_rolling_usage_for_records(&records),
            records,
        };

        let panel = CodexAuthApp::build_usage_panel_data(&dataset, Some(&selected), &registry);
        assert_eq!(
            panel.scope,
            CodexUsageScope::AccountAttributed {
                account_name: "qq_pro".to_string()
            }
        );
        assert_eq!(
            panel.attribution_state,
            CodexUsageAttributionState::AccountAttributed
        );
        assert_eq!(panel.rolling.all_time.total_requests, 1);
        assert_eq!(
            panel.top_model.as_ref().map(|top| top.model.as_str()),
            Some("gpt-5.4")
        );
    }

    fn make_test_app(accounts: Vec<CodexAuthItem>, selected_index: usize) -> CodexAuthApp {
        CodexAuthApp {
            accounts,
            selected_index,
            current_page: 0,
            overlay: None,
            toasts: ToastManager::new(),
            should_quit: false,
            login_state: LoginState::LoggedInSaved("main".to_string()),
            auth_registry: CodexAuthRegistry::default(),
            runtime_summary: None,
            service: CodexAuthService::from_dirs(PathBuf::from("."), PathBuf::from(".")),
            last_action: None,
            usage_state: UsageState::Loading,
            quota_state: QuotaState::Idle,
            quota_rx: None,
            usage_rx: None,
            codex_dir: None,
            list_area: Cell::new(None),
            quota_refresh: RefreshSchedulerState::new(QUOTA_REFRESH_INTERVAL_TICKS),
            pending_quota_confirm: false,
        }
    }

    #[test]
    fn on_activated_queues_selected_then_top_accounts_with_one_second_gap() {
        let now = Utc::now();
        let accounts = vec![
            CodexAuthItem {
                name: "main".to_string(),
                description: None,
                email: None,
                plan_type: None,
                is_current: true,
                is_virtual: false,
                saved_at: Some(now - Duration::days(5)),
                last_used: Some(now - Duration::hours(1)),
                last_refresh: None,
                freshness: TokenFreshness::Fresh,
                expires_at: None,
            },
            CodexAuthItem {
                name: "alt-1".to_string(),
                description: None,
                email: None,
                plan_type: None,
                is_current: false,
                is_virtual: false,
                saved_at: Some(now - Duration::days(4)),
                last_used: Some(now - Duration::hours(2)),
                last_refresh: None,
                freshness: TokenFreshness::Fresh,
                expires_at: None,
            },
            CodexAuthItem {
                name: "alt-2".to_string(),
                description: None,
                email: None,
                plan_type: None,
                is_current: false,
                is_virtual: false,
                saved_at: Some(now - Duration::days(3)),
                last_used: Some(now - Duration::hours(3)),
                last_refresh: None,
                freshness: TokenFreshness::Fresh,
                expires_at: None,
            },
            CodexAuthItem {
                name: "alt-3".to_string(),
                description: None,
                email: None,
                plan_type: None,
                is_current: false,
                is_virtual: false,
                saved_at: Some(now - Duration::days(2)),
                last_used: Some(now - Duration::hours(4)),
                last_refresh: None,
                freshness: TokenFreshness::Fresh,
                expires_at: None,
            },
        ];
        let mut app = make_test_app(accounts, 0);

        app.on_activated();

        assert!(app.usage_rx.is_some());
        assert_eq!(app.quota_refresh.pending_len(), 3);
        assert!(app.quota_refresh.next_ready(false).is_none());
        for _ in 0..QUOTA_REFRESH_INTERVAL_TICKS {
            app.quota_refresh.tick();
        }
        let next = app
            .quota_refresh
            .next_ready(false)
            .expect("selected quota task should become ready after initial cooldown");
        assert_eq!(next.key, "main");
        assert_eq!(next.tier, RefreshTier::Current);
    }

    #[test]
    fn selection_change_enqueues_hover_refresh_without_disrupting_inflight_policy() {
        let now = Utc::now();
        let accounts = vec![
            CodexAuthItem {
                name: "main".to_string(),
                description: None,
                email: None,
                plan_type: None,
                is_current: true,
                is_virtual: false,
                saved_at: Some(now - Duration::days(3)),
                last_used: Some(now - Duration::hours(1)),
                last_refresh: None,
                freshness: TokenFreshness::Fresh,
                expires_at: None,
            },
            CodexAuthItem {
                name: "cold".to_string(),
                description: None,
                email: None,
                plan_type: None,
                is_current: false,
                is_virtual: false,
                saved_at: Some(now - Duration::days(1)),
                last_used: None,
                last_refresh: None,
                freshness: TokenFreshness::Fresh,
                expires_at: None,
            },
        ];
        let mut app = make_test_app(accounts, 0);
        app.quota_refresh.set_cooldown(1);

        app.move_down();

        assert_eq!(
            app.selected_account().map(|account| account.name.as_str()),
            Some("cold")
        );
        assert_eq!(app.quota_refresh.pending_len(), 1);
        app.quota_refresh.tick();
        let next = app
            .quota_refresh
            .next_ready(false)
            .expect("hover refresh should dispatch after cooldown tick");
        assert_eq!(next.key, "cold");
        assert_eq!(next.tier, RefreshTier::HoverOnly);
    }
}
