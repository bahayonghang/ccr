// TUI UI 渲染模块
// 负责渲染所有 UI 组件

use super::app::{App, TabState};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};

/// 渲染主 UI
pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(5), // Footer (状态消息1行 + 快捷键3行 + 安全边距1行)
        ])
        .split(f.area());

    // 渲染标题栏
    render_header(f, app, chunks[0]);

    // 渲染内容区
    render_content(f, app, chunks[1]);

    // 渲染状态栏
    render_footer(f, app, chunks[2]);
}

/// 渲染标题栏
fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec![
        TabState::Configs.title(),
        TabState::History.title(),
        TabState::Sync.title(),
        TabState::System.title(),
    ];

    let index = match app.current_tab {
        TabState::Configs => 0,
        TabState::History => 1,
        TabState::Sync => 2,
        TabState::System => 3,
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" CCR TUI - Claude Code Configuration Manager ")
                .title_alignment(Alignment::Center),
        )
        .select(index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

/// 渲染内容区
fn render_content(f: &mut Frame, app: &mut App, area: Rect) {
    match app.current_tab {
        TabState::Configs => render_configs_tab(f, app, area),
        TabState::History => render_history_tab(f, app, area),
        TabState::Sync => render_sync_tab(f, app, area),
        TabState::System => render_system_tab(f, app, area),
    }
}

/// 渲染配置列表 Tab
fn render_configs_tab(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Configuration List ")
        .title_alignment(Alignment::Left);

    // 获取配置列表
    let config_list = match app.config_service.list_configs() {
        Ok(list) => list,
        Err(e) => {
            // 出错时显示错误信息
            let error_text = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "❌ Failed to load configurations",
                    Style::default().fg(Color::Red),
                )),
                Line::from(""),
                Line::from(format!("Error: {}", e)),
            ];
            let paragraph = Paragraph::new(error_text)
                .block(block)
                .alignment(Alignment::Left);
            f.render_widget(paragraph, area);
            return;
        }
    };

    if config_list.configs.is_empty() {
        // 没有配置时显示提示
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No configurations found",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from("Run 'ccr init' to create a configuration file."),
        ];
        let paragraph = Paragraph::new(empty_text)
            .block(block)
            .alignment(Alignment::Left);
        f.render_widget(paragraph, area);
        return;
    }

    // 构建配置列表项
    use ratatui::widgets::{List, ListItem};

    let items: Vec<ListItem> = config_list
        .configs
        .iter()
        .map(|config| {
            // 构建显示文本
            let mut markers = Vec::new();
            if config.is_current {
                markers.push("▶");
            }
            if config.is_default {
                markers.push("⭐");
            }

            let marker_text = if !markers.is_empty() {
                format!("[{}] ", markers.join(" "))
            } else {
                "    ".to_string()
            };

            let display_text = format!("{}{} - {}", marker_text, config.name, config.description);

            // 根据是否是当前配置设置颜色
            let style = if config.is_current {
                Style::default().fg(Color::Green)
            } else if config.is_default {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(display_text).style(style)
        })
        .collect();

    // 创建列表组件
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        )
        .highlight_symbol(">> ");

    // 创建列表状态
    use ratatui::widgets::ListState;
    let mut list_state = ListState::default();
    list_state.select(Some(app.config_list_index));

    // 渲染列表
    f.render_stateful_widget(list, area, &mut list_state);
}

/// 渲染历史记录 Tab
fn render_history_tab(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Operation History ")
        .title_alignment(Alignment::Left);

    // 获取历史记录
    let history_list = match app.history_service.get_recent(50) {
        Ok(list) => list,
        Err(e) => {
            // 出错时显示错误信息
            let error_text = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "❌ Failed to load history",
                    Style::default().fg(Color::Red),
                )),
                Line::from(""),
                Line::from(format!("Error: {}", e)),
            ];
            let paragraph = Paragraph::new(error_text)
                .block(block)
                .alignment(Alignment::Left);
            f.render_widget(paragraph, area);
            return;
        }
    };

    if history_list.is_empty() {
        // 没有历史记录时显示提示
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No operation history found",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from("History will appear here after you perform operations."),
        ];
        let paragraph = Paragraph::new(empty_text)
            .block(block)
            .alignment(Alignment::Left);
        f.render_widget(paragraph, area);
        return;
    }

    // 构建历史记录列表项
    use crate::managers::history::OperationResult;
    use ratatui::widgets::{List, ListItem};

    let items: Vec<ListItem> = history_list
        .iter()
        .map(|entry| {
            // 格式化时间戳
            let time = entry.timestamp.format("%m-%d %H:%M:%S").to_string();

            // 操作类型
            let op_type = entry.operation.as_str();

            // 详情(目标配置)
            let target = entry.details.to_config.as_deref().unwrap_or("N/A");

            // 结果图标和颜色
            let (result_icon, result_color) = match &entry.result {
                OperationResult::Success => ("✅", Color::Green),
                OperationResult::Failure(_) => ("❌", Color::Red),
                OperationResult::Warning(_) => ("⚠️", Color::Yellow),
            };

            // 构建显示文本
            let display_text = format!("{} {} {} → {}", time, result_icon, op_type, target);

            ListItem::new(display_text).style(Style::default().fg(result_color))
        })
        .collect();

    // 创建列表组件
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        )
        .highlight_symbol(">> ");

    // 创建列表状态
    use ratatui::widgets::ListState;
    let mut list_state = ListState::default();
    list_state.select(Some(app.history_list_index));

    // 渲染列表
    f.render_stateful_widget(list, area, &mut list_state);
}

