//! Explicit remote configuration inputs for existing synchronous domain owners.
use super::{
    Services,
    platforms::{PlatformError, PlatformSession},
};
use ccr_app::{
    environments::{
        config_mirror::ConfigMirror,
        io::{EnvironmentIo, Error},
    },
    runtime::{EnvironmentId, Platform},
};
use ccr_cli::application::{
    platform_roots::PlatformRoots, profile_lifecycle::ProfileLifecycleContext,
};
use std::{collections::BTreeSet, sync::Arc};

pub(super) fn error(error: Error) -> PlatformError {
    match error {
        Error::UnrepresentablePath => PlatformError::RemotePathUnsupported,
        Error::Cancelled | Error::Ssh("ssh_cancelled") => PlatformError::Cancelled,
        Error::Ssh("ssh_write_outcome_unknown" | "ssh_cleanup_failed") => {
            PlatformError::RemotePartial
        }
        Error::Busy => PlatformError::RemoteBusy,
        Error::Partial => PlatformError::RemotePartial,
        Error::Conflict => PlatformError::Conflict,
        Error::Unsupported => PlatformError::Unsupported,
        _ => PlatformError::Storage,
    }
}
impl Services {
    pub(super) fn remote_configuration_io(
        &self,
        environment: &EnvironmentId,
    ) -> Result<EnvironmentIo, Error> {
        if self.is_fixture() && !super::environments::owner_is_fixture_transport(self) {
            return Err(Error::Ssh("ssh_fixture_offline"));
        }
        EnvironmentIo::resolve(
            environment,
            || Err(Error::Unsupported),
            self.ssh.get().cloned(),
        )
    }
}

pub(super) fn history_path(source: &super::ConfigSource) -> Option<std::path::PathBuf> {
    match source {
        super::ConfigSource::Local => ccr_store::Database::default_path().ok(),
        super::ConfigSource::Fixture { registry, .. } => {
            registry.parent().map(|parent| parent.join("history.db"))
        }
        super::ConfigSource::Benchmark(layout) => Some(layout.root.join("remote-history.db")),
    }
}
pub(super) fn record(history: Option<&std::path::Path>, platform: Platform, success: bool) -> bool {
    let Some(history) = history else {
        return false;
    };
    let result = (|| {
        let database = history_database(history)?;
        let manager = ccr_store::HistoryManager::new(database);
        let mut entry = ccr_store::HistoryEntry::new(
            ccr_store::OperationType::Update,
            ccr_store::OperationDetails {
                from_config: None,
                to_config: None,
                backup_path: None,
                extra: Some(format!("remote_configuration:{}", platform.short_name())),
            },
            if success {
                ccr_store::OperationResult::Success
            } else {
                ccr_store::OperationResult::Failure("remote_configuration_failed".into())
            },
        );
        entry.actor = "CCR Desktop".into();
        manager.add(entry)
    })();
    result.is_ok()
}
pub(super) fn history_database(path: &std::path::Path) -> ccr_core::Result<ccr_store::Database> {
    // 非文件目标不会通过连接池的重试变成可用数据库，先立即拒绝。
    match std::fs::metadata(path) {
        Ok(metadata) if !metadata.is_file() => {
            return Err(ccr_core::CcrError::ConfigError(
                "history_path_not_file".into(),
            ));
        }
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
            return Err(ccr_core::CcrError::ConfigError(
                "history_path_unavailable".into(),
            ));
        }
        _ => {}
    }
    ccr_store::Database::init(path)
}

