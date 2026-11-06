//! GPUI-only preferences. File access stays on the bounded runtime worker.
use std::{
    io::Read,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use ccr_app::runtime::{
    AdmissionError, Completion, EnvironmentId, Platform, RequestKey, TaskSubscription,
};
use ccr_core::core::AtomicWriter;

use super::{ConfigSource, Services};
use crate::{locales::Language, theme::Preference};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DesktopPreferences {
    pub theme: Preference,
    pub language: Language,
    pub confirm_quit: bool,
}

impl Default for DesktopPreferences {
    fn default() -> Self {
        Self {
            theme: Preference::Light,
            language: Language::English,
            confirm_quit: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferencesError {
    Storage,
    Invalid,
}

#[derive(Clone)]
pub struct PreferencesRequests {
    services: Services,
    path: Option<PathBuf>,
    written_generation: Arc<Mutex<u64>>,
}

impl PreferencesRequests {
    pub fn new(services: &Services) -> Self {
        let root = match &services.source {
            ConfigSource::Local => dirs::config_dir().map(|root| root.join("ccr")),
            ConfigSource::Fixture { registry, .. } if registry.is_absolute() => {
                registry.parent().map(ToOwned::to_owned)
            }
            ConfigSource::Fixture { .. } => None,
        };
        Self {
            services: services.clone(),
            path: root.map(|root| root.join("desktop-gpui/preferences.json")),
            written_generation: Default::default(),
        }
    }

    pub fn path(&self) -> Option<&std::path::Path> {
        self.path.as_deref()
    }

    pub fn load(
        &self,
    ) -> Result<TaskSubscription<DesktopPreferences, PreferencesError>, AdmissionError> {
        let path = self.path.clone();
        self.services
            .executor
            .read(key(0), move |_, _| read(path.as_deref()))
    }

    pub fn save(
        &self,
        preferences: DesktopPreferences,
        generation: u64,
    ) -> Result<TaskSubscription<DesktopPreferences, PreferencesError>, AdmissionError> {
        let path = self.path.clone();
        let written_generation = self.written_generation.clone();
        self.services
            .executor
            .job(key(generation), move |cancel, _| {
                let Ok(mut latest) = written_generation.lock() else {
                    return Completion::Failed(PreferencesError::Storage);
                };
                if generation < *latest {
                    return Completion::Success(preferences);
                }
                *latest = generation;
                if cancel.is_cancelled() {
                    return Completion::Cancelled;
                }
                match write(path.as_deref(), preferences) {
                    Ok(()) => Completion::Success(preferences),
                    Err(error) => Completion::Failed(error),
                }
            })
    }
}

fn key(generation: u64) -> RequestKey {
    RequestKey {
        environment: EnvironmentId::Local("desktop-gpui-preferences".into()),
        platform: Platform::Claude,
        generation,
        request_id: generation,
    }
}

fn read(path: Option<&std::path::Path>) -> Result<DesktopPreferences, PreferencesError> {
    let path = path.ok_or(PreferencesError::Storage)?;
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(DesktopPreferences::default());
        }
        Err(_) => return Err(PreferencesError::Storage),
    };
    let mut bytes = Vec::new();
    file.take(16_385)
        .read_to_end(&mut bytes)
        .map_err(|_| PreferencesError::Storage)?;
    if bytes.len() > 16_384 {
        return Err(PreferencesError::Invalid);
    }
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| PreferencesError::Invalid)?;
    if value.get("version").and_then(|v| v.as_u64()) != Some(1) {
        return Err(PreferencesError::Invalid);
    }
    Ok(DesktopPreferences {
        theme: match value.get("theme").and_then(|v| v.as_str()) {
            Some("system") => Preference::System,
            Some("light") => Preference::Light,
            Some("dark") => Preference::Dark,
            _ => return Err(PreferencesError::Invalid),
        },
        language: match value.get("language").and_then(|v| v.as_str()) {
            Some("en") => Language::English,
            Some("zh") => Language::Chinese,
            _ => return Err(PreferencesError::Invalid),
        },
        confirm_quit: value
            .get("confirm_quit")
            .and_then(|v| v.as_bool())
            .ok_or(PreferencesError::Invalid)?,
    })
}

fn write(
    path: Option<&std::path::Path>,
    preferences: DesktopPreferences,
) -> Result<(), PreferencesError> {
    let path = path.ok_or(PreferencesError::Storage)?;
    // Do not silently overwrite an unreadable or newer-format preference file.
    read(Some(path))?;
    let value = serde_json::json!({
        "version": 1,
        "theme": match preferences.theme { Preference::System => "system", Preference::Light => "light", Preference::Dark => "dark" },
        "language": match preferences.language { Language::English => "en", Language::Chinese => "zh" },
        "confirm_quit": preferences.confirm_quit,
    });
    AtomicWriter::new(path)
        .write_string(&value.to_string())
        .map_err(|_| PreferencesError::Storage)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn preferences_round_trip_isolated_and_invalid_file_is_preserved() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("desktop-gpui/preferences.json");
        assert!(read(Some(&path)).unwrap() == DesktopPreferences::default());
        let selected = DesktopPreferences {
            theme: Preference::Dark,
            language: Language::Chinese,
            confirm_quit: false,
        };
        write(Some(&path), selected).unwrap();
        assert!(read(Some(&path)).unwrap() == selected);
        std::fs::write(&path, b"{broken").unwrap();
        assert_eq!(
            write(Some(&path), DesktopPreferences::default()),
            Err(PreferencesError::Invalid)
        );
        assert_eq!(std::fs::read(&path).unwrap(), b"{broken");
    }

    #[test]
    fn relative_fixture_preferences_have_no_storage_target() {
        let owner = ccr_app::runtime::RuntimeOwner::new().unwrap();
        let services = Services::new(
            owner.handle(),
            ConfigSource::Fixture {
                profiles: PathBuf::from("profiles.toml"),
                registry: PathBuf::from("config.toml"),
                delay: std::time::Duration::ZERO,
            },
        );
        let requests = PreferencesRequests::new(&services);
        assert!(requests.path().is_none());
        assert!(matches!(
            read(requests.path()),
            Err(PreferencesError::Storage)
        ));
        assert_eq!(
            write(requests.path(), DesktopPreferences::default()),
            Err(PreferencesError::Storage)
        );
        owner.shutdown();
    }
}
