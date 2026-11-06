//! Global monitoring uses explicit captured reads; hidden pages have no timer.
use crate::{
    locales::text,
    runtime::{
        Services,
        monitoring::{Error, Report, Requests},
    },
    shell::semantics,
    theme,
};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, TaskEvent};
use ccr_types::{MonitoringEntry, MonitoringFeedQuery, MonitoringLevel};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, Selectable,
    button::{Button, ButtonVariants},
    input::{Input, InputState},
};
use gpui_kit::{
    App, AppContext, Context, Entity, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Task, Window, div, px, uniform_list,
};
use std::sync::Arc;

pub struct MonitoringView {
    requests: Requests,
    channel: Entity<InputState>,
    level: Option<MonitoringLevel>,
    report: Option<Arc<Report>>,
    selected: Option<String>,
    generation: u64,
    task: Option<Task<()>>,
    busy: bool,
    error: Option<&'static str>,
    focus: FocusHandle,
}

pub fn storage_error(error: Error, cx: &App) -> &'static str {
    match error {
        Error::LocalOnly => text(
            cx,
            "Monitoring is available in Local only.",
            "全局监控仅支持本地环境。",
        ),
        Error::Missing => text(
            cx,
            "The main monitoring database does not exist. In-memory events remain available; no database was created.",
            "主监控数据库不存在。仍可查看内存事件；没有自动创建数据库。",
        ),
        Error::Storage => text(
            cx,
            "Cannot read or persist the captured main monitoring database. In-memory events remain available.",
            "无法读取或保存捕获的主监控数据库。仍可查看内存事件。",
        ),
    }
}

