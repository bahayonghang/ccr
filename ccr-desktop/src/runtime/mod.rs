//! Native adapter: explicit sources, typed read requests, no UI-thread I/O.
pub mod benchmark;
pub mod checkin;
pub mod environments;
pub mod insights;
pub mod install;
pub mod jobs;
pub mod mcp;
pub mod monitoring;
pub mod platforms;
pub mod preferences;
pub mod profiles;
pub mod resources;
pub mod sync;
pub mod tools;
pub mod ui_state;
use std::{ffi::OsString, path::PathBuf, sync::Arc, time::Duration};

use ccr_app::{
    config::{ConfigReadError, ConfigReadService, ConfigRow},
    runtime::{
        AdmissionError, EnvironmentId, Platform, RequestKey, RuntimeHandle, TaskSubscription,
    },
};

#[derive(Clone)]
pub enum ConfigSource {
    Local,
    Benchmark(Arc<benchmark::Layout>),
    Fixture {
        profiles: PathBuf,
        registry: PathBuf,
        delay: Duration,
    },
}

impl ConfigSource {
    pub fn is_fixture(&self) -> bool {
        !matches!(self, Self::Local)
    }
    pub fn benchmark(&self) -> Option<&Arc<benchmark::Layout>> {
        match self {
            Self::Benchmark(layout) => Some(layout),
            _ => None,
        }
    }
    pub fn is_benchmark(&self) -> bool {
        self.benchmark().is_some()
    }
}

pub enum LaunchMode {
    App(ConfigSource),
    Benchmark(PathBuf),
    Probe,
    AppWithInsights {
        source: ConfigSource,
        paths: insights::InsightsPaths,
    },
}

impl LaunchMode {
    /// Explicit catalog paths. Parsing changes no environment and creates no fixture data.
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let mut args = args.into_iter();
        let mut profiles = None;
        let mut registry = None;
        let mut delay = None;
        let mut probe = false;
        let mut benchmark_root = None;
        let mut insights_usage = None;
        let mut insights_archive = None;
        let mut insights_home = None;
        while let Some(arg) = args.next() {
            match arg.to_str() {
                Some("--benchmark-fixture-root") => {
                    if benchmark_root.is_some() {
                        return Err("Benchmark root may be supplied only once");
                    }
                    benchmark_root = Some(PathBuf::from(
                        args.next().ok_or("Missing benchmark fixture root")?,
                    ));
                }
                Some("--probe") => probe = true,
                Some("--insights-usage-root") => {
                    insights_usage = Some(PathBuf::from(
                        args.next().ok_or("Missing insights usage root")?,
                    ))
                }
                Some("--insights-archive-db") => {
                    insights_archive = Some(PathBuf::from(
                        args.next().ok_or("Missing insights archive database")?,
                    ))
                }
                Some("--insights-provider-home") => {
                    insights_home = Some(PathBuf::from(
                        args.next().ok_or("Missing insights provider home")?,
                    ))
                }
                Some("--fixture-profiles") => {
                    profiles = Some(PathBuf::from(
                        args.next().ok_or("Missing fixture profiles path")?,
                    ))
                }
                Some("--fixture-registry") => {
                    registry = Some(PathBuf::from(
                        args.next().ok_or("Missing fixture registry path")?,
                    ))
                }
                Some("--fixture-read-delay-ms") => {
                    let value = args.next().ok_or("Missing fixture delay")?;
                    let value = value
                        .to_str()
                        .and_then(|value| value.parse::<u64>().ok())
                        .filter(|value| *value <= 30_000)
                        .ok_or("Fixture delay must be 0 to 30000 ms")?;
                    delay = Some(Duration::from_millis(value));
                }
                _ => {
                    return Err(
                        "Expected --probe or paired --fixture-profiles and --fixture-registry paths; optional --fixture-read-delay-ms 0..30000",
                    );
                }
            }
        }
        if let Some(root) = benchmark_root {
            if probe
                || profiles.is_some()
                || registry.is_some()
                || delay.is_some()
                || insights_usage.is_some()
                || insights_archive.is_some()
                || insights_home.is_some()
            {
                return Err(
                    "Benchmark source cannot be combined with probe, legacy fixture or insights options",
                );
            }
            if !root.is_absolute()
                || root
                    .components()
                    .any(|part| matches!(part, std::path::Component::ParentDir))
            {
                return Err("Benchmark root must be absolute without traversal");
            }
            return Ok(Self::Benchmark(root));
        }
        let insights = match (insights_usage, insights_archive, insights_home) {
            (None, None, None) => None,
            (Some(usage_root), Some(archive_db), Some(provider_home))
                if usage_root.is_absolute()
                    && archive_db.is_absolute()
                    && provider_home.is_absolute() =>
            {
                Some(insights::InsightsPaths {
                    usage_root,
                    archive_db,
                    provider_home,
                })
            }
            _ => {
                return Err(
                    "Insights requires three absolute paths: --insights-usage-root, --insights-archive-db, --insights-provider-home",
                );
            }
        };
        if probe && insights.is_some() {
            return Err("--probe cannot load insights");
        }
        let mode = match (probe, profiles, registry, delay) {
            (true, None, None, None) => Ok(Self::Probe),
            (false, None, None, None) => Ok(Self::App(ConfigSource::Local)),
            (false, Some(profiles), Some(registry), delay) => {
                Ok(Self::App(ConfigSource::Fixture {
                    profiles,
                    registry,
                    delay: delay.unwrap_or_default(),
                }))
            }
            _ => Err(
                "Fixture paths must be paired; delay requires fixture paths; --probe cannot read configuration",
            ),
        }?;
        match (mode, insights) {
            (Self::App(source), Some(paths)) => Ok(Self::AppWithInsights { source, paths }),
            (mode, None) => Ok(mode),
            _ => Err("Insights cannot be combined with probe mode"),
        }
    }
}

