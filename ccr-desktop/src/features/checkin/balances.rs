use super::*;
use crate::shell::badge;
use gpui_kit::component::ActiveTheme;

fn amount(value: f64, cx: &App) -> String {
    if !value.is_finite() {
        return text(cx, "Unavailable", "不可用").to_string();
    }
    // Keep small nonzero balances visible while suppressing floating-point sum noise.
    let rounded = format!("{value:.6}");
    if value != 0. && rounded.parse::<f64>() == Ok(0.) {
        return value.to_string();
    }
    rounded
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

pub(super) fn summary(data: &CheckinData, cx: &App) -> gpui_kit::AnyElement {
    let balances = &data.balances;
    let count = data.accounts.len();
    let mut labels = Vec::new();
    let mut panel = div()
        .id("checkin-balance-summary")
        .test_support()
        .flex()
        .flex_col()
        .gap(px(theme::CONTROL_GAP))
        .bg(gpui_kit::rgb(theme::palette(cx.theme().mode).surface))
        .rounded(px(theme::INPUT_RADIUS))
        .p(px(theme::SECTION_GAP));
    if !balances.groups.is_empty() {
        panel = panel.child(badge::section_heading(
            text(cx, "Saved balances", "已保存余额"),
            None,
            cx,
        ));
    }
    for (currency, group) in &balances.groups {
        let from = group.from.format("%Y-%m-%d %H:%M:%S");
        let to = group.to.format("%Y-%m-%d %H:%M:%S");
        let range = if group.from == group.to {
            format!("{from} UTC")
        } else {
            format!("{from} – {to} UTC")
        };
        let coverage = format!(
            "{}/{} {} · {range}",
            group.accounts,
            count,
            text(cx, "accounts", "个账户")
        );
        let mut metrics = div()
            .flex()
            .flex_wrap()
            .gap(px(theme::SECTION_GAP))
            .items_baseline();
        labels.push(format!("{currency} · {coverage}"));
        for (label, value) in [
            (text(cx, "Balance", "余额"), group.remaining),
            (text(cx, "Total quota", "总额度"), group.total),
            (text(cx, "Used", "已用"), group.used),
        ] {
            let value = amount(value, cx);
            labels.push(format!("{label} {value}"));
            metrics = metrics.child(
                div()
                    .min_w(px(88.))
                    .flex()
                    .flex_col()
                    .items_end()
                    .child(
                        div()
                            .text_size(theme::text_size(theme::META_SIZE))
                            .text_color(cx.theme().muted_foreground)
                            .child(label),
                    )
                    .child(
                        div()
                            .font_features(theme::tabular_figures())
                            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                            .child(value),
                    ),
            );
        }
        panel = panel.child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .justify_between()
                .gap(px(theme::CONTROL_GAP))
                .child(
                    div().flex().flex_col().child(currency.clone()).child(
                        div()
                            .text_size(theme::text_size(theme::META_SIZE))
                            .text_color(cx.theme().muted_foreground)
                            .font_features(theme::tabular_figures())
                            .child(coverage),
                    ),
                )
                .child(metrics),
        );
    }
    let mut omitted = Vec::new();
    for (n, label) in [
        (
            balances.missing,
            text(cx, "No stored balance", "无余额快照"),
        ),
        (balances.unusable, text(cx, "Unusable values", "读数不可用")),
        (
            balances.unknown_unit,
            text(cx, "Unknown unit", "单位未保存"),
        ),
    ] {
        if n > 0 {
            omitted.push(format!("{label} · {n}/{count}"));
        }
    }
    if !omitted.is_empty() {
        let note = omitted.join("   ·   ");
        labels.push(note.clone());
        panel = panel.child(
            div()
                .text_color(cx.theme().muted_foreground)
                .text_size(theme::text_size(theme::META_SIZE))
                .child(note),
        );
    }
    panel
        .aria_label(SharedString::from(labels.join(" · ")))
        .into_any_element()
}
