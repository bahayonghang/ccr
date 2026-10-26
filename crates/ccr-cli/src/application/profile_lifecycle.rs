//! Shared synchronous profile lifecycle. Adapters schedule this owner once.
//! No await, terminal IO, or detached work is permitted inside the operation.

use crate::managers::{
    HistoryEntry, HistoryManager, OperationDetails, OperationResult, OperationType,
};
use crate::models::{Platform, PlatformConfig, PlatformPaths, ProfileConfig};
use crate::platforms::create_platform;
use ccr_config::{
    CatalogTransaction, ConfigManager, increment_usage_count, load_applied_operation,
};
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

/// Concrete dependencies for one synchronous profile operation. Construction
/// resolves paths only; every read/write is performed by the caller's worker.
pub struct ProfileLifecycleContext {
    paths: PlatformPaths,
    instance: std::sync::Arc<dyn PlatformConfig>,
    lock_dir: PathBuf,
    history_file: PathBuf,
    pub(crate) claude_runtime: Option<ccr_config::ClaudeRuntimePaths>,
    pub(crate) codex_runtime_dirs: Vec<PathBuf>,
    codex_auth: Option<ccr_codex::services::CodexAuthService>,
}

impl ProfileLifecycleContext {
    pub fn new(platform: Platform) -> Result<Self> {
        let paths = PlatformPaths::new(platform)?;
        let lock_dir = LockManager::with_default_path()?.lock_dir().to_path_buf();
        let history_file = ccr_store::storage::Database::default_path()?;
        let instance = create_platform(platform)?;
        let claude_runtime = if platform == Platform::Claude {
            Some(ccr_config::ClaudeRuntimePaths::from_env()?)
        } else {
            None
        };
        let codex_runtime_dirs = if platform == Platform::Codex {
            ccr_codex::CodexPlatform::login_prep_codex_dirs()?
        } else {
            Vec::new()
        };
        let codex_auth = if platform == Platform::Codex {
            Some(ccr_codex::services::CodexAuthService::new()?)
        } else {
            None
        };
        Ok(Self {
            paths,
            instance,
            lock_dir,
            history_file,
            claude_runtime,
            codex_runtime_dirs,
            codex_auth,
        })
    }

    pub fn for_claude(
        paths: PlatformPaths,
        runtime: ccr_config::ClaudeRuntimePaths,
        lock_dir: PathBuf,
        history_file: PathBuf,
    ) -> Result<Self> {
        let settings = crate::managers::SettingsManager::new(
            &runtime.settings_file,
            &runtime.backups_dir,
            LockManager::new(&lock_dir),
        );
        let instance = std::sync::Arc::new(crate::platforms::ClaudePlatform::from_parts(
            paths.clone(),
            settings,
            lock_dir.clone(),
        ));
        Ok(Self {
            paths,
            instance,
            lock_dir,
            history_file,
            claude_runtime: Some(runtime),
            codex_runtime_dirs: Vec::new(),
            codex_auth: None,
        })
    }

    pub fn for_codex(
        paths: PlatformPaths,
        runtime_dir: PathBuf,
        lock_dir: PathBuf,
        history_file: PathBuf,
    ) -> Result<Self> {
        let manager = || {
            ccr_codex::CodexConfigManager::new(
                runtime_dir.join("config.toml"),
                runtime_dir.join("auth.json"),
                runtime_dir.join("backups"),
                LockManager::new(&lock_dir),
            )
        };
        let runtime = ccr_codex::services::CodexRuntimeService::from_parts(
            paths.clone(),
            runtime_dir.clone(),
            manager(),
        );
        let instance = std::sync::Arc::new(ccr_codex::CodexPlatform::from_parts(
            paths.clone(),
            manager(),
            runtime,
        ));
        let codex_auth = Some(ccr_codex::services::CodexAuthService::from_dirs(
            paths.platform_dir.clone(),
            runtime_dir.clone(),
        ));
        Ok(Self {
            paths,
            instance,
            lock_dir,
            history_file,
            claude_runtime: None,
            codex_runtime_dirs: vec![runtime_dir],
            codex_auth,
        })
    }

