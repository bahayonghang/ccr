//! Synchronous compensation for an explicit set of operation files.
//! Preimages stay in memory. This is not an OS multi-file transaction.

use super::error::{CcrError, Result};
use super::guarded_write::{
    VersionedWriteOutcome, content_version_token, delete_guarded_versioned,
    restore_guarded_versioned,
};
use std::{
    cell::RefCell,
    marker::PhantomData,
    path::{Path, PathBuf},
    rc::Rc,
};

thread_local! { static ACTIVE: RefCell<Option<State>> = const { RefCell::new(None) }; }

pub(crate) struct Preimage {
    bytes: Vec<u8>,
    metadata: super::atomic_writer::FileMetadata,
}
struct Entry {
    path: PathBuf,
    before: Option<Preimage>,
    after: String,
}
struct State {
    paths: Vec<PathBuf>,
    versions: Vec<String>,
    entries: Vec<Entry>,
}

/// The Rc marker prevents Send/Sync: all work uses one synchronous thread.
/// Callers must not spawn work or await while the journal is active.
pub struct WriteJournal {
    finished: bool,
    thread: PhantomData<Rc<()>>,
}

fn absolute(path: &Path) -> Result<PathBuf> {
    #[cfg(windows)]
    let absolute = super::guarded_write::normalized_resource_path(path);
    #[cfg(not(windows))]
    let absolute = std::path::absolute(path);
    absolute.map_err(|_| CcrError::FileIoError("Cannot resolve operation path".into()))
}

impl WriteJournal {
    pub fn begin(paths: &[PathBuf]) -> Result<Self> {
        let paths = paths
            .iter()
            .map(|p| absolute(p))
            .collect::<Result<Vec<_>>>()?;
        let versions = paths
            .iter()
            .map(|p| match std::fs::read(p) {
                Ok(bytes) => Ok(content_version_token(&bytes)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
                Err(_) => Err(CcrError::FileIoError(
                    "Cannot prepare operation file version".into(),
                )),
            })
            .collect::<Result<Vec<_>>>()?;
        ACTIVE.with(|slot| {
            let mut state = slot.borrow_mut();
            if state.is_some() {
                return Err(CcrError::ConfigError(
                    "Nested write journal is not supported".into(),
                ));
            }
            *state = Some(State {
                paths,
                versions,
                entries: Vec::new(),
            });
            Ok(Self {
                finished: false,
                thread: PhantomData,
            })
        })
    }

    pub fn commit(mut self) {
        ACTIVE.with(|slot| {
            slot.borrow_mut().take();
        });
        self.finished = true;
    }

    pub fn changed_paths(&self) -> Vec<PathBuf> {
        ACTIVE.with(|slot| {
            slot.borrow()
                .as_ref()
                .map(|s| s.entries.iter().map(|e| e.path.clone()).collect())
                .unwrap_or_default()
        })
    }

    /// Bind a read-before-prepare token, for example a profile editor snapshot.
    pub fn expect_version(&self, path: &Path, version: String) -> Result<()> {
        let path = absolute(path)?;
        ACTIVE.with(|slot| {
            let mut state = slot.borrow_mut();
            let state = state
                .as_mut()
                .ok_or_else(|| CcrError::ConfigError("Inactive operation journal".into()))?;
            let index =
                state.paths.iter().position(|p| *p == path).ok_or_else(|| {
                    CcrError::ConfigError("File is outside operation scope".into())
                })?;
            state.versions[index] = version;
            Ok(())
        })
    }

    /// Verify every declared file against its latest expected version.
    pub fn verify(&self) -> Result<()> {
        ACTIVE.with(|slot| {
            let state = slot.borrow();
            if let Some(state) = state.as_ref() {
                for (index, path) in state.paths.iter().enumerate() {
                    let expected = state
                        .entries
                        .iter()
                        .rev()
                        .find(|entry| &entry.path == path)
                        .map(|entry| &entry.after)
                        .unwrap_or(&state.versions[index]);
                    let current = match std::fs::read(path) {
                        Ok(bytes) => content_version_token(&bytes),
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
                        Err(_) => {
                            return Err(CcrError::FileIoError(
                                "Cannot verify operation file".into(),
                            ));
                        }
                    };
                    if &current != expected {
                        return Err(CcrError::ConfigError(
                            "Operation file changed externally".into(),
                        ));
                    }
                }
            }
            Ok(())
        })
    }

    /// Return paths that require recovery. Retain newer external versions.
    pub fn rollback(mut self) -> Vec<PathBuf> {
        self.finished = true;
        rollback_active()
    }
}

impl Drop for WriteJournal {
    fn drop(&mut self) {
        if !self.finished {
            let failed = rollback_active();
            if !failed.is_empty() {
                tracing::error!(
                    count = failed.len(),
                    "Operation compensation requires recovery"
                );
            }
        }
    }
}

pub fn is_active() -> bool {
    ACTIVE.with(|slot| slot.borrow().is_some())
}

pub fn contains(path: &Path) -> bool {
    absolute(path).is_ok_and(|path| {
        ACTIVE.with(|slot| {
            slot.borrow()
                .as_ref()
                .is_some_and(|s| s.paths.contains(&path))
        })
    })
}

/// Called while the guarded writer holds its leaf lock.
pub(crate) fn before_write(path: &Path) -> Result<Option<Option<Preimage>>> {
    #[cfg(feature = "test-support")]
    fault::before(path)?;
    let path = absolute(path)?;
    if !contains(&path) {
        return Ok(None);
    }
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(_) => {
            return Err(CcrError::FileIoError(
                "Cannot read operation preimage".into(),
            ));
        }
    };
    let version = bytes
        .as_deref()
        .map(content_version_token)
        .unwrap_or_default();
    let unchanged = ACTIVE.with(|slot| {
        slot.borrow().as_ref().is_some_and(|s| {
            let expected = s
                .entries
                .iter()
                .rev()
                .find(|e| e.path == path)
                .map(|e| e.after.as_str())
                .or_else(|| {
                    s.paths
                        .iter()
                        .position(|p| *p == path)
                        .map(|index| s.versions[index].as_str())
                });
            expected == Some(version.as_str())
        })
    });
    if !unchanged {
        return Err(CcrError::ConfigError(
            "Operation file changed externally; retry after recovery".into(),
        ));
    }
    let before = bytes
        .map(|bytes| {
            super::atomic_writer::FileMetadata::capture(&path)
                .map(|metadata| Preimage { bytes, metadata })
        })
        .transpose()?;
    Ok(Some(before))
}

