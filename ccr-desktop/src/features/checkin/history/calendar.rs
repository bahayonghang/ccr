use super::*;
use crate::runtime::checkin::{DayBalance, HistoryDay};
use chrono::Months;
use gpui_kit::component::ActiveTheme;
use std::collections::BTreeSet;

fn balance_text(balance: &DayBalance, cx: &App) -> String {
    match balance {
        DayBalance::Missing => text(cx, "No balance sample", "缺少余额样本").into(),
        DayBalance::Unusable { source, unit } => format!(
            "{} · {} · {source}",
            text(cx, "Unusable sample", "样本不可用"),
            unit.as_deref()
                .unwrap_or(text(cx, "Unknown unit", "单位未知"))
        ),
        DayBalance::Measured(n) | DayBalance::CarriedForward(n) => {
            let state = if matches!(balance, DayBalance::Measured(_)) {
                text(cx, "Measured", "实测")
            } else {
                text(cx, "Carried forward", "延续")
            };
            format!(
                "{state} · {} {} · {} {} · {} {} · {} {}",
                n.remaining,
                n.unit,
                text(cx, "Total", "额度"),
                n.total,
                text(cx, "Used", "已用"),
                n.used,
                text(cx, "Source", "来源"),
                n.source
            )
        }
    }
}

fn day_text(day: &HistoryDay, cx: &App) -> String {
    let status = if day.checked {
        text(cx, "Checked in", "已签到")
    } else {
        text(cx, "No successful check-in", "无成功签到记录")
    };
    let mut line = format!(
        "{} · {status} · {}",
        day.date,
        balance_text(&day.balance, cx)
    );
    if let Some(reward) = &day.reward {
        line.push_str(&format!(" · {} {reward}", text(cx, "Reward", "奖励")));
    }
    line
}

