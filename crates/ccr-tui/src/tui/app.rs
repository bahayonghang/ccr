// TUI application state — Tab-based dispatch

use crate::tui::CompletedAction;
use crate::tui::action::Action;
use crate::tui::i18n::{self, Message};
use crate::tui::toast::{Toast, ToastManager};
use ccr_cli::managers::{TuiConfig, TuiConfigManager, TuiTabId};
use ccr_cli::models::{ClaudeRuntimeSummary, CodexRuntimeSummary};
use ccr_cli::models::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
use ccr_cli::platforms::create_platform;
use ccr_core::core::error::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use indexmap::IndexMap;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders};
use std::cell::Cell;
use std::sync::Arc;

use super::claude_auth::{ClaudeAuthActionRecord, ClaudeAuthApp};
use super::codex_auth::CodexAuthApp;
use super::opencode_auth::OpenCodeAuthApp;
use super::pagination::{DEFAULT_PAGE_SIZE, page_for_index, page_slice, total_pages};
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
    /// OpenCode openai account/auth management
    OpenCodeAuth,
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
    /// 离开该 tab 时保存的选中快照；None = 从未访问过（per-tab 会话级记忆）
    pub saved_selection: Option<TabSelection>,
}

impl PlatformTab {
    pub fn display_label(&self) -> &str {
        match (self.platform, self.variant) {
            (Platform::Claude, TabVariant::Profile) => {
                crate::tui_text!("Claude Code", "Claude 配置")
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
            (_, TabVariant::OpenCodeAuth) => {
                crate::tui_text!("OpenCode Auth", "OpenCode 认证")
            }
            _ => self.label.as_str(),
        }
    }

    pub fn compact_display_label(&self) -> &str {
        match (self.platform, self.variant) {
            (Platform::Claude, TabVariant::Profile) => "Claude",
            (Platform::Codex, TabVariant::Profile) => "Codex",
            (Platform::Grok, TabVariant::Profile) => "Grok",
            (_, TabVariant::ClaudeAuth | TabVariant::CodexAuth) => {
                crate::tui_text!("Auth", "认证")
            }
            (_, TabVariant::OpenCodeAuth) => "Open",
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

struct ProfileTabData {
    profiles: Vec<ProfileItem>,
    profile_configs: IndexMap<String, ProfileConfig>,
    profile_load_error: Option<String>,
    current_profile_error: Option<String>,
    claude_runtime_summary: Option<ClaudeRuntimeSummary>,
    codex_runtime_summary: Option<CodexRuntimeSummary>,
}

fn profile_source_path(platform: Platform) -> String {
    PlatformPaths::new(platform)
        .map(|paths| paths.profiles_file.display().to_string())
        .unwrap_or_else(|_| format!("~/.ccr/platforms/{}/profiles.toml", platform.short_name()))
}

fn current_profile_source_path(platform: Platform) -> String {
    match PlatformPaths::new(platform) {
        Ok(paths) if platform == Platform::Codex => format!(
            "{}\n{}: {}",
            paths.registry_file.display(),
            i18n::text(Message::Fallback),
            paths.profiles_file.display()
        ),
        Ok(paths) => paths.registry_file.display().to_string(),
        Err(_) if platform == Platform::Codex => format!(
            "~/.ccr/config.toml\n{}: ~/.ccr/platforms/{}/profiles.toml",
            i18n::text(Message::Fallback),
            platform.short_name()
        ),
        Err(_) => "~/.ccr/config.toml".to_string(),
    }
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
        (_, TabVariant::OpenCodeAuth) => Some(TuiTabId::OpencodeAuth),
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
    /// Embedded OpenCode Auth app (lazy initialized)
    pub opencode_auth_app: Option<OpenCodeAuthApp>,
    /// Last OpenCode Auth initialization error for placeholder rendering
    pub opencode_auth_error: Option<String>,
    /// Last opencode auth action info (action_type, account_name, success, error)
    pub last_opencode_action: Option<(CompletedAction, String, bool, Option<String>)>,
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
    /// 异步后台任务执行器
    pub(crate) task_executor: AsyncTaskExecutor,
}

impl App {
    fn build_profile_tab_data(instance: &Arc<dyn PlatformConfig>) -> ProfileTabData {
        let platform = instance.platform_type();
        let (current, current_profile_error) = match instance.get_current_profile() {
            Ok(current) => (current, None),
            Err(e) => {
                let err = format_issue(current_profile_source_path(platform), &e);
                tracing::warn!("{err}");
                (None, Some(err))
            }
        };

        let codex_runtime_summary = if platform == Platform::Codex {
            ccr_cli::services::CodexAuthService::new()
                .ok()
                .and_then(|service| service.get_runtime_summary().ok())
        } else {
            None
        };

        let claude_runtime_summary = if platform == Platform::Claude {
            ccr_cli::services::ClaudeAuthService::new()
                .ok()
                .and_then(|service| service.get_runtime_summary().ok())
        } else {
            None
        };

        match instance.load_profiles() {
            Ok(profile_configs) => {
                let profiles = profile_configs
                    .iter()
                    .map(|(name, config)| ProfileItem {
                        is_current: current.as_ref() == Some(name),
                        description: config.description.clone(),
                        name: name.clone(),
                    })
                    .collect();
                ProfileTabData {
                    profiles,
                    profile_configs,
                    profile_load_error: None,
                    current_profile_error,
                    claude_runtime_summary,
                    codex_runtime_summary,
                }
            }
            Err(e) => {
                let err = format_issue(profile_source_path(platform), &e);
                tracing::warn!("{err}");
                ProfileTabData {
                    profiles: Vec::new(),
                    profile_configs: IndexMap::new(),
                    profile_load_error: Some(err),
                    current_profile_error,
                    claude_runtime_summary,
                    codex_runtime_summary,
                }
            }
        }
    }

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
        self.tabs[self.active_tab].current_profile_error.as_deref()
    }

    pub fn current_codex_runtime_summary(&self) -> Option<&CodexRuntimeSummary> {
        self.tabs[self.active_tab].codex_runtime_summary.as_ref()
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
        i18n::set_language(tui_config.language);
        let mut tabs = Vec::new();

        for platform in Platform::implemented() {
            if !matches!(
                platform,
                Platform::Claude | Platform::Codex | Platform::Grok
            ) {
                continue;
            }

            match create_platform(platform) {
                Ok(instance) => {
                    let tab_data = Self::build_profile_tab_data(&instance);

                    match platform {
                        Platform::Claude => {
                            tabs.push(PlatformTab {
                                platform,
                                variant: TabVariant::ClaudeAuth,
                                label: "Claude Auth".to_string(),
                                profiles: Vec::new(),
                                profile_configs: IndexMap::new(),
                                profile_load_error: None,
                                current_profile_error: None,
                                claude_runtime_summary: tab_data.claude_runtime_summary.clone(),
                                codex_runtime_summary: None,
                                instance: Some(Arc::clone(&instance)),
                                saved_selection: None,
                            });
                            tabs.push(PlatformTab {
                                platform,
                                variant: TabVariant::Profile,
                                label: platform.display_name().to_string(),
                                profiles: tab_data.profiles,
                                profile_configs: tab_data.profile_configs,
                                profile_load_error: tab_data.profile_load_error,
                                current_profile_error: tab_data.current_profile_error,
                                claude_runtime_summary: tab_data.claude_runtime_summary,
                                codex_runtime_summary: tab_data.codex_runtime_summary,
                                instance: Some(instance),
                                saved_selection: None,
                            });
                        }
                        Platform::Codex => {
                            // Codex Auth tab (account management)
                            tabs.push(PlatformTab {
                                platform,
                                variant: TabVariant::CodexAuth,
                                label: "Codex Auth".to_string(),
                                profiles: Vec::new(),
                                profile_configs: IndexMap::new(),
                                profile_load_error: None,
                                current_profile_error: None,
                                claude_runtime_summary: None,
                                codex_runtime_summary: None,
                                instance: Some(Arc::clone(&instance)),
                                saved_selection: None,
                            });
                            // OpenCode Auth tab (manual OpenCode openai switching)
                            tabs.push(PlatformTab {
                                platform,
                                variant: TabVariant::OpenCodeAuth,
                                label: "OpenCode Auth".to_string(),
                                profiles: Vec::new(),
                                profile_configs: IndexMap::new(),
                                profile_load_error: None,
                                current_profile_error: None,
                                claude_runtime_summary: None,
                                codex_runtime_summary: None,
                                instance: Some(Arc::clone(&instance)),
                                saved_selection: None,
                            });
                            // Codex Profile tab (profile switching)
                            tabs.push(PlatformTab {
                                platform,
                                variant: TabVariant::Profile,
                                label: "Codex Profile".to_string(),
                                profiles: tab_data.profiles,
                                profile_configs: tab_data.profile_configs,
                                profile_load_error: tab_data.profile_load_error,
                                current_profile_error: tab_data.current_profile_error,
                                claude_runtime_summary: None,
                                codex_runtime_summary: tab_data.codex_runtime_summary,
                                instance: Some(instance),
                                saved_selection: None,
                            });
                        }
                        Platform::Grok => {
                            tabs.push(PlatformTab {
                                platform,
                                variant: TabVariant::Profile,
                                label: "Grok Profile".to_string(),
                                profiles: tab_data.profiles,
                                profile_configs: tab_data.profile_configs,
                                profile_load_error: tab_data.profile_load_error,
                                current_profile_error: tab_data.current_profile_error,
                                claude_runtime_summary: None,
                                codex_runtime_summary: None,
                                instance: Some(instance),
                                saved_selection: None,
                            });
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to create {} platform: {}", platform, e);
                }
            }
        }

        // Fallback: ensure at least Claude tab exists
        if tabs.is_empty() {
            tabs.push(PlatformTab {
                platform: Platform::Claude,
                variant: TabVariant::ClaudeAuth,
                label: "Claude Auth".to_string(),
                profiles: Vec::new(),
                profile_configs: IndexMap::new(),
                profile_load_error: None,
                current_profile_error: None,
                claude_runtime_summary: None,
                codex_runtime_summary: None,
                instance: None,
                saved_selection: None,
            });
            tabs.push(PlatformTab {
                platform: Platform::Claude,
                variant: TabVariant::Profile,
                label: Platform::Claude.display_name().to_string(),
                profiles: Vec::new(),
                profile_configs: IndexMap::new(),
                profile_load_error: None,
                current_profile_error: None,
                claude_runtime_summary: None,
                codex_runtime_summary: None,
                instance: None,
                saved_selection: None,
            });
        }
        tabs = reorder_tabs(tabs, &tui_config.tab_order);

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
            opencode_auth_app: None,
            opencode_auth_error: None,
            last_opencode_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
            task_executor,
        };
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
            KeyCode::Enter => Action::ApplyAndQuit,
            KeyCode::Char(' ') => Action::ApplySelected,
            KeyCode::Char('r') => Action::Reload,
            _ => Action::Noop,
        }
    }

    // -- Action dispatch (executes side effects) --

    fn dispatch(&mut self, action: Action) -> Result<bool> {
        match action {
            Action::Noop => {}
            Action::Quit => return Ok(true),
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
            Action::ApplyAndQuit => {
                self.apply_selected();
                return Ok(true);
            }
            Action::Reload => {
                self.reload_profiles();
                // 用量数据集与 profiles 一同刷新(后台异步拉取,不阻塞渲染)
                self.ensure_usage_engine();
                if let Some(engine) = self.usage_app.as_mut() {
                    engine.refresh();
                }
                self.push_active_toast(Toast::info(i18n::text(Message::ProfilesReloaded)));
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
        } else if self.is_opencode_auth_tab()
            && let Some(app) = self.opencode_auth_app.as_mut()
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

    fn apply_selected(&mut self) {
        let Some(selected) = self.selected_profile() else {
            self.toasts.push(Toast::warning(crate::tui_text!(
                "No profiles available",
                "没有可用的配置"
            )));
            return;
        };

        let tab = &self.tabs[self.active_tab];
        let platform_label = tab.label.clone();
        let profile_name = selected.name.clone();
        self.selected_profile_name = Some(profile_name.clone());

        if let Some(instance) = &tab.instance {
            match instance.apply_profile(&profile_name) {
                Ok(()) => {
                    self.toasts.push(Toast::success(crate::tui_format!(
                        "Switched to: {}",
                        "已切换到：{}",
                        profile_name
                    )));
                    self.last_applied = Some((platform_label, profile_name.clone(), true, None));

                    if let Ok(profiles) = instance.load_profiles()
                        && let Some(mut profile) = profiles.get(&profile_name).cloned()
                    {
                        profile.increment_usage();
                        let _ = instance.save_profile(&profile_name, &profile);
                    }

                    self.reload_profiles();
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    self.toasts.push(Toast::error(crate::tui_format!(
                        "Switch failed: {}",
                        "切换失败：{}",
                        err_msg
                    )));
                    self.last_applied = Some((platform_label, profile_name, false, Some(err_msg)));
                }
            }
        } else {
            self.toasts.push(Toast::error(crate::tui_text!(
                "Platform is not initialized",
                "平台未初始化"
            )));
        }
    }

    fn reload_profiles(&mut self) {
        for tab in &mut self.tabs {
            if tab.variant != TabVariant::Profile {
                continue;
            }
            if let Some(instance) = &tab.instance {
                let tab_data = Self::build_profile_tab_data(instance);
                tab.profiles = tab_data.profiles;
                tab.profile_configs = tab_data.profile_configs;
                tab.profile_load_error = tab_data.profile_load_error;
                tab.current_profile_error = tab_data.current_profile_error;
                tab.claude_runtime_summary = tab_data.claude_runtime_summary;
                tab.codex_runtime_summary = tab_data.codex_runtime_summary;
            }
        }
        self.sync_selection_to_profile_name();
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

    /// Ensure OpenCode Auth app is initialized before interaction/rendering
    fn ensure_opencode_auth_app(&mut self) {
        if self.opencode_auth_app.is_some() {
            return;
        }

        match OpenCodeAuthApp::with_task_executor(self.task_executor.clone()) {
            Ok(app) => {
                self.opencode_auth_app = Some(app);
                self.opencode_auth_error = None;
            }
            Err(e) => {
                let err = e.to_string();
                tracing::warn!("Failed to init OpenCodeAuthApp: {}", err);
                self.opencode_auth_error = Some(err.clone());
                self.toasts.push(Toast::error(crate::tui_format!(
                    "Failed to initialize OpenCode Auth: {}",
                    "OpenCode 认证初始化失败：{}",
                    err
                )));
            }
        }
    }

    /// Get mutable OpenCode Auth app, initializing it on demand
    fn opencode_auth_app_mut(&mut self) -> Option<&mut OpenCodeAuthApp> {
        self.ensure_opencode_auth_app();
        self.opencode_auth_app.as_mut()
    }

    /// Check if the currently active tab is the Claude Auth variant
    pub fn is_claude_auth_tab(&self) -> bool {
        self.tabs[self.active_tab].variant == TabVariant::ClaudeAuth
    }

    /// Check if the currently active tab is the Codex Auth variant
    pub fn is_codex_auth_tab(&self) -> bool {
        self.tabs[self.active_tab].variant == TabVariant::CodexAuth
    }

    /// Check if the currently active tab is the OpenCode Auth variant
    pub fn is_opencode_auth_tab(&self) -> bool {
        self.tabs[self.active_tab].variant == TabVariant::OpenCodeAuth
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

    /// Pre-select OpenCode Auth tab (for `ccr opencode` entry)
    pub fn with_opencode_auth_tab(mut self) -> Self {
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|t| t.variant == TabVariant::OpenCodeAuth)
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
        let is_claude_auth = self.is_claude_auth_tab();
        let is_codex_auth = self.is_codex_auth_tab();
        let is_opencode_auth = self.is_opencode_auth_tab();

        if !is_claude_auth && !is_codex_auth && !is_opencode_auth {
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

        if is_opencode_auth && let Some(opencode_app) = self.opencode_auth_app_mut() {
            opencode_app.on_activated();
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

    /// Delegate mouse event to embedded OpenCodeAuthApp
    fn delegate_mouse_to_opencode(&mut self, mouse: MouseEvent) -> Result<bool> {
        if let Some(opencode_app) = self.opencode_auth_app_mut() {
            opencode_app.handle_mouse(mouse)
        } else {
            Ok(false)
        }
    }
}

// -- Mouse hit-test helpers (pure functions for testability) --

/// Calculate which list item was clicked based on mouse row and list area.
/// Uses `Block::inner()` for robust border offset calculation.
/// Returns `None` if click is outside the list content area.
pub(crate) fn list_hit_test(area: Rect, mouse_row: u16, page_len: usize) -> Option<usize> {
    let inner = Block::default().borders(Borders::ALL).inner(area);
    if mouse_row >= inner.y && mouse_row < inner.y + inner.height {
        let clicked_row = (mouse_row - inner.y) as usize;
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

/// Calculate which tab was clicked based on mouse position and header area.
/// Returns `None` if no tab switch should occur (same tab, single tab, or outside header).
fn tab_hit_test(
    header: Rect,
    mouse_row: u16,
    mouse_col: u16,
    tab_count: usize,
    active_tab: usize,
) -> Option<usize> {
    if mouse_row < header.y || mouse_row >= header.y + header.height {
        return None;
    }
    if tab_count <= 1 {
        return None;
    }
    let tab_width = header.width / tab_count as u16;
    if tab_width == 0 {
        return None;
    }
    let rel_x = mouse_col.saturating_sub(header.x);
    let tab_idx = (rel_x / tab_width) as usize;
    if tab_idx < tab_count && tab_idx != active_tab {
        Some(tab_idx)
    } else {
        None
    }
}

// -- TuiApp trait implementation (tab-based dispatch) --

impl TuiApp for App {
    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        // Ctrl+C always quits the entire TUI
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(true);
        }

        if Self::is_language_switch_key(key) {
            self.toggle_language();
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
        } else if self.is_opencode_auth_tab() {
            if let Some(opencode_app) = self.opencode_auth_app_mut() {
                let quit = opencode_app.handle_key(key)?;
                if quit {
                    self.last_opencode_action = opencode_app.last_action.clone();
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
        match mouse.kind {
            // 🖱️ 左键点击
            MouseEventKind::Down(MouseButton::Left) => {
                // Tab 栏点击（所有 tab 共用）
                if let Some(header) = self.header_area.get() {
                    if let Some(tab_idx) = tab_hit_test(
                        header,
                        mouse.row,
                        mouse.column,
                        self.tabs.len(),
                        self.active_tab,
                    ) {
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
                if self.is_opencode_auth_tab() {
                    return self.delegate_mouse_to_opencode(mouse);
                }

                // Profile tabs (Claude / Codex Profile): 列表项点击
                if let Some(area) = self.list_area.get()
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
                if self.is_opencode_auth_tab() {
                    return self.delegate_mouse_to_opencode(mouse);
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
                if self.is_opencode_auth_tab() {
                    return self.delegate_mouse_to_opencode(mouse);
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
        if self.is_claude_auth_tab() {
            self.claude_auth_app.as_mut().is_some_and(|a| a.on_tick())
        } else if self.is_codex_auth_tab() {
            self.codex_auth_app.as_mut().is_some_and(|a| a.on_tick())
        } else if self.is_opencode_auth_tab() {
            self.opencode_auth_app.as_mut().is_some_and(|a| a.on_tick())
        } else {
            // Profile tab: 首次进入(含启动首帧)激活用量引擎,此后每 tick 泵
            // 后台任务消息;on_activated 仅在 Idle 态生效,不会重复拉取
            self.ensure_usage_engine();
            let usage_redraw = self.usage_app.as_mut().is_some_and(|engine| {
                engine.on_activated();
                engine.tick()
            });
            self.toasts.tick() | usage_redraw
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        ui::draw(frame, self);
    }
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

    // -- list_hit_test tests --

    #[test]
    fn list_hit_test_clicks_first_item() {
        // area: y=5, height=12 (border top at y=5, inner starts at y=6)
        let area = Rect::new(0, 5, 40, 12);
        assert_eq!(list_hit_test(area, 6, 5), Some(0));
    }

    #[test]
    fn list_hit_test_clicks_third_item() {
        let area = Rect::new(0, 5, 40, 12);
        assert_eq!(list_hit_test(area, 8, 5), Some(2));
    }

    #[test]
    fn list_hit_test_ignores_top_border() {
        let area = Rect::new(0, 5, 40, 12);
        // Click on the top border row (y=5) — should NOT select anything
        assert_eq!(list_hit_test(area, 5, 5), None);
    }

    #[test]
    fn list_hit_test_ignores_bottom_border() {
        let area = Rect::new(0, 5, 40, 12);
        // Bottom border is at y=5+12-1=16
        assert_eq!(list_hit_test(area, 16, 5), None);
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

    // -- tab_hit_test tests --

    #[test]
    fn tab_hit_test_clicks_second_tab() {
        // header: x=0, y=0, width=80, height=3
        let header = Rect::new(0, 0, 80, 3);
        // 2 tabs, each 40px wide. Click at col 50 → tab index 1
        assert_eq!(tab_hit_test(header, 1, 50, 2, 0), Some(1));
    }

    #[test]
    fn tab_hit_test_clicks_first_tab() {
        let header = Rect::new(0, 0, 80, 3);
        // Click at col 10 → tab index 0, but active_tab is already 0 → None
        assert_eq!(tab_hit_test(header, 1, 10, 2, 0), None);
    }

    #[test]
    fn tab_hit_test_switch_from_second_to_first() {
        let header = Rect::new(0, 0, 80, 3);
        // Active tab is 1, click at col 10 → tab index 0
        assert_eq!(tab_hit_test(header, 1, 10, 2, 1), Some(0));
    }

    #[test]
    fn tab_hit_test_ignores_click_outside_header() {
        let header = Rect::new(0, 0, 80, 3);
        // Click below header (row 5)
        assert_eq!(tab_hit_test(header, 5, 50, 2, 0), None);
    }

    #[test]
    fn tab_hit_test_single_tab_returns_none() {
        let header = Rect::new(0, 0, 80, 3);
        // Only 1 tab — no switching possible
        assert_eq!(tab_hit_test(header, 1, 10, 1, 0), None);
    }

    #[test]
    fn tab_hit_test_zero_tab_width_returns_none() {
        // Extremely narrow terminal: width=1, 2 tabs → tab_width = 0
        let header = Rect::new(0, 0, 1, 3);
        assert_eq!(tab_hit_test(header, 1, 0, 2, 0), None);
    }

    #[test]
    fn tab_hit_test_narrow_terminal_no_panic() {
        // width=0, 2 tabs — must not panic
        let header = Rect::new(0, 0, 0, 3);
        assert_eq!(tab_hit_test(header, 1, 0, 2, 0), None);
    }

    #[test]
    fn tab_hit_test_three_tabs() {
        // 3 tabs, width=90, each tab ~30px
        let header = Rect::new(0, 0, 90, 3);
        // Click at col 35 → tab index 1
        assert_eq!(tab_hit_test(header, 1, 35, 3, 0), Some(1));
        // Click at col 65 → tab index 2
        assert_eq!(tab_hit_test(header, 1, 65, 3, 0), Some(2));
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
            opencode_auth_app: None,
            opencode_auth_error: None,
            last_opencode_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
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
            instance: None,
            saved_selection: None,
        }
    }

    fn configured_tabs_in_original_order() -> Vec<PlatformTab> {
        vec![
            empty_tab(Platform::Claude, TabVariant::ClaudeAuth, "Claude Auth"),
            empty_tab(Platform::Claude, TabVariant::Profile, "Claude Code"),
            empty_tab(Platform::Codex, TabVariant::CodexAuth, "Codex Auth"),
            empty_tab(Platform::Codex, TabVariant::OpenCodeAuth, "OpenCode Auth"),
            empty_tab(Platform::Codex, TabVariant::Profile, "Codex Profile"),
            empty_tab(Platform::Grok, TabVariant::Profile, "Grok Profile"),
        ]
    }

    fn tab_order_ids(tabs: &[PlatformTab]) -> Vec<TuiTabId> {
        tabs.iter().filter_map(tab_config_id).collect()
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
                empty_tab(Platform::Codex, TabVariant::Profile, "OpenCode Profile"),
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
            opencode_auth_app: None,
            opencode_auth_error: None,
            last_opencode_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
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
            opencode_auth_app: None,
            opencode_auth_error: None,
            last_opencode_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
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
                TuiTabId::OpencodeAuth,
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
                TuiTabId::OpencodeAuth,
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
                TuiTabId::OpencodeAuth,
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
            opencode_auth_app: None,
            opencode_auth_error: None,
            last_opencode_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
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
            opencode_auth_app: None,
            opencode_auth_error: None,
            last_opencode_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
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
            opencode_auth_app: None,
            opencode_auth_error: None,
            last_opencode_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
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
            opencode_auth_app: None,
            opencode_auth_error: None,
            last_opencode_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
            task_executor: AsyncTaskExecutor::from_current_or_test(),
        }
        .with_codex_tab();

        assert_eq!(app.active_tab, 3);
        assert!(app.is_codex_auth_tab());
    }

    #[test]
    fn with_opencode_auth_tab_selects_opencode_auth_variant() {
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
                    instance: None,
                    saved_selection: None,
                },
                PlatformTab {
                    platform: Platform::Codex,
                    variant: TabVariant::OpenCodeAuth,
                    label: "OpenCode Auth".to_string(),
                    profiles: Vec::new(),
                    profile_configs: IndexMap::<String, ProfileConfig>::new(),
                    profile_load_error: None,
                    current_profile_error: None,
                    claude_runtime_summary: None,
                    codex_runtime_summary: None,
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
            opencode_auth_app: None,
            opencode_auth_error: None,
            last_opencode_action: None,
            usage_app: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
            detail_area: Cell::new(None),
            profile_detail_scroll: 0,
            task_executor: AsyncTaskExecutor::from_current_or_test(),
        }
        .with_opencode_auth_tab();

        assert_eq!(app.active_tab, 1);
        assert!(app.is_opencode_auth_tab());
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

        let tab_data = App::build_profile_tab_data(&platform);
        let error = tab_data.profile_load_error.unwrap();

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

        let tab_data = App::build_profile_tab_data(&platform);
        let error = tab_data.current_profile_error.unwrap();

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

        let tab_data = App::build_profile_tab_data(&platform);

        assert!(tab_data.profiles.is_empty());
        assert!(tab_data.profile_configs.is_empty());
        assert!(tab_data.profile_load_error.is_none());
        assert!(tab_data.current_profile_error.is_none());
        assert!(tab_data.claude_runtime_summary.is_none());
        assert!(tab_data.codex_runtime_summary.is_none());
    }
}
