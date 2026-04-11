// 💰 CCR 预算配置模型
// 定义预算控制相关的数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 💰 预算配置
///
/// 定义用户的预算限制和警告阈值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// ✅ 是否启用预算控制
    #[serde(default)]
    pub enabled: bool,

    /// 📅 每日预算限制（美元，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_limit: Option<f64>,

    /// 📅 每周预算限制（美元，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekly_limit: Option<f64>,

    /// 📅 每月预算限制（美元，可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_limit: Option<f64>,

    /// ⚠️ 警告阈值百分比（0-100）
    #[serde(default = "default_warn_threshold")]
    pub warn_at_percent: u8,

    /// 🔔 超出限制时的动作
    #[serde(default)]
    pub on_limit_exceeded: LimitAction,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            daily_limit: None,
            weekly_limit: None,
            monthly_limit: None,
            warn_at_percent: 80,
            on_limit_exceeded: LimitAction::Warn,
        }
    }
}

fn default_warn_threshold() -> u8 {
    80
}

/// 🔔 超出限制时的动作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LimitAction {
    /// 仅警告
    #[default]
    Warn,
    /// 记录日志
    Log,
    /// 不做任何操作
    None,
}

impl std::fmt::Display for LimitAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LimitAction::Warn => write!(f, "warn"),
            LimitAction::Log => write!(f, "log"),
            LimitAction::None => write!(f, "none"),
        }
    }
}

/// 📊 预算状态
///
/// 显示当前预算使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    /// ✅ 是否启用预算控制
    pub enabled: bool,

    /// 💰 当前周期成本
    pub current_costs: PeriodCosts,

    /// 📊 预算限制
    pub limits: BudgetLimits,

    /// ⚠️ 警告状态
    pub warnings: Vec<BudgetWarning>,

    /// 📅 最后更新时间
    pub last_updated: DateTime<Utc>,
}

/// 💰 周期成本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodCosts {
    /// 📅 今日成本
    pub today: f64,

    /// 📅 本周成本
    pub this_week: f64,

    /// 📅 本月成本
    pub this_month: f64,
}

/// 📊 预算限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLimits {
    /// 📅 每日限制
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily: Option<f64>,

    /// 📅 每周限制
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekly: Option<f64>,

    /// 📅 每月限制
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly: Option<f64>,
}

/// ⚠️ 预算警告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetWarning {
    /// 📅 周期类型
    pub period: BudgetPeriod,

    /// 💰 当前成本
    pub current_cost: f64,

    /// 📊 限制金额
    pub limit: f64,

    /// 📈 使用百分比
    pub usage_percent: f64,

    /// 📝 警告消息
    pub message: String,
}

/// 📅 预算周期类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BudgetPeriod {
    /// 每日
    Daily,
    /// 每周
    Weekly,
    /// 每月
    Monthly,
}

impl std::fmt::Display for BudgetPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BudgetPeriod::Daily => write!(f, "每日"),
            BudgetPeriod::Weekly => write!(f, "每周"),
            BudgetPeriod::Monthly => write!(f, "每月"),
        }
    }
}

impl BudgetConfig {
    /// 检查配置是否有效
    pub fn validate(&self) -> Result<(), String> {
        if self.warn_at_percent > 100 {
            return Err("警告阈值不能超过 100%".to_string());
        }

        if let Some(daily) = self.daily_limit
            && daily < 0.0
        {
            return Err("每日预算限制不能为负数".to_string());
        }

        if let Some(weekly) = self.weekly_limit
            && weekly < 0.0
        {
            return Err("每周预算限制不能为负数".to_string());
        }

        if let Some(monthly) = self.monthly_limit
            && monthly < 0.0
        {
            return Err("每月预算限制不能为负数".to_string());
        }

        Ok(())
    }

    /// 是否有任何预算限制
    #[allow(dead_code)]
    pub fn has_any_limit(&self) -> bool {
        self.daily_limit.is_some() || self.weekly_limit.is_some() || self.monthly_limit.is_some()
    }
}

#[allow(dead_code)]
impl BudgetStatus {
    /// 检查是否接近预算限制
    #[allow(dead_code)]
    pub fn is_near_limit(&self, threshold_percent: u8) -> bool {
        self.warnings
            .iter()
            .any(|w| w.usage_percent >= threshold_percent as f64)
    }

    /// 获取最严重的警告
    pub fn worst_warning(&self) -> Option<&BudgetWarning> {
        self.warnings.iter().max_by(|a, b| {
            a.usage_percent
                .partial_cmp(&b.usage_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_config_default() {
        let config = BudgetConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.warn_at_percent, 80);
        assert_eq!(config.on_limit_exceeded, LimitAction::Warn);
    }

    #[test]
    fn test_budget_config_validation() {
        let mut config = BudgetConfig::default();
        assert!(config.validate().is_ok());

        config.warn_at_percent = 150;
        assert!(config.validate().is_err());

        config.warn_at_percent = 80;
        config.daily_limit = Some(-10.0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_budget_config_has_any_limit() {
        let mut config = BudgetConfig::default();
        assert!(!config.has_any_limit());

        config.daily_limit = Some(10.0);
        assert!(config.has_any_limit());
    }

    #[test]
    fn test_limit_action_display() {
        assert_eq!(LimitAction::Warn.to_string(), "warn");
        assert_eq!(LimitAction::Log.to_string(), "log");
        assert_eq!(LimitAction::None.to_string(), "none");
    }

    #[test]
    fn test_budget_period_display() {
        assert_eq!(BudgetPeriod::Daily.to_string(), "每日");
        assert_eq!(BudgetPeriod::Weekly.to_string(), "每周");
        assert_eq!(BudgetPeriod::Monthly.to_string(), "每月");
    }
}
