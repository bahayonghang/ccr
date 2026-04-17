//! Skill 自动分类引擎：15 类 + synonym 索引 + Jaccard 合并建议。
//!
//! 移植自 skill-hub `server/scanner/{taxonomy,nlp,similarity}.ts`，关键差异：
//! - 简单 tokenizer 支持英文词边界 + 中文字符 n-gram（2-gram）
//! - Synonym 索引全局 `OnceLock` 只构建一次
//! - frontmatter `category` 永远覆盖自动分类（D04）

pub mod categories;
pub mod nlp;
pub mod synonyms;

use serde::{Deserialize, Serialize};

pub use categories::{CATEGORIES, CATEGORY_OTHER, CategoryDef, category_by_id};
pub use nlp::{jaccard, tokenize};

/// 分类输入：skill 的关键可分类字段。
#[derive(Debug, Clone)]
pub struct SkillInput<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub description: &'a str,
    /// 来自 frontmatter `category` — 任何非空值都会直接作为结果
    pub frontmatter_category: Option<&'a str>,
}

/// 单条 skill 的分类结果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Classification {
    pub skill_id: String,
    pub category_id: String,
    pub matched_by: MatchSource,
}

/// 分类命中方式，前端可据此展示"自动"或"用户指定"徽标。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MatchSource {
    Frontmatter,
    Keyword,
    Fallback,
}

/// 类别汇总（用于前端 chip 筛选）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategorySummary {
    pub id: String,
    pub name_en: String,
    pub name_zh: String,
    pub icon: String,
    pub count: usize,
    pub skill_ids: Vec<String>,
}

/// 合并建议：同类内两两 Jaccard ≥ 0.3 的 skill 对。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeSuggestion {
    pub category_id: String,
    pub category_name: String,
    pub reason: String,
    pub skills: [SkillRef; 2],
    /// 0.0-1.0，保留 2 位小数
    pub similarity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillRef {
    pub id: String,
    pub name: String,
}

/// 合并建议的 Jaccard 阈值。
pub const MERGE_SUGGESTION_THRESHOLD: f32 = 0.3;

/// 分类单条 skill。
pub fn classify(skill: &SkillInput) -> Classification {
    // 1. frontmatter.category 优先
    if let Some(raw) = skill.frontmatter_category {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            let normalized = trimmed.to_lowercase();
            // 先按 id 匹配
            if let Some(cat) = category_by_id(&normalized) {
                return Classification {
                    skill_id: skill.id.to_string(),
                    category_id: cat.id.to_string(),
                    matched_by: MatchSource::Frontmatter,
                };
            }
            // 再按 name_en / name_zh 原文匹配
            for cat in CATEGORIES {
                if cat.name_en.eq_ignore_ascii_case(trimmed) || cat.name_zh == trimmed {
                    return Classification {
                        skill_id: skill.id.to_string(),
                        category_id: cat.id.to_string(),
                        matched_by: MatchSource::Frontmatter,
                    };
                }
            }
        }
    }

    // 2. 按关键词 token 交集打分
    let synonyms = synonyms::get_synonym_index();
    let text = format!("{} {}", skill.name, skill.description);
    let skill_tokens = tokenize(&text, synonyms);

    if skill_tokens.is_empty() {
        return Classification {
            skill_id: skill.id.to_string(),
            category_id: CATEGORY_OTHER.to_string(),
            matched_by: MatchSource::Fallback,
        };
    }

    let mut best_id = CATEGORY_OTHER;
    let mut best_score = 0.0f32;
    let mut best_priority = 0u32;

    for cat in CATEGORIES {
        if cat.id == CATEGORY_OTHER || cat.keywords.is_empty() {
            continue;
        }
        let cat_tokens = tokenize(&cat.keywords.join(" "), synonyms);
        if cat_tokens.is_empty() {
            continue;
        }
        let overlap = skill_tokens
            .iter()
            .filter(|t| cat_tokens.contains(*t))
            .count();
        if overlap == 0 {
            continue;
        }
        let score = overlap as f32 / skill_tokens.len() as f32;
        if score > best_score || (score == best_score && cat.priority > best_priority) {
            best_score = score;
            best_priority = cat.priority;
            best_id = cat.id;
        }
    }

    Classification {
        skill_id: skill.id.to_string(),
        category_id: best_id.to_string(),
        matched_by: if best_id == CATEGORY_OTHER {
            MatchSource::Fallback
        } else {
            MatchSource::Keyword
        },
    }
}

