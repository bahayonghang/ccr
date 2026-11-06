//! App-owned Check-in mutations; all storage, encryption and HTTP are domain-owned.
use super::*;
mod credentials;
mod crypto_guard;
pub use credentials::AccountCredentials;
#[path = "extended.rs"]
mod extended;
use ccr_app::runtime::Completion;
use ccr_checkin::managers::checkin::builtin_providers::get_builtin_provider_by_id;
use ccr_checkin::{AccountManager, CheckinService, ProviderManager, RecordManager};
use ccr_db::{
    database::{self, DbAccess},
    models::checkin::*,
};
pub use extended::{
    ExtendedCommand, ExtendedReport, InspectionCommand, InspectionReport, RecordQuery,
};
use std::sync::{Arc, Mutex};
use tokio::sync::watch;

pub enum Operation {
    Manage,
    Extended(ExtendedCommand),
    AddBuiltin(String),
    AddProvider(CreateProviderRequest),
    UpdateProvider(String, UpdateProviderRequest),
    DeleteProvider(String),
    AddAccount(CreateAccountRequest),
    UpdateAccount(String, UpdateAccountRequest),
    DeleteAccount(String),
    Checkin(String),
    Balance(String),
    Records(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationError {
    Storage,
    Invalid,
    Busy,
    Unsupported,
    FixtureNetwork,
    FixturePath,
    Interrupted,
    ProviderInUse,
    Network,
    Challenge,
    Credentials,
    CryptoKeyUnavailable,
    ImportCredentials,
    ImportConflict,
    ImportInvalid,
    QueryInvalid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Loaded,
    Extended,
    Saved,
    Deleted,
    Checked(CheckinStatus),
    BalanceUpdated,
    Records,
}

#[derive(Clone)]
pub struct AccountSummary {
    pub id: String,
    pub provider_id: String,
    pub name: String,
    pub enabled: bool,
}
#[derive(Clone)]
pub struct RecordSummary {
    pub account_id: String,
    pub status: CheckinStatus,
    pub at: String,
}
#[derive(Clone, Default)]
pub struct ManagementSnapshot {
    pub providers: Vec<CheckinProvider>,
    pub accounts: Vec<AccountSummary>,
    pub records: Vec<RecordSummary>,
}
#[derive(Clone, Default)]
pub struct OperationState {
    pub busy: bool,
    pub environment: Option<EnvironmentId>,
    pub result: Option<Result<Outcome, OperationError>>,
    pub snapshot: Option<ManagementSnapshot>,
    pub extended: Option<Arc<ExtendedReport>>,
}
pub struct CheckinOperations {
    state: Mutex<OperationState>,
    updates: watch::Sender<Arc<OperationState>>,
    storage: Mutex<Option<Storage>>,
    #[cfg(test)]
    loopback_fixture: bool,
}
impl Default for CheckinOperations {
    fn default() -> Self {
        Self {
            state: Mutex::new(OperationState::default()),
            updates: watch::channel(Arc::new(OperationState::default())).0,
            storage: Mutex::new(None),
            #[cfg(test)]
            loopback_fixture: false,
        }
    }
}
#[derive(Clone)]
struct Storage {
    db: DbAccess,
    crypto: PathBuf,
    locks: Option<PathBuf>,
}
impl Storage {
    // Only synchronous domain calls may enter the thread-local writer scope.
    fn scoped<T>(
        &self,
        work: impl FnOnce() -> Result<T, OperationError>,
    ) -> Result<T, OperationError> {
        match &self.locks {
            Some(locks) => ccr_core::core::LockManager::with_scoped_directory(locks, || Ok(work()))
                .map_err(|_| OperationError::Storage)?,
            None => work(),
        }
    }
    fn accounts(&self) -> AccountManager {
        AccountManager::with_db(&self.crypto, self.db.clone())
    }
    fn providers(&self) -> ProviderManager {
        ProviderManager::with_db(self.db.clone())
    }
    fn snapshot(&self, records: Vec<RecordSummary>) -> Result<ManagementSnapshot, OperationError> {
        Ok(ManagementSnapshot {
            providers: self
                .providers()
                .list()
                .map_err(|_| OperationError::Storage)?
                .providers,
            accounts: self
                .accounts()
                .list()
                .map_err(|_| OperationError::Storage)?
                .accounts
                .into_iter()
                .map(|a| AccountSummary {
                    id: a.id,
                    provider_id: a.provider_id,
                    name: redact_log_text(&a.name),
                    enabled: a.enabled,
                })
                .collect(),
            records,
        })
    }
}
fn reject_links(path: &Path) -> Result<(), OperationError> {
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            return Err(OperationError::Storage);
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if meta.file_attributes() & 0x400 != 0 {
                return Err(OperationError::Storage);
            }
        }
        if meta.is_dir() {
            for child in std::fs::read_dir(path).map_err(|_| OperationError::Storage)? {
                reject_links(&child.map_err(|_| OperationError::Storage)?.path())?;
            }
        }
    }
    Ok(())
}
impl CheckinOperations {
    fn storage(&self, source: &ConfigSource) -> Result<Storage, OperationError> {
        let mut cache = self.storage.lock().map_err(|_| OperationError::Storage)?;
        if let Some(storage) = cache.as_ref() {
            return Ok(storage.clone());
        }
        let storage = match source {
            ConfigSource::Benchmark(layout) => Storage {
                db: DbAccess::Pool(layout.main_pool().map_err(|_| OperationError::Storage)?),
                crypto: layout.path("crypto/checkin"),
                locks: Some(layout.path("locks")),
            },
            ConfigSource::Local => Storage {
                db: DbAccess::Pool(match database::get_pool() {
                    Some(pool) => pool.clone(),
                    None => database::initialize_app_pool().map_err(|_| OperationError::Storage)?,
                }),
                crypto: CheckinService::default_checkin_dir()
                    .map_err(|_| OperationError::Storage)?,
                locks: None,
            },
            ConfigSource::Fixture { registry, .. } => {
                if !registry.is_absolute() {
                    return Err(OperationError::Storage);
                }
                let root = registry.parent().ok_or(OperationError::Storage)?;
                let canonical = root.canonicalize().map_err(|_| OperationError::Storage)?;
                let database_path = canonical.join("checkin.db");
                let crypto = canonical.join("checkin-crypto");
                let home = canonical.join("checkin-migration-home");
                let locks = canonical.join("checkin-locks");
                for path in [&database_path, &crypto, &home, &locks] {
                    reject_links(path)?;
                }
                std::fs::create_dir_all(&home).map_err(|_| OperationError::Storage)?;
                if std::fs::read_dir(&home)
                    .map_err(|_| OperationError::Storage)?
                    .next()
                    .is_some()
                {
                    return Err(OperationError::Storage);
                }
                let pool = database::create_pool(&database_path, None)
                    .map_err(|_| OperationError::Storage)?;
                {
                    let conn = pool.get().map_err(|_| OperationError::Storage)?;
                    database::migrations::run_all_migrations(&conn, &home)
                        .map_err(|_| OperationError::Storage)?;
                }
                Storage {
                    db: DbAccess::Pool(pool),
                    crypto,
                    locks: Some(locks),
                }
            }
        };
        *cache = Some(storage.clone());
        Ok(storage)
    }
    fn finish(
        &self,
        result: Result<Outcome, OperationError>,
        snapshot: Option<ManagementSnapshot>,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.busy = false;
        state.result = Some(result);
        if snapshot.is_some() {
            state.snapshot = snapshot;
        }
        self.updates.send_replace(Arc::new(state.clone()));
    }
}
struct Pending {
    owner: Arc<CheckinOperations>,
    finished: bool,
}
impl Drop for Pending {
    fn drop(&mut self) {
        if !self.finished {
            self.owner.finish(Err(OperationError::Interrupted), None);
        }
    }
}
impl Services {
    pub fn checkin_operation_state(&self) -> Arc<OperationState> {
        self.checkin_operations.updates.borrow().clone()
    }
    pub fn observe_checkin(&self) -> watch::Receiver<Arc<OperationState>> {
        self.checkin_operations.updates.subscribe()
    }
    pub fn discard_checkin_preview(&self) {
        let owner = &self.checkin_operations;
        let mut state = owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !state.busy && matches!(state.extended.as_deref(), Some(ExtendedReport::Preview(_))) {
            state.extended = None;
            owner.updates.send_replace(Arc::new(state.clone()));
        }
    }
    pub fn start_checkin(
        &self,
        key: RequestKey,
        operation: Operation,
    ) -> Result<(), OperationError> {
        if !matches!(key.environment, EnvironmentId::Local(_)) {
            return Err(OperationError::Unsupported);
        }
        let owner = self.checkin_operations.clone();
        {
            let mut state = owner.state.lock().map_err(|_| OperationError::Storage)?;
            if state.busy {
                return Err(OperationError::Busy);
            }
            state.busy = true;
            state.result = None;
            state.extended = None;
            state.environment = Some(key.environment.clone());
            owner.updates.send_replace(Arc::new(state.clone()));
        }
        let pending = Pending {
            owner: owner.clone(),
            finished: false,
        };
        let source = self.source.clone();
        let monitoring = self.monitoring.clone();
        self.executor
            .job(key, move |_, _| {
                let mut pending = pending;
                let storage = owner.storage(&source);
                let result = storage.as_ref().map_err(|e| *e).and_then(|storage| {
                    tokio::runtime::Handle::try_current()
                        .map_err(|_| OperationError::Interrupted)
                        .and_then(|handle| {
                            handle.block_on(execute(storage, &source, &owner, operation))
                        })
                });
                let records = result
                    .as_ref()
                    .ok()
                    .map(|(_, records)| records.clone())
                    .unwrap_or_default();
                let snapshot = storage.ok().and_then(|s| s.snapshot(records).ok());
                let outcome = result.map(|(outcome, _)| outcome);
                monitoring.terminal(
                    &source,
                    "checkin",
                    if outcome.is_ok() {
                        "checkin.operation.completed"
                    } else {
                        "checkin.operation.failed"
                    },
                    outcome.is_err(),
                );
                owner.finish(outcome, snapshot);
                pending.finished = true;
                match outcome {
                    Ok(value) => Completion::Success(value),
                    Err(error) => Completion::Failed(error),
                }
            })
            .map(|_| ())
            .map_err(|_| OperationError::Interrupted)
    }
}
fn valid_provider(name: &str, url: &str) -> Result<(), OperationError> {
    let url = reqwest::Url::parse(url).map_err(|_| OperationError::Invalid)?;
    if name.trim().is_empty()
        || !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(OperationError::Invalid);
    }
    Ok(())
}
fn valid_cookies(secret: &ccr_core::Secret) -> Result<(), OperationError> {
    CookieCredentials::from_json(secret.expose(), String::new())
        .map_err(|_| OperationError::Invalid)?;
    Ok(())
}
fn service_error(error: ccr_checkin::CheckinServiceError) -> OperationError {
    match error.error_code() {
        "waf_blocked" | "cf_blocked" => OperationError::Challenge,
        "cookie_expired" => OperationError::Credentials,
        _ => OperationError::Network,
    }
}
fn service_for_account(
    storage: &Storage,
    source: &ConfigSource,
    owner: &CheckinOperations,
    id: &str,
) -> Result<CheckinService, OperationError> {
    if source.is_benchmark() {
        return Err(OperationError::FixtureNetwork);
    }
    let account = storage
        .accounts()
        .get(id)
        .map_err(|_| OperationError::Invalid)?;
    let provider = storage
        .providers()
        .get(&account.provider_id)
        .map_err(|_| OperationError::Invalid)?;
    storage.require_existing_crypto()?;
    Ok(if source.is_fixture() {
        let allowed = {
            #[cfg(test)]
            {
                owner.loopback_fixture
            }
            #[cfg(not(test))]
            {
                let _ = owner;
                false
            }
        };
        let url = reqwest::Url::parse(&provider.base_url).map_err(|_| OperationError::Invalid)?;
        let loopback = url
            .host_str()
            .and_then(|host| host.parse::<std::net::IpAddr>().ok())
            .is_some_and(|ip| ip.is_loopback());
        if !allowed || !loopback || provider.builtin_id.is_some() {
            return Err(OperationError::FixtureNetwork);
        }
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|_| OperationError::Network)?;
        CheckinService::with_db_and_client(storage.crypto.clone(), storage.db.clone(), client)
    } else {
        CheckinService::with_db(storage.crypto.clone(), storage.db.clone())
    })
}

