//! Canonical local sync configuration and application-owned WebDAV operations.
use ccr_app::environments::EnvironmentPolicy;
use ccr_app::runtime::EnvironmentId;
mod operations;
mod remote;
#[cfg(test)]
pub(crate) mod test_support;
pub use operations::FolderResult;
pub use operations::{
    AssetDirection, AssetRequest, AssetRow, ConnectionTestStatus, FolderChange, FolderPatch,
    Operation, OperationKind, OperationState, PasswordUpdate, SettingsPatch, SyncOperations,
    SyncOutcome,
};

use std::path::{Path, PathBuf};

use ccr_core::core::log_redact::redact_log_text;
use ccr_sync::{
    SyncFolderManager, SyncFoldersConfig, WebDavConfig,
    sync::{assets::normalize_remote_base, normalize_href_identity},
};

use super::{ConfigSource, Services};
use ccr_app::runtime::{AdmissionError, RequestKey, TaskSubscription};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncError {
    AssetRead,
    AssetPassphrase,
    AssetLegacy,
    AssetDecrypt,
    AssetConflict,
    Unsupported,
    Storage,
    Unreadable,
    Invalid,
    Conflict,
    Busy,
    FixtureNetwork,
    Operation,
    Interrupted,
    LegacyFolders,
    LegacyRemaining,
    Remote,
    RemoteDatabase,
    RemotePartial,
    RemoteEmptyDirectory,
}

