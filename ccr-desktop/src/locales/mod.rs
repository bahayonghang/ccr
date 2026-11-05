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
    if is_chinese(cx) { zh } else { en }
}

/// Chinese only when that language was installed. A missing global stays English.
pub fn is_chinese(cx: &App) -> bool {
    cx.try_global::<Language>().copied() == Some(Language::Chinese)
}

/// Show one language from an existing `English / 中文` service line.
pub fn localized_line(cx: &App, value: &str) -> String {
    let Some((en, zh)) = value.split_once(" / ") else {
        return value.to_string();
    };
    if is_chinese(cx) {
        zh.to_string()
    } else {
        en.to_string()
    }
}

/// Select safe service copy without formatting secret-bearing source errors.
pub fn service_error(cx: &App, safe: &'static str) -> &'static str {
    let (en, zh) = safe.split_once(" / ").unwrap_or((safe, safe));
    text(cx, en, zh)
}
