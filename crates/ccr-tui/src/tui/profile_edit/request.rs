//! Draft to `EditProfileRequest`. Only changed fields and the secret action are included.

use ccr_cli::application::profile_edit::{EditProfileRequest, FieldEdit, SecretEdit};
use ccr_cli::models::{ApplyPolicy, Platform};
use ccr_config::managers::config::repository::validate_profile_name;
use ccr_core::Secret;
use serde_json::{Value, json};

use super::field::EditorField;
use super::schema::{self, FieldKind};

pub(crate) struct Draft<'a> {
    pub platform: Platform,
    pub original_name: &'a str,
    pub revision: &'a str,
    pub fields: &'a [EditorField],
}

pub(crate) struct EditRequest(pub EditProfileRequest);

impl EditRequest {
    pub(crate) fn platform(&self) -> Platform {
        self.0.platform
    }

    pub(crate) fn name(&self) -> &str {
        &self.0.name
    }

    pub(crate) fn into_inner(self) -> EditProfileRequest {
        self.0
    }
}

impl Clone for EditRequest {
    fn clone(&self) -> Self {
        Self(clone_request(&self.0))
    }
}

pub(crate) enum PreviewValue {
    Text { old: String, new: String },
    Replaced,
    Clear,
}

pub(crate) struct PreviewRow {
    pub key: String,
    pub value: PreviewValue,
}

pub(crate) fn validate(fields: &mut [EditorField], original_name: &str, name_taken: bool) -> bool {
    for field in fields.iter_mut() {
        field.error = None;
    }
    let mut ok = true;
    let name = field_text(fields, "name").trim().to_string();
    if validate_profile_name(&name).is_err() {
        set_error(
            fields,
            "name",
            crate::tui_text!("Name is empty or reserved", "名称为空或属于保留字段"),
        );
        ok = false;
    } else if name != original_name && name_taken {
        set_error(
            fields,
            "name",
            crate::tui_text!("A profile with this name already exists", "已有同名配置"),
        );
        ok = false;
    }
    let url = field_text(fields, "base_url").trim().to_string();
    if !url.is_empty() && !is_http_url(&url) {
        set_error(
            fields,
            "base_url",
            crate::tui_text!(
                "URL must start with http:// or https://",
                "地址必须以 http:// 或 https:// 开头"
            ),
        );
        ok = false;
    }
    let invalid_numbers = fields
        .iter()
        .filter(|field| field.spec.kind == FieldKind::Number)
        .filter(|field| {
            let text = field.text.value().trim();
            !text.is_empty() && text.parse::<u64>().is_err()
        })
        .map(|field| field.spec.key)
        .collect::<Vec<_>>();
    for key in invalid_numbers {
        set_error(
            fields,
            key,
            crate::tui_text!("Enter a non-negative integer", "请输入非负整数"),
        );
        ok = false;
    }
    if choice_value(fields, "credential") == "replace"
        && field_text(fields, "api_key").trim().is_empty()
    {
        set_error(
            fields,
            "api_key",
            crate::tui_text!("Enter an API key", "请输入 API 密钥"),
        );
        ok = false;
    }
    if choice_value(fields, "credential") == "env" {
        let env_key = field_text(fields, "env_key");
        let env_key = env_key.trim();
        if env_key.is_empty() || env_key.contains(',') || env_key.starts_with('[') {
            set_error(
                fields,
                "env_key",
                crate::tui_text!(
                    "Enter one environment variable name",
                    "请输入一个环境变量名"
                ),
            );
            ok = false;
        }
    }
    ok
}

