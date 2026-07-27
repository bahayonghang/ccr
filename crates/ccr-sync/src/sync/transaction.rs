use ccr_core::core::error::{CcrError, Result};
#[cfg(not(windows))]
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct PullTransaction {
    target: PathBuf,
    staging: PathBuf,
    committed: bool,
}

impl PullTransaction {
    pub fn begin(target: &Path) -> Result<Self> {
        let parent = target
            .parent()
            .ok_or_else(|| transaction_error("target", "同步目标没有父目录"))?;
        fs::create_dir_all(parent).map_err(|error| {
            transaction_error("mkdir", &format!("创建同步目标父目录失败: {error}"))
        })?;
        let staging = temporary_sibling_path(target, "stage");
        remove_path_if_exists(&staging)?;
        Ok(Self {
            target: target.to_path_buf(),
            staging,
            committed: false,
        })
    }

    pub fn staging_path(&self) -> &Path {
        &self.staging
    }

    pub async fn commit(mut self) -> Result<Option<PathBuf>> {
        let staging = self.staging.clone();
        let target = self.target.clone();
        let outcome = tokio::task::spawn_blocking(move || {
            commit_staged_path(&staging, &target, CommitFailpoint::None)
        })
        .await
        .map_err(|error| transaction_error("join", &format!("事务提交任务失败: {error}")))?;
        if outcome.is_ok() {
            self.committed = true;
        }
        outcome
    }
}

impl Drop for PullTransaction {
    fn drop(&mut self) {
        if !self.committed {
            let _ = remove_path_if_exists(&self.staging);
        }
    }
}

pub(crate) fn temporary_sibling_path(target: &Path, label: &str) -> PathBuf {
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("sync-target");
    target.with_file_name(format!(
        ".{name}.ccr-{label}-{}-{counter}",
        std::process::id()
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommitFailpoint {
    None,
    BeforeSync,
    BeforeBackupRename,
    BeforeInstallRename,
    AfterInstall,
    ParentSync,
    RestoreRename,
}

fn commit_staged_path(
    staging: &Path,
    target: &Path,
    failpoint: CommitFailpoint,
) -> Result<Option<PathBuf>> {
    if !staging.exists() {
        return Err(transaction_error("staging_missing", "同步 staging 不存在"));
    }
    if failpoint == CommitFailpoint::BeforeSync {
        return Err(transaction_error(
            "failpoint_sync",
            "staging fsync 故障注入",
        ));
    }
    sync_path(staging)?;

    let parent = target
        .parent()
        .ok_or_else(|| transaction_error("target", "同步目标没有父目录"))?;
    let backup = target
        .exists()
        .then(|| temporary_sibling_path(target, "backup"));

    if let Some(backup) = &backup {
        remove_path_if_exists(backup)?;
        if failpoint == CommitFailpoint::BeforeBackupRename {
            return Err(transaction_error(
                "failpoint_backup_rename",
                "active backup rename 故障注入",
            ));
        }
        fs::rename(target, backup).map_err(|error| {
            transaction_error("backup_rename", &format!("active 移入备份失败: {error}"))
        })?;
    }

    if failpoint == CommitFailpoint::RestoreRename {
        rollback_target_with_failpoint(target, backup.as_deref(), true)?;
        return Err(transaction_error(
            "failpoint_restore_rename",
            "backup restore 故障注入后已恢复原 active",
        ));
    }

    if failpoint == CommitFailpoint::BeforeInstallRename {
        rollback_target(target, backup.as_deref())?;
        return Err(transaction_error(
            "failpoint_install_rename",
            "staging install rename 故障注入",
        ));
    }
    if let Err(error) = fs::rename(staging, target) {
        rollback_target(target, backup.as_deref())?;
        return Err(transaction_error(
            "install_rename",
            &format!("staging 替换 active 失败，已恢复备份: {error}"),
        ));
    }

    if failpoint == CommitFailpoint::AfterInstall {
        rollback_target(target, backup.as_deref())?;
        return Err(transaction_error(
            "failpoint_install",
            "staging install 故障注入",
        ));
    }

    let parent_sync_result = if failpoint == CommitFailpoint::ParentSync {
        Err(transaction_error(
            "failpoint_parent_sync",
            "parent fsync 故障注入",
        ))
    } else {
        sync_directory(parent)
    };
    if let Err(error) = parent_sync_result {
        rollback_target(target, backup.as_deref())?;
        let _ = sync_directory(parent);
        return Err(transaction_error(
            "parent_sync",
            &format!("父目录 fsync 失败，已恢复原 active: {error}"),
        ));
    }

    Ok(backup)
}

fn rollback_target(target: &Path, backup: Option<&Path>) -> Result<()> {
    rollback_target_with_failpoint(target, backup, false)
}

fn rollback_target_with_failpoint(
    target: &Path,
    backup: Option<&Path>,
    fail_first_restore: bool,
) -> Result<()> {
    remove_path_if_exists(target)?;
    if let Some(backup) = backup
        && backup.exists()
    {
        let first_restore = if fail_first_restore {
            Err(std::io::Error::other("backup restore 故障注入"))
        } else {
            fs::rename(backup, target)
        };
        if let Err(first_error) = first_restore {
            fs::rename(backup, target).map_err(|retry_error| {
                transaction_error(
                    "rollback_restore",
                    &format!("恢复原 active 首次失败且重试失败: {first_error}; {retry_error}"),
                )
            })?;
        }
    }
    Ok(())
}

fn sync_path(path: &Path) -> Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|error| {
            transaction_error("read_staging", &format!("读取 staging 目录失败: {error}"))
        })? {
            let entry = entry.map_err(|error| {
                transaction_error("read_entry", &format!("读取 staging 条目失败: {error}"))
            })?;
            sync_path(&entry.path())?;
        }
        sync_directory(path)
    } else {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .and_then(|file| file.sync_all())
            .map_err(|error| {
                transaction_error("file_sync", &format!("staging 文件 fsync 失败: {error}"))
            })
    }
}

