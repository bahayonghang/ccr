//! Shared Profile workbench. Every value comes from the cached App snapshot.
use super::*;
use ccr_cli::application::profile_activity::ProfileActivityAvailability;
use ccr_cli::models::{ClaudeRuntimeMode, CodexRuntimeMode};
use ccr_cli::platforms::GrokActivationState;
use chrono::{DateTime, Utc};
use ratatui::widgets::{Cell as TableCell, Row, Table};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Column {
    Status,
    Name,
    Description,
    Model,
    LastUsed,
}

/// The same body Rect drives rendering, page capacity and App mouse hit testing.
struct TableGeometry {
    header: Rect,
    rule: Rect,
    body: Rect,
    columns: Vec<(Column, u16)>,
}

impl TableGeometry {
    fn new(inner: Rect) -> Self {
        let width = inner.width;
        let columns = if width >= 88 {
            vec![
                (Column::Status, 3),
                (Column::Name, 24),
                (Column::Description, width - 73),
                (Column::Model, 20),
                (Column::LastUsed, 14),
            ]
        } else if width >= 62 {
            vec![
                (Column::Status, 3),
                (Column::Name, 22),
                (Column::Description, width - 48),
                (Column::LastUsed, 14),
            ]
        } else if width >= 40 {
            vec![
                (Column::Status, 3),
                (Column::Name, width - 23),
                (Column::LastUsed, 14),
            ]
        } else if width >= 7 {
            vec![(Column::Status, 3), (Column::Name, width - 6)]
        } else {
            vec![(Column::Name, width)]
        };
        let header_height = inner.height.min(1);
        let rule_height = inner.height.saturating_sub(1).min(1);
        Self {
            header: Rect::new(inner.x, inner.y, width, header_height),
            rule: Rect::new(inner.x, inner.y + header_height, width, rule_height),
            body: Rect::new(
                inner.x,
                inner.y + header_height + rule_height,
                width,
                inner.height.saturating_sub(2),
            ),
            columns,
        }
    }

    fn widths(&self) -> Vec<Constraint> {
        let mut widths = Vec::new();
        for (index, (_, width)) in self.columns.iter().enumerate() {
            if index > 0 {
                widths.push(Constraint::Length(1));
            }
            widths.push(Constraint::Length(*width));
        }
        widths
    }

    fn cells(
        &self,
        mut value: impl FnMut(Column, usize) -> (String, Style),
    ) -> Vec<TableCell<'static>> {
        let mut cells = Vec::new();
        for (index, (column, width)) in self.columns.iter().enumerate() {
            if index > 0 {
                cells.push(TableCell::from("│").style(Style::default().fg(theme::border())));
            }
            let (text, style) = value(*column, *width as usize);
            cells.push(TableCell::from(truncate_text(&text, *width as usize)).style(style));
        }
        cells
    }
}

fn platform_name(platform: Platform) -> &'static str {
    match platform {
        Platform::Claude => "Claude",
        Platform::Codex => "Codex",
        Platform::Grok => "Grok",
        _ => platform.display_name(),
    }
}

pub(super) fn render_table(f: &mut Frame, app: &mut App, area: Rect) {
    let platform = app.current_platform();
    let block = Block::bordered()
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::border()))
        .title_style(theme::platform_style_for(platform))
        .padding(Padding::horizontal(1));
    let geometry = TableGeometry::new(block.inner(area));
    app.list_area.set(Some(geometry.body));
    app.sync_profile_page_size(super::super::pagination::visible_page_size(
        geometry.body.height,
    ));
    let total = app.current_profiles().len();
    let title = if app.total_pages() > 1 {
        crate::tui_format!(
            " {} Profiles ({}) · {}/{} ",
            " {} 配置 ({}) · {}/{} ",
            platform_name(platform),
            total,
            app.current_page + 1,
            app.total_pages()
        )
    } else {
        crate::tui_format!(
            " {} Profiles ({}) ",
            " {} 配置 ({}) ",
            platform_name(platform),
            total
        )
    };
    let block = block.title(title);
    if total == 0 {
        if app.current_profile_loading() {
            f.render_widget(
                Paragraph::new(crate::tui_text!("Loading profiles…", "正在加载配置…"))
                    .style(theme::muted_style())
                    .block(block),
                area,
            );
        } else {
            render_empty_state(f, app, area, block);
        }
        return;
    }
    let block = block.title_bottom(profile_list_legend_line());
    f.render_widget(block, area);
    let header = geometry.cells(|column, _| {
        (
            match column {
                Column::Status => crate::tui_text!("St", "态"),
                Column::Name => "Profile",
                Column::Description => crate::tui_text!("Description", "描述"),
                Column::Model => crate::tui_text!("Model", "模型"),
                Column::LastUsed => crate::tui_text!("Last used", "上次使用"),
            }
            .to_string(),
            theme::secondary_text_emphasis_style(),
        )
    });
    f.render_widget(
        Table::new(
            [Row::new(header).style(Style::default().bg(theme::surface()))],
            geometry.widths(),
        )
        .column_spacing(1),
        geometry.header,
    );
    f.render_widget(
        Paragraph::new("─".repeat(geometry.rule.width as usize))
            .style(Style::default().fg(theme::border())),
        geometry.rule,
    );
    let now = Utc::now();
    let rows: Vec<Row> = app
        .current_page_profiles()
        .iter()
        .enumerate()
        .map(|(index, profile)| {
            let selected = index == app.selected_index;
            let config = app.current_tab().profile_configs.get(&profile.name);
            let disabled = config.is_some_and(|config| !config.is_enabled());
            let pending = app.profile_mutation_pending(platform, Some(&profile.name));
            let style = if selected {
                theme::selected_row_style()
            } else if profile.is_current {
                theme::success_style()
            } else {
                theme::primary_text_style()
            };
            let cells = geometry.cells(|column, _| {
                let (value, tone) = match column {
                    Column::Status => (
                        format!(
                            "{}{}",
                            if selected { "▶" } else { " " },
                            if pending {
                                "~"
                            } else if disabled {
                                "×"
                            } else if profile.is_current {
                                "●"
                            } else {
                                "○"
                            }
                        ),
                        if disabled || pending {
                            theme::warning_style()
                        } else {
                            style
                        },
                    ),
                    Column::Name => (profile.name.clone(), style.add_modifier(Modifier::BOLD)),
                    Column::Description => (
                        profile.description.clone().unwrap_or_default(),
                        theme::secondary_text_style(),
                    ),
                    Column::Model => (
                        config
                            .and_then(|c| c.model.clone())
                            .unwrap_or_else(|| "—".into()),
                        theme::secondary_text_style(),
                    ),
                    Column::LastUsed => last_used(app, &profile.name, now),
                };
                (
                    value,
                    if selected {
                        theme::selected_row_style()
                    } else {
                        tone
                    },
                )
            });
            Row::new(cells).style(if selected {
                theme::selected_row_style()
            } else {
                theme::primary_text_style()
            })
        })
        .collect();
    f.render_widget(
        Table::new(rows, geometry.widths()).column_spacing(1),
        geometry.body,
    );
}

fn relative_time(timestamp: DateTime<Utc>, now: DateTime<Utc>) -> (String, bool) {
    let seconds = now.signed_duration_since(timestamp).num_seconds();
    if seconds < 0 {
        return (format!("!{}", timestamp.format("%m-%d %H:%M")), true);
    }
    let value = if seconds < 60 {
        crate::tui_text!("now", "刚刚").to_string()
    } else if seconds < 3600 {
        crate::tui_format!("{}m ago", "{}分钟前", seconds / 60)
    } else if seconds < 86400 {
        crate::tui_format!("{}h ago", "{}小时前", seconds / 3600)
    } else if seconds < 604800 {
        crate::tui_format!("{}d ago", "{}天前", seconds / 86400)
    } else {
        timestamp.format("%m-%d %H:%M").to_string()
    };
    (value, false)
}

