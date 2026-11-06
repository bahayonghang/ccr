//! Explicit, independent CCR main database and shared Claude Observer projections.
use super::*;
use crate::runtime::ConfigSource;
use ccr_app::usage::observer::{
    self as shared,
    subscription::{self, SubscriptionDto},
};
use rusqlite::{Connection, OpenFlags};
use serde_json::{Value, json};
use std::sync::OnceLock;
use tokio::sync::watch;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    LocalOnly,
    Missing,
    Storage,
    Invalid,
    Conflict,
    Busy,
    Interrupted,
}
#[derive(Clone, Debug)]
pub struct Report {
    pub usage: Result<Value, Error>,
    pub tools: Result<Value, Error>,
    pub insight: Result<Value, Error>,
    pub subscription: Result<Option<SubscriptionDto>, Error>,
}
#[derive(Clone, Debug, Default)]
pub enum Update {
    #[default]
    Idle,
    Running,
    Finished(Result<SubscriptionDto, Error>),
}
#[derive(Clone)]
struct Paths {
    database: PathBuf,
    home: PathBuf,
}
pub struct Owner {
    paths: OnceLock<Result<Paths, Error>>,
    state: Mutex<Update>,
    updates: watch::Sender<Update>,
}
impl Default for Owner {
    fn default() -> Self {
        Self {
            paths: OnceLock::new(),
            state: Mutex::new(Update::Idle),
            updates: watch::channel(Update::Idle).0,
        }
    }
}
impl Owner {
    fn paths(&self, source: &ConfigSource) -> Result<Paths, Error> {
        self.paths
            .get_or_init(|| match source {
                ConfigSource::Benchmark(layout) => Ok(Paths {
                    database: layout.path("ui/ccr-ui.db"),
                    home: layout.roots.home.clone(),
                }),
                ConfigSource::Local => Ok(Paths {
                    database: database::ui_database_path().map_err(|_| Error::Storage)?,
                    home: dirs::home_dir().ok_or(Error::Storage)?,
                }),
                ConfigSource::Fixture { registry, .. } => {
                    if !registry.is_absolute() {
                        return Err(Error::Storage);
                    }
                    let root = registry.parent().ok_or(Error::Storage)?;
                    Ok(Paths {
                        database: root.join("observer.db"),
                        home: root.join("observer-migration-home"),
                    })
                }
            })
            .clone()
    }
    fn finish(&self, result: Result<SubscriptionDto, Error>) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *state = Update::Finished(result);
        self.updates.send_replace(state.clone());
    }
}
fn reject_links(path: &Path) -> Result<(), Error> {
    for ancestor in path.ancestors() {
        match std::fs::symlink_metadata(ancestor) {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    return Err(Error::Storage);
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err(Error::Storage);
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(Error::Storage),
        }
    }
    Ok(())
}
fn connection(paths: &Paths, write: bool) -> Result<Connection, Error> {
    reject_links(&paths.database)?;
    let flags = if write {
        OpenFlags::SQLITE_OPEN_READ_WRITE
    } else {
        OpenFlags::SQLITE_OPEN_READ_ONLY
    };
    let conn = Connection::open_with_flags(&paths.database, flags).map_err(|_| Error::Storage)?;
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|_| Error::Storage)?;
    Ok(conn)
}
fn existing(paths: &Paths) -> Result<bool, Error> {
    reject_links(&paths.database)?;
    match std::fs::metadata(&paths.database) {
        Ok(meta) if meta.is_file() => Ok(true),
        Ok(_) => Err(Error::Storage),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(Error::Storage),
    }
}
fn usage(
    paths: Option<&InsightsPaths>,
    days: i64,
    clock: ccr_app::usage::report_clock::ReportClock,
) -> Result<Value, Error> {
    let paths = AppPaths::from_root(&paths.ok_or(Error::Missing)?.usage_root);
    Ok(json!({
        "daily": shared::daily_trend_with_clock(&paths, Some(days),clock).map_err(|_| Error::Storage)?,
        "projects": shared::cost_breakdown_with_clock(&paths, "project", Some(days), Some(20),clock).map_err(|_| Error::Storage)?,
        "models": shared::cost_breakdown_with_clock(&paths, "model", Some(days), Some(20),clock).map_err(|_| Error::Storage)?,
        "cache": shared::cache_stats(&paths).map_err(|_| Error::Storage)?,
    }))
}
fn read(
    owner: &Owner,
    source: &ConfigSource,
    insights: Option<&InsightsPaths>,
    days: i64,
    calls: bool,
) -> Result<Report, Error> {
    let paths = owner.paths(source)?;
    let clock = source
        .benchmark()
        .map(|layout| ccr_app::usage::report_clock::ReportClock::fixed_utc(layout.as_of))
        .unwrap_or_else(ccr_app::usage::report_clock::ReportClock::live);
    let usage = usage(insights, days, clock);
    if !existing(&paths)? {
        return Ok(Report {
            usage,
            tools: Err(Error::Missing),
            insight: Err(Error::Missing),
            subscription: Ok(None),
        });
    }
    let conn = match connection(&paths, false) {
        Ok(conn) => conn,
        Err(error) => {
            return Ok(Report {
                usage,
                tools: Err(error),
                insight: Err(error),
                subscription: Err(error),
            });
        }
    };
    let subscription = subscription::read(&conn)
        .map(Some)
        .map_err(|_| Error::Storage);
    let tools = (|| {
        Ok(json!({
        // The legacy wire projection has a compatibility tokens=0 placeholder.
        // Native reports omit unsupported token/model fields rather than presenting them as facts.
        "sessions": shared::top_sessions_with_clock(&conn, Some(20), Some(if calls { "calls" } else { "cost" }),clock).map_err(|_| Error::Storage)?.into_iter().map(|row| json!({"session_id":row.session_id,"project_path":row.project_path,"cost_usd":row.cost_usd,"tool_call_count":row.tool_call_count})).collect::<Vec<_>>(),
            "heatmap": shared::tool_heatmap_with_clock(&conn, Some(days),clock).map_err(|_| Error::Storage)?,
            "tools": shared::top_tools_with_clock(&conn, Some(days), Some(20),clock).map_err(|_| Error::Storage)?,
        }))
    })();
    let insight = insights
        .ok_or(Error::Missing)
        .and_then(|paths| {
            shared::insight_with_clock(&AppPaths::from_root(&paths.usage_root), &conn, clock)
                .map_err(|_| Error::Storage)
        })
        .and_then(|value| serde_json::to_value(value).map_err(|_| Error::Storage));
    Ok(Report {
        usage,
        tools,
        insight,
        subscription,
    })
}
fn initialize(paths: &Paths, source: &ConfigSource) -> Result<(), Error> {
    reject_links(&paths.home)?;
    if let Some(parent) = paths.database.parent() {
        std::fs::create_dir_all(parent).map_err(|_| Error::Storage)?;
    }
    if source.is_fixture() {
        std::fs::create_dir_all(&paths.home).map_err(|_| Error::Storage)?;
    }
    match source {
        ConfigSource::Benchmark(layout) => {
            layout.main_pool().map_err(|_| Error::Storage)?;
        }
        ConfigSource::Local => {
            if database::ui_database_path().map_err(|_| Error::Storage)? != paths.database
                || database::get_pool().is_some()
            {
                return Err(Error::Storage);
            }
            // Reuse the registered main app pool when observer and check-in
            // share storage; do not create a second main-database pool.
            database::initialize_app_pool().map_err(|_| Error::Storage)?;
        }
        ConfigSource::Fixture { .. } => {
            let pool = database::create_pool(&paths.database, None).map_err(|_| Error::Storage)?;
            let conn = pool.get().map_err(|_| Error::Storage)?;
            database::migrations::run_all_migrations(&conn, &paths.home)
                .map_err(|_| Error::Storage)?;
        }
    }
    Ok(())
}

