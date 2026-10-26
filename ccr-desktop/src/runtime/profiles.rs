//! Typed native requests. Domain owners retain catalog, journal, CAS and backup rules.
use super::{ConfigSource, Services};
use ccr_app::{
    config::{
        ConfigRow,
        source::{self, SourceSaveOutcome},
    },
    runtime::{
        AdmissionError, Completion, EnvironmentId, Platform as RequestPlatform, RequestKey,
        TaskSubscription,
    },
};
use ccr_cli::application::profile_lifecycle::{
    ApplyProfileRequest, ProfileLifecycleContext, ProfileOutcome,
};
use ccr_config::{
    ClaudeRuntimePaths, ConfigPatch, ConfigService, FieldPatch, Platform, PlatformPaths,
};
use ccr_core::{CcrError, Result};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileError {
    Conflict,
    Invalid,
    Missing,
    Storage,
    Unsupported,
}
impl ProfileError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Conflict => {
                "Catalog changed. Your draft is retained. Reload or cancel. / 配置已变化，草稿已保留，请重新读取或取消"
            }
            Self::Invalid => {
                "Invalid profile or action. Check fields and active-profile policy. / 配置或操作无效，请检查字段和当前配置保护规则"
            }
            Self::Missing => "Catalog or profile not found. / 配置目录或配置不存在",
            Self::Storage => {
                "Operation failed. Check file access, backup and recovery state before retrying. / 操作失败，请检查访问权限、备份及恢复状态后重试"
            }
            Self::Unsupported => {
                "This environment or platform is not supported. / 此环境或平台不支持该操作"
            }
        }
    }
}
fn safe_error(error: CcrError) -> ProfileError {
    match error {
        CcrError::ValidationError(ref message) if message.contains("版本冲突") => {
            ProfileError::Conflict
        }
        CcrError::ValidationError(_)
        | CcrError::ConfigFormatInvalid(_)
        | CcrError::TomlError(_) => ProfileError::Invalid,
        CcrError::ConfigMissing(_)
        | CcrError::ProfileNotFound(_)
        | CcrError::ConfigSectionNotFound(_) => ProfileError::Missing,
        CcrError::PlatformNotSupported(_) => ProfileError::Unsupported,
        _ => ProfileError::Storage,
    }
}

/// Deliberately not Debug: source/export/import may contain plaintext credentials.
pub enum ProfileCommand {
    GenericCreate {
        name: String,
        patch: ConfigPatch,
    },
    GenericSave {
        name: String,
        target: String,
        revision: String,
        patch: ConfigPatch,
    },
    GenericDelete {
        name: String,
    },
    GrokDelete {
        name: String,
        force: bool,
    },
    Create {
        name: String,
        patch: ConfigPatch,
    },
    Save {
        name: String,
        target: String,
        revision: String,
        patch: ConfigPatch,
    },
    Duplicate {
        name: String,
        target: String,
    },
    Delete {
        name: String,
    },
    Apply {
        name: String,
        enable: bool,
    },
    Off,
    Status,
    GenericValidate,
    ExportEnvironment {
        name: String,
        shell: bool,
    },
    Validate,
    Source,
    Inspect {
        name: String,
    },
    SaveSource {
        content: String,
        revision: String,
        force: bool,
    },
    Export {
        secrets: bool,
    },
    Import {
        content: String,
        replace: bool,
    },
    Backups,
    Restore {
        leaf: String,
    },
    CleanBackups,
    History,
    ClearHistory,
}
/// Plaintext results are scoped to the current editor entity and never logged.
pub enum ProfileResult {
    Outcome(ProfileOutcome),
    GrokUpdate {
        status: &'static str,
        old_name: String,
        new_name: String,
    },
    GrokDelete {
        name: String,
        reason: Option<ccr_cli::platforms::grok::GrokProfileDeleteBlockedReason>,
        attempted_force: bool,
    },
    Status {
        can_off: bool,
    },
    Done(String),
    Text(String),
    Source {
        content: String,
        revision: String,
        path: std::path::PathBuf,
        exists: bool,
    },
    SourceSave(SourceSaveOutcome),
    Lines(Vec<String>),
}

pub fn domain_platform(platform: &RequestPlatform) -> Result<Platform> {
    match platform {
        RequestPlatform::Claude => Ok(Platform::Claude),
        RequestPlatform::Codex => Ok(Platform::Codex),
        RequestPlatform::Grok => Ok(Platform::Grok),
        _ => Err(CcrError::PlatformNotSupported("native_profiles".into())),
    }
}

