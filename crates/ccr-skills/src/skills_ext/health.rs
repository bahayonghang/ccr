//! 健康诊断：聚合 conflicts / merge suggestions / disabled 等指标。
//!
//! 前端可直接展示一张健康卡片。

use serde::{Deserialize, Serialize};

use super::conflicts::ConflictGroup;
use super::taxonomy::MergeSuggestion;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    pub total: usize,
    pub conflicts: usize,
    pub merge_suggestions: usize,
    pub disabled: usize,
    pub plugin_locations: usize,
}

/// 根据各子系统的输入汇总健康快照。
pub fn compute(
    total_skills: usize,
    conflicts: &[ConflictGroup],
    merge_suggestions_list: &[MergeSuggestion],
    disabled_names: &[String],
    plugin_locations: usize,
) -> HealthReport {
    HealthReport {
        total: total_skills,
        conflicts: conflicts.len(),
        merge_suggestions: merge_suggestions_list.len(),
        disabled: disabled_names.len(),
        plugin_locations,
    }
}
