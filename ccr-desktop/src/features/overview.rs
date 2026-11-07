//! Home workbench. Numbers come from the captured usage database and session archive.
use crate::{
    locales::{is_chinese, service_error, text},
    runtime::insights::{InsightsRequests, SessionScope, WorkbenchDay, WorkbenchReport},
    shell::semantics,
    theme,
};
use ccr_app::runtime::{
    AdmissionError, Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent,
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{ActiveTheme, Icon, IconName, button::Button, tooltip::Tooltip};
use gpui_kit::{
    Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Task, Window, div, prelude::FluentBuilder, px,
};

const MISSING: &str = "No local usage source is configured. / 尚未配置本机用量来源。";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OverviewLink {
    Usage,
    Commands,
    Install,
    Sessions,
}

pub struct OverviewView {
    requests: InsightsRequests,
    fixture: bool,
    days: u16,
    report: Option<std::sync::Arc<WorkbenchReport>>,
    loading: bool,
    error: Option<&'static str>,
    generation: u64,
    gate: ResponseGate,
    task: Option<Task<()>>,
    pending: Option<OverviewLink>,
    focus: FocusHandle,
}

impl OverviewView {
    pub fn new(services: crate::runtime::Services, cx: &mut Context<Self>) -> Self {
        let requests = InsightsRequests::new(&services);
        Self {
            fixture: services.is_fixture(),
            days: 7,
            error: (!requests.configured()).then_some(MISSING),
            requests,
            report: None,
            loading: false,
            generation: 0,
            gate: ResponseGate::default(),
            task: None,
            pending: None,
            focus: cx.focus_handle(),
        }
    }

    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        window.focus_next(cx);
    }

    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.gate.clear();
        self.generation += 1;
        self.loading = false;
        self.pending = None;
        cx.notify();
    }

    pub fn requests_configured(&self) -> bool {
        self.requests.configured()
    }

    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    pub fn take_pending(&mut self) -> Option<OverviewLink> {
        self.pending.take()
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.gate.clear();
        self.generation += 1;
        self.error = None;
        if !self.requests.configured() {
            self.loading = false;
            self.report = None;
            self.error = Some(MISSING);
            cx.notify();
            return;
        }
        let key = RequestKey {
            environment: EnvironmentId::Local("overview".into()),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        };
        self.gate.replace(key.clone());
        self.loading = true;
        let days = self.days;
        match self.requests.load_workbench(key.clone(), days) {
            Ok(mut subscription) => {
                self.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.gate.accepts(&key) {
                                    return;
                                }
                                this.loading = false;
                                match result.as_ref() {
                                    Completion::Success(report) => {
                                        this.report = Some(report.clone());
                                        this.error = None;
                                    }
                                    Completion::Failed(error)
                                    | Completion::CleanupFailed(error) => {
                                        this.error = Some(error.message());
                                    }
                                    Completion::Cancelled => {
                                        this.error = Some("Read cancelled / 已取消读取");
                                    }
                                    Completion::WorkerFailed => {
                                        this.error = Some(
                                            "Read worker stopped. Retry. / 读取线程已停止，请重试",
                                        );
                                    }
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }));
            }
            Err(error) => {
                self.loading = false;
                self.error = Some(match error {
                    AdmissionError::Full => "Read queue busy. Retry. / 读取队列繁忙，请重试",
                    AdmissionError::Stopped => "Application is shutting down / 应用正在退出",
                });
            }
        }
        cx.notify();
    }

    fn set_days(&mut self, days: u16, cx: &mut Context<Self>) {
        if self.days == days {
            return;
        }
        self.days = days;
        self.report = None;
        self.refresh(cx);
    }

    fn open(&mut self, link: OverviewLink, cx: &mut Context<Self>) {
        self.pending = Some(link);
        cx.notify();
    }

    fn status_text(&self, cx: &mut Context<Self>) -> String {
        let state = if self.loading {
            Some(text(cx, "Reading local usage…", "正在读取本机用量…"))
        } else {
            self.error.map(|error| service_error(cx, error))
        };
        let Some(report) = self.report.as_ref() else {
            return state
                .unwrap_or_else(|| text(cx, "Usage has not been read yet.", "尚未读取用量。"))
                .into();
        };
        let zone = if self.requests.report_clock().timezone == ccr_usage::ReportTimezone::Utc {
            " UTC"
        } else {
            ""
        };
        let window = if is_chinese(cx) {
            format!(
                "本机用量 · 截至 {}{zone} 的 {} 日",
                report.as_of, report.window_days
            )
        } else {
            format!(
                "Local usage · {} days ending {}{zone}",
                report.window_days, report.as_of
            )
        };
        match state {
            Some(message) => format!(
                "{message} · {} {window}",
                text(cx, "Last report:", "上次报告：")
            ),
            None => window,
        }
    }
}

