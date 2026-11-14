//! Shared read-only `profile export` application service for Claude, Codex, and Grok.
//!
//! Guarantees zero disk/lock/migration side effects during read & serialization,
//! overlays Codex `profile_secrets.json` in memory, materializes explicit `auth_mode`
//! before `--no-secrets` redaction, and produces `ExchangeV1` TOML + JSON metadata.

use crate::application::profile_lifecycle::ProfileLifecycleContext;
use crate::models::{Platform, PlatformConfig, ProfileConfig};
use crate::platforms::{ClaudePlatform, GrokPlatform};
use ccr_codex::CodexPlatform;
use ccr_codex::models::{CodexProfileAuthMode, CodexProfileSecretStore};
use ccr_config::managers::ConfigSection;
use ccr_config::managers::config_validator::ConfigValidator;
use ccr_config::platforms::base::section_to_profile;
use ccr_config::{
    EXCHANGE_SCHEMA_VERSION, ExchangeSecretsMode, render_exchange_v1, scrub_section_secrets,
    section_contains_inline_secrets,
};
use ccr_core::Secret;
use ccr_core::core::error::{CcrError, Result};
use chrono::Utc;
use indexmap::IndexMap;
use serde::Serialize;

const CODEX_SAFE_EXTENSION_KEYS: &[&str] = &[
    "auth_mode",
    "openai_login_method",
    "forced_login_method",
    "preferred_auth_method",
    "requires_openai_auth",
    "env_key",
    "wire_api",
    "api_protocol",
    "model_reasoning_effort",
    "network_access",
    "cli_auth_credentials_store",
    "disable_response_storage",
    "approval_policy",
    "sandbox_mode",
    "model_catalog_json",
];

#[derive(Debug, Clone)]
pub struct ExportProfilesRequest {
    pub platform: Platform,
    pub names: Vec<String>,
    pub tags: Vec<String>,
    pub include_secrets: bool,
}

#[derive(Clone)]
pub struct ExportProfilesOutcome {
    pub platform: Platform,
    pub schema_version: i64,
    pub secrets_mode: ExchangeSecretsMode,
    pub contains_secrets: bool,
    pub exported_names: Vec<String>,
    pub warnings: Vec<String>,
    pub toml_content: String,
}

impl std::fmt::Debug for ExportProfilesOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExportProfilesOutcome")
            .field("platform", &self.platform)
            .field("schema_version", &self.schema_version)
            .field("secrets_mode", &self.secrets_mode)
            .field("contains_secrets", &self.contains_secrets)
            .field("exported_names", &self.exported_names)
            .field("warnings", &self.warnings)
            .field("toml_content", &"<redacted>")
            .finish()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportProfilesJsonReport {
    pub ok: bool,
    pub platform: String,
    pub output_path: String,
    pub schema_version: i64,
    pub secrets_mode: ExchangeSecretsMode,
    pub contains_secrets: bool,
    pub exported_count: usize,
    pub exported_names: Vec<String>,
    pub warnings: Vec<String>,
}

impl ExportProfilesOutcome {
    pub fn to_json_report(&self, output_path: impl Into<String>) -> ExportProfilesJsonReport {
        ExportProfilesJsonReport {
            ok: true,
            platform: self.platform.short_name().to_string(),
            output_path: output_path.into(),
            schema_version: self.schema_version,
            secrets_mode: self.secrets_mode,
            contains_secrets: self.contains_secrets,
            exported_count: self.exported_names.len(),
            exported_names: self.exported_names.clone(),
            warnings: self.warnings.clone(),
        }
    }
}

pub fn export_profiles(request: ExportProfilesRequest) -> Result<ExportProfilesOutcome> {
    ensure_supported_platform(request.platform)?;
    let context = ProfileLifecycleContext::new(request.platform)?;
    export_profiles_with_context(&context, request)
}

