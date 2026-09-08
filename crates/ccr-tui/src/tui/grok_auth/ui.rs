//! Responsive account list, local-session evidence and typed confirmations.
use super::app::{GrokAuthApp, Modal};
use crate::tui::{CompletedAction, pagination, theme};
use ccr_cli::platforms::grok::GrokActivationState;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthChar;

fn panel(title: &'static str) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(theme::background_style())
        .border_style(Style::default().fg(theme::border()))
}

fn truncate(value: &str, width: usize) -> String {
    if unicode_width::UnicodeWidthStr::width(value) <= width {
        return value.to_owned();
    }
    if width == 0 {
        return String::new();
    }
    let mut used = 0;
    let mut result = String::new();
    for c in value.chars() {
        let next = c.width().unwrap_or(0);
        if used + next > width.saturating_sub(1) {
            break;
        }
        result.push(c);
        used += next;
    }
    result.push('…');
    result
}

pub fn draw(f: &mut Frame, app: &mut GrokAuthApp) {
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(4)])
        .split(f.area());
    draw_embedded(f, app, areas[0], areas[1], theme::ViewportMode::Compact);
}

pub fn draw_embedded(
    f: &mut Frame,
    app: &mut GrokAuthApp,
    content: Rect,
    footer: Rect,
    _mode: theme::ViewportMode,
) {
    let wide = content.width >= 120 && content.height >= 16;
    let standard = content.width >= 90 && content.height >= 16;
    let areas = Layout::default()
        .direction(if wide {
            Direction::Horizontal
        } else {
            Direction::Vertical
        })
        .constraints(if wide {
            vec![Constraint::Percentage(55), Constraint::Percentage(45)]
        } else if standard {
            vec![Constraint::Percentage(45), Constraint::Percentage(55)]
        } else {
            vec![
                Constraint::Min(3),
                Constraint::Length(if content.height >= 12 { 7 } else { 2 }),
            ]
        })
        .split(content);
    draw_accounts(f, app, areas[0]);
    draw_details(f, app, areas[1]);
    draw_footer(f, footer, app);
    if app.modal.is_some() {
        draw_modal(f, app, content);
    }
}

fn draw_accounts(f: &mut Frame, app: &mut GrokAuthApp, area: Rect) {
    app.resize_list(area);
    let rows = pagination::page_slice(&app.snapshot.accounts, app.page(), app.page_size);
    let items: Vec<ListItem<'_>> = rows
        .iter()
        .enumerate()
        .map(|(row, account)| {
            let selected = row + app.page() * app.page_size == app.selected;
            let state = if account.expired {
                crate::tui_text!("expired locally", "本地过期")
            } else if account.expires_at.is_some() {
                crate::tui_text!("saved", "已保存")
            } else {
                crate::tui_text!("expiry unknown", "到期未知")
            };
            let matched = if account.local_match {
                crate::tui_text!("local match", "本地匹配")
            } else {
                ""
            };
            let suffix = format!(" {matched} {state}");
            let name_width = usize::from(area.width.saturating_sub(5))
                .saturating_sub(unicode_width::UnicodeWidthStr::width(suffix.as_str()));
            let text = format!(
                "{} {}{}",
                if selected { "›" } else { " " },
                truncate(&account.name, name_width.max(4)),
                suffix
            );
            ListItem::new(truncate(&text, usize::from(area.width.saturating_sub(2)))).style(
                if selected {
                    theme::info_style()
                } else {
                    theme::muted_style()
                },
            )
        })
        .collect();
    let title = crate::tui_text!(
        " Accounts (local match ≠ selected) ",
        " 账号（本地匹配≠选中） "
    );
    if items.is_empty() {
        f.render_widget(
            Paragraph::new(crate::tui_text!("No saved accounts", "暂无保存账号"))
                .block(panel(title))
                .wrap(Wrap { trim: true }),
            area,
        );
    } else {
        f.render_widget(List::new(items).block(panel(title)), area);
    }
}

