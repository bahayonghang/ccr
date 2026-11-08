//! Typed, explicit-path insights reads through the application-owned bounded executor.
pub mod capabilities;
pub mod import;
pub mod observer;
pub mod policies;
pub mod session_operations;
use super::Services;
use ccr_app::{
    runtime::{AdmissionError, Completion, EnvironmentId, RequestKey, TaskSubscription},
    sessions::{self, SessionDetail, SessionError, SessionListRequest, SessionPage},
    usage::{
        home::{
            HomeInsightsSnapshot, InsightsSessionCounts, assemble_home_insights,
            merge_home_insights,
        },
        read::{UsageSnapshot, read_dashboard_at},
        report_clock::ReportClock,
    },
};
use ccr_db::database::{self, DbPool};
use ccr_store::sessions::{AgentSessionProviderRegistry, AgentSessionProviderStatus};
use ccr_usage::{
    AppPaths, HomeOverviewPlatformStats, HomeOverviewSeriesItem, LogsPage, LogsQuery, QueryFilter,
    ReportTimezone, SourceKind, UsageError,
};
use chrono::{DateTime, NaiveDate, Utc};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// Explicit roots captured before the window opens. Production resolves the installed
/// llmusage and CCR archive files once; it does not create them. Fixture launches stay empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InsightsPaths {
    pub usage_root: PathBuf,
    pub archive_db: PathBuf,
    pub provider_home: PathBuf,
}

/// Resolve the same llmusage and CCR archive locations the installed tools already use.
/// Capture paths even before the first import. This function does not create directories
/// or databases, and opening the page never starts an import.
pub fn local_insights_paths() -> Option<InsightsPaths> {
    let usage = ccr_usage::AppPaths::discover().ok()?;
    let home = std::env::var_os("USERPROFILE")
        .filter(|value| !value.is_empty())
        .or_else(|| std::env::var_os("HOME").filter(|value| !value.is_empty()))
        .map(PathBuf::from)?;
    let ccr_root = std::env::var_os("CCR_DATA_DIR")
        .filter(|value| !value.is_empty())
        .or_else(|| std::env::var_os("CCR_ROOT").filter(|value| !value.is_empty()))
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".ccr"));
    Some(InsightsPaths {
        usage_root: usage.root_dir,
        archive_db: ccr_root.join("analytics").join("usage.db"),
        provider_home: home,
    })
}