fn last_used(app: &App, name: &str, now: DateTime<Utc>) -> (String, Style) {
    if let Some(activity) = app.profile_activity(name) {
        if let Some(timestamp) = activity.recent_activations.first() {
            let (mut text, future) = relative_time(*timestamp, now);
            if activity.invalid_timestamps > 0 && !future {
                text.insert(0, '!');
            }
            return (
                text,
                if future || activity.invalid_timestamps > 0 {
                    theme::warning_style()
                } else {
                    theme::secondary_text_style()
                },
            );
        }
        if activity.invalid_timestamps > 0 {
            return (
                crate::tui_text!("Invalid time", "时间无效").into(),
                theme::warning_style(),
            );
        }
    }
    let (label, style) = match app.current_activity_snapshot() {
        None => (
            crate::tui_text!("Loading…", "加载中…"),
            theme::muted_style(),
        ),
        Some(Err(_)) => (
            crate::tui_text!("Unavailable", "不可用"),
            theme::error_style(),
        ),
        Some(Ok(snapshot)) if snapshot.availability != ProfileActivityAvailability::Ready => (
            crate::tui_text!("Unavailable", "不可用"),
            theme::muted_style(),
        ),
        Some(Ok(_)) => (
            crate::tui_text!("No record", "无记录"),
            theme::muted_style(),
        ),
    };
    (label.into(), style)
}

pub(super) fn render_runtime(f: &mut Frame, app: &App, area: Rect) {
    let platform = app.current_platform();
    let (mode, mode_style, name, auth) = runtime_values(app);
    let model = name
        .as_deref()
        .and_then(|name| app.current_tab().profile_configs.get(name))
        .and_then(|config| config.model.as_deref());
    let profile = match (name.as_deref(), model) {
        (Some(name), Some(model)) => format!("{name} / {model}"),
        (Some(name), _) => name.to_string(),
        _ => crate::tui_text!("Not bound", "未绑定").into(),
    };
    let block = Block::bordered()
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::border()))
        .title(crate::tui_text!(" Current runtime ", " 当前生效 "))
        .title_style(theme::platform_style_for(platform))
        .padding(Padding::horizontal(1));
    let inner = block.inner(area);
    f.render_widget(block, area);
    if inner.height < 2 || inner.width < 86 {
        let mode_width = mode.width().min(inner.width as usize);
        let remainder = inner.width as usize - mode_width;
        let auth_width = (remainder.saturating_sub(6) / 2).min(auth.width());
        let profile_width = remainder.saturating_sub(6 + auth_width);
        let line = Line::from(vec![
            Span::styled(truncate_text(&mode, mode_width), mode_style),
            Span::styled(" │ ", theme::muted_style()),
            Span::styled(
                truncate_text(&profile, profile_width),
                theme::platform_style_for(platform),
            ),
            Span::styled(" │ ", theme::muted_style()),
            Span::styled(
                truncate_text(&auth, auth_width),
                theme::secondary_text_style(),
            ),
        ]);
        f.render_widget(Paragraph::new(line), inner);
    } else {
        let columns = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(40),
            Constraint::Percentage(35),
        ])
        .split(inner);
        for (column, label, value, style) in [
            (
                columns[0],
                crate::tui_text!("Mode", "运行方式"),
                mode,
                mode_style,
            ),
            (
                columns[1],
                crate::tui_text!("Current profile / model", "当前配置 / 模型"),
                profile,
                theme::platform_style_for(platform),
            ),
            (
                columns[2],
                crate::tui_text!("Auth", "认证"),
                auth,
                theme::secondary_text_emphasis_style(),
            ),
        ] {
            f.render_widget(
                Paragraph::new(vec![
                    Line::styled(label, theme::muted_style()),
                    Line::styled(
                        truncate_text(&value, column.width.saturating_sub(1) as usize),
                        style,
                    ),
                ]),
                column,
            );
        }
    }
}

fn runtime_values(app: &App) -> (String, Style, Option<String>, String) {
    let unresolved = || {
        (
            crate::tui_text!("Unresolved", "未解析").into(),
            theme::muted_style(),
            None,
            crate::tui_text!("Unknown", "未知").into(),
        )
    };
    if app.current_profile_status_error().is_some() {
        return (
            crate::tui_text!("Runtime unavailable", "运行态不可用").into(),
            theme::error_style(),
            None,
            crate::tui_text!("See details", "查看详情").into(),
        );
    }
    if app.current_profile_loading() {
        return (
            crate::tui_text!("Loading…", "加载中…").into(),
            theme::muted_style(),
            None,
            "—".into(),
        );
    }
    match app.current_platform() {
        Platform::Codex => {
            let Some(summary) = app.current_codex_runtime_summary() else {
                return unresolved();
            };
            let label = match summary.mode {
                CodexRuntimeMode::ProfileOnly => crate::tui_text!("Profile active", "配置生效"),
                CodexRuntimeMode::ProfileWithAuth => {
                    crate::tui_text!("Profile + Auth", "配置 + 认证")
                }
                CodexRuntimeMode::ProfilePendingAuth => {
                    crate::tui_text!("Waiting for auth", "等待认证")
                }
                CodexRuntimeMode::RuntimeOnly => {
                    crate::tui_text!("Runtime/Auth only", "仅运行时认证")
                }
                CodexRuntimeMode::Unresolved => crate::tui_text!("Unresolved", "未解析"),
            };
            (
                label.into(),
                runtime_mode_style(summary.mode),
                summary.current_profile_name.clone(),
                localize_codex_runtime_text(summary.auth_label()),
            )
        }
        Platform::Claude => {
            let Some(summary) = app.current_claude_runtime_summary() else {
                return unresolved();
            };
            let (label, style) = match summary.mode {
                ClaudeRuntimeMode::ProfileOnly => (
                    crate::tui_text!("Profile active", "配置生效"),
                    theme::success_style(),
                ),
                ClaudeRuntimeMode::ProfileWithAuth => (
                    crate::tui_text!("Profile + subscription", "配置 + 官方订阅"),
                    theme::success_style(),
                ),
                ClaudeRuntimeMode::ProfilePendingAuth => (
                    crate::tui_text!("Waiting for subscription", "等待官方订阅"),
                    theme::warning_style(),
                ),
                ClaudeRuntimeMode::RuntimeOnly => (
                    crate::tui_text!("Subscription runtime", "官方订阅运行时"),
                    theme::info_style(),
                ),
                ClaudeRuntimeMode::Unresolved => (
                    crate::tui_text!("Unresolved", "未解析"),
                    theme::muted_style(),
                ),
            };
            let auth = match summary.mode {
                ClaudeRuntimeMode::ProfileOnly => summary
                    .current_profile_auth_source
                    .clone()
                    .unwrap_or_else(|| "API key".into()),
                _ => summary.current_auth_name.clone().unwrap_or_else(|| {
                    crate::tui_text!("Official / not ready", "官方订阅 / 未就绪").into()
                }),
            };
            (
                label.into(),
                style,
                summary.current_profile_name.clone(),
                auth,
            )
        }
        Platform::Grok => {
            let (mode, style, name) = match app.current_grok_activation_state() {
                Some(GrokActivationState::Active { name }) => (
                    crate::tui_text!("Profile active", "配置生效"),
                    theme::success_style(),
                    Some(name.clone()),
                ),
                Some(GrokActivationState::Inactive) => (
                    crate::tui_text!("Runtime only", "仅运行时"),
                    theme::muted_style(),
                    None,
                ),
                Some(GrokActivationState::Drifted { name }) => (
                    crate::tui_text!("Runtime drifted", "运行态已偏离"),
                    theme::warning_style(),
                    Some(name.clone()),
                ),
                Some(GrokActivationState::UnsafeMissingEntryState { name }) => (
                    crate::tui_text!("Entry state missing", "缺少入口快照"),
                    theme::error_style(),
                    name.clone(),
                ),
                None => return unresolved(),
            };
            let auth = name
                .as_deref()
                .and_then(|name| app.current_tab().profile_configs.get(name))
                .map(|config| match GrokPlatform::profile_auth_mode(config) {
                    Ok(GrokProfileAuthMode::InlineApiKey) => {
                        crate::tui_text!("Inline API key", "内联 API 密钥").into()
                    }
                    Ok(GrokProfileAuthMode::EnvKey) => profile_platform_value(config, "env_key")
                        .map(|key| format!("env / {key}"))
                        .unwrap_or_else(|| {
                            crate::tui_text!("Environment key", "环境变量密钥").into()
                        }),
                    Ok(GrokProfileAuthMode::Session) => {
                        crate::tui_text!("Session: unknown", "会话：未知").into()
                    }
                    Err(_) => crate::tui_text!("Invalid auth configuration", "认证配置无效").into(),
                })
                .unwrap_or_else(|| crate::tui_text!("Session: unknown", "会话：未知").into());
            (mode.into(), style, name, auth)
        }
        _ => unresolved(),
    }
}

