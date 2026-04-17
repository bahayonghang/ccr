//! skills_ext 的目录布局辅助函数。
//!
//! 所有 skill-hub 能力嫁接的持久化数据（版本快照、回收站）都放在 `~/.ccr/skills/`，
//! 与 ccr 现有配置互不干扰，主人可直接打包备份整个目录。
//!
//! ## Layout
//! ```text
//! ~/.ccr/skills/
//! ├── versions/
//! │   └── <blake3(skill_path)[0..24]>/
//! │       └── <version_id>.json
//! └── trash/
//!     └── <trash_id>/
//!         ├── meta.json
//!         └── <original_skill_contents>
//! ```

use std::path::PathBuf;

/// Skills 扩展数据根目录。`~/.ccr/skills/`。
/// 当系统无法定位 home directory 时返回 `None`。
pub fn skills_root() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".ccr").join("skills"))
}

/// 版本快照存储目录。`~/.ccr/skills/versions/`。
pub fn versions_root() -> Option<PathBuf> {
    skills_root().map(|root| root.join("versions"))
}

/// 回收站根目录。`~/.ccr/skills/trash/`。
pub fn trash_root() -> Option<PathBuf> {
    skills_root().map(|root| root.join("trash"))
}

/// 单个 skill 的版本目录：`versions/<hash>/`。
/// `skill_path_hash` 在 Phase 2 由 `blake3(skill_path).hex()[..24]` 生成。
pub fn version_dir_for(skill_path_hash: &str) -> Option<PathBuf> {
    versions_root().map(|root| root.join(skill_path_hash))
}

/// 单个 trash 条目目录：`trash/<trash_id>/`。
pub fn trash_entry_dir(trash_id: &str) -> Option<PathBuf> {
    trash_root().map(|root| root.join(trash_id))
}
