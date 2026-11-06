//! Home workbench. Numbers come from the captured usage database and session archive.
use crate::{
    locales::{is_chinese, service_error, text},
    runtime::insights::{InsightsRequests, SessionScope, WorkbenchReport},
    shell::semantics,
    theme,
};
use ccr_app::runtime::{
    AdmissionError, Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent,
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, Selectable,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Task, Window, div, prelude::FluentBuilder, px, rgb,
};

const MISSING: &str = "No usage database and session archive were found on this computer. This page does not create them or fill in sample numbers. / 本机没有找到用量库和会话归档。此页不会创建它们，也不会填入示例数字。";

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
        self.refresh(cx);
    }

    fn open(&mut self, link: OverviewLink, cx: &mut Context<Self>) {
        self.pending = Some(link);
        cx.notify();
    }

    fn status_text(&self, cx: &mut Context<Self>) -> String {
        if self.loading {
            return text(cx, "Reading usage on this computer.", "正在读取本机用量。").to_string();
        }
        if let Some(error) = self.error {
            return service_error(cx, error).to_string();
        }
        if let Some(report) = &self.report {
            let report_date = format!(
                "{}{}",
                report.as_of,
                if self.requests.report_clock().timezone == ccr_usage::ReportTimezone::Utc {
                    " UTC"
                } else {
                    ""
                }
            );
            return if is_chinese(cx) {
                format!("已读取本机用量，报告日 {}", report_date)
            } else {
                format!("Local usage read. Report date {}", report_date)
            };
        }
        text(cx, "Usage has not been read yet.", "尚未读取用量。").to_string()
    }

    fn attention(&self) -> u32 {
        let mut count = 0;
        if self.error.is_some() {
            count += 1;
        }
        if self
            .report
            .as_ref()
            .is_some_and(|report| !report.sessions_indexed)
        {
            count += 1;
        }
        count
    }
}

impl Render for OverviewView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.status_text(cx);
        let attention = self.attention();
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
            .child(self.header(attention, cx))
            .child(
                div()
                    .id("overview-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(status.clone())
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(status),
            );
        if self.fixture {
            root = root.child(
                div()
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().warning)
                    .child(text(
                        cx,
                        "Demo mode does not read your usage database.",
                        "演示模式不会读取你的用量库。",
                    )),
            );
        }
        if let Some(report) = self.report.clone() {
            root = root
                .child(self.metrics(&report, cx))
                .child(self.platforms(&report, cx))
                .child(self.lower(&report, cx));
        } else if !self.loading {
            root = root.child(self.actions(cx));
        }
        root
    }
}

impl OverviewView {
    fn header(&self, attention: u32, cx: &mut Context<Self>) -> impl IntoElement {
        let mut windows = div().flex().gap(px(theme::CONTROL_GAP));
        for (id, days, label) in [
            ("overview-window-7", 7_u16, "7D"),
            ("overview-window-30", 30, "30D"),
            ("overview-window-90", 90, "90D"),
        ] {
            windows = windows.child(semantics::button(
                Button::new(id)
                    .label(label)
                    .ghost()
                    .selected(self.days == days)
                    .toggled(self.days == days)
                    .on_click(cx.listener(move |this, _, _, cx| this.set_days(days, cx))),
                self.loading || !self.requests.configured(),
                false,
                cx,
            ));
        }
        div()
            .flex()
            .items_start()
            .justify_between()
            .gap(px(theme::SECTION_GAP))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        div()
                            .text_size(px(theme::TITLE_SIZE))
                            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                            .child(text(cx, "Overview", "运行概览")),
                    )
                    .child(
                        div()
                            .text_size(px(theme::META_SIZE))
                            .text_color(cx.theme().muted_foreground)
                            .child(text(
                                cx,
                                "Status, then the numbers already stored on this computer, then the next action.",
                                "先看状态，再看本机已经记下的数字，最后看下一步。",
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .when(attention > 0, |row| {
                        row.child(
                            div()
                                .px(px(theme::CONTROL_GAP))
                                .h(px(theme::ROW_HEIGHT))
                                .rounded(px(theme::PILL_RADIUS))
                                .bg(cx.theme().secondary)
                                .text_size(px(theme::META_SIZE))
                                .child(format!(
                                    "{} {attention}",
                                    text(cx, "Needs attention", "需要处理")
                                )),
                        )
                    })
                    .child(windows)
                    .child(semantics::button(
                        Button::new("overview-refresh")
                            .label(text(cx, "Refresh", "刷新用量"))
                            .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                        self.loading || !self.requests.configured(),
                        self.loading,
                        cx,
                    )),
            )
    }

    fn metrics(&self, report: &WorkbenchReport, cx: &mut Context<Self>) -> impl IntoElement {
        let session_value = match report.session_scope {
            SessionScope::Unavailable => text(cx, "Not indexed", "未索引").to_string(),
            SessionScope::Window | SessionScope::AllHistory => report
                .session_count
                .map(grouped)
                .unwrap_or_else(|| text(cx, "Not indexed", "未索引").to_string()),
        };
        let session_hint = match report.session_scope {
            SessionScope::Window => text(cx, "This 7-day window", "这 7 日窗口"),
            SessionScope::AllHistory => {
                text(cx, "All history, not this window", "全历史，不是当前窗口")
            }
            SessionScope::Unavailable => text(cx, "Archive has no index", "归档尚未索引"),
        };
        let cost_value = report
            .cost_usd
            .map(format_usd)
            .unwrap_or_else(|| "—".to_string());
        let cost_hint = if report.cost_usd.is_some() {
            text(cx, "Recorded cost", "已记录成本")
        } else {
            text(cx, "Cost was not read", "这次没有读到成本")
        };
        div()
            .flex()
            .flex_wrap()
            .gap(px(theme::SECTION_GAP))
            .child(self.metric(
                text(cx, "Requests", "请求"),
                grouped(report.requests),
                text(cx, "Selected window", "当前窗口"),
                cx,
            ))
            .child(self.metric(
                text(cx, "Tokens", "TOKEN"),
                grouped(report.tokens),
                text(cx, "Selected window", "当前窗口"),
                cx,
            ))
            .child(self.metric(text(cx, "Cost", "成本"), cost_value, cost_hint, cx))
            .child(self.metric(
                text(cx, "Sessions", "会话"),
                session_value,
                session_hint,
                cx,
            ))
    }

    fn metric(
        &self,
        label: &str,
        value: String,
        hint: &str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .min_w(px(160.))
            .flex_1()
            .p(px(theme::SECTION_GAP))
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .rounded(px(theme::INPUT_RADIUS))
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(label.to_string()),
            )
            .child(
                div()
                    .text_size(px(theme::DISPLAY_SIZE))
                    .font_weight(gpui_kit::FontWeight::MEDIUM)
                    .font_family(cx.theme().mono_font_family.clone())
                    .child(value),
            )
            .child(
                div()
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(hint.to_string()),
            )
    }

