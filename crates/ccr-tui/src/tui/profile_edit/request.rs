//! Draft to `EditProfileRequest` / `CreateProfileRequest`.

use std::collections::BTreeSet;

use ccr_cli::application::profile_create::{CreateProfileRequest, CreateSecret, ProfileFieldValue};
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

pub(crate) struct CreateDraft<'a> {
    pub platform: Platform,
    pub revision: &'a str,
    pub fields: &'a [EditorField],
}

#[derive(Clone, Debug)]
pub(crate) struct CreateRequest(pub CreateProfileRequest);

impl CreateRequest {
    pub(crate) fn platform(&self) -> Platform {
        self.0.platform
    }

    pub(crate) fn name(&self) -> &str {
        &self.0.name
    }

    pub(crate) fn into_inner(self) -> CreateProfileRequest {
        self.0
    }
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
    Set(String),
    SecretSet,
    SecretUnset,
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
            PreviewValue::Text { .. } | PreviewValue::Set(_) => lines.push(label),
            PreviewValue::SecretSet => lines.push(format!(
                "{label} — {}",
                crate::tui_text!("‹set›", "‹已设置›")
            )),
            PreviewValue::SecretUnset => lines.push(format!(
                "{label} — {}",
                crate::tui_text!("‹unset›", "‹未设置›")
            )),
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

pub(crate) fn create_auth_mode(
    platform: Platform,
    fields: &[EditorField],
) -> schema::CreateAuthMode {
    fields
        .iter()
        .find(|field| field.spec.key == "auth_mode")
        .map(|field| field.choice_value().trim())
        .filter(|value| !value.is_empty())
        .map(|value| schema::CreateAuthMode::parse(platform, value))
        .unwrap_or_else(|| schema::CreateAuthMode::default_for(platform))
}

pub(crate) fn validate_create(
    platform: Platform,
    fields: &mut [EditorField],
    existing_names: &BTreeSet<String>,
) -> Result<(), Vec<usize>> {
    for field in fields.iter_mut() {
        field.error = None;
    }
    let mut invalid = Vec::new();
    let auth_mode = create_auth_mode(platform, fields);

    let name = field_text(fields, "name").trim().to_string();
    if name.is_empty() {
        push_field_error(
            fields,
            &mut invalid,
            "name",
            crate::tui_text!("Profile name is required", "配置名称不能为空"),
        );
    } else if validate_profile_name(&name).is_err() {
        let msg = crate::tui_format!(
            "'{name}' is reserved or invalid",
            "'{name}' 是系统保留字或非法名称"
        );
        push_field_error(fields, &mut invalid, "name", &msg);
    } else if existing_names.contains(&name) {
        let msg = crate::tui_format!(
            "Profile '{name}' already exists; choose another name",
            "配置 '{name}' 已存在，请更换名称"
        );
        push_field_error(fields, &mut invalid, "name", &msg);
    }

    if auth_mode.is_key_visible("base_url") {
        let url = field_text(fields, "base_url").trim().to_string();
        if url.is_empty() {
            if auth_mode.is_field_required("base_url") {
                push_field_error(
                    fields,
                    &mut invalid,
                    "base_url",
                    crate::tui_text!(
                        "Base URL is required for this auth mode",
                        "当前认证模式下地址为必填"
                    ),
                );
            } else if auth_mode.requires_base_url_for_third_party_model()
                && choice_value(fields, "provider_type") == "third_party_model"
            {
                push_field_error(
                    fields,
                    &mut invalid,
                    "base_url",
                    crate::tui_text!(
                        "Base URL is required when provider_type is third_party_model",
                        "第三方模型类型下地址为必填"
                    ),
                );
            }
        } else if !is_http_url(&url) {
            push_field_error(
                fields,
                &mut invalid,
                "base_url",
                crate::tui_text!(
                    "URL must start with http:// or https://",
                    "地址必须以 http:// 或 https:// 开头"
                ),
            );
        }
    }

    if auth_mode.is_key_visible("env_key") {
        let env_key = field_text(fields, "env_key").trim().to_string();
        if env_key.is_empty() {
            if auth_mode.is_field_required("env_key") {
                push_field_error(
                    fields,
                    &mut invalid,
                    "env_key",
                    crate::tui_text!("Environment variable name is required", "请输入环境变量名"),
                );
            }
        } else if !is_valid_env_var_name(&env_key) {
            push_field_error(
                fields,
                &mut invalid,
                "env_key",
                crate::tui_text!(
                    "Enter one valid env variable name (letters, digits, _)",
                    "请输入合法的环境变量名（字母或下划线开头）"
                ),
            );
        }
    }

    if auth_mode.is_field_required("auth_token")
        && field_text(fields, "auth_token").trim().is_empty()
    {
        push_field_error(
            fields,
            &mut invalid,
            "auth_token",
            crate::tui_text!(
                "Secret token is required for this auth mode",
                "当前认证模式下密钥为必填"
            ),
        );
    }

    if auth_mode.is_field_required("api_key") && field_text(fields, "api_key").trim().is_empty() {
        push_field_error(
            fields,
            &mut invalid,
            "api_key",
            crate::tui_text!("Enter an API key", "请输入 API 密钥"),
        );
    }

    if auth_mode.is_field_required("model") && model_value(fields, "model").is_empty() {
        push_field_error(
            fields,
            &mut invalid,
            "model",
            crate::tui_text!(
                "Model is required for this credential mode",
                "当前凭据模式下主模型为必填"
            ),
        );
    }

    if auth_mode.is_key_visible("provider_type")
        && choice_value(fields, "provider_type") == "official_relay"
        && auth_mode.conflicts_with_official_relay()
    {
        push_field_error(
            fields,
            &mut invalid,
            "provider_type",
            crate::tui_text!(
                "official_relay conflicts with the selected auth mode",
                "官方中转类型与当前认证模式冲突"
            ),
        );
    }

    let number_keys = fields
        .iter()
        .filter(|field| {
            field.spec.kind == FieldKind::Number && auth_mode.is_key_visible(field.spec.key)
        })
        .filter_map(|field| {
            let text = field.text.value().trim();
            if text.is_empty() {
                return None;
            }
            if field.spec.key == "context_window" {
                match text.parse::<u64>() {
                    Ok(value) if value > 0 => None,
                    _ => Some((field.spec.key, true)),
                }
            } else {
                text.parse::<u64>().err().map(|_| (field.spec.key, false))
            }
        })
        .collect::<Vec<_>>();
    for (key, positive_only) in number_keys {
        push_field_error(
            fields,
            &mut invalid,
            key,
            if positive_only {
                crate::tui_text!("Enter a positive integer", "请输入正整数")
            } else {
                crate::tui_text!("Enter a non-negative integer", "请输入非负整数")
            },
        );
    }

    invalid.sort_unstable();
    invalid.dedup();
    if invalid.is_empty() {
        Ok(())
    } else {
        Err(invalid)
    }
}

pub(crate) fn build_create(draft: &CreateDraft<'_>) -> Option<CreateRequest> {
    let name = field_text(draft.fields, "name").trim().to_string();
    if name.is_empty() {
        return None;
    }
    let auth_mode = create_auth_mode(draft.platform, draft.fields);
    let mut fields = Vec::new();
    if auth_mode.emits_auth_mode_field() {
        fields.push(ProfileFieldValue {
            field: "auth_mode".to_string(),
            value: json!(auth_mode.as_str()),
        });
    }
    for field in draft.fields {
        let key = field.spec.key;
        if matches!(
            key,
            "name" | "auth_mode" | "credential" | "advanced" | "auth_token" | "api_key"
        ) {
            continue;
        }
        if key == "env_key" && draft.platform == Platform::Grok {
            continue;
        }
        if !auth_mode.is_key_visible(key) {
            continue;
        }
        if let Some(entry) = create_field_value(field) {
            fields.push(entry);
        }
    }
    let secret = build_create_secret(auth_mode, draft.fields);
    Some(CreateRequest(CreateProfileRequest {
        platform: draft.platform,
        name,
        expected_revision: Some(draft.revision.to_string()),
        fields,
        secret,
    }))
}

pub(crate) fn create_preview_rows(draft: &CreateDraft<'_>) -> Vec<PreviewRow> {
    let auth_mode = create_auth_mode(draft.platform, draft.fields);
    let name = field_text(draft.fields, "name").trim().to_string();
    let mut rows = vec![
        PreviewRow {
            key: "name".to_string(),
            value: PreviewValue::Set(if name.is_empty() {
                crate::tui_text!("‹empty›", "‹未填写›").to_string()
            } else {
                name
            }),
        },
        PreviewRow {
            key: "auth_mode".to_string(),
            value: PreviewValue::Set(auth_mode.as_str().to_string()),
        },
    ];
    for field in draft.fields {
        let key = field.spec.key;
        if matches!(key, "name" | "auth_mode" | "credential" | "advanced") {
            continue;
        }
        if !auth_mode.is_key_visible(key) {
            continue;
        }
        match key {
            "auth_token" | "api_key" => {
                let has_secret = !field.text.value().trim().is_empty();
                rows.push(PreviewRow {
                    key: key.to_string(),
                    value: if has_secret {
                        PreviewValue::SecretSet
                    } else {
                        PreviewValue::SecretUnset
                    },
                });
            }
            "env_key" if draft.platform == Platform::Grok => {
                let env_name = field.text.value().trim();
                rows.push(PreviewRow {
                    key: "env_key".to_string(),
                    value: PreviewValue::Set(if env_name.is_empty() {
                        crate::tui_text!("‹empty›", "‹未填写›").to_string()
                    } else {
                        env_name.to_string()
                    }),
                });
            }
            _ => {
                if let Some(entry) = create_field_value(field) {
                    if key == "enabled" && entry.value == json!(true) {
                        continue;
                    }
                    let rendered = if key == "base_url" {
                        entry
                            .value
                            .as_str()
                            .map(ccr_cli::platforms::GrokPlatform::safe_base_url_for_display)
                            .unwrap_or_else(|| display_json(&entry.value))
                    } else {
                        display_json(&entry.value)
                    };
                    rows.push(PreviewRow {
                        key: key.to_string(),
                        value: PreviewValue::Set(rendered),
                    });
                }
            }
        }
    }
    rows
}

pub(crate) fn create_confirm_lines(draft: &CreateDraft<'_>) -> Vec<String> {
    let auth_mode = create_auth_mode(draft.platform, draft.fields);
    let mode_str = auth_mode.as_str();
    let mut lines = vec![crate::tui_format!(
        "• Auth mode: {mode_str}",
        "• 认证模式：{mode_str}"
    )];
    let mut field_names = Vec::new();
    for field in draft.fields {
        let key = field.spec.key;
        if matches!(
            key,
            "name" | "auth_mode" | "credential" | "advanced" | "auth_token" | "api_key"
        ) {
            continue;
        }
        if !auth_mode.is_key_visible(key) {
            continue;
        }
        if key == "env_key" && draft.platform == Platform::Grok {
            let env_name = field.text.value().trim();
            if !env_name.is_empty() {
                lines.push(crate::tui_format!(
                    "• Env variable: {env_name}",
                    "• 环境变量：{env_name}"
                ));
            }
            continue;
        }
        if let Some(entry) = create_field_value(field) {
            field_names.push(entry.field);
        }
    }
    if !field_names.is_empty() {
        let joined = field_names.join(", ");
        lines.push(crate::tui_format!(
            "• Fields: {joined}",
            "• 写入字段：{joined}"
        ));
    }
    if let Some(secret_key) = auth_mode.secret_field_key() {
        let set = !field_text(draft.fields, secret_key).trim().is_empty();
        let label = schema::label(secret_key);
        let status = if set {
            crate::tui_text!("‹set›", "‹已设置›")
        } else {
            crate::tui_text!("‹unset›", "‹未设置›")
        };
        lines.push(format!("• {label}: {status}"));
    }
    lines
}

fn create_field_value(field: &EditorField) -> Option<ProfileFieldValue> {
    let key = field.spec.key;
    match field.spec.kind {
        FieldKind::Toggle => {
            if key == "enabled" {
                Some(ProfileFieldValue {
                    field: key.to_string(),
                    value: json!(field.toggle),
                })
            } else if field.toggle {
                Some(ProfileFieldValue {
                    field: key.to_string(),
                    value: json!(true),
                })
            } else {
                None
            }
        }
        FieldKind::Number => {
            let text = field.text.value().trim();
            if text.is_empty() {
                None
            } else {
                let number = text.parse::<u64>().ok()?;
                Some(ProfileFieldValue {
                    field: key.to_string(),
                    value: json!(number),
                })
            }
        }
        FieldKind::Tags => {
            let tags = split_tags(field.text.value());
            (!tags.is_empty()).then(|| ProfileFieldValue {
                field: key.to_string(),
                value: json!(tags),
            })
        }
        FieldKind::Choice(_) => {
            let value = field.choice_value().trim();
            (!value.is_empty()).then(|| ProfileFieldValue {
                field: key.to_string(),
                value: json!(value),
            })
        }
        FieldKind::Model(_) => {
            let value = field.model_id().trim();
            (!value.is_empty()).then(|| ProfileFieldValue {
                field: key.to_string(),
                value: json!(value),
            })
        }
        FieldKind::Text => {
            let value = field.text.value().trim();
            (!value.is_empty()).then(|| ProfileFieldValue {
                field: key.to_string(),
                value: json!(value),
            })
        }
        FieldKind::Secret
        | FieldKind::AuthMode(_)
        | FieldKind::Credential
        | FieldKind::Advanced => None,
    }
}

fn build_create_secret(auth_mode: schema::CreateAuthMode, fields: &[EditorField]) -> CreateSecret {
    if let Some(secret_key) = auth_mode.secret_field_key() {
        let secret = field_text(fields, secret_key).trim().to_string();
        return if secret.is_empty() {
            CreateSecret::None
        } else {
            CreateSecret::Secret(Secret::new(secret))
        };
    }
    if auth_mode.uses_grok_env_key_secret() {
        let env_name = field_text(fields, "env_key").trim().to_string();
        return if env_name.is_empty() {
            CreateSecret::None
        } else {
            CreateSecret::UseEnvKey(env_name)
        };
    }
    CreateSecret::None
}

fn model_value(fields: &[EditorField], key: &str) -> String {
    fields
        .iter()
        .find(|field| field.spec.key == key)
        .map(|field| field.model_id().to_string())
        .unwrap_or_default()
}

fn push_field_error(
    fields: &mut [EditorField],
    invalid: &mut Vec<usize>,
    key: &str,
    message: &str,
) {
    if let Some((index, field)) = fields
        .iter_mut()
        .enumerate()
        .find(|(_, field)| field.spec.key == key)
    {
        field.error = Some(message.to_string());
        invalid.push(index);
    }
}

fn is_valid_env_var_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
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
    let is_base_url = field.spec.key == "base_url";
    let new = match &edit {
        FieldEdit::Clear { .. } => "—".to_string(),
        FieldEdit::Set { value, .. } => {
            if is_base_url {
                value
                    .as_str()
                    .map(ccr_cli::platforms::GrokPlatform::safe_base_url_for_display)
                    .unwrap_or_else(|| display_json(value))
            } else {
                display_json(value)
            }
        }
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
    } else if is_base_url {
        ccr_cli::platforms::GrokPlatform::safe_base_url_for_display(field.original_text.trim())
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
        FieldKind::Secret
        | FieldKind::AuthMode(_)
        | FieldKind::Credential
        | FieldKind::Advanced => None,
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

    #[test]
    fn create_auth_modes_filter_hidden_fields_preserve_buffers_and_redact_sentinel() {
        use crate::tui::profile_edit::{CreateEditorOpen, ProfileEditor};
        use ccr_cli::application::profile_create::CreateSecret;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        use std::collections::BTreeSet;

        // Claude: api_key vs subscription preserves typed buffers while excluding hidden fields.
        let mut claude = ProfileEditor::open_create(CreateEditorOpen {
            platform: Platform::Claude,
            revision: "",
            existing_names: BTreeSet::new(),
        });
        assert_eq!(claude.auth_mode(), "api_key");
        claude.set_text("name", "claude-work");
        claude.set_text("base_url", "https://relay.example.com");
        claude.set_text("auth_token", SENTINEL);
        claude.set_text("model", "claude-sonnet-4-5-20250929");
        assert!(claude.validate_create_fields());

        let api_key_req = claude.build_create_request().unwrap().into_inner();
        assert_eq!(api_key_req.expected_revision.as_deref(), Some(""));
        assert!(
            matches!(api_key_req.secret, CreateSecret::Secret(ref secret) if secret.expose() == SENTINEL)
        );
        assert!(
            api_key_req
                .fields
                .iter()
                .any(|f| f.field == "auth_mode" && f.value == json!("api_key"))
        );
        assert!(
            api_key_req
                .fields
                .iter()
                .any(|f| f.field == "enabled" && f.value == json!(true))
        );
        assert!(!format!("{api_key_req:?}").contains(SENTINEL));

        // Move focus from initial "name" up to "auth_mode", toggle to subscription via Right arrow,
        // and verify hidden buffers stay in memory yet are omitted from CreateProfileRequest.
        claude.handle_key(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(claude.focused_key(), "auth_mode");
        claude.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(claude.auth_mode(), "subscription");
        assert!(claude.validate_create_fields());
        let sub_req = claude.build_create_request().unwrap().into_inner();
        assert!(matches!(sub_req.secret, CreateSecret::None));
        assert!(!sub_req.fields.iter().any(|f| f.field == "base_url"));
        assert!(
            sub_req
                .fields
                .iter()
                .any(|f| f.field == "auth_mode" && f.value == json!("subscription"))
        );

        // Toggle back via Space -> buffers restored!
        claude.handle_key(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        assert_eq!(claude.auth_mode(), "api_key");
        let restored_req = claude.build_create_request().unwrap().into_inner();
        assert!(
            restored_req
                .fields
                .iter()
                .any(|f| f.field == "base_url" && f.value == json!("https://relay.example.com"))
        );
        assert!(
            matches!(restored_req.secret, CreateSecret::Secret(ref secret) if secret.expose() == SENTINEL)
        );
        let confirm = create_confirm_lines(&claude.create_draft());
        assert!(confirm.iter().all(|line| !line.contains(SENTINEL)));
        assert!(
            confirm
                .iter()
                .any(|line| line.contains("‹set›") || line.contains("‹已设置›"))
        );
        let overlay = crate::tui::overlay::Overlay::confirm_profile_create(
            "Claude",
            "claude-work",
            &confirm,
            None,
        );
        if let crate::tui::overlay::Overlay::ConfirmGate { message, .. } = overlay {
            assert!(message.iter().all(|line| !line.contains(SENTINEL)));
            assert!(
                message
                    .iter()
                    .any(|line| line.contains("not activated") || line.contains("不自动激活"))
            );
        } else {
            panic!("expected ConfirmGate");
        }

        // Codex: all 5 auth modes emit explicit auth_mode and respect mode visibility.
        let mut codex = ProfileEditor::open_create(CreateEditorOpen {
            platform: Platform::Codex,
            revision: "rev-cx",
            existing_names: BTreeSet::new(),
        });
        codex.set_text("name", "cx-new");
        codex.set_text("base_url", "https://codex-relay.example.com/v1");
        codex.set_text("env_key", "CODEX_PROVIDER_TOKEN");
        codex.set_text("auth_token", SENTINEL);

        for (mode, expects_secret, expects_env_key, expects_base_url) in [
            ("provider_bearer_token", true, false, true),
            ("provider_env_key", true, true, true),
            ("openai_api_key", true, false, true),
            ("openai_chatgpt", false, false, true),
            ("no_auth", false, false, true),
        ] {
            codex.set_choice("auth_mode", mode);
            assert!(codex.validate_create_fields(), "mode={mode}");
            let req = codex.build_create_request().unwrap().into_inner();
            assert!(
                req.fields
                    .iter()
                    .any(|f| f.field == "auth_mode" && f.value == json!(mode)),
                "mode={mode}"
            );
            assert_eq!(
                matches!(req.secret, CreateSecret::Secret(_)),
                expects_secret,
                "mode={mode}"
            );
            assert_eq!(
                req.fields.iter().any(|f| f.field == "env_key"),
                expects_env_key,
                "mode={mode}"
            );
            assert_eq!(
                req.fields.iter().any(|f| f.field == "base_url"),
                expects_base_url,
                "mode={mode}"
            );
            assert!(!format!("{req:?}").contains(SENTINEL));
        }

        // Grok: api_key, env_key, session map CreateSecret::{Secret, UseEnvKey, None}.
        let mut grok = ProfileEditor::open_create(CreateEditorOpen {
            platform: Platform::Grok,
            revision: "rev-gk",
            existing_names: BTreeSet::new(),
        });
        grok.set_text("name", "grok-new");
        grok.set_text("base_url", "https://api.x.ai/v1");
        grok.set_text("api_key", SENTINEL);
        grok.set_text("env_key", "XAI_API_KEY");
        grok.set_text("model", "grok-4");

        grok.set_choice("auth_mode", "api_key");
        assert!(grok.validate_create_fields());
        let gk_key = grok.build_create_request().unwrap().into_inner();
        assert!(matches!(gk_key.secret, CreateSecret::Secret(_)));
        assert!(!gk_key.fields.iter().any(|f| f.field == "env_key"));

        grok.set_choice("auth_mode", "env_key");
        assert!(grok.validate_create_fields());
        let gk_env = grok.build_create_request().unwrap().into_inner();
        assert!(matches!(gk_env.secret, CreateSecret::UseEnvKey(ref v) if v == "XAI_API_KEY"));
        assert!(!gk_env.fields.iter().any(|f| f.field == "env_key"));

        grok.set_choice("auth_mode", "session");
        assert!(grok.validate_create_fields());
        let gk_session = grok.build_create_request().unwrap().into_inner();
        assert!(matches!(gk_session.secret, CreateSecret::None));
        assert!(!gk_session.fields.iter().any(|f| f.field == "base_url"));
    }

    #[test]
    fn create_secret_masking_ctrl_d_clear_and_dirty_esc_guard() {
        use crate::tui::profile_edit::{CreateEditorOpen, EditorEvent, ProfileEditor};
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        use std::collections::BTreeSet;

        let mut editor = ProfileEditor::open_create(CreateEditorOpen {
            platform: Platform::Claude,
            revision: "",
            existing_names: BTreeSet::new(),
        });
        // Focus auth_token and paste multiline secret -> CR/LF stripped, dots match length.
        while editor.focused_key() != "auth_token" {
            editor.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        }
        editor.insert_paste("sk-part1\r\nsk-part2\n");
        let secret_field = editor
            .fields()
            .iter()
            .find(|f| f.spec.key == "auth_token")
            .unwrap();
        assert_eq!(secret_field.text.value(), "sk-part1sk-part2");
        assert_eq!(
            secret_field.text.secret_dots(),
            "•".repeat("sk-part1sk-part2".chars().count())
        );

        // Ctrl+D in Create mode clears buffer immediately and keeps secret_clear = false.
        editor.handle_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL));
        let cleared_field = editor
            .fields()
            .iter()
            .find(|f| f.spec.key == "auth_token")
            .unwrap();
        assert!(cleared_field.text.value().is_empty());
        assert!(!cleared_field.secret_clear);

        // Dirty Esc requires double press.
        editor.set_text("name", "draft-1");
        assert!(matches!(
            editor.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            EditorEvent::None
        ));
        assert!(matches!(
            editor.handle_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            EditorEvent::Cancel
        ));
    }

    #[test]
    fn create_local_validation_covers_names_urls_env_keys_numbers_and_jumps_to_advanced() {
        use crate::tui::profile_edit::{CreateEditorOpen, ProfileEditor};
        use std::collections::BTreeSet;

        let existing = BTreeSet::from(["existing-one".to_string()]);
        let mut claude = ProfileEditor::open_create(CreateEditorOpen {
            platform: Platform::Claude,
            revision: "",
            existing_names: existing.clone(),
        });

        // Empty name + missing required base_url + missing auth_token.
        assert!(!claude.validate_create_fields());
        assert_eq!(claude.focused_key(), "name");

        // Reserved names rejected.
        for reserved in ["default_config", "current_config", "settings"] {
            claude.set_text("name", reserved);
            claude.set_text("base_url", "https://relay.example.com");
            claude.set_text("auth_token", SENTINEL);
            assert!(!claude.validate_create_fields(), "reserved={reserved}");
            assert_eq!(claude.focused_key(), "name");
        }

        // Duplicate name rejected.
        claude.set_text("name", "existing-one");
        assert!(!claude.validate_create_fields());
        assert_eq!(claude.focused_key(), "name");

        // Invalid base_url rejected and focused.
        claude.set_text("name", "valid-name");
        claude.set_text("base_url", "ftp://bad.example.com");
        assert!(!claude.validate_create_fields());
        assert_eq!(claude.focused_key(), "base_url");

        // Invalid advanced u64 field auto-expands Advanced and focuses the invalid field.
        claude.set_text("base_url", "https://relay.example.com");
        claude.set_text("api_timeout_ms", "not-a-number");
        assert!(!claude.advanced_open());
        assert!(!claude.validate_create_fields());
        assert!(claude.advanced_open());
        assert_eq!(claude.focused_key(), "api_timeout_ms");
        claude.set_text("api_timeout_ms", "");
        assert!(claude.validate_create_fields());

        // Codex provider_bearer_token + official_relay provider_type rejected,
        // and provider_env_key rejects invalid env var identifier.
        let mut codex = ProfileEditor::open_create(CreateEditorOpen {
            platform: Platform::Codex,
            revision: "",
            existing_names: BTreeSet::new(),
        });
        codex.set_text("name", "cx-env");
        codex.set_text("base_url", "https://relay.example.com/v1");
        codex.set_text("auth_token", SENTINEL);
        codex.set_choice("provider_type", "official_relay");
        assert!(!codex.validate_create_fields());
        assert_eq!(codex.focused_key(), "provider_type");

        codex.set_choice("provider_type", "");
        codex.set_choice("auth_mode", "provider_env_key");
        codex.set_text("env_key", "1INVALID-ENV");
        assert!(!codex.validate_create_fields());
        assert_eq!(codex.focused_key(), "env_key");

        // Grok rejects 0 context_window.
        let mut grok = ProfileEditor::open_create(CreateEditorOpen {
            platform: Platform::Grok,
            revision: "",
            existing_names: BTreeSet::new(),
        });
        grok.set_text("name", "gk");
        grok.set_text("base_url", "https://api.x.ai/v1");
        grok.set_text("api_key", SENTINEL);
        grok.set_text("model", "grok-4");
        grok.set_text("context_window", "0");
        assert!(!grok.validate_create_fields());
        assert_eq!(grok.focused_key(), "context_window");
    }

    #[test]
    fn preview_rows_sanitize_base_url_credentials_query_and_fragment_in_create_and_edit() {
        use crate::tui::profile_edit::{CreateEditorOpen, EditorOpen, ProfileEditor};
        use ccr_cli::models::ProfileConfig;
        use std::collections::BTreeSet;

        const RAW_CREATE_URL: &str =
            "https://user:sk-secret@relay.example.com/v1?api_key=sk-secret#frag";
        const RAW_OLD_URL: &str = "https://olduser:sk-old@old.example.com/v1?token=sk-old#oldfrag";
        const RAW_NEW_URL: &str =
            "https://newuser:sk-new@new.example.com/v1?api_key=sk-new#newfrag";

        let mut create_editor = ProfileEditor::open_create(CreateEditorOpen {
            platform: Platform::Claude,
            revision: "",
            existing_names: BTreeSet::new(),
        });
        create_editor.set_text("name", "safe-preview");
        create_editor.set_text("base_url", RAW_CREATE_URL);
        create_editor.set_text("auth_token", SENTINEL);
        let create_rows = create_preview_rows(&create_editor.create_draft());
        let create_url_row = create_rows
            .iter()
            .find(|row| row.key == "base_url")
            .expect("base_url preview row");
        match &create_url_row.value {
            PreviewValue::Set(text) => {
                assert!(!text.contains("sk-secret"), "{text}");
                assert!(!text.contains("user:"), "{text}");
                assert!(!text.contains("frag"), "{text}");
                assert!(text.contains("https://relay.example.com/v1"), "{text}");
            }
            _ => panic!("expected PreviewValue::Set"),
        }

        let mut existing = ProfileConfig::new();
        existing.base_url = Some(RAW_OLD_URL.to_string());
        let mut edit_editor = ProfileEditor::open(EditorOpen {
            platform: Platform::Claude,
            name: "work",
            config: &existing,
            is_current: false,
            revision: "rev-1",
        });
        edit_editor.set_text("base_url", RAW_NEW_URL);
        let edit_rows = preview_rows(&edit_editor.draft());
        let edit_url_row = edit_rows
            .iter()
            .find(|row| row.key == "base_url")
            .expect("edit base_url preview row");
        match &edit_url_row.value {
            PreviewValue::Text { old, new } => {
                for leaked in [
                    "sk-old", "olduser", "oldfrag", "sk-new", "newuser", "newfrag",
                ] {
                    assert!(!old.contains(leaked), "old leaked {leaked}: {old}");
                    assert!(!new.contains(leaked), "new leaked {leaked}: {new}");
                }
                assert!(old.contains("https://old.example.com/v1"), "{old}");
                assert!(new.contains("https://new.example.com/v1"), "{new}");
            }
            _ => panic!("expected PreviewValue::Text"),
        }
    }

    #[test]
    fn codex_official_auth_modes_require_base_url_when_provider_type_is_third_party_model() {
        use crate::tui::profile_edit::{CreateEditorOpen, ProfileEditor};
        use std::collections::BTreeSet;

        let mut codex = ProfileEditor::open_create(CreateEditorOpen {
            platform: Platform::Codex,
            revision: "",
            existing_names: BTreeSet::new(),
        });
        codex.set_text("name", "cx-third-party");
        codex.set_text("auth_token", SENTINEL);

        for mode in ["openai_api_key", "openai_chatgpt"] {
            codex.set_choice("auth_mode", mode);
            codex.set_text("base_url", "");
            codex.set_choice("provider_type", "");
            assert!(
                codex.validate_create_fields(),
                "mode={mode} without third_party_model should allow empty base_url"
            );

            codex.set_choice("provider_type", "third_party_model");
            assert!(
                !codex.validate_create_fields(),
                "mode={mode} with third_party_model and empty base_url must fail"
            );
            assert_eq!(codex.focused_key(), "base_url", "mode={mode}");

            codex.set_text("base_url", "https://third-party.example.com/v1");
            assert!(codex.validate_create_fields(), "mode={mode}");
        }
    }
}
