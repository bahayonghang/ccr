use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShortcutHint<'a> {
    pub key: &'a str,
    pub label: &'a str,
}

impl<'a> ShortcutHint<'a> {
    pub const fn new(key: &'a str, label: &'a str) -> Self {
        Self { key, label }
    }
}

pub fn shortcut_line(hints: &[ShortcutHint<'_>], accent: Color) -> Line<'static> {
    let mut spans = Vec::with_capacity(hints.len().saturating_mul(3));

    for (index, hint) in hints.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled("  │  ", Style::default().fg(theme::muted())));
        }
        spans.push(Span::styled(
            hint.key.to_string(),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ));
        if !hint.label.is_empty() {
            spans.push(Span::styled(
                if hint.key.is_empty() {
                    hint.label.to_string()
                } else {
                    format!(" {}", hint.label)
                },
                theme::secondary_text_style(),
            ));
        }
    }

    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_line_uses_accent_description_and_separator_hierarchy() {
        let line = shortcut_line(
            &[
                ShortcutHint::new("Enter", "apply"),
                ShortcutHint::new("q", "quit"),
            ],
            Color::Cyan,
        );

        assert_eq!(line.spans[0].content, "Enter");
        assert_eq!(line.spans[0].style.fg, Some(Color::Cyan));
        assert!(line.spans[0].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(line.spans[1].style.fg, Some(theme::subtext()));
        assert_eq!(line.spans[2].content, "  │  ");
        assert_eq!(line.spans[2].style.fg, Some(theme::muted()));
    }
}
