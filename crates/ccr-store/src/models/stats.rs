// 📊 CCR 统计数据模型
// 定义成本、使用统计相关的数据结构

use ccr_types::official_model_rate_overrides;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 成本追踪相关模型
// ============================================================

/// 💰 成本记录
///
/// 记录单次 API 调用的成本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecord {
    /// 🆔 记录唯一标识
    pub id: String,

    /// ⏰ 时间戳
    pub timestamp: DateTime<Utc>,

    /// 📝 会话 ID（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// 📁 项目路径
    pub project: String,

    /// 🤖 使用的模型
    pub model: String,

    /// 🔢 Token 使用情况
    pub token_usage: TokenUsage,

    /// 💵 成本信息
    pub cost: Cost,

    /// ⏱️ 请求时长（毫秒）
    pub duration_ms: u64,

    /// 🏷️ 平台（Claude/Codex/Gemini等）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,

    /// 📝 描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 🎫 Token 使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// 📥 输入 Token 数
    pub input_tokens: u32,

    /// 📤 输出 Token 数
    pub output_tokens: u32,

    /// 💾 Cache 创建 Token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_tokens: Option<u32>,

    /// 📖 Cache 读取 Token 数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_tokens: Option<u32>,
}

/// 💵 成本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cost {
    /// 📥 输入成本
    pub input_cost: f64,

    /// 📤 输出成本
    pub output_cost: f64,

    /// 💾 Cache 成本
    pub cache_cost: f64,

    /// 💰 总成本
    pub total_cost: f64,
}

/// 💲 模型定价
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// 🤖 模型名称
    pub model: String,

    /// 📥 输入价格（美元/百万 Token）
    pub input_price: f64,

    /// 📤 输出价格（美元/百万 Token）
    pub output_price: f64,

    /// 📖 Cache 读取价格（美元/百万 Token）
    pub cache_read_price: Option<f64>,

    /// 💾 Cache 写入价格（美元/百万 Token）
    pub cache_write_price: Option<f64>,
}

impl ModelPricing {
    /// 获取默认的模型定价表
    pub fn default_pricing() -> HashMap<String, ModelPricing> {
        let official: HashMap<String, ModelPricing> = official_model_rate_overrides()
            .into_iter()
            .map(|item| {
                let model = item.model;
                (
                    model.clone(),
                    ModelPricing {
                        model,
                        input_price: item.input_price,
                        output_price: item.output_price,
                        cache_read_price: item.cache_read_price,
                        cache_write_price: item.cache_write_price,
                    },
                )
            })
            .collect();
        if !official.is_empty() {
            return official;
        }

        let mut pricing = HashMap::new();

        // Claude 3.5 Sonnet
        pricing.insert(
            "claude-3-5-sonnet-20241022".to_string(),
            ModelPricing {
                model: "claude-3-5-sonnet-20241022".to_string(),
                input_price: 3.0,
                output_price: 15.0,
                cache_read_price: Some(0.3),
                cache_write_price: Some(3.75),
            },
        );

        // Claude 3.5 Haiku
        pricing.insert(
            "claude-3-5-haiku-20241022".to_string(),
            ModelPricing {
                model: "claude-3-5-haiku-20241022".to_string(),
                input_price: 1.0,
                output_price: 5.0,
                cache_read_price: Some(0.1),
                cache_write_price: Some(1.25),
            },
        );

        // Claude 3 Opus
        pricing.insert(
            "claude-3-opus-20240229".to_string(),
            ModelPricing {
                model: "claude-3-opus-20240229".to_string(),
                input_price: 15.0,
                output_price: 75.0,
                cache_read_price: Some(1.5),
                cache_write_price: Some(18.75),
            },
        );

        // Claude 4.5 Sonnet (假设定价)
        pricing.insert(
            "claude-sonnet-4-5-20250929".to_string(),
            ModelPricing {
                model: "claude-sonnet-4-5-20250929".to_string(),
                input_price: 3.0,
                output_price: 15.0,
                cache_read_price: Some(0.3),
                cache_write_price: Some(3.75),
            },
        );

        // Claude 4.1 Opus (假设定价)
        pricing.insert(
            "claude-opus-4-1-20250924".to_string(),
            ModelPricing {
                model: "claude-opus-4-1-20250924".to_string(),
                input_price: 15.0,
                output_price: 75.0,
                cache_read_price: Some(1.5),
                cache_write_price: Some(18.75),
            },
        );

        pricing
    }

    /// 计算成本
    pub fn calculate_cost(&self, usage: &TokenUsage) -> Cost {
        let input_cost = (usage.input_tokens as f64) * self.input_price / 1_000_000.0;
        let output_cost = (usage.output_tokens as f64) * self.output_price / 1_000_000.0;

        let mut cache_cost = 0.0;
        if let Some(cache_write_tokens) = usage.cache_creation_tokens
            && let Some(cache_write_price) = self.cache_write_price
        {
            cache_cost += (cache_write_tokens as f64) * cache_write_price / 1_000_000.0;
        }
        if let Some(cache_read_tokens) = usage.cache_read_tokens
            && let Some(cache_read_price) = self.cache_read_price
        {
            cache_cost += (cache_read_tokens as f64) * cache_read_price / 1_000_000.0;
        }

        let total_cost = input_cost + output_cost + cache_cost;

        Cost {
            input_cost,
            output_cost,
            cache_cost,
            total_cost,
        }
    }
}

// ============================================================
// 统计汇总模型
// ============================================================

/// 📊 成本统计汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostStats {
    /// 💰 总成本
    pub total_cost: f64,

    /// 🔢 记录数量
    pub record_count: usize,

    /// 📊 Token 统计
    pub token_stats: TokenStats,

    /// 🏢 按提供商/平台分组（使用次数）
    #[serde(default)]
    pub by_provider: HashMap<String, u64>,

    /// 🤖 按模型分组
    pub by_model: HashMap<String, f64>,

    /// 📁 按项目分组
    pub by_project: HashMap<String, f64>,

    /// 📈 趋势数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trend: Option<Vec<DailyCost>>,
}

/// 🔢 Token 使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStats {
    /// 📥 总输入 Token
    pub total_input_tokens: u64,

    /// 📤 总输出 Token
    pub total_output_tokens: u64,

    /// 💾 总 Cache Token
    pub total_cache_tokens: u64,

    /// 📊 Cache 效率（命中率）
    pub cache_efficiency: f64,
}

/// 📅 每日成本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyCost {
    /// 📅 日期
    pub date: String,

    /// 💰 成本
    pub cost: f64,

    /// 🔢 记录数
    pub count: usize,
}

// ============================================================
// 时间范围枚举
// ============================================================

/// 📅 时间范围
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum TimeRange {
    /// 今日
    Today,

    /// 本周
    Week,

    /// 本月
    Month,

    /// 自定义
    Custom,
}

impl TimeRange {
    /// 获取显示名称
    #[allow(dead_code)]
    pub fn display_name(&self) -> &str {
        match self {
            TimeRange::Today => "今日",
            TimeRange::Week => "本周",
            TimeRange::Month => "本月",
            TimeRange::Custom => "自定义",
        }
    }
}
