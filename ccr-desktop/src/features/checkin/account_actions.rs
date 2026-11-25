//! Account-scoped diagnostics, CDK drafts and entry confirmations.
use super::*;
use crate::runtime::checkin::{
    ExtendedCommand, ExtendedReport, InspectionCommand, InspectionReport, Operation,
};
use ccr_checkin::managers::checkin::builtin_providers::get_builtin_provider_by_id;
use gpui_kit::component::input::Input;

pub(super) struct CdkDraft {
    code: Entity<InputState>,
    patch: Entity<InputState>,
}

pub(super) struct Confirmation {
    operation: Operation,
    notice: String,
    source: String,
    offset: Point<Pixels>,
}

impl CheckinView {
    pub(super) fn confirm_entry(
        &mut self,
        operation: Operation,
        notice: String,
        source: &str,
        cx: &mut Context<Self>,
    ) {
        self.confirmation = Some(Confirmation {
            operation,
            notice,
            source: source.to_owned(),
            offset: self.scroll.offset(),
        });
        self.scroll.set_offset(Default::default());
        self.focus_control("entry-cancel");
        cx.notify();
    }

    pub(super) fn confirm_account(
        &mut self,
        id: &str,
        operation: Operation,
        message: &str,
        source: &str,
        cx: &mut Context<Self>,
    ) {
        let Some(row) = self.rows().iter().find(|a| a.id == id) else {
            return;
        };
        let target = format!(
            "{} · {} · {message}",
            row.name,
            row.provider_name.as_deref().unwrap_or(&row.provider_id)
        );
        self.confirm_entry(operation, target, source, cx);
    }

    fn finish_confirmation(&mut self, run: bool, cx: &mut Context<Self>) {
        if let Some(pending) = self.confirmation.take() {
            self.scroll.set_offset(pending.offset);
            self.focus_control(&pending.source);
            if run {
                if matches!(
                    pending.operation,
                    Operation::Extended(
                        ExtendedCommand::Recharge { .. } | ExtendedCommand::CdkCredentials { .. }
                    )
                ) {
                    self.cdk = None;
                    self.focus_control("cdk");
                }
                self.start(pending.operation, cx);
            }
        }
        cx.notify();
    }

