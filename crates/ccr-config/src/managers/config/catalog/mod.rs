//! Per-platform profile catalog stored in `profiles.db`.
//!
//! One thread owns at most one [`CatalogTransaction`]. Inner load and mutate calls
//! join that transaction instead of opening another connection.

mod migrate;

use super::repository::{config_resource_name, validate_profile_name, version_conflict};
use super::{CcsConfig, ConfigSection, GlobalSettings, ProviderType};
use crate::platforms::base::parse_config_from_str;
use ccr_core::core::guarded_write::{
    content_version_token, normalized_resource_path, restrict_owner_only,
};
use ccr_core::core::{FileLock, LockManager};
use ccr_core::{CcrError, Result, Secret};
use indexmap::IndexMap;
use rusqlite::backup::{Backup, StepResult};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use std::cell::{Cell, RefCell};
use std::fs::{self, OpenOptions};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub use migrate::migrate_catalogs;
#[cfg(test)]
pub(crate) use migrate::{MigrationFailpoint, migrate_paths};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS document (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    current_config TEXT,
    default_config TEXT,
    settings_toml TEXT,
    extra_toml TEXT NOT NULL DEFAULT '',
    revision TEXT NOT NULL,
    migration_phase TEXT NOT NULL CHECK (migration_phase IN ('data_authoritative', 'source_retired')),
    migrated_at TEXT
);
CREATE TABLE IF NOT EXISTS profiles (
    name TEXT PRIMARY KEY,
    description TEXT,
    base_url TEXT,
    auth_token TEXT,
    model TEXT,
    small_fast_model TEXT,
    default_opus_model TEXT,
    default_sonnet_model TEXT,
    default_haiku_model TEXT,
    default_fable_model TEXT,
    default_opus_model_name TEXT,
    default_sonnet_model_name TEXT,
    default_haiku_model_name TEXT,
    default_fable_model_name TEXT,
    subagent_model TEXT,
    custom_model_option TEXT,
    custom_model_option_name TEXT,
    effort_level TEXT,
    claude_code_auto_compact_window TEXT,
    api_timeout_ms TEXT,
    claude_code_disable_nonessential_traffic TEXT,
    provider TEXT,
    provider_type TEXT,
    account TEXT,
    tags_json TEXT,
    usage_count INTEGER,
    enabled INTEGER,
    extra_toml TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS applied_operations (
    operation_id TEXT PRIMARY KEY,
    payload TEXT NOT NULL
);
";

const INSERT_PROFILE: &str = "
INSERT INTO profiles (
    name, description, base_url, auth_token, model, small_fast_model,
    default_opus_model, default_sonnet_model, default_haiku_model, default_fable_model,
    default_opus_model_name, default_sonnet_model_name, default_haiku_model_name, default_fable_model_name,
    subagent_model, custom_model_option, custom_model_option_name, effort_level,
    claude_code_auto_compact_window, api_timeout_ms, claude_code_disable_nonessential_traffic,
    provider, provider_type, account, tags_json, usage_count, enabled, extra_toml, sort_order, updated_at
) VALUES (
    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
    ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20,
    ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30
)
";

const PROFILE_SELECT: &str = "
SELECT name, description, base_url, auth_token, model, small_fast_model,
    default_opus_model, default_sonnet_model, default_haiku_model, default_fable_model,
    default_opus_model_name, default_sonnet_model_name, default_haiku_model_name, default_fable_model_name,
    subagent_model, custom_model_option, custom_model_option_name, effort_level,
    claude_code_auto_compact_window, api_timeout_ms, claude_code_disable_nonessential_traffic,
    provider, provider_type, account, tags_json, usage_count, enabled, extra_toml, sort_order
FROM profiles
ORDER BY sort_order ASC, name ASC
";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    DataAuthoritative,
    SourceRetired,
}

