//! File-backed version store for skill 目录快照。
//!
//! 对应参考实现 `skill-hub/server/versioning/store.ts`，关键差异：
//! - 纯 Rust 无新增依赖；使用 `blake3` (已在 deps) 替代 sha256
//! - `BTreeMap<String, String>` 保证附属文件序列化顺序稳定
//! - 50 版本 FIFO 淘汰（[`MAX_VERSIONS_PER_SKILL`]），防磁盘爆炸
//! - 单文件上限 1 MiB、二进制自动跳过（前 8 KB 含 NUL 判定）
//! - 原子写：temp + rename，不引入 fs2 / tempfile prod deps
//!
//! ## 存储布局
//! ```text
//! <root>/<path_hash>/<version_id>.json
//! ```
//! `<root>` 默认为 `~/.ccr/skills/versions/`，测试可注入临时目录。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::hash::{content_hash, path_hash};
use super::lcs::{self, DiffLine, DiffLineKind, TruncationInfo};
use super::paths::versions_root;

/// 单 skill 保留的最大版本数。超限按时间倒序 FIFO 淘汰。
pub const MAX_VERSIONS_PER_SKILL: usize = 50;

/// 单附属文件捕获上限（1 MiB）。超限静默跳过。
const MAX_FILE_BYTES: u64 = 1 << 20;

/// 二进制检测窗口大小。
const BINARY_PROBE_BYTES: usize = 8192;

// ============================================================================
// 错误类型
// ============================================================================

/// Version store 操作错误。自包含，不依赖 `anyhow`/`thiserror`。
#[derive(Debug)]
pub enum VersioningError {
    Io(io::Error),
    Json(serde_json::Error),
    NoHomeDir,
    SkillMdNotFound(PathBuf),
    VersionNotFound {
        skill_path: String,
        version_id: String,
    },
}

impl std::fmt::Display for VersioningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Json(e) => write!(f, "JSON serialization error: {e}"),
            Self::NoHomeDir => write!(f, "Cannot locate home directory"),
            Self::SkillMdNotFound(p) => write!(f, "SKILL.md not found at {}", p.display()),
            Self::VersionNotFound {
                skill_path,
                version_id,
            } => write!(
                f,
                "Version {version_id} not found for skill at {skill_path}"
            ),
        }
    }
}

impl std::error::Error for VersioningError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for VersioningError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for VersioningError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

pub type VersioningResult<T> = Result<T, VersioningError>;

// ============================================================================
// 数据类型
// ============================================================================

/// 快照来源：`Auto` = 编辑保存 / 回滚等自动触发；`Manual` = 用户手动"打标"。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotSource {
    Auto,
    Manual,
}

/// 版本元数据（列表视图用）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionMeta {
    pub id: String,
    pub skill_path: String,
    pub skill_name: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub source: SnapshotSource,
    pub content_hash: String,
}

/// 完整版本（含内容 + 附属文件）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    pub skill_path: String,
    pub skill_name: String,
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub source: SnapshotSource,
    pub content_hash: String,
    pub content: String,
    pub files: BTreeMap<String, String>,
}

impl From<&Version> for VersionMeta {
    fn from(v: &Version) -> Self {
        Self {
            id: v.id.clone(),
            skill_path: v.skill_path.clone(),
            skill_name: v.skill_name.clone(),
            timestamp: v.timestamp,
            message: v.message.clone(),
            source: v.source,
            content_hash: v.content_hash.clone(),
        }
    }
}

/// 两个版本间的 diff 统计。
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DiffStats {
    pub additions: usize,
    pub deletions: usize,
    pub unchanged: usize,
}

/// 完整 diff 结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffResult {
    pub old_version: VersionMeta,
    pub new_version: VersionMeta,
    pub lines: Vec<DiffLine>,
    pub stats: DiffStats,
    /// P2-3 修复：超长文本被 LCS 截断到 MAX_LINES 时 UI 可据此显示警告。
    #[serde(default)]
    pub truncation: TruncationInfo,
}

// ============================================================================
// FsVersionStore
// ============================================================================

/// 文件系统版本存储。
pub struct FsVersionStore {
    root: PathBuf,
}

