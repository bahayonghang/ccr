//! Shared profile edit used by the CLI, `set-field`, and in-process editors.
//!
//! The use case does not print, read stdin, or spawn a worker. Callers supply
//! already collected field and secret changes and run it on one synchronous worker.

use crate::application::profile_lifecycle::{
    ProfileLifecycleContext, ProfileOutcome, ProfileStatus, ProfileWarning,
};
use crate::models::{ApplyPolicy, Platform, PlatformConfig, ProfileConfig};
use crate::platforms::GrokPlatform;
use ccr_core::Secret;
use ccr_core::core::error::{CcrError, Result};
use serde_json::Value;

/// One non-secret field change. Values stay out of `Debug`.
pub enum FieldEdit {
    Set {
        field: String,
        value: Value,
    },
    Clear {
        field: String,
    },
    /// Original `set-field` inputs, including `--value-json` rejection rules.
    Legacy {
        field: String,
        value: Option<String>,
        value_json: Option<String>,
        clear: bool,
    },
}

impl FieldEdit {
    pub fn name(&self) -> &str {
        match self {
            Self::Set { field, .. } | Self::Clear { field } | Self::Legacy { field, .. } => field,
        }
    }

    fn is_clear(&self) -> bool {
        match self {
            Self::Clear { .. } => true,
            Self::Legacy { clear, .. } => *clear,
            Self::Set { .. } => false,
        }
    }
}

/// Credential change. `Debug` redacts replacement values.
pub enum SecretEdit {
    Keep,
    Replace(Secret),
    Clear,
    /// Grok environment-variable name. This is not the secret itself.
    UseEnvKey(String),
}

impl std::fmt::Debug for SecretEdit {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Keep => formatter.write_str("Keep"),
            Self::Replace(secret) => formatter.debug_tuple("Replace").field(secret).finish(),
            Self::Clear => formatter.write_str("Clear"),
            Self::UseEnvKey(name) => formatter.debug_tuple("UseEnvKey").field(name).finish(),
        }
    }
}

pub struct EditProfileRequest {
    pub platform: Platform,
    pub name: String,
    pub rename: Option<String>,
    pub expected_revision: Option<String>,
    pub fields: Vec<FieldEdit>,
    pub secret: SecretEdit,
    pub apply: ApplyPolicy,
}

impl std::fmt::Debug for EditProfileRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EditProfileRequest")
            .field("platform", &self.platform)
            .field("name", &self.name)
            .field("rename", &self.rename)
            .field("expected_revision", &self.expected_revision)
            .field(
                "fields",
                &self.fields.iter().map(FieldEdit::name).collect::<Vec<_>>(),
            )
            .field("secret", &self.secret)
            .field("apply", &self.apply)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrokEditSummary {
    Updated,
    Renamed,
    UpdatedApplyFailed,
    RenameApplyFailed,
    RenameCleanupFailed,
}

impl GrokEditSummary {
    fn as_str(self) -> &'static str {
        match self {
            Self::Updated => "updated",
            Self::Renamed => "renamed",
            Self::UpdatedApplyFailed => "updated_apply_failed",
            Self::RenameApplyFailed => "rename_apply_failed",
            Self::RenameCleanupFailed => "rename_cleanup_failed",
        }
    }
}

#[derive(Debug, Clone)]
pub enum EditLifecycle {
    Profile(ProfileOutcome),
    Grok(GrokEditSummary),
}

#[derive(Debug, Clone)]
pub struct EditProfileOutcome {
    pub platform: Platform,
    pub old_name: String,
    pub new_name: String,
    pub changed_fields: Vec<String>,
    pub reapplied: bool,
    pub revision: Option<String>,
    pub lifecycle: EditLifecycle,
    pub warnings: Vec<String>,
}

impl EditProfileOutcome {
    pub fn is_success(&self) -> bool {
        match &self.lifecycle {
            EditLifecycle::Profile(outcome) => matches!(
                outcome.status,
                ProfileStatus::Applied | ProfileStatus::AppliedWithWarning
            ),
            EditLifecycle::Grok(GrokEditSummary::Updated | GrokEditSummary::Renamed) => true,
            EditLifecycle::Grok(_) => false,
        }
    }