impl Phase {
    fn as_str(self) -> &'static str {
        match self {
            Self::DataAuthoritative => "data_authoritative",
            Self::SourceRetired => "source_retired",
        }
    }

    fn parse(value: &str) -> Result<Self> {
        match value {
            "data_authoritative" => Ok(Self::DataAuthoritative),
            "source_retired" => Ok(Self::SourceRetired),
            _ => Err(CcrError::ConfigError("配置目录迁移阶段无效".into())),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
enum Marker {
    Absent,
    Empty,
    Value(String),
}

impl Marker {
    fn from_sql(value: Option<String>) -> Self {
        match value {
            None => Self::Absent,
            Some(value) if value.is_empty() => Self::Empty,
            Some(value) => Self::Value(value),
        }
    }

    fn to_sql(&self) -> Option<String> {
        match self {
            Self::Absent => None,
            Self::Empty => Some(String::new()),
            Self::Value(value) => Some(value.clone()),
        }
    }
}

struct StoredProfile {
    name: String,
    description: Option<String>,
    base_url: Option<String>,
    auth_token: Option<String>,
    model: Option<String>,
    small_fast_model: Option<String>,
    default_opus_model: Option<String>,
    default_sonnet_model: Option<String>,
    default_haiku_model: Option<String>,
    default_fable_model: Option<String>,
    default_opus_model_name: Option<String>,
    default_sonnet_model_name: Option<String>,
    default_haiku_model_name: Option<String>,
    default_fable_model_name: Option<String>,
    subagent_model: Option<String>,
    custom_model_option: Option<String>,
    custom_model_option_name: Option<String>,
    effort_level: Option<String>,
    claude_code_auto_compact_window: Option<String>,
    api_timeout_ms: Option<String>,
    claude_code_disable_nonessential_traffic: Option<String>,
    provider: Option<String>,
    provider_type: Option<String>,
    account: Option<String>,
    tags_json: Option<String>,
    usage_count: Option<i64>,
    enabled: Option<i64>,
    extra_toml: String,
    sort_order: i64,
}

struct StoredCatalog {
    current: Marker,
    default_config: Marker,
    settings_toml: Option<String>,
    extra_toml: String,
    revision: String,
    phase: Phase,
    migrated_at: Option<String>,
    profiles: Vec<StoredProfile>,
}

pub(crate) enum DbInspect {
    Ready(Phase),
    NoPhase,
    Unreadable(CcrError),
}

enum Presence {
    Absent,
    NotMigrated,
    Ready,
}

struct Session {
    path: PathBuf,
    conn: RefCell<Connection>,
}

struct HeldLock {
    path: PathBuf,
    depth: u32,
}

thread_local! {
    static ACTIVE: RefCell<Option<Rc<Session>>> = const { RefCell::new(None) };
    static LOCKS: RefCell<Vec<HeldLock>> = const { RefCell::new(Vec::new()) };
    static EXCLUSIVE: Cell<u32> = const { Cell::new(0) };
}

/// Process-local catalog resource guard. Re-entering the same database on this
/// thread does not take a second file lock.
pub struct CatalogGuard {
    path: PathBuf,
    lock: Option<FileLock>,
    _not_send: PhantomData<*const ()>,
}

impl Drop for CatalogGuard {
    fn drop(&mut self) {
        let _lock = self.lock.take();
        LOCKS.with(|locks| {
            let mut locks = locks.borrow_mut();
            let Some(index) = locks.iter().position(|held| held.path == self.path) else {
                return;
            };
            if locks[index].depth > 1 {
                locks[index].depth -= 1;
            } else {
                locks.remove(index);
            }
        });
    }
}

struct ExclusiveGuard;

impl Drop for ExclusiveGuard {
    fn drop(&mut self) {
        EXCLUSIVE.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

/// One catalog transaction for the calling thread. It is not `Send`.
pub struct CatalogTransaction {
    session: Rc<Session>,
    joined: bool,
    finished: Cell<bool>,
    _not_send: PhantomData<*const ()>,
}

impl CatalogTransaction {
    /// Hold the catalog resource lock and one `BEGIN IMMEDIATE` for `body`.
    /// A nested call for the same database joins this transaction.
    pub fn execute<T>(
        db_path: &Path,
        body: impl FnOnce(&CatalogTransaction) -> Result<T>,
    ) -> Result<T> {
        if let Some(session) = active_session(db_path) {
            let joined = CatalogTransaction {
                session,
                joined: true,
                finished: Cell::new(true),
                _not_send: PhantomData,
            };
            return body(&joined);
        }

        let _guard = acquire_catalog_lock(db_path)?;
        let path = normalize(db_path)?;
        let connection = open_rw(&path)?;
        let session = Rc::new(Session {
            path,
            conn: RefCell::new(connection),
        });
        Self::execute_session(session, body)
    }

    /// Join an existing catalog or stage an absent catalog privately until the
    /// body and commit succeed. The empty expected token means absence only.
    /// Platform reads and writes inside the body join the same live-path session.
    pub fn execute_or_create<T>(
        legacy: &Path,
        db: &Path,
        expected: Option<&str>,
        body: impl FnOnce(&CatalogTransaction) -> Result<T>,
    ) -> Result<T> {
        let _guard = acquire_catalog_lock(db)?;
        if active_session(db).is_some() || matches!(presence(legacy, db)?, Presence::Ready) {
            return Self::execute(db, |transaction| {
                if let Some(expected) = expected
                    && expected != transaction.current_revision()?
                {
                    return Err(version_conflict());
                }
                body(transaction)
            });
        }
        if legacy.exists() {
            return Err(not_migrated());
        }
        if expected.is_some_and(|expected| !expected.is_empty()) {
            return Err(version_conflict());
        }
        if ACTIVE.with(|slot| slot.borrow().is_some()) {
            return Err(CcrError::ConfigError("目录事务不能跨目录嵌套".into()));
        }
        let staging = TempSnapshot::create()?;
        let connection = open_rw(&staging.path)?;
        write_all(&connection, &fresh_catalog()?)?;
        let session = Rc::new(Session {
            path: normalize(db)?,
            conn: RefCell::new(connection),
        });
        let result = Self::execute_session(session, body)?;
        if legacy.exists() {
            return Err(not_migrated());
        }
        // Publication uses create-new semantics. A foreign/preexisting target
        // is never replaced, and the temporary catalog is owned by its RAII guard.
        install_new_catalog(db, &staging.path)?;
        Ok(result)
    }

    fn execute_session<T>(
        session: Rc<Session>,
        body: impl FnOnce(&CatalogTransaction) -> Result<T>,
    ) -> Result<T> {
        session
            .conn
            .borrow()
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(sql_error)?;
        install_session(Rc::clone(&session));
        let transaction = CatalogTransaction {
            session: Rc::clone(&session),
            joined: false,
            finished: Cell::new(false),
            _not_send: PhantomData,
        };
        let result = body(&transaction);
        if result.is_ok() {
            #[cfg(any(test, feature = "test-support"))]
            if failpoints::take_commit() {
                rollback_session(&transaction.session);
                transaction.finished.set(true);
                clear_session(&transaction.session);
                return Err(CcrError::ConfigError("目录提交失败".into()));
            }
            let commit = transaction
                .session
                .conn
                .borrow()
                .execute_batch("COMMIT")
                .map_err(sql_error);
            transaction.finished.set(true);
            clear_session(&transaction.session);
            commit?;
            // COMMIT already published the rows. A later checkpoint error must not
            // look like an uncommitted transaction to the runtime journal.
            checkpoint_after_commit(&transaction.session.conn.borrow());
        } else {
            rollback_session(&transaction.session);
            transaction.finished.set(true);
            clear_session(&transaction.session);
        }
        result
    }

    pub fn current_revision(&self) -> Result<String> {
        self.session
            .conn
            .borrow()
            .query_row("SELECT revision FROM document WHERE id = 1", [], |row| {
                row.get(0)
            })
            .map_err(sql_error)
    }

    /// Record a committed operation in the open transaction. Replay reads it when
    /// the JSON record still cannot represent success.
    pub fn store_applied_operation(&self, operation_id: &str, payload: &str) -> Result<()> {
        self.session
            .conn
            .borrow()
            .execute(
                "INSERT INTO applied_operations (operation_id, payload) VALUES (?1, ?2)
                 ON CONFLICT(operation_id) DO UPDATE SET payload = excluded.payload",
                params![operation_id, payload],
            )
            .map_err(sql_error)?;
        Ok(())
    }
}

impl Drop for CatalogTransaction {
    fn drop(&mut self) {
        if self.joined || self.finished.get() {
            return;
        }
        rollback_session(&self.session);
        clear_session(&self.session);
        self.finished.set(true);
    }
}

#[cfg(test)]
pub(crate) struct DocumentProbe {
    pub current: Option<String>,
    pub default_config: Option<String>,
    pub settings_toml: Option<String>,
    pub phase: String,
    pub migrated_at: Option<String>,
    pub revision: String,
}

#[cfg(test)]
pub(crate) struct ProfileProbe {
    pub usage_count: Option<i64>,
    pub enabled: Option<i64>,
    pub extra_toml: String,
    pub sort_order: i64,
    pub auth_token: Option<String>,
}

pub(crate) fn locations(path: &Path) -> (PathBuf, PathBuf) {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return (path.to_path_buf(), path.with_extension("db"));
    };
    if name.eq_ignore_ascii_case("profiles.db") {
        (path.with_file_name("profiles.toml"), path.to_path_buf())
    } else if name.eq_ignore_ascii_case("profiles.toml") {
        (path.to_path_buf(), path.with_file_name("profiles.db"))
    } else {
        (path.to_path_buf(), path.with_extension("db"))
    }
}

pub(crate) fn acquire_catalog_lock(db: &Path) -> Result<CatalogGuard> {
    let path = normalize(db)?;
    let mut reentered = false;
    LOCKS.with(|locks| {
        let mut locks = locks.borrow_mut();
        if let Some(held) = locks.iter_mut().find(|held| held.path == path) {
            held.depth += 1;
            reentered = true;
        }
    });
    if reentered {
        return Ok(CatalogGuard {
            path,
            lock: None,
            _not_send: PhantomData,
        });
    }
    let lock = LockManager::with_default_path()?
        .lock_resource(&config_resource_name(&path)?, Duration::from_secs(10))?;
    LOCKS.with(|locks| {
        locks.borrow_mut().push(HeldLock {
            path: path.clone(),
            depth: 1,
        });
    });
    Ok(CatalogGuard {
        path,
        lock: Some(lock),
        _not_send: PhantomData,
    })
}

pub(crate) fn load_config(legacy: &Path, db: &Path) -> Result<CcsConfig> {
    load_with_revision(legacy, db).map(|(config, _revision)| config)
}

pub(crate) fn load_with_revision(legacy: &Path, db: &Path) -> Result<(CcsConfig, String)> {
    if let Some(session) = active_session(db) {
        let stored = read_stored(&session.conn.borrow())?;
        let revision = stored.revision.clone();
        return Ok((to_config(&stored)?, revision));
    }
    match presence(legacy, db)? {
        Presence::Absent => Err(CcrError::ConfigMissing(legacy.display().to_string())),
        Presence::NotMigrated => Err(not_migrated()),
        Presence::Ready => {
            let connection = open_live_ro(db)?;
            let stored = read_stored(&connection)?;
            let revision = stored.revision.clone();
            Ok((to_config(&stored)?, revision))
        }
    }
}

/// Canonical TOML and `document.revision`. Absent catalogs return `None` and
/// create nothing. A legacy TOML file is not read and is not imported.
pub(crate) fn canonical_document(legacy: &Path, db: &Path) -> Result<Option<(String, String)>> {
    if let Some(session) = active_session(db) {
        let stored = read_stored(&session.conn.borrow())?;
        let revision = stored.revision.clone();
        return Ok(Some((render_catalog(&stored)?, revision)));
    }
    match presence(legacy, db)? {
        Presence::Absent => Ok(None),
        Presence::NotMigrated => Err(not_migrated()),
        Presence::Ready => {
            let connection = open_live_ro(db)?;
            let stored = read_stored(&connection)?;
            let revision = stored.revision.clone();
            Ok(Some((render_catalog(&stored)?, revision)))
        }
    }
}

/// Replace the catalog document from canonical TOML. `expected` empty creates a
/// retired catalog only when both files are absent. A mismatch returns
/// [`version_conflict`] and writes nothing. An open [`CatalogTransaction`] on
/// this thread is joined and is not committed here.
pub(crate) fn replace_document(
    legacy: &Path,
    db: &Path,
    content: &str,
    expected: &str,
) -> Result<String> {
    let mut incoming = stored_from_toml(content)?;
    if let Some(session) = active_session(db) {
        return replace_joined(&session, &mut incoming, expected);
    }
    let _lock = acquire_catalog_lock(db)?;
    match presence(legacy, db)? {
        Presence::Absent => {
            if !expected.is_empty() {
                return Err(version_conflict());
            }
            replace_new(db, &mut incoming)
        }
        Presence::NotMigrated => Err(not_migrated()),
        Presence::Ready => replace_existing(db, &mut incoming, expected),
    }
}

fn replace_joined(
    session: &Session,
    incoming: &mut StoredCatalog,
    expected: &str,
) -> Result<String> {
    let current = read_stored(&session.conn.borrow())?;
    if current.revision != expected {
        return Err(version_conflict());
    }
    incoming.revision = new_revision();
    incoming.phase = current.phase;
    incoming.migrated_at = current.migrated_at;
    let revision = incoming.revision.clone();
    write_content(&session.conn.borrow(), incoming)?;
    Ok(revision)
}

fn replace_new(db: &Path, incoming: &mut StoredCatalog) -> Result<String> {
    incoming.revision = new_revision();
    incoming.phase = Phase::SourceRetired;
    incoming.migrated_at = Some(timestamp());
    let revision = incoming.revision.clone();
    let connection = create_database(db)?;
    let outcome = (|| {
        connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(sql_error)?;
        write_all(&connection, incoming)?;
        connection.execute_batch("COMMIT").map_err(sql_error)?;
        Ok(())
    })();
    if outcome.is_err() {
        drop(connection);
        remove_sqlite_files(db);
        return outcome.map(|()| revision);
    }
    checkpoint_after_commit(&connection);
    drop(connection);
    restrict_sidecars(db)?;
    Ok(revision)
}

fn replace_existing(db: &Path, incoming: &mut StoredCatalog, expected: &str) -> Result<String> {
    let connection = open_rw(db)?;
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(sql_error)?;
    let outcome = (|| {
        let current = read_stored(&connection)?;
        if current.revision != expected {
            return Err(version_conflict());
        }
        incoming.revision = new_revision();
        incoming.phase = current.phase;
        incoming.migrated_at = current.migrated_at;
        let revision = incoming.revision.clone();
        write_content(&connection, incoming)?;
        connection.execute_batch("COMMIT").map_err(sql_error)?;
        Ok(revision)
    })();
    if outcome.is_err() {
        let _ = connection.execute_batch("ROLLBACK");
    } else {
        checkpoint_after_commit(&connection);
        restrict_sidecars(db)?;
    }
    outcome
}

pub(crate) fn mutate_catalog<T>(
    legacy: &Path,
    db: &Path,
    expected: Option<&str>,
    create: bool,
    mutate: impl FnOnce(&mut CcsConfig) -> Result<T>,
) -> Result<T> {
    if active_session(db).is_some() {
        return mutate_joined(db, expected, mutate);
    }
    if EXCLUSIVE.with(|depth| depth.get()) > 0 {
        return Err(CcrError::ConfigError("目录修改不能嵌套".into()));
    }
    EXCLUSIVE.with(|depth| depth.set(depth.get() + 1));
    let _exclusive = ExclusiveGuard;
    let _lock = acquire_catalog_lock(db)?;
    mutate_exclusive(legacy, db, expected, create, mutate)
}

pub(crate) fn save_catalog(legacy: &Path, db: &Path, config: &CcsConfig) -> Result<()> {
    mutate_catalog(legacy, db, None, true, |current| {
        *current = config.clone();
        Ok(())
    })
}

#[cfg(test)]
pub(crate) fn document_probe(db: &Path) -> Result<DocumentProbe> {
    let connection = open_live_ro(db)?;
    let (current, default_config, settings_toml, _extra, revision, phase, migrated_at) =
        read_document_row(&connection)?;
    Ok(DocumentProbe {
        current,
        default_config,
        settings_toml,
        phase,
        migrated_at,
        revision,
    })
}

#[cfg(test)]
pub(crate) fn profile_probe(db: &Path, name: &str) -> Result<Option<ProfileProbe>> {
    let connection = open_live_ro(db)?;
    connection
        .query_row(
            "SELECT usage_count, enabled, extra_toml, sort_order, auth_token
             FROM profiles WHERE name = ?1",
            params![name],
            |row| {
                Ok(ProfileProbe {
                    usage_count: row.get(0)?,
                    enabled: row.get(1)?,
                    extra_toml: row.get(2)?,
                    sort_order: row.get(3)?,
                    auth_token: row.get(4)?,
                })
            },
        )
        .optional()
        .map_err(sql_error)
}

#[cfg(test)]
pub(crate) fn profile_names(db: &Path) -> Result<Vec<String>> {
    let connection = open_live_ro(db)?;
    let mut statement = connection
        .prepare("SELECT name FROM profiles ORDER BY sort_order ASC, name ASC")
        .map_err(sql_error)?;
    let names = statement
        .query_map([], |row| row.get(0))
        .map_err(sql_error)?
        .collect::<std::result::Result<Vec<String>, _>>()
        .map_err(sql_error)?;
    Ok(names)
}

#[cfg(test)]
pub(crate) fn schema_user_version(db: &Path) -> Result<i64> {
    let connection = open_live_ro(db)?;
    connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(sql_error)
}

pub(crate) fn inspect_db(db: &Path) -> Result<DbInspect> {
    if db.is_dir() {
        return Ok(DbInspect::Unreadable(CcrError::FileIoError(format!(
            "配置目录路径无效: {}",
            db.display()
        ))));
    }
    let connection = match Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(connection) => connection,
        Err(error) => {
            return Ok(DbInspect::Unreadable(CcrError::ConfigError(format!(
                "读取配置目录失败: {error}"
            ))));
        }
    };
    match read_phase(&connection) {
        Ok(Some(phase)) => Ok(DbInspect::Ready(phase)),
        Ok(None) => Ok(DbInspect::NoPhase),
        Err(error) => Ok(DbInspect::Unreadable(error)),
    }
}

pub(crate) fn attempt_file(db: &Path) -> PathBuf {
    with_suffix(db, ".attempt")
}

pub(crate) fn create_restricted_empty(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|error| {
            CcrError::FileIoError(format!("创建目录失败 {}: {error}", parent.display()))
        })?;
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(path).map_err(|error| {
        CcrError::FileIoError(format!("创建文件失败 {}: {error}", path.display()))
    })?;
    drop(file);
    restrict_owner_only(path)?;
    Ok(())
}

pub(crate) fn create_database(path: &Path) -> Result<Connection> {
    create_restricted_empty(path)?;
    let connection = Connection::open(path).map_err(sql_error)?;
    apply_pragmas(&connection)?;
    restrict_sidecars(path)?;
    ensure_schema(&connection)?;
    restrict_sidecars(path)?;
    Ok(connection)
}

pub(crate) fn open_rw(path: &Path) -> Result<Connection> {
    if !path.is_file() {
        return Err(CcrError::ConfigMissing(path.display().to_string()));
    }
    let connection = Connection::open(path).map_err(sql_error)?;
    apply_pragmas(&connection)?;
    ensure_schema(&connection)?;
    restrict_sidecars(path)?;
    Ok(connection)
}

pub fn load_applied_operation(db: &Path, operation_id: &str) -> Result<Option<String>> {
    if !db.is_file() {
        return Ok(None);
    }
    let connection = open_rw(db)?;
    connection
        .query_row(
            "SELECT payload FROM applied_operations WHERE operation_id = ?1",
            params![operation_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(sql_error)
}

/// Saturating activation counter. `NULL` becomes 1. `4294967295` stays there.
pub fn increment_usage_count(db: &Path, name: &str) -> Result<()> {
    #[cfg(any(test, feature = "test-support"))]
    if failpoints::take_usage() {
        return Err(CcrError::ConfigError("切换计数写入失败".into()));
    }
    let connection = open_rw(db)?;
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(sql_error)?;
    let outcome = (|| {
        let changed = connection
            .execute(
                "UPDATE profiles
                 SET usage_count = MIN(COALESCE(usage_count, 0) + 1, 4294967295),
                     updated_at = ?2
                 WHERE name = ?1",
                params![name, timestamp()],
            )
            .map_err(sql_error)?;
        if changed != 1 {
            return Err(CcrError::ProfileNotFound(name.to_owned()));
        }
        connection
            .execute(
                "UPDATE document SET revision = ?1 WHERE id = 1",
                params![new_revision()],
            )
            .map_err(sql_error)?;
        connection.execute_batch("COMMIT").map_err(sql_error)
    })();
    if outcome.is_err() {
        let _ = connection.execute_batch("ROLLBACK");
    }
    outcome?;
    checkpoint_after_commit(&connection);
    Ok(())
}

pub(crate) fn restrict_sidecars(db: &Path) -> Result<()> {
    restrict_owner_only(db)?;
    for suffix in ["-wal", "-shm"] {
        let path = with_suffix(db, suffix);
        if path.exists() {
            restrict_owner_only(&path)?;
        }
    }
    Ok(())
}

pub(crate) fn remove_sqlite_files(db: &Path) {
    for path in [
        db.to_path_buf(),
        with_suffix(db, "-wal"),
        with_suffix(db, "-shm"),
    ] {
        let _ = fs::remove_file(path);
    }
}

pub(crate) fn replace_with_import(connection: &Connection, content: &str) -> Result<()> {
    parse_config_from_str(content)?;
    let mut stored = stored_from_toml(content)?;
    stored.phase = Phase::DataAuthoritative;
    stored.migrated_at = None;
    stored.revision = new_revision();
    write_all(connection, &stored)
}

pub(crate) fn mark_source_retired(connection: &Connection) -> Result<()> {
    let changed = connection
        .execute(
            "UPDATE document
             SET migration_phase = 'source_retired', migrated_at = ?1
             WHERE id = 1 AND migration_phase = 'data_authoritative'",
            params![timestamp()],
        )
        .map_err(sql_error)?;
    if changed == 1 {
        checkpoint_after_commit(connection);
        return Ok(());
    }
    match read_phase(connection)? {
        Some(Phase::SourceRetired) => Ok(()),
        _ => Err(CcrError::ConfigError("配置目录无法标记为已停用".into())),
    }
}

#[cfg(test)]
pub(crate) fn bump_revision_for_test(db: &Path) -> Result<()> {
    let session =
        active_session(db).ok_or_else(|| CcrError::ConfigError("没有活动目录事务".into()))?;
    session
        .conn
        .borrow()
        .execute(
            "UPDATE document SET revision = 'external-revision' WHERE id = 1",
            [],
        )
        .map_err(sql_error)?;
    Ok(())
}

fn mutate_joined<T>(
    db: &Path,
    expected: Option<&str>,
    mutate: impl FnOnce(&mut CcsConfig) -> Result<T>,
) -> Result<T> {
    let session =
        active_session(db).ok_or_else(|| CcrError::ConfigError("没有活动目录事务".into()))?;
    let preview = read_stored(&session.conn.borrow())?;
    if expected.is_some_and(|token| token != preview.revision) {
        return Err(version_conflict());
    }
    let mut config = to_config(&preview)?;
    let result = mutate(&mut config)?;
    let current = read_stored(&session.conn.borrow())?;
    if current.revision != preview.revision {
        return Err(version_conflict());
    }
    let mut projected = project(&preview, &config)?;
    if content_equal(&current, &projected)? {
        return Ok(result);
    }
    projected.revision = new_revision();
    projected.phase = current.phase;
    projected.migrated_at = current.migrated_at;
    write_content(&session.conn.borrow(), &projected)?;
    Ok(result)
}

fn mutate_exclusive<T>(
    legacy: &Path,
    db: &Path,
    expected: Option<&str>,
    create: bool,
    mutate: impl FnOnce(&mut CcsConfig) -> Result<T>,
) -> Result<T> {
    if !db.exists() {
        return create_and_mutate(legacy, db, expected, create, mutate);
    }
    if !db.is_file() {
        return Err(CcrError::FileIoError(format!(
            "配置路径无效: {}",
            db.display()
        )));
    }
    match inspect_db(db)? {
        DbInspect::Ready(_) => {}
        DbInspect::NoPhase => {
            return Err(CcrError::ConfigError("配置目录尚未完成迁移".into()));
        }
        DbInspect::Unreadable(error) => return Err(error),
    }
    let preview = {
        let connection = open_live_ro(db)?;
        read_stored(&connection)?
    };
    if expected.is_some_and(|token| token != preview.revision) {
        return Err(version_conflict());
    }
    let mut config = to_config(&preview)?;
    let result = mutate(&mut config)?;
    let projected = project(&preview, &config)?;
    if content_equal(&preview, &projected)? {
        restrict_sidecars(db)?;
        return Ok(result);
    }
    let connection = open_rw(db)?;
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(sql_error)?;
    let written = (|| {
        let current = read_stored(&connection)?;
        if current.revision != preview.revision {
            return Err(version_conflict());
        }
        let mut projected = projected;
        projected.revision = new_revision();
        projected.phase = current.phase;
        projected.migrated_at = current.migrated_at;
        write_content(&connection, &projected)?;
        connection.execute_batch("COMMIT").map_err(sql_error)?;
        Ok(())
    })();
    if written.is_err() {
        let _ = connection.execute_batch("ROLLBACK");
    } else {
        checkpoint_after_commit(&connection);
    }
    written?;
    restrict_sidecars(db)?;
    Ok(result)
}

fn create_and_mutate<T>(
    legacy: &Path,
    db: &Path,
    expected: Option<&str>,
    create: bool,
    mutate: impl FnOnce(&mut CcsConfig) -> Result<T>,
) -> Result<T> {
    if legacy.is_file() {
        return Err(not_migrated());
    }
    if legacy.exists() || db.exists() {
        return Err(CcrError::FileIoError(format!(
            "配置路径无效: {}",
            legacy.display()
        )));
    }
    if !create {
        return Err(CcrError::ConfigMissing(legacy.display().to_string()));
    }
    if expected.is_some_and(|token| !token.is_empty()) {
        return Err(version_conflict());
    }
    let connection = create_database(db)?;
    let outcome = (|| {
        let stored = fresh_catalog()?;
        let mut config = to_config(&stored)?;
        let result = mutate(&mut config)?;
        let mut projected = project(&stored, &config)?;
        projected.revision = new_revision();
        projected.phase = Phase::SourceRetired;
        projected.migrated_at = Some(timestamp());
        connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(sql_error)?;
        write_all(&connection, &projected)?;
        connection.execute_batch("COMMIT").map_err(sql_error)?;
        Ok(result)
    })();
    if outcome.is_err() {
        drop(connection);
        remove_sqlite_files(db);
        return outcome;
    }
    checkpoint_after_commit(&connection);
    drop(connection);
    restrict_sidecars(db)?;
    outcome
}

fn presence(legacy: &Path, db: &Path) -> Result<Presence> {
    if legacy.is_dir() || db.is_dir() {
        return Err(CcrError::FileIoError(format!(
            "配置路径无效: {}",
            legacy.display()
        )));
    }
    if !db.exists() {
        if legacy.is_file() {
            return Ok(Presence::NotMigrated);
        }
        if legacy.exists() {
            return Err(CcrError::FileIoError(format!(
                "配置路径无效: {}",
                legacy.display()
            )));
        }
        return Ok(Presence::Absent);
    }
    if !db.is_file() {
        return Err(CcrError::FileIoError(format!(
            "配置路径无效: {}",
            db.display()
        )));
    }
    match inspect_db(db)? {
        DbInspect::Ready(_) => Ok(Presence::Ready),
        DbInspect::NoPhase => Err(CcrError::ConfigError("配置目录尚未完成迁移".into())),
        DbInspect::Unreadable(error) => Err(error),
    }
}

fn not_migrated() -> CcrError {
    CcrError::ConfigError("配置尚未迁移".into())
}

fn fresh_catalog() -> Result<StoredCatalog> {
    let config = parse_config_from_str("")?;
    Ok(StoredCatalog {
        current: Marker::Value(config.current_config),
        default_config: Marker::Value(config.default_config),
        settings_toml: Some(serialize_settings(&config.settings)?),
        extra_toml: String::new(),
        revision: new_revision(),
        phase: Phase::SourceRetired,
        migrated_at: Some(timestamp()),
        profiles: Vec::new(),
    })
}

fn project(original: &StoredCatalog, config: &CcsConfig) -> Result<StoredCatalog> {
    let (current, default_config) = project_markers(original, config);
    let mut profiles = Vec::with_capacity(config.sections.len());
    for (index, (name, section)) in config.sections.iter().enumerate() {
        validate_profile_name(name)?;
        profiles.push(profile_from_section(
            name,
            section,
            i64::try_from(index).unwrap_or(i64::MAX),
        )?);
    }
    Ok(StoredCatalog {
        current,
        default_config,
        settings_toml: write_settings(&original.settings_toml, &config.settings)?,
        extra_toml: original.extra_toml.clone(),
        revision: original.revision.clone(),
        phase: original.phase,
        migrated_at: original.migrated_at.clone(),
        profiles,
    })
}

fn project_markers(original: &StoredCatalog, config: &CcsConfig) -> (Marker, Marker) {
    if matches!(original.current, Marker::Absent) && config.current_config.is_empty() {
        return (Marker::Absent, Marker::Absent);
    }
    let current = if config.current_config.is_empty() {
        Marker::Empty
    } else {
        Marker::Value(config.current_config.clone())
    };
    let default_config = if config.default_config.is_empty() {
        Marker::Empty
    } else {
        Marker::Value(config.default_config.clone())
    };
    (current, default_config)
}

fn to_config(stored: &StoredCatalog) -> Result<CcsConfig> {
    let mut sections = IndexMap::new();
    for profile in &stored.profiles {
        sections.insert(profile.name.clone(), profile.to_section()?);
    }
    let invented = sections
        .keys()
        .next()
        .cloned()
        .unwrap_or_else(|| "default".to_string());
    let current_config = match &stored.current {
        Marker::Absent | Marker::Empty => String::new(),
        Marker::Value(value) => value.clone(),
    };
    let default_config = match &stored.default_config {
        Marker::Absent => invented,
        Marker::Empty => String::new(),
        Marker::Value(value) => value.clone(),
    };
    Ok(CcsConfig {
        default_config,
        current_config,
        settings: load_settings(stored.settings_toml.as_deref())?,
        sections,
    })
}

fn content_equal(left: &StoredCatalog, right: &StoredCatalog) -> Result<bool> {
    if left.current != right.current || left.default_config != right.default_config {
        return Ok(false);
    }
    if !same_optional_toml(&left.settings_toml, &right.settings_toml)? {
        return Ok(false);
    }
    if !same_toml(&left.extra_toml, &right.extra_toml)? {
        return Ok(false);
    }
    if left.profiles.len() != right.profiles.len() {
        return Ok(false);
    }
    for (left_profile, right_profile) in left.profiles.iter().zip(&right.profiles) {
        if !profile_equal(left_profile, right_profile)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn profile_equal(left: &StoredProfile, right: &StoredProfile) -> Result<bool> {
    Ok(left.name == right.name
        && left.description == right.description
        && left.base_url == right.base_url
        && left.auth_token == right.auth_token
        && left.model == right.model
        && left.small_fast_model == right.small_fast_model
        && left.default_opus_model == right.default_opus_model
        && left.default_sonnet_model == right.default_sonnet_model
        && left.default_haiku_model == right.default_haiku_model
        && left.default_fable_model == right.default_fable_model
        && left.default_opus_model_name == right.default_opus_model_name
        && left.default_sonnet_model_name == right.default_sonnet_model_name
        && left.default_haiku_model_name == right.default_haiku_model_name
        && left.default_fable_model_name == right.default_fable_model_name
        && left.subagent_model == right.subagent_model
        && left.custom_model_option == right.custom_model_option
        && left.custom_model_option_name == right.custom_model_option_name
        && left.effort_level == right.effort_level
        && left.claude_code_auto_compact_window == right.claude_code_auto_compact_window
        && left.api_timeout_ms == right.api_timeout_ms
        && left.claude_code_disable_nonessential_traffic
            == right.claude_code_disable_nonessential_traffic
        && left.provider == right.provider
        && left.provider_type == right.provider_type
        && left.account == right.account
        && same_optional_json(&left.tags_json, &right.tags_json)?
        && left.usage_count == right.usage_count
        && left.enabled == right.enabled
        && same_toml(&left.extra_toml, &right.extra_toml)?
        && left.sort_order == right.sort_order)
}

fn stored_from_toml(content: &str) -> Result<StoredCatalog> {
    let value: toml::Value = toml::from_str(content)
        .map_err(|_| CcrError::ConfigFormatInvalid("配置 TOML 无效".into()))?;
    let Some(mut table) = value.as_table().cloned() else {
        return Err(CcrError::ConfigFormatInvalid("配置必须是表".into()));
    };
    let current = marker_from_table(&table, "current_config")?;
    let default_config = marker_from_table(&table, "default_config")?;
    let settings_toml = match table.get("settings") {
        None => None,
        Some(toml::Value::Table(settings)) => Some(serialize_table(settings)?),
        Some(_) => {
            return Err(CcrError::ConfigFormatInvalid(
                "settings 的类型或值不符合要求".into(),
            ));
        }
    };
    table.remove("current_config");
    table.remove("default_config");
    table.remove("settings");

    let mut profiles = Vec::new();
    let mut extra = toml::Table::new();
    let mut order = 0_i64;
    for (key, value) in table {
        match value {
            toml::Value::Table(section) => {
                validate_profile_name(&key)?;
                profiles.push(profile_from_table(&key, section, order)?);
                order += 1;
            }
            other => {
                extra.insert(key, other);
            }
        }
    }
    Ok(StoredCatalog {
        current,
        default_config,
        settings_toml,
        extra_toml: if extra.is_empty() {
            String::new()
        } else {
            serialize_table(&extra)?
        },
        revision: String::new(),
        phase: Phase::DataAuthoritative,
        migrated_at: None,
        profiles,
    })
}

fn marker_from_table(table: &toml::Table, key: &str) -> Result<Marker> {
    match table.get(key) {
        None => Ok(Marker::Absent),
        Some(toml::Value::String(value)) if value.is_empty() => Ok(Marker::Empty),
        Some(toml::Value::String(value)) => Ok(Marker::Value(value.clone())),
        Some(_) => Err(CcrError::ConfigFormatInvalid(format!(
            "{key} 的类型或值不符合要求"
        ))),
    }
}

fn profile_from_table(
    name: &str,
    mut table: toml::Table,
    sort_order: i64,
) -> Result<StoredProfile> {
    let description = take_string(&mut table, "description")?;
    let base_url = take_string(&mut table, "base_url")?;
    let auth_token = take_string(&mut table, "auth_token")?;
    let model = take_string(&mut table, "model")?;
    let small_fast_model = take_string(&mut table, "small_fast_model")?;
    let default_opus_model = take_string(&mut table, "default_opus_model")?;
    let default_sonnet_model = take_string(&mut table, "default_sonnet_model")?;
    let default_haiku_model = take_string(&mut table, "default_haiku_model")?;
    let default_fable_model = take_string(&mut table, "default_fable_model")?;
    let default_opus_model_name = take_string(&mut table, "default_opus_model_name")?;
    let default_sonnet_model_name = take_string(&mut table, "default_sonnet_model_name")?;
    let default_haiku_model_name = take_string(&mut table, "default_haiku_model_name")?;
    let default_fable_model_name = take_string(&mut table, "default_fable_model_name")?;
    let subagent_model = take_string(&mut table, "subagent_model")?;
    let custom_model_option = take_string(&mut table, "custom_model_option")?;
    let custom_model_option_name = take_string(&mut table, "custom_model_option_name")?;
    let effort_level = take_string(&mut table, "effort_level")?;
    let claude_code_auto_compact_window =
        take_string(&mut table, "claude_code_auto_compact_window")?;
    let api_timeout_ms = take_string(&mut table, "api_timeout_ms")?;
    let claude_code_disable_nonessential_traffic =
        take_string(&mut table, "claude_code_disable_nonessential_traffic")?;
    let provider = take_string(&mut table, "provider")?;
    let provider_type = match table.remove("provider_type") {
        None => None,
        Some(toml::Value::String(value)) => Some(canonical_provider_type(&value)?),
        Some(_) => {
            return Err(CcrError::ConfigFormatInvalid(
                "provider_type 的类型或值不符合要求".into(),
            ));
        }
    };
    let account = take_string(&mut table, "account")?;
    let tags_json = match table.remove("tags") {
        None => None,
        Some(toml::Value::Array(items)) => Some(tags_array_to_json(items)?),
        Some(_) => {
            return Err(CcrError::ConfigFormatInvalid(
                "tags 的类型或值不符合要求".into(),
            ));
        }
    };
    let usage_count = match table.remove("usage_count") {
        None => None,
        Some(toml::Value::Integer(value)) if (0..=i64::from(u32::MAX)).contains(&value) => {
            Some(value)
        }
        Some(_) => {
            return Err(CcrError::ConfigFormatInvalid(
                "usage_count 的类型或值不符合要求".into(),
            ));
        }
    };
    let enabled = match table.remove("enabled") {
        None => None,
        Some(toml::Value::Boolean(value)) => Some(i64::from(value)),
        Some(_) => {
            return Err(CcrError::ConfigFormatInvalid(
                "enabled 的类型或值不符合要求".into(),
            ));
        }
    };
    let extra_toml = if table.is_empty() {
        String::new()
    } else {
        serialize_table(&table)?
    };
    Ok(StoredProfile {
        name: name.to_string(),
        description,
        base_url,
        auth_token,
        model,
        small_fast_model,
        default_opus_model,
        default_sonnet_model,
        default_haiku_model,
        default_fable_model,
        default_opus_model_name,
        default_sonnet_model_name,
        default_haiku_model_name,
        default_fable_model_name,
        subagent_model,
        custom_model_option,
        custom_model_option_name,
        effort_level,
        claude_code_auto_compact_window,
        api_timeout_ms,
        claude_code_disable_nonessential_traffic,
        provider,
        provider_type,
        account,
        tags_json,
        usage_count,
        enabled,
        extra_toml,
        sort_order,
    })
}

fn profile_from_section(
    name: &str,
    section: &ConfigSection,
    sort_order: i64,
) -> Result<StoredProfile> {
    Ok(StoredProfile {
        name: name.to_string(),
        description: section.description.clone(),
        base_url: section.base_url.clone(),
        auth_token: section
            .auth_token
            .as_ref()
            .map(|token| token.expose().to_string()),
        model: section.model.clone(),
        small_fast_model: section.small_fast_model.clone(),
        default_opus_model: section.default_opus_model.clone(),
        default_sonnet_model: section.default_sonnet_model.clone(),
        default_haiku_model: section.default_haiku_model.clone(),
        default_fable_model: section.default_fable_model.clone(),
        default_opus_model_name: section.default_opus_model_name.clone(),
        default_sonnet_model_name: section.default_sonnet_model_name.clone(),
        default_haiku_model_name: section.default_haiku_model_name.clone(),
        default_fable_model_name: section.default_fable_model_name.clone(),
        subagent_model: section.subagent_model.clone(),
        custom_model_option: section.custom_model_option.clone(),
        custom_model_option_name: section.custom_model_option_name.clone(),
        effort_level: section.effort_level.clone(),
        claude_code_auto_compact_window: section.claude_code_auto_compact_window.clone(),
        api_timeout_ms: section.api_timeout_ms.clone(),
        claude_code_disable_nonessential_traffic: section
            .claude_code_disable_nonessential_traffic
            .clone(),
        provider: section.provider.clone(),
        provider_type: section
            .provider_type
            .as_ref()
            .map(|kind| kind.to_string_value().to_string()),
        account: section.account.clone(),
        tags_json: tags_to_json(&section.tags)?,
        usage_count: section.usage_count.map(i64::from),
        enabled: section.enabled.map(i64::from),
        extra_toml: extra_from_other(&section.other)?,
        sort_order,
    })
}

impl StoredProfile {
    fn to_section(&self) -> Result<ConfigSection> {
        Ok(ConfigSection {
            description: self.description.clone(),
            base_url: self.base_url.clone(),
            auth_token: self.auth_token.as_ref().map(Secret::new),
            model: self.model.clone(),
            small_fast_model: self.small_fast_model.clone(),
            default_opus_model: self.default_opus_model.clone(),
            default_sonnet_model: self.default_sonnet_model.clone(),
            default_haiku_model: self.default_haiku_model.clone(),
            default_fable_model: self.default_fable_model.clone(),
            default_opus_model_name: self.default_opus_model_name.clone(),
            default_sonnet_model_name: self.default_sonnet_model_name.clone(),
            default_haiku_model_name: self.default_haiku_model_name.clone(),
            default_fable_model_name: self.default_fable_model_name.clone(),
            subagent_model: self.subagent_model.clone(),
            custom_model_option: self.custom_model_option.clone(),
            custom_model_option_name: self.custom_model_option_name.clone(),
            effort_level: self.effort_level.clone(),
            claude_code_auto_compact_window: self.claude_code_auto_compact_window.clone(),
            api_timeout_ms: self.api_timeout_ms.clone(),
            claude_code_disable_nonessential_traffic: self
                .claude_code_disable_nonessential_traffic
                .clone(),
            provider: self.provider.clone(),
            provider_type: provider_type_from_sql(self.provider_type.clone())?,
            account: self.account.clone(),
            tags: tags_from_json(self.tags_json.clone())?,
            usage_count: usage_from_sql(self.usage_count)?,
            enabled: enabled_from_sql(self.enabled)?,
            other: other_from_extra(&self.extra_toml)?,
        })
    }
}

fn take_string(table: &mut toml::Table, key: &str) -> Result<Option<String>> {
    match table.remove(key) {
        None => Ok(None),
        Some(toml::Value::String(value)) => Ok(Some(value)),
        Some(_) => Err(CcrError::ConfigFormatInvalid(format!(
            "{key} 的类型或值不符合要求"
        ))),
    }
}

fn canonical_provider_type(value: &str) -> Result<String> {
    match value {
        "official_relay" => Ok("official_relay".to_string()),
        "third_party" | "third_party_model" => Ok("third_party_model".to_string()),
        _ => Err(CcrError::ConfigFormatInvalid(
            "provider_type 的值不受支持；允许值为 official_relay、third_party_model".into(),
        )),
    }
}

fn provider_type_from_sql(value: Option<String>) -> Result<Option<ProviderType>> {
    match value.as_deref() {
        None => Ok(None),
        Some("official_relay") => Ok(Some(ProviderType::OfficialRelay)),
        Some("third_party_model") => Ok(Some(ProviderType::ThirdPartyModel)),
        Some(_) => Err(CcrError::ConfigFormatInvalid(
            "provider_type 的值不受支持；允许值为 official_relay、third_party_model".into(),
        )),
    }
}

fn tags_array_to_json(items: toml::value::Array) -> Result<String> {
    let mut tags = Vec::with_capacity(items.len());
    for item in items {
        let Some(tag) = item.as_str() else {
            return Err(CcrError::ConfigFormatInvalid(
                "tags 的类型或值不符合要求".into(),
            ));
        };
        tags.push(tag.to_string());
    }
    serde_json::to_string(&tags).map_err(|_| CcrError::ConfigError("无法序列化 tags".into()))
}

fn tags_to_json(tags: &Option<Vec<String>>) -> Result<Option<String>> {
    match tags {
        None => Ok(None),
        Some(tags) => serde_json::to_string(tags)
            .map(Some)
            .map_err(|_| CcrError::ConfigError("无法序列化 tags".into())),
    }
}

fn tags_from_json(value: Option<String>) -> Result<Option<Vec<String>>> {
    match value {
        None => Ok(None),
        Some(value) => serde_json::from_str(&value)
            .map(Some)
            .map_err(|_| CcrError::ConfigError("tags_json 无法读取".into())),
    }
}

fn usage_from_sql(value: Option<i64>) -> Result<Option<u32>> {
    match value {
        None => Ok(None),
        Some(value) if (0..=i64::from(u32::MAX)).contains(&value) => Ok(Some(value as u32)),
        Some(_) => Err(CcrError::ConfigError("usage_count 列无法读取".into())),
    }
}

fn enabled_from_sql(value: Option<i64>) -> Result<Option<bool>> {
    match value {
        None => Ok(None),
        Some(0) => Ok(Some(false)),
        Some(1) => Ok(Some(true)),
        Some(_) => Err(CcrError::ConfigError("enabled 列无法读取".into())),
    }
}

fn extra_from_other(other: &IndexMap<String, toml::Value>) -> Result<String> {
    if other.is_empty() {
        return Ok(String::new());
    }
    let mut table = toml::Table::new();
    for (key, value) in other {
        table.insert(key.clone(), value.clone());
    }
    serialize_table(&table)
}

fn other_from_extra(extra: &str) -> Result<IndexMap<String, toml::Value>> {
    let table = table_from_toml(extra)?;
    let mut other = IndexMap::new();
    for (key, value) in table {
        other.insert(key, value);
    }
    Ok(other)
}

fn write_settings(original: &Option<String>, settings: &GlobalSettings) -> Result<Option<String>> {
    if settings_match(original, settings)? {
        return Ok(original.clone());
    }
    let mut table = match original {
        Some(text) if !text.trim().is_empty() => table_from_toml(text)?,
        _ => toml::Table::new(),
    };
    table.insert(
        "skip_confirmation".to_string(),
        toml::Value::Boolean(settings.skip_confirmation),
    );
    match &settings.tui_theme {
        Some(theme) => {
            table.insert("tui_theme".to_string(), toml::Value::String(theme.clone()));
        }
        None => {
            table.remove("tui_theme");
        }
    }
    Ok(Some(serialize_table(&table)?))
}

fn settings_match(original: &Option<String>, settings: &GlobalSettings) -> Result<bool> {
    let projected = load_settings(original.as_deref())?;
    Ok(projected.skip_confirmation == settings.skip_confirmation
        && projected.tui_theme == settings.tui_theme)
}

fn load_settings(value: Option<&str>) -> Result<GlobalSettings> {
    let Some(value) = value.filter(|value| !value.trim().is_empty()) else {
        return Ok(GlobalSettings::default());
    };
    toml::from_str(value).map_err(|_| CcrError::ConfigError("settings 无法读取".into()))
}

fn serialize_settings(settings: &GlobalSettings) -> Result<String> {
    toml::to_string(settings).map_err(|_| CcrError::ConfigError("无法序列化 settings".into()))
}

fn serialize_table(table: &toml::Table) -> Result<String> {
    toml::to_string(table).map_err(|_| CcrError::ConfigError("无法序列化扩展字段".into()))
}

fn table_from_toml(text: &str) -> Result<toml::Table> {
    if text.trim().is_empty() {
        return Ok(toml::Table::new());
    }
    toml::from_str(text).map_err(|_| CcrError::ConfigError("扩展字段无法读取".into()))
}

fn render_catalog(stored: &StoredCatalog) -> Result<String> {
    let mut table = table_from_toml(&stored.extra_toml)?;
    insert_marker(&mut table, "current_config", &stored.current);
    insert_marker(&mut table, "default_config", &stored.default_config);
    if let Some(settings) = stored
        .settings_toml
        .as_deref()
        .filter(|text| !text.trim().is_empty())
    {
        let _ = table.insert(
            "settings".to_string(),
            toml::Value::Table(table_from_toml(settings)?),
        );
    }
    for profile in &stored.profiles {
        let _ = table.insert(
            profile.name.clone(),
            toml::Value::Table(render_profile(profile)?),
        );
    }
    toml::to_string(&table).map_err(|_| CcrError::ConfigError("无法序列化配置目录".into()))
}

fn insert_marker(table: &mut toml::Table, key: &str, marker: &Marker) {
    match marker {
        Marker::Absent => {
            table.remove(key);
        }
        Marker::Empty => {
            let _ = table.insert(key.to_string(), toml::Value::String(String::new()));
        }
        Marker::Value(value) => {
            let _ = table.insert(key.to_string(), toml::Value::String(value.clone()));
        }
    }
}

fn render_profile(profile: &StoredProfile) -> Result<toml::Table> {
    let mut table = table_from_toml(&profile.extra_toml)?;
    let fields = [
        ("description", &profile.description),
        ("base_url", &profile.base_url),
        ("auth_token", &profile.auth_token),
        ("model", &profile.model),
        ("small_fast_model", &profile.small_fast_model),
        ("default_opus_model", &profile.default_opus_model),
        ("default_sonnet_model", &profile.default_sonnet_model),
        ("default_haiku_model", &profile.default_haiku_model),
        ("default_fable_model", &profile.default_fable_model),
        ("default_opus_model_name", &profile.default_opus_model_name),
        (
            "default_sonnet_model_name",
            &profile.default_sonnet_model_name,
        ),
        (
            "default_haiku_model_name",
            &profile.default_haiku_model_name,
        ),
        (
            "default_fable_model_name",
            &profile.default_fable_model_name,
        ),
        ("subagent_model", &profile.subagent_model),
        ("custom_model_option", &profile.custom_model_option),
        (
            "custom_model_option_name",
            &profile.custom_model_option_name,
        ),
        ("effort_level", &profile.effort_level),
        (
            "claude_code_auto_compact_window",
            &profile.claude_code_auto_compact_window,
        ),
        ("api_timeout_ms", &profile.api_timeout_ms),
        (
            "claude_code_disable_nonessential_traffic",
            &profile.claude_code_disable_nonessential_traffic,
        ),
        ("provider", &profile.provider),
        ("provider_type", &profile.provider_type),
        ("account", &profile.account),
    ];
    for (key, value) in fields {
        put_optional_string(&mut table, key, value);
    }
    match tags_from_json(profile.tags_json.clone())? {
        Some(tags) => {
            let _ = table.insert(
                "tags".to_string(),
                toml::Value::Array(tags.into_iter().map(toml::Value::String).collect()),
            );
        }
        None => {
            table.remove("tags");
        }
    }
    match profile.usage_count {
        Some(value) => {
            let _ = table.insert("usage_count".to_string(), toml::Value::Integer(value));
        }
        None => {
            table.remove("usage_count");
        }
    }
    match enabled_from_sql(profile.enabled)? {
        Some(value) => {
            let _ = table.insert("enabled".to_string(), toml::Value::Boolean(value));
        }
        None => {
            table.remove("enabled");
        }
    }
    Ok(table)
}

fn put_optional_string(table: &mut toml::Table, key: &str, value: &Option<String>) {
    match value {
        Some(value) => {
            let _ = table.insert(key.to_string(), toml::Value::String(value.clone()));
        }
        None => {
            table.remove(key);
        }
    }
}

fn same_toml(left: &str, right: &str) -> Result<bool> {
    Ok(table_from_toml(left)? == table_from_toml(right)?)
}

fn same_optional_toml(left: &Option<String>, right: &Option<String>) -> Result<bool> {
    match (left, right) {
        (None, None) => Ok(true),
        (Some(left), Some(right)) => same_toml(left, right),
        _ => Ok(false),
    }
}

fn same_optional_json(left: &Option<String>, right: &Option<String>) -> Result<bool> {
    match (left, right) {
        (None, None) => Ok(true),
        (Some(left), Some(right)) => {
            let left: serde_json::Value = serde_json::from_str(left)
                .map_err(|_| CcrError::ConfigError("tags_json 无法读取".into()))?;
            let right: serde_json::Value = serde_json::from_str(right)
                .map_err(|_| CcrError::ConfigError("tags_json 无法读取".into()))?;
            Ok(left == right)
        }
        _ => Ok(false),
    }
}

fn read_stored(connection: &Connection) -> Result<StoredCatalog> {
    let (current, default_config, settings_toml, extra_toml, revision, phase, migrated_at) =
        read_document_row(connection)?;
    let phase = Phase::parse(&phase)?;
    let mut statement = connection.prepare(PROFILE_SELECT).map_err(sql_error)?;
    let profiles = statement
        .query_map([], read_profile_row)
        .map_err(sql_error)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(sql_error)?;
    Ok(StoredCatalog {
        current: Marker::from_sql(current),
        default_config: Marker::from_sql(default_config),
        settings_toml,
        extra_toml,
        revision,
        phase,
        migrated_at,
        profiles,
    })
}

type DocumentRow = (
    Option<String>,
    Option<String>,
    Option<String>,
    String,
    String,
    String,
    Option<String>,
);

fn read_document_row(connection: &Connection) -> Result<DocumentRow> {
    connection
        .query_row(
            "SELECT current_config, default_config, settings_toml, extra_toml, revision,
                    migration_phase, migrated_at
             FROM document WHERE id = 1",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => {
                CcrError::ConfigError("配置目录尚未完成迁移".into())
            }
            other => sql_error(other),
        })
}

fn read_profile_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredProfile> {
    Ok(StoredProfile {
        name: row.get(0)?,
        description: row.get(1)?,
        base_url: row.get(2)?,
        auth_token: row.get(3)?,
        model: row.get(4)?,
        small_fast_model: row.get(5)?,
        default_opus_model: row.get(6)?,
        default_sonnet_model: row.get(7)?,
        default_haiku_model: row.get(8)?,
        default_fable_model: row.get(9)?,
        default_opus_model_name: row.get(10)?,
        default_sonnet_model_name: row.get(11)?,
        default_haiku_model_name: row.get(12)?,
        default_fable_model_name: row.get(13)?,
        subagent_model: row.get(14)?,
        custom_model_option: row.get(15)?,
        custom_model_option_name: row.get(16)?,
        effort_level: row.get(17)?,
        claude_code_auto_compact_window: row.get(18)?,
        api_timeout_ms: row.get(19)?,
        claude_code_disable_nonessential_traffic: row.get(20)?,
        provider: row.get(21)?,
        provider_type: row.get(22)?,
        account: row.get(23)?,
        tags_json: row.get(24)?,
        usage_count: row.get(25)?,
        enabled: row.get(26)?,
        extra_toml: row.get(27)?,
        sort_order: row.get(28)?,
    })
}

fn write_all(connection: &Connection, stored: &StoredCatalog) -> Result<()> {
    replace_profiles(connection, &stored.profiles)?;
    connection
        .execute(
            "INSERT INTO document (
                id, current_config, default_config, settings_toml, extra_toml,
                revision, migration_phase, migrated_at
             ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                current_config = excluded.current_config,
                default_config = excluded.default_config,
                settings_toml = excluded.settings_toml,
                extra_toml = excluded.extra_toml,
                revision = excluded.revision,
                migration_phase = excluded.migration_phase,
                migrated_at = excluded.migrated_at",
            params![
                stored.current.to_sql(),
                stored.default_config.to_sql(),
                stored.settings_toml,
                stored.extra_toml,
                stored.revision,
                stored.phase.as_str(),
                stored.migrated_at,
            ],
        )
        .map_err(sql_error)?;
    Ok(())
}

fn write_content(connection: &Connection, stored: &StoredCatalog) -> Result<()> {
    replace_profiles(connection, &stored.profiles)?;
    let changed = connection
        .execute(
            "UPDATE document
             SET current_config = ?1, default_config = ?2, settings_toml = ?3,
                 extra_toml = ?4, revision = ?5
             WHERE id = 1",
            params![
                stored.current.to_sql(),
                stored.default_config.to_sql(),
                stored.settings_toml,
                stored.extra_toml,
                stored.revision,
            ],
        )
        .map_err(sql_error)?;
    if changed == 1 {
        Ok(())
    } else {
        Err(CcrError::ConfigError("配置目录缺少文档行".into()))
    }
}

fn replace_profiles(connection: &Connection, profiles: &[StoredProfile]) -> Result<()> {
    connection
        .execute("DELETE FROM profiles", [])
        .map_err(sql_error)?;
    let updated_at = timestamp();
    for profile in profiles {
        connection
            .execute(
                INSERT_PROFILE,
                params![
                    profile.name,
                    profile.description,
                    profile.base_url,
                    profile.auth_token,
                    profile.model,
                    profile.small_fast_model,
                    profile.default_opus_model,
                    profile.default_sonnet_model,
                    profile.default_haiku_model,
                    profile.default_fable_model,
                    profile.default_opus_model_name,
                    profile.default_sonnet_model_name,
                    profile.default_haiku_model_name,
                    profile.default_fable_model_name,
                    profile.subagent_model,
                    profile.custom_model_option,
                    profile.custom_model_option_name,
                    profile.effort_level,
                    profile.claude_code_auto_compact_window,
                    profile.api_timeout_ms,
                    profile.claude_code_disable_nonessential_traffic,
                    profile.provider,
                    profile.provider_type,
                    profile.account,
                    profile.tags_json,
                    profile.usage_count,
                    profile.enabled,
                    profile.extra_toml,
                    profile.sort_order,
                    updated_at,
                ],
            )
            .map_err(sql_error)?;
    }
    Ok(())
}

fn read_phase(connection: &Connection) -> Result<Option<Phase>> {
    let table: Option<String> = connection
        .query_row(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'document'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(sql_error)?;
    if table.is_none() {
        return Ok(None);
    }
    let phase: Option<String> = connection
        .query_row(
            "SELECT migration_phase FROM document WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(sql_error)?;
    match phase.as_deref() {
        None => Ok(None),
        Some(value) => Ok(Some(Phase::parse(value)?)),
    }
}

fn ensure_schema(connection: &Connection) -> Result<()> {
    let version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(sql_error)?;
    if version > 1 {
        return Err(CcrError::ConfigError("配置目录版本不受支持".into()));
    }
    connection.execute_batch(SCHEMA).map_err(sql_error)?;
    if version == 0 {
        connection
            .execute_batch("PRAGMA user_version = 1")
            .map_err(sql_error)?;
    }
    Ok(())
}

fn apply_pragmas(connection: &Connection) -> Result<()> {
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(sql_error)?;
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA synchronous = NORMAL;
             PRAGMA journal_mode = WAL;",
        )
        .map_err(sql_error)?;
    Ok(())
}

/// Read a live catalog, including commits that are still only in the WAL.
fn open_live_ro(path: &Path) -> Result<Connection> {
    let connection =
        Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(sql_error)?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(sql_error)?;
    Ok(connection)
}

/// Read a checkpointed snapshot file. `immutable=1` ignores any sibling WAL.
fn open_snapshot_ro(path: &Path) -> Result<Connection> {
    let uri = sqlite_read_uri(path)?;
    Connection::open_with_flags(
        &uri,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
    )
    .map_err(sql_error)
}

fn sqlite_read_uri(path: &Path) -> Result<String> {
    let absolute = std::path::absolute(path)
        .map_err(|error| CcrError::ConfigError(format!("解析配置路径失败: {error}")))?;
    let text = absolute.to_string_lossy().replace('\\', "/");
    let mut encoded = String::from("file:///");
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b':' | b'.' | b'-' | b'_' | b'~' => {
                encoded.push(char::from(byte));
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded.push_str("?mode=ro&immutable=1");
    Ok(encoded)
}

pub(crate) fn checkpoint(connection: &Connection) -> Result<()> {
    #[cfg(any(test, feature = "test-support"))]
    if failpoints::take_checkpoint() {
        return Err(CcrError::ConfigError("配置目录检查点失败".into()));
    }
    connection
        .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(sql_error)?;
    Ok(())
}

fn checkpoint_after_commit(connection: &Connection) {
    if let Err(error) = checkpoint(connection) {
        tracing::warn!(
            error = %error,
            "配置目录已提交，检查点未完成，后续读取仍可见已提交的 WAL"
        );
    }
}

const PUBLISH_BUSY: &str = "配置目录检查点忙碌，本次不发布";

/// Copy a locked catalog into a new owner-only snapshot and return those bytes.
///
/// The live database file is not read. A busy or failed checkpoint returns before
/// any snapshot bytes are produced.
pub fn snapshot_profile_catalog(db: &Path) -> Result<Vec<u8>> {
    let _guard = acquire_catalog_lock(db)?;
    let source = open_rw(db)?;
    checkpoint_for_publish(&source)?;
    let temp = TempSnapshot::create()?;
    let mut dest = Connection::open(&temp.path).map_err(sql_error)?;
    restrict_sidecars(&temp.path)?;
    copy_database(&source, &mut dest)?;
    checkpoint_for_publish(&dest)?;
    restrict_sidecars(&temp.path)?;
    ensure_snapshot_readable(&dest)?;
    drop(dest);
    drop(source);
    fs::read(&temp.path).map_err(|error| {
        CcrError::FileIoError(format!(
            "读取配置目录快照失败 {}: {error}",
            temp.path.display()
        ))
    })
}

/// Install `staging` into `live`. An existing live database is updated in place.
pub fn install_profile_catalog(live: &Path, staging: &Path) -> Result<()> {
    ensure_snapshot_file(staging)?;
    let _guard = acquire_catalog_lock(live)?;
    if live.is_file() {
        replace_live_catalog(live, staging)
    } else {
        install_new_catalog(live, staging)
    }
}

fn checkpoint_for_publish(connection: &Connection) -> Result<()> {
    connection
        .busy_timeout(Duration::from_millis(200))
        .map_err(sql_error)?;
    let queried = connection.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| {
        row.get::<_, i64>(0)
    });
    let _ = connection.busy_timeout(Duration::from_secs(5));
    let busy = queried.map_err(sql_error)?;
    if busy != 0 {
        return Err(CcrError::ConfigError(PUBLISH_BUSY.into()));
    }
    Ok(())
}

fn copy_database(source: &Connection, dest: &mut Connection) -> Result<()> {
    let backup = Backup::new(source, dest).map_err(sql_error)?;
    loop {
        match backup.step(-1).map_err(sql_error)? {
            StepResult::Done => return Ok(()),
            StepResult::More => {}
            StepResult::Busy | StepResult::Locked => {
                return Err(CcrError::ConfigError(PUBLISH_BUSY.into()));
            }
            _ => {
                return Err(CcrError::ConfigError("配置目录快照复制未完成".into()));
            }
        }
    }
}

fn ensure_snapshot_readable(connection: &Connection) -> Result<()> {
    let status: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(sql_error)?;
    if status != "ok" {
        return Err(CcrError::ConfigError(format!(
            "配置目录完整性检查失败: {status}"
        )));
    }
    let version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(sql_error)?;
    if version != 1 {
        return Err(CcrError::ConfigError("配置目录版本不受支持".into()));
    }
    let stored = read_stored(connection)?;
    if stored.revision.is_empty() {
        return Err(CcrError::ConfigError("配置目录快照缺少修订".into()));
    }
    let _profiles = stored.profiles.len();
    Ok(())
}

fn ensure_snapshot_file(path: &Path) -> Result<()> {
    let connection = open_snapshot_ro(path)?;
    ensure_snapshot_readable(&connection)
}

/// Reject a snapshot that fails integrity, schema, or document checks.
pub fn validate_profile_catalog_file(path: &Path) -> Result<()> {
    ensure_snapshot_file(path)
}

fn replace_live_catalog(live: &Path, staging: &Path) -> Result<()> {
    let connection = open_rw(live)?;
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(sql_error)?;
    let outcome = (|| {
        let stored = {
            let staging_connection = open_snapshot_ro(staging)?;
            ensure_snapshot_readable(&staging_connection)?;
            read_stored(&staging_connection)?
        };
        write_all(&connection, &stored)?;
        #[cfg(any(test, feature = "test-support"))]
        if failpoints::take_install() {
            return Err(CcrError::ConfigError("配置目录安装提交失败".into()));
        }
        connection.execute_batch("COMMIT").map_err(sql_error)?;
        Ok(())
    })();
    if outcome.is_err() {
        let _ = connection.execute_batch("ROLLBACK");
        return outcome;
    }
    checkpoint_after_commit(&connection);
    restrict_sidecars(live)?;
    Ok(())
}

fn install_new_catalog(live: &Path, staging: &Path) -> Result<()> {
    create_restricted_empty(live)?;
    let outcome = (|| {
        let mut dest = Connection::open(live).map_err(sql_error)?;
        restrict_sidecars(live)?;
        let source = open_snapshot_ro(staging)?;
        copy_database(&source, &mut dest)?;
        checkpoint_for_publish(&dest)?;
        restrict_sidecars(live)?;
        ensure_snapshot_readable(&dest)?;
        Ok(())
    })();
    if outcome.is_err() {
        remove_sqlite_files(live);
    }
    outcome
}

struct TempSnapshot {
    path: PathBuf,
}

impl TempSnapshot {
    fn create() -> Result<Self> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!(
            "ccr-profile-snapshot-{}-{nanos}.db",
            std::process::id()
        ));
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("profiles.db"))
        {
            return Err(CcrError::ConfigError("配置目录快照路径无效".into()));
        }
        create_restricted_empty(&path)?;
        Ok(Self { path })
    }
}

impl Drop for TempSnapshot {
    fn drop(&mut self) {
        remove_sqlite_files(&self.path);
    }
}

fn install_session(session: Rc<Session>) {
    ACTIVE.with(|slot| *slot.borrow_mut() = Some(session));
}

fn clear_session(session: &Rc<Session>) {
    ACTIVE.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot
            .as_ref()
            .is_some_and(|current| Rc::ptr_eq(current, session))
        {
            *slot = None;
        }
    });
}

