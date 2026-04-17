// OpenCode Auth TUI application state machine
// Manages manual save/switch/delete for OpenCode openai auth snapshots

use crate::models::{
    CodexAccountQuota, CodexToOpenCodeMigrationReport, OpenCodeAuthItem, OpenCodeAuthRegistry,
    OpenCodeLoginState, OpenCodeReadSnapshot,
};
use crate::services::{
    OpenCodeAuthService, OpenCodeQuotaService, OpenCodeRollingUsage, OpenCodeUsageRecord,
    OpenCodeUsageService,
};
use crate::tui::auth_refresh::{RefreshReason, RefreshSchedulerState, RefreshTask, RefreshTier};
use crate::tui::overlay::Overlay;
use crate::tui::runtime::TuiApp;
use crate::tui::toast::{Toast, ToastManager};
use ccr_core::core::error::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use indexmap::IndexMap;
use ratatui::Frame;
use ratatui::layout::Rect;
use std::cell::Cell;
use std::path::PathBuf;
use std::sync::mpsc::TryRecvError;

/// Maximum accounts per page
pub const PAGE_SIZE: usize = 10;
const QUOTA_REFRESH_INTERVAL_TICKS: u32 = 4;
const OPENAI_PROVIDER_ID: &str = "openai";
const CURRENT_RUNTIME_ACCOUNT_KEY: &str = "current-login";

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
    Loaded(Box<OpenCodeUsageDataset>),
    Error(String),
    NoData,
}

/// Quota query state
#[derive(Debug, Clone)]
pub enum QuotaState {
    Idle,
    Loading {
        account_name: String,
        cache: indexmap::IndexMap<String, CodexAccountQuota>,
    },
    Loaded {
        cache: indexmap::IndexMap<String, CodexAccountQuota>,
    },
    Error {
        account_name: String,
        message: String,
        cache: indexmap::IndexMap<String, CodexAccountQuota>,
    },
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
    /// Quota query state
    pub quota_state: QuotaState,
    /// Cached account list area for mouse hit-testing
    pub list_area: Cell<Option<Rect>>,
    /// Usage async result receiver
    usage_rx: Option<std::sync::mpsc::Receiver<OpenCodeUsageState>>,
    /// Quota async result receiver
    quota_rx:
        Option<std::sync::mpsc::Receiver<std::result::Result<CodexAccountQuota, (String, String)>>>,
    /// 账号级 quota 刷新调度器
    quota_refresh: RefreshSchedulerState<String>,
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
            quota_state: QuotaState::Idle,
            list_area: Cell::new(None),
            usage_rx: None,
            quota_rx: None,
            quota_refresh: RefreshSchedulerState::new(QUOTA_REFRESH_INTERVAL_TICKS),
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

    fn codex_import_preview_lines(report: &CodexToOpenCodeMigrationReport) -> Vec<String> {
        vec![
            "将从已保存的 Codex Auth 账号导入兼容的 OpenAI OAuth 账号。".to_string(),
            "不会覆盖现有 OpenCode 账号，也不会切换当前 OpenCode 登录。".to_string(),
            String::new(),
            format!("可导入: {}", report.imported),
            format!("同名跳过: {}", report.skipped_existing_name),
            format!("同 account_id 跳过: {}", report.skipped_existing_account_id),
            format!("认证不兼容: {}", report.skipped_incompatible_auth),
            format!("缺少快照: {}", report.skipped_missing_snapshot),
            format!("快照无效: {}", report.skipped_invalid_snapshot),
        ]
    }

