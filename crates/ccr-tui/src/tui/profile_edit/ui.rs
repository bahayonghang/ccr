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
use super::request::{PreviewValue, preview_rows};
use super::schema::{self, FieldKind};
use crate::tui::theme;
use crate::tui::ui::detail_layout::{plan_detail, ruler_line, wrap_lines};
use crate::tui::ui::{pad_text, truncate_text};

pub(crate) fn render_stacked(frame: &mut Frame, editor: &mut ProfileEditor, area: Rect) {
    let changes = preview_rows(&editor.draft()).len();
    let preview_height = u16::try_from((changes + 2).clamp(3, 8))
        .unwrap_or(8)
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
        let title = preview_title(width);
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
    lines.push(preview_title(width));
    lines.extend(preview);
    let (start, window) = window_lines(editor, &lines, focus, inner_height);
    let _ = start;
    frame.render_widget(block, area);
    if inner.height > 0 {
        frame.render_widget(Paragraph::new(window), inner);
    }
}

pub(crate) fn render_saving(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::border()))
        .title(crate::tui_text!(" Saving ", " 正在保存 "))
        .title_style(theme::secondary_text_emphasis_style());
    frame.render_widget(
        Paragraph::new(crate::tui_text!("Saving…", "正在保存…"))
            .style(theme::info_style())
            .block(block),
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
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::border()))
        .title(crate::tui_text!(" Changes ", " 改动 "))
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
    Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::border()))
        .title(format!(
            " {} ",
            crate::tui_format!("Edit · {}", "编辑 · {}", editor.original_name())
        ))
        .title_style(theme::platform_style_for(editor.platform()).add_modifier(Modifier::BOLD))
}

struct FormView {
    lines: Vec<Line<'static>>,
    focus_at: Vec<(usize, usize)>,
}

fn form_lines(editor: &ProfileEditor, width: usize) -> FormView {
    let mut lines = Vec::new();
    let mut focus_at = Vec::new();
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

fn field_line(
    field: &EditorField,
    focused: bool,
    advanced_open: bool,
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
    let marker = if field.is_dirty() { "• " } else { "  " };
    let label = pad_text(
        &truncate_text(schema::label(field.spec.key), label_width),
        label_width,
    );
    let value_budget = width.saturating_sub(marker.width() + label.width() + 1);
    let value = truncate_text(&field_value(field, advanced_open), value_budget.max(1));
    Line::from(vec![
        Span::styled(
            marker.to_string(),
            if field.is_dirty() && !focused {
                theme::warning_style()
            } else {
                row
            },
        ),
        Span::styled(label, row),
        Span::styled(format!(" {value}"), row),
    ])
    .style(row)
}

fn field_value(field: &EditorField, advanced_open: bool) -> String {
    match field.spec.kind {
        FieldKind::Secret if field.secret_clear => {
            crate::tui_text!("will clear", "将清除").to_string()
        }
        FieldKind::Secret if field.text.value().is_empty() => {
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

fn preview_title(width: usize) -> Line<'static> {
    ruler_line(
        crate::tui_text!("Changes", "改动"),
        None,
        width,
        theme::secondary_text_emphasis_style(),
    )
}

fn preview_lines(editor: &ProfileEditor, width: usize) -> Vec<Line<'static>> {
    let rows = preview_rows(&editor.draft());
    if rows.is_empty() || width == 0 {
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
