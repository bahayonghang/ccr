// 🔐 Codex Auth TUI 应用状态机
// 管理 Codex 多账号选择器的状态

use crate::core::error::Result;
use crate::models::{CodexAuthItem, LoginState, TokenFreshness};
use crate::services::{CodexAuthService, CodexRollingUsage};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use dirs::home_dir;
use std::path::PathBuf;

/// 每页最多显示的账号数量
pub const PAGE_SIZE: usize = 10;

/// 🎯 操作模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// 正常浏览模式
    Normal,
    /// 确认删除模式
    ConfirmDelete,
    /// 输入保存名称模式
    InputSaveName,
}

/// 📊 使用情况数据状态
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum UsageState {
    /// 加载中
    Loading,
    /// 加载成功
    Loaded(CodexRollingUsage),
    /// 加载失败
    Error(String),
    /// 无数据
    NoData,
}

/// 📱 Codex Auth TUI 应用
pub struct CodexAuthApp {
    /// 账号列表
    pub accounts: Vec<CodexAuthItem>,
    /// 当前选中索引
    pub selected_index: usize,
    /// 当前页码 (从0开始)
    pub current_page: usize,
    /// 当前模式
    pub mode: Mode,
    /// 状态消息 (消息, 是否错误)
    pub status_message: Option<(String, bool)>,
    /// 是否应该退出
    pub should_quit: bool,
    /// 登录状态
    pub login_state: LoginState,
    /// 服务实例
    service: CodexAuthService,
    /// 输入缓冲区 (用于保存名称)
    pub input_buffer: String,
    /// 最后操作信息 (操作类型, 账号名, 是否成功, 错误信息)
    pub last_action: Option<(String, String, bool, Option<String>)>,
    /// 使用情况数据状态
    pub usage_state: UsageState,
    /// Codex 目录
    #[allow(dead_code)]
    codex_dir: Option<std::path::PathBuf>,
}

impl CodexAuthApp {
    /// 🏗️ 创建新的应用实例
    pub fn new() -> Result<Self> {
        let service = CodexAuthService::new()?;
        let login_state = service.get_login_state()?;
        let accounts = service.list_accounts()?;

        // 找到当前账号的索引
        let selected_index = accounts.iter().position(|a| a.is_current).unwrap_or(0);

        // 获取 Codex 目录
        let codex_dir = home_dir().map(|d| d.join(".codex"));

        // 初始加载使用情况数据
        let usage_state = Self::load_usage_data(&codex_dir);

        Ok(Self {
            accounts,
            selected_index,
            current_page: 0,
            mode: Mode::Normal,
            status_message: None,
            should_quit: false,
            login_state,
            service,
            input_buffer: String::new(),
            last_action: None,
            usage_state,
            codex_dir,
        })
    }

    /// 📊 加载使用情况数据
    fn load_usage_data(codex_dir: &Option<PathBuf>) -> UsageState {
        let Some(dir) = codex_dir else {
            return UsageState::Error("无法获取用户目录".to_string());
        };

        use crate::services::CodexUsageService;
        let usage_service = CodexUsageService::new(dir.clone());

        match usage_service.compute_rolling_usage() {
            Ok(usage) => {
                if usage.all_time.total_requests == 0 {
                    UsageState::NoData
                } else {
                    UsageState::Loaded(usage)
                }
            }
            Err(e) => UsageState::Error(e.to_string()),
        }
    }

    /// 🔄 刷新使用情况数据
    #[allow(dead_code)]
    pub fn refresh_usage(&mut self) {
        self.usage_state = Self::load_usage_data(&self.codex_dir);
    }

    /// 🔄 重新加载账号列表
    pub fn reload_accounts(&mut self) -> Result<()> {
        self.login_state = self.service.get_login_state()?;
        self.accounts = self.service.list_accounts()?;

        // 确保选中索引有效
        if self.selected_index >= self.accounts.len() {
            self.selected_index = self.accounts.len().saturating_sub(1);
        }

        Ok(())
    }

    /// 📊 获取当前页的账号列表
    pub fn current_page_accounts(&self) -> &[CodexAuthItem] {
        let start = self.current_page * PAGE_SIZE;
        let end = (start + PAGE_SIZE).min(self.accounts.len());
        if start < self.accounts.len() {
            &self.accounts[start..end]
        } else {
            &[]
        }
    }

    /// 📄 获取总页数
    pub fn total_pages(&self) -> usize {
        if self.accounts.is_empty() {
            1
        } else {
            self.accounts.len().div_ceil(PAGE_SIZE)
        }
    }

    /// 🎯 获取当前选中的账号
    pub fn selected_account(&self) -> Option<&CodexAuthItem> {
        let page_accounts = self.current_page_accounts();
        page_accounts.get(self.selected_index)
    }

    /// ⌨️ 处理按键事件
    /// 返回 true 表示应该退出
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        // 清除状态消息
        self.status_message = None;

