//! NLP 辅助：tokenize 与 Jaccard 相似度。
//!
//! Tokenizer 规则：
//! - 英文：按 `\W+` 切词，全小写，过滤长度 <2
//! - 中文：字符 2-gram 捕获基本语义；避免 jieba 等重依赖
//! - Synonym 映射：同义词在入表时已折叠到 canonical form

use std::collections::{BTreeSet, HashMap};

/// 对文本分词并应用 synonym 映射。返回去重后的 BTreeSet 便于后续 Jaccard。
pub fn tokenize(text: &str, synonyms: &HashMap<String, String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let lower = text.to_lowercase();

    // 1. 英文词：按非字母数字（ASCII）切分
    for word in lower.split(|c: char| !c.is_ascii_alphanumeric() && !c.is_ascii_punctuation()) {
        let cleaned: String = word
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if cleaned.len() < 2 {
            continue;
        }
        if !cleaned.is_ascii() {
            continue;
        }
        let canonical = synonyms.get(&cleaned).cloned().unwrap_or(cleaned);
        out.insert(canonical);
    }

    // 2. 中文字符 2-gram（以及单字符放入 synonym 索引匹配）
    let cjk_chars: Vec<char> = lower.chars().filter(|c| is_cjk(*c)).collect();
    for i in 0..cjk_chars.len() {
        // 单字符 synonym 查表（用于 "微博" → "weibo" 之类的索引命中）
        let single: String = cjk_chars[i].to_string();
        if let Some(canonical) = synonyms.get(&single) {
            out.insert(canonical.clone());
        }
        // 2-gram
        if i + 1 < cjk_chars.len() {
            let bigram: String = cjk_chars[i..=i + 1].iter().collect();
            let canonical = synonyms.get(&bigram).cloned().unwrap_or(bigram);
            out.insert(canonical);
        }
    }

    out
}

/// Jaccard 相似度：|A ∩ B| / |A ∪ B|，范围 [0, 1]。
pub fn jaccard(a: &BTreeSet<String>, b: &BTreeSet<String>) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count();
    let union = a.len() + b.len() - intersection;
    if union == 0 {
        return 0.0;
    }
    intersection as f32 / union as f32
}

fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}'     // CJK Unified Ideographs
        | '\u{3400}'..='\u{4DBF}'   // CJK Extension A
        | '\u{3000}'..='\u{303F}'   // CJK Symbols & Punctuation
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_syn() -> HashMap<String, String> {
        HashMap::new()
    }

    #[test]
    fn tokenize_english_words() {
        let tokens = tokenize("TDD runner for Python debug", &empty_syn());
        assert!(tokens.contains("tdd"));
        assert!(tokens.contains("runner"));
        assert!(tokens.contains("python"));
        assert!(tokens.contains("debug"));
        // "for" 也会保留 — 本 tokenizer 不做 stop word 过滤
        assert!(tokens.contains("for"));
    }

    #[test]
    fn tokenize_chinese_bigrams() {
        let tokens = tokenize("小红书营销文案", &empty_syn());
        assert!(tokens.contains("小红"));
        assert!(tokens.contains("红书"));
        assert!(tokens.contains("书营"));
    }

    #[test]
    fn tokenize_empty() {
        assert!(tokenize("", &empty_syn()).is_empty());
        assert!(tokenize("    ", &empty_syn()).is_empty());
    }

    #[test]
    fn jaccard_identical() {
        let a: BTreeSet<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        assert_eq!(jaccard(&a, &a), 1.0);
    }

    #[test]
    fn jaccard_disjoint() {
        let a: BTreeSet<String> = ["a"].iter().map(|s| s.to_string()).collect();
        let b: BTreeSet<String> = ["b"].iter().map(|s| s.to_string()).collect();
        assert_eq!(jaccard(&a, &b), 0.0);
    }

    #[test]
    fn jaccard_half_overlap() {
        let a: BTreeSet<String> = ["a", "b"].iter().map(|s| s.to_string()).collect();
        let b: BTreeSet<String> = ["b", "c"].iter().map(|s| s.to_string()).collect();
        // |intersection|=1, |union|=3 → 1/3
        let sim = jaccard(&a, &b);
        assert!((sim - 1.0 / 3.0).abs() < 1e-6);
    }
}