async fn execute(
    storage: &Storage,
    source: &ConfigSource,
    owner: &CheckinOperations,
    operation: Operation,
) -> Result<(Outcome, Vec<RecordSummary>), OperationError> {
    let providers = storage.providers();
    let accounts = storage.accounts();
    let mut records = Vec::new();
    let outcome = match operation {
        Operation::Manage => Outcome::Loaded,
        Operation::Extended(command) => {
            let result = extended::execute_extended(storage, source, owner, command).await?;
            owner
                .state
                .lock()
                .map_err(|_| OperationError::Storage)?
                .extended = Some(Arc::new(result));
            Outcome::Extended
        }
        Operation::AddBuiltin(id) => {
            let provider = get_builtin_provider_by_id(&id)
                .ok_or(OperationError::Invalid)?
                .to_checkin_provider();
            providers
                .import_batch(vec![provider], false)
                .map_err(|_| OperationError::Storage)?;
            Outcome::Saved
        }
        Operation::AddProvider(request) => {
            valid_provider(&request.name, &request.base_url)?;
            providers
                .create(request)
                .map_err(|_| OperationError::Invalid)?;
            Outcome::Saved
        }
        Operation::UpdateProvider(id, request) => {
            let old = providers.get(&id).map_err(|_| OperationError::Invalid)?;
            valid_provider(
                request.name.as_deref().unwrap_or(&old.name),
                request.base_url.as_deref().unwrap_or(&old.base_url),
            )?;
            providers
                .update(&id, request)
                .map_err(|_| OperationError::Invalid)?;
            Outcome::Saved
        }
        Operation::DeleteProvider(id) => {
            let in_use = accounts
                .list()
                .map_err(|_| OperationError::Storage)?
                .accounts
                .iter()
                .any(|a| a.provider_id == id);
            if in_use {
                return Err(OperationError::ProviderInUse);
            }
            providers
                .delete(&id, false)
                .map_err(|_| OperationError::Storage)?;
            Outcome::Deleted
        }
        Operation::AddAccount(request) => {
            if request.name.trim().is_empty() {
                return Err(OperationError::Invalid);
            }
            providers
                .get(&request.provider_id)
                .map_err(|_| OperationError::Invalid)?;
            valid_cookies(&request.cookies_json)?;
            storage.scoped(|| {
                storage.guard_crypto_initialization()?;
                accounts
                    .create(request)
                    .map_err(|_| OperationError::Storage)
            })?;
            Outcome::Saved
        }
        Operation::UpdateAccount(id, request) => {
            if request.name.as_deref().is_some_and(|s| s.trim().is_empty()) {
                return Err(OperationError::Invalid);
            }
            if let Some(secret) = &request.cookies_json {
                valid_cookies(secret)?;
            }
            storage.scoped(|| {
                if request.cookies_json.is_some() {
                    storage.require_existing_crypto()?;
                }
                accounts
                    .update(&id, request)
                    .map_err(|_| OperationError::Storage)
            })?;
            Outcome::Saved
        }
        Operation::DeleteAccount(id) => {
            accounts.delete(&id).map_err(|_| OperationError::Storage)?;
            Outcome::Deleted
        }
        Operation::Records(id) => {
            records = RecordManager::with_db(storage.db.clone())
                .get_by_account(&id, Some(50))
                .map_err(|_| OperationError::Storage)?
                .records
                .into_iter()
                .map(|r| RecordSummary {
                    account_id: r.account_id,
                    status: r.status,
                    at: r.checked_in_at.to_rfc3339(),
                })
                .collect();
            Outcome::Records
        }
        operation @ (Operation::Checkin(_) | Operation::Balance(_)) => {
            let (id, checkin) = match operation {
                Operation::Checkin(id) => (id, true),
                Operation::Balance(id) => (id, false),
                _ => return Err(OperationError::Invalid),
            };
            let service = service_for_account(storage, source, owner, &id)?;
            if checkin {
                let result = service.checkin(&id).await.map_err(service_error)?;
                if result.status == CheckinStatus::Failed {
                    match result.error_code.as_deref() {
                        Some("waf_blocked" | "cf_blocked") => {
                            return Err(OperationError::Challenge);
                        }
                        Some("cookie_expired") => return Err(OperationError::Credentials),
                        _ => {}
                    }
                }
                Outcome::Checked(result.status)
            } else {
                service.query_balance(&id).await.map_err(service_error)?;
                Outcome::BalanceUpdated
            }
        }
    };
    Ok((outcome, records))
}

#[cfg(test)]
mod tests;