    fn platforms(&self, report: &WorkbenchReport, cx: &mut Context<Self>) -> impl IntoElement {
        let mut row = div().flex().flex_wrap().gap(px(theme::SECTION_GAP));
        for platform in &report.platforms {
            let recorded = platform.requests > 0 || platform.tokens > 0;
            let state = if recorded {
                text(cx, "Recorded", "已计入")
            } else {
                text(cx, "No usage", "无记录")
            };
            row = row.child(
                div()
                    .min_w(px(220.))
                    .flex_1()
                    .p(px(theme::SECTION_GAP))
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded(px(12.))
                    .flex()
                    .flex_col()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .child(
                                div()
                                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                    .child(platform_title(&platform.id)),
                            )
                            .child(
                                div()
                                    .text_size(px(theme::META_SIZE))
                                    .text_color(if recorded {
                                        cx.theme().primary
                                    } else {
                                        cx.theme().muted_foreground
                                    })
                                    .child(state),
                            ),
                    )
                    .child(sparkline(&platform.requests_by_day, platform.requests, cx))
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .text_size(px(theme::META_SIZE))
                            .child(text(cx, "Requests", "请求"))
                            .child(
                                div()
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .child(grouped(platform.requests)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .text_size(px(theme::META_SIZE))
                            .child("TOKEN")
                            .child(
                                div()
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .child(grouped(platform.tokens)),
                            ),
                    ),
            );
        }
        row
    }

    fn lower(&self, report: &WorkbenchReport, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_wrap()
            .gap(px(theme::SECTION_GAP))
            .items_start()
            .child(self.chart(report, cx))
            .child(self.actions(cx))
    }

