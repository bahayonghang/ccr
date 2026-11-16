//! Captured, metadata-only settings layers; fixture roots never inspect host policy.
use super::*;
use ccr_app::environments::EnvironmentPolicy;
use ccr_app::platform_settings::layers::{self, Layer};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct SourceLayer {
    pub id: String,
    pub label: String,
    pub path: Option<ccr_app::environments::io::EnvironmentPath>,
    pub exists: Option<bool>,
    pub size: Option<u64>,
    pub mtime: Option<u64>,
    pub editable: bool,
    pub state: layers::State,
}
impl From<Layer> for SourceLayer {
    fn from(layer: Layer) -> Self {
        Self {
            id: layer.id,
            label: layer.label,
            path: layer
                .path
                .map(ccr_app::environments::io::EnvironmentPath::Local),
            exists: layer.exists,
            size: layer.size,
            mtime: layer.mtime,
            editable: layer.editable,
            state: layer.state,
        }
    }
}

async fn remote_inventory(
    io: ccr_app::environments::io::EnvironmentIo,
    platform: Platform,
    project: Option<String>,
    cancel: tokio_util::sync::CancellationToken,
) -> Result<Vec<SourceLayer>, PlatformError> {
    use ccr_app::environments::{io::EnvironmentPath, ssh_security::RemotePosixPath};
    let absolute = io
        .at_remote_root(RemotePosixPath::root(Some("/")).map_err(|_| PlatformError::Invalid)?)
        .map_err(super::super::remote_config::error)?;
    let project = project
        .map(|path| {
            if !path.starts_with('/') {
                return Err(PlatformError::Invalid);
            }
            RemotePosixPath::root(Some(&path)).map_err(|_| PlatformError::Invalid)
        })
        .transpose()?;
    let mut sources: Vec<(&str, &str, Option<EnvironmentPath>, bool)> = Vec::new();
    let user = match platform {
        Platform::Claude => ".claude/settings.json",
        Platform::Codex => ".codex/config.toml",
        Platform::Grok => ".grok/config.toml",
        Platform::OpenCode => ".config/opencode/opencode.json",
        Platform::Gemini => ".gemini/antigravity-cli/settings.json",
        _ => return Err(PlatformError::Unsupported),
    };
    sources.push((
        "user",
        "User",
        Some(
            io.roots()
                .relative(user)
                .map_err(super::super::remote_config::error)?,
        ),
        true,
    ));
    if platform == Platform::Claude {
        sources.insert(
            0,
            (
                "managed",
                "Managed",
                Some(EnvironmentPath::Ssh(
                    RemotePosixPath::root(Some("/etc/claude-code/managed-settings.json"))
                        .map_err(|_| PlatformError::Invalid)?,
                )),
                false,
            ),
        );
        for (id, path) in [
            ("project", ".claude/settings.json"),
            ("local", ".claude/settings.local.json"),
        ] {
            sources.push((
                id,
                id,
                project
                    .as_ref()
                    .map(|root| root.join_relative(path).map(EnvironmentPath::Ssh))
                    .transpose()
                    .map_err(|_| PlatformError::Invalid)?,
                false,
            ));
        }
    }
    if platform == Platform::Grok {
        sources.push((
            "project",
            "Project",
            project
                .as_ref()
                .map(|root| {
                    root.join_relative(".grok/config.toml")
                        .map(EnvironmentPath::Ssh)
                })
                .transpose()
                .map_err(|_| PlatformError::Invalid)?,
            false,
        ));
        for (id, path) in [
            ("managed_user", ".grok/managed_config.toml"),
            ("requirements_user", ".grok/requirements.toml"),
        ] {
            sources.push((
                id,
                id,
                Some(
                    io.roots()
                        .relative(path)
                        .map_err(super::super::remote_config::error)?,
                ),
                false,
            ));
        }
        for (id, path) in [
            ("managed_system", "/etc/grok/managed_config.toml"),
            ("requirements_system", "/etc/grok/requirements.toml"),
        ] {
            sources.push((
                id,
                id,
                Some(EnvironmentPath::Ssh(
                    RemotePosixPath::root(Some(path)).map_err(|_| PlatformError::Invalid)?,
                )),
                false,
            ));
        }
    }
    let mut rows = Vec::new();
    for (id, label, path, editable) in sources {
        let metadata = match &path {
            Some(EnvironmentPath::Ssh(path)) => Some(
                absolute
                    .metadata(path.as_str().trim_start_matches('/'), cancel.clone())
                    .await
                    .map_err(super::super::remote_config::error)?,
            ),
            None => None,
            _ => return Err(PlatformError::Invalid),
        };
        let present = metadata.flatten();
        rows.push(SourceLayer {
            id: id.into(),
            label: label.into(),
            path,
            exists: metadata.map(|value| value.is_some()),
            size: present.map(|value| value.0),
            mtime: present.and_then(|value| value.1),
            editable,
            state: match metadata {
                None => layers::State::Unresolved,
                Some(None) => layers::State::Missing,
                Some(Some(_)) => layers::State::Present,
            },
        });
    }
    if platform == Platform::Codex {
        let mut names = io
            .list(".codex", cancel.clone())
            .await
            .map_err(super::super::remote_config::error)?;
        names.sort();
        for name in names
            .into_iter()
            .filter(|name| name.ends_with(".config.toml") && name != "config.toml")
        {
            let relative = format!(".codex/{name}");
            let metadata = io
                .metadata(&relative, cancel.clone())
                .await
                .map_err(super::super::remote_config::error)?;
            rows.push(SourceLayer {
                id: "profile_overlay".into(),
                label: name.trim_end_matches(".toml").into(),
                path: Some(
                    io.roots()
                        .relative(&relative)
                        .map_err(super::super::remote_config::error)?,
                ),
                exists: Some(metadata.is_some()),
                size: metadata.map(|value| value.0),
                mtime: metadata.and_then(|value| value.1),
                editable: false,
                state: if metadata.is_some() {
                    layers::State::Present
                } else {
                    layers::State::Missing
                },
            });
        }
    }
    Ok(rows)
}

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
            if let Some(layout) = source.benchmark() {
                layout
                    .check_path(&path)
                    .map_err(|_| PlatformError::Invalid)?;
            }
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
        project: Option<String>,
    ) -> Result<TaskSubscription<Vec<SourceLayer>, PlatformError>, AdmissionError> {
        if matches!(key.environment, ccr_app::runtime::EnvironmentId::Ssh(_)) {
            let io = self.remote_configuration_io(&key.environment);
            let platform = key.platform;
            return self.executor.read(key, move |cancel, _| {
                tokio::runtime::Handle::current().block_on(async move {
                    remote_inventory(
                        io.map_err(super::super::remote_config::error)?,
                        platform,
                        project,
                        cancel,
                    )
                    .await
                })
            });
        }
        let source = self.source.clone();
        let platform = key.platform;
        let local = EnvironmentPolicy::LocalOnly.admit(&key.environment).is_ok();
        self.executor.read(key, move |_, _| {
            if !local {
                return Err(PlatformError::Unsupported);
            }
            inventory(&source, platform, project.map(PathBuf::from))
                .map(|rows| rows.into_iter().map(SourceLayer::from).collect())
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
