// TUI application state — Tab-based dispatch

use crate::tui::CompletedAction;
use crate::tui::action::Action;
use crate::tui::i18n::{self, Message};
use crate::tui::toast::{Toast, ToastManager};
use ccr_cli::application::profile_activity::{
    ProfileActivity, ProfileActivityError, ProfileActivitySnapshot,
};
use ccr_cli::managers::{TuiConfig, TuiConfigManager, TuiTabId};
use ccr_cli::models::{ClaudeRuntimeSummary, CodexRuntimeSummary};
use ccr_cli::models::{Platform, PlatformConfig, ProfileConfig};
use ccr_cli::platforms::GrokActivationState;
use ccr_core::core::error::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use indexmap::IndexMap;
use ratatui::Frame;
use ratatui::layout::Rect;
use std::cell::Cell;
use std::sync::Arc;

use super::claude_auth::{ClaudeAuthActionRecord, ClaudeAuthApp};
use super::codex_auth::CodexAuthApp;
use super::grok_auth::GrokAuthApp;
use super::pagination::{DEFAULT_PAGE_SIZE, page_for_index, page_slice, total_pages};
use super::profile_tasks::{
    MutationOutcome, MutationRequest, ProfileCompletion, ProfileSnapshot, ProfileTasks, ReadIssue,
};
use super::runtime::{AsyncTaskExecutor, TuiApp};
use super::ui;
use super::usage::UsageApp;

/// A single profile entry for display
#[derive(Debug, Clone)]
pub struct ProfileItem {
    pub name: String,
    pub description: Option<String>,
    pub is_current: bool,
}

/// Distinguishes tab types for the same platform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabVariant {
    /// Standard profile switching (Claude, Codex, Grok)
    Profile,
    /// Claude official subscription account/auth management
    ClaudeAuth,
    /// Codex account/auth management
    CodexAuth,
    /// Grok official session status and auth off
    GrokAuth,
}

/// A tab representing one platform with its profiles loaded
pub struct PlatformTab {
    pub platform: Platform,
    pub variant: TabVariant,
    pub label: String,
    pub profiles: Vec<ProfileItem>,
    pub profile_configs: IndexMap<String, ProfileConfig>,
    pub profile_load_error: Option<String>,
    pub current_profile_error: Option<String>,
    pub claude_runtime_summary: Option<ClaudeRuntimeSummary>,
    pub codex_runtime_summary: Option<CodexRuntimeSummary>,
    pub instance: Option<Arc<dyn PlatformConfig>>,
    pub profile_loading: bool,
    pub profile_loaded: bool,
    pub grok_activation_state: Option<GrokActivationState>,
    pub runtime_error: Option<String>,
    pub activity: Option<std::result::Result<ProfileActivitySnapshot, ProfileActivityError>>,
    /// 离开该 tab 时保存的选中快照；None = 从未访问过（per-tab 会话级记忆）
    pub saved_selection: Option<TabSelection>,
}

impl PlatformTab {
    fn empty(platform: Platform, variant: TabVariant) -> Self {
        Self {
            platform,
            variant,
            label: format!(
                "{} {}",
                platform.short_name(),
                if variant == TabVariant::Profile {
                    "Profile"
                } else {
                    "Auth"
                }
            ),
            profiles: Vec::new(),
            profile_configs: IndexMap::new(),
            profile_load_error: None,
            current_profile_error: None,
            claude_runtime_summary: None,
            codex_runtime_summary: None,
            instance: None,
            profile_loading: false,
            profile_loaded: false,
            grok_activation_state: None,
            runtime_error: None,
            activity: None,
            saved_selection: None,
        }
    }

    pub fn display_label(&self) -> &str {
        match (self.platform, self.variant) {
            (Platform::Claude, TabVariant::Profile) => {
                crate::tui_text!("Claude Profile", "Claude 配置")
            }
            (Platform::Codex, TabVariant::Profile) => {
                crate::tui_text!("Codex Profile", "Codex 配置")
            }
            (Platform::Grok, TabVariant::Profile) => {
                crate::tui_text!("Grok Profile", "Grok 配置")
            }
            (_, TabVariant::ClaudeAuth) => {
                crate::tui_text!("Claude Auth", "Claude 认证")
            }
            (_, TabVariant::CodexAuth) => crate::tui_text!("Codex Auth", "Codex 认证"),
            (_, TabVariant::GrokAuth) => crate::tui_text!("Grok Auth", "Grok 认证"),
            _ => self.label.as_str(),
        }
    }

    pub fn compact_display_label(&self) -> &str {
        match (self.platform, self.variant) {
            (Platform::Claude, TabVariant::Profile) => "Claude",
            (Platform::Codex, TabVariant::Profile) => "Codex",
            (Platform::Grok, TabVariant::Profile) => "Grok",
            _ => self.display_label(),
        }
    }
}

/// 单个 tab 的选中状态快照，用于 per-tab 记忆光标位置
#[derive(Clone)]
pub struct TabSelection {
    pub selected_index: usize,
    pub current_page: usize,
    pub selected_profile_name: Option<String>,
}

fn format_issue(location: String, error: &dyn std::fmt::Display) -> String {
    let error = error.to_string();
    let error = strip_duplicate_issue_location(&error, &location);
    let location = indent_issue_value(&location);
    let error = indent_issue_value(&error);

    format!(
        "{}:\n  {location}\n\n{}:\n  {error}",
        i18n::text(Message::Where),
        i18n::text(Message::What)
    )
}

fn strip_duplicate_issue_location(error: &str, location: &str) -> String {
    let location_prefix = format!("{location}: ");
    if let Some(reason) = error.strip_prefix(&location_prefix) {
        return reason.to_string();
    }

    if let Some((category, detail)) = error.split_once(": ")
        && let Some(reason) = detail.strip_prefix(&location_prefix)
    {
        return format!("{category}: {reason}");
    }

    error.to_string()
}

fn indent_issue_value(value: &str) -> String {
    value.replace('\n', "\n  ")
}

fn tab_config_id(tab: &PlatformTab) -> Option<TuiTabId> {
    match (tab.platform, tab.variant) {
        (Platform::Codex, TabVariant::Profile) => Some(TuiTabId::CodexProfile),
        (Platform::Grok, TabVariant::Profile) => Some(TuiTabId::GrokProfile),
        (Platform::Claude, TabVariant::Profile) => Some(TuiTabId::ClaudeProfile),
        (_, TabVariant::CodexAuth) => Some(TuiTabId::CodexAuth),
        (_, TabVariant::ClaudeAuth) => Some(TuiTabId::ClaudeAuth),
        (_, TabVariant::GrokAuth) => Some(TuiTabId::GrokAuth),
        (_, TabVariant::Profile) => None,
    }
}

pub(super) fn load_tui_config() -> TuiConfig {
    match TuiConfigManager::with_default() {
        Ok(manager) => manager.load_or_default(),
        Err(error) => {
            tracing::warn!(
                "Failed to resolve TUI config path: {}. Falling back to default config.",
                error
            );
            TuiConfig::default()
        }
    }
}

fn reorder_tabs(mut tabs: Vec<PlatformTab>, tab_order: &[TuiTabId]) -> Vec<PlatformTab> {
    let mut reordered = Vec::with_capacity(tabs.len());

    for tab_id in tab_order {
        if let Some(index) = tabs
            .iter()
            .position(|tab| tab_config_id(tab) == Some(*tab_id))
        {
            reordered.push(tabs.remove(index));
        }
    }

    reordered.extend(tabs);
    reordered
}

/// Main TUI application state
pub struct App {
    /// Dynamic list of platform tabs
    pub tabs: Vec<PlatformTab>,
    /// Index of the currently active tab
    pub active_tab: usize,
    /// Index of the selected profile within the current page
    pub selected_index: usize,
    /// Current page number (0-based)
    pub current_page: usize,
    /// 当前列表可见行数驱动的分页容量
    pub page_size: usize,
    /// 当前选中的 profile 名称（跨刷新保持同步）
    pub selected_profile_name: Option<String>,
    /// Toast notification manager
    pub toasts: ToastManager,
    /// Last applied profile info (platform_name, profile_name, success, error)
    pub last_applied: Option<(String, String, bool, Option<String>)>,
    /// Embedded Claude Auth app (lazy initialized)
    pub claude_auth_app: Option<ClaudeAuthApp>,
    /// Last Claude Auth initialization error for placeholder rendering
    pub claude_auth_error: Option<String>,
    /// Last Claude auth action info (action_type, account_name, success, error, warnings)
    pub last_claude_action: Option<ClaudeAuthActionRecord>,
    /// Embedded Codex Auth app (lazy initialized)
    pub codex_auth_app: Option<CodexAuthApp>,
    /// Last Codex Auth initialization error for placeholder rendering
    pub codex_auth_error: Option<String>,
    /// Last codex auth action info (action_type, account_name, success, error)
    pub last_codex_action: Option<(CompletedAction, String, bool, Option<String>)>,
    /// Embedded Grok Auth app (lazy initialized)
    pub grok_auth_app: Option<GrokAuthApp>,
    /// Last Grok Auth initialization error for placeholder rendering
    pub grok_auth_error: Option<String>,
    /// Last Grok auth off result (success, error)
    pub last_grok_action: Option<(CompletedAction, String, bool, Option<String>)>,
    /// 用量数据引擎(懒初始化):后台加载 provider 用量,详情面板纯内存查找
    pub usage_app: Option<UsageApp>,
    /// 🖱️ Cached header (tab bar) area for mouse hit-testing
    pub header_area: Cell<Option<Rect>>,
    /// 🖱️ Cached profile list area for mouse hit-testing
    pub list_area: Cell<Option<Rect>>,
    /// 🖱️ Cached profile detail area for mouse-wheel detail scrolling
    pub detail_area: Cell<Option<Rect>>,
    /// Vertical scroll offset for the selected profile detail panel
    pub profile_detail_scroll: u16,
    /// Whether unset profile detail fields are expanded (default: collapsed summary)
    pub profile_details_expanded: bool,
    /// 异步后台任务执行器
    pub(crate) task_executor: AsyncTaskExecutor,
    pub(crate) profile_tasks: ProfileTasks,
}

impl App {
    fn remember_selected_profile(&mut self) {
        self.selected_profile_name = self.selected_profile().map(|profile| profile.name.clone());
    }

    fn reset_profile_detail_scroll(&mut self) {
        self.profile_detail_scroll = 0;
    }

    fn current_profile_global_index(&self) -> Option<usize> {
        self.current_profiles()
            .iter()
            .position(|profile| profile.is_current)
    }

    fn selected_profile_global_index(&self) -> Option<usize> {
        let page_len = self.current_page_profiles().len();
        if page_len == 0 {
            return None;
        }

        let clamped_index = self.selected_index.min(page_len.saturating_sub(1));
        let index = self.current_page * self.page_size + clamped_index;
        (index < self.current_profiles().len()).then_some(index)
    }

    fn move_to_page(&mut self, new_page: usize) {
        let old_relative_index = self.selected_index;
        self.current_page = new_page;
        let page_len = self.current_page_profiles().len();
        self.selected_index = if page_len == 0 {
            0
        } else {
            old_relative_index.min(page_len.saturating_sub(1))
        };
        self.remember_selected_profile();
    }

    pub fn selected_profile(&self) -> Option<&ProfileItem> {
        self.selected_profile_global_index()
            .and_then(|idx| self.current_profiles().get(idx))
    }

    pub fn selected_profile_config(&self) -> Option<&ProfileConfig> {
        let profile_name = self.selected_profile()?.name.as_str();
        self.tabs[self.active_tab].profile_configs.get(profile_name)
    }

    pub fn current_profile_load_error(&self) -> Option<&str> {
        self.tabs[self.active_tab].profile_load_error.as_deref()
    }

    pub fn current_profile_status_error(&self) -> Option<&str> {
        let tab = &self.tabs[self.active_tab];
        tab.current_profile_error
            .as_deref()
            .or(tab.runtime_error.as_deref())
    }

    pub fn current_codex_runtime_summary(&self) -> Option<&CodexRuntimeSummary> {
        self.tabs[self.active_tab].codex_runtime_summary.as_ref()
    }

    pub fn current_claude_runtime_summary(&self) -> Option<&ClaudeRuntimeSummary> {
        self.tabs[self.active_tab].claude_runtime_summary.as_ref()
    }

    pub fn current_grok_activation_state(&self) -> Option<&GrokActivationState> {
        self.tabs[self.active_tab].grok_activation_state.as_ref()
    }

    pub fn current_profile_loading(&self) -> bool {
        self.tabs[self.active_tab].profile_loading
    }

    pub fn profile_mutation_pending(&self, platform: Platform, name: Option<&str>) -> bool {
        self.profile_tasks.mutation_pending(platform, name)
    }

