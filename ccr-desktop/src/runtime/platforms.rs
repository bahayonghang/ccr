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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileState {
    Missing,
    Invalid,
    Present,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldFact {
    pub key: &'static str,
    pub value: String,
}
/// Local credential observation. None of these values claim a remote login.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CredentialState {
    PlatformUnsupported,
    StoreUnsupported,
    Unreadable,
    NotDetected,
    Detected,
    Invalid,
    /// Demo roots must not query the machine Grok account store.
    MachineStoreUnchecked,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderReport {
    Unsupported,
    Skipped,
    Invalid,
    Ready { hidden: usize },
}
/// No Debug/Serialize: documents and original revisions can contain credentials.
pub struct PlatformSession {
    pub roots: PlatformRoots,
    pub document: Document,
    pub snapshot: Snapshot,
    pub accounts: Vec<AccountRow>,
    /// Local reads never set this. Credential detection stays on `credential`.
    pub signed_in: bool,
    pub provider_ids: Vec<String>,
    pub provider_report: ProviderReport,
    pub surface: Surface,
    pub platform: Platform,
    pub file_state: FileState,
    pub file_label: &'static str,
    pub fields: Vec<FieldFact>,
    pub credential: CredentialState,
    pub runtime_auth_present: Option<bool>,
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
fn file_label(platform: Platform, surface: Surface, snapshot: &Snapshot) -> &'static str {
    match (platform, surface) {
        (Platform::Claude, _) => "settings.json",
        (Platform::Codex, _) => "config.toml",
        (Platform::Grok, _) => "config.toml",
        (Platform::OpenCode, Surface::Tui) => "tui.json",
        (Platform::OpenCode, _) => {
            if snapshot.exists
                && snapshot.source.file_name().and_then(|name| name.to_str()) == Some("config.json")
            {
                "legacy config.json"
            } else {
                "opencode.json"
            }
        }
        (Platform::Gemini, _) => "legacy settings.json",
        (Platform::Qwen | Platform::Droid, _) => "settings",
    }
}
fn safe_scalar(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty() || text.len() > 80 || text.chars().any(char::is_control) {
        return None;
    }
    let lower = text.to_ascii_lowercase();
    if lower.starts_with("sk-")
        || lower.starts_with("eyj")
        || lower.contains("bearer")
        || lower.contains("api_key")
        || lower.contains("token")
    {
        return None;
    }
    Some(text.to_string())
}
fn safe_name(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty() || text.len() > 64 || text.chars().any(char::is_control) {
        return None;
    }
    let lower = text.to_ascii_lowercase();
    if lower.starts_with("sk-") || lower.starts_with("eyj") || lower.contains("bearer") {
        return None;
    }
    Some(text.to_string())
}
fn json_scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => safe_scalar(text),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) if !value.is_f64() => Some(value.to_string()),
        _ => None,
    }
}
fn toml_scalar(value: &toml::Value) -> Option<String> {
    match value {
        toml::Value::String(text) => safe_scalar(text),
        toml::Value::Boolean(value) => Some(value.to_string()),
        toml::Value::Integer(value) => Some(value.to_string()),
        _ => None,
    }
}
fn push_json(fields: &mut Vec<FieldFact>, key: &'static str, value: Option<&Value>) {
    if let Some(value) = value.and_then(json_scalar) {
        fields.push(FieldFact { key, value });
    }
}
fn push_toml(fields: &mut Vec<FieldFact>, key: &'static str, value: Option<&toml::Value>) {
    if let Some(value) = value.and_then(toml_scalar) {
        fields.push(FieldFact { key, value });
    }
}
fn project_fields(
    platform: Platform,
    surface: Surface,
    snapshot: &Snapshot,
) -> (FileState, Vec<FieldFact>) {
    if !snapshot.exists {
        return (FileState::Missing, Vec::new());
    }
    let format = match (platform, surface) {
        (Platform::Codex | Platform::Grok, _) => Format::Toml,
        _ => Format::Json,
    };
    if platform_settings::validate(format, &snapshot.content).is_err() {
        return (FileState::Invalid, Vec::new());
    }
    if platform == Platform::Gemini {
        return (FileState::Present, Vec::new());
    }
    let mut fields = Vec::new();
    match (platform, format) {
        (Platform::Claude, Format::Json) | (Platform::OpenCode, Format::Json) => {
            let Ok(value) = serde_json::from_str::<Value>(&snapshot.content) else {
                return (FileState::Invalid, Vec::new());
            };
            if surface == Surface::Tui {
                push_json(&mut fields, "theme", value.get("theme"));
                push_json(&mut fields, "mouse", value.get("mouse"));
            } else if platform == Platform::Claude {
                push_json(&mut fields, "model", value.get("model"));
                push_json(&mut fields, "effortLevel", value.get("effortLevel"));
                push_json(&mut fields, "theme", value.get("theme"));
                push_json(&mut fields, "language", value.get("language"));
                push_json(
                    &mut fields,
                    "autoUpdatesChannel",
                    value.get("autoUpdatesChannel"),
                );
                push_json(
                    &mut fields,
                    "permissionMode",
                    value
                        .get("permissions")
                        .and_then(|entry| entry.get("defaultMode")),
                );
            } else {
                push_json(&mut fields, "model", value.get("model"));
                push_json(&mut fields, "small_model", value.get("small_model"));
                push_json(&mut fields, "default_agent", value.get("default_agent"));
                push_json(&mut fields, "share", value.get("share"));
                push_json(&mut fields, "snapshot", value.get("snapshot"));
                push_json(&mut fields, "autoupdate", value.get("autoupdate"));
            }
        }
        (Platform::Codex, Format::Toml) => {
            let Ok(value) = toml::from_str::<toml::Value>(&snapshot.content) else {
                return (FileState::Invalid, Vec::new());
            };
            for (key, path) in [
                ("model", "model"),
                ("model_provider", "model_provider"),
                ("model_reasoning_effort", "model_reasoning_effort"),
                ("approval_policy", "approval_policy"),
                ("sandbox_mode", "sandbox_mode"),
                ("personality", "personality"),
            ] {
                push_toml(&mut fields, key, value.get(path));
            }
        }
        (Platform::Grok, Format::Toml) => {
            let Ok(value) = toml::from_str::<toml::Value>(&snapshot.content) else {
                return (FileState::Invalid, Vec::new());
            };
            push_toml(
                &mut fields,
                "default_model",
                value.get("models").and_then(|entry| entry.get("default")),
            );
            push_toml(
                &mut fields,
                "reasoning_effort",
                value
                    .get("models")
                    .and_then(|entry| entry.get("default_reasoning_effort")),
            );
            push_toml(
                &mut fields,
                "theme",
                value.get("ui").and_then(|entry| entry.get("theme")),
            );
            push_toml(
                &mut fields,
                "channel",
                value.get("cli").and_then(|entry| entry.get("channel")),
            );
            push_toml(
                &mut fields,
                "auto_update",
                value.get("cli").and_then(|entry| entry.get("auto_update")),
            );
            push_toml(
                &mut fields,
                "auto_compact",
                value
                    .get("session")
                    .and_then(|entry| entry.get("auto_compact_threshold_percent")),
            );
        }
        _ => {}
    }
    (FileState::Present, fields)
}
fn remember_account(accounts: &mut Vec<AccountRow>, name: &str, current: bool) {
    let Some(name) = safe_name(name) else {
        return;
    };
    accounts.push(AccountRow { name, current });
}
fn project_auth(roots: &PlatformRoots, platform: Platform, surface: Surface) -> AuthProjection {
    let applicable = matches!(
        (platform, surface),
        (
            Platform::Claude | Platform::Codex | Platform::Grok,
            Surface::Settings | Surface::Auth
        )
    );
    if !applicable {
        return AuthProjection {
            credential: CredentialState::PlatformUnsupported,
            accounts: Vec::new(),
            runtime_auth_present: None,
            signed_in: false,
        };
    }
    match platform {
        Platform::Claude => {
            let service = roots.claude_auth();
            match service.read_auth_snapshot() {
                Ok(state) => {
                    let mut accounts = Vec::new();
                    for name in state.registry.accounts.keys() {
                        remember_account(
                            &mut accounts,
                            name,
                            state.current_account_name.as_ref() == Some(name),
                        );
                    }
                    AuthProjection {
                        credential: if state.runtime_usable {
                            CredentialState::Detected
                        } else {
                            CredentialState::NotDetected
                        },
                        signed_in: false,
                        runtime_auth_present: None,
                        accounts,
                    }
                }
                Err(_) => AuthProjection::unreadable(),
            }
        }
        Platform::Codex => {
            let service = roots.codex_auth();
            match service.read_auth_snapshot() {
                Ok(state) => {
                    let credential = match state.auth_state.status {
                        ccr_codex::AuthStateStatus::Valid => CredentialState::Detected,
                        ccr_codex::AuthStateStatus::Invalid => CredentialState::Invalid,
                        ccr_codex::AuthStateStatus::Missing => CredentialState::NotDetected,
                        ccr_codex::AuthStateStatus::Unsupported => {
                            CredentialState::StoreUnsupported
                        }
                    };
                    let Ok(rows) = service.build_account_items(&state) else {
                        return AuthProjection::unreadable();
                    };
                    let mut accounts = Vec::new();
                    for row in rows.into_iter().filter(|row| !row.is_virtual) {
                        remember_account(&mut accounts, &row.name, row.is_current);
                    }
                    AuthProjection {
                        signed_in: false,
                        credential,
                        accounts,
                        runtime_auth_present: None,
                    }
                }
                Err(_) => AuthProjection::unreadable(),
            }
        }
        Platform::Grok if roots.fixture => {
            let present = roots.grok_runtime.join("auth.json").try_exists().ok();
            AuthProjection {
                credential: if present.is_none() {
                    CredentialState::Unreadable
                } else {
                    CredentialState::MachineStoreUnchecked
                },
                accounts: Vec::new(),
                runtime_auth_present: present,
                signed_in: false,
            }
        }
        Platform::Grok => {
            match ccr_cli::services::grok_auth_service::GrokAuthService::new().read_snapshot() {
                Ok(state) => {
                    let mut accounts = Vec::new();
                    for account in &state.accounts {
                        remember_account(&mut accounts, &account.name, account.local_match);
                    }
                    let credential = if state.runtime_error.is_some() {
                        CredentialState::Invalid
                    } else if state.runtime_present {
                        CredentialState::Detected
                    } else {
                        CredentialState::NotDetected
                    };
                    AuthProjection {
                        credential,
                        accounts,
                        runtime_auth_present: Some(state.runtime_present),
                        signed_in: false,
                    }
                }
                Err(_) => AuthProjection::unreadable(),
            }
        }
        _ => AuthProjection {
            credential: CredentialState::PlatformUnsupported,
            accounts: Vec::new(),
            runtime_auth_present: None,
            signed_in: false,
        },
    }
}
struct AuthProjection {
    credential: CredentialState,
    accounts: Vec<AccountRow>,
    runtime_auth_present: Option<bool>,
    signed_in: bool,
}
impl AuthProjection {
    fn unreadable() -> Self {
        Self {
            credential: CredentialState::Unreadable,
            accounts: Vec::new(),
            runtime_auth_present: None,
            signed_in: false,
        }
    }
}
fn project_providers(
    platform: Platform,
    surface: Surface,
    file_state: FileState,
    snapshot: &Snapshot,
) -> (ProviderReport, Vec<String>) {
    let applicable =
        platform == Platform::OpenCode && matches!(surface, Surface::Settings | Surface::Providers);
    if !applicable {
        return (ProviderReport::Unsupported, Vec::new());
    }
    if file_state == FileState::Missing {
        return (ProviderReport::Skipped, Vec::new());
    }
    if file_state == FileState::Invalid {
        return (ProviderReport::Invalid, Vec::new());
    }
    match platform_settings::json::providers(snapshot) {
        Ok(value) => {
            let mut ids = Vec::new();
            let mut hidden = 0;
            if let Some(map) = value.as_object() {
                for key in map.keys() {
                    if let Some(name) = safe_name(key) {
                        ids.push(name);
                    } else {
                        hidden += 1;
                    }
                }
            }
            (ProviderReport::Ready { hidden }, ids)
        }
        Err(_) => (ProviderReport::Invalid, Vec::new()),
    }
}
fn read(
    source: &ConfigSource,
    platform: Platform,
    surface: Surface,
) -> Result<ResultValue, PlatformError> {
    let roots = roots(source)?;
    let document = document(&roots, platform, surface)?;
    let snapshot = document.read().map_err(settings_error)?;
    let (file_state, fields) = project_fields(platform, surface, &snapshot);
    let auth = project_auth(&roots, platform, surface);
    let (provider_report, provider_ids) =
        project_providers(platform, surface, file_state, &snapshot);
    let file_label = file_label(platform, surface, &snapshot);
    Ok(ResultValue::Loaded(Arc::new(PlatformSession {
        roots,
        document,
        snapshot,
        accounts: auth.accounts,
        signed_in: auth.signed_in,
        provider_ids,
        provider_report,
        platform,
        surface,
        file_state,
        file_label,
        fields,
        credential: auth.credential,
        runtime_auth_present: auth.runtime_auth_present,
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
                if platform == Platform::Gemini {
                    return Err(PlatformError::Unsupported);
                }
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
                    Platform::Claude | Platform::OpenCode => platform_settings::json::save_patch(
                        &session.document,
                        &session.snapshot,
                        &patch,
                    ),
                    _ => return Err(PlatformError::Unsupported),
                }
                .map_err(settings_error)?;
                Ok(ResultValue::Saved(outcome))
            }
            Command::Source { session, content } => {
                if platform == Platform::Gemini {
                    return Err(PlatformError::Unsupported);
                }
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
    use std::sync::Arc;
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
    fn fixture_source(root: &std::path::Path) -> ConfigSource {
        ConfigSource::Fixture {
            profiles: root.join("profiles.toml"),
            registry: root.join("registry.toml"),
            delay: std::time::Duration::ZERO,
        }
    }
    fn loaded(source: &ConfigSource, platform: Platform, surface: Surface) -> Arc<PlatformSession> {
        match read(source, platform, surface).unwrap() {
            ResultValue::Loaded(session) => session,
            _ => panic!("expected a loaded session"),
        }
    }
    #[test]
    fn settings_projection_reports_missing_invalid_and_hides_secret_values() {
        let temp = tempfile::tempdir().unwrap();
        let source = fixture_source(temp.path());
        let home = roots(&source).unwrap().home;
        let missing = loaded(&source, Platform::Gemini, Surface::LegacySettings);
        assert_eq!(missing.file_state, FileState::Missing);
        assert_eq!(missing.credential, CredentialState::PlatformUnsupported);
        assert!(missing.fields.is_empty());
        assert!(!home.join(".gemini").exists());

        let claude = document(
            &roots(&source).unwrap(),
            Platform::Claude,
            Surface::Settings,
        )
        .unwrap();
        std::fs::create_dir_all(claude.path().parent().unwrap()).unwrap();
        std::fs::write(
            claude.path(),
            r#"{"model":"sonnet","env":{"ANTHROPIC_API_KEY":"sk-platform-route-sentinel"},"permissions":{"defaultMode":"plan"}}"#,
        )
        .unwrap();
        let present = loaded(&source, Platform::Claude, Surface::Settings);
        assert_eq!(present.file_state, FileState::Present);
        assert_eq!(present.credential, CredentialState::NotDetected);
        assert!(!present.signed_in);
        assert!(present.accounts.is_empty());
        assert!(
            present
                .fields
                .iter()
                .any(|field| field.key == "model" && field.value == "sonnet")
        );
        assert!(
            present
                .fields
                .iter()
                .any(|field| field.key == "permissionMode" && field.value == "plan")
        );
        assert!(!format!("{:?}", present.fields).contains("sk-platform-route-sentinel"));
        assert!(!format!("{:?}", present.fields).contains("ANTHROPIC_API_KEY"));

        std::fs::write(claude.path(), "{not json").unwrap();
        let invalid = loaded(&source, Platform::Claude, Surface::Settings);
        assert_eq!(invalid.file_state, FileState::Invalid);
        assert!(invalid.fields.is_empty());
        assert_eq!(std::fs::read_to_string(claude.path()).unwrap(), "{not json");
    }
    #[test]
    fn opencode_lists_provider_ids_and_antigravity_hides_raw_values() {
        let temp = tempfile::tempdir().unwrap();
        let source = fixture_source(temp.path());
        let root = roots(&source).unwrap();
        let opencode = root.home.join(".config/opencode/opencode.json");
        std::fs::create_dir_all(opencode.parent().unwrap()).unwrap();
        std::fs::write(
            &opencode,
            r#"{"model":"opencode-model","provider":{"fixture-provider":{"apiKey":"sk-platform-route-sentinel"}}}"#,
        )
        .unwrap();
        let session = loaded(&source, Platform::OpenCode, Surface::Settings);
        assert_eq!(session.provider_ids, vec!["fixture-provider".to_string()]);
        assert!(matches!(
            session.provider_report,
            ProviderReport::Ready { hidden: 0 }
        ));
        assert!(
            session
                .fields
                .iter()
                .any(|field| field.key == "model" && field.value == "opencode-model")
        );
        assert_eq!(session.credential, CredentialState::PlatformUnsupported);
        let rendered = format!("{:?} {:?}", session.fields, session.provider_ids);
        assert!(!rendered.contains("sk-platform-route-sentinel"));
        assert!(!rendered.contains("apiKey"));

        let antigravity = root.home.join(".gemini/antigravity-cli/settings.json");
        std::fs::create_dir_all(antigravity.parent().unwrap()).unwrap();
        std::fs::write(
            &antigravity,
            r#"{"apiKey":"sk-platform-route-sentinel","theme":"dark"}"#,
        )
        .unwrap();
        let legacy = loaded(&source, Platform::Gemini, Surface::LegacySettings);
        assert_eq!(legacy.file_state, FileState::Present);
        assert!(legacy.fields.is_empty());
        assert!(!legacy.signed_in);
        assert_eq!(legacy.credential, CredentialState::PlatformUnsupported);
        assert!(!format!("{:?}", legacy.fields).contains("sk-platform-route-sentinel"));
        assert!(!format!("{:?}", legacy.fields).contains("dark"));
        let before = std::fs::read_to_string(&antigravity).unwrap();
        assert!(matches!(
            execute(
                &source,
                Platform::Gemini,
                Command::Patch {
                    session: legacy.clone(),
                    content: r#"{"theme":"dark"}"#.into(),
                },
            ),
            Err(PlatformError::Unsupported)
        ));
        assert!(matches!(
            execute(
                &source,
                Platform::Gemini,
                Command::Source {
                    session: legacy,
                    content: "{}".into(),
                },
            ),
            Err(PlatformError::Unsupported)
        ));
        assert_eq!(std::fs::read_to_string(&antigravity).unwrap(), before);
    }
    #[test]
    fn codex_auto_store_and_grok_demo_do_not_invent_login() {
        let temp = tempfile::tempdir().unwrap();
        let source = fixture_source(temp.path());
        let root = roots(&source).unwrap();
        let codex = root.codex_runtime.join("config.toml");
        std::fs::create_dir_all(codex.parent().unwrap()).unwrap();
        std::fs::write(
            &codex,
            "model = \"gpt-5\"\napi_key = \"sk-platform-route-sentinel\"\n",
        )
        .unwrap();
        let codex_session = loaded(&source, Platform::Codex, Surface::Settings);
        assert_eq!(codex_session.credential, CredentialState::StoreUnsupported);
        assert!(
            codex_session
                .fields
                .iter()
                .any(|field| field.key == "model" && field.value == "gpt-5")
        );
        assert!(!format!("{:?}", codex_session.fields).contains("sk-platform-route-sentinel"));
        assert!(!codex_session.signed_in);

        let grok = root.grok_runtime.join("config.toml");
        std::fs::create_dir_all(grok.parent().unwrap()).unwrap();
        std::fs::write(
            &grok,
            "[models]\ndefault = \"grok-3\"\napi_key = \"sk-platform-route-sentinel\"\n",
        )
        .unwrap();
        let grok_session = loaded(&source, Platform::Grok, Surface::Settings);
        assert_eq!(
            grok_session.credential,
            CredentialState::MachineStoreUnchecked
        );
        assert_eq!(grok_session.runtime_auth_present, Some(false));
        assert!(!grok_session.signed_in);
        assert!(grok_session.accounts.is_empty());
        assert!(
            grok_session
                .fields
                .iter()
                .any(|field| field.key == "default_model" && field.value == "grok-3")
        );
        assert!(!format!("{:?}", grok_session.fields).contains("sk-platform-route-sentinel"));
        assert!(!root.grok_runtime.join("auth.json").exists());
    }
    #[test]
    fn opt_in_local_projection_prints_safe_fields_only() {
        if std::env::var_os("CCR_DESKTOP_LOCAL_PLATFORM_PROBE").is_none() {
            return;
        }
        let source = ConfigSource::Local;
        for (name, platform, surface) in [
            ("claude-settings", Platform::Claude, Surface::Settings),
            ("codex-settings", Platform::Codex, Surface::Settings),
            ("grok-settings", Platform::Grok, Surface::Settings),
            ("opencode-settings", Platform::OpenCode, Surface::Settings),
            ("opencode-tui", Platform::OpenCode, Surface::Tui),
            (
                "antigravity-settings",
                Platform::Gemini,
                Surface::LegacySettings,
            ),
        ] {
            match read(&source, platform, surface) {
                Ok(ResultValue::Loaded(session)) => {
                    let field_keys = session
                        .fields
                        .iter()
                        .map(|field| field.key)
                        .collect::<Vec<_>>();
                    println!(
                        "{name} file={:?} label={} credential={:?} signed_in={} runtime_auth={:?} accounts={} providers={:?} field_keys={field_keys:?}",
                        session.file_state,
                        session.file_label,
                        session.credential,
                        session.signed_in,
                        session.runtime_auth_present,
                        session.accounts.len(),
                        session.provider_report,
                    );
                }
                Ok(_) => println!("{name} unexpected-result"),
                Err(error) => println!("{name} error={error:?}"),
            }
        }
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
