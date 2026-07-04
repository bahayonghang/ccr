use ccr_usage::{SourceKind, TaggedProviderBreakdown};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};

use crate::models::Platform;
use crate::tui::theme;

use super::app::{UsageApp, UsageDataset, UsageLoadState};

pub fn draw(frame: &mut Frame, app: &mut UsageApp) {
    draw_embedded(
        frame,
        app,
        frame.area(),
        Rect::new(0, 0, 0, 0),
        theme::viewport_mode(frame.area().width, frame.area().height),
    );
}

pub fn draw_embedded(
    frame: &mut Frame,
    app: &mut UsageApp,
    area: Rect,
    footer_area: Rect,
    mode: theme::ViewportMode,
) {
    match &app.state {
        UsageLoadState::Idle | UsageLoadState::Loading => {
            render_message(
                frame,
                area,
                "Usage",
                "Loading provider usage from llmusage...",
                theme::muted_style(),
            );
        }
        UsageLoadState::Empty => {
            render_message(
                frame,
                area,
                "Usage",
                "No Claude/Codex provider usage rows yet. Run llmusage sync/import, then press r.",
                theme::muted_style(),
            );
        }
        UsageLoadState::Unsupported(message) => {
            render_message(
                frame,
                area,
                "Usage unavailable",
                &format!("Update llmusage to schema 14 with provider_label support.\n{message}"),
                theme::warning_style(),
            );
        }
        UsageLoadState::Error(message) => {
            render_message(frame, area, "Usage error", message, theme::error_style());
        }
        UsageLoadState::Loaded(dataset) => {
            render_dataset(frame, area, dataset, mode);
        }
    }

    if footer_area.width > 0 && footer_area.height > 0 {
        render_footer(frame, app, footer_area);
    }
}

pub fn draw_loading_placeholder(
    frame: &mut Frame,
    area: Rect,
    footer_area: Rect,
    _mode: theme::ViewportMode,
    error: Option<&str>,
) {
    let message = error.unwrap_or("Preparing usage view...");
    let style = if error.is_some() {
        theme::error_style()
    } else {
        theme::muted_style()
    };
    render_message(frame, area, "Usage", message, style);
    if footer_area.width > 0 && footer_area.height > 0 {
        render_static_footer(frame, footer_area);
    }
}

fn render_dataset(
    frame: &mut Frame,
    area: Rect,
    dataset: &UsageDataset,
    mode: theme::ViewportMode,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);

    let table_area = chunks[0];
    let note_area = chunks[1];
    let compact = mode == theme::ViewportMode::Compact || table_area.width < 96;
    let rows = usage_rows(dataset, compact);
    let widths = if compact {
        vec![
            Constraint::Length(9),
            Constraint::Min(12),
            Constraint::Length(7),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(9),
        ]
    } else {
        vec![
            Constraint::Length(9),
            Constraint::Min(14),
            Constraint::Length(8),
            Constraint::Length(11),
            Constraint::Length(11),
            Constraint::Length(11),
            Constraint::Length(11),
            Constraint::Length(12),
        ]
    };
    let headers = if compact {
        Row::new(["Platform", "Provider", "Req", "In", "Out", "Cost"])
    } else {
        Row::new([
            "Platform", "Provider", "Requests", "Input", "Output", "Cache", "Total", "Approx $",
        ])
    }
    .style(theme::secondary_text_emphasis_style());

    let table = Table::new(rows, widths)
        .header(headers)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(symbols::border::ROUNDED)
                .border_style(Style::default().fg(theme::codex()))
                .title(" Provider Usage ")
                .title_style(theme::codex_style()),
        )
        .column_spacing(1);

    frame.render_widget(table, table_area);

    let note = Paragraph::new(Line::from(vec![
        Span::styled("Cost is ", theme::secondary_text_style()),
        Span::styled(
            "approx official-equivalent price",
            theme::secondary_text_emphasis_style(),
        ),
        Span::styled(
            "; third-party relay billing may differ.",
            theme::secondary_text_style(),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(theme::border()))
            .padding(ratatui::widgets::Padding::horizontal(1)),
    )
    .wrap(Wrap { trim: true });
    frame.render_widget(note, note_area);
}

fn usage_rows(dataset: &UsageDataset, compact: bool) -> Vec<Row<'static>> {
    let mut rows = Vec::new();
    for platform in [SourceKind::Claude, SourceKind::Codex] {
        for row in dataset.platform_rows(platform) {
            rows.push(render_row(row, compact));
        }
    }
    rows
}

