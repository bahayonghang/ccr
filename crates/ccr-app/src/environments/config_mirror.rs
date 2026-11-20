//! Private, bounded configuration materialization for synchronous domain owners.
//! Callers supply an explicit file allowlist, never an arbitrary HOME tree.
//! Remote paths remain relative POSIX strings; only the private mirror uses OS paths.

use super::io::{ByteSnapshot, EnvironmentIo, Error};
use ccr_core::core::{WriteOptions, write_guarded};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use tokio_util::sync::CancellationToken;

pub struct ConfigMirror {
    io: EnvironmentIo,
    directory: tempfile::TempDir,
    files: BTreeMap<String, ByteSnapshot>,
    auth_directory: Option<String>,
}

/// Identifies a failed read within the caller's own allowlist. All other
/// failures (including verification conflicts) remain whole-capture failures.
pub struct CaptureError {
    pub path: Option<String>,
    pub reason: Error,
}

impl From<Error> for CaptureError {
    fn from(reason: Error) -> Self {
        Self { path: None, reason }
    }
}

impl ConfigMirror {
    /// Capture only declared inputs, rejecting any live SQLite sidecar.
    pub async fn capture(
        io: EnvironmentIo,
        paths: BTreeSet<String>,
        auth_directory: Option<String>,
        cancel: CancellationToken,
    ) -> Result<Self, Error> {
        Self::capture_inputs(io, paths, auth_directory, cancel)
            .await
            .map_err(|error| error.reason)
    }

    pub async fn capture_inputs(
        io: EnvironmentIo,
        mut paths: BTreeSet<String>,
        auth_directory: Option<String>,
        cancel: CancellationToken,
    ) -> Result<Self, CaptureError> {
        let _guard = io.configuration_lock(cancel.clone()).await?;
        if let Some(auth) = &auth_directory {
            for name in io.list(auth, cancel.clone()).await? {
                if name.ends_with(".json") && !name.contains(['/', '\\']) {
                    paths.insert(format!("{auth}/{name}"));
                }
            }
        }
        if paths.len() > 256 {
            return Err(Error::TooLarge.into());
        }
        validate_materialized_paths(paths.iter().map(String::as_str))?;
        for path in paths.iter().filter(|path| is_sidecar(path)) {
            if io.metadata(path, cancel.clone()).await?.is_some() {
                return Err(Error::Busy.into());
            }
        }
        let directory = tempfile::Builder::new()
            .prefix("ccr-configuration-")
            .tempdir()
            .map_err(|_| Error::Io)?;
        ccr_core::core::guarded_write::restrict_owner_only_directory(directory.path())
            .map_err(|_| Error::Io)?;
        let mut files = BTreeMap::new();
        for path in paths {
            let snapshot = io
                .read_bytes(&path, cancel.clone())
                .await
                .map_err(|reason| CaptureError {
                    path: Some(path.clone()),
                    reason,
                })?;
            if snapshot.exists {
                if is_sidecar(&path) {
                    return Err(Error::Busy.into());
                }
                ccr_core::core::LockManager::with_scoped_directory(
                    &directory.path().join(".ccr-local-locks"),
                    || {
                        write_guarded(
                            &directory.path().join(&path),
                            &snapshot.content,
                            &WriteOptions {
                                secret: true,
                                ..Default::default()
                            },
                        )
                    },
                )
                .map_err(|_| Error::Io)?;
            }
            files.insert(path, snapshot);
        }
        let mirror = Self {
            io,
            directory,
            files,
            auth_directory,
        };
        mirror.verify_inputs(cancel).await?;
        for path in mirror
            .files
            .keys()
            .filter(|path| path.ends_with("profiles.db"))
        {
            let file = mirror.root().join(path);
            if file.exists() {
                ccr_config::validate_profile_catalog_file(&file).map_err(|_| Error::Io)?;
            }
        }
        Ok(mirror)
    }

    pub fn root(&self) -> &Path {
        self.directory.path()
    }
    pub fn io(&self) -> &EnvironmentIo {
        &self.io
    }

    /// Recheck every input, including absent sidecars, before any publication.
    pub async fn verify(&self, cancel: CancellationToken) -> Result<(), Error> {
        self.verify_inputs(cancel)
            .await
            .map_err(|error| error.reason)
    }

    async fn verify_inputs(&self, cancel: CancellationToken) -> Result<(), CaptureError> {
        for (path, expected) in &self.files {
            let actual = self
                .io
                .read_bytes(path, cancel.clone())
                .await
                .map_err(|reason| CaptureError {
                    path: Some(path.clone()),
                    reason,
                })?;
            if is_sidecar(path) && actual.exists {
                return Err(Error::Busy.into());
            }
            if actual.version() != expected.version() || actual.exists != expected.exists {
                return Err(Error::Conflict.into());
            }
        }
        Ok(())
    }

