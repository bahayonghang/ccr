use super::*;
use ccr_app::environments::io::{EnvironmentIo, Error};
use tokio_util::sync::CancellationToken;

pub struct PlatformSummary {
    pub platform: ccr_app::runtime::Platform,
    pub cli: bool,
    pub configuration: bool,
    pub session_sources: Option<usize>,
}
pub struct Overview {
    pub platforms: Vec<PlatformSummary>,
    pub usage: Result<WorkbenchReport, InsightsError>,
}

impl InsightsRequests {
    pub fn load_remote_overview(
        &self,
        key: RequestKey,
        platform: Option<ccr_app::runtime::Platform>,
        days: u16,
    ) -> Result<TaskSubscription<Arc<Overview>, InsightsError>, AdmissionError> {
        let requests = self.clone();
        let io = self.services.remote_configuration_io(&key.environment);
        self.services.executor.read(key, move |cancel, _| {
            let io = io.map_err(|_| InsightsError::Remote)?;
            let handle =
                tokio::runtime::Handle::try_current().map_err(|_| InsightsError::Remote)?;
            let cli = handle
                .block_on(io.detect_execution_cli(cancel.clone()))
                .map_err(|_| InsightsError::Remote)?;
            let sessions = requests
                .capture_remote(Ok(io.clone()), cancel.clone(), false)
                .ok();
            let statuses = sessions.as_ref().map(|s| s.registry.provider_statuses());
            let mut platforms = Vec::new();
            use ccr_app::runtime::Platform;
            for (id, config, tool, agent) in [
                (
                    Platform::Claude,
                    ".claude/settings.json",
                    "claude",
                    "claude",
                ),
                (Platform::Codex, ".codex/config.toml", "codex", "codex"),
                (Platform::Grok, ".grok/config.toml", "grok", "grok"),
                (
                    Platform::Gemini,
                    ".gemini/antigravity-cli/settings.json",
                    "gemini",
                    "antigravity",
                ),
                (
                    Platform::OpenCode,
                    ".config/opencode/opencode.json",
                    "opencode",
                    "opencode",
                ),
            ] {
                if platform.is_some_and(|platform| platform != id) {
                    continue;
                }
                let configuration = handle
                    .block_on(io.metadata(config, cancel.clone()))
                    .map_err(|_| InsightsError::Remote)?
                    .is_some();
                let session_sources = statuses
                    .as_ref()
                    .and_then(|statuses| statuses.iter().find(|s| s.agent.as_str() == agent))
                    .filter(|s| s.error_category.is_none())
                    .map(|s| s.source_count);
                platforms.push(PlatformSummary {
                    platform: id,
                    configuration,
                    cli: cli.iter().any(|cli| cli.name == tool && cli.installed),
                    session_sources,
                });
            }
            io.check_authority().map_err(|_| InsightsError::Remote)?;
            let usage = requests
                .scoped(platform)
                .remote_usage(&io, cancel.clone())
                .and_then(|(requests, _snapshot)| requests.read_workbench(days));
            io.check_authority().map_err(|_| InsightsError::Remote)?;
            Ok(Arc::new(Overview { platforms, usage }))
        })
    }
    pub(super) fn remote_cache(&self) -> Result<PathBuf, InsightsError> {
        match &self.services.source {
            super::super::ConfigSource::Local => dirs::data_local_dir()
                .map(|root| root.join("ccr/desktop-gpui/session-cache-v1"))
                .ok_or(InsightsError::Remote),
            super::super::ConfigSource::Fixture { registry, .. } => registry
                .parent()
                .map(|root| root.join("session-cache-v1"))
                .ok_or(InsightsError::Remote),
            _ => Err(InsightsError::Remote),
        }
    }
    pub(super) fn capture_remote(
        &self,
        io: Result<EnvironmentIo, Error>,
        cancel: CancellationToken,
        refresh: bool,
    ) -> Result<Arc<sessions::remote::SessionSnapshot>, InsightsError> {
        let io = io.map_err(|_| InsightsError::Remote)?;
        io.check_authority().map_err(|_| InsightsError::Remote)?;
        let mut cached = self.remote.lock().map_err(|_| InsightsError::Remote)?;
        if !refresh
            && let Some(snapshot) = cached.as_ref()
            && snapshot.io().environment() == io.environment()
            && snapshot.io().check_authority().is_ok()
        {
            return Ok(snapshot.clone());
        }
        *cached = None;
        let handle = tokio::runtime::Handle::try_current().map_err(|_| InsightsError::Remote)?;
        let snapshot = handle
            .block_on(sessions::remote::SessionSnapshot::capture(
                io,
                &self.remote_cache()?,
                cancel.clone(),
            ))
            .map_err(|_| InsightsError::Remote)?;
        snapshot
            .io()
            .check_authority()
            .map_err(|_| InsightsError::Remote)?;
        if cancel.is_cancelled() || self.environment() != *snapshot.io().environment() {
            return Err(InsightsError::Remote);
        }
        let snapshot = Arc::new(snapshot);
        *cached = Some(snapshot.clone());
        Ok(snapshot)
    }
    pub(super) fn read_remote(
        &self,
        io: Result<EnvironmentIo, Error>,
        query: InsightsQuery,
        cancel: CancellationToken,
    ) -> Result<InsightsSnapshot, InsightsError> {
        if !matches!(
            query,
            InsightsQuery::Sessions(_) | InsightsQuery::Detail { .. }
        ) {
            let io = io.map_err(|_| InsightsError::Remote)?;
            let (requests, _snapshot) = self.remote_usage(&io, cancel.clone())?;
            let result = match query {
                InsightsQuery::Home {
                    as_of,
                    generated_at,
                } => {
                    if self.services.platform_scope.is_some() {
                        return Err(InsightsError::Unsupported);
                    }
                    let paths = requests.paths.as_ref().ok_or(InsightsError::Remote)?;
                    let dashboard =
                        ccr_usage::open_dashboard(AppPaths::from_root(&paths.usage_root))?;
                    let projected = dashboard.insights(
                        &QueryFilter {
                            timezone: self.report_clock().timezone,
                            ..Default::default()
                        },
                        as_of,
                    )?;
                    Ok(InsightsSnapshot::Home(merge_home_insights(
                        projected,
                        InsightsSessionCounts::default(),
                        generated_at,
                    )))
                }
                query => requests.read(query),
            };
            io.check_authority().map_err(|_| InsightsError::Remote)?;
            if cancel.is_cancelled() {
                return Err(InsightsError::Remote);
            }
            return result;
        }
        let snapshot = self.capture_remote(io, cancel, false)?;
        match query {
            InsightsQuery::Sessions(request) => {
                if let Some(platform) = self.services.platform_scope {
                    let expected = match platform {
                        ccr_app::runtime::Platform::Claude => {
                            ccr_store::sessions::AgentSessionAgentId::Claude
                        }
                        ccr_app::runtime::Platform::Codex => {
                            ccr_store::sessions::AgentSessionAgentId::Codex
                        }
                        _ => return Err(InsightsError::Unsupported),
                    };
                    if request.agents.as_deref() != Some(&[expected]) {
                        return Err(InsightsError::Unsupported);
                    }
                }
                let page = sessions::list_sessions(&snapshot.pool, request)
                    .map_err(InsightsError::Session)?;
                Ok(InsightsSnapshot::Sessions {
                    page,
                    statuses: snapshot.registry.provider_statuses(),
                    indexed: true,
                })
            }
            InsightsQuery::Detail { archive_id, before } => {
                let (source, _) = sessions::resolve_source_with_registry(
                    &snapshot.pool,
                    &snapshot.registry,
                    &archive_id,
                )
                .map_err(InsightsError::Session)?;
                snapshot
                    .verify_source(&source.physical_path)
                    .map_err(|_| InsightsError::Remote)?;
                let detail = sessions::get_detail_with_registry(
                    &snapshot.pool,
                    &snapshot.registry,
                    &archive_id,
                    before,
                    Some(100),
                )
                .map_err(InsightsError::Session)?;
                if let Some(platform) = self.services.platform_scope
                    && platform.short_name() != detail.agent.as_str()
                {
                    return Err(InsightsError::Unsupported);
                }
                Ok(InsightsSnapshot::Detail(detail))
            }
            _ => Err(InsightsError::Remote),
        }
    }