fn action_result(app: &GrokAuthApp) -> String {
    if app.is_busy() {
        return crate::tui_text!("Working — wait before leaving", "处理中，请等待完成后离开")
            .into();
    }
    if let Some(notice) = app.notice {
        return match notice {
            "executor" => crate::tui_text!(
                "Background executor unavailable; nothing submitted",
                "后台执行器不可用，未提交操作"
            ),
            "unknown" => crate::tui_text!(
                "Task disconnected: outcome unknown; state reread, no retry",
                "任务中断：结果未知；重读状态，不重试操作"
            ),
            "login" => crate::tui_text!(
                "No supported OAuth source. Sign in with official Grok first",
                "无支持的 OAuth 来源，请先通过官方 Grok 登录"
            ),
            _ => "",
        }
        .into();
    }
    if let Some((action, name, success, error)) = &app.last_action {
        if !success {
            return crate::tui_format!(
                "Operation failed; refresh before retry: {}",
                "操作失败，重试前请刷新：{}",
                error.as_deref().unwrap_or("")
            );
        }
        return match action {
            CompletedAction::Save => crate::tui_format!(
                "Saved {} to CCR; current account can continue",
                "已保存 {} 到 CCR；当前账号可继续使用",
                name
            ),
            CompletedAction::Switch => crate::tui_format!(
                "Local credentials written: {}; new-session authentication unverified",
                "本地凭据已写入：{}；新会话认证未验证",
                name
            ),
            CompletedAction::Delete => crate::tui_format!(
                "Deleted saved item {}; runtime unchanged",
                "已删除保存项 {}；运行时不变",
                name
            ),
            CompletedAction::GrokLogout => crate::tui_text!(
                "Logout completed; CCR saved accounts retained",
                "登出操作完成；保留 CCR 保存账号"
            )
            .into(),
            _ => String::new(),
        };
    }
    crate::tui_text!("Actual authentication unverified", "实际认证未验证").into()
}