pub fn context(source: &ConfigSource, platform: Platform) -> Result<ProfileLifecycleContext> {
    match source {
        ConfigSource::Local => ProfileLifecycleContext::new(platform),
        ConfigSource::Fixture {
            profiles, registry, ..
        } => {
            if !profiles.is_absolute() || !registry.is_absolute() {
                return Err(CcrError::ValidationError(
                    "Fixture paths must be absolute".into(),
                ));
            }
            let parent = profiles
                .parent()
                .ok_or_else(|| CcrError::ConfigError("Fixture root missing".into()))?;
            let root = parent.join("desktop-profile-fixture");
            let platform_dir = if platform == Platform::Claude {
                parent.to_path_buf()
            } else {
                root.join("platforms").join(platform.short_name())
            };
            let paths = PlatformPaths {
                root: root.clone(),
                registry_file: registry.clone(),
                profiles_file: if platform == Platform::Claude {
                    profiles.clone()
                } else {
                    platform_dir.join("profiles.toml")
                },
                settings_file: platform_dir.join("settings.json"),
                history_file: root.join("history").join(format!("{platform}.json")),
                backups_dir: root.join("backups").join(platform.short_name()),
                platform_dir,
            };
            let locks = root.join("locks");
            let history = root.join("history.db");
            match platform {
                Platform::Claude => ProfileLifecycleContext::for_claude(
                    paths,
                    ClaudeRuntimePaths::resolve_with(root.join("home"), |_| None),
                    locks,
                    history,
                ),
                Platform::Codex => ProfileLifecycleContext::for_codex(
                    paths,
                    root.join("runtime-codex"),
                    locks,
                    history,
                ),
                Platform::Grok => ProfileLifecycleContext::for_grok(
                    paths,
                    root.join("runtime-grok/config.toml"),
                    locks,
                    history,
                ),
                _ => Err(CcrError::PlatformNotSupported("native_profiles".into())),
            }
        }
    }
}

fn rows(context: &ProfileLifecycleContext, filter: &str) -> Result<Vec<ConfigRow>> {
    context.run(|| {
        let snapshot = context.manager().snapshot()?;
        let current = context.instance().get_current_profile()?;
        let profiles = snapshot.config.sections.iter().map(|(name, section)| {
            (
                name.clone(),
                ccr_config::platforms::base::section_to_profile(section),
            )
        });
        let filter = filter.to_lowercase();
        Ok(profiles
            .into_iter()
            .filter(|(name, _)| name.to_lowercase().contains(&filter))
            .map(|(name, profile)| ConfigRow {
                version: snapshot.version.clone(),
                is_current: current.as_deref() == Some(name.as_str()),
                is_default: snapshot.config.default_config == name,
                name,
                description: profile.description.unwrap_or_default(),
                base_url: profile
                    .base_url
                    .as_deref()
                    .map(ccr_cli::platforms::GrokPlatform::safe_base_url_for_display)
                    .unwrap_or_default(),
                auth_token: profile
                    .auth_token
                    .as_ref()
                    .map(ToString::to_string)
                    .or_else(|| {
                        profile
                            .platform_data
                            .get("api_key")
                            .and_then(serde_json::Value::as_str)
                            .map(|value| ccr_core::Secret::new(value).to_string())
                    })
                    .unwrap_or_default(),
                model: profile.model,
                small_fast_model: profile.small_fast_model,
                provider: profile.provider,
                provider_type: profile.provider_type,
                account: profile.account,
                tags: profile.tags,
                usage_count: u64::from(profile.usage_count.unwrap_or_default()),
                enabled: profile.enabled.unwrap_or(true),
            })
            .collect())
    })
}

impl Services {
    pub fn profile_rows(
        &self,
        key: RequestKey,
        filter: String,
    ) -> std::result::Result<TaskSubscription<Arc<Vec<ConfigRow>>, ProfileError>, AdmissionError>
    {
        let source = self.source.clone();
        self.executor.read(key.clone(), move |cancel, _| {
            if !matches!(key.environment, EnvironmentId::Local(_)) {
                return Err(ProfileError::Unsupported);
            }
            if let ConfigSource::Fixture { delay, .. } = source {
                let deadline = std::time::Instant::now() + delay;
                while !cancel.is_cancelled() && std::time::Instant::now() < deadline {
                    std::thread::sleep(
                        deadline
                            .saturating_duration_since(std::time::Instant::now())
                            .min(std::time::Duration::from_millis(10)),
                    );
                }
            }
            if cancel.is_cancelled() {
                return Ok(Arc::new(Vec::new()));
            }
            let context = context(&source, domain_platform(&key.platform).map_err(safe_error)?)
                .map_err(safe_error)?;
            rows(&context, &filter).map(Arc::new).map_err(safe_error)
        })
    }
    pub fn profile_command(
        &self,
        key: RequestKey,
        command: ProfileCommand,
    ) -> std::result::Result<TaskSubscription<ProfileResult, ProfileError>, AdmissionError> {
        let source = self.source.clone();
        let read = matches!(
            command,
            ProfileCommand::Source
                | ProfileCommand::Inspect { .. }
                | ProfileCommand::Export { .. }
                | ProfileCommand::ExportEnvironment { .. }
                | ProfileCommand::Status
                | ProfileCommand::GenericValidate
                | ProfileCommand::Validate
                | ProfileCommand::History
                | ProfileCommand::Backups
        );
        let supported = matches!(key.environment, EnvironmentId::Local(_));
        let platform = key.platform;
        let work = move || {
            if !supported {
                return Err(ProfileError::Unsupported);
            }
            domain_platform(&platform)
                .and_then(|platform| context(&source, platform))
                .and_then(|context| {
                    let cleanup_root = match &source {
                        ConfigSource::Local => context.paths().root.as_path(),
                        ConfigSource::Fixture { profiles, .. } => {
                            profiles.parent().ok_or_else(|| {
                                CcrError::ValidationError("Fixture catalog parent missing".into())
                            })?
                        }
                    };
                    execute(&context, cleanup_root, command)
                })
                .map_err(safe_error)
        };
        if read {
            return self.executor.read(key, move |_, _| work());
        }
        self.executor.job(key, move |cancel, _| {
            if cancel.is_cancelled() {
                return Completion::Cancelled;
            }
            let result = work();
            // Once the owner returns success a late cancel cannot undo a durable write.
            match result {
                Ok(result) => Completion::Success(result),
                Err(error) => Completion::Failed(error),
            }
        })
    }
}

