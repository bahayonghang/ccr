//! Common collection patches. Private and unknown fields stay in the source owner.
use super::*;
use serde_json::{Value, json};

pub(super) struct CommonForm {
    fields: [Entity<InputState>; 2],
    multiline: Entity<TextareaState>,
    enabled: Option<bool>,
    pub advanced: bool,
}
impl CommonForm {
    pub fn new(window: &mut Window, cx: &mut Context<ClaudeResourcesView>) -> Self {
        Self {
            fields: std::array::from_fn(|_| cx.new(|cx| InputState::new(window, cx))),
            multiline: cx.new(|cx| TextareaState::new(window, cx)),
            enabled: None,
            advanced: false,
        }
    }
    pub fn clear(&mut self, window: &mut Window, cx: &mut Context<ClaudeResourcesView>) {
        for field in &self.fields {
            field.update(cx, |state, cx| state.set_value("", window, cx));
        }
        self.multiline
            .update(cx, |state, cx| state.set_value("", window, cx));
        self.enabled = None;
        self.advanced = false;
    }
    pub fn focus(&self, window: &mut Window, cx: &mut Context<ClaudeResourcesView>) {
        window.focus(&self.fields[0].read(cx).focus_handle(cx), cx);
    }
    pub fn patch(
        &self,
        kind: Kind,
        base: &str,
        cx: &Context<ClaudeResourcesView>,
    ) -> Result<String, Error> {
        let mut patch = serde_json::from_str::<Value>(base).map_err(|_| Error::Invalid)?;
        let object = patch.as_object_mut().ok_or(Error::Invalid)?;
        let first = self.fields[0].read(cx).value();
        let second = self.fields[1].read(cx).value();
        let multiline = self.multiline.read(cx).value();
        match kind {
            Kind::Agents => {
                if !first.is_empty() {
                    object.insert("model".into(), json!(first.to_string()));
                }
                if !second.is_empty() {
                    object.insert(
                        "tools".into(),
                        json!(
                            second
                                .split(',')
                                .map(str::trim)
                                .filter(|value| !value.is_empty())
                                .collect::<Vec<_>>()
                        ),
                    );
                }
                if !multiline.is_empty() {
                    object.insert("system_prompt".into(), json!(multiline.to_string()));
                }
            }
            Kind::SlashCommands => {
                if !first.is_empty() {
                    object.insert("description".into(), json!(first.to_string()));
                }
                if !second.is_empty() {
                    object.insert("command".into(), json!(second.to_string()));
                }
                if !multiline.is_empty() {
                    object.insert(
                        "args".into(),
                        json!(multiline.lines().map(str::to_string).collect::<Vec<_>>()),
                    );
                }
            }
            Kind::Plugins => {
                if !first.is_empty() {
                    object.insert("id".into(), json!(first.to_string()));
                }
                if !second.is_empty() {
                    object.insert("version".into(), json!(second.to_string()));
                }
            }
            _ => return Err(Error::Invalid),
        }
        if let Some(enabled) = self.enabled {
            object.insert(
                if kind == Kind::Plugins {
                    "enabled"
                } else {
                    "disabled"
                }
                .into(),
                json!(if kind == Kind::Plugins {
                    enabled
                } else {
                    !enabled
                }),
            );
        }
        serde_json::to_string(&patch).map_err(|_| Error::Invalid)
    }
    pub fn render(
        &self,
        kind: Kind,
        busy: bool,
        cx: &mut Context<ClaudeResourcesView>,
    ) -> gpui_kit::AnyElement {
        let labels = match kind {
            Kind::Agents => [
                ("Model", "模型"),
                ("Tools (comma separated)", "工具（逗号分隔）"),
            ],
            Kind::SlashCommands => [("Description", "描述"), ("Command", "命令")],
            _ => [("Plugin ID", "插件 ID"), ("Version", "版本")],
        };
        let mut root = div().id("claude-resource-form").test_support().flex_shrink_0().flex().flex_col().gap(px(theme::CONTROL_GAP))
            .child(text(cx, "Leave a field blank to retain its stored value. Existing credentials are never loaded. Advanced patch supports explicit clearing and extension fields.", "字段留空会保留已有值。已有凭据不会载入。显式清空和扩展字段可使用高级补丁。"));
        for (index, (en, zh)) in labels.into_iter().enumerate() {
            root = root.child(
                div().child(text(cx, en, zh)).child(
                    Input::new(&self.fields[index])
                        .id(SharedString::from(format!("claude-common-{index}")))
                        .aria_label(text(cx, en, zh))
                        .disabled(busy),
                ),
            );
        }
        if kind != Kind::Plugins {
            let label = if kind == Kind::Agents {
                text(cx, "System prompt", "系统提示词")
            } else {
                text(cx, "Arguments (one per line)", "参数（每行一个）")
            };
            root = root.child(
                div().child(label).child(
                    Textarea::new(&self.multiline)
                        .accessibility_id("claude-common-multiline")
                        .aria_label(label)
                        .h(px(theme::ROW_HEIGHT * 3.))
                        .disabled(busy),
                ),
            );
        }
        let mut state = div().flex().gap(px(theme::CONTROL_GAP));
        for (id, value, en, zh) in [
            (
                "claude-common-keep",
                None,
                "Keep enabled state",
                "保留启用状态",
            ),
            ("claude-common-enable", Some(true), "Enabled", "启用"),
            ("claude-common-disable", Some(false), "Disabled", "禁用"),
        ] {
            state = state.child(semantics::button(
                Button::new(id)
                    .label(text(cx, en, zh))
                    .ghost()
                    .toggled(self.enabled == value)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.form.enabled = value;
                        cx.notify();
                    })),
                busy,
                false,
                cx,
            ));
        }
        root.child(state).into_any_element()
    }
}

impl ClaudeResourcesView {
    pub(super) fn draft(&self, cx: &Context<Self>) -> Result<String, Error> {
        let value = self.editor.read(cx).value().to_string();
        if self.kind.is_file() || self.form.advanced {
            Ok(value)
        } else {
            self.form.patch(self.kind, &value, cx)
        }
    }
    pub(super) fn toggle_advanced(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.form.advanced {
            match self.draft(cx) {
                Ok(value) => {
                    self.form.clear(window, cx);
                    self.editor
                        .update(cx, |state, cx| state.set_value(value, window, cx));
                    self.form.advanced = true;
                }
                Err(error) => {
                    self.notice = Some(Notice::Error(error));
                    cx.notify();
                    return;
                }
            }
        } else {
            if !serde_json::from_str::<Value>(&self.editor.read(cx).value())
                .is_ok_and(|value| value.is_object())
            {
                self.notice = Some(Notice::Error(Error::Invalid));
                cx.notify();
                return;
            }
            self.form.advanced = false;
        }
        self.focus_editor(window, cx);
        cx.notify();
    }
    pub(super) fn focus_editor(&self, window: &mut Window, cx: &mut Context<Self>) {
        if self.kind.is_file() || self.form.advanced {
            window.focus(&self.editor.read(cx).focus_handle(cx), cx);
        } else {
            self.form.focus(window, cx);
        }
    }
}
