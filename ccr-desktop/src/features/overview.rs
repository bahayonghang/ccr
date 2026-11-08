//! Home workbench. Numbers come from the captured usage database and session archive.
use crate::{
    locales::{is_chinese, service_error, text},
    runtime::insights::{
        InsightsRequests, SessionScope, WorkbenchDay, WorkbenchPlatform, WorkbenchReport,
    },
    shell::{page_header, semantics},
    theme,
};
use ccr_app::runtime::{
    AdmissionError, Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent,
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, Icon, IconName, Selectable,
    button::{Button, ButtonVariants},
    tooltip::Tooltip,
};
use gpui_kit::{
    AnyView, App, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render,
    ScrollHandle, SharedString, StatefulInteractiveElement, Styled, Subscription, Task, Window,
    div, prelude::FluentBuilder, px,
};

/// 带稳定测试 id 的节点（`test_support()` 包装后的有状态 div）。
type Node = gpui_kit::base::ObservedElement<gpui_kit::Stateful<gpui_kit::Div>>;

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
    show_exact: bool,
    scroll: ScrollHandle,
    exact_scroll: ScrollHandle,
    exact_rows: Vec<(FocusHandle, Subscription)>,
    focus: FocusHandle,
    /// 外壳拥有的 lens 胶囊，嵌入标题行（同一行单一 h1）。
    lens: Option<AnyView>,
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
            show_exact: false,
            scroll: ScrollHandle::new(),
            exact_scroll: ScrollHandle::new(),
            exact_rows: Vec::new(),
            focus: cx.focus_handle(),
            lens: None,
        }
    }

    pub fn embed_lens(&mut self, lens: AnyView) {
        self.lens = Some(lens);
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

    fn zone(&self) -> &'static str {
        if self.requests.report_clock().timezone == ccr_usage::ReportTimezone::Utc {
            " UTC"
        } else {
            ""
        }
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
        let zone = self.zone();
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
        let report = self.report.clone();
        let report = report.as_deref();
        let rows = report.map_or_else(Vec::new, |report| exact_rows(&report.days));
        self.sync_exact_focus(rows.len(), window, cx);
        let stacked = window.viewport_size().width < px(theme::NARROW_WIDTH);
        let mut root = div()
            .id("overview-root")
            .test_support()
            .track_focus(&self.focus)
            .tab_stop(false)
            .size_full()
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .p(px(theme::CONTENT_PADDING))
            .gap(px(theme::SECTION_GAP))
            .child(self.header(cx));
        if let Some(report) = report {
            root = root.child(self.kpis(report, stacked, cx));
            if let Some(band) = self.platforms(report, cx) {
                root = root.child(band);
            }
        }
        // 主列与右栏等高；窄宽时右栏堆到下方，不横向滚动整页。
        root.child(
            div()
                .flex()
                .flex_shrink_0()
                .gap(px(theme::SECTION_GAP))
                .when(stacked, |row| row.flex_col())
                .when(!stacked, |row| row.items_stretch())
                .child(self.usage_panel(report, &rows, stacked, cx))
                .child(self.rail(report, stacked, cx)),
        )
    }
}

impl OverviewView {
    /// One focus handle per displayed exact-value row (days and folded quiet runs).
    fn sync_exact_focus(&mut self, count: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.exact_rows.truncate(count);
        while self.exact_rows.len() < count {
            let index = self.exact_rows.len();
            let focus = cx.focus_handle().tab_stop(true);
            let subscription = cx.on_focus(&focus, window, move |this, _, cx| {
                // 表头是第一个直接子项，显示行 index 对应子项 index + 1；先在表格内
                // reveal 当前行，再把表格视口 reveal 到外层页面滚动区中。
                this.exact_scroll.scroll_to_item(index + 1);
                let table = this.exact_scroll.bounds();
                let page = this.scroll.bounds();
                let mut offset = this.scroll.offset();
                if table.top() < page.top() {
                    offset.y += page.top() - table.top();
                } else if table.bottom() > page.bottom() {
                    offset.y += page.bottom() - table.bottom();
                }
                this.scroll.set_offset(offset);
                cx.notify();
            });
            self.exact_rows.push((focus, subscription));
        }
    }