fn execute(
    context: &ProfileLifecycleContext,
    cleanup_root: &std::path::Path,
    command: ProfileCommand,
) -> Result<ProfileResult> {
    use ccr_app::config::{backups, mutations};
    context.run(|| {
        if context.platform() != Platform::Claude
            && matches!(
                command,
                ProfileCommand::GenericCreate { .. }
                    | ProfileCommand::GenericSave { .. }
                    | ProfileCommand::GenericDelete { .. }
                    | ProfileCommand::GenericValidate
                    | ProfileCommand::Import { .. }
                    | ProfileCommand::Duplicate { .. }
                    | ProfileCommand::Backups
                    | ProfileCommand::Restore { .. }
                    | ProfileCommand::CleanBackups
                    | ProfileCommand::History
                    | ProfileCommand::ClearHistory
                    | ProfileCommand::Export { secrets: false }
            )
        {
            return Err(CcrError::PlatformNotSupported(
                "claude_generic_configs".into(),
            ));
        }
        if context.platform() == Platform::Grok
            && matches!(
                command,
                ProfileCommand::Source
                    | ProfileCommand::SaveSource { .. }
                    | ProfileCommand::Export { .. }
            )
        {
            return Err(CcrError::PlatformNotSupported("grok_source".into()));
        }
        let manager = context.manager();
        let service = ConfigService::new(Arc::new(context.manager()));
        match command {
            ProfileCommand::GrokDelete { name, force } => return grok_delete(context, name, force),
            ProfileCommand::GenericCreate { name, patch } => {
                let patch = mutations::parse_patch(&patch_value(&patch)?)?;
                mutations::add(&manager, context.instance(), &name, &patch)?;
            }
            ProfileCommand::GenericSave {
                name,
                target,
                revision,
                patch,
            } => {
                let patch = mutations::parse_patch(&patch_value(&patch)?)?;
                if context.platform() != Platform::Claude {
                    return Err(CcrError::PlatformNotSupported(
                        "claude_generic_configs".into(),
                    ));
                }
                if name == target {
                    let guard = context.operation_lock()?;
                    mutations::update(
                        &manager,
                        context.instance(),
                        &name,
                        &patch,
                        Some(&revision),
                        &guard,
                    )?;
                } else {
                    let outcome = context.update(&name, &target, Some(&revision), |profile| {
                        let section = patch
                            .apply(&ccr_config::platforms::base::profile_to_section(profile)?)?;
                        if context.instance().get_current_profile()?.as_deref()
                            == Some(name.as_str())
                            && !section.is_enabled()
                        {
                            return Err(CcrError::ValidationError(
                                "active_config_cannot_be_disabled".into(),
                            ));
                        }
                        *profile = ccr_config::platforms::base::section_to_profile(&section);
                        Ok(())
                    })?;
                    return Ok(ProfileResult::Outcome(outcome));
                }
            }
            ProfileCommand::GenericDelete { name } => {
                let guard = context.operation_lock()?;
                mutations::delete(&service, context.instance(), &name, &guard)?;
            }
            ProfileCommand::Create { name, patch } => {
                let value = patch_value(&patch)?;
                validate_fields(context.platform(), &value)?;
                let profile = match context.platform() {
                    Platform::Claude => ccr_app::config::profile_patch::claude::build(&value),
                    Platform::Codex => ccr_app::config::profile_patch::codex::build(&value),
                    Platform::Grok => ccr_app::config::profile_patch::grok::build(&value),
                    _ => return Err(CcrError::PlatformNotSupported("native_profiles".into())),
                }
                .map_err(|_| CcrError::ValidationError("Invalid profile fields".into()))?;
                context.create(&name, profile, None)?;
            }
            ProfileCommand::Save {
                name,
                target,
                revision,
                patch,
            } => {
                validate_fields(context.platform(), &patch_value(&patch)?)?;
                if context.platform() == Platform::Grok {
                    let _application = context.operation_lock()?;
                    use ccr_cli::platforms::grok::GrokProfileUpdateOutcome;
                    let outcome = context.grok_platform()?.update_profile_versioned(
                        &name,
                        &target,
                        Some(&revision),
                        |profile| {
                            ccr_app::config::profile_patch::grok::patch(
                                profile,
                                &patch_value(&patch)?,
                            )
                            .map_err(|_| {
                                CcrError::ValidationError("Invalid Grok profile patch".into())
                            })
                        },
                    )?;
                    let status = match outcome {
                        GrokProfileUpdateOutcome::Updated { .. } => "updated",
                        GrokProfileUpdateOutcome::Renamed { .. } => "renamed",
                        GrokProfileUpdateOutcome::RenameApplyFailed { .. } => "rename_apply_failed",
                        GrokProfileUpdateOutcome::RenameCleanupFailed { .. } => {
                            "rename_cleanup_failed"
                        }
                    };
                    return Ok(ProfileResult::GrokUpdate {
                        status,
                        old_name: name,
                        new_name: target,
                    });
                }
                let outcome = context.update(&name, &target, Some(&revision), |profile| {
                    let value = patch_value(&patch)?;
                    let result = match context.platform() {
                        Platform::Claude => {
                            ccr_app::config::profile_patch::claude::patch(profile, &value)
                        }
                        Platform::Codex => {
                            ccr_app::config::profile_patch::codex::patch(profile, &value)
                        }
                        Platform::Grok => {
                            ccr_app::config::profile_patch::grok::patch(profile, &value)
                        }
                        _ => return Err(CcrError::PlatformNotSupported("native_profiles".into())),
                    };
                    result.map_err(|_| CcrError::ValidationError("Invalid profile patch".into()))
                })?;
                return Ok(ProfileResult::Outcome(outcome));
            }
            ProfileCommand::Duplicate { name, target } => {
                let _guard = context.operation_lock()?;
                mutations::duplicate(&manager, &name, &target)?;
            }
            ProfileCommand::Delete { name } => {
                if context.platform() == Platform::Grok {
                    return grok_delete(context, name, false);
                }
                let _guard = context.operation_lock()?;
                context.instance().delete_profile(&name)?;
            }
            ProfileCommand::Apply { name, enable } => {
                let request = ApplyProfileRequest::new(context.platform(), name);
                let outcome = if enable {
                    context.enable(request)?
                } else {
                    context.apply(request)?
                };
                return Ok(ProfileResult::Outcome(outcome));
            }
            ProfileCommand::Status => {
                return context
                    .needs_login_prep()
                    .map(|can_off| ProfileResult::Status { can_off });
            }
            ProfileCommand::ExportEnvironment { name, shell } => {
                return context
                    .export_profile_env(&name, shell)
                    .map(ProfileResult::Text);
            }
            ProfileCommand::Off => {
                let outcome = context.off()?;
                return Ok(ProfileResult::Done(format!(
                    "Changed: {}; warning count: {}. Review CCR recovery records if warnings remain. / 已处理停用；如有警告请检查恢复记录。",
                    outcome.changed, outcome.warnings.len()
                )));
            }
            ProfileCommand::Validate => {
                let profiles = context.instance().load_profiles()?;
                let mut valid = 0;
                let mut invalid = 0;
                let mut lines = Vec::new();
                for (name, profile) in profiles {
                    let ok = context.instance().validate_profile(&profile).is_ok();
                    if ok {
                        valid += 1
                    } else {
                        invalid += 1
                    }
                    lines.push(format!(
                        "{name}: {}",
                        if ok { "valid" } else { "invalid; check fields" }
                    ));
                }
                lines.insert(0, format!("Valid: {valid}; invalid: {invalid}"));
                return Ok(ProfileResult::Lines(lines));
            }
            ProfileCommand::GenericValidate => {
                let report = service.validate_all()?;
                let mut lines = vec![format!("Valid: {}; invalid: {}", report.valid_count, report.invalid_count)];
                lines.extend(report.results.into_iter().map(|(name, valid, _)| format!("{name}: {}", if valid { "valid" } else { "invalid; check fields" })));
                return Ok(ProfileResult::Lines(lines));
            }
            ProfileCommand::Source => {
                let source = source::read(context.paths())?;
                return Ok(ProfileResult::Source {
                    content: source.content,
                    revision: source.token,
                    path: source.path,
                    exists: source.exists,
                });
            }
            ProfileCommand::Inspect{name}=>{
                let snapshot=manager.snapshot()?;
                let section=snapshot.config.get_section(&name)?;
                let content=toml::to_string_pretty(section).map_err(|_|CcrError::ConfigError("Cannot serialize selected profile".into()))?;
                return Ok(ProfileResult::Text(format!("# Selected profile: {name}\n# Catalog revision: {}\n{content}",snapshot.version)));
            }
            ProfileCommand::SaveSource {
                content,
                revision,
                force,
            } => {
                let current = context.instance().get_current_profile()?;
                return source::save(
                    context.paths(),
                    current.as_deref(),
                    &content,
                    &revision,
                    force,
                )
                .map(ProfileResult::SourceSave);
            }
            ProfileCommand::Export { secrets } => {
                return if secrets {
                    source::export(&context.paths().profiles_file, true)
                } else {
                    service.export_config(false)
                }
                .map(ProfileResult::Text);
            }
            ProfileCommand::Import { content, replace } => {
                let result = service.import_config(
                    &content,
                    if replace {
                        ccr_config::ImportMode::Replace
                    } else {
                        ccr_config::ImportMode::Merge
                    },
                    true,
                )?;
                return Ok(ProfileResult::Done(format!(
                    "Imported: {}; updated: {}; skipped: {}",
                    result.added, result.updated, result.skipped
                )));
            }
            ProfileCommand::Backups => {
                return Ok(ProfileResult::Lines(
                    manager
                        .list_backups()?
                        .iter()
                        .filter_map(|path| {
                            path.file_name().map(|s| s.to_string_lossy().into_owned())
                        })
                        .collect(),
                ));
            }
            ProfileCommand::Restore { leaf } => {
                let path =
                    backups::resolve_managed_restore_path(&context.paths().profiles_file, &leaf)?;
                backups::restore(&service, &path)?;
            }
            ProfileCommand::CleanBackups => {
                let result = backups::clean(cleanup_root);
                return Ok(ProfileResult::Done(format!(
                    "Deleted backups: {}",
                    result.deleted
                )));
            }
            ProfileCommand::History | ProfileCommand::ClearHistory => {
                let database = ccr_store::Database::init(context.history_file())?;
                let history = ccr_store::HistoryService::new(Arc::new(
                    ccr_store::HistoryManager::new(database),
                ));
                if matches!(command, ProfileCommand::ClearHistory) {
                    history.clear()?;
                } else {
                    return Ok(ProfileResult::Lines(
                        history
                            .get_recent(100)?
                            .iter()
                            .map(|entry| {
                                format!(
                                    "{} · {} · {}",
                                    entry.timestamp.format("%Y-%m-%d %H:%M"),
                                    entry.operation.as_str(),
                                    match entry.result {
                                        ccr_store::OperationResult::Success => "success",
                                        ccr_store::OperationResult::Failure(_) => "failed",
                                        ccr_store::OperationResult::Warning(_) => "warning",
                                    }
                                )
                            })
                            .collect(),
                    ));
                }
            }
        }
        Ok(ProfileResult::Done(
            "Operation completed. Refreshing catalog. / 操作完成，正在重新读取配置".into(),
        ))
    })
}

