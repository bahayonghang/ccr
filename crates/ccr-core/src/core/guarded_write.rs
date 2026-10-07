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

use crate::core::atomic_writer::{AtomicWriter, FileMetadata};
use crate::core::error::{CcrError, Result};
use crate::core::lock::LockManager;
use chrono::Local;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

/// Maximum number of rotated backup files kept per target.
pub const BACKUP_KEEP: usize = 10;

/// 默认锁超时（与既有调用方约定一致）
const DEFAULT_LOCK_TIMEOUT: Duration = Duration::from_secs(10);

/// Outcome of a compare-and-swap guarded write.
///
/// A version mismatch is an expected concurrency result rather than an I/O
/// error, so callers can map it to their own conflict protocol without
/// extending the frozen [`CcrError`] enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionedWriteOutcome {
    /// The expected version matched and the replacement completed.
    Written,
    /// The target changed since the caller read it; no backup or write ran.
    Conflict,
}

/// Options controlling a guarded write.
///
/// The default is: no backup, non-secret permissions, 10s lock timeout.
#[derive(Debug, Clone)]
pub struct WriteOptions {
    /// Backup policy applied before the target file is replaced.
    pub backup: BackupPolicy,
    /// When true, the file is written with owner-only permissions
    /// (owner-only Unix mode; private new Windows DACL / preserved existing DACL).
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
    /// `{filename}.{tag}_{timestamp}.{id}.bak` (or `{filename}.{timestamp}.{id}.bak`
    /// without a tag). Rotation keeps the newest [`BACKUP_KEEP`] files whose
    /// name matches `starts_with(filename) && ends_with(".bak")`.
    SameDir {
        /// Optional tag inserted before the timestamp.
        tag: Option<String>,
    },
    /// Backup into a dedicated directory, named
    /// `{prefix}.{timestamp}.{id}.{ext}.bak` (`ext` falls back to `bak` when the
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

    write_locked(&target, bytes, opts)?;
    Ok(())
}

/// Async variant of [`write_guarded`]. The blocking implementation runs on
/// the tokio blocking thread pool.
pub async fn write_guarded_async(path: &Path, bytes: Vec<u8>, opts: WriteOptions) -> Result<()> {
    if super::write_journal::is_active() {
        return Err(CcrError::ConfigError(
            "Async write inside a synchronous operation journal".into(),
        ));
    }
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || write_guarded(&path, &bytes, &opts))
        .await
        .map_err(|e| CcrError::FileIoError(format!("guarded write 后台任务失败: {}", e)))?
}

/// Returns a stable content version token for compare-and-swap writes.
pub fn content_version_token(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Writes only when the target still has `expected_token` while holding the
/// same path lock used for backup and replacement.
///
/// An empty token means the caller expects the target not to exist. Existing
/// empty files have the regular BLAKE3 token for an empty byte slice.
pub fn write_guarded_versioned(
    path: &Path,
    bytes: &[u8],
    expected_token: &str,
    opts: &WriteOptions,
) -> Result<VersionedWriteOutcome> {
    write_versioned_inner(path, bytes, expected_token, opts, None)
}

/// Enforce the existing secret-file policy without replacing matching content.
/// Returns false if the file is missing or its content version differs.
/// The leaf lock covers the handle read and permission update. Unix owner-only
/// modes and existing Windows DACLs are retained. No backup is created, and
/// content, inode and modification time remain unchanged.
pub fn enforce_secret_permissions_versioned(
    path: &Path,
    expected_token: &str,
    lock_timeout: Duration,
) -> Result<bool> {
    let target = absolute_path(path)?;
    let manager = LockManager::with_default_path()?;
    let _lock = manager.lock_resource(&lock_resource_name(&target), lock_timeout)?;
    let mut file = match fs::File::open(&target) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    if content_version_token(&bytes) != expected_token {
        return Ok(false);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = file.metadata()?.permissions().mode();
        let required = super::atomic_writer::secret_unix_mode(Some(mode));
        if mode & 0o7777 != required {
            // Check journal versions and faults before the metadata change.
            // Hardening adds no content rollback entry. Existing content
            // entries retain their original metadata restoration policy.
            let _ = super::write_journal::before_write(&target)?;
            file.set_permissions(fs::Permissions::from_mode(required))?;
            file.sync_all()?;
        }
    }
    Ok(true)
}

/// Restrict matching content to the current user without replacing the file.
///
/// The guarded leaf lock covers the handle read and metadata update. Missing
/// or stale content returns false without changing permissions. Windows uses
/// a protected process-token-user DACL; Unix retains stricter owner-only modes.
/// Content, file identity and modification time remain unchanged. No backup
/// or content journal entry is created.
pub fn enforce_owner_only_permissions_versioned(
    path: &Path,
    expected_token: &str,
    lock_timeout: Duration,
) -> Result<bool> {
    let target = absolute_path(path)?;
    let manager = LockManager::with_default_path()?;
    let _lock = manager.lock_resource(&lock_resource_name(&target), lock_timeout)?;
    #[cfg(windows)]
    let opened = {
        use std::os::windows::fs::OpenOptionsExt;

        fs::OpenOptions::new()
            .access_mode(0x8000_0000 | 0x0004_0000) // GENERIC_READ | WRITE_DAC
            .open(&target)
    };
    #[cfg(not(windows))]
    let opened = fs::File::open(&target);
    let mut file = match opened {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    if content_version_token(&bytes) != expected_token {
        return Ok(false);
    }
    #[cfg(windows)]
    {
        // Preserve the existing journal version/fault boundary. Metadata-only
        // hardening does not add a content compensation entry.
        let _ = super::write_journal::before_write(&target)?;
        super::atomic_writer::enforce_owner_only_windows_permissions(&file)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = file.metadata()?.permissions().mode();
        let required = super::atomic_writer::secret_unix_mode(Some(mode));
        if mode & 0o7777 != required {
            let _ = super::write_journal::before_write(&target)?;
            file.set_permissions(fs::Permissions::from_mode(required))?;
            file.sync_all()?;
        }
    }
    Ok(true)
}

pub(super) fn restore_guarded_versioned(
    path: &Path,
    bytes: &[u8],
    expected_token: &str,
    metadata: &FileMetadata,
) -> Result<VersionedWriteOutcome> {
    write_versioned_inner(
        path,
        bytes,
        expected_token,
        &WriteOptions {
            secret: true,
            ..Default::default()
        },
        Some(metadata),
    )
}

fn write_versioned_inner(
    path: &Path,
    bytes: &[u8],
    expected_token: &str,
    opts: &WriteOptions,
    metadata: Option<&FileMetadata>,
) -> Result<VersionedWriteOutcome> {
    let target = absolute_path(path)?;
    let lock_manager = LockManager::with_default_path()?;
    let resource = lock_resource_name(&target);
    let _lock = lock_manager.lock_resource(&resource, opts.lock_timeout)?;

    let current_token = match fs::read(&target) {
        Ok(current) => content_version_token(&current),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(CcrError::FileIoError(format!(
                "读取版本化写入目标 {} 失败: {}",
                target.display(),
                error
            )));
        }
    };

    if current_token != expected_token {
        return Ok(VersionedWriteOutcome::Conflict);
    }

    write_locked_with_metadata(&target, bytes, opts, metadata)?;
    Ok(VersionedWriteOutcome::Written)
}

/// Async variant of [`write_guarded_versioned`].
pub async fn write_guarded_versioned_async(
    path: &Path,
    bytes: Vec<u8>,
    expected_token: String,
    opts: WriteOptions,
) -> Result<VersionedWriteOutcome> {
    if super::write_journal::is_active() {
        return Err(CcrError::ConfigError(
            "Async write inside a synchronous operation journal".into(),
        ));
    }
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        write_guarded_versioned(&path, &bytes, &expected_token, &opts)
    })
    .await
    .map_err(|error| {
        CcrError::FileIoError(format!("versioned guarded write 后台任务失败: {error}"))
    })?
}

