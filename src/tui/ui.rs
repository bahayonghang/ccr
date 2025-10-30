// TUI UI 渲染模块
// 负责渲染所有 UI 组件

use super::app::{App, TabState};
use crate::managers::sync_config::SyncConfigManager;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};
use std::path::PathBuf;

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
                .title(" 🚀 CCR TUI - Claude Code Configuration Manager ")
                .title_alignment(Alignment::Center)
                .style(Style::default().fg(Color::White)),
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
        .title(" ⚙️  Configuration List ")
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
                markers.push("▶️");
            }
            if config.is_default {
                markers.push("⭐");
            }

            let marker_text = if !markers.is_empty() {
                format!("{} ", markers.join(" "))
            } else {
                "   ".to_string()
            };

            let display_text = format!("{} {} - {}", marker_text, config.name, config.description);

            // 根据是否是当前配置设置颜色
            let style = if config.is_current {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
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
        .title(" 📜 Operation History ")
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

            // 操作类型（格式化为固定宽度）
            let op_type = format!("{:<8}", entry.operation.as_str());

            // 详情(目标配置)
            let target = entry.details.to_config.as_deref().unwrap_or("N/A");

            // 结果图标和颜色
            let (result_icon, result_color) = match &entry.result {
                OperationResult::Success => ("✅", Color::Green),
                OperationResult::Failure(_) => ("❌", Color::Red),
                OperationResult::Warning(_) => ("⚠️ ", Color::Yellow),
            };

            // 构建显示文本
            let display_text = format!("[{}] {} {} → {}", time, result_icon, op_type, target);

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

    // 检查配置加载（仅用于错误检查）
    if let Err(e) = app.config_service.load_config() {
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

    let mut lines = vec![Line::from("")];

    // 从独立文件加载sync配置
    let sync_config_result = SyncConfigManager::default().and_then(|m| m.load());

    match sync_config_result {
        Ok(sync_config) if sync_config.enabled => {
            // clone数据以解决生命周期问题
            let webdav_url = sync_config.webdav_url.clone();
            let username = sync_config.username.clone();
            let remote_path = sync_config.remote_path.clone();
            let auto_sync = sync_config.auto_sync;

            // 同步已配置
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("状态         : ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    "✅ 已启用",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::from(""));

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("WebDAV 服务器: ", Style::default().fg(Color::Cyan)),
                Span::styled(webdav_url, Style::default().fg(Color::White)),
            ]));

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("用户名       : ", Style::default().fg(Color::Cyan)),
                Span::styled(username, Style::default().fg(Color::White)),
            ]));

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("远程路径     : ", Style::default().fg(Color::Cyan)),
                Span::styled(remote_path, Style::default().fg(Color::White)),
            ]));

            let auto_sync_status = if auto_sync {
                Span::styled("✅ 开启", Style::default().fg(Color::Green))
            } else {
                Span::styled("⭕ 关闭", Style::default().fg(Color::DarkGray))
            };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("自动同步     : ", Style::default().fg(Color::Cyan)),
                auto_sync_status,
            ]));

            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  ⚡ 可用操作:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled(
                    "[P]",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Push   - 上传配置到云端"),
            ]));
            lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled(
                    "[L]",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Pull   - 从云端下载配置"),
            ]));
            lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled(
                    "[S]",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Status - 查看同步状态"),
            ]));
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
                Span::styled(
                    "⚠️  未配置",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
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
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "1.",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" 退出 TUI (按 "),
                Span::styled(
                    "Q",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(")"),
            ]));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "2.",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" 运行命令: "),
                Span::styled("ccr sync config", Style::default().fg(Color::Green)),
            ]));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "3.",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" 输入 WebDAV 服务器信息"),
            ]));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "4.",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" 测试连接成功后即可使用"),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled(
                "  💡 支持的服务:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled("•", Style::default().fg(Color::Green)),
                Span::raw(" 坚果云 (推荐，免费)"),
            ]));
            lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled("•", Style::default().fg(Color::Green)),
                Span::raw(" Nextcloud / ownCloud"),
            ]));
            lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled("•", Style::default().fg(Color::Green)),
                Span::raw(" 其他标准 WebDAV 服务"),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .alignment(Alignment::Left);
    f.render_widget(paragraph, area);
}

/// 获取真实的文件路径（考虑环境变量覆盖）
fn get_real_path(env_var: &str, default_path: PathBuf) -> (String, bool) {
    if let Ok(custom_path) = std::env::var(env_var) {
        (custom_path, true)
    } else {
        (default_path.display().to_string(), false)
    }
}

/// 渲染系统信息 Tab
fn render_system_tab(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 💻 System Information ")
        .title_alignment(Alignment::Left);

    // 获取系统信息
    let hostname = whoami::devicename();
    let username = whoami::username();
    let os = whoami::platform().to_string();

    // 获取 CCR 版本
    let ccr_version = env!("CARGO_PKG_VERSION");

    // 获取主目录
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));

    // 获取真实路径（考虑环境变量）
    let (config_path, config_is_custom) =
        get_real_path("CCR_CONFIG_PATH", home.join(".ccs_config.toml"));
    let (settings_path, settings_is_custom) = get_real_path(
        "CCR_SETTINGS_PATH",
        home.join(".claude").join("settings.json"),
    );
    let (backup_dir, backup_is_custom) =
        get_real_path("CCR_BACKUP_DIR", home.join(".claude").join("backups"));
    let (history_path, history_is_custom) = get_real_path(
        "CCR_HISTORY_PATH",
        home.join(".claude").join("ccr_history.json"),
    );
    let (lock_dir, lock_is_custom) =
        get_real_path("CCR_LOCK_DIR", home.join(".claude").join(".locks"));

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

    // 辅助函数：创建路径显示行
    let make_path_line = |label: &str, path: String, is_custom: bool| {
        let mut spans = vec![
            Span::raw("  "),
            Span::styled(format!("{:<12}", label), Style::default().fg(Color::Cyan)),
        ];

        if is_custom {
            spans.push(Span::styled("🔧 ", Style::default().fg(Color::Yellow)));
        } else {
            spans.push(Span::raw("   "));
        }

        spans.push(Span::styled(path, Style::default().fg(Color::White)));

        Line::from(spans)
    };

    // 构建显示内容
    let mut text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "📊 System Information",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Hostname    : "),
            Span::styled(hostname, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("  User        : "),
            Span::styled(username, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("  OS          : "),
            Span::styled(os, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "⚙️  CCR Information",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Version     : "),
            Span::styled(ccr_version, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::raw("  Config      : "),
            Span::styled(current_config, Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![Span::raw("  Auto-Confirm: "), auto_confirm_status]),
        Line::from(""),
        Line::from(Span::styled(
            "📁 File Paths",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // 添加路径信息
    text.push(make_path_line("Config:", config_path, config_is_custom));
    text.push(make_path_line(
        "Settings:",
        settings_path,
        settings_is_custom,
    ));
    text.push(make_path_line("Backup:", backup_dir, backup_is_custom));
    text.push(make_path_line("History:", history_path, history_is_custom));
    text.push(make_path_line("Lock:", lock_dir, lock_is_custom));

    // 如果有自定义路径，添加说明
    let has_custom = config_is_custom
        || settings_is_custom
        || backup_is_custom
        || history_is_custom
        || lock_is_custom;
    if has_custom {
        text.push(Line::from(""));
        text.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("🔧", Style::default().fg(Color::Yellow)),
            Span::styled(
                " = Using custom path from environment variable",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

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