fn grok_delete(
    context: &ProfileLifecycleContext,
    name: String,
    force: bool,
) -> Result<ProfileResult> {
    use ccr_cli::platforms::grok::GrokProfileDeleteOutcome;
    let _application = context.operation_lock()?;
    let outcome = context
        .grok_platform()?
        .delete_profile_guarded(&name, force)?;
    let reason = match outcome {
        GrokProfileDeleteOutcome::Deleted { .. } => None,
        GrokProfileDeleteOutcome::Blocked { reason } => Some(reason),
    };
    Ok(ProfileResult::GrokDelete {
        name,
        reason,
        attempted_force: force,
    })
}

/// Explicit partial fields. Empty optional values mean remove; untouched fields stay absent.
pub fn string_patch(fields: impl IntoIterator<Item = (String, String)>) -> ConfigPatch {
    let mut patch = ConfigPatch::default();
    for (name, value) in fields {
        patch.fields.insert(
            name,
            if value.is_empty() {
                FieldPatch::Remove
            } else {
                FieldPatch::Set(toml::Value::String(value))
            },
        );
    }
    patch
}

fn patch_value(patch: &ConfigPatch) -> Result<serde_json::Value> {
    let mut object = serde_json::Map::new();
    for (field, value) in &patch.fields {
        object.insert(
            field.clone(),
            match value {
                FieldPatch::Remove => serde_json::Value::Null,
                FieldPatch::Set(value) => serde_json::to_value(value)
                    .map_err(|_| CcrError::ValidationError("Invalid profile patch".into()))?,
            },
        );
    }
    Ok(serde_json::Value::Object(object))
}
/// Strict JSON field patch parser. Diagnostics never include input text.
pub fn merge_patch_json(
    patch: &mut ConfigPatch,
    content: &str,
) -> std::result::Result<(), ProfileError> {
    if content.trim().is_empty() {
        return Ok(());
    }
    let object: serde_json::Value =
        serde_json::from_str(content).map_err(|_| ProfileError::Invalid)?;
    let object = object.as_object().ok_or(ProfileError::Invalid)?;
    for (field, value) in object {
        let value = if value.is_null() {
            FieldPatch::Remove
        } else {
            FieldPatch::Set(toml::Value::try_from(value).map_err(|_| ProfileError::Invalid)?)
        };
        patch.fields.insert(field.clone(), value);
    }
    Ok(())
}