        match self.mode {
            Mode::Normal => self.handle_normal_mode(key),
            Mode::ConfirmDelete => self.handle_confirm_delete(key),
            Mode::InputSaveName => self.handle_input_save_name(key),
        }
    }

    /// 处理正常模式按键
    fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            // 退出
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                return Ok(true);
            }
            // Ctrl+C 退出
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return Ok(true);
            }
            // 向上移动
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_up();
            }
            // 向下移动
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_down();
            }
            // 上一页
            KeyCode::PageUp | KeyCode::Char('h') => {
                self.prev_page();
            }
            // 下一页
            KeyCode::PageDown | KeyCode::Char('l') => {
                self.next_page();
            }
            // 切换账号
            KeyCode::Enter => {
                if self.switch_selected_account()? {
                    return Ok(true);
                }
            }
            // 保存当前登录
            KeyCode::Char('s') => {
                if matches!(self.login_state, LoginState::LoggedInUnsaved) {
                    self.mode = Mode::InputSaveName;
                    self.input_buffer.clear();
                } else {
                    self.status_message = Some(("当前登录已保存或未登录".to_string(), true));
                }
            }
            // 删除账号
            KeyCode::Char('d') | KeyCode::Delete => {
                if let Some(account) = self.selected_account() {
                    if !account.is_virtual {
                        self.mode = Mode::ConfirmDelete;
                    } else {
                        self.status_message = Some(("无法删除未保存的登录".to_string(), true));
                    }
                }
            }
            // 刷新
            KeyCode::Char('r') => {
                self.reload_accounts()?;
                self.status_message = Some(("已刷新账号列表".to_string(), false));
            }
            _ => {}
        }
        Ok(false)
    }

    /// 处理确认删除模式
    fn handle_confirm_delete(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(account) = self.selected_account().cloned() {
                    match self.service.delete_account(&account.name) {
                        Ok(()) => {
                            self.last_action =
                                Some(("已删除".to_string(), account.name.clone(), true, None));
                            self.status_message =
                                Some((format!("已删除账号: {}", account.name), false));
                            self.reload_accounts()?;
                        }
                        Err(e) => {
                            self.status_message = Some((format!("删除失败: {}", e), true));
                        }
                    }
                }
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.status_message = Some(("已取消删除".to_string(), false));
            }
            _ => {}
        }
        Ok(false)
    }

    /// 处理输入保存名称模式
    fn handle_input_save_name(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Enter => {
                if !self.input_buffer.is_empty() {
                    let name = self.input_buffer.clone();
                    match self.service.save_current(&name, None, None, false) {
                        Ok(()) => {
                            self.last_action =
                                Some(("已保存".to_string(), name.clone(), true, None));
                            self.status_message = Some((format!("已保存账号: {}", name), false));
                            self.reload_accounts()?;
                        }
                        Err(e) => {
                            self.status_message = Some((format!("保存失败: {}", e), true));
                        }
                    }
                }
                self.mode = Mode::Normal;
                self.input_buffer.clear();
            }
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.input_buffer.clear();
                self.status_message = Some(("已取消保存".to_string(), false));
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                // 只允许有效字符
                if (c.is_ascii_alphanumeric() || c == '_' || c == '-')
                    && self.input_buffer.len() < 32
                {
                    self.input_buffer.push(c);
                }
            }
            _ => {}
        }
        Ok(false)
    }

    /// 向上移动选择
    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else if self.current_page > 0 {
            self.current_page -= 1;
            self.selected_index = PAGE_SIZE - 1;
        }
    }

    /// 向下移动选择
    fn move_down(&mut self) {
        let page_accounts = self.current_page_accounts();
        if self.selected_index < page_accounts.len().saturating_sub(1) {
            self.selected_index += 1;
        } else if self.current_page < self.total_pages() - 1 {
            self.current_page += 1;
            self.selected_index = 0;
        }
    }

    /// 上一页
    fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.selected_index = 0;
        }
    }

    /// 下一页
    fn next_page(&mut self) {
        if self.current_page < self.total_pages() - 1 {
            self.current_page += 1;
            self.selected_index = 0;
        }
    }

    /// 切换到选中的账号
    /// 返回 true 表示切换成功，应该退出 TUI
    fn switch_selected_account(&mut self) -> Result<bool> {
        if let Some(account) = self.selected_account().cloned() {
            if account.is_virtual {
                self.status_message = Some(("这是当前登录，无需切换".to_string(), false));
                return Ok(false);
            }

            if account.is_current {
                self.status_message = Some(("已经是当前账号".to_string(), false));
                return Ok(false);
            }

            // 过期检查
            if CodexAuthService::is_expired(account.expires_at) {
                self.status_message = Some(("账号已过期，无法切换".to_string(), true));
                return Ok(false);
            }

            // 检测 Codex 进程
            let running = self.service.detect_codex_process();
            if !running.is_empty() {
                self.status_message = Some((
                    format!("警告: 检测到 {} 个 Codex 进程正在运行", running.len()),
                    true,
                ));
            }

            match self.service.switch_account(&account.name) {
                Ok(()) => {
                    self.last_action =
                        Some(("已切换到".to_string(), account.name.clone(), true, None));
                    self.status_message = Some((format!("已切换到账号: {}", account.name), false));
                    // 切换成功，设置退出标志
                    self.should_quit = true;
                    return Ok(true);
                }
                Err(e) => {
                    self.status_message = Some((format!("切换失败: {}", e), true));
                }
            }
        }
        Ok(false)
    }

    /// 获取新鲜度显示文本
    pub fn freshness_text(freshness: TokenFreshness) -> &'static str {
        match freshness {
            TokenFreshness::Fresh => "🟢 新鲜",
            TokenFreshness::Stale => "🟡 陈旧",
            TokenFreshness::Old => "🔴 过期",
            TokenFreshness::Unknown => "⚪ 未知",
        }
    }
}
