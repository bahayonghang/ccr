//! Profile editor form and change preview.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use unicode_width::UnicodeWidthStr;

use super::ProfileEditor;
use super::catalog;
use super::field::EditorField;
use super::request::{PreviewValue, create_preview_rows, preview_rows};
use super::schema::{self, EditorMode, FieldKind};
use crate::tui::theme;
use crate::tui::ui::detail_layout::{plan_detail, ruler_line, wrap_lines};
use crate::tui::ui::{pad_text, truncate_text};

pub(crate) fn render_stacked(frame: &mut Frame, editor: &mut ProfileEditor, area: Rect) {
    let changes = if editor.is_create() {
        create_preview_rows(&editor.create_draft())
            .len()
            .saturating_add(1)
    } else {
        preview_rows(&editor.draft()).len()
    };
    let preview_height = u16::try_from((changes + 2).clamp(3, 9))
        .unwrap_or(9)
        .min(area.height.saturating_sub(1));
    let chunks =
        Layout::vertical([Constraint::Min(1), Constraint::Length(preview_height)]).split(area);
    render_form(frame, editor, chunks[0]);
    render_preview(frame, editor, chunks[1]);
}

pub(crate) fn render_rail(frame: &mut Frame, editor: &mut ProfileEditor, area: Rect) {
    let width = area.width.saturating_sub(2) as usize;
    let mut block = editor_block(editor);
    let inner = block.inner(area);
    let form = form_lines(editor, width);
    let focus = focus_span(&form, editor.focus_index());
    let preview = preview_lines(editor, width);
    let preview_min = if preview.is_empty() { 1 } else { 2 };
    let inner_height = usize::from(inner.height);
    if inner_height > preview_min {
        let budget = inner_height - preview_min;
        let (start, window) = window_lines(editor, &form.lines, focus, budget);
        let title = preview_title(editor.is_create(), width);
        let plan = plan_detail(inner, window, Some(title), preview, 0);
        if form.lines.len() > budget {
            let end = (start + budget).min(form.lines.len());
            block = block.title_bottom(
                Line::styled(
                    crate::tui_format!(
                        " {}–{} / {} ",
                        " {}–{} / {} ",
                        start + 1,
                        end.max(start + 1),
                        form.lines.len()
                    ),
                    theme::muted_style(),
                )
                .alignment(ratatui::layout::Alignment::Right),
            );
        }
        frame.render_widget(block, area);
        paint_plan(frame, &plan, inner);
        return;
    }
    let mut lines = form.lines;
    lines.push(preview_title(editor.is_create(), width));
    lines.extend(preview);
    let (start, window) = window_lines(editor, &lines, focus, inner_height);
    let _ = start;
    frame.render_widget(block, area);
    if inner.height > 0 {
        frame.render_widget(Paragraph::new(window), inner);
    }
}

pub(crate) fn render_saving(frame: &mut Frame, area: Rect, creating: bool) {
    let (title, body) = if creating {
        (
            crate::tui_text!(" Creating ", " 正在创建 "),
            crate::tui_text!("Creating…", "正在创建…"),
        )
    } else {
        (
            crate::tui_text!(" Saving ", " 正在保存 "),
            crate::tui_text!("Saving…", "正在保存…"),
        )
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::border()))
        .title(title)
        .title_style(theme::secondary_text_emphasis_style());
    frame.render_widget(
        Paragraph::new(body).style(theme::info_style()).block(block),
        area,
    );
}

