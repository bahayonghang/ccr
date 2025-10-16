// 📱 TUI 应用状态机
// 管理应用状态、Tab 切换和用户交互

use crate::core::error::Result;
use crate::services::{ConfigService, HistoryService, SettingsService};
use crate::utils::Validatable;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// 📑 Tab 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabState {
    /// 配置列表
    Configs,
    /// 历史记录
    History,
    /// 系统信息
    System,
}

impl TabState {
    /// 获取下一个 Tab
    pub fn next(&self) -> Self {
        match self {
            Self::Configs => Self::History,
            Self::History => Self::System,
            Self::System => Self::Configs,
        }
    }

    /// 获取上一个 Tab
    pub fn previous(&self) -> Self {
        match self {
            Self::Configs => Self::System,
            Self::History => Self::Configs,
            Self::System => Self::History,
        }
    }

    /// 获取 Tab 标题
    pub fn title(&self) -> &'static str {
        match self {
            Self::Configs => "📋 Configs",
            Self::History => "📜 History",
            Self::System => "⚙️  System",
        }
    }
}

/// 📱 TUI 应用
pub struct App {
    /// 当前 Tab
    pub current_tab: TabState,
    /// YOLO 模式状态
    pub yolo_mode: bool,
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
}

impl App {
    /// 🏗️ 创建新的应用实例
    pub fn new() -> Result<Self> {
        let config_service = ConfigService::default()?;
        let history_service = HistoryService::default()?;
        let settings_service = SettingsService::default()?;

        // 读取 YOLO 模式状态
        let config = config_service.load_config()?;
        let yolo_mode = config.settings.yolo_mode;

        Ok(Self {
            current_tab: TabState::Configs,
            yolo_mode,
            config_service,
            history_service,
            settings_service,
            config_list_index: 0,
            history_list_index: 0,
            should_quit: false,
            status_message: None,
        })
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
            }
            KeyCode::BackTab => {
                self.current_tab = self.current_tab.previous();
            }

            // 数字键: 快速切换 Tab
            KeyCode::Char('1') => self.current_tab = TabState::Configs,
            KeyCode::Char('2') => self.current_tab = TabState::History,
            KeyCode::Char('3') => self.current_tab = TabState::System,

            // Y: 切换 YOLO 模式
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.yolo_mode = !self.yolo_mode;
                // TODO: 保存 YOLO 状态到配置文件
            }

            // 上下键: 列表导航
            KeyCode::Up | KeyCode::Char('k') => match self.current_tab {
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
            },
            KeyCode::Down | KeyCode::Char('j') => {
                match self.current_tab {
                    TabState::Configs => {
                        // 检查配置列表长度
                        if let Ok(config_list) = self.config_service.list_configs() {
                            if !config_list.configs.is_empty()
                                && self.config_list_index < config_list.configs.len() - 1
                            {
                                self.config_list_index += 1;
                            }
                        }
                    }
                    TabState::History => {
                        // 检查历史列表长度
                        if let Ok(history_list) = self.history_service.get_recent(100) {
                            if !history_list.is_empty()
                                && self.history_list_index < history_list.len() - 1
                            {
                                self.history_list_index += 1;
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Enter: 执行操作
            KeyCode::Enter => {
                match self.current_tab {
                    TabState::Configs => {
                        // 切换到选中的配置
                        self.switch_config();
                    }
                    _ => {}
                }
            }

            // d: 删除配置
            KeyCode::Char('d') | KeyCode::Char('D') => {
                match self.current_tab {
                    TabState::Configs => {
                        // 删除选中的配置
                        self.delete_config();
                    }
                    _ => {}
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
                self.status_message = Some((format!("Failed to load configs: {}", e), true));
                return;
            }
        };

        // 检查索引有效性
        if self.config_list_index >= config_list.configs.len() {
            self.status_message = Some(("Invalid config index".to_string(), true));
            return;
        }

        let selected_config = &config_list.configs[self.config_list_index];
        let config_name = selected_config.name.clone();

        // 检查是否已经是当前配置
        if selected_config.is_current {
            self.status_message = Some((format!("Already using config: {}", config_name), false));
            return;
        }

        // 获取完整配置节
        let section = match self.config_service.get_config(&config_name) {
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
                }
            }
            Err(e) => {
                self.status_message = Some((format!("Failed to get config: {}", e), true));
                return;
            }
        };

        // 验证配置
        if let Err(e) = section.validate() {
            self.status_message = Some((format!("Config validation failed: {}", e), true));
            return;
        }

        // 应用配置到 settings.json
        if let Err(e) = self.settings_service.apply_config(&section) {
            self.status_message = Some((format!("Failed to apply config: {}", e), true));
            return;
        }

        // 更新配置文件中的 current_config
        if let Err(e) = self.config_service.set_current(&config_name) {
            self.status_message = Some((format!("Failed to update current config: {}", e), true));
            return;
        }

        // 成功！
        self.status_message = Some((format!("✅ Switched to config: {}", config_name), false));
    }

    /// 🗑️ 删除配置
    fn delete_config(&mut self) {
        // TUI 中删除配置需要 YOLO 模式
        if !self.yolo_mode {
            self.status_message = Some((
                "⚠️ YOLO mode required to delete configs in TUI (Press Y)".to_string(),
                true,
            ));
            return;
        }

        // 获取配置列表
        let config_list = match self.config_service.list_configs() {
            Ok(list) => list,
            Err(e) => {
                self.status_message = Some((format!("Failed to load configs: {}", e), true));
                return;
            }
        };

        // 检查索引有效性
        if self.config_list_index >= config_list.configs.len() {
            self.status_message = Some(("Invalid config index".to_string(), true));
            return;
        }

        let selected_config = &config_list.configs[self.config_list_index];
        let config_name = selected_config.name.clone();

        // 不允许删除当前配置
        if selected_config.is_current {
            self.status_message = Some((
                format!("❌ Cannot delete current config: {}", config_name),
                true,
            ));
            return;
        }

        // 不允许删除默认配置
        if selected_config.is_default {
            self.status_message = Some((
                format!("❌ Cannot delete default config: {}", config_name),
                true,
            ));
            return;
        }

        // 执行删除
        if let Err(e) = self.config_service.delete_config(&config_name) {
            self.status_message = Some((format!("Failed to delete config: {}", e), true));
            return;
        }

        // 调整索引(如果删除的是最后一个)
        if self.config_list_index >= config_list.configs.len() - 1 && self.config_list_index > 0 {
            self.config_list_index -= 1;
        }

        // 成功！
        self.status_message = Some((format!("✅ Deleted config: {}", config_name), false));
    }
}
