//! Scope statements for platform setting surfaces: object, source and actual effect.
//!
//! 运行设置、供应商目录与 CCR 元数据分别成组，不合并语义；文案只陈述来源与作用域，
//! 不声称客户端已加载或网络可用。
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Category {
    /// A settings file that belongs to the platform client itself.
    Runtime,
    /// Provider entries; editing them never switches the active provider.
    ProviderCatalog,
    /// Lists kept by CCR, separate from the platform's runtime files.
    CcrMetadata,
}

/// The primary category of a surface. Accounts are not a settings category.
pub(super) fn category(platform: Platform, surface: Surface) -> Option<Category> {
    match (platform, surface) {
        (Platform::Codex, Surface::Models) => Some(Category::CcrMetadata),
        (Platform::Codex | Platform::OpenCode, Surface::Providers) => {
            Some(Category::ProviderCatalog)
        }
        (_, Surface::Settings | Surface::Tui | Surface::LegacySettings) => Some(Category::Runtime),
        _ => None,
    }
}

fn platform_name(platform: Platform) -> &'static str {
    match platform {
        Platform::Claude => "Claude Code",
        Platform::Codex => "Codex",
        Platform::Grok => "Grok",
        Platform::Gemini => "Antigravity",
        Platform::OpenCode => "OpenCode",
        Platform::Qwen | Platform::Droid => "",
    }
}

fn statement(category: Category, session: &PlatformSession, cx: &gpui_kit::App) -> String {
    let name = platform_name(session.platform);
    let chinese = is_chinese(cx);
    match (category, session.platform) {
        // Gemini 兼容路径与 Antigravity 客户端识别分开陈述，且此处只读。
        (Category::Runtime, Platform::Gemini) => text(
            cx,
            "Gemini-compatible legacy path. Whether the Antigravity client reads it is unverified; it stays read-only here.",
            "Gemini 兼容的旧版路径。Antigravity 客户端是否读取尚未验证；此处只读。",
        )
        .to_string(),
        (Category::Runtime, _) if chinese => {
            format!("{name} 的用户级运行设置。此处的修改不会改动已保存的配置方案或账户。")
        }
        (Category::Runtime, _) => format!(
            "User-level runtime settings of {name}. Edits here do not change saved profiles or accounts."
        ),
        (Category::ProviderCatalog, Platform::OpenCode) => text(
            cx,
            "Provider entries inside OpenCode's runtime settings. Editing an entry does not change the selected model.",
            "OpenCode 运行设置中的供应商条目。编辑条目不会更改已选模型。",
        )
        .to_string(),
        (Category::ProviderCatalog | Category::CcrMetadata, _) => text(
            cx,
            "Kept by CCR, separate from the platform's runtime settings.",
            "由 CCR 保存，独立于平台自身的运行设置。",
        )
        .to_string(),
    }
}

/// A labelled group heading. The heading scope is the source file label only;
/// `extra` is appended inside the same group (for example, a provider status line).
pub(super) fn group(
    category: Category,
    session: &PlatformSession,
    extra: Option<gpui_kit::AnyElement>,
    cx: &gpui_kit::App,
) -> gpui_kit::AnyElement {
    let (id, title) = match category {
        Category::Runtime => (
            "platform-group-runtime",
            text(cx, "Runtime settings", "运行设置"),
        ),
        Category::ProviderCatalog => (
            "platform-group-provider-catalog",
            text(cx, "Provider catalog", "供应商目录"),
        ),
        Category::CcrMetadata => (
            "platform-group-ccr-metadata",
            text(cx, "CCR metadata", "CCR 元数据"),
        ),
    };
    let statement = statement(category, session, cx);
    div()
        .id(id)
        .role(gpui_kit::Role::Group)
        .aria_label(title)
        .flex()
        .flex_col()
        .gap(px(theme::CONTROL_GAP))
        .child(badge::section_heading(
            title,
            Some(SharedString::from(session.file_label)),
            cx,
        ))
        .child(
            div()
                .id(SharedString::from(format!("{id}-scope")))
                .test_support()
                .aria_label(statement.clone())
                .text_size(px(theme::META_SIZE))
                .text_color(cx.theme().muted_foreground)
                .child(statement),
        )
        .children(extra)
        .test_support()
        .into_any_element()
}
