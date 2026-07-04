// 🛡️ 统一 guarded write 模块（持久化策略层）
//
// 分层：
//   fileio（序列化层）→ guarded_write（策略层）→ atomic_writer（物理层）
//
// 单次 guarded write 的内部顺序：
//   1. 由目标路径派生锁名，取跨进程文件锁（统一锁目录，CCR_LOCK_DIR 可覆盖）
//   2. 按 BackupPolicy 备份 + 轮换（keep = BACKUP_KEEP，源不存在则跳过）
//   3. AtomicWriter：temp →（可选 0o600）→ 写入 → fsync → 原子替换
//   4. RAII 释放锁
//
// 并发注意：
// - 本模块的文件锁是"写-写互斥"的叶子锁：锁名由路径派生，调用方不会直接持有它，
//   因此与调用方自身的 RMW 锁（CONFIG_LOCK / 命名锁）不构成环，无死锁风险。
// - load→mutate→save 序列的事务性仍由调用方负责，本模块只保证单次写的互斥与完整性。

use crate::core::atomic_writer::AtomicWriter;
use crate::core::error::{CcrError, Result};
use crate::core::lock::LockManager;
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Maximum number of rotated backup files kept per target.
pub const BACKUP_KEEP: usize = 10;

/// 默认锁超时（与既有调用方约定一致）
const DEFAULT_LOCK_TIMEOUT: Duration = Duration::from_secs(10);

/// Options controlling a guarded write.
///
/// The default is: no backup, non-secret permissions, 10s lock timeout.
#[derive(Debug, Clone)]
pub struct WriteOptions {
    /// Backup policy applied before the target file is replaced.
    pub backup: BackupPolicy,
    /// When true, the file is written with owner-only permissions
    /// (0o600 on Unix; no-op on Windows).
    pub secret: bool,
    /// Timeout for acquiring the per-path write lock.
    pub lock_timeout: Duration,
}

impl Default for WriteOptions {
    // 手动实现：Duration 的 derive 默认值是 0，而契约要求默认 10s
    fn default() -> Self {
        Self {
            backup: BackupPolicy::None,
            secret: false,
            lock_timeout: DEFAULT_LOCK_TIMEOUT,
        }
    }
}

/// Backup policy used by [`write_guarded`] and [`backup_guarded`].
#[derive(Debug, Clone, Default)]
pub enum BackupPolicy {
    /// No backup is taken.
    #[default]
    None,
    /// Backup next to the source file, named
    /// `{filename}.{tag}_{timestamp}.bak` (or `{filename}.{timestamp}.bak`
    /// without a tag). Rotation keeps the newest [`BACKUP_KEEP`] files whose
    /// name matches `starts_with(filename) && ends_with(".bak")`.
    SameDir {
        /// Optional tag inserted before the timestamp.
        tag: Option<String>,
    },
    /// Backup into a dedicated directory, named
    /// `{prefix}.{timestamp}.{ext}.bak` (`ext` falls back to `bak` when the
    /// source has no extension). Rotation keeps the newest [`BACKUP_KEEP`]
    /// files whose name matches `starts_with(prefix) && ends_with(".bak")`.
    Dir {
        /// Directory that receives the backup files (created when missing).
        dir: PathBuf,
        /// File name prefix for backup files in `dir`.
        prefix: String,
    },
}

