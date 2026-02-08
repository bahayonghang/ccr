// TUI application state — Tab-based dispatch (Claude + Codex only)

use crate::core::error::Result;
use crate::models::platform::{Platform, PlatformConfig};
use crate::platforms::create_platform;
use crate::tui::action::Action;
use crate::tui::toast::{Toast, ToastManager};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use std::sync::Arc;

use super::codex_auth::CodexAuthApp;
use super::runtime::TuiApp;
use super::ui;

/// Maximum profiles per page
pub const PAGE_SIZE: usize = 20;

/// A single profile entry for display
#[derive(Debug, Clone)]
pub struct ProfileItem {
    pub name: String,
    pub description: Option<String>,
    pub is_current: bool,
}

/// A tab representing one platform with its profiles loaded
pub struct PlatformTab {
    pub platform: Platform,
    pub profiles: Vec<ProfileItem>,
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
    /// Toast notification manager
    pub toasts: ToastManager,
    /// Last applied profile info (platform_name, profile_name, success, error)
    pub last_applied: Option<(String, String, bool, Option<String>)>,
    /// Embedded Codex Auth app (eagerly initialized)
    pub codex_auth_app: Option<CodexAuthApp>,
    /// Last codex auth action info (action_type, account_name, success, error)
    pub last_codex_action: Option<(String, String, bool, Option<String>)>,
}

impl App {
    /// Build the app with Claude + Codex tabs only.
    pub fn new() -> Result<Self> {
        let mut tabs = Vec::new();

        for platform in Platform::implemented() {
            // Only keep Claude and Codex tabs
            if !matches!(platform, Platform::Claude | Platform::Codex) {
                continue;
            }

            match create_platform(platform) {
                Ok(instance) => {
                    let current = instance.get_current_profile().ok().flatten();
                    match instance.load_profiles() {
                        Ok(profiles) => {
                            let items: Vec<ProfileItem> = profiles
                                .into_iter()
                                .map(|(name, config)| ProfileItem {
                                    is_current: current.as_ref() == Some(&name),
                                    description: config.description.clone(),
                                    name,
                                })
                                .collect();
                            tabs.push(PlatformTab {
                                platform,
                                profiles: items,
                                instance: Some(instance),
                            });
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load {} profiles: {}", platform, e);
                            // Still add core platforms even if load fails
                            tabs.push(PlatformTab {
                                platform,
                                profiles: Vec::new(),
                                instance: Some(instance),
                            });
                        }
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
                profiles: Vec::new(),
                instance: None,
            });
        }

        // Eagerly initialize CodexAuthApp
        let codex_auth_app = match CodexAuthApp::new() {
            Ok(app) => Some(app),
            Err(e) => {
                tracing::warn!("Failed to init CodexAuthApp: {}", e);
                None
            }
        };

        Ok(Self {
            tabs,
            active_tab: 0,
            selected_index: 0,
            current_page: 0,
            toasts: ToastManager::new(),
            last_applied: None,
            codex_auth_app,
            last_codex_action: None,
        })
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
                    self.active_tab = (self.active_tab + 1) % self.tabs.len();
                    self.current_page = 0;
                    self.selected_index = 0;
                }
            }
            Action::SwitchTab(idx) => {
                if idx < self.tabs.len() {
                    self.active_tab = idx;
                    self.current_page = 0;
                    self.selected_index = 0;
                }
            }
            Action::SelectPrev => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            Action::SelectNext => {
                let page_len = self.current_page_profiles().len();
                if page_len > 0 && self.selected_index < page_len - 1 {
                    self.selected_index += 1;
                }
            }
            Action::PrevPage => {
                if self.current_page > 0 {
                    self.current_page -= 1;
                    self.selected_index = 0;
                }
            }
            Action::NextPage => {
                if self.current_page < self.total_pages() - 1 {
                    self.current_page += 1;
                    self.selected_index = 0;
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
        let page_profiles = self.current_page_profiles();
        if page_profiles.is_empty() {
            self.toasts.push(Toast::warning("没有可用的配置"));
            return;
        }

        let selected = &page_profiles[self.selected_index];
        let tab = &self.tabs[self.active_tab];
        let platform_name = tab.platform.display_name().to_string();
        let profile_name = selected.name.clone();

        if let Some(instance) = &tab.instance {
            match instance.apply_profile(&profile_name) {
                Ok(()) => {
                    self.toasts
                        .push(Toast::success(format!("✅ 已切换到: {}", profile_name)));
                    self.last_applied = Some((platform_name, profile_name, true, None));
                    self.reload_profiles();
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    self.toasts
                        .push(Toast::error(format!("❌ 切换失败: {}", err_msg)));
                    self.last_applied = Some((platform_name, profile_name, false, Some(err_msg)));
                }
            }
        } else {
            self.toasts.push(Toast::error("平台未初始化"));
        }
    }

    fn reload_profiles(&mut self) {
        for tab in &mut self.tabs {
            if let Some(instance) = &tab.instance {
                let current = instance.get_current_profile().ok().flatten();
                if let Ok(profiles) = instance.load_profiles() {
                    tab.profiles = profiles
                        .into_iter()
                        .map(|(name, config)| ProfileItem {
                            is_current: current.as_ref() == Some(&name),
                            description: config.description.clone(),
                            name,
                        })
                        .collect();
                }
            }
        }
    }

    // -- Tab helpers --

    /// Check if the currently active tab is the Codex platform
    pub fn is_codex_tab(&self) -> bool {
        self.current_platform() == Platform::Codex
    }

    /// Pre-select Codex tab (for `ccr codex` entry)
    pub fn with_codex_tab(mut self) -> Self {
        if let Some(idx) = self.tabs.iter().position(|t| t.platform == Platform::Codex) {
            self.active_tab = idx;
        }
        self
    }
}

// -- TuiApp trait implementation (tab-based dispatch) --

impl TuiApp for App {
    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        // Ctrl+C always quits the entire TUI
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(true);
        }

        if self.is_codex_tab() {
            // Tab key: switch to next tab (intercepted before CodexAuthApp)
            if key.code == KeyCode::Tab {
                return self.dispatch(Action::NextTab);
            }
            // Delegate all other keys to CodexAuthApp
            if let Some(ref mut codex_app) = self.codex_auth_app {
                let quit = codex_app.handle_key(key)?;
                if quit {
                    self.last_codex_action = codex_app.last_action.clone();
                    return Ok(true);
                }
            }
            Ok(false)
        } else {
            // Claude tab: original key mapping + dispatch
            let action = self.map_key(key);
            self.dispatch(action)
        }
    }

    fn on_tick(&mut self) -> bool {
        if self.is_codex_tab() {
            self.codex_auth_app.as_mut().is_some_and(|a| a.on_tick())
        } else {
            self.toasts.tick()
        }
    }

    fn render(&self, frame: &mut Frame) {
        ui::draw(frame, self);
    }
}