    pub(super) fn remote_usage(
        &self,
        io: &EnvironmentIo,
        cancel: CancellationToken,
    ) -> Result<(Self, ccr_app::usage::import::remote::Snapshot), InsightsError> {
        let handle = tokio::runtime::Handle::try_current().map_err(|_| InsightsError::Remote)?;
        let snapshot = handle
            .block_on(ccr_app::usage::import::remote::Snapshot::capture(
                io,
                &self.remote_cache()?.join("usage"),
                cancel,
            ))
            .map_err(|error| match error {
                ccr_app::usage::import::error::LlmusageAdapterError::DbMissing(_) => {
                    InsightsError::Missing
                }
                ccr_app::usage::import::error::LlmusageAdapterError::SchemaUnsupported {
                    ..
                }
                | ccr_app::usage::import::error::LlmusageAdapterError::FeatureUnavailable {
                    ..
                } => InsightsError::Unsupported,
                _ => InsightsError::RemoteSnapshot,
            })?;
        let root = snapshot.paths().root_dir;
        let mut requests = self.clone();
        // 远端报告的归档/Observer不可借用本机路径；无会话索引保持 unknown。
        requests.paths = Some(InsightsPaths {
            usage_root: root.clone(),
            archive_db: root.join("unavailable-archive.db"),
            provider_home: root,
        });
        requests.archive = Default::default();
        Ok((requests, snapshot))
    }
}