    pub(super) fn entry_confirmation(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let Some(pending) = &self.confirmation else {
            return div().into_any_element();
        };
        let (id, label) = match pending.operation {
            Operation::Extended(ExtendedCommand::Delete(_)) if pending.source == "batch-delete" => {
                (
                    "checkin-tools-confirm",
                    text(cx, "Confirm deletion", "确认删除"),
                )
            }
            Operation::Extended(ExtendedCommand::Delete(_)) | Operation::DeleteProvider(_) => (
                "checkin-confirm-delete",
                text(cx, "Confirm deletion", "确认删除"),
            ),
            _ => ("checkin-entry-confirm", text(cx, "Confirm", "确认")),
        };
        div()
            .id("checkin-confirmation")
            .test_support()
            .role(gpui_kit::Role::Group)
            .aria_label(pending.notice.clone())
            .on_key_down(cx.listener(|this, event: &gpui_kit::KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape" {
                    this.finish_confirmation(false, cx);
                    cx.stop_propagation();
                }
            }))
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(pending.notice.clone())
            .child(
                div()
                    .flex()
                    .gap(px(theme::CONTROL_GAP))
                    .child(
                        self.origin_control(
                            "entry-cancel",
                            semantics::ghost_button(
                                Button::new("checkin-cancel-delete")
                                    .label(text(cx, "Cancel", "取消"))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.finish_confirmation(false, cx)
                                    })),
                                false,
                                false,
                                cx,
                            ),
                        ),
                    )
                    .child(semantics::button(
                        Button::new(id).label(label).on_click(
                            cx.listener(|this, _, _, cx| this.finish_confirmation(true, cx)),
                        ),
                        false,
                        false,
                        cx,
                    )),
            )
            .into_any_element()
    }

    pub(super) fn diagnostic_control(
        &self,
        account: &str,
        source: &'static str,
        waf: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let id = account.to_owned();
        let label = if waf {
            text(cx, "Validate cached WAF access", "校验已缓存 WAF 访问")
        } else {
            text(cx, "Test account connection", "测试账户连接")
        };
        let button = Button::new(source)
            .label(label)
            .on_click(cx.listener(move |this, _, _, cx| {
            let (command, message) = if waf {
                (
                    InspectionCommand::Waf(id.clone()),
                    text(
                        cx,
                        "Contact this provider with saved credentials to validate cached access?",
                        "使用已保存的凭据访问此提供商以校验缓存？",
                    ),
                )
            } else {
                (
                    InspectionCommand::Connection(id.clone()),
                    text(
                        cx,
                        "Contact this provider with the saved account credentials to test access?",
                        "使用此账户已保存的凭据访问该提供商以测试连接？",
                    ),
                )
            };
            this.confirm_account(
                &id,
                Operation::Extended(ExtendedCommand::Inspect(command)),
                message,
                source,
                cx,
            );
        }));
        self.origin_control(
            source,
            semantics::ghost_button(button, self.operation.busy, false, cx),
        )
    }

    pub(super) fn account_diagnostics(
        &self,
        account: &CheckinAccountRow,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let id = account.id.clone();
        let cdk_supported = self
            .data()
            .and_then(|data| data.providers.iter().find(|p| p.id == account.provider_id))
            .and_then(|p| p.builtin_id.as_deref())
            .and_then(get_builtin_provider_by_id)
            .is_some_and(|p| p.cdk_config.is_some());
        let mut actions = div()
            .flex()
            .flex_wrap()
            .gap(px(theme::CONTROL_GAP))
            .child(self.diagnostic_control(&id, "checkin-detail-connection", false, cx));
        if cdk_supported {
            actions = actions.child(
                self.origin_control(
                    "cdk",
                    semantics::ghost_button(
                        Button::new("checkin-detail-cdk")
                            .label(text(cx, "CDK recharge", "CDK 充值"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.cdk = if this.cdk.is_some() {
                                    None
                                } else {
                                    Some(CdkDraft {
                                        code: cx.new(|cx| InputState::new(window, cx).masked(true)),
                                        patch: cx
                                            .new(|cx| InputState::new(window, cx).masked(true)),
                                    })
                                };
                                cx.notify();
                            })),
                        self.operation.busy,
                        false,
                        cx,
                    ),
                ),
            );
        }
        let mut panel = div()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .child(actions);
        if let Some(draft) = self.cdk.as_ref().filter(|_| cdk_supported) {
            let code = draft.code.clone();
            let patch = draft.patch.clone();
            let charge_account = account.id.clone();
            let save_account = account.id.clone();
            let recharge = Button::new("checkin-cdk-recharge")
                .label(text(cx, "Recharge", "充值"))
                .on_click(cx.listener(move |this, _, _, cx| {
                    let command = ExtendedCommand::Recharge {
                        account: charge_account.clone(),
                        code: code.read(cx).value().to_string().into(),
                    };
                    let message = text(
                        cx,
                        "Send a recharge request? Redemption cannot be cancelled once sent.",
                        "发送兑换请求？提交后无法取消。",
                    );
                    this.confirm_account(
                        &charge_account,
                        Operation::Extended(command),
                        message,
                        "recharge",
                        cx,
                    );
                }));
            let save = Button::new("checkin-cdk-save")
                .label(text(cx, "Save CDK credentials", "保存 CDK 凭据"))
                .on_click(cx.listener(move |this, _, _, cx| {
                    let command = ExtendedCommand::CdkCredentials {
                        account: save_account.clone(),
                        patch: patch.read(cx).value().to_string().into(),
                    };
                    let message = text(
                        cx,
                        "Save the supplied CDK credentials on this device?",
                        "将提供的 CDK 凭据保存到本机？",
                    );
                    this.confirm_account(
                        &save_account,
                        Operation::Extended(command),
                        message,
                        "save-cdk",
                        cx,
                    );
                }));
            let code_label = text(
                cx,
                "Recharge code (blank to request a code from the provider)",
                "充值码（留空将向提供商获取充值码）",
            );
            let patch_label = text(
                cx,
                "CDK credential patch (JSON). Supported fields: fuli_cookies, b4u_cdk_cookies, x666_access_token. Omitted fields retain saved values.",
                "CDK 凭据补丁（JSON）。支持 fuli_cookies、b4u_cdk_cookies、x666_access_token，省略字段保留已有值。",
            );
            panel = panel
                .child(code_label)
                .child(
                    Input::new(&draft.code)
                        .id("checkin-cdk-code")
                        .aria_label(code_label),
                )
                .child(self.origin_control(
                    "recharge",
                    semantics::button(recharge, self.operation.busy, false, cx),
                ))
                .child(patch_label)
                .child(
                    Input::new(&draft.patch)
                        .id("checkin-cdk-credentials")
                        .aria_label(patch_label),
                )
                .child(self.origin_control(
                    "save-cdk",
                    semantics::button(save, self.operation.busy, false, cx),
                ));
        }
        if let Some(ExtendedReport::Inspection(report)) = self.operation.extended.as_deref()
            && matches!(report.as_ref(), InspectionReport::Connection { account: id, .. } if id == &account.id)
        {
            panel = panel.child(self.render_inspection_report(report, cx));
        }
        let message = match self.operation.extended.as_deref() {
            Some(ExtendedReport::CdkSaved { account: id }) if id == &account.id => {
                Some(text(cx, "CDK credentials saved.", "CDK 凭据已保存。"))
            }
            Some(ExtendedReport::Recharged {
                account: id,
                success,
            }) if id == &account.id => Some(if *success {
                text(cx, "Recharge succeeded.", "充值成功。")
            } else {
                text(cx, "Recharge failed.", "充值失败。")
            }),
            _ => None,
        };
        if let Some(message) = message {
            panel = panel.child(message);
        }
        panel
    }
}
