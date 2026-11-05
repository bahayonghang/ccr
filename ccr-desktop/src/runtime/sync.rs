//! Local canonical sync config. This reader never contacts WebDAV or writes a password.

use std::path::{Path, PathBuf};

use ccr_core::core::log_redact::redact_log_text;
use ccr_sync::{SyncFolderManager, SyncFoldersConfig};

use super::{ConfigSource, Services};
use ccr_app::runtime::{AdmissionError, EnvironmentId, RequestKey, TaskSubscription};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncError {
    Unsupported,
    Storage,
    Unreadable,
}

impl SyncError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Unsupported => {
                "Sync config is read only for the local environment. / 只在本地环境读取同步配置"
            }
            Self::Storage => {
                "The local sync config path could not be resolved. / 无法定位本地同步配置"
            }
            Self::Unreadable => {
                "The canonical sync config could not be read. Remote access was not tested. / 无法读取规范同步配置，未测试远程访问"
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncUrl {
    Missing,
    Stored,
    Host(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncFolderRow {
    pub name: String,
    pub enabled: bool,
    pub local_path: String,
    pub remote_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncLocal {
    pub webdav_enabled: bool,
    pub url: SyncUrl,
    pub username_present: bool,
    pub password_present: bool,
    pub folders: Vec<SyncFolderRow>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncSnapshot {
    Absent,
    Present(SyncLocal),
}

fn host_only(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }
    let rest = trimmed
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(trimmed);
    if rest.contains('@') || rest.contains(' ') {
        return None;
    }
    let host = rest.split(['/', '?', '#']).next().unwrap_or("").trim();
    if host.is_empty() {
        None
    } else {
        Some(redact_log_text(host))
    }
}

fn folder_row(folder: &ccr_sync::SyncFolder) -> SyncFolderRow {
    SyncFolderRow {
        name: redact_log_text(&folder.name),
        enabled: folder.enabled,
        local_path: redact_log_text(&folder.local_path),
        remote_path: redact_log_text(&folder.remote_path),
    }
}

fn project(config: SyncFoldersConfig) -> SyncLocal {
    let password_present = !config.webdav.password.is_empty();
    let username_present = !config.webdav.username.trim().is_empty();
    let url = if config.webdav.url.trim().is_empty() {
        SyncUrl::Missing
    } else if let Some(host) = host_only(&config.webdav.url) {
        SyncUrl::Host(host)
    } else {
        SyncUrl::Stored
    };
    let folders = config.folders.iter().map(folder_row).collect();
    SyncLocal {
        webdav_enabled: config.webdav.enabled,
        url,
        username_present,
        password_present,
        folders,
    }
}

/// The saved schema uses `folders`. Older examples use a `folder` array table.
fn legacy_folders(text: &str) -> Result<Vec<SyncFolderRow>, SyncError> {
    let value: toml::Value = toml::from_str(text).map_err(|_| SyncError::Unreadable)?;
    let Some(items) = value.get("folder") else {
        return Ok(Vec::new());
    };
    let items = items.as_array().ok_or(SyncError::Unreadable)?;
    items.iter().map(folder_from_value).collect()
}

fn folder_from_value(value: &toml::Value) -> Result<SyncFolderRow, SyncError> {
    let table = value.as_table().ok_or(SyncError::Unreadable)?;
    let text = |key: &str| {
        table
            .get(key)
            .and_then(toml::Value::as_str)
            .map(redact_log_text)
            .ok_or(SyncError::Unreadable)
    };
    let enabled = match table.get("enabled") {
        None => true,
        Some(item) => item.as_bool().ok_or(SyncError::Unreadable)?,
    };
    Ok(SyncFolderRow {
        name: text("name")?,
        enabled,
        local_path: text("local_path")?,
        remote_path: text("remote_path")?,
    })
}

pub(crate) fn read_file(path: &Path) -> Result<SyncSnapshot, SyncError> {
    if !path.is_file() {
        return Ok(SyncSnapshot::Absent);
    }
    let text = std::fs::read_to_string(path).map_err(|_| SyncError::Unreadable)?;
    let config: SyncFoldersConfig = toml::from_str(&text).map_err(|_| SyncError::Unreadable)?;
    let mut local = project(config);
    if local.folders.is_empty() {
        local.folders = legacy_folders(&text)?;
    }
    Ok(SyncSnapshot::Present(local))
}

fn canonical_file(source: &ConfigSource) -> Result<PathBuf, SyncError> {
    match source {
        ConfigSource::Local => SyncFolderManager::with_default()
            .map(|manager| manager.config_path().to_path_buf())
            .map_err(|_| SyncError::Storage),
        ConfigSource::Fixture { registry, .. } => {
            if !registry.is_absolute() {
                return Err(SyncError::Storage);
            }
            let root = registry.parent().ok_or(SyncError::Storage)?;
            Ok(root.join("sync_folders.toml"))
        }
    }
}

fn read_source(source: &ConfigSource) -> Result<SyncSnapshot, SyncError> {
    read_file(&canonical_file(source)?)
}

impl Services {
    pub fn sync_request(
        &self,
        key: RequestKey,
    ) -> Result<TaskSubscription<SyncSnapshot, SyncError>, AdmissionError> {
        let source = self.source.clone();
        let local = matches!(key.environment, EnvironmentId::Local(_));
        self.executor.read(key, move |_, _| {
            if !local {
                return Err(SyncError::Unsupported);
            }
            read_source(&source)
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn write(path: &Path, body: &str) {
        std::fs::write(path, body).unwrap();
    }

    #[test]
    fn missing_file_stays_absent_and_does_not_invent_a_host() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("sync_folders.toml");
        let snapshot = read_file(&path).unwrap();
        assert_eq!(snapshot, SyncSnapshot::Absent);
        assert!(!path.exists());
        assert!(!format!("{snapshot:?}").contains("jianguoyun"));
    }

    #[test]
    fn present_file_reports_storage_without_credentials_or_remote_status() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("sync_folders.toml");
        write(
            &path,
            r#"
version = "1.0"
[webdav]
enabled = true
url = "https://dav.example.test/dav/"
username = "sync-user-sentinel"
password = "sync-password-sentinel"
base_remote_path = "/ccr-sync"
[[folder]]
name = "claude-fixture"
local_path = "~/.claude"
remote_path = "/ccr-sync/claude"
enabled = true
"#,
        );
        let SyncSnapshot::Present(local) = read_file(&path).unwrap() else {
            panic!("expected config");
        };
        assert!(local.webdav_enabled);
        assert_eq!(local.url, SyncUrl::Host("dav.example.test".into()));
        assert!(local.username_present);
        assert!(local.password_present);
        assert_eq!(local.folders.len(), 1);
        assert_eq!(local.folders[0].name, "claude-fixture");
        let rendered = format!("{local:?}");
        assert!(!rendered.contains("sync-user-sentinel"));
        assert!(!rendered.contains("sync-password-sentinel"));
        assert!(!rendered.contains("connected"));
    }

    #[test]
    fn userinfo_url_and_invalid_file_hide_the_secret() {
        let root = tempfile::tempdir().unwrap();
        let hidden = root.path().join("hidden.toml");
        write(
            &hidden,
            r#"
[webdav]
enabled = false
url = "https://sync-user-sentinel:sync-password-sentinel@dav.example.test/dav/"
username = "sync-user-sentinel"
password = "sync-password-sentinel"
[[folder]]
name = "hidden-folder"
local_path = "/tmp/hidden"
remote_path = "/remote"
"#,
        );
        let SyncSnapshot::Present(local) = read_file(&hidden).unwrap() else {
            panic!("expected config");
        };
        assert_eq!(local.url, SyncUrl::Stored);
        assert!(!local.webdav_enabled);
        assert!(!format!("{local:?}").contains("sync-password-sentinel"));

        let broken = root.path().join("broken.toml");
        write(&broken, "password = \"sync-password-sentinel\"\n[webdav\n");
        let error = read_file(&broken).unwrap_err();
        assert_eq!(error, SyncError::Unreadable);
        assert!(!error.message().contains("sync-password-sentinel"));
    }

    #[test]
    fn fixture_path_ignores_an_absent_canonical_file() {
        let root = tempfile::tempdir().unwrap();
        write(
            &root.path().join("sync.toml"),
            "url = \"https://dav.jianguoyun.com/dav/\"\npassword = \"sync-password-sentinel\"\n",
        );
        let source = ConfigSource::Fixture {
            profiles: root.path().join("profiles.toml"),
            registry: root.path().join("registry.toml"),
            delay: std::time::Duration::ZERO,
        };
        let snapshot = read_source(&source).unwrap();
        assert_eq!(snapshot, SyncSnapshot::Absent);
        assert!(!format!("{snapshot:?}").contains("jianguoyun"));
        assert!(!format!("{snapshot:?}").contains("sync-password-sentinel"));
        assert!(!root.path().join("sync_folders.toml").exists());
    }

    #[test]
    fn saved_folders_key_is_read_without_the_password() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("sync_folders.toml");
        write(
            &path,
            r#"
version = "1.0"
[webdav]
enabled = true
url = "https://dav.example.test/dav/"
username = "sync-user-sentinel"
password = "sync-password-sentinel"
[[folders]]
name = "saved-folder"
local_path = "~/.claude"
remote_path = "/ccr-sync/claude"
enabled = true
"#,
        );
        let SyncSnapshot::Present(local) = read_file(&path).unwrap() else {
            panic!("expected config");
        };
        assert_eq!(local.folders.len(), 1);
        assert_eq!(local.folders[0].name, "saved-folder");
        assert_eq!(local.url, SyncUrl::Host("dav.example.test".into()));
        let rendered = format!("{local:?}");
        assert!(!rendered.contains("sync-user-sentinel"));
        assert!(!rendered.contains("sync-password-sentinel"));
    }
}
