//! Phase 6 — skills_ext Taxonomy 集成测试。

use ccr_skills::skills_ext::taxonomy::{
    CATEGORY_OTHER, MERGE_SUGGESTION_THRESHOLD, MatchSource, SkillInput, classify, classify_all,
    merge_suggestions,
};

fn input<'a>(id: &'a str, name: &'a str, desc: &'a str) -> SkillInput<'a> {
    SkillInput {
        id,
        name,
        description: desc,
        frontmatter_category: None,
    }
}

fn input_fm<'a>(id: &'a str, name: &'a str, desc: &'a str, fm: &'a str) -> SkillInput<'a> {
    SkillInput {
        id,
        name,
        description: desc,
        frontmatter_category: Some(fm),
    }
}

#[test]
fn test_classify_frontmatter_override_wins() {
    let skill = input_fm("1", "TDD Runner", "Python pytest automation", "image-gen");
    let c = classify(&skill);
    assert_eq!(c.category_id, "image-gen");
    assert_eq!(c.matched_by, MatchSource::Frontmatter);
}

#[test]
fn test_classify_frontmatter_accepts_chinese_name() {
    let skill = input_fm("1", "Something", "unrelated", "代码开发");
    let c = classify(&skill);
    assert_eq!(c.category_id, "code-dev");
}

#[test]
fn test_classify_by_keyword_code_dev() {
    let skill = input("1", "TDD Runner", "Python pytest + debug helper");
    let c = classify(&skill);
    assert_eq!(c.category_id, "code-dev");
    assert_eq!(c.matched_by, MatchSource::Keyword);
}

#[test]
fn test_classify_by_keyword_image_gen() {
    let skill = input("1", "Cover Maker", "生成文章封面配图");
    let c = classify(&skill);
    assert_eq!(c.category_id, "image-gen");
}

#[test]
fn test_classify_by_keyword_social() {
    let skill = input("1", "XHS Post Helper", "帮写小红书营销内容");
    let c = classify(&skill);
    assert_eq!(c.category_id, "social");
}

#[test]
fn test_classify_empty_falls_to_other() {
    let skill = input("1", "", "");
    let c = classify(&skill);
    assert_eq!(c.category_id, CATEGORY_OTHER);
    assert_eq!(c.matched_by, MatchSource::Fallback);
}

#[test]
fn test_classify_unknown_words_falls_to_other() {
    let skill = input("1", "zzqwxx", "unrelated stuff aabbcc");
    let c = classify(&skill);
    // 纯乱码英文词都没有匹配任何 category，应兜底
    assert_eq!(c.category_id, CATEGORY_OTHER);
}

#[test]
fn test_classify_all_summarizes_counts() {
    let skills = [
        input("a", "TDD Runner", "pytest"),
        input("b", "Golang API Debug", "backend"),
        input("c", "Cover Maker", "图片封面"),
    ];
    let (classifications, summaries) = classify_all(&skills);
    assert_eq!(classifications.len(), 3);

    let code_dev = summaries
        .iter()
        .find(|s| s.id == "code-dev")
        .expect("code-dev");
    assert_eq!(code_dev.count, 2);

    let image_gen = summaries
        .iter()
        .find(|s| s.id == "image-gen")
        .expect("image-gen");
    assert_eq!(image_gen.count, 1);
}

#[test]
fn test_merge_suggestions_above_threshold() {
    // 两个 skill 都关于 Python pytest debug — 应高 Jaccard
    let skills = [
        input("a", "Python pytest helper", "pytest debug runner"),
        input("b", "pytest runner debug", "Python pytest helper tool"),
        input("c", "小红书文案", "xhs content"),
    ];
    let (classifications, _) = classify_all(&skills);
    let suggestions = merge_suggestions(&skills, &classifications);

    // 至少应有一条 a+b 的合并建议
    let has_ab = suggestions.iter().any(|s| {
        let ids: Vec<&str> = s.skills.iter().map(|r| r.id.as_str()).collect();
        ids.contains(&"a") && ids.contains(&"b")
    });
    assert!(
        has_ab,
        "应产生 a+b 的合并建议，实际 suggestions: {suggestions:?}"
    );

    // 所有建议的 similarity >= 阈值
    for s in &suggestions {
        assert!(s.similarity >= MERGE_SUGGESTION_THRESHOLD);
    }
}

#[test]
fn test_merge_suggestions_skip_same_name_conflicts() {
    // 同名 skill 视为冲突（Phase 7 处理），不出现在 merge 建议里
    let skills = [
        input("a", "TDD Runner", "python"),
        input("b", "TDD Runner", "python tests debug"),
    ];
    let (classifications, _) = classify_all(&skills);
    let suggestions = merge_suggestions(&skills, &classifications);
    assert!(
        suggestions.is_empty(),
        "同名 skill 不应产生 merge 建议，实际: {suggestions:?}"
    );
}