    pub fn profile_activity(&self, name: &str) -> Option<&ProfileActivity> {
        self.current_activity_snapshot()?
            .as_ref()
            .ok()?
            .profiles
            .get(name)
    }

    pub fn current_activity_snapshot(
        &self,
    ) -> Option<&std::result::Result<ProfileActivitySnapshot, ProfileActivityError>> {
        self.tabs[self.active_tab].activity.as_ref()
    }

    fn ensure_active_profile_load(&mut self) {
        let tab = &self.tabs[self.active_tab];
        if tab.variant == TabVariant::Profile && !tab.profile_loaded && !tab.profile_loading {
            if self.profile_tasks.available() {
                self.profile_tasks.request_read(tab.platform, false);
            } else {
                let tab = &mut self.tabs[self.active_tab];
                tab.profile_loaded = true;
                tab.profile_load_error = Some(
                    crate::tui_text!(
                        "Background profile loading unavailable",
                        "后台配置加载不可用"
                    )
                    .to_string(),
                );
            }
            self.sync_loading_flags();
        }
    }

    fn schedule_activity_time_refresh(&mut self, now: chrono::DateTime<chrono::Utc>) {
        self.profile_tasks.time_refresh = self
            .current_activity_snapshot()
            .and_then(|snapshot| snapshot.as_ref().ok())
            .into_iter()
            .flat_map(|snapshot| snapshot.profiles.values())
            .filter_map(|activity| activity.recent_activations.first())
            .filter_map(|timestamp| {
                let age = now.signed_duration_since(*timestamp).num_seconds();
                let boundary = if age < 0 {
                    0
                } else if age < 60 {
                    60
                } else if age < 3600 {
                    (age / 60 + 1) * 60
                } else if age < 86400 {
                    (age / 3600 + 1) * 3600
                } else if age < 604800 {
                    (age / 86400 + 1) * 86400
                } else {
                    return None;
                };
                timestamp.checked_add_signed(chrono::Duration::seconds(boundary))
            })
            .min();
    }

    fn sync_loading_flags(&mut self) {
        for tab in &mut self.tabs {
            tab.profile_loading =
                tab.variant == TabVariant::Profile && self.profile_tasks.reading(tab.platform);
        }
    }

    fn apply_profile_snapshot(&mut self, platform: Platform, snapshot: ProfileSnapshot) {
        let Some(index) = self
            .tabs
            .iter()
            .position(|tab| tab.platform == platform && tab.variant == TabVariant::Profile)
        else {
            return;
        };
        let tab = &mut self.tabs[index];
        let was_loaded = tab.profile_loaded;
        tab.profile_loaded = true;
        let current = match snapshot.current {
            Ok(current) => {
                tab.current_profile_error = None;
                current
            }
            Err(issue) => {
                tab.current_profile_error = Some(format_read_issue(issue));
                None
            }
        };
        match snapshot.configs {
            Ok(configs) => {
                tab.profiles = configs
                    .iter()
                    .map(|(name, config)| ProfileItem {
                        name: name.clone(),
                        description: config.description.clone(),
                        is_current: current.as_ref() == Some(name),
                    })
                    .collect();
                tab.profile_configs = configs;
                tab.profile_load_error = None;
            }
            Err(issue) => {
                // Retain the previous usable list, with its refresh failure visible.
                tab.profile_load_error = Some(format_read_issue(issue));
                for profile in &mut tab.profiles {
                    profile.is_current = current.as_ref() == Some(&profile.name);
                }
            }
        }
        tab.instance = snapshot.instance;
        tab.claude_runtime_summary = snapshot.claude_runtime.clone();
        tab.codex_runtime_summary = snapshot.codex_runtime.clone();
        tab.grok_activation_state = snapshot.grok_activation;
        tab.runtime_error = snapshot.runtime_error;
        if snapshot.activity.is_some() {
            tab.activity = snapshot.activity;
        }
        // Embedded auth keeps its selection; only the dependent runtime read model changes.
        if platform == Platform::Codex
            && let Some(auth) = self.codex_auth_app.as_mut()
        {
            auth.runtime_summary = snapshot.codex_runtime;
        }
        if platform == Platform::Claude
            && let Some(auth) = self.claude_auth_app.as_mut()
        {
            auth.runtime_summary = snapshot.claude_runtime;
        }
        if index == self.active_tab {
            if was_loaded {
                self.align_selection_by_name();
            } else {
                self.focus_current_profile();
            }
            self.schedule_activity_time_refresh(chrono::Utc::now());
        }
    }

    fn pump_profile_tasks(&mut self) -> bool {
        let completed = self.profile_tasks.drain();
        let changed = !completed.is_empty();
        for completion in completed {
            match completion {
                ProfileCompletion::Read(platform, Ok(snapshot)) => self.apply_profile_snapshot(platform, *snapshot),
                ProfileCompletion::Read(platform, Err(error)) => self.profile_read_failed(platform, error),
                ProfileCompletion::ReadInterrupted(platform) => self.profile_read_failed(platform,
                    crate::tui_text!("Profile read interrupted; reload to retry", "配置读取中断，可重新加载").to_string()),
                ProfileCompletion::Mutation(request, result) => self.complete_profile_mutation(request, result),
                ProfileCompletion::MutationInterrupted(request) => self.complete_profile_mutation(request, Err(
                    crate::tui_text!("Operation outcome unknown; state is being reread. Do not repeat activation.", "操作结果未知，正在重新读取状态；请勿重复激活。").to_string())),
            }
        }
        self.sync_loading_flags();
        changed
    }

    fn profile_read_failed(&mut self, platform: Platform, error: String) {
        if let Some(tab) = self
            .tabs
            .iter_mut()
            .find(|tab| tab.platform == platform && tab.variant == TabVariant::Profile)
        {
            tab.profile_loaded = true;
            tab.profile_load_error = Some(error.clone());
            tab.current_profile_error = Some(error);
            tab.claude_runtime_summary = None;
            tab.codex_runtime_summary = None;
            tab.grok_activation_state = None;
            for profile in &mut tab.profiles {
                profile.is_current = false;
            }
        }
        if platform == Platform::Codex
            && let Some(auth) = self.codex_auth_app.as_mut()
        {
            auth.runtime_summary = None;
        }
        if platform == Platform::Claude
            && let Some(auth) = self.claude_auth_app.as_mut()
        {
            auth.runtime_summary = None;
        }
    }

    fn complete_profile_mutation(
        &mut self,
        request: MutationRequest,
        result: std::result::Result<MutationOutcome, String>,
    ) {
        let label = match request.platform() {
            Platform::Claude => "Claude Profile",
            Platform::Codex => "Codex Profile",
            _ => "Grok Profile",
        }
        .to_string();
        match result {
            Ok(MutationOutcome::Applied(outcome)) => {
                let toast = super::profile_backend::presentation(&outcome);
                self.last_applied = Some((
                    label,
                    outcome.profile.clone(),
                    outcome.activation_committed,
                    (!outcome.activation_committed || !outcome.warnings.is_empty())
                        .then(|| toast.message.clone()),
                ));
                self.toasts.push(toast);
            }
            Ok(MutationOutcome::Off(outcome)) => {
                let toast = if !outcome.warnings.is_empty() {
                    Toast::warning(crate::tui_text!(
                        "Profile mode exited; ancillary work needs attention",
                        "已退出配置模式，附属操作需要处理"
                    ))
                } else if outcome.changed {
                    Toast::success(crate::tui_text!(
                        "Exited profile mode and cleared login leftovers",
                        "已退出 Profile 并清理登录残留"
                    ))
                } else {
                    Toast::info(crate::tui_text!(
                        "No profile leftovers to clear",
                        "当前没有可清理的 Profile 残留"
                    ))
                };
                self.toasts.push(toast);
            }
            Err(error) => {
                if let Some(name) = request.profile() {
                    self.last_applied = Some((label, name.to_string(), false, Some(error.clone())));
                }
                self.toasts.push(Toast::error(crate::tui_format!(
                    "Profile operation failed: {}",
                    "配置操作失败：{}",
                    error
                )));
            }
        }
    }

    fn sync_selection_to_profile_name(&mut self) {
        let total = self.current_profiles().len();
        if total == 0 {
            self.current_page = 0;
            self.selected_index = 0;
            self.selected_profile_name = None;
            self.reset_profile_detail_scroll();
            return;
        }

        let preferred_index = if self.current_platform() == Platform::Codex {
            self.current_profile_global_index()
                .or_else(|| {
                    self.selected_profile_name.as_ref().and_then(|name| {
                        self.current_profiles()
                            .iter()
                            .position(|profile| profile.name == *name)
                    })
                })
                .unwrap_or(0)
        } else {
            self.selected_profile_name
                .as_ref()
                .and_then(|name| {
                    self.current_profiles()
                        .iter()
                        .position(|profile| profile.name == *name)
                })
                .or_else(|| self.selected_profile_global_index())
                .unwrap_or(0)
                .min(total.saturating_sub(1))
        };

        self.current_page = page_for_index(preferred_index, self.page_size);
        self.selected_index = super::pagination::index_in_page(preferred_index, self.page_size);
        self.remember_selected_profile();
        self.reset_profile_detail_scroll();
    }

    /// 把光标定位到当前 tab 的已启用项（is_current）；无已启用项则定位第 0 项。所有平台统一。
    fn focus_current_profile(&mut self) {
        let total = self.current_profiles().len();
        if total == 0 {
            self.current_page = 0;
            self.selected_index = 0;
            self.selected_profile_name = None;
            self.reset_profile_detail_scroll();
            return;
        }
        let target = self.current_profile_global_index().unwrap_or(0);
        self.current_page = page_for_index(target, self.page_size);
        self.selected_index = super::pagination::index_in_page(target, self.page_size);
        self.remember_selected_profile();
        self.reset_profile_detail_scroll();
    }

    /// 离开 tab：把当前工作副本写入该 tab 的选中快照（per-tab 记忆）
    fn save_active_tab_selection(&mut self) {
        if self.tabs[self.active_tab].variant == TabVariant::Profile
            && !self.tabs[self.active_tab].profile_loaded
            && self.current_profiles().is_empty()
        {
            return;
        }
        self.remember_selected_profile();
        self.tabs[self.active_tab].saved_selection = Some(TabSelection {
            selected_index: self.selected_index,
            current_page: self.current_page,
            selected_profile_name: self.selected_profile_name.clone(),
        });
    }

    /// 进入 tab：有快照则恢复并按名对齐（防 reload 后越界）；无快照则定位已启用项
    fn restore_active_tab_selection(&mut self) {
        match self.tabs[self.active_tab].saved_selection.clone() {
            Some(saved) => {
                self.current_page = saved.current_page;
                self.selected_index = saved.selected_index;
                self.selected_profile_name = saved.selected_profile_name;
                self.align_selection_by_name();
            }
            None => self.focus_current_profile(),
        }
    }

    /// 按 selected_profile_name 在当前 tab 重新对齐光标（无平台差异；name 失效则按残留索引 clamp）。
    /// 与 sync_selection_to_profile_name 的差异：本方法不让 Codex 的 is_current 抢占已恢复的快照位置。
    fn align_selection_by_name(&mut self) {
        let total = self.current_profiles().len();
        if total == 0 {
            self.current_page = 0;
            self.selected_index = 0;
            self.selected_profile_name = None;
            self.reset_profile_detail_scroll();
            return;
        }
        let target = self
            .selected_profile_name
            .as_ref()
            .and_then(|name| {
                self.current_profiles()
                    .iter()
                    .position(|profile| profile.name == *name)
            })
            .or_else(|| self.selected_profile_global_index())
            .unwrap_or(0)
            .min(total - 1);
        self.current_page = page_for_index(target, self.page_size);
        self.selected_index = super::pagination::index_in_page(target, self.page_size);
        self.remember_selected_profile();
        self.reset_profile_detail_scroll();
    }

    /// Build the app with supported profile and auth tabs.
    #[allow(dead_code)]
    pub fn new() -> Result<Self> {
        Self::with_task_executor(AsyncTaskExecutor::from_current_or_test())
    }

    pub fn with_task_executor(task_executor: AsyncTaskExecutor) -> Result<Self> {
        let tui_config = load_tui_config();
        Self::with_task_executor_and_config(task_executor, tui_config)
    }

    pub(super) fn with_task_executor_and_config(
        task_executor: AsyncTaskExecutor,
        tui_config: TuiConfig,
    ) -> Result<Self> {
        let profile_tasks = ProfileTasks::new(task_executor.clone());
        Self::with_profile_tasks(task_executor, tui_config, profile_tasks)
    }