/// Applies backup and replacement while the caller holds the target lock.
fn write_locked(target: &Path, bytes: &[u8], opts: &WriteOptions) -> Result<()> {
    write_locked_with_metadata(target, bytes, opts, None)
}

fn write_locked_with_metadata(
    target: &Path,
    bytes: &[u8],
    opts: &WriteOptions,
    metadata: Option<&FileMetadata>,
) -> Result<()> {
    let preimage = super::write_journal::before_write(target)?;
    perform_backup(target, &opts.backup)?;
    AtomicWriter::new(target)
        .secret(opts.secret)
        .write_with_commit(bytes, metadata, || {
            super::write_journal::committed(target, preimage, Some(bytes));
        })?;

    tracing::debug!("✅ guarded write 完成: {:?}", target);
    Ok(())
}

/// Remove a file under the same leaf lock used by guarded writes.
pub fn delete_guarded(path: &Path) -> Result<()> {
    delete_guarded_inner(path, None).map(|_| ())
}

/// Compare-and-delete. Missing files use the empty version token.
pub fn delete_guarded_versioned(path: &Path, expected: &str) -> Result<VersionedWriteOutcome> {
    delete_guarded_inner(path, Some(expected))
}

fn delete_guarded_inner(path: &Path, expected: Option<&str>) -> Result<VersionedWriteOutcome> {
    let target = absolute_path(path)?;
    let manager = LockManager::with_default_path()?;
    let _lock = manager.lock_resource(&lock_resource_name(&target), DEFAULT_LOCK_TIMEOUT)?;
    let current = match fs::read(&target) {
        Ok(bytes) => Some(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(_) => {
            return Err(CcrError::FileIoError(
                "Cannot read guarded deletion target".into(),
            ));
        }
    };
    let token = current
        .as_deref()
        .map(content_version_token)
        .unwrap_or_default();
    if expected.is_some_and(|expected| token != expected) {
        return Ok(VersionedWriteOutcome::Conflict);
    }
    if current.is_some() {
        let preimage = super::write_journal::before_write(&target)?;
        fs::remove_file(&target)
            .map_err(|_| CcrError::FileIoError("Cannot delete guarded target".into()))?;
        super::write_journal::committed(&target, preimage, None);
    }
    Ok(VersionedWriteOutcome::Written)
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
    #[cfg(windows)]
    let abs = normalized_resource_path(path).unwrap_or_else(|_| path.to_path_buf());
    #[cfg(not(windows))]
    let abs = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let normalized = abs.to_string_lossy().to_lowercase();
    let hash = fnv1a64(normalized.as_bytes());
    #[cfg(windows)]
    let stem = sanitized_stem(&abs);
    #[cfg(not(windows))]
    let stem = sanitized_stem(path);
    format!("gw_{stem}_{hash:016x}")
}

/// Return the lexical identity shared by configuration resource locks.
///
/// Windows disk/UNC verbatim prefixes, case aliases, and 8.3 short names of an
/// existing ancestor use the same identity. This does not resolve symlinks and
/// does not require the final target to exist.
pub fn normalized_resource_path(path: &Path) -> std::io::Result<PathBuf> {
    let absolute = std::path::absolute(path)?;
    #[cfg(windows)]
    let absolute = expand_existing_short_names(&absolute);
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            #[cfg(windows)]
            Component::Prefix(prefix) => match prefix.kind() {
                std::path::Prefix::VerbatimDisk(drive) => {
                    normalized.push(format!("{}:", char::from(drive)));
                }
                std::path::Prefix::VerbatimUNC(server, share) => {
                    let mut prefix = std::ffi::OsString::from(r"\\");
                    prefix.push(server);
                    prefix.push(r"\");
                    prefix.push(share);
                    normalized.push(prefix);
                }
                _ => normalized.push(prefix.as_os_str()),
            },
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    #[cfg(windows)]
    {
        Ok(PathBuf::from(normalized.to_string_lossy().to_lowercase()))
    }
    #[cfg(not(windows))]
    {
        Ok(normalized)
    }
}

/// 把已经存在的祖先从 8.3 短名展开成长名，再接上尚未创建的尾部。
///
/// `GetLongPathNameW` 不跟随符号链接。最终目标不存在时展开父目录，避免
/// 文件创建前后锁身份变化。展开失败时保留调用方路径。
#[cfg(windows)]
fn expand_existing_short_names(path: &Path) -> PathBuf {
    let mut pending = Vec::new();
    let mut existing = path.to_path_buf();
    while !existing.as_os_str().is_empty() && fs::symlink_metadata(&existing).is_err() {
        let Some(name) = existing.file_name() else {
            return path.to_path_buf();
        };
        pending.push(name.to_os_string());
        if !existing.pop() {
            return path.to_path_buf();
        }
    }
    if existing.as_os_str().is_empty() {
        return path.to_path_buf();
    }
    let mut expanded = long_path_name(&existing).unwrap_or(existing);
    for name in pending.into_iter().rev() {
        expanded.push(name);
    }
    expanded
}

#[cfg(windows)]
fn long_path_name(path: &Path) -> std::io::Result<PathBuf> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    let input: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut capacity = 512u32;
    loop {
        let mut buffer = vec![0u16; capacity as usize];
        // SAFETY: `input` is nul-terminated and lives for the call. `buffer`
        // is writable for `capacity` wide characters. GetLongPathNameW does
        // not retain either pointer.
        let length = unsafe { GetLongPathNameW(input.as_ptr(), buffer.as_mut_ptr(), capacity) };
        if length == 0 {
            return Err(std::io::Error::last_os_error());
        }
        if length < capacity {
            buffer.truncate(length as usize);
            return Ok(PathBuf::from(std::ffi::OsString::from_wide(&buffer)));
        }
        if length > 32_768 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Windows long path exceeds 32768 characters",
            ));
        }
        capacity = length;
    }
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetLongPathNameW(short_path: *const u16, long_path: *mut u16, buffer_length: u32) -> u32;
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
    perform_backup_at(
        source,
        policy,
        &Local::now().format("%Y%m%d_%H%M%S").to_string(),
    )
}

