//! Full-height profile detail geometry.
//!
//! Fixed sections keep a stable start row for one platform and viewport.
//! The remaining inner height is an independently scrolled elastic pane.
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use super::{pad_text, theme, truncate_text};

/// Connection fields share two columns once the inner width reaches this many cells.
pub(super) const CONNECTION_COLUMNS_MIN_WIDTH: usize = 64;
const COLUMN_GUTTER: usize = 2;

/// One connection slot. A missing value still occupies its cell so later sections do not jump.
pub(super) struct ConnectionSlot {
    pub label: String,
    pub value: Option<(String, Style)>,
    pub exclusive: bool,
}

#[derive(Clone)]
pub(crate) struct SplitDetail {
    pub fixed: Vec<Line<'static>>,
    pub fixed_rect: Rect,
    pub title: Line<'static>,
    pub title_rect: Rect,
    pub body: Vec<Line<'static>>,
    pub body_rect: Rect,
}

pub(crate) struct DetailPlan {
    pub whole: Vec<Line<'static>>,
    pub split: Option<SplitDetail>,
    pub scroll: usize,
    pub max_scroll: usize,
    pub visible: usize,
    pub total: usize,
}

/// Wrap styled spans on terminal display width. Scroll extent uses the same lines.
pub(crate) fn wrap_lines(lines: Vec<Line<'static>>, width: usize) -> Vec<Line<'static>> {
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

/// `─ title ─── status`, exactly `width` cells. The status stays on this one line.
pub(crate) fn ruler_line(
    title: &str,
    status: Option<(&str, Style)>,
    width: usize,
    title_style: Style,
) -> Line<'static> {
    if width == 0 {
        return Line::from("");
    }
    let border = Style::default().fg(theme::border());
    let title = format!("─ {title}");
    let Some((status, status_style)) = status else {
        let title_text = truncate_text(&title, width);
        let fill = width.saturating_sub(title_text.width());
        return Line::from(vec![
            Span::styled(title_text, title_style),
            Span::styled("─".repeat(fill), border),
        ]);
    };
    if status.width() >= width {
        return Line::from(Span::styled(truncate_text(status, width), status_style));
    }
    let status_text = status.to_string();
    let title_budget = width - status_text.width() - 1;
    let title_text = truncate_text(&title, title_budget);
    let fill = title_budget.saturating_sub(title_text.width());
    let mut spans = Vec::new();
    if !title_text.is_empty() {
        spans.push(Span::styled(title_text, title_style));
    }
    if fill > 0 {
        spans.push(Span::styled("─".repeat(fill), border));
    }
    spans.push(Span::styled(format!(" {status_text}"), status_style));
    Line::from(spans)
}

pub(super) fn spread_line(
    left: &str,
    left_style: Style,
    right: &str,
    right_style: Style,
    width: usize,
) -> Line<'static> {
    if width == 0 {
        return Line::from("");
    }
    if right.is_empty() {
        return Line::from(Span::styled(truncate_text(left, width), left_style));
    }
    if right.width() >= width {
        return Line::from(Span::styled(truncate_text(right, width), right_style));
    }
    let left_text = truncate_text(left, width - right.width() - 1);
    let gap = width - left_text.width() - right.width();
    Line::from(vec![
        Span::styled(left_text, left_style),
        Span::raw(" ".repeat(gap)),
        Span::styled(right.to_string(), right_style),
    ])
}

pub(super) fn layout_connection(slots: &[ConnectionSlot], width: usize) -> Vec<Line<'static>> {
    if width < CONNECTION_COLUMNS_MIN_WIDTH {
        return single_column(slots, width);
    }
    let column_width = width.saturating_sub(COLUMN_GUTTER) / 2;
    let mut lines = Vec::new();
    let mut index = 0;
    while index < slots.len() {
        if slot_is_exclusive(&slots[index], column_width) {
            lines.extend(exclusive_lines(&slots[index], width));
            index += 1;
            continue;
        }
        if index + 1 < slots.len() && !slot_is_exclusive(&slots[index + 1], column_width) {
            lines.push(pair_line(&slots[index], &slots[index + 1], column_width));
            index += 2;
        } else {
            lines.extend(exclusive_lines(&slots[index], width));
            index += 1;
        }
    }
    lines
}

