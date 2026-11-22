use super::*;
use ccr_app::usage::{read::UsageSnapshot, report_clock::ReportClock};
use ccr_usage::RecordedCost;
use chrono::{DateTime, Datelike, Local, NaiveDate};
use gpui_kit::{App, Div, Focusable, FontWeight, component::tooltip::Tooltip};

const MODEL_FIELD: usize = 1;
const PROVIDER_FIELD: usize = 2;
const PROJECT_FIELD: usize = 3;
const START_DATE_FIELD: usize = 4;
const END_DATE_FIELD: usize = 5;

#[derive(Clone, Copy, Default)]
enum Period {
    Week,
    #[default]
    Month,
    Quarter,
    All,
    Custom(NaiveDate, NaiveDate),
}

impl Period {
    fn label(self, cx: &App) -> &'static str {
        match self {
            Self::Week => text(cx, "Last 7 days", "最近 7 天"),
            Self::Month => text(cx, "Last 30 days", "最近 30 天"),
            Self::Quarter => text(cx, "Last 90 days", "最近 90 天"),
            Self::All => text(cx, "All history", "全历史"),
            Self::Custom(..) => text(cx, "Custom dates", "自定义日期"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Panel {
    Dates,
    Custom,
    Sources,
    Filters,
    Sync,
}

#[derive(Default)]
pub(super) struct Controls {
    period: Period,
    selected: QueryFilter,
    panel: Option<Panel>,
    trigger_focus: Option<FocusHandle>,
    invalid_dates: bool,
    providers_supported: Option<bool>,
    invalid_provider: bool,
}

impl Controls {
    pub(super) fn new(timezone: ReportTimezone, source: Option<SourceKind>) -> Self {
        Self {
            selected: QueryFilter {
                source,
                timezone,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub(super) fn filter(&self, mut clock: ReportClock) -> QueryFilter {
        clock.timezone = self.selected.timezone;
        let mut filter = self.selected.clone();
        let dates = match self.period {
            Period::Week => Some(clock.window(7)),
            Period::Month => Some(clock.window(30)),
            Period::Quarter => Some(clock.window(90)),
            Period::All => None,
            Period::Custom(since, until) => Some((since, until)),
        };
        filter.since = dates.map(|(since, _)| since);
        filter.until = dates.map(|(_, until)| until);
        filter.for_usage_page()
    }

    pub(super) fn close_panel(&mut self) {
        self.panel = None;
        self.trigger_focus = None;
        self.invalid_dates = false;
        self.invalid_provider = false;
    }

    pub(super) fn update_capabilities(&mut self, report: &UsageSnapshot) {
        self.providers_supported = Some(report.providers_supported);
    }

    pub(super) fn dismiss_panel(&mut self, window: &mut Window, cx: &mut App) {
        if let Some(focus) = self.trigger_focus.as_ref() {
            window.focus(focus, cx);
        }
        self.close_panel();
    }
}

fn toolbar_row() -> Div {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(px(theme::CONTROL_GAP))
}

impl InsightsView {
    pub(super) fn report_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let controls = &self.usage_controls;
        let mut row = toolbar_row().child(self.panel_button(
            "usage-range-menu",
            controls.period.label(cx),
            Panel::Dates,
            cx,
        ));
        row = if let Some(platform) = self.platform_scope {
            let name = crate::features::workbench::name(platform);
            row.child(
                div()
                    .id("usage-platform")
                    .test_support()
                    .role(gpui_kit::Role::Label)
                    .aria_label(text(cx, "Platform", "平台"))
                    .aria_value(controls.selected.source.map_or("", SourceKind::as_str))
                    .px(px(theme::CONTROL_GAP))
                    .child(name),
            )
        } else {
            row.child(self.panel_button(
                "usage-source",
                import::source_label(controls.selected.source, cx),
                Panel::Sources,
                cx,
            ))
        };
        let local = controls.selected.timezone == ReportTimezone::Local;
        let filters = row
            .child(semantics::button(
                Button::new("insights-timezone")
                    .label(if local {
                        text(cx, "Local", "本地")
                    } else {
                        "UTC"
                    })
                    .ghost()
                    .tooltip(if local {
                        text(
                            cx,
                            "Local report timezone. Switch to UTC",
                            "本地报告时区，切换到 UTC",
                        )
                    } else {
                        text(
                            cx,
                            "UTC report timezone. Switch to Local",
                            "UTC 报告时区，切换到本地",
                        )
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.usage_controls.selected.timezone =
                            match this.usage_controls.selected.timezone {
                                ReportTimezone::Local => ReportTimezone::Utc,
                                ReportTimezone::Utc => ReportTimezone::Local,
                            };
                        this.refresh(cx);
                    })),
                false,
                false,
                cx,
            ))
            .child(self.panel_button(
                "usage-filters",
                text(cx, "Filters", "筛选"),
                Panel::Filters,
                cx,
            ));
        let actions = toolbar_row()
            .child(semantics::ghost_button(
                Button::new("insights-refresh")
                    .label(text(cx, "Refresh report", "刷新报表"))
                    .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                self.loading,
                self.loading,
                cx,
            ))
            .child(self.panel_button(
                "usage-sync-menu",
                text(cx, "Sync usage", "同步用量"),
                Panel::Sync,
                cx,
            ));

        let panel = match controls.panel {
            Some(Panel::Dates) => Some(self.report_periods(cx).into_any_element()),
            Some(Panel::Custom) => Some(self.report_custom_dates(cx).into_any_element()),
            Some(Panel::Sources) => Some(self.report_sources(cx).into_any_element()),
            Some(Panel::Filters) => Some(self.report_advanced_fields(cx).into_any_element()),
            Some(Panel::Sync) => Some(self.import_controls(cx).into_any_element()),
            None => None,
        };
        div()
            .id("usage-toolbar")
            .test_support()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .gap(px(theme::CONTROL_GAP))
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape" && this.usage_controls.panel.is_some() {
                        this.usage_controls.dismiss_panel(window, cx);
                        this.close_import_source_menu();
                        cx.stop_propagation();
                        cx.notify();
                    }
                }),
            )
            .child(
                toolbar_row()
                    .justify_between()
                    .child(filters)
                    .child(actions),
            )
            .children(
                self.has_retained_import()
                    .then(|| self.retained_import_controls(cx)),
            )
            .children(panel.map(|panel| {
                div()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded(px(theme::PANEL_RADIUS))
                    .p(px(theme::CONTROL_GAP))
                    .child(panel)
            }))
            .children(self.applied_filters(cx))
            .children((self.error == Some(InsightsError::Missing.message())).then(|| {
                div()
                    .text_size(crate::theme::text_size(theme::META_SIZE))
                    .child(if self.requests.check_import_target(&self.requests.environment()).is_ok() {
                        text(cx,
                            "To create the usage database, open Sync usage. If llmusage is missing, open Usage → Install llmusage.",
                            "打开「同步用量」可建立用量库。缺少 llmusage 时，请打开「用量」→「安装 llmusage」。")
                    } else {
                        text(cx,
                            "Open Sync usage to check availability in this environment.",
                            "请打开「同步用量」查看当前环境的可用操作。")
                    })
            }))
    }