fn rollback_session(session: &Session) {
    let _ = session.conn.borrow().execute_batch("ROLLBACK");
}

fn active_session(db: &Path) -> Option<Rc<Session>> {
    let path = normalize(db).ok()?;
    ACTIVE.with(|slot| {
        slot.borrow()
            .as_ref()
            .and_then(|session| (session.path == path).then(|| Rc::clone(session)))
    })
}

fn normalize(path: &Path) -> Result<PathBuf> {
    normalized_resource_path(path)
        .map_err(|error| CcrError::ConfigError(format!("解析配置路径失败: {error}")))
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
    PathBuf::from(name)
}

pub(crate) fn sql_error(error: rusqlite::Error) -> CcrError {
    CcrError::ConfigError(format!("配置目录读写失败: {error}"))
}

fn new_revision() -> String {
    static SEQUENCE: AtomicU64 = AtomicU64::new(1);
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    content_version_token(format!("catalog-{nanos}-{sequence}-{}", std::process::id()).as_bytes())
}

fn timestamp() -> String {
    chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.6fZ")
        .to_string()
}

#[cfg(any(test, feature = "test-support"))]
mod failpoints {
    use std::cell::Cell;
    use std::sync::atomic::{AtomicBool, Ordering};

    thread_local! {
        static COMMIT: Cell<bool> = const { Cell::new(false) };
        static CHECKPOINT: Cell<bool> = const { Cell::new(false) };
        static USAGE: Cell<bool> = const { Cell::new(false) };
    }

