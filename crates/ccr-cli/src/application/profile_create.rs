//! Shared profile creation used by the CLI and in-process callers.
//!
//! The use case does not print, read stdin, or spawn background threads.
//! Callers supply already collected field values and credentials, and the use
//! case delegates catalog/secret/journal persistence to `ProfileLifecycleContext`.

use crate::application::profile_fields::{
    apply_json_value, create_field_allowed, is_secret_field, push_unique,
    update_profile_field_for_platform,
};
use crate::application::profile_lifecycle::ProfileLifecycleContext;
use crate::models::{Platform, ProfileConfig};
use ccr_core::Secret;
use ccr_core::core::error::{CcrError, Result};
use serde_json::Value;

/// One non-secret field assignment for profile creation. Values stay out of `Debug`.
#[derive(Clone)]
pub struct ProfileFieldValue {
    pub field: String,
    pub value: Value,
}

impl std::fmt::Debug for ProfileFieldValue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProfileFieldValue")
            .field("field", &self.field)
            .field("value", &"[REDACTED]")
            .finish()
    }
}

/// Credential channel for profile creation. `Debug` redacts all values.
#[derive(Clone)]
pub enum CreateSecret {
    None,
    Secret(Secret),
    /// Grok third-party env-var mode ONLY (stores env var name, mutually exclusive with api_key).
    UseEnvKey(String),
}

impl std::fmt::Debug for CreateSecret {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => formatter.write_str("None"),
            Self::Secret(secret) => formatter.debug_tuple("Secret").field(secret).finish(),
            Self::UseEnvKey(_) => formatter
                .debug_tuple("UseEnvKey")
                .field(&"[REDACTED]")
                .finish(),
        }
    }
}

pub struct CreateProfileRequest {
    pub platform: Platform,
    pub name: String,
    pub expected_revision: Option<String>,
    pub fields: Vec<ProfileFieldValue>,
    pub secret: CreateSecret,
}

impl std::fmt::Debug for CreateProfileRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CreateProfileRequest")
            .field("platform", &self.platform)
            .field("name", &self.name)
            .field("expected_revision", &self.expected_revision)
            .field(
                "fields",
                &self
                    .fields
                    .iter()
                    .map(|entry| entry.field.as_str())
                    .collect::<Vec<_>>(),
            )
            .field("secret", &self.secret)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateProfileOutcome {
    pub platform: Platform,
    pub name: String,
    pub enabled: bool,
    pub current_profile: Option<String>,
    pub revision: String,
    pub set_fields: Vec<String>,
}

pub fn create_profile(request: CreateProfileRequest) -> Result<CreateProfileOutcome> {
    ensure_platform_supported(request.platform)?;
    let context = ProfileLifecycleContext::new(request.platform)?;
    create_profile_with_context(&context, request)
}

pub fn create_profile_with_context(
    context: &ProfileLifecycleContext,
    request: CreateProfileRequest,
) -> Result<CreateProfileOutcome> {
    ensure_platform_supported(request.platform)?;
    if context.platform() != request.platform {
        return Err(CcrError::ValidationError(
            "Profile request belongs to another platform".into(),
        ));
    }

    let name = request.name.trim();
    if name.is_empty() {
        return Err(CcrError::ValidationError("Profile 名称不能为空".into()));
    }
    ccr_config::managers::config::repository::validate_profile_name(name)?;

    let (profile, set_fields) = build_profile(&request)?;
    let enabled = profile.is_enabled();
    let (revision, current_profile) =
        context.create_with_outcome(name, profile, request.expected_revision.as_deref())?;

    Ok(CreateProfileOutcome {
        platform: request.platform,
        name: name.to_string(),
        enabled,
        current_profile,
        revision,
        set_fields,
    })
}

