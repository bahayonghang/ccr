//! Home workbench. Numbers come from the captured usage database and session archive.
mod activity;
mod chart;
mod header;
mod insights;
mod kpis;
mod platforms;
mod rail;

use crate::{
    runtime::insights::{InsightsRequests, WorkbenchReport},
    theme,
};
use ccr_app::runtime::{
    AdmissionError, Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent,
};
use chart::exact_rows;
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    AnyView, App, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Pixels,
    Render, ScrollHandle, Size, StatefulInteractiveElement, Styled, Subscription, Task, Window,
    div, point, px,
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
    heat_scroll: ScrollHandle,
    distribution: insights::Distribution,
    leaders: insights::Leaders,
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
            heat_scroll: ScrollHandle::new(),
            distribution: insights::Distribution::Hourly,
            leaders: insights::Leaders::Models,
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
                                        this.show_report(report.clone());
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

    /// 接受新报告。热力图放不下时默认露出最新一周：偏移由布局钳制到可滚范围，
    /// 放得下时自然归零。
    fn show_report(&mut self, report: std::sync::Arc<WorkbenchReport>) {
        self.report = Some(report);
        self.error = None;
        self.heat_scroll
            .set_offset(point(px(-activity::HEAT_MAX_SCROLL), px(0.)));
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
}

impl Render for OverviewView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let report = self.report.clone();
        let report = report.as_deref();
        let rows = report.map_or_else(Vec::new, |report| exact_rows(&report.days));
        self.sync_exact_focus(rows.len(), window, cx);
        let geometry = Geometry::new(window.viewport_size());
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
            .p(px(geometry.padding))
            .gap(px(theme::SECTION_GAP))
            .child(self.header(cx));
        if let Some(report) = report {
            root = root.child(self.kpis(report, geometry.stacked, cx));
            if let Some(band) = self.platforms(report, cx) {
                root = root.child(band);
            }
        }
        root = root.child(self.lower(report, &rows, geometry, cx));
        match report.and_then(|report| report.activity.as_ref()) {
            Some(activity) => root.child(self.insights_row(activity, geometry.stacked, cx)),
            None => root,
        }
    }
}

impl OverviewView {
    /// 主列与右栏等高；窄宽时右栏排到主列之下，整页不横向滚动。
    fn lower(
        &self,
        report: Option<&WorkbenchReport>,
        rows: &[chart::ExactRow],
        geometry: Geometry,
        cx: &mut Context<Self>,
    ) -> Node {
        let row = div()
            .id("overview-lower")
            .test_support()
            .flex()
            .flex_shrink_0()
            .gap(px(theme::SECTION_GAP));
        let row = if geometry.stacked {
            row.flex_col()
        } else {
            row.items_stretch()
        };
        let main = self.main_column(report, rows, geometry, cx);
        row.child(main).child(self.rail(report, geometry, cx))
    }

    fn main_column(
        &self,
        report: Option<&WorkbenchReport>,
        rows: &[chart::ExactRow],
        geometry: Geometry,
        cx: &mut Context<Self>,
    ) -> Node {
        let column = div()
            .id("overview-main")
            .test_support()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(theme::SECTION_GAP));
        let column = if geometry.stacked {
            column.w_full()
        } else {
            column.flex_1()
        };
        let column = column.child(self.usage_panel(report, rows, cx));
        match report {
            Some(report) => column.child(activity::activity_panel(
                report,
                geometry,
                &self.heat_scroll,
                cx,
            )),
            None => column,
        }
    }
}

/// Page layout numbers derived from the viewport. Pure, so tests can assert the
/// formula itself instead of a rendered side effect.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Geometry {
    /// 窄窗整页单列。
    stacked: bool,
    padding: f32,
    rail_w: f32,
    main_w: f32,
}

impl Geometry {
    fn new(viewport: Size<Pixels>) -> Self {
        let padding = theme::content_padding(viewport.width);
        // 侧栏、场景边距和纸面亮边都取外壳实际使用的主题函数。
        let content = f32::from(viewport.width)
            - theme::sidebar_width(viewport.width)
            - 2. * theme::scene_inset(viewport)
            - theme::SHEET_RIM
            - 2. * padding;
        let stacked = viewport.width < px(theme::NARROW_WIDTH);
        let rail_w = ((content * theme::OVERVIEW_RAIL_SHARE / 8.).round() * 8.)
            .clamp(theme::OVERVIEW_RAIL_MIN, theme::OVERVIEW_RAIL_MAX);
        Self {
            stacked,
            padding,
            rail_w,
            main_w: if stacked {
                content
            } else {
                content - rail_w - theme::SECTION_GAP
            },
        }
    }
}

/// 实色阅读面板：概览各模块共用同一描边、圆角和内边距。
fn panel(id: &'static str, cx: &App) -> Node {
    div()
        .id(id)
        .test_support()
        .flex()
        .flex_col()
        .gap(px(theme::SECTION_GAP))
        .p(px(theme::SECTION_GAP))
        .bg(cx.theme().background)
        .border_1()
        .border_color(cx.theme().border)
        .rounded(px(theme::PANEL_RADIUS))
}

fn heading(label: &'static str) -> gpui_kit::Div {
    div()
        .text_size(px(theme::SECTION_TITLE_SIZE))
        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
        .child(label)
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

#[cfg(test)]
mod tests;
