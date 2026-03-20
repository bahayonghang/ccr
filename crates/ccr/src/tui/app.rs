// TUI application state — Tab-based dispatch (Claude + Codex only)

use crate::core::error::Result;
use crate::models::platform::{Platform, PlatformConfig, ProfileConfig};
use crate::platforms::create_platform;
use crate::tui::action::Action;
use crate::tui::toast::{Toast, ToastManager};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use indexmap::IndexMap;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders};
use std::cell::Cell;
use std::sync::Arc;

use super::codex_auth::CodexAuthApp;
use super::runtime::TuiApp;
use super::ui;

/// Maximum profiles per page
pub const PAGE_SIZE: usize = 10;

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
    /// Standard profile switching (Claude, Codex Profile)
    Profile,
    /// Codex account/auth management
    CodexAuth,
}

/// A tab representing one platform with its profiles loaded
pub struct PlatformTab {
    pub platform: Platform,
    pub variant: TabVariant,
    pub label: String,
    pub profiles: Vec<ProfileItem>,
    pub profile_configs: IndexMap<String, ProfileConfig>,
    pub instance: Option<Arc<dyn PlatformConfig>>,
}

/// Main TUI application state
pub struct App {
    /// Dynamic list of platform tabs (Claude + Codex only)
    pub tabs: Vec<PlatformTab>,
    /// Index of the currently active tab
    pub active_tab: usize,
    /// Index of the selected profile within the current page
    pub selected_index: usize,
    /// Current page number (0-based)
    pub current_page: usize,
    /// 当前选中的 profile 名称（跨刷新保持同步）
    pub selected_profile_name: Option<String>,
    /// Toast notification manager
    pub toasts: ToastManager,
    /// Last applied profile info (platform_name, profile_name, success, error)
    pub last_applied: Option<(String, String, bool, Option<String>)>,
    /// Embedded Codex Auth app (lazy initialized)
    pub codex_auth_app: Option<CodexAuthApp>,
    /// Last Codex Auth initialization error for placeholder rendering
    pub codex_auth_error: Option<String>,
    /// Last codex auth action info (action_type, account_name, success, error)
    pub last_codex_action: Option<(String, String, bool, Option<String>)>,
    /// 🖱️ Cached header (tab bar) area for mouse hit-testing
    pub header_area: Cell<Option<Rect>>,
    /// 🖱️ Cached profile list area for mouse hit-testing
    pub list_area: Cell<Option<Rect>>,
}

