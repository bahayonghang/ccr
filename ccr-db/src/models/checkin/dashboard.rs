// 签到账号 Dashboard 数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 账号概览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinDashboardAccount {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub provider_name: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_checkin_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_balance_check_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_balance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance_currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_quota: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub used_quota: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining_quota: Option<f64>,
}

/// 连续签到统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinDashboardStreak {
    pub current_streak: u32,
    pub longest_streak: u32,
    pub total_check_in_days: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_check_in_date: Option<String>,
}

/// 月度统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinDashboardMonthStats {
    pub total_days: u32,
    pub checked_in_days: u32,
    pub check_in_rate: f64,
    pub total_quota_increment: f64,
}

/// 日历单日数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinDashboardDay {
    pub date: String,
    pub is_checked_in: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub income_increment: Option<f64>,
    pub current_balance: f64,
    pub total_consumed: f64,
    pub total_quota: f64,
}

/// 月度日历
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinDashboardCalendar {
    pub account_id: String,
    pub year: i32,
    pub month: u32,
    pub days: Vec<CheckinDashboardDay>,
    pub month_stats: CheckinDashboardMonthStats,
}

/// 趋势数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinDashboardTrendPoint {
    pub date: String,
    pub total_quota: f64,
    pub income_increment: f64,
    pub current_balance: f64,
    pub is_checked_in: bool,
}

/// 趋势数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinDashboardTrend {
    pub account_id: String,
    pub start_date: String,
    pub end_date: String,
    pub data_points: Vec<CheckinDashboardTrendPoint>,
}

/// Dashboard 聚合响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckinAccountDashboardResponse {
    pub account: CheckinDashboardAccount,
    pub streak: CheckinDashboardStreak,
    pub calendar: CheckinDashboardCalendar,
    pub trend: CheckinDashboardTrend,
}