impl MonitoringView {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            requests: Requests::new(&services),
            channel: cx.new(|cx| InputState::new(window, cx)),
            level: None,
            report: None,
            selected: None,
            generation: 0,
            task: None,
            busy: false,
            error: None,
            focus: cx.focus_handle().tab_stop(false),
        }
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        window.focus_next(cx);
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.generation += 1;
        self.busy = false;
        self.report = None;
        self.selected = None;
        self.error = None;
        cx.notify();
    }
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.generation += 1;
        let generation = self.generation;
        let channel = self.channel.read(cx).value().trim().to_string();
        self.busy = true;
        self.error = None;
        self.selected = None;
        let query = MonitoringFeedQuery {
            count: Some(500),
            level: self.level,
            channel: (!channel.is_empty()).then_some(channel),
        };
        let key = RequestKey {
            environment: EnvironmentId::Local("monitoring".into()),
            platform: Platform::Claude,
            generation,
            request_id: generation,
        };
        match self.requests.refresh(key, query) {
            Ok(mut subscription) => {
                self.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if this.generation != generation {
                                    return;
                                }
                                this.busy = false;
                                match result.as_ref() {
                                    Completion::Success(report) => {
                                        this.report = Some(report.clone())
                                    }
                                    Completion::Failed(error)
                                    | Completion::CleanupFailed(error) => {
                                        this.error = Some(storage_error(*error, cx))
                                    }
                                    _ => {
                                        this.error = Some(text(
                                            cx,
                                            "Monitoring read interrupted. Refresh to retry.",
                                            "监控读取已中断，请刷新重试。",
                                        ))
                                    }
                                }
                                cx.notify();
                            });
                            break;
                        }
                    }
                }))
            }
            Err(_) => {
                self.busy = false;
                self.error = Some(text(
                    cx,
                    "Monitoring queue unavailable. Retry after other work finishes.",
                    "监控队列不可用，请在其他任务结束后重试。",
                ));
            }
        }
        cx.notify();
    }
    #[inline(never)]
    fn controls(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut levels = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
        for (level, id, en, zh) in [
            (None, "monitoring-level-all", "All levels", "全部级别"),
            (
                Some(MonitoringLevel::Debug),
                "monitoring-level-debug",
                "Debug",
                "调试",
            ),
            (
                Some(MonitoringLevel::Info),
                "monitoring-level-info",
                "Info",
                "信息",
            ),
            (
                Some(MonitoringLevel::Warn),
                "monitoring-level-warn",
                "Warning",
                "警告",
            ),
            (
                Some(MonitoringLevel::Error),
                "monitoring-level-error",
                "Error",
                "错误",
            ),
        ] {
            levels = levels.child(semantics::button(
                Button::new(id)
                    .label(text(cx, en, zh))
                    .ghost()
                    .selected(self.level == level)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.level = level;
                        this.refresh(cx);
                    })),
                self.busy,
                false,
                cx,
            ));
        }
        div()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(levels)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .child(div().child(text(cx, "Channel (empty = all)", "通道（留空查看全部）")))
                    .child(
                        div().flex_1().child(
                            Input::new(&self.channel)
                                .id("monitoring-channel")
                                .aria_label(text(cx, "Monitoring channel filter", "监控通道筛选")),
                        ),
                    )
                    .child(semantics::button(
                        Button::new("monitoring-refresh")
                            .label(text(cx, "Refresh monitoring", "刷新监控"))
                            .on_click(cx.listener(|this, _, _, cx| this.refresh(cx))),
                        self.busy,
                        self.busy,
                        cx,
                    )),
            )
            .into_any_element()
    }
    #[inline(never)]
    fn metrics(&self, cx: &App) -> gpui_kit::AnyElement {
        let mut line = div()
            .id("monitoring-metrics")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .py(px(theme::CONTROL_GAP))
            .border_t_1()
            .border_b_1()
            .border_color(cx.theme().border);
        if let Some(report) = &self.report {
            let metrics = &report.metrics;
            let rss = metrics
                .process_rss_bytes
                .map(|bytes| format!("{:.1} MiB", bytes as f64 / 1_048_576.))
                .unwrap_or_else(|| text(cx, "Unavailable", "不可用").into());
            line = line
                .child(format!(
                    "{}: {}    {}: {rss}",
                    text(
                        cx,
                        "Tracked operations (including this refresh)",
                        "持有的操作（包括本次刷新）"
                    ),
                    metrics.active_tasks,
                    text(cx, "Process resident memory", "进程驻留内存")
                ))
                .child(format!(
                    "{}: {} / 500    {}: {} / 524288    {}: {}",
                    text(cx, "Buffered events", "内存事件"),
                    metrics.retained_entries,
                    text(cx, "Payload bytes", "载荷字节"),
                    metrics.retained_payload_bytes,
                    text(cx, "Evicted events", "已淘汰事件"),
                    metrics.evicted_entries
                ));
        } else {
            line = line.child(text(
                cx,
                "Refresh to read runtime metrics.",
                "刷新后读取运行指标。",
            ));
        }
        line.child(div().text_size(px(theme::META_SIZE)).text_color(cx.theme().muted_foreground)
            .child(text(cx,"Command and database p95 latency are unavailable: this runtime has no measured latency histogram.","命令与数据库 p95 延迟不可用：当前运行时尚未采集耗时分布。"))).into_any_element()
    }
    fn row(&self, entry: &MonitoringEntry, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let id = entry.id.clone();
        let a11y_id = id.clone();
        let keyboard_id = id.clone();
        let weak = cx.weak_entity();
        let selected = self.selected.as_ref() == Some(&id);
        let label = format!(
            "{} {} {} {} {}",
            entry.timestamp,
            entry.level.as_str(),
            entry.channel,
            entry.event_type,
            entry.message
        );
        let level_color = match entry.level {
            MonitoringLevel::Error => cx.theme().danger,
            MonitoringLevel::Warn => cx.theme().warning,
            _ => cx.theme().foreground,
        };
        div()
            .id(SharedString::from(format!("monitoring-entry:{id}")))
            .test_support()
            .role(gpui_kit::Role::ListBoxOption)
            .aria_label(label)
            .aria_selected(selected)
            .tab_stop(true)
            .on_a11y_action(gpui_kit::AccessibleAction::Click, move |_, _, cx| {
                let _ = weak.update(cx, |this, cx| {
                    this.selected = Some(a11y_id.clone());
                    cx.notify();
                });
            })
            .h(px(theme::ROW_HEIGHT))
            .flex()
            .items_center()
            .gap(px(theme::CONTROL_GAP))
            .px(px(theme::CONTROL_GAP))
            .bg(if selected {
                cx.theme().secondary
            } else {
                cx.theme().background
            })
            .hover(|style| style.bg(cx.theme().muted))
            .focus(|style| style.border_2().border_color(cx.theme().ring))
            .on_key_down(
                cx.listener(move |this, event: &gpui_kit::KeyDownEvent, _, cx| {
                    if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                        this.selected = Some(keyboard_id.clone());
                        cx.notify();
                        cx.stop_propagation();
                    }
                }),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.selected = Some(id.clone());
                cx.notify();
            }))
            .child(
                div()
                    .w(px(180.))
                    .text_size(px(theme::META_SIZE))
                    .font_family(cx.theme().mono_font_family.clone())
                    .truncate()
                    .child(entry.timestamp.clone()),
            )
            .child(
                div()
                    .w(px(56.))
                    .text_color(level_color)
                    .child(entry.level.as_str()),
            )
            .child(div().w(px(100.)).truncate().child(entry.channel.clone()))
            .child(div().flex_1().truncate().child(entry.message.clone()))
            .into_any_element()
    }
    #[inline(never)]
    fn feed(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let entries = self
            .report
            .as_ref()
            .map_or(0, |report| report.entries.len());
        let mut feed = div().flex().flex_col().gap(px(theme::CONTROL_GAP)).child(
            div()
                .id("monitoring-feed-summary")
                .test_support()
                .child(format!(
                    "{}: {entries} / 500",
                    text(cx, "Visible events", "显示事件")
                )),
        );
        if entries == 0 && !self.busy {
            feed = feed.child(div().id("monitoring-empty").test_support().child(text(
                cx,
                "No events match these filters. Refresh after an operation finishes.",
                "没有符合筛选条件的事件。操作结束后可刷新查看。",
            )));
        }
        feed = feed.child(
            div()
                .id("monitoring-events-listbox")
                .role(gpui_kit::Role::ListBox)
                .aria_label(text(cx, "Monitoring events", "监控事件"))
                .child(
                    uniform_list(
                        "monitoring-feed",
                        entries,
                        cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                            range
                                .filter_map(|index| {
                                    this.report
                                        .as_ref()
                                        .and_then(|report| report.entries.get(index))
                                })
                                .map(|entry| this.row(entry, cx))
                                .collect()
                        }),
                    )
                    .h(px(320.))
                    .w_full(),
                ),
        );
        if let Some(entry) = self.report.as_ref().and_then(|report| {
            report
                .entries
                .iter()
                .find(|entry| self.selected.as_ref() == Some(&entry.id))
        }) {
            feed = feed.child(
                div()
                    .id("monitoring-detail")
                    .test_support()
                    .flex()
                    .flex_col()
                    .gap(px(theme::CONTROL_GAP))
                    .child(format!(
                        "{} · {} · {}",
                        entry.timestamp, entry.channel, entry.event_type
                    ))
                    .child(entry.message.clone()),
            );
        }
        feed.into_any_element()
    }
}
impl Render for MonitoringView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut root = div().id("global-monitoring").test_support().track_focus(&self.focus).tab_stop(false)
            .flex().flex_col().size_full().overflow_y_scroll().p(px(theme::CONTENT_PADDING)).gap(px(theme::SECTION_GAP))
            .child(div().text_size(px(theme::TITLE_SIZE)).font_weight(gpui_kit::FontWeight::SEMIBOLD).child(text(cx,"Global monitoring","全局监控")))
            .child(text(cx,"Captured main-database logs and this application's real lifecycle events. Refresh is explicit; hidden pages do not poll.","读取捕获的主数据库日志及本应用实际产生的生命周期事件。需要手动刷新，隐藏页面不轮询。"))
            .child(self.controls(cx)).child(self.metrics(cx));
        if let Some(error) = self.error {
            root = root.child(div().text_color(cx.theme().danger).child(error));
        }
        if let Some(report) = &self.report {
            if let Err(error) = report.storage {
                root = root.child(
                    div()
                        .id("monitoring-storage-error")
                        .test_support()
                        .text_color(cx.theme().warning)
                        .child(storage_error(error, cx)),
                );
            }
            if matches!(report.last_write, Some(Err(_))) {
                root = root.child(div().text_color(cx.theme().warning).child(text(
                    cx,
                    "The latest event could not be persisted. Its in-memory result is retained.",
                    "最新事件无法持久化，内存中的实际结果仍保留。",
                )));
            }
        }
        root.child(self.feed(cx))
    }
}

#[cfg(test)]
mod tests;
