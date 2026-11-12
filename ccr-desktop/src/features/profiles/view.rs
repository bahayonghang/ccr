use super::*;
use crate::theme;
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{
    ActiveTheme, FocusTrapElement,
    button::{Button, ButtonVariants},
    input::{Input, Textarea},
};
use gpui_kit::{
    InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled,
    div, px,
};

/// 解除当前方案的文案按平台说明运行凭据影响。
fn off_message(platform: Platform) -> &'static str {
    match platform {
        Platform::Codex => {
            "Deactivate this platform? CCR-managed runtime overrides are removed, saved profiles are retained. Official login is not restored automatically. Codex runtime authentication may be cleared. / 解除当前方案？移除 CCR 运行覆盖并保留方案，不保证自动恢复官方登录；Codex 运行认证缓存可能被清除。"
        }
        Platform::Grok => {
            "Deactivate this platform? CCR-managed runtime overrides are removed, saved profiles are retained. Official login is not restored automatically. Grok runtime state may be reset by the next activation. / 解除当前方案？移除 CCR 运行覆盖并保留方案，不保证自动恢复官方登录；下次启用可能重置 Grok 运行状态。"
        }
        _ => {
            "Deactivate this platform? CCR-managed runtime overrides are removed, saved profiles are retained. Official login is not restored automatically; Claude Code falls back to the credentials its settings reference. / 解除当前方案？移除 CCR 运行覆盖并保留方案，不保证自动恢复官方登录；Claude Code 将回落到其 settings 引用的凭据。"
        }
    }
}

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
                            this.request_close(window, cx);
                        }
                        cx.stop_propagation();
                    }
                }),
            );
        if let Some(confirm) = &mut self.confirmation {
            if std::mem::take(&mut confirm.entry_pending) {
                // 平台帧先排空回调再绘制；从首次 render 排队，确保下一帧能遍历已挂载的按钮。
                let generation = confirm.generation;
                let view = cx.weak_entity();
                window.on_next_frame(move |window, cx| {
                    let _ = view.update(cx, |this, cx| {
                        if this
                            .confirmation
                            .as_ref()
                            .is_some_and(|confirm| confirm.generation == generation)
                        {
                            window.focus(&this.confirmation_focus, cx);
                            window.focus_next(cx);
                        }
                    });
                });
            }
            let subject = confirm.subject.as_str();
            let message=match confirm.action {Action::DiscardDraft=>"Discard the unsaved draft? The saved profile and its revision are unchanged. / 放弃未保存的草稿？已保存的方案与版本不变。".into(),Action::GrokForceDelete=>format!("Deactivate Grok and delete {subject}? This changes runtime overrides and removes the saved profile. / 停用 Grok 并删除 {subject}？将修改运行覆盖并删除配置。"),Action::Delete=>format!("Delete {subject}? This removes the saved profile; local backup policy remains in force. / 删除 {subject}？将移除保存的配置。"),Action::Off=>off_message(self.platform).into(),Action::Source|Action::Inspect|Action::ExportSecrets|Action::Advanced|Action::Environment|Action::ShellEnvironment=>"This opens plaintext credentials in a scoped editor/output. Keep screenshots and logs private. Canonical profile TOML does not preserve comments. / 将显示含明文凭据的临时编辑区；规范 TOML 不保留注释。".into(),Action::Restore=>"Restore a managed backup? Current catalog is backed up before replacement. Enter only a managed backup filename. / 恢复受管备份？替换前将备份当前目录，只接受受管文件名。".into(),Action::RestoreExecute=>format!("Restore {subject} over this catalog? Current catalog is backed up before replacement. / 使用 {subject} 替换当前目录？替换前将备份当前目录。"),Action::Clean=>"Delete managed local backup files? This cannot be undone. Profiles and accounts are retained. / 删除受管本地备份？此操作不可撤销，配置与账户保留。".into(),Action::ClearHistory=>"Clear configuration operation history? This cannot be undone. / 清空配置操作历史？此操作不可撤销。".into(),_=>"Commit the reviewed import? Only explicitly selected conflicts are overwritten; replacement requires its preimage backup. Runtime configuration is not applied. / 提交已审阅的导入？仅覆盖明确选择的同名项，替换须有预备份；运行配置不会自动应用。".into()};
            let message = crate::locales::localized_line(cx, &message);
            let confirm_name = format!(
                "{}: {message}",
                text(cx, "Confirm profile action", "确认配置操作")
            );
            root = root
                .child(
                    div()
                        .id("profile-confirmation")
                        .test_support()
                        .role(gpui_kit::Role::AlertDialog)
                        .aria_label(confirm_name)
                        .child(message),
                )
                .child(
                    div()
                        .tab_group()
                        .tab_stop(false)
                        .flex()
                        .gap(px(8.))
                        .child(semantics::ghost_button(
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
                        .child(semantics::ghost_button(
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
        if self.mode == Mode::Closed {
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
                ("profile-apply", Action::Apply, "Apply", "应用", true),
                ("profile-delete", Action::Delete, "Delete", "删除", true),
                (
                    "profile-off",
                    Action::Off,
                    "Detach current profile",
                    "解除当前方案",
                    false,
                ),
                (
                    "profile-validate",
                    Action::Validate,
                    "Validate",
                    "校验",
                    false,
                ),
                (
                    "profile-source",
                    Action::Source,
                    "Edit all profiles (TOML)",
                    "编辑全部方案（TOML）",
                    false,
                ),
                (
                    "profile-inspect",
                    Action::Inspect,
                    "View full profile (with credentials)",
                    "查看完整方案（含凭据）",
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
                    "Export full copy (with credentials)",
                    "导出完整副本（含凭据）",
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
                    "Operation records (all platforms)",
                    "操作记录（跨平台）",
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
                    Platform::Claude => !matches!(
                        action,
                        Action::Clean
                            | Action::ClearHistory
                            | Action::GrokForceDelete
                            | Action::Environment
                            | Action::ShellEnvironment
                    ),
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
                if self.embedded
                    && (matches!(action, Action::Off | Action::Create)
                        || (self.management == needs_target))
                {
                    continue;
                }
                bar = bar.child(semantics::ghost_button(
                    Button::new(id)
                        .label(
                            if matches!(action, Action::Apply)
                                && self.target.as_ref().is_some_and(|row| !row.enabled)
                            {
                                text(cx, "Enable and apply", "启用并应用")
                            } else {
                                text(cx, en, zh)
                            },
                        )
                        .ghost()
                        .on_click(
                            cx.listener(move |this, _, window, cx| this.act(action, window, cx)),
                        ),
                    self.busy
                        || self.expanded()
                        || (needs_target && self.target.is_none())
                        // 当前运行方案必须先解除（Action::Off）再删除，不能静默换掉运行配置。
                        || (matches!(action, Action::Delete) && self.target_is_current())
                        || (matches!(action, Action::Off) && self.can_off != Some(true)),
                    self.busy || self.confirmation.is_some(),
                    cx,
                ));
            }
            root = root.child(bar);
        }
        if self.mode != Mode::Closed {
            root=root.flex_1().child(div().flex_shrink_0().flex().items_center().gap(px(8.)).child(semantics::ghost_button(Button::new("profile-editor-cancel").label(text(cx,"Cancel / back","取消/返回")).ghost().on_click(cx.listener(|this,_,window,cx|this.request_close(window,cx))),self.busy || self.confirmation.is_some(),self.busy,cx)).child(div().child(text(cx,"Draft keeps its original catalog revision. Untouched fields and credentials are retained.","草稿绑定原始版本，未修改字段和凭据保留。"))));
            if self.mode != Mode::Output {
                root = root.child(semantics::button(
                    Button::new("profile-save")
                        .label(if self.busy {
                            text(cx, "Saving", "保存中")
                        } else {
                            if self.mode == Mode::Import {
                                text(cx, "Preview import", "预览导入")
                            } else {
                                text(cx, "Save", "保存")
                            }
                        })
                        .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
                    self.busy || self.confirmation.is_some(),
                    self.busy,
                    cx,
                ));
            }
            if self.mode == Mode::Output {
                root = root.child(semantics::button(
                    Button::new("profile-export-save-file")
                        .label(text(
                            cx,
                            "Save this output to a new file…",
                            "将当前输出保存到新文件…",
                        ))
                        .primary()
                        .on_click(cx.listener(|this, _, _, cx| this.save_output(cx))),
                    self.busy || self.confirmation.is_some(),
                    self.busy,
                    cx,
                ));
            }
            if let Some(preview) = &self.import_preview {
                root = root.child(semantics::button(
                    Button::new("profile-import-commit")
                        .label(text(cx, "Confirm reviewed import", "确认已审阅的导入"))
                        .primary()
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.act(Action::ImportCommit, window, cx)
                        })),
                    self.busy
                        || self.confirmation.is_some()
                        || !preview.matches_input(&self.content.read(cx).value()),
                    false,
                    cx,
                ));
            }
            if self.mode == Mode::Source {
                root = root.child(
                    div().flex().gap(px(8.)).child(semantics::ghost_button(
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
                    )),
                );
            }
            if self.mode == Mode::Import {
                root = root.child(semantics::ghost_button(
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
            let mut fields = div()
                .id("profile-fields")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap(px(theme::CONTROL_GAP));
            if self.mode == Mode::Import
                && let Some(preview) = self.import_preview.clone()
            {
                fields = fields.child(div().id("profile-import-preview").child(format!(
                    "{}: {} / {}: {}",
                    text(cx, "New", "新增"),
                    preview.added.join(", "),
                    text(cx, "Matching names (kept by default)", "同名（默认保留）"),
                    preview.conflicts.join(", ")
                )));
                if preview.replace_all {
                    fields = fields.child(text(cx, "Replacement preview · backup happens on commit; current profile protection is re-checked then.", "替换预览 · 提交时才备份原内容，并重验当前方案保护。"));
                    if !preview.removed.is_empty() {
                        fields = fields.child(div().id("profile-import-removed").child(format!(
                            "{}: {}",
                            text(cx, "Removed by this replacement", "本次替换将移除"),
                            preview.removed.join(", ")
                        )));
                    }
                }
                for (index, name) in preview
                    .conflicts
                    .iter()
                    .enumerate()
                    .filter(|_| !preview.replace_all)
                {
                    let name = name.clone();
                    let selected = self.import_overwrite.contains(&name);
                    fields = fields.child(semantics::ghost_button(
                        Button::new(format!("profile-import-overwrite-{index}"))
                            .label(format!("{} {name}", text(cx, "Overwrite", "覆盖")))
                            .ghost()
                            .toggled(selected)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if !this.import_overwrite.remove(&name) {
                                    this.import_overwrite.insert(name.clone());
                                }
                                this.submit(
                                    ProfileCommand::PreviewImport {
                                        content: this.content.read(cx).value().to_string(),
                                        overwrite: this.import_overwrite.clone(),
                                        replace: false,
                                    },
                                    cx,
                                );
                            })),
                        self.busy || self.confirmation.is_some(),
                        false,
                        cx,
                    ));
                }
            }
            if matches!(self.mode, Mode::Edit | Mode::Duplicate | Mode::Restore) {
                fields = fields.child(text(cx, "Name / target", "名称/目标")).child(
                    Input::new(&self.name)
                        .disabled(self.busy || self.confirmation.is_some())
                        .id("profile-name")
                        .aria_label(text(cx, "Profile name", "配置名称")),
                );
            }
            if self.mode == Mode::Edit {
                fields = fields
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
                    fields = fields
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
                    fields = fields
                        .child(text(
                            cx,
                            if self.platform == Platform::Grok {
                                "Profile kind (blank preserves)"
                            } else {
                                "Auth mode (blank preserves)"
                            },
                            if self.platform == Platform::Grok {
                                "profile_kind（留空保留）"
                            } else {
                                "认证模式（留空保留）"
                            },
                        ))
                        .child(
                            Input::new(&self.auth_mode)
                                .id("profile-auth-mode")
                                .disabled(self.busy || self.confirmation.is_some())
                                .aria_label(if self.platform == Platform::Grok {
                                    "Profile kind"
                                } else {
                                    "Auth mode"
                                }),
                        );
                }
                fields = fields.child(semantics::ghost_button(
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
                    fields = fields.child(text(
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
                    fields = fields.child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child(location.clone()),
                    );
                }
                fields = fields.child(
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
                fields = fields.child(semantics::ghost_button(
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
            root = root.child(fields);
        }
        if self.grok_force_delete.is_some() {
            root = root.child(semantics::ghost_button(
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
            let recovery = if apply {
                format!(
                    "Saved {new}, previous {old} retained. Retry apply after fixing runtime access. / 新配置 {new} 已保存，旧配置 {old} 保留。修复运行目录权限后重试应用。"
                )
            } else {
                format!(
                    "{new} is active; {old} cleanup failed. Refresh and inspect the remaining names before cleanup. / 新配置 {new} 已激活，旧配置 {old} 清理未完成。请刷新并检查实际剩余配置后清理。"
                )
            };
            root = root.child(div().child(crate::locales::localized_line(cx, &recovery)));
            if apply {
                root = root.child(semantics::ghost_button(
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
            let line = crate::locales::localized_line(cx, message);
            root = root.child(
                div()
                    .id("profile-result")
                    .role(gpui_kit::Role::Status)
                    .aria_label(line.clone())
                    .text_color(if self.failed {
                        cx.theme().danger
                    } else {
                        cx.theme().muted_foreground
                    })
                    .child(line),
            );
        }
        root
    }
}
