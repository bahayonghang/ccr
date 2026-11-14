//! In-place profile editor and three-platform create wizard.

mod catalog;
mod field;
mod request;
mod schema;
pub(crate) mod ui;

use std::collections::BTreeSet;

use ccr_cli::models::{Platform, ProfileConfig};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(crate) use request::{CreateRequest, EditRequest};
pub(crate) use schema::{CreateAuthMode, EditorMode, platform_label};

use field::{EditorField, TextBuffer, load_create_fields, load_fields};
use request::{
    CreateDraft, Draft, build, build_create, confirm_lines, create_auth_mode, create_confirm_lines,
    validate, validate_create,
};
use schema::FieldKind;

pub(crate) struct EditorOpen<'a> {
    pub platform: Platform,
    pub name: &'a str,
    pub config: &'a ProfileConfig,
    pub is_current: bool,
    pub revision: &'a str,
}

pub(crate) struct CreateEditorOpen<'a> {
    pub platform: Platform,
    pub revision: &'a str,
    pub existing_names: BTreeSet<String>,
}

pub(crate) enum EditorEvent {
    None,
    Save,
    Cancel,
}

pub(crate) struct ProfileEditor {
    mode: EditorMode,
    platform: Platform,
    original_name: String,
    revision: String,
    is_current: bool,
    existing_names: BTreeSet<String>,
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
            mode: EditorMode::Edit,
            platform: source.platform,
            original_name: source.name.to_string(),
            revision: source.revision.to_string(),
            is_current: source.is_current,
            existing_names: BTreeSet::new(),
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

    pub(crate) fn open_create(source: CreateEditorOpen<'_>) -> Self {
        let fields = load_create_fields(source.platform);
        let initial_focus = fields
            .iter()
            .position(|field| field.spec.key == "name")
            .unwrap_or(0);
        let mut editor = Self {
            mode: EditorMode::Create,
            platform: source.platform,
            original_name: String::new(),
            revision: source.revision.to_string(),
            is_current: false,
            existing_names: source.existing_names,
            fields,
            focus: initial_focus,
            advanced_open: false,
            discard_armed: false,
            banner: None,
            form_scroll: 0,
        };
        editor.sync_shown();
        editor
    }

    pub(crate) fn mode(&self) -> EditorMode {
        self.mode
    }

    pub(crate) fn is_create(&self) -> bool {
        self.mode == EditorMode::Create
    }

    pub(crate) fn create_auth_mode_enum(&self) -> CreateAuthMode {
        create_auth_mode(self.platform, &self.fields)
    }

    #[cfg(test)]
    pub(crate) fn auth_mode(&self) -> &'static str {
        self.create_auth_mode_enum().as_str()
    }

    pub(crate) fn has_existing_name(&self, name: &str) -> bool {
        self.existing_names.contains(name)
    }

    pub(crate) fn draft_name(&self) -> String {
        self.fields
            .iter()
            .find(|field| field.spec.key == "name")
            .map(|field| field.text.value().trim().to_string())
            .unwrap_or_default()
    }

    pub(crate) fn update_catalog_state(
        &mut self,
        revision: String,
        existing_names: BTreeSet<String>,
    ) {
        if self.is_create() {
            self.revision = revision;
            self.existing_names = existing_names;
        }
    }

    pub(crate) fn attach_name_error(
        &mut self,
        field_error: impl Into<String>,
        banner: impl Into<String>,
    ) {
        if let Some((index, field)) = self
            .fields
            .iter_mut()
            .enumerate()
            .find(|(_, field)| field.spec.key == "name")
        {
            field.error = Some(field_error.into());
            self.focus = index;
        }
        self.banner = Some(banner.into());
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

    pub(crate) fn create_draft(&self) -> CreateDraft<'_> {
        CreateDraft {
            platform: self.platform,
            revision: &self.revision,
            fields: &self.fields,
        }
    }

    pub(crate) fn confirm_lines(&self) -> Vec<String> {
        if self.is_create() {
            create_confirm_lines(&self.create_draft())
        } else {
            confirm_lines(&self.draft())
        }
    }

    pub(crate) fn build_request(&self) -> Option<EditRequest> {
        build(&self.draft())
    }

    pub(crate) fn build_create_request(&self) -> Option<CreateRequest> {
        build_create(&self.create_draft())
    }

    pub(crate) fn validate_create_fields(&mut self) -> bool {
        match validate_create(self.platform, &mut self.fields, &self.existing_names) {
            Ok(()) => {
                self.banner = None;
                true
            }
            Err(invalid_indices) => {
                if let Some(&first) = invalid_indices.first() {
                    if self
                        .fields
                        .get(first)
                        .is_some_and(|field| field.spec.group == schema::FieldGroup::Advanced)
                    {
                        self.advanced_open = true;
                        self.sync_shown();
                    }
                    self.focus = first;
                    let first_err = self
                        .fields
                        .get(first)
                        .and_then(|field| field.error.as_deref())
                        .unwrap_or("");
                    let count = invalid_indices.len();
                    self.banner = Some(if count > 1 {
                        crate::tui_format!(
                            "Cannot create profile: {first_err} ({count} fields need attention)",
                            "无法提交：{first_err}（共 {count} 处待修正）"
                        )
                    } else {
                        crate::tui_format!(
                            "Cannot create profile: {first_err}",
                            "无法提交：{first_err}"
                        )
                    });
                }
                false
            }
        }
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
            Some(
                FieldKind::AuthMode(_)
                | FieldKind::Choice(_)
                | FieldKind::Credential
                | FieldKind::Model(_),
            ) => {
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
            Some(FieldKind::AuthMode(_) | FieldKind::Choice(_) | FieldKind::Credential)
                if ch == ' ' =>
            {
                if let Some(field) = self.focused_mut() {
                    field.cycle(1);
                }
                self.sync_shown();
            }
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
            Some(
                FieldKind::AuthMode(_)
                | FieldKind::Choice(_)
                | FieldKind::Credential
                | FieldKind::Model(_),
            ) => {
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
        let is_create = self.is_create();
        let Some(field) = self.focused_mut() else {
            return;
        };
        if field.spec.kind != FieldKind::Secret {
            return;
        }
        field.secret_clear = !is_create;
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
        let advanced = self.advanced_open;
        let platform = self.platform;
        if self.is_create() {
            let auth_mode = create_auth_mode(platform, &self.fields);
            for field in &mut self.fields {
                field.shown = auth_mode.is_field_visible(field.spec, advanced);
            }
        } else {
            let credential = self
                .fields
                .iter()
                .find(|field| field.spec.key == "credential")
                .map(|field| field.choice_value().to_string());
            for field in &mut self.fields {
                field.shown = match field.spec.kind {
                    FieldKind::Advanced => true,
                    _ if field.spec.group == schema::FieldGroup::Advanced => advanced,
                    _ => match field.spec.key {
                        "api_key" => credential.as_deref() == Some("replace"),
                        "env_key" if platform == Platform::Grok => {
                            credential.as_deref() == Some("env")
                        }
                        _ => true,
                    },
                };
            }
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