#[derive(Clone)]
pub struct Services {
    pub executor: RuntimeHandle,
    source: ConfigSource,
    ui_state_cache: ui_state::ManagerCache,
    insights_paths: Option<insights::InsightsPaths>,
    pub(super) platform_oauth: Arc<ccr_cli::application::codex_oauth::NativeSession>,
    pub(super) command_jobs: Arc<ccr_app::jobs::commands::CommandJobs>,
    pub(super) install: Arc<install::InstallOwner>,
    pub(super) tools: Arc<tools::Owner>,
    pub(super) monitoring: Arc<monitoring::Owner>,
    pub(super) platform_file_operations: Arc<resources::platform_files::FileOperations>,
    pub(super) codex_agent_operations: Arc<resources::codex_agents::AgentOperations>,
    pub(super) checkin_operations: Arc<checkin::CheckinOperations>,
    pub(super) sync_operations: Arc<sync::SyncOperations>,
    pub(super) usage_import: Arc<insights::import::ImportOwner>,
    pub(super) observer: Arc<insights::observer::Owner>,
    pub(super) benchmark_s2: Arc<std::sync::OnceLock<Arc<benchmark::stream::Adapter>>>,
    pub(super) ssh: Arc<std::sync::OnceLock<Arc<ccr_app::environments::ssh::Workspace>>>,
}

impl Services {
    pub fn new(executor: RuntimeHandle, source: ConfigSource) -> Self {
        Self {
            executor,
            source,
            ui_state_cache: Default::default(),
            insights_paths: None,
            platform_oauth: Default::default(),
            command_jobs: Default::default(),
            install: Default::default(),
            tools: Default::default(),
            monitoring: Default::default(),
            sync_operations: Default::default(),
            usage_import: Default::default(),
            observer: Default::default(),
            checkin_operations: Default::default(),
            platform_file_operations: Default::default(),
            codex_agent_operations: Default::default(),
            ssh: Default::default(),
            benchmark_s2: Default::default(),
        }
    }
    pub fn with_insights_paths(mut self, paths: insights::InsightsPaths) -> Self {
        self.insights_paths = Some(paths);
        self
    }
    pub fn insights_paths(&self) -> Option<insights::InsightsPaths> {
        match self.source.benchmark() {
            Some(layout) => Some(insights::InsightsPaths {
                usage_root: layout.path("llmusage"),
                archive_db: layout.path("ccr/analytics/usage.db"),
                provider_home: layout.roots.home.clone(),
            }),
            None => self.insights_paths.clone(),
        }
    }
    pub fn is_fixture(&self) -> bool {
        self.source.is_fixture()
    }
    pub fn report_clock(&self) -> ccr_app::usage::report_clock::ReportClock {
        self.source
            .benchmark()
            .map(|layout| ccr_app::usage::report_clock::ReportClock::fixed_utc(layout.as_of))
            .unwrap_or_else(ccr_app::usage::report_clock::ReportClock::live)
    }
    pub fn benchmark_clock(
        &self,
    ) -> Option<(chrono::DateTime<chrono::Utc>, ccr_usage::ReportTimezone)> {
        self.source
            .benchmark()
            .map(|layout| (layout.as_of, layout.timezone))
    }
    pub fn benchmark_title(&self) -> Option<String> {
        self.source
            .benchmark()
            .map(|layout| format!("CCR Desktop (GPUI) · Fixture {}", layout.id))
    }
    pub fn is_slow_fixture(&self) -> bool {
        matches!(self.source, ConfigSource::Fixture { delay, .. } if !delay.is_zero())
    }