impl FsVersionStore {
    /// 默认存储位置 `~/.ccr/skills/versions/`。
    pub fn open() -> VersioningResult<Self> {
        let root = versions_root().ok_or(VersioningError::NoHomeDir)?;
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    /// 用指定根目录打开（集成测试用）。
    pub fn with_root(root: PathBuf) -> VersioningResult<Self> {
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    fn skill_dir(&self, skill_path: &Path) -> PathBuf {
        self.root.join(path_hash(skill_path))
    }

    /// 创建一个快照。若最新版本的 content_hash 相同则复用（去重），不写新文件。
    pub fn snapshot(
        &self,
        skill_path: &Path,
        skill_name: &str,
        message: &str,
        source: SnapshotSource,
    ) -> VersioningResult<VersionMeta> {
        let skill_md = skill_path.join("SKILL.md");
        let content = match fs::read_to_string(&skill_md) {
            Ok(c) => c,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                return Err(VersioningError::SkillMdNotFound(skill_md));
            }
            Err(e) => return Err(VersioningError::Io(e)),
        };

        let files = collect_text_files(skill_path);
        let hash = content_hash(&content, &files);

        // 内容未变：复用已有最新版本，避免重复写文件。
        let history = self.history(skill_path)?;
        if let Some(latest) = history.first()
            && latest.content_hash == hash
        {
            return Ok(latest.clone());
        }

        let id = generate_version_id();
        let version = Version {
            id: id.clone(),
            skill_path: skill_path.to_string_lossy().into_owned(),
            skill_name: skill_name.to_string(),
            timestamp: Utc::now(),
            message: message.to_string(),
            source,
            content_hash: hash,
            content,
            files,
        };

        let dir = self.skill_dir(skill_path);
        fs::create_dir_all(&dir)?;
        let json = serde_json::to_vec_pretty(&version)?;
        let final_path = dir.join(format!("{id}.json"));
        atomic_write(&final_path, &json)?;

        self.enforce_retention(skill_path)?;

        Ok(VersionMeta::from(&version))
    }

    /// 获取按时间倒序排列的版本元列表。
    pub fn history(&self, skill_path: &Path) -> VersioningResult<Vec<VersionMeta>> {
        let dir = self.skill_dir(skill_path);
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };

        let mut versions = Vec::new();
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let bytes = match fs::read(&path) {
                Ok(b) => b,
                Err(_) => continue,
            };
            let v: Version = match serde_json::from_slice(&bytes) {
                Ok(v) => v,
                Err(_) => continue, // 坏文件静默跳过，防止整个列表爆掉
            };
            versions.push(VersionMeta::from(&v));
        }

