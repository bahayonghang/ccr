//! Configured providers and the bundled catalog share the read-only local projection.
use super::*;
use crate::runtime::checkin::Operation;
use ccr_checkin::managers::checkin::builtin_providers::{
    get_builtin_provider_by_id, get_builtin_providers,
};
use operations::EditorKind;

impl CheckinView {
    pub(super) fn builtin_catalog(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let mut catalog = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
        for (index, item) in get_builtin_providers().into_iter().enumerate() {
            let id = item.id;
            let configured = self.data().is_some_and(|data| {
                data.providers
                    .iter()
                    .any(|p| p.builtin_id.as_ref() == Some(&id))
            });
            let label = format!(
                "{} {}",
                if configured {
                    text(cx, "Configured", "已配置")
                } else {
                    text(cx, "Add", "添加")
                },
                item.name
            );
            catalog = catalog.child(semantics::button(
                Button::new(("checkin-builtin", index))
                    .label(label)
                    .ghost()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.start(Operation::AddBuiltin(id.clone()), cx)
                    })),
                self.operation.busy || configured,
                false,
                cx,
            ));
        }
        catalog
    }
    pub(super) fn render_provider_rows(
        &self,
        snapshot: &CheckinData,
        cx: &mut Context<Self>,
    ) -> Vec<gpui_kit::AnyElement> {
        let mut children = Vec::new();
        let locked = self.operation.busy;
        for (index, p) in snapshot.providers.iter().enumerate() {
            let id = p.id.clone();
            let delete = id.clone();
            let info = id.clone();
            let locate = id.clone();
            children.push(
                (div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(theme::CONTROL_GAP))
                    .child(p.name.clone())
                    .child(semantics::ghost_button(
                        Button::new(format!("checkin-provider-info-{index}"))
                            .label(text(cx, "Verification status", "验证状态"))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.provider_detail = (this.provider_detail.as_ref()
                                    != Some(&info))
                                .then(|| info.clone());
                                cx.notify();
                            })),
                        false,
                        false,
                        cx,
                    ))
                    .child(semantics::ghost_button(
                        Button::new(format!("checkin-provider-accounts-{index}"))
                            .label(text(cx, "View associated accounts", "查看关联账户"))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.switch_tab(CheckinTab::Accounts, window, cx);
                                this.provider_filter = Some(locate.clone());
                                this.query.clear();
                                this.search
                                    .update(cx, |input, cx| input.set_value("", window, cx));
                                this.scroll.set_offset(Default::default());
                                this.focus_control("checkin-new-account");
                                cx.notify();
                            })),
                        false,
                        false,
                        cx,
                    ))
                    .child(
                        self.origin_control(
                            &format!("provider-{}", p.id),
                            semantics::button(
                                Button::new(("checkin-edit-provider", index))
                                    .label(text(cx, "Edit provider", "编辑提供商"))
                                    .ghost()
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.remember_editor_source(&format!("provider-{id}"));
                                        this.edit(
                                            EditorKind::Provider(Some(id.clone())),
                                            window,
                                            cx,
                                        )
                                    })),
                                locked,
                                false,
                                cx,
                            ),
                        ),
                    )
                    .child(self.origin_control(&format!("provider-delete-{}",p.id),semantics::button(
                        Button::new(("checkin-delete-provider", index))
                            .label(text(cx, "Delete provider", "删除提供商"))
                            .ghost()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                let Some(provider) = this.data().and_then(|data| data.providers.iter().find(|p| p.id == delete)) else { return; };
                                let notice = format!("{} · {}", provider.name, text(cx,"Delete 1 saved provider? Associated accounts must be removed first.","删除 1 个已保存的提供商？需先移除关联账户。"));
                                this.confirm_entry(Operation::DeleteProvider(delete.clone()), notice, &format!("provider-delete-{delete}"),cx);
                            })),
                        locked,
                        false,
                        cx,
                    ))))
                .into_any_element(),
            );
            if self.provider_detail.as_ref() == Some(&p.id) {
                let policy = p.builtin_id.as_deref().and_then(get_builtin_provider_by_id);
                let label = if snapshot.valid_waf_providers.contains(&p.id) {
                    text(cx, "Verification cache available", "验证缓存有效")
                } else if policy.is_some_and(|p| p.requires_waf_bypass || p.requires_cf_clearance) {
                    text(
                        cx,
                        "Missing or expired verification cache",
                        "验证缓存缺失或已过期",
                    )
                } else {
                    text(
                        cx,
                        "No catalog verification requirement",
                        "目录未指定验证要求",
                    )
                };
                children.push(
                    div()
                        .id("checkin-provider-status")
                        .test_support()
                        .role(gpui_kit::Role::Status)
                        .aria_label(label)
                        .child(label)
                        .into_any_element(),
                );
            }
        }

        children
    }
}
