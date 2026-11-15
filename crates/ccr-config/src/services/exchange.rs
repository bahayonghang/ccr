//! Shared profile exchange (`ExchangeV1`), secret scrubbing, and 3-format safe TOML parser.
//!
//! Owns lossless native TOML (`ConfigSection` + `toml::Value::Datetime`) serialization,
//! `--no-secrets` / `export_config(false)` secret deletion & URL/extension scrubbing,
//! legacy masked-placeholder detection, and two-stage safe import parsing without value echo.

use crate::managers::config::repository::validate_profile_name;
use crate::managers::config::types::{ConfigSection, ProviderType};
use ccr_core::Secret;
use ccr_core::core::error::{CcrError, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Current `ExchangeV1` schema version.
pub const EXCHANGE_SCHEMA_VERSION: i64 = 1;

/// Header comment written at the top of `ExchangeV1` documents.
pub const EXCHANGE_HEADER_COMMENT: &str = "# CCR Profile Exchange File";

const SENSITIVE_QUERY_KEYS: &[&str] = &[
    "key",
    "api_key",
    "apikey",
    "x-api-key",
    "token",
    "auth",
    "auth_token",
    "access_token",
    "refresh_token",
    "id_token",
    "client_secret",
    "secret",
    "password",
    "passwd",
    "pwd",
    "sig",
    "signature",
];

const SAFE_EXTENSION_KEYS: &[&str] = &[
    "env_key",
    "cli_auth_credentials_store",
    "auth_mode",
    "auth_source",
    "wire_api",
    "api_protocol",
    "requires_openai_auth",
    "openai_login_method",
    "forced_login_method",
    "preferred_auth_method",
    "model_reasoning_effort",
    "reasoning_effort",
    "api_backend",
    "context_window",
    "supports_backend_search",
    "network_access",
    "disable_response_storage",
    "approval_policy",
    "sandbox_mode",
    "model_catalog_json",
    "max_tokens",
    "max_output_tokens",
    "max_completion_tokens",
    "prompt_caching",
    "token_limit",
    "token_budget",
    "token_count",
];

/// Declared secrets mode in an `ExchangeV1` document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExchangeSecretsMode {
    Included,
    Redacted,
}

impl ExchangeSecretsMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Included => "included",
            Self::Redacted => "redacted",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "included" => Some(Self::Included),
            "redacted" => Some(Self::Redacted),
            _ => None,
        }
    }
}

/// Detected source document format during `profile import`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportSourceFormat {
    ExchangeV1,
    LegacyCatalog,
    LegacySimplified,
}

impl ImportSourceFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExchangeV1 => "exchange_v1",
            Self::LegacyCatalog => "legacy_catalog",
            Self::LegacySimplified => "legacy_simplified",
        }
    }
}

/// Parsed import payload in source appearance order with runtime/catalog state stripped.
#[derive(Clone)]
pub struct ParsedImportDocument {
    pub source_format: ImportSourceFormat,
    pub secrets_mode: ExchangeSecretsMode,
    pub exported_at: Option<String>,
    pub profiles: IndexMap<String, ConfigSection>,
}

impl std::fmt::Debug for ParsedImportDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let profile_names: Vec<&str> = self.profiles.keys().map(String::as_str).collect();
        f.debug_struct("ParsedImportDocument")
            .field("source_format", &self.source_format)
            .field("secrets_mode", &self.secrets_mode)
            .field("exported_at", &self.exported_at)
            .field("profile_names", &profile_names)
            .finish()
    }
}

#[derive(Deserialize)]
struct OrderedExchangeProfiles {
    profiles: IndexMap<String, IndexMap<String, toml::Value>>,
}

/// Detect legacy masked placeholder strings (`***`, `...`, `sk-ant-...1234`, `REDACTED`, etc.).
///
/// Import deterministically rejects these across all 3 formats so broken masked strings
/// can never overwrite real credentials or land in catalogs / `profile_secrets.json`.
pub fn looks_like_legacy_masked_secret(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.chars().all(|ch| ch == '*') {
        return true;
    }
    if trimmed.contains("***") || trimmed.contains("...") {
        return true;
    }
    matches!(
        trimmed.to_ascii_lowercase().as_str(),
        "redacted" | "<redacted>" | "[redacted]"
    )
}

/// Scrub `userinfo@`, `#fragment`, and sensitive query parameters from a URL while
/// preserving non-secret query parameters (such as `?api-version=2024-10-21`).
///
/// Returns `(scrubbed_url, was_modified)`. Never echoes the raw URL in warnings.
pub fn scrub_base_url(raw: &str) -> (String, bool) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (raw.to_string(), false);
    }

    let mut changed = false;
    let without_fragment = match trimmed.split_once('#') {
        Some((before, _)) => {
            changed = true;
            before
        }
        None => trimmed,
    };

    let Some((scheme, after_scheme)) = without_fragment.split_once("://") else {
        // Malformed URL fallback: conservatively strip userinfo@ and ?query if present.
        let without_query = match without_fragment.split_once('?') {
            Some((before, _)) => {
                changed = true;
                before
            }
            None => without_fragment,
        };
        let clean = match without_query.rsplit_once('@') {
            Some((_, host_and_path)) => {
                changed = true;
                host_and_path
            }
            None => without_query,
        };
        return if changed {
            (clean.to_string(), true)
        } else {
            (raw.to_string(), false)
        };
    };

    let (authority_and_path, query_opt) = match after_scheme.split_once('?') {
        Some((before, query)) => (before, Some(query)),
        None => (after_scheme, None),
    };

    let slash_pos = authority_and_path.find('/');
    let (authority, path) = match slash_pos {
        Some(idx) => (&authority_and_path[..idx], &authority_and_path[idx..]),
        None => (authority_and_path, ""),
    };

    let clean_authority = match authority.rsplit_once('@') {
        Some((_, host_port)) => {
            changed = true;
            host_port
        }
        None => authority,
    };

    if clean_authority.is_empty() {
        return (format!("{scheme}://{path}"), true);
    }

    let clean_query = match query_opt {
        None => None,
        Some("") => {
            changed = true;
            None
        }
        Some(query) => {
            let mut kept = Vec::new();
            for pair in query.split('&') {
                if pair.is_empty() {
                    changed = true;
                    continue;
                }
                let (key, val) = pair.split_once('=').unwrap_or((pair, ""));
                let decoded_key = key.trim().to_ascii_lowercase();
                if SENSITIVE_QUERY_KEYS.contains(&decoded_key.as_str())
                    || value_looks_like_high_confidence_secret(val)
                {
                    changed = true;
                } else {
                    kept.push(pair);
                }
            }
            if kept.is_empty() {
                changed = true;
                None
            } else {
                Some(kept.join("&"))
            }
        }
    };

    if !changed {
        return (raw.to_string(), false);
    }

    let mut rebuilt = format!("{scheme}://{clean_authority}{path}");
    if let Some(q) = clean_query {
        rebuilt.push('?');
        rebuilt.push_str(&q);
    }
    (rebuilt, true)
}