fn slot_is_exclusive(slot: &ConnectionSlot, column_width: usize) -> bool {
    if slot.exclusive || column_width == 0 {
        return true;
    }
    let Some((value, _)) = &slot.value else {
        return false;
    };
    let label_width = slot.label.width().min(column_width);
    let gap = usize::from(column_width > label_width);
    value.width() > column_width.saturating_sub(label_width + gap)
}

fn single_column(slots: &[ConnectionSlot], width: usize) -> Vec<Line<'static>> {
    let label_width = slots
        .iter()
        .filter(|slot| slot.value.is_some())
        .map(|slot| slot.label.width())
        .max()
        .unwrap_or(0)
        .saturating_add(2)
        .min(width.saturating_sub(4).max(1));
    slots
        .iter()
        .flat_map(|slot| slot_lines(slot, label_width, width))
        .collect()
}

fn exclusive_lines(slot: &ConnectionSlot, width: usize) -> Vec<Line<'static>> {
    let label_width = slot
        .label
        .width()
        .saturating_add(2)
        .min(width.saturating_sub(4).max(1));
    slot_lines(slot, label_width, width)
}

fn slot_lines(slot: &ConnectionSlot, label_width: usize, width: usize) -> Vec<Line<'static>> {
    let Some((value, style)) = &slot.value else {
        return vec![Line::from("")];
    };
    if width == 0 {
        return vec![Line::from("")];
    }
    let value_width = width.saturating_sub(label_width).max(1);
    let wrapped = wrap_lines(
        vec![Line::from(Span::styled(value.clone(), *style))],
        value_width,
    );
    if wrapped.is_empty() {
        return vec![Line::from("")];
    }
    wrapped
        .into_iter()
        .enumerate()
        .map(|(index, value_line)| {
            let label = if index == 0 {
                pad_text(
                    &truncate_text(&slot.label, label_width.saturating_sub(1)),
                    label_width,
                )
            } else {
                " ".repeat(label_width)
            };
            let mut spans = vec![Span::styled(label, theme::secondary_text_style())];
            spans.extend(value_line.spans);
            Line::from(spans)
        })
        .collect()
}

fn pair_line(left: &ConnectionSlot, right: &ConnectionSlot, column_width: usize) -> Line<'static> {
    let mut spans = column_spans(left, column_width);
    spans.push(Span::raw(" ".repeat(COLUMN_GUTTER)));
    spans.extend(column_spans(right, column_width));
    Line::from(spans)
}

fn column_spans(slot: &ConnectionSlot, column_width: usize) -> Vec<Span<'static>> {
    if column_width == 0 {
        return Vec::new();
    }
    let Some((value, style)) = &slot.value else {
        return vec![Span::raw(" ".repeat(column_width))];
    };
    let label_limit = column_width.saturating_sub(1);
    let label_width = slot.label.width().min(label_limit);
    let label = pad_text(&truncate_text(&slot.label, label_width), label_width);
    let value_budget = column_width.saturating_sub(label.width() + 1);
    let value_text = truncate_text(value, value_budget.max(1));
    let used = label.width() + 1 + value_text.width();
    let pad = column_width.saturating_sub(used.min(column_width));
    vec![
        Span::styled(label, theme::secondary_text_style()),
        Span::raw(" "),
        Span::styled(value_text, *style),
        Span::raw(" ".repeat(pad)),
    ]
}

