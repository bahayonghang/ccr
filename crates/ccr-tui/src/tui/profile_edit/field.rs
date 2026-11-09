//! Single-line field editing. Cursor movement uses terminal display width.

use ccr_cli::models::{Platform, ProfileConfig};
#[cfg(test)]
use unicode_width::UnicodeWidthStr;

use super::catalog::ModelOption;
use super::schema::{FieldGroup, FieldKind, FieldSpec, specs};

#[derive(Clone, Debug)]
pub(crate) struct TextBuffer {
    value: String,
    cursor: usize,
}

impl TextBuffer {
    pub(crate) fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.len();
        Self { value, cursor }
    }

    pub(crate) fn value(&self) -> &str {
        &self.value
    }

    #[cfg(test)]
    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }

    #[cfg(test)]
    pub(crate) fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.value.len();
    }

    /// Display columns occupied by the text before the cursor.
    #[cfg(test)]
    pub(crate) fn display_column(&self) -> usize {
        UnicodeWidthStr::width(&self.value[..self.cursor])
    }

    pub(crate) fn move_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let previous = self.value[..self.cursor]
            .chars()
            .next_back()
            .map(char::len_utf8)
            .unwrap_or(0);
        self.cursor = self.cursor.saturating_sub(previous);
    }

    pub(crate) fn move_right(&mut self) {
        if self.cursor >= self.value.len() {
            return;
        }
        let next = self.value[self.cursor..]
            .chars()
            .next()
            .map(char::len_utf8)
            .unwrap_or(0);
        self.cursor = (self.cursor + next).min(self.value.len());
    }

    pub(crate) fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub(crate) fn move_end(&mut self) {
        self.cursor = self.value.len();
    }

    pub(crate) fn insert_char(&mut self, ch: char) {
        self.insert_text(ch.encode_utf8(&mut [0; 4]));
    }

    /// Inserts text at the cursor. CR and LF are removed so a paste stays on one line.
    pub(crate) fn insert_text(&mut self, text: &str) {
        let cleaned: String = text
            .chars()
            .filter(|ch| *ch != '\n' && *ch != '\r')
            .collect();
        if cleaned.is_empty() {
            return;
        }
        self.value.insert_str(self.cursor, &cleaned);
        self.cursor += cleaned.len();
    }

    pub(crate) fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        self.move_left();
        let end = self.value[self.cursor..]
            .chars()
            .next()
            .map(|ch| self.cursor + ch.len_utf8())
            .unwrap_or(self.cursor);
        self.value.replace_range(self.cursor..end, "");
    }

    pub(crate) fn delete(&mut self) {
        if self.cursor >= self.value.len() {
            return;
        }
        let end = self.value[self.cursor..]
            .chars()
            .next()
            .map(|ch| self.cursor + ch.len_utf8())
            .unwrap_or(self.cursor);
        self.value.replace_range(self.cursor..end, "");
    }

    pub(crate) fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    /// One bullet per character. The plaintext is not part of the result.
    pub(crate) fn secret_dots(&self) -> String {
        "•".repeat(self.value.chars().count())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ChoiceItem {
    pub value: String,
    pub custom: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct EditorField {
    pub spec: FieldSpec,
    pub text: TextBuffer,
    pub original_text: String,
    pub toggle: bool,
    pub original_toggle: bool,
    pub choices: Vec<ChoiceItem>,
    pub choice_index: usize,
    pub original_choice: usize,
    pub secret_clear: bool,
    pub error: Option<String>,
    pub shown: bool,
}

impl EditorField {
    pub(crate) fn is_dirty(&self) -> bool {
        match self.spec.kind {
            FieldKind::Secret => self.secret_clear || !self.text.value().is_empty(),
            FieldKind::Toggle => self.toggle != self.original_toggle,
            FieldKind::Choice(_) | FieldKind::Credential => {
                self.choice_index != self.original_choice
            }
            FieldKind::Model(_) => self.model_id() != self.original_text.trim(),
            FieldKind::Advanced => false,
            FieldKind::Text | FieldKind::Number | FieldKind::Tags => {
                self.text.value().trim() != self.original_text.trim()
            }
        }
    }

    /// Catalog id, or the typed custom id. Empty means the field is unset.
    pub(crate) fn model_id(&self) -> &str {
        if self
            .choices
            .get(self.choice_index)
            .is_some_and(|item| item.custom)
        {
            self.text.value().trim()
        } else {
            self.choice_value().trim()
        }
    }

    pub(crate) fn choice_value(&self) -> &str {
        self.choices
            .get(self.choice_index)
            .map(|item| item.value.as_str())
            .unwrap_or("")
    }

    pub(crate) fn cycle(&mut self, delta: isize) {
        if self.choices.is_empty() {
            return;
        }
        let len = self.choices.len() as isize;
        let next = (self.choice_index as isize + delta).rem_euclid(len) as usize;
        self.choice_index = next;
    }
}

pub(crate) fn load_fields(
    platform: Platform,
    name: &str,
    config: &ProfileConfig,
) -> Vec<EditorField> {
    specs(platform)
        .into_iter()
        .map(|spec| field_from_config(spec, platform, name, config))
        .collect()
}

fn field_from_config(
    spec: FieldSpec,
    platform: Platform,
    name: &str,
    config: &ProfileConfig,
) -> EditorField {
    let shown = spec.group == FieldGroup::General && spec.key != "api_key";
    let mut field = EditorField {
        spec,
        text: TextBuffer::new(String::new()),
        original_text: String::new(),
        toggle: false,
        original_toggle: false,
        choices: Vec::new(),
        choice_index: 0,
        original_choice: 0,
        secret_clear: false,
        error: None,
        shown: shown && !(platform == Platform::Grok && spec.key == "env_key"),
    };
    match spec.kind {
        FieldKind::Text | FieldKind::Number | FieldKind::Tags => {
            let value = initial_text(spec.key, name, config);
            field.original_text = value.clone();
            field.text = TextBuffer::new(value);
        }
        FieldKind::Secret => {}
        FieldKind::Toggle => {
            let enabled = initial_toggle(spec.key, config);
            field.toggle = enabled;
            field.original_toggle = enabled;
        }
        FieldKind::Choice(options) => {
            let current = initial_choice(spec.key, config);
            let (choices, index) = choices_with_current(options, current.as_deref(), true);
            field.choices = choices;
            field.choice_index = index;
            field.original_choice = index;
        }
        FieldKind::Model(options) => {
            let current = initial_model(spec.key, config);
            let (choices, index, custom, original) = model_choices(options, current.as_deref());
            field.original_text = original;
            field.text = TextBuffer::new(custom);
            field.choices = choices;
            field.choice_index = index;
            field.original_choice = index;
        }
        FieldKind::Credential => {
            let (choices, index) =
                choices_with_current(&["keep", "replace", "env", "clear"], None, false);
            field.choices = choices;
            field.choice_index = index;
            field.original_choice = index;
        }
        FieldKind::Advanced => {
            field.shown = true;
        }
    }
    field
}

fn choices_with_current(
    options: &[&str],
    current: Option<&str>,
    blank: bool,
) -> (Vec<ChoiceItem>, usize) {
    let mut items = Vec::new();
    if blank {
        items.push(ChoiceItem {
            value: String::new(),
            custom: false,
        });
    }
    for option in options {
        items.push(ChoiceItem {
            value: (*option).to_string(),
            custom: false,
        });
    }
    let Some(current) = current.map(str::trim).filter(|value| !value.is_empty()) else {
        return (items, 0);
    };
    if let Some(index) = items
        .iter()
        .position(|item| !item.value.is_empty() && item.value.eq_ignore_ascii_case(current))
    {
        return (items, index);
    }
    items.push(ChoiceItem {
        value: current.to_string(),
        custom: true,
    });
    let index = items.len() - 1;
    (items, index)
}

/// Blank, then catalog ids, then one custom slot. A known id selects that slot.
/// Anything else stays on the custom slot without being marked dirty.
fn model_choices(
    options: &[ModelOption],
    current: Option<&str>,
) -> (Vec<ChoiceItem>, usize, String, String) {
    let mut items = vec![ChoiceItem {
        value: String::new(),
        custom: false,
    }];
    for option in options {
        items.push(ChoiceItem {
            value: option.id.to_string(),
            custom: false,
        });
    }
    items.push(ChoiceItem {
        value: String::new(),
        custom: true,
    });
    let specify = items.len() - 1;
    let Some(current) = current.map(str::trim).filter(|value| !value.is_empty()) else {
        return (items, 0, String::new(), String::new());
    };
    if let Some(index) = items.iter().position(|item| {
        !item.custom && !item.value.is_empty() && item.value.eq_ignore_ascii_case(current)
    }) {
        let canonical = items[index].value.clone();
        return (items, index, String::new(), canonical);
    }
    (items, specify, current.to_string(), current.to_string())
}

fn initial_text(key: &str, name: &str, config: &ProfileConfig) -> String {
    match key {
        "name" => name.to_string(),
        "description" => config.description.clone().unwrap_or_default(),
        "base_url" => config.base_url.clone().unwrap_or_default(),
        "provider" => config.provider.clone().unwrap_or_default(),
        "tags" => config
            .tags
            .as_ref()
            .map(|tags| tags.join(", "))
            .unwrap_or_default(),
        "api_timeout_ms" => config.api_timeout_ms.clone().unwrap_or_default(),
        "claude_code_auto_compact_window" => config
            .claude_code_auto_compact_window
            .clone()
            .unwrap_or_default(),
        "env_key" => platform_string(config, "env_key").unwrap_or_default(),
        "context_window" => platform_string(config, "context_window").unwrap_or_default(),
        _ => String::new(),
    }
}

fn initial_model(key: &str, config: &ProfileConfig) -> Option<String> {
    let value = match key {
        "model" => config.model.clone(),
        "small_fast_model" => config.small_fast_model.clone(),
        "default_opus_model" => config.default_opus_model.clone(),
        "default_sonnet_model" => config.default_sonnet_model.clone(),
        "default_haiku_model" => config.default_haiku_model.clone(),
        "default_fable_model" => config.default_fable_model.clone(),
        "subagent_model" => config.subagent_model.clone(),
        _ => None,
    };
    value
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn initial_toggle(key: &str, config: &ProfileConfig) -> bool {
    match key {
        "enabled" => config.is_enabled(),
        "requires_openai_auth" | "supports_backend_search" => platform_bool(config, key),
        _ => false,
    }
}

fn initial_choice(key: &str, config: &ProfileConfig) -> Option<String> {
    match key {
        "effort_level" => config.effort_level.clone(),
        _ => platform_string(config, key),
    }
}

fn platform_string(config: &ProfileConfig, key: &str) -> Option<String> {
    config.platform_data.get(key).and_then(|value| match value {
        serde_json::Value::String(text) if !text.trim().is_empty() => Some(text.trim().to_string()),
        serde_json::Value::Number(number) => Some(number.to_string()),
        serde_json::Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    })
}

fn platform_bool(config: &ProfileConfig, key: &str) -> bool {
    match config.platform_data.get(key) {
        Some(serde_json::Value::Bool(flag)) => *flag,
        Some(serde_json::Value::String(text)) => text == "true" || text == "1",
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cjk_cursor_moves_by_display_width() {
        let mut buffer = TextBuffer::new("中a");
        assert_eq!(buffer.display_column(), 3);
        buffer.move_left();
        assert_eq!(buffer.display_column(), 2);
        assert_eq!(buffer.cursor(), "中".len());
        buffer.move_left();
        assert_eq!(buffer.display_column(), 0);
        buffer.move_right();
        assert_eq!(buffer.display_column(), 2);
    }

    #[test]
    fn known_model_stays_selected_and_unknown_model_stays_custom() {
        let mut known = ProfileConfig::new();
        known.model = Some("gpt-5.6-sol".into());
        let field = load_fields(Platform::Codex, "work", &known)
            .into_iter()
            .find(|field| field.spec.key == "model")
            .unwrap();
        assert_eq!(field.model_id(), "gpt-5.6-sol");
        assert!(!field.is_dirty());
        assert!(field.choices.iter().any(|item| item.value == "gpt-6.1-sol"));
        assert!(field.choices.iter().any(|item| item.value == "gpt-6-astra"));
        assert!(field.choices.last().is_some_and(|item| item.custom));

        let mut custom = ProfileConfig::new();
        custom.model = Some("provider/custom".into());
        let field = load_fields(Platform::Codex, "work", &custom)
            .into_iter()
            .find(|field| field.spec.key == "model")
            .unwrap();
        assert!(field.choices[field.choice_index].custom);
        assert_eq!(field.model_id(), "provider/custom");
        assert!(!field.is_dirty());

        let claude = load_fields(Platform::Claude, "work", &ProfileConfig::new())
            .into_iter()
            .find(|field| field.spec.key == "model")
            .unwrap();
        assert!(
            claude
                .choices
                .iter()
                .any(|item| item.value == "claude-opus-5-5")
        );
        assert_eq!(claude.model_id(), "");
        assert!(!claude.is_dirty());
    }

    #[test]
    fn paste_strips_newlines_and_inserts_at_the_cursor() {
        let mut buffer = TextBuffer::new("ac");
        buffer.move_home();
        buffer.move_right();
        buffer.insert_text("X\r\nY\n");
        assert_eq!(buffer.value(), "aXYc");
        assert!(!buffer.value().contains('\n'));
        assert!(!buffer.value().contains('\r'));
    }

    #[test]
    fn secret_dots_hide_every_character() {
        let mut buffer = TextBuffer::new(String::new());
        buffer.insert_text("sk-secret");
        assert_eq!(buffer.secret_dots(), "•••••••••");
        assert!(!buffer.secret_dots().contains("sk"));
        assert!(!buffer.secret_dots().contains('s'));
    }
}