impl SyncError {
    pub fn message(self) -> &'static str {
        match self {
            Self::RemoteEmptyDirectory => {
                "Creating a new empty directory through SSH sync is unavailable. No files were published. / SSH 同步不支持新建纯空目标目录，未发布任何文件。"
            }
            Self::Remote => {
                "Cannot access sync data in this SSH session. Reconnect and retry. / 无法访问此 SSH 会话中的同步数据，请重连后重试。"
            }
            Self::RemoteDatabase => {
                "SSH sync cannot safely replace profile databases without a remote transaction owner. This asset was not transferred. / 缺少远端事务所有者，SSH 同步不能安全替换方案数据库，此资产未传输。"
            }
            Self::RemotePartial => {
                "SSH publication stopped after some files were written. Backups of replaced files remain on the SSH host. / SSH 发布在部分文件写入后中止。被替换文件的备份保留在 SSH 主机。"
            }
            Self::LegacyRemaining => {
                "Canonical WebDAV credentials were cleared, but legacy cleanup did not finish. Read the configuration and disconnect again."
            }
            Self::AssetRead => "The asset paths could not be inspected. / 无法检查资产路径。",
            Self::AssetPassphrase => "An operation passphrase is required for sensitive assets.",
            Self::AssetLegacy => {
                "Plaintext v1 import requires explicit permission. Remote data has not been re-encrypted."
            }
            Self::AssetDecrypt => {
                "Asset decryption failed. Check the passphrase and envelope. Local data was not replaced."
            }
            Self::AssetConflict => {
                "Both copies exist. Review the overwrite consequences before retrying."
            }
            Self::LegacyFolders => {
                "Legacy folder entries are read-only. Migrate them to the canonical folders schema with CCR before editing."
            }
            Self::Invalid => "Invalid WebDAV settings or unavailable folder.",
            Self::Conflict => {
                "Sync settings changed elsewhere. Read the configuration again before retrying."
            }
            Self::Busy => "A sync operation is already queued or running.",
            Self::FixtureNetwork => "Remote sync is disabled for fixture data.",
            Self::Operation => {
                "WebDAV operation failed. Check the account, permissions and selected paths."
            }
            Self::Interrupted => "The sync worker stopped without a confirmed result.",
            Self::Unsupported => {
                "This sync operation is unavailable in the current execution environment. / 当前执行环境不支持此同步操作。"
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
    Endpoint {
        origin: String,
        path_prefix: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncFolderRow {
    pub identity: Option<String>,
    pub name: String,
    pub enabled: bool,
    pub local_path: String,
    pub remote_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncLocal {
    pub revision: String,
    pub webdav_enabled: bool,
    pub url: SyncUrl,
    pub asset_remote_base: Option<String>,
    pub username_present: bool,
    pub password_present: bool,
    pub folders: Vec<SyncFolderRow>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncSnapshot {
    Absent,
    Present(SyncLocal),
}

fn displayable_webdav_path(path: &str, webdav: &WebDavConfig) -> bool {
    let username = webdav.username.trim();
    let safe_text = |text: &str| {
        !text.contains(['\\', ':', '@', '?', '#'])
            && !text.chars().any(char::is_control)
            && (username.is_empty() || !text.contains(username))
            && !webdav.password.occurs_in(text)
            && redact_log_text(text) == text
    };
    if !safe_text(path) {
        return false;
    }
    let Ok(mut decoded) = normalize_href_identity(path) else {
        return false;
    };
    // Preserve the trailing separator when checking for literal credentials.
    if path.ends_with('/') && decoded != "/" {
        decoded.push('/');
    }
    !decoded.contains('%') && safe_text(&decoded)
}

fn asset_base_for_display(webdav: &WebDavConfig) -> Option<String> {
    if !displayable_webdav_path(&webdav.base_remote_path, webdav) {
        return None;
    }
    let base = normalize_remote_base(&webdav.base_remote_path);
    let base = if base.is_empty() {
        "/".to_owned()
    } else {
        base
    };
    displayable_webdav_path(&base, webdav).then_some(base)
}

fn webdav_endpoint_for_display(webdav: &WebDavConfig) -> Option<SyncUrl> {
    let raw = webdav.url.trim();
    // Check the original spelling alongside parsed components, since URL parsing
    // can normalize backslashes, percent-encoded host text and dot segments.
    let endpoint = raw.split(['?', '#']).next()?;
    if endpoint.contains('\\') || endpoint.chars().any(char::is_control) {
        return None;
    }
    let (_, destination) = endpoint.split_once("://")?;
    let (authority, raw_path) = destination
        .find('/')
        .map_or((destination, "/"), |start| destination.split_at(start));
    let url = reqwest::Url::parse(raw).ok()?;
    if authority.is_empty()
        || authority.contains(['@', '%'])
        || !matches!(url.scheme(), "https" | "http")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return None;
    }
    let origin = url.origin().ascii_serialization();
    if webdav.password.occurs_in(authority)
        || webdav.password.occurs_in(&origin)
        || redact_log_text(&origin) != origin
    {
        return None;
    }
    let path = url.path();
    let path_prefix = (url.query().is_none()
        && url.fragment().is_none()
        && !webdav.password.occurs_in(endpoint)
        && !webdav.password.occurs_in(&format!("{origin}{path}"))
        && displayable_webdav_path(raw_path, webdav)
        && displayable_webdav_path(path, webdav))
    .then(|| path.to_owned());
    Some(SyncUrl::Endpoint {
        origin,
        path_prefix,
    })
}

fn folder_row(folder: &ccr_sync::SyncFolder) -> SyncFolderRow {
    SyncFolderRow {
        identity: Some(ccr_core::core::content_version_token(
            folder.name.as_bytes(),
        )),
        name: redact_log_text(&folder.name),
        enabled: folder.enabled,
        local_path: redact_log_text(&folder.local_path),
        remote_path: redact_log_text(&folder.remote_path),
    }
}

fn project(config: SyncFoldersConfig, revision: String) -> SyncLocal {
    let password_present = !config.webdav.password.is_empty();
    let username_present = !config.webdav.username.trim().is_empty();
    let url = if config.webdav.url.trim().is_empty() {
        SyncUrl::Missing
    } else {
        webdav_endpoint_for_display(&config.webdav).unwrap_or(SyncUrl::Stored)
    };
    let folders = config.folders.iter().map(folder_row).collect();
    SyncLocal {
        revision,
        webdav_enabled: config.webdav.enabled,
        url,
        asset_remote_base: asset_base_for_display(&config.webdav),
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
        identity: None,
        name: text("name")?,
        enabled,
        local_path: text("local_path")?,
        remote_path: text("remote_path")?,
    })
}

pub(crate) fn read_file(path: &Path) -> Result<SyncSnapshot, SyncError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(SyncSnapshot::Absent);
        }
        Err(_) => return Err(SyncError::Unreadable),
    };
    let config: SyncFoldersConfig = toml::from_str(&text).map_err(|_| SyncError::Unreadable)?;
    let mut local = project(
        config,
        ccr_core::core::content_version_token(text.as_bytes()),
    );
    if local.folders.is_empty() {
        local.folders = legacy_folders(&text)?;
    }
    Ok(SyncSnapshot::Present(local))
}

#[cfg(feature = "native-sync-fixture")]
pub(super) fn validate_native_fixture(
    profiles: &Path,
    registry: &Path,
    checkout: &Path,
) -> Result<(), &'static str> {
    use std::io::Read;

    const ERROR: &str = "Sync fixture requires a marked directory under outputs/desktop";
    const IDENTITY: &[u8] = b"ccr-gpui-sync-fixture-v1";

