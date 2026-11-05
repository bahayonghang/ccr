//! Capability-driven platform consumers. Secret documents stay scoped to an editor.
use super::{ConfigSource, Services};
use ccr_app::{
    platform_settings::{self, Document, Format, SaveOutcome, Snapshot},
    runtime::{AdmissionError, Completion, EnvironmentId, Platform, RequestKey, TaskSubscription},
};
use ccr_cli::application::codex_oauth::{NativeError, NativeStatus};
use ccr_cli::application::platform_roots::PlatformRoots;
use ccr_core::core::{BackupPolicy, LockManager};
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    Settings,
    Tui,
    Auth,
    Providers,
    LegacySettings,
}
#[derive(Clone, Copy)]
pub struct Descriptor {
    pub platform: Platform,
    pub name: &'static str,
    pub settings: bool,
    pub auth: bool,
    pub providers: bool,
}
pub const DESCRIPTORS: &[Descriptor] = &[
    Descriptor {
        platform: Platform::Claude,
        name: "Claude Code",
        settings: true,
        auth: true,
        providers: false,
    },
    Descriptor {
        platform: Platform::Codex,
        name: "Codex",
        settings: true,
        auth: true,
        providers: false,
    },
    Descriptor {
        platform: Platform::Grok,
        name: "Grok",
        settings: true,
        auth: true,
        providers: false,
    },
    Descriptor {
        platform: Platform::OpenCode,
        name: "OpenCode",
        settings: true,
        auth: false,
        providers: true,
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformError {
    Unsupported,
    Invalid,
    Storage,
    ManagedLocked,
}
impl PlatformError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Unsupported => {
                "This action is not connected for this platform/environment. / 此平台或环境尚未接入该操作"
            }
            Self::Invalid => {
                "Invalid document or account operation. Draft retained; no success claimed. / 文档或账户操作无效，保留草稿"
            }
            Self::Storage => {
                "Cannot complete platform operation. Check access and domain recovery state. / 平台操作失败，请检查权限及领域恢复状态"
            }
            Self::ManagedLocked => {
                "Grok managed model fields are locked by an active or drifted profile. Off the profile first. / Grok 托管模型字段已锁定，请先退出配置模式"
            }
        }
    }
}
fn settings_error(error: platform_settings::Error) -> PlatformError {
    match error {
        platform_settings::Error::Invalid { .. } | platform_settings::Error::Utf8 => {
            PlatformError::Invalid
        }
        platform_settings::Error::Io => PlatformError::Storage,
    }
}
fn domain_error(error: ccr_core::CcrError) -> PlatformError {
    match error {
        ccr_core::CcrError::ValidationError(_)
        | ccr_core::CcrError::ConfigFormatInvalid(_)
        | ccr_core::CcrError::TomlError(_) => PlatformError::Invalid,
        ccr_core::CcrError::PlatformNotSupported(_) => PlatformError::Unsupported,
        _ => PlatformError::Storage,
    }
}

#[derive(Clone)]
pub struct AccountRow {
    pub name: String,
    pub current: bool,
}
/// No Debug/Serialize: documents and original revisions can contain credentials.
pub struct PlatformSession {
    pub roots: PlatformRoots,
    pub document: Document,
    pub snapshot: Snapshot,
    pub accounts: Vec<AccountRow>,
    pub signed_in: bool,
    pub provider_ids: Vec<String>,
    pub surface: Surface,
    pub platform: Platform,
}
pub enum AccountAction {
    Save,
    Switch,
    Delete,
}
pub enum Command {
    Read(Surface),
    Patch {
        session: Arc<PlatformSession>,
        content: String,
    },
    Source {
        session: Arc<PlatformSession>,
        content: String,
    },
    Provider {
        session: Arc<PlatformSession>,
        id: String,
        patch: Option<String>,
    },
    Account {
        session: Arc<PlatformSession>,
        name: String,
        action: AccountAction,
    },
}
pub enum ResultValue {
    Loaded(Arc<PlatformSession>),
    Saved(SaveOutcome),
    AccountChanged { warning: bool },
}