impl CheckinView {
    pub(in crate::features::checkin) fn calendar_panel(
        &self,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let month = self.history.month;
        let mut panel = div().flex().flex_col().min_w_0().gap(px(6.)).child(
            crate::shell::badge::section_heading(
                text(cx, "Calendar and balance trend", "月历与余额趋势"),
                None,
                cx,
            ),
        );
        let mut nav = div().flex().flex_wrap().gap(px(4.)).items_center();
        for (id, label, step) in [
            ("prev", text(cx, "Previous month", "上个月"), -1),
            ("next", text(cx, "Next month", "下个月"), 1),
        ] {
            let target = if step < 0 {
                month.checked_sub_months(Months::new(1))
            } else {
                month.checked_add_months(Months::new(1))
            }
            .filter(|d| (1..=9999).contains(&d.year()));
            nav = nav.child(semantics::ghost_button(
                Button::new(format!("checkin-month-{id}"))
                    .label(label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(date) = target {
                            this.history.month = date;
                            this.history.selected_day = None;
                            cx.notify();
                        }
                    })),
                target.is_none(),
                false,
                cx,
            ));
        }
        panel = panel.child(nav.child(note(
            "checkin-history-month",
            month.format("%Y-%m").to_string(),
        )));
        panel = panel.child(
            div().flex().child(semantics::ghost_button(
                Button::new("checkin-history-custom")
                    .label(text(cx, "Choose month or range", "选择月份或天数"))
                    .on_click(cx.listener(|this, _, w, cx| {
                        let h = &mut this.history;
                        h.range_editor = !h.range_editor;
                        h.range_invalid = false;
                        h.month_field.update(cx, |s, cx| {
                            s.set_value(h.month.format("%Y-%m").to_string(), w, cx)
                        });
                        h.days_field
                            .update(cx, |s, cx| s.set_value(h.trend_days.to_string(), w, cx));
                        cx.notify();
                    })),
                false,
                false,
                cx,
            )),
        );
        if self.history.range_editor {
            let month_label = text(cx, "Month (YYYY-MM)", "月份（YYYY-MM）");
            let days_label = text(
                cx,
                "Trend days (0 for selected month)",
                "趋势天数（0 表示所选月）",
            );
            panel = panel.child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(4.))
                    .child(month_label)
                    .child(
                        div().w(px(100.)).child(
                            Input::new(&self.history.month_field)
                                .id("checkin-range-month")
                                .aria_label(month_label),
                        ),
                    )
                    .child(days_label)
                    .child(
                        div().w(px(64.)).child(
                            Input::new(&self.history.days_field)
                                .id("checkin-range-days")
                                .aria_label(days_label),
                        ),
                    )
                    .child(semantics::button(
                        Button::new("checkin-apply-range")
                            .label(text(cx, "Apply", "应用"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                let h = &mut this.history;
                                let month = format!("{}-01", h.month_field.read(cx).value().trim())
                                    .parse::<NaiveDate>();
                                let days = h.days_field.read(cx).value().trim().parse::<u32>();
                                match (month, days) {
                                    (Ok(m), Ok(days @ 0..=366))
                                        if (1..=9999).contains(&m.year()) =>
                                    {
                                        h.month = m;
                                        h.trend_days = days;
                                        h.range_editor = false;
                                        h.range_invalid = false;
                                        h.selected_day = None;
                                    }
                                    _ => h.range_invalid = true,
                                }
                                cx.notify();
                            })),
                        false,
                        false,
                        cx,
                    )),
            );
            if self.history.range_invalid {
                panel = panel.child(text(
                    cx,
                    "Enter a month as YYYY-MM and 0–366 days.",
                    "请输入 YYYY-MM 格式的月份及 0–366 天。",
                ));
            }
        }
        let active = self.history.active.as_ref();
        let history = match self
            .history
            .detail
            .as_ref()
            .filter(|(q, _)| Some(q) == active)
            .map(|(_, v)| v)
        {
            Some(Ok(Some(h))) => h,
            Some(Ok(None)) => {
                return panel
                    .child(text(
                        cx,
                        "No local check-in database.",
                        "尚无本机签到数据库。",
                    ))
                    .into_any_element();
            }
            Some(Err(_)) => return panel.child(self.history_error(cx)).into_any_element(),
            None => {
                return panel
                    .child(text(cx, "Reading account history…", "正在读取账户历史…"))
                    .into_any_element();
            }
        };
        let days: Vec<_> = history
            .days
            .iter()
            .filter(|d| d.date.year() == month.year() && d.date.month() == month.month())
            .collect();
        let checked = days.iter().filter(|d| d.checked).count();
        let summary = format!(
            "{}: {checked} · {}: {}",
            text(cx, "Checked days", "签到天数"),
            text(cx, "Current streak", "当前连续签到"),
            history.current_streak
        );
        panel = panel.child(note("checkin-calendar-summary", summary));
        let offset = month.weekday().num_days_from_monday() as usize;
        let mut headings = div().flex().gap(px(2.));
        for (en, zh) in [
            ("Mo", "一"),
            ("Tu", "二"),
            ("We", "三"),
            ("Th", "四"),
            ("Fr", "五"),
            ("Sa", "六"),
            ("Su", "日"),
        ] {
            headings = headings.child(div().flex_1().text_center().child(text(cx, en, zh)));
        }
        panel = panel.child(headings);
        for week in 0..(days.len() + offset).div_ceil(7) {
            let mut row = div().flex().gap(px(2.));
            for col in 0..7 {
                let cell = div().flex_1().min_w_0();
                if let Some(d) = (week * 7 + col)
                    .checked_sub(offset)
                    .and_then(|i| days.get(i))
                {
                    let date = d.date;
                    let label = format!("{}{}", date.day(), if d.checked { " ✓" } else { "" });
                    let button = Button::new(format!("checkin-calendar-{date}"))
                        .w_full()
                        .label(label)
                        .accessibility_label(day_text(d, cx))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.history.selected_day = Some(date);
                            cx.notify();
                        }));
                    row = row.child(cell.child(crate::shell::badge::filter_chip(
                        button,
                        self.history.selected_day == Some(date),
                        false,
                        cx,
                    )));
                } else {
                    row = row.child(cell);
                }
            }
            panel = panel.child(row);
        }
        if let Some(d) = days
            .iter()
            .find(|d| Some(d.date) == self.history.selected_day)
        {
            panel = panel.child(note("checkin-selected-day", day_text(d, cx)));
        }
        let mut ranges = div()
            .flex()
            .flex_wrap()
            .gap(px(4.))
            .items_center()
            .child(text(cx, "Trend window", "趋势范围"));
        for (n, label) in [
            (0, text(cx, "This month", "所选月").into()),
            (7, format!("7 {}", text(cx, "days", "天"))),
            (30, format!("30 {}", text(cx, "days", "天"))),
            (90, format!("90 {}", text(cx, "days", "天"))),
        ] {
            ranges = ranges.child(crate::shell::badge::filter_chip(
                Button::new(format!("checkin-range-{n}"))
                    .label(label)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.history.trend_days = n;
                        cx.notify();
                    })),
                self.history.trend_days == n,
                false,
                cx,
            ));
        }
        panel = panel.child(ranges);
        let trend: Vec<_> = history
            .days
            .iter()
            .filter(|d| (history.trend_from..=history.trend_to).contains(&d.date))
            .collect();
        let range = format!("{} – {}", history.trend_from, history.trend_to);
        panel = panel.child(note("checkin-trend-range", range));
        let units: BTreeSet<_> = trend
            .iter()
            .filter_map(|d| d.balance.numbers().map(|n| n.unit.as_str()))
            .collect();
        if units.is_empty() {
            panel = panel.child(note(
                "checkin-trend-empty",
                text(
                    cx,
                    "No usable balance samples in this window.",
                    "此范围内没有可用的余额样本。",
                ),
            ));
        }
        for unit in units {
            panel = panel.child(unit_chart(unit, &trend, cx));
        }
        for (id, label, expanded) in [
            (
                "values",
                text(cx, "Daily values", "逐日数值"),
                self.history.show_values,
            ),
            (
                "samples",
                text(cx, "Saved balance samples", "原始余额快照"),
                self.history.show_samples,
            ),
        ] {
            panel = panel.child(
                div().flex().child(semantics::ghost_button(
                    Button::new(format!("checkin-history-{id}"))
                        .label(label)
                        .selected(expanded)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if id == "values" {
                                this.history.show_values = !expanded;
                            } else {
                                this.history.show_samples = !expanded;
                            }
                            cx.notify();
                        })),
                    false,
                    false,
                    cx,
                )),
            );
            if expanded && id == "values" {
                for d in &trend {
                    panel = panel.child(note(
                        format!("checkin-trend-day-{}", d.date),
                        day_text(d, cx),
                    ));
                }
            } else if expanded {
                let mut count = 0;
                for (at, balance) in history
                    .samples
                    .iter()
                    .rev()
                    .filter(|(t, _)| t.date_naive() >= month)
                {
                    count += 1;
                    panel = panel.child(format!(
                        "{} · {}",
                        at.to_rfc3339(),
                        balance_text(balance, cx)
                    ));
                }
                if count == 0 {
                    panel = panel.child(text(
                        cx,
                        "No samples recorded this month.",
                        "此月没有原始余额快照。",
                    ));
                }
            }
        }
        panel.into_any_element()
    }

    pub(super) fn history_error(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        div()
            .flex()
            .flex_wrap()
            .gap(px(6.))
            .items_center()
            .child(note(
                "checkin-history-error",
                text(cx, "Could not read local history.", "无法读取本机历史。"),
            ))
            .child(semantics::ghost_button(
                Button::new("checkin-history-retry")
                    .label(text(cx, "Retry", "重试"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.history.invalidate();
                        cx.notify();
                    })),
                false,
                false,
                cx,
            ))
            .into_any_element()
    }
}