/// Check whether a URL carries `userinfo@`, `#fragment`, or sensitive query parameters.
pub fn url_contains_credentials(raw: &str) -> bool {
    scrub_base_url(raw).1
}

/// Check whether an extension key name looks like a secret-bearing field.
pub fn key_looks_like_secret_field(key: &str, platform_whitelist: &[&str]) -> bool {
    let lower = key.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return false;
    }
    if SAFE_EXTENSION_KEYS.contains(&lower.as_str())
        || platform_whitelist
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(&lower))
        || lower.ends_with("_tokens")
    {
        return false;
    }

    const PATTERNS: &[&str] = &[
        "secret",
        "password",
        "passwd",
        "pwd",
        "api_key",
        "apikey",
        "api-key",
        "private_key",
        "access_key",
        "authorization",
        "bearer",
        "credential",
        "cookie",
        "session_key",
        "token",
    ];
    PATTERNS.iter().any(|pat| lower.contains(pat))
}

/// Check whether a string value matches a high-confidence credential/token pattern.
pub fn value_looks_like_high_confidence_secret(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }

    const PREFIXES: &[&str] = &[
        "sk-",
        "xai-",
        "ghp_",
        "gho_",
        "ghu_",
        "ghs_",
        "github_pat_",
        "glpat-",
        "AKIA",
        "AIza",
    ];
    if PREFIXES.iter().any(|prefix| trimmed.starts_with(prefix)) {
        return true;
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("bearer ") || lower.starts_with("basic ") {
        return true;
    }

    // 3-segment JWT starting with base64url JSON header `eyJ`
    if trimmed.starts_with("eyJ") {
        let mut parts = trimmed.split('.');
        if let (Some(h), Some(p), Some(s), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
            && !h.is_empty()
            && !p.is_empty()
            && !s.is_empty()
            && [h, p, s].into_iter().all(|seg| {
                seg.chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
            })
        {
            return true;
        }
    }

    // Credential-bearing URL inside a string field
    if (trimmed.starts_with("http://") || trimmed.starts_with("https://"))
        && url_contains_credentials(trimmed)
    {
        return true;
    }

    false
}

/// Check whether `section` currently carries any inline secret, credential-bearing URL,
/// or secret-shaped extension value.
pub fn section_contains_inline_secrets(
    section: &ConfigSection,
    platform_whitelist: &[&str],
) -> bool {
    if section
        .auth_token
        .as_ref()
        .is_some_and(|secret| !secret.expose().trim().is_empty())
    {
        return true;
    }
    if section
        .other
        .get("api_key")
        .and_then(toml::Value::as_str)
        .is_some_and(|val| !val.trim().is_empty())
    {
        return true;
    }
    if section
        .base_url
        .as_deref()
        .is_some_and(url_contains_credentials)
    {
        return true;
    }
    other_map_contains_secrets(&section.other, platform_whitelist)
}

fn other_map_contains_secrets(
    map: &IndexMap<String, toml::Value>,
    platform_whitelist: &[&str],
) -> bool {
    for (key, value) in map {
        if matches!(key.as_str(), "api_key" | "auth_token") {
            if value.as_str().is_some_and(|s| !s.trim().is_empty()) {
                return true;
            }
            continue;
        }
        if toml_entry_contains_secrets(key, value, platform_whitelist) {
            return true;
        }
    }
    false
}

fn toml_entry_contains_secrets(
    key: &str,
    value: &toml::Value,
    platform_whitelist: &[&str],
) -> bool {
    match value {
        toml::Value::String(text) => {
            key_looks_like_secret_field(key, platform_whitelist)
                || value_looks_like_high_confidence_secret(text)
        }
        toml::Value::Table(sub) => sub
            .iter()
            .any(|(sub_k, sub_v)| toml_entry_contains_secrets(sub_k, sub_v, platform_whitelist)),
        toml::Value::Array(items) => {
            if key_looks_like_secret_field(key, platform_whitelist)
                && items.iter().any(|item| item.as_str().is_some())
            {
                return true;
            }
            items
                .iter()
                .any(|item| toml_array_item_contains_secrets(item, platform_whitelist))
        }
        toml::Value::Integer(_)
        | toml::Value::Float(_)
        | toml::Value::Boolean(_)
        | toml::Value::Datetime(_) => false,
    }
}

fn toml_array_item_contains_secrets(item: &toml::Value, platform_whitelist: &[&str]) -> bool {
    match item {
        toml::Value::String(text) => value_looks_like_high_confidence_secret(text),
        toml::Value::Table(sub) => sub
            .iter()
            .any(|(sub_k, sub_v)| toml_entry_contains_secrets(sub_k, sub_v, platform_whitelist)),
        toml::Value::Array(nested) => nested
            .iter()
            .any(|child| toml_array_item_contains_secrets(child, platform_whitelist)),
        toml::Value::Integer(_)
        | toml::Value::Float(_)
        | toml::Value::Boolean(_)
        | toml::Value::Datetime(_) => false,
    }
}

/// Remove inline credentials (`auth_token`, `api_key`), scrub `base_url`, and recursively
/// scrub unknown secret-bearing extension fields from `section` in place.
///
/// Warnings record field paths only and never echo raw URLs or secret values.
pub fn scrub_section_secrets(
    section: &mut ConfigSection,
    profile_prefix: &str,
    platform_whitelist: &[&str],
    warnings: &mut Vec<String>,
) {
    section.auth_token = None;
    section.other.shift_remove("api_key");
    section.other.shift_remove("auth_token");

    if let Some(raw_url) = section.base_url.as_deref() {
        let (clean_url, changed) = scrub_base_url(raw_url);
        if changed {
            section.base_url = Some(clean_url);
            warnings.push(format!(
                "已清理 {profile_prefix}.base_url 中的内联凭据或查询密钥"
            ));
        }
    }

    let mut retained = IndexMap::with_capacity(section.other.len());
    for (key, mut value) in std::mem::take(&mut section.other) {
        let field_path = format!("{profile_prefix}.{key}");
        if scrub_toml_value(&key, &mut value, &field_path, platform_whitelist, warnings) {
            retained.insert(key, value);
        }
    }
    section.other = retained;
}

/// Scrub a raw `toml::Table` representing a canonical catalog document (`export_config(false)`).
pub fn scrub_catalog_toml_table(root: &mut toml::Table, warnings: &mut Vec<String>) {
    let keys: Vec<String> = root.keys().cloned().collect();
    for key in keys {
        if matches!(key.as_str(), "current_config" | "default_config") {
            continue;
        }
        if key == "settings" {
            if let Some(mut settings_value) = root.remove("settings") {
                match &mut settings_value {
                    toml::Value::Table(settings_table) => {
                        let was_empty = settings_table.is_empty();
                        scrub_raw_toml_table(settings_table, "settings", &[], warnings);
                        if was_empty || !settings_table.is_empty() {
                            root.insert("settings".into(), settings_value);
                        }
                    }
                    _ => {
                        if scrub_toml_value(
                            "settings",
                            &mut settings_value,
                            "settings",
                            &[],
                            warnings,
                        ) {
                            root.insert("settings".into(), settings_value);
                        }
                    }
                }
            }
            continue;
        }
        let Some(mut value) = root.remove(&key) else {
            continue;
        };
        match &mut value {
            toml::Value::Table(profile_table) => {
                profile_table.remove("auth_token");
                profile_table.remove("api_key");
                if let Some(toml::Value::String(raw_url)) = profile_table.get("base_url").cloned() {
                    let (clean_url, changed) = scrub_base_url(&raw_url);
                    if changed {
                        profile_table.insert("base_url".into(), toml::Value::String(clean_url));
                        warnings.push(format!("已清理 {key}.base_url 中的内联凭据或查询密钥"));
                    }
                }
                scrub_raw_toml_table(profile_table, &key, &[], warnings);
                root.insert(key, value);
            }
            _ => {
                let path = key.clone();
                if scrub_toml_value(&key, &mut value, &path, &[], warnings) {
                    root.insert(key, value);
                }
            }
        }
    }
}

fn scrub_raw_toml_table(
    table: &mut toml::Table,
    prefix: &str,
    platform_whitelist: &[&str],
    warnings: &mut Vec<String>,
) {
    let keys: Vec<String> = table.keys().cloned().collect();
    for key in keys {
        let Some(mut val) = table.remove(&key) else {
            continue;
        };
        let field_path = format!("{prefix}.{key}");
        if scrub_toml_value(&key, &mut val, &field_path, platform_whitelist, warnings) {
            table.insert(key, val);
        }
    }
}

/// Returns `true` if `value` should be kept, `false` if stripped.
fn scrub_toml_value(
    key: &str,
    value: &mut toml::Value,
    field_path: &str,
    platform_whitelist: &[&str],
    warnings: &mut Vec<String>,
) -> bool {
    match value {
        toml::Value::String(text) => {
            if key_looks_like_secret_field(key, platform_whitelist)
                || value_looks_like_high_confidence_secret(text)
            {
                warnings.push(format!("已剥离疑似敏感扩展字段 {field_path}"));
                false
            } else {
                true
            }
        }
        toml::Value::Table(sub) => {
            let was_non_empty = !sub.is_empty();
            scrub_raw_toml_table(sub, field_path, platform_whitelist, warnings);
            !(was_non_empty && sub.is_empty())
        }
        toml::Value::Array(items) => {
            if key_looks_like_secret_field(key, platform_whitelist)
                && items.iter().any(|item| item.as_str().is_some())
            {
                warnings.push(format!("已剥离疑似敏感扩展字段 {field_path}"));
                return false;
            }
            let original_len = items.len();
            let mut next_items = Vec::with_capacity(original_len);
            let mut stripped_string_item = false;
            for mut item in std::mem::take(items) {
                match &mut item {
                    toml::Value::String(text) => {
                        if value_looks_like_high_confidence_secret(text) {
                            stripped_string_item = true;
                        } else {
                            next_items.push(item);
                        }
                    }
                    toml::Value::Table(sub) => {
                        let was_non_empty = !sub.is_empty();
                        scrub_raw_toml_table(sub, field_path, platform_whitelist, warnings);
                        if !(was_non_empty && sub.is_empty()) {
                            next_items.push(item);
                        }
                    }
                    _ => next_items.push(item),
                }
            }
            if stripped_string_item {
                warnings.push(format!("已剥离疑似敏感扩展字段 {field_path}"));
            }
            *items = next_items;
            !(original_len > 0 && items.is_empty())
        }
        toml::Value::Integer(_)
        | toml::Value::Float(_)
        | toml::Value::Boolean(_)
        | toml::Value::Datetime(_) => true,
    }
}

/// Convert a `ConfigSection` into a `toml::Table` for `ExchangeV1` serialization.
///
/// Strips `usage_count` and preserves native `toml::Value` entries (including `Datetime`
/// and nested tables) in `section.other`.
pub fn section_to_exchange_table(section: &ConfigSection) -> toml::Table {
    let mut table = toml::Table::new();
    for (key, value) in &section.other {
        table.insert(key.clone(), value.clone());
    }

    let string_fields: &[(&str, &Option<String>)] = &[
        ("description", &section.description),
        ("base_url", &section.base_url),
        ("model", &section.model),
        ("small_fast_model", &section.small_fast_model),
        ("default_opus_model", &section.default_opus_model),
        ("default_sonnet_model", &section.default_sonnet_model),
        ("default_haiku_model", &section.default_haiku_model),
        ("default_fable_model", &section.default_fable_model),
        ("default_opus_model_name", &section.default_opus_model_name),
        (
            "default_sonnet_model_name",
            &section.default_sonnet_model_name,
        ),
        (
            "default_haiku_model_name",
            &section.default_haiku_model_name,
        ),
        (
            "default_fable_model_name",
            &section.default_fable_model_name,
        ),
        ("subagent_model", &section.subagent_model),
        ("custom_model_option", &section.custom_model_option),
        (
            "custom_model_option_name",
            &section.custom_model_option_name,
        ),
        ("effort_level", &section.effort_level),
        (
            "claude_code_auto_compact_window",
            &section.claude_code_auto_compact_window,
        ),
        ("api_timeout_ms", &section.api_timeout_ms),
        (
            "claude_code_disable_nonessential_traffic",
            &section.claude_code_disable_nonessential_traffic,
        ),
        ("provider", &section.provider),
        ("account", &section.account),
    ];

    for (key, opt) in string_fields {
        match opt {
            Some(val) => {
                table.insert((*key).to_string(), toml::Value::String(val.clone()));
            }
            None => {
                table.remove(*key);
            }
        }
    }

    match &section.auth_token {
        Some(secret) => {
            table.insert(
                "auth_token".into(),
                toml::Value::String(secret.expose().to_string()),
            );
        }
        None => {
            table.remove("auth_token");
        }
    }

    match &section.provider_type {
        Some(kind) => {
            table.insert(
                "provider_type".into(),
                toml::Value::String(kind.to_string_value().to_string()),
            );
        }
        None => {
            table.remove("provider_type");
        }
    }

    match &section.tags {
        Some(tags) => {
            let arr = tags
                .iter()
                .cloned()
                .map(toml::Value::String)
                .collect::<Vec<_>>();
            table.insert("tags".into(), toml::Value::Array(arr));
        }
        None => {
            table.remove("tags");
        }
    }

    // Never emit local usage_count in ExchangeV1.
    table.remove("usage_count");

    table.insert("enabled".into(), toml::Value::Boolean(section.is_enabled()));

    table
}

/// Render an `ExchangeV1` document preserving the exact iteration order of `profiles`.
pub fn render_exchange_v1(
    platform: &str,
    exported_at: &str,
    secrets_mode: ExchangeSecretsMode,
    profiles: &IndexMap<String, ConfigSection>,
) -> Result<String> {
    let mut out = String::new();
    out.push_str(EXCHANGE_HEADER_COMMENT);
    out.push('\n');
    // Render header keys in canonical spec order: schema_version, platform, exported_at, secrets_mode.
    out.push_str(&format!("schema_version = {EXCHANGE_SCHEMA_VERSION}\n"));
    out.push_str(&format!(
        "platform = {}\n",
        serialize_toml_string_literal(platform)?
    ));
    out.push_str(&format!(
        "exported_at = {}\n",
        serialize_toml_string_literal(exported_at)?
    ));
    out.push_str(&format!(
        "secrets_mode = {}\n",
        serialize_toml_string_literal(secrets_mode.as_str())?
    ));

    for (name, section) in profiles {
        validate_profile_name(name)?;
        let mut single_profiles = toml::Table::new();
        single_profiles.insert(
            name.clone(),
            toml::Value::Table(section_to_exchange_table(section)),
        );
        let mut wrapper = toml::Table::new();
        wrapper.insert("profiles".into(), toml::Value::Table(single_profiles));
        let block = toml::to_string(&wrapper)
            .map_err(|_| CcrError::ConfigError(format!("无法序列化 profile '{name}'")))?;
        out.push('\n');
        out.push_str(block.trim_end());
        out.push('\n');
    }

    Ok(out)
}

fn serialize_toml_string_literal(value: &str) -> Result<String> {
    let mut probe = toml::Table::new();
    probe.insert("v".into(), toml::Value::String(value.to_string()));
    let rendered =
        toml::to_string(&probe).map_err(|_| CcrError::ConfigError("无法序列化字符串".into()))?;
    let literal = rendered
        .trim()
        .strip_prefix("v = ")
        .ok_or_else(|| CcrError::ConfigError("无法提取 TOML 字符串字面量".into()))?;
    Ok(literal.to_string())
}

fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let clamped = offset.min(source.len());
    let prefix = &source[..clamped];
    let mut line = 1_usize;
    let mut col = 1_usize;
    for ch in prefix.chars() {
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

pub fn safe_canonical_toml_parse_error(source: &str, err: &toml::de::Error) -> CcrError {
    if let Some(span) = err.span() {
        let (line, col) = byte_offset_to_line_col(source, span.start);
        CcrError::ConfigError(format!("解析规范配置失败（第 {line} 行，第 {col} 列）"))
    } else {
        CcrError::ConfigError("解析规范配置失败".into())
    }
}

fn safe_toml_syntax_error(source: &str, err: &toml::de::Error) -> CcrError {
    if let Some(span) = err.span() {
        let (line, col) = byte_offset_to_line_col(source, span.start);
        CcrError::ConfigFormatInvalid(format!(
            "导入文件 TOML 语法错误（第 {line} 行，第 {col} 列）"
        ))
    } else {
        CcrError::ConfigFormatInvalid("导入文件 TOML 语法错误".into())
    }
}

/// Two-stage safe 3-format TOML parser (`ExchangeV1`, `LegacyCatalog`, `LegacySimplified`).
///
/// - Never echoes raw source lines or field values in error messages.
/// - Preserves source profile appearance order and native `toml::Value::Datetime`.
/// - Strips `current_config`, `default_config`, `settings`, `usage_count`, and root `extra_toml`.
/// - Rejects legacy masked placeholder secrets (`***`, `...`, `REDACTED`) across all 3 formats.
pub fn parse_import_document(
    content: &str,
    expected_platform: &str,
) -> Result<ParsedImportDocument> {
    let top: IndexMap<String, toml::Value> =
        toml::from_str(content).map_err(|err| safe_toml_syntax_error(content, &err))?;

    let has_exchange_header_keys = top.contains_key("schema_version")
        || top.contains_key("platform")
        || top.contains_key("exported_at")
        || top.contains_key("secrets_mode");

    let is_legacy_catalog = top.contains_key("current_config")
        || top.contains_key("default_config")
        || top.contains_key("settings");

    let has_exchange_profiles_table = top
        .get("profiles")
        .and_then(toml::Value::as_table)
        .is_some_and(|tbl| {
            (top.len() == 1 && tbl.is_empty())
                || (!is_legacy_catalog
                    && !tbl.is_empty()
                    && tbl.values().all(toml::Value::is_table))
        });

    if has_exchange_header_keys || has_exchange_profiles_table {
        return parse_exchange_v1(content, top, expected_platform);
    }

    if is_legacy_catalog {
        return parse_legacy_catalog(top);
    }

    parse_legacy_simplified(top)
}

fn parse_exchange_v1(
    content: &str,
    top: IndexMap<String, toml::Value>,
    expected_platform: &str,
) -> Result<ParsedImportDocument> {
    for key in top.keys() {
        if !matches!(
            key.as_str(),
            "schema_version" | "platform" | "exported_at" | "secrets_mode" | "profiles"
        ) {
            return Err(CcrError::ConfigFormatInvalid(format!(
                "ExchangeV1 包含未知顶层字段 '{key}'"
            )));
        }
    }

    match top.get("schema_version") {
        Some(toml::Value::Integer(EXCHANGE_SCHEMA_VERSION)) => {}
        Some(toml::Value::Integer(other)) => {
            return Err(CcrError::ConfigFormatInvalid(format!(
                "不支持的交换文件版本 schema_version = {other}（当前仅支持版本 1，请升级 CCR）"
            )));
        }
        Some(_) => {
            return Err(CcrError::ConfigFormatInvalid(
                "ExchangeV1 的 schema_version 必须为整数 1".into(),
            ));
        }
        None => {
            return Err(CcrError::ConfigFormatInvalid(
                "ExchangeV1 缺少必填字段 schema_version".into(),
            ));
        }
    }

    let file_platform = match top.get("platform") {
        Some(toml::Value::String(p)) if !p.trim().is_empty() => p.trim(),
        Some(_) => {
            return Err(CcrError::ConfigFormatInvalid(
                "ExchangeV1 的 platform 字段必须为非空字符串".into(),
            ));
        }
        None => {
            return Err(CcrError::ConfigFormatInvalid(
                "ExchangeV1 缺少必填字段 platform".into(),
            ));
        }
    };

    if file_platform != expected_platform {
        return Err(CcrError::ValidationError(format!(
            "导入文件声明的平台为 '{file_platform}'，与目标平台 '{expected_platform}' 不匹配；请改用 `ccr {file_platform} profile import`"
        )));
    }

    let exported_at = match top.get("exported_at") {
        None => None,
        Some(toml::Value::String(ts)) => Some(ts.clone()),
        Some(toml::Value::Datetime(dt)) => Some(dt.to_string()),
        Some(_) => {
            return Err(CcrError::ConfigFormatInvalid(
                "ExchangeV1 的 exported_at 字段类型无效".into(),
            ));
        }
    };

    let secrets_mode = match top.get("secrets_mode") {
        None => ExchangeSecretsMode::Included,
        Some(toml::Value::String(raw)) => ExchangeSecretsMode::parse(raw).ok_or_else(|| {
            CcrError::ValidationError(
                "ExchangeV1 的 secrets_mode 值无效；允许值为 included 或 redacted".into(),
            )
        })?,
        Some(_) => {
            return Err(CcrError::ValidationError(
                "ExchangeV1 的 secrets_mode 必须为字符串 included 或 redacted".into(),
            ));
        }
    };

    let Some(profiles_val) = top.get("profiles") else {
        return Err(CcrError::ConfigFormatInvalid(
            "ExchangeV1 缺少 [profiles] 表".into(),
        ));
    };
    let Some(profiles_tbl) = profiles_val.as_table() else {
        return Err(CcrError::ConfigFormatInvalid(
            "ExchangeV1 的 profiles 必须为表".into(),
        ));
    };
    if profiles_tbl.is_empty() {
        return Err(CcrError::ConfigFormatInvalid(
            "导入文件 [profiles] 为空，至少需要包含一个 profile".into(),
        ));
    }
    for (name, entry) in profiles_tbl {
        if !entry.is_table() {
            return Err(CcrError::ConfigFormatInvalid(format!(
                "profiles.'{name}' 必须为表"
            )));
        }
    }

    // Re-deserialize `profiles` into `IndexMap` to preserve exact file appearance order.
    let ordered: OrderedExchangeProfiles =
        toml::from_str(content).map_err(|err| safe_toml_syntax_error(content, &err))?;

    let mut profiles = IndexMap::with_capacity(ordered.profiles.len());
    for (name, raw_section) in ordered.profiles {
        validate_profile_name(&name)?;
        let section = parse_section_map(&name, raw_section, secrets_mode)?;
        profiles.insert(name, section);
    }

    Ok(ParsedImportDocument {
        source_format: ImportSourceFormat::ExchangeV1,
        secrets_mode,
        exported_at,
        profiles,
    })
}

fn parse_legacy_catalog(mut top: IndexMap<String, toml::Value>) -> Result<ParsedImportDocument> {
    for marker in ["current_config", "default_config"] {
        if let Some(val) = top.shift_remove(marker)
            && !val.is_str()
        {
            return Err(CcrError::ConfigFormatInvalid(format!(
                "{marker} 的类型必须为字符串"
            )));
        }
    }
    if let Some(settings) = top.shift_remove("settings")
        && !settings.is_table()
    {
        return Err(CcrError::ConfigFormatInvalid(
            "settings 的类型必须为表".into(),
        ));
    }

    let mut profiles = IndexMap::new();
    for (name, value) in top {
        // Strip document-root extra scalars/arrays from legacy catalog exports.
        if let toml::Value::Table(tbl) = value {
            validate_profile_name(&name)?;
            let map: IndexMap<String, toml::Value> = tbl.into_iter().collect();
            let section = parse_section_map(&name, map, ExchangeSecretsMode::Included)?;
            profiles.insert(name, section);
        }
    }

    if profiles.is_empty() {
        return Err(CcrError::ConfigFormatInvalid(
            "导入文件不包含任何 profile".into(),
        ));
    }

    Ok(ParsedImportDocument {
        source_format: ImportSourceFormat::LegacyCatalog,
        secrets_mode: ExchangeSecretsMode::Included,
        exported_at: None,
        profiles,
    })
}

fn parse_legacy_simplified(top: IndexMap<String, toml::Value>) -> Result<ParsedImportDocument> {
    if top.is_empty() {
        return Err(CcrError::ConfigFormatInvalid(
            "导入文件为空，不包含任何 profile".into(),
        ));
    }

    let mut profiles = IndexMap::with_capacity(top.len());
    for (name, value) in top {
        let toml::Value::Table(tbl) = value else {
            return Err(CcrError::ConfigFormatInvalid(format!(
                "顶层字段 '{name}' 必须为 profile 表"
            )));
        };
        validate_profile_name(&name)?;
        let map: IndexMap<String, toml::Value> = tbl.into_iter().collect();
        let section = parse_section_map(&name, map, ExchangeSecretsMode::Included)?;
        profiles.insert(name, section);
    }

    Ok(ParsedImportDocument {
        source_format: ImportSourceFormat::LegacySimplified,
        secrets_mode: ExchangeSecretsMode::Included,
        exported_at: None,
        profiles,
    })
}

fn take_opt_string(
    map: &mut IndexMap<String, toml::Value>,
    profile_name: &str,
    key: &str,
) -> Result<Option<String>> {
    match map.shift_remove(key) {
        None => Ok(None),
        Some(toml::Value::String(val)) => Ok(Some(val)),
        Some(_) => Err(CcrError::ConfigFormatInvalid(format!(
            "profile '{profile_name}' 的字段 '{key}' 必须为字符串"
        ))),
    }
}

fn parse_section_map(
    name: &str,
    mut map: IndexMap<String, toml::Value>,
    secrets_mode: ExchangeSecretsMode,
) -> Result<ConfigSection> {
    let description = take_opt_string(&mut map, name, "description")?;
    let base_url = take_opt_string(&mut map, name, "base_url")?;
    let auth_token_raw = take_opt_string(&mut map, name, "auth_token")?;
    let model = take_opt_string(&mut map, name, "model")?;
    let small_fast_model = take_opt_string(&mut map, name, "small_fast_model")?;
    let default_opus_model = take_opt_string(&mut map, name, "default_opus_model")?;
    let default_sonnet_model = take_opt_string(&mut map, name, "default_sonnet_model")?;
    let default_haiku_model = take_opt_string(&mut map, name, "default_haiku_model")?;
    let default_fable_model = take_opt_string(&mut map, name, "default_fable_model")?;
    let default_opus_model_name = take_opt_string(&mut map, name, "default_opus_model_name")?;
    let default_sonnet_model_name = take_opt_string(&mut map, name, "default_sonnet_model_name")?;
    let default_haiku_model_name = take_opt_string(&mut map, name, "default_haiku_model_name")?;
    let default_fable_model_name = take_opt_string(&mut map, name, "default_fable_model_name")?;
    let subagent_model = take_opt_string(&mut map, name, "subagent_model")?;
    let custom_model_option = take_opt_string(&mut map, name, "custom_model_option")?;
    let custom_model_option_name = take_opt_string(&mut map, name, "custom_model_option_name")?;
    let effort_level = take_opt_string(&mut map, name, "effort_level")?;
    let claude_code_auto_compact_window =
        take_opt_string(&mut map, name, "claude_code_auto_compact_window")?;
    let api_timeout_ms = take_opt_string(&mut map, name, "api_timeout_ms")?;
    let claude_code_disable_nonessential_traffic =
        take_opt_string(&mut map, name, "claude_code_disable_nonessential_traffic")?;
    let provider = take_opt_string(&mut map, name, "provider")?;

    let provider_type = match map.shift_remove("provider_type") {
        None => None,
        Some(toml::Value::String(raw)) => match raw.trim() {
            "official_relay" => Some(ProviderType::OfficialRelay),
            "third_party" | "third_party_model" => Some(ProviderType::ThirdPartyModel),
            _ => {
                return Err(CcrError::ValidationError(format!(
                    "profile '{name}' 的 provider_type 不受支持；允许值为 official_relay、third_party_model"
                )));
            }
        },
        Some(_) => {
            return Err(CcrError::ConfigFormatInvalid(format!(
                "profile '{name}' 的字段 'provider_type' 必须为字符串"
            )));
        }
    };

    let account = take_opt_string(&mut map, name, "account")?;

    let tags = match map.shift_remove("tags") {
        None => None,
        Some(toml::Value::Array(items)) => {
            let mut list = Vec::with_capacity(items.len());
            for item in items {
                let Some(tag) = item.as_str() else {
                    return Err(CcrError::ConfigFormatInvalid(format!(
                        "profile '{name}' 的字段 'tags' 元素必须为字符串"
                    )));
                };
                list.push(tag.to_string());
            }
            Some(list)
        }
        Some(_) => {
            return Err(CcrError::ConfigFormatInvalid(format!(
                "profile '{name}' 的字段 'tags' 必须为字符串数组"
            )));
        }
    };

    // Validate type of usage_count if present, then strip it completely.
    if let Some(val) = map.shift_remove("usage_count") {
        match val {
            toml::Value::Integer(n) if (0..=i64::from(u32::MAX)).contains(&n) => {}
            _ => {
                return Err(CcrError::ConfigFormatInvalid(format!(
                    "profile '{name}' 的字段 'usage_count' 必须为非负整数"
                )));
            }
        }
    }

    let enabled = match map.shift_remove("enabled") {
        None => None,
        Some(toml::Value::Boolean(flag)) => Some(flag),
        Some(_) => {
            return Err(CcrError::ConfigFormatInvalid(format!(
                "profile '{name}' 的字段 'enabled' 必须为布尔值"
            )));
        }
    };

    // Security gate 1: redacted ExchangeV1 must never carry inline secret fields.
    if secrets_mode == ExchangeSecretsMode::Redacted {
        if auth_token_raw
            .as_deref()
            .is_some_and(|s| !s.trim().is_empty())
        {
            return Err(CcrError::ValidationError(format!(
                "profile '{name}' 在 secrets_mode = \"redacted\" 文件中仍包含内联 auth_token"
            )));
        }
        if map
            .get("api_key")
            .and_then(toml::Value::as_str)
            .is_some_and(|s| !s.trim().is_empty())
        {
            return Err(CcrError::ValidationError(format!(
                "profile '{name}' 在 secrets_mode = \"redacted\" 文件中仍包含内联 api_key"
            )));
        }
    }

    // Security gate 2: deterministic rejection of legacy masked strings (`***`, `...`, `REDACTED`).
    if let Some(raw_token) = auth_token_raw.as_deref()
        && looks_like_legacy_masked_secret(raw_token)
    {
        return Err(CcrError::ValidationError(format!(
            "profile '{name}' 的 auth_token 疑似已脱敏占位串，已拒绝导入；请改用完整导出文件或使用 `--no-secrets` 导出"
        )));
    }
    if let Some(api_key_val) = map.get("api_key") {
        let Some(raw_api_key) = api_key_val.as_str() else {
            return Err(CcrError::ValidationError(format!(
                "profile '{name}' 的 api_key 必须是字符串"
            )));
        };
        if looks_like_legacy_masked_secret(raw_api_key) {
            return Err(CcrError::ValidationError(format!(
                "profile '{name}' 的 api_key 疑似已脱敏占位串，已拒绝导入；请改用完整导出文件或使用 `--no-secrets` 导出"
            )));
        }
    }

    Ok(ConfigSection {
        description,
        base_url,
        auth_token: auth_token_raw.map(Secret::new),
        model,
        small_fast_model,
        default_opus_model,
        default_sonnet_model,
        default_haiku_model,
        default_fable_model,
        default_opus_model_name,
        default_sonnet_model_name,
        default_haiku_model_name,
        default_fable_model_name,
        subagent_model,
        custom_model_option,
        custom_model_option_name,
        effort_level,
        claude_code_auto_compact_window,
        api_timeout_ms,
        claude_code_disable_nonessential_traffic,
        provider,
        provider_type,
        account,
        tags,
        usage_count: None,
        enabled,
        other: map,
    })
}

/// Produce a deterministic canonical TOML representation of an already-normalized `ConfigSection`
/// for spec-equality (`unchanged` vs `updated`) and `--dedup` comparison.
///
/// Key ordering in `other` and nested tables is canonicalized via `BTreeMap` (`toml::Table`),
/// `usage_count` is excluded, `tags` empty vs `None` are canonicalized, and native `Datetime`
/// values are preserved losslessly.
pub fn canonical_section_spec_bytes(section: &ConfigSection) -> Result<Vec<u8>> {
    let mut normalized = section.clone();
    normalized.usage_count = None;
    normalized.enabled = Some(section.is_enabled());
    if normalized.tags.as_ref().is_some_and(Vec::is_empty) {
        normalized.tags = None;
    }
    let table = canonicalize_toml_table(&section_to_exchange_table(&normalized));
    let rendered =
        toml::to_string(&table).map_err(|_| CcrError::ConfigError("无法规范化配置规格".into()))?;
    Ok(rendered.into_bytes())
}

fn canonicalize_toml_table(table: &toml::Table) -> BTreeMap<String, toml::Value> {
    let mut sorted = BTreeMap::new();
    for (k, v) in table {
        sorted.insert(k.clone(), canonicalize_toml_value(v));
    }
    sorted
}

fn canonicalize_toml_value(value: &toml::Value) -> toml::Value {
    match value {
        toml::Value::Table(sub) => {
            let mut out = toml::Table::new();
            for (k, v) in canonicalize_toml_table(sub) {
                out.insert(k, v);
            }
            toml::Value::Table(out)
        }
        toml::Value::Array(items) => {
            toml::Value::Array(items.iter().map(canonicalize_toml_value).collect())
        }
        other => other.clone(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn legacy_masked_secret_detector_catches_placeholders_without_false_positives() {
        assert!(looks_like_legacy_masked_secret("********"));
        assert!(looks_like_legacy_masked_secret("sk-ant-***1234"));
        assert!(looks_like_legacy_masked_secret("sk-ant-...1234"));
        assert!(looks_like_legacy_masked_secret("sk-a...cdef"));
        assert!(looks_like_legacy_masked_secret("REDACTED"));
        assert!(looks_like_legacy_masked_secret("<redacted>"));
        assert!(looks_like_legacy_masked_secret("[REDACTED]"));

        assert!(!looks_like_legacy_masked_secret(""));
        assert!(!looks_like_legacy_masked_secret("   "));
        assert!(!looks_like_legacy_masked_secret(
            "sk-ant-live-real-token-9988"
        ));
        assert!(!looks_like_legacy_masked_secret("xai-real-key-123456"));
    }

    #[test]
    fn scrub_base_url_strips_userinfo_fragment_and_secret_queries_preserving_api_version() {
        let (clean, changed) = scrub_base_url(
            "https://user:pass@proxy.example.com:8443/v1?api-version=2024-10-21&api_key=sk-secret&region=east#frag",
        );
        assert!(changed);
        assert_eq!(
            clean,
            "https://proxy.example.com:8443/v1?api-version=2024-10-21&region=east"
        );

        let (clean2, changed2) = scrub_base_url("https://proxy.example.com/v1?token=abc");
        assert!(changed2);
        assert_eq!(clean2, "https://proxy.example.com/v1");

        let (unchanged_url, changed3) =
            scrub_base_url("https://proxy.example.com/v1?api-version=2024-10-21");
        assert!(!changed3);
        assert_eq!(
            unchanged_url,
            "https://proxy.example.com/v1?api-version=2024-10-21"
        );
    }

    #[test]
    fn exchange_v1_preserves_profile_order_and_native_toml_datetime_and_scrubs_extensions() {
        let parsed_dt: toml::Value = toml::from_str::<toml::Value>("ts = 2026-04-20T09:15:30Z")
            .unwrap()
            .as_table()
            .unwrap()
            .get("ts")
            .unwrap()
            .clone();
        assert!(matches!(parsed_dt, toml::Value::Datetime(_)));

        let mut zeta = ConfigSection {
            description: Some("Zeta first".into()),
            base_url: Some("https://user:pw@proxy.example.com/v1?api-version=1&key=secret".into()),
            auth_token: Some(Secret::new("sk-zeta-secret")),
            model: Some("claude-sonnet-4-5".into()),
            usage_count: Some(42),
            enabled: Some(true),
            other: IndexMap::from([
                ("auth_mode".into(), toml::Value::String("api_key".into())),
                (
                    "env_key".into(),
                    toml::Value::String("MY_PROVIDER_KEY".into()),
                ),
                ("created_at".into(), parsed_dt),
                (
                    "custom_bearer".into(),
                    toml::Value::String("Bearer secret-xyz".into()),
                ),
                ("max_tokens".into(), toml::Value::Integer(8192)),
            ]),
            ..Default::default()
        };
        let mut warnings = Vec::new();
        scrub_section_secrets(&mut zeta, "profiles.zeta", &[], &mut warnings);
        assert!(zeta.auth_token.is_none());
        assert_eq!(
            zeta.base_url.as_deref(),
            Some("https://proxy.example.com/v1?api-version=1")
        );
        assert_eq!(
            zeta.other.get("env_key").and_then(toml::Value::as_str),
            Some("MY_PROVIDER_KEY")
        );
        assert_eq!(
            zeta.other
                .get("max_tokens")
                .and_then(toml::Value::as_integer),
            Some(8192)
        );
        assert!(matches!(
            zeta.other.get("created_at"),
            Some(toml::Value::Datetime(_))
        ));
        assert!(!zeta.other.contains_key("custom_bearer"));
        assert_eq!(warnings.len(), 2);
        assert!(!warnings.iter().any(|w| w.contains("secret")));

        let alpha = ConfigSection {
            model: Some("claude-opus-4".into()),
            enabled: Some(true),
            other: IndexMap::from([(
                "auth_mode".into(),
                toml::Value::String("subscription".into()),
            )]),
            ..Default::default()
        };

        let mut profiles = IndexMap::new();
        profiles.insert("zeta".into(), zeta);
        profiles.insert("alpha".into(), alpha);

        let rendered = render_exchange_v1(
            "claude",
            "2026-04-20T09:15:30Z",
            ExchangeSecretsMode::Redacted,
            &profiles,
        )
        .unwrap();

        let zeta_pos = rendered.find("[profiles.zeta]").unwrap();
        let alpha_pos = rendered.find("[profiles.alpha]").unwrap();
        assert!(zeta_pos < alpha_pos, "rendered:\n{rendered}");
        assert!(!rendered.contains("usage_count"));
        assert!(rendered.contains("created_at = 2026-04-20T09:15:30Z"));

        let imported = parse_import_document(&rendered, "claude").unwrap();
        assert_eq!(imported.source_format, ImportSourceFormat::ExchangeV1);
        assert_eq!(imported.secrets_mode, ExchangeSecretsMode::Redacted);
        let keys: Vec<&str> = imported.profiles.keys().map(String::as_str).collect();
        assert_eq!(keys, vec!["zeta", "alpha"]);
        assert!(matches!(
            imported.profiles["zeta"].other.get("created_at"),
            Some(toml::Value::Datetime(_))
        ));
    }

    #[test]
    fn parser_never_echoes_secret_on_syntax_error_and_rejects_masked_tokens() {
        let sentinel = "sk-live-SUPER-SECRET-UNCLOSED-9911";
        let broken = format!("[work]\nauth_token = \"{sentinel}\n");
        let err = parse_import_document(&broken, "claude").unwrap_err();
        assert_eq!(err.exit_code(), 14);
        let msg = err.to_string();
        assert!(!msg.contains(sentinel), "leaked sentinel: {msg}");
        assert!(msg.contains("第 2 行"), "missing line info: {msg}");

        for masked in ["********", "sk-ant-...1234", "REDACTED"] {
            let doc = format!(
                "[work]\nbase_url = \"https://api.example.com\"\nauth_token = \"{masked}\"\n"
            );
            let err = parse_import_document(&doc, "claude").unwrap_err();
            assert_eq!(err.exit_code(), 90);
            assert!(!err.to_string().contains(masked));
        }
    }

    #[test]
    fn parsed_import_document_debug_redacts_secrets_and_urls() {
        let token_sentinel = "sk-ant-debug-sentinel-4455";
        let grok_sentinel = "xai-debug-sentinel-6677";
        let url_sentinel = "https://user:urlpass99@proxy.example.com/v1?api_key=urlkey88";
        let doc = format!(
            "schema_version = 1\nplatform = \"grok\"\nexported_at = \"2026-04-20T09:15:30Z\"\n\n[profiles.work]\nbase_url = \"{url_sentinel}\"\nauth_token = \"{token_sentinel}\"\napi_key = \"{grok_sentinel}\"\n"
        );
        let parsed = parse_import_document(&doc, "grok").unwrap();
        let dbg = format!("{parsed:?}");
        assert!(dbg.contains("ParsedImportDocument"));
        assert!(dbg.contains("work"));
        assert!(!dbg.contains(token_sentinel), "leaked auth_token: {dbg}");
        assert!(!dbg.contains(grok_sentinel), "leaked api_key: {dbg}");
        assert!(!dbg.contains("urlpass99"), "leaked url password: {dbg}");
        assert!(!dbg.contains("urlkey88"), "leaked url query secret: {dbg}");
    }

    #[test]
    fn headerless_profiles_table_with_stray_top_level_table_fails_as_exchange_v1() {
        let malformed = r#"
[profiles.work]
model = "claude-sonnet-4-5"

[extra]
foo = "bar"
"#;
        let err = parse_import_document(malformed, "claude").unwrap_err();
        assert_eq!(err.exit_code(), 14, "expected ConfigFormatInvalid: {err}");
        assert!(matches!(err, CcrError::ConfigFormatInvalid(_)));
    }

    #[test]
    fn safe_canonical_toml_parse_error_never_echoes_secrets() {
        let secret = "sk-canonical-unclosed-secret-5544";
        let broken = format!("[work]\nauth_token = \"{secret}\n");
        let raw_err = toml::from_str::<toml::Table>(&broken).unwrap_err();
        let mapped = safe_canonical_toml_parse_error(&broken, &raw_err);
        assert_eq!(mapped.exit_code(), 10);
        let text = mapped.to_string();
        assert!(!text.contains(secret), "leaked secret: {text}");
        assert!(text.contains("第 2 行"), "expected line number: {text}");
    }

    #[test]
    fn scrub_catalog_toml_table_scrubs_secrets_inside_settings_table() {
        let secret = "sk-settings-secret-8899";
        let raw = format!(
            "default_config = \"work\"\ncurrent_config = \"work\"\n\n[settings]\nskip_confirmation = true\ntui_theme = \"ocean\"\napi_key = \"{secret}\"\n\n[work]\nbase_url = \"https://api.example.com\"\nauth_token = \"sk-work-secret\"\n"
        );
        let mut table: toml::Table = toml::from_str(&raw).unwrap();
        let mut warnings = Vec::new();
        scrub_catalog_toml_table(&mut table, &mut warnings);

        let rendered = toml::to_string(&table).unwrap();
        assert!(
            !rendered.contains(secret),
            "settings secret leaked: {rendered}"
        );
        assert!(!rendered.contains("sk-work-secret"));
        assert_eq!(table["settings"]["skip_confirmation"].as_bool(), Some(true));
        assert_eq!(table["settings"]["tui_theme"].as_str(), Some("ocean"));
        assert!(
            warnings.iter().any(|w| w.contains("settings.api_key")),
            "expected settings redaction warning: {warnings:?}"
        );
    }
}
