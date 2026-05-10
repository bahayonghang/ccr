//! llmusage integration boundary for CCR Desktop.
//!
//! The Vue/Tauri usage contract stays owned by ccr-ui. This module only owns
//! runtime initialization and stable filter/source normalization for the
//! llmusage 0.5.x library surface.

#[cfg(test)]
use std::ffi::OsString;
#[cfg(test)]
use std::sync::Mutex;

use chrono::NaiveDate;
use llmusage::models::SourceKind;
use llmusage::store::{BootstrapOptions, Store};
use llmusage::sync::JobRegistry;
use llmusage::{AppPaths, Dashboard, QueryFilter, ReportTimezone};

pub mod queries;

#[derive(Debug, Clone)]
pub struct LlmusageHandle {
    store: Store,
    jobs: JobRegistry,
}

impl LlmusageHandle {
    pub fn init() -> Result<Self, String> {
        Self::init_with_paths(discover_llmusage_paths()?)
    }

    pub(crate) fn init_with_paths(paths: AppPaths) -> Result<Self, String> {
        let store = Store::new(&paths)
            .map_err(|error| format!("Initialize llmusage store failed: {error}"))?;
        store
            .bootstrap_with(BootstrapOptions::default().with_raw_archive(true))
            .map_err(|error| format!("Bootstrap llmusage store failed: {error}"))?;

        Ok(Self {
            store,
            jobs: JobRegistry::default(),
        })
    }

    pub fn dashboard(&self) -> Result<Dashboard, llmusage::LlmusageError> {
        Dashboard::open(&self.store)
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    pub fn jobs(&self) -> &JobRegistry {
        &self.jobs
    }
}

pub fn discover_llmusage_paths() -> Result<AppPaths, String> {
    AppPaths::discover().map_err(|error| format!("Discover llmusage paths failed: {error}"))
}

pub fn build_filter(
    platform: Option<String>,
    model: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<QueryFilter, String> {
    Ok(QueryFilter {
        source: platform.as_deref().and_then(parse_source_filter),
        model: model.and_then(non_empty_string),
        since: parse_optional_date(start_date.as_deref(), "start_date")?,
        until: parse_optional_date(end_date.as_deref(), "end_date")?,
        project_hash: None,
        timezone: ReportTimezone::Local,
    })
}

pub fn canonical_source_id(raw: Option<&str>) -> Option<String> {
    raw.and_then(parse_source_filter)
        .map(|source| source.as_str().to_string())
}

pub fn platform_scope_label(raw: Option<&str>) -> String {
    canonical_source_id(raw).unwrap_or_else(|| "all".to_string())
}

fn parse_source_filter(raw: &str) -> Option<SourceKind> {
    match raw.trim().to_lowercase().as_str() {
        "" | "all" | "*" => None,
        "claude" | "claude-code" | "claude code" => Some(SourceKind::Claude),
        "codex" | "openai-codex" | "openai codex" => Some(SourceKind::Codex),
        "gemini" | "gemini-cli" | "gemini cli" | "google-gemini" | "google gemini" => {
            Some(SourceKind::Gemini)
        }
        "opencode" | "open-code" | "open code" => Some(SourceKind::Opencode),
        value => SourceKind::parse_id(value),
    }
}

fn parse_optional_date(value: Option<&str>, label: &str) -> Result<Option<NaiveDate>, String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(Some)
        .map_err(|error| format!("Invalid {label} '{value}': {error}"))
}

fn non_empty_string(value: String) -> Option<String> {
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn restore_env(key: &str, value: Option<OsString>) {
        unsafe {
            if let Some(value) = value {
                std::env::set_var(key, value);
            } else {
                std::env::remove_var(key);
            }
        }
    }

    #[test]
    fn discovers_llmusage_home_override() {
        let _guard = ENV_LOCK.lock().expect("env lock should be available");
        let temp = TempDir::new().expect("temp dir should be created");
        let saved = std::env::var_os("LLMUSAGE_HOME");
        unsafe { std::env::set_var("LLMUSAGE_HOME", temp.path()) };

        let paths = discover_llmusage_paths().expect("LLMUSAGE_HOME should be discovered");
        assert_eq!(paths.root_dir, temp.path());
        assert_eq!(paths.db_path, temp.path().join("llmusage.db"));

        restore_env("LLMUSAGE_HOME", saved);
    }

    #[test]
    fn default_discovery_uses_llmusage_root_not_legacy_ccr_root() {
        let _guard = ENV_LOCK.lock().expect("env lock should be available");
        let temp_home = TempDir::new().expect("temp home should be created");
        let saved_llmusage = std::env::var_os("LLMUSAGE_HOME");
        let saved_home = std::env::var_os("HOME");
        let saved_userprofile = std::env::var_os("USERPROFILE");
        unsafe {
            std::env::remove_var("LLMUSAGE_HOME");
            std::env::set_var("HOME", temp_home.path());
            std::env::set_var("USERPROFILE", temp_home.path());
        }

        let paths = discover_llmusage_paths().expect("default llmusage root should be discovered");
        assert_eq!(paths.root_dir, temp_home.path().join(".llmusage"));
        assert_ne!(
            paths.root_dir,
            temp_home.path().join(".ccr").join("llmusage")
        );

        restore_env("LLMUSAGE_HOME", saved_llmusage);
        restore_env("HOME", saved_home);
        restore_env("USERPROFILE", saved_userprofile);
    }

    #[test]
    fn init_bootstraps_explicit_root_without_using_legacy_ccr_layout() {
        let temp = TempDir::new().expect("temp root should be created");
        let paths = AppPaths::with_root(temp.path().to_path_buf())
            .expect("explicit llmusage root should be valid");

        let handle = LlmusageHandle::init_with_paths(paths)
            .expect("explicit llmusage root should initialize");

        assert_eq!(handle.store().paths.root_dir, temp.path());
        assert!(temp.path().join("llmusage.db").is_file());
        assert!(!temp.path().join("llmusage").join("llmusage.db").exists());
    }

    #[test]
    fn source_filter_accepts_ccr_aliases() {
        assert_eq!(
            canonical_source_id(Some("Claude Code")).as_deref(),
            Some("claude")
        );
        assert_eq!(
            canonical_source_id(Some("openai-codex")).as_deref(),
            Some("codex")
        );
        assert_eq!(
            canonical_source_id(Some("gemini-cli")).as_deref(),
            Some("gemini")
        );
        assert_eq!(
            canonical_source_id(Some("Open Code")).as_deref(),
            Some("opencode")
        );
        assert_eq!(canonical_source_id(Some("all")), None);
    }

    #[test]
    fn build_filter_rejects_invalid_dates() {
        let error = build_filter(None, None, Some("2026/05/09".to_string()), None)
            .expect_err("invalid date should be rejected");
        assert!(error.contains("start_date"));
    }
}
