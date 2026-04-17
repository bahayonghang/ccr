//! 内建同义词索引（中英混合），一次性初始化的全局 `OnceLock`。
//!
//! 简化规则：映射到 canonical token（已小写）；分类器 tokenize 后查表替换。
//! 例如 `"js"` / `"javascript"` 都折叠为 `"javascript"`。
//!
//! 如需扩展，在 [`BUILTIN_PAIRS`] 追加 `(variant, canonical)` 即可。

use std::collections::HashMap;
use std::sync::OnceLock;

/// `(variant, canonical)` 对。variant 必须小写。
const BUILTIN_PAIRS: &[(&str, &str)] = &[
    // 编程语言别名
    ("js", "javascript"),
    ("ts", "typescript"),
    ("py", "python"),
    ("rb", "ruby"),
    ("go", "golang"),
    // 中英互通（中文 → 对应英文关键词）
    ("视频", "video"),
    ("音频", "audio"),
    ("字幕", "subtitle"),
    ("封面", "cover"),
    ("配图", "image"),
    ("插画", "illustration"),
    ("文章", "article"),
    ("写作", "write"),
    ("创作", "write"),
    ("翻译", "translate"),
    ("多语言", "i18n"),
    ("本地化", "l10n"),
    ("数据", "data"),
    ("报告", "report"),
    ("报表", "report"),
    ("统计", "stats"),
    ("搜索", "search"),
    ("抓取", "scrape"),
    ("爬虫", "crawl"),
    ("小红书", "xhs"),
    ("微博", "weibo"),
    ("公众号", "wechat"),
    ("抖音", "tiktok"),
    ("飞书", "feishu"),
    ("邮件", "email"),
    ("钉钉", "dingtalk"),
    ("设计", "design"),
    ("运维", "devops"),
    ("部署", "deploy"),
    ("人格", "persona"),
    ("角色", "persona"),
    ("财务", "finance"),
    ("金融", "finance"),
    ("投资", "finance"),
    ("发票", "invoice"),
    ("报销", "expense"),
    ("文档", "document"),
    ("大纲", "outline"),
    ("稿件", "article"),
    ("选题", "topic"),
];

static INDEX: OnceLock<HashMap<String, String>> = OnceLock::new();

/// 获取全局 synonym 索引。首次调用时构建，之后 O(1)。
pub fn get_synonym_index() -> &'static HashMap<String, String> {
    INDEX.get_or_init(|| {
        let mut map = HashMap::with_capacity(BUILTIN_PAIRS.len());
        for (variant, canonical) in BUILTIN_PAIRS {
            map.insert(variant.to_string(), canonical.to_string());
        }
        map
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_index_returns_same_instance() {
        let a = get_synonym_index() as *const _;
        let b = get_synonym_index() as *const _;
        assert_eq!(a, b, "OnceLock 必须返回同一实例");
    }

    #[test]
    fn builtin_pairs_cover_common_zh_en() {
        let idx = get_synonym_index();
        assert_eq!(idx.get("小红书").map(|s| s.as_str()), Some("xhs"));
        assert_eq!(idx.get("js").map(|s| s.as_str()), Some("javascript"));
        assert_eq!(idx.get("翻译").map(|s| s.as_str()), Some("translate"));
    }
}
