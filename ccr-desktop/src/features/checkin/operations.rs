use super::*;
use crate::runtime::checkin::{Operation, OperationError, Outcome};
use ccr_checkin::managers::checkin::builtin_providers::get_builtin_providers;
use ccr_core::Secret;
use ccr_db::models::checkin::{
    CheckinStatus, CreateAccountRequest, CreateProviderRequest, UpdateAccountRequest,
    UpdateProviderRequest,
};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::{AppContext, Entity};
use std::sync::Arc;

pub(super) enum EditorKind {
    Provider(Option<String>),
    Account(Option<String>),
}
pub(super) struct Editor {
    kind: EditorKind,
    fields: Vec<Entity<InputState>>,
    provider: Option<String>,
    enabled: bool,
}
pub(super) enum Confirmation {
    Account(String),
    Provider(String),
}

pub(super) fn status(cx: &App, value: CheckinStatus) -> &'static str {
    match value {
        CheckinStatus::Success => text(cx, "Check-in succeeded.", "签到成功。"),
        CheckinStatus::AlreadyCheckedIn => text(cx, "Already checked in today.", "今日已签到。"),
        CheckinStatus::Failed => text(
            cx,
            "Check-in failed. Check the account credentials and provider access.",
            "签到失败，请检查账户凭据和提供商访问情况。",
        ),
        CheckinStatus::Skipped => text(
            cx,
            "Check-in skipped: account or provider disabled, or provider does not support it.",
            "已跳过：账户或提供商已停用，或提供商不支持签到。",
        ),
    }
}
pub(super) fn error(cx: &App, value: OperationError) -> &'static str {
    match value {
        OperationError::FixturePath => text(
            cx,
            "Choose an absolute file path inside the fixture directory. Its parent directory must already exist.",
            "请选择演示数据目录内的绝对文件路径，且父目录必须已存在。",
        ),
        OperationError::QueryInvalid => text(
            cx,
            "Enter a valid year, month (1–12), trend range (1–366 days), and at most one account for record filtering.",
            "请输入有效年份、月份（1–12）和趋势范围（1–366 天）；记录筛选最多选择一个账户。",
        ),
        OperationError::ImportCredentials => text(
            cx,
            "Import rejected: encrypted cookies require the matching existing key, or a source with readable cookies. Existing credentials were retained.",
            "导入被拒绝：加密 Cookies 需要匹配的既有密钥，或使用可解密的来源；已有凭据已保留。",
        ),
        OperationError::ImportConflict => text(
            cx,
            "Import source or local configuration changed. Preview again before importing.",
            "导入来源或本地配置已变化，请重新预览后导入。",
        ),
        OperationError::ImportInvalid => text(
            cx,
            "Import rejected. Check the legacy configuration format, file path and unsupported fields.",
            "导入被拒绝。请检查旧版配置格式、文件路径及不受支持的字段。",
        ),
        OperationError::Storage => text(
            cx,
            "The operation did not finish. Read saved data before retrying; earlier items may already be saved.",
            "操作未完成。重试前请读取已保存数据，前面的条目可能已保存。",
        ),
        OperationError::Invalid => text(
            cx,
            "Check the required fields, unique provider name and cookies JSON object.",
            "请检查必填项、提供商名称是否重复，以及 Cookies JSON 对象。",
        ),
        OperationError::Busy => text(
            cx,
            "A check-in operation is already running.",
            "签到操作正在运行。",
        ),
        OperationError::Unsupported => text(
            cx,
            "Check-in is available only in Local.",
            "签到仅在本地环境可用。",
        ),
        OperationError::FixtureNetwork => text(
            cx,
            "Provider requests are disabled in fixture mode.",
            "演示数据模式禁止请求提供商。",
        ),
        OperationError::Interrupted => text(
            cx,
            "Operation interrupted; read saved data before retrying.",
            "操作中断，重试前请读取已保存的数据。",
        ),
        OperationError::ProviderInUse => text(
            cx,
            "Delete the provider's accounts first.",
            "请先删除此提供商关联的账户。",
        ),
        OperationError::Network => text(
            cx,
            "Provider request failed. Check connectivity and credentials.",
            "提供商请求失败，请检查连接和凭据。",
        ),
        OperationError::Challenge => text(
            cx,
            "Provider requires a browser challenge. Browser login is not supported here; update cookies from a trusted browser.",
            "提供商需要浏览器验证。此处尚不支持浏览器登录，请从可信浏览器更新 Cookies。",
        ),
        OperationError::Credentials => text(
            cx,
            "Cookies expired. Edit this account to replace them.",
            "Cookies 已过期，请编辑账户并替换。",
        ),
    }
}
impl CheckinView {
    pub(super) fn observe(&mut self, cx: &mut Context<Self>) {
        if self.observation.is_some() {
            return;
        }
        let mut updates = self.services.observe_checkin();
        self.operation = updates.borrow_and_update().clone();
        self.observation = Some(cx.spawn(async move |weak, cx| {
            while updates.changed().await.is_ok() {
                let state = updates.borrow_and_update().clone();
                if weak
                    .update(cx, |this, cx| {
                        let refresh = !state.busy && state.result.is_some();
                        if matches!(state.result, Some(Ok(Outcome::Saved | Outcome::Deleted))) {
                            this.editor = None;
                        }
                        this.operation = state;
                        if refresh {
                            this.refresh(cx);
                        }
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        }));
    }
    pub(super) fn start(&mut self, operation: Operation, cx: &mut Context<Self>) {
        if self.operation.busy {
            return;
        }
        self.observe(cx);
        self.confirmation = None;
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
            platform: Platform::Claude,
            generation: self.generation,
            request_id: self.generation,
        };
        if let Err(value) = self.services.start_checkin(key, operation) {
            let mut state = (*self.operation).clone();
            state.result = Some(Err(value));
            self.operation = Arc::new(state);
        } else {
            self.operation = self.services.checkin_operation_state();
        }
        cx.notify();
    }
    fn edit(&mut self, kind: EditorKind, window: &mut Window, cx: &mut Context<Self>) {
        if self.operation.busy {
            return;
        }
        let count = match kind {
            EditorKind::Provider(_) => 7,
            EditorKind::Account(_) => 3,
        };
        let mut enabled = true;
        let mut provider = self
            .operation
            .snapshot
            .as_ref()
            .and_then(|s| s.providers.first())
            .map(|p| p.id.clone());
        match &kind {
            EditorKind::Provider(Some(id)) => {
                if let Some(p) = self
                    .operation
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.providers.iter().find(|p| &p.id == id))
                {
                    enabled = p.enabled;
                }
            }
            EditorKind::Account(Some(id)) => {
                if let Some(a) = self
                    .operation
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.accounts.iter().find(|a| &a.id == id))
                {
                    enabled = a.enabled;
                    provider = Some(a.provider_id.clone());
                }
            }
            _ => {}
        }
        self.editor = Some(Editor {
            kind,
            provider,
            enabled,
            fields: (0..count)
                .map(|index| {
                    cx.new(|cx| InputState::new(window, cx).masked(count == 3 && index != 0))
                })
                .collect(),
        });
        self.confirmation = None;
        cx.notify();
    }
    fn save(&mut self, cx: &mut Context<Self>) {
        let Some(editor) = &self.editor else { return };
        let values: Vec<String> = editor
            .fields
            .iter()
            .map(|f| f.read(cx).value().to_string())
            .collect();
        let optional = |index: usize| (!values[index].is_empty()).then(|| values[index].clone());
        let operation = match &editor.kind {
            EditorKind::Provider(None) => Operation::AddProvider(CreateProviderRequest {
                name: values[0].clone(),
                base_url: values[1].clone(),
                checkin_path: optional(2),
                balance_path: optional(3),
                user_info_path: optional(4),
                auth_header: optional(5),
                auth_prefix: optional(6),
                builtin_id: None,
            }),
            EditorKind::Provider(Some(id)) => Operation::UpdateProvider(
                id.clone(),
                UpdateProviderRequest {
                    name: optional(0),
                    base_url: optional(1),
                    checkin_path: optional(2),
                    balance_path: optional(3),
                    user_info_path: optional(4),
                    auth_header: optional(5),
                    auth_prefix: optional(6),
                    enabled: Some(editor.enabled),
                },
            ),
            EditorKind::Account(None) => {
                let Some(provider) = &editor.provider else {
                    return;
                };
                Operation::AddAccount(CreateAccountRequest {
                    provider_id: provider.clone(),
                    name: values[0].clone(),
                    cookies_json: Secret::new(values[1].clone()),
                    api_user: values[2].clone(),
                    extra_config: "{}".into(),
                })
            }
            EditorKind::Account(Some(id)) => Operation::UpdateAccount(
                id.clone(),
                UpdateAccountRequest {
                    name: optional(0),
                    cookies_json: optional(1).map(Secret::new),
                    api_user: optional(2),
                    enabled: Some(editor.enabled),
                    extra_config: None,
                },
            ),
        };
        self.start(operation, cx);
    }
    pub(super) fn render_operation_result(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let locked = self.operation.busy;
        let mut section = div().flex_shrink_0();
        if locked {
            section=section.child(text(cx,"Operation running. You may leave this page; it will retain the result. Active provider calls cannot be cancelled.","操作正在运行。离开此页仍会保留结果；进行中的提供商请求无法取消。"));
        } else if let Some(result) = self.operation.result {
            let message = match result {
                Ok(Outcome::Extended) => text(
                    cx,
                    "Account operation completed; see the result below.",
                    "账户操作完成，请查看下方结果。",
                ),
                Ok(Outcome::Loaded) => text(cx, "Management data loaded.", "管理数据已读取。"),
                Ok(Outcome::Saved) => text(cx, "Check-in settings saved.", "签到设置已保存。"),
                Ok(Outcome::Deleted) => text(cx, "Deleted.", "已删除。"),
                Ok(Outcome::Checked(value)) => status(cx, value),
                Ok(Outcome::BalanceUpdated) => {
                    text(cx, "Balance refreshed and stored.", "余额已刷新并保存。")
                }
                Ok(Outcome::Records) => {
                    text(cx, "Latest saved records loaded.", "已读取最近保存的记录。")
                }
                Err(value) => error(cx, value),
            };
            section = section.child(
                div()
                    .id("checkin-operation-status")
                    .test_support()
                    .role(gpui_kit::Role::Status)
                    .aria_label(message)
                    .child(message),
            );
        }
        section
    }
    pub(super) fn render_operations(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let locked = self.operation.busy;
        let mut section = div()
            .id("checkin-operations")
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP));
        section = section.child(self.render_operation_result(cx));
        section = section.child(semantics::button(
            Button::new("checkin-manage")
                .label(text(
                    cx,
                    "Manage providers and accounts",
                    "管理提供商与账户",
                ))
                .on_click(cx.listener(|this, _, _, cx| this.start(Operation::Manage, cx))),
            locked,
            false,
            cx,
        ));
        section = section.children(self.render_selected_actions(cx));
        if let Some(snapshot) = &self.operation.snapshot {
            section = section.children(self.render_management_actions(snapshot, cx));
            section = section.children(self.render_provider_rows(snapshot, cx));
            section = section.children(self.render_account_rows(snapshot, cx));
            section = section.children(self.render_saved_records(snapshot, cx));
        }
        section = section.children(self.render_management_editor(cx));
        section = section.children(self.render_entry_confirmation(cx));
        section.into_any_element()
    }

    fn render_selected_actions(&self, cx: &mut Context<Self>) -> Vec<gpui_kit::AnyElement> {
        let mut children = Vec::new();
        let locked = self.operation.busy;
        if let Some(id) = self.selected.clone() {
            let balance = id.clone();
            let records = id.clone();
            children.push(
                (div()
                    .flex()
                    .gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(
                        Button::new("checkin-execute")
                            .label(text(cx, "Check in now", "立即签到"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.start(Operation::Checkin(id.clone()), cx)
                            })),
                        locked,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("checkin-balance")
                            .label(text(cx, "Refresh balance", "刷新余额"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.start(Operation::Balance(balance.clone()), cx)
                            })),
                        locked,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("checkin-records")
                            .label(text(cx, "Read records", "读取记录"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.start(Operation::Records(records.clone()), cx)
                            })),
                        locked,
                        false,
                        cx,
                    )))
                .into_any_element(),
            );
        }

        children
    }

    fn render_management_actions(
        &self,
        snapshot: &crate::runtime::checkin::ManagementSnapshot,
        cx: &mut Context<Self>,
    ) -> Vec<gpui_kit::AnyElement> {
        let mut children = Vec::new();
        let locked = self.operation.busy;

        children.push(
            (div()
                .flex()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("checkin-new-provider")
                        .label(text(cx, "Add provider", "新增提供商"))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.edit(EditorKind::Provider(None), window, cx)
                        })),
                    locked,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("checkin-new-account")
                        .label(text(cx, "Add account", "新增账户"))
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.edit(EditorKind::Account(None), window, cx)
                        })),
                    locked || snapshot.providers.is_empty(),
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("checkin-catalog")
                        .label(text(cx, "Builtin providers", "内置提供商"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.catalog = !this.catalog;
                            cx.notify();
                        })),
                    locked,
                    false,
                    cx,
                )))
            .into_any_element(),
        );
        if self.catalog {
            for (index, builtin) in get_builtin_providers().into_iter().enumerate() {
                let id = builtin.id;
                children.push(
                    (semantics::button(
                        Button::new(("checkin-builtin", index))
                            .label(builtin.name)
                            .ghost()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.start(Operation::AddBuiltin(id.clone()), cx)
                            })),
                        locked,
                        false,
                        cx,
                    ))
                    .into_any_element(),
                );
            }
        }

        children
    }

    fn render_provider_rows(
        &self,
        snapshot: &crate::runtime::checkin::ManagementSnapshot,
        cx: &mut Context<Self>,
    ) -> Vec<gpui_kit::AnyElement> {
        let mut children = Vec::new();
        let locked = self.operation.busy;
        for (index, p) in snapshot.providers.iter().enumerate() {
            let id = p.id.clone();
            let delete = id.clone();
            children.push(
                (div()
                    .flex()
                    .gap(px(theme::CONTROL_GAP))
                    .child(ccr_core::core::log_redact::redact_log_text(&p.name))
                    .child(semantics::button(
                        Button::new(("checkin-edit-provider", index))
                            .label(text(cx, "Edit provider", "编辑提供商"))
                            .ghost()
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.edit(EditorKind::Provider(Some(id.clone())), window, cx)
                            })),
                        locked,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new(("checkin-delete-provider", index))
                            .label(text(cx, "Delete provider", "删除提供商"))
                            .ghost()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.confirmation = Some(Confirmation::Provider(delete.clone()));
                                cx.notify();
                            })),
                        locked,
                        false,
                        cx,
                    )))
                .into_any_element(),
            );
        }

        children
    }

    fn render_account_rows(
        &self,
        snapshot: &crate::runtime::checkin::ManagementSnapshot,
        cx: &mut Context<Self>,
    ) -> Vec<gpui_kit::AnyElement> {
        let mut children = Vec::new();
        let locked = self.operation.busy;
        for (index, a) in snapshot.accounts.iter().enumerate() {
            let id = a.id.clone();
            let delete = id.clone();
            children.push(
                (div()
                    .flex()
                    .gap(px(theme::CONTROL_GAP))
                    .child(a.name.clone())
                    .child(semantics::button(
                        Button::new(("checkin-edit-account", index))
                            .label(text(cx, "Edit account", "编辑账户"))
                            .ghost()
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.edit(EditorKind::Account(Some(id.clone())), window, cx)
                            })),
                        locked,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new(("checkin-delete-account", index))
                            .label(text(cx, "Delete account", "删除账户"))
                            .ghost()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.confirmation = Some(Confirmation::Account(delete.clone()));
                                cx.notify();
                            })),
                        locked,
                        false,
                        cx,
                    )))
                .into_any_element(),
            );
        }

        children
    }

    fn render_saved_records(
        &self,
        snapshot: &crate::runtime::checkin::ManagementSnapshot,
        cx: &mut Context<Self>,
    ) -> Vec<gpui_kit::AnyElement> {
        let mut children = Vec::new();
        for record in &snapshot.records {
            if self.selected.as_deref() == Some(&record.account_id) {
                children.push(
                    (format!("{} {}", record.at, status(cx, record.status))).into_any_element(),
                );
            }
        }

        children
    }

    fn render_management_editor(&self, cx: &mut Context<Self>) -> Vec<gpui_kit::AnyElement> {
        let mut children = Vec::new();
        let locked = self.operation.busy;
        if let Some(editor) = &self.editor {
            let provider = matches!(editor.kind, EditorKind::Provider(_));
            let labels = if provider {
                vec![
                    ("Name", "名称"),
                    ("Base URL", "基础 URL"),
                    ("Check-in path", "签到路径"),
                    ("Balance path", "余额路径"),
                    ("User info path", "用户信息路径"),
                    ("Auth header", "认证头"),
                    ("Auth prefix", "认证前缀"),
                ]
            } else {
                vec![
                    ("Account name", "账户名称"),
                    ("Cookies JSON", "Cookies JSON"),
                    ("API user", "API 用户标识"),
                ]
            };
            let mut form=div().id("checkin-editor").flex().flex_col().gap(px(theme::CONTROL_GAP))
                .child(text(cx,"For edits, blank fields retain saved values. Cookies must be a JSON object and are encrypted before storage. Browser login and CDK redemption are not available here.","编辑时留空保留原值。Cookies 必须是 JSON 对象，写入前会加密。此处尚不支持浏览器登录及 CDK 兑换。"));
            for (index, field) in editor.fields.iter().enumerate() {
                form = form.child(
                    Input::new(field)
                        .aria_label(text(cx, labels[index].0, labels[index].1))
                        .accessibility_id(format!("checkin-field-{index}")),
                );
            }
            if matches!(editor.kind, EditorKind::Account(None))
                && let Some(snapshot) = &self.operation.snapshot
            {
                for (index, p) in snapshot.providers.iter().enumerate() {
                    let id = p.id.clone();
                    form = form.child(semantics::button(
                        Button::new(("checkin-provider-choice", index))
                            .label(ccr_core::core::log_redact::redact_log_text(&p.name))
                            .selected(editor.provider.as_deref() == Some(&p.id))
                            .ghost()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if let Some(editor) = &mut this.editor {
                                    editor.provider = Some(id.clone());
                                }
                                cx.notify();
                            })),
                        locked,
                        false,
                        cx,
                    ));
                }
            }
            if matches!(
                editor.kind,
                EditorKind::Account(Some(_)) | EditorKind::Provider(Some(_))
            ) {
                form = form.child(semantics::button(
                    Button::new("checkin-editor-enabled")
                        .label(if editor.enabled {
                            text(cx, "Enabled", "已启用")
                        } else {
                            text(cx, "Disabled", "已停用")
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(editor) = &mut this.editor {
                                editor.enabled = !editor.enabled;
                            }
                            cx.notify();
                        })),
                    locked,
                    false,
                    cx,
                ));
            }
            children.push(
                (form.child(
                    div()
                        .flex()
                        .gap(px(theme::CONTROL_GAP))
                        .child(semantics::button(
                            Button::new("checkin-save")
                                .label(text(cx, "Save", "保存"))
                                .on_click(cx.listener(|this, _, _, cx| this.save(cx))),
                            locked,
                            false,
                            cx,
                        ))
                        .child(semantics::button(
                            Button::new("checkin-cancel-edit")
                                .label(text(cx, "Cancel editing", "取消编辑"))
                                .ghost()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.editor = None;
                                    cx.notify();
                                })),
                            locked,
                            false,
                            cx,
                        )),
                ))
                .into_any_element(),
            );
        }

        children
    }

    fn render_entry_confirmation(&self, cx: &mut Context<Self>) -> Vec<gpui_kit::AnyElement> {
        let mut children = Vec::new();
        let locked = self.operation.busy;
        if self.confirmation.is_some() {
            children.push((div().flex().flex_col().gap(px(theme::CONTROL_GAP)).child(text(cx,"Delete this saved entry? Deleting an account also removes its records and balances.","删除此条目？删除账户也会移除其记录和余额。"))
                .child(semantics::button(Button::new("checkin-confirm-delete").label(text(cx,"Confirm deletion","确认删除")).on_click(cx.listener(|this,_,_,cx|{if let Some(value)=this.confirmation.take(){this.start(match value{Confirmation::Account(id)=>Operation::DeleteAccount(id),Confirmation::Provider(id)=>Operation::DeleteProvider(id)},cx);}})),locked,false,cx))
                .child(semantics::button(Button::new("checkin-cancel-delete").label(text(cx,"Cancel","取消")).ghost().on_click(cx.listener(|this,_,_,cx|{this.confirmation=None;cx.notify();})),locked,false,cx))).into_any_element());
        }
        children
    }
}
