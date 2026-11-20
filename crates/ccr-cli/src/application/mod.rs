pub mod auth_off;
pub mod codex_oauth;
pub mod platform_roots;
pub mod platform_switch;
pub mod profile_activity;
#[cfg(any(test, feature = "test-support"))]
pub mod profile_contract;
pub mod profile_create;
pub mod profile_edit;
pub mod profile_export;
pub(crate) mod profile_fields;
pub mod profile_import;
pub mod profile_lifecycle;
pub mod profile_observation;
pub mod profile_off;
pub mod profile_switch;
pub mod types;

pub use auth_off::{
    AuthOffPath, AuthOffResult, auth_off_for_platform, codex_local_auth_off, needs_auth_off,
};
pub use platform_switch::switch_platform;
pub use profile_create::{
    CreateProfileOutcome, CreateProfileRequest, CreateSecret, ProfileFieldValue, create_profile,
    create_profile_with_context,
};
pub use profile_export::{
    ExportProfilesJsonReport, ExportProfilesOutcome, ExportProfilesRequest, export_profiles,
    export_profiles_with_context,
};
pub(crate) use profile_export::{
    enrich_missing_secret_enable_error, is_credential_pending_section,
    overlay_codex_secret_if_present, platform_secret_stdin_flag,
};
pub use profile_import::{
    ConflictStrategy, ImportModeKind, ImportProfilesRequest, ProfileImportCounts,
    ProfileImportReport, RenamedProfileItem, SkippedDuplicateItem, commit_import,
    commit_import_with_context, preview_import, preview_import_with_context,
};
pub use profile_lifecycle::{ProfileLifecycleContext, migrate_profile_catalogs};
pub use profile_off::{ProfileOffResult, needs_login_prep, profile_off_for_platform};
pub use types::SwitchPlatformRequest;