/// Writes `bytes` to `path` under the unified persistence policy:
/// cross-process file lock, optional backup with rotation, then an atomic
/// replacement (temp file, optional 0o600, fsync, rename).
///
/// This guarantees mutual exclusion and integrity for a single write. It does
/// not make read-modify-write sequences transactional; callers keep owning
/// their own RMW locks.
pub fn write_guarded(path: &Path, bytes: &[u8], opts: &WriteOptions) -> Result<()> {
    // 绝对化：目标可能尚不存在，不能用 canonicalize
    let target = absolute_path(path)?;

    // 1. 派生锁名并获取跨进程文件锁（RAII 释放）
    let lock_manager = LockManager::with_default_path()?;
    let resource = lock_resource_name(&target);
    let _lock = lock_manager.lock_resource(&resource, opts.lock_timeout)?;

    // 2. 备份 + 轮换（policy=None 或源不存在时跳过）
    perform_backup(&target, &opts.backup)?;

    // 3. 原子写（temp → 可选 0o600 → 写入 → fsync → rename）
    AtomicWriter::new(&target)
        .secret(opts.secret)
        .write(bytes)?;

    tracing::debug!("✅ guarded write 完成: {:?}", target);
    Ok(())
}

/// Async variant of [`write_guarded`]. The blocking implementation runs on
/// the tokio blocking thread pool.
pub async fn write_guarded_async(path: &Path, bytes: Vec<u8>, opts: WriteOptions) -> Result<()> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || write_guarded(&path, &bytes, &opts))
        .await
        .map_err(|e| CcrError::FileIoError(format!("guarded write 后台任务失败: {}", e)))?
}

/// Takes an explicit backup of `path` according to `policy`, holding the same
/// per-path lock as [`write_guarded`].
///
/// Returns `Ok(None)` when `policy` is [`BackupPolicy::None`] or the source
/// file does not exist; otherwise returns the created backup path.
pub fn backup_guarded(path: &Path, policy: &BackupPolicy) -> Result<Option<PathBuf>> {
    if matches!(policy, BackupPolicy::None) {
        return Ok(None);
    }

    let source = absolute_path(path)?;
    if !source.exists() {
        return Ok(None);
    }

    let lock_manager = LockManager::with_default_path()?;
    let _lock = lock_manager.lock_resource(&lock_resource_name(&source), DEFAULT_LOCK_TIMEOUT)?;

    perform_backup(&source, policy)
}

/// 派生跨进程锁资源名：`gw_{消毒后的 file_stem}_{fnv1a64(绝对路径小写):016x}`
///
/// - 用 `std::path::absolute` 而非 canonicalize（目标首次写入时尚不存在）
/// - 统一小写后哈希（Windows 文件系统大小写不敏感；Unix 上仅可能"过度互斥"，无害）
/// - std 的 SipHash 每进程随机种子，不能用于跨进程锁名，故内联 FNV-1a
pub(crate) fn lock_resource_name(path: &Path) -> String {
    let abs = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let normalized = abs.to_string_lossy().to_lowercase();
    let hash = fnv1a64(normalized.as_bytes());
    format!("gw_{}_{:016x}", sanitized_stem(path), hash)
}

/// FNV-1a 64-bit（跨进程稳定的路径哈希）
fn fnv1a64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &byte in data {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// 锁名中的可读片段：file_stem 小写、非 ASCII 字母数字替换为 `_`、截断到 32 字符
fn sanitized_stem(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file")
        .to_lowercase();
    let mut sanitized: String = stem
        .chars()
        .take(32)
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    if sanitized.is_empty() {
        sanitized.push_str("file");
    }
    sanitized
}

/// 绝对化目标路径（相对路径的 parent 可能是空字符串，统一在入口处理）
fn absolute_path(path: &Path) -> Result<PathBuf> {
    std::path::absolute(path)
        .map_err(|e| CcrError::FileIoError(format!("解析目标路径 {} 失败: {}", path.display(), e)))
}

/// 执行备份 + 轮换（调用方需已持有对应的路径锁）
fn perform_backup(source: &Path, policy: &BackupPolicy) -> Result<Option<PathBuf>> {
    if !source.exists() {
        return Ok(None);
    }

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");

    match policy {
        BackupPolicy::None => Ok(None),
        BackupPolicy::SameDir { tag } => {
            let dir = source.parent().ok_or_else(|| {
                CcrError::FileIoError(format!("无法获取备份目录: {}", source.display()))
            })?;
            let file_name = source.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
                CcrError::FileIoError(format!("无效的源文件名: {}", source.display()))
            })?;

            // 命名与 config_file_handler 既有格式逐字节一致
            let backup_name = match tag {
                Some(tag) => format!("{file_name}.{tag}_{timestamp}.bak"),
                None => format!("{file_name}.{timestamp}.bak"),
            };
            let backup_path = dir.join(backup_name);

            copy_backup(source, &backup_path)?;
            rotate_backups(dir, file_name)?;
            Ok(Some(backup_path))
        }
        BackupPolicy::Dir { dir, prefix } => {
            fs::create_dir_all(dir)
                .map_err(|e| CcrError::FileIoError(format!("创建备份目录失败 {:?}: {}", dir, e)))?;

            // 命名与 platforms/base.rs backup_with_rotation 既有格式逐字节一致
            let extension = source
                .extension()
                .and_then(|ext| ext.to_str())
                .filter(|ext| !ext.is_empty())
                .unwrap_or("bak");
            let backup_path = dir.join(format!("{prefix}.{timestamp}.{extension}.bak"));

            copy_backup(source, &backup_path)?;
            rotate_backups(dir, prefix)?;
            Ok(Some(backup_path))
        }
    }
}

