// 📱 TUI 应用状态机
// 管理应用状态、Tab 切换和用户交互

use crate::core::error::Result;
use crate::services::{ConfigService, HistoryService, SettingsService};
use crate::utils::Validatable;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

/// 📑 Tab 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabState {
    /// 配置列表
    Configs,
    /// 历史记录
    History,
    /// 云端同步
    Sync,
    /// 系统信息
    System,
}

impl TabState {
    /// 获取下一个 Tab
    pub fn next(&self) -> Self {
        match self {
            Self::Configs => Self::History,
            Self::History => Self::Sync,
            Self::Sync => Self::System,
            Self::System => Self::Configs,
        }
    }

    /// 获取上一个 Tab
    pub fn previous(&self) -> Self {
        match self {
            Self::Configs => Self::System,
            Self::History => Self::Configs,
            Self::Sync => Self::History,
            Self::System => Self::Sync,
        }
    }

    /// 获取 Tab 标题
    pub fn title(&self) -> &'static str {
        match self {
            Self::Configs => "📋 Configs",
            Self::History => "📜 History",
            Self::Sync => "☁️  Sync",
            Self::System => "⚙️  System",
        }
    }
}

/// 📱 TUI 应用
pub struct App {
    /// 当前 Tab
    pub current_tab: TabState,
    /// 自动确认模式状态（运行时临时设置）
    pub auto_confirm_mode: bool,
    /// 配置服务
    pub config_service: ConfigService,
    /// 历史服务
    pub history_service: HistoryService,
    /// 设置服务
    pub settings_service: SettingsService,
    /// 配置列表索引
    pub config_list_index: usize,
    /// 历史记录索引
    pub history_list_index: usize,
    /// 是否应该退出
    pub should_quit: bool,
    /// 状态消息 (消息文本, 是否是错误)
    pub status_message: Option<(String, bool)>,
    /// 消息显示帧计数器（确保消息至少显示N帧）
    message_frame_count: u8,
    /// 周期性刷新标记（用于 tick 控制）
    needs_redraw_on_tick: bool,
    /// 上次 tick 时间（用于节流）
    last_tick: Instant,
    /// tick 间隔（毫秒级）
    tick_interval: Duration,
    /// 缓存：配置列表（减少渲染阶段的 I/O）
    cached_config_list: Option<crate::services::config_service::ConfigList>,
    /// 缓存：最近历史记录（减少渲染阶段的 I/O）
    cached_history_list: Option<Vec<crate::managers::history::HistoryEntry>>,
}

impl App {
    /// 🏗️ 创建新的应用实例
    pub fn new() -> Result<Self> {
        let config_service = ConfigService::with_default()?;
        let history_service = HistoryService::with_default()?;
        let settings_service = SettingsService::with_default()?;

        // 读取自动确认模式状态
        let config = config_service.load_config()?;
        let auto_confirm_mode = config.settings.skip_confirmation;

        Ok(Self {
            current_tab: TabState::Configs,
            auto_confirm_mode,
            config_service,
            history_service,
            settings_service,
            config_list_index: 0,
            history_list_index: 0,
            should_quit: false,
            status_message: None,
            message_frame_count: 0,
            needs_redraw_on_tick: false,
            last_tick: Instant::now(),
            tick_interval: Duration::from_millis(250),
            cached_config_list: None,
            cached_history_list: None,
        })
    }

    /// 📝 设置状态消息（自动重置帧计数器）
    pub fn set_status(&mut self, message: String, is_error: bool) {
        self.status_message = Some((message, is_error));
        self.message_frame_count = 3; // 至少显示3帧（约750ms）
        self.needs_redraw_on_tick = true;
    }

    /// 🧹 尝试清除状态消息（仅当帧计数器归零时）
    fn try_clear_status(&mut self) {
        if self.message_frame_count > 0 {
            // 消息受保护，不清除
            return;
        }
        self.status_message = None;
    }

    /// 📉 递减消息帧计数器（在每次渲染时调用）
    pub fn tick_message(&mut self) {
        if self.message_frame_count > 0 {
            self.message_frame_count -= 1;
            self.needs_redraw_on_tick = true;
        } else {
            self.needs_redraw_on_tick = false;
        }
    }