        versions.sort_by_key(|version| std::cmp::Reverse(version.timestamp));
        Ok(versions)
    }

    /// 读取某个版本的完整内容。不存在返回 `Ok(None)`。
    pub fn get(&self, skill_path: &Path, version_id: &str) -> VersioningResult<Option<Version>> {
        let path = self
            .skill_dir(skill_path)
            .join(format!("{version_id}.json"));
        match fs::read(&path) {
            Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// 对比两个版本。任一版本不存在返回 `Ok(None)`。
    pub fn diff(
        &self,
        skill_path: &Path,
        old_id: &str,
        new_id: &str,
    ) -> VersioningResult<Option<DiffResult>> {
        let Some(old_v) = self.get(skill_path, old_id)? else {
            return Ok(None);
        };
        let Some(new_v) = self.get(skill_path, new_id)? else {
            return Ok(None);
        };
        let (lines, truncation) = lcs::diff_with_truncation(&old_v.content, &new_v.content);
        let stats = DiffStats {
            additions: lines.iter().filter(|l| l.kind == DiffLineKind::Add).count(),
            deletions: lines
                .iter()
                .filter(|l| l.kind == DiffLineKind::Remove)
                .count(),
            unchanged: lines
                .iter()
                .filter(|l| l.kind == DiffLineKind::Same)
                .count(),
        };
        Ok(Some(DiffResult {
            old_version: VersionMeta::from(&old_v),
            new_version: VersionMeta::from(&new_v),
            lines,
            stats,
            truncation,
        }))
    }

    /// 回滚到指定版本。回滚流程：
    /// 1. 对当前未保存状态自动快照（安全网）
    /// 2. 覆盖写 SKILL.md 与附属文件（含子目录）
    /// 3. 对回滚后状态再次快照（审计痕迹）
    pub fn rollback(&self, skill_path: &Path, version_id: &str) -> VersioningResult<VersionMeta> {
        let target =
            self.get(skill_path, version_id)?
                .ok_or_else(|| VersioningError::VersionNotFound {
                    skill_path: skill_path.to_string_lossy().into_owned(),
                    version_id: version_id.to_string(),
                })?;

        // 回滚前安全网：尽力保存当前状态，即使失败也继续。
        let _ = self.snapshot(
            skill_path,
            &target.skill_name,
            &format!("rollback-safety: before restoring {version_id}"),
            SnapshotSource::Auto,
        );

        // P1-5 修复：先删除"当前存在但目标快照中不存在"的 orphan 文件。
        // 防止 rollback 产生混合状态（snapshot={a,b}，current={a,b,c} → 回滚后 c 仍然残留）。
        // SKILL.md 永远由后续步骤覆盖写，所以不在此处理。
        let current_files = collect_text_files(skill_path);
        for rel in current_files.keys() {
            if !target.files.contains_key(rel) {
                let full = skill_path.join(rel);
                let _ = fs::remove_file(&full);
                // 不删空目录：保留结构，避免 fs::create_dir_all 重复工作；
                // walkdir 下次扫描自然会跳过空目录
            }
        }

        // 还原 SKILL.md
        let skill_md = skill_path.join("SKILL.md");
        atomic_write(&skill_md, target.content.as_bytes())?;

        // 还原附属文件（含子目录）
        for (rel, body) in &target.files {
            let full = skill_path.join(rel);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent)?;
            }
            atomic_write(&full, body.as_bytes())?;
        }

        // 回滚后审计快照
        self.snapshot(
            skill_path,
            &target.skill_name,
            &format!("rollback-applied: restored {version_id}"),
            SnapshotSource::Auto,
        )
    }

    /// 硬删指定版本；不存在返回 `Ok(false)`。
    pub fn delete(&self, skill_path: &Path, version_id: &str) -> VersioningResult<bool> {
        let path = self
            .skill_dir(skill_path)
            .join(format!("{version_id}.json"));
        match fs::remove_file(&path) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    /// 按时间倒序保留最新 [`MAX_VERSIONS_PER_SKILL`] 个，其余删除。
    fn enforce_retention(&self, skill_path: &Path) -> VersioningResult<()> {
        let history = self.history(skill_path)?;
        if history.len() <= MAX_VERSIONS_PER_SKILL {
            return Ok(());
        }
        let dir = self.skill_dir(skill_path);
        for meta in history.iter().skip(MAX_VERSIONS_PER_SKILL) {
            let path = dir.join(format!("{}.json", meta.id));
            let _ = fs::remove_file(&path);
        }
        Ok(())
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 全局单调计数器，用于版本 id 与原子写临时文件名。
static GLOBAL_COUNTER: AtomicU64 = AtomicU64::new(0);

fn generate_version_id() -> String {
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let seq = GLOBAL_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mix = (ns ^ ((seq as u128) << 64)).to_le_bytes();
    let digest = blake3::hash(&mix);
    digest.to_hex().as_str()[..16].to_string()
}

/// 原子写文件：tmp → rename。在 Windows 上会覆盖已存在的目标 (MOVEFILE_REPLACE_EXISTING)。
fn atomic_write(final_path: &Path, data: &[u8]) -> VersioningResult<()> {
    let parent = final_path.parent().ok_or_else(|| {
        VersioningError::Io(io::Error::new(io::ErrorKind::InvalidInput, "invalid path"))
    })?;
    fs::create_dir_all(parent)?;

    let pid = std::process::id();
    let seq = GLOBAL_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_name = format!(".ccr-skills-tmp-{pid:x}-{seq:x}");
    let tmp_path = parent.join(tmp_name);

    fs::write(&tmp_path, data)?;
    if let Err(e) = fs::rename(&tmp_path, final_path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(VersioningError::Io(e));
    }
    Ok(())
}

/// 递归收集 skill 目录下的文本文件（排除 `SKILL.md` 本身与隐藏目录）。
/// 静默跳过：无法访问 / 超 1 MiB / 前 8 KB 含 NUL / 非 UTF-8。
fn collect_text_files(base: &Path) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    for entry in walkdir::WalkDir::new(base)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if e.depth() == 0 {
                return true;
            }
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.')
        })
    {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_file() {
            continue;
        }
        let Ok(rel) = entry.path().strip_prefix(base) else {
            continue;
        };
        if rel == Path::new("SKILL.md") {
            continue;
        }
        let Ok(meta) = entry.metadata() else { continue };
        if meta.len() > MAX_FILE_BYTES {
            continue;
        }
        let Ok(bytes) = fs::read(entry.path()) else {
            continue;
        };
        let probe = &bytes[..bytes.len().min(BINARY_PROBE_BYTES)];
        if probe.contains(&0) {
            continue;
        }
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        files.insert(rel_str, text);
    }
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_id_is_16_hex_chars() {
        let id = generate_version_id();
        assert_eq!(id.len(), 16);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn version_ids_from_same_process_are_unique() {
        let a = generate_version_id();
        let b = generate_version_id();
        assert_ne!(a, b, "连续调用必须 ID 不同");
    }
}
