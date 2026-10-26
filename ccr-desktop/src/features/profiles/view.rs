use super::*;
use crate::theme;
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, FocusTrapElement, Selectable,
    button::{Button, ButtonVariants},
    input::{Input, Textarea},
};
use gpui_kit::{
    InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled,
    div, px,
};
impl Render for ProfileTools {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.clear_sensitive {
            self.clear_sensitive = false;
            self.original = None;
            self.revision.clear();
            self.pending_content = None;
            self.advanced = false;
            for input in [
                &self.name,
                &self.description,
                &self.model,
                &self.credential,
                &self.base_url,
                &self.auth_mode,
            ] {
                input.update(cx, |state, cx| state.set_value("", window, cx));
            }
            self.content
                .update(cx, |state, cx| state.set_value("", window, cx));
        }
        if let Some(content) = self.pending_content.take() {
            self.content
                .update(cx, |state, cx| state.set_value(content, window, cx));
        }
        let mut root = div()
            .id("profile-tools")
            .flex()
            .flex_col()
            .min_h_0()
            .gap(px(theme::CONTROL_GAP))
            .track_focus(&self.focus)
            .tab_stop(false)
            .on_key_down(
                cx.listener(|this, event: &gpui_kit::KeyDownEvent, window, cx| {
                    if event.keystroke.key == "escape" && !this.busy {
                        if let Some(confirm) = this.confirmation.take() {
                            if let Some(focus) = confirm.return_focus {
                                window.focus(&focus, cx);
                            }
                            cx.notify();
                        } else {
                            this.close(window, cx);
                        }
                        cx.stop_propagation();
                    }
                }),
            );
        if let Some(confirm) = &self.confirmation {
            let subject = confirm.subject.as_str();
            let message=match confirm.action {Action::GrokForceDelete=>format!("Deactivate Grok and delete {subject}? This changes runtime overrides and removes the saved profile. / 停用 Grok 并删除 {subject}？将修改运行覆盖并删除配置。"),Action::Delete=>format!("Delete {subject}? This removes the saved profile; local backup policy remains in force. / 删除 {subject}？将移除保存的配置。"),Action::Off=>"Deactivate this platform? Runtime overrides are removed to prepare official login. Accounts are not logged out. / 停用当前平台配置？将清理运行覆盖以准备官方登录，不会登出账户。".into(),Action::Source|Action::Inspect|Action::ExportSecrets|Action::Advanced|Action::Environment|Action::ShellEnvironment=>"This opens plaintext credentials in a scoped editor/output. Keep screenshots and logs private. Canonical profile TOML does not preserve comments. / 将显示含明文凭据的临时编辑区；规范 TOML 不保留注释。".into(),Action::ForceSource=>"Remove the active profile from the catalog? This force applies only to activation protection; the original revision is still required. / 强制移除当前配置？仅跳过当前项保护，不绕过版本检查。".into(),Action::Restore=>"Restore a managed backup? Current catalog is backed up before replacement. Enter only a managed backup filename. / 恢复受管备份？替换前将备份当前目录，只接受受管文件名。".into(),Action::RestoreExecute=>format!("Restore {subject} over this catalog? Current catalog is backed up before replacement. / 使用 {subject} 替换当前目录？替换前将备份当前目录。"),Action::Clean=>"Delete managed local backup files? This cannot be undone. Profiles and accounts are retained. / 删除受管本地备份？此操作不可撤销，配置与账户保留。".into(),Action::ClearHistory=>"Clear configuration operation history? This cannot be undone. / 清空配置操作历史？此操作不可撤销。".into(),_=>"Import profiles? Existing matching names may be replaced; a preimage backup is required. / 导入配置？同名项可能被替换，必须先创建备份。".into()};
            root = root
                .child(
                    div()
                        .id("profile-confirmation")
                        .test_support()
                        .role(gpui_kit::Role::AlertDialog)
                        .aria_label(format!("Confirm profile action: {message}"))
                        .child(message),
                )
                .child(
                    div()
                        .tab_group()
                        .tab_stop(false)
                        .flex()
                        .gap(px(8.))
                        .child(semantics::button(
                            Button::new("profile-confirm-cancel")
                                .label(text(cx, "Cancel", "取消"))
                                .ghost()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    if let Some(confirm) = this.confirmation.take()
                                        && let Some(focus) = confirm.return_focus
                                    {
                                        window.focus(&focus, cx);
                                    }
                                    cx.notify();
                                })),
                            false,
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new("profile-confirm")
                                .label(text(cx, "Confirm", "确认"))
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.confirm(window, cx)),
                                ),
                            false,
                            false,
                            cx,
                        ))
                        .focus_trap("profile-confirmation-focus", &self.confirmation_focus),
                );
        }
        {
            if self.platform == Platform::Claude {
                root = root.child(
                    div()
                        .flex()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(
                            Button::new("profile-policy-profiles")
                                .selected(!self.generic)
                                .toggled(!self.generic)
                                .label(text(cx, "Profiles", "配置集"))
                                .ghost()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.generic = false;
                                    cx.notify();
                                })),
                            self.busy || self.expanded(),
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new("profile-policy-configurations")
                                .selected(self.generic)
                                .toggled(self.generic)
                                .label(text(cx, "Configurations", "配置目录"))
                                .ghost()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.generic = true;
                                    cx.notify();
                                })),
                            self.busy || self.expanded(),
                            false,
                            cx,
                        )),
                );
            }
            let mut bar = div().flex().flex_wrap().gap(px(theme::SELECTION_RADIUS));
            for (id, action, en, zh, needs_target) in [
                ("profile-create", Action::Create, "New", "新建", false),
                (
                    "profile-edit",
                    Action::Edit,
                    "Edit / rename",
                    "编辑/改名",
                    true,
                ),
                (
                    "profile-duplicate",
                    Action::Duplicate,
                    "Duplicate",
                    "复制",
                    true,
                ),
                (
                    "profile-apply",
                    Action::Apply,
                    "Apply / enable",
                    "应用/启用",
                    true,
                ),
                ("profile-delete", Action::Delete, "Delete", "删除", true),
                ("profile-off", Action::Off, "Off", "停用", false),
                (
                    "profile-validate",
                    Action::Validate,
                    "Validate",
                    "校验",
                    false,
                ),
                ("profile-source", Action::Source, "Source", "源码", false),
                (
                    "profile-inspect",
                    Action::Inspect,
                    "Inspect selected profile",
                    "查看完整配置",
                    true,
                ),
                (
                    "profile-export",
                    Action::Export,
                    "Redacted export",
                    "脱敏导出",
                    false,
                ),
                (
                    "profile-export-secrets",
                    Action::ExportSecrets,
                    "Canonical export",
                    "规范导出",
                    false,
                ),
                ("profile-import", Action::Import, "Import", "导入", false),
                ("profile-backups", Action::Backups, "Backups", "备份", false),
                ("profile-restore", Action::Restore, "Restore", "恢复", false),
                (
                    "profile-clean-backups",
                    Action::Clean,
                    "Clean backups",
                    "清理备份",
                    false,
                ),
                (
                    "profile-history",
                    Action::History,
                    "Operation history",
                    "操作历史",
                    false,
                ),
                (
                    "profile-clear-history",
                    Action::ClearHistory,
                    "Clear history",
                    "清空历史",
                    false,
                ),
                (
                    "profile-env",
                    Action::Environment,
                    "Export environment",
                    "导出环境",
                    true,
                ),
                (
                    "profile-shell-env",
                    Action::ShellEnvironment,
                    "Export shell script",
                    "导出脚本",
                    true,
                ),
            ] {
                let supported = match self.platform {
                    Platform::Claude => {
                        if self.generic {
                            matches!(
                                action,
                                Action::Create
                                    | Action::Edit
                                    | Action::Duplicate
                                    | Action::Apply
                                    | Action::Delete
                                    | Action::Validate
                                    | Action::Export
                                    | Action::Import
                                    | Action::Backups
                                    | Action::Restore
                                    | Action::Clean
                                    | Action::History
                                    | Action::ClearHistory
                            )
                        } else {
                            matches!(
                                action,
                                Action::Create
                                    | Action::Edit
                                    | Action::Delete
                                    | Action::Apply
                                    | Action::Off
                                    | Action::Validate
                                    | Action::Source
                                    | Action::Inspect
                                    | Action::ExportSecrets
                            )
                        }
                    }
                    Platform::Codex => matches!(
                        action,
                        Action::Create
                            | Action::Edit
                            | Action::Delete
                            | Action::Apply
                            | Action::Off
                            | Action::Validate
                            | Action::Source
                            | Action::Inspect
                            | Action::ExportSecrets
                            | Action::Environment
                            | Action::ShellEnvironment
                    ),
                    Platform::Grok => matches!(
                        action,
                        Action::Create
                            | Action::Edit
                            | Action::Delete
                            | Action::Apply
                            | Action::Off
                            | Action::Inspect
                            | Action::Validate
                    ),
                    _ => false,
                };
                if !supported {
                    continue;
                }
                bar = bar.child(semantics::button(
                    Button::new(id).label(text(cx, en, zh)).ghost().on_click(
                        cx.listener(move |this, _, window, cx| this.act(action, window, cx)),
                    ),
                    self.busy
                        || self.expanded()
                        || (needs_target && self.target.is_none())
                        || (matches!(action, Action::Off) && self.can_off != Some(true)),
                    self.busy || self.confirmation.is_some(),
                    cx,
                ));
            }
            root = root.child(bar);
        }
        if self.mode != Mode::Closed {
            root=root.flex_1().overflow_y_scroll().child(div().flex().items_center().gap(px(8.)).child(semantics::button(Button::new("profile-editor-cancel").label(text(cx,"Cancel / back","取消/返回")).ghost().on_click(cx.listener(|this,_,window,cx|this.close(window,cx))),self.busy || self.confirmation.is_some(),self.busy,cx)).child(div().child(text(cx,"Draft keeps its original catalog revision. Untouched fields and credentials are retained.","草稿绑定原始版本，未修改字段和凭据保留。"))));
            if matches!(self.mode, Mode::Edit | Mode::Duplicate | Mode::Restore) {
                root = root.child(text(cx, "Name / target", "名称/目标")).child(
                    Input::new(&self.name)
                        .disabled(self.busy || self.confirmation.is_some())
                        .id("profile-name")
                        .aria_label(text(cx, "Profile name", "配置名称")),
                );
            }
            if self.mode == Mode::Edit {
                root = root
                    .child(text(cx, "Description", "说明"))
                    .child(
                        Input::new(&self.description)
                            .disabled(self.busy || self.confirmation.is_some())
                            .id("profile-description")
                            .aria_label(text(cx, "Description", "说明")),
                    )
                    .child(text(cx, "Model (empty removes)", "模型（留空清除）"))
                    .child(
                        Input::new(&self.model)
                            .disabled(self.busy || self.confirmation.is_some())
                            .id("profile-model")
                            .aria_label(text(cx, "Model", "模型")),
                    );
                let official = self.platform == Platform::Grok
                    && self
                        .original
                        .as_ref()
                        .is_some_and(|row| row.provider_type.as_deref() == Some("official"));
                if !official {
                    root = root
                        .child(text(
                            cx,
                            "Replace credential (blank keeps existing)",
                            "替换凭据（留空保留现有值）",
                        ))
                        .child(
                            Input::new(&self.credential)
                                .disabled(self.busy || self.confirmation.is_some())
                                .id("profile-credential")
                                .aria_label(text(cx, "Replacement credential", "替换凭据")),
                        )
                        .child(text(
                            cx,
                            "Replacement base URL (blank keeps existing)",
                            "替换端点（留空保留现有值）",
                        ))
                        .child(
                            Input::new(&self.base_url)
                                .id("profile-base-url")
                                .disabled(self.busy || self.confirmation.is_some())
                                .aria_label("Replacement base URL"),
                        );
                }
                if !self.generic && !(self.platform == Platform::Grok && self.original.is_some()) {
                    root = root
                        .child(text(
                            cx,
                            "Auth mode / Grok profile_kind (blank preserves)",
                            "认证模式 / Grok profile_kind（留空保留）",
                        ))
                        .child(
                            Input::new(&self.auth_mode)
                                .id("profile-auth-mode")
                                .disabled(self.busy || self.confirmation.is_some())
                                .aria_label("Auth mode or Grok profile kind"),
                        );
                }
                root = root.child(semantics::button(
                    Button::new("profile-advanced")
                        .label(text(cx, "Advanced field patch", "高级字段补丁"))
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.act(Action::Advanced, window, cx)
                        })),
                    self.busy || self.confirmation.is_some(),
                    false,
                    cx,
                ));
                if self.advanced {
                    root = root.child(text(
                        cx,
                        "JSON object: omitted keeps, null removes. Never paste display masks.",
                        "JSON 对象：省略保留，null 清除，勿粘贴显示掩码。",
                    ));
                }
            }
            if matches!(self.mode, Mode::Source | Mode::Import | Mode::Output) || self.advanced {
                if self.mode == Mode::Source
                    && let Some(location) = &self.source_location
                {
                    root = root.child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child(location.clone()),
                    );
                }
                root = root.child(
                    Textarea::new(&self.content)
                        .accessibility_id("profile-source-content")
                        .aria_label(text(cx, "Scoped profile text", "临时配置文本"))
                        .h(px(240.))
                        .readonly(
                            self.mode == Mode::Output || self.busy || self.confirmation.is_some(),
                        ),
                );
            }
            if self.mode == Mode::Edit && self.failed {
                root = root.child(semantics::button(
                    Button::new("profile-draft-reload")
                        .label(text(cx, "Reload and discard draft", "重读并丢弃草稿"))
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.close(window, cx);
                            this.refresh_version += 1;
                            cx.notify();
                        })),
                    self.busy || self.confirmation.is_some(),
                    false,
                    cx,
                ));
            }
            if self.mode == Mode::Source {
                root = root.child(
                    div()
                        .flex()
                        .gap(px(8.))
                        .child(semantics::button(
                            Button::new("profile-source-reload")
                                .label(text(
                                    cx,
                                    "Reload and discard source draft",
                                    "重读并丢弃源码草稿",
                                ))
                                .ghost()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.submit(ProfileCommand::Source, cx)
                                })),
                            self.busy || self.confirmation.is_some(),
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new("profile-source-force")
                                .label(text(cx, "Force active-profile removal", "强制移除当前项"))
                                .ghost()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.act(Action::ForceSource, window, cx)
                                })),
                            self.busy || self.confirmation.is_some() || !self.activation_conflict,
                            false,
                            cx,
                        )),
                );
            }
            if self.mode == Mode::Import {
                root = root.child(semantics::button(
                    Button::new("profile-import-replace")
                        .label(text(
                            cx,
                            "Replace catalog after confirmation",
                            "确认后替换目录",
                        ))
                        .ghost()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.act(Action::ImportReplace, window, cx)
                        })),
                    self.busy || self.confirmation.is_some(),
                    false,
                    cx,
                ));
            }
            if self.mode != Mode::Output {
                root = root.child(semantics::button(
                    Button::new("profile-save")
                        .label(if self.busy {
                            text(cx, "Saving", "保存中")
                        } else {
                            text(cx, "Save", "保存")
                        })
                        .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
                    self.busy || self.confirmation.is_some(),
                    self.busy,
                    cx,
                ));
            }
        }
        if self.grok_force_delete.is_some() {
            root = root.child(semantics::button(
                Button::new("profile-grok-force-delete")
                    .label(text(cx, "Review Off and delete", "审阅停用并删除"))
                    .ghost()
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.act(Action::GrokForceDelete, window, cx)
                    })),
                self.busy || self.confirmation.is_some(),
                false,
                cx,
            ));
        }
        if let Some((old, new, apply)) = self.grok_recovery.clone() {
            root=root.child(div().child(if apply{format!("Saved {new}, previous {old} retained. Retry apply after fixing runtime access. / 新配置已保存，旧配置保留。修复运行目录权限后重试应用。 ")}else{format!("{new} is active; {old} cleanup failed. Refresh and inspect the remaining names before cleanup. / 新配置已激活，清理未完成；请刷新并检查实际剩余配置后清理。 ")}));
            if apply {
                root = root.child(semantics::button(
                    Button::new("profile-grok-retry-apply")
                        .label(text(cx, "Retry new profile apply", "重试应用新配置"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.submit(
                                ProfileCommand::Apply {
                                    name: new.clone(),
                                    enable: false,
                                },
                                cx,
                            )
                        })),
                    self.busy || self.confirmation.is_some(),
                    self.busy,
                    cx,
                ));
            }
        }
        if let Some(message) = &self.message {
            root = root.child(
                div()
                    .id("profile-result")
                    .role(gpui_kit::Role::Status)
                    .aria_label(message.clone())
                    .text_color(if self.failed {
                        cx.theme().danger
                    } else {
                        cx.theme().muted_foreground
                    })
                    .child(message.clone()),
            );
        }
        root
    }
}
