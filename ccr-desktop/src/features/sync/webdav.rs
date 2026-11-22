use super::*;
use crate::runtime::sync::ConnectionTestStatus;
use gpui_kit::component::ActiveTheme;
use gpui_kit::{AnyElement, Focusable};

impl SyncView {
    pub(super) fn edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.busy || self.operation.busy || self.has_environment_draft() {
            return;
        }
        let Some(revision) = self.revision() else {
            return;
        };
        let url = cx.new(|cx| InputState::new(window, cx));
        window.focus(&url.read(cx).focus_handle(cx), cx);
        self.editor = Some(Editor {
            focus: cx.focus_handle().tab_stop(false),
            saved: false,
            revision,
            url,
            username: cx.new(|cx| InputState::new(window, cx)),
            password: cx.new(|cx| InputState::new(window, cx).masked(true)),
            base: cx.new(|cx| InputState::new(window, cx)),
            enabled: matches!(&self.snapshot, Some(SyncSnapshot::Present(local)) if local.webdav_enabled),
            password_mode: PasswordMode::Keep,
        });
        cx.notify();
    }

    pub(super) fn close_editor(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor = None;
        self.restore_action_focus(self.edit_anchor.clone(), window, cx);
        cx.notify();
    }

    pub(super) fn finish_editor_save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.busy
            && let Some(editor) = self.editor.take_if(|editor| editor.saved)
            && (editor.focus.is_focused(window) || editor.focus.contains_focused(window, cx))
        {
            let anchor = if self.snapshot.is_some() {
                &self.edit_anchor
            } else {
                &self.focus
            };
            self.restore_action_focus(anchor.clone(), window, cx);
        }
    }

    pub(super) fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(editor) = &self.editor else {
            return;
        };
        if self.revision().as_ref() != Some(&editor.revision) {
            return;
        }
        let value = |input: &Entity<InputState>| {
            let value = input.read(cx).value().to_string();
            (!value.trim().is_empty()).then(|| value.trim().to_string())
        };
        let password = match editor.password_mode {
            PasswordMode::Replace => {
                PasswordUpdate::Replace(editor.password.read(cx).value().to_string().into())
            }
            PasswordMode::Clear => PasswordUpdate::Clear,
            PasswordMode::Keep => PasswordUpdate::Keep,
        };
        let focus = editor.focus.clone();
        let owns_focus = focus.contains_focused(window, cx);
        self.start(
            Operation::Save(SettingsPatch {
                revision: editor.revision.clone(),
                enabled: editor.enabled,
                url: value(&editor.url),
                username: value(&editor.username),
                base_remote_path: value(&editor.base),
                password,
            }),
            cx,
        );
        // The Save button loses its focus binding while the request is in flight.
        if owns_focus && self.operation.busy && self.operation.kind == Some(OperationKind::Save) {
            window.focus(&focus, cx);
        }
    }

    pub(super) fn render_webdav(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let locked = self.busy || self.operation.busy || self.has_environment_draft();
        let present = matches!(self.snapshot, Some(SyncSnapshot::Present(_)));
        let (config_status, color) = match &self.snapshot {
            _ if self.busy => (
                text(cx, "Reading configuration…", "正在读取配置…"),
                cx.theme().muted_foreground,
            ),
            Some(SyncSnapshot::Absent) => (
                text(cx, "Not configured", "未配置"),
                cx.theme().muted_foreground,
            ),
            Some(SyncSnapshot::Present(local)) if local.webdav_enabled => (
                text(cx, "Saved · Enabled", "已保存 · 已启用"),
                cx.theme().foreground,
            ),
            Some(SyncSnapshot::Present(_)) => (
                text(cx, "Saved · Disabled", "已保存 · 已停用"),
                cx.theme().muted_foreground,
            ),
            None if self.message.is_some() => (
                text(cx, "Configuration read failed", "配置读取失败"),
                cx.theme().danger,
            ),
            None => (
                text(cx, "Waiting to read configuration", "等待配置读取"),
                cx.theme().muted_foreground,
            ),
        };
        let test = self
            .revision()
            .and_then(|revision| self.operation.connection_test(&self.environment, &revision));
        let (connection, color_connection) = match test {
            Some(ConnectionTestStatus::Running) => (
                text(cx, "Testing connection…", "正在测试连接…").into(),
                cx.theme().muted_foreground,
            ),
            Some(ConnectionTestStatus::Succeeded) => (
                text(cx, "Connection test succeeded.", "连接测试成功。").into(),
                cx.theme().success,
            ),
            Some(ConnectionTestStatus::Failed(error)) => (
                format!(
                    "{} {}",
                    text(cx, "Connection test failed.", "连接测试失败。"),
                    sync_error(cx, error)
                ),
                cx.theme().danger,
            ),
            None => (
                text(cx, "Remote access was not tested.", "未测试远程访问。").into(),
                cx.theme().muted_foreground,
            ),
        };
        let mut section = reading_panel("sync-webdav", cx)
            .child(
                sync_actions()
                    .items_center()
                    .child(section_heading(text(cx, "WebDAV", "WebDAV")))
                    .child(sync_status("sync-webdav-config", config_status).text_color(color)),
            )
            .child(sync_status("sync-webdav-connection", connection).text_color(color_connection));

        match &self.snapshot {
            Some(SyncSnapshot::Present(local)) => {
                let summary = format!(
                    "{} {} {}",
                    url_line(cx, &local.url),
                    if local.username_present {
                        text(cx, "A WebDAV username is stored.", "已保存 WebDAV 用户名。")
                    } else {
                        text(
                            cx,
                            "No WebDAV username is stored.",
                            "未保存 WebDAV 用户名。",
                        )
                    },
                    if local.password_present {
                        text(
                            cx,
                            "A WebDAV password is stored.",
                            "配置中已保存 WebDAV 密码。",
                        )
                    } else {
                        text(cx, "No WebDAV password is stored.", "未保存 WebDAV 密码。")
                    },
                );
                section = section.child(
                    div()
                        .id("sync-local")
                        .test_support()
                        .aria_label(summary.clone())
                        .text_size(theme::text_size(theme::META_SIZE))
                        .text_color(cx.theme().muted_foreground)
                        .child(summary),
                );
            }
            Some(SyncSnapshot::Absent) => {
                let reason = text(
                    cx,
                    "No canonical sync config file was found. Legacy sync config is not read.",
                    "没有找到规范同步配置文件。不会读取旧的同步配置。",
                );
                section = section.child(
                    div()
                        .id("sync-absent")
                        .test_support()
                        .aria_label(reason)
                        .text_size(theme::text_size(theme::META_SIZE))
                        .text_color(cx.theme().muted_foreground)
                        .child(reason),
                );
            }
            None => {}
        }
        section = section.child(
            sync_actions()
                .child(
                    div()
                        .id("sync-edit-action")
                        .track_focus(&self.edit_anchor)
                        .tab_stop(false)
                        .child(semantics::button(
                            Button::new("sync-edit")
                                .label(if present {
                                    text(cx, "Edit WebDAV settings", "编辑 WebDAV 设置")
                                } else {
                                    text(cx, "Configure WebDAV", "配置 WebDAV")
                                })
                                .primary()
                                .on_click(cx.listener(|this, _, window, cx| this.edit(window, cx))),
                            locked || self.snapshot.is_none(),
                            false,
                            cx,
                        )),
                )
                .child(semantics::ghost_button(
                    Button::new("sync-read")
                        .label(text(
                            cx,
                            "Read current environment config",
                            "读取当前环境配置",
                        ))
                        .on_click(cx.listener(|this, _, _, cx| {
                            if !this.has_environment_draft() {
                                this.refresh(cx);
                            }
                        })),
                    locked,
                    self.busy,
                    cx,
                ))
                .child(semantics::ghost_button(
                    Button::new("sync-test")
                        .label(text(cx, "Test connection", "测试连接"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            if !this.has_environment_draft()
                                && let Some(revision) = this.revision()
                            {
                                this.start(Operation::Test { revision }, cx);
                            }
                        })),
                    locked || !present,
                    matches!(test, Some(ConnectionTestStatus::Running)),
                    cx,
                ))
                .child(self.render_disconnect(cx)),
        );
        if self.is_ssh() {
            let limitations = text(
                cx,
                "Profile databases, disconnecting WebDAV and creating empty destination directories are unavailable in SSH. Multi-file publication can partially complete. Backups of replaced files stay on the SSH host.",
                "SSH 不支持方案数据库、断开 WebDAV 或新建纯空目标目录。多文件发布可能部分完成。被替换文件的备份保留在 SSH 主机。",
            );
            section = section.child(
                div()
                    .id("sync-intro")
                    .test_support()
                    .aria_label(limitations)
                    .text_size(theme::text_size(theme::META_SIZE))
                    .text_color(cx.theme().muted_foreground)
                    .child(limitations),
            );
        }
        if let Some(message) = self.message {
            section = section.child(
                sync_status("sync-status", service_error(cx, message))
                    .text_color(cx.theme().danger),
            );
        }
        section
    }

    pub(super) fn render_editor(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let editor = self.editor.as_ref()?;
        let locked = self.busy || self.operation.busy;
        let mut pane = div().id("sync-editor").test_support().flex().flex_col().gap(px(theme::CONTROL_GAP))
            .track_focus(&editor.focus)
            .tab_stop(false)
            .child(section_heading(text(cx, "WebDAV settings", "WebDAV 设置")))
            .child(div().text_color(cx.theme().muted_foreground).child(text(cx,
                "Leave URL, username or base path blank to keep the saved value. Saving does not contact the server.",
                "URL、用户名或基础路径留空会保留原值。保存不会连接服务器。",
            )))
            .child(field("sync-url", text(cx, "WebDAV URL", "WebDAV URL"), &editor.url))
            .child(field("sync-username", text(cx, "WebDAV username", "WebDAV 用户名"), &editor.username))
            .child(field("sync-base", text(cx, "Remote base path", "远程基础路径"), &editor.base))
            .child(sync_actions().child(semantics::button(
                Button::new("sync-enabled")
                    .label(if editor.enabled { text(cx, "WebDAV enabled", "WebDAV 已启用") } else { text(cx, "WebDAV disabled", "WebDAV 已停用") })
                    .ghost()
                    .toggled(editor.enabled)
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(editor) = &mut this.editor {
                            editor.enabled = !editor.enabled;
                        }
                        cx.notify();
                    })),
                locked, false, cx,
            )))
            .child(sync_actions().children([
                (PasswordMode::Keep, text(cx, "Keep password", "保留密码")),
                (PasswordMode::Replace, text(cx, "Replace password", "替换密码")),
                (PasswordMode::Clear, text(cx, "Clear password", "清除密码")),
            ].into_iter().enumerate().map(|(index, (mode, label))| {
                semantics::tab(
                    Button::new(("sync-password-mode", index))
                        .label(label)
                        .ghost()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            if let Some(editor) = &mut this.editor {
                                editor.password_mode = mode;
                                if mode != PasswordMode::Replace {
                                    editor.password.update(cx, |input, cx| input.set_value("", window, cx));
                                }
                            }
                            cx.notify();
                        })),
                    editor.password_mode == mode, locked, cx,
                )
            })));
        if editor.password_mode == PasswordMode::Replace {
            pane = pane.child(field(
                "sync-password",
                text(cx, "New WebDAV password", "新 WebDAV 密码"),
                &editor.password,
            ));
        }
        if editor.password_mode == PasswordMode::Clear {
            pane = pane.child(div().text_color(cx.theme().warning).child(text(
                cx,
                "Disable WebDAV before clearing the saved password.",
                "请先停用 WebDAV，再清除已保存的密码。",
            )));
        }
        if self.operation.kind == Some(OperationKind::Save) {
            pane = pane.children(self.render_operation_status(cx));
        }
        Some(
            pane.child(
                sync_actions()
                    .child(semantics::button(
                        Button::new("sync-save")
                            .label(text(cx, "Save settings", "保存设置"))
                            .primary()
                            .on_click(cx.listener(|this, _, window, cx| this.save(window, cx))),
                        locked,
                        self.operation.busy && self.operation.kind == Some(OperationKind::Save),
                        cx,
                    ))
                    .child(semantics::ghost_button(
                        Button::new("sync-edit-cancel")
                            .label(text(cx, "Cancel editing", "取消编辑"))
                            .on_click(
                                cx.listener(|this, _, window, cx| this.close_editor(window, cx)),
                            ),
                        locked,
                        false,
                        cx,
                    )),
            )
            .into_any_element(),
        )
    }
}

fn url_line(cx: &App, url: &SyncUrl) -> String {
    match url {
        SyncUrl::Missing => text(cx, "No WebDAV URL is stored.", "未保存 WebDAV URL。").into(),
        SyncUrl::Stored => text(cx, "A WebDAV URL is stored.", "已保存 WebDAV URL。").into(),
        SyncUrl::Host(host) if is_chinese(cx) => format!("WebDAV 主机：{host}"),
        SyncUrl::Host(host) => format!("WebDAV host: {host}"),
    }
}

fn field(id: &'static str, label: &'static str, input: &Entity<InputState>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(label)
        .child(Input::new(input).aria_label(label).accessibility_id(id))
}
