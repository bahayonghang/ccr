//! Metadata-only configuration source inventory. Callers capture roots/project scope.
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Unresolved,
    Missing,
    Present,
    Unreadable,
}

#[derive(Clone, Debug, Serialize)]
pub struct Layer {
    pub id: String,
    pub label: String,
    pub path: Option<PathBuf>,
    pub exists: Option<bool>,
    pub size: Option<u64>,
    pub mtime: Option<u64>,
    pub editable: bool,
    #[serde(skip)]
    pub state: State,
}

pub fn file(id: &str, label: &str, path: Option<&Path>, editable: bool) -> Layer {
    let metadata = path.map(fs::metadata);
    let state = match &metadata {
        None => State::Unresolved,
        Some(Ok(_)) => State::Present,
        Some(Err(error)) if error.kind() == std::io::ErrorKind::NotFound => State::Missing,
        Some(Err(_)) => State::Unreadable,
    };
    let metadata = metadata.and_then(Result::ok);
    Layer {
        id: id.into(),
        label: label.into(),
        path: path.map(Path::to_path_buf),
        exists: path.map(Path::exists),
        size: metadata.as_ref().map(fs::Metadata::len),
        mtime: metadata
            .as_ref()
            .and_then(|value| value.modified().ok())
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .and_then(|value| u64::try_from(value.as_millis()).ok()),
        editable,
        state,
    }
}

/// Precedence order follows Claude's source inventory. A project is explicit;
/// absent scope remains unknown instead of falling back to the process cwd.
pub fn claude(user: &Path, managed: &Path, project: Option<&Path>) -> Vec<Layer> {
    vec![
        file("managed", "Managed", Some(managed), false),
        file(
            "project",
            "Project",
            project.map(|p| p.join(".claude/settings.json")).as_deref(),
            false,
        ),
        file(
            "local",
            "Project local",
            project
                .map(|p| p.join(".claude/settings.local.json"))
                .as_deref(),
            false,
        ),
        file("user", "User", Some(user), true),
    ]
}

pub fn codex(user: &Path) -> Result<Vec<Layer>, &'static str> {
    let mut layers = vec![file("user", "User", Some(user), true)];
    if let Some(directory) = user.parent()
        && directory.exists()
    {
        let mut overlays = Vec::new();
        for entry in fs::read_dir(directory).map_err(|_| "config_layers_unreadable")? {
            let entry = entry.map_err(|_| "config_layers_unreadable")?;
            let path = entry.path();
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".config.toml") && name != "config.toml")
            {
                overlays.push(path);
            }
        }
        overlays.sort();
        layers.extend(overlays.iter().map(|path| {
            file(
                "profile_overlay",
                path.file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or("Profile overlay"),
                Some(path),
                false,
            )
        }));
    }
    Ok(layers)
}

/// User config is an editable source, not the final effective configuration.
/// Managed constraints and requirements remain separate read-only layers.
pub fn grok(user: &Path, home: &Path, project: Option<&Path>, system: &Path) -> Vec<Layer> {
    vec![
        file("user", "User", Some(user), true),
        file(
            "project",
            "Project",
            project.map(|p| p.join(".grok/config.toml")).as_deref(),
            false,
        ),
        file(
            "managed_user",
            "Managed (user)",
            Some(&home.join(".grok/managed_config.toml")),
            false,
        ),
        file(
            "managed_system",
            "Managed (system)",
            Some(&system.join("managed_config.toml")),
            false,
        ),
        file(
            "requirements_user",
            "Requirements (user)",
            Some(&home.join(".grok/requirements.toml")),
            false,
        ),
        file(
            "requirements_system",
            "Requirements (system)",
            Some(&system.join("requirements.toml")),
            false,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sources_preserve_read_only_layers_unknown_project_and_overlay_order() {
        let root = tempfile::tempdir().unwrap_or_else(|e| panic!("fixture: {e}"));
        let user = root.path().join("config.toml");
        for name in [
            "z.config.toml",
            "a.config.toml",
            "config.toml",
            "unrelated.toml",
        ] {
            fs::write(root.path().join(name), "secret-sentinel")
                .unwrap_or_else(|e| panic!("fixture: {e}"));
        }
        let codex = codex(&user).unwrap_or_else(|e| panic!("inventory: {e}"));
        assert_eq!(codex.len(), 3);
        assert!(codex[0].editable);
        assert_eq!(codex[1].label, "a.config");
        assert!(codex[1..].iter().all(|layer| !layer.editable));
        let claude = claude(&user, &root.path().join("managed.json"), None);
        assert_eq!(claude[0].state, State::Missing);
        assert_eq!(claude[1].state, State::Unresolved);
        assert_eq!(claude[1].exists, None);
        assert!(claude[1].path.is_none());
        let grok = grok(
            &user,
            root.path(),
            Some(root.path()),
            &root.path().join("system"),
        );
        assert_eq!(grok.len(), 6);
        assert!(grok[1..].iter().all(|layer| !layer.editable));
        let wire = serde_json::to_string(&codex).unwrap_or_else(|e| panic!("wire: {e}"));
        assert!(!wire.contains("secret-sentinel"));
        assert!(!wire.contains("state"));
    }
}