fn perform_backup_at(
    source: &Path,
    policy: &BackupPolicy,
    timestamp: &str,
) -> Result<Option<PathBuf>> {
    if !source.exists() {
        return Ok(None);
    }

    match policy {
        BackupPolicy::None => Ok(None),
        BackupPolicy::SameDir { tag } => {
            let dir = source.parent().ok_or_else(|| {
                CcrError::FileIoError(format!("无法获取备份目录: {}", source.display()))
            })?;
            let file_name = source.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
                CcrError::FileIoError(format!("无效的源文件名: {}", source.display()))
            })?;

            let backup_path = copy_backup_new(source, dir, || {
                let id = uuid::Uuid::new_v4().simple();
                dir.join(match tag {
                    Some(tag) => format!("{file_name}.{tag}_{timestamp}.{id}.bak"),
                    None => format!("{file_name}.{timestamp}.{id}.bak"),
                })
            })?;
            rotate_backups(dir, file_name)?;
            Ok(Some(backup_path))
        }
        BackupPolicy::Dir { dir, prefix } => {
            fs::create_dir_all(dir)
                .map_err(|e| CcrError::FileIoError(format!("创建备份目录失败 {:?}: {}", dir, e)))?;

            // Preserve the prefix and extension used by existing discovery.
            let extension = source
                .extension()
                .and_then(|ext| ext.to_str())
                .filter(|ext| !ext.is_empty())
                .unwrap_or("bak");
            let backup_path = copy_backup_new(source, dir, || {
                let id = uuid::Uuid::new_v4().simple();
                dir.join(format!("{prefix}.{timestamp}.{id}.{extension}.bak"))
            })?;
            rotate_backups(dir, prefix)?;
            Ok(Some(backup_path))
        }
    }
}