/// 批量分类并汇总。
pub fn classify_all(skills: &[SkillInput]) -> (Vec<Classification>, Vec<CategorySummary>) {
    let classifications: Vec<Classification> = skills.iter().map(classify).collect();

    let mut summaries: Vec<CategorySummary> = Vec::new();
    for cat in CATEGORIES {
        let ids: Vec<String> = classifications
            .iter()
            .filter(|c| c.category_id == cat.id)
            .map(|c| c.skill_id.clone())
            .collect();
        if ids.is_empty() {
            continue;
        }
        summaries.push(CategorySummary {
            id: cat.id.to_string(),
            name_en: cat.name_en.to_string(),
            name_zh: cat.name_zh.to_string(),
            icon: cat.icon.to_string(),
            count: ids.len(),
            skill_ids: ids,
        });
    }

    (classifications, summaries)
}

/// 基于分类结果生成合并建议。
pub fn merge_suggestions(
    skills: &[SkillInput],
    classifications: &[Classification],
) -> Vec<MergeSuggestion> {
    let synonyms = synonyms::get_synonym_index();
    let mut result = Vec::new();

    for cat in CATEGORIES {
        let ids_in_cat: Vec<&str> = classifications
            .iter()
            .filter(|c| c.category_id == cat.id)
            .map(|c| c.skill_id.as_str())
            .collect();
        if ids_in_cat.len() < 2 {
            continue;
        }

        // 为该类所有 skill tokenize 一次
        let tokens_by_id: std::collections::HashMap<&str, std::collections::BTreeSet<String>> =
            ids_in_cat
                .iter()
                .filter_map(|id| {
                    skills.iter().find(|s| s.id == *id).map(|s| {
                        let text = format!("{} {}", s.name, s.description);
                        (*id, tokenize(&text, synonyms))
                    })
                })
                .collect();

        for i in 0..ids_in_cat.len() {
            for j in (i + 1)..ids_in_cat.len() {
                let id_a = ids_in_cat[i];
                let id_b = ids_in_cat[j];
                let skill_a = skills.iter().find(|s| s.id == id_a);
                let skill_b = skills.iter().find(|s| s.id == id_b);
                let (Some(a), Some(b)) = (skill_a, skill_b) else {
                    continue;
                };
                if a.name == b.name {
                    continue; // 同名归为冲突，不是合并候选
                }
                let ta = tokens_by_id.get(id_a);
                let tb = tokens_by_id.get(id_b);
                let (Some(ta), Some(tb)) = (ta, tb) else {
                    continue;
                };
                let sim = jaccard(ta, tb);
                if sim >= MERGE_SUGGESTION_THRESHOLD {
                    result.push(MergeSuggestion {
                        category_id: cat.id.to_string(),
                        category_name: cat.name_zh.to_string(),
                        reason: format!(
                            "同属「{}」({}%) — 建议合并为单一 skill 涵盖子场景",
                            cat.name_zh,
                            (sim * 100.0).round() as u32
                        ),
                        skills: [
                            SkillRef {
                                id: a.id.to_string(),
                                name: a.name.to_string(),
                            },
                            SkillRef {
                                id: b.id.to_string(),
                                name: b.name.to_string(),
                            },
                        ],
                        similarity: (sim * 100.0).round() / 100.0,
                    });
                }
            }
        }
    }

    result.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    result
}
