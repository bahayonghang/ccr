// 📊 CCR 统计数据模型
// 定义成本、使用统计相关的数据结构

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

impl TokenUsage {
    /// 计算总 Token 数
    #[allow(dead_code)] // 预留用于未来的统计功能
    pub fn total(&self) -> u32 {
        self.input_tokens
            + self.output_tokens
            + self.cache_creation_tokens.unwrap_or(0)
            + self.cache_read_tokens.unwrap_or(0)
    }
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
#[derive(Debug, Clone)]
#[allow(dead_code)] // 结构体字段在内部使用，外部可能未直接读取
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

#[allow(dead_code)]
impl ModelPricing {
    /// 获取默认的模型定价表
    pub fn default_pricing() -> HashMap<String, ModelPricing> {
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
    #[allow(dead_code)] // 预留方法，当前使用 CostTracker::calculate_cost
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
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostStats {
    /// 💰 总成本
    pub total_cost: f64,

    /// 🔢 记录数量
    pub record_count: usize,

    /// 📊 Token 统计
    pub token_stats: TokenStats,

    /// 🤖 按模型分组
    pub by_model: HashMap<String, f64>,

    /// 📁 按项目分组
    pub by_project: HashMap<String, f64>,

    /// 📈 趋势数据（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trend: Option<Vec<DailyCost>>,
}

/// 🔢 Token 使用统计
#[allow(dead_code)]
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
#[allow(dead_code)]
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
// 使用统计模型
// ============================================================

/// 📈 会话记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 预留用于会话级别的统计分析
pub struct SessionRecord {
    /// 🆔 会话 ID
    pub session_id: String,

    /// 📁 项目路径
    pub project: String,

    /// ⏰ 开始时间
    pub start_time: DateTime<Utc>,

    /// ⏱️ 结束时间（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<DateTime<Utc>>,

    /// 💬 消息数量
    pub message_count: usize,

    /// 🤖 使用的模型
    pub model: String,

    /// 💰 总成本
    pub total_cost: f64,

    /// 🏷️ 平台
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
}

/// 📊 使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 预留用于综合统计报告
pub struct UsageStats {
    /// 📝 总会话数
    pub total_sessions: usize,

    /// 💬 总消息数
    pub total_messages: usize,

    /// ⏱️ 平均会话时长（秒）
    pub avg_session_duration: f64,

    /// 📊 按项目分组
    pub by_project: HashMap<String, ProjectUsage>,

    /// 🤖 按模型分组
    pub by_model: HashMap<String, ModelUsage>,
}

/// 📁 项目使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 预留用于项目级别的统计
pub struct ProjectUsage {
    /// 📝 会话数
    pub sessions: usize,

    /// 💬 消息数
    pub messages: usize,

    /// 💰 成本
    pub cost: f64,
}

/// 🤖 模型使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 预留用于模型级别的统计
pub struct ModelUsage {
    /// 📊 请求次数
    pub requests: usize,

    /// 🎫 Token 数
    pub tokens: u64,

    /// 💰 成本
    pub cost: f64,

    /// 📊 占比
    pub percentage: f32,
}

// ============================================================
// 代码统计模型
// ============================================================

/// 📝 代码变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 预留用于代码变更分析功能
pub struct CodeChangeRecord {
    /// 🆔 记录 ID
    pub id: String,

    /// ⏰ 时间戳
    pub timestamp: DateTime<Utc>,

    /// 📝 会话 ID（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// 📁 项目路径
    pub project: String,

    /// 📊 文件统计
    pub file_stats: FileStats,

    /// 🔤 语言分布
    pub language_stats: HashMap<String, LanguageStats>,
}

/// 📊 文件统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 预留用于文件级别的统计
pub struct FileStats {
    /// ✨ 创建文件数
    pub files_created: usize,

    /// 📝 修改文件数
    pub files_modified: usize,

    /// 🗑️ 删除文件数
    pub files_deleted: usize,

    /// ➕ 新增代码行数
    pub lines_added: usize,

    /// ➖ 删除代码行数
    pub lines_deleted: usize,
}

/// 🔤 语言统计
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // 预留用于编程语言分析
pub struct LanguageStats {
    /// 📄 文件数
    pub files: usize,

    /// ➕ 新增行数
    pub lines_added: usize,

    /// ➖ 删除行数
    pub lines_deleted: usize,
}

// ============================================================
// 时间范围枚举
// ============================================================

/// 📅 时间范围
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
    #[allow(dead_code)] // 预留方法
    pub fn display_name(&self) -> &str {
        match self {
            TimeRange::Today => "今日",
            TimeRange::Week => "本周",
            TimeRange::Month => "本月",
            TimeRange::Custom => "自定义",
        }
    }
}