    fn header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let status = self.status_text(cx);
        let ink = if self.error.is_some() {
            cx.theme().warning
        } else {
            cx.theme().muted_foreground
        };
        let mut title = div()
            .min_w_0()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(theme::SECTION_GAP))
            .child(
                div()
                    .id("overview-heading")
                    .test_support()
                    .role(gpui_kit::Role::Heading)
                    .aria_level(1)
                    .text_size(px(theme::TITLE_SIZE))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(text(cx, "Overview", "运行概览")),
            );
        if let Some(lens) = &self.lens {
            title = title.child(lens.clone());
        }
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap(px(theme::CONTROL_GAP))
                    .child(title)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(theme::CONTROL_GAP))
                            .child(self.windows(cx))
                            .child(semantics::button(
                                Button::new("overview-refresh")
                                    .label(text(cx, "Refresh", "刷新用量"))
                                    .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                                self.loading || !self.requests.configured(),
                                self.loading,
                                cx,
                            )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .child(status_dot(if self.error.is_some() {
                        ink
                    } else if self.report.is_some() && !self.loading {
                        cx.theme().primary
                    } else {
                        cx.theme().input
                    }))
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

    /// Page-level 7D/30D/90D range capsule; it applies to the whole page.
    fn windows(&self, cx: &mut Context<Self>) -> gpui_kit::Div {
        let disabled = !self.requests.configured();
        let mut group = page_header::capsule(cx).self_center();
        for (id, days, label) in [
            ("overview-window-7", 7_u16, "7D"),
            ("overview-window-30", 30, "30D"),
            ("overview-window-90", 90, "90D"),
        ] {
            group = group.child(semantics::tab(
                semantics::button(
                    Button::new(id)
                        .label(label)
                        .ghost()
                        .on_click(cx.listener(move |this, _, _, cx| this.set_days(days, cx))),
                    disabled,
                    false,
                    cx,
                ),
                self.days == days,
                disabled,
                cx,
            ));
        }
        group
    }

    /// One panel, four columns separated by hairlines; cost is the primary reading.
    fn kpis(&self, report: &WorkbenchReport, stacked: bool, cx: &mut Context<Self>) -> Node {
        let zh = is_chinese(cx);
        let days = f64::from(report.window_days.max(1));
        let span = if zh {
            format!("{} 日", report.window_days)
        } else {
            format!("{} days", report.window_days)
        };
        // 派生值（日均 = 总量 ÷ 窗口天数）必须带“日均 / avg”标签。
        let average = |value: String| {
            if zh {
                format!("{span} · 日均 {value}")
            } else {
                format!("{span} · avg {value}/day")
            }
        };
        let cost = match report.cost_usd {
            Some(value) => kpi_cell(
                text(cx, "Cost", "成本"),
                reading(
                    "total-cost",
                    format!("{}: USD {value}", text(cx, "Cost", "成本")),
                    format_usd(value),
                    theme::KPI_PRIMARY_SIZE,
                    gpui_kit::FontWeight::SEMIBOLD,
                ),
                // 日均只是辅助说明：≥$1 时两位小数即可，小额仍保留精度。
                average({
                    let daily = value / days;
                    if daily.abs() >= 1.0 {
                        format!("${daily:.2}")
                    } else {
                        format_usd(daily)
                    }
                }),
                cx,
            ),
            None => kpi_cell(
                text(cx, "Cost", "成本"),
                reading(
                    "total-cost",
                    format!(
                        "{}: {}",
                        text(cx, "Cost", "成本"),
                        text(cx, "Cost unavailable", "成本不可用")
                    ),
                    "–".into(),
                    theme::KPI_PRIMARY_SIZE,
                    gpui_kit::FontWeight::SEMIBOLD,
                ),
                text(cx, "Cost unavailable", "成本不可用").into(),
                cx,
            ),
        };
        let requests = kpi_cell(
            text(cx, "Requests", "请求"),
            count_reading(
                "total-requests",
                text(cx, "Requests", "请求"),
                report.requests,
            ),
            average(format_average(report.requests as f64 / days)),
            cx,
        );
        let tokens = kpi_cell(
            text(cx, "Tokens", "Token"),
            count_reading("total-tokens", text(cx, "Tokens", "Token"), report.tokens),
            span.clone(),
            cx,
        );
        let sessions = self.session_cell(report, &span, cx);
        let mut panel = div()
            .id("overview-kpis")
            .test_support()
            .flex_shrink_0()
            .flex()
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .rounded(px(theme::PANEL_RADIUS));
        let hairline = cx.theme().border;
        if stacked {
            // 窄宽 2×2：两行，每行第二格左侧 hairline，第二行顶部 hairline。
            panel = panel
                .flex_col()
                .child(
                    div()
                        .flex()
                        .child(cost)
                        .child(requests.border_l_1().border_color(hairline)),
                )
                .child(
                    div()
                        .flex()
                        .border_t_1()
                        .border_color(hairline)
                        .child(tokens)
                        .child(sessions.border_l_1().border_color(hairline)),
                );
        } else {
            panel = panel
                .child(cost)
                .child(requests.border_l_1().border_color(hairline))
                .child(tokens.border_l_1().border_color(hairline))
                .child(sessions.border_l_1().border_color(hairline));
        }
        panel
    }

    fn session_cell(&self, report: &WorkbenchReport, span: &str, cx: &App) -> gpui_kit::Div {
        let title = match report.session_scope {
            SessionScope::Window if is_chinese(cx) => format!("会话 · {} 日", report.window_days),
            SessionScope::Window => format!("Sessions · {}D", report.window_days),
            SessionScope::AllHistory => text(cx, "Sessions · all time", "会话 · 全历史").into(),
            SessionScope::Unindexed | SessionScope::Unavailable => {
                text(cx, "Sessions", "会话").into()
            }
        };
        let (short, exact, size, note): (String, String, f32, String) =
            match (report.session_scope, report.session_count) {
                (SessionScope::Window, Some(count)) => (
                    compact_count(count),
                    grouped(count),
                    theme::KPI_SIZE,
                    span.into(),
                ),
                (SessionScope::AllHistory, Some(count)) => (
                    compact_count(count),
                    grouped(count),
                    theme::KPI_SIZE,
                    text(cx, "all time", "全历史").into(),
                ),
                (SessionScope::Unindexed, _) => (
                    text(cx, "Unindexed", "未索引").into(),
                    text(cx, "Archive not indexed", "归档未索引").into(),
                    theme::KPI_STATE_SIZE,
                    text(cx, "Archive not indexed", "归档未索引").into(),
                ),
                _ => (
                    text(cx, "Unavailable", "不可用").into(),
                    text(cx, "Session count unavailable", "会话数不可用").into(),
                    theme::KPI_STATE_SIZE,
                    text(cx, "Session count unavailable", "会话数不可用").into(),
                ),
            };
        kpi_cell(
            text(cx, "Sessions", "会话"),
            reading(
                "total-sessions",
                format!("{title}: {exact}"),
                short,
                size,
                gpui_kit::FontWeight::MEDIUM,
            ),
            note,
            cx,
        )
    }

    /// Platforms with daily records get a sparkline cell; the rest share one
    /// compact "Other sources" cell with their reason (never zero-filled).
    fn platforms(
        &self,
        report: &WorkbenchReport,
        cx: &mut Context<Self>,
    ) -> Option<gpui_kit::AnyElement> {
        let (recorded, quiet): (Vec<&WorkbenchPlatform>, Vec<&WorkbenchPlatform>) = report
            .platforms
            .iter()
            .partition(|platform| platform.requests_by_day.iter().any(|value| *value > 0));
        if recorded.is_empty() && quiet.is_empty() {
            return None;
        }
        let mut band = div()
            .id("overview-platform-band")
            .test_support()
            .w_full()
            .flex_shrink_0()
            .flex()
            .bg(cx.theme().background)
            .rounded(px(theme::PANEL_RADIUS))
            .border_1()
            .border_color(cx.theme().border)
            .overflow_x_scroll();
        for (index, platform) in recorded.iter().enumerate() {
            band = band.child(platform_cell(platform, index > 0, cx));
        }
        if !quiet.is_empty() {
            band = band.child(other_sources(&quiet, !recorded.is_empty(), cx));
        }
        Some(band.into_any_element())
    }

    fn usage_panel(
        &self,
        report: Option<&WorkbenchReport>,
        rows: &[ExactRow],
        stacked: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let panel = div()
            .id("overview-usage")
            .test_support()
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
            .rounded(px(theme::PANEL_RADIUS));
        let Some(report) = report else {
            return panel
                .child(heading(text(cx, "Usage and cost", "用量与成本")))
                .child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child(if self.loading {
                            text(cx, "Reading the selected period…", "正在读取所选时段…")
                        } else {
                            text(cx, "No report available", "暂无可用报告")
                        }),
                );
        };
        panel.child(self.chart(report, rows, cx))
    }

    fn chart(
        &self,
        report: &WorkbenchReport,
        rows: &[ExactRow],
        cx: &mut Context<Self>,
    ) -> gpui_kit::Div {
        // 自动收紧到首个有数据日的前一天；查询窗口仍写在页头状态行。
        let start = trimmed_start(&report.days);
        let visible = &report.days[start..];
        let max = visible.iter().map(day_total).max().unwrap_or(0);
        let mut title = div()
            .flex()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .child(heading(text(cx, "Daily requests", "逐日请求")));
        if let Some(first) = visible.first().filter(|_| start > 0) {
            let since = if is_chinese(cx) {
                format!("自 {} 起", first.label)
            } else {
                format!("since {}", first.label)
            };
            title = title.child(
                div()
                    .id("overview-chart-since")
                    .test_support()
                    .aria_label(since.clone())
                    .text_size(px(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(since),
            );
        }
        let mut chart = div().flex().flex_col().gap(px(theme::CONTROL_GAP)).child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap(px(theme::CONTROL_GAP))
                .child(title)
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(theme::SECTION_GAP))
                        .child(legend(report, cx))
                        .child(semantics::button(
                            Button::new("overview-exact-toggle")
                                .label(text(cx, "Exact values", "精确数值"))
                                .selected(self.show_exact)
                                .toggled(self.show_exact)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.show_exact = !this.show_exact;
                                    cx.notify();
                                })),
                            false,
                            false,
                            cx,
                        )),
                ),
        );
        if visible.is_empty() {
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
        } else {
            chart = chart
                .child(bars(visible, max, cx))
                .child(ticks(visible, cx));
        }
        chart
            .when(self.show_exact, |el| {
                el.child(self.exact_table(report, rows, cx))
            })
            .when(report.other_requests > 0, |el| {
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

    fn exact_table(
        &self,
        report: &WorkbenchReport,
        rows: &[ExactRow],
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let mut table = div()
            .id("overview-exact-table")
            .test_support()
            .role(gpui_kit::Role::Table)
            .aria_label(text(
                cx,
                "Daily requests by chart source",
                "图表来源逐日请求",
            ))
            .max_h(px(240.))
            .overflow_x_scroll()
            .overflow_y_scroll()
            .track_scroll(&self.exact_scroll)
            .font_features(theme::tabular_figures());
        let mut header = div().flex().min_w(px(640.)).py(px(4.));
        for (index, label) in [
            text(cx, "Date", "日期"),
            "Claude Code",
            "Codex",
            "Antigravity",
            "OpenCode",
            text(cx, "Chart total", "图中合计"),
        ]
        .into_iter()
        .enumerate()
        {
            header = header.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .text_size(px(theme::META_SIZE))
                    .when(index > 0, |cell| cell.text_right())
                    .child(label),
            );
        }
        table = table.child(header);
        if report.days.is_empty() {
            table = table.child(div().text_color(cx.theme().muted_foreground).child(text(
                cx,
                "Daily breakdown unavailable",
                "逐日明细不可用",
            )));
        }
        // 最新在上；连续全零日折叠为一行。显示行与焦点句柄按同一顺序一一对应。
        for (index, row) in rows.iter().enumerate() {
            let Some((focus, _)) = self.exact_rows.get(index) else {
                break;
            };
            table = table.child(match *row {
                ExactRow::Day(day) => day_row(&report.days[day], focus, cx),
                ExactRow::Quiet { first, last } => quiet_row(
                    &report.days[first],
                    &report.days[last],
                    last - first + 1,
                    focus,
                    cx,
                ),
            });
        }
        table
    }

    fn rail(
        &self,
        report: Option<&WorkbenchReport>,
        stacked: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut rail = div()
            .id("overview-rail")
            .test_support()
            .when(stacked, |el| el.w_full())
            .when(!stacked, |el| {
                el.w(px(theme::OVERVIEW_RAIL_WIDTH)).flex_shrink_0()
            })
            .flex()
            .flex_col()
            .p(px(theme::SECTION_GAP))
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .rounded(px(theme::PANEL_RADIUS))
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
                action_row(id, label, icon, cx)
                    .on_click(cx.listener(move |this, _, _, cx| this.open(link, cx))),
            );
        }
        rail.child(self.data_source(report, cx))
    }

    /// Data source summary from state already in this view; no extra query.
    fn data_source(&self, report: Option<&WorkbenchReport>, cx: &App) -> Node {
        let configured = self.requests.configured();
        let muted = cx.theme().muted_foreground;
        let mut section = div()
            .id("overview-insights")
            .test_support()
            .mt(px(theme::SECTION_GAP))
            .pt(px(theme::SECTION_GAP))
            .border_t_1()
            .border_color(cx.theme().border)
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .text_size(px(theme::META_SIZE))
            .child(
                div()
                    .text_size(px(theme::BODY_SIZE))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(text(cx, "Data source", "数据源")),
            )
            .child(
                div()
                    .id("overview-source-state")
                    .test_support()
                    .flex()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .child(status_dot(if configured {
                        cx.theme().primary
                    } else {
                        cx.theme().input
                    }))
                    .child(if configured {
                        text(cx, "Local usage source configured", "已配置本机用量来源")
                    } else {
                        text(cx, "No local usage source configured", "未配置本机用量来源")
                    }),
            );
        if self.fixture {
            section = section.child(div().text_color(cx.theme().warning).child(text(
                cx,
                "Isolated demo",
                "隔离演示",
            )));
        }
        let Some(report) = report else {
            return section.child(div().text_color(muted).child(text(
                cx,
                "Read local usage to inspect source and archive coverage.",
                "读取本机用量后，可查看来源与归档覆盖范围。",
            )));
        };
        let zone = self.zone();
        section = section.child(div().text_color(muted).child(if is_chinese(cx) {
            format!("报告截至 {}{zone}", report.as_of)
        } else {
            format!("Report as of {}{zone}", report.as_of)
        }));
        let active = report.days.iter().filter(|day| day_total(day) > 0).count();
        let coverage = if report.days.is_empty() {
            text(cx, "Daily breakdown unavailable", "逐日明细不可用").to_string()
        } else if is_chinese(cx) {
            format!(
                "图表的四个来源在 {} 个日历日中有 {active} 个活跃日；其他来源不计入活跃日。",
                report.days.len()
            )
        } else {
            format!(
                "The four chart sources were active on {active} of {} calendar days; other sources are excluded from active days.",
                report.days.len()
            )
        };
        section = section.child(div().text_color(muted).child(coverage));
        if matches!(
            report.session_scope,
            SessionScope::Unindexed | SessionScope::Unavailable
        ) {
            section = section.child(div().text_color(muted).child(text(
                cx,
                "Session archive coverage is unknown. Browse sessions to inspect the archive.",
                "会话归档覆盖范围未知，可浏览会话检查归档。",
            )));
        }
        section
    }
}