pub(super) async fn capture(
    io: EnvironmentIo,
    platform: Platform,
    cancel: tokio_util::sync::CancellationToken,
) -> Result<Arc<ConfigMirror>, PlatformError> {
    let mut paths = BTreeSet::new();
    let runtime: &[&str] = match platform {
        Platform::Claude => &[
            ".claude/settings.json",
            ".claude/.credentials.json",
            ".claude.json",
        ],
        Platform::Codex => &[".codex/config.toml", ".codex/auth.json"],
        Platform::Grok => &[".grok/config.toml", ".grok/auth.json"],
        Platform::Gemini => &[".gemini/antigravity-cli/settings.json"],
        Platform::OpenCode => &[
            ".config/opencode/opencode.json",
            ".config/opencode/tui.json",
            ".opencode/config.json",
            ".opencode/keybindings.json",
        ],
        _ => return Err(PlatformError::Unsupported),
    };
    paths.extend(runtime.iter().map(|path| (*path).to_owned()));
    let auth = if matches!(
        platform,
        Platform::Claude | Platform::Codex | Platform::Grok
    ) {
        paths.insert(".ccr/config.toml".into());
        let directory = format!(".ccr/platforms/{}", platform.short_name());
        for name in [
            "profiles.toml",
            "profiles.db",
            "profiles.db-wal",
            "profiles.db-shm",
            "profiles.db-journal",
            "settings.json",
            "auth_registry.toml",
            "profile_secrets.json",
            "profile_entry_auth_state.json",
            "profile_entry_config_state.json",
        ] {
            paths.insert(format!("{directory}/{name}"));
        }
        Some(format!("{directory}/auth"))
    } else {
        None
    };
    ConfigMirror::capture(io, paths, auth, cancel)
        .await
        .map(Arc::new)
        .map_err(error)
}

/// Only inputs used by profile observation; account stores are unrelated. When
/// a file is inaccessible, recapture the remaining declared inputs against
/// the same authority. Cancellation, disconnect and catalog errors stay fatal.
pub(super) async fn capture_profile_catalog(
    io: EnvironmentIo,
    platform: Platform,
    cancel: tokio_util::sync::CancellationToken,
) -> Result<(ConfigMirror, super::profiles::UnavailableInputs), super::profiles::ProfileError> {
    use super::profiles::{ProfileError, UnavailableInputs, remote_error};
    use ccr_cli::application::profile_observation::RuntimeState;
    let directory = format!(".ccr/platforms/{}", platform.short_name());
    let required: BTreeSet<_> = [
        "profiles.toml",
        "profiles.db",
        "profiles.db-shm",
        "profiles.db-wal",
        "profiles.db-journal",
    ]
    .into_iter()
    .map(|file| format!("{directory}/{file}"))
    .collect();
    let mut paths = required.clone();
    paths.insert(".ccr/config.toml".into());
    match platform {
        Platform::Claude => {
            paths.insert(".claude/settings.json".into());
        }
        Platform::Codex => {
            paths.extend([
                ".codex/config.toml".into(),
                ".codex/auth.json".into(),
                format!("{directory}/profile_secrets.json"),
            ]);
        }
        Platform::Grok => {
            paths.extend([
                ".grok/config.toml".into(),
                format!("{directory}/profile_entry_config_state.json"),
            ]);
        }
        _ => return Err(ProfileError::Unsupported),
    }
    let mut unavailable = UnavailableInputs::default();
    loop {
        match ConfigMirror::capture_inputs(io.clone(), paths.clone(), None, cancel.clone()).await {
            Ok(mirror) => return Ok((mirror, unavailable)),
            Err(error) => {
                let state = match error.reason {
                    Error::TooLarge
                    | Error::Ssh("ssh_file_too_large" | "ssh_file_access_denied") => {
                        RuntimeState::Unreadable
                    }
                    Error::Unsupported | Error::Ssh("ssh_invalid_remote_path") => {
                        RuntimeState::Unsupported
                    }
                    reason => return Err(remote_error(reason)),
                };
                let Some(path) = error.path else {
                    return Err(remote_error(error.reason));
                };
                if required.contains(&path) || !paths.remove(&path) {
                    return Err(remote_error(error.reason));
                }
                if path == ".ccr/config.toml" {
                    unavailable.registry = Some(state);
                } else {
                    unavailable.runtime = Some(state);
                }
            }
        }
    }
}

