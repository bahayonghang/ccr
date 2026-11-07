//! Calendar-spaced fee trend for the Claude home.
use super::*;
use ccr_usage::DailyTrendDto;
use chrono::{Duration, NaiveDate};
use gpui_kit::base::plot::{
    AxisLabelSide, AxisText, Curve, Grid, PlotAxis, axis_gutter, label::measure_text_width,
    shape::Line,
};
use gpui_kit::{App, Bounds, Pixels, SharedString, TextAlign, canvas, point, size};

const DAYS: usize = 30;
const PLOT_HEIGHT: f32 = 194.;

struct FeeTrend {
    start: NaiveDate,
    points: Vec<(usize, f64)>,
}

impl FeeTrend {
    fn from_rows(until: NaiveDate, rows: &[DailyTrendDto]) -> Self {
        let start = until - Duration::days(DAYS as i64 - 1);
        let mut days = [None; DAYS];
        for row in rows {
            let Ok(date) = NaiveDate::parse_from_str(&row.date, "%Y-%m-%d") else {
                continue;
            };
            let day = (date - start).num_days();
            if (0..DAYS as i64).contains(&day) && row.cost_usd.is_finite() {
                days[day as usize] = Some(row.cost_usd);
            }
        }
        Self {
            start,
            points: days
                .into_iter()
                .enumerate()
                .filter_map(|(day, fee)| Some((day, fee?)))
                .collect(),
        }
    }

    // The Kit LineChart positions consecutive data by index. Use its underlying Line
    // per uninterrupted run so a missing day is neither a zero nor a connecting line.
    fn runs(&self) -> impl Iterator<Item = &[(usize, f64)]> {
        self.points.chunk_by(|a, b| b.0 == a.0 + 1)
    }

    fn cost_ticks(&self) -> Vec<f64> {
        let (low, high) = self
            .points
            .iter()
            .fold((0.0_f64, 0.0_f64), |(low, high), &(_, fee)| {
                (low.min(fee), high.max(fee))
            });
        // Round to 1, 2 or 5 times a power of ten, like the Tauri cost axis.
        if low == high {
            return vec![0., 0.5, 1.];
        }
        let target = high / 3. - low / 3.;
        let base = 10_f64.powf(target.log10().floor());
        let step = [1., 2., 5., 10.]
            .into_iter()
            .map(|n| base * n)
            .find(|n| *n >= target)
            .unwrap_or(target);
        let first = (low / step).floor();
        let last = (high / step).ceil();
        if !(first * step).is_finite() || !(last * step).is_finite() {
            // At the limits of SQLite REAL a nice tick can exceed the finite range.
            return vec![low, high];
        }
        (first as i32..=last as i32)
            .map(|i| f64::from(i) * step)
            .collect()
    }

    fn paint(&self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let ticks = self.cost_ticks();
        let low = ticks[0];
        let high = ticks[ticks.len() - 1];
        let labels: Vec<SharedString> = ticks
            .iter()
            .copied()
            .map(|value| {
                if value != 0. && !(0.0001..1.0e7).contains(&value.abs()) {
                    format!("${value:.2e}").into()
                } else {
                    format!("${value:.4}")
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_owned()
                        .into()
                }
            })
            .collect();
        let font_size = px(theme::META_SIZE);
        let gutter = labels
            .iter()
            .map(|label| measure_text_width(label, font_size, window))
            .fold(0., f32::max)
            + 10.;
        let plot = Bounds::new(
            bounds.origin + point(px(gutter), px(8.)),
            size(
                bounds.size.width - px(gutter + 4.),
                bounds.size.height - px(8. + axis_gutter(font_size)),
            ),
        );
        let width = plot.size.width.as_f32();
        let height = plot.size.height.as_f32();
        if width <= 0. || height <= 0. {
            return;
        }

        let x = move |day: usize| width * day as f32 / (DAYS - 1) as f32;
        let scale = low.abs().max(high.abs());
        let y = move |fee: f64| {
            height * ((high / scale - fee / scale) / (high / scale - low / scale)) as f32
        };
        let muted = cx.theme().muted_foreground;
        let stroke = theme::platform_color("claude", cx.theme().mode);
        Grid::new()
            .y(ticks.iter().map(|&v| px(y(v))))
            .stroke(cx.theme().border)
            .paint(&plot, window);
        PlotAxis::new()
            .stroke(cx.theme().border)
            .x(px(height))
            .x_label([0, 7, 14, 21, DAYS - 1].map(|day| {
                let date = self.start + Duration::days(day as i64);
                let align = match day {
                    0 => TextAlign::Left,
                    n if n == DAYS - 1 => TextAlign::Right,
                    _ => TextAlign::Center,
                };
                AxisText::new(date.format("%m-%d").to_string(), px(x(day)), muted)
                    .align(align)
                    .font_size(font_size)
            }))
            .y(px(0.))
            .y_label_side(AxisLabelSide::Start)
            .y_label(labels.into_iter().zip(ticks).map(|(label, fee)| {
                AxisText::new(label, px(y(fee)), muted)
                    .align(TextAlign::Right)
                    .font_size(font_size)
            }))
            .paint(&plot, window, cx);
        for run in self.runs() {
            Line::new()
                .data(run.iter().copied())
                .x(move |p| Some(x(p.0)))
                .y(move |p| Some(y(p.1)))
                .curve(Curve::Linear)
                .stroke_width(2.)
                .stroke(stroke)
                .dot()
                .dot_size(4.)
                .dot_fill(stroke)
                .paint(&plot, window);
        }
    }
}