fn validate_fields(platform: Platform, value: &serde_json::Value) -> Result<()> {
    if platform == Platform::Grok
        && let Some(fields) = value.as_object()
        && (fields.contains_key("api_key") || fields.contains_key("env_key"))
        && !fields.contains_key("credential_action")
    {
        return Err(CcrError::ValidationError(
            "Choose an explicit credential action".into(),
        ));
    }
    match platform {
        Platform::Claude => ccr_app::config::profile_patch::claude::validate_fields(value),
        Platform::Codex => ccr_app::config::profile_patch::codex::validate_fields(value),
        Platform::Grok => ccr_app::config::profile_patch::grok::validate_fields(value),
        _ => return Err(CcrError::PlatformNotSupported("native_profiles".into())),
    }
    .map_err(|_| CcrError::ValidationError("Unknown profile patch field".into()))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use ccr_app::runtime::{RuntimeOwner, TaskEvent};
    use std::time::Duration;
    pub(crate) fn fixture() -> (tempfile::TempDir, ConfigSource) {
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let source = ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        };
        let context = context(&source, Platform::Claude).unwrap_or_else(|_| panic!("context"));
        context
            .run(|| {
                let mut config = ccr_config::CcsConfig {
                    default_config: "alpha".into(),
                    current_config: "alpha".into(),
                    settings: Default::default(),
                    sections: Default::default(),
                };
                config.sections.insert(
                    "alpha".into(),
                    ccr_config::ConfigSection {
                        description: Some("Fixture".into()),
                        base_url: Some("https://fixture.invalid".into()),
                        auth_token: Some(ccr_core::Secret::new(
                            "sk-native-profile-secret-sentinel",
                        )),
                        model: Some("fixture-model".into()),
                        enabled: Some(true),
                        ..Default::default()
                    },
                );
                let extensions: toml::Table = toml::from_str(
                    "unknown_native = 'extension retained'\nwhen_native = 2026-10-18T00:00:00Z\n",
                )?;
                config.get_section_mut("alpha")?.other.extend(extensions);
                config
                    .sections
                    .insert("beta".into(), config.sections["alpha"].clone());
                context.manager().save(&config)
            })
            .unwrap_or_else(|_| panic!("seed"));
        (root, source)
    }
    fn key(id: u64) -> RequestKey {
        RequestKey {
            environment: EnvironmentId::Local("fixture".into()),
            platform: RequestPlatform::Claude,
            generation: id,
            request_id: id,
        }
    }
    async fn terminal(
        mut subscription: TaskSubscription<ProfileResult, ProfileError>,
    ) -> Arc<Completion<ProfileResult, ProfileError>> {
        while let Some(event) = subscription.next().await {
            if let TaskEvent::Terminal(result) = event {
                return result;
            }
        }
        panic!("terminal missing")
    }
    #[gpui_kit::test]
    async fn fixture_cleanup_reaches_domain_backups_without_changing_catalog_or_runtime(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        cx.executor().allow_parking();
        let (root, source) = fixture();
        let context = context(&source, Platform::Claude).unwrap_or_else(|_| panic!("context"));
        let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
        let services = Services::new(owner.handle(), source);
        assert!(matches!(
            terminal(
                services
                    .profile_command(
                        key(1),
                        ProfileCommand::Apply {
                            name: "beta".into(),
                            enable: false
                        }
                    )
                    .unwrap_or_else(|_| panic!("admission"))
            )
            .await
            .as_ref(),
            Completion::Success(_)
        ));
        let before = context
            .manager()
            .snapshot()
            .unwrap_or_else(|_| panic!("snapshot"))
            .version;
        let runtime_path = context.instance().get_settings_path();
        let runtime_before = std::fs::read(&runtime_path).unwrap_or_else(|_| panic!("runtime"));
        let service = ConfigService::new(Arc::new(context.manager()));
        let backup = context
            .run(|| service.backup_config(Some("native-cleanup")))
            .unwrap_or_else(|_| panic!("backup"));
        assert_eq!(backup.parent(), Some(root.path()));
        let leaf = backup
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_else(|| panic!("leaf"));
        assert!(
            context
                .manager()
                .list_backups()
                .unwrap_or_else(|_| panic!("backups"))
                .contains(&backup)
        );
        assert_eq!(
            ccr_app::config::backups::resolve_managed_restore_path(
                &context.paths().profiles_file,
                leaf
            )
            .unwrap_or_else(|_| panic!("resolve")),
            backup
        );
        let unrelated = root.path().join("keep.txt");
        std::fs::write(&unrelated, "fixture unrelated data")
            .unwrap_or_else(|_| panic!("unrelated"));
        let mut directories = vec![root.path().to_path_buf()];
        let mut owned_backups = Vec::new();
        for _ in 0..3 {
            let mut next = Vec::new();
            for directory in directories {
                for entry in
                    std::fs::read_dir(directory).unwrap_or_else(|_| panic!("fixture listing"))
                {
                    let entry = entry.unwrap_or_else(|_| panic!("fixture entry"));
                    let path = entry.path();
                    if entry
                        .file_type()
                        .unwrap_or_else(|_| panic!("fixture type"))
                        .is_dir()
                    {
                        next.push(path);
                    } else if matches!(
                        path.extension().and_then(|extension| extension.to_str()),
                        Some("bak" | "backup")
                    ) {
                        owned_backups.push(path);
                    }
                }
            }
            directories = next;
        }
        assert_eq!(
            owned_backups.len(),
            2,
            "domain apply preimage plus explicit backup: {owned_backups:?}"
        );
        let result = terminal(
            services
                .profile_command(key(2), ProfileCommand::CleanBackups)
                .unwrap_or_else(|_| panic!("admission")),
        )
        .await;
        let Completion::Success(ProfileResult::Done(message)) = result.as_ref() else {
            panic!("cleanup must return success");
        };
        assert_eq!(
            message,
            &format!("Deleted backups: {}", owned_backups.len())
        );
        assert!(owned_backups.iter().all(|path| !path.exists()));
        assert!(!backup.exists());
        assert!(
            context
                .manager()
                .list_backups()
                .unwrap_or_else(|_| panic!("backups"))
                .is_empty()
        );
        assert_eq!(
            context
                .manager()
                .snapshot()
                .unwrap_or_else(|_| panic!("snapshot"))
                .version,
            before
        );
        assert_eq!(
            std::fs::read(runtime_path).unwrap_or_else(|_| panic!("runtime")),
            runtime_before
        );
        assert_eq!(
            std::fs::read_to_string(unrelated).unwrap_or_else(|_| panic!("unrelated")),
            "fixture unrelated data"
        );
        owner.shutdown();
    }
    #[gpui_kit::test]
    async fn original_draft_cas_one_client_wins_and_fields_remain_secret(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        cx.executor().allow_parking();
        let (_root, source) = fixture();
        let context = context(&source, Platform::Claude).unwrap_or_else(|_| panic!("context"));
        let revision = context
            .manager()
            .snapshot()
            .unwrap_or_else(|_| panic!("snapshot"))
            .version;
        let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
        let services = Services::new(owner.handle(), source);
        let first = services
            .profile_command(
                key(1),
                ProfileCommand::Save {
                    name: "beta".into(),
                    target: "beta".into(),
                    revision: revision.clone(),
                    patch: string_patch([("description".into(), "first".into())]),
                },
            )
            .unwrap_or_else(|_| panic!("admission"));
        let second = services
            .profile_command(
                key(2),
                ProfileCommand::Save {
                    name: "beta".into(),
                    target: "beta".into(),
                    revision,
                    patch: string_patch([("description".into(), "second".into())]),
                },
            )
            .unwrap_or_else(|_| panic!("admission"));
        let first = terminal(first).await;
        let second = terminal(second).await;
        assert_eq!(
            usize::from(matches!(first.as_ref(), Completion::Success(_)))
                + usize::from(matches!(second.as_ref(), Completion::Success(_))),
            1
        );
        assert!(
            matches!(first.as_ref(), Completion::Failed(ProfileError::Conflict))
                || matches!(second.as_ref(), Completion::Failed(ProfileError::Conflict))
        );
        let actual = context.manager().load().unwrap_or_else(|_| panic!("read"));
        let profile = &actual.sections["beta"];
        assert!(matches!(
            profile.description.as_deref(),
            Some("first" | "second")
        ));
        assert_eq!(
            profile.auth_token.as_ref().map(ccr_core::Secret::expose),
            Some("sk-native-profile-secret-sentinel")
        );
        let rows = rows(&context, "").unwrap_or_else(|_| panic!("rows"));
        assert!(!format!("{rows:?}").contains("sk-native-profile-secret-sentinel"));
        owner.shutdown();
    }
    #[test]
    fn invalid_patch_source_conflict_and_missing_fixture_are_safe() {
        let (root, source) = fixture();
        let context = context(&source, Platform::Claude).unwrap_or_else(|_| panic!("context"));
        let before = context
            .manager()
            .snapshot()
            .unwrap_or_else(|_| panic!("read"));
        let mut patch = ConfigPatch::default();
        assert!(merge_patch_json(&mut patch, "{\"model\": 42}").is_ok());
        assert!(
            execute(
                &context,
                &context.paths().root,
                ProfileCommand::Save {
                    name: "beta".into(),
                    target: "beta".into(),
                    revision: before.version.clone(),
                    patch
                }
            )
            .is_err()
        );
        assert_eq!(
            context
                .manager()
                .snapshot()
                .unwrap_or_else(|_| panic!("read"))
                .version,
            before.version
        );
        let source = source::read(context.paths()).unwrap_or_else(|_| panic!("source"));
        assert!(matches!(
            execute(
                &context,
                &context.paths().root,
                ProfileCommand::SaveSource {
                    content: source.content,
                    revision: "stale-token".into(),
                    force: true
                }
            ),
            Ok(ProfileResult::SourceSave(SourceSaveOutcome::Conflict))
        ));
        for error in [
            CcrError::ConfigError("secret-sentinel".into()),
            CcrError::ValidationError("secret-sentinel".into()),
        ] {
            assert!(!safe_error(error).message().contains("secret-sentinel"));
        }
        let missing = ConfigSource::Fixture {
            profiles: root.path().join("absent/profiles.toml"),
            registry: root.path().join("absent/registry.toml"),
            delay: Duration::ZERO,
        };
        let context =
            super::context(&missing, Platform::Claude).unwrap_or_else(|_| panic!("context"));
        assert!(rows(&context, "").is_err());
        assert!(!root.path().join("absent").exists());
    }
    #[test]
    fn native_apply_off_duplicate_restore_and_validation_use_domain_owner() {
        let (_root, source) = fixture();
        let context = context(&source, Platform::Claude).unwrap_or_else(|_| panic!("context"));
        let inspected_revision = context
            .manager()
            .snapshot()
            .unwrap_or_else(|_| panic!("read"))
            .version;
        assert!(
            matches!(execute(&context, &context.paths().root, ProfileCommand::Inspect{name:"alpha".into()}),Ok(ProfileResult::Text(content)) if content.contains("sk-native-profile-secret-sentinel")&&content.contains("unknown_native")&&content.contains("when_native = 2026-10-18T00:00:00Z")&&content.contains(&inspected_revision))
        );
        assert_eq!(
            context
                .manager()
                .snapshot()
                .unwrap_or_else(|_| panic!("read"))
                .version,
            inspected_revision
        );
        assert!(
            matches!(execute(&context, &context.paths().root, ProfileCommand::Validate),Ok(ProfileResult::Lines(lines)) if lines[0]=="Valid: 2; invalid: 0")
        );
        execute(
            &context,
            &context.paths().root,
            ProfileCommand::Duplicate {
                name: "beta".into(),
                target: "duplicate".into(),
            },
        )
        .unwrap_or_else(|_| panic!("duplicate"));
        execute(
            &context,
            &context.paths().root,
            ProfileCommand::Apply {
                name: "beta".into(),
                enable: false,
            },
        )
        .unwrap_or_else(|_| panic!("apply"));
        assert_eq!(
            context
                .instance()
                .get_current_profile()
                .unwrap_or_else(|_| panic!("current"))
                .as_deref(),
            Some("beta")
        );
        assert!(context.instance().get_settings_path().is_file());
        execute(&context, &context.paths().root, ProfileCommand::Off)
            .unwrap_or_else(|_| panic!("off"));
        assert_eq!(
            context
                .instance()
                .get_current_profile()
                .unwrap_or_else(|_| panic!("current")),
            None
        );
        execute(
            &context,
            &context.paths().root,
            ProfileCommand::Delete {
                name: "duplicate".into(),
            },
        )
        .unwrap_or_else(|_| panic!("delete"));
        assert!(
            !context
                .manager()
                .load()
                .unwrap_or_else(|_| panic!("read"))
                .sections
                .contains_key("duplicate")
        );
        assert!(
            execute(
                &context,
                &context.paths().root,
                ProfileCommand::Restore {
                    leaf: "../outside.bak".into()
                }
            )
            .is_err()
        );
        assert!(
            matches!(execute(&context, &context.paths().root, ProfileCommand::Export{secrets:false}),Ok(ProfileResult::Text(content)) if !content.contains("sk-native-profile-secret-sentinel"))
        );
        assert!(
            matches!(execute(&context, &context.paths().root, ProfileCommand::Export{secrets:true}),Ok(ProfileResult::Text(content)) if content.contains("sk-native-profile-secret-sentinel"))
        );
    }

    #[test]
    fn generic_policy_rejects_unknown_and_active_disable_and_history_hides_legacy_diagnostics() {
        let (_root, source) = fixture();
        let context = context(&source, Platform::Claude).unwrap_or_else(|_| panic!("context"));
        let version = context
            .manager()
            .snapshot()
            .unwrap_or_else(|_| panic!("read"))
            .version;
        let expected = ConfigService::new(Arc::new(context.manager()))
            .validate_all()
            .unwrap_or_else(|_| panic!("generic validation"));
        assert!(
            matches!(execute(&context, &context.paths().root, ProfileCommand::GenericValidate), Ok(ProfileResult::Lines(lines)) if lines[0] == format!("Valid: {}; invalid: {}", expected.valid_count, expected.invalid_count) && lines.len() == expected.results.len() + 1)
        );
        let mut patch = ConfigPatch::default();
        merge_patch_json(&mut patch, "{\"enabled\":false}").unwrap_or_else(|_| panic!("patch"));
        assert!(
            execute(
                &context,
                &context.paths().root,
                ProfileCommand::GenericSave {
                    name: "alpha".into(),
                    target: "alpha".into(),
                    revision: version.clone(),
                    patch
                }
            )
            .is_err()
        );
        let mut patch = ConfigPatch::default();
        merge_patch_json(&mut patch, "{\"enabled\":false}").unwrap_or_else(|_| panic!("patch"));
        assert!(
            execute(
                &context,
                &context.paths().root,
                ProfileCommand::GenericSave {
                    name: "alpha".into(),
                    target: "renamed-disabled".into(),
                    revision: version.clone(),
                    patch,
                }
            )
            .is_err()
        );
        assert!(
            !context
                .manager()
                .load()
                .unwrap_or_else(|_| panic!("read"))
                .sections
                .contains_key("renamed-disabled")
        );
        let mut patch = ConfigPatch::default();
        merge_patch_json(&mut patch, "{\"unknown_native_field\":\"secret-sentinel\"}")
            .unwrap_or_else(|_| panic!("patch"));
        assert!(
            execute(
                &context,
                &context.paths().root,
                ProfileCommand::GenericSave {
                    name: "beta".into(),
                    target: "beta".into(),
                    revision: version.clone(),
                    patch
                }
            )
            .is_err()
        );
        assert_eq!(
            context
                .manager()
                .snapshot()
                .unwrap_or_else(|_| panic!("read"))
                .version,
            version
        );
        for result in [
            ccr_store::OperationResult::Failure("history-secret-sentinel".into()),
            ccr_store::OperationResult::Warning("history-secret-sentinel".into()),
        ] {
            let database = ccr_store::Database::init(context.history_file())
                .unwrap_or_else(|_| panic!("history"));
            let manager = ccr_store::HistoryManager::new(database);
            manager
                .add(ccr_store::HistoryEntry::new(
                    ccr_store::OperationType::Update,
                    ccr_store::OperationDetails {
                        from_config: None,
                        to_config: None,
                        backup_path: None,
                        extra: None,
                    },
                    result,
                ))
                .unwrap_or_else(|_| panic!("history entry"));
        }
        assert!(
            matches!(execute(&context, &context.paths().root, ProfileCommand::History),Ok(ProfileResult::Lines(lines)) if lines.len()==2 && !lines.join("\n").contains("history-secret-sentinel"))
        );
    }
}
