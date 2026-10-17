use ccr_core::core::error::{CcrError, Result};
use std::collections::HashSet;
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
            if is_profile_catalog(&target) {
                ccr_config::install_profile_catalog(&target, &staging)?;
                let _ = remove_path_if_exists(&staging);
                Ok(None)
            } else if staging.is_dir() {
                commit_staged_directory(&staging, &target)
            } else {
                commit_staged_path(&staging, &target, CommitFailpoint::None)
            }
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

fn is_profile_catalog(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("profiles.db"))
}

fn is_catalog_sidecar(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.eq_ignore_ascii_case("profiles.db-wal")
                || name.eq_ignore_ascii_case("profiles.db-shm")
        })
}

struct StagedTree {
    catalogs: Vec<PathBuf>,
    files: Vec<PathBuf>,
    sidecars: Vec<PathBuf>,
}

fn commit_staged_directory(staging: &Path, target: &Path) -> Result<Option<PathBuf>> {
    let staged = classify_staged_tree(staging)?;
    if staged.catalogs.is_empty() {
        return commit_staged_path(staging, target, CommitFailpoint::None);
    }
    commit_directory_with_catalogs(staging, target, &staged)
}

fn commit_directory_with_catalogs(
    staging: &Path,
    target: &Path,
    staged: &StagedTree,
) -> Result<Option<PathBuf>> {
    for relative in &staged.catalogs {
        ccr_config::validate_profile_catalog_file(&staging.join(relative))
            .map_err(|error| transaction_error("catalog_validate", &error.to_string()))?;
    }
    for sidecar in &staged.sidecars {
        fs::remove_file(sidecar).map_err(|error| {
            transaction_error(
                "cleanup",
                &format!("移除 staging 目录库附属文件失败: {error}"),
            )
        })?;
    }
    for relative in &staged.catalogs {
        let live = target.join(relative);
        if let Some(parent) = live.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                transaction_error("mkdir", &format!("创建配置目录父目录失败: {error}"))
            })?;
        }
        ccr_config::install_profile_catalog(&live, &staging.join(relative))
            .map_err(|error| transaction_error("catalog_install", &error.to_string()))?;
        fs::remove_file(staging.join(relative)).map_err(|error| {
            transaction_error("cleanup", &format!("移除 staging 目录库失败: {error}"))
        })?;
    }
    merge_remaining_tree(staging, target)?;
    let mut keep = HashSet::new();
    keep.extend(staged.files.iter().cloned());
    keep.extend(staged.catalogs.iter().cloned());
    retain_pulled_files(target, target, &keep)?;
    remove_empty_dirs(target, target)?;
    let _ = remove_path_if_exists(staging);
    Ok(None)
}

fn classify_staged_tree(root: &Path) -> Result<StagedTree> {
    let mut staged = StagedTree {
        catalogs: Vec::new(),
        files: Vec::new(),
        sidecars: Vec::new(),
    };
    classify_directory(root, root, &mut staged)?;
    staged.catalogs.sort();
    Ok(staged)
}

fn classify_directory(root: &Path, dir: &Path, staged: &mut StagedTree) -> Result<()> {
    let mut entries = fs::read_dir(dir)
        .map_err(|error| transaction_error("read_dir", &format!("读取 staging 目录失败: {error}")))?
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|error| {
            transaction_error("read_dir", &format!("读取 staging 目录项失败: {error}"))
        })?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let file_type = entry.file_type().map_err(|error| {
            transaction_error("read_dir", &format!("读取 staging 条目类型失败: {error}"))
        })?;
        if file_type.is_symlink() {
            return Err(transaction_error("symlink", "同步目录中不允许符号链接"));
        }
        let path = entry.path();
        if file_type.is_dir() {
            classify_directory(root, &path, staged)?;
            continue;
        }
        if is_profile_catalog(&path) {
            staged.catalogs.push(relative_to(root, &path)?);
        } else if is_catalog_sidecar(&path) {
            staged.sidecars.push(path);
        } else {
            staged.files.push(relative_to(root, &path)?);
        }
    }
    Ok(())
}