fn ensure_platform_supported(platform: Platform) -> Result<()> {
    if matches!(
        platform,
        Platform::Claude | Platform::Codex | Platform::Grok
    ) {
        Ok(())
    } else {
        Err(CcrError::PlatformNotSupported(format!(
            "{platform} 目前不支持 profile create"
        )))
    }
}

fn is_optional_text_field(field: &str) -> bool {
    matches!(
        field,
        "description"
            | "base_url"
            | "model"
            | "small_fast_model"
            | "default_opus_model"
            | "default_sonnet_model"
            | "default_haiku_model"
            | "default_fable_model"
            | "subagent_model"
            | "provider"
            | "account"
    )
}

fn profile_field_is_set(profile: &ProfileConfig, field: &str) -> bool {
    match field {
        "description" => profile.description.is_some(),
        "base_url" => profile.base_url.is_some(),
        "model" => profile.model.is_some(),
        "small_fast_model" => profile.small_fast_model.is_some(),
        "default_opus_model" => profile.default_opus_model.is_some(),
        "default_sonnet_model" => profile.default_sonnet_model.is_some(),
        "default_haiku_model" => profile.default_haiku_model.is_some(),
        "default_fable_model" => profile.default_fable_model.is_some(),
        "subagent_model" => profile.subagent_model.is_some(),
        "effort_level" => profile.effort_level.is_some(),
        "api_timeout_ms" => profile.api_timeout_ms.is_some(),
        "claude_code_auto_compact_window" => profile.claude_code_auto_compact_window.is_some(),
        "provider" => profile.provider.is_some(),
        "provider_type" => profile.provider_type.is_some(),
        "account" => profile.account.is_some(),
        "tags" => profile.tags.as_ref().is_some_and(|tags| !tags.is_empty()),
        "enabled" => profile.enabled.is_some(),
        other => profile
            .platform_data
            .get(other)
            .is_some_and(|v| !v.is_null()),
    }
}