/// Fixed sections stay put when they fit with at least a one-line elastic title.
/// A non-empty elastic body also needs one visible row; otherwise the whole pane scrolls.
pub(crate) fn plan_detail(
    inner: Rect,
    fixed: Vec<Line<'static>>,
    elastic_title: Option<Line<'static>>,
    elastic_body: Vec<Line<'static>>,
    scroll: usize,
) -> DetailPlan {
    let Some(title) = elastic_title else {
        return whole_plan(inner, fixed, scroll);
    };
    let min_elastic = if elastic_body.is_empty() { 1 } else { 2 };
    let fixed_height = u16::try_from(fixed.len()).unwrap_or(u16::MAX);
    let fits = inner.height > 0
        && usize::from(inner.height) >= fixed.len().saturating_add(min_elastic)
        && fixed_height < inner.height;
    if !fits {
        let mut lines = fixed;
        lines.push(title);
        lines.extend(elastic_body);
        return whole_plan(inner, lines, scroll);
    }
    let elastic_height = inner.height - fixed_height;
    let title_rect = Rect::new(inner.x, inner.y + fixed_height, inner.width, 1);
    let body_height = elastic_height.saturating_sub(1);
    let body_rect = Rect::new(inner.x, title_rect.y + 1, inner.width, body_height);
    let total = elastic_body.len();
    let (scroll, max_scroll, visible) = scroll_window(total, usize::from(body_height), scroll);
    DetailPlan {
        whole: Vec::new(),
        split: Some(SplitDetail {
            fixed,
            fixed_rect: Rect::new(inner.x, inner.y, inner.width, fixed_height),
            title,
            title_rect,
            body: elastic_body,
            body_rect,
        }),
        scroll,
        max_scroll,
        visible,
        total,
    }
}

fn whole_plan(inner: Rect, lines: Vec<Line<'static>>, scroll: usize) -> DetailPlan {
    let total = lines.len();
    let (scroll, max_scroll, visible) = scroll_window(total, usize::from(inner.height), scroll);
    DetailPlan {
        whole: lines,
        split: None,
        scroll,
        max_scroll,
        visible,
        total,
    }
}

fn scroll_window(total: usize, visible: usize, scroll: usize) -> (usize, usize, usize) {
    let max_scroll = if visible == 0 {
        0
    } else {
        total.saturating_sub(visible)
    };
    (scroll.min(max_scroll), max_scroll, visible.min(total))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    fn slot(label: &str, value: Option<&str>, exclusive: bool) -> ConnectionSlot {
        ConnectionSlot {
            label: label.to_string(),
            value: value.map(|value| (value.to_string(), Style::default())),
            exclusive,
        }
    }

    #[test]
    fn ruler_keeps_cjk_status_inside_the_requested_width() {
        let line = ruler_line(
            "用量 · 提供商 / Grok",
            Some(("暂无记录", Style::default())),
            36,
            Style::default(),
        );
        assert_eq!(line.width(), 36);
        let text: String = line
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(text.contains("暂无记录"));
        assert!(text.contains("用量"));
    }

    #[test]
    fn short_connection_values_share_a_row_only_at_64_columns() {
        let slots = [
            slot("模型", Some("grok-4.6"), false),
            slot("推理强度", Some("high"), false),
            slot("地址", Some("https://example.invalid/v1"), true),
        ];
        let wide = layout_connection(&slots, 64);
        assert_eq!(wide.len(), 2, "{wide:?}");
        assert!(wide.iter().all(|line| line.width() <= 64));
        let paired: String = wide[0]
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect();
        assert!(paired.contains("grok-4.6"));
        assert!(paired.contains("high"));
        let narrow = layout_connection(&slots, 63);
        assert_eq!(narrow.len(), 3);
        assert!(narrow.iter().all(|line| line.width() <= 63));
    }

    #[test]
    fn missing_connection_slots_keep_the_following_row() {
        let present = [
            slot("Model", Some("gpt"), false),
            slot("Reasoning", Some("high"), false),
            slot("Endpoint", Some("https://example.invalid"), true),
        ];
        let missing = [
            slot("Model", Some("gpt"), false),
            slot("Reasoning", None, false),
            slot("Endpoint", Some("https://example.invalid"), true),
        ];
        assert_eq!(
            layout_connection(&present, 80).len(),
            layout_connection(&missing, 80).len()
        );
    }
}