impl Render for OverviewView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let report = self.report.as_deref();
        let stacked = window.viewport_size().width < px(theme::NARROW_WIDTH);
        let mut root = div()
            .id("overview-root")
            .track_focus(&self.focus)
            .tab_stop(false)
            .size_full()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .p(px(theme::CONTENT_PADDING))
            .gap(px(theme::SECTION_GAP))
            .child(self.header(cx));
        if let Some(report) = report {
            root = root.child(self.platforms(report, cx));
        }
        root.child(
            div()
                .flex()
                .flex_shrink_0()
                .items_start()
                .gap(px(theme::SECTION_GAP))
                .when(stacked, |row| row.flex_col())
                .child(self.usage_panel(report, stacked, cx))
                .child(self.actions(stacked, cx)),
        )
    }
}

impl OverviewView {
    fn header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.status_text(cx);
        let ink = if self.error.is_some() {
            cx.theme().warning
        } else {
            cx.theme().muted_foreground
        };
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(theme::DISPLAY_SIZE))
                            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                            .child(text(cx, "Overview", "运行概览")),
                    )
                    .child(semantics::button(
                        Button::new("overview-refresh")
                            .label(text(cx, "Refresh", "刷新用量"))
                            .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                        self.loading || !self.requests.configured(),
                        self.loading,
                        cx,
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        div()
                            .w(px(7.))
                            .h(px(7.))
                            .rounded_full()
                            .bg(if self.error.is_some() {
                                ink
                            } else if self.report.is_some() && !self.loading {
                                cx.theme().primary
                            } else {
                                cx.theme().border
                            }),
                    )
                    .child(
                        div()
                            .id("overview-status")
                            .test_support()
                            .role(gpui_kit::Role::Status)
                            .aria_label(status.clone())
                            .text_size(px(theme::META_SIZE))
                            .text_color(ink)
                            .child(status),
                    )
                    .when(self.fixture, |row| {
                        row.child(
                            div()
                                .text_size(px(theme::META_SIZE))
                                .text_color(cx.theme().warning)
                                .child(text(cx, "Isolated demo", "隔离演示")),
                        )
                    }),
            )
    }

    fn platforms(&self, report: &WorkbenchReport, cx: &mut Context<Self>) -> impl IntoElement {
        let mut band = div()
            .w_full()
            .flex_shrink_0()
            .flex()
            .flex_wrap()
            .bg(cx.theme().background)
            .rounded(px(theme::INPUT_RADIUS))
            .border_1()
            .border_color(cx.theme().border)
            .overflow_hidden();
        for (index, platform) in report.platforms.iter().enumerate() {
            let ink = theme::platform_color(&platform.id, cx.theme().mode);
            band = band.child(
                div()
                    .min_w(px(158.))
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(theme::CONTROL_GAP))
                    .p(px(theme::SECTION_GAP))
                    .when(index > 0, |cell| {
                        cell.border_l_1().border_color(cx.theme().border)
                    })
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(theme::CONTROL_GAP))
                            .child(div().w(px(7.)).h(px(7.)).rounded_full().bg(ink))
                            .child(
                                div()
                                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                    .child(platform_title(&platform.id)),
                            ),
                    )
                    .child(sparkline(
                        &platform.requests_by_day,
                        platform.requests,
                        ink,
                        cx,
                    ))
                    .child(
                        div()
                            .flex()
                            .gap(px(theme::CONTROL_GAP))
                            .child(count_metric(
                                format!("{}-requests", platform.id),
                                text(cx, "Requests", "请求"),
                                platform.requests,
                                theme::BODY_SIZE,
                                cx,
                            ))
                            .child(count_metric(
                                format!("{}-tokens", platform.id),
                                text(cx, "Tokens", "Token"),
                                platform.tokens,
                                theme::BODY_SIZE,
                                cx,
                            )),
                    ),
            );
        }
        band
    }

    fn windows(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut group = div()
            .flex()
            .items_center()
            .p(px(2.))
            .rounded(px(theme::INPUT_RADIUS))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().sidebar);
        for (id, days, label) in [
            ("overview-window-7", 7_u16, "7D"),
            ("overview-window-30", 30, "30D"),
            ("overview-window-90", 90, "90D"),
        ] {
            let selected = self.days == days;
            let colors = cx.theme();
            group = group.child(
                gpui_kit::base::Button::new(id)
                    .accessibility_label(label)
                    .aria_toggled(if selected {
                        gpui_kit::accesskit::Toggled::True
                    } else {
                        gpui_kit::accesskit::Toggled::False
                    })
                    .disabled(!self.requests.configured())
                    .h(px(28.))
                    .w(px(42.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(theme::SELECTION_RADIUS))
                    .border_2()
                    .border_color(gpui_kit::transparent_black())
                    .bg(if selected {
                        colors.background
                    } else {
                        gpui_kit::transparent_black()
                    })
                    .text_color(if selected {
                        colors.primary
                    } else {
                        colors.foreground
                    })
                    .when(selected, |el| {
                        el.font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    })
                    .hover(|style| style.bg(colors.secondary))
                    .focus_visible(|style| style.border_color(colors.ring))
                    .child(label)
                    .on_click(cx.listener(move |this, _, _, cx| this.set_days(days, cx))),
            );
        }
        group
    }

    fn usage_panel(
        &self,
        report: Option<&WorkbenchReport>,
        stacked: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut panel = div()
            .flex()
            .flex_col()
            .min_w_0()
            .when(stacked, |el| el.w_full())
            .when(!stacked, |el| el.flex_1())
            .gap(px(theme::SECTION_GAP))
            .p(px(theme::SECTION_GAP))
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .rounded(px(theme::INPUT_RADIUS))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_between()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .child(heading(text(cx, "Usage and cost", "用量与成本")))
                    .child(self.windows(cx)),
            );
        let Some(report) = report else {
            return panel.child(div().text_color(cx.theme().muted_foreground).child(
                if self.loading {
                    text(cx, "Reading the selected period…", "正在读取所选时段…")
                } else {
                    text(cx, "No report available", "暂无可用报告")
                },
            ));
        };
        let session_title = match report.session_scope {
            SessionScope::Window => text(cx, "Sessions · 7D", "会话 · 7 日"),
            SessionScope::AllHistory => text(cx, "Sessions · all time", "会话 · 全历史"),
            SessionScope::Unindexed | SessionScope::Unavailable => text(cx, "Sessions", "会话"),
        };
        let session = match (report.session_scope, report.session_count) {
            (SessionScope::Window | SessionScope::AllHistory, Some(count)) => count_metric(
                "total-sessions".into(),
                session_title,
                count,
                theme::TITLE_SIZE,
                cx,
            ),
            (SessionScope::Unindexed, _) => metric(
                "total-sessions".into(),
                session_title,
                text(cx, "Unindexed", "未索引").into(),
                text(cx, "Archive not indexed", "归档未索引").into(),
                theme::BODY_SIZE,
                cx,
            ),
            _ => metric(
                "total-sessions".into(),
                session_title,
                text(cx, "Unavailable", "不可用").into(),
                text(cx, "Session count unavailable", "会话数不可用").into(),
                theme::BODY_SIZE,
                cx,
            ),
        };
        let cost = report
            .cost_usd
            .map(format_usd)
            .unwrap_or_else(|| "–".into());
        panel = panel.child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(theme::CONTROL_GAP))
                .child(count_metric(
                    "total-requests".into(),
                    text(cx, "Requests", "请求"),
                    report.requests,
                    theme::TITLE_SIZE,
                    cx,
                ))
                .child(count_metric(
                    "total-tokens".into(),
                    text(cx, "Tokens", "Token"),
                    report.tokens,
                    theme::TITLE_SIZE,
                    cx,
                ))
                .child(metric(
                    "total-cost".into(),
                    text(cx, "Cost", "成本"),
                    cost,
                    report
                        .cost_usd
                        .map(|v| format!("USD {v}"))
                        .unwrap_or_else(|| text(cx, "Cost unavailable", "成本不可用").into()),
                    theme::TITLE_SIZE,
                    cx,
                ))
                .child(session),
        );
        panel.child(self.chart(report, cx))
    }

    fn chart(&self, report: &WorkbenchReport, cx: &mut Context<Self>) -> impl IntoElement {
        let max = report
            .days
            .iter()
            .map(|d| d.claude + d.codex + d.antigravity + d.opencode)
            .max()
            .unwrap_or(0);
        let mut chart = div()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .border_t_1()
            .border_color(cx.theme().border)
            .pt(px(theme::CONTROL_GAP))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .justify_between()
                    .gap(px(theme::CONTROL_GAP))
                    .child(text(cx, "Daily requests", "逐日请求"))
                    .child(legend(report, cx)),
            );
        if max > 0 {
            let mut bars = div()
                .relative()
                .flex()
                .items_end()
                .h(px(CHART_HEIGHT))
                .border_b_1()
                .border_color(cx.theme().border)
                .gap(px(if report.days.len() > 30 { 1. } else { 3. }));
            for fraction in [0.0, 0.5] {
                bars = bars.child(
                    div()
                        .absolute()
                        .top(px(CHART_HEIGHT * fraction))
                        .w_full()
                        .h(px(1.))
                        .bg(cx.theme().border),
                );
            }
            for day in &report.days {
                bars = bars.child(day_stack(day, max, cx));
            }
            // Match the plot's equal-width day slots. Long windows label the
            // midpoint of each bucket so the outer dates fit inside the plot.
            let mut ticks = div()
                .flex()
                .gap(px(if report.days.len() > 30 { 1. } else { 3. }))
                .text_size(px(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground);
            let count = report.days.len();
            let buckets = if count <= 7 { count } else { 6 };
            for (index, day) in report.days.iter().enumerate() {
                let marked = (0..buckets).any(|i| (2 * i + 1) * count / (2 * buckets) == index);
                ticks = ticks.child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .justify_center()
                        .when(marked, |slot| {
                            slot.child(div().flex_shrink_0().child(day.label.clone()))
                        }),
                );
            }
            chart = chart.child(bars).child(ticks);
        } else {
            chart = chart.child(
                div()
                    .py(px(theme::SECTION_GAP))
                    .text_color(cx.theme().muted_foreground)
                    .child(if report.requests == 0 {
                        text(cx, "No requests in this period.", "此时段没有请求。")
                    } else {
                        text(
                            cx,
                            "No daily breakdown available.",
                            "没有可绘制的逐日明细。",
                        )
                    }),
            );
        }
        chart.when(report.other_requests > 0, |el| {
            el.child(
                div()
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "{} {}",
                        grouped(report.other_requests),
                        text(
                            cx,
                            "requests included in the total have no bar in this chart.",
                            "条请求计入总数，未画入此图。"
                        )
                    )),
            )
        })
    }

    fn actions(&self, stacked: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let mut rail = div()
            .when(stacked, |el| el.w_full())
            .when(!stacked, |el| el.w(px(222.)).flex_shrink_0())
            .flex()
            .flex_col()
            .p(px(theme::SECTION_GAP))
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .rounded(px(theme::INPUT_RADIUS))
            .child(heading(text(cx, "Next actions", "下一步")));
        for (id, label, link, icon) in [
            (
                "overview-open-usage",
                text(cx, "View usage report", "查看用量报告"),
                OverviewLink::Usage,
                IconName::ChartPie,
            ),
            (
                "overview-open-commands",
                text(cx, "Run a command", "运行命令"),
                OverviewLink::Commands,
                IconName::Play,
            ),
            (
                "overview-open-sessions",
                text(cx, "Browse sessions", "浏览会话"),
                OverviewLink::Sessions,
                IconName::BookOpen,
            ),
            (
                "overview-open-install",
                text(cx, "Install llmusage", "安装 llmusage"),
                OverviewLink::Install,
                IconName::SquareTerminal,
            ),
        ] {
            rail = rail.child(
                gpui_kit::base::Button::new(id)
                    .accessibility_label(label)
                    .mt(px(theme::CONTROL_GAP))
                    .w_full()
                    .px(px(theme::CONTROL_GAP))
                    .h(px(theme::ROW_HEIGHT))
                    .rounded(px(theme::SELECTION_RADIUS))
                    .border_1()
                    .border_color(cx.theme().border)
                    .hover(|style| style.bg(cx.theme().sidebar))
                    .focus_visible(|style| style.border_color(cx.theme().ring))
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .items_center()
                            .gap(px(theme::CONTROL_GAP))
                            .child(
                                Icon::new(icon)
                                    .size(px(16.))
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(div().flex_1().child(label))
                            .child("›"),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| this.open(link, cx))),
            );
        }
        rail
    }
}