fn copy_backup_new(
    source: &Path,
    dir: &Path,
    mut next_path: impl FnMut() -> PathBuf,
) -> Result<PathBuf> {
    let mut copy = || -> std::io::Result<PathBuf> {
        let mut input = fs::File::open(source)?;
        let mut temporary = tempfile::NamedTempFile::new_in(dir)?;
        // Apply the source policy before copying any bytes. In particular, a
        // private Windows target must not inherit the backup directory DACL.
        #[cfg(windows)]
        {
            use crate::core::atomic_writer::{apply_windows_dacl, capture_windows_dacl};
            apply_windows_dacl(temporary.path(), &capture_windows_dacl(source)?)?;
        }
        let source_permissions = input.metadata()?.permissions();
        temporary
            .as_file()
            .set_permissions(source_permissions.clone())?;
        std::io::copy(&mut input, temporary.as_file_mut())?;
        temporary.as_file().sync_all()?;
        for _ in 0..16 {
            let path = next_path();
            match temporary.persist_noclobber(&path) {
                Ok(file) => {
                    // tempfile clears Windows file attributes when publishing.
                    // Restore the source read-only flag; the DACL was applied
                    // before copying the payload and remains unchanged.
                    #[cfg(windows)]
                    file.set_permissions(source_permissions.clone())?;
                    drop(file);
                    return Ok(path);
                }
                Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
                    temporary = error.file;
                }
                Err(error) => return Err(error.error),
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "backup name allocation exhausted",
        ))
    };
    let path = copy().map_err(|error| {
        CcrError::FileIoError(format!("备份文件失败 {}: {error}", source.display()))
    })?;
    tracing::debug!("已创建备份: {:?}", path);
    Ok(path)
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
        b_time.cmp(&a_time).then_with(|| b.cmp(a))
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
    use std::sync::{Arc, Barrier};
    use std::thread;
    use tempfile::tempdir;

    // 断言时间戳段符合 %Y%m%d_%H%M%S（8 位日期 + '_' + 6 位时间）
    fn assert_timestamp_segment(segment: &str) {
        let (segment, id) = segment.split_once('.').unwrap();
        assert_eq!(id.len(), 32);
        assert!(id.bytes().all(|byte| byte.is_ascii_hexdigit()));
        assert_eq!(segment.len(), 15, "时间戳段长度应为 15: {segment}");
        assert!(segment[..8].chars().all(|c| c.is_ascii_digit()));
        assert_eq!(&segment[8..9], "_");
        assert!(segment[9..].chars().all(|c| c.is_ascii_digit()));
    }

    #[cfg(windows)]
    #[test]
    fn windows_canonical_path_keeps_resource_identity() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        fs::write(&path, b"before").unwrap();
        let canonical = fs::canonicalize(&path).unwrap();
        assert_eq!(
            normalized_resource_path(&path).unwrap(),
            normalized_resource_path(&canonical).unwrap()
        );
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
    fn test_fixed_clock_preserves_each_preimage_and_legacy_backup() {
        for separate in [false, true] {
            let dir = tempdir().unwrap();
            let source = dir.path().join("profiles.toml");
            let prefix = if separate {
                "profiles"
            } else {
                "profiles.toml"
            };
            let policy = if separate {
                BackupPolicy::Dir {
                    dir: dir.path().to_path_buf(),
                    prefix: prefix.into(),
                }
            } else {
                BackupPolicy::SameDir { tag: None }
            };
            let legacy = dir
                .path()
                .join(format!("{prefix}.20260928_120000.toml.bak"));
            fs::write(&legacy, b"legacy").unwrap();
            let mut paths = Vec::new();
            for content in ["first", "second", "third"] {
                fs::write(&source, content).unwrap();
                paths.push(
                    perform_backup_at(&source, &policy, "20260928_120000")
                        .unwrap()
                        .unwrap(),
                );
            }
            assert_eq!(fs::read(&legacy).unwrap(), b"legacy");
            for (path, content) in paths.iter().zip(["first", "second", "third"]) {
                assert_eq!(fs::read_to_string(path).unwrap(), content);
                // Old restore consumers select by path and read complete bytes.
                AtomicWriter::new(&source)
                    .write(&fs::read(path).unwrap())
                    .unwrap();
                assert_eq!(fs::read_to_string(&source).unwrap(), content);
            }
            assert_ne!(paths[0], paths[1]);
            assert_ne!(paths[1], paths[2]);
        }
    }

    #[test]
    fn test_backup_collision_retries_without_overwriting() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source");
        let collision = dir.path().join("existing.bak");
        let destination = dir.path().join("new.bak");
        fs::write(&source, b"new preimage").unwrap();
        fs::write(&collision, b"old preimage").unwrap();
        let mut attempts = 0;
        let result = copy_backup_new(&source, dir.path(), || {
            attempts += 1;
            if attempts == 1 {
                collision.clone()
            } else {
                destination.clone()
            }
        })
        .unwrap();
        assert_eq!(attempts, 2);
        assert_eq!(result, destination);
        assert_eq!(fs::read(collision).unwrap(), b"old preimage");
        assert_eq!(fs::read(destination).unwrap(), b"new preimage");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 3);
    }

    #[test]
    fn test_fixed_clock_backup_child() {
        let Some(target) = std::env::var_os("CCR_GUARDED_BACKUP_CHILD") else {
            return;
        };
        let source = PathBuf::from(target);
        let payload = std::env::var("CCR_GUARDED_BACKUP_PAYLOAD").unwrap();
        let manager = LockManager::with_default_path().unwrap();
        let _lock = manager
            .lock_resource(&lock_resource_name(&source), Duration::from_secs(10))
            .unwrap();
        perform_backup_at(
            &source,
            &BackupPolicy::SameDir { tag: None },
            "20260928_120000",
        )
        .unwrap();
        AtomicWriter::new(&source)
            .write(payload.as_bytes())
            .unwrap();
    }

    #[test]
    fn test_fixed_clock_multiprocess_backups_preserve_every_version() {
        use std::collections::BTreeSet;
        let dir = tempdir().unwrap();
        let _locks = TestLockDirEnv::new(&dir.path().join("locks"));
        let source = dir.path().join("profiles.toml");
        fs::write(&source, "initial").unwrap();
        let mut children: Vec<_> = ["one", "two", "three"]
            .into_iter()
            .map(|payload| {
                std::process::Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "core::guarded_write::tests::test_fixed_clock_backup_child",
                        "--test-threads=1",
                    ])
                    .env("CCR_GUARDED_BACKUP_CHILD", &source)
                    .env("CCR_GUARDED_BACKUP_PAYLOAD", payload)
                    .stdout(std::process::Stdio::null())
                    .spawn()
                    .unwrap()
            })
            .collect();
        for child in &mut children {
            assert!(child.wait().unwrap().success());
        }
        let backups: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "bak"))
            .collect();
        assert_eq!(backups.len(), 3);
        let mut contents: BTreeSet<_> = backups
            .iter()
            .map(|path| fs::read_to_string(path).unwrap())
            .collect();
        contents.insert(fs::read_to_string(source).unwrap());
        assert_eq!(
            contents,
            ["initial", "one", "two", "three"]
                .into_iter()
                .map(String::from)
                .collect()
        );
    }

    #[test]
    fn test_backup_failure_preserves_target_and_removes_temporary() {
        let dir = tempdir().unwrap();
        let _locks = TestLockDirEnv::new(&dir.path().join("locks"));
        let source = dir.path().join("source");
        let blocked = dir.path().join("not-a-directory");
        fs::write(&source, b"old").unwrap();
        fs::write(&blocked, b"block").unwrap();
        assert!(
            write_guarded(
                &source,
                b"new",
                &WriteOptions {
                    backup: BackupPolicy::Dir {
                        dir: blocked,
                        prefix: "source".into()
                    },
                    ..Default::default()
                }
            )
            .is_err()
        );
        assert_eq!(fs::read(&source).unwrap(), b"old");
        let collision = dir.path().join("occupied.bak");
        fs::write(&collision, b"kept").unwrap();
        let before = fs::read_dir(dir.path()).unwrap().count();
        assert!(copy_backup_new(&source, dir.path(), || collision.clone()).is_err());
        assert_eq!(fs::read(collision).unwrap(), b"kept");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), before);
    }

    #[test]
    fn test_rotation_ties_are_stable_for_old_and_new_names() {
        let dir = tempdir().unwrap();
        let timestamp = std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(12345);
        let mut paths = Vec::new();
        for index in 0..12 {
            let suffix = if index % 2 == 0 { "" } else { ".unique" };
            let path = dir
                .path()
                .join(format!("profiles.20260928_1200{index:02}{suffix}.toml.bak"));
            fs::write(&path, index.to_string()).unwrap();
            fs::File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_modified(timestamp)
                .unwrap();
            paths.push(path);
        }
        rotate_backups(dir.path(), "profiles").unwrap();
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), BACKUP_KEEP);
        assert!(!paths[0].exists());
        assert!(!paths[1].exists());
        assert!(paths[2..].iter().all(|path| path.exists()));
    }

    #[cfg(windows)]
    #[test]
    fn test_readonly_backup_collision_failure_removes_temporary() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.json");
        let collision = dir.path().join("existing.bak");
        fs::write(&source, b"private preimage").unwrap();
        fs::write(&collision, b"old preimage").unwrap();
        let original_permissions = fs::metadata(&source).unwrap().permissions();
        let mut permissions = original_permissions.clone();
        permissions.set_readonly(true);
        fs::set_permissions(&source, permissions.clone()).unwrap();
        let mut attempts = 0;
        let result = copy_backup_new(&source, dir.path(), || {
            attempts += 1;
            collision.clone()
        });
        fs::set_permissions(&source, original_permissions).unwrap();
        assert!(result.is_err());
        assert_eq!(attempts, 16);
        assert_eq!(fs::read(&source).unwrap(), b"private preimage");
        assert_eq!(fs::read(&collision).unwrap(), b"old preimage");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
    }

    #[cfg(windows)]
    #[test]
    fn test_backup_preserves_windows_readonly_attribute() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.json");
        fs::write(&source, b"private preimage").unwrap();
        let original_permissions = fs::metadata(&source).unwrap().permissions();
        let mut permissions = original_permissions.clone();
        permissions.set_readonly(true);
        fs::set_permissions(&source, permissions).unwrap();
        let backup = perform_backup_at(
            &source,
            &BackupPolicy::SameDir { tag: None },
            "20260928_120000",
        )
        .unwrap()
        .unwrap();
        let readonly = fs::metadata(&backup).unwrap().permissions().readonly();
        fs::set_permissions(&source, original_permissions.clone()).unwrap();
        fs::set_permissions(&backup, original_permissions).unwrap();
        assert!(
            readonly,
            "backup must preserve the source read-only attribute"
        );
        assert_eq!(fs::read(backup).unwrap(), b"private preimage");
    }

    #[cfg(windows)]
    #[test]
    fn test_backup_preserves_private_windows_dacl() {
        use crate::core::atomic_writer::capture_windows_dacl;
        let dir = tempdir().unwrap();
        let source = dir.path().join("private.json");
        AtomicWriter::new(&source)
            .secret(true)
            .write(b"private preimage")
            .unwrap();
        let backup = perform_backup_at(
            &source,
            &BackupPolicy::SameDir { tag: None },
            "20260928_120000",
        )
        .unwrap()
        .unwrap();
        let ace_bytes = |path: &Path| {
            let descriptor = capture_windows_dacl(path).unwrap();
            let offset = u32::from_le_bytes(descriptor[16..20].try_into().unwrap()) as usize;
            let length =
                u16::from_le_bytes(descriptor[offset + 2..offset + 4].try_into().unwrap()) as usize;
            descriptor[offset..offset + length].to_vec()
        };
        assert_eq!(ace_bytes(&source), ace_bytes(&backup));
        assert_eq!(fs::read(&backup).unwrap(), b"private preimage");
    }

    #[cfg(unix)]
    #[test]
    fn test_backup_preserves_private_unix_mode() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempdir().unwrap();
        let source = dir.path().join("private.json");
        for mode in [0o400, 0o600] {
            AtomicWriter::new(&source)
                .secret(true)
                .write(b"private preimage")
                .unwrap();
            fs::set_permissions(&source, fs::Permissions::from_mode(mode)).unwrap();
            let backup = perform_backup_at(
                &source,
                &BackupPolicy::SameDir { tag: None },
                "20260928_120000",
            )
            .unwrap()
            .unwrap();
            assert_eq!(
                fs::metadata(backup).unwrap().permissions().mode() & 0o777,
                mode
            );
        }
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

    // Unix mode assertions complement the Windows DACL tests above.
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
    fn test_content_version_token_is_stable_and_content_sensitive() {
        let first = content_version_token(b"settings-v1");
        let second = content_version_token(b"settings-v1");

        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
        assert_ne!(first, content_version_token(b"settings-v2"));
    }

    #[test]
    fn test_secret_permission_no_op_requires_an_existing_matching_version() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("settings.json");
        let timeout = Duration::from_secs(1);
        assert!(!enforce_secret_permissions_versioned(&target, "", timeout).unwrap());
        assert!(!target.exists());
        fs::write(&target, b"current").unwrap();
        let before = fs::metadata(&target).unwrap().modified().unwrap();
        assert!(
            !enforce_secret_permissions_versioned(
                &target,
                &content_version_token(b"stale"),
                timeout,
            )
            .unwrap()
        );
        assert_eq!(fs::read(&target).unwrap(), b"current");
        assert_eq!(fs::metadata(&target).unwrap().modified().unwrap(), before);
    }

    #[test]
    fn test_secret_permission_no_op_waits_for_the_guarded_leaf_lock() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("settings.json");
        fs::write(&target, b"current").unwrap();
        let manager = LockManager::with_default_path().unwrap();
        let _held = manager
            .lock_resource(&lock_resource_name(&target), Duration::from_secs(1))
            .unwrap();
        let result = enforce_secret_permissions_versioned(
            &target,
            &content_version_token(b"current"),
            Duration::from_millis(100),
        );
        assert!(matches!(result, Err(CcrError::LockTimeout(_))));
        assert_eq!(fs::read(&target).unwrap(), b"current");
    }

    #[cfg(windows)]
    fn windows_set_test_dacl(path: &Path, sddl: &str) {
        #[link(name = "advapi32")]
        unsafe extern "system" {
            fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text: *const u16,
                revision: u32,
                descriptor: *mut *mut std::ffi::c_void,
                length: *mut u32,
            ) -> i32;
        }
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn LocalFree(memory: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        }
        let sddl: Vec<u16> = sddl.encode_utf16().chain(std::iter::once(0)).collect();
        let mut descriptor = std::ptr::null_mut();
        let mut size = 0;
        // SAFETY: The converter owns the allocated descriptor until LocalFree.
        let ok = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                1,
                &mut descriptor,
                &mut size,
            )
        };
        assert_ne!(ok, 0);
        // SAFETY: The successful converter reports the allocated buffer size.
        let bytes =
            unsafe { std::slice::from_raw_parts(descriptor.cast::<u8>(), size as usize).to_vec() };
        // SAFETY: Release the converter's allocation after copying the bytes.
        unsafe { LocalFree(descriptor) };
        super::super::atomic_writer::apply_windows_dacl(path, &bytes).unwrap();
    }

    #[cfg(windows)]
    fn windows_process_user_sid() -> Vec<u8> {
        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetCurrentProcess() -> *mut std::ffi::c_void;
            fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
        }
        #[link(name = "advapi32")]
        unsafe extern "system" {
            fn OpenProcessToken(
                process: *mut std::ffi::c_void,
                access: u32,
                token: *mut *mut std::ffi::c_void,
            ) -> i32;
            fn GetTokenInformation(
                token: *mut std::ffi::c_void,
                class: u32,
                data: *mut std::ffi::c_void,
                length: u32,
                needed: *mut u32,
            ) -> i32;
            fn GetLengthSid(sid: *const std::ffi::c_void) -> u32;
        }
        // SAFETY: TOKEN_USER storage is aligned. The SID lives in that storage
        // until copied. Close the token handle before assertions.
        unsafe {
            let mut token = std::ptr::null_mut();
            assert_ne!(OpenProcessToken(GetCurrentProcess(), 0x0008, &mut token), 0);
            let mut needed = 0;
            GetTokenInformation(token, 1, std::ptr::null_mut(), 0, &mut needed);
            let mut info = vec![0_usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
            let ok = GetTokenInformation(token, 1, info.as_mut_ptr().cast(), needed, &mut needed);
            CloseHandle(token);
            assert_ne!(ok, 0);
            let sid = *(info.as_ptr().cast::<*const std::ffi::c_void>());
            std::slice::from_raw_parts(sid.cast::<u8>(), GetLengthSid(sid) as usize).to_vec()
        }
    }

    #[cfg(windows)]
    fn assert_windows_owner_only_dacl(path: &Path) {
        let descriptor = super::super::atomic_writer::capture_windows_dacl(path).unwrap();
        let control = u16::from_le_bytes(descriptor[2..4].try_into().unwrap());
        assert_ne!(control & 0x1000, 0, "the DACL must be protected");
        let offset = u32::from_le_bytes(descriptor[16..20].try_into().unwrap()) as usize;
        let ace_count = u16::from_le_bytes(descriptor[offset + 4..offset + 6].try_into().unwrap());
        assert_eq!(ace_count, 1, "only the process user receives access");
        let ace = &descriptor[offset + 8..];
        assert_eq!(ace[0], 0, "the single ACE must allow access");
        assert_eq!(ace[1], 0, "the ACE must not be inherited");
        let ace_size = u16::from_le_bytes(ace[2..4].try_into().unwrap()) as usize;
        let mask = u32::from_le_bytes(ace[4..8].try_into().unwrap());
        assert_eq!(mask, 0x001f_01ff, "the process user receives full control");
        // Do not print either SID if the equality assertion fails.
        assert!(
            ace[8..ace_size] == windows_process_user_sid(),
            "the ACE must match the independently queried process token user"
        );
    }

    #[cfg(windows)]
    fn windows_file_identity(path: &Path) -> (u32, u32, u32) {
        use std::os::windows::io::AsRawHandle;

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetFileInformationByHandle(
                file: *mut std::ffi::c_void,
                information: *mut u32,
            ) -> i32;
        }
        let file = fs::File::open(path).unwrap();
        // BY_HANDLE_FILE_INFORMATION consists of 13 DWORDs, including FILETIMEs.
        let mut information = [0_u32; 13];
        // SAFETY: The live file handle and aligned DWORD buffer cover the call.
        let ok =
            unsafe { GetFileInformationByHandle(file.as_raw_handle(), information.as_mut_ptr()) };
        assert_ne!(ok, 0);
        (information[7], information[11], information[12])
    }

    #[cfg(windows)]
    #[test]
    fn owner_only_permissions_tighten_windows_dacl_without_replacement() {
        let directory = tempdir().unwrap();
        let _locks = TestLockDirEnv::new(&directory.path().join("locks"));
        let target = directory.path().join("auth.json");
        fs::write(&target, b"synthetic credential bytes").unwrap();
        windows_set_test_dacl(&target, "D:P(A;;FA;;;WD)");
        let before = fs::metadata(&target).unwrap().modified().unwrap();
        let identity = windows_file_identity(&target);
        assert!(
            enforce_owner_only_permissions_versioned(
                &target,
                &content_version_token(b"synthetic credential bytes"),
                Duration::from_secs(1),
            )
            .unwrap()
        );
        assert_eq!(fs::read(&target).unwrap(), b"synthetic credential bytes");
        assert_eq!(fs::metadata(&target).unwrap().modified().unwrap(), before);
        assert_eq!(windows_file_identity(&target), identity);
        assert_windows_owner_only_dacl(&target);
        assert!(
            enforce_owner_only_permissions_versioned(
                &target,
                &content_version_token(b"synthetic credential bytes"),
                Duration::from_secs(1),
            )
            .unwrap()
        );
        assert_eq!(fs::metadata(&target).unwrap().modified().unwrap(), before);
        assert_eq!(windows_file_identity(&target), identity);
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
        assert_windows_owner_only_dacl(&target);
    }

    #[test]
    fn owner_only_permissions_require_an_existing_matching_version() {
        let directory = tempdir().unwrap();
        let _locks = TestLockDirEnv::new(&directory.path().join("locks"));
        let target = directory.path().join("auth.json");
        assert!(
            !enforce_owner_only_permissions_versioned(&target, "", Duration::from_secs(1)).unwrap()
        );
        assert!(!target.exists());
        fs::write(&target, b"current").unwrap();
        #[cfg(windows)]
        windows_set_test_dacl(&target, "D:P(A;;FA;;;WD)");
        #[cfg(windows)]
        let dacl = super::super::atomic_writer::capture_windows_dacl(&target).unwrap();
        #[cfg(windows)]
        let identity = windows_file_identity(&target);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target, fs::Permissions::from_mode(0o644)).unwrap();
        }
        let before = fs::metadata(&target).unwrap().modified().unwrap();
        assert!(
            !enforce_owner_only_permissions_versioned(
                &target,
                &content_version_token(b"stale"),
                Duration::from_secs(1),
            )
            .unwrap()
        );
        assert_eq!(fs::read(&target).unwrap(), b"current");
        assert_eq!(fs::metadata(&target).unwrap().modified().unwrap(), before);
        #[cfg(windows)]
        {
            assert_eq!(windows_file_identity(&target), identity);
            assert!(super::super::atomic_writer::capture_windows_dacl(&target).unwrap() == dacl);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&target).unwrap().permissions().mode() & 0o777,
                0o644
            );
        }
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 2);
    }

    #[test]
    fn owner_only_permissions_wait_for_the_guarded_leaf_lock() {
        let directory = tempdir().unwrap();
        let _locks = TestLockDirEnv::new(&directory.path().join("locks"));
        let target = directory.path().join("auth.json");
        fs::write(&target, b"current").unwrap();
        let manager = LockManager::with_default_path().unwrap();
        let _held = manager
            .lock_resource(&lock_resource_name(&target), Duration::from_secs(1))
            .unwrap();
        let result = enforce_owner_only_permissions_versioned(
            &target,
            &content_version_token(b"current"),
            Duration::from_millis(100),
        );
        assert!(matches!(result, Err(CcrError::LockTimeout(_))));
        assert_eq!(fs::read(&target).unwrap(), b"current");
    }

    #[cfg(windows)]
    #[test]
    fn owner_only_permissions_username_independent_child() {
        let Some(target) = std::env::var_os("CCR_OWNER_ONLY_TEST_CHILD") else {
            return;
        };
        assert!(std::env::var_os("USERNAME").is_none());
        let target = PathBuf::from(target);
        assert!(
            enforce_owner_only_permissions_versioned(
                &target,
                &content_version_token(b"current"),
                Duration::from_secs(1),
            )
            .unwrap()
        );
        assert_windows_owner_only_dacl(&target);
    }

    #[cfg(windows)]
    #[test]
    fn owner_only_permissions_use_the_token_user_without_username() {
        let directory = tempdir().unwrap();
        let target = directory.path().join("auth.json");
        fs::write(&target, b"current").unwrap();
        windows_set_test_dacl(&target, "D:P(A;;FA;;;WD)");
        let before = fs::metadata(&target).unwrap().modified().unwrap();
        let identity = windows_file_identity(&target);
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "core::guarded_write::tests::owner_only_permissions_username_independent_child",
                "--nocapture",
            ])
            .env("CCR_OWNER_ONLY_TEST_CHILD", &target)
            .env("CCR_LOCK_DIR", directory.path().join("locks"))
            .env_remove("USERNAME")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "USERNAME-independent child failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(fs::read(&target).unwrap(), b"current");
        assert_eq!(fs::metadata(&target).unwrap().modified().unwrap(), before);
        assert_eq!(windows_file_identity(&target), identity);
        assert_windows_owner_only_dacl(&target);
    }

    #[cfg(windows)]
    #[test]
    fn owner_only_permissions_propagate_native_write_dac_denial() {
        use std::os::windows::fs::OpenOptionsExt;

        let directory = tempdir().unwrap();
        let _locks = TestLockDirEnv::new(&directory.path().join("locks"));
        let target = directory.path().join("auth.json");
        fs::write(&target, b"current").unwrap();
        // Keep a granted handle so the fixture can restore its DACL after
        // denying WRITE_DAC, including the owner's normally implicit right.
        let restoration = fs::OpenOptions::new()
            .access_mode(0x8000_0000 | 0x0004_0000)
            .open(&target)
            .unwrap();
        windows_set_test_dacl(&target, "D:P(D;;WD;;;OW)(A;;FR;;;WD)");
        let before_dacl = super::super::atomic_writer::capture_windows_dacl(&target).unwrap();
        let before = fs::metadata(&target).unwrap().modified().unwrap();
        let identity = windows_file_identity(&target);
        let result = enforce_owner_only_permissions_versioned(
            &target,
            &content_version_token(b"current"),
            Duration::from_secs(1),
        );
        let unchanged_dacl =
            super::super::atomic_writer::capture_windows_dacl(&target).unwrap() == before_dacl;
        super::super::atomic_writer::enforce_owner_only_windows_permissions(&restoration).unwrap();
        assert!(
            matches!(result, Err(CcrError::IoError(error)) if error.kind() == std::io::ErrorKind::PermissionDenied)
        );
        assert!(unchanged_dacl);
        assert_eq!(fs::read(&target).unwrap(), b"current");
        assert_eq!(fs::metadata(&target).unwrap().modified().unwrap(), before);
        assert_eq!(windows_file_identity(&target), identity);
        assert_windows_owner_only_dacl(&target);
    }

    #[cfg(all(any(unix, windows), feature = "test-support"))]
    #[test]
    fn owner_only_permissions_policy_failure_preserves_metadata() {
        let directory = tempdir().unwrap();
        let _locks = TestLockDirEnv::new(&directory.path().join("locks"));
        let target = directory.path().join("auth.json");
        fs::write(&target, b"current").unwrap();
        #[cfg(windows)]
        windows_set_test_dacl(&target, "D:P(A;;FA;;;WD)");
        #[cfg(windows)]
        let dacl = super::super::atomic_writer::capture_windows_dacl(&target).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target, fs::Permissions::from_mode(0o644)).unwrap();
        }
        let before = fs::metadata(&target).unwrap().modified().unwrap();
        let _fault = crate::core::write_journal::fault::install(|_| {
            Err(CcrError::FileIoError(
                "injected permission policy failure".into(),
            ))
        });
        assert!(
            enforce_owner_only_permissions_versioned(
                &target,
                &content_version_token(b"current"),
                Duration::from_secs(1),
            )
            .is_err()
        );
        assert_eq!(fs::read(&target).unwrap(), b"current");
        assert_eq!(fs::metadata(&target).unwrap().modified().unwrap(), before);
        #[cfg(windows)]
        assert!(super::super::atomic_writer::capture_windows_dacl(&target).unwrap() == dacl);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&target).unwrap().permissions().mode() & 0o777,
                0o644
            );
        }
    }

    #[cfg(any(unix, windows))]
    #[test]
    fn owner_only_permissions_check_journal_version_without_a_content_entry() {
        let directory = tempdir().unwrap();
        let _locks = TestLockDirEnv::new(&directory.path().join("locks"));
        let target = directory.path().join("auth.json");
        fs::write(&target, b"current").unwrap();
        #[cfg(windows)]
        windows_set_test_dacl(&target, "D:P(A;;FA;;;WD)");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target, fs::Permissions::from_mode(0o644)).unwrap();
        }
        let journal =
            crate::core::write_journal::WriteJournal::begin(std::slice::from_ref(&target)).unwrap();
        journal
            .expect_version(&target, content_version_token(b"older"))
            .unwrap();
        assert!(
            enforce_owner_only_permissions_versioned(
                &target,
                &content_version_token(b"current"),
                Duration::from_secs(1),
            )
            .is_err()
        );
        journal
            .expect_version(&target, content_version_token(b"current"))
            .unwrap();
        assert!(
            enforce_owner_only_permissions_versioned(
                &target,
                &content_version_token(b"current"),
                Duration::from_secs(1),
            )
            .unwrap()
        );
        assert!(journal.changed_paths().is_empty());
        assert!(journal.rollback().is_empty());
        #[cfg(windows)]
        assert_windows_owner_only_dacl(&target);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&target).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn owner_only_permissions_preserve_stricter_unix_modes_and_inode() {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        let directory = tempdir().unwrap();
        let _locks = TestLockDirEnv::new(&directory.path().join("locks"));
        for (mode, expected) in [(0o644, 0o600), (0o600, 0o600), (0o400, 0o400)] {
            let target = directory.path().join(format!("auth-{mode:o}.json"));
            fs::write(&target, b"current").unwrap();
            fs::set_permissions(&target, fs::Permissions::from_mode(mode)).unwrap();
            let before = fs::metadata(&target).unwrap();
            assert!(
                enforce_owner_only_permissions_versioned(
                    &target,
                    &content_version_token(b"current"),
                    Duration::from_secs(1),
                )
                .unwrap()
            );
            let after = fs::metadata(&target).unwrap();
            assert_eq!(after.permissions().mode() & 0o777, expected);
            assert_eq!(after.ino(), before.ino());
            assert_eq!(after.modified().unwrap(), before.modified().unwrap());
            assert_eq!(fs::read(&target).unwrap(), b"current");
        }
    }

    #[cfg(all(unix, feature = "test-support"))]
    #[test]
    fn test_secret_permission_no_op_policy_fault_preserves_metadata() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("settings.json");
        fs::write(&target, b"current").unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o644)).unwrap();
        let before = fs::metadata(&target).unwrap().modified().unwrap();
        let _fault = crate::core::write_journal::fault::install(|_| {
            Err(CcrError::FileIoError(
                "injected permission policy failure".into(),
            ))
        });
        assert!(
            enforce_secret_permissions_versioned(
                &target,
                &content_version_token(b"current"),
                Duration::from_secs(1),
            )
            .is_err()
        );
        let metadata = fs::metadata(&target).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o777, 0o644);
        assert_eq!(metadata.modified().unwrap(), before);
        assert_eq!(fs::read(&target).unwrap(), b"current");
    }

    #[test]
    fn test_write_guarded_versioned_writes_matching_version_with_backup() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("settings.json");
        let backup_dir = temp_dir.path().join("backups");
        fs::write(&target, b"version-one").unwrap();
        let expected = content_version_token(b"version-one");
        let opts = WriteOptions {
            backup: BackupPolicy::Dir {
                dir: backup_dir.clone(),
                prefix: "settings.json".into(),
            },
            ..Default::default()
        };

        let outcome = write_guarded_versioned(&target, b"version-two", &expected, &opts).unwrap();

        assert_eq!(outcome, VersionedWriteOutcome::Written);
        assert_eq!(fs::read(&target).unwrap(), b"version-two");
        assert_eq!(fs::read_dir(&backup_dir).unwrap().count(), 1);
    }

    #[test]
    fn test_write_guarded_versioned_rejects_conflict_without_backup() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("settings.json");
        let backup_dir = temp_dir.path().join("backups");
        fs::write(&target, b"external-change").unwrap();
        let opts = WriteOptions {
            backup: BackupPolicy::Dir {
                dir: backup_dir.clone(),
                prefix: "settings.json".into(),
            },
            ..Default::default()
        };

        let outcome = write_guarded_versioned(
            &target,
            b"stale-editor-content",
            &content_version_token(b"older-content"),
            &opts,
        )
        .unwrap();

        assert_eq!(outcome, VersionedWriteOutcome::Conflict);
        assert_eq!(fs::read(&target).unwrap(), b"external-change");
        assert!(!backup_dir.exists());
    }

    #[test]
    fn test_write_guarded_versioned_empty_token_only_creates_missing_file() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("new-config.toml");

        let created =
            write_guarded_versioned(&target, b"model = 'first'", "", &WriteOptions::default())
                .unwrap();
        let conflict =
            write_guarded_versioned(&target, b"model = 'second'", "", &WriteOptions::default())
                .unwrap();

        assert_eq!(created, VersionedWriteOutcome::Written);
        assert_eq!(conflict, VersionedWriteOutcome::Conflict);
        assert_eq!(fs::read(&target).unwrap(), b"model = 'first'");
    }

    #[test]
    fn test_write_guarded_versioned_allows_only_one_concurrent_cas_writer() {
        let temp_dir = tempdir().unwrap();
        let _lock_dir = TestLockDirEnv::new(temp_dir.path().join("locks").as_path());
        let target = temp_dir.path().join("settings.json");
        fs::write(&target, b"base").unwrap();
        let expected = content_version_token(b"base");
        let barrier = Arc::new(Barrier::new(4));

        let handles: Vec<_> = (0..4)
            .map(|index| {
                let target = target.clone();
                let expected = expected.clone();
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    write_guarded_versioned(
                        &target,
                        format!("writer-{index}").as_bytes(),
                        &expected,
                        &WriteOptions::default(),
                    )
                    .unwrap()
                })
            })
            .collect();

        let outcomes: Vec<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| **outcome == VersionedWriteOutcome::Written)
                .count(),
            1
        );
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| **outcome == VersionedWriteOutcome::Conflict)
                .count(),
            3
        );
        assert!(fs::read_to_string(&target).unwrap().starts_with("writer-"));
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
