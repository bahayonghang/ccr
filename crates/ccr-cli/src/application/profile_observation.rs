//! Read-only evidence for an explicitly resolved profile catalog and runtime.
use super::ProfileLifecycleContext;
use crate::platforms::{ClaudePlatform, GrokActivationState};
use ccr_config::{CcsConfig, ConfigSection, Platform, UnifiedConfig, section_to_profile};
use ccr_core::CcrError;
use ccr_types::{ClaudeProfileAuthMode, ClaudeSettings, env_keys::CCR_MANAGED_KEYS};
use std::{io::ErrorKind, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeState {
    NoMarker,
    NoMatch,
    Inactive,
    Matched,
    Drifted,
    OverridesCleared,
    OverridesPresent,
    BaselineMissing,
    Missing,
    Invalid,
    Unreadable,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileObservation {
    pub name: Option<String>,
    pub state: RuntimeState,
}

impl ProfileObservation {
    fn new(name: Option<String>, state: RuntimeState) -> Self {
        Self { name, state }
    }
}

fn read_runtime(path: &Path) -> Result<Vec<u8>, RuntimeState> {
    std::fs::read(path).map_err(|error| match error.kind() {
        ErrorKind::NotFound => RuntimeState::Missing,
        _ => RuntimeState::Unreadable,
    })
}

fn reason(error: CcrError) -> RuntimeState {
    match error {
        CcrError::ConfigFormatInvalid(_)
        | CcrError::TomlError(_)
        | CcrError::JsonError(_)
        | CcrError::ValidationError(_) => RuntimeState::Invalid,
        CcrError::PlatformNotSupported(_) => RuntimeState::Unsupported,
        CcrError::ConfigMissing(_) | CcrError::SettingsMissing(_) => RuntimeState::Missing,
        _ => RuntimeState::Unreadable,
    }
}

fn claude_state(path: &Path, section: &ConfigSection) -> Result<RuntimeState, RuntimeState> {
    let bytes = read_runtime(path)?;
    let document: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| RuntimeState::Invalid)?;
    let object = document.as_object().ok_or(RuntimeState::Invalid)?;
    let env = object
        .get("env")
        .map(|v| v.as_object().ok_or(RuntimeState::Invalid))
        .transpose()?;
    if env.is_some_and(|env| {
        CCR_MANAGED_KEYS
            .iter()
            .any(|key| env.get(*key).is_some_and(|value| !value.is_string()))
    }) {
        return Err(RuntimeState::Invalid);
    }
    let mode = ClaudePlatform::profile_auth_mode(&section_to_profile(section));
    // Use the writer's key registry and complete mapping. Only managed keys are
    // inspected, and neither a value nor a raw parser error leaves this boundary.
    let mut expected = ClaudeSettings::default();
    if mode == ClaudeProfileAuthMode::ApiKey {
        expected.apply_managed_env(section.to_managed_env_pairs());
    }
    let matches = CCR_MANAGED_KEYS.iter().all(|key| {
        match (env.and_then(|env| env.get(*key)), expected.env.get(*key)) {
            (Some(value), Some(expected)) => value.as_str() == Some(expected.as_str()),
            (None, None) => true,
            _ => false,
        }
    });
    Ok(match (mode, matches) {
        (ClaudeProfileAuthMode::ApiKey, true) => RuntimeState::Matched,
        (ClaudeProfileAuthMode::ApiKey, false) => RuntimeState::Drifted,
        (ClaudeProfileAuthMode::Subscription, true) => RuntimeState::OverridesCleared,
        (ClaudeProfileAuthMode::Subscription, false) => RuntimeState::OverridesPresent,
    })
}

impl ProfileLifecycleContext {
    /// Inspect only paths already bound to this context. Browsing never invokes
    /// repair, applies a profile, or resolves a candidate by comparing its shape.
    pub fn observe_profile(&self, catalog: &CcsConfig) -> ProfileObservation {
        let path = self.instance().get_settings_path();
        let observation = match self.platform() {
            Platform::Claude => self.observe_claude(catalog, &path),
            Platform::Codex => self.observe_codex(&path),
            Platform::Grok => self
                .grok_platform()
                .and_then(|p| p.inspect_activation_state())
                .map_err(reason)
                .map(|state| match state {
                    GrokActivationState::Inactive => {
                        ProfileObservation::new(None, RuntimeState::Inactive)
                    }
                    GrokActivationState::Active { name } => {
                        ProfileObservation::new(Some(name), RuntimeState::Matched)
                    }
                    GrokActivationState::Drifted { name } => {
                        let state = match path.try_exists() {
                            Ok(true) => RuntimeState::Drifted,
                            Ok(false) => RuntimeState::Missing,
                            Err(_) => RuntimeState::Unreadable,
                        };
                        ProfileObservation::new(Some(name), state)
                    }
                    GrokActivationState::UnsafeMissingEntryState { name } => {
                        ProfileObservation::new(name, RuntimeState::BaselineMissing)
                    }
                }),
            _ => Err(RuntimeState::Unsupported),
        };
        observation.unwrap_or_else(|state| ProfileObservation::new(None, state))
    }

    fn observe_claude(
        &self,
        catalog: &CcsConfig,
        path: &Path,
    ) -> Result<ProfileObservation, RuntimeState> {
        let file = catalog.current_config.trim();
        let marked = if let Some(entry) = catalog
            .sections
            .get_key_value(file)
            .filter(|_| !file.is_empty())
        {
            Some(entry)
        } else {
            let bytes = match read_runtime(&self.paths().registry_file) {
                Err(RuntimeState::Missing) => {
                    return Ok(ProfileObservation::new(None, RuntimeState::NoMarker));
                }
                result => result?,
            };
            let registry: UnifiedConfig =
                toml::from_str(std::str::from_utf8(&bytes).map_err(|_| RuntimeState::Invalid)?)
                    .map_err(|_| RuntimeState::Invalid)?;
            registry
                .get_platform("claude")
                .ok()
                .and_then(|platform| platform.current_profile.as_ref())
                .and_then(|name| catalog.sections.get_key_value(name))
        };
        Ok(match marked {
            None => ProfileObservation::new(None, RuntimeState::NoMarker),
            Some((name, section)) => ProfileObservation::new(
                Some(name.clone()),
                claude_state(path, section).unwrap_or_else(|state| state),
            ),
        })
    }

    fn observe_codex(&self, path: &Path) -> Result<ProfileObservation, RuntimeState> {
        let bytes = read_runtime(path)?;
        let config =
            toml::from_str(std::str::from_utf8(&bytes).map_err(|_| RuntimeState::Invalid)?)
                .map_err(|_| RuntimeState::Invalid)?;
        let name = self
            .codex_platform()
            .map_err(reason)?
            .match_current_config(&config)
            .map_err(reason)?;
        let state = if name.is_some() {
            RuntimeState::Matched
        } else {
            RuntimeState::NoMatch
        };
        Ok(ProfileObservation::new(name, state))
    }
}
