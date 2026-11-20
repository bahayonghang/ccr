use super::*;
use gpui_kit::{AnyElement, App, FontWeight, KeyDownEvent, Role};

pub(super) enum AccountForm {
    Save,
    Rename(String),
}

impl PlatformsView {
    pub(crate) fn account_draft_dirty(&self, cx: &App) -> bool {
        self.surface == Surface::Auth
            && ((self.account_form.is_some()
                && (!self.account_input.read(cx).value().is_empty()
                    || !self.account_scope.read(cx).value().is_empty()))
                || self.workspace_dirty(cx))
    }
    pub(super) fn matched_account(&self) -> Option<&str> {
        let mut matches = self
            .session
            .as_ref()?
            .accounts
            .iter()
            .filter(|row| row.current);
        let first = matches.next()?;
        matches.next().is_none().then_some(first.name.as_str())
    }

    pub(super) fn account_result(&self, command: &Command, cx: &App) -> Option<String> {
        let Command::Account { name, action, .. } = command else {
            return None;
        };
        Some(match action {
            AccountAction::Switch => format!(
                "{} · {name}",
                text(cx, "Switched runtime account", "已切换运行账户")
            ),
            AccountAction::Delete => format!(
                "{} · {name}",
                text(cx, "Deleted saved account", "已删除保存的账户")
            ),
            AccountAction::Off => {
                text(cx, "Cleared runtime authentication", "已清除运行认证").to_owned()
            }
            AccountAction::Save | AccountAction::SaveFrom(_) => {
                text(cx, "Saved current credentials", "已保存当前运行凭据").to_owned()
            }
            _ => text(cx, "Updated saved accounts", "已更新保存的账户").to_owned(),
        })
    }
    pub(super) fn reset_account_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.account_form = None;
        self.account_input
            .update(cx, |i, cx| i.set_value("", window, cx));
        self.account_scope
            .update(cx, |i, cx| i.set_value("", window, cx));
    }
    pub(super) fn close_account_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_account_form(window, cx);
        if let Some(focus) = self.account_return.take() {
            window.focus(&focus, cx);
        }
        cx.notify();
    }
    pub(super) fn cancel_account_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.account_draft_dirty(cx) {
            self.decide(Decision::DiscardAccountForm, window, cx);
        } else {
            self.close_account_form(window, cx);
        }
    }
    fn open_account_form(
        &mut self,
        form: AccountForm,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.blocked() || self.account_form.is_some() {
            return;
        }
        self.account_return = window.focused(cx);
        self.account_form = Some(form);
        window.focus(&self.account_input.read(cx).focus_handle(cx), cx);
        cx.notify();
    }
    fn submit_account_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.blocked() {
            return;
        }
        let name = self.account_input.read(cx).value().trim().to_owned();
        if name.is_empty() || name.len() > 32 {
            self.message = Some(
                text(
                    cx,
                    "Enter an account name (1–32 bytes).",
                    "请输入账户名（1–32 字节）。",
                )
                .into(),
            );
            cx.notify();
            return;
        }
        match &self.account_form {
            Some(AccountForm::Rename(target))
                if self.selected_account().as_ref() == Some(target) =>
            {
                self.decide(Decision::RenameAccount, window, cx)
            }
            Some(AccountForm::Save)
                if self.platform == Platform::Codex
                    && matches!(
                        self.services.environment(),
                        ccr_app::runtime::EnvironmentId::Local(_)
                    ) =>
            {
                self.review_codex_save(window, cx)
            }
            Some(AccountForm::Save) => {
                let action = if self.platform == Platform::Grok {
                    AccountAction::SaveFrom(self.account_scope.read(cx).value().trim().to_owned())
                } else {
                    AccountAction::Save
                };
                self.account(name, action, cx);
            }
            _ => {}
        }
    }
    fn render_name_form(&self, blocked: bool, cx: &mut Context<Self>) -> AnyElement {
        let title = match &self.account_form {
            Some(AccountForm::Rename(target)) => format!(
                "{} · {target}",
                text(cx, "Rename saved account", "重命名已保存账户")
            ),
            _ => text(cx, "Save current credentials", "保存当前运行凭据").to_owned(),
        };
        let label = text(cx, "New account name", "新账户名");
        let mut form = div()
            .id("platform-account-form")
            .test_support()
            .role(Role::Group)
            .aria_label(title.clone())
            .flex()
            .flex_col()
            .w_full()
            .max_w(px(400.))
            .gap(px(6.))
            .child(title)
            .child(label)
            .child(
                Input::new(&self.account_input)
                    .id("platform-account-name")
                    .aria_label(label)
                    .readonly(blocked),
            );
        if self.platform == Platform::Grok {
            let label = text(
                cx,
                "OAuth source (issuer::client)",
                "OAuth 来源（issuer::client）",
            );
            form = form.child(label);
            if let Some(session) = &self.session {
                for fact in session.fields.iter().filter(|f| f.key == "oauth_source") {
                    form = form.child(
                        div()
                            .text_size(theme::text_size(12.))
                            .child(fact.value.clone()),
                    );
                }
            }
            form = form.child(
                Input::new(&self.account_scope)
                    .aria_label(label)
                    .readonly(blocked),
            );
        }
        form.child(
            div()
                .flex()
                .gap(px(6.))
                .child(semantics::button(
                    Button::new("platform-name-submit")
                        .label(text(cx, "Save", "保存"))
                        .on_click(cx.listener(|this, _, w, cx| this.submit_account_form(w, cx))),
                    blocked,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("platform-name-cancel")
                        .label(text(cx, "Cancel", "取消"))
                        .ghost()
                        .on_click(cx.listener(|this, _, w, cx| this.cancel_account_form(w, cx))),
                    blocked,
                    false,
                    cx,
                )),
        )
        .into_any_element()
    }
    pub(super) fn reload_accounts(&self, blocked: bool, cx: &mut Context<Self>) -> Button {
        semantics::button(
            Button::new("platform-load")
                .ghost()
                .label(text(cx, "Refresh accounts", "刷新账户"))
                .on_click(cx.listener(|this, _, _, cx| this.load(cx))),
            blocked,
            false,
            cx,
        )
    }
    #[inline(never)]
    pub(super) fn render_accounts(
        &mut self,
        mut root: gpui_kit::Stateful<gpui_kit::Div>,
        blocked: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
        oauth_active: bool,
    ) -> gpui_kit::Stateful<gpui_kit::Div> {
        let Some(session) = self.session.clone() else {
            return root;
        };
        let editing = self.account_form.is_some();
        let controls_blocked = blocked || editing;
        let writes_blocked =
            controls_blocked || session.credential.file_account_write_access().is_err();
        let runtime = text(cx, "Runtime credentials", "运行凭据");
        let source = match self.platform {
            Platform::Claude => ".claude/.credentials.json",
            Platform::Grok => ".grok/auth.json",
            _ => "auth.json",
        };
        let scope = if session.remote.is_some() {
            text(cx, "SSH host", "SSH 主机")
        } else {
            text(cx, "Local", "本机")
        };
        let status = auth_status(cx, &session);
        let mut summary = div()
            .id("platform-accounts-current")
            .test_support()
            .role(Role::Group)
            .aria_label(runtime)
            .track_focus(&self.account_summary_focus)
            .tab_stop(false)
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(badge::section_heading(runtime, None, cx))
            .child(
                div()
                    .text_size(theme::text_size(12.))
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("{scope} · {source}")),
            )
            .child(
                div()
                    .id("platform-auth-status")
                    .test_support()
                    .role(Role::Status)
                    .aria_label(status.clone())
                    .child(status),
            );
        for (index, fact) in session
            .fields
            .iter()
            .enumerate()
            .filter(|(_, f)| f.key == "oauth_source")
        {
            let label = format!(
                "{} · {}",
                text(cx, "OAuth source", "OAuth 来源"),
                fact.value
            );
            summary = summary.child(
                div()
                    .id(SharedString::from(format!("platform-oauth-source-{index}")))
                    .test_support()
                    .role(Role::Label)
                    .aria_label(label.clone())
                    .text_size(theme::text_size(12.))
                    .text_color(cx.theme().muted_foreground)
                    .child(label),
            );
        }
        if session.credential == CredentialState::Detected {
            let matches: Vec<_> = session
                .accounts
                .iter()
                .filter(|a| a.current)
                .map(|a| a.name.as_str())
                .collect();
            let fact = if matches.is_empty() {
                text(
                    cx,
                    "No unique saved account matches these credentials.",
                    "未匹配到唯一的已保存账户。",
                )
                .to_owned()
            } else {
                format!(
                    "{} · {}",
                    text(cx, "Current account", "当前账户"),
                    matches.join(", ")
                )
            };
            summary = summary.child(
                div()
                    .id("platform-matched-account")
                    .test_support()
                    .role(Role::Status)
                    .aria_label(fact.clone())
                    .child(fact),
            );
        }
        let can_save = session.credential == CredentialState::Detected;
        let can_clear = matches!(
            session.credential,
            CredentialState::Detected | CredentialState::Invalid
        );
        summary = summary.child(
            div()
                .flex()
                .flex_wrap()
                .gap(px(6.))
                .child(semantics::button(
                    Button::new("platform-account-save")
                        .ghost()
                        .label(text(cx, "Save current login", "保存当前登录"))
                        .on_click(
                            cx.listener(|v, _, w, cx| {
                                v.open_account_form(AccountForm::Save, w, cx)
                            }),
                        ),
                    controls_blocked || !can_save,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("platform-auth-off")
                        .ghost()
                        .label(text(cx, "Clear runtime authentication", "清除运行认证"))
                        .on_click(cx.listener(|v, _, w, cx| v.decide(Decision::AuthOff, w, cx))),
                    controls_blocked || !can_clear,
                    false,
                    cx,
                ))
                .child(self.reload_accounts(controls_blocked, cx)),
        );
        if matches!(self.account_form, Some(AccountForm::Save)) {
            summary = summary.child(self.render_name_form(blocked, cx));
        }
        root = root.child(summary);
        let title = text(cx, "Saved accounts", "已保存账户");
        let mut group = div()
            .id("platform-accounts-saved")
            .test_support()
            .role(Role::Group)
            .aria_label(title)
            .w_full()
            .max_w(px(680.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .p(px(12.))
            .mt(px(8.))
            .rounded(px(12.))
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(badge::section_heading(title, None, cx));
        let selected = self.selected_account();
        if show_saved_accounts(&session) {
            if session.accounts.is_empty() {
                group = group.child(no_saved_accounts(cx));
            } else {
                group = group.child(self.render_account_list(
                    &session.accounts,
                    selected.as_deref(),
                    controls_blocked,
                    cx,
                ));
                if let Some(name) = selected {
                    let selected = format!("{} · {name}", text(cx, "Selected", "已选"));
                    group = group
                        .child(
                            div()
                                .id("platform-accounts-selection")
                                .test_support()
                                .role(Role::Status)
                                .aria_label(selected.clone())
                                .child(selected),
                        )
                        .child(self.render_selected_actions(&name, writes_blocked, cx));
                } else {
                    let hint = text(
                        cx,
                        "No saved account selected. Select one to switch or manage it.",
                        "选择已保存账户后可切换或管理。",
                    );
                    group = group.child(
                        div()
                            .id("platform-accounts-selection")
                            .test_support()
                            .role(Role::Status)
                            .aria_label(hint)
                            .text_size(theme::text_size(12.))
                            .text_color(cx.theme().muted_foreground)
                            .child(hint),
                    );
                }
            }
        } else {
            group = group.child(text(
                cx,
                "Saved accounts are unavailable for this source.",
                "当前来源无法提供已保存账户。",
            ));
        }
        if matches!(self.account_form, Some(AccountForm::Rename(_))) {
            group = group.child(self.render_name_form(blocked, cx));
        }
        root = root.child(group);
        if self.platform == Platform::Codex {
            root = self.render_workspaces(root, window, cx, writes_blocked, oauth_active);
        }
        root
    }
    fn render_account_list(
        &mut self,
        accounts: &[AccountRow],
        selected: Option<&str>,
        blocked: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut list = div()
            .id("platform-accounts-list")
            .test_support()
            .role(Role::ListBox)
            .aria_label(text(cx, "Saved accounts", "已保存账户"))
            .flex()
            .flex_col();
        for (index, row) in accounts.iter().enumerate() {
            list = list.child(self.render_account_row(
                row,
                index,
                accounts.len(),
                selected == Some(row.name.as_str()),
                blocked,
                cx,
            ));
        }
        list.into_any_element()
    }
    fn render_account_row(
        &mut self,
        row: &AccountRow,
        index: usize,
        count: usize,
        selected: bool,
        blocked: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let name = row.name.clone();
        let click = name.clone();
        let keyboard = name.clone();
        let accessible = name.clone();
        let focus = self
            .account_focus
            .entry(name.clone())
            .or_insert_with(|| cx.focus_handle())
            .clone();
        let clicked_focus = focus.clone();
        let weak = cx.weak_entity();
        let label = account_label(row, cx);
        let mut line = div()
            .id(SharedString::from(format!("platform-account-{name}")))
            .test_support()
            .role(Role::ListBoxOption)
            .aria_label(label)
            .aria_selected(selected)
            .aria_position_in_set(index + 1)
            .aria_size_of_set(count)
            .track_focus(&focus)
            .tab_stop(!blocked)
            .a11y_synthetic_children(move |t| {
                if blocked {
                    t.parent_node().set_disabled()
                }
            })
            .on_a11y_action(gpui_kit::AccessibleAction::Click, move |_, _, cx| {
                if !blocked {
                    let _ = weak.update(cx, |v, cx| v.select_account(accessible.clone(), cx));
                }
            })
            .on_click(cx.listener(move |v, _, w, cx| {
                if !blocked {
                    w.focus(&clicked_focus, cx);
                    v.select_account(click.clone(), cx);
                }
            }))
            .on_key_down(cx.listener(move |v, e: &KeyDownEvent, w, cx| {
                if blocked {
                    return;
                }
                let target = match e.keystroke.key.as_str() {
                    "up" | "left" => index.saturating_sub(1),
                    "down" | "right" => (index + 1).min(count - 1),
                    "home" => 0,
                    "end" => count - 1,
                    "enter" | "space" => {
                        v.select_account(keyboard.clone(), cx);
                        cx.stop_propagation();
                        return;
                    }
                    _ => return,
                };
                if let Some(name) = v
                    .session
                    .as_ref()
                    .and_then(|s| s.accounts.get(target))
                    .map(|r| r.name.clone())
                {
                    if let Some(focus) = v.account_focus.get(&name) {
                        w.focus(focus, cx);
                    }
                    v.select_account(name, cx);
                }
                cx.stop_propagation();
            }))
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(8.))
            .px(px(8.))
            .py(px(6.))
            .rounded(px(4.))
            .border_2()
            .border_color(gpui_kit::transparent_black())
            .bg(if selected {
                cx.theme().secondary
            } else {
                cx.theme().background
            })
            .focus_visible(|s| s.border_color(cx.theme().ring))
            .child(div().font_weight(FontWeight::MEDIUM).child(name.clone()));
        if row.current {
            line = line.child(badge::tag(
                format!("account-current-{name}"),
                text(cx, "Current account", "当前账户"),
                theme::Tone::Success,
                cx,
            ));
        }
        div()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(line)
            .into_any_element()
    }
    fn render_selected_actions(
        &self,
        name: &str,
        blocked: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let target = name.to_owned();
        let mut actions = div()
            .flex()
            .flex_wrap()
            .gap(px(6.))
            .child(semantics::button(
                Button::new("platform-account-switch")
                    .label(format!("{} {name}", text(cx, "Switch to", "切换到")))
                    .on_click(cx.listener(|v, _, _, cx| {
                        if let Some(n) = v.selected_account() {
                            v.account(n, AccountAction::Switch, cx)
                        }
                    })),
                blocked,
                false,
                cx,
            ));
        if self.platform == Platform::Codex {
            actions = actions.child(semantics::button(
                Button::new("platform-account-rename")
                    .ghost()
                    .label(format!("{} {name}", text(cx, "Rename", "重命名")))
                    .on_click(cx.listener(move |v, _, w, cx| {
                        v.open_account_form(AccountForm::Rename(target.clone()), w, cx)
                    })),
                blocked,
                false,
                cx,
            ));
        }
        actions
            .child(semantics::button(
                Button::new("platform-account-delete")
                    .ghost()
                    .label(format!("{} {name}", text(cx, "Delete", "删除")))
                    .text_color(cx.theme().danger)
                    .on_click(cx.listener(|v, _, w, cx| v.decide(Decision::DeleteAccount, w, cx))),
                blocked,
                false,
                cx,
            ))
            .into_any_element()
    }
}
