//! Explicit local tool management embedded in the native installation page.
use crate::{
    locales::text,
    runtime::{
        Services,
        tools::{Action, Error, Requests, Snapshot, Status},
    },
    shell::semantics,
    theme,
};
use ccr_app::runtime::{EnvironmentId, Platform, RequestKey};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, FocusTrapElement,
    button::{Button, ButtonVariants},
};
use gpui_kit::{
    Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render,
    StatefulInteractiveElement, Styled, Task, Window, div, px,
};
use std::sync::Arc;

pub(super) struct ToolsView {
    requests: Requests,
    snapshot: Arc<Snapshot>,
    observer: Option<Task<()>>,
    error: Option<Error>,
    request: u64,
    decision: bool,
    decision_generation: u64,
    entry: bool,
    focus: FocusHandle,
    return_focus: Option<FocusHandle>,
}
impl ToolsView {
    pub fn new(services: &Services, cx: &mut Context<Self>) -> Self {
        let requests = Requests::new(services);
        Self {
            snapshot: requests.snapshot(),
            requests,
            observer: None,
            error: None,
            request: 0,
            decision: false,
            decision_generation: 0,
            entry: false,
            focus: cx.focus_handle().tab_stop(false),
            return_focus: None,
        }
    }
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.snapshot = self.requests.snapshot();
        if self.observer.is_none() {
            let mut updates = self.requests.observe();
            self.snapshot = updates.borrow_and_update().clone();
            self.observer = Some(cx.spawn(async move |weak, cx| {
                while updates.changed().await.is_ok() {
                    let snapshot = updates.borrow_and_update().clone();
                    if weak
                        .update(cx, |this, cx| {
                            this.snapshot = snapshot;
                            cx.notify();
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            }));
        }
        cx.notify();
    }
    pub fn suspend(&mut self, cx: &mut Context<Self>) {
        self.observer = None;
        self.decision = false;
        self.decision_generation += 1;
        self.entry = false;
        self.return_focus = None;
        cx.notify();
    }
    fn perform(&mut self, action: Action, confirmed: bool, cx: &mut Context<Self>) {
        self.refresh(cx);
        self.request += 1;
        self.error = self
            .requests
            .perform(
                RequestKey {
                    environment: EnvironmentId::Local("tools".into()),
                    platform: Platform::Claude,
                    generation: self.request,
                    request_id: self.request,
                },
                action,
                confirmed,
            )
            .err();
        self.snapshot = self.requests.snapshot();
        cx.notify();
    }
    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.decision = false;
        self.decision_generation += 1;
        self.entry = false;
        if let Some(focus) = self.return_focus.take() {
            window.focus(&focus, cx);
        }
        cx.notify();
    }
    fn controls(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let blocked = self.snapshot.status.active() || self.decision;
        let fixture = self.requests.is_fixture();
        let mut row = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
        for (action, id, en, zh) in [
            (
                Action::Versions,
                "tools-versions",
                "Check CLI versions",
                "检测 CLI 版本",
            ),
            (
                Action::DetectBridge,
                "tools-detect",
                "Detect SkillPort",
                "检测 SkillPort",
            ),
            (
                Action::OpenBridge,
                "tools-open",
                "Open SkillPort / skills-manage",
                "打开 SkillPort / skills-manage",
            ),
        ] {
            let missing_bridge = action == Action::OpenBridge
                && self.snapshot.bridge.as_ref().is_none_or(|b| !b.installed);
            row = row.child(semantics::button(
                Button::new(id)
                    .label(text(cx, en, zh))
                    .ghost()
                    .on_click(cx.listener(move |this, _, _, cx| this.perform(action, false, cx))),
                blocked || fixture || missing_bridge,
                false,
                cx,
            ));
        }
        row.child(semantics::button(
            Button::new("tools-review-update")
                .label(text(cx, "Review CCR CLI update", "核对 CCR CLI 更新"))
                .on_click(cx.listener(|this, _, window, cx| {
                    this.return_focus = window.focused(cx);
                    this.decision = true;
                    this.entry = true;
                    this.decision_generation += 1;
                    cx.notify();
                })),
            blocked,
            false,
            cx,
        ))
        .into_any_element()
    }
    fn confirmation(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        if std::mem::take(&mut self.entry) {
            let generation = self.decision_generation;
            let weak = cx.weak_entity();
            window.on_next_frame(move |window, cx| {
                let _ = weak.update(cx, |this, cx| {
                    if this.decision && this.decision_generation == generation {
                        window.focus(&this.focus, cx);
                        window.focus_next(cx);
                    }
                });
            });
        }
        div().id("tools-update-confirmation").test_support().role(gpui_kit::Role::AlertDialog)
            .aria_label(text(cx,"Confirm user CCR CLI update","确认用户 CCR CLI 更新"))
            .flex().flex_col().gap(px(theme::CONTROL_GAP))
            .child(text(cx,
                "Runs cargo install ccr --root <your home>/.cargo. Cargo and network access are required. This changes the user CLI in .cargo/bin; it does not update this desktop app or its verified bundled sidecar. A desktop located in that bin directory is refused. Continue?",
                "执行 cargo install ccr --root <用户目录>/.cargo，需要 Cargo 和网络。更新目标是 .cargo/bin 中的用户 CLI；桌面应用及其校验绑定的便携 sidecar 不在更新范围内。桌面若位于此 bin 目录将拒绝更新。是否继续？"))
            .child(div().tab_group().tab_stop(false).flex().gap(px(theme::CONTROL_GAP))
                .child(semantics::button(Button::new("tools-update-cancel").label(text(cx,"Cancel","取消")).ghost()
                    .on_click(cx.listener(|this,_,window,cx| this.dismiss(window,cx))),false,false,cx))
                .child(semantics::button(Button::new("tools-update-confirm").label(text(cx,"Update user CLI","更新用户 CLI"))
                    .on_click(cx.listener(|this,_,window,cx| {
                        this.dismiss(window,cx); this.perform(Action::UpdateCcr,true,cx);
                    })),self.requests.is_fixture(),false,cx))
                .focus_trap("tools-update-trap",&self.focus)).into_any_element()
    }
}
fn error_label(error: Error, cx: &gpui_kit::App) -> &'static str {
    match error {
        Error::LocalOnly => text(
            cx,
            "Tool management is local-only.",
            "工具管理仅支持本地环境。",
        ),
        Error::FixtureDisabled => text(
            cx,
            "Fixture mode cannot inspect or launch installed tools or update the CLI.",
            "夹具模式不检测或启动本机工具，也不更新 CLI。",
        ),
        Error::Busy => text(
            cx,
            "A tool operation is already running.",
            "已有工具操作正在运行。",
        ),
        Error::Confirmation => text(
            cx,
            "Review and confirm the user CLI update first.",
            "请先核对并确认用户 CLI 更新。",
        ),
        Error::Operation => text(
            cx,
            "The operation failed or was refused. Check Cargo, the installation scope and tool availability.",
            "操作失败或被拒绝，请检查 Cargo、安装作用域及工具是否可用。",
        ),
        Error::Interrupted => text(
            cx,
            "No terminal result was received. Success is unconfirmed.",
            "未收到终态，尚未确认成功。",
        ),
    }
}
impl Render for ToolsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status = match self.snapshot.status {
            Status::Idle => text(cx, "No tool operation has run.", "尚未执行工具操作。"),
            Status::Queued => text(cx, "Tool operation queued.", "工具操作已排队。"),
            Status::Running => text(
                cx,
                "Tool operation running; leaving this page keeps it running.",
                "工具操作正在运行，离页后仍继续执行。",
            ),
            Status::Finished if self.snapshot.action == Some(Action::OpenBridge) => text(
                cx,
                "Application launch requested; delivery is not confirmed.",
                "已请求启动应用，尚未确认实际送达。",
            ),
            Status::Finished => text(cx, "Tool operation finished.", "工具操作已完成。"),
            Status::Failed(error) => error_label(error, cx),
        };
        let mut root = div().id("native-tools").test_support().flex().flex_col().gap(px(theme::CONTROL_GAP))
            .on_key_down(cx.listener(|this,event: &gpui_kit::KeyDownEvent,window,cx| {
                if event.keystroke.key == "escape" && this.decision { this.dismiss(window,cx); cx.stop_propagation(); }
            }))
            .child(div().text_size(px(theme::SECTION_TITLE_SIZE)).child(text(cx,"CLI and companion tools","CLI 与配套工具")))
            .child(text(cx,"Installed CLI versions are self-reported. Bundled sidecar identity is checked separately by SHA-256. Automatic desktop updates remain disabled.","已安装 CLI 版本由工具自报告；便携 sidecar 身份另由 SHA-256 校验。桌面自动更新仍禁用。"))
            .child(self.controls(cx))
            .child(div().id("tools-status").test_support().role(gpui_kit::Role::Status).child(status));
        if self.requests.is_fixture() {
            root = root.child(error_label(Error::FixtureDisabled, cx));
        }
        if let Some(error) = self.error {
            root = root.child(
                div()
                    .text_color(cx.theme().danger)
                    .child(error_label(error, cx)),
            );
        }
        if let Some(bridge) = &self.snapshot.bridge {
            root = root.child(if !bridge.supported {
                text(
                    cx,
                    "SkillPort automatic launch is unsupported on this platform.",
                    "此平台不支持自动启动 SkillPort。",
                )
                .to_string()
            } else if bridge.installed {
                format!(
                    "SkillPort / skills-manage · {}",
                    text(cx, "Detected", "已检测到")
                )
            } else {
                text(
                    cx,
                    "SkillPort / skills-manage was not found.",
                    "未检测到 SkillPort / skills-manage。",
                )
                .into()
            });
        }
        for version in &self.snapshot.versions {
            root = root.child(format!(
                "{} · {} · {}",
                version.platform,
                version.status,
                version.version.as_deref().unwrap_or("—")
            ));
        }
        if let Some(output) = &self.snapshot.update_output {
            root = root.child(
                div()
                    .id("tools-update-output")
                    .max_h(px(120.))
                    .overflow_y_scroll()
                    .font_family(cx.theme().mono_font_family.clone())
                    .child(output.clone()),
            );
        }
        if self.decision {
            root = root.child(self.confirmation(window, cx));
        }
        root
    }
}
#[cfg(test)]
mod tests;