pub(crate) fn build(draft: &Draft<'_>) -> Option<EditRequest> {
    let mut fields = Vec::new();
    let mut secret = SecretEdit::Keep;
    let mut rename = None;
    for field in draft.fields {
        match field.spec.key {
            "name" => {
                let new_name = field.text.value().trim();
                if new_name != draft.original_name {
                    rename = Some(new_name.to_string());
                }
            }
            "credential" => secret = credential_secret(draft.fields, field),
            "auth_token" => secret = token_secret(field),
            "api_key" | "advanced" => {}
            "env_key" if draft.platform == Platform::Grok => {}
            _ => {
                if let Some(edit) = field_edit(field) {
                    fields.push(edit);
                }
            }
        }
    }
    let renamed = rename.is_some();
    let secret_change = !matches!(secret, SecretEdit::Keep);
    if fields.is_empty() && !secret_change && !renamed {
        return None;
    }
    Some(EditRequest(EditProfileRequest {
        platform: draft.platform,
        name: draft.original_name.to_string(),
        rename,
        expected_revision: Some(draft.revision.to_string()),
        fields,
        secret,
        apply: ApplyPolicy::IfCurrent,
    }))
}

pub(crate) fn confirm_lines(draft: &Draft<'_>) -> Vec<String> {
    let mut lines = Vec::new();
    for row in preview_rows(draft) {
        let label = schema::label(&row.key).to_string();
        match row.value {
            PreviewValue::Text { .. } => lines.push(label),
            PreviewValue::Replaced => lines.push(format!(
                "{label} — {}",
                crate::tui_text!("replaced", "已替换")
            )),
            PreviewValue::Clear => lines.push(format!(
                "{label} — {}",
                crate::tui_text!("will clear", "将清除")
            )),
        }
    }
    lines
}

pub(crate) fn preview_rows(draft: &Draft<'_>) -> Vec<PreviewRow> {
    let mut rows = Vec::new();
    for field in draft.fields {
        match field.spec.key {
            "name" if field.text.value().trim() != draft.original_name => rows.push(PreviewRow {
                key: "name".to_string(),
                value: PreviewValue::Text {
                    old: draft.original_name.to_string(),
                    new: field.text.value().trim().to_string(),
                },
            }),
            "credential" => match credential_secret(draft.fields, field) {
                SecretEdit::Replace(_) => rows.push(secret_row("api_key", PreviewValue::Replaced)),
                SecretEdit::Clear => rows.push(secret_row("api_key", PreviewValue::Clear)),
                SecretEdit::UseEnvKey(name) => {
                    let old = draft
                        .fields
                        .iter()
                        .find(|item| item.spec.key == "env_key")
                        .map(|item| item.original_text.trim().to_string())
                        .filter(|value| !value.is_empty())
                        .unwrap_or_else(|| "—".to_string());
                    rows.push(PreviewRow {
                        key: "env_key".to_string(),
                        value: PreviewValue::Text { old, new: name },
                    });
                }
                SecretEdit::Keep => {}
            },
            "auth_token" => match token_secret(field) {
                SecretEdit::Replace(_) => {
                    rows.push(secret_row("auth_token", PreviewValue::Replaced))
                }
                SecretEdit::Clear => rows.push(secret_row("auth_token", PreviewValue::Clear)),
                SecretEdit::Keep | SecretEdit::UseEnvKey(_) => {}
            },
            "api_key" | "advanced" => {}
            "env_key" if draft.platform == Platform::Grok => {}
            _ => {
                if let Some(edit) = field_edit(field) {
                    rows.push(text_preview(field, edit));
                }
            }
        }
    }
    rows
}

fn secret_row(key: &str, value: PreviewValue) -> PreviewRow {
    PreviewRow {
        key: key.to_string(),
        value,
    }
}

fn text_preview(field: &EditorField, edit: FieldEdit) -> PreviewRow {
    let new = match &edit {
        FieldEdit::Clear { .. } => "—".to_string(),
        FieldEdit::Set { value, .. } => display_json(value),
        FieldEdit::Legacy { .. } => "—".to_string(),
    };
    let old = if matches!(field.spec.kind, FieldKind::Model(_)) {
        let original = field.original_text.trim();
        if original.is_empty() {
            "—".to_string()
        } else {
            original.to_string()
        }
    } else if field.original_text.trim().is_empty() && !matches!(field.spec.kind, FieldKind::Toggle)
    {
        "—".to_string()
    } else if matches!(field.spec.kind, FieldKind::Toggle) {
        yes_no(field.original_toggle)
    } else if matches!(field.spec.kind, FieldKind::Choice(_)) {
        field
            .choices
            .get(field.original_choice)
            .map(|item| item.value.clone())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "—".to_string())
    } else {
        field.original_text.trim().to_string()
    };
    PreviewRow {
        key: field.spec.key.to_string(),
        value: PreviewValue::Text { old, new },
    }
}

