//! In-place profile editor. Writes go through `edit_profile` on the mutation worker.

mod catalog;
mod field;
mod request;
mod schema;
pub(crate) mod ui;

use ccr_cli::models::{Platform, ProfileConfig};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(crate) use request::EditRequest;

use field::{EditorField, TextBuffer, load_fields};
use request::{Draft, build, confirm_lines, validate};
use schema::FieldKind;

pub(crate) struct EditorOpen<'a> {
    pub platform: Platform,
    pub name: &'a str,
    pub config: &'a ProfileConfig,
    pub is_current: bool,
    pub revision: &'a str,
}

pub(crate) enum EditorEvent {
    None,
    Save,
    Cancel,
}

pub(crate) struct ProfileEditor {
    platform: Platform,
    original_name: String,
    revision: String,
    is_current: bool,
    fields: Vec<EditorField>,
    focus: usize,
    advanced_open: bool,
    discard_armed: bool,
    banner: Option<String>,
    form_scroll: usize,
}

impl ProfileEditor {
    pub(crate) fn open(source: EditorOpen<'_>) -> Self {
        let mut editor = Self {
            platform: source.platform,
            original_name: source.name.to_string(),
            revision: source.revision.to_string(),
            is_current: source.is_current,
            fields: load_fields(source.platform, source.name, source.config),
            focus: 0,
            advanced_open: false,
            discard_armed: false,
            banner: None,
            form_scroll: 0,
        };
        editor.sync_shown();
        editor
    }

    pub(crate) fn platform(&self) -> Platform {
        self.platform
    }

    pub(crate) fn original_name(&self) -> &str {
        &self.original_name
    }

    pub(crate) fn is_current(&self) -> bool {
        self.is_current
    }

    #[cfg(test)]
    pub(crate) fn revision(&self) -> &str {
        &self.revision
    }

    pub(crate) fn banner(&self) -> Option<&str> {
        self.banner.as_deref()
    }

    pub(crate) fn set_banner(&mut self, message: impl Into<String>) {
        self.banner = Some(message.into());
    }

    pub(crate) fn fields(&self) -> &[EditorField] {
        &self.fields
    }

    pub(crate) fn focus_index(&self) -> usize {
        self.focus
    }

    #[cfg(test)]
    pub(crate) fn focused_key(&self) -> &str {
        self.fields
            .get(self.focus)
            .map(|field| field.spec.key)
            .unwrap_or("")
    }

    pub(crate) fn form_scroll(&self) -> usize {
        self.form_scroll
    }

    pub(crate) fn set_form_scroll(&mut self, scroll: usize) {
        self.form_scroll = scroll;
    }

    pub(crate) fn align_scroll(&mut self, focus_start: usize, focus_len: usize, viewport: usize) {
        if viewport == 0 {
            return;
        }
        let focus_end = focus_start.saturating_add(focus_len.max(1));
        if focus_start < self.form_scroll {
            self.form_scroll = focus_start;
        } else if focus_end > self.form_scroll.saturating_add(viewport) {
            self.form_scroll = focus_end.saturating_sub(viewport);
        }
    }

    pub(crate) fn advanced_open(&self) -> bool {
        self.advanced_open
    }