pub(crate) fn committed(path: &Path, before: Option<Option<Preimage>>, after: Option<&[u8]>) {
    if let Some(before) = before {
        let Ok(path) = absolute(path) else {
            return;
        };
        ACTIVE.with(|slot| {
            if let Some(state) = slot.borrow_mut().as_mut() {
                state.entries.push(Entry {
                    path: path.to_owned(),
                    before,
                    after: after.map(content_version_token).unwrap_or_default(),
                });
            }
        });
    }
}

fn rollback_active() -> Vec<PathBuf> {
    let state = ACTIVE.with(|slot| slot.borrow_mut().take());
    let mut failed = Vec::new();
    if let Some(state) = state {
        for entry in state.entries.into_iter().rev() {
            if failed.contains(&entry.path) {
                continue;
            }
            let restored = match entry.before {
                Some(before) => restore_guarded_versioned(
                    &entry.path,
                    &before.bytes,
                    &entry.after,
                    &before.metadata,
                ),
                None => delete_guarded_versioned(&entry.path, &entry.after),
            };
            if !matches!(restored, Ok(VersionedWriteOutcome::Written)) {
                failed.push(entry.path);
            }
        }
    }
    failed
}

/// Test-only thread-local fault injection. No environment or production switch.
#[cfg(feature = "test-support")]
pub mod fault {
    use super::*;
    type Hook = Box<dyn FnMut(&Path) -> Result<()>>;
    thread_local! { static HOOK: RefCell<Option<Hook>> = const { RefCell::new(None) }; }
    thread_local! { static PUBLISHED: RefCell<Option<Hook>> = const { RefCell::new(None) }; }
    pub struct PublishGuard;
    pub fn install_after_publish(hook: impl FnMut(&Path) -> Result<()> + 'static) -> PublishGuard {
        PUBLISHED.with(|slot| *slot.borrow_mut() = Some(Box::new(hook)));
        PublishGuard
    }
    pub(crate) fn after_publish(path: &Path) -> Result<()> {
        PUBLISHED.with(|slot| slot.borrow_mut().as_mut().map_or(Ok(()), |hook| hook(path)))
    }
    impl Drop for PublishGuard {
        fn drop(&mut self) {
            PUBLISHED.with(|slot| {
                slot.borrow_mut().take();
            });
        }
    }
    pub struct Guard;
    pub fn install(hook: impl FnMut(&Path) -> Result<()> + 'static) -> Guard {
        HOOK.with(|slot| *slot.borrow_mut() = Some(Box::new(hook)));
        Guard
    }
    pub(super) fn before(path: &Path) -> Result<()> {
        HOOK.with(|slot| slot.borrow_mut().as_mut().map_or(Ok(()), |hook| hook(path)))
    }
    impl Drop for Guard {
        fn drop(&mut self) {
            HOOK.with(|slot| {
                slot.borrow_mut().take();
            });
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::core::guarded_write::{WriteOptions, delete_guarded, write_guarded};

    #[test]
    fn journal_restores_repeated_write_delete_and_recreate() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        std::fs::write(&path, b"before").unwrap();
        let journal = WriteJournal::begin(std::slice::from_ref(&path)).unwrap();
        write_guarded(&path, b"first", &WriteOptions::default()).unwrap();
        delete_guarded(&path).unwrap();
        write_guarded(&path, b"recreated", &WriteOptions::default()).unwrap();
        assert!(journal.rollback().is_empty());
        assert_eq!(std::fs::read(path).unwrap(), b"before");
    }