fn relative_to(root: &Path, path: &Path) -> Result<PathBuf> {
    path.strip_prefix(root)
        .map(Path::to_path_buf)
        .map_err(|_| transaction_error("path", "staging 相对路径无效"))
}

fn merge_remaining_tree(staging: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)
        .map_err(|error| transaction_error("mkdir", &format!("创建同步目标目录失败: {error}")))?;
    let mut entries = fs::read_dir(staging)
        .map_err(|error| transaction_error("read_dir", &format!("读取 staging 目录失败: {error}")))?
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|error| {
            transaction_error("read_dir", &format!("读取 staging 目录项失败: {error}"))
        })?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            transaction_error("read_dir", &format!("读取 staging 条目类型失败: {error}"))
        })?;
        if file_type.is_symlink() {
            return Err(transaction_error("symlink", "同步目录中不允许符号链接"));
        }
        let dest = target.join(entry.file_name());
        if file_type.is_dir() {
            merge_remaining_tree(&path, &dest)?;
            let _ = fs::remove_dir(&path);
            continue;
        }
        if is_profile_catalog(&path) || is_catalog_sidecar(&path) {
            return Err(transaction_error(
                "catalog_left",
                "staging 中的 profiles.db 未完成安装",
            ));
        }
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                transaction_error("mkdir", &format!("创建同步目标父目录失败: {error}"))
            })?;
        }
        let backup = commit_staged_path(&path, &dest, CommitFailpoint::None)?;
        if let Some(backup) = backup {
            let _ = remove_path_if_exists(&backup);
        }
    }
    Ok(())
}

fn retain_pulled_files(dir: &Path, root: &Path, keep: &HashSet<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(dir)
        .map_err(|error| transaction_error("read_dir", &format!("读取同步目标失败: {error}")))?
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|error| transaction_error("read_dir", &format!("读取同步目标项失败: {error}")))?;
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            transaction_error("read_dir", &format!("读取同步目标类型失败: {error}"))
        })?;
        if file_type.is_dir() {
            retain_pulled_files(&path, root, keep)?;
            continue;
        }
        if is_catalog_sidecar(&path) {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|_| transaction_error("path", "本地相对路径无效"))?;
        if !keep.contains(relative) {
            fs::remove_file(&path).map_err(|error| {
                transaction_error("cleanup", &format!("移除未被拉取的文件失败: {error}"))
            })?;
        }
    }
    Ok(())
}

fn remove_empty_dirs(dir: &Path, root: &Path) -> Result<()> {
    let entries = fs::read_dir(dir)
        .map_err(|error| transaction_error("read_dir", &format!("读取同步目标失败: {error}")))?
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|error| transaction_error("read_dir", &format!("读取同步目标项失败: {error}")))?;
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            remove_empty_dirs(&path, root)?;
        }
    }
    if dir != root
        && fs::read_dir(dir)
            .ok()
            .is_some_and(|mut items| items.next().is_none())
    {
        let _ = fs::remove_dir(dir);
    }
    Ok(())
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

    #[tokio::test]
    async fn directory_without_catalog_still_renames_the_tree() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("active");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("old.txt"), b"old").unwrap();
        let transaction = PullTransaction::begin(&target).unwrap();
        let staging = transaction.staging_path();
        fs::create_dir_all(staging.join("nested")).unwrap();
        fs::write(staging.join("nested").join("new.txt"), b"new").unwrap();
        let backup = transaction.commit().await.unwrap().unwrap();
        assert_eq!(
            fs::read(target.join("nested").join("new.txt")).unwrap(),
            b"new"
        );
        assert_eq!(fs::read(backup.join("old.txt")).unwrap(), b"old");
    }
}