    pub(crate) fn draft(&self) -> Draft<'_> {
        Draft {
            platform: self.platform,
            original_name: &self.original_name,
            revision: &self.revision,
            fields: &self.fields,
        }
    }

    pub(crate) fn confirm_lines(&self) -> Vec<String> {
        confirm_lines(&self.draft())
    }

    pub(crate) fn build_request(&self) -> Option<EditRequest> {
        build(&self.draft())
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> EditorEvent {
        if self.is_cancel(key) {
            return self.cancel();
        }
        self.discard_armed = false;
        if self.is_save(key) {
            self.banner = None;
            return EditorEvent::Save;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('d' | 'D'))
        {
            self.clear_secret();
            return EditorEvent::None;
        }
        match key.code {
            KeyCode::Up => self.move_focus(-1),
            KeyCode::Down => self.move_focus(1),
            KeyCode::BackTab => self.move_focus(-1),
            KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => self.move_focus(-1),
            KeyCode::Tab => self.move_focus(1),
            KeyCode::Left => self.edit_horizontal(-1),
            KeyCode::Right => self.edit_horizontal(1),
            KeyCode::Home => self.edit_home(),
            KeyCode::End => self.edit_end(),
            KeyCode::Backspace => self.edit_backspace(),
            KeyCode::Delete => self.edit_delete(),
            KeyCode::Enter => self.activate_focused(),
            KeyCode::Char(ch)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.insert_char(ch);
            }
            _ => {}
        }
        EditorEvent::None
    }

    pub(crate) fn insert_paste(&mut self, text: &str) {
        self.discard_armed = false;
        if matches!(self.focused_kind(), Some(FieldKind::Model(_))) {
            if self.prepare_model_text(false)
                && let Some(field) = self.focused_mut()
            {
                field.text.insert_text(text);
                field.error = None;
            }
            return;
        }
        let Some(field) = self.focused_mut() else {
            return;
        };
        if !is_text_kind(field.spec.kind) {
            return;
        }
        field.secret_clear = false;
        field.text.insert_text(text);
        field.error = None;
    }

    pub(crate) fn validate_with(&mut self, name_taken: impl Fn(&str) -> bool) -> bool {
        let original = self.original_name.clone();
        let name = self
            .fields
            .iter()
            .find(|field| field.spec.key == "name")
            .map(|field| field.text.value().trim().to_string())
            .unwrap_or_default();
        let taken = name != original && name_taken(&name);
        validate(&mut self.fields, &original, taken)
    }

    fn cancel(&mut self) -> EditorEvent {
        let dirty = self.fields.iter().any(EditorField::is_dirty);
        if dirty && !self.discard_armed {
            self.discard_armed = true;
            self.banner = Some(
                crate::tui_text!("Press Esc again to discard changes", "再按 Esc 放弃改动")
                    .to_string(),
            );
            return EditorEvent::None;
        }
        EditorEvent::Cancel
    }

    fn is_cancel(&self, key: KeyEvent) -> bool {
        key.code == KeyCode::Esc
            || (key.modifiers.contains(KeyModifiers::CONTROL)
                && matches!(key.code, KeyCode::Char('c' | 'C')))
    }

    fn is_save(&self, key: KeyEvent) -> bool {
        key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('s' | 'S'))
    }

    fn move_focus(&mut self, delta: isize) {
        let visible = self.visible_positions();
        if visible.is_empty() {
            return;
        }
        let current = visible
            .iter()
            .position(|index| *index == self.focus)
            .unwrap_or(0);
        let next = if delta < 0 {
            current.saturating_sub(delta.unsigned_abs())
        } else {
            (current + delta as usize).min(visible.len() - 1)
        };
        self.focus = visible[next];
    }

    fn visible_positions(&self) -> Vec<usize> {
        self.fields
            .iter()
            .enumerate()
            .filter(|(_, field)| field.shown)
            .map(|(index, _)| index)
            .collect()
    }

    fn edit_horizontal(&mut self, delta: isize) {
        match self.focused_kind() {
            Some(FieldKind::Choice(_) | FieldKind::Credential | FieldKind::Model(_)) => {
                if let Some(field) = self.focused_mut() {
                    field.cycle(delta);
                }
                self.sync_shown();
            }
            Some(FieldKind::Text | FieldKind::Secret | FieldKind::Number | FieldKind::Tags) => {
                if let Some(field) = self.focused_mut() {
                    if delta < 0 {
                        field.text.move_left();
                    } else {
                        field.text.move_right();
                    }
                }
            }
            _ => {}
        }
    }

    fn edit_home(&mut self) {
        if matches!(self.focused_kind(), Some(FieldKind::Model(_))) {
            if self.prepare_model_text(true)
                && let Some(field) = self.focused_mut()
            {
                field.text.move_home();
            }
            return;
        }
        if let Some(field) = self.text_field_mut() {
            field.text.move_home();
        }
    }

    fn edit_end(&mut self) {
        if matches!(self.focused_kind(), Some(FieldKind::Model(_))) {
            if self.prepare_model_text(true)
                && let Some(field) = self.focused_mut()
            {
                field.text.move_end();
            }
            return;
        }
        if let Some(field) = self.text_field_mut() {
            field.text.move_end();
        }
    }

    fn edit_backspace(&mut self) {
        if matches!(self.focused_kind(), Some(FieldKind::Model(_))) {
            if self.prepare_model_text(true)
                && let Some(field) = self.focused_mut()
            {
                field.text.backspace();
                field.error = None;
            }
            return;
        }
        if let Some(field) = self.text_field_mut() {
            field.secret_clear = false;
            field.text.backspace();
            field.error = None;
        }
    }

    fn edit_delete(&mut self) {
        if matches!(self.focused_kind(), Some(FieldKind::Model(_))) {
            if self.prepare_model_text(true)
                && let Some(field) = self.focused_mut()
            {
                field.text.delete();
                field.error = None;
            }
            return;
        }
        if let Some(field) = self.text_field_mut() {
            field.secret_clear = false;
            field.text.delete();
            field.error = None;
        }
    }

    fn insert_char(&mut self, ch: char) {
        let kind = self.focused_kind();
        match kind {
            Some(FieldKind::Toggle) if ch == ' ' => self.toggle_focused(),
            Some(FieldKind::Advanced) if ch == ' ' => self.toggle_advanced(),
            Some(FieldKind::Text | FieldKind::Secret | FieldKind::Number | FieldKind::Tags) => {
                if let Some(field) = self.text_field_mut() {
                    field.secret_clear = false;
                    field.text.insert_char(ch);
                    field.error = None;
                }
            }
            Some(FieldKind::Model(_)) => {
                if self.prepare_model_text(false)
                    && let Some(field) = self.focused_mut()
                {
                    field.text.insert_char(ch);
                    field.error = None;
                }
            }
            _ => {}
        }
    }

    fn activate_focused(&mut self) {
        let kind = self.focused_kind();
        match kind {
            Some(FieldKind::Choice(_) | FieldKind::Credential | FieldKind::Model(_)) => {
                if let Some(field) = self.focused_mut() {
                    field.cycle(1);
                }
                self.sync_shown();
            }
            Some(FieldKind::Toggle) => self.toggle_focused(),
            Some(FieldKind::Advanced) => self.toggle_advanced(),
            _ => {}
        }
    }

    fn toggle_focused(&mut self) {
        if let Some(field) = self.focused_mut()
            && field.spec.kind == FieldKind::Toggle
        {
            field.toggle = !field.toggle;
            field.error = None;
        }
    }

    fn toggle_advanced(&mut self) {
        self.advanced_open = !self.advanced_open;
        self.sync_shown();
    }

    fn clear_secret(&mut self) {
        let Some(field) = self.focused_mut() else {
            return;
        };
        if field.spec.kind != FieldKind::Secret {
            return;
        }
        field.secret_clear = true;
        field.text.clear();
        field.error = None;
    }

    fn focused_kind(&self) -> Option<FieldKind> {
        self.fields.get(self.focus).map(|field| field.spec.kind)
    }

    fn focused_mut(&mut self) -> Option<&mut EditorField> {
        self.fields.get_mut(self.focus)
    }

    fn text_field_mut(&mut self) -> Option<&mut EditorField> {
        let field = self.fields.get_mut(self.focus)?;
        is_text_kind(field.spec.kind).then_some(field)
    }

    fn sync_shown(&mut self) {
        let credential = self
            .fields
            .iter()
            .find(|field| field.spec.key == "credential")
            .map(|field| field.choice_value().to_string());
        let advanced = self.advanced_open;
        let platform = self.platform;
        for field in &mut self.fields {
            field.shown = match field.spec.kind {
                FieldKind::Advanced => true,
                _ if field.spec.group == schema::FieldGroup::Advanced => advanced,
                _ => match field.spec.key {
                    "api_key" => credential.as_deref() == Some("replace"),
                    "env_key" if platform == Platform::Grok => credential.as_deref() == Some("env"),
                    _ => true,
                },
            };
        }
        if self
            .fields
            .get(self.focus)
            .is_some_and(|field| !field.shown)
            && let Some(index) = self.visible_positions().into_iter().next()
        {
            self.focus = index;
        }
    }

    /// Switch a model field onto its custom slot and replace the typed id.
    fn prepare_model_text(&mut self, seed_preset: bool) -> bool {
        let Some(field) = self.fields.get_mut(self.focus) else {
            return false;
        };
        if !matches!(field.spec.kind, FieldKind::Model(_)) {
            return false;
        }
        if field
            .choices
            .get(field.choice_index)
            .is_some_and(|item| item.custom)
        {
            return true;
        }
        let seeded = if seed_preset {
            field.choice_value().to_string()
        } else {
            String::new()
        };
        let Some(index) = field.choices.iter().position(|item| item.custom) else {
            return false;
        };
        field.choice_index = index;
        field.text = TextBuffer::new(seeded);
        field.error = None;
        true
    }

    #[cfg(test)]
    pub(crate) fn set_text(&mut self, key: &str, value: &str) {
        if let Some(field) = self.fields.iter_mut().find(|field| field.spec.key == key) {
            if matches!(field.spec.kind, FieldKind::Model(_))
                && let Some(index) = field.choices.iter().position(|item| item.custom)
            {
                field.choice_index = index;
            }
            field.text.set_value(value);
        }
    }

    #[cfg(test)]
    pub(crate) fn set_choice(&mut self, key: &str, value: &str) {
        if let Some(field) = self.fields.iter_mut().find(|field| field.spec.key == key)
            && let Some(index) = field.choices.iter().position(|item| item.value == value)
        {
            field.choice_index = index;
        }
        self.sync_shown();
    }

    #[cfg(test)]
    pub(crate) fn mark_secret_clear(&mut self, key: &str) {
        if let Some(field) = self.fields.iter_mut().find(|field| field.spec.key == key) {
            field.secret_clear = true;
            field.text.clear();
        }
    }
}

fn is_text_kind(kind: FieldKind) -> bool {
    matches!(
        kind,
        FieldKind::Text | FieldKind::Secret | FieldKind::Number | FieldKind::Tags
    )
}
