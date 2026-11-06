//! Read-only Grok activation facts from the same owner as Tauri's dashboard.
use super::*;
use ccr_cli::platforms::{GrokActivationState, GrokPlatform, GrokProfileAuthMode};
use ccr_config::PlatformConfig;

#[derive(Clone, Debug)]
pub struct GrokOverview {
    pub activation: GrokActivationState,
    pub auth_mode: Option<GrokProfileAuthMode>,
    pub profiles_total: usize,
    pub profiles_enabled: usize,
    pub config_exists: bool,
}

fn read(source: &ConfigSource) -> std::result::Result<GrokOverview, ProfileError> {
    let context = context(source, Platform::Grok).map_err(safe_error)?;
    context
        .run(|| {
            let platform = context.grok_platform()?;
            let profiles = platform.load_profiles()?;
            let activation = platform.inspect_activation_state()?;
            let name = match &activation {
                GrokActivationState::Inactive => None,
                GrokActivationState::Active { name } | GrokActivationState::Drifted { name } => {
                    Some(name)
                }
                GrokActivationState::UnsafeMissingEntryState { name } => name.as_ref(),
            };
            let auth_mode = name
                .and_then(|name| profiles.get(name))
                .map(GrokPlatform::profile_auth_mode)
                .transpose()?;
            let config_exists = match std::fs::metadata(platform.get_settings_path()) {
                Ok(metadata) if metadata.is_file() => true,
                Ok(_) => {
                    return Err(CcrError::ConfigError(
                        "Invalid Grok configuration source".into(),
                    ));
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                Err(error) => return Err(error.into()),
            };
            Ok(GrokOverview {
                activation,
                auth_mode,
                profiles_total: profiles.len(),
                profiles_enabled: profiles
                    .values()
                    .filter(|profile| profile.is_enabled())
                    .count(),
                config_exists,
            })
        })
        .map_err(safe_error)
}

impl Services {
    pub fn grok_overview(
        &self,
        key: RequestKey,
    ) -> std::result::Result<TaskSubscription<GrokOverview, ProfileError>, AdmissionError> {
        let supported = matches!(key.environment, EnvironmentId::Local(_))
            && key.platform == RequestPlatform::Grok;
        let source = self.source.clone();
        self.executor.read(key, move |_, _| {
            if !supported {
                return Err(ProfileError::Unsupported);
            }
            read(&source)
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn grok_overview_distinguishes_activation_drift_and_missing_baseline_without_repair() {
        let root = tempfile::tempdir().unwrap();
        let source = ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: Duration::ZERO,
        };
        let context = context(&source, Platform::Grok).unwrap();
        let platform = context.grok_platform().unwrap();
        context
            .run(|| {
                let mut config = ccr_config::CcsConfig {
                    default_config: "relay".into(),
                    current_config: String::new(),
                    settings: Default::default(),
                    sections: Default::default(),
                };
                config.sections.insert(
                    "relay".into(),
                    ccr_config::ConfigSection {
                        base_url: Some("https://fixture.invalid".into()),
                        auth_token: Some(ccr_core::Secret::new("synthetic-overview-secret")),
                        model: Some("fixture-model".into()),
                        enabled: Some(true),
                        ..Default::default()
                    },
                );
                config.sections.insert(
                    "disabled".into(),
                    ccr_config::ConfigSection {
                        enabled: Some(false),
                        ..Default::default()
                    },
                );
                context.manager().save(&config)
            })
            .unwrap();
        let inactive = read(&source).unwrap();
        assert_eq!(inactive.activation, GrokActivationState::Inactive);
        assert_eq!((inactive.profiles_total, inactive.profiles_enabled), (2, 1));
        assert!(!inactive.config_exists);
        assert_eq!(inactive.auth_mode, None);

        context.run(|| platform.apply_profile("relay")).unwrap();
        let runtime = platform.get_settings_path();
        let original = std::fs::read(&runtime).unwrap();
        let registry = std::fs::read(&context.paths().registry_file).unwrap();
        let active = read(&source).unwrap();
        assert_eq!(
            active.activation,
            GrokActivationState::Active {
                name: "relay".into()
            }
        );
        assert_eq!(active.auth_mode, Some(GrokProfileAuthMode::InlineApiKey));
        assert!(active.config_exists);
        assert!(!format!("{active:?}").contains("synthetic-overview-secret"));
        assert_eq!(std::fs::read(&runtime).unwrap(), original);

        let drifted = String::from_utf8(original)
            .unwrap()
            .replace("fixture-model", "different-model");
        std::fs::write(&runtime, &drifted).unwrap();
        assert_eq!(
            read(&source).unwrap().activation,
            GrokActivationState::Drifted {
                name: "relay".into()
            }
        );
        assert_eq!(std::fs::read_to_string(&runtime).unwrap(), drifted);
        assert_eq!(
            std::fs::read(&context.paths().registry_file).unwrap(),
            registry
        );
        std::fs::remove_file(
            context
                .paths()
                .platform_dir
                .join("profile_entry_config_state.json"),
        )
        .unwrap();
        assert_eq!(
            read(&source).unwrap().activation,
            GrokActivationState::UnsafeMissingEntryState {
                name: Some("relay".into())
            }
        );
        assert_eq!(std::fs::read_to_string(&runtime).unwrap(), drifted);
    }

    #[tokio::test]
    async fn grok_overview_rejects_wrong_scope_before_source_resolution() {
        use ccr_app::runtime::{RuntimeOwner, TaskEvent};
        let owner = RuntimeOwner::new().unwrap();
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: "relative".into(),
                registry: "relative".into(),
                delay: Duration::ZERO,
            },
        );
        for (environment, platform) in [
            (
                EnvironmentId::Ssh("synthetic".into()),
                RequestPlatform::Grok,
            ),
            (
                EnvironmentId::Local("synthetic".into()),
                RequestPlatform::Claude,
            ),
        ] {
            let mut sub = services
                .grok_overview(RequestKey {
                    environment,
                    platform,
                    generation: 0,
                    request_id: 0,
                })
                .unwrap();
            loop {
                match sub.next().await {
                    Some(TaskEvent::Terminal(result)) => {
                        assert!(matches!(
                            result.as_ref(),
                            Completion::Failed(ProfileError::Unsupported)
                        ));
                        break;
                    }
                    Some(_) => {}
                    None => panic!("missing terminal"),
                }
            }
        }
        tokio::task::spawn_blocking(move || owner.shutdown())
            .await
            .unwrap();
    }
}