    let checkout = checkout.canonicalize().map_err(|_| ERROR)?;
    let outputs = checkout.join("outputs").join("desktop");
    if outputs.canonicalize().map_err(|_| ERROR)? != outputs {
        return Err(ERROR);
    }
    let root = registry.parent().ok_or(ERROR)?;
    let canonical_root = root.canonicalize().map_err(|_| ERROR)?;
    if !root.is_absolute()
        || canonical_root == outputs
        || !canonical_root.starts_with(&outputs)
        || profiles.parent() != Some(root)
        || profiles.file_name() != Some(std::ffi::OsStr::new("profiles.toml"))
        || registry.file_name() != Some(std::ffi::OsStr::new("config.toml"))
    {
        return Err(ERROR);
    }
    for name in ["profiles.toml", "sync_folders.toml", "sync.toml"] {
        let path = root.join(name);
        match path.symlink_metadata() {
            Ok(metadata)
                if metadata.is_file()
                    && path.canonicalize().map_err(|_| ERROR)? == canonical_root.join(name) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            _ => return Err(ERROR),
        }
    }
    for path in [
        registry,
        &root.join("profiles.db"),
        &root.join(".ccr-gpui-sync-fixture"),
    ] {
        if path.canonicalize().map_err(|_| ERROR)?
            != canonical_root.join(path.file_name().ok_or(ERROR)?)
            || !path.is_file()
        {
            return Err(ERROR);
        }
    }
    let mut identity = Vec::new();
    std::fs::File::open(root.join(".ccr-gpui-sync-fixture"))
        .map_err(|_| ERROR)?
        .take(IDENTITY.len() as u64 + 1)
        .read_to_end(&mut identity)
        .map_err(|_| ERROR)?;
    if identity != IDENTITY {
        return Err(ERROR);
    }
    Ok(())
}

#[cfg(all(test, feature = "native-sync-fixture"))]
#[test]
#[allow(clippy::unwrap_used)]
fn native_fixture_requires_an_isolated_catalog_and_exact_marker() {
    let checkout = tempfile::tempdir().unwrap();
    let root = checkout.path().join("outputs/desktop/sample");
    std::fs::create_dir_all(&root).unwrap();
    let profiles = root.join("profiles.toml");
    let registry = root.join("config.toml");
    std::fs::write(&registry, "").unwrap();
    std::fs::write(root.join("profiles.db"), "").unwrap();
    let marker = root.join(".ccr-gpui-sync-fixture");

    assert!(validate_native_fixture(&profiles, &registry, checkout.path()).is_err());
    std::fs::write(&marker, "ccr-gpui-sync-fixture-v1\n").unwrap();
    assert!(validate_native_fixture(&profiles, &registry, checkout.path()).is_err());
    std::fs::write(&marker, "ccr-gpui-sync-fixture-v1").unwrap();
    assert!(validate_native_fixture(&profiles, &registry, checkout.path()).is_ok());
    assert!(
        validate_native_fixture(
            &checkout.path().join("profiles.toml"),
            &registry,
            checkout.path()
        )
        .is_err()
    );
}

