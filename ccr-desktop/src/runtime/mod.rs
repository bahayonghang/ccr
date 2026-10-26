//! Native adapter: explicit sources, typed read requests, no UI-thread I/O.
pub mod profiles;
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
    Fixture {
        profiles: PathBuf,
        registry: PathBuf,
        delay: Duration,
    },
}

pub enum LaunchMode {
    App(ConfigSource),
    Probe,
}

impl LaunchMode {
    /// Explicit catalog paths. Parsing changes no environment and creates no fixture data.
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, &'static str> {
        let mut args = args.into_iter();
        let mut profiles = None;
        let mut registry = None;
        let mut delay = None;
        let mut probe = false;
        while let Some(arg) = args.next() {
            match arg.to_str() {
                Some("--probe") => probe = true,
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
        match (probe, profiles, registry, delay) {
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
        }
    }
}

#[derive(Clone)]
pub struct Services {
    pub executor: RuntimeHandle,
    source: ConfigSource,
    ui_state_cache: ui_state::ManagerCache,
}

impl Services {
    pub fn new(executor: RuntimeHandle, source: ConfigSource) -> Self {
        Self {
            executor,
            source,
            ui_state_cache: Default::default(),
        }
    }
    pub fn is_fixture(&self) -> bool {
        matches!(self.source, ConfigSource::Fixture { .. })
    }
    pub fn is_slow_fixture(&self) -> bool {
        matches!(self.source, ConfigSource::Fixture { delay, .. } if !delay.is_zero())
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