fn unit_chart(unit: &str, days: &[&HistoryDay], cx: &App) -> impl IntoElement + use<> {
    let value = |d: &HistoryDay| {
        d.balance
            .numbers()
            .filter(|n| n.unit == unit)
            .map(|n| n.remaining)
    };
    let (min, max) = days
        .iter()
        .filter_map(|d| value(d))
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), v| {
            (min.min(v), max.max(v))
        });
    let mut plot = div()
        .flex()
        .items_end()
        .w_full()
        .h(px(48.))
        .bg(cx.theme().secondary);
    for day in days {
        let mut col = div().flex_1().min_w_0().h_full().relative();
        if let Some(v) = value(day) {
            let position = if min == max {
                0.5
            } else {
                let scale = min.abs().max(max.abs());
                ((v / scale - min / scale) / (max / scale - min / scale)).clamp(0., 1.)
            };
            let color = if matches!(day.balance, DayBalance::Measured(_)) {
                cx.theme().primary
            } else {
                cx.theme().muted_foreground
            };
            col = col.child(
                div()
                    .id(format!("checkin-trend-point-{unit}-{}", day.date))
                    .test_support()
                    .aria_label(day_text(day, cx))
                    .absolute()
                    .bottom(px((position * 43.) as f32))
                    .left_0()
                    .w_full()
                    .h(px(4.))
                    .bg(color),
            );
        }
        plot = plot.child(col);
    }
    div()
        .flex()
        .flex_col()
        .child(format!("{unit} · {min} – {max}"))
        .child(plot)
}
