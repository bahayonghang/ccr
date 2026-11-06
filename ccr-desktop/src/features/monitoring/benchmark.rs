//! S2 controls remain inside Monitoring and exist only for the captured S2 source.
use crate::{locales::text, runtime::Services, shell::semantics, theme};
use ccr_app::{
    benchmark::s2::{Phase, Snapshot},
    runtime::{EnvironmentId, Platform, RequestKey},
};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{ActiveTheme, FocusTrapElement};
use gpui_kit::{
    Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Task, Window, div, px,
};
pub(super) struct BenchmarkView {
    services: Services,
    snapshot: Snapshot,
    task: Option<Task<()>>,
    review: Option<Review>,
    review_focus: FocusHandle,
    error: bool,
    generation: u64,
    watch_generation: u64,
}
struct Review {
    generation: u64,
    entry_pending: bool,
    armed: bool,
    return_focus: Option<FocusHandle>,
}
impl BenchmarkView {
    pub fn new(services: Services, cx: &mut Context<Self>) -> Self {
        let snapshot = services.benchmark_s2_status().unwrap_or_default();
        Self {
            services,
            snapshot,
            task: None,
            review: None,
            review_focus: cx.focus_handle().tab_stop(false),
            error: false,
            generation: 0,
            watch_generation: 0,
        }
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.task = None;
        self.watch_generation += 1;
        self.review = None;
        self.generation += 1;
        cx.notify();
    }
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        if let Ok(snapshot) = self.services.benchmark_s2_status() {
            self.snapshot = snapshot;
        }
        if self.task.is_none()
            && let Ok(mut changes) = self.services.benchmark_s2_observe()
        {
            self.watch_generation += 1;
            let generation = self.watch_generation;
            self.task = Some(cx.spawn(async move |weak, cx| {
                while changes.changed().await.is_ok() {
                    let snapshot = changes.borrow().clone();
                    let terminal = snapshot.phase.terminal();
                    let _ = weak.update(cx, |this, cx| {
                        if this.watch_generation != generation {
                            return;
                        }
                        if snapshot.revision >= this.snapshot.revision {
                            this.snapshot = snapshot;
                        }
                        cx.notify();
                    });
                    if terminal {
                        break;
                    }
                }
            }));
        }
        cx.notify();
    }
    fn ask_start(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.snapshot.phase != Phase::NotStarted || self.review.is_some() {
            return;
        }
        self.generation += 1;
        self.review = Some(Review {
            generation: self.generation,
            entry_pending: true,
            armed: false,
            return_focus: window.focused(cx),
        });
        cx.notify();
    }
    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(review) = self.review.take()
            && let Some(focus) = review.return_focus
        {
            window.focus(&focus, cx);
        }
        self.generation += 1;
        cx.notify();
    }
    fn confirm_start(&mut self, generation: u64, window: &mut Window, cx: &mut Context<Self>) {
        if !self
            .review
            .as_ref()
            .is_some_and(|review| review.generation == generation && review.armed)
            || self.snapshot.phase != Phase::NotStarted
        {
            return;
        }
        self.dismiss(window, cx);
        let key = RequestKey {
            environment: EnvironmentId::Local("benchmark-s2".into()),
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        };
        match self.services.benchmark_s2_start(key) {
            Ok(snapshot) => {
                self.snapshot = snapshot;
                self.error = false;
            }
            Err(_) => self.error = true,
        };
        self.refresh(cx);
    }
    fn cancel(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = &self.snapshot.run_id {
            match self.services.benchmark_s2_cancel(id) {
                Ok(snapshot) => self.snapshot = snapshot,
                Err(_) => self.error = true,
            }
        }
        cx.notify();
    }
    #[inline(never)]
    fn review(&mut self, window: &mut Window, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let Some(review) = self.review.as_mut() else {
            return div().into_any_element();
        };
        let generation = review.generation;
        if std::mem::take(&mut review.entry_pending) {
            let weak = cx.weak_entity();
            // This is focus setup after the panel mounts, never a present clock.
            window.on_next_frame(move |window, cx| {
                let _ = weak.update(cx, |this, cx| {
                    if let Some(review) = this.review.as_mut()
                        && review.generation == generation
                    {
                        review.armed = true;
                        window.focus(&this.review_focus, cx);
                        window.focus_next(cx);
                        cx.notify();
                    }
                });
            });
        }
        let armed = review.armed;
        div().id("benchmark-s2-review").test_support().role(gpui_kit::Role::AlertDialog).aria_label(text(cx,"Confirm synthetic stream","确认合成日志流"))
            .track_focus(&self.review_focus).tab_stop(false).flex().flex_col().gap(px(theme::CONTROL_GAP))
            .on_key_down(cx.listener(|this,event:&gpui_kit::KeyDownEvent,window,cx|{
                if event.keystroke.key=="escape"{this.dismiss(window,cx);cx.stop_propagation();}
                else if event.keystroke.key=="enter" && !this.review.as_ref().is_some_and(|review|review.armed){cx.stop_propagation();}
            }))
            .child(text(cx,"Start the fixed 60-second stream? This writes synthetic monitoring logs and consumes this process's single trial. Restore the seed after stopping owners before another trial.","启动固定 60 秒日志流？这会写入合成监控日志并消耗本进程唯一一次试验。再次试验前须停止所有 owner 并恢复 seed。"))
            .child(div().tab_group().tab_stop(false).flex().gap(px(theme::CONTROL_GAP))
                .child(semantics::button(Button::new("benchmark-s2-review-cancel").label(text(cx,"Cancel","取消")).ghost().on_click(cx.listener(|this,_,window,cx|this.dismiss(window,cx))),false,false,cx))
                .child(semantics::button(Button::new("benchmark-s2-review-confirm").label(text(cx,"Confirm start","确认启动")).on_click(cx.listener(move|this,_,window,cx|this.confirm_start(generation,window,cx))),!armed,false,cx))
                .focus_trap("benchmark-s2-review-focus",&self.review_focus)).into_any_element()
    }
}
fn rate(value: Option<f64>) -> String {
    value.map_or_else(|| "—".into(), |value| format!("{value:.1}/s"))
}
impl Render for BenchmarkView {
    #[inline(never)]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.review.is_some() {
            return self.review(window, cx);
        }
        let s = &self.snapshot;
        let active = matches!(s.phase, Phase::Queued | Phase::Running);
        let phase = match s.phase {
            Phase::NotStarted => text(cx, "Not started", "尚未开始"),
            Phase::Queued => text(cx, "Queued", "排队中"),
            Phase::Running => text(cx, "Running", "运行中"),
            Phase::Cleaning => text(
                cx,
                "Cleaning — waiting for actual IO and join",
                "清理中：等待实际 IO 与 join",
            ),
            Phase::Completed => text(cx, "Completed", "已完成"),
            Phase::Cancelled => text(cx, "Cancelled", "已取消"),
            Phase::Failed => text(cx, "Failed", "失败"),
        };
        let mut root=div().id("benchmark-s2-panel").test_support().flex().flex_col().gap(px(theme::CONTROL_GAP)).child(text(cx,"Synthetic S2 stream","合成 S2 日志流"))
            .child(text(cx,"Requested 1,000 slots/s for 60 s · queue 128 · persistent storage. Startup does not start the stream. One run per process; stop owners and restore the seed before a new trial.","请求 1,000 槽/秒，持续 60 秒 · 队列 128 · 真实持久化。启动不会自动运行。每进程仅一次；新试验须先停止所有 owner 并恢复 seed。"))
            .child(phase)
            .child(format!("{}: {} · {}: {} · {}: {} / {}",text(cx,"Attempted","实际尝试"),s.attempted,text(cx,"Skipped","跳过"),s.scheduler_skipped,text(cx,"Queue dropped / abandoned","队列丢弃 / 放弃"),s.queue_dropped,s.queue_abandoned))
            .child(format!("Ring {} / {} · evicted {} · DB {} / {} / {} · pending {} · emit {} / {}",s.ring_accepted,s.ring_rejected,s.ring_evicted,s.db_attempted,s.db_committed,s.db_failed,s.db_pending,s.emit_attempted,s.emit_failed))
            .child(format!("{}: {} · {} · {}",text(cx,"Actual attempted / delivered / committed","实际尝试 / 接收 / 提交率"),rate(s.actual_attempted_per_second),rate(s.actual_delivered_per_second),rate(s.actual_committed_per_second)))
            .child(text(cx,"Attempt rate uses real producer time. Delivery and commit rates include drain, final flush and join; report timestamps do not control pacing. No performance comparison is available.","尝试率使用真实 producer 时长；接收与提交率包含 drain、最终 flush 和 join。报告时间不控制调度。当前没有可比较的性能样本。"));
        if s.cleanup_over_budget {
            root = root.child(div().text_color(cx.theme().warning).child(text(
                cx,
                "Cleanup exceeded 500 ms; actual IO remains joined before completion.",
                "清理已超过 500 毫秒；完成前仍等待实际 IO 与 join。",
            )));
        }
        if self.error || s.failure_code.is_some() {
            root=root.child(div().text_color(cx.theme().danger).child(text(cx,"This trial failed. Review its counters and restore the seed before another process run.","本次试验失败，请核对计数并在新进程试验前恢复 seed。")));
        }
        root.child(
            div()
                .flex()
                .gap(px(theme::CONTROL_GAP))
                .child(
                    semantics::button(
                        Button::new("benchmark-s2-start"),
                        s.phase != Phase::NotStarted,
                        false,
                        cx,
                    )
                    .primary()
                    .label(text(cx, "Start synthetic stream", "启动合成日志流"))
                    .on_click(cx.listener(|this, _, window, cx| this.ask_start(window, cx))),
                )
                .child(
                    semantics::button(Button::new("benchmark-s2-cancel"), !active, false, cx)
                        .label(text(cx, "Cancel stream", "取消日志流"))
                        .on_click(cx.listener(|this, _, _, cx| this.cancel(cx))),
                ),
        )
        .into_any_element()
    }
}

#[cfg(test)]
mod tests;
