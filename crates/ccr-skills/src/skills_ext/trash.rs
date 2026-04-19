//! File-backed trash store for soft-deleted skills.
//!
//! 对应参考实现 `skill-hub/server/trash/store.ts`。关键差异：
//! - 零新增依赖（复用 chrono/serde_json/walkdir + 自写 Clock）
//! - 恢复名冲突时用 `-restored-<unix_ts>` 后缀（skill-hub 原版语义）
//! - 用 `trait Clock` 注入时间，便于 TTL 单测不用 `sleep`
//!
//! ## 目录布局
//! ```text
//! ~/.ccr/skills/trash/
//! └── <trash_id>/
//!     ├── meta.json       # TrashEntry 序列化
//!     └── contents/       # 原 skill 目录整体 rename 过来
//!         └── SKILL.md + ...
//! ```
//!
//! ## Trash ID 稳定性
//! `blake3(ns_timestamp ^ atomic_counter)[..16]`，与 versioning 的生成策略一致
//! 但使用独立计数器，避免语义混用。

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::paths::trash_root;

/// 回收站保留期。超期 skill 在启动时 / 手动 purge 时被永久删除。
pub const TRASH_TTL_DAYS: i64 = 7;

/// 回收站内 skill 内容子目录名。
const CONTENTS_DIR: &str = "contents";

/// 元数据文件名。
const META_FILE: &str = "meta.json";

// ============================================================================
// 错误类型
// ============================================================================

#[derive(Debug)]
pub enum TrashError {
    Io(io::Error),
    Json(serde_json::Error),
    NoHomeDir,
    EntryNotFound(String),
    SourceNotFound(PathBuf),
    InvalidEntry(String),
}

impl std::fmt::Display for TrashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::NoHomeDir => write!(f, "Cannot locate home directory"),
            Self::EntryNotFound(id) => write!(f, "Trash entry {id} not found"),
            Self::SourceNotFound(p) => write!(f, "Skill to delete not found: {}", p.display()),
            Self::InvalidEntry(reason) => write!(f, "Invalid trash entry: {reason}"),
        }
    }
}

impl std::error::Error for TrashError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for TrashError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for TrashError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

pub type TrashResult<T> = Result<T, TrashError>;

// ============================================================================
// Clock 抽象（便于注入时间做 TTL 测试）
// ============================================================================

/// 时间源抽象。生产用 [`SystemClock`]，测试用 [`MutableClock`]。
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// 真实系统时钟。
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// 可变时钟（测试用）。`advance_to()` 线程安全。
pub struct MutableClock {
    inner: Mutex<DateTime<Utc>>,
}

impl MutableClock {
    pub fn new(initial: DateTime<Utc>) -> Self {
        Self {
            inner: Mutex::new(initial),
        }
    }

    pub fn advance_to(&self, t: DateTime<Utc>) {
        let mut g = self.inner.lock().expect("MutableClock mutex poisoned");
        *g = t;
    }
}

impl Clock for MutableClock {
    fn now(&self) -> DateTime<Utc> {
        *self.inner.lock().expect("MutableClock mutex poisoned")
    }
}

