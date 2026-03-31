//! Shared CCR configuration contracts and platform profile helpers.

pub mod managers;
pub mod models;
pub mod platforms;
pub mod sync;

pub use ccr_core::{AutoCompletable, CcrError, Result, Validatable};
pub use managers::{
    CcsConfig, ConfigSection, GlobalSettings, PlatformConfigEntry, PlatformConfigManager,
    ProviderType, SyncConfig, SyncConfigManager, UnifiedConfig,
};
pub use models::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
pub use platforms::base::{
    get_current_profile_from_registry, load_profiles_from_toml, profile_to_section,
    reconcile_registry_current_profile_after_delete, save_profiles_to_toml, section_to_profile,
    update_current_config, update_registry_current_profile,
};