    fn fixture_delay(&self) -> Duration {
        match self.source {
            ConfigSource::Fixture { delay, .. } => delay,
            ConfigSource::Local | ConfigSource::Benchmark(_) => Duration::ZERO,
        }
    }

    pub fn configurations(
        &self,
        key: RequestKey,
        filter: String,
    ) -> Result<TaskSubscription<Arc<Vec<ConfigRow>>, ConfigReadError>, AdmissionError> {
        let source = self.source.clone();
        let supported =
            matches!(key.environment, EnvironmentId::Local(_)) && key.platform == Platform::Claude;
        self.executor.read(key, move |cancel, _| {
            if !supported {
                return Err(ConfigReadError::PlatformNotSupported(
                    "native_config_unsupported_environment_or_platform".into(),
                ));
            }
            let service = match source {
                ConfigSource::Benchmark(layout) => {
                    let paths = layout.platform_paths(ccr_config::Platform::Claude)?;
                    ConfigReadService::from_paths(paths.profiles_file, paths.registry_file)
                }
                ConfigSource::Local => ConfigReadService::for_claude()?,
                ConfigSource::Fixture {
                    profiles,
                    registry,
                    delay,
                } => {
                    // Explicit native QA only. Interrupt the artificial delay on
                    // cancellation; genuine domain I/O still completes before terminal.
                    let deadline = std::time::Instant::now() + delay;
                    while !cancel.is_cancelled() && std::time::Instant::now() < deadline {
                        std::thread::sleep(
                            deadline
                                .saturating_duration_since(std::time::Instant::now())
                                .min(Duration::from_millis(10)),
                        );
                    }
                    ConfigReadService::from_paths(profiles, registry)
                }
            };
            if cancel.is_cancelled() {
                return Ok(Arc::new(Vec::new()));
            }
            let mut rows = service.list()?;
            if !filter.is_empty() {
                let filter = filter.to_lowercase();
                rows.retain(|row| row.name.to_lowercase().contains(&filter));
            }
            Ok(Arc::new(rows))
        })
    }
}