fn draw_details(f: &mut Frame, app: &GrokAuthApp, area: Rect) {
    if area.height <= 3 {
        let result = if app.stale {
            if app.last_action.as_ref().is_some_and(|action| action.2) {
                crate::tui_text!("Action succeeded; refresh failed", "操作成功；刷新失败")
            } else if app.last_action.is_some() {
                crate::tui_text!("Action failed; refresh failed", "操作失败；刷新失败")
            } else {
                crate::tui_text!("Refresh failed; stale state", "刷新失败；显示旧状态")
            }
            .to_owned()
        } else {
            action_result(app)
        };
        let status = if app.stale {
            crate::tui_text!("Stale; refresh before retry", "旧状态；重试前请刷新")
        } else if app.snapshot.runtime_error.is_some() {
            crate::tui_text!("Runtime unreadable", "运行时不可读")
        } else if !app.warnings.is_empty() {
            crate::tui_text!(
                "Operation warning; see wide view",
                "操作有警告；宽屏查看详情"
            )
        } else {
            crate::tui_text!("Authentication unverified", "实际认证未验证")
        };
        f.render_widget(
            Paragraph::new(vec![
                Line::from(truncate(&result, usize::from(area.width))),
                Line::styled(status, theme::warning_style()),
            ]),
            area,
        );
        return;
    }
    let result = action_result(app);
    let mut lines = vec![Line::from(if area.height < 14 {
        truncate(&result, usize::from(area.width.saturating_sub(2)))
    } else {
        result
    })];
    if let Some(toast) = app.toasts.active() {
        lines.push(Line::from(toast.message.clone()));
    }
    if app.stale {
        lines.push(Line::styled(
            crate::tui_format!(
                "Refresh failed; displayed state is stale: {}",
                "刷新失败，显示旧状态：{}",
                app.error.as_deref().unwrap_or("")
            ),
            theme::error_style(),
        ));
    }
    if let Some(error) = &app.snapshot.runtime_error {
        lines.push(Line::styled(
            crate::tui_format!("Runtime unreadable: {}", "运行时不可读：{}", error),
            theme::warning_style(),
        ));
    }
    for warning in &app.warnings {
        lines.push(Line::styled(warning.clone(), theme::warning_style()));
    }
    lines.push(Line::from(crate::tui_text!(
        "Actual authentication unverified",
        "实际认证未验证"
    )));
    let route = match &app.activation {
        Some(GrokActivationState::Inactive) => {
            crate::tui_text!("No CCR profile activation recorded", "未记录 CCR 配置启用")
        }
        Some(GrokActivationState::Active { .. }) => crate::tui_text!(
            "Profile route active; may take priority",
            "配置路线已启用，可能优先"
        ),
        Some(_) => crate::tui_text!(
            "Profile activation drifted / unsafe",
            "配置启用状态漂移或不安全"
        ),
        None => crate::tui_text!("Profile activation unknown", "配置启用状态未知"),
    };
    lines.push(Line::from(route));
    if let Some(account) = app.snapshot.accounts.get(app.selected) {
        lines.push(Line::from(crate::tui_format!(
            "Selected: {}",
            "选中：{}",
            account.name
        )));
        lines.push(Line::from(format!(
            "{} / {}",
            account.email.as_deref().unwrap_or("?"),
            account.team.as_deref().unwrap_or("?")
        )));
        lines.push(Line::from(crate::tui_format!(
            "Saved: {}",
            "保存时间：{}",
            account.saved_at
        )));
        lines.push(Line::from(crate::tui_format!(
            "Expiry: {}",
            "到期：{}",
            account.expires_at.as_deref().unwrap_or("?")
        )));
        lines.push(Line::from(crate::tui_format!(
            "Scope: {}",
            "范围：{}",
            account.scope
        )));
    }
    for source in &app.snapshot.sources {
        lines.push(Line::from(crate::tui_format!(
            "Local scope match: {} → {}",
            "本地范围匹配：{} → {}",
            source.scope,
            source
                .matched_account
                .as_deref()
                .unwrap_or(crate::tui_text!("unsaved / unknown", "未保存 / 未知"))
        )));
    }
    lines.push(Line::from(crate::tui_text!(
        "Auth actions preserve profile routes and MCP",
        "账号操作保留配置路线与 MCP"
    )));
    // Very short screens keep result/error visible without spending two rows on borders.
    let paragraph = Paragraph::new(lines)
        .style(theme::muted_style())
        .wrap(Wrap { trim: true });
    f.render_widget(
        if area.height > 3 {
            paragraph.block(panel(crate::tui_text!(
                " Selected / session / result ",
                " 选中 / 会话 / 结果 "
            )))
        } else {
            paragraph
        },
        area,
    );
}

