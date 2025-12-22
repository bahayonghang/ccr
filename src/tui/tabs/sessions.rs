//! 📚 Sessions Tab
//!
//! TUI Session 管理标签页

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::models::Platform;
use crate::sessions::models::SessionFilter;
use crate::sessions::{SessionIndexer, SessionSummary};

/// Sessions Tab 状态
pub struct SessionsTab {
    /// Session 列表
    sessions: Vec<SessionSummary>,
    /// 当前选中索引
    selected_index: usize,
    /// 平台过滤器
    platform_filter: Option<Platform>,
    /// 是否需要刷新
    needs_refresh: bool,
    /// 错误信息
    error: Option<String>,
}

impl Default for SessionsTab {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl SessionsTab {
    /// 创建新的 Sessions Tab
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            selected_index: 0,
            platform_filter: None,
            needs_refresh: true,
            error: None,
        }
    }

    /// 刷新 Session 列表
    pub fn refresh(&mut self) {
        self.needs_refresh = false;
        self.error = None;

        match SessionIndexer::new() {
            Ok(indexer) => {
                // 先尝试索引
                let _ = indexer.index_all();

                // 获取列表
                let filter = SessionFilter {
                    platform: self.platform_filter,
                    limit: Some(50),
                    ..Default::default()
                };

                match indexer.list(filter) {
                    Ok(sessions) => {
                        self.sessions = sessions;
                        if self.selected_index >= self.sessions.len() && !self.sessions.is_empty() {
                            self.selected_index = self.sessions.len() - 1;
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("加载失败: {}", e));
                    }
                }
            }
            Err(e) => {
                self.error = Some(format!("初始化失败: {}", e));
            }
        }
    }

    /// 向上移动选择
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// 向下移动选择
    pub fn move_down(&mut self) {
        if !self.sessions.is_empty() && self.selected_index < self.sessions.len() - 1 {
            self.selected_index += 1;
        }
    }

    /// 切换平台过滤
    pub fn toggle_platform(&mut self) {
        self.platform_filter = match self.platform_filter {
            None => Some(Platform::Claude),
            Some(Platform::Claude) => Some(Platform::Codex),
            Some(Platform::Codex) => Some(Platform::Gemini),
            Some(Platform::Gemini) => None,
            Some(_) => None,
        };
        self.needs_refresh = true;
    }

    /// 获取选中的 Session
    pub fn selected_session(&self) -> Option<&SessionSummary> {
        self.sessions.get(self.selected_index)
    }

    /// 渲染 Tab
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // 如果需要刷新，先刷新
        if self.needs_refresh {
            self.refresh();
        }

        // 分割布局：左侧列表 70%，右侧详情 30%
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        // 渲染列表
        self.render_list(frame, chunks[0]);

        // 渲染详情
        self.render_detail(frame, chunks[1]);
    }

    /// 渲染 Session 列表
    fn render_list(&self, frame: &mut Frame, area: Rect) {
        let filter_text = match &self.platform_filter {
            None => "All".to_string(),
            Some(p) => format!("{:?}", p),
        };

        let title = format!("📚 Sessions [{}] ({} 个)", filter_text, self.sessions.len());

        let items: Vec<ListItem> = self
            .sessions
            .iter()
            .enumerate()
            .map(|(i, session)| {
                let platform_icon = match session.platform {
                    Platform::Claude => "🔮",
                    Platform::Codex => "🐙",
                    Platform::Gemini => "💎",
                    _ => "📦",
                };

                let title = session.display_title();
                let title_short = if title.len() > 40 {
                    format!("{}...", &title[..37])
                } else {
                    title.to_string()
                };

                let style = if i == self.selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let content = Line::from(vec![
                    Span::raw(platform_icon),
                    Span::raw(" "),
                    Span::styled(title_short, style),
                    Span::raw(" "),
                    Span::styled(
                        session.relative_time(),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]);

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(list, area);
    }

    /// 渲染详情面板
    fn render_detail(&self, frame: &mut Frame, area: Rect) {
        let content = if let Some(ref error) = self.error {
            vec![
                Line::from(vec![Span::styled(
                    "❌ 错误",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(Span::raw(error.as_str())),
            ]
        } else if let Some(session) = self.selected_session() {
            vec![
                Line::from(vec![Span::styled(
                    "📋 详情",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("ID: ", Style::default().fg(Color::Gray)),
                    Span::raw(&session.id),
                ]),
                Line::from(vec![
                    Span::styled("平台: ", Style::default().fg(Color::Gray)),
                    Span::raw(format!("{:?}", session.platform)),
                ]),
                Line::from(vec![
                    Span::styled("消息: ", Style::default().fg(Color::Gray)),
                    Span::raw(format!("{}", session.message_count)),
                ]),
                Line::from(vec![
                    Span::styled("时间: ", Style::default().fg(Color::Gray)),
                    Span::raw(session.relative_time()),
                ]),
                Line::from(vec![
                    Span::styled("目录: ", Style::default().fg(Color::Gray)),
                    Span::raw(&session.cwd),
                ]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "[R] 恢复  [F] 过滤  [Refresh] R",
                    Style::default().fg(Color::DarkGray),
                )]),
            ]
        } else {
            vec![
                Line::from(vec![Span::styled(
                    "📋 详情",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(""),
                Line::from(Span::styled(
                    "无 Session 数据",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "运行 AI CLI 创建 session",
                    Style::default().fg(Color::DarkGray),
                )),
            ]
        };

        let detail = Paragraph::new(content).block(
            Block::default()
                .title("详情")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );

        frame.render_widget(detail, area);
    }

    /// 请求刷新
    pub fn request_refresh(&mut self) {
        self.needs_refresh = true;
    }
}