    pub fn failure_message(&self) -> &'static str {
        match &self.lifecycle {
            EditLifecycle::Profile(outcome) => match outcome.status {
                ProfileStatus::RecoveryRequired => "重新应用失败，需要恢复；外部改动已保留",
                ProfileStatus::Unchanged => "重新应用失败，已恢复先前的配置",
                ProfileStatus::Applied | ProfileStatus::AppliedWithWarning => "Profile 已更新",
            },
            EditLifecycle::Grok(GrokEditSummary::UpdatedApplyFailed) => {
                "Profile 已保存，但重新应用运行时失败"
            }
            EditLifecycle::Grok(GrokEditSummary::RenameApplyFailed) => {
                "改名后的 profile 已保存，但重新应用运行时失败"
            }
            EditLifecycle::Grok(GrokEditSummary::RenameCleanupFailed) => {
                "改名后的 profile 已保存，但清理旧 profile 失败"
            }
            EditLifecycle::Grok(_) => "Profile 已更新",
        }
    }

    pub fn status_name(&self) -> &'static str {
        match &self.lifecycle {
            EditLifecycle::Profile(outcome) => match outcome.status {
                ProfileStatus::Unchanged => "unchanged",
                ProfileStatus::Applied => "applied",
                ProfileStatus::AppliedWithWarning => "applied_with_warning",
                ProfileStatus::RecoveryRequired => "recovery_required",
            },
            EditLifecycle::Grok(summary) => summary.as_str(),
        }
    }

    pub fn human_message(&self) -> String {
        let identity = if self.old_name == self.new_name {
            format!("'{}'", self.new_name)
        } else {
            format!("'{}' → '{}'", self.old_name, self.new_name)
        };
        let fields = self
            .changed_fields
            .iter()
            .filter(|field| field.as_str() != "name")
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let mut message = format!("已更新 {} Profile {identity}", self.platform.short_name());
        if !fields.is_empty() {
            message.push('：');
            message.push_str(&fields);
        }
        if self.reapplied {
            message.push_str("（已重新应用）");
        }
        message
    }

    pub fn to_json_value(&self) -> Value {
        serde_json::json!({
            "ok": self.is_success(),
            "platform": self.platform.short_name(),
            "old_name": self.old_name,
            "new_name": self.new_name,
            "changed_fields": self.changed_fields,
            "reapplied": self.reapplied,
            "revision": self.revision,
            "status": self.status_name(),
            "warnings": self.warnings,
        })
    }
}

/// Edit one profile through the platform's versioned lifecycle.
pub fn edit_profile(request: EditProfileRequest) -> Result<EditProfileOutcome> {
    let changed_fields = validate_request(&request)?;
    let platform = request.platform;
    let old_name = request.name.clone();
    let new_name = request
        .rename
        .clone()
        .filter(|name| name != &old_name)
        .unwrap_or_else(|| old_name.clone());
    let policy = request.apply;
    match platform {
        Platform::Claude | Platform::Codex => edit_with_lifecycle(
            request,
            platform,
            &old_name,
            &new_name,
            policy,
            changed_fields,
        ),
        Platform::Grok => edit_grok(request, &old_name, &new_name, policy, changed_fields),
        Platform::Gemini | Platform::Qwen | Platform::Droid => Err(CcrError::PlatformNotSupported(
            format!("{platform} 目前不支持 profile edit"),
        )),
    }
}

pub(crate) fn ensure_set_field_allowed(platform: Platform, field: &str) -> Result<()> {
    if set_field_allowed(platform, field) {
        Ok(())
    } else {
        Err(CcrError::ValidationError(format!(
            "平台 '{platform}' 不允许编辑字段 '{field}'"
        )))
    }
}