/// Explicit cell wrapping means scroll extent and rendered lines cannot disagree.
fn wrap_lines(lines: Vec<Line<'static>>, width: usize) -> Vec<Line<'static>> {
    if width == 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for line in lines {
        let mut spans = Vec::new();
        let mut used = 0;
        for span in line.spans {
            let mut text = String::new();
            for ch in span.content.chars() {
                let cells = ch.width().unwrap_or(0);
                if ch == '\n' || used + cells > width {
                    if !text.is_empty() {
                        spans.push(Span::styled(std::mem::take(&mut text), span.style));
                    }
                    out.push(Line::from(std::mem::take(&mut spans)).style(line.style));
                    used = 0;
                    if ch == '\n' {
                        continue;
                    }
                }
                if cells <= width && !ch.is_control() {
                    text.push(ch);
                    used += cells;
                }
            }
            if !text.is_empty() {
                spans.push(Span::styled(text, span.style));
            }
        }
        out.push(Line::from(spans).style(line.style));
    }
    out
}

fn connection_rows(
    platform: Platform,
    config: &ProfileConfig,
    expanded: bool,
    width: usize,
) -> Vec<Line<'static>> {
    let details = match platform {
        Platform::Codex => codex_profile_details("", config, false, None, false, expanded),
        Platform::Claude => claude_profile_details("", config, false, None, false, expanded),
        Platform::Grok => grok_profile_details("", config, false, false, expanded),
        _ => generic_profile_details("", config, false, platform, false, expanded),
    };
    let mut fields: Vec<_> = details
        .fields
        .into_iter()
        .filter(|field| {
            !matches!(
                field.key,
                DetailKey::Description | DetailKey::SwitchCount | DetailKey::Tags
            ) && (expanded || !field.unset)
        })
        .collect();
    fields.sort_by_key(|field| match field.key {
        DetailKey::Model => 0,
        DetailKey::ReasoningEffort => 1,
        DetailKey::Provider => 2,
        DetailKey::BaseUrl => 3,
        DetailKey::AuthMode => 4,
        DetailKey::AuthSource => 5,
        _ => 6,
    });
    let mut rows: Vec<(String, String, Style)> = fields
        .into_iter()
        .map(|field| {
            (
                connection_label(field.key).into(),
                field.value,
                detail_tone_style(field.tone),
            )
        })
        .collect();
    if platform == Platform::Claude {
        for (label, value) in [
            ("Opus", config.default_opus_model.as_deref()),
            ("Sonnet", config.default_sonnet_model.as_deref()),
            ("Haiku", config.default_haiku_model.as_deref()),
            ("Fable", config.default_fable_model.as_deref()),
            (
                crate::tui_text!("Subagent", "子代理"),
                config.subagent_model.as_deref(),
            ),
            (
                crate::tui_text!("Effort", "思考强度"),
                config.effort_level.as_deref(),
            ),
            (
                crate::tui_text!("Timeout ms", "超时毫秒"),
                config.api_timeout_ms.as_deref(),
            ),
            (
                crate::tui_text!("Auto compact", "自动压缩"),
                config.claude_code_auto_compact_window.as_deref(),
            ),
        ] {
            if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
                rows.push((label.into(), value.into(), theme::info_style()));
            } else if expanded {
                rows.push((label.into(), "—".into(), theme::muted_style()));
            }
        }
    }
    if platform == Platform::Grok
        && let Some(provider) = config
            .provider
            .as_deref()
            .filter(|value| !value.trim().is_empty())
    {
        rows.insert(
            1.min(rows.len()),
            (
                crate::tui_text!("Provider", "提供商").into(),
                provider.into(),
                theme::platform_style_for(platform),
            ),
        );
    }
    let label_width = rows
        .iter()
        .map(|(label, _, _)| label.width())
        .max()
        .unwrap_or(0)
        .saturating_add(2)
        .min(width.saturating_sub(4).max(1));
    let mut lines = Vec::new();
    for (label, value, style) in rows {
        let value_lines = wrap_lines(
            vec![Line::from(Span::styled(value, style))],
            width.saturating_sub(label_width).max(1),
        );
        for (index, value) in value_lines.into_iter().enumerate() {
            let mut spans = vec![Span::styled(
                if index == 0 {
                    pad_text(
                        &truncate_text(&label, label_width.saturating_sub(1)),
                        label_width,
                    )
                } else {
                    " ".repeat(label_width)
                },
                theme::secondary_text_style(),
            )];
            spans.extend(value.spans);
            lines.push(Line::from(spans));
        }
    }
    lines
}

fn connection_label(key: DetailKey) -> &'static str {
    match key {
        DetailKey::Model => crate::tui_text!("Model", "模型"),
        DetailKey::ReasoningEffort => crate::tui_text!("Reasoning", "推理强度"),
        DetailKey::BaseUrl => crate::tui_text!("Endpoint", "地址"),
        DetailKey::Provider => crate::tui_text!("Provider", "提供商"),
        DetailKey::AuthMode => crate::tui_text!("Auth", "认证"),
        DetailKey::AuthSource => crate::tui_text!("Source", "来源"),
        DetailKey::Token => crate::tui_text!("Key", "密钥"),
        DetailKey::EnvKey => crate::tui_text!("Environment", "环境变量键"),
        DetailKey::WireApi => crate::tui_text!("Wire API", "协议 API"),
        DetailKey::ApiBackend => crate::tui_text!("Backend", "后端"),
        DetailKey::SmallFast => crate::tui_text!("Fast model", "快速模型"),
        DetailKey::ContextWindow => crate::tui_text!("Context", "上下文"),
        DetailKey::BackendSearch => crate::tui_text!("Search", "搜索"),
        DetailKey::ProviderType => crate::tui_text!("Type", "类型"),
        DetailKey::Account => crate::tui_text!("Account", "账号"),
        DetailKey::RequiresOpenAi => crate::tui_text!("OpenAI auth", "OpenAI 认证"),
        DetailKey::OpenAiLogin => crate::tui_text!("OpenAI login", "OpenAI 登录"),
        _ => key.label(),
    }
}