    fn with_profile_tasks(
        task_executor: AsyncTaskExecutor,
        tui_config: TuiConfig,
        profile_tasks: ProfileTasks,
    ) -> Result<Self> {
        i18n::set_language(tui_config.language);
        // No platform construction or profile/runtime I/O on the terminal thread.
        let tabs = [Platform::Claude, Platform::Codex, Platform::Grok]
            .into_iter()
            .flat_map(|platform| {
                let auth = match platform {
                    Platform::Claude => TabVariant::ClaudeAuth,
                    Platform::Codex => TabVariant::CodexAuth,
                    _ => TabVariant::GrokAuth,
                };
                [
                    PlatformTab::empty(platform, TabVariant::Profile),
                    PlatformTab::empty(platform, auth),
                ]
            })
            .collect();
        let tabs = reorder_tabs(tabs, &tui_config.tab_order);

        let mut app = Self {
            tabs,
            active_tab: 0,
            selected_index: 0,
            current_page: 0,
            page_size: DEFAULT_PAGE_SIZE,
            selected_profile_name: None,
            toasts: ToastManager::new(),
            last_applied: None,
            claude_auth_app: None,
            claude_auth_error: None,
            last_claude_action: None,
            codex_auth_app: None,
            codex_auth_error: None,
            last_codex_action: None,
            grok_auth_app: None,
            grok_auth_error: None,
            last_grok_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
            profile_details_expanded: false,
            profile_tasks,
            task_executor,
        };
        app.ensure_active_profile_load();
        app.focus_current_profile();
        Ok(app)
    }

    // -- Accessors --

    #[allow(dead_code)]
    pub fn current_tab(&self) -> &PlatformTab {
        &self.tabs[self.active_tab]
    }

    pub fn current_platform(&self) -> Platform {
        self.tabs[self.active_tab].platform
    }

    pub fn current_profiles(&self) -> &[ProfileItem] {
        &self.tabs[self.active_tab].profiles
    }

    pub fn current_page_profiles(&self) -> &[ProfileItem] {
        page_slice(self.current_profiles(), self.current_page, self.page_size)
    }

    pub fn total_pages(&self) -> usize {
        total_pages(self.current_profiles().len(), self.page_size)
    }

    pub fn sync_profile_page_size(&mut self, page_size: usize) {
        let page_size = page_size.max(1);
        if self.page_size == page_size {
            return;
        }

        let selected_name = self.selected_profile().map(|profile| profile.name.clone());
        self.page_size = page_size;

        if let Some(name) = selected_name
            && let Some(index) = self
                .current_profiles()
                .iter()
                .position(|profile| profile.name == name)
        {
            self.current_page = page_for_index(index, self.page_size);
            self.selected_index = super::pagination::index_in_page(index, self.page_size);
            self.selected_profile_name = Some(name);
            return;
        }

        self.sync_selection_to_profile_name();
    }

    // -- Key to Action mapping (pure logic, no side effects) --

    fn tab_key_action(key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::BackTab => Some(Action::PrevTab),
            KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => Some(Action::PrevTab),
            KeyCode::Tab => Some(Action::NextTab),
            _ => None,
        }
    }

    fn map_key(&self, key: KeyEvent) -> Action {
        // Ctrl+C always quits
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Action::Quit;
        }

        if let Some(action) = Self::tab_key_action(key) {
            return action;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Left | KeyCode::Char('h') => Action::PrevPage,
            KeyCode::Right | KeyCode::Char('l') => Action::NextPage,
            KeyCode::PageUp => Action::ScrollDetailsUp,
            KeyCode::PageDown => Action::ScrollDetailsDown,
            KeyCode::Up | KeyCode::Char('k') => Action::SelectPrev,
            KeyCode::Down | KeyCode::Char('j') => Action::SelectNext,
            KeyCode::Enter | KeyCode::Char(' ') => Action::ApplySelected,
            KeyCode::Char('o') | KeyCode::Char('O') => Action::ProfileOff,
            KeyCode::Char('x') => Action::ToggleDetailsExpanded,
            KeyCode::Char('r') => Action::Reload,
            _ => Action::Noop,
        }
    }

    // -- Action dispatch (executes side effects) --

    fn dispatch(&mut self, action: Action) -> Result<bool> {
        if self.profile_tasks.mutation_active() {
            let target = match action {
                Action::NextTab => Some((self.active_tab + 1) % self.tabs.len()),
                Action::PrevTab => Some((self.active_tab + self.tabs.len() - 1) % self.tabs.len()),
                Action::SwitchTab(index) if index < self.tabs.len() => Some(index),
                _ => None,
            };
            if target.is_some_and(|index| self.tabs[index].variant != TabVariant::Profile) {
                self.profile_busy_toast();
                return Ok(false);
            }
        }
        if matches!(
            action,
            Action::NextTab | Action::PrevTab | Action::SwitchTab(_)
        ) && let Some(grok) = self.grok_auth_app.as_mut()
        {
            if grok.is_busy() {
                return Ok(false);
            }
            grok.on_deactivated();
        }
        match action {
            Action::Noop => {}
            Action::Quit => {
                if self.profile_tasks.mutation_active() {
                    self.profile_tasks.exit_requested = true;
                    self.toasts.push(Toast::info(crate::tui_text!(
                        "Exiting after profile operation completes",
                        "配置操作完成后退出"
                    )));
                    return Ok(false);
                }
                return Ok(true);
            }
            Action::NextTab => {
                if self.tabs.len() > 1 {
                    self.save_active_tab_selection();
                    self.active_tab = (self.active_tab + 1) % self.tabs.len();
                    self.restore_active_tab_selection();
                    self.reset_profile_detail_scroll();
                    self.notify_tab_activated();
                }
            }
            Action::PrevTab => {
                if self.tabs.len() > 1 {
                    self.save_active_tab_selection();
                    self.active_tab = if self.active_tab == 0 {
                        self.tabs.len() - 1
                    } else {
                        self.active_tab - 1
                    };
                    self.restore_active_tab_selection();
                    self.reset_profile_detail_scroll();
                    self.notify_tab_activated();
                }
            }
            Action::SwitchTab(idx) => {
                if idx < self.tabs.len() {
                    self.save_active_tab_selection();
                    self.active_tab = idx;
                    self.restore_active_tab_selection();
                    self.reset_profile_detail_scroll();
                    self.notify_tab_activated();
                }
            }
            Action::SelectPrev => {
                let page_len = self.current_page_profiles().len();
                let next_index =
                    super::selection::previous_index_in_page(page_len, self.selected_index);
                if self.selected_index != next_index {
                    self.selected_index = next_index;
                    self.remember_selected_profile();
                    self.reset_profile_detail_scroll();
                }
            }
            Action::SelectNext => {
                let page_len = self.current_page_profiles().len();
                let next_index =
                    super::selection::next_index_in_page(page_len, self.selected_index);
                if self.selected_index != next_index {
                    self.selected_index = next_index;
                    self.remember_selected_profile();
                    self.reset_profile_detail_scroll();
                }
            }
            Action::SelectAt(idx) => {
                let page_len = self.current_page_profiles().len();
                if idx < page_len {
                    self.selected_index = idx;
                    self.remember_selected_profile();
                    self.reset_profile_detail_scroll();
                }
            }
            Action::PrevPage => {
                if self.current_page > 0 {
                    self.move_to_page(self.current_page - 1);
                    self.reset_profile_detail_scroll();
                }
            }
            Action::NextPage => {
                if self.current_page < self.total_pages() - 1 {
                    self.move_to_page(self.current_page + 1);
                    self.reset_profile_detail_scroll();
                }
            }
            Action::ScrollDetailsUp => {
                self.scroll_profile_details(-3);
            }
            Action::ScrollDetailsDown => {
                self.scroll_profile_details(3);
            }
            Action::ApplySelected => {
                self.apply_selected();
            }
            Action::ToggleDetailsExpanded => {
                self.profile_details_expanded = !self.profile_details_expanded;
            }
            Action::ProfileOff => {
                self.off_selected();
            }
            Action::Reload => {
                self.reload_profiles();
                // 用量数据集与 profiles 一同刷新(后台异步拉取,不阻塞渲染)
                self.ensure_usage_engine();
                if let Some(engine) = self.usage_app.as_mut() {
                    engine.refresh();
                }
                self.push_active_toast(Toast::info(crate::tui_text!(
                    "Refreshing profiles",
                    "正在刷新配置"
                )));
            }
        }
        Ok(false)
    }

    fn push_active_toast(&mut self, toast: Toast) {
        if self.is_claude_auth_tab()
            && let Some(app) = self.claude_auth_app.as_mut()
        {
            app.toasts.push(toast);
        } else if self.is_codex_auth_tab()
            && let Some(app) = self.codex_auth_app.as_mut()
        {
            app.toasts.push(toast);
        } else if self.is_grok_auth_tab()
            && let Some(app) = self.grok_auth_app.as_mut()
        {
            app.toasts.push(toast);
        } else {
            self.toasts.push(toast);
        }
    }

    fn toggle_language(&mut self) {
        self.toggle_language_with_manager(TuiConfigManager::with_default());
    }

    fn toggle_language_with_manager(&mut self, manager: Result<TuiConfigManager>) {
        let language = i18n::toggle_language();
        let save_result = manager.and_then(|manager| {
            let mut config = manager.load_or_default();
            config.language = language;
            manager.save(&config)
        });

        match save_result {
            Ok(()) => self.push_active_toast(Toast::success(i18n::language_changed(language))),
            Err(error) => {
                tracing::warn!("Failed to save TUI language: {error}");
                self.push_active_toast(Toast::error(i18n::language_save_failed(&error)));
            }
        }
    }

    fn is_language_switch_key(key: KeyEvent) -> bool {
        key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('l') | KeyCode::Char('L'))
    }

    fn scroll_profile_details(&mut self, delta: i16) {
        if delta.is_negative() {
            self.profile_detail_scroll = self
                .profile_detail_scroll
                .saturating_sub(delta.unsigned_abs());
        } else {
            self.profile_detail_scroll = self.profile_detail_scroll.saturating_add(delta as u16);
        }
    }

    fn apply_selected(&mut self) -> bool {
        if self.tabs[self.active_tab].variant != TabVariant::Profile {
            return false;
        }
        if self.profile_tasks.mutation_active() {
            self.profile_busy_toast();
            return false;
        }
        let Some(selected) = self.selected_profile() else {
            self.toasts.push(Toast::warning(crate::tui_text!(
                "No profiles available",
                "没有可用的配置"
            )));
            return false;
        };
        let request = ccr_cli::application::profile_lifecycle::ApplyProfileRequest::new(
            self.current_platform(),
            selected.name.clone(),
        );
        self.start_profile_mutation(MutationRequest::Apply(request))
    }

    fn off_selected(&mut self) {
        if self.tabs[self.active_tab].variant == TabVariant::Profile {
            self.start_profile_mutation(MutationRequest::Off(self.current_platform()));
        }
    }

    fn profile_busy_toast(&mut self) {
        self.toasts.push(Toast::info(crate::tui_text!(
            "Profile operation in progress",
            "配置操作正在进行"
        )));
    }

    fn start_profile_mutation(&mut self, request: MutationRequest) -> bool {
        let platform = request.platform();
        if self.profile_tasks.start_mutation(request) {
            if platform == Platform::Codex
                && let Some(auth) = self.codex_auth_app.as_mut()
            {
                auth.runtime_summary = None;
            }
            if platform == Platform::Claude
                && let Some(auth) = self.claude_auth_app.as_mut()
            {
                auth.runtime_summary = None;
            }
            self.profile_busy_toast();
            true
        } else {
            self.toasts.push(Toast::warning(crate::tui_text!(
                "Profile operation unavailable or already running",
                "配置操作不可用或正在进行"
            )));
            false
        }
    }

    fn reload_profiles(&mut self) {
        let platform = self.current_platform();
        self.profile_tasks.request_read(platform, true);
        self.sync_loading_flags();
    }

    // -- Tab helpers --

    /// Ensure Claude Auth app is initialized before interaction/rendering
    fn ensure_claude_auth_app(&mut self) {
        if self.claude_auth_app.is_some() {
            return;
        }

        match ClaudeAuthApp::new() {
            Ok(app) => {
                self.claude_auth_app = Some(app);
                self.claude_auth_error = None;
            }
            Err(e) => {
                let err = e.to_string();
                tracing::warn!("Failed to init ClaudeAuthApp: {}", err);
                self.claude_auth_error = Some(err.clone());
                self.toasts.push(Toast::error(crate::tui_format!(
                    "Failed to initialize Claude Auth: {}",
                    "Claude 认证初始化失败：{}",
                    err
                )));
            }
        }
    }

    /// Get mutable Claude Auth app, initializing it on demand
    fn claude_auth_app_mut(&mut self) -> Option<&mut ClaudeAuthApp> {
        self.ensure_claude_auth_app();
        self.claude_auth_app.as_mut()
    }

    /// Ensure Codex Auth app is initialized before interaction/rendering
    fn ensure_codex_auth_app(&mut self) {
        if self.codex_auth_app.is_some() {
            return;
        }

        match CodexAuthApp::with_task_executor(self.task_executor.clone()) {
            Ok(app) => {
                self.codex_auth_app = Some(app);
                self.codex_auth_error = None;
            }
            Err(e) => {
                let err = e.to_string();
                tracing::warn!("Failed to init CodexAuthApp: {}", err);
                self.codex_auth_error = Some(err.clone());
                self.toasts.push(Toast::error(crate::tui_format!(
                    "Failed to initialize Codex Auth: {}",
                    "Codex 认证初始化失败：{}",
                    err
                )));
            }
        }
    }

    /// Get mutable Codex Auth app, initializing it on demand
    fn codex_auth_app_mut(&mut self) -> Option<&mut CodexAuthApp> {
        self.ensure_codex_auth_app();
        self.codex_auth_app.as_mut()
    }

    /// Ensure Grok Auth app is initialized before interaction/rendering
    fn ensure_grok_auth_app(&mut self) {
        if self.grok_auth_app.is_some() {
            return;
        }

        match GrokAuthApp::with_task_executor(self.task_executor.clone()) {
            Ok(app) => {
                self.grok_auth_app = Some(app);
                self.grok_auth_error = None;
            }
            Err(e) => {
                let err = e.to_string();
                tracing::warn!("Failed to init GrokAuthApp: {}", err);
                self.grok_auth_error = Some(err.clone());
                self.toasts.push(Toast::error(crate::tui_format!(
                    "Failed to initialize Grok Auth: {}",
                    "Grok 认证初始化失败：{}",
                    err
                )));
            }
        }
    }

    /// Get mutable Grok Auth app, initializing it on demand
    fn grok_auth_app_mut(&mut self) -> Option<&mut GrokAuthApp> {
        self.ensure_grok_auth_app();
        self.grok_auth_app.as_mut()
    }

    /// Check if the currently active tab is the Claude Auth variant
    pub fn is_claude_auth_tab(&self) -> bool {
        self.tabs[self.active_tab].variant == TabVariant::ClaudeAuth
    }

    /// Check if the currently active tab is the Codex Auth variant
    pub fn is_codex_auth_tab(&self) -> bool {
        self.tabs[self.active_tab].variant == TabVariant::CodexAuth
    }

    /// Check if the currently active tab is the Grok Auth variant
    pub fn is_grok_auth_tab(&self) -> bool {
        self.tabs[self.active_tab].variant == TabVariant::GrokAuth
    }

    /// 确保用量数据引擎已就绪(懒初始化,构造无 I/O;数据由后台任务拉取)
    fn ensure_usage_engine(&mut self) {
        if self.usage_app.is_none() {
            self.usage_app = Some(UsageApp::with_task_executor(self.task_executor.clone()));
        }
    }

    /// Pre-select Claude Auth tab (for `ccr claude` entry)
    pub fn with_claude_auth_tab(mut self) -> Self {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.variant == TabVariant::ClaudeAuth)
        {
            self.remember_selected_profile();
            self.active_tab = idx;
            self.sync_selection_to_profile_name();
            self.notify_tab_activated();
        }
        self
    }

    /// Pre-select Codex Auth tab (for `ccr codex` entry)
    pub fn with_codex_tab(mut self) -> Self {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.platform == Platform::Codex && t.variant == TabVariant::CodexAuth)
        {
            self.remember_selected_profile();
            self.active_tab = idx;
            self.sync_selection_to_profile_name();
            self.notify_tab_activated();
        }
        self
    }

    /// Pre-select Grok Auth tab (for `ccr grok auth` entry)
    pub fn with_grok_auth_tab(mut self) -> Self {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.variant == TabVariant::GrokAuth)
        {
            self.remember_selected_profile();
            self.active_tab = idx;
            self.sync_selection_to_profile_name();
            self.notify_tab_activated();
        }
        self
    }

    /// Notify the active tab's sub-app that it became active
    fn notify_tab_activated(&mut self) {
        self.schedule_activity_time_refresh(chrono::Utc::now());
        let is_claude_auth = self.is_claude_auth_tab();
        let is_codex_auth = self.is_codex_auth_tab();
        let is_grok_auth = self.is_grok_auth_tab();

        if !is_claude_auth && !is_codex_auth && !is_grok_auth {
            self.ensure_active_profile_load();
            // profile tab 的选中定位已由切 tab 时的 restore/focus 完成，无需再 sync;
            // 用量引擎的激活由 on_tick 的 profile 分支统一驱动(含启动首帧)
            return;
        }

        if is_claude_auth {
            if let Some(claude_app) = self.claude_auth_app_mut() {
                claude_app.on_activated();
            }
            return;
        }

        if is_codex_auth {
            if let Some(codex_app) = self.codex_auth_app_mut() {
                codex_app.on_activated();
            }
            return;
        }

        if is_grok_auth && let Some(grok_app) = self.grok_auth_app_mut() {
            grok_app.on_activated();
        }
    }

    /// Delegate mouse event to embedded ClaudeAuthApp
    fn delegate_mouse_to_claude(&mut self, mouse: MouseEvent) -> Result<bool> {
        if let Some(claude_app) = self.claude_auth_app_mut() {
            claude_app.handle_mouse(mouse)
        } else {
            Ok(false)
        }
    }

    /// 🖱️ Delegate mouse event to embedded CodexAuthApp
    fn delegate_mouse_to_codex(&mut self, mouse: MouseEvent) -> Result<bool> {
        if let Some(codex_app) = self.codex_auth_app_mut() {
            codex_app.handle_mouse(mouse)
        } else {
            Ok(false)
        }
    }

    /// Delegate mouse event to embedded GrokAuthApp
    fn delegate_mouse_to_grok(&mut self, mouse: MouseEvent) -> Result<bool> {
        if let Some(grok_app) = self.grok_auth_app_mut() {
            grok_app.handle_mouse(mouse)
        } else {
            Ok(false)
        }
    }
}

