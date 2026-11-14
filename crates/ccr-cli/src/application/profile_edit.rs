//! Shared profile edit used by the CLI, `set-field`, and in-process editors.
//!
//! The use case does not print, read stdin, or spawn a worker. Callers supply
//! already collected field and secret changes and run it on one synchronous worker.

pub(crate) use crate::application::profile_fields::ensure_set_field_allowed;
use crate::application::profile_fields::{
    apply_json_value, edit_field_allowed, enforce_grok_credential_exclusivity, is_grok_credential,
    push_unique, update_profile_field_for_platform,
};
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

fn apply_edits(
    platform: Platform,
    profile: &mut ProfileConfig,
    fields: &[FieldEdit],
    secret: &SecretEdit,
) -> Result<()> {
    let mut codex_env_key_set = false;
    for edit in fields {
        let clearing = edit.is_clear();
        apply_field(platform, profile, edit)?;
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

fn apply_field(platform: Platform, profile: &mut ProfileConfig, edit: &FieldEdit) -> Result<()> {
    match edit {
        FieldEdit::Clear { field } => {
            update_profile_field_for_platform(Some(platform), profile, field, None, None, true)
        }
        FieldEdit::Legacy {
            field,
            value,
            value_json,
            clear,
        } => update_profile_field_for_platform(
            Some(platform),
            profile,
            field,
            value.clone(),
            value_json.clone(),
            *clear,
        ),
        FieldEdit::Set { field, value } => apply_json_value(Some(platform), profile, field, value),
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
            update_profile_field_for_platform(
                Some(platform),
                profile,
                "env_key",
                Some(name.clone()),
                None,
                false,
            )?;
            profile.auth_token = None;
            profile.platform_data.shift_remove("auth_token");
            profile.platform_data.shift_remove("api_key");
            Ok(())
        }
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::application::profile_fields::update_profile_field;

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