/// Never format a source error into UI/logs: parser text and paths can contain secrets.
pub fn read_error_message(error: &ConfigReadError) -> &'static str {
    match error {
        ConfigReadError::PlatformNotSupported(_) => {
            "Configuration reads for this environment or platform are not available yet. / 此环境或平台尚未接入配置读取"
        }
        ConfigReadError::ConfigMissing(_) => {
            "No configuration catalog. Create or import it with the existing CCR tools. / 配置目录尚不存在"
        }
        ConfigReadError::ConfigFormatInvalid(_) => {
            "Invalid configuration format. Check it with CCR, then retry. / 配置格式无效"
        }
        ConfigReadError::ConfigError(_) => {
            "Cannot read this catalog. It may require migration or repair with CCR. / 请先使用 CCR 检查或迁移配置"
        }
        _ => {
            "Cannot read configuration. Check file access with CCR, then retry. / 读取失败，请检查访问权限后重试"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccr_app::runtime::{Completion, RuntimeOwner, TaskEvent};
    fn parse(args: &[&str]) -> Result<LaunchMode, &'static str> {
        LaunchMode::parse(args.iter().map(OsString::from))
    }
    #[test]
    fn fixture_paths_are_explicit_and_delay_cannot_affect_production() {
        assert!(matches!(
            parse(&[]),
            Ok(LaunchMode::App(ConfigSource::Local))
        ));
        assert!(matches!(parse(&["--probe"]), Ok(LaunchMode::Probe)));
        assert!(parse(&["--fixture-profiles", "missing"]).is_err());
        assert!(parse(&["--fixture-read-delay-ms", "100"]).is_err());
        assert!(parse(&["--probe", "--fixture-read-delay-ms", "0"]).is_err());
        assert!(
            parse(&[
                "--fixture-profiles",
                "p",
                "--fixture-registry",
                "r",
                "--fixture-read-delay-ms",
                "30001"
            ])
            .is_err()
        );
        assert!(matches!(
            parse(&[
                "--fixture-profiles",
                "p",
                "--fixture-registry",
                "r",
                "--fixture-read-delay-ms",
                "30000"
            ]),
            Ok(LaunchMode::App(ConfigSource::Fixture { .. }))
        ));
    }
    #[test]
    fn insights_launch_requires_all_explicit_absolute_roots() {
        let root = tempfile::tempdir().unwrap_or_else(|_| panic!("fixture root"));
        let argument = |name: &str| root.path().join(name).into_os_string();
        assert!(
            LaunchMode::parse([OsString::from("--insights-usage-root"), argument("usage")])
                .is_err()
        );
        assert!(
            LaunchMode::parse([
                OsString::from("--insights-usage-root"),
                argument("usage"),
                OsString::from("--insights-archive-db"),
                argument("archive.db"),
                OsString::from("--insights-provider-home"),
                argument("home"),
            ])
            .is_ok_and(|mode| matches!(
                mode,
                LaunchMode::AppWithInsights {
                    source: ConfigSource::Local,
                    ..
                }
            ))
        );
        assert!(
            parse(&[
                "--insights-usage-root",
                "relative",
                "--insights-archive-db",
                "relative",
                "--insights-provider-home",
                "relative"
            ])
            .is_err()
        );
    }
    #[gpui_kit::test]
    async fn opencode_identity_cannot_dispatch_claude_configuration_or_profiles(
        cx: &mut gpui_kit::TestAppContext,
    ) {
        cx.executor().allow_parking();
        let owner = RuntimeOwner::new().unwrap_or_else(|_| panic!("runtime"));
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: "must-not-read-claude/profiles.toml".into(),
                registry: "must-not-read-claude/registry.toml".into(),
                delay: Duration::ZERO,
            },
        );
        let key = RequestKey {
            environment: EnvironmentId::Local("fixture".into()),
            platform: Platform::OpenCode,
            generation: 1,
            request_id: 1,
        };
        let mut catalog = services
            .configurations(key.clone(), String::new())
            .unwrap_or_else(|_| panic!("catalog admission"));
        assert!(
            matches!(catalog.next().await, Some(TaskEvent::Terminal(value)) if matches!(value.as_ref(), Completion::Failed(ConfigReadError::PlatformNotSupported(_))))
        );
        let mut profiles = services
            .profile_rows(key, String::new())
            .unwrap_or_else(|_| panic!("profile admission"));
        assert!(
            matches!(profiles.next().await, Some(TaskEvent::Terminal(value)) if matches!(value.as_ref(), Completion::Failed(profiles::ProfileError::Unsupported)))
        );
        owner.shutdown();
    }
    #[test]
    fn error_display_never_contains_domain_payloads() {
        for error in [
            ConfigReadError::ConfigError("credential-literal".into()),
            ConfigReadError::ConfigFormatInvalid("credential-literal".into()),
            ConfigReadError::ConfigMissing("credential-literal".into()),
        ] {
            assert!(!read_error_message(&error).contains("credential-literal"));
        }
    }

    #[gpui_kit::test]
    async fn remote_request_does_not_fall_back_to_local_data(cx: &mut gpui_kit::TestAppContext) {
        cx.executor().allow_parking();
        let owner = RuntimeOwner::new().unwrap_or_else(|error| panic!("runtime: {error}"));
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: "must-not-be-read/profiles.toml".into(),
                registry: "must-not-be-read/config.toml".into(),
                delay: Duration::ZERO,
            },
        );
        let key = RequestKey {
            environment: EnvironmentId::Ssh("fixture-remote".into()),
            platform: Platform::Claude,
            generation: 0,
            request_id: 0,
        };
        let mut read = services
            .configurations(key, String::new())
            .unwrap_or_else(|error| panic!("admission: {error:?}"));
        let event = read.next().await;
        assert!(
            matches!(event, Some(TaskEvent::Terminal(value)) if matches!(value.as_ref(), Completion::Failed(ConfigReadError::PlatformNotSupported(_))))
        );
        owner.shutdown();
    }
}
