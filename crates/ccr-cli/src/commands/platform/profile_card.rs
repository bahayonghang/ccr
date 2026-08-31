//! 平台共享的 Profile 状态卡渲染。
//!
//! `claude profile current` 与 `codex profile current`（以及未来的 gemini / droid /
//! opencode 同款命令）的状态卡布局完全一致——只是 header 用各自 platform 的 display
//! name。把这层渲染抽出来后，平台命令模块只保留各自的标题、文案和 `Platform` enum。

use crate::commands::common::new_utf8_table;
use crate::services::{PlatformStatusCard, StatusAuthKind, StatusHealth};
use comfy_table::{Attribute, Cell, Color as TableColor, ContentArrangement};

/// 渲染单个平台的 profile 状态卡到 stdout。
///
/// 调用方：`claude profile current` / `codex profile current` 等子命令的非 JSON 路径。
/// 文案与颜色规则统一在此处维护，平台 dispatcher 只负责挑选 `card`。
pub fn print_status_card(card: &PlatformStatusCard) {
    let mut table = new_utf8_table();
    table
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(vec![
            Cell::new(card.display_name.as_str())
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new(render_health(card.health))
                .add_attribute(Attribute::Bold)
                .fg(health_color(card.health)),
        ]);

    table.add_row(vec![
        Cell::new("Profile").fg(TableColor::Yellow),
        Cell::new(card.profile.as_str())
            .fg(TableColor::Green)
            .add_attribute(Attribute::Bold),
    ]);
    if let Some(provider) = card.provider.as_deref().filter(|value| !value.is_empty()) {
        table.add_row(vec![Cell::new("Provider"), Cell::new(provider)]);
    }
    if let Some(model) = card.model.as_deref().filter(|value| !value.is_empty()) {
        table.add_row(vec![Cell::new("主模型"), Cell::new(model)]);
    }
    table.add_row(vec![
        Cell::new("认证").fg(TableColor::Yellow),
        Cell::new(card.auth.as_str()).fg(auth_color(card.auth_kind)),
    ]);
    table.add_row(vec![Cell::new("说明"), Cell::new(card.note.as_str())]);

    println!("{table}");
}

fn render_health(health: StatusHealth) -> &'static str {
    match health {
        StatusHealth::Ready => "✓ 就绪",
        StatusHealth::NeedsLogin => "⚠ 需登录",
        StatusHealth::Invalid => "✗ 无效",
        StatusHealth::Unsupported => "○ 不支持",
        StatusHealth::Error => "✗ 错误",
    }
}

fn health_color(health: StatusHealth) -> TableColor {
    match health {
        StatusHealth::Ready => TableColor::Green,
        StatusHealth::NeedsLogin => TableColor::Yellow,
        StatusHealth::Invalid | StatusHealth::Error => TableColor::Red,
        StatusHealth::Unsupported => TableColor::DarkGrey,
    }
}

fn auth_color(kind: StatusAuthKind) -> TableColor {
    match kind {
        StatusAuthKind::OfficialAuth => TableColor::Green,
        StatusAuthKind::ThirdPartyApi | StatusAuthKind::ProviderKey => TableColor::Cyan,
        StatusAuthKind::NoAuth => TableColor::DarkGrey,
        StatusAuthKind::Missing | StatusAuthKind::Unknown => TableColor::Yellow,
    }
}
