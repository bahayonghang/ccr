//! Builtin and saved prompt templates feed the existing versioned instruction draft.
use crate::{
    locales::text,
    runtime::{
        Services,
        resources::templates::{Catalog, Command, Outcome},
    },
    shell::semantics,
    theme,
};
use ccr_app::{
    resources::prompt_presets::{Error, Mutation},
    runtime::{Completion, EnvironmentId, Platform, RequestKey, ResponseGate, TaskEvent},
};
use ccr_skills::{PromptPreset, PromptTarget};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    FocusTrapElement,
    button::{Button, ButtonVariants},
    input::{Input, InputState, Textarea, TextareaState},
};
use gpui_kit::{
    AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Task,
    Window, div, px,
};
use std::sync::Arc;

pub enum TemplateEvent {
    Use(String),
    Close,
}
enum Intent {
    Use(String),
    Save(PromptPreset),
    Delete(String),
}
pub struct TemplatesView {
    services: Services,
    platform: Platform,
    target: PromptTarget,
    catalog: Option<Arc<Catalog>>,
    name: Entity<InputState>,
    editor: Entity<TextareaState>,
    pending_text: Option<String>,
    selected: Option<PromptPreset>,
    selected_saved: bool,
    page: usize,
    busy: bool,
    gate: ResponseGate,
    generation: u64,
    task: Option<Task<()>>,
    error: Option<Error>,
    intent: Option<Intent>,
    enter_confirmation: bool,
    confirmation_focus: FocusHandle,
    focus: FocusHandle,
}
impl EventEmitter<TemplateEvent> for TemplatesView {}
impl TemplatesView {
    pub fn new(services: Services, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            services,
            platform: Platform::Claude,
            target: PromptTarget::Claude,
            catalog: None,
            name: cx.new(|cx| InputState::new(window, cx)),
            editor: cx.new(|cx| TextareaState::new(window, cx)),
            pending_text: None,
            selected: None,
            selected_saved: false,
            page: 0,
            busy: false,
            gate: ResponseGate::default(),
            generation: 0,
            task: None,
            error: None,
            intent: None,
            enter_confirmation: false,
            confirmation_focus: cx.focus_handle(),
            focus: cx.focus_handle(),
        }
    }
    pub fn focus_main(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self.intent.is_some() {
            window.focus(&self.confirmation_focus, cx);
        } else if self.selected.is_some() {
            window.focus(&self.editor.read(cx).focus_handle(cx), cx);
        } else {
            window.focus(&self.focus, cx);
            window.focus_next(cx);
        }
    }
    pub fn open(
        &mut self,
        platform: Platform,
        target: PromptTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.leave(window, cx);
        self.platform = platform;
        self.target = target;
        self.request(Command::List, cx);
        window.focus(&self.focus, cx);
    }
    pub fn leave(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.task = None;
        self.gate.clear();
        self.catalog = None;
        self.selected = None;
        self.intent = None;
        self.pending_text = None;
        self.busy = false;
        self.error = None;
        self.selected_saved = false;
        self.editor
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.name
            .update(cx, |state, cx| state.set_value("", window, cx));
        cx.notify();
    }
    fn request(&mut self, command: Command, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.generation += 1;
        let key = RequestKey {
            environment: EnvironmentId::Local(
                if self.services.is_fixture() {
                    "fixture"
                } else {
                    "local"
                }
                .into(),
            ),
            platform: self.platform,
            generation: self.generation,
            request_id: self.generation,
        };
        self.gate.replace(key.clone());
        self.busy = true;
        self.error = None;
        match self.services.template_request(key.clone(), command) {
            Ok(mut subscription) => {
                self.task = Some(cx.spawn(async move |weak, cx| {
                    while let Some(event) = subscription.next().await {
                        if let TaskEvent::Terminal(result) = event {
                            let _ = weak.update(cx, |this, cx| {
                                if !this.gate.accepts(&key) {
                                    return;
                                }
                                this.busy = false;
                                match result.as_ref() {
                                    Completion::Success(Outcome::Listed(catalog)) => {
                                        this.catalog = Some(catalog.clone())
                                    }
                                    Completion::Success(Outcome::Committed) => {
                                        this.selected = None;
                                        this.pending_text = Some(String::new());
                                        this.request(Command::List, cx);
                                    }
                                    Completion::Failed(error)
                                    | Completion::CleanupFailed(error) => this.error = Some(*error),
                                    _ => this.error = Some(Error::Storage),
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
                self.error = Some(Error::Storage);
            }
        }
        cx.notify();
    }
    fn select(
        &mut self,
        preset: PromptPreset,
        saved: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.busy {
            return;
        }
        self.name.update(cx, |state, cx| {
            state.set_value(preset.name.clone(), window, cx)
        });
        self.editor.update(cx, |state, cx| {
            state.set_value(preset.content.clone(), window, cx)
        });
        self.selected = Some(preset);
        self.selected_saved = saved;
        self.intent = None;
        cx.notify();
    }
    fn prepare(&mut self, intent: Intent, cx: &mut Context<Self>) {
        if self.busy {
            return;
        }
        self.intent = Some(intent);
        self.enter_confirmation = true;
        cx.notify();
    }
    fn confirm(&mut self, cx: &mut Context<Self>) {
        let Some(intent) = self.intent.take() else {
            return;
        };
        self.enter_confirmation = false;
        match intent {
            Intent::Use(content) => cx.emit(TemplateEvent::Use(content)),
            Intent::Save(preset) => {
                if let Some(catalog) = self.catalog.clone() {
                    self.request(
                        Command::Commit {
                            catalog,
                            mutation: Mutation::Upsert(preset),
                        },
                        cx,
                    );
                }
            }
            Intent::Delete(name) => {
                if let Some(catalog) = self.catalog.clone() {
                    self.request(
                        Command::Commit {
                            catalog,
                            mutation: Mutation::Delete(name),
                        },
                        cx,
                    );
                }
            }
        }
        cx.notify();
    }
}
impl Render for TemplatesView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(content) = self.pending_text.take() {
            self.editor
                .update(cx, |state, cx| state.set_value(content, window, cx));
        }
        let blocked = self.busy || self.intent.is_some();
        let mut root = div().id("prompt-templates").test_support().track_focus(&self.focus).on_key_down(cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" && this.intent.is_some() {
                    this.intent = None; this.enter_confirmation = false;
                    window.focus(&this.editor.read(cx).focus_handle(cx), cx); cx.stop_propagation(); cx.notify();
                }
            })).size_full().overflow_y_scroll().flex().flex_col().gap(px(theme::CONTROL_GAP))
            .child(div().flex_shrink_0().child(text(cx, "Builtin templates and saved prompt presets", "内置模板与已保存提示词预设")))
            .child(div().flex_shrink_0().child(text(cx, "Use a template to replace the open instruction draft, then review and save that file separately.", "使用模板替换当前指令草稿，随后检查并单独保存该文件。")))
            .child(semantics::button(Button::new("template-close").label(text(cx, "Back to instruction draft", "返回指令草稿")).ghost()
                .on_click(cx.listener(|_, _, _, cx| cx.emit(TemplateEvent::Close))), blocked, false, cx));
        if let Some(error) = self.error {
            let label = match error {
                Error::Conflict => text(
                    cx,
                    "The preset library changed. Your draft is retained. Close and reopen templates before retrying.",
                    "预设库已被修改，草稿已保留。请关闭并重新打开模板后重试。",
                ),
                Error::Invalid => text(
                    cx,
                    "Enter a valid preset name and content.",
                    "请输入有效的预设名称与内容。",
                ),
                _ => text(
                    cx,
                    "The preset operation failed. Your draft is retained.",
                    "预设操作失败，草稿已保留。",
                ),
            };
            root = root.child(
                div()
                    .id("template-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(label)
                    .flex_shrink_0()
                    .child(label),
            );
        }
        if self.selected.is_some() {
            root = root
                .child(
                    div().flex_shrink_0().child(
                        Input::new(&self.name)
                            .id("template-name")
                            .aria_label(text(cx, "Preset name", "预设名称"))
                            .disabled(blocked),
                    ),
                )
                .child(
                    div().flex_shrink_0().child(
                        Textarea::new(&self.editor)
                            .accessibility_id("template-content")
                            .aria_label(text(cx, "Template content", "模板内容"))
                            .disabled(blocked)
                            .h(px(240.)),
                    ),
                );
            if self.intent.is_some() {
                if self.enter_confirmation {
                    self.enter_confirmation = false;
                    window.focus(&self.confirmation_focus, cx);
                }
                let label = match self.intent {
                    Some(Intent::Use(_)) => text(
                        cx,
                        "Replace the open instruction draft with this content?",
                        "用此内容替换当前指令草稿？",
                    ),
                    Some(Intent::Delete(_)) => text(
                        cx,
                        "Delete this saved preset? The instruction file is unchanged.",
                        "删除此已保存预设？指令文件保持不变。",
                    ),
                    _ => text(
                        cx,
                        "Save this preset to the selected platform library? An existing preset with the same name will be updated.",
                        "将预设保存到所选平台的预设库？已有同名预设将更新。",
                    ),
                };
                root = root.child(div().flex_shrink_0().child(label)).child(
                    div()
                        .flex()
                        .flex_shrink_0()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(
                            Button::new("template-cancel")
                                .label(text(cx, "Cancel", "取消"))
                                .ghost()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.intent = None;
                                    cx.notify();
                                })),
                            false,
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new("template-confirm")
                                .label(text(cx, "Confirm", "确认"))
                                .primary()
                                .on_click(cx.listener(|this, _, _, cx| this.confirm(cx))),
                            false,
                            false,
                            cx,
                        ))
                        .focus_trap("template-confirmation-trap", &self.confirmation_focus),
                );
            } else {
                let mut actions = div()
                    .flex()
                    .flex_wrap()
                    .flex_shrink_0()
                    .gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(
                        Button::new("template-use")
                            .label(text(cx, "Use in instruction draft", "用于指令草稿"))
                            .primary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.prepare(
                                    Intent::Use(this.editor.read(cx).value().to_string()),
                                    cx,
                                )
                            })),
                        blocked,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("template-save")
                            .label(text(cx, "Save preset", "保存预设"))
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                let mut preset = this.selected.clone().unwrap_or(PromptPreset {
                                    name: String::new(),
                                    description: None,
                                    target_file: this.target.clone(),
                                    content: String::new(),
                                    tags: Vec::new(),
                                });
                                preset.name = this.name.read(cx).value().to_string();
                                preset.content = this.editor.read(cx).value().to_string();
                                preset.target_file = this.target.clone();
                                this.prepare(Intent::Save(preset), cx);
                            })),
                        blocked,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("template-browse")
                            .label(text(cx, "Browse templates", "浏览模板"))
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.selected = None;
                                this.pending_text = Some(String::new());
                                cx.notify();
                            })),
                        blocked,
                        false,
                        cx,
                    ));
                if self.selected_saved {
                    actions = actions.child(semantics::button(
                        Button::new("template-delete")
                            .label(text(cx, "Delete preset", "删除预设"))
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                if let Some(preset) = this.selected.as_ref() {
                                    this.prepare(Intent::Delete(preset.name.clone()), cx);
                                }
                            })),
                        blocked,
                        false,
                        cx,
                    ));
                }
                root = root.child(actions);
            }
        } else if let Some(catalog) = self.catalog.clone() {
            let mut presets: Vec<(PromptPreset, bool)> = catalog
                .builtin
                .iter()
                .map(|item| {
                    (
                        PromptPreset {
                            name: item.name.clone(),
                            description: Some(item.description.clone()),
                            target_file: self.target.clone(),
                            content: item.content.clone(),
                            tags: item.tags.clone(),
                        },
                        false,
                    )
                })
                .collect();
            presets.extend(
                catalog
                    .saved
                    .presets
                    .iter()
                    .filter(|preset| preset.target_file == self.target)
                    .cloned()
                    .map(|preset| (preset, true)),
            );
            let pages = presets.len().div_ceil(30).max(1);
            self.page = self.page.min(pages - 1);
            root = root.child(semantics::button(
                Button::new("template-new")
                    .label(text(cx, "New saved preset", "新建预设"))
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.select(
                            PromptPreset {
                                name: String::new(),
                                description: None,
                                target_file: this.target.clone(),
                                content: String::new(),
                                tags: Vec::new(),
                            },
                            false,
                            window,
                            cx,
                        );
                    })),
                blocked,
                false,
                cx,
            ));
            for (index, (preset, saved)) in presets
                .into_iter()
                .skip(self.page * 30)
                .take(30)
                .enumerate()
            {
                let label = format!(
                    "{} · {}",
                    preset.name,
                    if saved {
                        text(cx, "saved", "已保存")
                    } else {
                        text(cx, "builtin", "内置")
                    }
                );
                root = root.child(
                    div().flex_shrink_0().child(semantics::button(
                        Button::new(SharedString::from(format!("template-row-{index}")))
                            .label(label)
                            .ghost()
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.select(preset.clone(), saved, window, cx)
                            })),
                        blocked,
                        false,
                        cx,
                    )),
                );
            }
            root = root.child(
                div()
                    .flex()
                    .flex_shrink_0()
                    .gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(
                        Button::new("template-prev")
                            .label(text(cx, "Previous", "上一页"))
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.page = this.page.saturating_sub(1);
                                cx.notify();
                            })),
                        self.page == 0 || blocked,
                        false,
                        cx,
                    ))
                    .child(format!("{} / {}", self.page + 1, pages))
                    .child(semantics::button(
                        Button::new("template-next")
                            .label(text(cx, "Next", "下一页"))
                            .ghost()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.page += 1;
                                cx.notify();
                            })),
                        self.page + 1 >= pages || blocked,
                        false,
                        cx,
                    )),
            );
        }
        root
    }
}
