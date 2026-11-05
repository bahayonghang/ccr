//! Typed, explicit-path insights reads through the application-owned bounded executor.
pub mod policies;
use super::Services;
use ccr_app::{
    runtime::{AdmissionError, Completion, EnvironmentId, RequestKey, TaskSubscription},
    sessions::{self, SessionDetail, SessionError, SessionListRequest, SessionPage},
    usage::{
        home::{HomeInsightsSnapshot, assemble_home_insights},
        read::{UsageSnapshot, read_dashboard},
    },
};
use ccr_db::database::{self, DbPool};
use ccr_store::sessions::{AgentSessionProviderRegistry, AgentSessionProviderStatus};
use ccr_usage::{AppPaths, LogsPage, LogsQuery, QueryFilter, UsageError};
use chrono::{DateTime, NaiveDate, Utc};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// All three roots are explicit. No default-home/provider environment discovery.
#[derive(Clone, Debug)]
pub struct InsightsPaths {
    pub usage_root: PathBuf,
    pub archive_db: PathBuf,
    pub provider_home: PathBuf,
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
}
impl InsightsRequests {
    /// Capture configuration only. No filesystem work until admitted on a worker.
    pub fn new(services: &Services) -> Self {
        Self {
            services: services.clone(),
            paths: services.insights_paths(),
            archive: Default::default(),
            refresh: Arc::new(Mutex::new(RefreshStatus::Idle)),
        }
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
            InsightsQuery::Sessions(request) => {
                let pool = self.archive_pool(paths)?;
                let page =
                    sessions::list_sessions(&pool, request).map_err(InsightsError::Session)?;
                let registry = AgentSessionProviderRegistry::new(paths.provider_home.clone())
                    .without_environment_overrides();
                Ok(InsightsSnapshot::Sessions {
                    page,
                    statuses: registry.provider_statuses(),
                })
            }
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
                        let projected = dashboard.insights(&QueryFilter::default(), as_of)?;
                        assemble_home_insights(projected, &self.archive_pool(paths)?, generated_at)
                            .map(InsightsSnapshot::Home)
                            .map_err(|_| InsightsError::Archive)
                    }
                    InsightsQuery::Usage(filter) => read_dashboard(&dashboard, &filter, Some(366))
                        .map(InsightsSnapshot::Usage)
                        .map_err(Into::into),
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