// -- Mouse hit-test helpers (pure functions for testability) --

/// Calculate which list item was clicked based on mouse row and list area.
/// The cached rectangle is the exact data body, excluding headers and rules.
/// Returns `None` if click is outside the list content area.
pub(crate) fn list_hit_test(area: Rect, mouse_row: u16, page_len: usize) -> Option<usize> {
    if mouse_row >= area.y && mouse_row < area.y.saturating_add(area.height) {
        let clicked_row = (mouse_row - area.y) as usize;
        if clicked_row < page_len {
            return Some(clicked_row);
        }
    }
    None
}

fn point_in_rect(area: Rect, row: u16, column: u16) -> bool {
    row >= area.y
        && row < area.y.saturating_add(area.height)
        && column >= area.x
        && column < area.x.saturating_add(area.width)
}

// -- TuiApp trait implementation (tab-based dispatch) --

impl TuiApp for App {
    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        // Ctrl+C always quits the entire TUI
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return self.dispatch(Action::Quit);
        }

        if Self::is_language_switch_key(key) {
            self.toggle_language();
            return Ok(false);
        }

        if self
            .grok_auth_app
            .as_ref()
            .is_some_and(GrokAuthApp::is_busy)
        {
            return Ok(false);
        }

        if let Some(action) = Self::tab_key_action(key) {
            return self.dispatch(action);
        }

        if self.is_claude_auth_tab() {
            if let Some(claude_app) = self.claude_auth_app_mut() {
                let quit = claude_app.handle_key(key)?;
                if quit {
                    self.last_claude_action = claude_app.last_action.clone();
                    return Ok(true);
                }
            }
            Ok(false)
        } else if self.is_codex_auth_tab() {
            // Delegate all other keys to CodexAuthApp
            if let Some(codex_app) = self.codex_auth_app_mut() {
                let quit = codex_app.handle_key(key)?;
                if quit {
                    self.last_codex_action = codex_app.last_action.clone();
                    return Ok(true);
                }
            }
            Ok(false)
        } else if self.is_grok_auth_tab() {
            if let Some(grok_app) = self.grok_auth_app_mut() {
                let quit = grok_app.handle_key(key)?;
                if quit {
                    self.last_grok_action = grok_app.last_action.clone();
                    return Ok(true);
                }
            }
            Ok(false)
        } else {
            // Profile tabs (Claude / Codex Profile): key mapping + dispatch
            let action = self.map_key(key);
            self.dispatch(action)
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<bool> {
        if self
            .grok_auth_app
            .as_ref()
            .is_some_and(GrokAuthApp::is_busy)
        {
            return Ok(false);
        }
        match mouse.kind {
            // 🖱️ 左键点击
            MouseEventKind::Down(MouseButton::Left) => {
                // Tab 栏点击（所有 tab 共用）
                if let Some(header) = self.header_area.get() {
                    if let Some(tab_idx) = ui::tab_hit_index(self, header, mouse.column, mouse.row)
                    {
                        return self.dispatch(Action::SwitchTab(tab_idx));
                    }
                    // 点击了 Tab 栏区域但未触发切换，直接返回
                    if mouse.row >= header.y && mouse.row < header.y + header.height {
                        return Ok(false);
                    }
                }

                // Codex Auth tab: 委托给 CodexAuthApp
                if self.is_claude_auth_tab() {
                    return self.delegate_mouse_to_claude(mouse);
                }
                if self.is_codex_auth_tab() {
                    return self.delegate_mouse_to_codex(mouse);
                }
                if self.is_grok_auth_tab() {
                    return self.delegate_mouse_to_grok(mouse);
                }

                // Profile tabs (Claude / Codex Profile): 列表项点击
                if let Some(area) = self.list_area.get()
                    && point_in_rect(area, mouse.row, mouse.column)
                    && let Some(idx) =
                        list_hit_test(area, mouse.row, self.current_page_profiles().len())
                {
                    return self.dispatch(Action::SelectAt(idx));
                }
            }

            // 🖱️ 滚轮上
            MouseEventKind::ScrollUp => {
                if self.is_claude_auth_tab() {
                    return self.delegate_mouse_to_claude(mouse);
                }
                if self.is_codex_auth_tab() {
                    return self.delegate_mouse_to_codex(mouse);
                }
                if self.is_grok_auth_tab() {
                    return self.delegate_mouse_to_grok(mouse);
                }
                if let Some(area) = self.detail_area.get()
                    && point_in_rect(area, mouse.row, mouse.column)
                {
                    return self.dispatch(Action::ScrollDetailsUp);
                }
                return self.dispatch(Action::SelectPrev);
            }

            // 🖱️ 滚轮下
            MouseEventKind::ScrollDown => {
                if self.is_claude_auth_tab() {
                    return self.delegate_mouse_to_claude(mouse);
                }
                if self.is_codex_auth_tab() {
                    return self.delegate_mouse_to_codex(mouse);
                }
                if self.is_grok_auth_tab() {
                    return self.delegate_mouse_to_grok(mouse);
                }
                if let Some(area) = self.detail_area.get()
                    && point_in_rect(area, mouse.row, mouse.column)
                {
                    return self.dispatch(Action::ScrollDetailsDown);
                }
                return self.dispatch(Action::SelectNext);
            }

            _ => {}
        }
        Ok(false)
    }

    fn on_tick(&mut self) -> bool {
        let profile_redraw = self.pump_profile_tasks();
        let now = chrono::Utc::now();
        let time_redraw = self
            .profile_tasks
            .time_refresh
            .is_some_and(|deadline| now >= deadline);
        if time_redraw {
            self.schedule_activity_time_refresh(now);
        }
        let grok_redraw = if let Some(grok) = self.grok_auth_app.as_mut() {
            let redraw = grok.on_tick();
            self.last_grok_action = grok.last_action.clone();
            redraw
        } else {
            false
        };
        let redraw = if self.is_claude_auth_tab() {
            self.claude_auth_app.as_mut().is_some_and(|a| a.on_tick())
        } else if self.is_codex_auth_tab() {
            self.codex_auth_app.as_mut().is_some_and(|a| a.on_tick())
        } else if self.is_grok_auth_tab() {
            grok_redraw
        } else {
            // Profile tab: 首次进入(含启动首帧)激活用量引擎,此后每 tick 泵
            // 后台任务消息;on_activated 仅在 Idle 态生效,不会重复拉取
            self.ensure_usage_engine();
            let usage_redraw = self.usage_app.as_mut().is_some_and(|engine| {
                engine.on_activated();
                engine.tick()
            });
            self.toasts.tick() | usage_redraw
        };
        profile_redraw | time_redraw | redraw
    }

    fn has_pending_work(&self) -> bool {
        self.profile_tasks.pending() || self.usage_app.as_ref().is_some_and(UsageApp::is_pending)
    }

    fn poll_background(&mut self) -> bool {
        let profile_redraw = self.pump_profile_tasks();
        let usage_redraw = self
            .usage_app
            .as_mut()
            .is_some_and(UsageApp::poll_background);
        profile_redraw | usage_redraw
    }

    fn ready_to_exit(&self) -> bool {
        self.profile_tasks.ready_to_exit()
    }

    fn render(&mut self, frame: &mut Frame) {
        ui::draw(frame, self);
    }
}

