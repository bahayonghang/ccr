use crate::features::insights::{InsightsSnapshot, InsightsView};
use crate::{locales::text, theme};
use ccr_store::sessions::ProviderAvailability;
use gpui_kit::component::ActiveTheme;
use gpui_kit::{Context, IntoElement, ParentElement, Styled, div, px};

pub(super) fn providers(view: &InsightsView, cx: &mut Context<InsightsView>) -> impl IntoElement {
    let Some(InsightsSnapshot::Sessions { statuses, .. }) = view.snapshot.as_deref() else {
        return div();
    };
    let mut row = div().flex().flex_wrap().gap(px(theme::CONTROL_GAP));
    for status in statuses {
        let (label, tone_ok) = match status.availability {
            ProviderAvailability::Available => (
                format!("{} {}", status.agent.as_str(), text(cx, "readable", "可读")),
                true,
            ),
            ProviderAvailability::NoData => (
                format!(
                    "{} {}",
                    status.agent.as_str(),
                    text(cx, "no files", "无文件")
                ),
                false,
            ),
            ProviderAvailability::NotInstalled => (
                format!(
                    "{} {}",
                    status.agent.as_str(),
                    text(cx, "not installed", "未安装")
                ),
                false,
            ),
            ProviderAvailability::Error => (
                format!(
                    "{} {}",
                    status.agent.as_str(),
                    text(cx, "unreadable", "不可读")
                ),
                false,
            ),
        };
        row = row.child(
            div()
                .px(px(theme::CONTROL_GAP))
                .h(px(28.))
                .flex()
                .items_center()
                .rounded(px(theme::PILL_RADIUS))
                .bg(cx.theme().secondary)
                .text_size(px(theme::META_SIZE))
                .text_color(if tone_ok {
                    cx.theme().primary
                } else {
                    cx.theme().muted_foreground
                })
                .child(label),
        );
    }
    row
}
