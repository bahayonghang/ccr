//! Bounded, read-only session materialization. No provider parser or SQL lives here.
use crate::environments::{
    config_mirror::validate_materialized_paths,
    io::{EntryKind, EnvironmentIo, Error},
};
use ccr_db::database::{self, DbPool};
use ccr_store::sessions::AgentSessionProviderRegistry;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::Path,
    time::{Duration, UNIX_EPOCH},
};
use tokio_util::sync::CancellationToken;

pub struct SessionSnapshot {
    // 先关闭 SQLite 池，再删除私有快照目录。
    pub pool: DbPool,
    pub registry: AgentSessionProviderRegistry,
    io: EnvironmentIo,
    directory: tempfile::TempDir,
    pub report: super::refresh::RefreshReport,
    versions: BTreeMap<String, String>,
}
impl SessionSnapshot {
    /// Each publication owns a fresh private directory under a hashed host cache.
    /// It is never reused after reconnect, treated as a remote mtime, or written back.
    pub async fn capture(
        io: EnvironmentIo,
        cache: &Path,
        cancel: CancellationToken,
    ) -> Result<Self, Error> {
        let host_cache = io.cache_root(cache)?;
        reject_links(&host_cache)?;
        std::fs::create_dir_all(&host_cache).map_err(|_| Error::Io)?;
        reject_links(&host_cache)?;
        ccr_core::core::guarded_write::restrict_owner_only_directory(&host_cache)
            .map_err(|_| Error::Io)?;
        let directory = tempfile::Builder::new()
            .prefix("sessions-v1-")
            .tempdir_in(&host_cache)
            .map_err(|_| Error::Io)?;
        ccr_core::core::guarded_write::restrict_owner_only_directory(directory.path())
            .map_err(|_| Error::Io)?;
        let mut pending = AgentSessionProviderRegistry::snapshot_roots()
            .into_iter()
            .map(|root| (root, 0))
            .collect::<Vec<_>>();
        let mut files = BTreeSet::new();
        let mut visited = 0;
        while let Some((path, depth)) = pending.pop() {
            visited += 1;
            if visited > 8192 || depth > 12 {
                return Err(Error::TooLarge);
            }
            let Some(entries) = io.entries(&path, cancel.clone()).await? else {
                continue;
            };
            for entry in entries {
                let child = format!("{path}/{}", entry.name);
                match entry.kind {
                    EntryKind::Directory => pending.push((child, depth + 1)),
                    EntryKind::File if AgentSessionProviderRegistry::snapshot_input(&child) => {
                        files.insert(child);
                    }
                    EntryKind::Unsupported => return Err(Error::InvalidPath),
                    _ => {}
                }
            }
            if files.len() > 4096 {
                return Err(Error::TooLarge);
            }
        }
        validate_materialized_paths(files.iter().map(String::as_str))?;
        let mut total = 0usize;
        let mut versions = BTreeMap::new();
        for path in files {
            for suffix in ["-wal", "-shm", "-journal"] {
                if path.ends_with(".db")
                    && io
                        .metadata(&(path.clone() + suffix), cancel.clone())
                        .await?
                        .is_some()
                {
                    return Err(Error::Busy);
                }
            }
            let metadata = io
                .metadata(&path, cancel.clone())
                .await?
                .ok_or(Error::Conflict)?;
            let modified = metadata.1.ok_or(Error::Io)?;
            let bytes = io.read_session_bytes(&path, cancel.clone()).await?;
            versions.insert(path.clone(), ccr_core::core::content_version_token(&bytes));
            total = total.checked_add(bytes.len()).ok_or(Error::TooLarge)?;
            if total > 128 * 1024 * 1024 {
                return Err(Error::TooLarge);
            }
            let destination = directory.path().join(&path);
            ccr_core::core::LockManager::with_scoped_directory(
                &directory.path().join(".locks"),
                || {
                    ccr_core::core::write_guarded(
                        &destination,
                        &bytes,
                        &ccr_core::core::WriteOptions {
                            secret: true,
                            ..Default::default()
                        },
                    )
                },
            )
            .map_err(|_| Error::Io)?;
            let file = std::fs::OpenOptions::new()
                .write(true)
                .open(destination)
                .map_err(|_| Error::Io)?;
            file.set_times(
                std::fs::FileTimes::new()
                    .set_modified(UNIX_EPOCH + Duration::from_millis(modified)),
            )
            .map_err(|_| Error::Io)?;
            if io.metadata(&path, cancel.clone()).await? != Some(metadata) {
                return Err(Error::Conflict);
            }
            // SFTP mtime 通常只有秒精度；SQLite 同尺寸原位写入还需要内容复读。
            // 这证明两次稳定观测，不宣称服务端事务快照。
            if path.ends_with(".db") && io.read_session_bytes(&path, cancel.clone()).await? != bytes
            {
                return Err(Error::Conflict);
            }
            for suffix in ["-wal", "-shm", "-journal"] {
                if path.ends_with(".db")
                    && io
                        .metadata(&(path.clone() + suffix), cancel.clone())
                        .await?
                        .is_some()
                {
                    return Err(Error::Busy);
                }
            }
        }
        if cancel.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let pool = database::create_pool(&directory.path().join("archive.db"), None)
            .map_err(|_| Error::Io)?;
        // 空迁移 HOME 只属于本快照，不读取真实用户的旧数据库。
        let migration = directory.path().join("migration");
        std::fs::create_dir(&migration).map_err(|_| Error::Io)?;
        let connection = pool.get().map_err(|_| Error::Io)?;
        database::migrations::run_all_migrations(&connection, &migration).map_err(|_| Error::Io)?;
        drop(connection);
        let registry = AgentSessionProviderRegistry::new(directory.path().to_path_buf())
            .without_environment_overrides();
        let report = super::refresh::refresh_archive_with_registry(&pool, &registry)
            .map_err(|_| Error::Io)?;
        if cancel.is_cancelled() {
            return Err(Error::Cancelled);
        }
        Ok(Self {
            pool,
            registry,
            io,
            directory,
            report,
            versions,
        })
    }
    pub fn io(&self) -> &EnvironmentIo {
        &self.io
    }
    pub fn root(&self) -> &Path {
        self.directory.path()
    }
    /// A local cache edit never becomes fresh authority over the remote source.
    pub fn verify_source(&self, path: &Path) -> Result<(), Error> {
        self.io.check_authority()?;
        reject_links(path)?;
        let relative = path
            .strip_prefix(self.root())
            .map_err(|_| Error::InvalidPath)?
            .to_string_lossy()
            .replace('\\', "/");
        let expected = self.versions.get(&relative).ok_or(Error::InvalidPath)?;
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .map_err(|_| Error::Io)?
            .take(64 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| Error::Io)?;
        if bytes.len() > 64 * 1024 * 1024
            || ccr_core::core::content_version_token(&bytes) != *expected
        {
            return Err(Error::Conflict);
        }
        Ok(())
    }
}
fn reject_links(path: &Path) -> Result<(), Error> {
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
                    return Err(Error::InvalidPath);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => return Err(Error::Io),
        }
    }
    Ok(())
}