    fn panel_button(
        &self,
        id: &'static str,
        label: &'static str,
        panel: Panel,
        cx: &mut Context<Self>,
    ) -> Button {
        let open = self.usage_controls.panel == Some(panel)
            || (panel == Panel::Dates && self.usage_controls.panel == Some(Panel::Custom));
        let button = Button::new(id)
            .label(label)
            .toggled(open)
            .on_click(
                cx.listener(move |this, event: &gpui_kit::ClickEvent, window, cx| {
                    this.usage_controls.close_panel();
                    this.close_import_source_menu();
                    if !open {
                        this.usage_controls.panel = Some(panel);
                        if event.is_keyboard() {
                            this.usage_controls.trigger_focus = window.focused(cx);
                        }
                    }
                    cx.notify();
                }),
            );
        semantics::disclosure(semantics::button(button, false, false, cx), open)
    }

    fn report_periods(&self, cx: &mut Context<Self>) -> Div {
        let mut row = toolbar_row();
        for (id, period) in [
            ("usage-range-7", Period::Week),
            ("usage-range-30", Period::Month),
            ("usage-range-90", Period::Quarter),
            ("usage-range-all", Period::All),
        ] {
            row = row.child(semantics::ghost_button(
                Button::new(id)
                    .label(period.label(cx))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.usage_controls.period = period;
                        this.usage_controls.dismiss_panel(window, cx);
                        this.refresh(cx);
                    })),
                false,
                false,
                cx,
            ));
        }
        row.child(semantics::ghost_button(
            Button::new("usage-range-custom")
                .label(text(cx, "Custom dates", "自定义日期"))
                .on_click(
                    cx.listener(|this, event: &gpui_kit::ClickEvent, window, cx| {
                        if let Some(filter) = this.submitted_usage.as_ref() {
                            for (field, date) in [
                                (START_DATE_FIELD, filter.since),
                                (END_DATE_FIELD, filter.until),
                            ] {
                                let value = date.map(|date| date.to_string()).unwrap_or_default();
                                this.fields[field]
                                    .update(cx, |input, cx| input.set_value(value, window, cx));
                            }
                        }
                        this.usage_controls.panel = Some(Panel::Custom);
                        if event.is_keyboard() {
                            let view = cx.weak_entity();
                            window.on_next_frame(move |window, cx| {
                                if let Some(view) = view.upgrade()
                                    && view.read(cx).usage_controls.panel == Some(Panel::Custom)
                                {
                                    let state = &view.read(cx).fields[START_DATE_FIELD];
                                    let focus = state.read(cx).focus_handle(cx);
                                    window.focus(&focus, cx);
                                }
                            });
                        }
                        cx.notify();
                    }),
                ),
            false,
            false,
            cx,
        ))
    }

    fn report_custom_dates(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(
                toolbar_row()
                    .items_end()
                    .child(self.report_input(
                        START_DATE_FIELD,
                        text(cx, "Start • YYYY-MM-DD", "开始 · YYYY-MM-DD"),
                        cx,
                    ))
                    .child(self.report_input(
                        END_DATE_FIELD,
                        text(cx, "End • YYYY-MM-DD", "结束 · YYYY-MM-DD"),
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("usage-apply-dates")
                            .label(text(cx, "Apply dates", "应用日期"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.apply_report_dates(window, cx)
                            })),
                        false,
                        false,
                        cx,
                    )),
            )
            .children(self.usage_controls.invalid_dates.then(|| {
                let message = text(
                    cx,
                    "Enter two valid dates. Start must be on or before end.",
                    "请输入两个有效日期，开始日期须早于或等于结束日期。",
                );
                div()
                    .id("usage-date-error")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(message)
                    .text_color(cx.theme().danger)
                    .child(message)
            }))
    }

    fn apply_report_dates(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let parse = |i: usize| {
            let value = self.fields[i].read(cx).value();
            let value = value.trim();
            let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()?;
            (date.year() > 0 && date.to_string() == value && value.len() == 10).then_some(date)
        };
        match parse(START_DATE_FIELD).zip(parse(END_DATE_FIELD)) {
            Some((since, until)) if since <= until => {
                self.usage_controls.period = Period::Custom(since, until);
                self.usage_controls.dismiss_panel(window, cx);
                self.refresh(cx);
            }
            _ => {
                self.usage_controls.invalid_dates = true;
                cx.notify();
            }
        }
    }

    fn report_sources(&self, cx: &mut Context<Self>) -> Div {
        let mut row = toolbar_row();
        for source in std::iter::once(None).chain(SourceKind::ALL.into_iter().map(Some)) {
            let id = source.map_or("all", SourceKind::as_str);
            row = row.child(semantics::ghost_button(
                Button::new(SharedString::from(format!("usage-source-{id}")))
                    .label(import::source_label(source, cx))
                    .toggled(self.usage_controls.selected.source == source)
                    .on_click(cx.listener(move |this, _, window, cx| {
                        if this.platform_scope.is_none() {
                            this.usage_controls.selected.source = source;
                            this.usage_controls.dismiss_panel(window, cx);
                            this.refresh(cx);
                        }
                    })),
                false,
                false,
                cx,
            ));
        }
        row
    }

    fn report_input(&self, index: usize, label: &'static str, cx: &App) -> Div {
        div()
            .w(px(166.))
            .child(
                div()
                    .text_size(theme::text_size(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(label),
            )
            .child(
                Input::new(&self.fields[index])
                    .id(SharedString::from(format!("insights-filter-{index}")))
                    .aria_label(label),
            )
    }

    fn report_advanced_fields(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(
                toolbar_row()
                    .items_end()
                    .child(self.report_input(MODEL_FIELD, text(cx, "Model", "模型"), cx))
                    .child(self.report_input(PROVIDER_FIELD, text(cx, "Provider", "提供商"), cx))
                    .child(self.report_input(
                        PROJECT_FIELD,
                        text(cx, "Project hash", "项目哈希"),
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("usage-apply-filters")
                            .label(text(cx, "Apply filters", "应用筛选"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.apply_advanced_filters(window, cx)
                            })),
                        false,
                        false,
                        cx,
                    )),
            )
            .children(self.usage_controls.invalid_provider.then(|| {
                let message = text(
                    cx,
                    "Provider filtering is unavailable for this database.",
                    "当前数据库不支持提供商筛选。",
                );
                div()
                    .id("usage-provider-error")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(message)
                    .text_color(cx.theme().danger)
                    .child(message)
            }))
    }

    fn apply_advanced_filters(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let read = |field: usize| {
            let value = self.fields[field].read(cx).value().trim().to_string();
            (!value.is_empty()).then_some(value)
        };
        let provider = read(PROVIDER_FIELD);
        if provider.is_some() && self.usage_controls.providers_supported == Some(false) {
            self.usage_controls.invalid_provider = true;
            cx.notify();
            return;
        }
        self.usage_controls.selected.model = read(MODEL_FIELD);
        self.usage_controls.selected.provider = provider;
        self.usage_controls.selected.project_hash = read(PROJECT_FIELD);
        self.usage_controls.dismiss_panel(window, cx);
        self.refresh(cx);
    }

    fn applied_filters(&self, cx: &mut Context<Self>) -> Option<Div> {
        let filter = &self.usage_controls.selected;
        if filter.model.is_none() && filter.provider.is_none() && filter.project_hash.is_none() {
            return None;
        }
        let mut chips = toolbar_row();
        for (id, label, value) in [
            (
                "usage-filter-chip-model",
                text(cx, "Model", "模型"),
                &filter.model,
            ),
            (
                "usage-filter-chip-provider",
                text(cx, "Provider", "提供商"),
                &filter.provider,
            ),
            (
                "usage-filter-chip-project",
                text(cx, "Project", "项目"),
                &filter.project_hash,
            ),
        ] {
            if let Some(value) = value {
                chips = chips.child(
                    div()
                        .id(id)
                        .test_support()
                        .role(gpui_kit::Role::Label)
                        .aria_label(label)
                        .aria_value(value.clone())
                        .max_w_full()
                        .truncate()
                        .px(px(theme::CONTROL_GAP))
                        .py(px(3.))
                        .rounded(px(theme::PILL_RADIUS))
                        .bg(cx.theme().muted)
                        .child(format!("{label} · {value}")),
                );
            }
        }
        Some(
            chips.child(semantics::ghost_button(
                Button::new("usage-clear-filters")
                    .label(text(cx, "Clear filters", "清除筛选"))
                    .on_click(cx.listener(|this, _, window, cx| {
                        for field in [MODEL_FIELD, PROVIDER_FIELD, PROJECT_FIELD] {
                            this.fields[field]
                                .update(cx, |input, cx| input.set_value("", window, cx));
                        }
                        this.usage_controls.selected.model = None;
                        this.usage_controls.selected.provider = None;
                        this.usage_controls.selected.project_hash = None;
                        this.refresh(cx);
                    })),
                false,
                false,
                cx,
            )),
        )
    }
}

