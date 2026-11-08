//! Home workbench. Numbers come from the captured usage database and session archive.
mod chart;
mod header;
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
use gpui_kit::{
    AnyView, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render,
    ScrollHandle, StatefulInteractiveElement, Styled, Subscription, Task, Window, div,
    prelude::FluentBuilder, px,
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