    fn codex_import_summary(report: &CodexToOpenCodeMigrationReport) -> String {
        format!(
            "导入 {}，同名 {}，同 account_id {}，不兼容 {}，缺少快照 {}，快照无效 {}",
            report.imported,
            report.skipped_existing_name,
            report.skipped_existing_account_id,
            report.skipped_incompatible_auth,
            report.skipped_missing_snapshot,
            report.skipped_invalid_snapshot
        )
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

    /// Called when this tab becomes active
    pub fn on_activated(&mut self) {
        self.start_usage_fetch();
        self.rebuild_quota_refresh_plan(false);
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
                    OpenCodeUsageState::Loaded(Box::new(OpenCodeUsageDataset {
                        provider_id: OPENAI_PROVIDER_ID.to_string(),
                        rolling: OpenCodeUsageService::compute_rolling_usage_for_records(&records),
                        records,
                    }))
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

    fn selected_quota_key(&self) -> Option<String> {
        self.selected_account().map(|account| account.name.clone())
    }

    fn warm_quota_account_names(&self) -> Vec<String> {
        let mut candidates: Vec<_> = self
            .accounts
            .iter()
            .filter(|account| {
                !account.is_virtual && !OpenCodeAuthService::is_expired(account.expires_at)
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
                selected_key,
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

        std::thread::spawn(move || {
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(error) => {
                    let _ = tx.send(Err((
                        account_key.clone(),
                        format!("创建运行时失败: {}", error),
                    )));
                    return;
                }
            };

            rt.block_on(async {
                match OpenCodeQuotaService::new() {
                    Ok(service) => {
                        let quota = if account_key == CURRENT_RUNTIME_ACCOUNT_KEY {
                            if force_refresh {
                                service.fetch_current_quota_force_refresh().await
                            } else {
                                service.fetch_current_quota().await
                            }
                        } else if force_refresh {
                            service
                                .fetch_account_quota_force_refresh(&account_key)
                                .await
                        } else {
                            service.fetch_account_quota(&account_key).await
                        };
                        let _ = tx.send(Ok(quota));
                    }
                    Err(error) => {
                        let _ = tx.send(Err((
                            account_key,
                            format!("初始化 OpenCode 配额服务失败: {}", error),
                        )));
                    }
                }
            });
        });

        true
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
                    account_name: account_name.clone(),
                    message,
                    cache,
                };
                self.quota_refresh.finish(&account_name);
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
                    message: "OpenCode 配额查询通道已断开".to_string(),
                    cache,
                };
                self.quota_refresh.finish(&account_name);
                self.quota_rx = None;
                true
            }
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
                self.rebuild_quota_refresh_plan(true);
                self.try_start_quota_refresh(true);
                self.toasts.push(Toast::info("已刷新 OpenCode 账号与统计"));
            }
            KeyCode::Char('i') => match self.service.import_saved_codex_accounts(true) {
                Ok(report) => {
                    if !report.has_importable_accounts() {
                        if report.total() == 0 {
                            self.toasts.push(Toast::info("未发现已保存的 Codex 账号"));
                        } else {
                            self.toasts.push(Toast::info(format!(
                                "没有可导入账号: {}",
                                Self::codex_import_summary(&report)
                            )));
                        }
                    } else {
                        self.overlay = Some(Overlay::confirm_import_codex(
                            Self::codex_import_preview_lines(&report),
                        ));
                    }
                }
                Err(err) => {
                    self.last_action = Some((
                        "导入失败".to_string(),
                        "codex".to_string(),
                        false,
                        Some(err.to_string()),
                    ));
                    self.toasts
                        .push(Toast::error(format!("导入预览失败: {err}")));
                }
            },
            _ => {}
        }
        Ok(false)
    }

    fn handle_overlay_key(&mut self, key: KeyEvent) -> Result<bool> {
        let is_confirm = matches!(
            self.overlay,
            Some(Overlay::Confirm { .. }) | Some(Overlay::ImportCodexConfirm { .. })
        );
        if is_confirm {
            self.handle_confirm_key(key)
        } else {
            self.handle_input_key(key)
        }
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                match &self.overlay {
                    Some(Overlay::Confirm { subject, .. }) => {
                        let subject = subject.clone();
                        match self.service.delete_account(&subject) {
                            Ok(()) => {
                                self.last_action =
                                    Some(("已删除".to_string(), subject.clone(), true, None));
                                self.toasts.push(Toast::success(format!(
                                    "已删除 OpenCode 账号: {subject}"
                                )));
                                self.reload_accounts()?;
                                self.rebuild_quota_refresh_plan(false);
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
                    }
                    Some(Overlay::ImportCodexConfirm { .. }) => {
                        match self.service.import_saved_codex_accounts(false) {
                            Ok(report) => {
                                self.last_action = Some((
                                    "已导入".to_string(),
                                    format!("{} 个账号", report.imported),
                                    true,
                                    None,
                                ));
                                self.toasts.push(Toast::success(format!(
                                    "Codex -> OpenCode 导入完成: {}",
                                    Self::codex_import_summary(&report)
                                )));
                                self.reload_accounts()?;
                                self.rebuild_quota_refresh_plan(false);
                            }
                            Err(err) => {
                                self.last_action = Some((
                                    "导入失败".to_string(),
                                    "codex".to_string(),
                                    false,
                                    Some(err.to_string()),
                                ));
                                self.toasts.push(Toast::error(format!("导入失败: {err}")));
                            }
                        }
                    }
                    _ => return Ok(false),
                }
                self.overlay = None;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                let message = if matches!(self.overlay, Some(Overlay::ImportCodexConfirm { .. })) {
                    "已取消导入"
                } else {
                    "已取消删除"
                };
                self.overlay = None;
                self.toasts.push(Toast::info(message));
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
                        self.rebuild_quota_refresh_plan(false);
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
        let before = self.selected_account().map(|account| account.name.clone());
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if self.current_page > 0 {
            self.current_page -= 1;
            self.selected_index = self.current_page_accounts().len().saturating_sub(1);
        }
        self.maybe_request_quota_for_selection_change(before);
    }

    fn move_down(&mut self) {
        let before = self.selected_account().map(|account| account.name.clone());
        let page_accounts = self.current_page_accounts();
        if self.selected_index + 1 < page_accounts.len() {
            self.selected_index += 1;
        } else if self.current_page + 1 < self.total_pages() {
            self.current_page += 1;
            self.selected_index = 0;
        }
        self.maybe_request_quota_for_selection_change(before);
    }

    fn prev_page(&mut self) {
        let before = self.selected_account().map(|account| account.name.clone());
        if self.current_page > 0 {
            self.current_page -= 1;
            self.selected_index = 0;
        }
        self.maybe_request_quota_for_selection_change(before);
    }

    fn next_page(&mut self) {
        let before = self.selected_account().map(|account| account.name.clone());
        if self.current_page + 1 < self.total_pages() {
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
                    let before = self.selected_account().map(|account| account.name.clone());
                    self.selected_index = index;
                    self.maybe_request_quota_for_selection_change(before);
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use chrono::{Duration, Utc};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use serde_json::{Map as JsonMap, json};
    use std::path::Path;
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

    fn create_import_test_service() -> (
        OpenCodeAuthService,
        tempfile::TempDir,
        tempfile::TempDir,
        PathBuf,
    ) {
        let ccr = tempdir().unwrap();
        let opencode = tempdir().unwrap();
        let codex_ccr_dir = ccr.path().join("platforms").join("codex");
        (
            OpenCodeAuthService::from_dirs_with_codex(
                ccr.path().join("platforms").join("opencode"),
                opencode.path().to_path_buf(),
                codex_ccr_dir.clone(),
            ),
            ccr,
            opencode,
            codex_ccr_dir,
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

    fn write_codex_source_account(codex_ccr_dir: &Path, name: &str, account_id: &str) {
        std::fs::create_dir_all(codex_ccr_dir.join("auth")).unwrap();

        let mut registry = crate::models::CodexAuthRegistry::default();
        registry.accounts.insert(
            name.to_string(),
            crate::models::CodexAuthAccount {
                description: None,
                account_id: account_id.to_string(),
                auth_method: Some(crate::models::OpenAiAuthMethod::Chatgpt),
                api_base_url: None,
                api_provider_name: None,
                email: None,
                plan_type: None,
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: Some(Utc::now()),
                expires_at: Some(Utc::now() + Duration::days(3)),
            },
        );
        std::fs::write(
            codex_ccr_dir.join("auth_registry.toml"),
            toml::to_string_pretty(&registry).unwrap(),
        )
        .unwrap();

        let snapshot = crate::models::CodexAuthJson {
            openai_api_key: None,
            tokens: Some(crate::models::CodexAuthTokens {
                id_token: Some(format!(
                    "{}.{}.signature",
                    URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#),
                    URL_SAFE_NO_PAD.encode(json!({ "email": "codex@example.com" }).to_string())
                )),
                access_token: Some(format!(
                    "{}.{}.signature",
                    URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#),
                    URL_SAFE_NO_PAD.encode(
                        json!({
                            "email": "codex@example.com",
                            "chatgpt_account_id": account_id,
                            "chatgpt_plan_type": "plus",
                            "exp": (Utc::now() + Duration::days(3)).timestamp()
                        })
                        .to_string()
                    )
                )),
                refresh_token: Some(format!("rt_{account_id}")),
                account_id: Some(account_id.to_string()),
            }),
            last_refresh: Some(Utc::now().to_rfc3339()),
        };
        std::fs::write(
            codex_ccr_dir.join("auth").join(format!("{name}.json")),
            serde_json::to_string_pretty(&snapshot).unwrap(),
        )
        .unwrap();
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

    #[test]
    fn import_codex_overlay_imports_accounts_after_confirmation() {
        let (service, ccr, opencode, codex_ccr_dir) = create_import_test_service();
        write_auth_json(
            &opencode.path().join("auth.json"),
            "runtime-acc",
            "runtime@example.com",
            "plus",
        );
        write_codex_source_account(&codex_ccr_dir, "codex-work", "codex-acc-1");

        let mut app = OpenCodeAuthApp::from_service(OpenCodeAuthService::from_dirs_with_codex(
            ccr.path().join("platforms").join("opencode"),
            opencode.path().to_path_buf(),
            codex_ccr_dir,
        ))
        .unwrap();

        app.handle_key(key(KeyCode::Char('i'))).unwrap();
        assert!(matches!(
            app.overlay,
            Some(Overlay::ImportCodexConfirm { .. })
        ));

        app.handle_key(key(KeyCode::Char('y'))).unwrap();

        let registry = service.load_registry().unwrap();
        assert!(registry.accounts.contains_key("codex-work"));
        assert_eq!(
            app.last_action,
            Some(("已导入".to_string(), "1 个账号".to_string(), true, None))
        );
    }

    fn make_refresh_test_app(
        accounts: Vec<OpenCodeAuthItem>,
        selected_index: usize,
    ) -> OpenCodeAuthApp {
        OpenCodeAuthApp {
            accounts,
            selected_index,
            current_page: 0,
            overlay: None,
            toasts: ToastManager::new(),
            should_quit: false,
            login_state: OpenCodeLoginState::LoggedInSaved("main".to_string()),
            auth_registry: OpenCodeAuthRegistry::default(),
            service: OpenCodeAuthService::from_dirs(PathBuf::from("."), PathBuf::from(".")),
            opencode_dir: PathBuf::from("."),
            last_action: None,
            usage_state: OpenCodeUsageState::Loading,
            quota_state: QuotaState::Idle,
            list_area: Cell::new(None),
            usage_rx: None,
            quota_rx: None,
            quota_refresh: RefreshSchedulerState::new(QUOTA_REFRESH_INTERVAL_TICKS),
        }
    }

    #[test]
    fn on_activated_queues_selected_then_top_accounts_with_initial_delay() {
        let now = Utc::now();
        let accounts = vec![
            OpenCodeAuthItem {
                name: "main".to_string(),
                account_id: Some("acc-main".to_string()),
                email: None,
                plan_type: Some("PLUS".to_string()),
                is_current: true,
                is_virtual: false,
                saved_at: Some(now - Duration::days(5)),
                last_used: Some(now - Duration::hours(1)),
                freshness: crate::models::TokenFreshness::Fresh,
                expires_at: Some(now + Duration::days(7)),
            },
            OpenCodeAuthItem {
                name: "alt-1".to_string(),
                account_id: Some("acc-1".to_string()),
                email: None,
                plan_type: Some("PLUS".to_string()),
                is_current: false,
                is_virtual: false,
                saved_at: Some(now - Duration::days(4)),
                last_used: Some(now - Duration::hours(2)),
                freshness: crate::models::TokenFreshness::Fresh,
                expires_at: Some(now + Duration::days(7)),
            },
            OpenCodeAuthItem {
                name: "alt-2".to_string(),
                account_id: Some("acc-2".to_string()),
                email: None,
                plan_type: Some("PLUS".to_string()),
                is_current: false,
                is_virtual: false,
                saved_at: Some(now - Duration::days(3)),
                last_used: Some(now - Duration::hours(3)),
                freshness: crate::models::TokenFreshness::Fresh,
                expires_at: Some(now + Duration::days(7)),
            },
            OpenCodeAuthItem {
                name: "alt-3".to_string(),
                account_id: Some("acc-3".to_string()),
                email: None,
                plan_type: Some("PLUS".to_string()),
                is_current: false,
                is_virtual: false,
                saved_at: Some(now - Duration::days(2)),
                last_used: Some(now - Duration::hours(4)),
                freshness: crate::models::TokenFreshness::Fresh,
                expires_at: Some(now + Duration::days(7)),
            },
        ];
        let mut app = make_refresh_test_app(accounts, 0);

        app.on_activated();

        assert!(app.usage_rx.is_some());
        assert_eq!(app.quota_refresh.pending_len(), 3);
        assert!(app.quota_refresh.next_ready(false).is_none());
        for _ in 0..QUOTA_REFRESH_INTERVAL_TICKS {
            app.quota_refresh.tick();
        }
        let next = app.quota_refresh.next_ready(false).unwrap();
        assert_eq!(next.key, "main");
        assert_eq!(next.tier, RefreshTier::Current);
    }

    #[test]
    fn selection_change_enqueues_hover_refresh_for_cold_account() {
        let now = Utc::now();
        let accounts = vec![
            OpenCodeAuthItem {
                name: "main".to_string(),
                account_id: Some("acc-main".to_string()),
                email: None,
                plan_type: Some("PLUS".to_string()),
                is_current: true,
                is_virtual: false,
                saved_at: Some(now - Duration::days(3)),
                last_used: Some(now - Duration::hours(1)),
                freshness: crate::models::TokenFreshness::Fresh,
                expires_at: Some(now + Duration::days(7)),
            },
            OpenCodeAuthItem {
                name: "cold".to_string(),
                account_id: Some("acc-cold".to_string()),
                email: None,
                plan_type: Some("PLUS".to_string()),
                is_current: false,
                is_virtual: false,
                saved_at: Some(now - Duration::days(1)),
                last_used: None,
                freshness: crate::models::TokenFreshness::Fresh,
                expires_at: Some(now + Duration::days(7)),
            },
        ];
        let mut app = make_refresh_test_app(accounts, 0);
        app.quota_refresh.set_cooldown(1);

        app.move_down();

        assert_eq!(
            app.selected_account().map(|account| account.name.as_str()),
            Some("cold")
        );
        assert_eq!(app.quota_refresh.pending_len(), 1);
        app.quota_refresh.tick();
        let next = app.quota_refresh.next_ready(false).unwrap();
        assert_eq!(next.key, "cold");
        assert_eq!(next.tier, RefreshTier::HoverOnly);
    }
}
