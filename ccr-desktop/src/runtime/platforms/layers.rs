//! Captured, metadata-only settings layers; fixture roots never inspect host policy.
use super::*;
use ccr_app::platform_settings::layers::{self, Layer};
use std::path::{Path, PathBuf};

fn check_path(path: &Path) -> Result<(), PlatformError> {
    for ancestor in path.ancestors() {
        match std::fs::symlink_metadata(ancestor) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(PlatformError::Invalid);
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if metadata.file_attributes() & 0x400 != 0 {
                        return Err(PlatformError::Invalid);
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(PlatformError::Storage),
        }
    }
    Ok(())
}

fn inventory(
    source: &ConfigSource,
    platform: Platform,
    project: Option<PathBuf>,
) -> Result<Vec<Layer>, PlatformError> {
    let roots = roots(source)?;
    let project = project
        .map(|path| {
            if !path.is_absolute() {
                return Err(PlatformError::Invalid);
            }
            check_path(&path)?;
            let path = path.canonicalize().map_err(|_| PlatformError::Invalid)?;
            if !path.is_dir() {
                return Err(PlatformError::Invalid);
            }
            if let ConfigSource::Fixture { profiles, .. } = source {
                let parent = profiles
                    .parent()
                    .ok_or(PlatformError::Invalid)?
                    .canonicalize()
                    .map_err(|_| PlatformError::Invalid)?;
                if !path.starts_with(parent) {
                    return Err(PlatformError::Invalid);
                }
            }
            Ok(path)
        })
        .transpose()?;
    let fixture_system = roots
        .home
        .parent()
        .ok_or(PlatformError::Invalid)?
        .join("system");
    let layers = match platform {
        Platform::Claude => {
            let managed = if roots.fixture {
                fixture_system.join("ClaudeCode/managed-settings.json")
            } else {
                #[cfg(windows)]
                let path = std::env::var_os("PROGRAMDATA")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
                    .join("ClaudeCode/managed-settings.json");
                #[cfg(target_os = "macos")]
                let path =
                    PathBuf::from("/Library/Application Support/ClaudeCode/managed-settings.json");
                #[cfg(all(not(windows), not(target_os = "macos")))]
                let path = PathBuf::from("/etc/claude-code/managed-settings.json");
                path
            };
            for path in [
                Some(roots.claude.settings_file.as_path()),
                Some(managed.as_path()),
                project.as_deref(),
            ]
            .into_iter()
            .flatten()
            {
                check_path(path)?;
            }
            if let Some(project) = &project {
                check_path(&project.join(".claude/settings.json"))?;
                check_path(&project.join(".claude/settings.local.json"))?;
            }
            layers::claude(&roots.claude.settings_file, &managed, project.as_deref())
        }
        Platform::Codex => {
            let path = roots.codex_runtime.join("config.toml");
            check_path(&path)?;
            if let Some(directory) = path.parent()
                && directory.exists()
            {
                for entry in std::fs::read_dir(directory).map_err(|_| PlatformError::Storage)? {
                    let path = entry.map_err(|_| PlatformError::Storage)?.path();
                    if path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.ends_with(".config.toml"))
                    {
                        check_path(&path)?;
                    }
                }
            }
            layers::codex(&path).map_err(|_| PlatformError::Storage)?
        }
        Platform::Grok => {
            let user = roots.grok_runtime.join("config.toml");
            let system = if roots.fixture {
                fixture_system.join("grok")
            } else {
                PathBuf::from("/etc/grok")
            };
            for path in [
                &user,
                &roots.home.join(".grok/managed_config.toml"),
                &roots.home.join(".grok/requirements.toml"),
                &system.join("managed_config.toml"),
                &system.join("requirements.toml"),
            ] {
                check_path(path)?;
            }
            if let Some(project) = &project {
                check_path(&project.join(".grok/config.toml"))?;
            }
            layers::grok(&user, &roots.home, project.as_deref(), &system)
        }
        _ => return Err(PlatformError::Unsupported),
    };
    Ok(layers)
}

impl Services {
    pub fn platform_layers_request(
        &self,
        key: RequestKey,
        project: Option<PathBuf>,
    ) -> Result<TaskSubscription<Vec<Layer>, PlatformError>, AdmissionError> {
        let source = self.source.clone();
        let platform = key.platform;
        let local = matches!(key.environment, EnvironmentId::Local(_));
        self.executor.read(key, move |_, _| {
            if !local {
                return Err(PlatformError::Unsupported);
            }
            inventory(&source, platform, project)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixture_layers_use_captured_roots_and_reject_external_project_scope() {
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let source = ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: std::time::Duration::ZERO,
        };
        let layers = inventory(&source, Platform::Claude, None)
            .unwrap_or_else(|e| panic!("inventory: {e:?}"));
        assert!(
            layers[0]
                .path
                .as_ref()
                .is_some_and(|path| path.starts_with(root.path()))
        );
        assert!(!layers[0].editable);
        assert_eq!(layers[1].exists, None);
        let external = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        assert!(inventory(&source, Platform::Grok, Some(external.path().into())).is_err());
        assert!(inventory(&source, Platform::Claude, Some(PathBuf::from("relative"))).is_err());
    }
}
