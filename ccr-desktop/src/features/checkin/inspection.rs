//! Explicit account diagnostics. Historical queries live in the records tab and detail.
use super::*;
use crate::runtime::checkin::InspectionReport;
impl CheckinView {
    pub(super) fn render_inspection(&self, cx: &mut Context<Self>) -> gpui_kit::AnyElement {
        let mut row = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
        if let Some(editor) = &self.extended
            && editor.accounts.len() == 1
            && let Some(account) = editor.accounts.first()
        {
            row = row
                .child(self.diagnostic_control(account, "checkin-test-connection", false, cx))
                .child(self.diagnostic_control(account, "checkin-validate-waf", true, cx));
        }
        row.into_any_element()
    }
    pub(in crate::features::checkin) fn render_inspection_report(
        &self,
        report: &InspectionReport,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let mut root = div()
            .id("checkin-inspection-result")
            .test_support()
            .flex()
            .flex_col()
            .gap(px(theme::CONTROL_GAP))
            .flex_shrink_0();
        match report {
            InspectionReport::Connection { account, success } => {
                let label = format!(
                    "{} · {}",
                    account,
                    if *success {
                        text(cx, "Connection succeeded", "连接成功")
                    } else {
                        text(
                            cx,
                            "Provider response did not confirm access",
                            "提供商响应未确认访问成功",
                        )
                    }
                );
                root = root
                    .role(gpui_kit::Role::Status)
                    .aria_label(SharedString::from(label.clone()))
                    .child(label);
            }
            InspectionReport::Waf {
                account,
                success,
                status,
                challenge,
            } => {
                let label = format!(
                    "{} · {} · {} · {}",
                    account,
                    if *success {
                        text(cx, "WAF access validated", "WAF 访问校验成功")
                    } else {
                        text(cx, "WAF access was not validated", "未能确认 WAF 访问成功")
                    },
                    status
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| text(cx, "No HTTP response", "无 HTTP 响应").into()),
                    challenge
                );
                root = root
                    .role(gpui_kit::Role::Status)
                    .aria_label(SharedString::from(label.clone()))
                    .child(label);
            }
        }
        root.into_any_element()
    }
}
