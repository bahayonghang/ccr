//! Provider chips: one toggleable filter per session family, stating what the index read.
//!
//! 状态只陈述提供方扫描给出的结果：可读时写来源数，无文件、未安装、不可读照实写；
//! 不编造会话数，也不推断“运行中”。未安装与不可读的 chip 不可选。
use crate::features::insights::{InsightsSnapshot, InsightsView};
use crate::{
    locales::text,
    shell::badge,
    theme::{self, Tone},
};
use ccr_store::sessions::{AgentSessionAgentId, AgentSessionProviderStatus, ProviderAvailability};
use gpui_kit::base::TestSupportExt;
use gpui_kit::component::{ActiveTheme, Icon, IconName, button::Button};
use gpui_kit::{
    AnyElement, App, Context, InteractiveElement, IntoElement, ParentElement, Role, SharedString,
    StatefulInteractiveElement, Styled, div, px, rgb,
};

/// Display name of a session family.
pub(super) fn agent_name(agent: AgentSessionAgentId) -> &'static str {
    match agent {
        AgentSessionAgentId::Grok => "Grok",
        AgentSessionAgentId::Claude => "Claude",
        AgentSessionAgentId::Codex => "Codex",
        AgentSessionAgentId::OpenCode => "OpenCode",
        AgentSessionAgentId::Pi => "Pi",
        AgentSessionAgentId::Omp => "OMP",
        AgentSessionAgentId::Antigravity => "Antigravity",
        AgentSessionAgentId::Kimi => "Kimi",
    }
}

pub(super) fn providers(
    view: &InsightsView,
    narrow: bool,
    cx: &mut Context<InsightsView>,
) -> AnyElement {
    // 平台作用域锁定提供方：不渲染 chip，只写一行说明。
    if let Some(agent) = view.locked_agent() {
        let line = text(cx, "Showing {name} sessions", "只显示 {name} 会话")
            .replace("{name}", agent_name(agent));
        return div()
            .id("session-providers-locked")
            .test_support()
            .role(Role::Status)
            .aria_label(line.clone())
            .flex_shrink_0()
            .text_size(crate::theme::text_size(theme::META_SIZE))
            .text_color(cx.theme().muted_foreground)
            .child(line)
            .into_any_element();
    }
    let Some(InsightsSnapshot::Sessions { statuses, .. }) = view.snapshot.as_deref() else {
        return div().into_any_element();
    };
    let mut row = div()
        .id("session-providers")
        .test_support()
        .role(Role::Group)
        .aria_label(text(cx, "Session providers", "会话提供方"))
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap(px(theme::CONTROL_GAP));
    if narrow {
        row = row.overflow_x_scroll();
    } else {
        row = row.flex_wrap();
    }
    for status in statuses {
        let pressed = view.agents.contains(&status.agent);
        row = row.child(provider_chip(status, pressed, cx));
    }
    row.into_any_element()
}

/// `(selectable, short state)` for one provider status.
fn reading(status: &AgentSessionProviderStatus, cx: &App) -> (bool, String) {
    match status.availability {
        ProviderAvailability::Available => {
            let count = status.source_count;
            let short = if count == 1 {
                text(cx, "1 source", "1 个来源").to_string()
            } else {
                text(cx, "{n} sources", "{n} 个来源").replace("{n}", &count.to_string())
            };
            (true, short)
        }
        ProviderAvailability::NoData => (true, text(cx, "no files", "无文件").into()),
        ProviderAvailability::NotInstalled => (false, text(cx, "not installed", "未安装").into()),
        ProviderAvailability::Error => (false, text(cx, "unreadable", "不可读").into()),
    }
}

// 每个 chip 在独立函数里构建，避免增大 render 栈帧（Windows UI 线程只有 1 MiB）。
fn provider_chip(
    status: &AgentSessionProviderStatus,
    pressed: bool,
    cx: &mut Context<InsightsView>,
) -> AnyElement {
    let agent = status.agent;
    let name = agent_name(agent);
    let (selectable, short) = reading(status, cx);
    let mode = cx.theme().mode;
    let label = format!("{name} · {short}");
    let indicator: AnyElement = match status.availability {
        ProviderAvailability::Available => {
            badge::dot(theme::platform_color(agent.as_str(), mode)).into_any_element()
        }
        ProviderAvailability::NoData => badge::dot(cx.theme().muted_foreground).into_any_element(),
        ProviderAvailability::NotInstalled => {
            badge::hollow_dot(cx.theme().muted_foreground).into_any_element()
        }
        ProviderAvailability::Error => Icon::new(IconName::CircleAlert)
            .size(px(theme::META_SIZE))
            .text_color(theme::tone_colors(Tone::Danger, mode).ink)
            .into_any_element(),
    };
    let mut button = Button::new(SharedString::from(format!(
        "session-provider-{}",
        agent.as_str()
    )))
    .accessibility_label(label.clone())
    .child(
        div()
            .flex()
            .items_center()
            .gap(px(theme::SELECTION_RADIUS + 2.))
            .font_features(theme::tabular_figures())
            .whitespace_nowrap()
            .child(indicator)
            .child(label),
    );
    // 不可读时 tooltip 给出提供方报告的错误类别，不含路径或正文。
    if status.availability == ProviderAvailability::Error {
        let category = status
            .error_category
            .clone()
            .unwrap_or_else(|| text(cx, "Unknown error", "未知错误").into());
        button = button.tooltip(category);
    }
    if selectable {
        let agent_id = agent;
        button = button
            .on_click(cx.listener(move |this, _, _, cx| this.toggle_session_agent(agent_id, cx)));
    }
    let chip = badge::filter_chip(button, pressed && selectable, !selectable, cx);
    if status.availability == ProviderAvailability::NotInstalled {
        chip.border_dashed()
            .border_color(rgb(theme::palette(mode).subtle))
            .into_any_element()
    } else {
        chip.into_any_element()
    }
}