pub(crate) fn normalize_tags(tags: Vec<String>) -> Option<Vec<String>> {
    let normalized = tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn edit_with_lifecycle(
    request: EditProfileRequest,
    platform: Platform,
    old_name: &str,
    new_name: &str,
    policy: ApplyPolicy,
    changed_fields: Vec<String>,
) -> Result<EditProfileOutcome> {
    let context = ProfileLifecycleContext::new(platform)?;
    let outcome = context.update_with_policy(
        old_name,
        new_name,
        request.expected_revision.as_deref(),
        policy,
        |profile| apply_edits(platform, profile, &request.fields, &request.secret),
    )?;
    let reapplied = outcome.activation_committed
        && matches!(
            outcome.status,
            ProfileStatus::Applied | ProfileStatus::AppliedWithWarning
        );
    let warnings = warning_names(&outcome.warnings);
    let revision = context.manager().snapshot()?.version;
    Ok(EditProfileOutcome {
        platform,
        old_name: old_name.to_string(),
        new_name: new_name.to_string(),
        changed_fields,
        reapplied,
        revision: Some(revision),
        lifecycle: EditLifecycle::Profile(outcome),
        warnings,
    })
}

fn edit_grok(
    request: EditProfileRequest,
    old_name: &str,
    new_name: &str,
    policy: ApplyPolicy,
    changed_fields: Vec<String>,
) -> Result<EditProfileOutcome> {
    let platform = GrokPlatform::new()?;
    let outcome = platform.update_profile_versioned_with_policy(
        old_name,
        new_name,
        request.expected_revision.as_deref(),
        policy,
        |profile| apply_edits(Platform::Grok, profile, &request.fields, &request.secret),
    )?;
    let summary = match outcome {
        crate::platforms::grok::GrokProfileUpdateOutcome::Updated { .. } => {
            GrokEditSummary::Updated
        }
        crate::platforms::grok::GrokProfileUpdateOutcome::Renamed { .. } => {
            GrokEditSummary::Renamed
        }
        crate::platforms::grok::GrokProfileUpdateOutcome::UpdatedApplyFailed { .. } => {
            GrokEditSummary::UpdatedApplyFailed
        }
        crate::platforms::grok::GrokProfileUpdateOutcome::RenameApplyFailed { .. } => {
            GrokEditSummary::RenameApplyFailed
        }
        crate::platforms::grok::GrokProfileUpdateOutcome::RenameCleanupFailed { .. } => {
            GrokEditSummary::RenameCleanupFailed
        }
    };
    let current = platform.get_current_profile()?;
    let reapplied = policy == ApplyPolicy::IfCurrent
        && matches!(summary, GrokEditSummary::Updated | GrokEditSummary::Renamed)
        && current.as_deref() == Some(new_name);
    let revision = ccr_config::ConfigManager::new(&platform.paths().profiles_file)
        .snapshot()?
        .version;
    Ok(EditProfileOutcome {
        platform: Platform::Grok,
        old_name: old_name.to_string(),
        new_name: new_name.to_string(),
        changed_fields,
        reapplied,
        revision: Some(revision),
        lifecycle: EditLifecycle::Grok(summary),
        warnings: Vec::new(),
    })
}

fn validate_request(request: &EditProfileRequest) -> Result<Vec<String>> {
    if request.name.trim().is_empty() {
        return Err(CcrError::ValidationError("Profile 名称不能为空".into()));
    }
    if matches!(request.secret, SecretEdit::UseEnvKey(_)) && request.platform != Platform::Grok {
        return Err(CcrError::ValidationError(
            "仅 Grok 支持 env_key 凭据".into(),
        ));
    }
    let mut changed = Vec::new();
    let mut seen = Vec::new();
    for edit in &request.fields {
        let name = edit.name();
        if name.trim().is_empty() {
            return Err(CcrError::ValidationError("字段名不能为空".into()));
        }
        if seen.iter().any(|existing: &String| existing == name) {
            return Err(CcrError::ValidationError(format!("字段 '{name}' 重复")));
        }
        if !edit_field_allowed(request.platform, name) {
            return Err(CcrError::ValidationError(format!(
                "平台 '{}' 不允许编辑字段 '{name}'",
                request.platform
            )));
        }
        seen.push(name.to_string());
        push_unique(&mut changed, name);
        if request.platform == Platform::Grok && !edit.is_clear() && is_grok_credential(name) {
            for extra in ["api_key", "env_key", "auth_token"] {
                push_unique(&mut changed, extra);
            }
        }
    }
    match &request.secret {
        SecretEdit::Keep => {}
        SecretEdit::Replace(_) | SecretEdit::Clear if request.platform != Platform::Grok => {
            push_unique(&mut changed, "auth_token");
        }
        SecretEdit::Replace(_) | SecretEdit::Clear | SecretEdit::UseEnvKey(_) => {
            for extra in ["api_key", "env_key", "auth_token"] {
                push_unique(&mut changed, extra);
            }
        }
    }
    let renamed = request
        .rename
        .as_ref()
        .is_some_and(|name| name != &request.name);
    if renamed {
        changed.insert(0, "name".to_string());
    }
    let has_secret = !matches!(request.secret, SecretEdit::Keep);
    if request.fields.is_empty() && !has_secret && !renamed {
        return Err(CcrError::ValidationError("没有要修改的字段".into()));
    }
    Ok(changed)
}

fn push_unique(fields: &mut Vec<String>, field: &str) {
    if !fields.iter().any(|existing| existing == field) {
        fields.push(field.to_string());
    }
}

fn is_grok_credential(field: &str) -> bool {
    matches!(field, "api_key" | "env_key" | "auth_token")
}

fn edit_field_allowed(platform: Platform, field: &str) -> bool {
    const COMMON: &[&str] = &[
        "description",
        "base_url",
        "model",
        "provider",
        "tags",
        "enabled",
    ];
    if COMMON.contains(&field) || set_field_allowed(platform, field) {
        return true;
    }
    match platform {
        Platform::Claude => matches!(
            field,
            "default_opus_model"
                | "default_sonnet_model"
                | "default_haiku_model"
                | "default_fable_model"
                | "subagent_model"
                | "effort_level"
                | "api_timeout_ms"
                | "claude_code_auto_compact_window"
        ),
        Platform::Codex => matches!(
            field,
            "model_reasoning_effort" | "wire_api" | "env_key" | "requires_openai_auth"
        ),
        Platform::Grok | Platform::Gemini | Platform::Qwen | Platform::Droid => false,
    }
}

fn set_field_allowed(platform: Platform, field: &str) -> bool {
    editable_fields(platform).contains(&field)
}

fn editable_fields(platform: Platform) -> &'static [&'static str] {
    match platform {
        Platform::Codex => ccr_codex::CodexPlatform::editable_fields(),
        Platform::Claude => crate::platforms::ClaudePlatform::editable_fields(),
        Platform::Grok => GrokPlatform::editable_fields(),
        Platform::Gemini | Platform::Qwen | Platform::Droid => DEFAULT_EDITABLE_FIELDS,
    }
}

