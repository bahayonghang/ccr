//! Selectable model ids. The stored value is the provider id, not the short label.
//!
//! An unknown current value stays selectable as a custom id and is not rewritten
//! until the user changes it.

use ccr_cli::models::Platform;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ModelOption {
    pub id: &'static str,
    pub label: &'static str,
    /// Recommended starting point. Shown as "default"; opening the editor does not write it.
    pub recommended: bool,
}

const fn option(id: &'static str, label: &'static str, recommended: bool) -> ModelOption {
    ModelOption {
        id,
        label,
        recommended,
    }
}

pub(crate) const CODEX: &[ModelOption] = &[
    option("gpt-6.1-sol", "6.1 Sol", true),
    option("gpt-6-astra", "6 Astra", false),
    option("gpt-6.1-terra", "6.1 Terra", false),
    option("gpt-6-sol", "6 Sol", false),
    option("gpt-6-luna", "6 Luna", false),
    option("gpt-5.6-sol", "5.6 Sol", false),
    option("gpt-5.6-terra", "5.6 Terra", false),
    option("gpt-5.6-luna", "5.6 Luna", false),
];

pub(crate) const CLAUDE: &[ModelOption] = &[
    option("claude-opus-5-5", "Opus 5.5", true),
    option("claude-sonnet-5-5", "Sonnet 5.5", false),
    option("claude-haiku-5-5", "Haiku 5.5", false),
    option("claude-fable-5-1", "Fable 5.1", false),
    option("claude-opus-5", "Opus 5", false),
    option("claude-sonnet-5", "Sonnet 5", false),
    option("claude-fable-5", "Fable 5", false),
    option("claude-opus-4-8", "Opus 4.8", false),
    option("claude-opus-4-7", "Opus 4.7", false),
    option("claude-sonnet-4-6", "Sonnet 4.6", false),
    option("claude-opus-4-6", "Opus 4.6", false),
    option("claude-opus-4-5-20251101", "Opus 4.5", false),
    option("claude-sonnet-4-5-20250929", "Sonnet 4.5", false),
    option("claude-haiku-4-5-20251001", "Haiku 4.5", false),
];

pub(crate) const CLAUDE_OPUS: &[ModelOption] = &[
    option("claude-opus-5-5", "Opus 5.5", true),
    option("claude-opus-5", "Opus 5", false),
    option("claude-opus-4-8", "Opus 4.8", false),
    option("claude-opus-4-7", "Opus 4.7", false),
    option("claude-opus-4-6", "Opus 4.6", false),
    option("claude-opus-4-5-20251101", "Opus 4.5", false),
];

pub(crate) const CLAUDE_SONNET: &[ModelOption] = &[
    option("claude-sonnet-5-5", "Sonnet 5.5", true),
    option("claude-sonnet-5", "Sonnet 5", false),
    option("claude-sonnet-4-6", "Sonnet 4.6", false),
    option("claude-sonnet-4-5-20250929", "Sonnet 4.5", false),
];

pub(crate) const CLAUDE_HAIKU: &[ModelOption] = &[
    option("claude-haiku-5-5", "Haiku 5.5", true),
    option("claude-haiku-4-5-20251001", "Haiku 4.5", false),
];

pub(crate) const CLAUDE_FABLE: &[ModelOption] = &[
    option("claude-fable-5-1", "Fable 5.1", true),
    option("claude-fable-5", "Fable 5", false),
];

pub(crate) const GROK: &[ModelOption] = &[
    option("grok-4.6", "4.6", false),
    option("grok-4", "4", false),
    option("grok-3", "3", false),
];

pub(crate) fn main_for(platform: Platform) -> &'static [ModelOption] {
    match platform {
        Platform::Codex => CODEX,
        Platform::Claude => CLAUDE,
        Platform::Grok => GROK,
        Platform::Gemini | Platform::Qwen | Platform::Droid => &[],
    }
}

pub(crate) fn display(id: &str, options: &[ModelOption]) -> String {
    let Some(option) = options
        .iter()
        .find(|option| option.id.eq_ignore_ascii_case(id))
    else {
        return id.to_string();
    };
    if option.recommended {
        format!("{} · {}", option.label, crate::tui_text!("default", "默认"))
    } else {
        option.label.to_string()
    }
}