fn display_json(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Bool(flag) => yes_no(*flag),
        Value::Number(number) => number.to_string(),
        Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        _ => value.to_string(),
    }
}

fn yes_no(flag: bool) -> String {
    if flag {
        crate::tui_text!("yes", "是").to_string()
    } else {
        crate::tui_text!("no", "否").to_string()
    }
}

fn credential_secret(fields: &[EditorField], field: &EditorField) -> SecretEdit {
    if !field.is_dirty() {
        return SecretEdit::Keep;
    }
    match field.choice_value() {
        "replace" => SecretEdit::Replace(Secret::new(
            field_text(fields, "api_key").trim().to_string(),
        )),
        "env" => SecretEdit::UseEnvKey(field_text(fields, "env_key").trim().to_string()),
        "clear" => SecretEdit::Clear,
        _ => SecretEdit::Keep,
    }
}

fn token_secret(field: &EditorField) -> SecretEdit {
    if field.secret_clear {
        SecretEdit::Clear
    } else if !field.text.value().is_empty() {
        SecretEdit::Replace(Secret::new(field.text.value().trim().to_string()))
    } else {
        SecretEdit::Keep
    }
}

fn field_edit(field: &EditorField) -> Option<FieldEdit> {
    if !field.is_dirty() {
        return None;
    }
    let key = field.spec.key.to_string();
    match field.spec.kind {
        FieldKind::Toggle => Some(FieldEdit::Set {
            field: key,
            value: json!(field.toggle),
        }),
        FieldKind::Number => {
            let text = field.text.value().trim();
            if text.is_empty() {
                Some(FieldEdit::Clear { field: key })
            } else {
                let number = text.parse::<u64>().ok()?;
                Some(FieldEdit::Set {
                    field: key,
                    value: json!(number),
                })
            }
        }
        FieldKind::Tags => {
            let tags = split_tags(field.text.value());
            if tags.is_empty() {
                Some(FieldEdit::Clear { field: key })
            } else {
                Some(FieldEdit::Set {
                    field: key,
                    value: json!(tags),
                })
            }
        }
        FieldKind::Choice(_) => {
            let value = field.choice_value();
            if value.is_empty() {
                Some(FieldEdit::Clear { field: key })
            } else {
                Some(FieldEdit::Set {
                    field: key,
                    value: json!(value),
                })
            }
        }
        FieldKind::Model(_) => {
            let value = field.model_id();
            if value.is_empty() {
                Some(FieldEdit::Clear { field: key })
            } else {
                Some(FieldEdit::Set {
                    field: key,
                    value: json!(value),
                })
            }
        }
        FieldKind::Text => {
            let text = field.text.value().trim();
            if text.is_empty() {
                Some(FieldEdit::Clear { field: key })
            } else {
                Some(FieldEdit::Set {
                    field: key,
                    value: json!(text),
                })
            }
        }
        FieldKind::Secret | FieldKind::Credential | FieldKind::Advanced => None,
    }
}

fn split_tags(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(str::to_string)
        .collect()
}

fn field_text(fields: &[EditorField], key: &str) -> String {
    fields
        .iter()
        .find(|field| field.spec.key == key)
        .map(|field| field.text.value().to_string())
        .unwrap_or_default()
}

fn choice_value(fields: &[EditorField], key: &str) -> String {
    fields
        .iter()
        .find(|field| field.spec.key == key)
        .map(|field| field.choice_value().to_string())
        .unwrap_or_default()
}