fn section(lines: &mut Vec<Line<'static>>, title: String, width: usize, platform: Platform) {
    lines.push(Line::styled(
        "─".repeat(width),
        Style::default().fg(theme::border()),
    ));
    lines.extend(wrap_lines(
        vec![Line::styled(title, theme::platform_style_for(platform))],
        width,
    ));
}

fn token_bar(value: i64, max: i64, width: usize) -> String {
    if value <= 0 || max <= 0 {
        return " ".repeat(width);
    }
    let eighths = ((value as u128 * width as u128 * 8 + max as u128 / 2) / max as u128)
        .min(width as u128 * 8) as usize;
    let full = eighths / 8;
    let partial = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'][eighths % 8];
    let mut bar = "█".repeat(full);
    if !eighths.is_multiple_of(8) {
        bar.push(partial);
    }
    pad_text(&bar, width)
}

fn usage_ledger(app: &App, provider: Option<&str>, width: usize) -> Vec<Line<'static>> {
    let platform = app.current_platform();
    let source = match platform {
        Platform::Codex => SourceKind::Codex,
        Platform::Grok => SourceKind::Grok,
        _ => SourceKind::Claude,
    };
    let mut lines = Vec::new();
    section(
        &mut lines,
        crate::tui_text!("Provider usage", "提供商用量").into(),
        width,
        platform,
    );
    let provider = provider.filter(|value| !value.trim().is_empty());
    if let Some(provider) = provider {
        lines.push(Line::styled(
            format!("{provider} / {}", platform_name(platform)),
            theme::secondary_text_style(),
        ));
    }
    lines.push(Line::styled(
        crate::tui_text!(
            "All-time provider total · approx official-equivalent USD",
            "提供商全时段总量 · 费用为官方等价美元估算"
        ),
        theme::muted_style(),
    ));
    let state = app.usage_app.as_ref().map(|engine| &engine.state);
    let row = if let (Some(provider), Some(UsageLoadState::Loaded(dataset))) = (provider, state) {
        dataset
            .platform_rows(source)
            .find(|row| row.breakdown.provider.as_deref() == Some(provider))
            .map(|row| &row.breakdown)
    } else {
        None
    };
    let Some(row) = row else {
        let (message, style) = if provider.is_none() {
            (
                crate::tui_text!(
                    "No provider label; usage unattributed",
                    "缺少提供商标签，无法归因用量"
                )
                .into(),
                theme::muted_style(),
            )
        } else {
            match state {
                Some(UsageLoadState::Unsupported(message)) => {
                    (message.clone(), theme::warning_style())
                }
                Some(UsageLoadState::Error(message)) => (message.clone(), theme::error_style()),
                Some(UsageLoadState::Loaded(_) | UsageLoadState::Empty) => (
                    crate::tui_text!("No usage recorded", "暂无用量记录").into(),
                    theme::muted_style(),
                ),
                _ => (
                    crate::tui_text!("Loading usage…", "正在加载用量…").into(),
                    theme::muted_style(),
                ),
            }
        };
        lines.push(Line::styled(message, style));
        return wrap_lines(lines, width);
    };
    let metrics = [
        (
            crate::tui_text!("Requests", "请求"),
            format_count(row.request_count),
            theme::primary_text_emphasis_style(),
        ),
        (
            crate::tui_text!("Total tokens", "总令牌"),
            format_count(row.total_tokens),
            theme::primary_text_emphasis_style(),
        ),
        (
            crate::tui_text!("Approx cost", "估算费用"),
            format_cost(row.cost_with_cache_usd),
            theme::warning_style().add_modifier(Modifier::BOLD),
        ),
    ];
    if width >= 48 {
        let cell_width = width / 3;
        lines.push(
            Line::from(
                metrics
                    .iter()
                    .map(|(label, _, _)| {
                        Span::styled(pad_text(label, cell_width), theme::secondary_text_style())
                    })
                    .collect::<Vec<_>>(),
            )
            .style(Style::default().bg(theme::surface())),
        );
        lines.push(
            Line::from(
                metrics
                    .iter()
                    .map(|(_, value, style)| Span::styled(pad_text(value, cell_width), *style))
                    .collect::<Vec<_>>(),
            )
            .style(Style::default().bg(theme::surface())),
        );
    } else {
        for (label, value, style) in metrics {
            lines.push(Line::from(vec![
                Span::styled(format!("{label}  "), theme::secondary_text_style()),
                Span::styled(value, style),
            ]));
        }
    }
    let categories = [
        (crate::tui_text!("Input", "输入"), row.input_tokens),
        (
            crate::tui_text!("Cache read", "缓存读取"),
            row.cache_read_tokens,
        ),
        (
            crate::tui_text!("Cache write", "缓存写入"),
            row.cache_creation_tokens,
        ),
        (crate::tui_text!("Output", "输出"), row.output_tokens),
        (
            crate::tui_text!("Reasoning", "推理"),
            row.reasoning_output_tokens,
        ),
    ];
    let max = categories
        .iter()
        .map(|(_, value)| *value)
        .max()
        .unwrap_or(0)
        .max(0);
    lines.push(Line::styled(
        crate::tui_format!(
            "Category scale: {} tokens",
            "分类刻度：{} 令牌",
            format_count(max)
        ),
        theme::muted_style(),
    ));
    let label_width = categories
        .iter()
        .map(|(label, _)| label.width())
        .max()
        .unwrap_or(10)
        + 2;
    let count_width = categories
        .iter()
        .map(|(_, value)| {
            if *value < 0 {
                crate::tui_text!("Invalid", "无效").width()
            } else {
                format_count(*value).width()
            }
        })
        .max()
        .unwrap_or(1)
        .max(5);
    let bar_width = if width >= 58 {
        width.saturating_sub(label_width + count_width + 2)
    } else {
        0
    };
    for (label, value) in categories {
        let count = if value < 0 {
            crate::tui_text!("Invalid", "无效").to_string()
        } else {
            format_count(value)
        };
        let mut spans = vec![Span::styled(
            pad_text(label, label_width),
            theme::secondary_text_style(),
        )];
        if bar_width > 0 {
            spans.push(Span::styled(
                token_bar(value, max, bar_width),
                Style::default()
                    .fg(theme::accent_for(platform))
                    .bg(theme::bg_secondary()),
            ));
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(
            format!(
                "{}{count}",
                " ".repeat(count_width.saturating_sub(count.width()))
            ),
            if value < 0 {
                theme::warning_style()
            } else {
                theme::primary_text_style()
            },
        ));
        lines.push(Line::from(spans));
    }
    wrap_lines(lines, width)
}