fn format_read_issue(issue: ReadIssue) -> String {
    let location = issue
        .locations
        .into_iter()
        .enumerate()
        .map(|(index, path)| {
            if index == 0 {
                path
            } else {
                format!("{}: {}", i18n::text(Message::Fallback), path)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    format_issue(location, &issue.reason)
}

// ═══════════════════════════════════════════════════════════
// Unit tests for mouse hit-testing pure functions
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use ccr_cli::managers::TuiLanguage;
    use ccr_cli::models::Platform;
    use ccr_cli::models::ProfileConfig;
    use ccr_core::core::error::{CcrError, Result};
    use std::path::PathBuf;
    use std::sync::Arc;

    fn async_snapshot(name: &str) -> ProfileSnapshot {
        ProfileSnapshot {
            instance: None,
            configs: Ok(IndexMap::from([
                (name.into(), ProfileConfig::default()),
                ("other".into(), ProfileConfig::default()),
            ])),
            current: Ok(Some(name.into())),
            claude_runtime: None,
            codex_runtime: None,
            grok_activation: None,
            runtime_error: None,
            activity: Some(Ok(ProfileActivitySnapshot::default())),
        }
    }

    fn stub_usage(app: &mut App) {
        let mut usage = UsageApp::with_loader(
            AsyncTaskExecutor::Disabled,
            Arc::new(|| panic!("no usage I/O in Profile fixture")),
        );
        usage.state = super::super::usage::app::UsageLoadState::Empty;
        app.usage_app = Some(usage);
    }

    fn pump_until(app: &mut App, condition: impl Fn(&App) -> bool) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !condition(app) {
            app.on_tick();
            assert!(
                std::time::Instant::now() < deadline,
                "profile worker did not finish"
            );
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    #[test]
    fn profile_startup_loads_active_first_and_navigation_has_no_reads() {
        use std::sync::{Mutex, mpsc};
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let release_rx = Mutex::new(release_rx);
        let reads = Arc::new(Mutex::new(Vec::new()));
        let counter = Arc::clone(&reads);
        let executor = AsyncTaskExecutor::from_current_or_test();
        let tasks = ProfileTasks::with_backends(
            executor.clone(),
            Arc::new(move |platform| {
                counter.lock().unwrap().push(platform);
                if platform == Platform::Codex {
                    started_tx.send(()).unwrap();
                    let _ = release_rx
                        .lock()
                        .unwrap()
                        .recv_timeout(std::time::Duration::from_secs(5));
                }
                let mut snapshot = async_snapshot("work");
                if platform == Platform::Codex
                    && let Ok(configs) = snapshot.configs.as_mut()
                {
                    configs.swap_indices(0, 1);
                }
                Ok(snapshot)
            }),
            Arc::new(|_| panic!("read fixture must not mutate")),
        );
        let mut app = App::with_profile_tasks(executor, TuiConfig::default(), tasks).unwrap();
        stub_usage(&mut app);
        started_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        assert_eq!(*reads.lock().unwrap(), vec![Platform::Codex]);
        assert!(app.current_profile_loading());
        assert!(app.tabs.iter().all(|tab| tab.instance.is_none()));
        // The slow initial platform does not postpone another visible tab.
        let claude = app
            .tabs
            .iter()
            .position(|tab| tab.platform == Platform::Claude && tab.variant == TabVariant::Profile)
            .unwrap();
        app.dispatch(Action::SwitchTab(claude)).unwrap();
        pump_until(&mut app, |app| !app.current_profile_loading());
        assert_eq!(app.selected_profile_name.as_deref(), Some("work"));
        for _ in 0..1000 {
            app.dispatch(Action::SelectNext).unwrap();
            app.dispatch(Action::ScrollDetailsDown).unwrap();
        }
        assert_eq!(reads.lock().unwrap().len(), 2);
        release_tx.send(()).unwrap();
        pump_until(&mut app, |app| !app.profile_tasks.pending());
        assert_eq!(app.current_platform(), Platform::Claude);
        assert_eq!(reads.lock().unwrap().len(), 2);
        let codex = app
            .tabs
            .iter()
            .position(|tab| tab.platform == Platform::Codex && tab.variant == TabVariant::Profile)
            .unwrap();
        app.dispatch(Action::SwitchTab(codex)).unwrap();
        assert_eq!(app.selected_profile_name.as_deref(), Some("work"));
        assert_eq!(
            app.selected_index, 1,
            "visiting a loading tab must not replace initial current focus"
        );
        for _ in 0..1000 {
            assert!(!app.poll_background());
            assert!(!app.on_tick());
        }
        assert_eq!(reads.lock().unwrap().len(), 2);
    }

    #[test]
    fn profile_mutation_freezes_target_blocks_duplicates_and_defers_every_quit_key() {
        use ccr_cli::application::profile_lifecycle::{
            ProfileOutcome, ProfileStatus, ProfileWarning,
        };
        use std::sync::{Mutex, mpsc};
        for quit in [
            KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        ] {
            let (started_tx, started_rx) = mpsc::channel();
            let (release_tx, release_rx) = mpsc::channel();
            let release_rx = Mutex::new(release_rx);
            let mut app = app_with_profile_tabs(vec![
                profile_tab(
                    Platform::Codex,
                    "Codex Profile",
                    &["work", "other"],
                    Some("work"),
                ),
                profile_tab(
                    Platform::Claude,
                    "Claude Profile",
                    &["claude"],
                    Some("claude"),
                ),
                empty_tab(Platform::Codex, TabVariant::CodexAuth, "Codex Auth"),
            ]);
            app.profile_tasks = ProfileTasks::with_backends(
                app.task_executor.clone(),
                Arc::new(|_| panic!("deferred exit must not wait for a new read")),
                Arc::new(move |request| {
                    let MutationRequest::Apply(request) = request else {
                        panic!("apply expected");
                    };
                    started_tx
                        .send((
                            request.platform,
                            request.name.clone(),
                            request.operation_id.clone(),
                        ))
                        .unwrap();
                    let _ = release_rx
                        .lock()
                        .unwrap()
                        .recv_timeout(std::time::Duration::from_secs(5));
                    Ok(MutationOutcome::Applied(ProfileOutcome {
                        operation_id: request.operation_id,
                        platform: request.platform.short_name().into(),
                        profile: request.name,
                        previous_profile: None,
                        status: ProfileStatus::AppliedWithWarning,
                        activation_committed: true,
                        warnings: vec![ProfileWarning::HistoryFailed],
                        recovery_paths: vec![],
                    }))
                }),
            );
            stub_usage(&mut app);
            app.handle_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
                .unwrap();
            let (platform, target, operation) = started_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            assert_eq!(platform, Platform::Codex);
            assert_eq!(target, "work");
            assert!(!operation.is_empty());
            app.dispatch(Action::SelectNext).unwrap();
            assert_eq!(app.selected_profile().unwrap().name, "other");
            assert!(!app.apply_selected());
            app.off_selected();
            app.dispatch(Action::SwitchTab(1)).unwrap();
            assert_eq!(app.active_tab, 1);
            app.dispatch(Action::SwitchTab(2)).unwrap();
            assert_eq!(app.active_tab, 1, "Auth entry is gated during the write");
            assert!(!app.handle_key(quit).unwrap());
            assert!(!app.ready_to_exit());
            release_tx.send(()).unwrap();
            pump_until(&mut app, |app| app.ready_to_exit());
            let (platform, profile, committed, warning) = app.last_applied.as_ref().unwrap();
            assert_eq!(platform, "Codex Profile");
            assert_eq!(profile, "work");
            assert!(*committed);
            assert!(warning.is_some());
            assert!(started_rx.try_recv().is_err(), "exactly one write");
        }
    }

    #[test]
    fn profile_read_failure_preserves_list_and_selection_without_claiming_current() {
        let mut app = app_with_profile_tabs(vec![profile_tab(
            Platform::Codex,
            "Codex Profile",
            &["work", "other"],
            Some("work"),
        )]);
        app.dispatch(Action::SelectNext).unwrap();
        let mut snapshot = async_snapshot("work");
        snapshot.configs = Err(ReadIssue {
            locations: vec!["fixture/profiles.toml".into()],
            reason: "invalid fixture".into(),
        });
        snapshot.current = Err(ReadIssue {
            locations: vec!["fixture/registry.toml".into()],
            reason: "missing marker".into(),
        });
        app.apply_profile_snapshot(Platform::Codex, snapshot);
        assert_eq!(app.current_profiles().len(), 2);
        assert_eq!(app.selected_profile().unwrap().name, "other");
        assert!(
            app.current_profiles()
                .iter()
                .all(|profile| !profile.is_current)
        );
        assert!(
            app.current_profile_load_error()
                .unwrap()
                .contains("invalid fixture")
        );
    }

    #[test]
    fn failed_profile_worker_invalidates_runtime_but_preserves_browsing_state() {
        let (_dir, auth) = super::super::codex_auth::ui::tests::presentation_fixture();
        let mut app = app_with_profile_tabs(vec![profile_tab(
            Platform::Codex,
            "Codex Profile",
            &["work", "other"],
            Some("work"),
        )]);
        app.tabs[0].codex_runtime_summary = auth.runtime_summary.clone();
        app.codex_auth_app = Some(auth);
        app.dispatch(Action::SelectNext).unwrap();
        app.profile_read_failed(Platform::Codex, "fixture worker failed".into());
        assert_eq!(app.current_profiles().len(), 2);
        assert_eq!(app.selected_profile().unwrap().name, "other");
        assert!(
            app.current_profiles()
                .iter()
                .all(|profile| !profile.is_current)
        );
        assert!(app.current_codex_runtime_summary().is_none());
        assert!(
            app.codex_auth_app
                .as_ref()
                .unwrap()
                .runtime_summary
                .is_none()
        );
        assert_eq!(
            app.current_profile_status_error(),
            Some("fixture worker failed")
        );
    }

    #[test]
    fn profile_activity_clock_only_schedules_actual_relative_label_boundaries() {
        let mut app = app_with_profile_tabs(vec![profile_tab(
            Platform::Codex,
            "Codex Profile",
            &["work"],
            None,
        )]);
        let timestamp = chrono::DateTime::parse_from_rfc3339("2026-10-12T10:00:07Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let mut snapshot = ProfileActivitySnapshot::default();
        snapshot.profiles.insert(
            "work".into(),
            ProfileActivity {
                recent_activations: vec![timestamp],
                invalid_timestamps: 0,
            },
        );
        app.tabs[0].activity = Some(Ok(snapshot));
        for (age, next) in [
            (0, Some(60)),
            (59, Some(60)),
            (61, Some(120)),
            (3601, Some(7200)),
            (86401, Some(172800)),
            (604801, None),
        ] {
            app.schedule_activity_time_refresh(timestamp + chrono::Duration::seconds(age));
            assert_eq!(
                app.profile_tasks.time_refresh,
                next.map(|seconds| timestamp + chrono::Duration::seconds(seconds))
            );
        }
    }

    #[test]
    fn profile_completion_updates_initialized_codex_auth_without_resetting_selection() {
        let (_dir, mut auth) = super::super::codex_auth::ui::tests::presentation_fixture();
        auth.selected_index = 1;
        let mut summary = auth.runtime_summary.clone().unwrap();
        summary.current_profile_name = Some("fresh-profile".into());
        let mut app = app_with_profile_tabs(vec![profile_tab(
            Platform::Codex,
            "Codex Profile",
            &["work"],
            Some("work"),
        )]);
        app.codex_auth_app = Some(auth);
        let mut snapshot = async_snapshot("work");
        snapshot.codex_runtime = Some(summary.clone());
        app.apply_profile_snapshot(Platform::Codex, snapshot);
        assert_eq!(app.codex_auth_app.as_ref().unwrap().selected_index, 1);
        assert_eq!(
            app.codex_auth_app
                .as_ref()
                .unwrap()
                .runtime_summary
                .as_ref(),
            Some(&summary)
        );
        let mut failed = async_snapshot("work");
        failed.runtime_error = Some("fixture runtime read failed".into());
        app.apply_profile_snapshot(Platform::Codex, failed);
        assert!(
            app.codex_auth_app
                .as_ref()
                .unwrap()
                .runtime_summary
                .is_none()
        );
        assert_eq!(app.codex_auth_app.as_ref().unwrap().selected_index, 1);
    }

    #[test]
    #[ignore = "one-second synthetic writer timing; report only, never a CI latency assertion"]
    fn profile_one_second_writer_responsiveness_benchmark() {
        use ccr_cli::application::profile_lifecycle::{ProfileOutcome, ProfileStatus};
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::time::{Duration, Instant};
        let writes = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&writes);
        let finished = Arc::new(std::sync::Mutex::new(None));
        let worker_finished = Arc::clone(&finished);
        let mut app = profile_navigation_app(1000, 0, 0);
        app.profile_tasks = ProfileTasks::with_backends(
            app.task_executor.clone(),
            Arc::new(|_| {
                let mut snapshot = async_snapshot("profile-01");
                snapshot.configs = Ok((1..=1000)
                    .map(|index| (format!("profile-{index:02}"), ProfileConfig::default()))
                    .collect());
                Ok(snapshot)
            }),
            Arc::new(move |request| {
                counter.fetch_add(1, Ordering::SeqCst);
                std::thread::sleep(Duration::from_secs(1));
                let MutationRequest::Apply(request) = request else {
                    panic!("apply fixture");
                };
                let result = MutationOutcome::Applied(ProfileOutcome {
                    operation_id: request.operation_id,
                    platform: request.platform.short_name().into(),
                    profile: request.name,
                    previous_profile: None,
                    status: ProfileStatus::Applied,
                    activation_committed: true,
                    warnings: vec![],
                    recovery_paths: vec![],
                });
                *worker_finished.lock().unwrap() = Some(Instant::now());
                Ok(result)
            }),
        );
        stub_usage(&mut app);
        let mut ack = Vec::new();
        let mut navigation = Vec::new();
        let mut propagation = Vec::new();
        let mut complete = Vec::new();
        for _ in 0..30 {
            let started = Instant::now();
            assert!(app.apply_selected());
            ack.push(started.elapsed().as_micros());
            for _ in 0..1000 {
                let event = Instant::now();
                app.dispatch(Action::SelectNext).unwrap();
                navigation.push(event.elapsed().as_nanos());
            }
            // Pure resize and language state still change while the writer is blocked.
            app.sync_profile_page_size(25);
            i18n::toggle_language();
            assert!(app.profile_tasks.mutation_active());
            while app.profile_tasks.mutation_active() {
                std::thread::sleep(Duration::from_millis(50));
                app.poll_background();
            }
            complete.push(started.elapsed().as_micros());
            propagation.push(finished.lock().unwrap().unwrap().elapsed().as_micros());
            pump_until(&mut app, |app| !app.profile_tasks.pending());
        }
        ack.sort_unstable();
        navigation.sort_unstable();
        propagation.sort_unstable();
        complete.sort_unstable();
        assert_eq!(writes.load(Ordering::SeqCst), 30);
        println!(
            "One-second writer: samples=30 writes=30, ack p50={}us p95={}us; completion propagation p50={}us p95={}us; total p50={}us p95={}us; navigation samples={} p50={}ns p95={}ns",
            ack[15],
            ack[28],
            propagation[15],
            propagation[28],
            complete[15],
            complete[28],
            navigation.len(),
            navigation[navigation.len() / 2],
            navigation[navigation.len() * 95 / 100]
        );
        i18n::set_language(TuiLanguage::English);
    }

    // -- list_hit_test tests --

    #[test]
    fn list_hit_test_clicks_first_item() {
        // Exact body starts at y=5; header and rule are outside this rectangle.
        let area = Rect::new(0, 5, 40, 12);
        assert_eq!(list_hit_test(area, 5, 5), Some(0));
    }

    #[test]
    fn list_hit_test_clicks_third_item() {
        let area = Rect::new(0, 5, 40, 12);
        assert_eq!(list_hit_test(area, 7, 5), Some(2));
    }

    #[test]
    fn list_hit_test_ignores_header_before_body() {
        let area = Rect::new(0, 5, 40, 12);
        // A header/rule row preceding the body never selects a profile.
        assert_eq!(list_hit_test(area, 4, 5), None);
    }

    #[test]
    fn list_hit_test_ignores_row_after_body() {
        let area = Rect::new(0, 5, 40, 12);
        assert_eq!(list_hit_test(area, 17, 20), None);
        assert_eq!(list_hit_test(area, 16, 12), Some(11));
    }

    #[test]
    fn list_hit_test_ignores_click_beyond_items() {
        let area = Rect::new(0, 5, 40, 12);
        // Only 3 items in the list, click on row index 3 (4th position)
        assert_eq!(list_hit_test(area, 9, 3), None);
    }

    #[test]
    fn list_hit_test_ignores_blank_rows_when_page_has_fewer_items_than_space() {
        let area = Rect::new(0, 5, 40, 18);
        assert_eq!(list_hit_test(area, 15, 5), None);
    }

    #[test]
    fn list_hit_test_ignores_click_outside_area() {
        let area = Rect::new(0, 5, 40, 12);
        // Click above the area
        assert_eq!(list_hit_test(area, 2, 5), None);
        // Click below the area
        assert_eq!(list_hit_test(area, 20, 5), None);
    }

    #[test]
    fn list_hit_test_zero_height_area() {
        let area = Rect::new(0, 5, 40, 0);
        assert_eq!(list_hit_test(area, 5, 3), None);
    }

    #[test]
    fn ctrl_l_is_the_only_language_switch_key() {
        assert!(App::is_language_switch_key(key_with_modifiers(
            KeyCode::Char('l'),
            KeyModifiers::CONTROL
        )));
        assert!(App::is_language_switch_key(key_with_modifiers(
            KeyCode::Char('L'),
            KeyModifiers::CONTROL
        )));
        assert!(!App::is_language_switch_key(key_with_modifiers(
            KeyCode::Char('l'),
            KeyModifiers::NONE
        )));
    }

    #[test]
    fn language_toggle_persists_without_changing_app_selection() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("tui.toml");
        let manager = TuiConfigManager::new(&path);
        manager.save(&TuiConfig::default()).unwrap();

        let mut app = tab_switching_app(1);
        app.selected_index = 2;
        let active_tab = app.active_tab;
        let selected_index = app.selected_index;
        i18n::set_language(TuiLanguage::English);

        app.toggle_language_with_manager(Ok(manager));

        assert_eq!(i18n::active_language(), TuiLanguage::SimplifiedChinese);
        assert_eq!(app.active_tab, active_tab);
        assert_eq!(app.selected_index, selected_index);
        assert_eq!(
            TuiConfigManager::new(&path).load().unwrap().language,
            TuiLanguage::SimplifiedChinese
        );

        i18n::set_language(TuiLanguage::English);
    }

    #[test]
    fn language_toggle_remains_active_when_persistence_fails() {
        let mut app = tab_switching_app(1);
        i18n::set_language(TuiLanguage::English);

        app.toggle_language_with_manager(Err(CcrError::ConfigFormatInvalid(
            "storage unavailable".to_string(),
        )));

        assert_eq!(i18n::active_language(), TuiLanguage::SimplifiedChinese);
        let toast = app.toasts.active().unwrap();
        assert_eq!(toast.kind, crate::tui::toast::ToastKind::Error);
        assert!(toast.message.contains("无法保存设置"));

        i18n::set_language(TuiLanguage::English);
    }

    #[test]
    fn map_key_o_is_profile_off() {
        let app = profile_navigation_app(1, 0, 0);
        let key = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
        assert!(matches!(app.map_key(key), Action::ProfileOff));
        let apply = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert!(matches!(app.map_key(apply), Action::ApplySelected));
    }

    #[test]
    fn enter_and_space_both_map_to_apply_selected() {
        // Enter 与 Space 同义: 应用并停留, 不再存在单键 apply+quit
        // (Action 枚举已删除 ApplyAndQuit, 由编译期保证)
        let app = profile_navigation_app(1, 0, 0);
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let space = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        assert!(matches!(app.map_key(enter), Action::ApplySelected));
        assert!(matches!(app.map_key(space), Action::ApplySelected));
    }

    #[test]
    fn map_key_x_toggles_details_expansion() {
        let app = profile_navigation_app(1, 0, 0);
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(matches!(app.map_key(key), Action::ToggleDetailsExpanded));
    }

    #[test]
    fn toggle_details_expanded_flips_app_flag() {
        let mut app = profile_navigation_app(1, 0, 0);
        assert!(!app.profile_details_expanded);

        app.dispatch(Action::ToggleDetailsExpanded).unwrap();
        assert!(app.profile_details_expanded);

        app.dispatch(Action::ToggleDetailsExpanded).unwrap();
        assert!(!app.profile_details_expanded);
    }

    #[test]
    fn apply_selected_without_instance_does_not_mark_success() {
        let mut app = profile_navigation_app(1, 0, 0);
        assert!(!app.apply_selected());
        assert!(app.last_applied.is_none());
    }

    fn profile_navigation_app(count: usize, selected_index: usize, current_page: usize) -> App {
        let profiles = (1..=count)
            .map(|index| ProfileItem {
                name: format!("profile-{index:02}"),
                description: None,
                is_current: index == 1,
            })
            .collect();

        App {
            tabs: vec![PlatformTab {
                platform: Platform::Claude,
                variant: TabVariant::Profile,
                label: "Claude Code".to_string(),
                profiles,
                profile_configs: IndexMap::<String, ProfileConfig>::new(),
                profile_load_error: None,
                current_profile_error: None,
                claude_runtime_summary: None,
                codex_runtime_summary: None,
                profile_loading: false,
                profile_loaded: true,
                grok_activation_state: None,
                runtime_error: None,
                activity: None,
                instance: None,
                saved_selection: None,
            }],
            active_tab: 0,
            selected_index,
            current_page,
            page_size: DEFAULT_PAGE_SIZE,
            selected_profile_name: None,
            toasts: ToastManager::new(),
            last_applied: None,
            claude_auth_app: None,
            claude_auth_error: None,
            last_claude_action: None,
            codex_auth_app: None,
            codex_auth_error: None,
            last_codex_action: None,
            grok_auth_app: None,
            grok_auth_error: None,
            last_grok_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
            profile_details_expanded: false,
            profile_tasks: ProfileTasks::default(),
            task_executor: AsyncTaskExecutor::from_current_or_test(),
        }
    }

    fn empty_tab(platform: Platform, variant: TabVariant, label: &str) -> PlatformTab {
        PlatformTab {
            platform,
            variant,
            label: label.to_string(),
            profiles: Vec::new(),
            profile_configs: IndexMap::<String, ProfileConfig>::new(),
            profile_load_error: None,
            current_profile_error: None,
            claude_runtime_summary: None,
            codex_runtime_summary: None,
            profile_loading: false,
            profile_loaded: true,
            grok_activation_state: None,
            runtime_error: None,
            activity: None,
            instance: None,
            saved_selection: None,
        }
    }

    fn configured_tabs_in_original_order() -> Vec<PlatformTab> {
        vec![
            empty_tab(Platform::Claude, TabVariant::ClaudeAuth, "Claude Auth"),
            empty_tab(Platform::Claude, TabVariant::Profile, "Claude Code"),
            empty_tab(Platform::Codex, TabVariant::CodexAuth, "Codex Auth"),
            empty_tab(Platform::Grok, TabVariant::GrokAuth, "Grok Auth"),
            empty_tab(Platform::Codex, TabVariant::Profile, "Codex Profile"),
            empty_tab(Platform::Grok, TabVariant::Profile, "Grok Profile"),
        ]
    }

    fn tab_order_ids(tabs: &[PlatformTab]) -> Vec<TuiTabId> {
        tabs.iter().filter_map(tab_config_id).collect()
    }

    #[test]
    fn grok_auth_tab_has_stable_config_id_and_labels() {
        let tab = empty_tab(Platform::Grok, TabVariant::GrokAuth, "Grok Auth");

        assert_eq!(tab_config_id(&tab), Some(TuiTabId::GrokAuth));
        i18n::set_language(TuiLanguage::English);
        assert_eq!(tab.display_label(), "Grok Auth");
        i18n::set_language(TuiLanguage::SimplifiedChinese);
        assert_eq!(tab.display_label(), "Grok 认证");
        i18n::set_language(TuiLanguage::English);
    }

    #[test]
    fn grok_profile_tab_has_stable_config_id_and_labels() {
        let tab = empty_tab(Platform::Grok, TabVariant::Profile, "Grok Profile");

        assert_eq!(tab_config_id(&tab), Some(TuiTabId::GrokProfile));
        assert_eq!(tab.compact_display_label(), "Grok");

        i18n::set_language(TuiLanguage::English);
        assert_eq!(tab.display_label(), "Grok Profile");
        i18n::set_language(TuiLanguage::SimplifiedChinese);
        assert_eq!(tab.display_label(), "Grok 配置");
        i18n::set_language(TuiLanguage::English);
    }

    fn tab_switching_app(active_tab: usize) -> App {
        App {
            tabs: vec![
                empty_tab(Platform::Claude, TabVariant::Profile, "Claude Code"),
                empty_tab(Platform::Codex, TabVariant::Profile, "Codex Profile"),
                empty_tab(Platform::Grok, TabVariant::Profile, "Grok Profile"),
            ],
            active_tab,
            selected_index: 0,
            current_page: 0,
            page_size: DEFAULT_PAGE_SIZE,
            selected_profile_name: None,
            toasts: ToastManager::new(),
            last_applied: None,
            claude_auth_app: None,
            claude_auth_error: None,
            last_claude_action: None,
            codex_auth_app: None,
            codex_auth_error: None,
            last_codex_action: None,
            grok_auth_app: None,
            grok_auth_error: None,
            last_grok_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
            profile_details_expanded: false,
            profile_tasks: ProfileTasks::default(),
            task_executor: AsyncTaskExecutor::from_current_or_test(),
        }
    }

    fn profile_tab(
        platform: Platform,
        label: &str,
        names: &[&str],
        current: Option<&str>,
    ) -> PlatformTab {
        let profiles = names
            .iter()
            .map(|name| ProfileItem {
                name: (*name).to_string(),
                description: None,
                is_current: current == Some(*name),
            })
            .collect();
        PlatformTab {
            platform,
            variant: TabVariant::Profile,
            label: label.to_string(),
            profiles,
            profile_configs: IndexMap::<String, ProfileConfig>::new(),
            profile_load_error: None,
            current_profile_error: None,
            claude_runtime_summary: None,
            codex_runtime_summary: None,
            profile_loading: false,
            profile_loaded: true,
            grok_activation_state: None,
            runtime_error: None,
            activity: None,
            instance: None,
            saved_selection: None,
        }
    }

    fn app_with_profile_tabs(tabs: Vec<PlatformTab>) -> App {
        App {
            tabs,
            active_tab: 0,
            selected_index: 0,
            current_page: 0,
            page_size: DEFAULT_PAGE_SIZE,
            selected_profile_name: None,
            toasts: ToastManager::new(),
            last_applied: None,
            claude_auth_app: None,
            claude_auth_error: None,
            last_claude_action: None,
            codex_auth_app: None,
            codex_auth_error: None,
            last_codex_action: None,
            grok_auth_app: None,
            grok_auth_error: None,
            last_grok_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
            profile_details_expanded: false,
            profile_tasks: ProfileTasks::default(),
            task_executor: AsyncTaskExecutor::from_current_or_test(),
        }
    }

    #[test]
    fn switching_to_tab_first_time_focuses_current_profile() {
        let mut app = app_with_profile_tabs(vec![
            profile_tab(
                Platform::Codex,
                "Codex Profile",
                &["c1", "c2", "c3"],
                Some("c1"),
            ),
            profile_tab(
                Platform::Claude,
                "Claude Code",
                &["a1", "a2", "a3", "a4", "a5"],
                Some("a4"),
            ),
        ]);
        // 模拟构造后的初始定位（with_task_executor 会调 focus_current_profile）
        app.focus_current_profile();
        // 用户在 Codex tab 把光标移到非启用项，制造跨 tab 残留索引
        app.dispatch(Action::SelectNext).unwrap();
        app.dispatch(Action::SelectNext).unwrap();
        assert_eq!(app.selected_profile().unwrap().name, "c3");

        // 首次切到 Claude Code（无快照）：应定位已启用项 a4，而非继承索引 2
        app.dispatch(Action::SwitchTab(1)).unwrap();
        assert_eq!(app.selected_profile().unwrap().name, "a4");
    }

    #[test]
    fn revisiting_tab_restores_saved_selection() {
        let mut app = app_with_profile_tabs(vec![
            profile_tab(
                Platform::Codex,
                "Codex Profile",
                &["c1", "c2", "c3"],
                Some("c1"),
            ),
            profile_tab(
                Platform::Claude,
                "Claude Code",
                &["a1", "a2", "a3", "a4", "a5"],
                Some("a4"),
            ),
        ]);
        app.focus_current_profile();

        // 进入 Claude tab：首次定位已启用项 a4
        app.dispatch(Action::SwitchTab(1)).unwrap();
        assert_eq!(app.selected_profile().unwrap().name, "a4");
        // 移动到非启用项 a5
        app.dispatch(Action::SelectNext).unwrap();
        assert_eq!(app.selected_profile().unwrap().name, "a5");

        // 切回 Codex，再切回 Claude：应恢复上次离开位置 a5（per-tab 记忆）
        app.dispatch(Action::SwitchTab(0)).unwrap();
        app.dispatch(Action::SwitchTab(1)).unwrap();
        assert_eq!(app.selected_profile().unwrap().name, "a5");
    }

    #[test]
    fn tabs_keep_independent_selection() {
        let mut app = app_with_profile_tabs(vec![
            profile_tab(
                Platform::Codex,
                "Codex Profile",
                &["c1", "c2", "c3"],
                Some("c1"),
            ),
            profile_tab(
                Platform::Claude,
                "Claude Code",
                &["a1", "a2", "a3"],
                Some("a1"),
            ),
        ]);
        app.focus_current_profile();

        // Codex tab 选到 c3（非启用项）
        app.dispatch(Action::SelectNext).unwrap();
        app.dispatch(Action::SelectNext).unwrap();
        assert_eq!(app.selected_profile().unwrap().name, "c3");

        // 切到 Claude tab，选到 a2
        app.dispatch(Action::SwitchTab(1)).unwrap();
        app.dispatch(Action::SelectNext).unwrap();
        assert_eq!(app.selected_profile().unwrap().name, "a2");

        // 来回切换：两 tab 各自保持选中，互不串扰
        app.dispatch(Action::SwitchTab(0)).unwrap();
        assert_eq!(app.selected_profile().unwrap().name, "c3");
        app.dispatch(Action::SwitchTab(1)).unwrap();
        assert_eq!(app.selected_profile().unwrap().name, "a2");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn reload_action_refreshes_usage_engine() {
        use crate::tui::usage::app::{UsageApp, UsageDataset, UsageLoadState};

        let mut app = app_with_profile_tabs(vec![profile_tab(
            Platform::Codex,
            "Codex Profile",
            &["c1"],
            Some("c1"),
        )]);
        app.usage_app = Some(UsageApp::with_loader(
            app.task_executor.clone(),
            Arc::new(|| Ok(UsageDataset { rows: Vec::new() })),
        ));

        // r → Reload: profiles 与用量数据集一同刷新,状态机回到 Loading
        app.dispatch(Action::Reload).unwrap();
        assert!(matches!(
            app.usage_app.as_ref().unwrap().state,
            UsageLoadState::Loading
        ));

        // 数据返回后由 on_tick 泵消息更新状态(空数据集 → Empty)
        for _ in 0..200 {
            app.on_tick();
            if !matches!(
                app.usage_app.as_ref().unwrap().state,
                UsageLoadState::Loading
            ) {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert!(matches!(
            app.usage_app.as_ref().unwrap().state,
            UsageLoadState::Empty
        ));
    }

    fn key_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn profile_tab_key_navigation_supports_forward_and_reverse_cycles() {
        let mut app = tab_switching_app(0);

        app.handle_key(key_with_modifiers(KeyCode::Tab, KeyModifiers::NONE))
            .unwrap();
        assert_eq!(app.active_tab, 1);

        app.handle_key(key_with_modifiers(KeyCode::Tab, KeyModifiers::SHIFT))
            .unwrap();
        assert_eq!(app.active_tab, 0);

        app.handle_key(key_with_modifiers(KeyCode::Tab, KeyModifiers::SHIFT))
            .unwrap();
        assert_eq!(app.active_tab, 2);
    }

    #[test]
    fn backtab_keycode_reverse_cycles_profile_tabs() {
        let mut app = tab_switching_app(0);

        app.handle_key(key_with_modifiers(KeyCode::BackTab, KeyModifiers::NONE))
            .unwrap();

        assert_eq!(app.active_tab, 2);
    }

    #[test]
    fn reorder_tabs_uses_default_profile_first_order() {
        let tabs = reorder_tabs(
            configured_tabs_in_original_order(),
            &TuiTabId::default_order(),
        );

        assert_eq!(
            tab_order_ids(&tabs),
            vec![
                TuiTabId::CodexProfile,
                TuiTabId::ClaudeProfile,
                TuiTabId::GrokProfile,
                TuiTabId::CodexAuth,
                TuiTabId::ClaudeAuth,
                TuiTabId::GrokAuth,
            ]
        );
    }

    #[test]
    fn reorder_tabs_honors_custom_full_order() {
        let tabs = reorder_tabs(
            configured_tabs_in_original_order(),
            &[
                TuiTabId::ClaudeAuth,
                TuiTabId::CodexAuth,
                TuiTabId::GrokAuth,
                TuiTabId::ClaudeProfile,
                TuiTabId::CodexProfile,
                TuiTabId::GrokProfile,
            ],
        );

        assert_eq!(
            tab_order_ids(&tabs),
            vec![
                TuiTabId::ClaudeAuth,
                TuiTabId::CodexAuth,
                TuiTabId::GrokAuth,
                TuiTabId::ClaudeProfile,
                TuiTabId::CodexProfile,
                TuiTabId::GrokProfile,
            ]
        );
    }

    #[test]
    fn default_order_selects_codex_profile_first() {
        let app = App {
            tabs: reorder_tabs(
                configured_tabs_in_original_order(),
                &TuiTabId::default_order(),
            ),
            active_tab: 0,
            selected_index: 0,
            current_page: 0,
            page_size: DEFAULT_PAGE_SIZE,
            selected_profile_name: None,
            toasts: ToastManager::new(),
            last_applied: None,
            claude_auth_app: None,
            claude_auth_error: None,
            last_claude_action: None,
            codex_auth_app: None,
            codex_auth_error: None,
            last_codex_action: None,
            grok_auth_app: None,
            grok_auth_error: None,
            last_grok_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
            profile_details_expanded: false,
            profile_tasks: ProfileTasks::default(),
            task_executor: AsyncTaskExecutor::from_current_or_test(),
        };

        assert_eq!(
            tab_config_id(app.current_tab()),
            Some(TuiTabId::CodexProfile)
        );
        assert_eq!(app.current_tab().label, "Codex Profile");
    }

    #[test]
    fn auth_tab_shift_tab_is_intercepted_before_embedded_app_dispatch() {
        let mut app = tab_switching_app(1);
        app.tabs[1] = empty_tab(Platform::Claude, TabVariant::ClaudeAuth, "Claude Auth");

        app.handle_key(key_with_modifiers(KeyCode::Tab, KeyModifiers::SHIFT))
            .unwrap();

        assert_eq!(app.active_tab, 0);
        assert!(app.claude_auth_app.is_none());
    }

    #[test]
    fn navigation_wrap_profile_select_prev_stays_on_current_page() {
        let mut app = profile_navigation_app(28, 0, 0);

        app.dispatch(Action::SelectPrev).unwrap();

        assert_eq!(app.current_page, 0);
        assert_eq!(app.selected_index, DEFAULT_PAGE_SIZE - 1);
        assert_eq!(app.selected_profile_name.as_deref(), Some("profile-10"));
    }

    #[test]
    fn navigation_wrap_profile_select_next_stays_on_current_page() {
        let mut app = profile_navigation_app(28, DEFAULT_PAGE_SIZE - 1, 0);

        app.dispatch(Action::SelectNext).unwrap();

        assert_eq!(app.current_page, 0);
        assert_eq!(app.selected_index, 0);
        assert_eq!(app.selected_profile_name.as_deref(), Some("profile-01"));
    }

    #[test]
    fn current_page_profiles_respects_dynamic_page_size() {
        let mut app = profile_navigation_app(20, 0, 0);

        app.sync_profile_page_size(14);

        assert_eq!(app.current_page_profiles().len(), 14);
        assert_eq!(app.total_pages(), 2);

        app.sync_profile_page_size(25);

        assert_eq!(app.current_page_profiles().len(), 20);
        assert_eq!(app.total_pages(), 1);
    }

    #[test]
    fn profile_page_size_growth_preserves_selected_profile_identity() {
        let mut app = profile_navigation_app(28, 2, 1);

        assert_eq!(
            app.selected_profile().map(|profile| profile.name.as_str()),
            Some("profile-13")
        );

        app.sync_profile_page_size(20);

        assert_eq!(app.page_size, 20);
        assert_eq!(app.current_page, 0);
        assert_eq!(app.selected_index, 12);
        assert_eq!(app.selected_profile_name.as_deref(), Some("profile-13"));
        assert_eq!(
            app.selected_profile().map(|profile| profile.name.as_str()),
            Some("profile-13")
        );
    }

    #[test]
    fn profile_mouse_click_on_blank_row_keeps_selection() {
        let mut app = profile_navigation_app(5, 1, 0);
        app.sync_profile_page_size(14);
        app.list_area.set(Some(Rect::new(0, 5, 40, 18)));

        let result = app.handle_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 2,
            row: 15,
            modifiers: KeyModifiers::NONE,
        });

        assert!(result.is_ok());
        assert_eq!(app.selected_index, 1);
        assert_eq!(app.selected_profile_name.as_deref(), Some("profile-02"));
        assert_eq!(
            app.selected_profile().map(|profile| profile.name.as_str()),
            Some("profile-02")
        );
    }

    #[test]
    fn current_profile_error_accessors_expose_tab_failures() {
        let app = App {
            tabs: vec![PlatformTab {
                platform: Platform::Claude,
                variant: TabVariant::Profile,
                label: "Claude Code".to_string(),
                profiles: Vec::new(),
                profile_configs: IndexMap::<String, ProfileConfig>::new(),
                profile_load_error: Some("load failed".to_string()),
                current_profile_error: Some("current failed".to_string()),
                claude_runtime_summary: None,
                codex_runtime_summary: None,
                profile_loading: false,
                profile_loaded: true,
                grok_activation_state: None,
                runtime_error: None,
                activity: None,
                instance: None,
                saved_selection: None,
            }],
            active_tab: 0,
            selected_index: 0,
            current_page: 0,
            page_size: DEFAULT_PAGE_SIZE,
            selected_profile_name: None,
            toasts: ToastManager::new(),
            last_applied: None,
            claude_auth_app: None,
            claude_auth_error: None,
            last_claude_action: None,
            codex_auth_app: None,
            codex_auth_error: None,
            last_codex_action: None,
            grok_auth_app: None,
            grok_auth_error: None,
            last_grok_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
            profile_details_expanded: false,
            profile_tasks: ProfileTasks::default(),
            task_executor: AsyncTaskExecutor::from_current_or_test(),
        };

        assert_eq!(app.current_profile_load_error(), Some("load failed"));
        assert_eq!(app.current_profile_status_error(), Some("current failed"));
        assert!(app.selected_profile().is_none());
        assert!(app.selected_profile_config().is_none());
    }

    #[test]
    fn with_claude_auth_tab_selects_claude_auth_variant() {
        let app = App {
            tabs: vec![
                PlatformTab {
                    platform: Platform::Claude,
                    variant: TabVariant::Profile,
                    label: "Claude Code".to_string(),
                    profiles: Vec::new(),
                    profile_configs: IndexMap::<String, ProfileConfig>::new(),
                    profile_load_error: None,
                    current_profile_error: None,
                    claude_runtime_summary: None,
                    codex_runtime_summary: None,
                    profile_loading: false,
                    profile_loaded: true,
                    grok_activation_state: None,
                    runtime_error: None,
                    activity: None,
                    instance: None,
                    saved_selection: None,
                },
                PlatformTab {
                    platform: Platform::Claude,
                    variant: TabVariant::ClaudeAuth,
                    label: "Claude Auth".to_string(),
                    profiles: Vec::new(),
                    profile_configs: IndexMap::<String, ProfileConfig>::new(),
                    profile_load_error: None,
                    current_profile_error: None,
                    claude_runtime_summary: None,
                    codex_runtime_summary: None,
                    profile_loading: false,
                    profile_loaded: true,
                    grok_activation_state: None,
                    runtime_error: None,
                    activity: None,
                    instance: None,
                    saved_selection: None,
                },
            ],
            active_tab: 0,
            selected_index: 0,
            current_page: 0,
            page_size: DEFAULT_PAGE_SIZE,
            selected_profile_name: None,
            toasts: ToastManager::new(),
            last_applied: None,
            claude_auth_app: None,
            claude_auth_error: None,
            last_claude_action: None,
            codex_auth_app: None,
            codex_auth_error: None,
            last_codex_action: None,
            grok_auth_app: None,
            grok_auth_error: None,
            last_grok_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
            profile_details_expanded: false,
            profile_tasks: ProfileTasks::default(),
            task_executor: AsyncTaskExecutor::from_current_or_test(),
        }
        .with_claude_auth_tab();

        assert_eq!(app.active_tab, 1);
        assert!(app.is_claude_auth_tab());
    }

    #[test]
    fn with_codex_tab_selects_codex_auth_variant_after_reordering() {
        let app = App {
            tabs: reorder_tabs(
                configured_tabs_in_original_order(),
                &TuiTabId::default_order(),
            ),
            active_tab: 0,
            selected_index: 0,
            current_page: 0,
            page_size: DEFAULT_PAGE_SIZE,
            selected_profile_name: None,
            toasts: ToastManager::new(),
            last_applied: None,
            claude_auth_app: None,
            claude_auth_error: None,
            last_claude_action: None,
            codex_auth_app: None,
            codex_auth_error: None,
            last_codex_action: None,
            grok_auth_app: None,
            grok_auth_error: None,
            last_grok_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
            profile_details_expanded: false,
            profile_tasks: ProfileTasks::default(),
            task_executor: AsyncTaskExecutor::from_current_or_test(),
        }
        .with_codex_tab();

        assert_eq!(app.active_tab, 3);
        assert!(app.is_codex_auth_tab());
    }

    #[test]
    fn with_grok_auth_tab_selects_grok_auth_variant() {
        let app = App {
            tabs: vec![
                PlatformTab {
                    platform: Platform::Claude,
                    variant: TabVariant::Profile,
                    label: "Claude Code".to_string(),
                    profiles: Vec::new(),
                    profile_configs: IndexMap::<String, ProfileConfig>::new(),
                    profile_load_error: None,
                    current_profile_error: None,
                    claude_runtime_summary: None,
                    codex_runtime_summary: None,
                    profile_loading: false,
                    profile_loaded: true,
                    grok_activation_state: None,
                    runtime_error: None,
                    activity: None,
                    instance: None,
                    saved_selection: None,
                },
                PlatformTab {
                    platform: Platform::Grok,
                    variant: TabVariant::GrokAuth,
                    label: "Grok Auth".to_string(),
                    profiles: Vec::new(),
                    profile_configs: IndexMap::<String, ProfileConfig>::new(),
                    profile_load_error: None,
                    current_profile_error: None,
                    claude_runtime_summary: None,
                    codex_runtime_summary: None,
                    profile_loading: false,
                    profile_loaded: true,
                    grok_activation_state: None,
                    runtime_error: None,
                    activity: None,
                    instance: None,
                    saved_selection: None,
                },
            ],
            active_tab: 0,
            selected_index: 0,
            current_page: 0,
            page_size: DEFAULT_PAGE_SIZE,
            selected_profile_name: None,
            toasts: ToastManager::new(),
            last_applied: None,
            claude_auth_app: None,
            claude_auth_error: None,
            last_claude_action: None,
            codex_auth_app: None,
            codex_auth_error: None,
            last_codex_action: None,
            grok_auth_app: None,
            grok_auth_error: None,
            last_grok_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
            profile_details_expanded: false,
            profile_tasks: ProfileTasks::default(),
            task_executor: AsyncTaskExecutor::from_current_or_test(),
        }
        .with_grok_auth_tab();

        assert_eq!(app.active_tab, 1);
        assert!(app.is_grok_auth_tab());
    }

    struct FailingPlatform {
        platform: Platform,
        current_profile_error: Option<CcrError>,
        profile_load_error: Option<CcrError>,
    }

    impl PlatformConfig for FailingPlatform {
        fn platform_name(&self) -> &str {
            self.platform.short_name()
        }

        fn platform_type(&self) -> Platform {
            self.platform
        }

        fn load_profiles(&self) -> Result<IndexMap<String, ProfileConfig>> {
            if let Some(err) = &self.profile_load_error {
                return Err(match err {
                    CcrError::ConfigError(message) => CcrError::ConfigError(message.clone()),
                    CcrError::ConfigMissing(message) => CcrError::ConfigMissing(message.clone()),
                    CcrError::ConfigSectionNotFound(message) => {
                        CcrError::ConfigSectionNotFound(message.clone())
                    }
                    CcrError::ConfigFormatInvalid(message) => {
                        CcrError::ConfigFormatInvalid(message.clone())
                    }
                    other => CcrError::ConfigError(other.to_string()),
                });
            }
            Ok(IndexMap::new())
        }

        fn save_profile(&self, _name: &str, _profile: &ProfileConfig) -> Result<()> {
            Ok(())
        }

        fn delete_profile(&self, _name: &str) -> Result<()> {
            Ok(())
        }

        fn get_settings_path(&self) -> PathBuf {
            PathBuf::new()
        }

        fn apply_profile(&self, _name: &str) -> Result<()> {
            Ok(())
        }

        fn validate_profile(&self, _profile: &ProfileConfig) -> Result<()> {
            Ok(())
        }

        fn get_current_profile(&self) -> Result<Option<String>> {
            if let Some(err) = &self.current_profile_error {
                return Err(match err {
                    CcrError::ConfigError(message) => CcrError::ConfigError(message.clone()),
                    CcrError::ConfigMissing(message) => CcrError::ConfigMissing(message.clone()),
                    CcrError::ConfigSectionNotFound(message) => {
                        CcrError::ConfigSectionNotFound(message.clone())
                    }
                    CcrError::ConfigFormatInvalid(message) => {
                        CcrError::ConfigFormatInvalid(message.clone())
                    }
                    other => CcrError::ConfigError(other.to_string()),
                });
            }
            Ok(Some("default".to_string()))
        }
    }

    #[test]
    fn build_profile_tab_data_includes_profile_file_location_for_load_errors() {
        let platform: Arc<dyn PlatformConfig> = Arc::new(FailingPlatform {
            platform: Platform::Claude,
            current_profile_error: None,
            profile_load_error: Some(CcrError::ConfigFormatInvalid(
                "TOML 解析失败: invalid string".to_string(),
            )),
        });

        let tab_data = super::super::profile_tasks::read_instance_profiles(platform.as_ref());
        let error = format_read_issue(tab_data.configs.err().unwrap());

        assert!(error.contains("Where:\n  "));
        assert!(error.contains("profiles.toml"));
        assert!(error.contains("\n\nWhat:\n  "));
        assert!(error.contains("TOML 解析失败"));
    }

    #[test]
    fn format_issue_separates_long_location_and_removes_duplicate_path() {
        i18n::set_language(TuiLanguage::English);
        let location = format!(
            "C:\\Users\\lyh\\.ccr\\platforms\\grok\\{}\\profiles.toml",
            "nested\\directory\\".repeat(8)
        );
        let error = CcrError::ConfigFormatInvalid(format!(
            "{location}: profile 结构错误（第 13 行，第 17 列）：provider_type 的值不受支持"
        ));

        let formatted = format_issue(location.clone(), &error);

        assert!(formatted.starts_with("Where:\n  "));
        assert!(formatted.contains("\n\nWhat:\n  配置格式无效: profile 结构错误"));
        assert!(formatted.contains("第 13 行，第 17 列"));
        assert!(formatted.contains("provider_type 的值不受支持"));
        assert_eq!(formatted.matches(&location).count(), 1);
    }

    #[test]
    fn format_issue_indents_multiline_location_values() {
        i18n::set_language(TuiLanguage::English);
        let formatted = format_issue(
            "C:\\registry.toml\nFallback: C:\\profiles.toml".to_string(),
            &CcrError::ConfigError("registry broken".to_string()),
        );

        assert!(formatted.contains("Where:\n  C:\\registry.toml\n  Fallback: C:\\profiles.toml"));
        assert!(formatted.contains("\n\nWhat:\n  配置文件错误: registry broken"));
    }

    #[test]
    fn build_profile_tab_data_includes_registry_location_for_current_profile_errors() {
        let platform: Arc<dyn PlatformConfig> = Arc::new(FailingPlatform {
            platform: Platform::Codex,
            current_profile_error: Some(CcrError::ConfigError("registry broken".to_string())),
            profile_load_error: None,
        });

        let tab_data = super::super::profile_tasks::read_instance_profiles(platform.as_ref());
        let error = format_read_issue(tab_data.current.err().unwrap());

        assert!(error.contains("Where:\n  "));
        assert!(error.contains("config.toml"));
        assert!(error.contains("Fallback:"));
        assert!(error.contains("profiles.toml"));
        assert!(error.contains("\n\nWhat:\n  "));
        assert!(error.contains("registry broken"));
    }

    #[test]
    fn build_grok_profile_tab_data_allows_an_empty_profile_set() {
        let platform: Arc<dyn PlatformConfig> = Arc::new(FailingPlatform {
            platform: Platform::Grok,
            current_profile_error: None,
            profile_load_error: None,
        });

        let tab_data = super::super::profile_tasks::read_instance_profiles(platform.as_ref());

        assert!(tab_data.configs.ok().unwrap().is_empty());
        assert!(tab_data.current.is_ok());
        assert!(tab_data.claude_runtime.is_none());
        assert!(tab_data.codex_runtime.is_none());
    }
}
