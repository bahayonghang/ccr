//! Configuration list orchestration shared by the native and Tauri adapters.

use ccr_config::platforms::base;
use ccr_config::{ConfigManager, Platform, PlatformPaths};
use ccr_core::Result;
use std::path::PathBuf;

pub mod backups;
pub mod mutations;
pub mod profile_patch;
pub mod source;

pub use ccr_core::CcrError as ConfigReadError;
pub type ConfigReadResult = std::result::Result<Vec<ConfigRow>, ConfigReadError>;

/// A display projection. Credentials have already passed through `Secret::Display`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigRow {
    pub version: String,
    pub name: String,
    pub description: String,
    pub base_url: String,
    pub auth_token: String,
    pub model: Option<String>,
    pub small_fast_model: Option<String>,
    pub is_current: bool,
    pub is_default: bool,
    pub provider: Option<String>,
    pub provider_type: Option<String>,
    pub account: Option<String>,
    pub tags: Option<Vec<String>>,
    pub usage_count: u64,
    pub enabled: bool,
}

/// Path-only construction; `list` must run on a blocking worker.
#[derive(Clone)]
pub struct ConfigReadService {
    profiles_path: PathBuf,
    registry_path: PathBuf,
}

impl ConfigReadService {
    pub fn for_claude() -> Result<Self> {
        let paths = PlatformPaths::new(Platform::Claude)?;
        Ok(Self::from_paths(paths.profiles_file, paths.registry_file))
    }

    pub fn from_paths(profiles_path: PathBuf, registry_path: PathBuf) -> Self {
        Self {
            profiles_path,
            registry_path,
        }
    }

    /// Pure query: never initialize, migrate, repair markers, or acquire a write lock.
    pub fn list(&self) -> Result<Vec<ConfigRow>> {
        let snapshot = ConfigManager::new(&self.profiles_path).snapshot()?;
        let current = base::read_file_current_profile_with_paths(
            &self.profiles_path,
            &self.registry_path,
            "claude",
        )?
        .current;
        let config = snapshot.config;
        Ok(config
            .sections
            .iter()
            .map(|(name, section)| ConfigRow {
                version: snapshot.version.clone(),
                name: name.clone(),
                description: section.description.clone().unwrap_or_default(),
                base_url: section.base_url.clone().unwrap_or_default(),
                auth_token: section
                    .auth_token
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                model: section.model.clone(),
                small_fast_model: section.small_fast_model.clone(),
                is_current: current.as_deref() == Some(name.as_str()),
                is_default: name == &config.default_config,
                provider: section.provider.clone(),
                provider_type: section
                    .provider_type
                    .as_ref()
                    .map(|kind| kind.to_string_value().to_string()),
                account: section.account.clone(),
                tags: section.tags.clone(),
                usage_count: u64::from(section.usage_count()),
                enabled: section.is_enabled(),
            })
            .collect())
    }
}