fn render_form(frame: &mut Frame, editor: &mut ProfileEditor, area: Rect) {
    let mut block = editor_block(editor);
    let inner = block.inner(area);
    let width = inner.width as usize;
    let form = form_lines(editor, width);
    let focus = focus_span(&form, editor.focus_index());
    let height = usize::from(inner.height);
    let (start, window) = window_lines(editor, &form.lines, focus, height);
    if form.lines.len() > height && height > 0 {
        let end = (start + height).min(form.lines.len());
        block = block.title_bottom(
            Line::styled(
                crate::tui_format!(
                    " {}–{} / {} ",
                    " {}–{} / {} ",
                    start + 1,
                    end.max(start + 1),
                    form.lines.len()
                ),
                theme::muted_style(),
            )
            .alignment(ratatui::layout::Alignment::Right),
        );
    }
    frame.render_widget(block, area);
    if inner.height > 0 {
        frame.render_widget(Paragraph::new(window).wrap(Wrap { trim: false }), inner);
    }
}

fn render_preview(frame: &mut Frame, editor: &ProfileEditor, area: Rect) {
    let width = area.width.saturating_sub(2) as usize;
    let title = if editor.is_create() {
        crate::tui_text!(
            " New profile (not activated) ",
            " 新建摘要（创建后不自动激活） "
        )
    } else {
        crate::tui_text!(" Changes ", " 改动 ")
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::border()))
        .title(title)
        .title_style(theme::secondary_text_emphasis_style());
    let lines = preview_lines(editor, width);
    let body = if lines.is_empty() {
        vec![Line::styled(
            crate::tui_text!("No changes", "没有改动"),
            theme::muted_style(),
        )]
    } else {
        lines
    };
    frame.render_widget(Paragraph::new(body).block(block), area);
}

fn editor_block(editor: &ProfileEditor) -> Block<'static> {
    let title = if editor.is_create() {
        crate::tui_format!(
            "New {} profile (* required)",
            "新建 {} 配置 (* 为必填)",
            schema::platform_label(editor.platform())
        )
    } else {
        crate::tui_format!("Edit · {}", "编辑 · {}", editor.original_name())
    };
    Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::border()))
        .title(format!(" {title} "))
        .title_style(theme::platform_style_for(editor.platform()).add_modifier(Modifier::BOLD))
}

struct FormView {
    lines: Vec<Line<'static>>,
    focus_at: Vec<(usize, usize)>,
}

fn form_lines(editor: &ProfileEditor, width: usize) -> FormView {
    let mut lines = Vec::new();
    let mut focus_at = Vec::new();
    let mode = editor.mode();
    let platform = editor.platform();
    let auth_mode = editor.create_auth_mode_enum();
    if let Some(banner) = editor.banner() {
        let wrapped = wrap_lines(
            vec![Line::styled(banner.to_string(), theme::warning_style())],
            width.max(1),
        );
        lines.extend(wrapped);
    }
    let label_width = editor
        .fields()
        .iter()
        .filter(|field| field.shown)
        .map(|field| schema::label(field.spec.key).width())
        .max()
        .unwrap_or(8)
        .saturating_add(2)
        .min(width.saturating_sub(8).max(4));
    for (index, field) in editor.fields().iter().enumerate() {
        if !field.shown {
            focus_at.push((lines.len(), 1));
            continue;
        }
        let start = lines.len();
        lines.push(field_line(
            field,
            index == editor.focus_index(),
            editor.advanced_open(),
            mode,
            platform,
            auth_mode,
            label_width,
            width,
        ));
        if let Some(error) = &field.error {
            lines.push(Line::styled(
                truncate_text(&format!("  {error}"), width.max(1)),
                theme::error_style(),
            ));
        }
        focus_at.push((start, lines.len() - start));
    }
    FormView { lines, focus_at }
}

fn focus_span(form: &FormView, focus: usize) -> (usize, usize) {
    form.focus_at.get(focus).copied().unwrap_or((0, 1))
}

