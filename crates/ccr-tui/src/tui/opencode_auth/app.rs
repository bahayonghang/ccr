// OpenCode Auth TUI application state machine
// Manages manual save/switch/delete for OpenCode openai auth snapshots

use crate::tui::CompletedAction;
use crate::tui::auth_refresh::{RefreshReason, RefreshSchedulerState, RefreshTask, RefreshTier};
use crate::tui::overlay::Overlay;
use crate::tui::pagination::{
    DEFAULT_PAGE_SIZE, index_in_page, page_for_index, page_slice, total_pages,
};
use crate::tui::runtime::{AsyncTaskExecutor, TuiApp};
use crate::tui::toast::{Toast, ToastManager};
use ccr_cli::models::{
    CodexAccountQuota, CodexToOpenCodeMigrationReport, OpenCodeAuthItem, OpenCodeAuthRegistry,
    OpenCodeLoginState, OpenCodeReadSnapshot,
};
use ccr_cli::services::{
    OpenCodeAuthService, OpenCodeQuotaService, OpenCodeRollingUsage, OpenCodeUsageRecord,
    OpenCodeUsageService,
};
use ccr_core::core::error::Result;
use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use indexmap::IndexMap;
use ratatui::Frame;
use ratatui::layout::Rect;
use std::cell::Cell;
use std::path::PathBuf;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, error::TryRecvError};

const ACTIVATION_DELAY_TICKS: u32 = 4;
const QUOTA_REFRESH_INTERVAL_TICKS: u32 = 4;
const PREVIEW_TTL_SECS: i64 = 60;
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