fn render_row(row: &TaggedProviderBreakdown, compact: bool) -> Row<'static> {
    let platform = platform_label(row.source);
    let provider = truncate(
        row.breakdown.display_provider(),
        if compact { 18 } else { 24 },
    );
    let style = platform_style(row.source);
    if compact {
        Row::new(vec![
            Cell::from(Span::styled(platform, style)),
            Cell::from(provider),
            Cell::from(format_count(row.breakdown.request_count)),
            Cell::from(format_count(row.breakdown.input_tokens)),
            Cell::from(format_count(row.breakdown.output_tokens_total())),
            Cell::from(format_cost(row.breakdown.cost_with_cache_usd)),
        ])
    } else {
        Row::new(vec![
            Cell::from(Span::styled(platform, style)),
            Cell::from(provider),
            Cell::from(format_count(row.breakdown.request_count)),
            Cell::from(format_count(row.breakdown.input_tokens)),
            Cell::from(format_count(row.breakdown.output_tokens_total())),
            Cell::from(format_count(row.breakdown.cache_tokens_total())),
            Cell::from(format_count(row.breakdown.total_tokens)),
            Cell::from(format_cost(row.breakdown.cost_with_cache_usd)),
        ])
    }
}

fn render_message(frame: &mut Frame, area: Rect, title: &str, message: &str, style: Style) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(theme::codex()))
        .title(format!(" {title} "))
        .title_style(theme::codex_style())
        .padding(ratatui::widgets::Padding::horizontal(1));

    let paragraph = Paragraph::new(message.to_string())
        .block(block)
        .style(style)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame, app: &UsageApp, area: Rect) {
    let shortcuts = "Tab/Shift+Tab switch  |  r refresh  |  q quit";
    let text = if let Some(toast) = app.toasts.active() {
        format!("{}  |  {}", toast.message, shortcuts)
    } else {
        shortcuts.to_string()
    };
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_set(symbols::border::ROUNDED)
                .border_style(Style::default().fg(theme::border()))
                .title(" Keys ")
                .title_alignment(Alignment::Center)
                .title_style(theme::muted_style()),
        )
        .style(theme::secondary_text_style())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn render_static_footer(frame: &mut Frame, area: Rect) {
    let paragraph = Paragraph::new("Tab/Shift+Tab switch  |  r refresh  |  q quit")
        .style(theme::secondary_text_style())
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn platform_label(platform: SourceKind) -> &'static str {
    match platform {
        SourceKind::Claude => "Claude",
        SourceKind::Codex => "Codex",
        SourceKind::Gemini => "Gemini",
        SourceKind::Opencode => "OpenCode",
    }
}

fn platform_style(platform: SourceKind) -> Style {
    match platform {
        SourceKind::Claude => theme::platform_style_for(Platform::Claude),
        SourceKind::Codex => theme::platform_style_for(Platform::Codex),
        SourceKind::Gemini => theme::platform_style_for(Platform::Gemini),
        SourceKind::Opencode => theme::opencode_style(),
    }
}

fn truncate(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        return value.to_string();
    }
    if width <= 1 {
        return "…".to_string();
    }
    let mut out: String = value.chars().take(width - 1).collect();
    out.push('…');
    out
}

fn format_count(value: i64) -> String {
    let abs = value.abs();
    if abs >= 1_000_000 {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    } else if abs >= 1_000 {
        format!("{:.1}K", value as f64 / 1_000.0)
    } else {
        value.to_string()
    }
}

fn format_cost(value: f64) -> String {
    if value >= 100.0 {
        format!("${value:.0}")
    } else if value >= 1.0 {
        format!("${value:.2}")
    } else {
        format!("${value:.4}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_formatting_compacts_large_values() {
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(1_500), "1.5K");
        assert_eq!(format_count(2_000_000), "2.0M");
    }

    #[test]
    fn row_data_uses_unattributed_provider_label() {
        let row = ccr_usage::ProviderBreakdownDto {
            provider: None,
            request_count: 1,
            input_tokens: 2,
            cache_read_tokens: 3,
            cache_creation_tokens: 4,
            output_tokens: 5,
            reasoning_output_tokens: 6,
            total_tokens: 20,
            cost_with_cache_usd: 0.0123,
            cost_without_cache_usd: 0.02,
        };

        assert_eq!(row.display_provider(), "unattributed");
        assert_eq!(truncate(row.display_provider(), 6), "unatt…");
    }
}