#[cfg(windows)]
fn sync_directory(path: &Path) -> Result<()> {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| transaction_error("directory_sync", &format!("目录 fsync 失败: {error}")))
}

#[cfg(not(windows))]
fn sync_directory(path: &Path) -> Result<()> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| transaction_error("directory_sync", &format!("目录 fsync 失败: {error}")))
}

pub(crate) fn remove_path_if_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let result = if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };
    result.map_err(|error| transaction_error("cleanup", &format!("清理事务路径失败: {error}")))
}

fn transaction_error(code: &str, message: &str) -> CcrError {
    CcrError::SyncError(format!("sync_transaction_{code}: {message}"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn prepare_file_case() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("active.toml");
        let staging = temp.path().join("staging.toml");
        fs::write(&target, b"old-bytes").unwrap();
        fs::write(&staging, b"new-bytes").unwrap();
        (temp, target, staging)
    }

    #[test]
    fn successful_commit_swaps_and_retains_backup() {
        let (_temp, target, staging) = prepare_file_case();
        let backup = commit_staged_path(&staging, &target, CommitFailpoint::None)
            .unwrap()
            .unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"new-bytes");
        assert_eq!(fs::read(&backup).unwrap(), b"old-bytes");
        assert!(!staging.exists());
    }

    #[test]
    fn successful_directory_commit_swaps_complete_tree() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("active");
        let staging = temp.path().join("staging");
        fs::create_dir_all(&target).unwrap();
        fs::create_dir_all(staging.join("nested")).unwrap();
        fs::write(target.join("old.txt"), b"old").unwrap();
        fs::write(staging.join("nested/new.txt"), b"new").unwrap();

        let backup = commit_staged_path(&staging, &target, CommitFailpoint::None)
            .unwrap()
            .unwrap();
        assert_eq!(fs::read(target.join("nested/new.txt")).unwrap(), b"new");
        assert_eq!(fs::read(backup.join("old.txt")).unwrap(), b"old");
    }

    #[test]
    fn failures_before_and_during_commit_preserve_active_bytes() {
        for failpoint in [
            CommitFailpoint::BeforeSync,
            CommitFailpoint::BeforeBackupRename,
            CommitFailpoint::BeforeInstallRename,
            CommitFailpoint::AfterInstall,
            CommitFailpoint::ParentSync,
            CommitFailpoint::RestoreRename,
        ] {
            let (_temp, target, staging) = prepare_file_case();
            assert!(commit_staged_path(&staging, &target, failpoint).is_err());
            assert_eq!(
                fs::read(&target).unwrap(),
                b"old-bytes",
                "failpoint {failpoint:?}"
            );
        }
    }

    #[tokio::test]
    async fn pull_transaction_cleans_uncommitted_staging() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("active");
        let staging = {
            let transaction = PullTransaction::begin(&target).unwrap();
            let staging = transaction.staging_path().to_path_buf();
            fs::write(&staging, b"partial").unwrap();
            staging
        };
        assert!(!staging.exists());
    }
}