    pub fn for_grok(
        paths: PlatformPaths,
        runtime_config: PathBuf,
        lock_dir: PathBuf,
        history_file: PathBuf,
    ) -> Result<Self> {
        let instance = std::sync::Arc::new(crate::platforms::GrokPlatform::from_parts(
            paths.clone(),
            runtime_config,
        ));
        Ok(Self {
            paths,
            instance,
            lock_dir,
            history_file,
            claude_runtime: None,
            codex_runtime_dirs: Vec::new(),
            codex_auth: None,
        })
    }

    pub fn platform(&self) -> Platform {
        self.instance.platform_type()
    }
    pub fn paths(&self) -> &PlatformPaths {
        &self.paths
    }
    pub fn history_file(&self) -> &Path {
        &self.history_file
    }
    /// Rebuild the typed owner from the context's explicit dependencies only.
    pub fn codex_platform(&self) -> Result<ccr_codex::CodexPlatform> {
        if self.platform() != Platform::Codex {
            return Err(CcrError::PlatformNotSupported("codex".into()));
        }
        let runtime = self.instance.get_settings_path();
        let directory = runtime
            .parent()
            .ok_or_else(|| CcrError::ConfigError("Missing runtime directory".into()))?;
        let manager = || {
            ccr_codex::CodexConfigManager::new(
                directory.join("config.toml"),
                directory.join("auth.json"),
                directory.join("backups"),
                LockManager::new(&self.lock_dir),
            )
        };
        let service = ccr_codex::services::CodexRuntimeService::from_parts(
            self.paths.clone(),
            directory.to_path_buf(),
            manager(),
        );
        Ok(ccr_codex::CodexPlatform::from_parts(
            self.paths.clone(),
            manager(),
            service,
        ))
    }
    pub fn grok_platform(&self) -> Result<crate::platforms::GrokPlatform> {
        if self.platform() != Platform::Grok {
            return Err(CcrError::PlatformNotSupported("grok".into()));
        }
        Ok(crate::platforms::GrokPlatform::from_parts(
            self.paths.clone(),
            self.instance.get_settings_path(),
        ))
    }
    /// Explicit plaintext export. Callers must keep this value out of logs and
    /// display it only after the existing sensitive-output warning.
    pub fn export_profile_env(&self, name: &str, shell: bool) -> Result<String> {
        self.run(|| {
            let platform = self.codex_platform()?;
            if shell {
                platform.export_profile_shell_script(name)
            } else {
                serde_json::to_string_pretty(&platform.export_profile_env(name)?)
                    .map_err(|_| CcrError::ConfigError("Cannot encode profile environment".into()))
            }
        })
    }
    pub(crate) fn lock_dir(&self) -> &Path {
        &self.lock_dir
    }
    pub fn instance(&self) -> &dyn PlatformConfig {
        self.instance.as_ref()
    }
    pub fn manager(&self) -> ConfigManager {
        ConfigManager::new(&self.paths.profiles_file)
    }
    pub fn operation_lock(&self) -> Result<FileLock> {
        LockManager::new(&self.lock_dir).lock_resource(
            &format!("profile_application_{}", self.platform()),
            Duration::from_secs(10),
        )
    }
    pub fn run<T>(&self, operation: impl FnOnce() -> Result<T>) -> Result<T> {
        LockManager::with_scoped_directory(&self.lock_dir, operation)
    }
    pub fn apply(&self, request: ApplyProfileRequest) -> Result<ProfileOutcome> {
        self.check_request(&request)?;
        self.run(|| {
            let _operation = self.operation_lock()?;
            self.with_instance(|instance| apply_with_context(self, instance, request))
        })
    }
    pub fn enable(&self, request: ApplyProfileRequest) -> Result<ProfileOutcome> {
        self.check_request(&request)?;
        self.run(|| {
            let _operation = self.operation_lock()?;
            self.with_instance(|instance| enable_with_context(self, instance, request))
        })
    }
    pub fn update(
        &self,
        name: &str,
        target: &str,
        expected_version: Option<&str>,
        patch: impl FnOnce(&mut ProfileConfig) -> Result<()>,
    ) -> Result<ProfileOutcome> {
        self.run(|| {
            let _operation = self.operation_lock()?;
            self.with_instance(|instance| {
                update_with_context(self, instance, name, target, expected_version, patch)
            })
        })
    }
    /// Create through the platform owner so its auth normalization and secret
    /// storage remain authoritative. The catalog resource guard spans the
    /// baseline check and save; existing catalogs also join one SQL transaction.
    pub fn create(
        &self,
        name: &str,
        profile: ProfileConfig,
        expected_version: Option<&str>,
    ) -> Result<()> {
        self.run(|| {
            let _operation = self.operation_lock()?;
            self.with_instance(|instance| {
                ccr_config::managers::config::repository::validate_profile_name(name)?;
                let manager = self.manager();
                let _resource = manager.lock_mutation()?;
                let current = manager
                    .canonical_toml()?
                    .map(|(_, revision)| revision)
                    .unwrap_or_default();
                if expected_version.is_some_and(|expected| expected != current) {
                    return Err(CcrError::ValidationError(
                        "配置版本冲突，请重新读取后重试".into(),
                    ));
                }
                if instance.load_profiles()?.contains_key(name) {
                    return Err(CcrError::ValidationError("Profile already exists".into()));
                }
                instance.validate_profile(&profile)?;
                let files = operation_paths_with(&self.paths, self.platform(), instance)?;
                let journal = WriteJournal::begin(&files)?;
                let save = || {
                    instance.save_profile(name, &profile)?;
                    journal.verify()
                };
                let result = CatalogTransaction::execute_or_create(
                    manager.config_path(),
                    manager.profiles_db(),
                    Some(&current),
                    |_| save(),
                );
                match result {
                    Ok(()) => {
                        journal.commit();
                        Ok(())
                    }
                    Err(error) => {
                        let failed = journal.rollback();
                        if failed.is_empty() {
                            Err(error)
                        } else {
                            Err(CcrError::ConfigError(
                            "Profile creation requires recovery; external changes were retained"
                                .into(),
                        ))
                        }
                    }
                }
            })
        })
    }
    pub fn off(&self) -> Result<super::profile_off::ProfileOffResult> {
        self.run(|| super::profile_off::profile_off_with_context(self))
    }
    pub fn needs_login_prep(&self) -> Result<bool> {
        self.run(|| super::profile_off::needs_login_prep_with_context(self))
    }
    fn with_instance<T>(
        &self,
        operation: impl FnOnce(&dyn PlatformConfig) -> Result<T>,
    ) -> Result<T> {
        if self.platform() == Platform::Grok {
            self.grok_platform()?.with_locked(operation)
        } else {
            operation(self.instance())
        }
    }

