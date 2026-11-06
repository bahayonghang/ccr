//! Application-lifetime monitoring; captured main storage and a bounded safe memory feed.
pub mod system;
use super::{ConfigSource, Services};
use ccr_app::runtime::{AdmissionError, EnvironmentId, RequestKey, TaskSubscription};
use ccr_db::services::log_persistence::LogPersistenceService;
use ccr_types::{MonitoringEntry, MonitoringFeedQuery, MonitoringLevel};
use rusqlite::{Connection, OpenFlags};
use std::{
    collections::{HashSet, VecDeque},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};

pub const ENTRY_LIMIT: usize = 500;
pub const PAYLOAD_LIMIT: usize = 512 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    LocalOnly,
    Missing,
    Storage,
}
#[derive(Default)]
struct State {
    entries: VecDeque<MonitoringEntry>,
    bytes: usize,
    evicted: usize,
    last_write: Option<Result<(), Error>>,
    eviction_watch: Option<(&'static str, u64)>,
}
pub struct Owner {
    path: OnceLock<Result<PathBuf, Error>>,
    state: Mutex<State>,
    #[cfg(test)]
    reads: std::sync::atomic::AtomicUsize,
}
impl Default for Owner {
    fn default() -> Self {
        Self {
            path: OnceLock::new(),
            state: Mutex::new(State::default()),
            #[cfg(test)]
            reads: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Metrics {
    pub active_tasks: usize,
    pub retained_entries: usize,
    pub retained_payload_bytes: usize,
    pub evicted_entries: usize,
    pub process_rss_bytes: Option<u64>,
}
#[derive(Clone, Debug)]
pub struct Report {
    pub entries: Vec<MonitoringEntry>,
    pub storage: Result<(), Error>,
    pub last_write: Option<Result<(), Error>>,
    pub metrics: Metrics,
}
pub(super) struct RecordReceipt {
    pub attempted: bool,
    pub persisted: Result<(), Error>,
}
impl Owner {
    pub(super) fn track_evictions(&self, prefix: &'static str) -> Result<(), Error> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.eviction_watch.is_some() {
            return Err(Error::Storage);
        }
        state.eviction_watch = Some((prefix, 0));
        Ok(())
    }
    /// Reading the final delta and detaching are atomic with every real ring pop.
    pub(super) fn eviction_delta(&self, detach: bool) -> u64 {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if detach {
            return state.eviction_watch.take().map_or(0, |(_, count)| count);
        }
        state
            .eviction_watch
            .as_mut()
            .map_or(0, |(_, count)| std::mem::take(count))
    }

    #[cfg(test)]
    pub(crate) fn read_count(&self) -> usize {
        self.reads.load(std::sync::atomic::Ordering::Relaxed)
    }
    fn path(&self, source: &ConfigSource) -> Result<&Path, Error> {
        self.path
            .get_or_init(|| match source {
                ConfigSource::Benchmark(layout) => Ok(layout.path("ui/ccr-ui.db")),
                ConfigSource::Local => {
                    ccr_db::database::ui_database_path().map_err(|_| Error::Storage)
                }
                ConfigSource::Fixture { registry, .. } => {
                    if !registry.is_absolute() {
                        return Err(Error::Storage);
                    }
                    Ok(registry.parent().ok_or(Error::Storage)?.join("observer.db"))
                }
            })
            .as_deref()
            .map_err(|error| *error)
    }
    /// Called only by bounded-worker lifecycle owners; the API accepts no dynamic message/args.
    pub fn terminal(
        &self,
        source: &ConfigSource,
        channel: &'static str,
        event: &'static str,
        failed: bool,
    ) {
        self.record(
            source,
            MonitoringEntry::new(
                if failed {
                    MonitoringLevel::Error
                } else {
                    MonitoringLevel::Info
                },
                channel,
                event,
                "gpui",
                event,
            ),
        );
    }
    fn record(&self, source: &ConfigSource, entry: MonitoringEntry) {
        let _ = self.record_checked(source, entry);
    }
    pub(super) fn record_checked(
        &self,
        source: &ConfigSource,
        entry: MonitoringEntry,
    ) -> RecordReceipt {
        let entry = sanitize(entry);
        {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.bytes += payload_size(&entry);
            state.entries.push_front(entry.clone());
            while state.entries.len() > ENTRY_LIMIT || state.bytes > PAYLOAD_LIMIT {
                if let Some(entry) = state.entries.pop_back() {
                    state.bytes = state.bytes.saturating_sub(payload_size(&entry));
                    state.evicted += 1;
                    if let Some((prefix, count)) = state.eviction_watch.as_mut()
                        && ccr_app::benchmark::s2::owns_event(&entry.id)
                        && entry.id.starts_with(*prefix)
                    {
                        *count += 1;
                    }
                }
            }
        }
        let mut attempted = false;
        let result = self
            .path(source)
            .and_then(|path| connection(path, true))
            .and_then(|conn| {
                attempted = true;
                LogPersistenceService::append_entry_with_connection(&conn, &entry)
                    .map_err(|_| Error::Storage)
            });
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .last_write = Some(result);
        RecordReceipt {
            attempted,
            persisted: result,
        }
    }
    fn read(
        &self,
        source: &ConfigSource,
        query: &MonitoringFeedQuery,
        active_tasks: usize,
    ) -> Report {
        #[cfg(test)]
        self.reads
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let count = query.count.unwrap_or(100).clamp(1, ENTRY_LIMIT);
        let level = query.level.map(MonitoringLevel::as_str);
        let database = self
            .path(source)
            .and_then(|path| connection(path, false))
            .and_then(|conn| {
                LogPersistenceService::query_logs_with_connection(
                    &conn,
                    level,
                    query.channel.as_deref(),
                    count,
                )
                .map_err(|_| Error::Storage)
            });
        let storage = database.as_ref().map(|_| ()).map_err(|error| *error);
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut entries = database
            .unwrap_or_default()
            .into_iter()
            .map(sanitize)
            .chain(state.entries.iter().cloned())
            .filter(|entry| {
                query.level.is_none_or(|level| entry.level == level)
                    && query
                        .channel
                        .as_ref()
                        .is_none_or(|channel| entry.channel == *channel)
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| right.timestamp.cmp(&left.timestamp));
        let mut seen = HashSet::new();
        entries.retain(|entry| seen.insert(entry.id.clone()));
        entries.truncate(count);
        let last_write = state.last_write;
        let retained_entries = state.entries.len();
        let retained_payload_bytes = state.bytes;
        let evicted_entries = state.evicted;
        drop(state);
        Report {
            entries,
            storage,
            last_write,
            metrics: Metrics {
                active_tasks,
                retained_entries,
                retained_payload_bytes,
                evicted_entries,
                process_rss_bytes: ccr_app::monitoring::current_process_rss_bytes(),
            },
        }
    }
}
fn connection(path: &Path, write: bool) -> Result<Connection, Error> {
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
    match std::fs::metadata(path) {
        Ok(meta) if meta.is_file() => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Err(Error::Missing),
        _ => return Err(Error::Storage),
    }
    let flags = if write {
        OpenFlags::SQLITE_OPEN_READ_WRITE
    } else {
        OpenFlags::SQLITE_OPEN_READ_ONLY
    };
    let conn = Connection::open_with_flags(path, flags).map_err(|_| Error::Storage)?;
    conn.busy_timeout(std::time::Duration::from_secs(2))
        .map_err(|_| Error::Storage)?;
    Ok(conn)
}
fn bounded(value: &str, limit: usize) -> String {
    let value = ccr_core::core::log_redact::redact_log_text(value);
    value
        .chars()
        .filter(|ch| !ch.is_control() || *ch == '\n' || *ch == '\t')
        .scan(0, |bytes, ch| {
            *bytes += ch.len_utf8();
            (*bytes <= limit).then_some(ch)
        })
        .collect()
}
fn sanitize(mut entry: MonitoringEntry) -> MonitoringEntry {
    entry.id = bounded(&entry.id, 128);
    entry.timestamp = bounded(&entry.timestamp, 64);
    entry.channel = bounded(&entry.channel, 128);
    entry.source = bounded(&entry.source, 128);
    entry.event_type = bounded(&entry.event_type, 128);
    entry.message = bounded(&entry.message, 8192);
    // Arbitrary legacy metadata/correlations may contain credentials; no detail UI needs them.
    entry.fields = None;
    entry.correlation_id = None;
    entry
}
fn payload_size(entry: &MonitoringEntry) -> usize {
    entry.id.len()
        + entry.timestamp.len()
        + entry.channel.len()
        + entry.source.len()
        + entry.event_type.len()
        + entry.message.len()
}
#[derive(Clone)]
pub struct Requests(Services);
impl Requests {
    pub fn new(services: &Services) -> Self {
        Self(services.clone())
    }
    pub fn refresh(
        &self,
        key: RequestKey,
        query: MonitoringFeedQuery,
    ) -> Result<TaskSubscription<Arc<Report>, Error>, AdmissionError> {
        let services = self.0.clone();
        self.0.executor.read(key.clone(), move |_, _| {
            if !matches!(key.environment, EnvironmentId::Local(_)) {
                return Err(Error::LocalOnly);
            }
            Ok(Arc::new(services.monitoring.read(
                &services.source,
                &query,
                services.executor.active_tasks(),
            )))
        })
    }
}
impl Services {
    /// Queue the actual environment-selection lifecycle event without blocking GPUI.
    pub fn record_environment_selection(&self, key: RequestKey) {
        let services = self.clone();
        let _ = self.executor.job_with_finalizer(
            key,
            |_, _| ccr_app::runtime::Completion::<(), Error>::Success(()),
            move || {
                services.monitoring.terminal(
                    &services.source,
                    "environment",
                    "environment.changed",
                    false,
                );
            },
        );
    }
}
#[cfg(test)]
pub(crate) mod tests;
