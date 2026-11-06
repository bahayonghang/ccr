//! Explicit-path WebDAV disconnection shared by desktop adapters.
use ccr_core::core::guarded_write::{
    BackupPolicy, VersionedWriteOutcome, WriteOptions, backup_guarded, content_version_token,
    delete_guarded_versioned, write_guarded_versioned,
};
use std::path::{Path, PathBuf};

/// Only identities and versions are retained while the user confirms; no credentials.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClearSession {
    canonical: PathBuf,
    legacy: PathBuf,
    canonical_version: String,
    legacy_version: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClearError {
    Invalid,
    Storage,
    Conflict,
    /// Canonical credentials were cleared; legacy cleanup failed or conflicted.
    LegacyRemaining,
}

fn read(path: &Path) -> Result<(Vec<u8>, String), ClearError> {
    if !path.is_absolute() {
        return Err(ClearError::Invalid);
    }
    for ancestor in path.ancestors() {
        match std::fs::symlink_metadata(ancestor) {
            Ok(metadata) => {
                let linked = metadata.file_type().is_symlink();
                #[cfg(windows)]
                let linked = {
                    use std::os::windows::fs::MetadataExt;
                    linked || metadata.file_attributes() & 0x400 != 0
                };
                if linked {
                    return Err(ClearError::Invalid);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(ClearError::Storage),
        }
    }
    match std::fs::read(path) {
        Ok(bytes) => {
            let version = content_version_token(&bytes);
            Ok((bytes, version))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok((Vec::new(), String::new()))
        }
        Err(_) => Err(ClearError::Storage),
    }
}

pub fn prepare(canonical: &Path, legacy: &Path) -> Result<ClearSession, ClearError> {
    if ccr_core::core::guarded_write::normalized_resource_path(canonical)
        .map_err(|_| ClearError::Invalid)?
        == ccr_core::core::guarded_write::normalized_resource_path(legacy)
            .map_err(|_| ClearError::Invalid)?
    {
        return Err(ClearError::Invalid);
    }
    Ok(ClearSession {
        canonical: canonical.to_owned(),
        legacy: legacy.to_owned(),
        canonical_version: read(canonical)?.1,
        legacy_version: read(legacy)?.1,
    })
}

impl ClearSession {
    pub fn matches(&self, canonical: &Path, legacy: &Path, revision: &str) -> bool {
        self.canonical == canonical && self.legacy == legacy && self.canonical_version == revision
    }

    /// Two files cannot be atomically replaced together. Clear canonical first and
    /// report partial cleanup explicitly; never restore old credentials on failure.
    pub fn clear(&self) -> Result<(), ClearError> {
        let (bytes, canonical_version) = read(&self.canonical)?;
        if canonical_version != self.canonical_version
            || read(&self.legacy)?.1 != self.legacy_version
        {
            return Err(ClearError::Conflict);
        }
        let mut value: toml::Value = if canonical_version.is_empty() {
            toml::Value::Table(toml::map::Map::new())
        } else {
            toml::from_str(std::str::from_utf8(&bytes).map_err(|_| ClearError::Invalid)?)
                .map_err(|_| ClearError::Invalid)?
        };
        let webdav = value
            .as_table_mut()
            .ok_or(ClearError::Invalid)?
            .entry("webdav")
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()))
            .as_table_mut()
            .ok_or(ClearError::Invalid)?;
        for key in ["url", "username", "password"] {
            webdav.insert(key.into(), "".into());
        }
        webdav.insert("enabled".into(), false.into());
        webdav.insert("auto_sync".into(), false.into());
        webdav.insert("base_remote_path".into(), "/ccr-sync".into());
        let content = toml::to_string_pretty(&value).map_err(|_| ClearError::Invalid)?;
        let backup = BackupPolicy::SameDir {
            tag: Some("disconnect".into()),
        };
        let result = write_guarded_versioned(
            &self.canonical,
            content.as_bytes(),
            &self.canonical_version,
            &WriteOptions {
                secret: true,
                backup: backup.clone(),
                ..Default::default()
            },
        )
        .map_err(|_| ClearError::Storage)?;
        if result == VersionedWriteOutcome::Conflict {
            return Err(ClearError::Conflict);
        }
        // The legacy version is checked again under its existing writer lock.
        read(&self.legacy).map_err(|_| ClearError::LegacyRemaining)?;
        backup_guarded(&self.legacy, &backup).map_err(|_| ClearError::LegacyRemaining)?;
        match delete_guarded_versioned(&self.legacy, &self.legacy_version) {
            Ok(VersionedWriteOutcome::Written) => Ok(()),
            _ => Err(ClearError::LegacyRemaining),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn disconnect_preserves_extensions_and_folders_with_or_without_legacy() {
        for legacy_exists in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let canonical = root.path().join("sync_folders.toml");
            let legacy = root.path().join("sync.toml");
            std::fs::write(&canonical, "future=42\n[webdav]\nenabled=true\nurl='https://fixture.invalid/'\nusername='fixture'\npassword='secret-fixture'\nfuture='preserved'\n[[folders]]\nname='existing'\nlocal_path='/fixture'\nremote_path='/existing'\n").unwrap();
            if legacy_exists {
                std::fs::write(&legacy, "legacy-secret-fixture").unwrap();
            }
            let session = prepare(&canonical, &legacy).unwrap();
            assert!(!format!("{session:?}").contains("secret-fixture"));
            let result = ccr_core::core::LockManager::with_scoped_directory(
                &root.path().join("locks"),
                || Ok(session.clear()),
            )
            .unwrap();
            assert_eq!(result, Ok(()));
            assert!(!legacy.exists());
            let stored: toml::Value =
                toml::from_str(&std::fs::read_to_string(&canonical).unwrap()).unwrap();
            assert_eq!(stored["future"].as_integer(), Some(42));
            assert_eq!(stored["folders"][0]["name"].as_str(), Some("existing"));
            assert_eq!(stored["webdav"]["future"].as_str(), Some("preserved"));
            assert_eq!(stored["webdav"]["password"].as_str(), Some(""));
            assert_eq!(stored["webdav"]["enabled"].as_bool(), Some(false));
            assert!(
                std::fs::read_dir(root.path())
                    .unwrap()
                    .filter_map(Result::ok)
                    .any(|entry| {
                        entry
                            .file_name()
                            .to_string_lossy()
                            .starts_with("sync_folders.toml.disconnect_")
                            && std::fs::read_to_string(entry.path())
                                .unwrap()
                                .contains("secret-fixture")
                    })
            );
        }
    }

    #[test]
    fn either_changed_revision_refuses_disconnection_before_mutation() {
        for change_legacy in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let canonical = root.path().join("sync_folders.toml");
            let legacy = root.path().join("sync.toml");
            std::fs::write(&canonical, "[webdav]\npassword='secret-fixture'\n").unwrap();
            std::fs::write(&legacy, "legacy-secret-fixture").unwrap();
            let session = prepare(&canonical, &legacy).unwrap();
            let changed = if change_legacy { &legacy } else { &canonical };
            std::fs::write(changed, "external edit").unwrap();
            let canonical_before = std::fs::read(&canonical).unwrap();
            let legacy_before = std::fs::read(&legacy).unwrap();
            assert_eq!(session.clear(), Err(ClearError::Conflict));
            assert_eq!(std::fs::read(&canonical).unwrap(), canonical_before);
            assert_eq!(std::fs::read(&legacy).unwrap(), legacy_before);
        }
    }
}