pub fn export_profiles_with_context(
    context: &ProfileLifecycleContext,
    request: ExportProfilesRequest,
) -> Result<ExportProfilesOutcome> {
    ensure_supported_platform(request.platform)?;
    if context.platform() != request.platform {
        return Err(CcrError::ValidationError(
            "导出请求平台与上下文平台不一致".into(),
        ));
    }
    if !request.names.is_empty() && !request.tags.is_empty() {
        return Err(CcrError::ValidationError(
            "位置参数 [NAME]... 与 --tag 不能同时使用".into(),
        ));
    }

    // Pure read: never acquires locks, never initializes profiles.db, never migrates legacy TOML.
    let catalog = context.manager().load()?;
    let selected = select_sections(&catalog.sections, &request.names, &request.tags)?;

    let codex_secrets = if request.platform == Platform::Codex {
        Some(
            context
                .codex_platform()?
                .runtime_service()
                .load_secret_store()?,
        )
    } else {
        None
    };

    let whitelist = platform_safe_extension_keys(request.platform);
    let secrets_mode = if request.include_secrets {
        ExchangeSecretsMode::Included
    } else {
        ExchangeSecretsMode::Redacted
    };

    let mut warnings = Vec::new();
    let mut prepared = IndexMap::with_capacity(selected.len());

    for (name, mut section) in selected {
        section.usage_count = None;
        overlay_codex_secret_section(&name, &mut section, codex_secrets.as_ref());

        // Materialize explicit auth_mode BEFORE `--no-secrets` deletes `auth_token` so
        // receivers never degrade auth mode when inline tokens are stripped.
        materialize_explicit_auth_mode(request.platform, &mut section);

        if secrets_mode == ExchangeSecretsMode::Redacted {
            scrub_section_secrets(
                &mut section,
                &format!("profiles.{name}"),
                whitelist,
                &mut warnings,
            );
        }

        prepared.insert(name, section);
    }

    let contains_secrets = secrets_mode == ExchangeSecretsMode::Included
        && prepared
            .values()
            .any(|section| section_contains_inline_secrets(section, whitelist));

    if contains_secrets {
        warnings.insert(
            0,
            "导出内容包含明文密钥，请勿提交到版本库；如需安全分享请追加 `--no-secrets`".to_string(),
        );
    }

    let exported_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let toml_content = render_exchange_v1(
        request.platform.short_name(),
        &exported_at,
        secrets_mode,
        &prepared,
    )?;
    let exported_names = prepared.keys().cloned().collect();

    Ok(ExportProfilesOutcome {
        platform: request.platform,
        schema_version: EXCHANGE_SCHEMA_VERSION,
        secrets_mode,
        contains_secrets,
        exported_names,
        warnings,
        toml_content,
    })
}

pub(crate) fn ensure_supported_platform(platform: Platform) -> Result<()> {
    if matches!(
        platform,
        Platform::Claude | Platform::Codex | Platform::Grok
    ) {
        Ok(())
    } else {
        Err(CcrError::PlatformNotSupported(format!(
            "平台 '{platform}' 不支持 profile export/import"
        )))
    }
}

pub(crate) fn platform_safe_extension_keys(platform: Platform) -> &'static [&'static str] {
    match platform {
        Platform::Claude => ClaudePlatform::editable_fields(),
        Platform::Codex => CODEX_SAFE_EXTENSION_KEYS,
        Platform::Grok => GrokPlatform::editable_fields(),
        _ => &[],
    }
}

fn materialize_explicit_auth_mode(platform: Platform, section: &mut ConfigSection) {
    match platform {
        Platform::Claude => {
            let profile = section_to_profile(section);
            let mode = ClaudePlatform::profile_auth_mode(&profile);
            section.other.insert(
                "auth_mode".into(),
                toml::Value::String(mode.as_str().to_string()),
            );
        }
        Platform::Codex => {
            let profile = section_to_profile(section);
            let mode = CodexPlatform::resolve_auth_mode(&profile);
            section.other.insert(
                "auth_mode".into(),
                toml::Value::String(mode.as_str().to_string()),
            );
        }
        _ => {}
    }
}

fn select_sections(
    sections: &IndexMap<String, ConfigSection>,
    names: &[String],
    tags: &[String],
) -> Result<IndexMap<String, ConfigSection>> {
    if !names.is_empty() {
        let mut selected = IndexMap::new();
        for raw_name in names {
            let name = raw_name.trim();
            if name.is_empty() {
                return Err(CcrError::ValidationError("Profile 名称不能为空".into()));
            }
            if selected.contains_key(name) {
                continue;
            }
            let Some(section) = sections.get(name) else {
                return Err(CcrError::ProfileNotFound(name.to_string()));
            };
            selected.insert(name.to_string(), section.clone());
        }
        if selected.is_empty() {
            return Err(CcrError::ValidationError(
                "未匹配到任何可导出的 Profile".into(),
            ));
        }
        return Ok(selected);
    }

    if !tags.is_empty() {
        let normalized_tags: Vec<&str> = tags
            .iter()
            .map(String::as_str)
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .collect();
        if normalized_tags.is_empty() {
            return Err(CcrError::ValidationError("过滤标签不能为空".into()));
        }
        let mut selected = IndexMap::new();
        for (name, section) in sections {
            let matches_tag = section.tags.as_ref().is_some_and(|profile_tags| {
                profile_tags
                    .iter()
                    .any(|pt| normalized_tags.contains(&pt.trim()))
            });
            if matches_tag {
                selected.insert(name.clone(), section.clone());
            }
        }
        if selected.is_empty() {
            return Err(CcrError::ValidationError(format!(
                "没有匹配标签 [{}] 的 Profile",
                normalized_tags.join(", ")
            )));
        }
        return Ok(selected);
    }

    if sections.is_empty() {
        return Err(CcrError::ValidationError(
            "当前平台目录为空，没有可导出的 Profile".into(),
        ));
    }

    Ok(sections.clone())
}