fn window_lines(
    editor: &mut ProfileEditor,
    lines: &[Line<'static>],
    focus: (usize, usize),
    viewport: usize,
) -> (usize, Vec<Line<'static>>) {
    if viewport == 0 || lines.is_empty() {
        return (0, Vec::new());
    }
    editor.align_scroll(focus.0, focus.1, viewport);
    let max_start = lines.len().saturating_sub(viewport);
    if editor.form_scroll() > max_start {
        editor.set_form_scroll(max_start);
    }
    let start = editor.form_scroll();
    let end = (start + viewport).min(lines.len());
    (start, lines[start..end].to_vec())
}

#[allow(clippy::too_many_arguments)]
fn field_line(
    field: &EditorField,
    focused: bool,
    advanced_open: bool,
    mode: EditorMode,
    platform: ccr_cli::models::Platform,
    auth_mode: schema::CreateAuthMode,
    label_width: usize,
    width: usize,
) -> Line<'static> {
    let row = if focused {
        Style::default()
            .bg(theme::selection_bg())
            .fg(theme::selection_fg())
    } else {
        Style::default()
    };
    let required = mode == EditorMode::Create && auth_mode.is_field_required(field.spec.key);
    let marker = if required {
        "* "
    } else if field.is_dirty() {
        "• "
    } else {
        "  "
    };
    let marker_style = if focused {
        row
    } else if field.error.is_some() {
        theme::error_style()
    } else if required {
        theme::platform_style_for(platform).add_modifier(Modifier::BOLD)
    } else if field.is_dirty() {
        theme::warning_style()
    } else {
        row
    };
    let label = pad_text(
        &truncate_text(schema::label(field.spec.key), label_width),
        label_width,
    );
    let value_budget = width.saturating_sub(marker.width() + label.width() + 1);
    let value = truncate_text(
        &field_value(field, advanced_open, mode, platform),
        value_budget.max(1),
    );
    Line::from(vec![
        Span::styled(marker.to_string(), marker_style),
        Span::styled(label, row),
        Span::styled(format!(" {value}"), row),
    ])
    .style(row)
}

fn field_value(
    field: &EditorField,
    advanced_open: bool,
    mode: EditorMode,
    platform: ccr_cli::models::Platform,
) -> String {
    match field.spec.kind {
        FieldKind::Secret if mode == EditorMode::Create => {
            if field.text.value().trim().is_empty() {
                crate::tui_text!("‹empty›", "‹未填写›").to_string()
            } else {
                field.text.secret_dots()
            }
        }
        FieldKind::Secret if field.secret_clear => {
            crate::tui_text!("will clear", "将清除").to_string()
        }
        FieldKind::Secret if field.text.value().trim().is_empty() => {
            crate::tui_text!("unchanged", "保留").to_string()
        }
        FieldKind::Secret => field.text.secret_dots(),
        FieldKind::Toggle => {
            if field.toggle {
                crate::tui_text!("yes", "是").to_string()
            } else {
                crate::tui_text!("no", "否").to_string()
            }
        }
        FieldKind::AuthMode(_) => {
            format!(
                "◀ {} ▶",
                schema::auth_mode_label(platform, field.choice_value())
            )
        }
        FieldKind::Choice(_) | FieldKind::Credential => choice_label(field),
        FieldKind::Model(options) => model_label(field, options),
        FieldKind::Advanced => {
            if advanced_open {
                crate::tui_text!("hide", "收起").to_string()
            } else {
                crate::tui_text!("show", "展开").to_string()
            }
        }
        FieldKind::Text | FieldKind::Number | FieldKind::Tags => field.text.value().to_string(),
    }
}

fn model_label(field: &EditorField, options: &[catalog::ModelOption]) -> String {
    if field
        .choices
        .get(field.choice_index)
        .is_some_and(|item| item.custom)
    {
        let typed = field.text.value().trim();
        if typed.is_empty() {
            return crate::tui_text!("type a model id", "输入模型 ID").to_string();
        }
        return format!("{typed} {}", crate::tui_text!("(custom)", "（自定义）"));
    }
    let id = field.choice_value();
    if id.is_empty() {
        "—".to_string()
    } else {
        catalog::display(id, options)
    }
}