fn draw_footer(f: &mut Frame, area: Rect, app: &GrokAuthApp) {
    let text = if app.is_busy() {
        crate::tui_text!(
            "Busy · Ctrl+L language · wait to leave",
            "处理中 · Ctrl+L 语言 · 完成后离开"
        )
    } else {
        match &app.modal {
            Some(Modal::Confirm(_)) => crate::tui_text!(
                "y confirm · Enter/n/Esc cancel · Ctrl+L language",
                "y 确认 · Enter/n/Esc 取消 · Ctrl+L 语言"
            ),
            Some(Modal::Name { .. }) => crate::tui_text!(
                "Type alias · Enter save · Esc cancel · Ctrl+L language",
                "输入别名 · Enter 保存 · Esc 取消 · Ctrl+L 语言"
            ),
            Some(Modal::Source { .. }) => crate::tui_text!(
                "↑↓ source · Enter select · Esc cancel · Ctrl+L language",
                "↑↓ 来源 · Enter 选择 · Esc 取消 · Ctrl+L 语言"
            ),
            None if area.width < 60 => crate::tui_text!(
                "q quit · s save · Enter switch\nd delete · o logout · r reload",
                "q 退出 · s 保存 · Enter 切换\nd 删除 · o 登出 · r 刷新"
            ),
            None if area.width < 90 => crate::tui_text!(
                "q quit · s save · Enter switch · d delete · o logout · r reload\n↑↓/jk select · PgUp/PgDn page · Tab tabs · Ctrl+L language",
                "q 退出 · s 保存 · Enter 切换 · d 删除 · o 登出 · r 刷新\n↑↓/jk 选择 · PgUp/PgDn 翻页 · Tab 换页 · Ctrl+L 语言"
            ),
            None => crate::tui_text!(
                "↑↓/jk select · PgUp/PgDn page · Enter switch · s save · d delete · o logout · r reload · Tab page · Ctrl+L language · q quit",
                "↑↓/jk 选择 · PgUp/PgDn 翻页 · Enter 切换 · s 保存 · d 删除 · o 登出 · r 刷新 · Tab 换页 · Ctrl+L 语言 · q 退出"
            ),
        }
    };
    let paragraph = Paragraph::new(text)
        .style(theme::muted_style())
        .wrap(Wrap { trim: true });
    f.render_widget(
        if area.height > 2 {
            paragraph.block(
                Block::default()
                    .borders(Borders::TOP)
                    .title(crate::tui_text!(" Keys ", " 按键 ")),
            )
        } else {
            paragraph
        },
        area,
    );
}

