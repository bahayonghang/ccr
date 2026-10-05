//! Shared synchronous profile lifecycle. Adapters schedule this owner once.
//! No await, terminal IO, or detached work is permitted inside the operation.

use crate::managers::{
    HistoryEntry, HistoryManager, OperationDetails, OperationResult, OperationType,
};
use crate::models::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
use crate::platforms::create_platform;
use ccr_config::ConfigManager;
use ccr_core::core::guarded_write::write_guarded;
use ccr_core::core::write_journal::WriteJournal;
use ccr_core::core::{FileLock, LockManager, WriteOptions};
use ccr_core::{CcrError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/profiles/")
)]
pub enum ProfileStatus {
    Unchanged,
    Applied,
    AppliedWithWarning,
    RecoveryRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/profiles/")
)]
pub enum ProfileWarning {
    UsageCountFailed,
    HistoryFailed,
    AuthRegistryFailed,
    AnalyticsFailed,
    OperationRecordFailed,
    AncillaryPending,
    ActivationFailed,
    Interrupted,
}

/// Contains identifiers and status only. Never include profile data or IO errors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(
    feature = "ts",
    ts(export, export_to = "../../../ccr-ui/src/types/generated/profiles/")
)]
pub struct ProfileOutcome {
    pub operation_id: String,
    pub platform: String,
    pub profile: String,
    pub previous_profile: Option<String>,
    pub status: ProfileStatus,
    pub activation_committed: bool,
    pub warnings: Vec<ProfileWarning>,
    pub recovery_paths: Vec<String>,
}

impl ProfileOutcome {
    pub fn message(&self) -> &'static str {
        match self.status {
            ProfileStatus::Applied if !self.activation_committed => "Profile updated",
            ProfileStatus::Applied => "Profile applied",
            ProfileStatus::AppliedWithWarning => {
                "Profile operation committed; ancillary work needs attention. Do not repeat activation."
            }
            ProfileStatus::RecoveryRequired => {
                "Profile operation requires recovery. External changes were retained."
            }
            ProfileStatus::Unchanged if self.activation_committed => "Profile is already applied",
            ProfileStatus::Unchanged => "Profile was not applied. Previous files were restored.",
        }
    }
}

#[derive(Clone)]
pub struct ApplyProfileRequest {
    pub platform: Platform,
    pub name: String,
    pub operation_id: String,
}

impl ApplyProfileRequest {
    pub fn new(platform: Platform, name: impl Into<String>) -> Self {
        Self {
            platform,
            name: name.into(),
            operation_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

/// Direct repository adapters use the same operation lock for active-profile
/// checks followed by mutation. Keep the guard inside one synchronous worker;
/// never call apply, enable, or update_profile while already holding this lock.
pub fn operation_lock(platform: Platform) -> Result<FileLock> {
    LockManager::with_default_path()?.lock_resource(
        &format!("profile_application_{platform}"),
        Duration::from_secs(10),
    )
}

fn validate_target(instance: &dyn PlatformConfig, name: &str) -> Result<ProfileConfig> {
    let profiles = instance.load_profiles()?;
    let profile = profiles
        .get(name)
        .ok_or_else(|| CcrError::ProfileNotFound(name.to_owned()))?;
    if !profile.is_enabled() {
        return Err(CcrError::ValidationError(
            "Disabled profile cannot be activated".into(),
        ));
    }
    instance.validate_profile(profile)?;
    Ok(profile.clone())
}

pub(crate) fn operation_paths(
    platform: Platform,
    instance: &dyn PlatformConfig,
) -> Result<Vec<PathBuf>> {
    let paths = PlatformPaths::new(platform)?;
    let mut files = vec![
        paths.profiles_file,
        paths.registry_file,
        instance.get_settings_path(),
    ];
    if platform == Platform::Codex {
        let dir = instance
            .get_settings_path()
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| CcrError::ConfigError("Missing runtime directory".into()))?;
        files.extend([
            dir.join("auth.json"),
            paths.platform_dir.join("profile_secrets.json"),
            paths.platform_dir.join("profile_entry_auth_state.json"),
        ]);
    }
    if platform == Platform::Grok {
        files.push(paths.platform_dir.join("profile_entry_config_state.json"));
    }
    Ok(files)
}

fn record_path(request: &ApplyProfileRequest) -> Result<PathBuf> {
    uuid::Uuid::parse_str(&request.operation_id)
        .map_err(|_| CcrError::ValidationError("Invalid profile operation ID".into()))?;
    Ok(PlatformPaths::new(request.platform)?
        .platform_dir
        .join("profile-operations")
        .join(format!("{}.json", request.operation_id)))
}

fn save_record(path: &Path, outcome: &ProfileOutcome) -> Result<()> {
    let bytes = serde_json::to_vec(outcome)
        .map_err(|_| CcrError::ConfigError("Cannot encode profile operation".into()))?;
    write_guarded(
        path,
        &bytes,
        &WriteOptions {
            secret: true,
            ..Default::default()
        },
    )
}

fn replay(path: &Path, request: &ApplyProfileRequest) -> Result<Option<ProfileOutcome>> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => {
            return Err(CcrError::FileIoError(
                "Cannot read profile operation record".into(),
            ));
        }
    };
    let outcome: ProfileOutcome = serde_json::from_slice(&bytes).map_err(|_| {
        CcrError::ConfigError("Invalid profile operation record; recovery required".into())
    })?;
    if outcome.platform != request.platform.to_string()
        || outcome.profile != request.name
        || outcome.operation_id != request.operation_id
    {
        return Err(CcrError::ValidationError(
            "Profile operation ID belongs to another request".into(),
        ));
    }
    Ok(Some(outcome))
}