// ============================================================================
// TrashEntry
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashEntry {
    pub id: String,
    pub skill_name: String,
    pub original_path: String,
    pub deleted_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

// ============================================================================
// FsTrashStore
// ============================================================================

pub struct FsTrashStore {
    root: PathBuf,
    clock: Arc<dyn Clock>,
}

static TRASH_COUNTER: AtomicU64 = AtomicU64::new(0);

impl FsTrashStore {
    /// 默认存储位置 `~/.ccr/skills/trash/`，使用系统时钟。
    pub fn open() -> TrashResult<Self> {
        let root = trash_root().ok_or(TrashError::NoHomeDir)?;
        Self::with_root_and_clock(root, Arc::new(SystemClock))
    }

    /// 测试友好：自定义根目录，系统时钟。
    pub fn with_root(root: PathBuf) -> TrashResult<Self> {
        Self::with_root_and_clock(root, Arc::new(SystemClock))
    }

    /// 测试友好：自定义根目录 + 自定义时钟。
    pub fn with_root_and_clock(root: PathBuf, clock: Arc<dyn Clock>) -> TrashResult<Self> {
        fs::create_dir_all(&root)?;
        Ok(Self { root, clock })
    }

    fn entry_dir(&self, trash_id: &str) -> PathBuf {
        self.root.join(trash_id)
    }

    fn meta_path(&self, trash_id: &str) -> PathBuf {
        self.entry_dir(trash_id).join(META_FILE)
    }

    fn contents_path(&self, trash_id: &str) -> PathBuf {
        self.entry_dir(trash_id).join(CONTENTS_DIR)
    }

    /// 将 skill 目录软删除到回收站。
    /// 先尝试原子 `fs::rename`，失败时（如跨卷）退回到 copy+remove fallback。
    pub fn move_to_trash(&self, skill_path: &Path, skill_name: &str) -> TrashResult<TrashEntry> {
        if !skill_path.exists() {
            return Err(TrashError::SourceNotFound(skill_path.to_path_buf()));
        }

        let now = self.clock.now();
        let expires_at = now + Duration::days(TRASH_TTL_DAYS);
        let id = generate_trash_id();

        let entry_dir = self.entry_dir(&id);
        fs::create_dir_all(&entry_dir)?;

        // 目标 contents 路径不应存在 — 刚 mkdir 父目录
        let contents_dst = self.contents_path(&id);

        // P1-3 修复：跨卷 rename 在 Windows 多磁盘 / Linux 不同挂载点会报
        // `Invalid cross-device link`。先尝试 rename，失败则退回到 copy+remove。
        if let Err(e) = move_dir_robust(skill_path, &contents_dst) {
            // 清理半成品目录，别留垃圾
            let _ = fs::remove_dir_all(&entry_dir);
            return Err(TrashError::Io(e));
        }

        let meta = TrashEntry {
            id: id.clone(),
            skill_name: skill_name.to_string(),
            original_path: skill_path.to_string_lossy().into_owned(),
            deleted_at: now,
            expires_at,
        };

        let meta_bytes = serde_json::to_vec_pretty(&meta)?;
        if let Err(e) = fs::write(self.meta_path(&id), &meta_bytes) {
            // 尽量恢复：把 contents 搬回原位
            let _ = move_dir_robust(&contents_dst, skill_path);
            let _ = fs::remove_dir_all(&entry_dir);
            return Err(TrashError::Io(e));
        }

        Ok(meta)
    }

    /// 列出所有回收站条目，按删除时间倒序。
    pub fn list(&self) -> TrashResult<Vec<TrashEntry>> {
        let read = match fs::read_dir(&self.root) {
            Ok(r) => r,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };

        let mut entries = Vec::new();
        for dir_entry in read {
            let dir_entry = dir_entry?;
            if !dir_entry.file_type()?.is_dir() {
                continue;
            }
            let id = dir_entry.file_name().to_string_lossy().into_owned();
            let meta_path = self.entry_dir(&id).join(META_FILE);
            let bytes = match fs::read(&meta_path) {
                Ok(b) => b,
                Err(_) => continue, // 坏条目或无 meta 的静默跳过
            };
            let entry: TrashEntry = match serde_json::from_slice(&bytes) {
                Ok(e) => e,
                Err(_) => continue,
            };
            entries.push(entry);
        }

        entries.sort_by_key(|entry| std::cmp::Reverse(entry.deleted_at));
        Ok(entries)
    }

    /// 恢复到原路径；若原位置已有同名则用 `<path>-restored-<unix_ts>` 后缀。
    /// 返回实际恢复后的完整路径。
    pub fn restore(&self, trash_id: &str) -> TrashResult<PathBuf> {
        let meta_path = self.meta_path(trash_id);
        let bytes = fs::read(&meta_path).map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => TrashError::EntryNotFound(trash_id.to_string()),
            _ => TrashError::Io(e),
        })?;
        let entry: TrashEntry = serde_json::from_slice(&bytes)?;

        let contents = self.contents_path(trash_id);
        if !contents.exists() {
            return Err(TrashError::InvalidEntry(format!(
                "entry {trash_id} missing contents/"
            )));
        }

        let original = PathBuf::from(&entry.original_path);
        let target = if original.exists() {
            let ts = self.clock.now().timestamp();
            let name = format!(
                "{}-restored-{ts}",
                original
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "skill".to_string())
            );
            original
                .parent()
                .map(|p| p.join(&name))
                .unwrap_or_else(|| PathBuf::from(&name))
        } else {
            original
        };

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        // P1-3 修复：restore 同样可能跨卷，走 robust move
        move_dir_robust(&contents, &target)?;
        // 清理 trash entry 剩余 meta + 空目录
        let _ = fs::remove_file(&meta_path);
        let _ = fs::remove_dir_all(self.entry_dir(trash_id));

        Ok(target)
    }

    /// 永久删除回收站条目。不存在返回 `Ok(false)`。
    pub fn permanent_delete(&self, trash_id: &str) -> TrashResult<bool> {
        let dir = self.entry_dir(trash_id);
        match fs::remove_dir_all(&dir) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    /// 清理所有 `expires_at < now()` 的条目，返回被清理的数量。
    /// **不会 panic**；单条失败不影响其他条目。
    pub fn purge_expired(&self) -> TrashResult<usize> {
        let now = self.clock.now();
        let entries = self.list()?;
        let mut purged = 0;
        for entry in entries {
            if entry.expires_at < now {
                let dir = self.entry_dir(&entry.id);
                if fs::remove_dir_all(&dir).is_ok() {
                    purged += 1;
                }
            }
        }
        Ok(purged)
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

fn generate_trash_id() -> String {
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let seq = TRASH_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mix = (ns ^ ((seq as u128) << 64)).to_le_bytes();
    let digest = blake3::hash(&mix);
    digest.to_hex().as_str()[..16].to_string()
}

/// 健壮的目录移动：优先 `fs::rename`（同卷 atomic），失败时退回到 copy+remove。
/// 处理 Windows 多磁盘 / Linux 不同 mount point 下的跨卷 rename 失败。
fn move_dir_robust(src: &Path, dst: &Path) -> io::Result<()> {
    match fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(e) => {
            // 常见跨卷错误：
            //   Windows: ERROR_NOT_SAME_DEVICE (raw_os_error = 17)
            //   Linux/macOS: EXDEV (raw_os_error = 18)
            // 还有 ErrorKind::CrossesDevices (stable 1.85+)。稳妥起见全部 fallback。
            copy_dir_recursive(src, dst).map_err(|copy_err| {
                // 原始 rename 错误比 copy 错误更有诊断价值，优先返回
                io::Error::new(
                    e.kind(),
                    format!("rename failed: {e}; copy fallback also failed: {copy_err}"),
                )
            })?;
            fs::remove_dir_all(src).map_err(|rm_err| {
                io::Error::other(format!(
                    "copied to {} but failed to remove source {}: {rm_err}",
                    dst.display(),
                    src.display()
                ))
            })?;
            Ok(())
        }
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let sub_src = entry.path();
        let sub_dst = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&sub_src, &sub_dst)?;
        } else if file_type.is_symlink() {
            // 符号链接：读取 target 后在目标位置重建
            let target = fs::read_link(&sub_src)?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, &sub_dst)
                .or_else(|_| fs::copy(&sub_src, &sub_dst).map(|_| ()))?;
            #[cfg(windows)]
            {
                // Windows 重建符号链接需特权；退回到拷贝内容
                let _ = target;
                fs::copy(&sub_src, &sub_dst)?;
            }
            #[cfg(not(any(unix, windows)))]
            fs::copy(&sub_src, &sub_dst)?;
        } else {
            fs::copy(&sub_src, &sub_dst)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trash_id_is_16_hex_chars() {
        let id = generate_trash_id();
        assert_eq!(id.len(), 16);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn trash_ids_are_unique_within_process() {
        let a = generate_trash_id();
        let b = generate_trash_id();
        assert_ne!(a, b);
    }

    #[test]
    fn mutable_clock_advances() {
        let t0 = Utc::now();
        let clock = MutableClock::new(t0);
        assert_eq!(clock.now(), t0);
        let t1 = t0 + Duration::days(10);
        clock.advance_to(t1);
        assert_eq!(clock.now(), t1);
    }
}
