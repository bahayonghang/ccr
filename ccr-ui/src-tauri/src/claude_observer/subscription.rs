//! 订阅模式 / 月费 user_settings 封装
//!
//! 三个键：
//!   - `claude.user_mode`             : `auto | api_key | subscription`
//!   - `claude.subscription_plan`     : `free_pro | max5x | max20x | team | enterprise | custom`
//!   - `claude.subscription_monthly_usd` : f64 字符串
//!
//! 数据校验保持宽松：mode / plan 用字符串 enum，越界值由前端展示层兜底；
//! monthly_usd 解析失败时回退到默认 200.0（max20x 套餐基准价）。

use rusqlite::Connection;

/// 订阅设置 DTO（同时对外暴露给 Tauri 命令）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, ts_rs::TS)]
#[ts(export, export_to = "../../src/types/generated/claude_observer/")]
pub struct SubscriptionDto {
    pub mode: String,
    pub plan: String,
    pub monthly_usd: f64,
}

impl Default for SubscriptionDto {
    fn default() -> Self {
        ccr_app::usage::observer::subscription::SubscriptionDto::default().into()
    }
}

pub fn get(conn: &Connection) -> SubscriptionDto {
    ccr_app::usage::observer::subscription::get(conn).into()
}
pub fn set(
    conn: &Connection,
    mode: &str,
    plan: &str,
    monthly_usd: f64,
) -> Result<SubscriptionDto, rusqlite::Error> {
    ccr_app::usage::observer::subscription::set(conn, mode, plan, monthly_usd).map(Into::into)
}
impl From<ccr_app::usage::observer::subscription::SubscriptionDto> for SubscriptionDto {
    fn from(value: ccr_app::usage::observer::subscription::SubscriptionDto) -> Self {
        Self {
            mode: value.mode,
            plan: value.plan,
            monthly_usd: value.monthly_usd,
        }
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_db::database::migrations::{run_initial_migration, run_migration_v14};

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_initial_migration(&conn).unwrap();
        run_migration_v14(&conn).unwrap();
        conn
    }

    #[test]
    fn defaults_on_empty_db() {
        let conn = fresh_conn();
        let s = get(&conn);
        assert_eq!(s.mode, "auto");
        assert_eq!(s.plan, "team");
        assert!((s.monthly_usd - 200.0).abs() < 1e-9);
    }

    #[test]
    fn set_then_get_roundtrip() {
        let conn = fresh_conn();
        set(&conn, "subscription", "max20x", 220.5).unwrap();
        let s = get(&conn);
        assert_eq!(s.mode, "subscription");
        assert_eq!(s.plan, "max20x");
        assert!((s.monthly_usd - 220.5).abs() < 1e-9);
    }

    #[test]
    fn negative_monthly_falls_back_to_default() {
        let conn = fresh_conn();
        let s = set(&conn, "subscription", "max20x", -10.0).unwrap();
        assert!((s.monthly_usd - 200.0).abs() < 1e-9);
    }
}