pub(super) fn heading(cx: &App) -> impl IntoElement {
    let title = text(cx, "Usage", "用量");
    div()
        .id("usage-heading")
        .test_support()
        .role(gpui_kit::Role::Heading)
        .aria_level(1)
        .aria_label(title)
        .text_size(theme::text_size(theme::TITLE_SIZE))
        .font_weight(FontWeight::SEMIBOLD)
        .child(title)
}

pub(super) fn context_row(
    filter: &QueryFilter,
    report: Option<&UsageSnapshot>,
    cx: &App,
) -> impl IntoElement {
    let range = if let Some((since, until)) = filter.since.zip(filter.until) {
        format!("{since} – {until}")
    } else {
        let all = text(cx, "All history", "全历史");
        report
            .and_then(|report| report.trends.first().zip(report.trends.last()))
            .map_or_else(
                || all.to_string(),
                |(first, last)| format!("{all} · {} – {}", first.date, last.date),
            )
    };
    let zone = match filter.timezone {
        ReportTimezone::Local => text(cx, "Local", "本地"),
        ReportTimezone::Utc => "UTC",
    };
    let range = format!("{range} · {zone}");
    let context = toolbar_row()
        .flex_shrink_0()
        .text_size(theme::text_size(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground)
        .gap(px(theme::SECTION_GAP))
        .child(
            div()
                .id("usage-date-range")
                .test_support()
                .role(gpui_kit::Role::Label)
                .aria_label(range.clone())
                .aria_value(range.clone())
                .child(range),
        );
    context.children(report.map(|report| {
        let sync = report
            .overview
            .last_sync_at
            .as_deref()
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|instant| match filter.timezone {
                ReportTimezone::Local => instant.with_timezone(&Local).naive_local(),
                ReportTimezone::Utc => instant.naive_utc(),
            })
            .map(|date| date.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| text(cx, "not recorded", "未记录").into());
        let prefix = text(
            cx,
            "Last successful sync (any source)",
            "上次成功同步（任意来源）",
        );
        let label = format!("{prefix} · {sync}");
        div()
            .id("usage-last-sync")
            .test_support()
            .role(gpui_kit::Role::Label)
            .aria_label(label.clone())
            .aria_value(label.clone())
            .child(label)
    }))
}