fn copy_backup(source: &Path, backup_path: &Path) -> Result<()> {
    fs::copy(source, backup_path).map_err(|e| {
        CcrError::FileIoError(format!(
            "备份文件失败 {:?} -> {:?}: {}",
            source, backup_path, e
        ))
    })?;
    tracing::debug!("💾 已创建备份: {:?}", backup_path);
    Ok(())
}

/// 轮换：目录内匹配 `starts_with(match_prefix) && ends_with(".bak")` 的文件，
/// 按 mtime 倒序保留最近 BACKUP_KEEP 个
fn rotate_backups(dir: &Path, match_prefix: &str) -> Result<()> {
    let mut backups: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| CcrError::FileIoError(format!("读取备份目录失败 {:?}: {}", dir, e)))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(match_prefix) && name.ends_with(".bak"))
        })
        .collect();

    if backups.len() <= BACKUP_KEEP {
        return Ok(());
    }

    backups.sort_by(|a, b| {
        let a_time = fs::metadata(a).and_then(|m| m.modified()).ok();
        let b_time = fs::metadata(b).and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    for old in &backups[BACKUP_KEEP..] {
        // 清理失败仅告警，不阻断写入
        if let Err(err) = fs::remove_file(old) {
            tracing::warn!("清理旧备份失败 {:?}: {}", old, err);
        } else {
            tracing::debug!("🗑️ 已删除旧备份: {:?}", old);
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestLockDirEnv;
    use std::thread;
    use tempfile::tempdir;

    // 断言时间戳段符合 %Y%m%d_%H%M%S（8 位日期 + '_' + 6 位时间）
    fn assert_timestamp_segment(segment: &str) {
        assert_eq!(segment.len(), 15, "时间戳段长度应为 15: {segment}");
        assert!(segment[..8].chars().all(|c| c.is_ascii_digit()));
        assert_eq!(&segment[8..9], "_");
        assert!(segment[9..].chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_write_guarded_basic_and_overwrite() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("data.toml");

        write_guarded(&target, b"first", &WriteOptions::default()).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "first");

        write_guarded(&target, b"second", &WriteOptions::default()).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "second");
    }

    #[test]
    fn test_backup_guarded_none_policy_and_missing_source() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("data.toml");

        // policy = None → Ok(None)
        fs::write(&target, "content").unwrap();
        assert!(
            backup_guarded(&target, &BackupPolicy::None)
                .unwrap()
                .is_none()
        );

        // 源不存在 → Ok(None)
        let missing = temp_dir.path().join("missing.toml");
        let policy = BackupPolicy::SameDir { tag: None };
        assert!(backup_guarded(&missing, &policy).unwrap().is_none());
    }

    #[test]
    fn test_same_dir_backup_naming() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("data.toml");
        fs::write(&target, "v1").unwrap();

        // 有 tag：{filename}.{tag}_{ts}.bak
        let policy = BackupPolicy::SameDir {
            tag: Some("import_backup".into()),
        };
        let backup = backup_guarded(&target, &policy).unwrap().unwrap();
        let name = backup.file_name().unwrap().to_str().unwrap();
        let segment = name
            .strip_prefix("data.toml.import_backup_")
            .unwrap()
            .strip_suffix(".bak")
            .unwrap();
        assert_timestamp_segment(segment);
        assert_eq!(fs::read_to_string(&backup).unwrap(), "v1");

        // 无 tag：{filename}.{ts}.bak
        let backup = backup_guarded(&target, &BackupPolicy::SameDir { tag: None })
            .unwrap()
            .unwrap();
        let name = backup.file_name().unwrap().to_str().unwrap();
        let segment = name
            .strip_prefix("data.toml.")
            .unwrap()
            .strip_suffix(".bak")
            .unwrap();
        assert_timestamp_segment(segment);
    }

    #[test]
    fn test_dir_backup_naming() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("profiles.toml");
        fs::write(&target, "v1").unwrap();

        let backup_dir = temp_dir.path().join("backups");
        let policy = BackupPolicy::Dir {
            dir: backup_dir.clone(),
            prefix: "profiles".into(),
        };
        let backup = backup_guarded(&target, &policy).unwrap().unwrap();
        assert_eq!(backup.parent().unwrap(), backup_dir);

        // {prefix}.{ts}.{ext}.bak，ext 取自源文件扩展名
        let name = backup.file_name().unwrap().to_str().unwrap();
        let segment = name
            .strip_prefix("profiles.")
            .unwrap()
            .strip_suffix(".toml.bak")
            .unwrap();
        assert_timestamp_segment(segment);
        assert_eq!(fs::read_to_string(&backup).unwrap(), "v1");
    }

    #[test]
    fn test_same_dir_backup_rotation_keeps_ten() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("data.toml");
        fs::write(&target, "v1").unwrap();

        // 预置 15 个匹配轮换过滤器的旧备份（10ms 间隔确保 mtime 有序）
        for idx in 0..15 {
            let fake = temp_dir.path().join(format!("data.toml.fake{idx:02}.bak"));
            fs::write(&fake, format!("fake-{idx}")).unwrap();
            thread::sleep(Duration::from_millis(10));
        }

        let opts = WriteOptions {
            backup: BackupPolicy::SameDir { tag: None },
            ..Default::default()
        };
        write_guarded(&target, b"v2", &opts).unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "v2");

        // 轮换后只保留最近 BACKUP_KEEP 个 .bak，最老的 fake 被删除
        let remaining: Vec<_> = fs::read_dir(temp_dir.path())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with("data.toml") && name.ends_with(".bak"))
            .collect();
        assert_eq!(remaining.len(), BACKUP_KEEP);
        assert!(!remaining.iter().any(|name| name.contains("fake00")));
    }

    #[test]
    fn test_dir_backup_rotation_keeps_ten() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("profiles.toml");
        fs::write(&target, "v1").unwrap();

        let backup_dir = temp_dir.path().join("backups");
        fs::create_dir_all(&backup_dir).unwrap();
        for idx in 0..15 {
            let fake = backup_dir.join(format!("profiles.fake{idx:02}.toml.bak"));
            fs::write(&fake, format!("fake-{idx}")).unwrap();
            thread::sleep(Duration::from_millis(10));
        }

        let opts = WriteOptions {
            backup: BackupPolicy::Dir {
                dir: backup_dir.clone(),
                prefix: "profiles".into(),
            },
            ..Default::default()
        };
        write_guarded(&target, b"v2", &opts).unwrap();

        let remaining: Vec<_> = fs::read_dir(&backup_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with("profiles") && name.ends_with(".bak"))
            .collect();
        assert_eq!(remaining.len(), BACKUP_KEEP);
        assert!(!remaining.iter().any(|name| name.contains("fake00")));
    }

    // 🔐 secret 权限断言仅在 Unix 有意义；Windows 权限模型（NTFS ACL）不同，
    // secret 选项在 Windows 上为 no-op，故无对应测试。
    #[cfg(unix)]
    #[test]
    fn test_write_guarded_secret_sets_owner_only_mode() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("sync.toml");

        let opts = WriteOptions {
            secret: true,
            ..Default::default()
        };
        write_guarded(&target, b"password = \"s3cret\"", &opts).unwrap();

        let mode = fs::metadata(&target).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn test_write_guarded_lock_contention_times_out() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("data.toml");

        // 手工持有派生锁，模拟并发写入方
        let lock_manager = LockManager::with_default_path().unwrap();
        let resource = lock_resource_name(&target);
        let _held = lock_manager
            .lock_resource(&resource, Duration::from_secs(5))
            .unwrap();

        let opts = WriteOptions {
            lock_timeout: Duration::from_millis(100),
            ..Default::default()
        };
        let result = write_guarded(&target, b"blocked", &opts);
        assert!(matches!(result, Err(CcrError::LockTimeout(_))));
        assert!(!target.exists());
    }

    #[test]
    fn test_write_guarded_multithreaded_no_torn_writes() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("stress.toml");

        // 4 线程各写 50 次不同长度的完整 payload，终态必须是某个完整 payload
        let payloads: Vec<String> = (0..4)
            .map(|idx| format!("payload-{idx}-").repeat((idx + 1) * 64))
            .collect();

        let handles: Vec<_> = payloads
            .iter()
            .cloned()
            .map(|payload| {
                let target = target.clone();
                thread::spawn(move || {
                    for _ in 0..50 {
                        write_guarded(&target, payload.as_bytes(), &WriteOptions::default())
                            .unwrap();
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }

        let final_content = fs::read_to_string(&target).unwrap();
        assert!(
            payloads.contains(&final_content),
            "终态内容必须是某个完整 payload，实际长度 {}",
            final_content.len()
        );
    }

    #[test]
    fn test_write_guarded_fails_when_parent_is_file() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());

        // 崩溃安全代理：父路径是文件 → temp 创建失败，返回 Err 且不 panic
        let blocker = temp_dir.path().join("blocker");
        fs::write(&blocker, "not a directory").unwrap();
        let target = blocker.join("child.toml");

        let result = write_guarded(&target, b"data", &WriteOptions::default());
        assert!(result.is_err());
        // 旧文件（blocker）内容原样保留
        assert_eq!(fs::read_to_string(&blocker).unwrap(), "not a directory");
    }

    #[tokio::test]
    async fn test_write_guarded_async_basic() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("async.toml");

        write_guarded_async(&target, b"async content".to_vec(), WriteOptions::default())
            .await
            .unwrap();
        assert_eq!(fs::read_to_string(&target).unwrap(), "async content");
    }

    #[test]
    fn test_lock_resource_name_format_and_stability() {
        let temp_dir = tempdir().unwrap();
        let target = temp_dir.path().join("Config.TOML");

        let first = lock_resource_name(&target);
        let second = lock_resource_name(&target);
        assert_eq!(first, second, "同一路径的锁名必须稳定");

        // 格式：gw_{小写消毒 stem}_{16 位十六进制哈希}
        assert!(first.starts_with("gw_config_"), "实际锁名: {first}");
        let hash = first.rsplit('_').next().unwrap();
        assert_eq!(hash.len(), 16);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        // 大小写归一：大小写不同的路径派生相同锁名（Windows 文件系统大小写不敏感）
        let lower = temp_dir.path().join("config.toml");
        assert_eq!(lock_resource_name(&lower), first);
    }
}