fn choice_label(field: &EditorField) -> String {
    let value = field.choice_value();
    let label = if field.spec.kind == FieldKind::Credential {
        schema::credential_label(value).to_string()
    } else if value.is_empty() {
        "—".to_string()
    } else if let Some(mapped) = schema::choice_value_label(field.spec.key, value) {
        mapped.to_string()
    } else {
        value.to_string()
    };
    let custom = field
        .choices
        .get(field.choice_index)
        .is_some_and(|item| item.custom);
    if custom {
        format!("{label} {}", crate::tui_text!("(custom)", "（自定义）"))
    } else {
        label
    }
}

fn preview_title(is_create: bool, width: usize) -> Line<'static> {
    let title = if is_create {
        crate::tui_text!("New profile · not activated", "新建摘要 · 不自动激活")
    } else {
        crate::tui_text!("Changes", "改动")
    };
    ruler_line(title, None, width, theme::secondary_text_emphasis_style())
}

fn preview_lines(editor: &ProfileEditor, width: usize) -> Vec<Line<'static>> {
    if width == 0 {
        return Vec::new();
    }
    let rows = if editor.is_create() {
        create_preview_rows(&editor.create_draft())
    } else {
        preview_rows(&editor.draft())
    };
    if rows.is_empty() && !editor.is_create() {
        return Vec::new();
    }
    let key_width = rows
        .iter()
        .map(|row| schema::label(&row.key).width())
        .max()
        .unwrap_or(1)
        .min(width.saturating_sub(4).max(1));
    let mut lines = Vec::new();
    for row in rows {
        let key = pad_text(
            &truncate_text(schema::label(&row.key), key_width),
            key_width,
        );
        let value = match row.value {
            PreviewValue::Text { old, new } => format!("{old} → {new}"),
            PreviewValue::Set(text) => text,
            PreviewValue::SecretSet => crate::tui_text!("‹set›", "‹已设置›").to_string(),
            PreviewValue::SecretUnset => crate::tui_text!("‹unset›", "‹未设置›").to_string(),
            PreviewValue::Replaced => crate::tui_text!("‹replaced›", "‹已替换›").to_string(),
            PreviewValue::Clear => crate::tui_text!("will clear", "将清除").to_string(),
        };
        let prefix = format!("{key} = ");
        let wrapped = wrap_lines(
            vec![Line::from(Span::styled(value, theme::primary_text_style()))],
            width.saturating_sub(prefix.width()).max(1),
        );
        if wrapped.is_empty() {
            lines.push(Line::from(Span::styled(
                prefix,
                theme::secondary_text_style(),
            )));
            continue;
        }
        for (index, value_line) in wrapped.into_iter().enumerate() {
            let mut spans = vec![Span::styled(
                if index == 0 {
                    prefix.clone()
                } else {
                    " ".repeat(prefix.width())
                },
                theme::secondary_text_style(),
            )];
            spans.extend(value_line.spans);
            lines.push(Line::from(spans));
        }
    }
    if editor.is_create() {
        lines.extend(wrap_lines(
            vec![Line::styled(
                crate::tui_text!(
                    "Creates inactive profile; current runtime unchanged.",
                    "新建后默认未激活；不修改当前运行时。"
                )
                .to_string(),
                theme::muted_style(),
            )],
            width.max(1),
        ));
    }
    lines
}

fn paint_plan(frame: &mut Frame, plan: &crate::tui::ui::detail_layout::DetailPlan, inner: Rect) {
    if let Some(split) = &plan.split {
        if split.fixed_rect.height > 0 {
            frame.render_widget(Paragraph::new(split.fixed.clone()), split.fixed_rect);
        }
        if split.title_rect.height > 0 {
            frame.render_widget(Paragraph::new(split.title.clone()), split.title_rect);
        }
        if split.body_rect.height > 0 {
            frame.render_widget(Paragraph::new(split.body.clone()), split.body_rect);
        }
    } else if inner.height > 0 {
        frame.render_widget(
            Paragraph::new(plan.whole.clone()).scroll((u16::try_from(plan.scroll).unwrap_or(0), 0)),
            inner,
        );
    }
}