fn heading(label: &'static str) -> gpui_kit::Div {
    div()
        .text_size(px(theme::SECTION_TITLE_SIZE))
        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
        .child(label)
}

fn status_dot(color: gpui_kit::Hsla) -> gpui_kit::Div {
    div()
        .flex_shrink_0()
        .w(px(7.))
        .h(px(7.))
        .rounded_full()
        .bg(color)
}

fn action_row(
    id: &'static str,
    label: &'static str,
    icon: IconName,
    cx: &App,
) -> gpui_kit::base::Button {
    gpui_kit::base::Button::new(id)
        .accessibility_label(label)
        .mt(px(theme::CONTROL_GAP))
        .w_full()
        .px(px(theme::CONTROL_GAP))
        .h(px(theme::ROW_HEIGHT))
        .rounded(px(theme::SELECTION_RADIUS))
        .border_2()
        .border_color(gpui_kit::transparent_black())
        .hover(|style| style.bg(cx.theme().muted))
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
                .child(div().flex_1().child(label)),
        )
}

/// A KPI column: 12px muted label, the reading, and a 12px note from real data.
fn kpi_cell(label: &'static str, value: Node, note: String, cx: &App) -> gpui_kit::Div {
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .gap(px(4.))
        .p(px(theme::SECTION_GAP))
        .child(
            div()
                .text_size(px(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(label),
        )
        .child(value)
        .child(
            div()
                .text_size(px(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(note),
        )
}

fn count_reading(id: &'static str, label: &'static str, value: i64) -> Node {
    reading(
        id,
        format!("{label}: {}", grouped(value)),
        compact_count(value),
        theme::KPI_SIZE,
        gpui_kit::FontWeight::MEDIUM,
    )
}

/// A labelled reading whose exact value stays in its accessible value and tooltip.
fn reading(
    id: &str,
    description: String,
    short: String,
    size: f32,
    weight: gpui_kit::FontWeight,
) -> Node {
    div()
        .id(SharedString::from(format!("overview-{id}")))
        .test_support()
        .role(gpui_kit::Role::Label)
        .aria_label(description.clone())
        .aria_value(description.clone())
        .font_features(theme::tabular_figures())
        .font_weight(weight)
        .text_size(px(size))
        .line_height(gpui_kit::relative(1.2))
        .child(short)
        .tooltip(move |window, cx| Tooltip::new(description.clone()).build(window, cx))
}

/// Compact label/value row used inside platform cells.
fn inline_metric(id: String, label: &'static str, value: i64, cx: &App) -> gpui_kit::Div {
    let description = format!("{label}: {}", grouped(value));
    div()
        .min_w_0()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(theme::CONTROL_GAP))
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
                .font_features(theme::tabular_figures())
                .font_weight(gpui_kit::FontWeight::MEDIUM)
                .text_right()
                .child(compact_count(value))
                .tooltip(move |window, cx| Tooltip::new(description.clone()).build(window, cx)),
        )
}

fn platform_cell(platform: &WorkbenchPlatform, divided: bool, cx: &App) -> gpui_kit::Div {
    let ink = theme::platform_color(&platform.id, cx.theme().mode);
    div()
        .relative()
        .min_w(px(theme::SPARK_CELL_MIN_WIDTH))
        .flex_1()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(theme::CONTROL_GAP))
        .p(px(theme::SECTION_GAP))
        .when(divided, |cell| {
            cell.border_l_1().border_color(cx.theme().border)
        })
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .h(px(2.))
                .bg(ink),
        )
        .child(
            div()
                .text_size(px(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(platform_title(&platform.id)),
        )
        .child(sparkline(&platform.requests_by_day, ink, cx))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(theme::CONTROL_GAP))
                .child(inline_metric(
                    format!("{}-requests", platform.id),
                    text(cx, "Requests", "请求"),
                    platform.requests,
                    cx,
                ))
                .child(inline_metric(
                    format!("{}-tokens", platform.id),
                    text(cx, "Tokens", "Token"),
                    platform.tokens,
                    cx,
                )),
        )
}

