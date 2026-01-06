// 📱 TUI 应用状态机
// 管理双Tab配置选择器的状态

use crate::core::error::Result;
use crate::models::platform::{Platform, PlatformConfig};
use crate::platforms::create_platform;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;

/// 每页最多显示的配置数量
pub const PAGE_SIZE: usize = 20;

/// 🏷️ Tab状态 - Claude 或 Codex
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabState {
    Claude,
    Codex,
}

impl TabState {
    /// 切换到另一个Tab
    pub fn toggle(&self) -> Self {
        match self {
            TabState::Claude => TabState::Codex,
            TabState::Codex => TabState::Claude,
        }
    }

    /// 获取Tab标题
    pub fn title(&self) -> &'static str {
        match self {
            TabState::Claude => "Claude",
            TabState::Codex => "Codex",
        }
    }

    /// 获取对应的Platform枚举
    pub fn platform(&self) -> Platform {
        match self {
            TabState::Claude => Platform::Claude,
            TabState::Codex => Platform::Codex,
        }
    }
}

/// 📋 配置项信息
#[derive(Debug, Clone)]
pub struct ProfileItem {
    /// 配置名称
    pub name: String,
    /// 配置描述
    pub description: Option<String>,
    /// 是否是当前激活的配置
    pub is_current: bool,
}

/// 📱 TUI 应用
pub struct App {
    /// 当前Tab
    pub current_tab: TabState,
    /// Claude配置列表
    pub claude_profiles: Vec<ProfileItem>,
    /// Codex配置列表
    pub codex_profiles: Vec<ProfileItem>,
    /// 当前选中索引 (在当前页内的索引)
    pub selected_index: usize,
    /// 当前页码 (从0开始)
    pub current_page: usize,
    /// 状态消息 (消息, 是否错误)
    pub status_message: Option<(String, bool)>,
    /// 是否应该退出
    pub should_quit: bool,
    /// Claude平台实例
    claude_platform: Option<Arc<dyn PlatformConfig>>,
    /// Codex平台实例
    codex_platform: Option<Arc<dyn PlatformConfig>>,
    /// 最后应用的配置信息 (平台, 配置名, 是否成功, 错误信息)
    pub last_applied: Option<(String, String, bool, Option<String>)>,
}

impl App {
    /// 🏗️ 创建新的应用实例
    pub fn new() -> Result<Self> {
        let mut app = Self {
            current_tab: TabState::Claude,
            claude_profiles: Vec::new(),
            codex_profiles: Vec::new(),
            selected_index: 0,
            current_page: 0,
            status_message: None,
            should_quit: false,
            claude_platform: None,
            codex_platform: None,
            last_applied: None,
        };

        // 加载配置
        app.load_profiles()?;

        Ok(app)
    }

    /// 📖 加载所有平台的配置
    fn load_profiles(&mut self) -> Result<()> {
        // 加载 Claude 配置
        match create_platform(Platform::Claude) {
            Ok(platform) => {
                let current = platform.get_current_profile().ok().flatten();
                match platform.load_profiles() {
                    Ok(profiles) => {
                        self.claude_profiles = profiles
                            .into_iter()
                            .map(|(name, config)| ProfileItem {
                                is_current: current.as_ref() == Some(&name),
                                description: config.description.clone(),
                                name,
                            })
                            .collect();
                        self.claude_platform = Some(platform);
                    }
                    Err(e) => {
                        tracing::warn!("加载 Claude 配置失败: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("创建 Claude 平台失败: {}", e);
            }
        }

        // 加载 Codex 配置
        match create_platform(Platform::Codex) {
            Ok(platform) => {
                let current = platform.get_current_profile().ok().flatten();
                match platform.load_profiles() {
                    Ok(profiles) => {
                        self.codex_profiles = profiles
                            .into_iter()
                            .map(|(name, config)| ProfileItem {
                                is_current: current.as_ref() == Some(&name),
                                description: config.description.clone(),
                                name,
                            })
                            .collect();
                        self.codex_platform = Some(platform);
                    }
                    Err(e) => {
                        tracing::warn!("加载 Codex 配置失败: {}", e);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("创建 Codex 平台失败: {}", e);
            }
        }

        Ok(())
    }

    /// 📋 获取当前Tab的所有配置列表
    pub fn current_profiles(&self) -> &[ProfileItem] {
        match self.current_tab {
            TabState::Claude => &self.claude_profiles,
            TabState::Codex => &self.codex_profiles,
        }
    }

    /// 📄 获取当前页的配置列表
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

    /// 📊 获取总页数
    pub fn total_pages(&self) -> usize {
        let total = self.current_profiles().len();
        if total == 0 {
            1
        } else {
            total.div_ceil(PAGE_SIZE)
        }
    }

    /// 🔄 切换Tab
    fn switch_tab(&mut self) {
        self.current_tab = self.current_tab.toggle();
        // 重置页码和选中索引
        self.current_page = 0;
        self.selected_index = 0;
    }

    /// ◀️ 上一页
    fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.selected_index = 0;
        }
    }

    /// ▶️ 下一页
    fn next_page(&mut self) {
        if self.current_page < self.total_pages() - 1 {
            self.current_page += 1;
            self.selected_index = 0;
        }
    }

    /// ⬆️ 向上选择
    fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// ⬇️ 向下选择
    fn select_next(&mut self) {
        let page_items = self.current_page_profiles().len();
        if page_items > 0 && self.selected_index < page_items - 1 {
            self.selected_index += 1;
        }
    }

    /// ✅ 应用选中的配置
    fn apply_selected(&mut self) {
        let page_profiles = self.current_page_profiles();
        if page_profiles.is_empty() {
            self.set_status("没有可用的配置".to_string(), true);
            return;
        }

        let selected = &page_profiles[self.selected_index];
        let platform = match self.current_tab {
            TabState::Claude => &self.claude_platform,
            TabState::Codex => &self.codex_platform,
        };

        if let Some(platform) = platform {
            let platform_name = self.current_tab.title().to_string();
            let profile_name = selected.name.clone();
            match platform.apply_profile(&profile_name) {
                Ok(()) => {
                    self.set_status(format!("✅ 已切换到: {}", profile_name), false);
                    self.last_applied = Some((platform_name, profile_name, true, None));
                    // 重新加载配置以更新当前状态
                    let _ = self.load_profiles();
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    self.set_status(format!("❌ 切换失败: {}", err_msg), true);
                    self.last_applied = Some((platform_name, profile_name, false, Some(err_msg)));
                }
            }
        } else {
            self.set_status("平台未初始化".to_string(), true);
        }
    }

    /// 📝 设置状态消息
    pub fn set_status(&mut self, message: String, is_error: bool) {
        self.status_message = Some((message, is_error));
    }

    /// ⌨️ 处理键盘输入
    ///
    /// 返回: true 表示应该退出应用
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        // Ctrl+C 强制退出
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(true);
        }

        match key.code {
            // 退出
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                return Ok(true);
            }

            // Tab 键切换平台
            KeyCode::Tab => {
                self.switch_tab();
            }

            // 左右方向键翻页
            KeyCode::Left | KeyCode::Char('h') => {
                self.prev_page();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.next_page();
            }

            // 上下选择
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
            }

            // 应用配置并退出
            KeyCode::Enter => {
                self.apply_selected();
                self.should_quit = true;
                return Ok(true);
            }

            // 应用配置但不退出 (Space键)
            KeyCode::Char(' ') => {
                self.apply_selected();
            }

            _ => {}
        }

        Ok(false)
    }
}
