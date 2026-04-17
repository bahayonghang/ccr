//! skills_ext — skill-hub 能力嫁接扩展模块
//!
//! 提供 skill 版本历史、回收站、自动分类、冲突检测、Agent 注册表等增强能力。
//! 本模块是**增量扩展**，不修改 `services::skills_service` 的现有 API。
//!
//! ## Layout
//! - `agents/`  — Agent 注册表 (`trait AgentLocator` + 5 实现)
//! - `paths`    — `~/.ccr/skills/` 目录布局辅助
//! - `SkillScope` / `SkillSourceKind` / `SkillHealthReport` — 公共类型

pub mod agents;
pub mod conflicts;
pub mod hash;
pub mod health;
pub mod lcs;
pub mod paths;
pub mod plugins;
pub mod taxonomy;
pub mod toggle;
pub mod trash;
pub mod versioning;

pub use agents::{AgentId, AgentLocator, all_agents, locator_for};
pub use conflicts::{ConflictGroup, detect_conflicts};
pub use health::{HealthReport, compute as compute_health};
pub use plugins::{enabled_plugin_install_locations, find_plugin_skills_dirs};
pub use taxonomy::{
    CATEGORIES, CATEGORY_OTHER, CategorySummary, Classification, MERGE_SUGGESTION_THRESHOLD,
    MatchSource, MergeSuggestion, SkillInput, SkillRef, classify, classify_all, merge_suggestions,
};
pub use toggle::{ToggleError, ToggleResult, ToggleStore, default_settings_path};
pub use trash::{
    Clock, FsTrashStore, MutableClock, SystemClock, TRASH_TTL_DAYS, TrashEntry, TrashError,
    TrashResult,
};
pub use versioning::{
    DiffResult, DiffStats, FsVersionStore, MAX_VERSIONS_PER_SKILL, SnapshotSource, Version,
    VersionMeta, VersioningError, VersioningResult,
};

use serde::{Deserialize, Serialize};

/// Skill 所属范围。对应 skill-hub 中的 `global / project / plugin`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillScope {
    Global,
    Project,
    Plugin,
}

/// Skill 物理来源识别。
/// `Local` = 真实文件；其它几种为符号链接指向的共享目录。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillSourceKind {
    Local,
    Newmax,
    Agents,
    Symlink,
    Unknown,
}

/// Skill 仓库健康诊断汇总 (Phase 7 填充)。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillHealthReport {
    pub total: usize,
    pub conflicts: usize,
    pub similar_groups: usize,
    pub disabled: usize,
    pub orphans: usize,
}