fn other_sources(quiet: &[&WorkbenchPlatform], divided: bool, cx: &App) -> Node {
    let zh = is_chinese(cx);
    let mut cell = div()
        .id("overview-other-sources")
        .test_support()
        .min_w(px(theme::SPARK_CELL_MIN_WIDTH))
        .flex_1()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(4.))
        .p(px(theme::SECTION_GAP))
        .text_size(px(theme::META_SIZE))
        .when(divided, |cell| {
            cell.border_l_1().border_color(cx.theme().border)
        })
        .child(div().text_color(cx.theme().muted_foreground).child(text(
            cx,
            "Other sources",
            "其他来源",
        )));
    for platform in quiet {
        let name = platform_title(&platform.id);
        // 有总数但无逐日明细时保留真实总数；无记录时只写原因，不铺 0。
        let description = match (platform.requests > 0, zh) {
            (true, true) => format!(
                "{name} · {} 次请求 · 无逐日明细",
                grouped(platform.requests)
            ),
            (true, false) => format!(
                "{name} · {} requests · daily split unavailable",
                grouped(platform.requests)
            ),
            (false, true) => format!("{name} · 无请求"),
            (false, false) => format!("{name} · no requests"),
        };
        cell = cell.child(
            div()
                .id(SharedString::from(format!(
                    "overview-other-{}",
                    platform.id
                )))
                .test_support()
                .aria_label(description.clone())
                .text_color(cx.theme().secondary_foreground)
                .child(description),
        );
    }
    cell
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