const DEFAULT_EDITABLE_FIELDS: &[&str] = &[
    "description",
    "base_url",
    "auth_token",
    "model",
    "small_fast_model",
    "provider",
    "provider_type",
    "account",
    "tags",
];

fn apply_edits(
    platform: Platform,
    profile: &mut ProfileConfig,
    fields: &[FieldEdit],
    secret: &SecretEdit,
) -> Result<()> {
    let mut codex_env_key_set = false;
    for edit in fields {
        let clearing = edit.is_clear();
        apply_field(profile, edit)?;
        if platform == Platform::Grok && !clearing && is_grok_credential(edit.name()) {
            enforce_grok_credential_exclusivity(profile, edit.name());
        }
        if platform == Platform::Codex && edit.name() == "env_key" && !clearing {
            codex_env_key_set = true;
        }
    }
    // Codex save re-derives auth_mode and would drop a newly written env_key
    // while an older explicit mode such as no_auth is still present.
    if codex_env_key_set {
        profile.platform_data.shift_remove("auth_mode");
    }
    apply_secret(platform, profile, secret)
}

fn apply_field(profile: &mut ProfileConfig, edit: &FieldEdit) -> Result<()> {
    match edit {
        FieldEdit::Clear { field } => update_profile_field(profile, field, None, None, true),
        FieldEdit::Legacy {
            field,
            value,
            value_json,
            clear,
        } => update_profile_field(profile, field, value.clone(), value_json.clone(), *clear),
        FieldEdit::Set { field, value } => apply_json_value(profile, field, value),
    }
}

fn apply_json_value(profile: &mut ProfileConfig, field: &str, value: &Value) -> Result<()> {
    match value {
        Value::Null => update_profile_field(profile, field, None, None, true),
        Value::String(text) => {
            update_profile_field(profile, field, Some(text.clone()), None, false)
        }
        Value::Bool(flag) => update_profile_field(
            profile,
            field,
            Some(if *flag { "true" } else { "false" }.to_string()),
            None,
            false,
        ),
        Value::Number(number) => {
            update_profile_field(profile, field, Some(number.to_string()), None, false)
        }
        Value::Array(_) if field == "tags" => {
            update_profile_field(profile, field, None, Some(value.to_string()), false)
        }
        _ => Err(CcrError::ValidationError(format!(
            "字段 '{field}' 的值类型不受支持"
        ))),
    }
}