fn canonical_file(source: &ConfigSource) -> Result<PathBuf, SyncError> {
    match source {
        ConfigSource::Benchmark(layout) => Ok(layout.path("ccr/sync_folders.toml")),
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
        let local = EnvironmentPolicy::LocalOnly.admit(&key.environment).is_ok();
        let remote = (!local).then(|| self.remote_configuration_io(&key.environment));
        self.executor.read(key, move |cancel, _| {
            if !local {
                let io = remote
                    .ok_or(SyncError::Remote)?
                    .map_err(|_| SyncError::Remote)?;
                let handle =
                    tokio::runtime::Handle::try_current().map_err(|_| SyncError::Remote)?;
                return handle.block_on(remote::read(&io, cancel));
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
    fn invalid_config_path_reports_unreadable() {
        let root = tempfile::tempdir().unwrap();
        let path = root
            .path()
            .join("invalid\0directory")
            .join("sync_folders.toml");
        assert_eq!(read_file(&path), Err(SyncError::Unreadable));
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
        assert_eq!(
            local.url,
            SyncUrl::Endpoint {
                origin: "https://dav.example.test".into(),
                path_prefix: Some("/dav/".into()),
            }
        );
        assert_eq!(local.asset_remote_base.as_deref(), Some("/ccr-sync"));
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
    fn same_host_webdav_endpoints_keep_their_distinct_paths() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("sync_folders.toml");
        let read_endpoint = |scope| {
            write(
                &path,
                &format!(
                    "[webdav]\nenabled = true\nurl = 'https://dav.example.test/{scope}/'\nusername = 'fixture-user'\npassword = 'fixture-password'\nbase_remote_path = '/assets'\n"
                ),
            );
            let SyncSnapshot::Present(local) = read_file(&path).unwrap() else {
                panic!("expected config");
            };
            assert_eq!(local.asset_remote_base.as_deref(), Some("/assets"));
            local.url
        };
        for (scope, path_prefix) in [
            ("team-a", "/team-a/"),
            ("team-b", "/team-b/"),
            ("Team%20A", "/Team%20A/"),
            ("Team%20B", "/Team%20B/"),
            ("团队甲", "/%E5%9B%A2%E9%98%9F%E7%94%B2/"),
            ("团队乙", "/%E5%9B%A2%E9%98%9F%E4%B9%99/"),
        ] {
            assert_eq!(
                read_endpoint(scope),
                SyncUrl::Endpoint {
                    origin: "https://dav.example.test".into(),
                    path_prefix: Some(path_prefix.into()),
                },
            );
        }
    }

    #[test]
    fn unsafe_webdav_components_do_not_reach_the_snapshot() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("sync_folders.toml");
        let cases = [
            (
                "https://dav.example.test/fixture-password-suffix/",
                "/prefix-fixture-password",
                "fixture-password",
                "fixture-password",
                true,
            ),
            (
                r"https://dav.example.test/fixture\password/",
                r"/fixture\password/",
                r"fixture\password",
                "fixture/password",
                false,
            ),
            (
                "https://dav.example.test/space%20secret/",
                "/space%20secret/",
                "space secret",
                "space%20secret",
                true,
            ),
            (
                "https://dav.example.test/fixture%2Dcredential/",
                "/fixture%2Dcredential/",
                "fixture-credential/",
                "fixture%2Dcredential",
                true,
            ),
            (
                "https://dav.example.test/accounts/fixture-user-files/",
                "/accounts/fixture%2Duser-files",
                "fixture-password",
                "fixture-user",
                true,
            ),
            (
                "https://dav.example.test/accounts/%66ixture-user-files/",
                "/accounts/fixture-user-files",
                "fixture-password",
                "fixture-user",
                true,
            ),
            (
                "https://dav.example.test/unsafe%2Fsegment/",
                "/unsafe%5Csegment/",
                "fixture-password",
                "unsafe",
                true,
            ),
            (
                "https://dav.example.test/unsafe%0Asegment/",
                "/unsafe%00segment/",
                "fixture-password",
                "unsafe",
                true,
            ),
            (
                "https://dav.example.test/unsafe%252Fsegment/",
                "/unsafe%250Asegment/",
                "fixture-password",
                "unsafe",
                true,
            ),
            (
                "https://dav.example.test/unsafe/%2e%2e/other/",
                "/unsafe/%2e%2e/other/",
                "fixture-password",
                "other",
                true,
            ),
            (
                "https://dav.example.test/team/?token=query-sentinel#fragment-sentinel",
                "/unsafe?query-sentinel",
                "query-sentinel",
                "query-sentinel",
                true,
            ),
        ];
        for (url, base, password, hidden, host_visible) in cases {
            write(
                &path,
                &format!(
                    "[webdav]\nenabled=true\nurl='{url}'\nusername='fixture-user'\npassword='{password}'\nbase_remote_path='{base}'\n"
                ),
            );
            let SyncSnapshot::Present(local) = read_file(&path).unwrap() else {
                panic!("expected config");
            };
            assert_eq!(local.asset_remote_base, None);
            let expected = if host_visible {
                SyncUrl::Endpoint {
                    origin: "https://dav.example.test".into(),
                    path_prefix: None,
                }
            } else {
                SyncUrl::Stored
            };
            assert_eq!(local.url, expected);
            let snapshot = format!("{local:?}");
            assert!(!snapshot.contains(hidden));
            assert!(!snapshot.contains("fragment-sentinel"));
        }
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
base_remote_path = "/an-accidental-copy/sync-password-sentinel"
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
        assert_eq!(local.asset_remote_base, None);
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
        assert_eq!(
            local.url,
            SyncUrl::Endpoint {
                origin: "https://dav.example.test".into(),
                path_prefix: Some("/dav/".into()),
            }
        );
        let rendered = format!("{local:?}");
        assert!(!rendered.contains("sync-user-sentinel"));
        assert!(!rendered.contains("sync-password-sentinel"));
    }
}