/// Labelled derived averages: compact from 1K, whole numbers from 10, one decimal below.
fn format_average(value: f64) -> String {
    if value >= 1_000. {
        compact_count(value.round() as i64)
    } else if value >= 10. {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
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

pub(super) fn grouped(value: i64) -> String {
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

pub(super) fn format_usd(value: f64) -> String {
    if value.abs() >= 100.0 {
        format!("${value:.2}")
    } else {
        format!("${value:.4}")
    }
}

fn sparkline(values: &[i64], ink: gpui_kit::Hsla, cx: &App) -> gpui_kit::Div {
    let max = values.iter().copied().max().unwrap_or(0);
    let row = div()
        .h(px(theme::SPARK_HEIGHT))
        .flex()
        .min_w_0()
        .items_end()
        .border_b_1()
        .border_color(cx.theme().border);
    if max == 0 {
        // 只有有逐日记录的平台才单独成格；此分支仅作防御，绘制虚线空基线。
        return row.border_dashed();
    }
    let subtle = gpui_kit::Hsla::from(gpui_kit::rgb(theme::palette(cx.theme().mode).subtle));
    let mut row = row.gap(px(if values.len() > 30 { 0. } else { 1. }));
    for value in values {
        let height = if *value == 0 {
            0.
        } else {
            (*value as f32 / max as f32 * (theme::SPARK_HEIGHT - 2.)).max(1.)
        };
        row = row.child(div().flex_1().min_w_0().h(px(height)).bg(if *value == max {
            ink
        } else {
            subtle
        }));
    }
    row
}

const SERIES: [&str; 4] = ["claude", "codex", "antigravity", "opencode"];

fn day_values(day: &WorkbenchDay) -> [i64; 4] {
    [day.claude, day.codex, day.antigravity, day.opencode]
}

fn day_total(day: &WorkbenchDay) -> i64 {
    day_values(day).iter().sum()
}

/// First plotted day: one day before the first day with chart data, so a long
/// window does not open with an empty stretch. No data keeps the full window.
fn trimmed_start(days: &[WorkbenchDay]) -> usize {
    days.iter()
        .position(|day| day_total(day) > 0)
        .map_or(0, |first| first.saturating_sub(1))
}

/// A displayed exact-value row; indices point into `WorkbenchReport::days`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExactRow {
    Day(usize),
    /// 连续全零日（含单日），`first` 更早、`last` 更晚。
    Quiet {
        first: usize,
        last: usize,
    },
}

/// Newest-first rows with every run of all-zero days folded into one row.
fn exact_rows(days: &[WorkbenchDay]) -> Vec<ExactRow> {
    let mut rows = Vec::new();
    let mut index = days.len();
    while index > 0 {
        index -= 1;
        if day_total(&days[index]) > 0 {
            rows.push(ExactRow::Day(index));
            continue;
        }
        let last = index;
        while index > 0 && day_total(&days[index - 1]) == 0 {
            index -= 1;
        }
        rows.push(ExactRow::Quiet { first: index, last });
    }
    rows
}

fn exact_row(id: SharedString, description: String, focus: &FocusHandle, cx: &App) -> Node {
    div()
        .id(id)
        .test_support()
        .role(gpui_kit::Role::Row)
        .aria_label(description.clone())
        .aria_value(description)
        .track_focus(focus)
        .tab_stop(true)
        .flex()
        .min_w(px(640.))
        .py(px(4.))
        .border_2()
        .border_color(gpui_kit::transparent_black())
        .focus(|style| style.border_color(cx.theme().ring))
}

fn day_row(day: &WorkbenchDay, focus: &FocusHandle, cx: &App) -> gpui_kit::AnyElement {
    let values = day_values(day);
    let total: i64 = values.iter().sum();
    let description = format!(
        "{}: Claude Code {}; Codex {}; Antigravity {}; OpenCode {}; {} {}",
        day.label,
        grouped(values[0]),
        grouped(values[1]),
        grouped(values[2]),
        grouped(values[3]),
        text(cx, "Chart total", "图中合计"),
        grouped(total),
    );
    let mut row = exact_row(
        SharedString::from(format!("overview-exact-{}", day.label)),
        description,
        focus,
        cx,
    );
    for (index, value) in std::iter::once(day.label.clone())
        .chain(values.into_iter().chain([total]).map(grouped))
        .enumerate()
    {
        row = row.child(
            div()
                .flex_1()
                .min_w_0()
                .when(index > 0, |cell| cell.text_right())
                .child(value),
        );
    }
    row.into_any_element()
}

fn quiet_row(
    first: &WorkbenchDay,
    last: &WorkbenchDay,
    count: usize,
    focus: &FocusHandle,
    cx: &App,
) -> gpui_kit::AnyElement {
    let description = match (count, is_chinese(cx)) {
        (1, true) => format!("{} · 无记录", first.label),
        (1, false) => format!("{} · no records", first.label),
        (_, true) => format!("{} – {} · {count} 日无记录", first.label, last.label),
        (_, false) => format!(
            "{} – {} · no records ({count} days)",
            first.label, last.label
        ),
    };
    exact_row(
        SharedString::from(format!("overview-exact-quiet-{}", first.label)),
        description.clone(),
        focus,
        cx,
    )
    .text_color(cx.theme().muted_foreground)
    .child(div().flex_1().min_w_0().child(description))
    .into_any_element()
}

fn bars(visible: &[WorkbenchDay], max: i64, cx: &App) -> gpui_kit::Div {
    let mut bars = div()
        .relative()
        .flex()
        .items_end()
        .h(px(theme::OVERVIEW_CHART_HEIGHT))
        .border_b_1()
        .border_color(cx.theme().border)
        .gap(px(if visible.len() > 30 { 1. } else { 3. }));
    for fraction in [0.25, 0.5, 0.75] {
        bars = bars.child(
            div()
                .absolute()
                .top(px(theme::OVERVIEW_CHART_HEIGHT * fraction))
                .w_full()
                .h(px(1.))
                .bg(cx.theme().border),
        );
    }
    for day in visible {
        bars = bars.child(day_stack(day, max.max(1), cx));
    }
    bars
}

// 与绘图区等宽日槽对齐；长窗口标注每段中点，使两端日期落在绘图区内。
fn ticks(visible: &[WorkbenchDay], cx: &App) -> gpui_kit::Div {
    let mut ticks = div()
        .flex()
        .gap(px(if visible.len() > 30 { 1. } else { 3. }))
        .text_size(px(theme::META_SIZE))
        .text_color(cx.theme().muted_foreground);
    let count = visible.len();
    let buckets = if count <= 7 { count } else { 6 };
    for (index, day) in visible.iter().enumerate() {
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
    ticks
}

fn day_stack(day: &WorkbenchDay, max: i64, cx: &App) -> impl IntoElement {
    let column = div()
        .id(SharedString::from(format!("overview-day-{}", day.label)))
        .test_support()
        .flex_1()
        .min_w_0()
        .h(px(theme::OVERVIEW_CHART_HEIGHT))
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
    if values.iter().all(|value| *value == 0) {
        return column
            .child(
                div()
                    .w_full()
                    .border_b_1()
                    .border_dashed()
                    .border_color(cx.theme().border),
            )
            .tooltip(move |window, cx| Tooltip::new(detail.clone()).build(window, cx));
    }
    let mut column = column;
    let mut top = true;
    for (id, value) in SERIES.into_iter().zip(values).rev() {
        if value > 0 {
            let segment_detail = format!(
                "{} · {}: {} {}",
                day.label,
                platform_title(id),
                grouped(value),
                text(cx, "requests", "请求")
            );
            column = column.child(
                div()
                    .id(SharedString::from(format!(
                        "overview-day-{}-{id}",
                        day.label
                    )))
                    .w_full()
                    // 段间保留 1px 阅读面间隙，其下至少 1px 墨色；仅最上段做小圆角。
                    .h(px((value as f32 / max as f32
                        * theme::OVERVIEW_CHART_HEIGHT)
                        .max(2.)))
                    .border_t_1()
                    .border_color(cx.theme().background)
                    .when(top, |segment| {
                        segment.rounded_t(px(theme::CHART_BAR_RADIUS))
                    })
                    .bg(theme::chart_series(id, cx.theme().mode))
                    .tooltip(move |window, cx| {
                        Tooltip::new(segment_detail.clone()).build(window, cx)
                    }),
            );
            top = false;
        }
    }
    column.tooltip(move |window, cx| Tooltip::new(detail.clone()).build(window, cx))
}

fn legend(report: &WorkbenchReport, cx: &App) -> impl IntoElement {
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
                            .rounded(px(theme::CHART_BAR_RADIUS))
                            .bg(theme::chart_series(id, cx.theme().mode)),
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

    fn days(values: &[i64]) -> Vec<WorkbenchDay> {
        values
            .iter()
            .enumerate()
            .map(|(index, value)| WorkbenchDay {
                label: format!("d{index}"),
                claude: *value,
                codex: 0,
                antigravity: 0,
                opencode: 0,
            })
            .collect()
    }

    #[test]
    fn exact_rows_are_newest_first_and_fold_every_quiet_run() {
        use ExactRow::{Day, Quiet};
        assert_eq!(exact_rows(&[]), Vec::new());
        assert_eq!(
            exact_rows(&days(&[0, 0, 2, 0, 0, 0, 5, 0])),
            vec![
                Quiet { first: 7, last: 7 },
                Day(6),
                Quiet { first: 3, last: 5 },
                Day(2),
                Quiet { first: 0, last: 1 },
            ]
        );
        assert_eq!(exact_rows(&days(&[1, 2])), vec![Day(1), Day(0)]);
        assert_eq!(
            exact_rows(&days(&[0, 0, 0])),
            vec![Quiet { first: 0, last: 2 }]
        );
    }

    #[test]
    fn chart_trims_to_one_day_before_the_first_record() {
        assert_eq!(trimmed_start(&days(&[0, 0, 0, 4, 0])), 2);
        assert_eq!(trimmed_start(&days(&[0, 4])), 0);
        assert_eq!(trimmed_start(&days(&[4, 0])), 0);
        // 没有任何记录时保留完整窗口（全部虚线基线），不假装有起点。
        assert_eq!(trimmed_start(&days(&[0, 0])), 0);
        assert_eq!(format_average(1500.), "1.5K");
        assert_eq!(format_average(143.4), "143");
        assert_eq!(format_average(0.25), "0.2");
    }

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
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(gpui_kit::WindowBounds::Windowed(gpui_kit::Bounds::new(
                        gpui_kit::point(px(0.), px(0.)),
                        gpui_kit::size(px(1280.), px(800.)),
                    ))),
                    ..WindowOptions::default()
                },
                cx,
                |_, cx| cx.new(|cx| OverviewView::new(services, cx)),
            )
            .unwrap_or_else(|e| panic!("overview window: {e}"))
        });
        cx.update_window(handle, |_, window, _| window.activate_window())
            .unwrap_or_else(|e| panic!("activate overview: {e}"));
        cx.run_until_parked();
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
            assert!(window.try_find("overview-exact-table").is_none());
            view.update(cx, |this, cx| this.focus_main(window, cx));
            for _ in 0..10 {
                window.render_frame(cx);
                if window.find("overview-exact-toggle").focused() == Some(true) {
                    break;
                }
                window.press("tab", cx);
            }
            assert_eq!(window.find("overview-exact-toggle").focused(), Some(true));
            window.press("enter", cx);
            assert!(window.try_find("overview-exact-table").is_some());
            window.press("tab", cx);
            assert_eq!(window.find("overview-exact-11-06").focused(), Some(true));
            assert_eq!(
                window.find("overview-exact-11-06").value(),
                Some("11-06: Claude Code 0; Codex 2; Antigravity 0; OpenCode 0; Chart total 2"),
            );
            window.press("shift-tab", cx);
            assert_eq!(window.find("overview-exact-toggle").focused(), Some(true));
            window.press("enter", cx);
            assert!(window.try_find("overview-exact-table").is_none());
        })
        .unwrap_or_else(|e| panic!("overview exact values: {e}"));
        let seeded = cx.update(|cx| {
            view.read(cx)
                .report
                .clone()
                .unwrap_or_else(|| panic!("seeded report"))
        });
        for day_count in [30_u16, 90] {
            let start = chrono::NaiveDate::from_ymd_opt(2026, 8, 1)
                .unwrap_or_else(|| panic!("fixture date"));
            cx.update_window(handle, |_, window, cx| {
                view.update(cx, |this, cx| {
                    let mut report = seeded.as_ref().clone();
                    report.window_days = day_count;
                    report.days = (0..day_count)
                        .map(|index| WorkbenchDay {
                            label: (start + chrono::Duration::days(i64::from(index)))
                                .format("%m-%d")
                                .to_string(),
                            claude: 0,
                            codex: 1,
                            antigravity: 0,
                            opencode: 0,
                        })
                        .collect();
                    this.report = Some(Arc::new(report));
                    this.scroll.set_offset(Default::default());
                    this.exact_scroll.set_offset(Default::default());
                    cx.notify();
                });
                window.render_frame(cx);
                view.update(cx, |this, cx| this.focus_main(window, cx));
                for _ in 0..10 {
                    window.render_frame(cx);
                    if window.find("overview-exact-toggle").focused() == Some(true) {
                        break;
                    }
                    window.press("tab", cx);
                }
                assert_eq!(window.find("overview-exact-toggle").focused(), Some(true));
                window.press("enter", cx);
            })
            .unwrap_or_else(|e| panic!("long report: {e}"));
            for (index, key) in (0..day_count)
                .map(|index| (index, "tab"))
                .chain((0..day_count - 1).rev().map(|index| (index, "shift-tab")))
            {
                cx.update_window(handle, |_, window, cx| {
                    window.press(key, cx);
                    // Focus observers run after paint; render their requested
                    // scroll before checking the focused row's visible bounds.
                    window.render_frame(cx);
                    // 精确值表最新在上：第 index 个表行是倒数第 index 个日期。
                    let id = format!(
                        "overview-exact-{}",
                        (start + chrono::Duration::days(i64::from(day_count - 1 - index)))
                            .format("%m-%d")
                    );
                    let row = window.find(SharedString::from(id.clone()));
                    assert_eq!(row.focused(), Some(true), "{day_count}D {key}: {id}");
                    let row = row.bounds();
                    let table = window.find("overview-exact-table").bounds();
                    let page = window.find("overview-root").bounds();
                    assert!(
                        row.top() >= table.top().max(page.top())
                            && row.bottom() <= table.bottom().min(page.bottom()),
                        "{day_count}D {key}: {id} {row:?} outside table {table:?} / page {page:?}",
                    );
                })
                .unwrap_or_else(|e| panic!("exact row reveal: {e}"));
            }
            cx.update_window(handle, |_, window, cx| {
                window.press("shift-tab", cx);
                assert_eq!(window.find("overview-exact-toggle").focused(), Some(true));
                window.press("enter", cx);
                assert!(window.try_find("overview-exact-table").is_none());
            })
            .unwrap_or_else(|e| panic!("close long report: {e}"));
        }
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |this, cx| {
                let mut report = seeded.as_ref().clone();
                report.window_days = 7;
                report.platforms = vec![
                    WorkbenchPlatform {
                        id: "codex".into(),
                        requests: 4,
                        tokens: 10,
                        requests_by_day: vec![0, 0, 1, 0, 0, 3, 0],
                    },
                    WorkbenchPlatform {
                        id: "grok".into(),
                        requests: 9,
                        tokens: 3,
                        requests_by_day: Vec::new(),
                    },
                    WorkbenchPlatform {
                        id: "pi".into(),
                        requests: 0,
                        tokens: 0,
                        requests_by_day: vec![0; 7],
                    },
                ];
                report.days = [
                    "10-31", "11-01", "11-02", "11-03", "11-04", "11-05", "11-06",
                ]
                .into_iter()
                .zip([0, 0, 1, 0, 0, 3, 0])
                .map(|(label, codex)| WorkbenchDay {
                    label: label.into(),
                    claude: 0,
                    codex,
                    antigravity: 0,
                    opencode: 0,
                })
                .collect();
                this.report = Some(Arc::new(report));
                this.show_exact = false;
                this.scroll.set_offset(Default::default());
                this.exact_scroll.set_offset(Default::default());
                cx.notify();
            });
            window.render_frame(cx);
            // 页头：标题、范围切换与刷新同一行。
            let heading = window.find("overview-heading").bounds();
            for id in ["overview-window-7", "overview-refresh"] {
                let control = window.find(id).bounds();
                assert!(
                    control.top() < heading.bottom() && heading.top() < control.bottom(),
                    "{id} is not on the heading row: {control:?} vs {heading:?}"
                );
            }
            // KPI 四栏同一行；右栏与主列等高。
            assert_eq!(
                window.find("overview-total-cost").bounds().top(),
                window.find("overview-total-requests").bounds().top()
            );
            assert_eq!(
                window.find("overview-rail").bounds().bottom(),
                window.find("overview-usage").bounds().bottom()
            );
            assert!(window.find("overview-insights").visible());
            // 只有有逐日记录的平台单独成格；其余合并到“其他来源”并写明原因。
            assert_eq!(
                window.find("overview-codex-requests").value(),
                Some("Requests: 4")
            );
            assert!(window.try_find("overview-grok-requests").is_none());
            assert_eq!(
                window.find("overview-other-grok").label(),
                Some("Grok · 9 requests · daily split unavailable")
            );
            assert_eq!(
                window.find("overview-other-pi").label(),
                Some("Pi · no requests")
            );
            // 趋势图收紧到首个有数据日（11-02）的前一天。
            assert_eq!(
                window.find("overview-chart-since").label(),
                Some("since 11-01")
            );
            assert!(window.try_find("overview-day-10-31").is_none());
            assert!(window.try_find("overview-day-11-01").is_some());
            view.update(cx, |this, cx| this.focus_main(window, cx));
            for _ in 0..10 {
                window.render_frame(cx);
                if window.find("overview-exact-toggle").focused() == Some(true) {
                    break;
                }
                window.press("tab", cx);
            }
            assert_eq!(window.find("overview-exact-toggle").focused(), Some(true));
            window.press("enter", cx);
            // 最新在上，连续全零日折叠为一行，折叠行同样可聚焦。
            let rows = [
                ("overview-exact-quiet-11-06", "11-06 · no records"),
                (
                    "overview-exact-11-05",
                    "11-05: Claude Code 0; Codex 3; Antigravity 0; OpenCode 0; Chart total 3",
                ),
                (
                    "overview-exact-quiet-11-03",
                    "11-03 – 11-04 · no records (2 days)",
                ),
                (
                    "overview-exact-11-02",
                    "11-02: Claude Code 0; Codex 1; Antigravity 0; OpenCode 0; Chart total 1",
                ),
                (
                    "overview-exact-quiet-10-31",
                    "10-31 – 11-01 · no records (2 days)",
                ),
            ];
            for id in [
                "overview-exact-11-03",
                "overview-exact-11-04",
                "overview-exact-11-06",
            ] {
                assert!(window.try_find(id).is_none(), "{id} should be folded");
            }
            for (id, value) in rows {
                window.press("tab", cx);
                window.render_frame(cx);
                let row = window.find(id);
                assert_eq!(row.focused(), Some(true), "tab: {id}");
                assert_eq!(row.value(), Some(value));
            }
            for (id, _) in rows.iter().rev().skip(1) {
                window.press("shift-tab", cx);
                window.render_frame(cx);
                assert_eq!(window.find(*id).focused(), Some(true), "shift-tab: {id}");
            }
            window.press("shift-tab", cx);
            assert_eq!(window.find("overview-exact-toggle").focused(), Some(true));
            window.press("enter", cx);
            assert!(window.try_find("overview-exact-table").is_none());
        })
        .unwrap_or_else(|e| panic!("overview folded exact rows: {e}"));
        cx.update_window(handle, |_, window, cx| {
            view.update(cx, |this, cx| {
                this.report = Some(seeded);
                this.scroll.set_offset(Default::default());
                cx.notify();
            });
            window.render_frame(cx);
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
