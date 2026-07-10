use ccr_cli::managers::{TuiConfigManager, TuiLanguage};
use std::cell::Cell;

thread_local! {
    static ACTIVE_LANGUAGE: Cell<TuiLanguage> = const { Cell::new(TuiLanguage::English) };
}

macro_rules! define_messages {
    ($($variant:ident => ($english:literal, $chinese:literal)),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Message {
            $($variant),+
        }

        impl Message {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
        }

        pub fn text_for(language: TuiLanguage, message: Message) -> &'static str {
            match message {
                $(Message::$variant => match language {
                    TuiLanguage::English => $english,
                    TuiLanguage::SimplifiedChinese => $chinese,
                }),+
            }
        }
    };
}

define_messages! {
    Keys => ("Keys", "按键"),
    Language => ("language", "语言"),
    LanguageEnglish => ("English", "英文"),
    LanguageSimplifiedChinese => ("Simplified Chinese", "简体中文"),
    ProfilesReloaded => ("Profile list reloaded", "已刷新配置列表"),
    Where => ("Where", "位置"),
    What => ("What", "问题"),
    Fallback => ("Fallback", "备用位置"),
}

pub fn active_language() -> TuiLanguage {
    ACTIVE_LANGUAGE.get()
}

pub fn set_language(language: TuiLanguage) {
    ACTIVE_LANGUAGE.set(language);
}

pub fn initialize_from_config() {
    let language = TuiConfigManager::with_default()
        .map(|manager| manager.load_or_default().language)
        .unwrap_or_default();
    set_language(language);
}

pub fn toggle_language() -> TuiLanguage {
    let language = active_language().toggled();
    set_language(language);
    language
}

pub fn text(message: Message) -> &'static str {
    text_for(active_language(), message)
}

pub fn bilingual(english: &'static str, chinese: &'static str) -> &'static str {
    match active_language() {
        TuiLanguage::English => english,
        TuiLanguage::SimplifiedChinese => chinese,
    }
}

#[macro_export]
macro_rules! tui_text {
    ($english:literal, $chinese:literal) => {
        $crate::tui::i18n::bilingual($english, $chinese)
    };
}

#[macro_export]
macro_rules! tui_format {
    ($english:literal, $chinese:literal $(, $argument:expr)* $(,)?) => {
        match $crate::tui::i18n::active_language() {
            ccr_cli::managers::TuiLanguage::English => format!($english $(, $argument)*),
            ccr_cli::managers::TuiLanguage::SimplifiedChinese => {
                format!($chinese $(, $argument)*)
            }
        }
    };
}

pub fn language_name(language: TuiLanguage) -> &'static str {
    let message = match language {
        TuiLanguage::English => Message::LanguageEnglish,
        TuiLanguage::SimplifiedChinese => Message::LanguageSimplifiedChinese,
    };
    text_for(language, message)
}

pub fn language_changed(language: TuiLanguage) -> String {
    match language {
        TuiLanguage::English => format!("Language changed to {}", language_name(language)),
        TuiLanguage::SimplifiedChinese => {
            format!("语言已切换为{}", language_name(language))
        }
    }
}

pub fn language_save_failed(error: &dyn std::fmt::Display) -> String {
    match active_language() {
        TuiLanguage::English => {
            format!("Language changed for this session, but could not save it: {error}")
        }
        TuiLanguage::SimplifiedChinese => {
            format!("语言已在本次会话中切换，但无法保存设置：{error}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Message, language_changed, set_language, text_for};
    use ccr_cli::managers::TuiLanguage;

    #[test]
    fn every_message_has_non_empty_english_and_chinese_text() {
        for message in Message::ALL {
            assert!(!text_for(TuiLanguage::English, *message).is_empty());
            assert!(!text_for(TuiLanguage::SimplifiedChinese, *message).is_empty());
        }
    }

    #[test]
    fn language_change_message_uses_the_selected_language() {
        assert_eq!(
            language_changed(TuiLanguage::English),
            "Language changed to English"
        );
        assert_eq!(
            language_changed(TuiLanguage::SimplifiedChinese),
            "语言已切换为简体中文"
        );
    }

    #[test]
    fn bilingual_macros_follow_active_language() {
        set_language(TuiLanguage::English);
        assert_eq!(crate::tui_text!("Ready", "就绪"), "Ready");
        assert_eq!(crate::tui_format!("Count: {}", "数量：{}", 3), "Count: 3");

        set_language(TuiLanguage::SimplifiedChinese);
        assert_eq!(crate::tui_text!("Ready", "就绪"), "就绪");
        assert_eq!(crate::tui_format!("Count: {}", "数量：{}", 3), "数量：3");

        set_language(TuiLanguage::English);
    }
}
