// ─────────────────────────────────────────────────────────────────────
// 端口自 vibe-observer（MIT License）
//   原始路径: ref/repo/vibe-observer/crates/observer-core/src/pricing.rs
//   原始价目: ref/repo/vibe-observer/crates/observer-core/src/pricing.json
// 仅做最小化裁剪：本仓不持有 pricing_models 表，所以去掉 SQLite upsert 与
// `valid_from` 维度的多版本支持；当前用例只需要最长前缀匹配与版本字符串暴露。
// ─────────────────────────────────────────────────────────────────────

use std::sync::OnceLock;

use serde::Deserialize;

const EMBEDDED_PRICING: &str = include_str!("../../resources/claude_pricing.json");

/// 单条模型价目（每 1M token 美元）
///
/// 仅 `cost_usd` 路径使用。当前 InsightDto 走 llmusage 的 `cost_with_cache_usd`，
/// 这里的字段在「价目过期警告」/ 离线兜底重定价场景才被读取——保留 dead_code
/// allow 让编译期不报红，等后续 follow-up issue 启用时直接接入。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct ModelPrice {
    pub model_id: String,
    pub valid_from: String,
    pub input_per_mtok: f64,
    pub output_per_mtok: f64,
    pub cache_read_per_mtok: f64,
    pub cache_write_per_mtok: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct PricingFile {
    pub version: String,
    // 仅给未来 `cost_usd` 重定价兜底用，目前没有调用方读取这个字段
    #[allow(dead_code)]
    pub models: Vec<ModelPrice>,
}

/// 全局价目表缓存；首次访问时反序列化一次后驻留进程内存。
fn catalog() -> &'static PricingFile {
    static CATALOG: OnceLock<PricingFile> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(EMBEDDED_PRICING)
            .expect("claude_pricing.json must parse at build time")
    })
}

/// 按最长前缀匹配查找价目。
///
/// Claude Code 上报的 model_id 通常带日期后缀，例如
/// `claude-haiku-4-5-20251001`；价目表里只保留无后缀的家族键。
/// 这里复刻 vibe-observer 的策略：优先匹配最长 `model_id` 前缀。
///
/// 当前未被命令直接调用，仅供 `cost_usd` 与单测使用——保留 dead_code allow，
/// 等价目过期警告 follow-up 启用时直接接入。
#[allow(dead_code)]
fn lookup(model_id: &str) -> Option<&'static ModelPrice> {
    /* ====================================================================
     * 步骤1：候选筛选 + 长度排序
     * ====================================================================
     * 目标：在 catalog().models 中找出 model_id LIKE "{key}%" 的全部候选，
     *      然后取 key 字符长度最长的一个，以解决「claude-haiku-4-5」与
     *      「claude-haiku-4」可能同时存在时的歧义。
     */
    catalog()
        .models
        .iter()
        .filter(|m| model_id.starts_with(&m.model_id))
        .max_by_key(|m| m.model_id.len())
}

/// 计算单次事件的 USD 成本。
///
/// 入参单位为「实际 token 数」，内部按每百万 token 单价折算。
/// 未命中模型返回 `None`，调用方可视情况降级（例如显示 N/A 或回退到 list price）。
///
/// 当前 InsightDto 直接复用 llmusage 的 `cost_with_cache_usd`，所以这里
/// 暂时没有调用方；保留 API 与单测，留给后续「价目过期警告」/ 离线兜底重定价。
#[allow(dead_code)]
pub fn cost_usd(
    model_id: &str,
    input: i64,
    output: i64,
    cache_read: i64,
    cache_write: i64,
) -> Option<f64> {
    let price = lookup(model_id)?;
    let cost = (input as f64) * price.input_per_mtok / 1_000_000.0
        + (output as f64) * price.output_per_mtok / 1_000_000.0
        + (cache_read as f64) * price.cache_read_per_mtok / 1_000_000.0
        + (cache_write as f64) * price.cache_write_per_mtok / 1_000_000.0;
    Some(cost)
}

/// 返回嵌入式价目表的版本字符串（如 `"2026-05-15"`）。
/// 用于在 Insight DTO 中暴露给前端 transparency banner。
pub fn pricing_version() -> &'static str {
    catalog().version.as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_embedded_catalog() {
        assert!(!pricing_version().is_empty());
        assert!(
            catalog()
                .models
                .iter()
                .any(|m| m.model_id == "claude-opus-4-7")
        );
    }

    #[test]
    fn exact_match_cost() {
        // 1M input opus = 15.0 USD
        let cost = cost_usd("claude-opus-4-7", 1_000_000, 0, 0, 0).unwrap();
        assert!((cost - 15.0).abs() < 1e-9);
    }

    #[test]
    fn date_suffixed_resolves_via_prefix() {
        // claude-haiku-4-5-20251001 应命中 claude-haiku-4-5 = 0.8/Mtok input
        let cost = cost_usd("claude-haiku-4-5-20251001", 1_000_000, 0, 0, 0).unwrap();
        assert!((cost - 0.8).abs() < 1e-9, "got {cost}");
    }

    #[test]
    fn unknown_model_returns_none() {
        assert!(cost_usd("claude-imaginary-9000", 1_000_000, 0, 0, 0).is_none());
    }
}
