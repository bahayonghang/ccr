//! Small native copy catalog, no second translation runtime.
use gpui_kit::{App, Global};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    English,
    Chinese,
}
impl Global for Language {}

pub fn text(cx: &App, en: &'static str, zh: &'static str) -> &'static str {
    match cx.try_global::<Language>().copied().unwrap_or_default() {
        Language::English => en,
        Language::Chinese => zh,
    }
}

/// Select safe service copy without formatting secret-bearing source errors.
pub fn service_error(cx: &App, safe: &'static str) -> &'static str {
    let (en, zh) = safe.split_once(" / ").unwrap_or((safe, safe));
    text(cx, en, zh)
}
