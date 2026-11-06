//! Explicit host observations, independent of the process RSS/feed refresh.
use crate::{
    locales::text,
    runtime::{
        Services,
        monitoring::system::{Error, Memory, Report},
    },
    shell::semantics,
    theme,
};
use ccr_app::runtime::{Completion, EnvironmentId, Platform, RequestKey, TaskEvent};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement,
    Styled, Task, Window, div, px,
};

pub(super) struct SystemView {
    services: Services,
    generation: u64,
    task: Option<Task<()>>,
    busy: bool,
    report: Option<Report>,
    error: Option<Error>,
    #[cfg(test)]
    synthetic: Option<(Report, std::time::Duration)>,
}
impl SystemView {
    pub fn new(services: Services, _cx: &mut Context<Self>) -> Self {
        Self {
            services,
            generation: 0,
            task: None,
            busy: false,
            report: None,
            error: None,
            #[cfg(test)]
            synthetic: None,
        }
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.generation += 1;
        self.task = None;
        self.busy = false;
        self.report = None;
        self.error = None;
        cx.notify();
    }
    fn finish(&mut self, generation: u64, result: Result<Report, Error>, cx: &mut Context<Self>) {
        if self.generation != generation {
            return;
        }
        self.task = None;
        self.busy = false;
        match result {
            Ok(report) => self.report = Some(report),
            Err(error) => self.error = Some(error),
        }
        cx.notify();
    }
    fn read(&mut self, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.generation += 1;
        let generation = self.generation;
        self.busy = true;
        self.report = None;
        self.error = None;
        let key = RequestKey {
            environment: EnvironmentId::Local("monitoring-host".into()),
            platform: Platform::Claude,
            generation,
            request_id: generation,
        };
        #[cfg(test)]
        let request = if let Some((report, delay)) = self.synthetic.take() {
            self.services.system_info_synthetic(key, report, delay)
        } else {
            self.services.system_info(key)
        };
        #[cfg(not(test))]
        let request = self.services.system_info(key);
        self.task = Some(cx.spawn(async move |weak, cx| {
            let result = match request {
                Err(_) => Err(Error::QueueBusy),
                Ok(mut subscription) => loop {
                    match subscription.next().await {
                        Some(TaskEvent::Terminal(result)) => {
                            break match result.as_ref() {
                                Completion::Success(report) => Ok(report.clone()),
                                Completion::Failed(error) | Completion::CleanupFailed(error) => {
                                    Err(*error)
                                }
                                Completion::Cancelled => Err(Error::Interrupted),
                                _ => Err(Error::Unavailable),
                            };
                        }
                        Some(_) => {}
                        None => break Err(Error::Interrupted),
                    }
                },
            };
            let _ = weak.update(cx, |this, cx| this.finish(generation, result, cx));
        }));
        cx.notify();
    }
    fn error_message(error: Error, cx: &gpui_kit::App) -> &'static str {
        match error {
            Error::LocalOnly => text(
                cx,
                "Host information is available in Local only.",
                "主机信息仅支持本地环境。",
            ),
            Error::Fixture => text(
                cx,
                "Host probing is disabled for fixture and benchmark sources.",
                "演示与基准来源禁止探测真实主机。",
            ),
            Error::Unsupported => text(
                cx,
                "Host information is unavailable on this system.",
                "当前系统不支持读取主机信息。",
            ),
            Error::Interrupted => text(
                cx,
                "Host read interrupted. Read again to retry.",
                "主机读取已中断，可再次读取重试。",
            ),
            Error::Unavailable => text(
                cx,
                "Host read failed. No sample is shown.",
                "主机读取失败，不显示采样值。",
            ),
            Error::QueueBusy => text(
                cx,
                "Read queue unavailable. Retry after other work finishes.",
                "读取队列不可用，请在其他任务结束后重试。",
            ),
        }
    }
    fn facts(report: &Report, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let missing = text(cx, "Unavailable", "不可用");
        let value = |value: &Option<String>| value.clone().unwrap_or_else(|| missing.into());
        let cpu = report
            .cpu_usage_percent
            .map(|v| format!("{v:.1}%"))
            .unwrap_or_else(|| missing.into());
        let count = report
            .logical_cpus
            .map(|n| n.to_string())
            .unwrap_or_else(|| missing.into());
        let uptime = report
            .uptime_seconds
            .map(|s| format!("{}d {}h {}m ({s} s)", s / 86400, s / 3600 % 24, s / 60 % 60))
            .unwrap_or_else(|| missing.into());
        let rows = [
            (
                "monitoring-host-sampled",
                format!(
                    "{} {} UTC · {} {} ms",
                    text(cx, "Sampled", "采样于"),
                    report.sampled_at.format("%Y-%m-%d %H:%M:%S"),
                    text(cx, "CPU observation interval", "CPU 观测间隔"),
                    report.cpu_sample_ms
                ),
            ),
            (
                "monitoring-host-os",
                format!(
                    "{}: {} · {}: {} · {}: {}",
                    text(cx, "OS", "操作系统"),
                    value(&report.os_name),
                    text(cx, "Version", "版本"),
                    value(&report.os_version),
                    text(cx, "Kernel", "内核"),
                    value(&report.kernel_version)
                ),
            ),
            (
                "monitoring-host-name",
                format!(
                    "{}: {} · {}: {}",
                    text(cx, "Host", "主机"),
                    value(&report.hostname),
                    text(cx, "App architecture", "应用架构"),
                    report.arch
                ),
            ),
            (
                "monitoring-host-cpu",
                format!(
                    "{}: {} · {}: {count} · {}: {cpu}",
                    text(cx, "CPU", "CPU"),
                    value(&report.cpu_brand),
                    text(cx, "Logical CPUs", "逻辑 CPU"),
                    text(cx, "Mean CPU usage", "CPU 平均使用率")
                ),
            ),
            (
                "monitoring-host-memory",
                format!(
                    "{}: {}",
                    text(cx, "Host RAM (used / total)", "主机内存（已用 / 总量）"),
                    memory(&report.memory, missing)
                ),
            ),
            (
                "monitoring-host-swap",
                format!(
                    "{}: {}",
                    text(
                        cx,
                        "Host swap (used / total)",
                        "主机交换空间（已用 / 总量）"
                    ),
                    memory(&report.swap, missing)
                ),
            ),
            (
                "monitoring-host-uptime",
                format!("{}: {uptime}", text(cx, "Host uptime", "主机运行时间")),
            ),
        ];
        let mut facts = div()
            .id("monitoring-host-facts")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP));
        for (id, value) in rows {
            facts = facts.child(
                div()
                    .id(id)
                    .test_support()
                    .aria_label(value.clone())
                    .text_size(px(theme::BODY_SIZE))
                    .child(value),
            );
        }
        facts.into_any_element()
    }
}
fn memory(memory: &Memory, missing: &str) -> String {
    let capacity = |value: Option<u64>| {
        value
            .map(|bytes| format!("{:.2} GiB", bytes as f64 / 1_073_741_824.))
            .unwrap_or_else(|| missing.into())
    };
    let percent = memory
        .used_percent()
        .map(|p| format!(" ({p:.1}%)"))
        .unwrap_or_default();
    format!(
        "{} / {}{percent}",
        capacity(memory.used_bytes),
        capacity(memory.total_bytes)
    )
}
impl Render for SystemView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut root=div().id("monitoring-host").test_support().flex_shrink_0().flex().flex_col().gap(px(theme::CONTROL_GAP))
            .child(semantics::button(Button::new("monitoring-host-read").label(if self.busy {text(cx,"Sampling host information…","正在采样主机信息…")} else {text(cx,"Read system information","读取系统信息")}).ghost().on_click(cx.listener(|this,_,_,cx|this.read(cx))),self.busy,self.busy,cx))
            .child(div().text_size(px(theme::META_SIZE)).text_color(cx.theme().muted_foreground).child(text(cx,"Local host running CCR Desktop. Explicit reads only; host RAM is separate from process RSS. Missing/zero capacity stays unavailable, including swap.","运行 CCR Desktop 的本地主机。仅手动读取；主机内存与进程 RSS 分开统计。缺失或报告为零的容量保持不可用，包括交换空间。")));
        if let Some(report) = &self.report {
            root = root.child(Self::facts(report, cx));
        } else {
            let message = self
                .error
                .map(|error| Self::error_message(error, cx))
                .unwrap_or_else(|| {
                    if self.busy {
                        text(
                            cx,
                            "Waiting for two CPU observations…",
                            "正在等待两次 CPU 观测…",
                        )
                    } else {
                        text(
                            cx,
                            "System information has not been read.",
                            "尚未读取系统信息。",
                        )
                    }
                });
            root = root.child(
                div()
                    .id("monitoring-host-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(message)
                    .child(message),
            );
        }
        root
    }
}

#[cfg(test)]
mod tests;