impl WorkbenchView {
    pub(super) fn claude_trend(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let title = text(
            cx,
            "Daily recorded cost · last 30 days (USD)",
            "近 30 天每日记录费用（USD）",
        );
        let section = div()
            .id("workbench-claude-trend")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .p(px(theme::SECTION_GAP))
            .rounded(px(theme::PANEL_RADIUS))
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .id("workbench-claude-trend-title")
                    .role(gpui_kit::Role::Heading)
                    .aria_level(3)
                    .aria_label(title)
                    .text_size(px(theme::SECTION_TITLE_SIZE))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(title),
            );
        let state = match self.usage.as_ref() {
            Some(Ok(report)) => match report.as_ref() {
                InsightsSnapshot::Usage(report) => {
                    let trend = FeeTrend::from_rows(self.usage_until, &report.trends);
                    if !trend.points.is_empty() {
                        let range = format!(
                            "{} · {} – {}",
                            text(cx, "llmusage · Reporting period", "llmusage · 统计周期"),
                            trend.start,
                            self.usage_until
                        );
                        return section
                            .child(
                                div()
                                    .text_size(px(theme::META_SIZE))
                                    .text_color(cx.theme().muted_foreground)
                                    .child(range.clone()),
                            )
                            .child(
                                div()
                                    .id("workbench-claude-trend-plot")
                                    .test_support()
                                    .role(gpui_kit::Role::Image)
                                    .aria_label(format!(
                                        "{title}. {range}. {}: {}",
                                        text(cx, "Days with records", "有记录的天数"),
                                        trend.points.len()
                                    ))
                                    .w_full()
                                    .h(px(PLOT_HEIGHT))
                                    .child(
                                        canvas(
                                            |_, _, _| {},
                                            move |bounds, _, window, cx| {
                                                trend.paint(bounds, window, cx)
                                            },
                                        )
                                        .size_full(),
                                    ),
                            )
                            .into_any_element();
                    }
                    text(
                        cx,
                        "No daily costs recorded in this period.",
                        "此时段没有逐日费用记录。",
                    )
                }
                _ => text(cx, "Daily costs unavailable.", "逐日费用不可用。"),
            },
            Some(Err(error)) => service_error(cx, error),
            None => text(cx, "Reading daily costs…", "正在读取逐日费用…"),
        };
        section
            .child(
                div()
                    .id("workbench-claude-trend-status")
                    .role(gpui_kit::Role::Status)
                    .aria_label(state)
                    .text_color(cx.theme().muted_foreground)
                    .child(state),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn daily(date: NaiveDate, requests: i64, cost_usd: f64) -> DailyTrendDto {
        DailyTrendDto {
            date: date.to_string(),
            request_count: requests,
            total_tokens: 0,
            input_tokens: 0,
            output_tokens: 0,
            reasoning_output_tokens: 0,
            cache_read_tokens: 0,
            cache_creation_tokens: 0,
            cost_usd,
        }
    }

    #[test]
    fn trend_keeps_calendar_positions_cost_and_gaps_across_the_entire_query() {
        let start = NaiveDate::from_ymd_opt(2026, 10, 9).unwrap_or_default();
        let date = |day| start + Duration::days(day);
        let mut rows: Vec<_> = (0..15)
            .map(|day| daily(date(day), 100 - day, day as f64))
            .collect();
        rows.extend([
            daily(date(29), 1, 72.),
            daily(date(-1), 1, 99.),
            daily(date(30), 1, 99.),
        ]);
        let trend = FeeTrend::from_rows(date(29), &rows);
        let runs: Vec<_> = trend.runs().collect();
        assert_eq!(trend.start, start);
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].len(), 15);
        assert_eq!(runs[0][0], (0, 0.));
        assert_eq!(runs[0][14], (14, 14.));
        assert_eq!(runs[1], [(29, 72.)]);
    }

    #[test]
    fn trend_distinguishes_empty_window_from_isolated_zero() {
        let end = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap_or_default();
        assert!(FeeTrend::from_rows(end, &[]).points.is_empty());
        let trend = FeeTrend::from_rows(end, &[daily(end - Duration::days(8), 5, 0.)]);
        assert_eq!(trend.points, [(21, 0.)]);
    }

    #[test]
    fn cost_axis_is_bounded_for_finite_recorded_amounts() {
        let date = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap_or_default();
        for (low, high) in [
            (0., 143.5663),
            (0., f64::from_bits(1)),
            (0., f64::MAX),
            (f64::MIN, f64::MAX),
        ] {
            let ticks = FeeTrend {
                start: date,
                points: vec![(0, low), (1, high)],
            }
            .cost_ticks();
            assert!((2..=6).contains(&ticks.len()));
            assert!(ticks.iter().all(|v| v.is_finite()));
            assert!(ticks.windows(2).all(|v| v[0] < v[1]));
            assert!(ticks[0] <= low && ticks[ticks.len() - 1] >= high);
        }
    }
}