fn heading(label: &'static str) -> gpui_kit::Div {
    div()
        .text_size(px(theme::SECTION_TITLE_SIZE))
        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
        .child(label)
}

fn count_metric(
    id: String,
    label: &'static str,
    value: i64,
    size: f32,
    cx: &Context<OverviewView>,
) -> gpui_kit::Div {
    metric(id, label, compact_count(value), grouped(value), size, cx)
}

fn metric(
    id: String,
    label: &'static str,
    short: String,
    exact: String,
    size: f32,
    cx: &Context<OverviewView>,
) -> gpui_kit::Div {
    let description = format!("{label}: {exact}");
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(2.))
        .child(
            div()
                .text_size(px(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(label),
        )
        .child(
            div()
                .id(SharedString::from(format!("overview-{id}")))
                .test_support()
                .role(gpui_kit::Role::Label)
                .aria_label(description.clone())
                .aria_value(description.clone())
                .font_family(cx.theme().mono_font_family.clone())
                .font_weight(gpui_kit::FontWeight::MEDIUM)
                .text_size(px(size))
                .child(short)
                .tooltip(move |window, cx| Tooltip::new(description.clone()).build(window, cx)),
        )
}

fn compact_count(value: i64) -> String {
    let n = value.unsigned_abs();
    for (threshold, suffix) in [
        (1_000_000_000_000, "T"),
        (1_000_000_000, "B"),
        (1_000_000, "M"),
        (1_000, "K"),
    ] {
        if n >= threshold {
            return format!("{:.1}{suffix}", value as f64 / threshold as f64);
        }
    }
    value.to_string()
}

fn platform_title(id: &str) -> String {
    match id {
        "claude" => "Claude Code",
        "codex" => "Codex",
        "antigravity" => "Antigravity",
        "opencode" => "OpenCode",
        "grok" => "Grok",
        "kimi_code" => "Kimi",
        "pi" => "Pi",
        "zcode" => "Zcode",
        "deepseek_harness" => "DeepSeek",
        other => other,
    }
    .to_string()
}

fn grouped(value: i64) -> String {
    let digits = value.unsigned_abs().to_string();
    let mut reversed = String::new();
    for (index, ch) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            reversed.push(',');
        }
        reversed.push(ch);
    }
    let mut text: String = reversed.chars().rev().collect();
    if value < 0 {
        text.insert(0, '-');
    }
    text
}