fn activity_lines(app: &App, name: &str, switches: u32, width: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    section(
        &mut lines,
        crate::tui_format!(
            "Recorded activations · {} switches",
            "启用记录 · 切换 {} 次",
            switches
        ),
        width,
        app.current_platform(),
    );
    let mut has_records = false;
    if let Some(activity) = app.profile_activity(name) {
        let now = Utc::now();
        for (index, timestamp) in activity.recent_activations.iter().take(3).enumerate() {
            has_records = true;
            lines.push(Line::from(vec![
                Span::styled(
                    pad_text(
                        if index == 0 {
                            crate::tui_text!("Last used", "上次使用")
                        } else {
                            ""
                        },
                        12.min(width),
                    ),
                    theme::secondary_text_style(),
                ),
                Span::styled(
                    timestamp.format("%Y-%m-%d %H:%M +00:00").to_string(),
                    if *timestamp > now {
                        theme::warning_style()
                    } else if index == 0 {
                        theme::primary_text_emphasis_style()
                    } else {
                        theme::secondary_text_style()
                    },
                ),
            ]));
        }
        if activity.invalid_timestamps > 0 {
            lines.push(Line::styled(
                crate::tui_text!(
                    "Partial history: invalid timestamps omitted",
                    "部分历史：已跳过无效时间"
                ),
                theme::warning_style(),
            ));
        }
        if activity
            .recent_activations
            .iter()
            .any(|timestamp| *timestamp > now)
        {
            lines.push(Line::styled(
                crate::tui_text!(
                    "Future timestamp; check the clock",
                    "时间在未来，请检查时钟"
                ),
                theme::warning_style(),
            ));
        }
    }
    if !has_records {
        let (text, style) = match app.current_activity_snapshot() {
            Some(Ok(snapshot))
                if snapshot.availability == ProfileActivityAvailability::MissingDatabase =>
            {
                (
                    crate::tui_text!("History database is not present", "尚无历史数据库").into(),
                    theme::muted_style(),
                )
            }
            Some(Ok(snapshot))
                if snapshot.availability == ProfileActivityAvailability::MissingSchema =>
            {
                (
                    crate::tui_text!("History schema unavailable", "历史表结构不可用").into(),
                    theme::warning_style(),
                )
            }
            Some(Err(_)) => (
                crate::tui_text!(
                    "History read failed; reload to retry",
                    "历史读取失败，可重新加载"
                )
                .into(),
                theme::error_style(),
            ),
            _ => last_used(app, name, Utc::now()),
        };
        lines.push(Line::styled(text, style));
    }
    lines.push(Line::styled(
        crate::tui_text!(
            "Last used = successful CCR activation",
            "上次使用按 CCR 成功启用记录统计"
        ),
        theme::muted_style(),
    ));
    wrap_lines(lines, width)
}

fn detail_lines(app: &App, width: usize) -> Vec<Line<'static>> {
    let (Some(profile), Some(config)) = (app.selected_profile(), app.selected_profile_config())
    else {
        return wrap_lines(profile_status_text(app), width);
    };
    let platform = app.current_platform();
    let mut lines = vec![Line::from(vec![
        Span::styled(
            format!("{}  ", profile.name),
            theme::platform_style_for(platform),
        ),
        Span::styled(
            if profile.is_current {
                crate::tui_text!("Current", "当前生效")
            } else {
                crate::tui_text!("Available", "可用")
            },
            if profile.is_current {
                theme::success_style()
            } else {
                theme::secondary_text_style()
            },
        ),
        Span::styled(
            if config.is_enabled() {
                crate::tui_text!(" / Enabled", " / 已启用")
            } else {
                crate::tui_text!(" / Disabled", " / 已禁用")
            },
            if config.is_enabled() {
                theme::success_style()
            } else {
                theme::warning_style()
            },
        ),
    ])];
    if let Some(description) = config
        .description
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        lines.push(Line::styled(
            description.to_string(),
            theme::secondary_text_style(),
        ));
    }
    let tags = tags_field(config);
    if !tags.unset {
        lines.push(Line::styled(tags.value, theme::muted_style()));
    }
    if let Some(text) = last_apply_message(
        &profile.name,
        app.last_applied
            .as_ref()
            .filter(|entry| entry.0 == format!("{} Profile", platform_name(platform))),
    ) {
        let style = if app
            .last_applied
            .as_ref()
            .is_some_and(|entry| entry.2 && entry.3.is_some())
        {
            theme::warning_style()
        } else if app.last_applied.as_ref().is_some_and(|entry| entry.2) {
            theme::success_style()
        } else {
            detail_tone_style(DetailTone::Error)
        };
        lines.push(Line::styled(text, style));
    }
    if app.profile_mutation_pending(platform, Some(&profile.name)) {
        lines.push(Line::styled(
            crate::tui_text!("Applying…", "正在应用…"),
            theme::warning_style(),
        ));
    }
    if app.current_profile_loading() {
        lines.push(Line::styled(
            crate::tui_text!(
                "Refreshing; showing previous snapshot",
                "正在刷新，显示上次快照"
            ),
            theme::muted_style(),
        ));
    }
    if let Some(error) = app.current_profile_load_error() {
        lines.push(Line::styled(error.to_string(), theme::error_style()));
    }
    if let Some(error) = app
        .current_profile_status_error()
        .filter(|error| Some(*error) != app.current_profile_load_error())
    {
        lines.push(Line::styled(error.to_string(), theme::error_style()));
    }
    lines.extend(connection_rows(
        platform,
        config,
        app.profile_details_expanded,
        width,
    ));
    lines.extend(usage_ledger(app, config.provider.as_deref(), width));
    lines.extend(activity_lines(
        app,
        &profile.name,
        config.usage_count(),
        width,
    ));
    if app.profile_details_expanded {
        let (mode, style, name, auth) = runtime_values(app);
        section(
            &mut lines,
            crate::tui_text!("Current runtime", "当前生效").into(),
            width,
            platform,
        );
        lines.push(Line::styled(mode, style));
        if let Some(name) = name {
            let model = app
                .current_tab()
                .profile_configs
                .get(&name)
                .and_then(|config| config.model.as_deref());
            lines.push(Line::styled(
                model.map_or_else(|| name.clone(), |model| format!("{name} / {model}")),
                theme::platform_style_for(platform),
            ));
        }
        lines.push(Line::styled(auth, theme::secondary_text_style()));
    }
    wrap_lines(lines, width)
}