fn build_profile(request: &CreateProfileRequest) -> Result<(ProfileConfig, Vec<String>)> {
    if matches!(request.secret, CreateSecret::UseEnvKey(_)) && request.platform != Platform::Grok {
        return Err(CcrError::ValidationError(
            "仅 Grok 支持 env_key 凭据通道；Codex 请通过字段 env_key 传入".into(),
        ));
    }

    let mut profile = ProfileConfig::new();
    profile.enabled = Some(true);

    let mut set_fields = Vec::new();
    let mut seen_fields = Vec::new();

    for entry in &request.fields {
        let field = entry.field.trim();
        if field.is_empty() {
            return Err(CcrError::ValidationError("字段名不能为空".into()));
        }
        if is_secret_field(field) {
            return Err(CcrError::ValidationError(
                "密钥请通过专用凭据通道传入，不要放入普通字段".into(),
            ));
        }
        if request.platform == Platform::Grok && field == "env_key" {
            return Err(CcrError::ValidationError(
                "Grok 创建 Profile 请通过 CreateSecret::UseEnvKey（CLI --env-key）传入 env_key，不要放入普通字段"
                    .into(),
            ));
        }
        if seen_fields
            .iter()
            .any(|existing: &String| existing == field)
        {
            return Err(CcrError::ValidationError(format!("字段 '{field}' 重复")));
        }
        if !create_field_allowed(request.platform, field) {
            return Err(CcrError::ValidationError(format!(
                "平台 '{}' 不允许创建字段 '{field}'",
                request.platform
            )));
        }
        if entry.value.is_null() {
            return Err(CcrError::ValidationError(format!(
                "创建字段 '{field}' 的值不能为 null"
            )));
        }
        seen_fields.push(field.to_string());
        if is_optional_text_field(field)
            && entry
                .value
                .as_str()
                .is_some_and(|text| text.trim().is_empty())
        {
            continue;
        }
        apply_json_value(Some(request.platform), &mut profile, field, &entry.value)?;
        if profile_field_is_set(&profile, field) {
            push_unique(&mut set_fields, field);
        }
    }

    match &request.secret {
        CreateSecret::None => {}
        CreateSecret::Secret(secret) => {
            let trimmed = secret.expose().trim().to_string();
            if trimmed.is_empty() {
                return Err(CcrError::ValidationError("密钥不能为空".into()));
            }
            if request.platform == Platform::Grok {
                profile
                    .platform_data
                    .insert("api_key".to_string(), Value::String(trimmed));
                push_unique(&mut set_fields, "api_key");
            } else {
                profile.auth_token = Some(Secret::new(trimmed));
                push_unique(&mut set_fields, "auth_token");
            }
        }
        CreateSecret::UseEnvKey(env_name) => {
            update_profile_field_for_platform(
                Some(Platform::Grok),
                &mut profile,
                "env_key",
                Some(env_name.clone()),
                None,
                false,
            )?;
            push_unique(&mut set_fields, "env_key");
        }
    }

    Ok((profile, set_fields))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::PlatformPaths;
    use serde_json::json;
    use std::path::Path;

    const SENTINEL: &str = "sk-create-secret-sentinel-987654321";

    fn test_context(root: &Path, platform: Platform) -> ProfileLifecycleContext {
        let ccr = root.join("ccr");
        let platform_dir = ccr.join("platforms").join(platform.short_name());
        let paths = PlatformPaths {
            root: ccr.clone(),
            registry_file: ccr.join("config.toml"),
            profiles_file: platform_dir.join("profiles.toml"),
            settings_file: platform_dir.join("settings.json"),
            history_file: ccr.join("history").join(format!("{platform}.json")),
            backups_dir: platform_dir.join("backups"),
            platform_dir,
        };
        let locks = ccr.join(".locks");
        let history = ccr.join("history.db");
        let runtime = root.join("runtime").join(platform.short_name());
        match platform {
            Platform::Claude => ProfileLifecycleContext::for_claude(
                paths,
                ccr_config::ClaudeRuntimePaths {
                    settings_file: runtime.join("settings.json"),
                    credentials_file: runtime.join(".credentials.json"),
                    state_file: root.join("claude-state.json"),
                    backups_dir: runtime.join("backups"),
                    config_dir: runtime,
                },
                locks,
                history,
            )
            .unwrap(),
            Platform::Codex => {
                ProfileLifecycleContext::for_codex(paths, runtime, locks, history).unwrap()
            }
            Platform::Grok => ProfileLifecycleContext::for_grok(
                paths,
                runtime.join("config.toml"),
                locks,
                history,
            )
            .unwrap(),
            other => panic!("unexpected platform {other:?}"),
        }
    }

    fn base_fields_for(platform: Platform) -> Vec<ProfileFieldValue> {
        let mut fields = vec![
            ProfileFieldValue {
                field: "base_url".into(),
                value: json!("https://relay.example.com/v1"),
            },
            ProfileFieldValue {
                field: "model".into(),
                value: json!("fixture-model"),
            },
        ];
        if platform == Platform::Codex {
            fields.push(ProfileFieldValue {
                field: "auth_mode".into(),
                value: json!("openai_api_key"),
            });
        }
        fields
    }

    #[test]
    fn debug_redacts_field_values_and_secrets() {
        let request = CreateProfileRequest {
            platform: Platform::Claude,
            name: "work".into(),
            expected_revision: Some("rev-1".into()),
            fields: vec![ProfileFieldValue {
                field: "base_url".into(),
                value: json!(SENTINEL),
            }],
            secret: CreateSecret::Secret(Secret::new(SENTINEL)),
        };
        let rendered = format!("{request:?} {:?} {:?}", request.fields[0], request.secret);
        assert!(!rendered.contains(SENTINEL), "{rendered}");

        let env_secret = CreateSecret::UseEnvKey(SENTINEL.into());
        assert!(!format!("{env_secret:?}").contains(SENTINEL));
    }

    #[test]
    fn create_profile_first_and_second_across_three_platforms_keep_first_inactive() {
        for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
            let temp = tempfile::tempdir().unwrap();
            let context = test_context(temp.path(), platform);

            let first = create_profile_with_context(
                &context,
                CreateProfileRequest {
                    platform,
                    name: "alpha".into(),
                    expected_revision: Some(String::new()),
                    fields: base_fields_for(platform),
                    secret: CreateSecret::Secret(Secret::new(SENTINEL)),
                },
            )
            .unwrap();

            assert_eq!(first.platform, platform);
            assert_eq!(first.name, "alpha");
            assert!(first.enabled);
            assert_eq!(first.current_profile, None);
            assert!(!first.revision.is_empty());
            assert!(context.manager().load().unwrap().current_config.is_empty());
            assert_eq!(context.manager().load().unwrap().default_config, "alpha");
            assert!(!context.instance().get_settings_path().exists());

            // Duplicate profile name returns ResourceAlreadyExists (exit code 64).
            let dup_err = create_profile_with_context(
                &context,
                CreateProfileRequest {
                    platform,
                    name: "alpha".into(),
                    expected_revision: Some(first.revision.clone()),
                    fields: base_fields_for(platform),
                    secret: CreateSecret::Secret(Secret::new(SENTINEL)),
                },
            )
            .unwrap_err();
            assert!(matches!(dup_err, CcrError::ResourceAlreadyExists(_)));
            assert_eq!(dup_err.exit_code(), 64);

            // Second creation with expected_revision advances revision and keeps inactive.
            let second = create_profile_with_context(
                &context,
                CreateProfileRequest {
                    platform,
                    name: "beta".into(),
                    expected_revision: Some(first.revision.clone()),
                    fields: base_fields_for(platform),
                    secret: CreateSecret::Secret(Secret::new(SENTINEL)),
                },
            )
            .unwrap();
            assert_ne!(second.revision, first.revision);
            assert_eq!(second.current_profile, None);
            assert_eq!(context.instance().load_profiles().unwrap().len(), 2);
        }
    }

    #[test]
    fn codex_cas_mismatch_rejects_without_writing_profile_secrets() {
        let temp = tempfile::tempdir().unwrap();
        let context = test_context(temp.path(), Platform::Codex);
        let err = create_profile_with_context(
            &context,
            CreateProfileRequest {
                platform: Platform::Codex,
                name: "team".into(),
                expected_revision: Some("stale-revision".into()),
                fields: base_fields_for(Platform::Codex),
                secret: CreateSecret::Secret(Secret::new(SENTINEL)),
            },
        )
        .unwrap_err();
        assert!(matches!(err, CcrError::ValidationError(_)));
        assert!(!context.manager().profiles_db().exists());
        assert!(
            !temp
                .path()
                .join("ccr/platforms/codex/profile_secrets.json")
                .exists()
        );
    }

    #[test]
    fn grok_use_env_key_vs_codex_field_env_key_contract() {
        let temp = tempfile::tempdir().unwrap();

        // UseEnvKey is rejected on Claude and Codex.
        for platform in [Platform::Claude, Platform::Codex] {
            let context = test_context(&temp.path().join(platform.short_name()), platform);
            let err = create_profile_with_context(
                &context,
                CreateProfileRequest {
                    platform,
                    name: "env-wrong".into(),
                    expected_revision: None,
                    fields: base_fields_for(platform),
                    secret: CreateSecret::UseEnvKey("OPENAI_KEY".into()),
                },
            )
            .unwrap_err();
            assert!(matches!(err, CcrError::ValidationError(_)));
        }

        // Grok accepts UseEnvKey and stores env_key.
        let grok_ctx = test_context(&temp.path().join("grok"), Platform::Grok);
        let grok_out = create_profile_with_context(
            &grok_ctx,
            CreateProfileRequest {
                platform: Platform::Grok,
                name: "relay".into(),
                expected_revision: None,
                fields: vec![
                    ProfileFieldValue {
                        field: "base_url".into(),
                        value: json!("https://grok-relay.example.com/v1"),
                    },
                    ProfileFieldValue {
                        field: "model".into(),
                        value: json!("grok-4"),
                    },
                ],
                secret: CreateSecret::UseEnvKey("XAI_RELAY_KEY".into()),
            },
        )
        .unwrap();
        assert!(grok_out.set_fields.contains(&"env_key".to_string()));
        let grok_profiles = grok_ctx.instance().load_profiles().unwrap();
        assert_eq!(
            grok_profiles["relay"].platform_data.get("env_key"),
            Some(&json!("XAI_RELAY_KEY"))
        );

        // Grok rejects env_key inside regular fields (must use CreateSecret::UseEnvKey).
        let grok_field_err = create_profile_with_context(
            &grok_ctx,
            CreateProfileRequest {
                platform: Platform::Grok,
                name: "relay-bad-field".into(),
                expected_revision: None,
                fields: vec![
                    ProfileFieldValue {
                        field: "base_url".into(),
                        value: json!("https://grok-relay.example.com/v1"),
                    },
                    ProfileFieldValue {
                        field: "model".into(),
                        value: json!("grok-4"),
                    },
                    ProfileFieldValue {
                        field: "env_key".into(),
                        value: json!(SENTINEL),
                    },
                ],
                secret: CreateSecret::None,
            },
        )
        .unwrap_err();
        let grok_field_msg = grok_field_err.to_string();
        assert!(matches!(grok_field_err, CcrError::ValidationError(_)));
        assert!(grok_field_msg.contains("UseEnvKey"), "{grok_field_msg}");
        assert!(!grok_field_msg.contains(SENTINEL), "{grok_field_msg}");

        // Codex accepts env_key in fields alongside CreateSecret::Secret, persisting secret sidecar
        // while keeping catalog auth_token scrubbed.
        let codex_ctx = test_context(&temp.path().join("codex-env"), Platform::Codex);
        let codex_out = create_profile_with_context(
            &codex_ctx,
            CreateProfileRequest {
                platform: Platform::Codex,
                name: "provider-env".into(),
                expected_revision: None,
                fields: vec![
                    ProfileFieldValue {
                        field: "base_url".into(),
                        value: json!("https://codex-relay.example.com/v1"),
                    },
                    ProfileFieldValue {
                        field: "model".into(),
                        value: json!("gpt-5"),
                    },
                    ProfileFieldValue {
                        field: "auth_mode".into(),
                        value: json!("provider_env_key"),
                    },
                    ProfileFieldValue {
                        field: "env_key".into(),
                        value: json!("RELAY_API_KEY"),
                    },
                ],
                secret: CreateSecret::Secret(Secret::new(SENTINEL)),
            },
        )
        .unwrap();
        assert!(codex_out.set_fields.contains(&"env_key".to_string()));
        assert!(codex_out.set_fields.contains(&"auth_token".to_string()));
        let secrets_path = temp
            .path()
            .join("codex-env/ccr/platforms/codex/profile_secrets.json");
        assert!(secrets_path.is_file());
        let catalog = codex_ctx.manager().load().unwrap();
        assert!(catalog.sections["provider-env"].auth_token.is_none());
    }

    #[test]
    fn validation_errors_never_echo_sentinel_values() {
        let temp = tempfile::tempdir().unwrap();
        for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
            let context = test_context(&temp.path().join(platform.short_name()), platform);

            // Secret field inside regular fields is rejected with fixed error.
            for secret_field in ["auth_token", "api_key"] {
                let err = create_profile_with_context(
                    &context,
                    CreateProfileRequest {
                        platform,
                        name: "bad".into(),
                        expected_revision: None,
                        fields: vec![ProfileFieldValue {
                            field: secret_field.into(),
                            value: json!(SENTINEL),
                        }],
                        secret: CreateSecret::None,
                    },
                )
                .unwrap_err();
                let msg = err.to_string();
                assert!(msg.contains("专用凭据通道"), "{msg}");
                assert!(!msg.contains(SENTINEL), "{msg}");
            }

            // Invalid enum / URL / integer values never echo the sentinel value.
            let candidate_fields: &[&str] = match platform {
                Platform::Claude => &[
                    "base_url",
                    "provider_type",
                    "auth_mode",
                    "effort_level",
                    "api_timeout_ms",
                    "enabled",
                ],
                Platform::Codex => &[
                    "base_url",
                    "provider_type",
                    "auth_mode",
                    "wire_api",
                    "model_reasoning_effort",
                    "requires_openai_auth",
                    "enabled",
                ],
                Platform::Grok => &[
                    "base_url",
                    "provider_type",
                    "api_backend",
                    "reasoning_effort",
                    "context_window",
                    "supports_backend_search",
                    "enabled",
                ],
                _ => unreachable!(),
            };
            for field in candidate_fields {
                let err = create_profile_with_context(
                    &context,
                    CreateProfileRequest {
                        platform,
                        name: "bad-val".into(),
                        expected_revision: None,
                        fields: vec![ProfileFieldValue {
                            field: (*field).into(),
                            value: json!(SENTINEL),
                        }],
                        secret: CreateSecret::Secret(Secret::new("sk-valid-placeholder")),
                    },
                )
                .unwrap_err();
                let msg = err.to_string();
                assert!(
                    !msg.contains(SENTINEL),
                    "platform={platform} field={field} leaked sentinel in error: {msg}"
                );
            }
        }
    }

    #[test]
    fn rejects_null_values_and_records_only_actually_set_fields() {
        let temp = tempfile::tempdir().unwrap();
        for platform in [Platform::Claude, Platform::Codex, Platform::Grok] {
            let context = test_context(&temp.path().join(platform.short_name()), platform);
            let mut fields = base_fields_for(platform);
            fields.push(ProfileFieldValue {
                field: "description".into(),
                value: Value::Null,
            });
            let err = create_profile_with_context(
                &context,
                CreateProfileRequest {
                    platform,
                    name: "null-field".into(),
                    expected_revision: None,
                    fields,
                    secret: CreateSecret::Secret(Secret::new(SENTINEL)),
                },
            )
            .unwrap_err();
            assert!(matches!(err, CcrError::ValidationError(_)));
            assert!(err.to_string().contains("null"));
        }

        // Empty/whitespace optional text and empty tags array are ignored and omitted from set_fields.
        let claude_ctx = test_context(&temp.path().join("claude-set-fields"), Platform::Claude);
        let outcome = create_profile_with_context(
            &claude_ctx,
            CreateProfileRequest {
                platform: Platform::Claude,
                name: "clean-fields".into(),
                expected_revision: None,
                fields: vec![
                    ProfileFieldValue {
                        field: "base_url".into(),
                        value: json!("https://relay.example.com"),
                    },
                    ProfileFieldValue {
                        field: "model".into(),
                        value: json!("claude-sonnet-4-20250514"),
                    },
                    ProfileFieldValue {
                        field: "description".into(),
                        value: json!("   "),
                    },
                    ProfileFieldValue {
                        field: "tags".into(),
                        value: json!(["  ", ""]),
                    },
                ],
                secret: CreateSecret::Secret(Secret::new(SENTINEL)),
            },
        )
        .unwrap();
        assert_eq!(
            outcome.set_fields,
            vec![
                "base_url".to_string(),
                "model".to_string(),
                "auth_token".to_string()
            ]
        );
        let stored = claude_ctx.instance().load_profiles().unwrap();
        assert!(stored["clean-fields"].description.is_none());
        assert!(stored["clean-fields"].tags.is_none());
    }
}