fn draw_modal(f: &mut Frame, app: &GrokAuthApp, area: Rect) {
    let Some(modal) = &app.modal else {
        return;
    };
    let width = area.width.min(78);
    let popup = Rect::new(
        area.x + (area.width - width) / 2,
        area.y,
        width,
        area.height.min(16),
    );
    let (title, lines) = match modal {
        Modal::Source { selected, .. } => {
            let page_size = usize::from(popup.height.saturating_sub(3)).max(1);
            let start = selected / page_size * page_size;
            let mut lines = vec![Line::from(crate::tui_text!(
                "Copy to CCR; account can continue",
                "保存副本，当前账号继续使用"
            ))];
            lines.extend(
                app.snapshot
                    .sources
                    .iter()
                    .enumerate()
                    .skip(start)
                    .take(page_size)
                    .map(|(i, s)| {
                        Line::from(truncate(
                            &format!(
                                "{} {} {}",
                                if i == *selected { "›" } else { " " },
                                s.email.as_deref().unwrap_or("?"),
                                s.scope
                            ),
                            usize::from(popup.width.saturating_sub(2)),
                        ))
                    }),
            );
            (crate::tui_text!(" Save source ", " 保存来源 "), lines)
        }
        Modal::Name { value, .. } => (
            crate::tui_text!(" Save account copy ", " 保存账号副本 "),
            vec![
                Line::from(crate::tui_text!(
                    "Copy to CCR; account can continue",
                    "复制到 CCR；当前账号可继续使用"
                )),
                Line::from(crate::tui_text!(
                    "1–32: A-Z a-z 0-9 _ -; no default",
                    "1–32 位字母/数字/_/-；禁用 default"
                )),
                Line::from(format!("{}▏", value)),
                Line::from("<CCR_ROOT>/platforms/grok/auth/accounts.json"),
            ],
        ),
        Modal::Confirm(r) => {
            let (title, description) = match r.action {
                CompletedAction::Save => (
                    crate::tui_text!(" Overwrite saved copy? ", " 覆盖保存副本？ "),
                    crate::tui_text!(
                        "Replace this CCR copy; current account can continue",
                        "覆盖此 CCR 副本；当前账号可继续使用"
                    ),
                ),
                CompletedAction::Switch => (
                    crate::tui_text!(" Switch local session? ", " 切换本地会话？ "),
                    crate::tui_text!(
                        "Stop Grok first. Write credentials for NEW sessions; authentication unverified. Expired copies may need official reauthentication. Profile route stays unchanged.",
                        "请先结束 Grok。写入凭据供新会话使用，认证未验证；过期副本可能需官方重新认证。配置路线保持不变。"
                    ),
                ),
                CompletedAction::Delete => (
                    crate::tui_text!(" Delete saved account? ", " 删除保存账号？ "),
                    crate::tui_text!(
                        "Delete only the CCR saved item. Runtime remains unchanged",
                        "仅删除 CCR 保存项，运行时保持不变"
                    ),
                ),
                _ => (
                    crate::tui_text!(
                        " Log out ALL runtime credentials? ",
                        " 登出全部运行时凭据？ "
                    ),
                    crate::tui_text!(
                        "Remove the entire auth.json, including other scopes/API-key cache and unsaved credentials. All CCR saved accounts remain.",
                        "删除整个 auth.json，包括其他范围/API-key 缓存及未保存凭据。保留全部 CCR 保存账号。"
                    ),
                ),
            };
            let lines = if popup.height <= 8 {
                let mut lines = match r.action {
                    CompletedAction::GrokLogout => vec![
                        Line::from(crate::tui_text!(
                            "Delete ALL auth.json credentials",
                            "删除全部 auth.json 凭据"
                        )),
                        Line::from(crate::tui_text!(
                            "Unsaved + other scopes/API keys too",
                            "含未保存凭据及其他范围/API key"
                        )),
                        Line::from(crate::tui_text!(
                            "Keep all CCR saved accounts",
                            "保留全部 CCR 保存账号"
                        )),
                    ],
                    CompletedAction::Switch => vec![
                        Line::from(crate::tui_text!(
                            "Stop Grok BEFORE confirming",
                            "确认前请先结束 Grok"
                        )),
                        Line::from(crate::tui_text!(
                            "NEW sessions; auth unverified",
                            "仅供新会话；认证未验证"
                        )),
                        Line::from(crate::tui_text!(
                            "Profile route unchanged",
                            "配置认证路线保持不变"
                        )),
                    ],
                    _ => vec![Line::from(description)],
                };
                if !r.name.is_empty() {
                    lines.push(Line::from(truncate(
                        &r.name,
                        usize::from(popup.width.saturating_sub(2)),
                    )));
                }
                lines.push(Line::from(crate::tui_text!(
                    "Default: cancel",
                    "默认：取消"
                )));
                lines
            } else {
                vec![
                    Line::from(r.name.clone()),
                    Line::from(description),
                    Line::from(crate::tui_text!("Default: cancel", "默认：取消")),
                    Line::from(r.scope.clone()),
                ]
            };
            (title, lines)
        }
    };
    f.render_widget(Clear, popup);
    f.render_widget(
        Paragraph::new(lines)
            .block(panel(title))
            .style(theme::info_style())
            .wrap(Wrap { trim: true }),
        popup,
    );
}

pub fn draw_loading_placeholder(
    f: &mut Frame,
    content: Rect,
    footer: Rect,
    _mode: theme::ViewportMode,
    error: Option<&str>,
) {
    let message = error.map_or_else(
        || crate::tui_text!("Loading Grok accounts…", "正在加载 Grok 账号…").to_owned(),
        |e| {
            crate::tui_format!(
                "Unable to load Grok accounts: {}",
                "无法加载 Grok 账号：{}",
                e
            )
        },
    );
    f.render_widget(
        Paragraph::new(message)
            .block(panel(" Grok Auth "))
            .wrap(Wrap { trim: true }),
        content,
    );
    f.render_widget(
        Paragraph::new(crate::tui_text!(
            "Tab page · Ctrl+L language · q quit",
            "Tab 换页 · Ctrl+L 语言 · q 退出"
        )),
        footer,
    );
}