    /// ⏱️ 通用 tick 入口（包含节流与消息处理）
    pub fn tick(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_tick) >= self.tick_interval {
            self.last_tick = now;
            self.tick_message();
        }
    }

    /// 是否需要在 Tick 上重绘
    pub fn should_redraw_on_tick(&self) -> bool {
        self.needs_redraw_on_tick
    }

    /// 🔁 刷新缓存（减少渲染阶段的 I/O）
    pub fn refresh_caches(&mut self) -> Result<()> {
        self.cached_config_list = Some(self.config_service.list_configs()?);
        self.cached_history_list = Some(self.history_service.get_recent(100)?);
        Ok(())
    }

    /// 读取缓存：配置列表
    pub fn get_cached_config_list(&self) -> Option<&crate::services::config_service::ConfigList> {
        self.cached_config_list.as_ref()
    }

    /// 读取缓存：最近历史
    pub fn get_cached_history(&self) -> Option<&Vec<crate::managers::history::HistoryEntry>> {
        self.cached_history_list.as_ref()
    }

    /// ⌨️ 处理键盘输入
    ///
    /// 返回: true 表示应该退出应用
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            // Q 或 Ctrl+C: 退出
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
                return Ok(true);
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return Ok(true);
            }

            // Tab / Shift+Tab: 切换 Tab
            KeyCode::Tab => {
                self.current_tab = self.current_tab.next();
                self.try_clear_status(); // 尝试清除旧状态消息
                let _ = self.refresh_caches();
            }
            KeyCode::BackTab => {
                self.current_tab = self.current_tab.previous();
                self.try_clear_status(); // 尝试清除旧状态消息
                let _ = self.refresh_caches();
            }

            // 数字键: 快速切换 Tab
            KeyCode::Char('1') => {
                self.current_tab = TabState::Configs;
                self.try_clear_status(); // 尝试清除旧状态消息
                let _ = self.refresh_caches();
            }
            KeyCode::Char('2') => {
                self.current_tab = TabState::History;
                self.try_clear_status(); // 尝试清除旧状态消息
                let _ = self.refresh_caches();
            }
            KeyCode::Char('3') => {
                self.current_tab = TabState::Sync;
                self.try_clear_status(); // 尝试清除旧状态消息
                let _ = self.refresh_caches();
            }
            KeyCode::Char('4') => {
                self.current_tab = TabState::System;
                self.try_clear_status(); // 尝试清除旧状态消息
                let _ = self.refresh_caches();
            }

            // P/L/S: Sync 操作（在 Sync 标签页时）
            KeyCode::Char('p') | KeyCode::Char('P') => {
                if self.current_tab == TabState::Sync {
                    self.set_status("💡 退出 TUI 后运行: ccr sync push".to_string(), false);
                }
            }
            KeyCode::Char('l') | KeyCode::Char('L') => {
                if self.current_tab == TabState::Sync {
                    self.set_status("💡 退出 TUI 后运行: ccr sync pull".to_string(), false);
                }
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                if self.current_tab == TabState::Sync {
                    self.set_status("💡 退出 TUI 后运行: ccr sync status".to_string(), false);
                }
            }

            // Y: 切换自动确认模式（仅本次会话有效）
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.auto_confirm_mode = !self.auto_confirm_mode;
                // 注意：此状态仅在当前TUI会话有效，不保存到配置文件
                self.needs_redraw_on_tick = true;
            }

            // 上下键: 列表导航
            KeyCode::Up | KeyCode::Char('k') => {
                self.try_clear_status(); // 尝试清除旧状态消息
                match self.current_tab {
                    TabState::Configs => {
                        if self.config_list_index > 0 {
                            self.config_list_index -= 1;
                        }
                    }
                    TabState::History => {
                        if self.history_list_index > 0 {
                            self.history_list_index -= 1;
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.try_clear_status(); // 尝试清除旧状态消息
                match self.current_tab {
                    TabState::Configs => {
                        // 检查配置列表长度
                        if let Some(config_list) = self.get_cached_config_list()
                            && !config_list.configs.is_empty()
                            && self.config_list_index < config_list.configs.len() - 1
                        {
                            self.config_list_index += 1;
                        }
                    }
                    TabState::History => {
                        // 检查历史列表长度
                        if let Some(history_list) = self.get_cached_history()
                            && !history_list.is_empty()
                            && self.history_list_index < history_list.len() - 1
                        {
                            self.history_list_index += 1;
                        }
                    }
                    _ => {}
                }
            }

            // Enter: 执行操作
            KeyCode::Enter => {
                if self.current_tab == TabState::Configs {
                    // 切换到选中的配置
                    self.switch_config();
                }
            }

            // d: 删除配置
            KeyCode::Char('d') | KeyCode::Char('D') => {
                if self.current_tab == TabState::Configs {
                    // 删除选中的配置
                    self.delete_config();
                }
            }

            _ => {}
        }

        Ok(false)
    }

    /// 🔄 切换配置
    fn switch_config(&mut self) {
        // 获取配置列表
        let config_list = match self.config_service.list_configs() {
            Ok(list) => list,
            Err(e) => {
                self.set_status(format!("Failed to load configs: {}", e), true);
                return;
            }
        };

        // 检查索引有效性
        if self.config_list_index >= config_list.configs.len() {
            self.set_status("Invalid config index".to_string(), true);
            return;
        }

        let selected_config = &config_list.configs[self.config_list_index];

        // 检查是否已经是当前配置
        if selected_config.is_current {
            self.set_status(
                format!("Already using config: {}", selected_config.name),
                false,
            );
            return;
        }

        // 获取完整配置节
        let section = match self.config_service.get_config(&selected_config.name) {
            Ok(info) => {
                // 从 ConfigInfo 构建 ConfigSection
                crate::managers::config::ConfigSection {
                    description: Some(info.description),
                    base_url: info.base_url,
                    auth_token: info.auth_token,
                    model: info.model,
                    small_fast_model: info.small_fast_model,
                    provider: info.provider,
                    provider_type: info.provider_type.as_deref().and_then(|s| match s {
                        "official_relay" => {
                            Some(crate::managers::config::ProviderType::OfficialRelay)
                        }
                        "third_party_model" => {
                            Some(crate::managers::config::ProviderType::ThirdPartyModel)
                        }
                        _ => None,
                    }),
                    account: info.account,
                    tags: info.tags,
                    usage_count: Some(info.usage_count),
                    enabled: Some(info.enabled),
                    other: indexmap::IndexMap::new(),
                }
            }
            Err(e) => {
                self.set_status(format!("Failed to get config: {}", e), true);
                return;
            }
        };

        // 验证配置
        if let Err(e) = section.validate() {
            self.set_status(format!("Config validation failed: {}", e), true);
            return;
        }

        // 应用配置到 settings.json
        if let Err(e) = self.settings_service.apply_config(&section) {
            self.set_status(format!("Failed to apply config: {}", e), true);
            return;
        }

        // 更新配置文件中的 current_config
        if let Err(e) = self.config_service.set_current(&selected_config.name) {
            self.set_status(format!("Failed to update current config: {}", e), true);
            return;
        }

        // 成功！
        self.set_status(
            format!("✅ Switched to config: {}", selected_config.name),
            false,
        );
        // 刷新缓存，确保 UI 与新状态一致
        let _ = self.refresh_caches();
    }

    /// 🗑️ 删除配置
    fn delete_config(&mut self) {
        // TUI 中删除配置需要启用自动确认模式（安全措施）
        if !self.auto_confirm_mode {
            self.set_status(
                "⚠️ Press [Y] to enable Auto-Confirm mode before deleting".to_string(),
                true,
            );
            return;
        }

        // 获取配置列表
        let config_list = match self.config_service.list_configs() {
            Ok(list) => list,
            Err(e) => {
                self.set_status(format!("Failed to load configs: {}", e), true);
                return;
            }
        };

        // 检查索引有效性
        if self.config_list_index >= config_list.configs.len() {
            self.set_status("Invalid config index".to_string(), true);
            return;
        }

        let selected_config = &config_list.configs[self.config_list_index];

        // 不允许删除当前配置
        if selected_config.is_current {
            self.set_status(
                format!("❌ Cannot delete current config: {}", selected_config.name),
                true,
            );
            return;
        }

        // 不允许删除默认配置
        if selected_config.is_default {
            self.set_status(
                format!("❌ Cannot delete default config: {}", selected_config.name),
                true,
            );
            return;
        }

        // 执行删除
        if let Err(e) = self.config_service.delete_config(&selected_config.name) {
            self.set_status(format!("Failed to delete config: {}", e), true);
            return;
        }

        // 调整索引(如果删除的是最后一个)
        if self.config_list_index >= config_list.configs.len() - 1 && self.config_list_index > 0 {
            self.config_list_index -= 1;
        }

        // 成功！
        self.set_status(
            format!("✅ Deleted config: {}", selected_config.name),
            false,
        );
        // 刷新缓存，更新列表与历史
        let _ = self.refresh_caches();
    }
}
