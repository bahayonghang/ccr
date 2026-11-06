mod assets;
mod management;
use super::*;
pub use assets::{AssetDirection, AssetRequest, AssetResult, AssetRow};
use ccr_app::platform_settings::{Document, Format, SaveOutcome};
use ccr_app::runtime::Completion;
use ccr_core::{Secret, core::BackupPolicy};
use ccr_sync::{SyncConfig, SyncService};
pub use management::FolderResult;
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

pub enum PasswordUpdate {
    Keep,
    Replace(Secret),
    Clear,
}

pub struct SettingsPatch {
    pub revision: String,
    pub enabled: bool,
    pub url: Option<String>,
    pub username: Option<String>,
    pub base_remote_path: Option<String>,
    pub password: PasswordUpdate,
}

pub struct FolderPatch {
    pub name: Option<String>,
    pub local_path: Option<String>,
    pub remote_path: Option<String>,
}

pub enum FolderChange {
    Add(FolderPatch),
    Edit {
        identity: String,
        patch: FolderPatch,
    },
    Delete {
        identity: String,
    },
    Enabled {
        identity: String,
        enabled: bool,
    },
}

pub enum Operation {
    PrepareClear {
        revision: String,
    },
    Clear {
        revision: String,
        session: Arc<ccr_sync::sync::clear::ClearSession>,
    },
    Batch {
        revision: String,
        upload: bool,
    },
    Assets(AssetRequest),
    AssetInventory {
        revision: String,
        probe: bool,
    },
    Save(SettingsPatch),
    Folder {
        revision: String,
        change: FolderChange,
    },
    Test {
        revision: String,
    },
    Push {
        revision: String,
        folder: usize,
    },
    Pull {
        revision: String,
        folder: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationKind {
    PrepareClear,
    Clear,
    BatchPush,
    BatchPull,
    Assets,
    AssetInventory,
    Save,
    Folder,
    Test,
    Push,
    Pull,
}

impl Operation {
    fn kind(&self) -> OperationKind {
        match self {
            Self::PrepareClear { .. } => OperationKind::PrepareClear,
            Self::Clear { .. } => OperationKind::Clear,
            Self::Batch { upload, .. } => {
                if *upload {
                    OperationKind::BatchPush
                } else {
                    OperationKind::BatchPull
                }
            }
            Self::Assets(_) => OperationKind::Assets,
            Self::AssetInventory { .. } => OperationKind::AssetInventory,
            Self::Save(_) => OperationKind::Save,
            Self::Folder { .. } => OperationKind::Folder,
            Self::Test { .. } => OperationKind::Test,
            Self::Push { .. } => OperationKind::Push,
            Self::Pull { .. } => OperationKind::Pull,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncOutcome {
    ClearPrepared(Arc<ccr_sync::sync::clear::ClearSession>),
    Cleared,
    Batch(Vec<FolderResult>),
    Assets(Vec<AssetResult>),
    AssetInventory(Vec<AssetRow>),
    Saved,
    FolderSaved,
    Connected,
    Pushed,
    Pulled { backup: Option<String> },
}

#[derive(Clone, Debug, Default)]
pub struct OperationState {
    pub busy: bool,
    pub kind: Option<OperationKind>,
    pub environment: Option<EnvironmentId>,
    pub result: Option<Result<SyncOutcome, SyncError>>,
}

pub struct SyncOperations {
    state: Mutex<OperationState>,
    updates: watch::Sender<Arc<OperationState>>,
    #[cfg(test)]
    loopback_fixture: bool,
}

impl Default for SyncOperations {
    fn default() -> Self {
        Self {
            state: Mutex::new(OperationState::default()),
            updates: watch::channel(Arc::new(OperationState::default())).0,
            #[cfg(test)]
            loopback_fixture: false,
        }
    }
}

impl SyncOperations {
    fn finish(&self, result: Result<SyncOutcome, SyncError>) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.busy = false;
        state.result = Some(result);
        self.updates.send_replace(Arc::new(state.clone()));
    }
}

struct Pending {
    owner: Arc<SyncOperations>,
    finished: bool,
}

impl Drop for Pending {
    fn drop(&mut self) {
        if !self.finished {
            self.owner.finish(Err(SyncError::Interrupted));
        }
    }
}

impl Services {
    pub fn sync_operation_state(&self) -> Arc<OperationState> {
        self.sync_operations.updates.borrow().clone()
    }

    pub fn observe_sync(&self) -> watch::Receiver<Arc<OperationState>> {
        self.sync_operations.updates.subscribe()
    }

    /// Side effects outlive a detached view. Active transfers finish using the
    /// domain's bounded request/operation budgets; no false cancellation is exposed.
    pub fn start_sync(&self, key: RequestKey, operation: Operation) -> Result<(), SyncError> {
        if !matches!(key.environment, EnvironmentId::Local(_)) {
            return Err(SyncError::Unsupported);
        }
        let owner = self.sync_operations.clone();
        {
            let mut state = owner
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.busy {
                return Err(SyncError::Busy);
            }
            *state = OperationState {
                busy: true,
                kind: Some(operation.kind()),
                environment: Some(key.environment.clone()),
                result: None,
            };
            owner.updates.send_replace(Arc::new(state.clone()));
        }
        let pending = Pending {
            owner: owner.clone(),
            finished: false,
        };
        let source = self.source.clone();
        let monitoring = self.monitoring.clone();
        let loopback = {
            #[cfg(test)]
            {
                owner.loopback_fixture
            }
            #[cfg(not(test))]
            {
                false
            }
        };
        self.executor
            .job(key, move |_, _| {
                let mut pending = pending;
                let result = tokio::runtime::Handle::try_current()
                    .map_err(|_| SyncError::Operation)
                    .and_then(|handle| handle.block_on(execute(&source, operation, loopback)));
                monitoring.terminal(
                    &source,
                    "sync",
                    if result.is_ok() {
                        "sync.operation.completed"
                    } else {
                        "sync.operation.failed"
                    },
                    result.is_err(),
                );
                owner.finish(result.clone());
                pending.finished = true;
                match result {
                    Ok(outcome) => Completion::Success(outcome),
                    Err(error) => Completion::Failed(error),
                }
            })
            .map(|_| ())
            .map_err(|_| SyncError::Interrupted)
    }
}

fn document(source: &ConfigSource) -> Result<Document, SyncError> {
    let path = canonical_file(source)?;
    let parent = path.parent().ok_or(SyncError::Storage)?;
    let locks = matches!(source, ConfigSource::Fixture { .. }).then(|| parent.join("locks"));
    Ok(Document::new(
        path.clone(),
        None,
        BackupPolicy::Dir {
            dir: parent.join("backups"),
            prefix: "sync_folders".into(),
        },
        locks,
        Format::Toml,
    ))
}

fn save(source: &ConfigSource, patch: SettingsPatch) -> Result<SyncOutcome, SyncError> {
    let document = document(source)?;
    let before = document.read().map_err(|_| SyncError::Unreadable)?;
    if before.token != patch.revision {
        return Err(SyncError::Conflict);
    }
    let mut value: toml::Value = if before.exists {
        toml::from_str(&before.content).map_err(|_| SyncError::Unreadable)?
    } else {
        // Do not invent a provider URL from the domain's historical default.
        toml::from_str("[webdav]\nenabled=false\nurl=''\nusername=''\npassword=''\nbase_remote_path='/ccr-sync'\nauto_sync=false\n")
            .map_err(|_| SyncError::Invalid)?
    };
    let webdav = value
        .get_mut("webdav")
        .and_then(toml::Value::as_table_mut)
        .ok_or(SyncError::Invalid)?;
    webdav.insert("enabled".into(), patch.enabled.into());
    for (key, value) in [
        ("url", patch.url),
        ("username", patch.username),
        ("base_remote_path", patch.base_remote_path),
    ] {
        if let Some(value) = value {
            webdav.insert(key.into(), value.into());
        }
    }
    match patch.password {
        PasswordUpdate::Keep => {}
        PasswordUpdate::Replace(password) => {
            if password.is_empty() {
                return Err(SyncError::Invalid);
            }
            webdav.insert("password".into(), password.expose().to_string().into());
        }
        PasswordUpdate::Clear => {
            webdav.insert("password".into(), "".into());
        }
    }
    let content = toml::to_string_pretty(&value).map_err(|_| SyncError::Invalid)?;
    let config: SyncFoldersConfig = toml::from_str(&content).map_err(|_| SyncError::Invalid)?;
    if !config.validate().is_empty() {
        return Err(SyncError::Invalid);
    }
    if !config.webdav.url.is_empty() {
        ccr_sync::validate_webdav_url(&config.webdav.url, false).map_err(|_| SyncError::Invalid)?;
    }
    if config.webdav.enabled
        && (config.webdav.url.is_empty()
            || config.webdav.username.is_empty()
            || config.webdav.password.is_empty())
    {
        return Err(SyncError::Invalid);
    }
    match document
        .save_validated(&content, &before)
        .map_err(|_| SyncError::Storage)?
    {
        SaveOutcome::Saved { .. } => Ok(SyncOutcome::Saved),
        SaveOutcome::Conflict => Err(SyncError::Conflict),
    }
}

fn save_folder(
    source: &ConfigSource,
    revision: &str,
    change: FolderChange,
) -> Result<SyncOutcome, SyncError> {
    let document = document(source)?;
    let before = document.read().map_err(|_| SyncError::Unreadable)?;
    if before.token != revision {
        return Err(SyncError::Conflict);
    }
    if !before.exists {
        return Err(SyncError::Invalid);
    }
    let mut value: toml::Value =
        toml::from_str(&before.content).map_err(|_| SyncError::Unreadable)?;
    let table = value.as_table_mut().ok_or(SyncError::Invalid)?;
    if table.contains_key("folder") {
        return Err(SyncError::LegacyFolders);
    }
    let folders = table
        .entry("folders")
        .or_insert_with(|| toml::Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or(SyncError::Invalid)?;
    let identity = match &change {
        FolderChange::Add(_) => None,
        FolderChange::Edit { identity, .. }
        | FolderChange::Delete { identity }
        | FolderChange::Enabled { identity, .. } => Some(identity),
    };
    let index = identity
        .map(|identity| {
            let matches = folders
                .iter()
                .enumerate()
                .filter(|(_, folder)| {
                    folder
                        .get("name")
                        .and_then(toml::Value::as_str)
                        .is_some_and(|name| {
                            ccr_core::core::content_version_token(name.as_bytes()) == *identity
                        })
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            if matches.len() == 1 {
                Ok(matches[0])
            } else {
                Err(SyncError::Conflict)
            }
        })
        .transpose()?;
    let changed_index = match change {
        FolderChange::Add(patch) => {
            let folder = ccr_sync::SyncFolder {
                name: patch.name.ok_or(SyncError::Invalid)?,
                description: String::new(),
                local_path: patch.local_path.ok_or(SyncError::Invalid)?,
                remote_path: patch.remote_path.ok_or(SyncError::Invalid)?,
                enabled: true,
                auto_sync: false,
                exclude_patterns: Vec::new(),
            };
            folders.push(toml::Value::try_from(folder).map_err(|_| SyncError::Invalid)?);
            Some(folders.len() - 1)
        }
        FolderChange::Edit { patch, .. } => {
            let index = index.ok_or(SyncError::Conflict)?;
            let folder = folders[index].as_table_mut().ok_or(SyncError::Invalid)?;
            for (key, value) in [
                ("name", patch.name),
                ("local_path", patch.local_path),
                ("remote_path", patch.remote_path),
            ] {
                if let Some(value) = value {
                    folder.insert(key.into(), value.into());
                }
            }
            Some(index)
        }
        FolderChange::Delete { .. } => {
            folders.remove(index.ok_or(SyncError::Conflict)?);
            None
        }
        FolderChange::Enabled { enabled, .. } => {
            let index = index.ok_or(SyncError::Conflict)?;
            folders[index]
                .as_table_mut()
                .ok_or(SyncError::Invalid)?
                .insert("enabled".into(), enabled.into());
            Some(index)
        }
    };
    // Validate only the selected folder's fixture path before the domain's home
    // expansion. No fixture edit can create a later transfer outside its root.
    if let Some(index) = changed_index {
        let folder: ccr_sync::SyncFolder = folders[index]
            .clone()
            .try_into()
            .map_err(|_| SyncError::Invalid)?;
        if matches!(source, ConfigSource::Fixture { .. }) {
            local_folder(source, &folder)?;
        }
        if !folder.validate().is_empty() {
            return Err(SyncError::Invalid);
        }
    }
    let content = toml::to_string_pretty(&value).map_err(|_| SyncError::Invalid)?;
    let config: SyncFoldersConfig = toml::from_str(&content).map_err(|_| SyncError::Invalid)?;
    if !config.validate().is_empty() {
        return Err(SyncError::Invalid);
    }
    match document
        .save_validated(&content, &before)
        .map_err(|_| SyncError::Storage)?
    {
        SaveOutcome::Saved { .. } => Ok(SyncOutcome::FolderSaved),
        SaveOutcome::Conflict => Err(SyncError::Conflict),
    }
}

async fn execute(
    source: &ConfigSource,
    operation: Operation,
    loopback: bool,
) -> Result<SyncOutcome, SyncError> {
    let operation = match operation {
        Operation::PrepareClear { revision } => return management::prepare(source, &revision),
        Operation::Clear { revision, session } => {
            return management::clear(source, &revision, &session);
        }
        Operation::Batch { revision, upload } => {
            return management::batch(source, &revision, upload, loopback).await;
        }
        Operation::Assets(request) => return assets::transfer(source, request, loopback).await,
        Operation::AssetInventory { revision, probe } => {
            return assets::inventory(source, &revision, probe, loopback).await;
        }
        Operation::Save(patch) => return save(source, patch),
        Operation::Folder { revision, change } => return save_folder(source, &revision, change),
        operation => operation,
    };
    if matches!(source, ConfigSource::Fixture { .. }) && !loopback {
        return Err(SyncError::FixtureNetwork);
    }
    let (revision, folder_index) = match &operation {
        Operation::Test { revision } => (revision, None),
        Operation::Push { revision, folder } | Operation::Pull { revision, folder } => {
            (revision, Some(*folder))
        }
        Operation::PrepareClear { .. }
        | Operation::Clear { .. }
        | Operation::Batch { .. }
        | Operation::Save(_)
        | Operation::Folder { .. }
        | Operation::Assets(_)
        | Operation::AssetInventory { .. } => return Err(SyncError::Invalid),
    };
    let snapshot = document(source)?
        .read()
        .map_err(|_| SyncError::Unreadable)?;
    if &snapshot.token != revision {
        return Err(SyncError::Conflict);
    }
    let config: SyncFoldersConfig =
        toml::from_str(&snapshot.content).map_err(|_| SyncError::Unreadable)?;
    let webdav = config.webdav;
    if !webdav.enabled
        || webdav.url.is_empty()
        || webdav.username.is_empty()
        || webdav.password.is_empty()
    {
        return Err(SyncError::Invalid);
    }
    let folder = folder_index
        .map(|index| config.folders.get(index).ok_or(SyncError::Invalid))
        .transpose()?;
    if folder.is_some_and(|folder| !folder.enabled) {
        return Err(SyncError::Invalid);
    }
    let local = folder
        .map(|folder| local_folder(source, folder))
        .transpose()?;
    let mut remote_path = folder
        .map(|folder| folder.remote_path.clone())
        .unwrap_or(webdav.base_remote_path);
    if local.as_ref().is_some_and(|path| path.is_dir()) && !remote_path.ends_with('/') {
        remote_path.push('/');
    }
    let config = SyncConfig {
        enabled: webdav.enabled,
        webdav_url: webdav.url,
        username: webdav.username,
        password: webdav.password,
        remote_path,
        auto_sync: false,
    };
    // The fixture flag cannot enable external HTTP: the domain validates loopback.
    #[cfg(test)]
    let service = if loopback {
        SyncService::new_for_test_loopback(&config).await
    } else {
        SyncService::new(&config).await
    };
    #[cfg(not(test))]
    let service = SyncService::new(&config).await;
    let service = service.map_err(|_| SyncError::Invalid)?;
    if matches!(operation, Operation::Test { .. }) {
        service
            .test_connection()
            .await
            .map_err(|_| SyncError::Operation)?;
        return Ok(SyncOutcome::Connected);
    }
    let local = local.ok_or(SyncError::Invalid)?;
    match operation {
        Operation::Push { .. } => {
            service
                .push(&local, None)
                .await
                .map_err(|_| SyncError::Operation)?;
            Ok(SyncOutcome::Pushed)
        }
        Operation::Pull { .. } => {
            let backup = service
                .pull_with_backup(&local)
                .await
                .map_err(|_| SyncError::Operation)?;
            Ok(SyncOutcome::Pulled {
                backup: backup.map(|path| redact_log_text(&path.to_string_lossy())),
            })
        }
        _ => Err(SyncError::Invalid),
    }
}

fn local_folder(
    source: &ConfigSource,
    folder: &ccr_sync::SyncFolder,
) -> Result<PathBuf, SyncError> {
    if matches!(source, ConfigSource::Fixture { .. }) {
        // Explicit fixture paths never expand ~ into the user's real home.
        let local = PathBuf::from(&folder.local_path);
        let root = canonical_file(source)?
            .parent()
            .ok_or(SyncError::Storage)?
            .canonicalize()
            .map_err(|_| SyncError::Storage)?;
        let parent = local
            .parent()
            .ok_or(SyncError::Invalid)?
            .canonicalize()
            .map_err(|_| SyncError::Invalid)?;
        if !local.is_absolute()
            || !parent.starts_with(&root)
            || (local.exists()
                && !local
                    .canonicalize()
                    .map_err(|_| SyncError::Invalid)?
                    .starts_with(&root))
        {
            return Err(SyncError::Invalid);
        }
        Ok(local)
    } else {
        folder.expand_local_path().map_err(|_| SyncError::Invalid)
    }
}

#[cfg(test)]
mod tests;