pub(super) async fn capture_settings(
    io: EnvironmentIo,
    platform: Platform,
    surface: super::platforms::Surface,
    cancel: tokio_util::sync::CancellationToken,
) -> Result<Arc<ConfigMirror>, PlatformError> {
    if platform == Platform::Codex
        && matches!(
            surface,
            super::platforms::Surface::Providers | super::platforms::Surface::Models
        )
    {
        let path = if surface == super::platforms::Surface::Providers {
            ".ccr/platforms/codex/model_providers.json"
        } else {
            ".ccr/platforms/codex/custom-models.toml"
        };
        return ConfigMirror::capture(io, BTreeSet::from([path.into()]), None, cancel)
            .await
            .map(Arc::new)
            .map_err(error);
    }
    if platform == Platform::OpenCode {
        let (primary, fallback) = if surface == super::platforms::Surface::Tui {
            (".config/opencode/tui.json", ".opencode/keybindings.json")
        } else {
            (".config/opencode/opencode.json", ".opencode/config.json")
        };
        let exists = io
            .read_bytes(primary, cancel.clone())
            .await
            .map_err(error)?
            .exists;
        let mut paths = BTreeSet::from([primary.into()]);
        if !exists {
            paths.insert(fallback.into());
        }
        return ConfigMirror::capture(io, paths, None, cancel)
            .await
            .map(Arc::new)
            .map_err(error);
    }
    let files: &[&str] = match (platform, surface) {
        (Platform::Claude, _) => &[".claude/settings.json"],
        (Platform::Codex, _) => &[".codex/config.toml"],
        (Platform::Grok, _) => &[".grok/config.toml"],
        (Platform::Gemini, _) => &[".gemini/antigravity-cli/settings.json"],
        _ => return Err(PlatformError::Unsupported),
    };
    ConfigMirror::capture(
        io,
        files.iter().map(|path| (*path).into()).collect(),
        None,
        cancel,
    )
    .await
    .map(Arc::new)
    .map_err(error)
}

pub(super) async fn capture_grok_settings_write(
    io: EnvironmentIo,
    cancel: tokio_util::sync::CancellationToken,
) -> Result<Arc<ConfigMirror>, PlatformError> {
    let paths = [
        ".grok/config.toml",
        ".ccr/config.toml",
        ".ccr/platforms/grok/profiles.toml",
        ".ccr/platforms/grok/profiles.db",
        ".ccr/platforms/grok/profiles.db-wal",
        ".ccr/platforms/grok/profiles.db-shm",
        ".ccr/platforms/grok/profiles.db-journal",
        ".ccr/platforms/grok/profile_entry_config_state.json",
    ];
    ConfigMirror::capture(
        io,
        paths.into_iter().map(str::to_owned).collect(),
        None,
        cancel,
    )
    .await
    .map(Arc::new)
    .map_err(error)
}

pub(super) fn roots(mirror: &ConfigMirror) -> PlatformRoots {
    let home = mirror.root().to_path_buf();
    let ccr = home.join(".ccr");
    PlatformRoots {
        claude: ccr_config::ClaudeRuntimePaths::resolve_with(&home, |_| None),
        codex_store: ccr.join("platforms/codex"),
        codex_runtime: home.join(".codex"),
        grok_runtime: home.join(".grok"),
        locks: home.join(".ccr-local-locks"),
        home,
        ccr,
        fixture: true,
    }
}

pub(super) fn context(
    mirror: &ConfigMirror,
    platform: ccr_config::Platform,
) -> ccr_core::Result<ProfileLifecycleContext> {
    let roots = roots(mirror);
    let name = platform.short_name();
    let directory = roots.ccr.join("platforms").join(name);
    let paths = ccr_config::PlatformPaths {
        root: roots.ccr.clone(),
        registry_file: roots.ccr.join("config.toml"),
        profiles_file: directory.join("profiles.toml"),
        settings_file: directory.join("settings.json"),
        history_file: roots.ccr.join("history").join(format!("{name}.json")),
        backups_dir: directory.join("backups"),
        platform_dir: directory,
    };
    let history = roots.home.join(".ccr-local-history.db");
    match platform {
        ccr_config::Platform::Claude => {
            ProfileLifecycleContext::for_claude(paths, roots.claude, roots.locks, history)
                .map(|context| context.with_process_environment(false))
        }
        ccr_config::Platform::Codex => {
            ProfileLifecycleContext::for_codex(paths, roots.codex_runtime, roots.locks, history)
        }
        ccr_config::Platform::Grok => ProfileLifecycleContext::for_grok(
            paths,
            roots.grok_runtime.join("config.toml"),
            roots.locks,
            history,
        ),
        _ => Err(ccr_core::CcrError::PlatformNotSupported(
            "remote_profiles".into(),
        )),
    }
}

pub(super) fn session_matches(session: &PlatformSession, environment: &EnvironmentId) -> bool {
    session
        .remote
        .as_ref()
        .is_some_and(|mirror| mirror.io().environment() == environment)
}

#[cfg(test)]
mod tests;
