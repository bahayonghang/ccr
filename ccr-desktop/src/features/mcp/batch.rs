//! Multi-destination MCP review; only names, actions and safe outcomes are shown.
use crate::{
    locales::{service_error, text},
    runtime::{
        Services,
        mcp::{
            BatchAction, BatchInput, BatchOperation, BatchOutcome, BatchRow, McpBatchSession,
            McpError, McpTarget,
        },
    },
    shell::{badge, semantics},
    theme,
};
use ccr_app::runtime::{Completion, RequestKey, TaskEvent};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, FocusTrapElement, IconName,
    button::{Button, ButtonVariants},
    input::{Input, InputState},
};
use gpui_kit::{
    App, AppContext, Context, Div, Entity, EventEmitter, FocusHandle, InteractiveElement,
    IntoElement, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Task,
    UniformListScrollHandle, Window, div, px, uniform_list,
};
use std::{sync::Arc, time::Duration};

pub(super) struct Closed;
pub(super) struct McpBatch {
    services: Services,
    key: RequestKey,
    sync: bool,
    preset: String,
    source: usize,
    targets: [bool; 3],
    replace: bool,
    credential: Entity<InputState>,
    session: Option<Arc<McpBatchSession>>,
    pub(super) operation: Option<Arc<BatchOperation>>,
    task: Option<Task<()>>,
    busy: bool,
    message: Option<&'static str>,
    focus: FocusHandle,
    confirmation: FocusHandle,
    enter_confirmation: bool,
    review_focus: FocusHandle,
    scroll: UniformListScrollHandle,
}
impl EventEmitter<Closed> for McpBatch {}
fn target(index: usize) -> McpTarget {
    match index {
        1 => McpTarget::Codex,
        2 => McpTarget::Antigravity,
        _ => McpTarget::ClaudeUser,
    }
}
fn target_label(cx: &App, target: &McpTarget) -> &'static str {
    match target {
        McpTarget::ClaudeUser => text(cx, "Claude user", "Claude 用户"),
        McpTarget::ClaudeLocal(_) => text(cx, "Claude local", "Claude 本地"),
        McpTarget::ClaudeProject(_) => text(cx, "Claude project", "Claude 项目"),
        McpTarget::Codex => "Codex",
        McpTarget::Antigravity => "Antigravity",
    }
}
fn row_label(cx: &App, row: &BatchRow) -> String {
    let action = match row.action {
        BatchAction::Add => text(cx, "Add", "新增"),
        BatchAction::Replace => text(cx, "Replace", "替换"),
    };
    let state = match row.outcome {
        BatchOutcome::Pending => text(cx, "Pending", "待执行"),
        BatchOutcome::Saved => text(cx, "Saved", "已保存"),
        BatchOutcome::NotRun => text(cx, "Not run", "未执行"),
        BatchOutcome::Failed(error) => service_error(cx, error.message()),
    };
    format!(
        "{} · {} · {action} · {state}",
        target_label(cx, &row.target),
        row.name
    )
}
fn row_summary(cx: &App, row: &BatchRow) -> String {
    let action = match row.action {
        BatchAction::Add => text(cx, "Add", "新增"),
        BatchAction::Replace => text(cx, "Replace", "替换"),
    };
    let state = match row.outcome {
        BatchOutcome::Pending => text(cx, "Pending", "待执行"),
        BatchOutcome::Saved => text(cx, "Saved", "已保存"),
        BatchOutcome::NotRun => text(cx, "Not run", "未执行"),
        BatchOutcome::Failed(error) => match error {
            McpError::Conflict => text(cx, "Revision changed", "版本已变化"),
            McpError::Duplicate => text(cx, "Name exists", "名称已存在"),
            McpError::UnsupportedCopy => text(cx, "Unsupported options", "不兼容选项"),
            McpError::Storage => text(cx, "Read/write failed", "读写失败"),
            McpError::Missing => text(cx, "Entry missing", "条目缺失"),
            McpError::Scope => text(cx, "Invalid scope", "作用域无效"),
            McpError::Invalid => text(cx, "Invalid entry", "条目无效"),
            McpError::Unsupported | McpError::UnsupportedSource => {
                text(cx, "Source unavailable", "来源不可用")
            }
        },
    };
    format!(
        "{} · {action} · {state} · {}",
        target_label(cx, &row.target),
        row.name
    )
}
impl McpBatch {
    pub(super) fn new(
        services: Services,
        key: RequestKey,
        operation: Option<Arc<BatchOperation>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut this = Self {
            services,
            key,
            sync: false,
            preset: ccr_skills::get_builtin_presets()
                .first()
                .map(|preset| preset.id.clone())
                .unwrap_or_default(),
            source: 0,
            targets: [false, true, true],
            replace: false,
            credential: cx.new(|cx| InputState::new(window, cx).masked(true)),
            session: None,
            operation,
            task: None,
            busy: false,
            message: None,
            focus: cx.focus_handle().tab_stop(false),
            confirmation: cx.focus_handle(),
            enter_confirmation: false,
            review_focus: cx.focus_handle().tab_stop(false),
            scroll: UniformListScrollHandle::new(),
        };
        this.observe(cx);
        this
    }
    fn observe(&mut self, cx: &mut Context<Self>) {
        let Some(operation) = self.operation.clone() else {
            return;
        };
        if !operation.snapshot().running {
            return;
        }
        self.task = Some(cx.spawn(async move |weak, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
                let done = !operation.snapshot().running;
                if weak.update(cx, |_, cx| cx.notify()).is_err() || done {
                    break;
                }
            }
        }));
    }
    fn prepare(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.busy
            || self
                .operation
                .as_ref()
                .is_some_and(|op| op.snapshot().running)
        {
            return;
        }
        let input = if self.sync {
            BatchInput::Sync {
                source: target(self.source),
            }
        } else {
            BatchInput::Preset {
                id: self.preset.clone(),
                api_key: Some(self.credential.read(cx).value().to_string()),
            }
        };
        let targets = self
            .targets
            .iter()
            .enumerate()
            .filter_map(|(index, selected)| {
                if *selected && (!self.sync || index != self.source) {
                    Some(target(index))
                } else {
                    None
                }
            })
            .collect();
        self.key.request_id += 1;
        self.busy = true;
        self.message = None;
        match self.services.mcp_batch_prepare_request(
            self.key.clone(),
            input,
            targets,
            self.replace,
        ) {
            Ok(mut read) => {
                self.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = read.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                this.busy = false;
                                match result.as_ref() {
                                    Completion::Success(session) => {
                                        this.session = Some(session.clone());
                                        this.enter_confirmation = true;
                                    }
                                    Completion::Failed(error)
                                    | Completion::CleanupFailed(error) => {
                                        this.message = Some(error.message())
                                    }
                                    _ => {
                                        this.message =
                                            Some("Batch review did not finish. / 批量检查未完成")
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
                self.message = Some("Queue unavailable. Retry. / 队列不可用，请重试");
            }
        }
        cx.notify();
    }
    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.session = None;
        self.enter_confirmation = false;
        let request_id = self.key.request_id;
        let weak = cx.weak_entity();
        window.on_next_frame(move |window, cx| {
            let _ = weak.update(cx, |this, cx| {
                if this.key.request_id == request_id
                    && this.session.is_none()
                    && this.operation.is_none()
                {
                    window.focus(&this.review_focus, cx);
                    window.focus_next(cx);
                }
            });
        });
        cx.notify();
    }
    fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(session) = self.session.take() else {
            return;
        };
        self.key.request_id += 1;
        match self
            .services
            .mcp_batch_save_request(self.key.clone(), session.clone())
        {
            Ok(operation) => {
                self.operation = Some(operation);
                self.credential
                    .update(cx, |state, cx| state.set_value("", window, cx));
                self.observe(cx);
                window.focus(&self.focus, cx);
            }
            Err(_) => {
                self.session = Some(session);
                self.message = Some("Queue unavailable. Retry. / 队列不可用，请重试");
            }
        }
        cx.notify();
    }
    fn rows(&self) -> Vec<BatchRow> {
        self.session
            .as_ref()
            .map(|session| session.preview().to_vec())
            .or_else(|| {
                self.operation
                    .as_ref()
                    .map(|operation| operation.snapshot().rows)
            })
            .unwrap_or_default()
    }
    fn setup(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut root = div().flex().flex_col().gap(px(theme::CONTROL_GAP));
        // 两种模式是互斥的分段选项：胶囊 + tab 语义，选中态由 toggled 报告。
        root = root.child(
            crate::shell::page_header::capsule(cx)
                .child(mode_tab(
                    "mcp-batch-mode-preset",
                    text(cx, "Install a preset", "安装预设"),
                    false,
                    self,
                    cx,
                ))
                .child(mode_tab(
                    "mcp-batch-mode-sync",
                    text(cx, "Sync every source entry", "同步来源全部条目"),
                    true,
                    self,
                    cx,
                )),
        );
        if self.sync {
            root = root.child(text(cx, "Source (user scope)", "来源（用户作用域）"));
            // pane 最窄 320px：选项行允许换行，不横向溢出。
            let mut sources = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
            for index in 0..3 {
                sources = sources.child(badge::filter_chip(
                    Button::new(SharedString::from(format!("mcp-batch-source-{index}")))
                        .label(target_label(cx, &target(index)))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.source = index;
                            cx.notify();
                        })),
                    self.source == index,
                    self.busy,
                    cx,
                ));
            }
            root = root.child(sources);
        } else {
            root = root.child(
                div()
                    .text_size(crate::theme::text_size(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(text(cx, "Preset", "预设")),
            );
            let mut presets = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
            for preset in ccr_skills::get_builtin_presets() {
                let id = preset.id.clone();
                let pressed = self.preset == id;
                presets = presets.child(badge::filter_chip(
                    Button::new(SharedString::from(format!("mcp-batch-preset-{id}")))
                        .label(preset.name)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.preset = id.clone();
                            cx.notify();
                        })),
                    pressed,
                    self.busy,
                    cx,
                ));
            }
            root = root.child(presets);
            if let Some(key) = ccr_skills::get_builtin_presets()
                .into_iter()
                .find(|preset| preset.id == self.preset)
                .and_then(|preset| preset.api_key_env)
            {
                root = root
                    .child(format!(
                        "{}: {key}",
                        text(cx, "Required API key", "必填 API 密钥")
                    ))
                    .child(
                        Input::new(&self.credential)
                            .id("mcp-batch-credential")
                            .aria_label(text(cx, "Preset API key", "预设 API 密钥"))
                            .disabled(self.busy),
                    );
            }
        }
        root = root.child(text(cx, "Destinations", "目标"));
        let mut targets = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
        for index in 0..3 {
            if self.sync && self.source == index {
                continue;
            }
            targets = targets.child(badge::filter_chip(
                Button::new(SharedString::from(format!("mcp-batch-target-{index}")))
                    .label(target_label(cx, &target(index)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.targets[index] = !this.targets[index];
                        cx.notify();
                    })),
                self.targets[index],
                self.busy,
                cx,
            ));
        }
        root = root
            .child(targets)
            .child(
                div()
                    .text_size(crate::theme::text_size(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(text(cx, "Options", "选项")),
            )
            .child(
                badge::filter_chip(
                    Button::new("mcp-batch-replace")
                        .label(text(cx, "Replace existing entries", "替换已有条目"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.replace = !this.replace;
                            cx.notify();
                        })),
                    self.replace,
                    self.busy,
                    cx,
                )
                .self_start(),
            );
        // 按钮标签单行不换行：后果说明拆成下方的 muted 辅助句，窄 pane 不溢出。
        root.child(
            div()
                .text_size(crate::theme::text_size(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(text(
                    cx,
                    "Replacing also removes that entry's custom options.",
                    "替换会同时移除该条目的自定义选项。",
                )),
        )
        .into_any_element()
    }
}
impl McpBatch {
    /// 标题与“返回 MCP 列表”同一行：分栏 pane 较矮时，退出操作不落到折叠线以下。
    fn heading(&self, cx: &mut Context<Self>) -> Div {
        let row = div()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap(px(theme::CONTROL_GAP))
            .child(
                div()
                    .min_w_0()
                    .text_size(crate::theme::text_size(theme::SECTION_TITLE_SIZE))
                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                    .child(text(cx, "MCP batch operations", "MCP 批量操作")),
            );
        // 确认区挂载时只保留确认区自己的返回，焦点陷阱之外不再放第二个出口。
        if self.session.is_some() {
            return row;
        }
        row.child(semantics::button(
            Button::new("mcp-batch-close")
                .icon(IconName::ArrowLeft)
                .label(text(cx, "Back to MCP list", "返回 MCP 列表"))
                .ghost()
                .on_click(cx.listener(|_, _, _, cx| cx.emit(Closed))),
            self.busy,
            false,
            cx,
        ))
    }
}
// 选项构建放在独立函数中，避免增大 render 栈帧（Windows UI 线程只有 1 MiB）。
fn mode_tab(
    id: &'static str,
    label: &'static str,
    sync: bool,
    batch: &McpBatch,
    cx: &mut Context<McpBatch>,
) -> Button {
    semantics::tab(
        semantics::button(
            Button::new(id)
                .label(label)
                .ghost()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.sync = sync;
                    cx.notify();
                })),
            batch.busy,
            false,
            cx,
        ),
        batch.sync == sync,
        batch.busy,
        cx,
    )
}
impl Render for McpBatch {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let running = self
            .operation
            .as_ref()
            .is_some_and(|operation| operation.snapshot().running);
        let mut root=div().id("mcp-batch").test_support().track_focus(&self.focus).tab_stop(false).flex().flex_col().gap(px(theme::CONTROL_GAP))
            .child(self.heading(cx))
            .child(div().text_size(crate::theme::text_size(theme::META_SIZE)).text_color(cx.theme().muted_foreground).child(text(cx,"Review each destination before writing. Entries with unsupported options remain unchanged. Each destination commits once; successful destinations remain saved if another fails. No server process starts.","写入前逐项检查目标。含不兼容选项的条目保持原样。每个目标一次提交；其他目标失败不会回滚已保存的目标。不会启动服务器进程。")));
        if let Some(message) = self.message {
            root = root.child(
                div()
                    .id("mcp-batch-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .child(service_error(cx, message)),
            );
        }
        if self.operation.is_none() {
            root = root.child(
                div()
                    .id("mcp-batch-review-actions")
                    .track_focus(&self.review_focus)
                    .tab_stop(false)
                    .flex()
                    .child(semantics::button(
                        Button::new("mcp-batch-review")
                            .label(text(cx, "Review batch", "检查批量更改"))
                            .primary()
                            .on_click(cx.listener(|this, _, window, cx| this.prepare(window, cx))),
                        self.busy || self.session.is_some(),
                        self.busy,
                        cx,
                    )),
            );
        }
        let rows = self.rows();
        // 决策控件（检查、确认、结果状态）在设置选项与逐行列表之前：分栏 pane 较矮时，
        // 主按钮与焦点所在的确认按钮不落到折叠线以下。
        if self.session.is_some() {
            if self.enter_confirmation {
                self.enter_confirmation = false;
                let request_id = self.key.request_id;
                let weak = cx.weak_entity();
                window.on_next_frame(move |window, cx| {
                    let _ = weak.update(cx, |this, cx| {
                        if this.session.is_some() && this.key.request_id == request_id {
                            window.focus(&this.confirmation, cx);
                            window.focus_next(cx);
                        }
                    });
                });
            }
            let can_save = rows.iter().any(|row| row.outcome == BatchOutcome::Pending);
            let confirmation=div().id("mcp-batch-confirmation").test_support().on_key_down(cx.listener(|this,event:&gpui_kit::KeyDownEvent,window,cx|{if event.keystroke.key=="escape"{this.dismiss(window,cx);cx.stop_propagation();}})).flex().flex_col().gap(px(theme::CONTROL_GAP))
                .child(text(cx,"Confirm these Add/Replace actions? Replace discards all existing options for that entry. Unsupported and duplicate entries are skipped; credentials are copied without displaying them.","确认这些新增/替换操作？替换会移除该条目的所有旧选项。不兼容与重名条目将跳过；凭据会复制但不会显示。"))
                .child(div().flex().gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(Button::new("mcp-batch-back").label(text(cx,"Keep editing","继续编辑")).ghost().on_click(cx.listener(|this,_,window,cx|{this.dismiss(window,cx);})),false,false,cx))
                    .child(semantics::button(Button::new("mcp-batch-confirm").label(text(cx,"Confirm batch writes","确认批量写入")).primary().on_click(cx.listener(|this,_,window,cx|this.save(window,cx))),!can_save,false,cx)).focus_trap("mcp-batch-trap",&self.confirmation));
            root = root.child(confirmation);
        } else if self.operation.is_some() {
            root=root.child(div().id("mcp-batch-operation-status").test_support().role(gpui_kit::Role::Status).child(if running{text(cx,"Writing reviewed destinations. Leaving this page does not cancel committed work.","正在写入已检查的目标。离页不会取消已开始的操作。")}else{text(cx,"Batch finished. Check every row; a failed destination does not undo saved destinations.","批量操作已结束。请检查每行结果；失败目标不会回滚已保存目标。") }));
            root = root.child(
                semantics::button(
                    Button::new("mcp-batch-new")
                        .label(text(cx, "Review another batch", "检查另一批更改"))
                        .ghost()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.operation = None;
                            cx.notify();
                        })),
                    running,
                    false,
                    cx,
                )
                .self_start(),
            );
        }
        if self.session.is_none() && self.operation.is_none() {
            root = root.child(self.setup(cx));
        }
        if self.session.is_some() || self.operation.is_some() {
            let saved = rows
                .iter()
                .filter(|row| row.outcome == BatchOutcome::Saved)
                .count();
            root = root.child(format!(
                "{}: {} · {}: {saved}",
                text(cx, "Entries", "条目"),
                rows.len(),
                text(cx, "Saved", "已保存")
            ));
            if rows.is_empty() {
                root = root.child(text(
                    cx,
                    "The selected source has no entries. Nothing will be written.",
                    "所选来源没有条目，不会写入。",
                ));
            }
            let list = uniform_list(
                "mcp-batch-rows",
                rows.len(),
                cx.processor(|this, range: std::ops::Range<usize>, _, cx| {
                    let rows = this.rows();
                    range
                        .filter_map(|index| {
                            rows.get(index).map(|row| {
                                div()
                                    .id(SharedString::from(format!("mcp-batch-row-{index}")))
                                    .test_support()
                                    .aria_label(row_label(cx, row))
                                    .h(crate::theme::text_size(theme::ROW_HEIGHT))
                                    .min_w_0()
                                    .text_ellipsis()
                                    .child(row_summary(cx, row))
                            })
                        })
                        .collect::<Vec<_>>()
                }),
            )
            .h(px(theme::ROW_HEIGHT * 6.))
            .track_scroll(&self.scroll);
            root = root.child(list);
            let mut reasons = Vec::new();
            for row in &rows {
                if let BatchOutcome::Failed(error) = row.outcome
                    && !reasons.contains(&error)
                {
                    reasons.push(error);
                    root = root.child(
                        div()
                            .text_size(crate::theme::text_size(theme::META_SIZE))
                            .child(service_error(cx, error.message())),
                    );
                }
            }
        }
        root.text_color(cx.theme().foreground)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{features::mcp::McpView, runtime::ConfigSource};
    use ccr_app::runtime::RuntimeOwner;
    use gpui_kit::{
        TestAppContext, WindowOptions,
        test::{TestAppContextExt, TestWindowExt},
    };

    #[gpui_kit::test]
    async fn rendered_batch_review_focus_escape_confirm_and_navigation_keep_safe_receipt(
        cx: &mut TestAppContext,
    ) {
        cx.executor().allow_parking();
        let owner = RuntimeOwner::new().unwrap();
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join("desktop-mcp-fixture/home/.claude.json");
        std::fs::create_dir_all(state.parent().unwrap()).unwrap();
        std::fs::write(&state,r#"{"mcpServers":{"fixture":{"command":"fixture-tool","env":{"TOKEN":"batch-control-secret-sentinel"}}}}"#).unwrap();
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: root.path().join("profiles.toml"),
                registry: root.path().join("registry.toml"),
                delay: Duration::ZERO,
            },
        );
        let (handle, view) = cx.update(|cx| {
            gpui_kit::init(cx);
            gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| {
                cx.new(|cx| McpView::new(services, window, cx))
            })
            .unwrap()
        });
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("mcp-batch-open", cx);
            window.render_frame(cx);
            window.click("mcp-batch-mode-sync", cx);
            window.render_frame(cx);
            window.click("mcp-batch-review", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx)
                .batch
                .as_ref()
                .is_some_and(|batch| batch.read(cx).session.is_some())
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(window.find("mcp-batch-back").focused(), Some(true));
            window.press("tab", cx);
            assert_eq!(window.find("mcp-batch-confirm").focused(), Some(true));
            window.press("shift-tab", cx);
            assert_eq!(window.find("mcp-batch-back").focused(), Some(true));
            let labels = gpui_kit::base::test_support::snapshots(window)
                .into_iter()
                .filter_map(|node| node.label().map(str::to_owned))
                .collect::<Vec<_>>()
                .join("\n");
            assert!(!labels.contains("batch-control-secret-sentinel"));
            window.press("escape", cx);
            window.render_frame(cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            assert_eq!(window.find("mcp-batch-review").focused(), Some(true));
            assert!(
                !root
                    .path()
                    .join("desktop-mcp-fixture/home/.codex/config.toml")
                    .exists()
            );
            window.click("mcp-batch-review", cx);
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx)
                .batch
                .as_ref()
                .is_some_and(|batch| batch.read(cx).session.is_some())
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.simulate_next_frame(cx);
            window.render_frame(cx);
            window.click("mcp-batch-confirm", cx);
            view.update(cx, |view, cx| view.suspend(cx));
            assert!(view.read(cx).batch.is_none());
            assert!(view.read(cx).last_batch.is_some());
        })
        .unwrap();
        cx.wait_for(handle, Duration::from_secs(10), |_, cx| {
            view.read(cx)
                .last_batch
                .as_ref()
                .is_some_and(|operation| !operation.snapshot().running)
        })
        .await;
        cx.update_window(handle, |_, window, cx| {
            window.render_frame(cx);
            window.click("mcp-batch-open", cx);
            window.render_frame(cx);
            let operation = view
                .read(cx)
                .batch
                .as_ref()
                .unwrap()
                .read(cx)
                .operation
                .as_ref()
                .unwrap()
                .snapshot();
            assert_eq!(operation.rows.len(), 2);
            assert!(
                operation
                    .rows
                    .iter()
                    .all(|row| row.outcome == BatchOutcome::Saved)
            );
            assert!(!format!("{operation:?}").contains("batch-control-secret-sentinel"));
            window.remove_window();
        })
        .unwrap();
        owner.shutdown();
    }
}