    fn chart(&self, report: &WorkbenchReport, cx: &mut Context<Self>) -> impl IntoElement {
        let max = report
            .days
            .iter()
            .map(|day| day.claude + day.codex + day.antigravity + day.opencode)
            .max()
            .unwrap_or(0);
        let mut columns = div()
            .h(px(160.))
            .flex()
            .items_end()
            .gap(px(if report.days.len() > 40 { 0. } else { 2. }));
        if max == 0 {
            let (en, zh) = if report.requests > 0 {
                (
                    "These bars do not include every source. The totals above still count them.",
                    "这些柱子没有画入全部来源。上面的总数仍然计入。",
                )
            } else {
                ("No requests in this window.", "这个窗口没有请求。")
            };
            columns = columns.child(
                div()
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(text(cx, en, zh)),
            );
        } else {
            for day in &report.days {
                columns = columns.child(day_stack(day, max, cx));
            }
        }
        let range = match (report.days.first(), report.days.last()) {
            (Some(first), Some(last)) => format!("{} – {}", first.label, last.label),
            _ => String::new(),
        };
        let mut chart = div()
            .min_w(px(360.))
            .flex_1()
            .p(px(theme::SECTION_GAP))
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .rounded(px(12.))
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(text(cx, "Requests by day", "逐日请求")),
            )
            .child(columns)
            .child(div().h(px(1.)).bg(cx.theme().border))
            .child(legend(cx))
            .child(
                div()
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(range),
            );
        if report.other_requests > 0 {
            chart = chart.child(
                div()
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "{} {}",
                        grouped(report.other_requests),
                        text(
                            cx,
                            "requests are in the totals and are not drawn in these bars.",
                            "条请求已计入总数，没有画进这些柱子。",
                        )
                    )),
            );
        }
        chart
    }

    fn actions(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .min_w(px(240.))
            .w(px(280.))
            .p(px(theme::SECTION_GAP))
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .rounded(px(12.))
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(text(cx, "Next", "下一步")),
            )
            .child(
                div()
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(text(
                        cx,
                        "These open existing pages. They do not invent usage.",
                        "这些入口只打开已有页面，不会编造用量。",
                    )),
            )
            .child(self.link_button(
                "overview-open-usage",
                text(cx, "Usage detail", "用量明细"),
                OverviewLink::Usage,
                cx,
            ))
            .child(self.link_button(
                "overview-open-commands",
                text(cx, "Commands", "命令"),
                OverviewLink::Commands,
                cx,
            ))
            .child(self.link_button(
                "overview-open-sessions",
                text(cx, "Agent sessions", "Agent 会话"),
                OverviewLink::Sessions,
                cx,
            ))
            .child(self.link_button(
                "overview-open-install",
                text(cx, "Install llmusage", "安装 llmusage"),
                OverviewLink::Install,
                cx,
            ))
    }

    fn link_button(
        &self,
        id: &'static str,
        label: &'static str,
        link: OverviewLink,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        semantics::button(
            Button::new(id)
                .label(label)
                .ghost()
                .w_full()
                .on_click(cx.listener(move |this, _, _, cx| this.open(link, cx))),
            false,
            false,
            cx,
        )
    }
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

fn sparkline(values: &[i64], requests: i64, cx: &mut Context<OverviewView>) -> impl IntoElement {
    let max = values.iter().copied().max().unwrap_or(0);
    if values.is_empty() || max == 0 {
        let (en, zh) = if values.is_empty() && requests > 0 {
            (
                "Included in the totals. Not drawn day by day.",
                "已计入总数，没有按日拆开。",
            )
        } else if values.is_empty() {
            ("No daily series", "没有逐日序列")
        } else {
            ("No requests in this window", "这个窗口没有请求")
        };
        return div()
            .h(px(36.))
            .text_size(px(theme::META_SIZE))
            .text_color(cx.theme().muted_foreground)
            .child(text(cx, en, zh))
            .into_any_element();
    }
    let mut row = div().h(px(36.)).flex().items_end().gap(px(2.));
    for value in values {
        let height = (*value as f32 / max as f32) * 36.;
        row = row.child(
            div()
                .flex_1()
                .h(px(height.max(1.)))
                .bg(cx.theme().primary)
                .rounded(px(2.)),
        );
    }
    row.into_any_element()
}

fn day_stack(
    day: &crate::runtime::insights::WorkbenchDay,
    max: i64,
    cx: &mut Context<OverviewView>,
) -> impl IntoElement {
    let palette = theme::palette(cx.theme().mode);
    let segments = [
        (
            day.opencode,
            gpui_kit::Hsla::from(rgb(palette.secondary_text)),
        ),
        (day.antigravity, cx.theme().muted_foreground),
        (day.codex, gpui_kit::Hsla::from(rgb(palette.output))),
        (day.claude, cx.theme().primary),
    ];
    let mut column = div().flex_1().h(px(160.)).flex().flex_col().justify_end();
    for (value, color) in segments {
        if value <= 0 {
            continue;
        }
        let height = value as f32 / max as f32 * 160.;
        column = column.child(div().w_full().h(px(height.max(1.))).bg(color));
    }
    column
}

fn legend(cx: &mut Context<OverviewView>) -> impl IntoElement {
    let palette = theme::palette(cx.theme().mode);
    let items = [
        ("Claude Code", cx.theme().primary),
        ("Codex", gpui_kit::Hsla::from(rgb(palette.output))),
        ("Antigravity", cx.theme().muted_foreground),
        (
            "OpenCode",
            gpui_kit::Hsla::from(rgb(palette.secondary_text)),
        ),
    ];
    let mut row = div()
        .flex()
        .flex_wrap()
        .gap(px(theme::SECTION_GAP))
        .text_size(px(theme::META_SIZE));
    for (label, color) in items {
        row = row.child(
            div()
                .flex()
                .items_center()
                .gap(px(theme::CONTROL_GAP))
                .child(
                    div()
                        .w(px(theme::CONTROL_GAP))
                        .h(px(theme::CONTROL_GAP))
                        .rounded(px(theme::SELECTION_RADIUS))
                        .bg(color),
                )
                .child(label),
        );
    }
    row
}