fn format_usd(value: f64) -> String {
    if value.abs() >= 100.0 {
        format!("${value:.2}")
    } else {
        format!("${value:.4}")
    }
}

fn sparkline(
    values: &[i64],
    requests: i64,
    ink: gpui_kit::Hsla,
    cx: &Context<OverviewView>,
) -> impl IntoElement {
    let max = values.iter().copied().max().unwrap_or(0);
    let mut row = div()
        .h(px(34.))
        .flex()
        .min_w_0()
        .items_end()
        .border_b_1()
        .border_color(cx.theme().border);
    if max == 0 {
        return row.child(
            div()
                .pb(px(3.))
                .text_size(px(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(if values.is_empty() && requests > 0 {
                    text(cx, "Daily split unavailable", "无逐日明细")
                } else {
                    text(cx, "No requests", "无请求")
                }),
        );
    }
    row = row.gap(px(if values.len() > 30 { 0. } else { 1. }));
    for value in values {
        let height = if *value == 0 {
            0.
        } else {
            (*value as f32 / max as f32 * 32.).max(1.)
        };
        row = row.child(div().flex_1().min_w_0().h(px(height)).bg(ink));
    }
    row
}

const CHART_HEIGHT: f32 = 144.;
const SERIES: [&str; 4] = ["claude", "codex", "antigravity", "opencode"];

fn day_values(day: &WorkbenchDay) -> [i64; 4] {
    [day.claude, day.codex, day.antigravity, day.opencode]
}

fn day_stack(day: &WorkbenchDay, max: i64, cx: &Context<OverviewView>) -> impl IntoElement {
    let mut column = div()
        .id(SharedString::from(format!("overview-day-{}", day.label)))
        .flex_1()
        .min_w_0()
        .h(px(CHART_HEIGHT))
        .flex()
        .flex_col()
        .justify_end();
    let values = day_values(day);
    let detail = format!(
        "{} · {} {}",
        day.label,
        grouped(values.iter().sum()),
        text(cx, "requests", "请求")
    );
    for (id, value) in SERIES.into_iter().zip(values).rev() {
        if value > 0 {
            column = column.child(
                div()
                    .w_full()
                    .h(px((value as f32 / max as f32 * CHART_HEIGHT).max(1.)))
                    .bg(theme::platform_color(id, cx.theme().mode)),
            );
        }
    }
    column.tooltip(move |window, cx| Tooltip::new(detail.clone()).build(window, cx))
}

fn legend(report: &WorkbenchReport, cx: &Context<OverviewView>) -> impl IntoElement {
    let mut row = div()
        .flex()
        .flex_wrap()
        .gap(px(theme::CONTROL_GAP))
        .text_size(px(theme::META_SIZE));
    for (index, id) in SERIES.into_iter().enumerate() {
        if report.days.iter().any(|d| day_values(d)[index] > 0) {
            row = row.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .child(
                        div()
                            .w(px(8.))
                            .h(px(8.))
                            .rounded(px(2.))
                            .bg(theme::platform_color(id, cx.theme().mode)),
                    )
                    .child(platform_title(id)),
            );
        }
    }
    row
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{
        ConfigSource, Services,
        insights::{InsightsPaths, WorkbenchPlatform},
    };
    use ccr_app::runtime::RuntimeOwner;
    use gpui_kit::{
        AppContext, TestAppContext, WindowOptions,
        test::{TestAppContextExt, TestWindowExt},
    };
    use std::{sync::Arc, time::Duration};

    #[gpui_kit::test]
    async fn overview_ranges_hide_stale_totals_and_preserve_exact_metrics(cx: &mut TestAppContext) {
        cx.executor().allow_parking();
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let owner = RuntimeOwner::new().unwrap_or_else(|e| panic!("runtime: {e}"));
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("config.toml"),
                delay: Duration::ZERO,
            },
        )
        .with_insights_paths(InsightsPaths {
            usage_root: root.path().join("missing-usage"),
            archive_db: root.path().join("missing-archive"),
            provider_home: root.path().join("home"),
        });
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |_, cx| {
                cx.new(|cx| OverviewView::new(services, cx))
            })
            .unwrap_or_else(|e| panic!("overview window: {e}"))
        });
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |this, cx| {
                this.report = Some(Arc::new(WorkbenchReport {
                    as_of: "2026-11-06".into(),
                    window_days: 7,
                    requests: 1002,
                    tokens: 338926579,
                    cost_usd: None,
                    sessions_indexed: true,
                    session_count: Some(3),
                    session_scope: SessionScope::Window,
                    other_requests: 1000,
                    platforms: vec![WorkbenchPlatform {
                        id: "codex".into(),
                        requests: 2,
                        tokens: 10,
                        requests_by_day: vec![2],
                    }],
                    days: vec![WorkbenchDay {
                        label: "11-06".into(),
                        claude: 0,
                        codex: 2,
                        antigravity: 0,
                        opencode: 0,
                    }],
                }));
                cx.notify();
            });
            window.render_frame(cx);
            assert_eq!(
                window.find("overview-total-tokens").value(),
                Some("Tokens: 338,926,579")
            );
            assert_eq!(
                window.find("overview-codex-requests").value(),
                Some("Requests: 2")
            );
            assert_eq!(
                window.find("overview-total-sessions").value(),
                Some("Sessions · 7D: 3")
            );
            assert_eq!(
                window.find("overview-total-cost").value(),
                Some("Cost: Cost unavailable")
            );
            for (scope, description) in [
                (SessionScope::Unindexed, "Sessions: Archive not indexed"),
                (
                    SessionScope::Unavailable,
                    "Sessions: Session count unavailable",
                ),
            ] {
                view.update(cx, |this, cx| {
                    let Some(report) = this.report.as_mut() else {
                        panic!("expected the seeded overview report");
                    };
                    let report = Arc::make_mut(report);
                    report.session_scope = scope;
                    report.session_count = None;
                    report.sessions_indexed = false;
                    cx.notify();
                });
                window.render_frame(cx);
                assert_eq!(
                    window.find("overview-total-sessions").value(),
                    Some(description)
                );
            }
            for (id, link) in [
                ("overview-open-usage", OverviewLink::Usage),
                ("overview-open-sessions", OverviewLink::Sessions),
                ("overview-open-commands", OverviewLink::Commands),
                ("overview-open-install", OverviewLink::Install),
            ] {
                window.click(id, cx);
                assert!(view.update(cx, |v, _| v.take_pending()) == Some(link));
            }
            window.click("overview-window-30", cx);
            assert_eq!(window.find("overview-window-30").checked(), Some(true));
            assert!(window.try_find("overview-total-tokens").is_none());
            let old = RequestKey {
                environment: EnvironmentId::Local("overview".into()),
                platform: Platform::Claude,
                generation: view.read(cx).generation,
                request_id: view.read(cx).generation,
            };
            window.click("overview-window-90", cx);
            assert!(!view.read(cx).gate.accepts(&old));
            assert_eq!(window.find("overview-window-90").checked(), Some(true));
        })
        .unwrap_or_else(|e| panic!("overview interaction: {e}"));
        cx.wait_for(handle, Duration::from_secs(5), |_, cx| {
            view.read(cx).error.is_some()
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            assert!(window.try_find("overview-total-tokens").is_none());
            assert!(
                window
                    .find("overview-status")
                    .label()
                    .is_some_and(|s| !s.is_empty())
            );
            window.remove_window();
        })
        .unwrap_or_else(|e| panic!("overview failed read: {e}"));
        owner.shutdown();
    }
}