    static INSTALL: AtomicBool = AtomicBool::new(false);

    pub(super) fn arm_commit() {
        COMMIT.set(true);
    }

    pub(super) fn take_commit() -> bool {
        COMMIT.replace(false)
    }

    pub(super) fn arm_checkpoint() {
        CHECKPOINT.set(true);
    }

    pub(super) fn take_checkpoint() -> bool {
        CHECKPOINT.replace(false)
    }

    pub(super) fn arm_usage() {
        USAGE.set(true);
    }

    pub(super) fn take_usage() -> bool {
        USAGE.replace(false)
    }

    pub(super) fn arm_install() {
        INSTALL.store(true, Ordering::SeqCst);
    }

    pub(super) fn take_install() -> bool {
        INSTALL.swap(false, Ordering::SeqCst)
    }
}

#[cfg(any(test, feature = "test-support"))]
pub fn fail_next_catalog_commit() {
    failpoints::arm_commit();
}

#[cfg(any(test, feature = "test-support"))]
pub fn fail_next_catalog_checkpoint() {
    failpoints::arm_checkpoint();
}

#[cfg(any(test, feature = "test-support"))]
pub fn fail_next_usage_increment() {
    failpoints::arm_usage();
}

#[cfg(any(test, feature = "test-support"))]
pub fn fail_next_profile_catalog_install() {
    failpoints::arm_install();
}

#[cfg(test)]
mod tests;