/// CLI, TUI, and desktop adapters call this same synchronous owner.
pub fn apply_profile(request: ApplyProfileRequest) -> Result<ProfileOutcome> {
    let path = record_path(&request)?;
    if let Some(outcome) = replay(&path, &request)? {
        return Ok(outcome);
    }
    let instance = create_platform(request.platform)?;
    validate_target(instance.as_ref(), &request.name)?;
    let _operation = operation_lock(request.platform)?;
    if let Some(outcome) = replay(&path, &request)? {
        return Ok(outcome);
    }
    let version = ConfigManager::for_platform(&request.platform.to_string())?
        .snapshot()?
        .version;
    validate_target(instance.as_ref(), &request.name)?;
    execute(&request, instance.as_ref(), path, true, version, || {
        instance.apply_profile(&request.name)
    })
}

/// Persist the enabled policy and activate within one compensation boundary.
/// Disabled targets are allowed here; every other platform rule remains required.
pub fn enable_profile(request: ApplyProfileRequest) -> Result<ProfileOutcome> {
    let path = record_path(&request)?;
    if let Some(outcome) = replay(&path, &request)? {
        return Ok(outcome);
    }
    let instance = create_platform(request.platform)?;
    let _operation = operation_lock(request.platform)?;
    if let Some(outcome) = replay(&path, &request)? {
        return Ok(outcome);
    }
    let manager = ConfigManager::for_platform(&request.platform.to_string())?;
    let snapshot = manager.snapshot()?;
    let mut profile = instance
        .load_profiles()?
        .get(&request.name)
        .cloned()
        .ok_or_else(|| CcrError::ProfileNotFound(request.name.clone()))?;
    profile.enabled = Some(true);
    instance.validate_profile(&profile)?;
    execute(
        &request,
        instance.as_ref(),
        path,
        true,
        snapshot.version,
        || {
            manager.mutate(|config| {
                config.get_section_mut(&request.name)?.enabled = Some(true);
                Ok(())
            })?;
            instance.apply_profile(&request.name)
        },
    )
}