#[derive(Debug, Clone)]
pub struct QuotaPreviewEntry {
    pub quota: CodexAccountQuota,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewMetricWindow {
    FiveHour,
    SevenDay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaPreviewCellState {
    Waiting,
    Loading,
    Ready,
    Error,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaPreviewCell {
    pub text: String,
    pub state: QuotaPreviewCellState,
}

enum OpenCodeAuthTaskMessage {
    Usage(OpenCodeUsageState),
    Preview(Vec<CodexAccountQuota>),
    Quota(std::result::Result<CodexAccountQuota, (String, String)>),
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
    /// Page capacity derived from visible account rows
    pub page_size: usize,
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
    pub last_action: Option<(CompletedAction, String, bool, Option<String>)>,
    /// Usage data state
    pub usage_state: OpenCodeUsageState,
    /// Quota query state
    pub quota_state: QuotaState,
    /// Cached account list area for mouse hit-testing
    pub list_area: Cell<Option<Rect>>,
    /// 账号级 quota 刷新调度器
    quota_refresh: RefreshSchedulerState<String>,
    /// 账号预览缓存（供列表速览与 Focus 共用）
    pub(crate) preview_cache: IndexMap<String, QuotaPreviewEntry>,
    /// 激活页签后的 1s 延迟门控
    activation_delay_ticks: Option<u32>,
    /// 异步后台任务执行器
    task_executor: AsyncTaskExecutor,
    /// 后台任务消息发送端
    task_tx: UnboundedSender<OpenCodeAuthTaskMessage>,
    /// 后台任务消息接收端
    task_rx: UnboundedReceiver<OpenCodeAuthTaskMessage>,
    /// Usage 后台任务是否进行中
    usage_task_active: bool,
    /// Preview 后台任务是否进行中
    preview_task_active: bool,
    /// Quota 后台任务是否进行中
    quota_task_active: bool,
}

impl OpenCodeAuthApp {
    /// Create a new application instance
    #[allow(dead_code)]
    pub fn new() -> Result<Self> {
        Self::with_task_executor(AsyncTaskExecutor::from_current_or_test())
    }

    pub fn with_task_executor(task_executor: AsyncTaskExecutor) -> Result<Self> {
        Self::from_service_with_task_executor(OpenCodeAuthService::new()?, task_executor)
    }

    #[allow(dead_code)]
    pub(crate) fn from_service(service: OpenCodeAuthService) -> Result<Self> {
        Self::from_service_with_task_executor(service, AsyncTaskExecutor::from_current_or_test())
    }

    pub(crate) fn from_service_with_task_executor(
        service: OpenCodeAuthService,
        task_executor: AsyncTaskExecutor,
    ) -> Result<Self> {
        let snapshot = service.read_auth_snapshot()?;
        let accounts = service.build_account_items(&snapshot)?;
        let (task_tx, task_rx) = tokio::sync::mpsc::unbounded_channel();
        let selected_index = accounts
            .iter()
            .position(|account| account.is_current)
            .map(|index| index_in_page(index, DEFAULT_PAGE_SIZE))
            .unwrap_or(0);
        let current_page = accounts
            .iter()
            .position(|account| account.is_current)
            .map(|index| page_for_index(index, DEFAULT_PAGE_SIZE))
            .unwrap_or(0);
        let opencode_dir = service.opencode_dir().to_path_buf();

        Ok(Self {
            accounts,
            selected_index,
            current_page,
            page_size: DEFAULT_PAGE_SIZE,
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
            quota_refresh: RefreshSchedulerState::new(QUOTA_REFRESH_INTERVAL_TICKS),
            preview_cache: IndexMap::new(),
            activation_delay_ticks: None,
            task_executor,
            task_tx,
            task_rx,
            usage_task_active: false,
            preview_task_active: false,
            quota_task_active: false,
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
        self.current_page = page_for_index(preferred, self.page_size);
        let page_len = self.current_page_accounts().len();
        self.selected_index = if page_len == 0 {
            0
        } else {
            index_in_page(preferred, self.page_size)
        };
        self.reconcile_preview_cache();

        Ok(())
    }

    /// Reload account list from disk
    pub fn reload_accounts(&mut self) -> Result<()> {
        let snapshot = self.service.read_auth_snapshot()?;
        self.apply_snapshot(snapshot)?;
        self.arm_activation_gate();
        Ok(())
    }

    fn codex_import_preview_lines(report: &CodexToOpenCodeMigrationReport) -> Vec<String> {
        vec![
            crate::tui_text!(
                "Compatible OpenAI OAuth accounts will be imported from saved Codex Auth accounts.",
                "将从已保存的 Codex Auth 账号导入兼容的 OpenAI OAuth 账号。"
            )
            .to_string(),
            crate::tui_text!(
                "Existing OpenCode accounts will not be overwritten and the current login will not change.",
                "不会覆盖现有 OpenCode 账号，也不会切换当前 OpenCode 登录。"
            )
            .to_string(),
            String::new(),
            crate::tui_format!("Importable: {}", "可导入：{}", report.imported),
            crate::tui_format!("Same-name skipped: {}", "同名跳过：{}", report.skipped_existing_name),
            crate::tui_format!("Same account_id skipped: {}", "同 account_id 跳过：{}", report.skipped_existing_account_id),
            crate::tui_format!("Incompatible auth: {}", "认证不兼容：{}", report.skipped_incompatible_auth),
            crate::tui_format!("Missing snapshot: {}", "缺少快照：{}", report.skipped_missing_snapshot),
            crate::tui_format!("Invalid snapshot: {}", "快照无效：{}", report.skipped_invalid_snapshot),
        ]
    }

    fn codex_import_summary(report: &CodexToOpenCodeMigrationReport) -> String {
        crate::tui_format!(
            "imported {}, same name {}, same account_id {}, incompatible {}, missing snapshot {}, invalid snapshot {}",
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
        page_slice(&self.accounts, self.current_page, self.page_size)
    }

    /// Get total pages
    pub fn total_pages(&self) -> usize {
        total_pages(self.accounts.len(), self.page_size)
    }

    /// Get currently selected account
    pub fn selected_account(&self) -> Option<&OpenCodeAuthItem> {
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

    pub fn selected_quota(&self) -> Option<&CodexAccountQuota> {
        let selected_name = self.selected_account()?.name.as_str();
        self.preview_cache
            .get(selected_name)
            .map(|entry| &entry.quota)
            .or_else(|| self.quota_cache().get(selected_name))
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
            _ => self
                .selected_account()
                .and_then(|account| self.preview_cache.get(&account.name))
                .and_then(|entry| entry.quota.error.as_deref()),
        }
    }

    pub fn selected_preview_entry(&self) -> Option<&QuotaPreviewEntry> {
        let selected_name = self.selected_account()?.name.as_str();
        self.preview_cache.get(selected_name)
    }

    #[allow(dead_code)]
    pub fn selected_preview_reset_text(&self) -> String {
        self.selected_preview_entry()
            .map(|entry| Self::preview_reset_detail_summary_text(&entry.quota))
            .unwrap_or_else(|| {
                if self.is_activation_gate_pending() {
                    crate::tui_text!("waiting for 1s activation gate", "等待 1s 激活门控")
                        .to_string()
                } else if self.preview_task_active {
                    crate::tui_text!("querying batch…", "正在批量查询…").to_string()
                } else {
                    "-".to_string()
                }
            })
    }

    pub fn preview_reset_cell_for_account(&self, account_name: &str) -> QuotaPreviewCell {
        if let Some(entry) = self.preview_cache.get(account_name) {
            if entry.quota.error.is_some() {
                return QuotaPreviewCell {
                    text: "ERR".to_string(),
                    state: QuotaPreviewCellState::Error,
                };
            }

            return QuotaPreviewCell {
                text: Self::preview_reset_summary_text(&entry.quota),
                state: QuotaPreviewCellState::Ready,
            };
        }

        if self.is_activation_gate_pending() {
            return QuotaPreviewCell {
                text: "1s…".to_string(),
                state: QuotaPreviewCellState::Waiting,
            };
        }

        if self.preview_task_active
            || self
                .quota_refresh
                .in_flight_key()
                .is_some_and(|key| key == account_name)
        {
            return QuotaPreviewCell {
                text: "…".to_string(),
                state: QuotaPreviewCellState::Loading,
            };
        }

        QuotaPreviewCell {
            text: "-".to_string(),
            state: QuotaPreviewCellState::Empty,
        }
    }

    /// 返回指定账号的完整 quota 快照(若已缓存)。
    /// UI 层的 5h / 7d 单元格需要基于它读 reset 时间戳拼括号文案。
    pub fn preview_quota_for_account(&self, account_name: &str) -> Option<&CodexAccountQuota> {
        self.preview_cache
            .get(account_name)
            .map(|entry| &entry.quota)
    }

    pub fn preview_cell_for_account(
        &self,
        account_name: &str,
        window: PreviewMetricWindow,
    ) -> QuotaPreviewCell {
        if let Some(entry) = self.preview_cache.get(account_name) {
            if let Some(quota) = entry.quota.quota.as_ref() {
                let value = match window {
                    PreviewMetricWindow::FiveHour => quota.hourly_percentage,
                    PreviewMetricWindow::SevenDay => quota.weekly_percentage,
                };
                return QuotaPreviewCell {
                    text: format!("{value}%"),
                    state: QuotaPreviewCellState::Ready,
                };
            }

            if entry.quota.error.is_some() {
                return QuotaPreviewCell {
                    text: "ERR".to_string(),
                    state: QuotaPreviewCellState::Error,
                };
            }
        }

        if self.is_activation_gate_pending() {
            return QuotaPreviewCell {
                text: "1s…".to_string(),
                state: QuotaPreviewCellState::Waiting,
            };
        }

        if self.preview_task_active
            || self
                .quota_refresh
                .in_flight_key()
                .is_some_and(|key| key == account_name)
        {
            return QuotaPreviewCell {
                text: "…".to_string(),
                state: QuotaPreviewCellState::Loading,
            };
        }

        QuotaPreviewCell {
            text: "-".to_string(),
            state: QuotaPreviewCellState::Empty,
        }
    }

    pub fn is_activation_gate_pending(&self) -> bool {
        self.activation_delay_ticks.is_some()
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
        self.arm_activation_gate();
    }

    pub fn usage_panel_data(&self) -> Option<OpenCodeAuthUsagePanelData> {
        let OpenCodeUsageState::Loaded(dataset) = &self.usage_state else {
            return None;
        };

        let selected = self.selected_account();
        let top_model = Self::top_model_from_usage(&dataset.rolling);
        let fallback_reason = match selected {
            None => Some(crate::tui_text!(
                "No account is selected; showing local usage for this machine's OpenCode openai provider",
                "当前未选中账号，以下展示本机 OpenCode openai provider 的本地 usage 汇总"
            ).to_string()),
            Some(account) if account.is_virtual => Some(
                crate::tui_text!(
                    "The current login is not saved as a CCR account. OpenCode logs only contain provider/model, not saved account ids; showing the openai provider aggregate",
                    "当前登录尚未保存为 CCR 账号；OpenCode 本地日志仅记录 provider/model，不包含保存账号 id，以下为 openai provider 汇总"
                ).to_string(),
            ),
            Some(account) if account.is_current => Some(
                crate::tui_text!(
                    "The selected account is the current login, but OpenCode logs only contain provider/model; showing the openai provider aggregate rather than exact account attribution",
                    "当前选中账号就是当前登录，但 OpenCode 本地日志仍只记录 provider/model；以下为 openai provider 汇总，而不是按账号精确归因"
                ).to_string(),
            ),
            Some(_) => Some(
                crate::tui_text!(
                    "The selected account is not the current login. OpenCode logs do not contain saved account ids; this machine's openai provider aggregate does not represent separate account history",
                    "所选账号不是当前登录；OpenCode 本地日志不包含保存账号 id，以下为当前机器 openai provider 汇总，不代表该账号独立历史"
                ).to_string(),
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

    fn preview_account_keys(&self) -> Vec<String> {
        let mut keys = Vec::with_capacity(self.accounts.len());
        for account in &self.accounts {
            if !keys.iter().any(|existing| existing == &account.name) {
                keys.push(account.name.clone());
            }
        }
        keys
    }

    fn reconcile_preview_cache(&mut self) {
        let valid_keys = self.preview_account_keys();
        self.preview_cache
            .retain(|key, _| valid_keys.iter().any(|candidate| candidate == key));
    }

    fn preview_keys_to_refresh(&self, force_refresh: bool) -> Vec<String> {
        self.preview_account_keys()
            .into_iter()
            .filter(|key| force_refresh || self.preview_entry_needs_refresh(key))
            .collect()
    }

    fn preview_entry_needs_refresh(&self, account_key: &str) -> bool {
        self.preview_cache
            .get(account_key)
            .is_none_or(Self::is_preview_entry_stale)
    }

    fn is_preview_entry_stale(entry: &QuotaPreviewEntry) -> bool {
        (Utc::now() - entry.quota.fetched_at).num_seconds() >= PREVIEW_TTL_SECS
    }

    fn preview_reset_summary_text(quota: &CodexAccountQuota) -> String {
        match quota.quota.as_ref() {
            Some(quota) => format!(
                "{}/{}",
                Self::short_reset_label(quota.hourly_reset_time),
                Self::short_reset_label(quota.weekly_reset_time)
            ),
            None if quota.error.is_some() => "ERR".to_string(),
            None => "-".to_string(),
        }
    }

    #[allow(dead_code)]
    fn preview_reset_detail_summary_text(quota: &CodexAccountQuota) -> String {
        match quota.quota.as_ref() {
            Some(quota) => format!(
                "{}/{}",
                Self::detailed_reset_label(quota.hourly_reset_time),
                Self::detailed_reset_label(quota.weekly_reset_time)
            ),
            None if quota.error.is_some() => "ERR".to_string(),
            None => "-".to_string(),
        }
    }

    fn short_reset_label(reset_timestamp: Option<i64>) -> String {
        let Some(reset_timestamp) = reset_timestamp else {
            return "-".to_string();
        };
        let seconds = (reset_timestamp - Utc::now().timestamp()).max(0);
        if seconds < 3600 {
            format!("{}m", seconds / 60)
        } else if seconds < 86_400 {
            format!("{}h", seconds / 3600)
        } else {
            format!("{}d", seconds / 86_400)
        }
    }

    #[allow(dead_code)]
    fn detailed_reset_label(reset_timestamp: Option<i64>) -> String {
        reset_timestamp
            .map(Self::long_reset_label)
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn quota_reset_detail_text(reset_timestamp: Option<i64>) -> String {
        let Some(reset_timestamp) = reset_timestamp else {
            return "-".to_string();
        };
        let dt = chrono::DateTime::from_timestamp(reset_timestamp, 0)
            .map(|value| value.with_timezone(&chrono::Local));
        let relative = Self::long_reset_label(reset_timestamp);
        if let Some(local) = dt {
            format!("{} ({})", relative, local.format("%m/%d %H:%M"))
        } else {
            relative
        }
    }

    fn long_reset_label(reset_timestamp: i64) -> String {
        let seconds = (reset_timestamp - Utc::now().timestamp()).max(0);
        if seconds < 3600 {
            format!("{}m", seconds / 60)
        } else if seconds < 86_400 {
            let hours = seconds / 3600;
            let minutes = (seconds % 3600) / 60;
            if minutes == 0 {
                format!("{hours}h")
            } else {
                format!("{hours}h{minutes}m")
            }
        } else {
            let days = seconds / 86_400;
            let hours = (seconds % 86_400) / 3600;
            let minutes = (seconds % 3600) / 60;
            if hours == 0 && minutes == 0 {
                format!("{days}d")
            } else if minutes == 0 {
                format!("{days}d{hours}h")
            } else if hours == 0 {
                format!("{days}d{minutes}m")
            } else {
                format!("{days}d{hours}h{minutes}m")
            }
        }
    }

    fn arm_activation_gate(&mut self) {
        if self.usage_task_active || self.preview_task_active {
            self.activation_delay_ticks = None;
            return;
        }

        let needs_preview = !self.preview_keys_to_refresh(false).is_empty();
        if needs_preview || !self.usage_task_active {
            self.activation_delay_ticks = Some(ACTIVATION_DELAY_TICKS);
        } else {
            self.activation_delay_ticks = None;
        }
    }

    fn start_usage_fetch(&mut self) {
        if self.usage_task_active {
            return;
        }

        self.usage_state = OpenCodeUsageState::Loading;
        let usage_service = OpenCodeUsageService::new(self.opencode_dir.clone());
        let tx = self.task_tx.clone();
        self.usage_task_active = true;

        self.task_executor.spawn_blocking(move || {
            let state = Self::load_usage_data(&usage_service);
            let _ = tx.send(OpenCodeAuthTaskMessage::Usage(state));
        });
    }

    fn start_preview_prefetch(&mut self, force_refresh: bool) -> bool {
        if self.preview_task_active {
            return false;
        }

        let account_names = self.preview_keys_to_refresh(force_refresh);
        if account_names.is_empty() {
            return false;
        }

        let tx = self.task_tx.clone();
        self.preview_task_active = true;

        self.task_executor.spawn(async move {
            let quotas = match OpenCodeQuotaService::new() {
                Ok(service) => {
                    if force_refresh {
                        service
                            .fetch_quotas_for_accounts_force_refresh(&account_names)
                            .await
                    } else {
                        service.fetch_quotas_for_accounts(&account_names).await
                    }
                }
                Err(error) => account_names
                    .iter()
                    .map(|account_name| CodexAccountQuota {
                        account_name: account_name.clone(),
                        email: None,
                        quota: None,
                        error: Some(error.to_string()),
                        fetched_at: Utc::now(),
                    })
                    .collect(),
            };
            let _ = tx.send(OpenCodeAuthTaskMessage::Preview(quotas));
        });

        true
    }

    fn selected_quota_key(&self) -> Option<String> {
        self.selected_account().map(|account| account.name.clone())
    }

    fn rebuild_quota_refresh_plan(&mut self, force_selected_refresh: bool) {
        self.quota_refresh.clear_pending();

        if force_selected_refresh && let Some(selected_key) = self.selected_quota_key() {
            self.quota_refresh.push(RefreshTask::new(
                selected_key,
                RefreshTier::Current,
                RefreshReason::ManualRefresh,
                force_selected_refresh,
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
        if self.quota_task_active {
            return false;
        }

        let cache = self.quota_cache().clone();
        self.quota_state = QuotaState::Loading {
            account_name: account_key.clone(),
            cache,
        };
        let tx = self.task_tx.clone();
        self.quota_task_active = true;

        self.task_executor.spawn(async move {
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
                    let _ = tx.send(OpenCodeAuthTaskMessage::Quota(Ok(quota)));
                }
                Err(error) => {
                    let _ = tx.send(OpenCodeAuthTaskMessage::Quota(Err((
                        account_key,
                        error.to_string(),
                    ))));
                }
            }
        });

        true
    }

    fn drain_task_messages(&mut self) -> bool {
        let mut changed = false;

        loop {
            match self.task_rx.try_recv() {
                Ok(OpenCodeAuthTaskMessage::Usage(state)) => {
                    self.usage_state = state;
                    self.usage_task_active = false;
                    changed = true;
                }
                Ok(OpenCodeAuthTaskMessage::Preview(quotas)) => {
                    for quota in quotas {
                        self.preview_cache
                            .insert(quota.account_name.clone(), QuotaPreviewEntry { quota });
                    }
                    self.preview_task_active = false;
                    changed = true;
                }
                Ok(OpenCodeAuthTaskMessage::Quota(Ok(quota))) => {
                    let mut cache = self.quota_cache().clone();
                    let account_name = quota.account_name.clone();
                    cache.insert(account_name.clone(), quota.clone());
                    self.preview_cache
                        .insert(account_name.clone(), QuotaPreviewEntry { quota });
                    self.quota_state = QuotaState::Loaded { cache };
                    self.quota_refresh.finish(&account_name);
                    self.quota_task_active = false;
                    changed = true;
                }
                Ok(OpenCodeAuthTaskMessage::Quota(Err((account_name, message)))) => {
                    let error_account_name = account_name.clone();
                    let error_message = message.clone();
                    let cache = self.quota_cache().clone();
                    self.quota_state = QuotaState::Error {
                        account_name: account_name.clone(),
                        message,
                        cache,
                    };
                    self.quota_refresh.finish(&account_name);
                    self.preview_cache.insert(
                        error_account_name.clone(),
                        QuotaPreviewEntry {
                            quota: CodexAccountQuota {
                                account_name: error_account_name,
                                email: None,
                                quota: None,
                                error: Some(error_message),
                                fetched_at: Utc::now(),
                            },
                        },
                    );
                    self.quota_task_active = false;
                    changed = true;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }

        changed
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
            KeyCode::Char('s') => match self.login_state {
                OpenCodeLoginState::LoggedInUnsaved => {
                    self.overlay = Some(Overlay::save_input());
                }
                OpenCodeLoginState::LoggedInSaved(_) => {
                    self.toasts.push(Toast::warning(crate::tui_text!(
                        "The current login is already saved",
                        "当前登录已保存"
                    )));
                }
                OpenCodeLoginState::NotLoggedIn => {
                    self.toasts.push(Toast::warning(crate::tui_text!(
                        "No OpenCode openai login detected",
                        "当前未检测到 OpenCode openai 登录"
                    )));
                }
            },
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some(account) = self.selected_account() {
                    if account.is_virtual {
                        self.toasts.push(Toast::warning(crate::tui_text!(
                            "The unsaved current login cannot be deleted",
                            "未保存的当前登录无法删除"
                        )));
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
                self.toasts.push(Toast::info(crate::tui_text!(
                    "OpenCode accounts and statistics reloaded",
                    "已刷新 OpenCode 账号与统计"
                )));
            }
            KeyCode::Char('i') => match self.service.import_saved_codex_accounts(true) {
                Ok(report) => {
                    if !report.has_importable_accounts() {
                        if report.total() == 0 {
                            self.toasts.push(Toast::info(crate::tui_text!(
                                "No saved Codex accounts found",
                                "未发现已保存的 Codex 账号"
                            )));
                        } else {
                            self.toasts.push(Toast::info(crate::tui_format!(
                                "No accounts can be imported: {}",
                                "没有可导入账号：{}",
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
                        CompletedAction::Import,
                        "codex".to_string(),
                        false,
                        Some(err.to_string()),
                    ));
                    self.toasts.push(Toast::error(crate::tui_format!(
                        "Failed to preview import: {err}",
                        "导入预览失败：{err}"
                    )));
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
                                    Some((CompletedAction::Delete, subject.clone(), true, None));
                                self.toasts.push(Toast::success(crate::tui_format!(
                                    "Deleted OpenCode account: {subject}",
                                    "已删除 OpenCode 账号：{subject}"
                                )));
                                self.reload_accounts()?;
                                self.rebuild_quota_refresh_plan(false);
                            }
                            Err(e) => {
                                self.last_action = Some((
                                    CompletedAction::Delete,
                                    subject.clone(),
                                    false,
                                    Some(e.to_string()),
                                ));
                                self.toasts.push(Toast::error(crate::tui_format!(
                                    "Delete failed: {e}",
                                    "删除失败：{e}"
                                )));
                            }
                        }
                    }
                    Some(Overlay::ImportCodexConfirm { .. }) => {
                        match self.service.import_saved_codex_accounts(false) {
                            Ok(report) => {
                                self.last_action = Some((
                                    CompletedAction::Import,
                                    report.imported.to_string(),
                                    true,
                                    None,
                                ));
                                self.toasts.push(Toast::success(crate::tui_format!(
                                    "Codex -> OpenCode import complete: {}",
                                    "Codex -> OpenCode 导入完成：{}",
                                    Self::codex_import_summary(&report)
                                )));
                                self.reload_accounts()?;
                                self.rebuild_quota_refresh_plan(false);
                            }
                            Err(err) => {
                                self.last_action = Some((
                                    CompletedAction::Import,
                                    "codex".to_string(),
                                    false,
                                    Some(err.to_string()),
                                ));
                                self.toasts.push(Toast::error(crate::tui_format!(
                                    "Import failed: {err}",
                                    "导入失败：{err}"
                                )));
                            }
                        }
                    }
                    _ => return Ok(false),
                }
                self.overlay = None;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                let message = if matches!(self.overlay, Some(Overlay::ImportCodexConfirm { .. })) {
                    crate::tui_text!("Import cancelled", "已取消导入")
                } else {
                    crate::tui_text!("Deletion cancelled", "已取消删除")
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

                if name.trim().is_empty() {
                    self.overlay = Some(Overlay::save_input());
                    self.toasts.push(Toast::warning(crate::tui_text!(
                        "Account name cannot be empty",
                        "账号名称不能为空"
                    )));
                    return Ok(false);
                }

                match self.service.save_current(name.trim(), false) {
                    Ok(()) => {
                        let saved_name = name.trim().to_string();
                        self.last_action =
                            Some((CompletedAction::Save, saved_name.clone(), true, None));
                        self.toasts.push(Toast::success(crate::tui_format!(
                            "Saved OpenCode account: {saved_name}",
                            "已保存 OpenCode 账号：{saved_name}"
                        )));
                        self.reload_accounts()?;
                        self.rebuild_quota_refresh_plan(false);
                    }
                    Err(e) => {
                        self.last_action = Some((
                            CompletedAction::Save,
                            name.trim().to_string(),
                            false,
                            Some(e.to_string()),
                        ));
                        self.toasts.push(Toast::error(crate::tui_format!(
                            "Save failed: {e}",
                            "保存失败：{e}"
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
        let before = self.selected_account().map(|account| account.name.clone());
        self.selected_index = super::super::selection::previous_index_in_page(
            self.current_page_accounts().len(),
            self.selected_index,
        );
        self.maybe_request_quota_for_selection_change(before);
    }

    fn move_down(&mut self) {
        let before = self.selected_account().map(|account| account.name.clone());
        self.selected_index = super::super::selection::next_index_in_page(
            self.current_page_accounts().len(),
            self.selected_index,
        );
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
        if current != previous
            && !self.is_activation_gate_pending()
            && current
                .as_deref()
                .is_some_and(|account_name| self.preview_entry_needs_refresh(account_name))
        {
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
                self.toasts.push(Toast::info(crate::tui_text!(
                    "This is the current login; no switch is needed",
                    "这是当前登录，无需切换"
                )));
                return Ok(false);
            }

            if account.is_current {
                self.toasts.push(Toast::info(crate::tui_text!(
                    "This account is already active",
                    "已经是当前账号"
                )));
                return Ok(false);
            }

            match self.service.switch_account(&account.name) {
                Ok(()) => {
                    self.last_action =
                        Some((CompletedAction::Switch, account.name.clone(), true, None));
                    self.toasts.push(Toast::success(crate::tui_format!(
                        "Switched to OpenCode account: {}",
                        "已切换到 OpenCode 账号：{}",
                        account.name
                    )));
                    self.should_quit = true;
                    return Ok(true);
                }
                Err(e) => {
                    self.last_action = Some((
                        CompletedAction::Switch,
                        account.name.clone(),
                        false,
                        Some(e.to_string()),
                    ));
                    self.toasts.push(Toast::error(crate::tui_format!(
                        "Switch failed: {e}",
                        "切换失败：{e}"
                    )));
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

        if let Some(remaining) = self.activation_delay_ticks.as_mut() {
            needs_redraw = true;
            if *remaining > 0 {
                *remaining -= 1;
            }
            if *remaining == 0 {
                self.activation_delay_ticks = None;
                self.start_usage_fetch();
                needs_redraw |= self.start_preview_prefetch(false);
            }
        }

        needs_redraw |= self.drain_task_messages();
        self.quota_refresh.tick();

        if self.try_start_quota_refresh(false) {
            needs_redraw = true;
        }

        needs_redraw
    }

    fn render(&mut self, frame: &mut Frame) {
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

        let mut registry = ccr_cli::models::CodexAuthRegistry::default();
        registry.accounts.insert(
            name.to_string(),
            ccr_cli::models::CodexAuthAccount {
                description: None,
                account_id: account_id.to_string(),
                auth_method: Some(ccr_cli::models::OpenAiAuthMethod::Chatgpt),
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

        let snapshot = ccr_cli::models::CodexAuthJson {
            openai_api_key: None,
            tokens: Some(ccr_cli::models::CodexAuthTokens {
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
            Some((CompletedAction::Import, "1".to_string(), true, None))
        );
    }

    fn make_refresh_test_app(
        accounts: Vec<OpenCodeAuthItem>,
        selected_index: usize,
    ) -> OpenCodeAuthApp {
        let (task_tx, task_rx) = tokio::sync::mpsc::unbounded_channel();
        OpenCodeAuthApp {
            accounts,
            selected_index,
            current_page: 0,
            page_size: DEFAULT_PAGE_SIZE,
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
            quota_refresh: RefreshSchedulerState::new(QUOTA_REFRESH_INTERVAL_TICKS),
            preview_cache: IndexMap::new(),
            activation_delay_ticks: None,
            task_executor: AsyncTaskExecutor::from_current_or_test(),
            task_tx,
            task_rx,
            usage_task_active: false,
            preview_task_active: false,
            quota_task_active: false,
        }
    }

    fn navigation_account(index: usize) -> OpenCodeAuthItem {
        let now = Utc::now();
        OpenCodeAuthItem {
            name: format!("account-{index:02}"),
            account_id: Some(format!("acc-{index:02}")),
            email: None,
            plan_type: Some("PLUS".to_string()),
            is_current: index == 1,
            is_virtual: false,
            saved_at: Some(now - Duration::days(index as i64)),
            last_used: None,
            expires_at: Some(now + Duration::days(7)),
        }
    }

    fn navigation_accounts(count: usize) -> Vec<OpenCodeAuthItem> {
        (1..=count).map(navigation_account).collect()
    }

    fn make_refresh_test_app_on_page(
        accounts: Vec<OpenCodeAuthItem>,
        selected_index: usize,
        current_page: usize,
    ) -> OpenCodeAuthApp {
        let mut app = make_refresh_test_app(accounts, selected_index);
        app.current_page = current_page;
        app
    }

    #[test]
    fn navigation_wrap_opencode_auth_stays_on_current_page() {
        let mut app = make_refresh_test_app_on_page(navigation_accounts(12), 0, 1);

        app.move_up();

        assert_eq!(app.current_page, 1);
        assert_eq!(app.selected_index, 1);

        app.move_down();

        assert_eq!(app.current_page, 1);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn navigation_wrap_opencode_auth_enqueues_hover_refresh_for_wrapped_account() {
        let mut app = make_refresh_test_app_on_page(navigation_accounts(12), 1, 1);
        app.quota_refresh.set_cooldown(1);

        app.move_down();

        assert_eq!(app.current_page, 1);
        assert_eq!(
            app.selected_account().map(|account| account.name.as_str()),
            Some("account-11")
        );
        assert_eq!(app.quota_refresh.pending_len(), 1);
        app.quota_refresh.tick();
        let next = app.quota_refresh.next_ready(false).unwrap();
        assert_eq!(next.key, "account-11");
        assert_eq!(next.tier, RefreshTier::HoverOnly);
    }

    #[test]
    fn current_page_accounts_respects_dynamic_page_size() {
        let mut app = make_refresh_test_app(navigation_accounts(20), 0);

        app.sync_page_size(14);

        assert_eq!(app.current_page_accounts().len(), 14);
        assert_eq!(app.total_pages(), 2);

        app.sync_page_size(25);

        assert_eq!(app.current_page_accounts().len(), 20);
        assert_eq!(app.total_pages(), 1);
    }

    #[test]
    fn page_size_growth_preserves_selected_account_identity() {
        let mut app = make_refresh_test_app_on_page(navigation_accounts(28), 2, 1);

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
    fn on_activated_waits_one_second_before_starting_usage_and_preview_fetch() {
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
                expires_at: Some(now + Duration::days(7)),
            },
        ];
        let mut app = make_refresh_test_app(accounts, 0);

        app.on_activated();

        assert_eq!(app.activation_delay_ticks, Some(ACTIVATION_DELAY_TICKS));
        assert!(!app.usage_task_active);
        assert!(!app.preview_task_active);

        for _ in 0..(ACTIVATION_DELAY_TICKS - 1) {
            assert!(app.on_tick());
            assert!(!app.usage_task_active);
            assert!(!app.preview_task_active);
        }

        assert!(app.on_tick());
        assert!(app.usage_task_active);
        assert!(app.preview_task_active);
    }

    #[test]
    fn preview_keys_to_refresh_reuses_fresh_cache_but_refreshes_stale_or_missing_entries() {
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
                expires_at: Some(now + Duration::days(7)),
            },
            OpenCodeAuthItem {
                name: "alt".to_string(),
                account_id: Some("acc-alt".to_string()),
                email: None,
                plan_type: Some("PLUS".to_string()),
                is_current: false,
                is_virtual: false,
                saved_at: Some(now - Duration::days(2)),
                last_used: Some(now - Duration::hours(2)),
                expires_at: Some(now + Duration::days(7)),
            },
        ];
        let mut app = make_refresh_test_app(accounts, 0);
        app.preview_cache.insert(
            "main".to_string(),
            QuotaPreviewEntry {
                quota: CodexAccountQuota {
                    account_name: "main".to_string(),
                    email: None,
                    quota: None,
                    error: Some("fresh".to_string()),
                    fetched_at: Utc::now(),
                },
            },
        );
        app.preview_cache.insert(
            "alt".to_string(),
            QuotaPreviewEntry {
                quota: CodexAccountQuota {
                    account_name: "alt".to_string(),
                    email: None,
                    quota: None,
                    error: Some("stale".to_string()),
                    fetched_at: Utc::now() - chrono::Duration::seconds(PREVIEW_TTL_SECS + 5),
                },
            },
        );

        let pending = app.preview_keys_to_refresh(false);
        assert_eq!(pending, vec!["alt".to_string()]);
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