pub(super) fn render_details(f: &mut Frame, app: &mut App, area: Rect) {
    let platform = app.current_platform();
    let block = Block::bordered()
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::border()))
        .title(crate::tui_text!(" Selected profile ", " 所选配置 "))
        .title_style(theme::platform_style_for(platform))
        .padding(Padding::horizontal(1));
    let width = block.inner(area).width as usize;
    let lines = detail_lines(app, width);
    let height = area
        .height
        .min(u16::try_from(lines.len().saturating_add(2)).unwrap_or(u16::MAX));
    let area = Rect::new(area.x, area.y, area.width, height);
    let inner = block.inner(area);
    app.detail_area.set(Some(inner));
    let max_scroll = lines.len().saturating_sub(inner.height as usize);
    let scroll = usize::from(app.profile_detail_scroll).min(max_scroll);
    app.profile_detail_scroll = u16::try_from(scroll).unwrap_or(u16::MAX);
    let block = if max_scroll > 0 {
        block.title_bottom(
            Line::styled(
                crate::tui_format!(
                    " {}–{} / {} lines ",
                    " {}–{} / {} 行 ",
                    scroll + 1,
                    (scroll + inner.height as usize).min(lines.len()),
                    lines.len()
                ),
                theme::muted_style(),
            )
            .alignment(Alignment::Right),
        )
    } else {
        block
    };
    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .scroll((app.profile_detail_scroll, 0)),
        area,
    );
    if max_scroll > 0 {
        let mut state = ScrollbarState::new(max_scroll).position(scroll);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            area,
            &mut state,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::ProfileItem;
    use crate::tui::runtime::AsyncTaskExecutor;
    use crate::tui::usage::app::{UsageApp, UsageDataset};
    use ccr_cli::application::profile_activity::{ProfileActivity, ProfileActivitySnapshot};
    use ccr_cli::managers::TuiLanguage;
    use ratatui::{Terminal, backend::TestBackend};
    use std::{collections::HashMap, sync::Arc};

    fn fixture(platform: Platform) -> App {
        let mut config = ProfileConfig::new();
        config.model = Some(
            match platform {
                Platform::Claude => "claude-example",
                Platform::Grok => "grok-example",
                _ => "gpt-example",
            }
            .into(),
        );
        config.description = Some(crate::tui_text!("Daily development", "日常开发配置").into());
        config.provider = Some("example-provider".into());
        config.base_url = Some("https://api.example.invalid/v1".into());
        config.usage_count = Some(23);
        config.tags = Some(vec!["work".into(), "daily".into()]);
        config.auth_token = Some(ccr_core::Secret::new("fixture-secret-never-render-this"));
        match platform {
            Platform::Codex => {
                config
                    .platform_data
                    .insert("model_reasoning_effort".into(), serde_json::json!("high"));
            }
            Platform::Claude => {
                config.default_sonnet_model = Some("sonnet-example".into());
            }
            Platform::Grok => {
                config.auth_token = None;
                config
                    .platform_data
                    .insert("env_key".into(), serde_json::json!("EXAMPLE_API_KEY"));
                config
                    .platform_data
                    .insert("reasoning_effort".into(), serde_json::json!("high"));
            }
            _ => {}
        }
        let profile = ProfileItem {
            name: "production".into(),
            description: Some("Main route".into()),
            is_current: true,
        };
        let mut app =
            super::super::tests::sample_profile_app_for(platform, profile, config.clone());
        for index in 1..25 {
            let name = if index == 1 {
                "work".into()
            } else {
                format!("profile-{index:02}")
            };
            app.tabs[0].profiles.push(ProfileItem {
                name: name.clone(),
                description: config.description.clone(),
                is_current: false,
            });
            app.tabs[0].profile_configs.insert(name, config.clone());
        }
        app.selected_index = 1;
        app.selected_profile_name = Some("work".into());
        let timestamp: DateTime<Utc> = "2026-10-12T17:42:00Z"
            .parse()
            .expect("valid fixed fixture timestamp");
        app.tabs[0].activity = Some(Ok(ProfileActivitySnapshot {
            availability: ProfileActivityAvailability::Ready,
            profiles: HashMap::from([
                (
                    "production".into(),
                    ProfileActivity {
                        recent_activations: vec![timestamp],
                        invalid_timestamps: 0,
                    },
                ),
                (
                    "work".into(),
                    ProfileActivity {
                        recent_activations: vec![
                            timestamp - chrono::Duration::hours(2),
                            timestamp - chrono::Duration::days(1),
                            timestamp - chrono::Duration::days(2),
                        ],
                        invalid_timestamps: 0,
                    },
                ),
            ]),
        }));
        match platform {
            Platform::Codex => {
                app.tabs[0].codex_runtime_summary =
                    Some(super::super::tests::codex_runtime_summary_fixture(
                        CodexRuntimeMode::ProfileOnly,
                        Some("production"),
                    ))
            }
            Platform::Grok => {
                app.tabs[0].grok_activation_state = Some(GrokActivationState::Active {
                    name: "production".into(),
                })
            }
            Platform::Claude => {
                app.tabs[0].claude_runtime_summary = Some(ccr_cli::models::ClaudeRuntimeSummary {
                    mode: ClaudeRuntimeMode::ProfileOnly,
                    current_profile_name: Some("production".into()),
                    current_profile_provider: Some("example-provider".into()),
                    current_profile_auth_mode: Some(ccr_cli::models::ClaudeProfileAuthMode::ApiKey),
                    current_profile_auth_source: Some("settings:anthropic_env".into()),
                    current_login_name: None,
                    official_login_state: ccr_cli::models::ClaudeLoginState::NotLoggedIn,
                    current_auth_name: None,
                    login_state: ccr_cli::models::ClaudeLoginState::ApiKeyActive,
                    auth_diagnosis: Default::default(),
                })
            }
            _ => {}
        }
        let source = match platform {
            Platform::Claude => SourceKind::Claude,
            Platform::Grok => SourceKind::Grok,
            _ => SourceKind::Codex,
        };
        let mut row = super::super::tests::usage_row(source, Some("example-provider"), 8600);
        row.breakdown.input_tokens = 59_400_000;
        row.breakdown.cache_read_tokens = 44_900_000;
        row.breakdown.cache_creation_tokens = 1_000_000;
        row.breakdown.output_tokens = 4_200_000;
        row.breakdown.reasoning_output_tokens = 300_000;
        row.breakdown.total_tokens = 109_800_000;
        row.breakdown.cost_with_cache_usd = 32.60;
        let mut usage = UsageApp::with_loader(
            AsyncTaskExecutor::Disabled,
            Arc::new(|| panic!("render performed I/O")),
        );
        usage.state = UsageLoadState::Loaded(UsageDataset { rows: vec![row] });
        app.usage_app = Some(usage);
        app
    }

    fn text(lines: Vec<Line<'static>>) -> String {
        lines
            .into_iter()
            .map(|line| {
                line.spans
                    .into_iter()
                    .map(|span| span.content.into_owned())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn header_body_and_columns_share_exact_cell_budget()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        for width in 0..220 {
            for height in [0, 1, 2, 3, 16, 40] {
                let geometry = TableGeometry::new(Rect::new(5, 9, width, height));
                let columns: u16 = geometry.columns.iter().map(|(_, width)| width).sum();
                assert_eq!(columns + 3 * (geometry.columns.len() as u16 - 1), width);
                assert_eq!(
                    geometry.header.height + geometry.rule.height + geometry.body.height,
                    height
                );
                assert_eq!(geometry.body.bottom(), 9 + height);
            }
        }
        Ok(())
    }

    #[test]
    fn tab_mouse_hits_only_the_rendered_title_cells()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut app = fixture(Platform::Codex);
        app.tabs.push(super::super::tests::empty_platform_tab(
            Platform::Claude,
            TabVariant::Profile,
            "",
        ));
        app.tabs.push(super::super::tests::empty_platform_tab(
            Platform::Grok,
            TabVariant::Profile,
            "",
        ));
        for width in [80, 140, 200] {
            let mut terminal = Terminal::new(TestBackend::new(width, 3))?;
            terminal.draw(|frame| render_header(frame, &app, frame.area()))?;
            for (index, label) in [(1, "Claude"), (2, "Grok")] {
                let row = (0..width)
                    .map(|x| {
                        terminal
                            .backend()
                            .buffer()
                            .cell((x, 1))
                            .expect("cell inside fixture terminal")
                            .symbol()
                    })
                    .collect::<String>();
                let column =
                    row[..row.find(label).ok_or("rendered tab title missing")?].width() as u16;
                assert_eq!(
                    tab_hit_index(&app, Rect::new(0, 0, width, 3), column, 1),
                    Some(index)
                );
                assert_eq!(
                    tab_hit_index(&app, Rect::new(0, 0, width, 3), column, 0),
                    None
                );
                assert_eq!(
                    tab_hit_index(&app, Rect::new(0, 0, width, 3), column, 2),
                    None
                );
            }
            assert_eq!(
                tab_hit_index(&app, Rect::new(0, 0, width, 3), width - 1, 1),
                None
            );
        }
        Ok(())
    }

    #[test]
    fn relative_dates_have_explicit_boundaries_and_future_warning()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let now: DateTime<Utc> = "2026-10-12T12:00:00Z".parse()?;
        assert_eq!(
            relative_time(now - chrono::Duration::seconds(59), now),
            ("now".into(), false)
        );
        assert_eq!(
            relative_time(now - chrono::Duration::seconds(60), now),
            ("1m ago".into(), false)
        );
        assert_eq!(
            relative_time(now - chrono::Duration::hours(1), now),
            ("1h ago".into(), false)
        );
        assert_eq!(
            relative_time(now + chrono::Duration::seconds(1), now),
            ("!10-12 12:00".into(), true)
        );
        Ok(())
    }

    #[test]
    fn six_tab_header_keeps_platform_names_and_mouse_targets_at_narrow_widths()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        for language in [TuiLanguage::English, TuiLanguage::SimplifiedChinese] {
            crate::tui::i18n::set_language(language);
            let mut app = fixture(Platform::Codex);
            app.tabs = [
                (Platform::Codex, TabVariant::Profile),
                (Platform::Claude, TabVariant::Profile),
                (Platform::Grok, TabVariant::Profile),
                (Platform::Codex, TabVariant::CodexAuth),
                (Platform::Claude, TabVariant::ClaudeAuth),
                (Platform::Grok, TabVariant::GrokAuth),
            ]
            .into_iter()
            .map(|(platform, variant)| {
                super::super::tests::empty_platform_tab(platform, variant, "")
            })
            .collect();
            for width in [60, 80, 100] {
                for active in 0..6 {
                    app.active_tab = active;
                    let mut terminal = Terminal::new(TestBackend::new(width, 3))?;
                    terminal.draw(|frame| render_header(frame, &app, frame.area()))?;
                    let buffer = terminal.backend().buffer();
                    for (index, tab) in app.tabs.iter().enumerate() {
                        let label = tab.compact_display_label();
                        let label_chars: Vec<_> = label.chars().collect();
                        let start = (1..width - 1)
                            .find(|&x| {
                                let mut position = x;
                                let matches = label_chars.iter().all(|ch| {
                                    let matches = buffer
                                        .cell((position, 1))
                                        .is_some_and(|cell| cell.symbol() == ch.to_string());
                                    position += ch.width().unwrap_or(0) as u16;
                                    matches
                                });
                                matches
                                    && (index < 3
                                        || tab_hit_index(&app, Rect::new(0, 0, width, 3), x, 1)
                                            == (index != active).then_some(index))
                            })
                            .expect("every platform title is completely visible");
                        assert_eq!(
                            tab_hit_index(&app, Rect::new(0, 0, width, 3), start, 1),
                            (index != active).then_some(index)
                        );
                        let end = start + label.width() as u16 - 1;
                        assert_eq!(
                            tab_hit_index(&app, Rect::new(0, 0, width, 3), end, 1),
                            (index != active).then_some(index)
                        );
                        assert!(end < width - 1);
                    }
                }
            }
        }
        crate::tui::i18n::set_language(TuiLanguage::English);
        Ok(())
    }

    #[test]
    fn bars_keep_common_scale_zero_partial_and_extreme_values()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        assert_eq!(token_bar(0, 100, 8), "        ");
        assert_eq!(token_bar(1, 64, 8), "▏       ");
        assert_eq!(token_bar(32, 64, 8), "████    ");
        assert_eq!(token_bar(i64::MAX, i64::MAX, 8), "████████");
        assert_eq!(token_bar(-1, 64, 8), "        ");
        assert_eq!(token_bar(1, 64, 0), "");
        Ok(())
    }

    #[test]
    fn actual_detail_rows_wrap_without_losing_final_lines_or_secrets()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        for platform in [Platform::Codex, Platform::Claude, Platform::Grok] {
            let mut app = fixture(platform);
            let config = app.tabs[0]
                .profile_configs
                .get_mut("work")
                .ok_or("fixture work profile missing")?;
            config.base_url = Some("https://secret-user:secret-password@api.example.invalid/very-long-路径/path?key=secret-query#secret-fragment".into());
            config.model = Some("very-long-model-name-中文-012345678901234567890123456789".into());
            for width in [20, 28, 56, 58, 70, 102] {
                let lines = detail_lines(&app, width);
                assert!(
                    lines.iter().all(|line| line.width() <= width),
                    "{platform:?} {width}"
                );
                let rendered = text(lines);
                for secret in [
                    "fixture-secret-never-render-this",
                    "secret-user",
                    "secret-password",
                    "secret-query",
                    "secret-fragment",
                ] {
                    assert!(!rendered.contains(secret));
                }
                assert!(
                    rendered.contains("Last used = successful")
                        || rendered
                            .replace('\n', "")
                            .contains("Last used = successful")
                );
                assert_eq!(rendered.replace('\n', "").matches("2026-10-12").count(), 1);
            }
            let mut terminal = Terminal::new(TestBackend::new(60, 20))?;
            app.profile_detail_scroll = u16::MAX;
            terminal.draw(|frame| render_details(frame, &mut app, frame.area()))?;
            let buffer = terminal.backend().buffer();
            let rendered = buffer
                .content
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>();
            assert!(rendered.contains("successful CCR activation"));
        }
        Ok(())
    }

    #[test]
    fn provider_scope_unknowns_and_numeric_narrow_fallback_remain_truthful()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut app = fixture(Platform::Grok);
        let wide = text(usage_ledger(&app, Some("example-provider"), 80));
        assert!(wide.contains("59.4M") && wide.contains("109.8M") && wide.contains("Grok"));
        assert!(wide.contains("official-equivalent USD") && wide.contains("All-time provider"));
        assert!(wide.contains('█'));
        let narrow = text(usage_ledger(&app, Some("example-provider"), 42));
        assert!(!narrow.contains('█'));
        assert!(narrow.contains("59.4M"));
        assert!(!text(usage_ledger(&app, None, 80)).contains("59.4M"));
        assert!(!text(usage_ledger(&app, Some("different-provider"), 80)).contains("59.4M"));
        app.usage_app
            .as_mut()
            .ok_or("fixture usage engine missing")?
            .state = UsageLoadState::Error("fixture query error".into());
        let error = text(usage_ledger(&app, Some("example-provider"), 80));
        assert!(error.contains("fixture query error") && !error.contains("$0"));
        Ok(())
    }

    #[test]
    fn invalid_category_counts_stay_on_their_aligned_row()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut app = fixture(Platform::Codex);
        if let UsageLoadState::Loaded(dataset) = &mut app
            .usage_app
            .as_mut()
            .ok_or("fixture usage engine missing")?
            .state
        {
            dataset.rows[0].breakdown.input_tokens = -1;
        }
        for width in [58, 80] {
            let lines = usage_ledger(&app, Some("example-provider"), width);
            let input = lines
                .into_iter()
                .find(|line| {
                    line.spans
                        .first()
                        .is_some_and(|span| span.content.starts_with("Input"))
                })
                .expect("the input category remains present");
            assert!(text(vec![input]).contains("Invalid"));
        }
        Ok(())
    }

    #[test]
    fn current_runtime_does_not_follow_selection_and_grok_uses_auth_mode()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        for platform in [Platform::Codex, Platform::Claude, Platform::Grok] {
            let mut app = fixture(platform);
            let initial = runtime_values(&app);
            app.selected_index = 8;
            app.selected_profile_name = Some("profile-08".into());
            assert_eq!(runtime_values(&app), initial);
            assert_eq!(initial.2.as_deref(), Some("production"));
            if platform == Platform::Grok {
                assert_eq!(initial.3, "env / EXAMPLE_API_KEY");
            }
        }
        Ok(())
    }

    #[test]
    fn connection_values_keep_semantic_colors_after_wrapping()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut config = ProfileConfig::new();
        config.model = Some("model-with-a-very-long-name-that-wraps".into());
        config
            .platform_data
            .insert("model_reasoning_effort".into(), serde_json::json!("high"));
        let lines = connection_rows(Platform::Codex, &config, false, 26);
        let value_start = lines[0].spans[0].width() as u16;
        let value_spans: Vec<_> = lines
            .iter()
            .flat_map(|line| line.spans.iter().skip(1))
            .collect();
        assert!(
            value_spans
                .iter()
                .filter(|span| span.content.contains("model")
                    || span.content.contains("wraps")
                    || span.content == "high")
                .all(
                    |span| span.style.fg == Some(theme::accent_for(Platform::Codex))
                        && span.style.add_modifier.contains(Modifier::BOLD)
                )
        );
        let mut terminal = Terminal::new(TestBackend::new(30, 16))?;
        terminal.draw(|frame| frame.render_widget(Paragraph::new(lines), frame.area()))?;
        let model_start = terminal
            .backend()
            .buffer()
            .cell((value_start, 0))
            .ok_or("model cell outside terminal")?;
        assert_eq!(model_start.fg, theme::accent_for(Platform::Codex));
        assert!(model_start.modifier.contains(Modifier::BOLD));
        Ok(())
    }

    #[test]
    fn three_platforms_two_locales_and_responsive_geometry_use_real_cells()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        for language in [TuiLanguage::English, TuiLanguage::SimplifiedChinese] {
            crate::tui::i18n::set_language(language);
            for platform in [Platform::Codex, Platform::Claude, Platform::Grok] {
                for (width, height) in [
                    (60, 16),
                    (80, 20),
                    (89, 22),
                    (90, 22),
                    (100, 30),
                    (119, 30),
                    (120, 22),
                    (140, 30),
                    (200, 50),
                ] {
                    let mut app = fixture(platform);
                    let mut terminal = Terminal::new(TestBackend::new(width, height))?;
                    terminal.draw(|frame| draw(frame, &mut app))?;
                    let body = app
                        .list_area
                        .get()
                        .ok_or("render did not expose list geometry")?;
                    assert_eq!(app.page_size, usize::from(body.height.max(1)));
                    assert!(body.right() <= width && body.bottom() <= height);
                    assert!(body.y >= 9, "header/rule must not hit a record");
                    let selected_y = body.y + app.selected_index as u16;
                    if selected_y < body.bottom() {
                        let cell = terminal
                            .backend()
                            .buffer()
                            .cell((body.x, selected_y))
                            .ok_or("selected cell outside terminal")?;
                        assert_eq!(cell.bg, theme::selection_bg());
                    }
                }
            }
        }
        crate::tui::i18n::set_language(TuiLanguage::English);
        Ok(())
    }

    /// Run alone, once per theme. This exports actual Ratatui cells; it never loads user data.
    #[test]
    #[ignore = "explicit fixture artifact export; CCR_TUI_RENDER_OUTPUT and CCR_TUI_RENDER_THEME required"]
    fn export_profile_workbench_cells() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let output = std::path::PathBuf::from(
            std::env::var("CCR_TUI_RENDER_OUTPUT").expect("artifact output directory"),
        );
        let theme_name = std::env::var("CCR_TUI_RENDER_THEME").expect("mocha or latte");
        theme::set_theme(match theme_name.as_str() {
            "mocha" => theme::ThemeVariant::Mocha,
            "latte" => theme::ThemeVariant::Latte,
            _ => panic!("unsupported theme"),
        });
        std::fs::create_dir_all(&output)?;
        for language in [TuiLanguage::English, TuiLanguage::SimplifiedChinese] {
            crate::tui::i18n::set_language(language);
            let locale = if language == TuiLanguage::English {
                "en"
            } else {
                "zh"
            };
            for platform in [Platform::Codex, Platform::Claude, Platform::Grok] {
                for (width, height) in [(200, 50), (80, 20)] {
                    let mut app = fixture(platform);
                    export_cells(
                        &output,
                        &format!(
                            "{}-{locale}-{theme_name}-{width}x{height}",
                            platform.short_name()
                        ),
                        &mut app,
                        width,
                        height,
                    )?;
                }
            }
        }
        crate::tui::i18n::set_language(TuiLanguage::English);
        for variant in ["long", "error", "empty", "scroll-end", "zero"] {
            let mut app = fixture(Platform::Codex);
            match variant {
                "long" => {
                    let config = app.tabs[0]
                        .profile_configs
                        .get_mut("work")
                        .ok_or("fixture work profile missing")?;
                    config.model =
                        Some("long-model-name-中文-012345678901234567890123456789".into());
                    config.base_url = Some(
                        "https://api.example.invalid/very-long/compatible-mode/v1/chat/completions"
                            .into(),
                    );
                }
                "error" => {
                    app.usage_app
                        .as_mut()
                        .ok_or("fixture usage engine missing")?
                        .state = UsageLoadState::Error("fixture database unavailable".into());
                    app.tabs[0].activity = Some(Err(
                        ccr_cli::application::profile_activity::ProfileActivityError::Query,
                    ));
                }
                "empty" => {
                    app.usage_app
                        .as_mut()
                        .ok_or("fixture usage engine missing")?
                        .state = UsageLoadState::Empty;
                    app.tabs[0]
                        .activity
                        .as_mut()
                        .ok_or("fixture activity missing")?
                        .as_mut()
                        .map_err(|error| *error)?
                        .profiles
                        .clear();
                }
                "scroll-end" => app.profile_detail_scroll = u16::MAX,
                "zero" => {
                    if let UsageLoadState::Loaded(dataset) = &mut app
                        .usage_app
                        .as_mut()
                        .ok_or("fixture usage engine missing")?
                        .state
                    {
                        let row = &mut dataset.rows[0].breakdown;
                        row.input_tokens = 0;
                        row.cache_read_tokens = 0;
                        row.cache_creation_tokens = 0;
                        row.output_tokens = 0;
                        row.reasoning_output_tokens = 0;
                        row.total_tokens = 0;
                        row.request_count = 0;
                        row.cost_with_cache_usd = 0.0;
                    }
                }
                _ => {}
            }
            export_cells(
                &output,
                &format!("codex-en-{theme_name}-{variant}"),
                &mut app,
                140,
                30,
            )?;
        }
        Ok(())
    }

    fn export_cells(
        output: &std::path::Path,
        name: &str,
        app: &mut App,
        width: u16,
        height: u16,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let selected_platform = app.current_platform();
        let mut active = Some(app.tabs.remove(0));
        for (platform, variant) in [
            (Platform::Codex, TabVariant::Profile),
            (Platform::Claude, TabVariant::Profile),
            (Platform::Grok, TabVariant::Profile),
            (Platform::Codex, TabVariant::CodexAuth),
            (Platform::Claude, TabVariant::ClaudeAuth),
            (Platform::Grok, TabVariant::GrokAuth),
        ] {
            if platform == selected_platform && variant == TabVariant::Profile {
                app.active_tab = app.tabs.len();
                app.tabs.push(
                    active
                        .take()
                        .ok_or("selected fixture tab already consumed")?,
                );
            } else {
                app.tabs.push(super::super::tests::empty_platform_tab(
                    platform, variant, "",
                ));
            }
        }
        let mut terminal = Terminal::new(TestBackend::new(width, height))?;
        terminal.draw(|frame| draw(frame, app))?;
        let cells: Vec<_> = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| {
                serde_json::json!([
                    cell.symbol(),
                    format!("{:?}", cell.fg),
                    format!("{:?}", cell.bg),
                    cell.modifier.bits()
                ])
            })
            .collect();
        let value = serde_json::json!({"fixture_only":true,"renderer":"Ratatui TestBackend","width":width,"height":height,"cells":cells});
        std::fs::write(
            output.join(format!("{name}.json")),
            serde_json::to_vec(&value)?,
        )?;
        Ok(())
    }
}