    fn check_request(&self, request: &ApplyProfileRequest) -> Result<()> {
        if request.platform != self.platform() {
            return Err(CcrError::ValidationError(
                "Profile request belongs to another platform".into(),
            ));
        }
        Ok(())
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

#[cfg(any(test, feature = "test-support"))]
pub(crate) fn operation_paths(
    platform: Platform,
    instance: &dyn PlatformConfig,
) -> Result<Vec<PathBuf>> {
    let paths = PlatformPaths::new(platform)?;
    operation_paths_with(&paths, platform, instance)
}

fn operation_paths_with(
    paths: &PlatformPaths,
    platform: Platform,
    instance: &dyn PlatformConfig,
) -> Result<Vec<PathBuf>> {
    let mut files = vec![paths.registry_file.clone(), instance.get_settings_path()];
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

fn record_path(
    context: &ProfileLifecycleContext,
    request: &ApplyProfileRequest,
) -> Result<PathBuf> {
    uuid::Uuid::parse_str(&request.operation_id)
        .map_err(|_| CcrError::ValidationError("Invalid profile operation ID".into()))?;
    Ok(context
        .paths
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

fn history_manager(path: &Path) -> Result<HistoryManager> {
    // Match Database::init_default's parent preparation using the already bound
    // explicit location, including its immediate directory-failure behavior.
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    ccr_store::storage::Database::init(path).map(HistoryManager::new)
}

fn committed_record(outcome: &ProfileOutcome) -> bool {
    matches!(
        outcome.status,
        ProfileStatus::Applied | ProfileStatus::AppliedWithWarning
    )
}

fn same_request(outcome: &ProfileOutcome, request: &ApplyProfileRequest) -> bool {
    outcome.platform == request.platform.to_string()
        && outcome.profile == request.name
        && outcome.operation_id == request.operation_id
}

fn reject_foreign(outcome: &ProfileOutcome, request: &ApplyProfileRequest) -> Result<()> {
    if same_request(outcome, request) {
        Ok(())
    } else {
        Err(CcrError::ValidationError(
            "Profile operation ID belongs to another request".into(),
        ))
    }
}

fn replay(
    context: &ProfileLifecycleContext,
    path: &Path,
    request: &ApplyProfileRequest,
) -> Result<Option<ProfileOutcome>> {
    let recorded = match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice::<ProfileOutcome>(&bytes).ok(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(_) => {
            return Err(CcrError::FileIoError(
                "Cannot read profile operation record".into(),
            ));
        }
    };
    if let Some(outcome) = recorded.as_ref() {
        reject_foreign(outcome, request)?;
        if committed_record(outcome) {
            return Ok(Some(outcome.clone()));
        }
    }
    let manager = context.manager();
    let Some(payload) = load_applied_operation(manager.profiles_db(), &request.operation_id)?
    else {
        return Ok(None);
    };
    let outcome: ProfileOutcome = serde_json::from_str(&payload).map_err(|_| {
        CcrError::ConfigError("Invalid profile operation record; recovery required".into())
    })?;
    reject_foreign(&outcome, request)?;
    Ok(Some(outcome))
}

/// Import legacy profile documents before a CLI, TUI, or desktop command runs.
pub fn migrate_profile_catalogs() -> Result<()> {
    ccr_config::migrate_catalogs()
}

/// CLI, TUI, and desktop adapters call this same synchronous owner.
pub fn apply_profile(request: ApplyProfileRequest) -> Result<ProfileOutcome> {
    ProfileLifecycleContext::new(request.platform)?.apply(request)
}

fn apply_with_context(
    context: &ProfileLifecycleContext,
    instance: &dyn PlatformConfig,
    request: ApplyProfileRequest,
) -> Result<ProfileOutcome> {
    let path = record_path(context, &request)?;
    if let Some(outcome) = replay(context, &path, &request)? {
        return Ok(outcome);
    }
    validate_target(instance, &request.name)?;
    if let Some(outcome) = replay(context, &path, &request)? {
        return Ok(outcome);
    }
    let version = context.manager().snapshot()?.version;
    validate_target(instance, &request.name)?;
    execute(context, &request, instance, path, true, version, || {
        instance.apply_profile(&request.name)
    })
}

/// Persist the enabled policy and activate within one compensation boundary.
/// Disabled targets are allowed here; every other platform rule remains required.
pub fn enable_profile(request: ApplyProfileRequest) -> Result<ProfileOutcome> {
    ProfileLifecycleContext::new(request.platform)?.enable(request)
}

fn enable_with_context(
    context: &ProfileLifecycleContext,
    instance: &dyn PlatformConfig,
    request: ApplyProfileRequest,
) -> Result<ProfileOutcome> {
    let path = record_path(context, &request)?;
    if let Some(outcome) = replay(context, &path, &request)? {
        return Ok(outcome);
    }
    if let Some(outcome) = replay(context, &path, &request)? {
        return Ok(outcome);
    }
    let manager = context.manager();
    let snapshot = manager.snapshot()?;
    let mut profile = instance
        .load_profiles()?
        .get(&request.name)
        .cloned()
        .ok_or_else(|| CcrError::ProfileNotFound(request.name.clone()))?;
    profile.enabled = Some(true);
    instance.validate_profile(&profile)?;
    execute(
        context,
        &request,
        instance,
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
    context: &ProfileLifecycleContext,
    request: &ApplyProfileRequest,
    instance: &dyn PlatformConfig,
    path: PathBuf,
    activation: bool,
    expected_profile: String,
    mutate: impl FnOnce() -> Result<()>,
) -> Result<ProfileOutcome> {
    let root = &context.paths.root;
    let previous = instance.get_current_profile()?;
    let mut paths = operation_paths_with(&context.paths, request.platform, instance)?;
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
    // Interrupted is not a committed success. The same operation id re-executes
    // unless the catalog row written in the committed transaction is present.
    save_record(&path, &outcome)?;
    paths.push(path.clone());
    let journal = WriteJournal::begin(&paths)?;
    let manager = context.manager();
    let database = manager.profiles_db().to_path_buf();
    if manager.snapshot()?.version != expected_profile {
        let _failed = journal.rollback();
        return Err(CcrError::ValidationError(
            "配置版本冲突，请重新读取后重试".into(),
        ));
    }
    let mut stored = outcome.clone();
    stored.activation_committed = activation;
    stored.status = ProfileStatus::AppliedWithWarning;
    stored.warnings = vec![ProfileWarning::AncillaryPending];
    stored.recovery_paths.clear();
    let payload = serde_json::to_string(&stored)
        .map_err(|_| CcrError::ConfigError("Cannot encode profile operation".into()))?;
    let applied = CatalogTransaction::execute(&database, |transaction| {
        let revision = transaction.current_revision()?;
        if revision != expected_profile {
            return Err(CcrError::ValidationError(
                "配置版本冲突，请重新读取后重试".into(),
            ));
        }
        mutate()?;
        // Re-check every declared runtime file after the last writer. A change
        // during a later file must fail before the catalog transaction commits.
        journal.verify()?;
        transaction.store_applied_operation(&request.operation_id, &payload)?;
        Ok(())
    });
    if let Err(error) = applied {
        let failed = journal.rollback();
        if matches!(
            &error,
            CcrError::ValidationError(message)
                if message == "配置版本冲突，请重新读取后重试"
        ) {
            return Err(error);
        }
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
    // The catalog commit is durable before the file preimages are dropped.
    // Replays return the saved outcome and do not repeat activation or the count.
    outcome = stored;
    if save_record(&path, &outcome).is_err() {
        outcome.warnings.push(ProfileWarning::OperationRecordFailed);
        return Ok(outcome);
    }
    outcome.warnings.clear();
    if activation && increment_usage_count(&database, &request.name).is_err() {
        outcome.warnings.push(ProfileWarning::UsageCountFailed);
    }
    let history = history_manager(&context.history_file).and_then(|manager| {
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
            root,
            &request.platform.to_string(),
            &request.name,
        )
        .is_err()
    {
        outcome.warnings.push(ProfileWarning::AnalyticsFailed);
    }
    if activation
        && let Some(service) = &context.codex_auth
        && service.sync_current_auth_registry().is_err()
    {
        outcome.warnings.push(ProfileWarning::AuthRegistryFailed);
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
    ProfileLifecycleContext::new(platform)?.update(name, target, None, patch)
}

/// Save a draft using its original catalog revision, checked inside the transaction.
pub fn update_profile_versioned(
    platform: Platform,
    name: &str,
    target: &str,
    expected_version: &str,
    patch: impl FnOnce(&mut ProfileConfig) -> Result<()>,
) -> Result<ProfileOutcome> {
    ProfileLifecycleContext::new(platform)?.update(name, target, Some(expected_version), patch)
}

fn update_with_context(
    context: &ProfileLifecycleContext,
    instance: &dyn PlatformConfig,
    name: &str,
    target: &str,
    expected_version: Option<&str>,
    patch: impl FnOnce(&mut ProfileConfig) -> Result<()>,
) -> Result<ProfileOutcome> {
    let platform = context.platform();
    let snapshot = context.manager().snapshot()?;
    let version = expected_version.unwrap_or(&snapshot.version).to_owned();
    if version != snapshot.version {
        return Err(CcrError::ValidationError(
            "配置版本冲突，请重新读取后重试".into(),
        ));
    }
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
        context,
        &request,
        instance,
        record_path(context, &request)?,
        active,
        version,
        || {
            // Seed the exact TOML section before the JSON projection is patched.
            // The platform repository merges the delta and retains TOML datetimes.
            if target != name {
                context.manager().mutate(|config| {
                    let original = config.get_section(name)?.clone();
                    config.sections.insert(target.into(), original);
                    Ok(())
                })?;
            }
            instance.save_profile(target, &profile)?;
            if target != name {
                context.manager().mutate(|config| {
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

#[cfg(test)]
#[path = "profile_lifecycle_tests.rs"]
mod injected_tests;