#[cfg(test)]
pub(crate) fn insights_paths_if_present(
    usage_root: PathBuf,
    usage_db: &Path,
    archive_db: PathBuf,
    provider_home: PathBuf,
) -> Option<InsightsPaths> {
    // Either existing file is enough. The missing one stays unread and is not created.
    if usage_db.is_file() || archive_db.is_file() {
        Some(InsightsPaths {
            usage_root,
            archive_db,
            provider_home,
        })
    } else {
        None
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsightsError {
    Unconfigured,
    LocalOnly,
    Missing,
    Unreadable,
    Unsupported,
    Query,
    Archive,
    Session(SessionError),
}
impl InsightsError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Unconfigured => {
                "Select explicit usage, archive and provider roots with the insights launch options. / 请通过启动参数指定用量、归档和会话根目录"
            }
            Self::LocalOnly => "Insights are local-only. / 观测仅支持本地环境",
            Self::Missing => {
                "No llmusage database at the supplied root. Sync with existing tools, then retry. / 指定目录尚无用量数据库，请通过现有工具同步后重试"
            }
            Self::Unreadable => {
                "Cannot read the usage database. Check file access. / 无法读取用量数据库，请检查访问权限"
            }
            Self::Unsupported => {
                "This database cannot support the selected report or provider filter. / 当前数据库不支持所选报告或提供商筛选"
            }
            Self::Query => {
                "Invalid filters or failed usage query. Check the reporting dates. / 筛选无效或用量查询失败，请检查报告日期"
            }
            Self::Archive => {
                "Cannot read the supplied session archive. Initialize it with existing CCR tools. / 无法读取指定会话归档，请先通过现有 CCR 工具初始化"
            }
            Self::Session(SessionError::SourceUnavailable) => {
                "The original session is unavailable. Its indexed summary remains. / 原始会话已不可用，归档摘要仍保留"
            }
            Self::Session(
                SessionError::SourceValidation
                | SessionError::UnknownSourceKind
                | SessionError::UnknownProvider,
            ) => {
                "The session source failed provider validation. No transcript was opened. / 会话来源未通过提供方验证，未打开正文"
            }
            Self::Session(SessionError::NotFound) => {
                "This archived session no longer exists. Refresh the list. / 归档会话已不存在，请刷新列表"
            }
            Self::Session(_) => {
                "Cannot read sessions. Check the filters and archive. / 会话读取失败，请检查筛选与归档"
            }
        }
    }
}
impl From<UsageError> for InsightsError {
    fn from(error: UsageError) -> Self {
        match error {
            UsageError::DbMissing(_) => Self::Missing,
            UsageError::DbUnreadable(_) => Self::Unreadable,
            UsageError::SchemaUnsupported { .. } | UsageError::FeatureUnavailable { .. } => {
                Self::Unsupported
            }
            UsageError::Query(_) => Self::Query,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionScope {
    Window,
    AllHistory,
    Unindexed,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkbenchReport {
    pub as_of: String,
    pub window_days: u16,
    pub requests: i64,
    pub tokens: i64,
    pub cost_usd: Option<f64>,
    pub sessions_indexed: bool,
    pub session_count: Option<i64>,
    pub session_scope: SessionScope,
    pub other_requests: i64,
    pub platforms: Vec<WorkbenchPlatform>,
    pub days: Vec<WorkbenchDay>,
    /// Raw RFC3339 finish time of the last successful sync run.
    pub last_sync_at: Option<String>,
    /// `last_sync_at` in the report timezone, computed here so rendering stays pure.
    pub freshness: SyncFreshness,
    /// All-history activity; `None` only when the usage projection cannot be read.
    pub activity: Option<HomeInsightsSnapshot>,
}

/// How current the imported usage is, in the report timezone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncFreshness {
    /// `MM-DD HH:mm`; `None` when no readable sync time is recorded.
    pub local: Option<String>,
    /// `MM-DD` of the last sync, the last day the report can contain.
    pub date: Option<String>,
    /// The last sync's local date precedes the report date, or no sync is known.
    pub stale: bool,
}

impl SyncFreshness {
    /// Convert a stored sync time for one report clock. A value that is not
    /// RFC3339 has no known instant, so it carries no time and counts as stale.
    pub fn at(last_sync_at: Option<&str>, clock: ReportClock) -> Self {
        let Some(instant) = last_sync_at.and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
        else {
            return Self {
                local: None,
                date: None,
                stale: true,
            };
        };
        let local = match clock.timezone {
            ReportTimezone::Utc => instant.naive_utc(),
            ReportTimezone::Local => instant.with_timezone(&chrono::Local).naive_local(),
        };
        Self {
            local: Some(local.format("%m-%d %H:%M").to_string()),
            date: Some(local.format("%m-%d").to_string()),
            stale: local.date() < clock.date(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkbenchPlatform {
    pub id: String,
    pub requests: i64,
    pub tokens: i64,
    pub requests_by_day: Vec<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkbenchDay {
    pub label: String,
    pub claude: i64,
    pub codex: i64,
    pub antigravity: i64,
    pub opencode: i64,
}

fn platform_stats(
    map: &BTreeMap<String, HomeOverviewPlatformStats>,
    key: &str,
) -> HomeOverviewPlatformStats {
    map.get(key).cloned().unwrap_or_default()
}

fn dense_days(
    series: &[HomeOverviewSeriesItem],
    since: NaiveDate,
    until: NaiveDate,
) -> Vec<WorkbenchDay> {
    let mut cursor = since;
    let mut days = Vec::new();
    loop {
        let key = cursor.format("%Y-%m-%d").to_string();
        let found = series.iter().find(|item| item.date == key);
        days.push(WorkbenchDay {
            label: key.get(5..10).unwrap_or(&key).to_string(),
            claude: found.map(|item| item.claude.requests).unwrap_or(0),
            codex: found.map(|item| item.codex.requests).unwrap_or(0),
            antigravity: found.map(|item| item.antigravity.requests).unwrap_or(0),
            opencode: found.map(|item| item.opencode.requests).unwrap_or(0),
        });
        if cursor >= until || days.len() == 366 {
            break;
        }
        let Some(next) = cursor.succ_opt() else {
            break;
        };
        cursor = next;
    }
    days
}

fn day_column(days: &[WorkbenchDay], pick: fn(&WorkbenchDay) -> i64) -> Vec<i64> {
    days.iter().map(pick).collect()
}

fn canonical_source_id(raw: &str) -> String {
    SourceKind::parse_id(raw)
        .map(|kind| kind.as_str().to_string())
        .unwrap_or_else(|| raw.to_string())
}

fn fold_platform_stats(
    map: &BTreeMap<String, HomeOverviewPlatformStats>,
) -> BTreeMap<String, HomeOverviewPlatformStats> {
    let mut folded: BTreeMap<String, HomeOverviewPlatformStats> = BTreeMap::new();
    for (id, stats) in map {
        let entry = folded.entry(canonical_source_id(id)).or_default();
        entry.requests += stats.requests;
        entry.tokens += stats.tokens;
        entry.sessions += stats.sessions;
    }
    folded
}

fn workbench_platforms(
    map: &BTreeMap<String, HomeOverviewPlatformStats>,
    days: &[WorkbenchDay],
) -> Vec<WorkbenchPlatform> {
    const PRIMARY: [&str; 4] = ["claude", "codex", "antigravity", "opencode"];
    let map = fold_platform_stats(map);
    let mut platforms = Vec::new();
    for id in PRIMARY {
        let stats = platform_stats(&map, id);
        let requests_by_day = match id {
            "claude" => day_column(days, |day| day.claude),
            "codex" => day_column(days, |day| day.codex),
            "antigravity" => day_column(days, |day| day.antigravity),
            _ => day_column(days, |day| day.opencode),
        };
        // The fixed daily series only contains stored claude/codex/antigravity/opencode
        // keys. Alias totals stay on the card and are left out of a zero sparkline.
        let requests_by_day = if requests_by_day.iter().sum::<i64>() == stats.requests {
            requests_by_day
        } else {
            Vec::new()
        };
        platforms.push(WorkbenchPlatform {
            id: id.to_string(),
            requests: stats.requests,
            tokens: stats.tokens,
            requests_by_day,
        });
    }
    let mut extras: Vec<_> = map
        .iter()
        .filter(|(id, stats)| {
            !PRIMARY.contains(&id.as_str()) && (stats.requests > 0 || stats.tokens > 0)
        })
        .collect();
    extras.sort_by(|left, right| left.0.cmp(right.0));
    for (id, stats) in extras {
        platforms.push(WorkbenchPlatform {
            id: id.clone(),
            requests: stats.requests,
            tokens: stats.tokens,
            requests_by_day: Vec::new(),
        });
    }
    platforms
}

pub enum InsightsQuery {
    Home {
        as_of: NaiveDate,
        generated_at: DateTime<Utc>,
    },
    Usage(QueryFilter),
    Logs {
        filter: QueryFilter,
        cursor: Option<String>,
    },
    Sessions(SessionListRequest),
    Detail {
        archive_id: String,
        before: Option<u32>,
    },
}
pub enum InsightsSnapshot {
    Home(HomeInsightsSnapshot),
    Usage(UsageSnapshot),
    Logs(LogsPage),
    Sessions {
        page: SessionPage,
        statuses: Vec<AgentSessionProviderStatus>,
        /// False means the archive has no indexed summaries. An empty page is not a measured zero.
        indexed: bool,
    },
    Detail(SessionDetail),
}

#[derive(Clone, Debug)]
pub enum RefreshStatus {
    Idle,
    Running(u64),
    Finished(u64, Arc<sessions::refresh::RefreshReport>),
    Failed(u64, InsightsError),
    /// The queued/started runner did not publish a result. No success/cancellation is inferred.
    Interrupted(u64),
}
struct RefreshRun {
    status: Arc<Mutex<RefreshStatus>>,
    request_id: u64,
}
impl Drop for RefreshRun {
    fn drop(&mut self) {
        let mut status = self
            .status
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if matches!(*status, RefreshStatus::Running(id) if id == self.request_id) {
            *status = RefreshStatus::Interrupted(self.request_id);
        }
    }
}
#[derive(Clone)]
pub struct InsightsRequests {
    services: Services,
    paths: Option<InsightsPaths>,
    archive: Arc<Mutex<Option<DbPool>>>,
    refresh: Arc<Mutex<RefreshStatus>>,
    session_operations: Arc<session_operations::Owner>,
}
impl InsightsRequests {
    /// Capture configuration only. No filesystem work until admitted on a worker.
    pub fn new(services: &Services) -> Self {
        Self {
            services: services.clone(),
            paths: services.insights_paths(),
            archive: Default::default(),
            refresh: Arc::new(Mutex::new(RefreshStatus::Idle)),
            session_operations: Default::default(),
        }
    }
    pub fn report_clock(&self) -> ccr_app::usage::report_clock::ReportClock {
        self.services.report_clock()
    }
    pub fn configured(&self) -> bool {
        self.paths.is_some()
    }
    pub fn refresh_status(&self) -> RefreshStatus {
        self.refresh
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
    /// Refresh mutates only CCR's archive. The app owns the runner after view detachment.
    /// Existing archive refresh has no cancellation API; do not invent a cancelled terminal.
    pub fn refresh_archive(
        &self,
        key: RequestKey,
    ) -> Result<
        Option<TaskSubscription<Arc<sessions::refresh::RefreshReport>, InsightsError>>,
        AdmissionError,
    > {
        let mut status = self
            .refresh
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if matches!(*status, RefreshStatus::Running(_)) {
            return Ok(None);
        }
        let request_id = key.request_id;
        *status = RefreshStatus::Running(request_id);
        let run = RefreshRun {
            status: self.refresh.clone(),
            request_id,
        };
        drop(status);
        let requests = self.clone();
        let local = matches!(key.environment, EnvironmentId::Local(_));
        self.services
            .executor
            .job(key, move |_, _| {
                let result = (|| {
                    if !local {
                        return Err(InsightsError::LocalOnly);
                    }
                    let paths = requests.paths.as_ref().ok_or(InsightsError::Unconfigured)?;
                    let pool = requests.archive_pool(paths)?;
                    let registry = AgentSessionProviderRegistry::new(paths.provider_home.clone())
                        .without_environment_overrides();
                    sessions::refresh::refresh_archive_with_registry(&pool, &registry)
                        .map(Arc::new)
                        .map_err(|_| InsightsError::Archive)
                })();
                let mut status = run
                    .status
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                *status = match &result {
                    Ok(report) => RefreshStatus::Finished(request_id, report.clone()),
                    Err(error) => RefreshStatus::Failed(request_id, *error),
                };
                drop(status);
                match result {
                    Ok(report) => Completion::Success(report),
                    Err(error) => Completion::Failed(error),
                }
            })
            .map(Some)
    }
    fn read_sessions(
        &self,
        paths: &InsightsPaths,
        request: sessions::SessionListRequest,
    ) -> Result<InsightsSnapshot, InsightsError> {
        let registry = AgentSessionProviderRegistry::new(paths.provider_home.clone())
            .without_environment_overrides();
        let statuses = registry.provider_statuses();
        let empty = SessionPage {
            items: Vec::new(),
            next_cursor: None,
        };
        if !paths.archive_db.is_file() {
            return Ok(InsightsSnapshot::Sessions {
                page: empty,
                statuses,
                indexed: false,
            });
        }
        let pool = self.archive_pool(paths)?;
        let indexed = {
            let conn = pool.get().map_err(|_| InsightsError::Archive)?;
            database::repositories::usage_repo::has_any_session_archive(&conn)
                .map_err(|_| InsightsError::Archive)?
        };
        if !indexed {
            return Ok(InsightsSnapshot::Sessions {
                page: empty,
                statuses,
                indexed: false,
            });
        }
        let page = sessions::list_sessions(&pool, request).map_err(InsightsError::Session)?;
        Ok(InsightsSnapshot::Sessions {
            page,
            statuses,
            indexed: true,
        })
    }

    fn archive_pool(&self, paths: &InsightsPaths) -> Result<DbPool, InsightsError> {
        let mut cached = self.archive.lock().map_err(|_| InsightsError::Archive)?;
        if let Some(pool) = cached.as_ref() {
            return Ok(pool.clone());
        }
        // 不初始化、不迁移、不发现旧存储。只消费调用方已有的归档。
        if !paths.archive_db.is_file() {
            return Err(InsightsError::Archive);
        }
        let pool =
            database::create_pool(&paths.archive_db, None).map_err(|_| InsightsError::Archive)?;
        *cached = Some(pool.clone());
        Ok(pool)
    }
    fn read(&self, query: InsightsQuery) -> Result<InsightsSnapshot, InsightsError> {
        let paths = self.paths.as_ref().ok_or(InsightsError::Unconfigured)?;
        match query {
            InsightsQuery::Sessions(request) => self.read_sessions(paths, request),
            InsightsQuery::Detail { archive_id, before } => {
                let pool = self.archive_pool(paths)?;
                let registry = AgentSessionProviderRegistry::new(paths.provider_home.clone())
                    .without_environment_overrides();
                sessions::get_detail_with_registry(&pool, &registry, &archive_id, before, Some(100))
                    .map(InsightsSnapshot::Detail)
                    .map_err(InsightsError::Session)
            }
            query => {
                let dashboard =
                    ccr_usage::open_dashboard(AppPaths::from_root(paths.usage_root.clone()))?;
                match query {
                    InsightsQuery::Home {
                        as_of,
                        generated_at,
                    } => {
                        let projected = dashboard.insights(
                            &QueryFilter {
                                timezone: self.report_clock().timezone,
                                ..Default::default()
                            },
                            as_of,
                        )?;
                        assemble_home_insights(projected, &self.archive_pool(paths)?, generated_at)
                            .map(InsightsSnapshot::Home)
                            .map_err(|_| InsightsError::Archive)
                    }
                    InsightsQuery::Usage(filter) => {
                        // Cohort: successful report body after DB open. Excludes
                        // admission/queue, open_dashboard, failures, join and UI.
                        let as_of = self.services.benchmark_clock().map(|(as_of, _)| as_of);
                        let started = std::time::Instant::now();
                        let report = read_dashboard_at(&dashboard, &filter, Some(366), as_of);
                        let elapsed = started.elapsed();
                        if report.is_ok() {
                            self.services
                                .usage_report_latency
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner)
                                .record(elapsed);
                        }
                        report.map(InsightsSnapshot::Usage).map_err(Into::into)
                    }
                    InsightsQuery::Logs { filter, cursor } => dashboard
                        .logs(&LogsQuery {
                            filter,
                            cursor,
                            page_size: 100,
                            include_total: true,
                            include_raw_json: false,
                        })
                        .map(InsightsSnapshot::Logs)
                        .map_err(Into::into),
                    _ => unreachable!("session reads returned before opening usage"),
                }
            }
        }
    }

    pub fn load_workbench(
        &self,
        key: RequestKey,
        days: u16,
    ) -> Result<TaskSubscription<Arc<WorkbenchReport>, InsightsError>, AdmissionError> {
        let requests = self.clone();
        let local = matches!(key.environment, EnvironmentId::Local(_));
        self.services.executor.read(key, move |_, _| {
            if !local {
                return Err(InsightsError::LocalOnly);
            }
            requests.read_workbench(days).map(Arc::new)
        })
    }

    fn read_workbench(&self, days: u16) -> Result<WorkbenchReport, InsightsError> {
        let paths = self.paths.as_ref().ok_or(InsightsError::Unconfigured)?;
        let dashboard = ccr_usage::open_dashboard(AppPaths::from_root(paths.usage_root.clone()))?;
        let days = days.clamp(1, 366);
        let clock = self.report_clock();
        let as_of = clock.date();
        let span = i64::from(days.saturating_sub(1));
        let since = as_of
            .checked_sub_signed(chrono::Duration::days(span))
            .unwrap_or(as_of);
        let filter = QueryFilter {
            timezone: clock.timezone,
            since: Some(since),
            until: Some(as_of),
            ..QueryFilter::default()
        };
        let home = dashboard.home_overview(&filter)?;
        let overview = if self.services.benchmark_clock().is_some() {
            dashboard.overview_at(&filter, clock.as_of)
        } else {
            dashboard.overview(&filter)
        }
        .ok();
        let cost_usd = overview
            .as_ref()
            .map(|payload| payload.total_cost_usd)
            .filter(|cost| cost.is_finite());
        let last_sync_at = overview.and_then(|payload| payload.last_sync_at);
        let freshness = SyncFreshness::at(last_sync_at.as_deref(), clock);
        let (activity, sessions_indexed, session_count, session_scope) =
            self.activity_window(paths, &dashboard, clock, days);
        let series = dense_days(&home.series, since, as_of);
        let drawn_requests: i64 = series
            .iter()
            .map(|day| day.claude + day.codex + day.antigravity + day.opencode)
            .sum();
        Ok(WorkbenchReport {
            as_of: as_of.format("%Y-%m-%d").to_string(),
            window_days: days,
            requests: home.summary.total_requests,
            tokens: home.summary.total_tokens,
            cost_usd,
            sessions_indexed,
            session_count,
            session_scope,
            other_requests: home.summary.total_requests.saturating_sub(drawn_requests),
            platforms: workbench_platforms(&home.by_platform, &series),
            days: series,
            last_sync_at,
            freshness,
            activity,
        })
    }

    /// All-history activity plus the session reading for the selected window.
    /// Each source degrades alone: without the usage projection there is no
    /// activity, and without the archive the usage half stays while sessions
    /// read as unavailable.
    fn activity_window(
        &self,
        paths: &InsightsPaths,
        dashboard: &ccr_usage::Dashboard,
        clock: ReportClock,
        days: u16,
    ) -> (
        Option<HomeInsightsSnapshot>,
        bool,
        Option<i64>,
        SessionScope,
    ) {
        let Ok(projected) = dashboard.insights(
            &QueryFilter {
                timezone: clock.timezone,
                ..Default::default()
            },
            clock.date(),
        ) else {
            return (None, false, None, SessionScope::Unavailable);
        };
        // assemble 会消费投影；留一份给归档读取失败时的纯用量快照。
        let assembled = self
            .archive_pool(paths)
            .ok()
            .and_then(|pool| assemble_home_insights(projected.clone(), &pool, clock.as_of).ok());
        let Some(snapshot) = assembled else {
            let usage_only =
                merge_home_insights(projected, InsightsSessionCounts::default(), clock.as_of);
            return (Some(usage_only), false, None, SessionScope::Unavailable);
        };
        if !snapshot.sessions_indexed {
            return (Some(snapshot), false, None, SessionScope::Unindexed);
        }
        let (count, scope) = if days == 7 {
            (snapshot.last_7_days.sessions, SessionScope::Window)
        } else {
            (snapshot.totals.sessions, SessionScope::AllHistory)
        };
        (Some(snapshot), true, Some(count), scope)
    }

    pub fn load(
        &self,
        key: RequestKey,
        query: InsightsQuery,
    ) -> Result<TaskSubscription<Arc<InsightsSnapshot>, InsightsError>, AdmissionError> {
        let requests = self.clone();
        let local = matches!(key.environment, EnvironmentId::Local(_));
        self.services.executor.read(key, move |_, _| {
            if !local {
                return Err(InsightsError::LocalOnly);
            }
            requests.read(query).map(Arc::new)
        })
    }
}

#[cfg(test)]
mod tests;