fn apply_secret(
    platform: Platform,
    profile: &mut ProfileConfig,
    secret: &SecretEdit,
) -> Result<()> {
    match secret {
        SecretEdit::Keep => Ok(()),
        SecretEdit::Replace(secret) if platform == Platform::Grok => {
            let api_key = secret.expose().trim().to_string();
            if api_key.is_empty() {
                return Err(CcrError::ValidationError("密钥不能为空".into()));
            }
            profile.auth_token = None;
            profile.platform_data.shift_remove("auth_token");
            profile.platform_data.shift_remove("env_key");
            profile
                .platform_data
                .insert("api_key".to_string(), Value::String(api_key));
            Ok(())
        }
        SecretEdit::Replace(secret) => {
            let token = secret.expose().trim().to_string();
            if token.is_empty() {
                return Err(CcrError::ValidationError("密钥不能为空".into()));
            }
            profile.auth_token = Some(Secret::new(token));
            Ok(())
        }
        SecretEdit::Clear => {
            profile.auth_token = None;
            if platform == Platform::Grok {
                profile.platform_data.shift_remove("auth_token");
                profile.platform_data.shift_remove("api_key");
                profile.platform_data.shift_remove("env_key");
            }
            Ok(())
        }
        SecretEdit::UseEnvKey(name) => {
            update_profile_field(profile, "env_key", Some(name.clone()), None, false)?;
            profile.auth_token = None;
            profile.platform_data.shift_remove("auth_token");
            profile.platform_data.shift_remove("api_key");
            Ok(())
        }
    }
}

fn enforce_grok_credential_exclusivity(profile: &mut ProfileConfig, field: &str) {
    match field {
        "api_key" => {
            profile.auth_token = None;
            profile.platform_data.shift_remove("auth_token");
            profile.platform_data.shift_remove("env_key");
        }
        "env_key" => {
            profile.auth_token = None;
            profile.platform_data.shift_remove("auth_token");
            profile.platform_data.shift_remove("api_key");
        }
        "auth_token" => {
            profile.platform_data.shift_remove("api_key");
            profile.platform_data.shift_remove("env_key");
            profile.platform_data.shift_remove("auth_token");
        }
        _ => {}
    }
}

fn warning_names(warnings: &[ProfileWarning]) -> Vec<String> {
    warnings
        .iter()
        .map(|warning| {
            match warning {
                ProfileWarning::UsageCountFailed => "usage_count_failed",
                ProfileWarning::HistoryFailed => "history_failed",
                ProfileWarning::AuthRegistryFailed => "auth_registry_failed",
                ProfileWarning::AnalyticsFailed => "analytics_failed",
                ProfileWarning::OperationRecordFailed => "operation_record_failed",
                ProfileWarning::AncillaryPending => "ancillary_pending",
                ProfileWarning::ActivationFailed => "activation_failed",
                ProfileWarning::Interrupted => "interrupted",
            }
            .to_string()
        })
        .collect()
}

fn normalize_effort_level(value: &str) -> Result<String> {
    let normalized = value.trim().to_ascii_lowercase();
    if matches!(
        normalized.as_str(),
        "low" | "medium" | "high" | "xhigh" | "max"
    ) {
        Ok(normalized)
    } else {
        Err(CcrError::ValidationError(
            "effort_level 必须为 low、medium、high、xhigh 或 max".into(),
        ))
    }
}

fn non_negative_integer_field(
    field: &str,
    value: Option<String>,
    value_json: Option<String>,
    clear: bool,
) -> Result<Option<String>> {
    if clear {
        return Ok(None);
    }
    if value_json.is_some() {
        return Err(CcrError::ValidationError(format!("{field} 需为非负整数")));
    }
    let raw = value.ok_or_else(|| CcrError::ValidationError(format!("{field} 需为非负整数")))?;
    let trimmed = raw.trim();
    if trimmed.is_empty() || !trimmed.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(CcrError::ValidationError(format!("{field} 需为非负整数")));
    }
    let number = trimmed
        .parse::<u64>()
        .map_err(|_| CcrError::ValidationError(format!("{field} 需为非负整数")))?;
    Ok(Some(number.to_string()))
}

