// 💰 CCR 预算管理器
// 负责预算配置的管理和成本监控

use crate::managers::CostTracker;
use crate::models::budget::{
    BudgetConfig, BudgetLimits, BudgetPeriod, BudgetStatus, BudgetWarning, PeriodCosts,
};
use ccr_core::core::error::{CcrError, Result};
use chrono::{Datelike, Duration, Utc};
use std::fs;
use std::path::{Path, PathBuf};

/// 💰 预算管理器
pub struct BudgetManager {
    /// 📁 配置文件路径
    config_path: PathBuf,

    /// 💰 预算配置
    config: BudgetConfig,
}

impl BudgetManager {
    /// 创建新的预算管理器
    pub fn new(config_path: PathBuf) -> Result<Self> {
        let config = if config_path.exists() {
            Self::load_config(&config_path)?
        } else {
            BudgetConfig::default()
        };

        Ok(Self {
            config_path,
            config,
        })
    }

    /// 获取默认配置路径
    pub fn default_config_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".to_string()))?;
        Ok(home.join(".claude").join("budget.toml"))
    }

    /// 从默认路径创建预算管理器
    pub fn with_default() -> Result<Self> {
        let config_path = Self::default_config_path()?;
        Self::new(config_path)
    }

    /// 加载配置文件
    fn load_config(path: &Path) -> Result<BudgetConfig> {
        let content = fs::read_to_string(path)?;
        let config: BudgetConfig = toml::from_str(&content)
            .map_err(|e| CcrError::ConfigError(format!("解析预算配置失败: {}", e)))?;

        config.validate().map_err(CcrError::ValidationError)?;
        Ok(config)
    }

    /// 保存配置文件
    fn save_config(&self) -> Result<()> {
        self.config.validate().map_err(CcrError::ValidationError)?;

        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| CcrError::ConfigError(format!("序列化预算配置失败: {}", e)))?;

        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.config_path, content)?;
        Ok(())
    }

    /// 获取当前配置
    #[allow(dead_code)]
    pub fn get_config(&self) -> &BudgetConfig {
        &self.config
    }

    /// 启用预算控制
    pub fn enable(&mut self) -> Result<()> {
        self.config.enabled = true;
        self.save_config()
    }

    /// 禁用预算控制
    pub fn disable(&mut self) -> Result<()> {
        self.config.enabled = false;
        self.save_config()
    }

    /// 设置每日预算限制
    pub fn set_daily_limit(&mut self, limit: Option<f64>) -> Result<()> {
        if let Some(val) = limit
            && val < 0.0
        {
            return Err(CcrError::ValidationError("预算限制不能为负数".to_string()));
        }
        self.config.daily_limit = limit;
        self.save_config()
    }

    /// 设置每周预算限制
    pub fn set_weekly_limit(&mut self, limit: Option<f64>) -> Result<()> {
        if let Some(val) = limit
            && val < 0.0
        {
            return Err(CcrError::ValidationError("预算限制不能为负数".to_string()));
        }
        self.config.weekly_limit = limit;
        self.save_config()
    }

    /// 设置每月预算限制
    pub fn set_monthly_limit(&mut self, limit: Option<f64>) -> Result<()> {
        if let Some(val) = limit
            && val < 0.0
        {
            return Err(CcrError::ValidationError("预算限制不能为负数".to_string()));
        }
        self.config.monthly_limit = limit;
        self.save_config()
    }

    /// 设置警告阈值
    pub fn set_warn_threshold(&mut self, percent: u8) -> Result<()> {
        if percent > 100 {
            return Err(CcrError::ValidationError(
                "警告阈值不能超过 100%".to_string(),
            ));
        }
        self.config.warn_at_percent = percent;
        self.save_config()
    }

    /// 重置所有预算限制
    pub fn reset_limits(&mut self) -> Result<()> {
        self.config.daily_limit = None;
        self.config.weekly_limit = None;
        self.config.monthly_limit = None;
        self.save_config()
    }

    /// 检查预算状态
    pub fn check_status(&self, tracker: &CostTracker) -> Result<BudgetStatus> {
        // 如果未启用，返回空状态
        if !self.config.enabled {
            return Ok(BudgetStatus {
                enabled: false,
                current_costs: PeriodCosts {
                    today: 0.0,
                    this_week: 0.0,
                    this_month: 0.0,
                },
                limits: BudgetLimits {
                    daily: self.config.daily_limit,
                    weekly: self.config.weekly_limit,
                    monthly: self.config.monthly_limit,
                },
                warnings: Vec::new(),
                last_updated: Utc::now(),
            });
        }

        // 计算当前周期成本
        let current_costs = self.calculate_period_costs(tracker)?;

        // 检查预算限制并生成警告
        let warnings = self.check_limits(&current_costs);

        Ok(BudgetStatus {
            enabled: true,
            current_costs,
            limits: BudgetLimits {
                daily: self.config.daily_limit,
                weekly: self.config.weekly_limit,
                monthly: self.config.monthly_limit,
            },
            warnings,
            last_updated: Utc::now(),
        })
    }

    /// 计算当前周期成本
    fn calculate_period_costs(&self, tracker: &CostTracker) -> Result<PeriodCosts> {
        let now = Utc::now();

        // 今日成本
        let today_start = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| CcrError::ConfigError("无效的日期时间".into()))?
            .and_utc();
        let today_stats = tracker.generate_stats(today_start, now)?;

        // 本周成本
        let week_start = now - Duration::days(7);
        let week_stats = tracker.generate_stats(week_start, now)?;

        // 本月成本
        let month_start = now
            .date_naive()
            .with_day(1)
            .ok_or_else(|| CcrError::ConfigError("无法设置日期为每月第一天".into()))?
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| CcrError::ConfigError("无效的日期时间".into()))?
            .and_utc();
        let month_stats = tracker.generate_stats(month_start, now)?;

        Ok(PeriodCosts {
            today: today_stats.total_cost,
            this_week: week_stats.total_cost,
            this_month: month_stats.total_cost,
        })
    }

    /// 检查预算限制并生成警告
    fn check_limits(&self, costs: &PeriodCosts) -> Vec<BudgetWarning> {
        let mut warnings = Vec::new();

        // 检查每日限制
        if let Some(daily_limit) = self.config.daily_limit {
            let usage_percent = (costs.today / daily_limit) * 100.0;
            if usage_percent >= self.config.warn_at_percent as f64 {
                warnings.push(BudgetWarning {
                    period: BudgetPeriod::Daily,
                    current_cost: costs.today,
                    limit: daily_limit,
                    usage_percent,
                    message: self.generate_warning_message(
                        BudgetPeriod::Daily,
                        usage_percent,
                        costs.today,
                        daily_limit,
                    ),
                });
            }
        }

        // 检查每周限制
        if let Some(weekly_limit) = self.config.weekly_limit {
            let usage_percent = (costs.this_week / weekly_limit) * 100.0;
            if usage_percent >= self.config.warn_at_percent as f64 {
                warnings.push(BudgetWarning {
                    period: BudgetPeriod::Weekly,
                    current_cost: costs.this_week,
                    limit: weekly_limit,
                    usage_percent,
                    message: self.generate_warning_message(
                        BudgetPeriod::Weekly,
                        usage_percent,
                        costs.this_week,
                        weekly_limit,
                    ),
                });
            }
        }

        // 检查每月限制
        if let Some(monthly_limit) = self.config.monthly_limit {
            let usage_percent = (costs.this_month / monthly_limit) * 100.0;
            if usage_percent >= self.config.warn_at_percent as f64 {
                warnings.push(BudgetWarning {
                    period: BudgetPeriod::Monthly,
                    current_cost: costs.this_month,
                    limit: monthly_limit,
                    usage_percent,
                    message: self.generate_warning_message(
                        BudgetPeriod::Monthly,
                        usage_percent,
                        costs.this_month,
                        monthly_limit,
                    ),
                });
            }
        }

        warnings
    }

    /// 生成警告消息
    fn generate_warning_message(
        &self,
        period: BudgetPeriod,
        usage_percent: f64,
        current: f64,
        limit: f64,
    ) -> String {
        if usage_percent >= 100.0 {
            format!(
                "⚠️ {} 预算已超出限制！当前: ${:.2}, 限制: ${:.2} ({:.1}%)",
                period, current, limit, usage_percent
            )
        } else {
            format!(
                "⚠️ {} 预算使用已达 {:.1}%！当前: ${:.2}, 限制: ${:.2}",
                period, usage_percent, current, limit
            )
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_budget_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("budget.toml");

        let manager = BudgetManager::new(config_path).unwrap();
        assert!(!manager.get_config().enabled);
    }

    #[test]
    fn test_enable_disable() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("budget.toml");

        let mut manager = BudgetManager::new(config_path).unwrap();

        manager.enable().unwrap();
        assert!(manager.get_config().enabled);

        manager.disable().unwrap();
        assert!(!manager.get_config().enabled);
    }

    #[test]
    fn test_set_limits() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("budget.toml");

        let mut manager = BudgetManager::new(config_path).unwrap();

        manager.set_daily_limit(Some(10.0)).unwrap();
        assert_eq!(manager.get_config().daily_limit, Some(10.0));

        manager.set_weekly_limit(Some(50.0)).unwrap();
        assert_eq!(manager.get_config().weekly_limit, Some(50.0));

        manager.set_monthly_limit(Some(200.0)).unwrap();
        assert_eq!(manager.get_config().monthly_limit, Some(200.0));

        // 测试负数验证
        assert!(manager.set_daily_limit(Some(-10.0)).is_err());
    }

    #[test]
    fn test_warn_threshold() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("budget.toml");

        let mut manager = BudgetManager::new(config_path).unwrap();

        manager.set_warn_threshold(90).unwrap();
        assert_eq!(manager.get_config().warn_at_percent, 90);

        // 测试无效阈值
        assert!(manager.set_warn_threshold(150).is_err());
    }

    #[test]
    fn test_reset_limits() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("budget.toml");

        let mut manager = BudgetManager::new(config_path).unwrap();

        manager.set_daily_limit(Some(10.0)).unwrap();
        manager.set_weekly_limit(Some(50.0)).unwrap();
        manager.set_monthly_limit(Some(200.0)).unwrap();

        manager.reset_limits().unwrap();

        assert!(manager.get_config().daily_limit.is_none());
        assert!(manager.get_config().weekly_limit.is_none());
        assert!(manager.get_config().monthly_limit.is_none());
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("budget.toml");

        {
            let mut manager = BudgetManager::new(config_path.clone()).unwrap();
            manager.enable().unwrap();
            manager.set_daily_limit(Some(10.0)).unwrap();
        }

        // 重新加载配置
        let manager = BudgetManager::new(config_path).unwrap();
        assert!(manager.get_config().enabled);
        assert_eq!(manager.get_config().daily_limit, Some(10.0));
    }

    #[test]
    fn test_warning_message_generation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("budget.toml");

        let manager = BudgetManager::new(config_path).unwrap();

        let msg = manager.generate_warning_message(BudgetPeriod::Daily, 85.0, 8.5, 10.0);
        assert!(msg.contains("85.0%"));
        assert!(msg.contains("$8.50"));

        let msg = manager.generate_warning_message(BudgetPeriod::Monthly, 105.0, 210.0, 200.0);
        assert!(msg.contains("超出限制"));
    }
}