pub(super) fn readings(report: Option<&UsageSnapshot>, narrow: bool, cx: &App) -> impl IntoElement {
    let cost = text(cx, "Recorded cost (USD)", "记录费用（USD）");
    let estimate = text(cx, "llmusage estimate", "llmusage 估算");
    let unknown = text(cx, "Estimate unavailable", "估算不可用");
    let partial = text(cx, "Partial estimate", "部分记录可估算");
    let mixed = text(cx, "Mixed pricing coverage", "估算覆盖范围待确认");
    let (amount, note) =
        match report.map(|report| report.recorded_cost.unwrap_or(RecordedCost::Unavailable)) {
            None => (None, text(cx, "Reading", "正在读取")),
            Some(RecordedCost::Unavailable) => (None, unknown),
            Some(RecordedCost::Partial(amount)) => (Some(amount), partial),
            Some(RecordedCost::Mixed(amount)) => (Some(amount), mixed),
            Some(RecordedCost::Known(amount)) => (Some(amount), estimate),
        };
    let (short_cost, exact_cost) = amount.map_or_else(
        || ("—".to_string(), note.to_string()),
        |amount| {
            let exact = amount.to_string();
            let cents = format!("${amount:.2}");
            let exact = if exact
                .split_once('.')
                .is_some_and(|(_, decimals)| decimals.len() > 2)
            {
                format!("${exact}")
            } else {
                cents.clone()
            };
            let short = crate::features::workbench::format_recorded_cost(amount);
            let exact = if note == partial || note == mixed {
                format!("{exact} · {note}")
            } else {
                exact
            };
            (short, exact)
        },
    );
    let active_days = report.map(|report| {
        report
            .trends
            .iter()
            .filter(|day| day.request_count > 0)
            .count() as i64
    });
    let [cost_cell, tokens, requests, days] = [
        reading("usage-cost", cost, short_cost, exact_cost, Some(note), cx),
        count_reading(
            "usage-tokens",
            text(cx, "Tokens", "Token"),
            report.map(|report| report.overview.total.total_tokens),
            cx,
        ),
        count_reading(
            "usage-requests",
            text(cx, "Requests", "请求"),
            report.map(|report| report.overview.total_events),
            cx,
        ),
        count_reading(
            "usage-active-days",
            text(cx, "Active days", "活跃天数"),
            active_days,
            cx,
        ),
    ];
    let panel = div()
        .id("usage-readings")
        .test_support()
        .flex()
        .flex_shrink_0()
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .rounded(px(theme::PANEL_RADIUS));
    if narrow {
        panel
            .flex_col()
            .child(reading_row([cost_cell, tokens], cx))
            .child(div().h(px(1.)).bg(cx.theme().border))
            .child(reading_row([requests, days], cx))
    } else {
        panel.child(reading_row([cost_cell, tokens, requests, days], cx))
    }
}