pub enum OAuthControl {
    Callback {
        login_id: String,
        url: String,
    },
    Complete {
        login_id: String,
        name: Option<String>,
    },
    RetryCleanup,
}
pub enum OAuthResult {
    Finished,
    CallbackAccepted,
    AccountCommitted,
    CleanupFinished,
}
fn oauth_error(error: NativeError) -> PlatformError {
    match error {
        NativeError::Invalid => PlatformError::Invalid,
        _ => PlatformError::Storage,
    }
}

fn roots(source: &ConfigSource) -> Result<PlatformRoots, PlatformError> {
    match source {
        ConfigSource::Local => PlatformRoots::capture().map_err(domain_error),
        ConfigSource::Fixture {
            profiles, registry, ..
        } => {
            if !profiles.is_absolute() || !registry.is_absolute() {
                return Err(PlatformError::Invalid);
            }
            let root = profiles
                .parent()
                .ok_or(PlatformError::Invalid)?
                .join("desktop-platform-fixture");
            PlatformRoots::fixture(&root).map_err(domain_error)
        }
    }
}
fn document(
    roots: &PlatformRoots,
    platform: Platform,
    surface: Surface,
) -> Result<Document, PlatformError> {
    let (path, fallback, format) = match (platform, surface) {
        (Platform::Claude, Surface::Settings | Surface::Auth) => {
            (roots.claude.settings_file.clone(), None, Format::Json)
        }
        (Platform::Codex, Surface::Settings | Surface::Auth) => {
            (roots.codex_runtime.join("config.toml"), None, Format::Toml)
        }
        (Platform::Grok, Surface::Settings | Surface::Auth) => {
            (roots.grok_runtime.join("config.toml"), None, Format::Toml)
        }
        (Platform::OpenCode, Surface::Settings | Surface::Providers) => (
            roots.home.join(".config/opencode/opencode.json"),
            Some(roots.home.join(".opencode/config.json")),
            Format::Json,
        ),
        (Platform::OpenCode, Surface::Tui) => (
            roots.home.join(".config/opencode/tui.json"),
            None,
            Format::Json,
        ),
        (Platform::Gemini, Surface::LegacySettings) => (
            roots.home.join(".gemini/antigravity-cli/settings.json"),
            None,
            Format::Json,
        ),
        _ => return Err(PlatformError::Unsupported),
    };
    let backup = if platform == Platform::Grok {
        BackupPolicy::None
    } else {
        BackupPolicy::Dir {
            dir: roots.ccr.join("backups/platform-settings"),
            prefix: format!("{}-settings", platform.short_name()),
        }
    };
    Ok(Document::new(
        path,
        fallback,
        backup,
        Some(roots.locks.clone()),
        format,
    ))
}
fn read(
    source: &ConfigSource,
    platform: Platform,
    surface: Surface,
) -> Result<ResultValue, PlatformError> {
    let roots = roots(source)?;
    let document = document(&roots, platform, surface)?;
    let snapshot = document.read().map_err(settings_error)?;
    let mut accounts = Vec::new();
    let mut signed_in = false;
    if surface == Surface::Auth {
        match platform {
            Platform::Claude => {
                let service = roots.claude_auth();
                let state = service.read_auth_snapshot().map_err(domain_error)?;
                signed_in = state.runtime_usable;
                // 不调用会解析环境外 catalog 的 runtime summary。
                accounts = state
                    .registry
                    .accounts
                    .keys()
                    .map(|name| AccountRow {
                        name: name.clone(),
                        current: state.current_account_name.as_ref() == Some(name),
                    })
                    .collect();
            }
            Platform::Codex => {
                let service = roots.codex_auth();
                let state = service.read_auth_snapshot().map_err(domain_error)?;
                signed_in = state.auth_state.status == ccr_codex::AuthStateStatus::Valid;
                accounts = service
                    .build_account_items(&state)
                    .map_err(domain_error)?
                    .into_iter()
                    .filter(|row| !row.is_virtual)
                    .map(|row| AccountRow {
                        name: row.name,
                        current: row.is_current,
                    })
                    .collect();
            }
            Platform::Grok => {
                signed_in = roots
                    .grok_runtime
                    .join("auth.json")
                    .try_exists()
                    .map_err(|_| PlatformError::Storage)?;
            }
            _ => return Err(PlatformError::Unsupported),
        }
    }
    let provider_ids = if surface == Surface::Providers {
        platform_settings::json::providers(&snapshot)
            .map_err(settings_error)?
            .as_object()
            .ok_or(PlatformError::Invalid)?
            .keys()
            .cloned()
            .collect()
    } else {
        Vec::new()
    };
    Ok(ResultValue::Loaded(Arc::new(PlatformSession {
        roots,
        document,
        snapshot,
        accounts,
        signed_in,
        provider_ids,
        platform,
        surface,
    })))
}
fn grok_managed_guard(session: &PlatformSession, content: &str) -> Result<(), PlatformError> {
    use ccr_cli::platforms::grok::{GrokActivationState, GrokPlatform};
    use ccr_config::PlatformPaths;
    let roots = &session.roots;
    let directory = roots.ccr.join("platforms/grok");
    let platform = GrokPlatform::from_parts(
        PlatformPaths {
            root: roots.ccr.clone(),
            platform_dir: directory.clone(),
            profiles_file: directory.join("profiles.toml"),
            registry_file: roots.ccr.join("config.toml"),
            settings_file: roots.grok_runtime.join("config.toml"),
            history_file: directory.join("history.json"),
            backups_dir: directory.join("backups"),
        },
        roots.grok_runtime.join("config.toml"),
    );
    if matches!(
        platform.inspect_activation_state().map_err(domain_error)?,
        GrokActivationState::Inactive
    ) {
        return Ok(());
    }
    let before = if session.snapshot.exists {
        toml::from_str::<toml::Value>(&session.snapshot.content)
            .map_err(|_| PlatformError::Invalid)?
    } else {
        toml::Value::Table(Default::default())
    };
    let after = toml::from_str::<toml::Value>(content).map_err(|_| PlatformError::Invalid)?;
    for key in ["default", "default_reasoning_effort"] {
        if before.get("models").and_then(|value| value.get(key))
            != after.get("models").and_then(|value| value.get(key))
        {
            return Err(PlatformError::ManagedLocked);
        }
    }
    if before.get("model").and_then(|value| value.get("custom"))
        != after.get("model").and_then(|value| value.get("custom"))
    {
        return Err(PlatformError::ManagedLocked);
    }
    Ok(())
}
fn execute(
    source: &ConfigSource,
    platform: Platform,
    command: Command,
) -> Result<ResultValue, PlatformError> {
    if let Command::Read(surface) = command {
        return read(source, platform, surface);
    }
    let session = match &command {
        Command::Patch { session, .. }
        | Command::Source { session, .. }
        | Command::Provider { session, .. }
        | Command::Account { session, .. } => session,
        Command::Read(_) => return Err(PlatformError::Invalid),
    };
    if session.platform != platform {
        return Err(PlatformError::Unsupported);
    }
    LockManager::with_scoped_directory(&session.roots.locks, || {
        Ok((|| match &command {
            Command::Patch { session, content } => {
                let patch: Value =
                    serde_json::from_str(content).map_err(|_| PlatformError::Invalid)?;
                if !patch.is_object() || patch.as_object().is_some_and(|map| map.is_empty()) {
                    return Err(PlatformError::Invalid);
                }
                let outcome = match platform {
                    Platform::Codex => platform_settings::codex::save_patch(
                        &session.document,
                        &session.snapshot,
                        &patch,
                    ),
                    Platform::Claude | Platform::OpenCode | Platform::Gemini => {
                        platform_settings::json::save_patch(
                            &session.document,
                            &session.snapshot,
                            &patch,
                        )
                    }
                    _ => return Err(PlatformError::Unsupported),
                }
                .map_err(settings_error)?;
                Ok(ResultValue::Saved(outcome))
            }
            Command::Source { session, content } => {
                if platform == Platform::Grok {
                    grok_managed_guard(session, content)?;
                }
                Ok(ResultValue::Saved(
                    session
                        .document
                        .save_text(content, &session.snapshot)
                        .map_err(settings_error)?,
                ))
            }
            Command::Provider { session, id, patch } => {
                if platform != Platform::OpenCode || session.surface != Surface::Providers {
                    return Err(PlatformError::Unsupported);
                }
                let patch = patch
                    .as_ref()
                    .map(|text| {
                        serde_json::from_str::<Value>(text).map_err(|_| PlatformError::Invalid)
                    })
                    .transpose()?;
                Ok(ResultValue::Saved(
                    platform_settings::json::save_provider(
                        &session.document,
                        &session.snapshot,
                        id,
                        patch.as_ref(),
                    )
                    .map_err(settings_error)?,
                ))
            }
            Command::Account {
                session,
                name,
                action,
            } => {
                if session.surface != Surface::Auth {
                    return Err(PlatformError::Invalid);
                }
                let mut warning = false;
                match platform {
                    Platform::Claude => {
                        let service = session.roots.claude_auth();
                        match action {
                            AccountAction::Save => {
                                service
                                    .save_current(name, None, false)
                                    .map_err(domain_error)?;
                            }
                            AccountAction::Switch => {
                                let outcome = service.switch_account(name).map_err(domain_error)?;
                                warning = !outcome.warnings.is_empty()
                                    || !outcome.remaining_suppressors.is_empty();
                            }
                            AccountAction::Delete => {
                                service.delete_account(name).map_err(domain_error)?
                            }
                        }
                    }
                    Platform::Codex => {
                        let service = session.roots.codex_auth();
                        match action {
                            AccountAction::Save => service
                                .save_current(name, None, false)
                                .map_err(domain_error)?,
                            AccountAction::Switch => {
                                service.switch_account(name).map_err(domain_error)?
                            }
                            AccountAction::Delete => {
                                service.delete_account(name).map_err(domain_error)?
                            }
                        }
                    }
                    _ => return Err(PlatformError::Unsupported),
                }
                Ok(ResultValue::AccountChanged { warning })
            }
            Command::Read(_) => Err(PlatformError::Invalid),
        })())
    })
    .map_err(domain_error)?
}
impl Services {
    pub fn platform_oauth_status(&self) -> Option<NativeStatus> {
        self.platform_oauth.status()
    }
    pub fn platform_oauth_cancel(&self) {
        self.platform_oauth.request_cancel();
    }
    pub fn platform_oauth_active(&self) -> bool {
        self.platform_oauth.is_active()
    }
    pub fn platform_oauth_start(
        &self,
        key: RequestKey,
        session: Arc<PlatformSession>,
    ) -> Result<TaskSubscription<OAuthResult, PlatformError>, AdmissionError> {
        if !matches!(key.environment, EnvironmentId::Local(_))
            || key.platform != Platform::Codex
            || session.platform != Platform::Codex
            || session.surface != Surface::Auth
        {
            return self
                .executor
                .read(key, |_, _| Err(PlatformError::Unsupported));
        }
        if self.platform_oauth.claim().is_err() {
            return Err(AdmissionError::Full);
        }
        let owner = self.platform_oauth.clone();
        let task = self.executor.job(key, move |cancel, progress| {
            match owner.run(
                session.roots.clone(),
                || cancel.is_cancelled(),
                move || {
                    progress.report(ccr_app::runtime::Progress {
                        completed: 1,
                        total: None,
                    });
                },
            ) {
                Ok(()) => Completion::Success(OAuthResult::Finished),
                Err(NativeError::Cancelled) => Completion::Cancelled,
                Err(NativeError::CleanupFailed) => {
                    Completion::CleanupFailed(PlatformError::Storage)
                }
                Err(error) => Completion::Failed(oauth_error(error)),
            }
        });
        if task.is_err() {
            self.platform_oauth.abandon_claim();
        }
        task
    }
    pub fn platform_oauth_control(
        &self,
        key: RequestKey,
        control: OAuthControl,
    ) -> Result<TaskSubscription<OAuthResult, PlatformError>, AdmissionError> {
        let owner = self.platform_oauth.clone();
        self.executor.job(key.clone(), move |_, _| {
            if !matches!(key.environment, EnvironmentId::Local(_))
                || key.platform != Platform::Codex
            {
                return Completion::Failed(PlatformError::Unsupported);
            }
            let result = match control {
                OAuthControl::Callback { login_id, url } => owner
                    .submit(&login_id, &url)
                    .map(|()| OAuthResult::CallbackAccepted),
                OAuthControl::Complete { login_id, name } => owner
                    .complete(&login_id, name)
                    .map(|_| OAuthResult::AccountCommitted),
                OAuthControl::RetryCleanup => {
                    owner.retry_cleanup().map(|()| OAuthResult::CleanupFinished)
                }
            };
            match result {
                Ok(result) => Completion::Success(result),
                Err(error) => Completion::Failed(oauth_error(error)),
            }
        })
    }
    pub fn platform_request(
        &self,
        key: RequestKey,
        command: Command,
    ) -> Result<TaskSubscription<ResultValue, PlatformError>, AdmissionError> {
        let source = self.source.clone();
        let supported = matches!(key.environment, EnvironmentId::Local(_));
        let platform = key.platform;
        let is_read = matches!(command, Command::Read(_));
        let run = move || {
            if supported {
                execute(&source, platform, command)
            } else {
                Err(PlatformError::Unsupported)
            }
        };
        if is_read {
            self.executor.read(key, move |_, _| run())
        } else {
            self.executor.job(key, move |_, _| match run() {
                Ok(value) => Completion::Success(value),
                Err(error) => Completion::Failed(error),
            })
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    #[test]
    fn explicit_fixture_read_and_two_native_clients_preserve_unknown_fields() {
        let temp = tempfile::tempdir().unwrap();
        let source = ConfigSource::Fixture {
            profiles: temp.path().join("profiles.toml"),
            registry: temp.path().join("registry.toml"),
            delay: std::time::Duration::ZERO,
        };
        let root = roots(&source).unwrap();
        let doc = document(&root, Platform::Claude, Surface::Settings).unwrap();
        assert!(!doc.read().unwrap().exists);
        assert!(!root.home.exists());
        std::fs::create_dir_all(doc.path().parent().unwrap()).unwrap();
        std::fs::write(
            doc.path(),
            r#"{"env":{"unknown":"retained","MODEL":"old"},"extension":7}"#,
        )
        .unwrap();
        let ResultValue::Loaded(a) = read(&source, Platform::Claude, Surface::Settings).unwrap()
        else {
            panic!("snapshot")
        };
        let ResultValue::Loaded(b) = read(&source, Platform::Claude, Surface::Settings).unwrap()
        else {
            panic!("snapshot")
        };
        assert!(matches!(
            execute(
                &source,
                Platform::Claude,
                Command::Patch {
                    session: a,
                    content: r#"{"env":{"MODEL":"new"}}"#.into()
                }
            )
            .unwrap(),
            ResultValue::Saved(SaveOutcome::Saved { .. })
        ));
        assert!(matches!(
            execute(
                &source,
                Platform::Claude,
                Command::Patch {
                    session: b,
                    content: r#"{"extension":9}"#.into()
                }
            )
            .unwrap(),
            ResultValue::Saved(SaveOutcome::Conflict)
        ));
        let saved: Value = serde_json::from_str(&doc.read().unwrap().content).unwrap();
        assert_eq!(saved["env"]["unknown"], "retained");
        assert_eq!(saved["extension"], 7);
    }
    #[test]
    fn descriptor_truth_has_no_antigravity_settings_or_opencode_auth() {
        assert!(
            !DESCRIPTORS
                .iter()
                .any(|item| item.platform == Platform::Gemini)
        );
        assert!(
            !DESCRIPTORS
                .iter()
                .find(|item| item.platform == Platform::OpenCode)
                .unwrap()
                .auth
        );
    }
}