pub(crate) const fn platform_secret_stdin_flag(platform: Platform) -> &'static str {
    match platform {
        Platform::Grok => "--api-key-stdin",
        _ => "--auth-token-stdin",
    }
}

pub(crate) fn overlay_codex_secret_section(
    name: &str,
    section: &mut ConfigSection,
    store: Option<&CodexProfileSecretStore>,
) {
    let Some(secret_entry) = store.and_then(|s| s.profiles.get(name)) else {
        return;
    };
    if matches!(
        secret_entry.auth_mode,
        CodexProfileAuthMode::OpenAiApiKey
            | CodexProfileAuthMode::ProviderEnvKey
            | CodexProfileAuthMode::ProviderBearerToken
    ) && section.auth_token.is_none()
    {
        section.auth_token = Some(Secret::new(secret_entry.secret.clone()));
    }
    if let Some(env_key) = &secret_entry.env_key
        && !section.other.contains_key("env_key")
    {
        section
            .other
            .insert("env_key".into(), toml::Value::String(env_key.clone()));
    }
}

pub(crate) fn overlay_codex_secret_if_present(
    name: &str,
    section: &ConfigSection,
    store: Option<&CodexProfileSecretStore>,
) -> ConfigSection {
    let mut cloned = section.clone();
    overlay_codex_secret_section(name, &mut cloned, store);
    cloned
}

pub(crate) fn inject_synthetic_secret_into_section(
    platform: Platform,
    section: &mut ConfigSection,
) {
    if platform == Platform::Grok {
        section.other.insert(
            "api_key".into(),
            toml::Value::String("xai-synthetic-probe".into()),
        );
    } else {
        section.auth_token = Some(Secret::new("sk-synthetic-probe"));
    }
}

pub(crate) fn is_credential_pending_section(
    platform: Platform,
    section: &ConfigSection,
    instance: &dyn PlatformConfig,
) -> bool {
    let has_auth_token = section
        .auth_token
        .as_ref()
        .is_some_and(|s| !s.expose().trim().is_empty());
    let has_api_key = section
        .other
        .get("api_key")
        .and_then(toml::Value::as_str)
        .is_some_and(|s| !s.trim().is_empty());
    if has_auth_token || has_api_key {
        return false;
    }

    let mut probe = section.clone();
    inject_synthetic_secret_into_section(platform, &mut probe);

    ConfigValidator::new()
        .validate_section_with(&probe, |candidate| {
            instance.validate_profile(&section_to_profile(candidate))
        })
        .is_ok()
}

pub(crate) fn enrich_missing_secret_enable_error(
    platform: Platform,
    name: &str,
    profile: &ProfileConfig,
    instance: &dyn PlatformConfig,
    err: CcrError,
) -> CcrError {
    let has_auth_token = profile
        .auth_token
        .as_ref()
        .is_some_and(|s| !s.expose().trim().is_empty());
    let has_api_key = profile
        .platform_data
        .get("api_key")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|s| !s.trim().is_empty());
    if has_auth_token || has_api_key {
        return err;
    }

    let mut probe = profile.clone();
    if platform == Platform::Grok {
        probe.platform_data.insert(
            "api_key".into(),
            serde_json::Value::String("xai-synthetic-probe".into()),
        );
    } else {
        probe.auth_token = Some(Secret::new("sk-synthetic-probe"));
    }
    if instance.validate_profile(&probe).is_ok() {
        let stdin_flag = platform_secret_stdin_flag(platform);
        return CcrError::ValidationError(format!(
            "无法启用 Profile '{name}'：当前配置缺少必填密钥；请先执行 `ccr {} profile edit {name} {stdin_flag} --enable` 补全密钥",
            platform.short_name()
        ));
    }
    err
}
