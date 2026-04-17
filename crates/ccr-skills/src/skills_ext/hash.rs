//! Content & path hashing helpers for skills_ext.
//!
//! 复用 workspace 已有的 `blake3` 依赖 — 零新增外部 crate。
//! - `path_hash`: 稳定 skill 目录标识符（24 hex = 96-bit 抗碰撞）
//! - `content_hash`: SKILL.md 与附属文件的组合指纹（32 hex = 128-bit）

use std::collections::BTreeMap;
use std::path::Path;

/// 为 skill 的文件系统路径生成稳定短 id。
/// 返回 blake3 的前 24 个 hex 字符 (96-bit 碰撞空间)，足够 100 万级 skill 数据。
pub fn path_hash(skill_path: &Path) -> String {
    let s = skill_path.to_string_lossy();
    let digest = blake3::hash(s.as_bytes());
    digest.to_hex().as_str()[..24].to_string()
}

/// 为一次 skill 快照生成内容指纹。
/// 组合 SKILL.md 正文 + 附属文件（已按字典序排列的 BTreeMap）。
/// 使用哨兵字节 `\x00 / \x01 / \x02` 避免 "文件名 + 内容" 拼接歧义。
pub fn content_hash(content: &str, files: &BTreeMap<String, String>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(content.as_bytes());
    hasher.update(b"\x00FILES\x00");
    for (name, body) in files {
        hasher.update(name.as_bytes());
        hasher.update(b"\x01");
        hasher.update(body.as_bytes());
        hasher.update(b"\x02");
    }
    let digest = hasher.finalize();
    digest.to_hex().as_str()[..32].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_hash_is_deterministic_and_short() {
        let p = Path::new("/home/a/.claude/skills/my-skill");
        let h1 = path_hash(p);
        let h2 = path_hash(p);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 24);
    }

    #[test]
    fn path_hash_is_sensitive_to_input() {
        let h1 = path_hash(Path::new("/a/b"));
        let h2 = path_hash(Path::new("/a/c"));
        assert_ne!(h1, h2);
    }

    #[test]
    fn content_hash_is_deterministic_across_btreemap_iterations() {
        let mut files = BTreeMap::new();
        files.insert("z.txt".to_string(), "zzz".to_string());
        files.insert("a.txt".to_string(), "aaa".to_string());
        let h1 = content_hash("body", &files);
        let h2 = content_hash("body", &files);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 32);
    }

    #[test]
    fn content_hash_sentinel_prevents_concat_ambiguity() {
        // 若无哨兵，("ab", "") 与 ("a", "b") 会冲突。blake3 + 哨兵消除此风险。
        let mut a = BTreeMap::new();
        a.insert("ab".to_string(), String::new());
        let mut b = BTreeMap::new();
        b.insert("a".to_string(), "b".to_string());
        assert_ne!(content_hash("", &a), content_hash("", &b));
    }
}
