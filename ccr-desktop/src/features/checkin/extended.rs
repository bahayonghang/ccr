//! Native account tools. Secret fields start empty and are released on leaving.
use super::*;
#[path = "inspection.rs"]
mod inspection;
use crate::runtime::checkin::{ExtendedCommand, ExtendedReport, Operation};
use gpui_kit::component::{
    FocusTrapElement,
    input::{Input, InputState},
};
use gpui_kit::{AppContext, Entity};
use inspection::InspectionUi;
use std::collections::BTreeSet;

pub(super) struct ExtendedUi {
    accounts: BTreeSet<String>,
    provider: Option<String>,
    path: Entity<InputState>,
    cookies: Entity<InputState>,
    code: Entity<InputState>,
    cdk_config: Entity<InputState>,
    pending: Option<ExtendedCommand>,
    enter_confirmation: bool,
    focus: FocusHandle,
    clear_after: bool,
    inspection: InspectionUi,
}
impl CheckinView {
    pub(super) fn open_extended(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.operation.busy {
            return;
        }
        self.extended = Some(ExtendedUi {
            accounts: BTreeSet::new(),
            provider: None,
            path: cx.new(|cx| InputState::new(window, cx)),
            cookies: cx.new(|cx| InputState::new(window, cx).masked(true)),
            cdk_config: cx.new(|cx| InputState::new(window, cx).masked(true)),
            code: cx.new(|cx| InputState::new(window, cx).masked(true)),
            pending: None,
            enter_confirmation: false,
            focus: cx.focus_handle(),
            clear_after: false,
            inspection: InspectionUi::new(window, cx),
        });
        self.observe(cx);
        self.operation = self.services.checkin_operation_state();
        if self.operation.snapshot.is_none() {
            self.start(Operation::Manage, cx);
        } else {
            cx.notify();
        }
    }
    fn propose_extended(&mut self, command: ExtendedCommand, cx: &mut Context<Self>) {
        if self.operation.busy {
            return;
        }
        if let Some(editor) = &mut self.extended {
            editor.pending = Some(command);
            editor.enter_confirmation = true;
        }
        cx.notify();
    }
    pub(super) fn render_extended(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let Some(editor) = &mut self.extended else {
            return div().into_any_element();
        };
        if editor.clear_after && !self.operation.busy {
            editor.clear_after = false;
            if self.operation.result.is_some_and(|result| result.is_ok()) {
                for input in [&editor.cookies, &editor.code, &editor.cdk_config] {
                    input.update(cx, |state, cx| state.set_value("", window, cx));
                }
            }
        }
        if let Some(snapshot) = &self.operation.snapshot {
            editor
                .accounts
                .retain(|id| snapshot.accounts.iter().any(|account| &account.id == id));
        }
        let blocked = self.operation.busy || editor.pending.is_some();
        let mut root = div()
            .id("checkin-extended")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .flex_shrink_0()
            .on_key_down(cx.listener(|this, event: &gpui_kit::KeyDownEvent, _, cx| {
                if event.keystroke.key == "escape"
                    && let Some(editor) = &mut this.extended
                    && editor.pending.is_some()
                {
                    editor.pending = None;
                    editor.enter_confirmation = false;
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .child(text(cx, "Account tools", "账户工具"))
            .child(semantics::button(
                Button::new("checkin-tools-close")
                    .label(text(cx, "Back to accounts", "返回账户"))
                    .ghost()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.extended = None;
                        this.services.discard_checkin_preview();
                        this.operation = this.services.checkin_operation_state();
                        cx.notify();
                    })),
                blocked,
                false,
                cx,
            ));
        root = root.child(semantics::button(
            Button::new("checkin-tools-refresh")
                .label(text(cx, "Read management data", "读取管理数据"))
                .ghost()
                .on_click(cx.listener(|this, _, _, cx| this.start(Operation::Manage, cx))),
            blocked,
            false,
            cx,
        ));
        if self
            .extended
            .as_ref()
            .is_some_and(|editor| editor.pending.is_some())
        {
            return root
                .children(self.render_extended_confirmation(window, cx))
                .into_any_element();
        }
        if let Some(snapshot) = &self.operation.snapshot {
            root = root.children(self.render_batch_accounts(snapshot, cx));
            root = root.children(self.render_cdk_controls(cx));
            root = root.children(self.render_waf_controls(snapshot, cx));
        }
        root = root.child(self.render_inspection(cx));
        root = root.children(self.render_transfer_controls(cx));
        root = root.children(self.render_extended_results(cx));
        root.into_any_element()
    }

    fn render_extended_confirmation(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<gpui_kit::AnyElement> {
        let mut children = Vec::new();
        let Some(editor) = &mut self.extended else {
            return children;
        };

        if editor.enter_confirmation {
            editor.enter_confirmation = false;
            window.focus(&editor.focus, cx);
        }
        let message = match editor.pending.as_ref() {
            Some(ExtendedCommand::Delete(_)) => text(
                cx,
                "Delete the selected accounts and their stored records?",
                "删除所选账户及其保存的记录？",
            ),
            Some(ExtendedCommand::Checkin(_)) => text(
                cx,
                "Send check-in requests for the selected accounts? Each outcome is retained.",
                "向所选账户发送签到请求？每个账户的结果都会保留。",
            ),
            Some(ExtendedCommand::CdkCredentials { .. }) => text(
                cx,
                "Save the supplied CDK credential fields? Existing omitted fields remain unchanged.",
                "保存提供的 CDK 凭据字段？未提供的既有字段保留。",
            ),
            Some(ExtendedCommand::Recharge { .. }) => text(
                cx,
                "Recharge this account? This sends an external request and cannot be cancelled once sent.",
                "为此账户充值？操作将发送外部请求，发出后无法取消。",
            ),
            Some(ExtendedCommand::ExportStats(_)) => text(
                cx,
                "Export today's saved statistics to the new file?",
                "将今日已保存的统计导出到新文件？",
            ),
            Some(ExtendedCommand::Export(_)) => text(
                cx,
                "Export configuration to the new file? Cookies remain encrypted for this device. This is not a full database backup.",
                "导出配置到新文件？Cookies 保持当前设备加密；此文件不是完整数据库备份。",
            ),
            Some(ExtendedCommand::Import { .. }) => text(
                cx,
                "Import the previewed configuration? A local database snapshot is saved first and requires this device's existing crypto data.",
                "导入已预览配置？操作前保存本地数据库快照，恢复仍依赖当前设备已有的加密数据。",
            ),
            _ => text(
                cx,
                "Update cached WAF cookies for this provider?",
                "更新此提供商缓存的 WAF Cookies？",
            ),
        };
        match editor.pending.as_ref() {
            Some(ExtendedCommand::Import { session, .. }) => {
                children.push((session.path().display().to_string()).into_any_element());
            }
            Some(ExtendedCommand::Export(path) | ExtendedCommand::ExportStats(path)) => {
                children.push((path.display().to_string()).into_any_element());
            }
            _ => {}
        }
        children.push(
            (div()
                .flex()
                .flex_col()
                .gap(px(theme::CONTROL_GAP))
                .child(message)
                .child(semantics::button(
                    Button::new("checkin-tools-cancel")
                        .label(text(cx, "Cancel", "取消"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(editor) = &mut this.extended {
                                editor.pending = None;
                            }
                            cx.notify();
                        })),
                    false,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("checkin-tools-confirm")
                        .label(text(cx, "Confirm", "确认"))
                        .primary()
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(command) = this
                                .extended
                                .as_mut()
                                .and_then(|editor| editor.pending.take())
                            {
                                if let Some(editor) = &mut this.extended {
                                    editor.clear_after = matches!(
                                        &command,
                                        ExtendedCommand::WafSave { .. }
                                            | ExtendedCommand::CdkCredentials { .. }
                                            | ExtendedCommand::Recharge { .. }
                                    );
                                }
                                this.start(Operation::Extended(command), cx);
                            }
                        })),
                    false,
                    false,
                    cx,
                ))
                .focus_trap("checkin-tools-trap", &editor.focus))
            .into_any_element(),
        );

        children
    }

    fn render_batch_accounts(
        &self,
        snapshot: &crate::runtime::checkin::ManagementSnapshot,
        cx: &mut Context<Self>,
    ) -> Vec<gpui_kit::AnyElement> {
        let mut children = Vec::new();
        let Some(editor) = &self.extended else {
            return children;
        };
        let blocked = self.operation.busy || editor.pending.is_some();
        let chosen = &editor.accounts;

        children.push(
            (text(
                cx,
                "Select accounts for batch operations",
                "选择需要批量操作的账户",
            ))
            .into_any_element(),
        );
        for (index, account) in snapshot.accounts.iter().enumerate() {
            let id = account.id.clone();
            let selected = chosen.contains(&id);
            children.push(
                (semantics::button(
                    Button::new(SharedString::from(format!("checkin-select-{index}")))
                        .label(account.name.clone())
                        .ghost()
                        .selected(selected)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(editor) = &mut this.extended
                                && !editor.accounts.insert(id.clone())
                            {
                                editor.accounts.remove(&id);
                            }
                            cx.notify();
                        })),
                    blocked,
                    false,
                    cx,
                ))
                .into_any_element(),
            );
        }
        children.push(
            (div()
                .flex()
                .flex_wrap()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("checkin-batch-run")
                        .label(text(cx, "Check in selected", "签到所选账户"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(editor) = &this.extended {
                                this.propose_extended(
                                    ExtendedCommand::Checkin(
                                        editor.accounts.iter().cloned().collect(),
                                    ),
                                    cx,
                                );
                            }
                        })),
                    blocked || chosen.is_empty(),
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("checkin-batch-delete")
                        .label(text(cx, "Delete selected", "删除所选账户"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(editor) = &this.extended {
                                this.propose_extended(
                                    ExtendedCommand::Delete(
                                        editor.accounts.iter().cloned().collect(),
                                    ),
                                    cx,
                                );
                            }
                        })),
                    blocked || chosen.is_empty(),
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("checkin-today-stats")
                        .label(text(cx, "Today's statistics", "今日统计"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.start(Operation::Extended(ExtendedCommand::Stats), cx)
                        })),
                    blocked,
                    false,
                    cx,
                )))
            .into_any_element(),
        );
        children.push(
            (semantics::button(
                Button::new("checkin-balance-stats")
                    .label(text(
                        cx,
                        "Saved balance totals by currency",
                        "按货币统计已保存余额",
                    ))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.start(Operation::Extended(ExtendedCommand::BalanceStats), cx)
                    })),
                blocked,
                false,
                cx,
            ))
            .into_any_element(),
        );

        children
    }

    fn render_cdk_controls(&self, cx: &mut Context<Self>) -> Vec<gpui_kit::AnyElement> {
        let mut children = Vec::new();
        let Some(editor) = &self.extended else {
            return children;
        };
        let blocked = self.operation.busy || editor.pending.is_some();
        let chosen = &editor.accounts;
        let code = editor.code.clone();
        let cdk_config = editor.cdk_config.clone();
        if chosen.len() == 1 {
            let account = chosen.first().cloned().unwrap_or_default();
            let credentials = account.clone();
            let balance = account.clone();
            let records = account.clone();
            children.push(
                (div()
                    .flex()
                    .flex_wrap()
                    .gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(
                        Button::new("checkin-history")
                            .label(text(cx, "Saved balance history", "已保存余额历史"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.start(
                                    Operation::Extended(ExtendedCommand::Balances(balance.clone())),
                                    cx,
                                )
                            })),
                        blocked,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("checkin-cdk-records")
                            .label(text(cx, "Check-in records", "签到记录"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.start(Operation::Records(records.clone()), cx)
                            })),
                        blocked,
                        false,
                        cx,
                    )))
                .into_any_element(),
            );
            children.push((text(cx,"CDK recharge uses the provider's existing support. Leave the code blank to fetch a code. Recharge history is not stored separately.","CDK 充值复用提供商既有能力。留空将自动获取充值码；系统不单独保存充值历史。")).into_any_element());
            children.push(
                (Input::new(&code)
                    .id("checkin-cdk-code")
                    .aria_label(text(cx, "CDK code", "CDK 充值码"))
                    .disabled(blocked))
                .into_any_element(),
            );
            children.push(
                (semantics::button(
                    Button::new("checkin-cdk-recharge")
                        .label(text(cx, "Recharge", "充值"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(editor) = &this.extended {
                                let value =
                                    ccr_core::Secret::new(editor.code.read(cx).value().to_string());
                                this.propose_extended(
                                    ExtendedCommand::Recharge {
                                        account: account.clone(),
                                        code: value,
                                    },
                                    cx,
                                );
                            }
                        })),
                    blocked,
                    false,
                    cx,
                ))
                .into_any_element(),
            );
            children.push((text(cx,"CDK credentials patch: fuli_cookies, b4u_cdk_cookies or x666_access_token. Omitted values remain saved; null removes a field.","CDK 凭据补丁：fuli_cookies、b4u_cdk_cookies 或 x666_access_token。省略字段保留，null 删除字段。")).into_any_element());
            children.push(
                (Input::new(&cdk_config)
                    .id("checkin-cdk-credentials")
                    .aria_label(text(cx, "CDK credentials JSON patch", "CDK 凭据 JSON 补丁"))
                    .disabled(blocked))
                .into_any_element(),
            );
            children.push(
                (semantics::button(
                    Button::new("checkin-cdk-save")
                        .label(text(cx, "Save CDK credentials", "保存 CDK 凭据"))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            if let Some(editor) = &this.extended {
                                let patch = ccr_core::Secret::new(
                                    editor.cdk_config.read(cx).value().to_string(),
                                );
                                this.propose_extended(
                                    ExtendedCommand::CdkCredentials {
                                        account: credentials.clone(),
                                        patch,
                                    },
                                    cx,
                                );
                            }
                        })),
                    blocked,
                    false,
                    cx,
                ))
                .into_any_element(),
            );
        }

        children
    }

    fn render_waf_controls(
        &self,
        snapshot: &crate::runtime::checkin::ManagementSnapshot,
        cx: &mut Context<Self>,
    ) -> Vec<gpui_kit::AnyElement> {
        let mut children = Vec::new();
        let Some(editor) = &self.extended else {
            return children;
        };
        let blocked = self.operation.busy || editor.pending.is_some();
        let cookies = editor.cookies.clone();
        let provider = editor.provider.clone();
        children.push((text(cx, "WAF cookies", "WAF Cookies")).into_any_element());
        for (index, item) in snapshot.providers.iter().enumerate() {
            let id = item.id.clone();
            children.push(
                (semantics::button(
                    Button::new(SharedString::from(format!("checkin-waf-provider-{index}")))
                        .label(item.name.clone())
                        .selected(provider.as_ref() == Some(&id))
                        .ghost()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            if let Some(editor) = &mut this.extended {
                                editor.provider = Some(id.clone());
                                editor
                                    .cookies
                                    .update(cx, |input, cx| input.set_value("", window, cx));
                            }
                            cx.notify();
                        })),
                    blocked,
                    false,
                    cx,
                ))
                .into_any_element(),
            );
        }
        children.push(
            (semantics::button(
                Button::new("checkin-waf-status")
                    .label(text(cx, "Read WAF cache status", "读取 WAF 缓存状态"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.start(Operation::Extended(ExtendedCommand::WafList), cx)
                    })),
                blocked,
                false,
                cx,
            ))
            .into_any_element(),
        );
        children.push((text(cx,"Paste a cookie JSON object from your trusted browser. Saved cookie values are never shown here. Embedded browser verification is unavailable.","粘贴可信浏览器中的 Cookie JSON 对象。此处不显示已保存的 Cookie 值，尚无内嵌浏览器验证能力。")).into_any_element());
        children.push(
            (Input::new(&cookies)
                .id("checkin-waf-cookies")
                .aria_label(text(cx, "WAF cookies JSON", "WAF Cookies JSON"))
                .disabled(blocked))
            .into_any_element(),
        );
        if let Some(id) = provider {
            let delete = id.clone();
            children.push(
                (div()
                    .flex()
                    .gap(px(theme::CONTROL_GAP))
                    .child(semantics::button(
                        Button::new("checkin-waf-save")
                            .label(text(cx, "Save WAF cookies", "保存 WAF Cookies"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if let Some(editor) = &this.extended {
                                    let value = ccr_core::Secret::new(
                                        editor.cookies.read(cx).value().to_string(),
                                    );
                                    this.propose_extended(
                                        ExtendedCommand::WafSave {
                                            provider: id.clone(),
                                            cookies: value,
                                        },
                                        cx,
                                    );
                                }
                            })),
                        blocked,
                        false,
                        cx,
                    ))
                    .child(semantics::button(
                        Button::new("checkin-waf-delete")
                            .label(text(cx, "Clear WAF cookies", "清除 WAF Cookies"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.propose_extended(
                                    ExtendedCommand::WafDelete(delete.clone()),
                                    cx,
                                )
                            })),
                        blocked,
                        false,
                        cx,
                    )))
                .into_any_element(),
            );
        }

        children
    }

    fn render_transfer_controls(&self, cx: &mut Context<Self>) -> Vec<gpui_kit::AnyElement> {
        let mut children = Vec::new();
        let Some(editor) = &self.extended else {
            return children;
        };
        let blocked = self.operation.busy || editor.pending.is_some();
        let path = editor.path.clone();

        children.push(
            (text(cx, "Configuration import and export", "配置导入与导出")).into_any_element(),
        );
        children.push(
            (Input::new(&path)
                .id("checkin-transfer-path")
                .aria_label(text(
                    cx,
                    "Absolute configuration file path",
                    "配置文件绝对路径",
                ))
                .disabled(blocked))
            .into_any_element(),
        );
        children.push(
            (div()
                .flex()
                .gap(px(theme::CONTROL_GAP))
                .child(semantics::button(
                    Button::new("checkin-export")
                        .label(text(cx, "Export encrypted configuration", "导出加密配置"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(editor) = &this.extended {
                                let path = editor.path.read(cx).value().to_string().into();
                                this.propose_extended(ExtendedCommand::Export(path), cx);
                            }
                        })),
                    blocked,
                    false,
                    cx,
                ))
                .child(semantics::button(
                    Button::new("checkin-import-preview")
                        .label(text(cx, "Preview import", "预览导入"))
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(editor) = &this.extended {
                                let path = editor.path.read(cx).value().to_string().into();
                                this.start(
                                    Operation::Extended(ExtendedCommand::PreviewImport(path)),
                                    cx,
                                );
                            }
                        })),
                    blocked,
                    false,
                    cx,
                )))
            .into_any_element(),
        );
        children.push(
            (semantics::button(
                Button::new("checkin-export-stats")
                    .label(text(cx, "Export today's statistics", "导出今日统计"))
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(editor) = &this.extended {
                            let path = editor.path.read(cx).value().to_string().into();
                            this.propose_extended(ExtendedCommand::ExportStats(path), cx);
                        }
                    })),
                blocked,
                false,
                cx,
            ))
            .into_any_element(),
        );

        children
    }

    fn render_extended_results(&self, cx: &mut Context<Self>) -> Vec<gpui_kit::AnyElement> {
        let mut children = Vec::new();
        let Some(editor) = &self.extended else {
            return children;
        };
        let blocked = self.operation.busy || editor.pending.is_some();
        if let Some(snapshot) = &self.operation.snapshot {
            for record in &snapshot.records {
                children.push(
                    (format!(
                        "{} · {} · {}",
                        record.account_id,
                        record.at,
                        super::operations::status(cx, record.status)
                    ))
                    .into_any_element(),
                );
            }
        }
        if let Some(report) = self.operation.extended.clone() {
            match report.as_ref() {
                ExtendedReport::Inspection(report) => {
                    children.push((self.render_inspection_report(report, cx)).into_any_element());
                }
                ExtendedReport::Batch(items) => {
                    for item in items {
                        children.push(
                            (format!(
                                "{}: {}",
                                item.account,
                                match item.result {
                                    Ok(value) => super::operations::status(cx, value),
                                    Err(error) => super::operations::error(cx, error),
                                }
                            ))
                            .into_any_element(),
                        );
                    }
                }
                ExtendedReport::Deleted { completed, failed } => {
                    let summary = format!(
                        "{}: {completed}; {}: {}",
                        text(cx, "Deleted", "已删除"),
                        text(cx, "Failed", "失败"),
                        failed.len()
                    );
                    children.push(
                        (div()
                            .id("checkin-tools-result")
                            .test_support()
                            .role(gpui_kit::Role::Status)
                            .aria_label(SharedString::from(summary.clone()))
                            .child(summary))
                        .into_any_element(),
                    );
                }
                ExtendedReport::Stats {
                    total,
                    checked,
                    pending,
                    failed,
                } => {
                    children.push(
                        (format!(
                            "{}: {total}; {}: {checked}; {}: {pending}; {}: {failed}",
                            text(cx, "Enabled accounts", "启用账户"),
                            text(cx, "Checked in", "已签到"),
                            text(cx, "Pending", "待签到"),
                            text(cx, "Failed", "失败")
                        ))
                        .into_any_element(),
                    );
                }
                ExtendedReport::Balances(value) => {
                    for row in &value.history {
                        children.push(
                            (format!(
                                "{} · {} {} · {} {}",
                                row.recorded_at,
                                row.remaining_quota,
                                row.currency,
                                text(cx, "Used", "已用"),
                                row.used_quota
                            ))
                            .into_any_element(),
                        );
                    }
                }
                ExtendedReport::Waf(rows) => {
                    for row in rows {
                        children.push(
                            (format!(
                                "{} · {} · {}",
                                row.provider,
                                if row.available {
                                    text(cx, "Cached", "已缓存")
                                } else {
                                    text(cx, "Missing or expired", "缺失或过期")
                                },
                                row.required.join(", ")
                            ))
                            .into_any_element(),
                        );
                    }
                }
                ExtendedReport::WafSaved => {
                    children.push(
                        (text(cx, "WAF cache updated.", "WAF 缓存已更新。")).into_any_element(),
                    );
                }
                ExtendedReport::BalanceStats { groups, unusable } => {
                    for group in groups {
                        children.push(
                            (format!(
                                "{} · {}: {} · {}: {} · {}: {} · {}: {}",
                                group.currency,
                                text(cx, "Accounts", "账户"),
                                group.accounts,
                                text(cx, "Remaining", "剩余"),
                                group.remaining,
                                text(cx, "Used", "已用"),
                                group.used,
                                text(cx, "Total quota", "总配额"),
                                group.total
                            ))
                            .into_any_element(),
                        );
                    }
                    if *unusable > 0 {
                        children.push(
                            (format!(
                                "{}: {unusable}",
                                text(cx, "Invalid saved rows omitted", "已排除无效保存数据")
                            ))
                            .into_any_element(),
                        );
                    }
                }
                ExtendedReport::Exported { path, stats } => {
                    children.push(
                        (format!(
                            "{} {}",
                            if *stats {
                                text(cx, "Statistics exported:", "统计已导出：")
                            } else {
                                text(cx, "Configuration exported:", "配置已导出：")
                            },
                            path.display()
                        ))
                        .into_any_element(),
                    );
                }
                ExtendedReport::Preview(session) => {
                    children.push((session.path().display().to_string()).into_any_element());
                    children.push(
                        (format!(
                            "{}: {} {}; {}: {} {}",
                            text(cx, "Providers", "提供商"),
                            session.preview.new_providers,
                            session.preview.conflicting_providers,
                            text(cx, "Accounts", "账户"),
                            session.preview.new_accounts,
                            session.preview.conflicting_accounts
                        ))
                        .into_any_element(),
                    );
                    for overwrite in [false, true] {
                        let session = session.clone();
                        children.push(
                            (semantics::button(
                                Button::new(if overwrite {
                                    "checkin-import-overwrite"
                                } else {
                                    "checkin-import-skip"
                                })
                                .label(if overwrite {
                                    text(cx, "Import and overwrite conflicts", "导入并覆盖冲突")
                                } else {
                                    text(cx, "Import and skip conflicts", "导入并跳过冲突")
                                })
                                .on_click(cx.listener(
                                    move |this, _, _, cx| {
                                        this.propose_extended(
                                            ExtendedCommand::Import {
                                                session: session.clone(),
                                                overwrite,
                                            },
                                            cx,
                                        )
                                    },
                                )),
                                blocked,
                                false,
                                cx,
                            ))
                            .into_any_element(),
                        );
                    }
                }
                ExtendedReport::Imported(value) => {
                    children.push(
                        (format!(
                            "{}: {}; {}: {}; {}: {}",
                            text(cx, "Imported providers", "已导入提供商"),
                            value.providers_imported,
                            text(cx, "Imported accounts", "已导入账户"),
                            value.accounts_imported,
                            text(
                                cx,
                                "Accounts requiring new cookies",
                                "需重新输入 Cookies 的账户"
                            ),
                            value.accounts_need_reauth
                        ))
                        .into_any_element(),
                    );
                }
                ExtendedReport::CdkSaved => {
                    children.push(
                        (text(cx, "CDK credentials saved.", "CDK 凭据已保存。")).into_any_element(),
                    );
                }
                ExtendedReport::Recharged { success } => {
                    children.push(
                        (if *success {
                            text(cx, "Recharge succeeded.", "充值成功。")
                        } else {
                            text(
                                cx,
                                "Recharge failed. Check provider access and credentials.",
                                "充值失败，请检查提供商访问与凭据。",
                            )
                        })
                        .into_any_element(),
                    );
                }
            }
        }

        children
    }
}