fn set_error(fields: &mut [EditorField], key: &str, message: &str) {
    if let Some(field) = fields.iter_mut().find(|field| field.spec.key == key) {
        field.error = Some(message.to_string());
    }
}

fn is_http_url(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

fn clone_request(request: &EditProfileRequest) -> EditProfileRequest {
    EditProfileRequest {
        platform: request.platform,
        name: request.name.clone(),
        rename: request.rename.clone(),
        expected_revision: request.expected_revision.clone(),
        fields: request.fields.iter().map(clone_field).collect(),
        secret: clone_secret(&request.secret),
        apply: request.apply,
    }
}

fn clone_field(edit: &FieldEdit) -> FieldEdit {
    match edit {
        FieldEdit::Set { field, value } => FieldEdit::Set {
            field: field.clone(),
            value: value.clone(),
        },
        FieldEdit::Clear { field } => FieldEdit::Clear {
            field: field.clone(),
        },
        FieldEdit::Legacy {
            field,
            value,
            value_json,
            clear,
        } => FieldEdit::Legacy {
            field: field.clone(),
            value: value.clone(),
            value_json: value_json.clone(),
            clear: *clear,
        },
    }
}

fn clone_secret(secret: &SecretEdit) -> SecretEdit {
    match secret {
        SecretEdit::Keep => SecretEdit::Keep,
        SecretEdit::Replace(secret) => SecretEdit::Replace(Secret::new(secret.expose())),
        SecretEdit::Clear => SecretEdit::Clear,
        SecretEdit::UseEnvKey(name) => SecretEdit::UseEnvKey(name.clone()),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use crate::tui::profile_edit::field::load_fields;
    use ccr_cli::models::ProfileConfig;
    use serde_json::json;

    const SENTINEL: &str = "CCR_EDITOR_SECRET_SENTINEL_9f3a";

    fn draft<'a>(
        platform: Platform,
        name: &'a str,
        revision: &'a str,
        fields: &'a [EditorField],
    ) -> Draft<'a> {
        Draft {
            platform,
            original_name: name,
            revision,
            fields,
        }
    }

    fn edited(platform: Platform, name: &str) -> (ProfileConfig, Vec<EditorField>) {
        let mut config = ProfileConfig::new();
        config.description = Some("old-desc".into());
        config.model = Some("old-model".into());
        config
            .platform_data
            .insert("approval_policy".into(), json!("never"));
        config
            .platform_data
            .insert("untouched".into(), json!("keep-me"));
        if platform == Platform::Grok {
            config.auth_token = Some(Secret::new(SENTINEL));
            config
                .platform_data
                .insert("api_key".into(), json!(SENTINEL));
        }
        let fields = load_fields(platform, name, &config);
        (config, fields)
    }

    fn set_text(fields: &mut [EditorField], key: &str, value: &str) {
        let field = fields
            .iter_mut()
            .find(|field| field.spec.key == key)
            .unwrap();
        if matches!(field.spec.kind, FieldKind::Model(_))
            && let Some(index) = field.choices.iter().position(|item| item.custom)
        {
            field.choice_index = index;
        }
        field.text.set_value(value);
    }

    fn set_choice(fields: &mut [EditorField], key: &str, value: &str) {
        let field = fields
            .iter_mut()
            .find(|field| field.spec.key == key)
            .unwrap();
        field.choice_index = field
            .choices
            .iter()
            .position(|item| item.value == value)
            .unwrap();
    }

    #[test]
    fn description_and_model_leave_other_keys_out_of_the_request() {
        for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
            let (_config, mut fields) = edited(platform, "work");
            set_text(&mut fields, "description", "new-desc");
            set_text(&mut fields, "model", "new-model");
            let request = build(&draft(platform, "work", "rev-1", &fields))
                .unwrap()
                .into_inner();
            let names = request
                .fields
                .iter()
                .map(FieldEdit::name)
                .collect::<Vec<_>>();
            assert_eq!(names, vec!["description", "model"], "{platform:?}");
            assert!(request.fields.iter().any(|edit| matches!(
                edit,
                FieldEdit::Set { field, value }
                    if field == "model" && value == &json!("new-model")
            )));
            assert!(matches!(request.secret, SecretEdit::Keep));
            assert_eq!(request.expected_revision.as_deref(), Some("rev-1"));
            assert_eq!(request.apply, ApplyPolicy::IfCurrent);
            assert!(request.rename.is_none());
            assert!(!format!("{request:?}").contains("approval_policy"));
            assert!(!format!("{request:?}").contains(SENTINEL));
        }
    }

    #[test]
    fn codex_model_selection_writes_the_catalog_id() {
        let (_config, mut fields) = edited(Platform::Codex, "work");
        set_choice(&mut fields, "model", "gpt-6-astra");
        let request = build(&draft(Platform::Codex, "work", "rev", &fields))
            .unwrap()
            .into_inner();
        assert!(request.fields.iter().any(|edit| matches!(
            edit,
            FieldEdit::Set { field, value }
                if field == "model" && value == &json!("gpt-6-astra")
        )));
    }

    #[test]
    fn empty_secret_is_keep_and_typed_or_cleared_secrets_omit_the_value() {
        let (_config, mut fields) = edited(Platform::Claude, "work");
        let kept = build(&draft(Platform::Claude, "work", "rev", &fields)).is_none();
        assert!(kept);

        set_text(&mut fields, "auth_token", SENTINEL);
        let replaced = build(&draft(Platform::Claude, "work", "rev", &fields))
            .unwrap()
            .into_inner();
        match &replaced.secret {
            SecretEdit::Replace(secret) => assert_eq!(secret.expose(), SENTINEL),
            other => panic!("expected replace, got {other:?}"),
        }
        let debug = format!("{replaced:?}");
        assert!(!debug.contains(SENTINEL), "{debug}");
        let lines = confirm_lines(&draft(Platform::Claude, "work", "rev", &fields));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("已替换") || line.contains("replaced"))
        );
        assert!(lines.iter().all(|line| !line.contains(SENTINEL)));

        set_text(&mut fields, "auth_token", "");
        fields
            .iter_mut()
            .find(|field| field.spec.key == "auth_token")
            .unwrap()
            .secret_clear = true;
        let cleared = build(&draft(Platform::Claude, "work", "rev", &fields))
            .unwrap()
            .into_inner();
        assert!(matches!(cleared.secret, SecretEdit::Clear));
        assert!(!format!("{cleared:?}").contains(SENTINEL));
    }

    #[test]
    fn grok_credential_modes_map_without_a_token_field() {
        let (_config, mut fields) = edited(Platform::Grok, "work");
        assert!(
            build(&draft(Platform::Grok, "work", "rev", &fields)).is_none(),
            "keep is not a write"
        );

        set_choice(&mut fields, "credential", "replace");
        set_text(&mut fields, "api_key", SENTINEL);
        let replaced = build(&draft(Platform::Grok, "work", "rev", &fields))
            .unwrap()
            .into_inner();
        assert!(matches!(replaced.secret, SecretEdit::Replace(_)));
        assert!(replaced.fields.is_empty());
        assert!(!format!("{replaced:?}").contains(SENTINEL));

        set_choice(&mut fields, "credential", "env");
        set_text(&mut fields, "env_key", "XAI_API_KEY");
        let env = build(&draft(Platform::Grok, "work", "rev", &fields))
            .unwrap()
            .into_inner();
        assert!(matches!(env.secret, SecretEdit::UseEnvKey(ref name) if name == "XAI_API_KEY"));

        set_choice(&mut fields, "credential", "clear");
        let cleared = build(&draft(Platform::Grok, "work", "rev", &fields))
            .unwrap()
            .into_inner();
        assert!(matches!(cleared.secret, SecretEdit::Clear));
        let lines = confirm_lines(&draft(Platform::Grok, "work", "rev", &fields));
        assert!(lines.iter().all(|line| !line.contains(SENTINEL)));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("将清除") || line.contains("will clear"))
        );
    }
}