    /// Publish only allowed differences. A partial publication is reported
    /// explicitly; preimage backups remain on the remote host for recovery.
    pub async fn publish(&self, cancel: CancellationToken) -> Result<(), Error> {
        self.publish_outputs(BTreeSet::new(), cancel).await
    }

    /// Publish explicitly selected owner outputs, refusing pre-existing unobserved files.
    pub async fn publish_outputs(
        &self,
        outputs: BTreeSet<String>,
        cancel: CancellationToken,
    ) -> Result<(), Error> {
        validate_materialized_paths(outputs.iter().map(String::as_str))?;
        let _guard = self.io.configuration_lock(cancel.clone()).await?;
        self.verify(cancel.clone()).await?;
        let mut paths = self
            .files
            .keys()
            .filter(|path| !is_sidecar(path))
            .cloned()
            .collect::<BTreeSet<_>>();
        if let Some(auth) = &self.auth_directory {
            if let Some(platform) = auth.strip_suffix("/auth") {
                let directory = self.root().join(platform);
                if directory.exists() {
                    for entry in std::fs::read_dir(directory).map_err(|_| Error::Io)? {
                        let entry = entry.map_err(|_| Error::Io)?;
                        let name = entry
                            .file_name()
                            .into_string()
                            .map_err(|_| Error::InvalidPath)?;
                        if name.starts_with("profiles.toml.")
                            && name.ends_with(".bak")
                            && entry.file_type().map_err(|_| Error::Io)?.is_file()
                        {
                            paths.insert(format!("{platform}/{name}"));
                        }
                    }
                }
            }
            let directory = self.root().join(auth);
            if directory.exists() {
                for entry in std::fs::read_dir(directory).map_err(|_| Error::Io)? {
                    let entry = entry.map_err(|_| Error::Io)?;
                    let name = entry
                        .file_name()
                        .into_string()
                        .map_err(|_| Error::InvalidPath)?;
                    if name.ends_with(".json")
                        && entry.file_type().map_err(|_| Error::Io)?.is_file()
                    {
                        paths.insert(format!("{auth}/{name}"));
                    }
                }
            }
        }
        paths.extend(outputs);
        let mut changes = Vec::new();
        for path in paths {
            let expected = match self.files.get(&path) {
                Some(expected) => expected.clone(),
                None => {
                    let expected = self.io.read_bytes(&path, cancel.clone()).await?;
                    if expected.exists {
                        return Err(Error::Conflict);
                    }
                    expected
                }
            };
            let mut content = match std::fs::read(self.root().join(&path)) {
                Ok(bytes) => Some(bytes),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(_) => return Err(Error::Io),
            };
            if content.as_deref() == expected.exists.then_some(expected.content.as_slice()) {
                continue;
            }
            if path.ends_with("profiles.db") && content.is_some() {
                content = Some(
                    ccr_core::core::LockManager::with_scoped_directory(
                        &self.root().join(".ccr-local-locks"),
                        || ccr_config::snapshot_profile_catalog(&self.root().join(&path)),
                    )
                    .map_err(|_| Error::Io)?,
                );
            }
            changes.push((path, expected, content));
        }
        // 运行文件先提交，目录/注册表最后提交；任何中断都不能伪称整个领域操作成功。
        changes.sort_by_key(|(path, _, _)| {
            (
                if path.ends_with(".bak") {
                    0
                } else if path.starts_with(".ccr/") {
                    2
                } else {
                    1
                },
                path.clone(),
            )
        });
        for (index, (_, expected, content)) in changes.iter().enumerate() {
            if let Err(error) = self
                .io
                .write_bytes(expected, content.as_deref(), cancel.clone())
                .await
            {
                return Err(if index > 0 { Error::Partial } else { error });
            }
        }
        Ok(())
    }
}

/// Reject names that cannot retain their remote identity on a Windows mirror.
/// The same portable subset is enforced on every desktop host.
pub fn validate_materialized_paths<'a>(
    paths: impl IntoIterator<Item = &'a str>,
) -> Result<(), Error> {
    let mut identities = BTreeSet::new();
    for path in paths {
        for part in path.split('/') {
            if part.is_empty()
                || matches!(part, "." | "..")
                || part.ends_with('.')
                || !part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
            {
                return Err(Error::UnrepresentablePath);
            }
            let stem = part
                .split('.')
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase();
            if matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
                || stem
                    .strip_prefix("com")
                    .or_else(|| stem.strip_prefix("lpt"))
                    .is_some_and(|number| {
                        matches!(number, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
                    })
            {
                return Err(Error::UnrepresentablePath);
            }
        }
        if !identities.insert(path.to_ascii_lowercase()) {
            return Err(Error::UnrepresentablePath);
        }
    }
    Ok(())
}

fn is_sidecar(path: &str) -> bool {
    ["profiles.db-wal", "profiles.db-shm", "profiles.db-journal"]
        .iter()
        .any(|suffix| path.ends_with(suffix))
}

#[cfg(test)]
mod tests;