impl App {
    fn build_profile_tab_data(
        instance: &Arc<dyn PlatformConfig>,
    ) -> (Vec<ProfileItem>, IndexMap<String, ProfileConfig>) {
        let current = instance.get_current_profile().ok().flatten();
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
                (profiles, profile_configs)
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to load {} profiles: {}",
                    instance.platform_name(),
                    e
                );
                (Vec::new(), IndexMap::new())
            }
        }
    }

    fn remember_selected_profile(&mut self) {
        self.selected_profile_name = self.selected_profile().map(|profile| profile.name.clone());
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
        let index = self.current_page * PAGE_SIZE + clamped_index;
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

    fn sync_selection_to_profile_name(&mut self) {
        let total = self.current_profiles().len();
        if total == 0 {
            self.current_page = 0;
            self.selected_index = 0;
            self.selected_profile_name = None;
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

        self.current_page = preferred_index / PAGE_SIZE;
        self.selected_index = preferred_index % PAGE_SIZE;
        self.remember_selected_profile();
    }

    /// Build the app with Claude + Codex tabs only.
    pub fn new() -> Result<Self> {
        let mut tabs = Vec::new();

        for platform in Platform::implemented() {
            // Only keep Claude and Codex platforms
            if !matches!(platform, Platform::Claude | Platform::Codex) {
                continue;
            }

            match create_platform(platform) {
                Ok(instance) => {
                    let (items, profile_configs) = Self::build_profile_tab_data(&instance);

                    match platform {
                        Platform::Claude => {
                            tabs.push(PlatformTab {
                                platform,
                                variant: TabVariant::Profile,
                                label: platform.display_name().to_string(),
                                profiles: items,
                                profile_configs,
                                instance: Some(instance),
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
                                instance: Some(Arc::clone(&instance)),
                            });
                            // Codex Profile tab (profile switching)
                            tabs.push(PlatformTab {
                                platform,
                                variant: TabVariant::Profile,
                                label: "Codex Profile".to_string(),
                                profiles: items,
                                profile_configs,
                                instance: Some(instance),
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
                variant: TabVariant::Profile,
                label: Platform::Claude.display_name().to_string(),
                profiles: Vec::new(),
                profile_configs: IndexMap::new(),
                instance: None,
            });
        }

        let mut app = Self {
            tabs,
            active_tab: 0,
            selected_index: 0,
            current_page: 0,
            selected_profile_name: None,
            toasts: ToastManager::new(),
            last_applied: None,
            codex_auth_app: None,
            codex_auth_error: None,
            last_codex_action: None,
            header_area: Cell::new(None),
            list_area: Cell::new(None),
        };
        app.sync_selection_to_profile_name();
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
        let all = self.current_profiles();
        let start = self.current_page * PAGE_SIZE;
        let end = (start + PAGE_SIZE).min(all.len());
        if start >= all.len() {
            &[]
        } else {
            &all[start..end]
        }
    }

    pub fn total_pages(&self) -> usize {
        let total = self.current_profiles().len();
        if total == 0 {
            1
        } else {
            total.div_ceil(PAGE_SIZE)
        }
    }

    // -- Key to Action mapping (pure logic, no side effects) --

    fn map_key(&self, key: KeyEvent) -> Action {
        // Ctrl+C always quits
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Action::Quit;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Tab => Action::NextTab,
            KeyCode::Left | KeyCode::Char('h') => Action::PrevPage,
            KeyCode::Right | KeyCode::Char('l') => Action::NextPage,
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
                    self.remember_selected_profile();
                    self.active_tab = (self.active_tab + 1) % self.tabs.len();
                    self.sync_selection_to_profile_name();
                    self.notify_tab_activated();
                }
            }
            Action::SwitchTab(idx) => {
                if idx < self.tabs.len() {
                    self.remember_selected_profile();
                    self.active_tab = idx;
                    self.sync_selection_to_profile_name();
                    self.notify_tab_activated();
                }
            }
            Action::SelectPrev => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    self.remember_selected_profile();
                }
            }
            Action::SelectNext => {
                let page_len = self.current_page_profiles().len();
                if page_len > 0 && self.selected_index < page_len - 1 {
                    self.selected_index += 1;
                    self.remember_selected_profile();
                }
            }
            Action::SelectAt(idx) => {
                let page_len = self.current_page_profiles().len();
                if idx < page_len {
                    self.selected_index = idx;
                    self.remember_selected_profile();
                }
            }
            Action::PrevPage => {
                if self.current_page > 0 {
                    self.move_to_page(self.current_page - 1);
                }
            }
            Action::NextPage => {
                if self.current_page < self.total_pages() - 1 {
                    self.move_to_page(self.current_page + 1);
                }
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
                self.toasts.push(Toast::info("已刷新配置列表"));
            }
        }
        Ok(false)
    }

    fn apply_selected(&mut self) {
        let Some(selected) = self.selected_profile() else {
            self.toasts.push(Toast::warning("没有可用的配置"));
            return;
        };

        let tab = &self.tabs[self.active_tab];
        let platform_label = tab.label.clone();
        let profile_name = selected.name.clone();
        self.selected_profile_name = Some(profile_name.clone());

        if let Some(instance) = &tab.instance {
            match instance.apply_profile(&profile_name) {
                Ok(()) => {
                    self.toasts
                        .push(Toast::success(format!("✅ 已切换到: {}", profile_name)));
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
                    self.toasts
                        .push(Toast::error(format!("❌ 切换失败: {}", err_msg)));
                    self.last_applied = Some((platform_label, profile_name, false, Some(err_msg)));
                }
            }
        } else {
            self.toasts.push(Toast::error("平台未初始化"));
        }
    }

    fn reload_profiles(&mut self) {
        for tab in &mut self.tabs {
            if tab.variant != TabVariant::Profile {
                continue;
            }
            if let Some(instance) = &tab.instance {
                let (profiles, profile_configs) = Self::build_profile_tab_data(instance);
                tab.profiles = profiles;
                tab.profile_configs = profile_configs;
            }
        }
        self.sync_selection_to_profile_name();
    }

    // -- Tab helpers --

    /// Ensure Codex Auth app is initialized before interaction/rendering
    fn ensure_codex_auth_app(&mut self) {
        if self.codex_auth_app.is_some() {
            return;
        }

        match CodexAuthApp::new() {
            Ok(app) => {
                self.codex_auth_app = Some(app);
                self.codex_auth_error = None;
            }
            Err(e) => {
                let err = e.to_string();
                tracing::warn!("Failed to init CodexAuthApp: {}", err);
                self.codex_auth_error = Some(err.clone());
                self.toasts
                    .push(Toast::error(format!("Codex Auth 初始化失败: {}", err)));
            }
        }
    }

    /// Get mutable Codex Auth app, initializing it on demand
    fn codex_auth_app_mut(&mut self) -> Option<&mut CodexAuthApp> {
        self.ensure_codex_auth_app();
        self.codex_auth_app.as_mut()
    }

    /// Check if the currently active tab is the Codex Auth variant
    pub fn is_codex_auth_tab(&self) -> bool {
        self.tabs[self.active_tab].variant == TabVariant::CodexAuth
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

    /// Notify the active tab's sub-app that it became active
    fn notify_tab_activated(&mut self) {
        if !self.is_codex_auth_tab() {
            self.sync_selection_to_profile_name();
            return;
        }

        if let Some(codex_app) = self.codex_auth_app_mut() {
            codex_app.on_activated();
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

        if self.is_codex_auth_tab() {
            // Tab key: switch to next tab (intercepted before CodexAuthApp)
            if key.code == KeyCode::Tab {
                return self.dispatch(Action::NextTab);
            }
            // Delegate all other keys to CodexAuthApp
            if let Some(codex_app) = self.codex_auth_app_mut() {
                let quit = codex_app.handle_key(key)?;
                if quit {
                    self.last_codex_action = codex_app.last_action.clone();
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
                if self.is_codex_auth_tab() {
                    return self.delegate_mouse_to_codex(mouse);
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
                if self.is_codex_auth_tab() {
                    return self.delegate_mouse_to_codex(mouse);
                }
                return self.dispatch(Action::SelectPrev);
            }

            // 🖱️ 滚轮下
            MouseEventKind::ScrollDown => {
                if self.is_codex_auth_tab() {
                    return self.delegate_mouse_to_codex(mouse);
                }
                return self.dispatch(Action::SelectNext);
            }

            _ => {}
        }
        Ok(false)
    }

    fn on_tick(&mut self) -> bool {
        if self.is_codex_auth_tab() {
            self.codex_auth_app.as_mut().is_some_and(|a| a.on_tick())
        } else {
            self.toasts.tick()
        }
    }

    fn render(&self, frame: &mut Frame) {
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
}
