//! 冲突检测：同名 skill 跨 scope 且 realPath 不同视为冲突。
//!
//! Symlink 解析后 realPath 相同的条目自动去重（skill-hub 原版语义）。
//! P2-1 修复：路径比较时归一化大小写与分隔符，避免 `C:\Foo\skills` vs `c:\foo\skills`
//! 被误判为冲突。Unix 大小写敏感，Windows 大小写不敏感。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 冲突组：同 name 但不同 realPath 的 skill 条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictGroup {
    pub name: String,
    pub skill_ids: Vec<String>,
    pub real_paths: Vec<String>,
}

/// 归一化路径：统一分隔符为 `/`；Windows 下额外折叠大小写。
fn normalize_path(path: &str) -> String {
    let unified = path.replace('\\', "/");
    if cfg!(windows) {
        unified.to_ascii_lowercase()
    } else {
        unified
    }
}

/// 输入：`(skill_id, name, real_path)` 三元组。
/// 输出：conflict group 列表（按 name 排序）。
pub fn detect_conflicts(entries: &[(&str, &str, &str)]) -> Vec<ConflictGroup> {
    // 按 name 分组；每组内按归一化后的 real_path 去重
    let mut by_name: BTreeMap<&str, Vec<(&str, &str, String)>> = BTreeMap::new();
    for (id, name, real_path) in entries {
        let normalized = normalize_path(real_path);
        by_name
            .entry(*name)
            .or_default()
            .push((id, real_path, normalized));
    }

    let mut result = Vec::new();
    for (name, items) in by_name {
        let mut unique_normalized: Vec<&str> = items.iter().map(|(_, _, n)| n.as_str()).collect();
        unique_normalized.sort();
        unique_normalized.dedup();
        if unique_normalized.len() < 2 {
            continue;
        }
        // 展示用原始路径（不是归一化结果）
        let mut display_paths: Vec<&str> = items.iter().map(|(_, p, _)| *p).collect();
        display_paths.sort();
        display_paths.dedup();
        result.push(ConflictGroup {
            name: name.to_string(),
            skill_ids: items.iter().map(|(id, _, _)| id.to_string()).collect(),
            real_paths: display_paths.into_iter().map(String::from).collect(),
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_unifies_separators() {
        let a = normalize_path("C:\\Foo\\skills");
        let b = normalize_path("C:/Foo/skills");
        assert_eq!(a, b);
    }

    #[test]
    #[cfg(windows)]
    fn normalize_case_folds_on_windows() {
        assert_eq!(
            normalize_path("C:/Foo/skills"),
            normalize_path("c:/foo/skills")
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn normalize_preserves_case_on_unix() {
        assert_ne!(normalize_path("/Foo/skills"), normalize_path("/foo/skills"));
    }
}
