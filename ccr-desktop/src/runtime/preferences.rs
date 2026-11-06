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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowPlacement {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub maximized: bool,
}

impl WindowPlacement {
    fn valid(self) -> bool {
        [
            self.x,
            self.y,
            self.width,
            self.height,
            self.x + self.width,
            self.y + self.height,
        ]
        .into_iter()
        .all(f32::is_finite)
            && self.width > 0.
            && self.height > 0.
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct DesktopPreferences {
    pub theme: Preference,
    pub language: Language,
    pub confirm_quit: bool,
    pub window: Option<WindowPlacement>,
}

impl Default for DesktopPreferences {
    fn default() -> Self {
        Self {
            theme: Preference::Light,
            language: Language::English,
            confirm_quit: true,
            window: None,
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
            ConfigSource::Benchmark(layout) => Some(layout.path("preferences")),
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
    reject_redirected_path(path)?;
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
        window: match value.get("window") {
            None | Some(serde_json::Value::Null) => None,
            Some(window) => {
                let number = |field| {
                    window
                        .get(field)
                        .and_then(serde_json::Value::as_f64)
                        .map(|value| value as f32)
                        .ok_or(PreferencesError::Invalid)
                };
                let placement = WindowPlacement {
                    x: number("x")?,
                    y: number("y")?,
                    width: number("width")?,
                    height: number("height")?,
                    maximized: window
                        .get("maximized")
                        .and_then(serde_json::Value::as_bool)
                        .ok_or(PreferencesError::Invalid)?,
                };
                if !placement.valid() {
                    return Err(PreferencesError::Invalid);
                }
                Some(placement)
            }
        },
    })
}

fn reject_redirected_path(path: &std::path::Path) -> Result<(), PreferencesError> {
    if !path.is_absolute() {
        return Err(PreferencesError::Storage);
    }
    for ancestor in path.ancestors() {
        match std::fs::symlink_metadata(ancestor) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(PreferencesError::Storage);
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if metadata.file_attributes() & 0x400 != 0 {
                        return Err(PreferencesError::Storage);
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(PreferencesError::Storage),
        }
    }
    Ok(())
}

fn write(
    path: Option<&std::path::Path>,
    preferences: DesktopPreferences,
) -> Result<(), PreferencesError> {
    let path = path.ok_or(PreferencesError::Storage)?;
    // Do not silently overwrite an unreadable or newer-format preference file.
    read(Some(path))?;
    if preferences
        .window
        .is_some_and(|placement| !placement.valid())
    {
        return Err(PreferencesError::Invalid);
    }
    let value = serde_json::json!({
        "version": 1,
        "theme": match preferences.theme { Preference::System => "system", Preference::Light => "light", Preference::Dark => "dark" },
        "language": match preferences.language { Language::English => "en", Language::Chinese => "zh" },
        "confirm_quit": preferences.confirm_quit,
        "window": preferences.window.map(|placement| serde_json::json!({
            "x": placement.x, "y": placement.y,
            "width": placement.width, "height": placement.height,
            "maximized": placement.maximized,
        })),
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
            window: Some(WindowPlacement {
                x: 50.,
                y: 75.,
                width: 1280.,
                height: 800.,
                maximized: true,
            }),
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
    fn version_one_without_geometry_loads_and_invalid_geometry_is_not_overwritten() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("preferences.json");
        let mut value = serde_json::json!({
            "version":1, "theme":"dark", "language":"zh", "confirm_quit":false,
        });
        std::fs::write(&path, value.to_string()).unwrap();
        let preferences = read(Some(&path)).unwrap();
        assert!(preferences.window.is_none());
        assert!(preferences.theme == Preference::Dark);
        value["window"] = serde_json::json!({
            "x":0, "y":0, "width":-1, "height":800, "maximized":false,
        });
        let bytes = value.to_string();
        std::fs::write(&path, &bytes).unwrap();
        assert!(matches!(read(Some(&path)), Err(PreferencesError::Invalid)));
        assert_eq!(
            write(Some(&path), preferences),
            Err(PreferencesError::Invalid)
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), bytes);
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

    #[cfg(any(unix, windows))]
    #[test]
    fn linked_fixture_preferences_refuse_read_and_automatic_write() {
        let root = tempfile::tempdir().unwrap();
        let fixture = root.path().join("fixture");
        let outside = root.path().join("outside");
        std::fs::create_dir_all(&fixture).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        let target = outside.join("preferences.json");
        write(Some(&target), DesktopPreferences::default()).unwrap();
        let original = std::fs::read(&target).unwrap();
        let redirected = fixture.join("desktop-gpui");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &redirected).unwrap();
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let result = std::process::Command::new("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command", "$ErrorActionPreference = 'Stop'; New-Item -ItemType Junction -Path $env:CCR_FIXTURE_LINK -Value $env:CCR_FIXTURE_TARGET | Out-Null"])
                .env("CCR_FIXTURE_LINK", &redirected)
                .env("CCR_FIXTURE_TARGET", &outside)
                .creation_flags(0x08000000)
                .output()
                .unwrap();
            assert!(result.status.success(), "fixture junction creation failed");
        }
        let path = redirected.join("preferences.json");
        assert!(matches!(read(Some(&path)), Err(PreferencesError::Storage)));
        assert_eq!(
            write(
                Some(&path),
                DesktopPreferences {
                    confirm_quit: false,
                    ..DesktopPreferences::default()
                }
            ),
            Err(PreferencesError::Storage),
        );
        assert_eq!(std::fs::read(&target).unwrap(), original);
    }
}