    #[test]
    fn journal_removes_new_file_and_keeps_unlisted_file() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("new");
        let other = temp.path().join("other");
        let journal = WriteJournal::begin(std::slice::from_ref(&path)).unwrap();
        write_guarded(&path, b"new", &WriteOptions::default()).unwrap();
        write_guarded(&other, b"outside", &WriteOptions::default()).unwrap();
        assert!(journal.rollback().is_empty());
        assert!(!path.exists());
        assert!(other.exists());
    }

    #[test]
    fn journal_external_change_blocks_further_write_and_compensation() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        std::fs::write(&path, b"before").unwrap();
        let journal = WriteJournal::begin(std::slice::from_ref(&path)).unwrap();
        write_guarded(&path, b"ours", &WriteOptions::default()).unwrap();
        std::fs::write(&path, b"external").unwrap();
        assert!(write_guarded(&path, b"again", &WriteOptions::default()).is_err());
        assert!(journal.verify().is_err());
        assert_eq!(journal.rollback(), vec![absolute(&path).unwrap()]);
        assert_eq!(std::fs::read(path).unwrap(), b"external");
    }

    #[test]
    fn journal_scope_unwind_restores_and_rejects_nested_scope() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        std::fs::write(&path, b"before").unwrap();
        let result = std::panic::catch_unwind(|| {
            let _journal = WriteJournal::begin(std::slice::from_ref(&path)).unwrap();
            assert!(WriteJournal::begin(std::slice::from_ref(&path)).is_err());
            write_guarded(&path, b"changed", &WriteOptions::default()).unwrap();
            panic!("synthetic interrupted scope");
        });
        assert!(result.is_err());
        assert!(!is_active());
        assert_eq!(std::fs::read(path).unwrap(), b"before");
    }

    #[cfg(feature = "test-support")]
    #[test]
    fn journal_restores_post_publish_failure() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        std::fs::write(&path, b"before").unwrap();
        let journal = WriteJournal::begin(std::slice::from_ref(&path)).unwrap();
        let mut once = true;
        let _fault = fault::install_after_publish(move |_| {
            if once {
                once = false;
                return Err(CcrError::FileIoError(
                    "synthetic directory sync failure".into(),
                ));
            }
            Ok(())
        });
        assert!(write_guarded(&path, b"published", &WriteOptions::default()).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"published");
        assert!(journal.rollback().is_empty());
        assert_eq!(std::fs::read(path).unwrap(), b"before");
    }

    #[cfg(windows)]
    #[test]
    fn journal_restores_deleted_windows_dacl() {
        use crate::core::atomic_writer::{AtomicWriter, capture_windows_dacl};
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        AtomicWriter::new(&path)
            .secret(true)
            .write(b"before")
            .unwrap();
        // A inherited directory ACL differs from AtomicWriter's private default.
        let inherited = temp.path().join("inherited");
        std::fs::write(&inherited, b"fixture").unwrap();
        let original = capture_windows_dacl(&inherited).unwrap();
        crate::core::atomic_writer::apply_windows_dacl(&path, &original).unwrap();
        let original = capture_windows_dacl(&path).unwrap();
        let journal = WriteJournal::begin(std::slice::from_ref(&path)).unwrap();
        delete_guarded(&path).unwrap();
        assert!(journal.rollback().is_empty());
        assert!(
            capture_windows_dacl(&path).unwrap() == original,
            "Original DACL must be restored"
        );
        assert_eq!(std::fs::read(path).unwrap(), b"before");
    }

    #[cfg(windows)]
    #[test]
    fn journal_restores_deleted_windows_readonly() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        std::fs::write(&path, b"before").unwrap();
        let writable = std::fs::metadata(&path).unwrap().permissions();
        let mut readonly = writable.clone();
        readonly.set_readonly(true);
        std::fs::set_permissions(&path, readonly).unwrap();
        let journal = WriteJournal::begin(std::slice::from_ref(&path)).unwrap();
        delete_guarded(&path).unwrap();
        assert!(journal.rollback().is_empty());
        assert!(std::fs::metadata(&path).unwrap().permissions().readonly());
        assert_eq!(std::fs::read(&path).unwrap(), b"before");
        std::fs::set_permissions(&path, writable).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn journal_rollback_obeys_windows_alias_leaf_lock() {
        use crate::core::guarded_write::lock_resource_name;
        use crate::core::lock::LockManager;
        use crate::test_support::TestLockDirEnv;
        use std::sync::mpsc;
        use std::time::Duration;

        let temp = tempfile::tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(&temp.path().join("locks"));
        let path = temp.path().join("mixedCase");
        std::fs::write(&path, b"before").unwrap();
        let alias = PathBuf::from(
            std::fs::canonicalize(&path)
                .unwrap()
                .to_string_lossy()
                .to_uppercase(),
        );
        let (ready_tx, ready_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let operation_path = path.clone();
        let operation_alias = alias.clone();
        let operation = std::thread::spawn(move || {
            let journal = WriteJournal::begin(&[operation_path]).unwrap();
            write_guarded(&operation_alias, b"ours", &WriteOptions::default()).unwrap();
            ready_tx.send(()).unwrap();
            resume_rx.recv().unwrap();
            done_tx.send(journal.rollback()).unwrap();
        });
        ready_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let locks = LockManager::with_default_path().unwrap();
        let held = locks
            .lock_resource(&lock_resource_name(&alias), Duration::from_secs(1))
            .unwrap();
        resume_tx.send(()).unwrap();
        let early = done_rx.recv_timeout(Duration::from_millis(150));
        let blocked = matches!(early, Err(mpsc::RecvTimeoutError::Timeout));
        drop(held);
        let failures =
            early.unwrap_or_else(|_| done_rx.recv_timeout(Duration::from_secs(5)).unwrap());
        operation.join().unwrap();
        assert!(
            blocked,
            "Compensation must wait for the alias writer's leaf lock"
        );
        assert!(failures.is_empty());
        assert_eq!(std::fs::read(&path).unwrap(), b"before");
    }

    #[cfg(windows)]
    #[test]
    fn guarded_write_obeys_windows_alias_leaf_lock() {
        use crate::core::guarded_write::{lock_resource_name, write_guarded_versioned};
        use crate::core::lock::LockManager;
        use crate::test_support::TestLockDirEnv;
        use std::time::Duration;

        let temp = tempfile::tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(&temp.path().join("locks"));
        let path = temp.path().join("mixedCase");
        std::fs::write(&path, b"before").unwrap();
        let alias = PathBuf::from(
            std::fs::canonicalize(&path)
                .unwrap()
                .to_string_lossy()
                .to_uppercase(),
        );
        let locks = LockManager::with_default_path().unwrap();
        let held = locks
            .lock_resource(&lock_resource_name(&path), Duration::from_secs(1))
            .unwrap();
        let writer = std::thread::spawn(move || {
            write_guarded_versioned(
                &alias,
                b"external",
                &content_version_token(b"before"),
                &WriteOptions {
                    lock_timeout: Duration::from_millis(100),
                    ..Default::default()
                },
            )
        });
        let result = writer.join().unwrap();
        drop(held);
        assert!(
            matches!(result, Err(CcrError::LockTimeout(_))),
            "Alias writer must not bypass the held leaf lock"
        );
        assert_eq!(std::fs::read(path).unwrap(), b"before");
    }

    #[cfg(windows)]
    #[test]
    fn journal_covers_windows_case_and_verbatim_aliases() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("mixedCase");
        std::fs::write(&path, b"before").unwrap();
        let journal = WriteJournal::begin(std::slice::from_ref(&path)).unwrap();
        let alias = PathBuf::from(
            std::fs::canonicalize(&path)
                .unwrap()
                .to_string_lossy()
                .to_uppercase(),
        );
        assert!(contains(&alias));
        write_guarded(&alias, b"ours", &WriteOptions::default()).unwrap();
        assert!(journal.rollback().is_empty());
        assert_eq!(std::fs::read(path).unwrap(), b"before");
    }

    #[cfg(unix)]
    #[test]
    fn journal_restores_deleted_unix_mode() {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        std::fs::write(&path, b"before").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o400)).unwrap();
        let journal = WriteJournal::begin(std::slice::from_ref(&path)).unwrap();
        delete_guarded(&path).unwrap();
        assert!(journal.rollback().is_empty());
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o400
        );
    }

    #[test]
    fn journal_rejects_source_change_before_first_write() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("source");
        std::fs::write(&path, b"prepared").unwrap();
        let token = content_version_token(b"prepared");
        std::fs::write(&path, b"external").unwrap();
        let journal = WriteJournal::begin(std::slice::from_ref(&path)).unwrap();
        journal.expect_version(&path, token).unwrap();
        assert!(journal.verify().is_err());
        assert!(write_guarded(&path, b"ours", &WriteOptions::default()).is_err());
        assert!(journal.rollback().is_empty());
        assert_eq!(std::fs::read(path).unwrap(), b"external");
    }

    #[test]
    fn journal_uses_relative_paths_and_does_not_cross_threads() {
        let temp = tempfile::tempdir_in(std::env::current_dir().unwrap()).unwrap();
        let path = temp.path().join("file");
        let relative = path
            .strip_prefix(std::env::current_dir().unwrap())
            .unwrap()
            .to_owned();
        std::fs::write(&path, b"before").unwrap();
        let journal = WriteJournal::begin(std::slice::from_ref(&relative)).unwrap();
        std::thread::spawn(|| assert!(!is_active())).join().unwrap();
        write_guarded(&relative, b"ours", &WriteOptions::default()).unwrap();
        journal.verify().unwrap();
        assert!(journal.rollback().is_empty());
        assert_eq!(std::fs::read(path).unwrap(), b"before");
    }

    #[test]
    fn journal_rejects_async_write_before_worker_dispatch() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file");
        let journal = WriteJournal::begin(std::slice::from_ref(&path)).unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        assert!(
            runtime
                .block_on(crate::core::guarded_write::write_guarded_async(
                    &path,
                    b"new".to_vec(),
                    WriteOptions::default()
                ))
                .is_err()
        );
        assert!(!path.exists());
        journal.commit();
    }
}