/// 渲染云端同步 Tab
fn render_sync_tab(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ☁️  Cloud Sync ")
        .title_alignment(Alignment::Left);

    // 获取同步配置
    let config = match app.config_service.load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            let error_text = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "❌ Failed to load configuration",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(format!("Error: {}", e)),
            ];
            let paragraph = Paragraph::new(error_text)
                .block(block)
                .alignment(Alignment::Center);
            f.render_widget(paragraph, area);
            return;
        }
    };

    let mut lines = vec![Line::from("")];

    match &config.settings.sync {
        Some(sync_config) if sync_config.enabled => {
            // 同步已配置
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("状态: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    "✓ 已启用",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(""));

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("WebDAV 服务器: ", Style::default().fg(Color::Cyan)),
                Span::raw(&sync_config.webdav_url),
            ]));

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("用户名: ", Style::default().fg(Color::Cyan)),
                Span::raw(&sync_config.username),
            ]));

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("远程路径: ", Style::default().fg(Color::Cyan)),
                Span::raw(&sync_config.remote_path),
            ]));

            let auto_sync_status = if sync_config.auto_sync {
                Span::styled("✓ 开启", Style::default().fg(Color::Green))
            } else {
                Span::styled("✗ 关闭", Style::default().fg(Color::DarkGray))
            };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("自动同步: ", Style::default().fg(Color::Cyan)),
                auto_sync_status,
            ]));

            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  📝 可用操作:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from("     [P] Push   - 上传配置到云端"));
            lines.push(Line::from("     [L] Pull   - 从云端下载配置"));
            lines.push(Line::from("     [S] Status - 查看同步状态"));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  💡 提示: 这些操作会在退出 TUI 后在命令行执行",
                Style::default().fg(Color::DarkGray),
            )));
        }
        _ => {
            // 同步未配置
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("状态: ", Style::default().fg(Color::Cyan)),
                Span::styled("未配置", Style::default().fg(Color::Yellow)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled(
                "  📝 配置 WebDAV 同步",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from("  1. 退出 TUI (按 Q)"));
            lines.push(Line::from("  2. 运行命令: ccr sync config"));
            lines.push(Line::from("  3. 输入 WebDAV 服务器信息"));
            lines.push(Line::from("  4. 测试连接成功后即可使用"));
            lines.push(Line::from(""));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled(
                "  💡 支持的服务:",
                Style::default().fg(Color::Cyan),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from("     • 坚果云 (推荐，免费)"));
            lines.push(Line::from("     • Nextcloud / ownCloud"));
            lines.push(Line::from("     • 其他标准 WebDAV 服务"));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);
    f.render_widget(paragraph, area);
}

/// 渲染系统信息 Tab
fn render_system_tab(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" System Information ")
        .title_alignment(Alignment::Left);

    // 获取系统信息
    let hostname = whoami::devicename();
    let username = whoami::username();
    let os = whoami::platform().to_string();

    // 获取 CCR 版本
    let ccr_version = env!("CARGO_PKG_VERSION");

    // 获取配置路径
    let home = dirs::home_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let config_path = format!("{}/.ccs_config.toml", home);
    let settings_path = format!("{}/.claude/settings.json", home);
    let history_path = format!("{}/.claude/ccr_history.json", home);

    // 获取当前配置名称
    let current_config = app
        .config_service
        .get_current()
        .ok()
        .map(|c| c.name)
        .unwrap_or_else(|| "N/A".to_string());

    // 自动确认模式状态
    let auto_confirm_status = if app.auto_confirm_mode {
        Span::styled(
            "ON (session-only)",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled("OFF", Style::default().fg(Color::Green))
    };

    // 构建显示内容
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "System Information",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Hostname: "),
            Span::styled(hostname, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("  User: "),
            Span::styled(username, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("  OS: "),
            Span::styled(os, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "CCR Information",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Version: "),
            Span::styled(ccr_version, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("  Current Config: "),
            Span::styled(current_config, Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![Span::raw("  Auto-Confirm (Y): "), auto_confirm_status]),
        Line::from(""),
        Line::from(Span::styled(
            "File Paths",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Config: "),
            Span::styled(config_path, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("  Settings: "),
            Span::styled(settings_path, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("  History: "),
            Span::styled(history_path, Style::default().fg(Color::Yellow)),
        ]),
    ];

    let paragraph = Paragraph::new(text).block(block).alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    // 渲染状态消息和快捷键帮助
    // 如果有状态消息,显示在顶部
    if let Some((message, is_error)) = &app.status_message {
        // 分割为两行：状态消息1行 + 快捷键至少3行（包括边框）
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // 状态消息
                Constraint::Min(3),    // 快捷键（至少3行，包括边框）
            ])
            .split(area);

        // 渲染状态消息
        let style = if *is_error {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        };

        let status = Paragraph::new(Line::from(Span::styled(format!(" {} ", message), style)))
            .alignment(Alignment::Left);

        f.render_widget(status, chunks[0]);

        // 渲染快捷键
        render_help_line(f, app, chunks[1]);
    } else {
        // 没有状态消息,只渲染快捷键
        render_help_line(f, app, area);
    }
}

/// 渲染快捷键帮助行
fn render_help_line(f: &mut Frame, app: &App, area: Rect) {
    let confirm_status = if app.auto_confirm_mode {
        Span::styled(
            " AUTO ",
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(" SAFE ", Style::default().fg(Color::Green).bg(Color::Black))
    };

    let help_text = vec![
        Span::raw(
            " [1-4] Tab | [↑↓/jk] Nav | [Enter] Switch | [d] Delete | [P/L/S] Sync | [Y] Auto | ",
        ),
        confirm_status,
        Span::raw(" | [Q] Quit "),
    ];

    let footer = Paragraph::new(Line::from(help_text))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    f.render_widget(footer, area);
}