fn reading_row<const N: usize>(cells: [Div; N], cx: &App) -> Div {
    let mut row = div().flex().w_full();
    for (index, cell) in cells.into_iter().enumerate() {
        row = row.child(if index == 0 {
            cell
        } else {
            cell.border_l_1().border_color(cx.theme().border)
        });
    }
    row
}

fn count_reading(id: &'static str, label: &'static str, value: Option<i64>, cx: &App) -> Div {
    let (short, exact) = value.map_or_else(
        || ("—".into(), text(cx, "Reading", "正在读取").into()),
        |value| {
            (
                crate::features::overview::compact_count(value),
                value.to_string(),
            )
        },
    );
    reading(id, label, short, exact, None, cx)
}

fn reading(
    id: &'static str,
    label: &'static str,
    value: String,
    exact: String,
    note: Option<&'static str>,
    cx: &App,
) -> Div {
    let primary = note.is_some();
    let description = format!("{label} · {exact}");
    let number = div()
        .id(id)
        .test_support()
        .role(gpui_kit::Role::Label)
        .aria_label(description.clone())
        .aria_value(description.clone())
        .font_features(theme::tabular_figures())
        .font_weight(if primary {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::MEDIUM
        })
        .text_size(theme::text_size(if primary {
            theme::KPI_PRIMARY_SIZE
        } else {
            theme::KPI_SIZE
        }))
        .line_height(gpui_kit::relative(1.2))
        .tooltip(move |window, cx| Tooltip::new(description.clone()).build(window, cx))
        .child(value);
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(4.))
        .p(px(theme::SECTION_GAP))
        .child(
            div()
                .text_size(theme::text_size(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(label),
        )
        .child(number)
        .children(note.map(|note| {
            div()
                .id("usage-cost-state")
                .test_support()
                .role(gpui_kit::Role::Label)
                .aria_label(note)
                .text_size(theme::text_size(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(note)
        }))
}