fn update_profile_field(
    profile: &mut ProfileConfig,
    field: &str,
    value: Option<String>,
    value_json: Option<String>,
    clear: bool,
) -> Result<()> {
    match field {
        "description" => profile.description = if clear { None } else { value },
        "base_url" => profile.base_url = if clear { None } else { value },
        "auth_token" => profile.auth_token = if clear { None } else { value.map(Secret::new) },
        "api_key" => {
            if clear {
                profile.platform_data.shift_remove("api_key");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError("api_key 需要非空字符串".into()));
                }
                let api_key = value
                    .ok_or_else(|| CcrError::ValidationError("api_key 需要非空字符串".into()))?
                    .trim()
                    .to_string();
                if api_key.is_empty() {
                    return Err(CcrError::ValidationError("api_key 需要非空字符串".into()));
                }
                profile
                    .platform_data
                    .insert("api_key".to_string(), Value::String(api_key));
            }
        }
        "model" => profile.model = if clear { None } else { value },
        "small_fast_model" => profile.small_fast_model = if clear { None } else { value },
        "default_opus_model" => profile.default_opus_model = if clear { None } else { value },
        "default_sonnet_model" => profile.default_sonnet_model = if clear { None } else { value },
        "default_haiku_model" => profile.default_haiku_model = if clear { None } else { value },
        "default_fable_model" => profile.default_fable_model = if clear { None } else { value },
        "subagent_model" => profile.subagent_model = if clear { None } else { value },
        "effort_level" => {
            if clear {
                profile.effort_level = None;
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "effort_level 需要非空字符串".into(),
                    ));
                }
                let value = value.ok_or_else(|| {
                    CcrError::ValidationError("effort_level 需要非空字符串".into())
                })?;
                profile.effort_level = Some(normalize_effort_level(&value)?);
            }
        }
        "api_timeout_ms" => {
            profile.api_timeout_ms =
                non_negative_integer_field("api_timeout_ms", value, value_json, clear)?;
        }
        "claude_code_auto_compact_window" => {
            profile.claude_code_auto_compact_window = non_negative_integer_field(
                "claude_code_auto_compact_window",
                value,
                value_json,
                clear,
            )?;
        }
        "provider" => profile.provider = if clear { None } else { value },
        "provider_type" => profile.provider_type = if clear { None } else { value },
        "account" => profile.account = if clear { None } else { value },
        "auth_mode" => {
            if clear {
                profile
                    .platform_data
                    .shift_remove(crate::platforms::ClaudePlatform::AUTH_MODE_FIELD);
            } else {
                let value = value
                    .ok_or_else(|| CcrError::ValidationError("auth_mode 需要非空字符串".into()))?;
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    return Err(CcrError::ValidationError("auth_mode 不能为空字符串".into()));
                }
                profile.platform_data.insert(
                    crate::platforms::ClaudePlatform::AUTH_MODE_FIELD.to_string(),
                    Value::String(trimmed.to_string()),
                );
            }
        }
        "api_backend" => {
            if clear {
                profile.platform_data.shift_remove("api_backend");
            } else {
                let backend = value
                    .ok_or_else(|| CcrError::ValidationError("api_backend 需要非空字符串".into()))?
                    .trim()
                    .to_ascii_lowercase();
                if !matches!(
                    backend.as_str(),
                    "chat_completions" | "responses" | "messages"
                ) {
                    return Err(CcrError::ValidationError(
                        "api_backend 仅支持 chat_completions、responses 或 messages".into(),
                    ));
                }
                profile
                    .platform_data
                    .insert("api_backend".to_string(), Value::String(backend));
            }
        }
        "wire_api" => {
            if clear {
                profile.platform_data.shift_remove("wire_api");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "wire_api 仅支持 responses".into(),
                    ));
                }
                let value = value
                    .ok_or_else(|| CcrError::ValidationError("wire_api 仅支持 responses".into()))?;
                let normalized = value.trim().to_ascii_lowercase();
                if normalized != "responses" {
                    return Err(CcrError::ValidationError(
                        "wire_api 仅支持 responses".into(),
                    ));
                }
                profile
                    .platform_data
                    .insert("wire_api".to_string(), Value::String(normalized));
            }
        }
        "requires_openai_auth" => {
            if clear {
                profile.platform_data.shift_remove("requires_openai_auth");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "requires_openai_auth 仅支持 true、false、1 或 0".into(),
                    ));
                }
                let enabled = match value.as_deref().map(str::trim) {
                    Some("true" | "1") => true,
                    Some("false" | "0") => false,
                    _ => {
                        return Err(CcrError::ValidationError(
                            "requires_openai_auth 仅支持 true、false、1 或 0".into(),
                        ));
                    }
                };
                profile
                    .platform_data
                    .insert("requires_openai_auth".to_string(), Value::Bool(enabled));
            }
        }
        "env_key" => {
            if clear {
                profile.platform_data.shift_remove("env_key");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "env_key MVP 仅支持单个环境变量名，不接受数组".into(),
                    ));
                }
                let env_key = value
                    .ok_or_else(|| CcrError::ValidationError("env_key 需要非空字符串".into()))?
                    .trim()
                    .to_string();
                if env_key.is_empty() || env_key.starts_with('[') || env_key.contains(',') {
                    return Err(CcrError::ValidationError(
                        "env_key MVP 仅支持单个环境变量名，不接受数组".into(),
                    ));
                }
                profile
                    .platform_data
                    .insert("env_key".to_string(), Value::String(env_key));
            }
        }
        "context_window" => {
            if clear {
                profile.platform_data.shift_remove("context_window");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "context_window 需为正整数".into(),
                    ));
                }
                let context_window = value
                    .as_deref()
                    .and_then(|raw| raw.trim().parse::<u64>().ok())
                    .filter(|value| *value > 0)
                    .ok_or_else(|| CcrError::ValidationError("context_window 需为正整数".into()))?;
                profile.platform_data.insert(
                    "context_window".to_string(),
                    Value::Number(context_window.into()),
                );
            }
        }
        "supports_backend_search" => {
            if clear {
                profile
                    .platform_data
                    .shift_remove("supports_backend_search");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "supports_backend_search 仅支持 true、false、1 或 0".into(),
                    ));
                }
                let enabled = match value.as_deref().map(str::trim) {
                    Some("true" | "1") => true,
                    Some("false" | "0") => false,
                    _ => {
                        return Err(CcrError::ValidationError(
                            "supports_backend_search 仅支持 true、false、1 或 0".into(),
                        ));
                    }
                };
                profile
                    .platform_data
                    .insert("supports_backend_search".to_string(), Value::Bool(enabled));
            }
        }
        "reasoning_effort" => {
            if clear {
                profile.platform_data.shift_remove("reasoning_effort");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "reasoning_effort 需要非空字符串".into(),
                    ));
                }
                let value = value.ok_or_else(|| {
                    CcrError::ValidationError("reasoning_effort 需要非空字符串".into())
                })?;
                let normalized = GrokPlatform::normalize_reasoning_effort(&value)?;
                profile
                    .platform_data
                    .insert("reasoning_effort".to_string(), Value::String(normalized));
            }
        }
        "model_reasoning_effort" => {
            if clear {
                profile.platform_data.shift_remove("model_reasoning_effort");
            } else {
                if value_json.is_some() {
                    return Err(CcrError::ValidationError(
                        "model_reasoning_effort 需要非空字符串".into(),
                    ));
                }
                let value = value.ok_or_else(|| {
                    CcrError::ValidationError("model_reasoning_effort 需要非空字符串".into())
                })?;
                let normalized = value.trim().to_ascii_lowercase();
                if !matches!(
                    normalized.as_str(),
                    "minimal" | "low" | "medium" | "high" | "xhigh"
                ) {
                    return Err(CcrError::ValidationError(format!(
                        "model_reasoning_effort 必须为 minimal/low/medium/high/xhigh，当前值: {value}"
                    )));
                }
                profile.platform_data.insert(
                    "model_reasoning_effort".to_string(),
                    Value::String(normalized),
                );
            }
        }
        "enabled" => {
            if clear {
                return Err(CcrError::ValidationError(
                    "enabled 请使用启用或禁用，不能清空".into(),
                ));
            }
            let enabled = match value.as_deref().map(str::trim) {
                Some("true" | "1") => true,
                Some("false" | "0") => false,
                _ => {
                    return Err(CcrError::ValidationError(
                        "enabled 仅支持 true 或 false".into(),
                    ));
                }
            };
            profile.enabled = Some(enabled);
        }
        "tags" => {
            if clear {
                profile.tags = None;
            } else if let Some(raw_json) = value_json {
                let tags: Vec<String> = serde_json::from_str(&raw_json).map_err(|error| {
                    CcrError::ValidationError(format!(
                        "tags 的 value-json 不是合法 JSON 数组: {error}"
                    ))
                })?;
                profile.tags = normalize_tags(tags);
            } else {
                profile.tags = normalize_tags(
                    value
                        .unwrap_or_default()
                        .split(',')
                        .map(str::to_string)
                        .collect(),
                );
            }
        }
        _ => {
            return Err(CcrError::ValidationError(format!("不支持的字段 '{field}'")));
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn empty_request() -> EditProfileRequest {
        EditProfileRequest {
            platform: Platform::Claude,
            name: "work".into(),
            rename: None,
            expected_revision: None,
            fields: Vec::new(),
            secret: SecretEdit::Keep,
            apply: ApplyPolicy::IfCurrent,
        }
    }

    #[test]
    fn empty_edit_errors_before_any_catalog_access() {
        let error = edit_profile(empty_request()).unwrap_err();
        assert!(error.to_string().contains("没有要修改的字段"));
    }

    #[test]
    fn same_name_rename_is_an_empty_edit() {
        let mut request = empty_request();
        request.rename = Some("work".into());
        let error = edit_profile(request).unwrap_err();
        assert!(error.to_string().contains("没有要修改的字段"));
    }

    #[test]
    fn rejected_field_errors_before_any_catalog_access() {
        let mut request = empty_request();
        request.platform = Platform::Codex;
        request.fields.push(FieldEdit::Set {
            field: "not_a_field".into(),
            value: Value::String("x".into()),
        });
        let error = edit_profile(request).unwrap_err();
        assert!(error.to_string().contains("不允许编辑字段"));
    }

    #[test]
    fn update_profile_field_supports_tags_json() {
        let mut profile = ProfileConfig::new();
        update_profile_field(
            &mut profile,
            "tags",
            None,
            Some(r#"[" work ","team",""]"#.to_string()),
            false,
        )
        .unwrap();

        assert_eq!(
            profile.tags,
            Some(vec!["work".to_string(), "team".to_string()])
        );
    }

    #[test]
    fn update_profile_field_parses_grok_specific_values() {
        let mut profile = ProfileConfig::new();

        update_profile_field(
            &mut profile,
            "api_backend",
            Some("MESSAGES".into()),
            None,
            false,
        )
        .unwrap();
        update_profile_field(
            &mut profile,
            "api_key",
            Some("INLINE_SECRET_SENTINEL".into()),
            None,
            false,
        )
        .unwrap();
        update_profile_field(
            &mut profile,
            "env_key",
            Some("GROK_RELAY_KEY".into()),
            None,
            false,
        )
        .unwrap();
        update_profile_field(
            &mut profile,
            "context_window",
            Some("1000000".into()),
            None,
            false,
        )
        .unwrap();
        update_profile_field(
            &mut profile,
            "supports_backend_search",
            Some("1".into()),
            None,
            false,
        )
        .unwrap();
        update_profile_field(
            &mut profile,
            "reasoning_effort",
            Some(" HIGH ".into()),
            None,
            false,
        )
        .unwrap();

        assert_eq!(profile.platform_data["api_backend"], "messages");
        assert_eq!(profile.platform_data["api_key"], "INLINE_SECRET_SENTINEL");
        assert_eq!(profile.platform_data["env_key"], "GROK_RELAY_KEY");
        assert_eq!(profile.platform_data["context_window"], 1_000_000);
        assert_eq!(profile.platform_data["supports_backend_search"], true);
        assert_eq!(profile.platform_data["reasoning_effort"], "high");

        for field in [
            "api_backend",
            "api_key",
            "env_key",
            "context_window",
            "supports_backend_search",
            "reasoning_effort",
        ] {
            update_profile_field(&mut profile, field, None, None, true).unwrap();
            assert!(!profile.platform_data.contains_key(field));
        }
    }

    #[test]
    fn update_profile_field_rejects_invalid_grok_specific_values() {
        let invalid = [
            ("api_backend", Some("legacy".to_string()), None),
            ("api_key", Some("  ".to_string()), None),
            ("context_window", Some("0".to_string()), None),
            ("supports_backend_search", Some("maybe".to_string()), None),
            ("env_key", None, Some(r#"["A","B"]"#.to_string())),
            ("reasoning_effort", Some("  ".to_string()), None),
            (
                "reasoning_effort",
                Some("model-specific-option".to_string()),
                None,
            ),
            ("reasoning_effort", None, Some(r#""high""#.to_string())),
        ];

        for (field, value, value_json) in invalid {
            let error =
                update_profile_field(&mut ProfileConfig::new(), field, value, value_json, false)
                    .unwrap_err();
            assert!(matches!(error, CcrError::ValidationError(_)));
        }
    }

    #[test]
    fn set_field_allow_list_rejects_codex_base_url() {
        let error = ensure_set_field_allowed(Platform::Codex, "base_url").unwrap_err();
        assert!(error.to_string().contains("不允许编辑字段"));
        assert!(edit_field_allowed(Platform::Codex, "base_url"));
    }

    #[test]
    fn secret_debug_redacts_replacement_value() {
        let secret = SecretEdit::Replace(Secret::new("synthetic-profile-edit-secret-sentinel"));
        let rendered = format!("{secret:?}");
        assert!(!rendered.contains("synthetic-profile-edit-secret-sentinel"));
    }
}