fn execute(
    request: &ApplyProfileRequest,
    instance: &dyn PlatformConfig,
    path: PathBuf,
    activation: bool,
    expected_profile: String,
    mutate: impl FnOnce() -> Result<()>,
) -> Result<ProfileOutcome> {
    let root = PlatformPaths::new(request.platform)?.root;
    let previous = instance.get_current_profile()?;
    let mut paths = operation_paths(request.platform, instance)?;
    let mut outcome = ProfileOutcome {
        operation_id: request.operation_id.clone(),
        platform: request.platform.to_string(),
        profile: request.name.clone(),
        previous_profile: previous,
        status: ProfileStatus::RecoveryRequired,
        activation_committed: false,
        warnings: vec![ProfileWarning::Interrupted],
        recovery_paths: paths.iter().map(|p| p.display().to_string()).collect(),
    };
    // A process interruption leaves an explicit recovery marker, never a retryable success.
    save_record(&path, &outcome)?;
    paths.push(path.clone());
    let journal = WriteJournal::begin(&paths)?;
    journal.expect_version(
        ConfigManager::for_platform(&request.platform.to_string())?.config_path(),
        expected_profile,
    )?;
    let applied = journal
        .verify()
        .and_then(|()| mutate())
        .and_then(|()| journal.verify())
        .and_then(|()| {
            outcome.activation_committed = activation;
            outcome.status = ProfileStatus::AppliedWithWarning;
            outcome.warnings = vec![ProfileWarning::AncillaryPending];
            outcome.recovery_paths.clear();
            save_record(&path, &outcome)
        });
    if applied.is_err() {
        let failed = journal.rollback();
        outcome.activation_committed = false;
        outcome.status = if failed.is_empty() {
            ProfileStatus::Unchanged
        } else {
            ProfileStatus::RecoveryRequired
        };
        outcome.warnings = vec![ProfileWarning::ActivationFailed];
        outcome.recovery_paths = failed.iter().map(|p| p.display().to_string()).collect();
        if save_record(&path, &outcome).is_err() {
            outcome.warnings.push(ProfileWarning::OperationRecordFailed);
        }
        return Ok(outcome);
    }
    journal.commit();
    // Activation is durable before success side effects. Replays return the saved
    // outcome and never repeat activation or partially completed side effects.
    outcome.warnings.clear();
    if activation
        && ConfigManager::for_platform(&request.platform.to_string())
            .and_then(|manager| {
                manager.mutate(|config| {
                    let profile = config
                        .sections
                        .get_mut(&request.name)
                        .ok_or_else(|| CcrError::ProfileNotFound(request.name.clone()))?;
                    profile.usage_count = Some(profile.usage_count.unwrap_or(0).saturating_add(1));
                    Ok(())
                })
            })
            .is_err()
    {
        outcome.warnings.push(ProfileWarning::UsageCountFailed);
    }
    let history = HistoryManager::with_default().and_then(|manager| {
        let mut entry = HistoryEntry::new(
            if activation {
                OperationType::Switch
            } else {
                OperationType::Update
            },
            OperationDetails {
                from_config: outcome.previous_profile.clone(),
                to_config: Some(request.name.clone()),
                backup_path: None,
                extra: Some(request.platform.to_string()),
            },
            OperationResult::Success,
        );
        entry.id = request.operation_id.clone();
        manager.add(entry)
    });
    if history.is_err() {
        outcome.warnings.push(ProfileWarning::HistoryFailed);
    }
    if activation
        && ccr_config::managers::provider_activation::try_record_activation(
            &root,
            &request.platform.to_string(),
            &request.name,
        )
        .is_err()
    {
        outcome.warnings.push(ProfileWarning::AnalyticsFailed);
    }
    if activation && request.platform == Platform::Codex {
        let synced = ccr_codex::services::CodexAuthService::new()
            .and_then(|service| service.sync_current_auth_registry());
        if synced.is_err() {
            outcome.warnings.push(ProfileWarning::AuthRegistryFailed);
        }
    }
    outcome.status = if outcome.warnings.is_empty() {
        ProfileStatus::Applied
    } else {
        ProfileStatus::AppliedWithWarning
    };
    if save_record(&path, &outcome).is_err() {
        outcome.status = ProfileStatus::AppliedWithWarning;
        outcome.warnings.push(ProfileWarning::OperationRecordFailed);
    }
    Ok(outcome)
}

/// Update/rename under the same lifecycle and compensation boundary as apply.
/// The mutator receives the current profile while the operation lock is held.
pub fn update_profile(
    platform: Platform,
    name: &str,
    target: &str,
    patch: impl FnOnce(&mut ProfileConfig) -> Result<()>,
) -> Result<ProfileOutcome> {
    let instance = create_platform(platform)?;
    let _operation = operation_lock(platform)?;
    let version = ConfigManager::for_platform(&platform.to_string())?
        .snapshot()?
        .version;
    let profiles = instance.load_profiles()?;
    let mut profile = profiles
        .get(name)
        .cloned()
        .ok_or_else(|| CcrError::ProfileNotFound(name.into()))?;
    if target.trim().is_empty()
        || matches!(target, "default_config" | "current_config" | "settings")
        || (target != name && profiles.contains_key(target))
    {
        return Err(CcrError::ValidationError(
            "Invalid or existing profile target".into(),
        ));
    }
    patch(&mut profile)?;
    instance.validate_profile(&profile)?;
    let active = target != name && instance.get_current_profile()?.as_deref() == Some(name);
    if active && !profile.is_enabled() {
        return Err(CcrError::ValidationError(
            "Active profile cannot be disabled during rename".into(),
        ));
    }
    let request = ApplyProfileRequest::new(platform, target);
    execute(
        &request,
        instance.as_ref(),
        record_path(&request)?,
        active,
        version,
        || {
            // Seed the exact TOML section before the JSON projection is patched.
            // The platform repository merges the delta and retains TOML datetimes.
            if target != name {
                ConfigManager::for_platform(&platform.to_string())?.mutate(|config| {
                    let original = config.get_section(name)?.clone();
                    config.sections.insert(target.into(), original);
                    Ok(())
                })?;
            }
            instance.save_profile(target, &profile)?;
            if target != name {
                ConfigManager::for_platform(&platform.to_string())?.mutate(|config| {
                    if config.current_config == name {
                        config.current_config = target.into();
                    }
                    if config.default_config == name {
                        config.default_config = target.into();
                    }
                    Ok(())
                })?;
                instance.delete_profile(name)?;
            }
            if active {
                instance.apply_profile(target)?;
            }
            Ok(())
        },
    )
}