fn save(
    owner: &Owner,
    source: &ConfigSource,
    expected: Option<SubscriptionDto>,
    proposed: SubscriptionDto,
) -> Result<SubscriptionDto, Error> {
    if !matches!(proposed.mode.as_str(), "auto" | "api_key" | "subscription")
        || !matches!(
            proposed.plan.as_str(),
            "free_pro" | "max5x" | "max20x" | "team" | "enterprise" | "custom"
        )
        || !proposed.monthly_usd.is_finite()
        || proposed.monthly_usd < 0.0
    {
        return Err(Error::Invalid);
    }
    let paths = owner.paths(source)?;
    let present = existing(&paths)?;
    if expected.is_none() && present {
        return Err(Error::Conflict);
    }
    if expected.is_some() && !present {
        return Err(Error::Conflict);
    }
    if !present {
        initialize(&paths, source)?;
    }
    let conn = connection(&paths, true)?;
    match subscription::save_if_current(&conn, &expected.unwrap_or_default(), &proposed)
        .map_err(|_| Error::Storage)?
    {
        subscription::SaveOutcome::Saved(saved) => Ok(saved),
        subscription::SaveOutcome::Conflict => Err(Error::Conflict),
    }
}
struct Pending {
    owner: Arc<Owner>,
    done: bool,
}
impl Drop for Pending {
    fn drop(&mut self) {
        if !self.done {
            self.owner.finish(Err(Error::Interrupted));
        }
    }
}
impl InsightsRequests {
    /// Explicit import tail. Reads only the captured provider root and never resolves HOME.
    pub(super) fn ingest_observer_tools(
        &self,
        provider_home: &Path,
        cancelled: impl Fn() -> bool,
    ) -> Result<shared::scanner::IngestReport, Error> {
        if cancelled() {
            return Err(Error::Interrupted);
        }
        let root = provider_home.join(".claude/projects");
        reject_links(&root)?;
        match std::fs::metadata(&root) {
            Ok(meta) if meta.is_dir() => {}
            Ok(_) => return Err(Error::Storage),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Default::default());
            }
            Err(_) => return Err(Error::Storage),
        }
        let paths = self.services.observer.paths(&self.services.source)?;
        if !existing(&paths)? {
            initialize(&paths, &self.services.source)?;
        }
        if cancelled() {
            return Err(Error::Interrupted);
        }
        let mut conn = connection(&paths, true)?;
        shared::scanner::ingest_incremental_at(&mut conn, &root, cancelled)
            .map_err(|_| Error::Interrupted)
    }
    pub fn observer_read(
        &self,
        key: RequestKey,
        days: i64,
        calls: bool,
    ) -> Result<TaskSubscription<Report, Error>, AdmissionError> {
        let requests = self.clone();
        let local = matches!(key.environment, EnvironmentId::Local(_));
        self.services.executor.read(key, move |_, _| {
            if !local {
                return Err(Error::LocalOnly);
            }
            read(
                &requests.services.observer,
                &requests.services.source,
                requests.paths.as_ref(),
                days.clamp(1, 365),
                calls,
            )
        })
    }
    pub fn observer_update(&self) -> Update {
        self.services.observer.updates.borrow().clone()
    }
    pub fn observe_observer(&self) -> watch::Receiver<Update> {
        self.services.observer.updates.subscribe()
    }
    pub fn observer_save(
        &self,
        key: RequestKey,
        expected: Option<SubscriptionDto>,
        proposed: SubscriptionDto,
    ) -> Result<(), Error> {
        if !matches!(key.environment, EnvironmentId::Local(_)) {
            return Err(Error::LocalOnly);
        }
        let owner = self.services.observer.clone();
        {
            let mut state = owner.state.lock().map_err(|_| Error::Storage)?;
            if matches!(*state, Update::Running) {
                return Err(Error::Busy);
            }
            *state = Update::Running;
            owner.updates.send_replace(state.clone());
        }
        let pending = Pending {
            owner: owner.clone(),
            done: false,
        };
        let source = self.services.source.clone();
        self.services
            .executor
            .job(key, move |_, _| {
                let mut pending = pending;
                let result = save(&owner, &source, expected, proposed);
                owner.finish(result.clone());
                pending.done = true;
                match result {
                    Ok(value) => Completion::Success(value),
                    Err(error) => Completion::Failed(error),
                }
            })
            .map(drop)
            .map_err(|_| Error::Interrupted)
    }
}

#[cfg(test)]
mod tests;
