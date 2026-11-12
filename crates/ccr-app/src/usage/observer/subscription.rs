//! 订阅模式 / 月费 user_settings 封装
//!
//! 三个键：
//!   - `claude.user_mode`             : `auto | api_key | subscription`
//!   - `claude.subscription_plan`     : `free_pro | max5x | max20x | team | enterprise | custom`
//!   - `claude.subscription_monthly_usd` : f64 字符串
//!
//! 数据校验保持宽松：mode / plan 用字符串 enum，越界值由前端展示层兜底；
//! monthly_usd 解析失败时回退到默认 200.0（max20x 套餐基准价）。

use ccr_db::database::repositories::user_settings_repo;
use rusqlite::Connection;
use tracing::{debug, info, warn};

const KEY_MODE: &str = "claude.user_mode";
const KEY_PLAN: &str = "claude.subscription_plan";
const KEY_MONTHLY: &str = "claude.subscription_monthly_usd";

const DEFAULT_MODE: &str = "auto";
const DEFAULT_PLAN: &str = "team";
const DEFAULT_MONTHLY: f64 = 200.0;

/// 订阅设置 DTO（同时对外暴露给 Tauri 命令）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct SubscriptionDto {
    pub mode: String,
    pub plan: String,
    pub monthly_usd: f64,
}

impl Default for SubscriptionDto {
    fn default() -> Self {
        Self {
            mode: DEFAULT_MODE.to_string(),
            plan: DEFAULT_PLAN.to_string(),
            monthly_usd: DEFAULT_MONTHLY,
        }
    }
}

/// Missing keys use the domain defaults; unreadable storage remains an error.
pub fn read(conn: &Connection) -> Result<SubscriptionDto, rusqlite::Error> {
    Ok(SubscriptionDto {
        mode: user_settings_repo::get(conn, KEY_MODE)?.unwrap_or_else(|| DEFAULT_MODE.into()),
        plan: user_settings_repo::get(conn, KEY_PLAN)?.unwrap_or_else(|| DEFAULT_PLAN.into()),
        monthly_usd: user_settings_repo::get(conn, KEY_MONTHLY)?
            .and_then(|raw| raw.parse::<f64>().ok())
            .filter(|v| v.is_finite() && *v >= 0.0)
            .unwrap_or(DEFAULT_MONTHLY),
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum SaveOutcome {
    Saved(SubscriptionDto),
    Conflict,
}

/// Missing keys are distinct from explicitly saved defaults; storage errors propagate.
pub fn read_saved(conn: &Connection) -> Result<Option<SubscriptionDto>, rusqlite::Error> {
    let tx = conn.unchecked_transaction()?;
    let value = read_saved_keys(&tx)?;
    tx.commit()?;
    Ok(value)
}

fn read_saved_keys(conn: &Connection) -> Result<Option<SubscriptionDto>, rusqlite::Error> {
    let values = [
        user_settings_repo::get(conn, KEY_MODE)?,
        user_settings_repo::get(conn, KEY_PLAN)?,
        user_settings_repo::get(conn, KEY_MONTHLY)?,
    ];
    if values.iter().all(Option::is_none) {
        return Ok(None);
    }
    if values.iter().any(Option::is_none) {
        return Err(rusqlite::Error::InvalidQuery);
    }
    if !values[2]
        .as_deref()
        .and_then(|raw| raw.parse::<f64>().ok())
        .is_some_and(|value| value.is_finite() && value >= 0.)
    {
        return Err(rusqlite::Error::InvalidQuery);
    }
    Ok(Some(read(conn)?))
}

/// Atomically compare saved presence and values before updating all subscription keys.
pub fn save_saved_if_current(
    conn: &Connection,
    expected: Option<&SubscriptionDto>,
    proposed: &SubscriptionDto,
) -> Result<SaveOutcome, rusqlite::Error> {
    let tx = rusqlite::Transaction::new_unchecked(conn, rusqlite::TransactionBehavior::Immediate)?;
    let current = read_saved_keys(&tx)?;
    if current.as_ref() != expected {
        return Ok(SaveOutcome::Conflict);
    }
    let saved = set(&tx, &proposed.mode, &proposed.plan, proposed.monthly_usd)?;
    tx.commit()?;
    Ok(SaveOutcome::Saved(saved))
}

/// Compare and update all three keys under the same writer transaction.
pub fn save_if_current(
    conn: &Connection,
    expected: &SubscriptionDto,
    proposed: &SubscriptionDto,
) -> Result<SaveOutcome, rusqlite::Error> {
    let tx = rusqlite::Transaction::new_unchecked(conn, rusqlite::TransactionBehavior::Immediate)?;
    if &read(&tx)? != expected {
        return Ok(SaveOutcome::Conflict);
    }
    let saved = set(&tx, &proposed.mode, &proposed.plan, proposed.monthly_usd)?;
    tx.commit()?;
    Ok(SaveOutcome::Saved(saved))
}

/// 读取订阅设置。缺键 / 解析失败时返回默认值，永不报错。
pub fn get(conn: &Connection) -> SubscriptionDto {
    /* ====================================================================
     * 步骤1：逐键读取 user_settings，失败回落默认
     * ====================================================================
     */
    let mode = user_settings_repo::get(conn, KEY_MODE)
        .ok()
        .flatten()
        .unwrap_or_else(|| DEFAULT_MODE.to_string());
    let plan = user_settings_repo::get(conn, KEY_PLAN)
        .ok()
        .flatten()
        .unwrap_or_else(|| DEFAULT_PLAN.to_string());
    let monthly_usd = user_settings_repo::get(conn, KEY_MONTHLY)
        .ok()
        .flatten()
        .and_then(|raw| raw.parse::<f64>().ok())
        .unwrap_or(DEFAULT_MONTHLY);

    debug!(
        "[claude_observer] subscription get: mode={} plan={} monthly_usd={}",
        mode, plan, monthly_usd
    );
    SubscriptionDto {
        mode,
        plan,
        monthly_usd,
    }
}

/// 写入订阅设置，并返回写入后的快照。
///
/// `mode` / `plan` 不在已知枚举集合时会写入但打一条 warn——保持向前兼容。
pub fn set(
    conn: &Connection,
    mode: &str,
    plan: &str,
    monthly_usd: f64,
) -> Result<SubscriptionDto, rusqlite::Error> {
    /* ====================================================================
     * 步骤1：宽松校验（仅 warn，不阻断）
     * ====================================================================
     */
    if !matches!(mode, "auto" | "api_key" | "subscription") {
        warn!("[claude_observer] subscription set: 未知 mode={mode}");
    }
    if !matches!(
        plan,
        "free_pro" | "max5x" | "max20x" | "team" | "enterprise" | "custom"
    ) {
        warn!("[claude_observer] subscription set: 未知 plan={plan}");
    }
    let monthly_usd = if monthly_usd.is_finite() && monthly_usd >= 0.0 {
        monthly_usd
    } else {
        warn!(
            "[claude_observer] subscription set: 非法 monthly_usd={monthly_usd}, fallback {DEFAULT_MONTHLY}"
        );
        DEFAULT_MONTHLY
    };

    /* ====================================================================
     * 步骤2：写入 user_settings 三键
     * ====================================================================
     */
    user_settings_repo::set(conn, KEY_MODE, mode)?;
    user_settings_repo::set(conn, KEY_PLAN, plan)?;
    user_settings_repo::set(conn, KEY_MONTHLY, &format!("{monthly_usd}"))?;

    info!(
        "[claude_observer] subscription set: mode={} plan={} monthly_usd={}",
        mode, plan, monthly_usd
    );

    Ok(SubscriptionDto {
        mode: mode.to_string(),
        plan: plan.to_string(),
        monthly_usd,
    })
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
    fn saved_presence_distinguishes_defaults_and_preserves_cas() {
        let conn = fresh_conn();
        let defaults = SubscriptionDto::default();
        assert_eq!(read_saved(&conn).unwrap(), None);
        assert_eq!(
            save_saved_if_current(&conn, None, &defaults).unwrap(),
            SaveOutcome::Saved(defaults.clone())
        );
        assert_eq!(read_saved(&conn).unwrap(), Some(defaults.clone()));
        assert_eq!(
            save_saved_if_current(&conn, None, &defaults).unwrap(),
            SaveOutcome::Conflict
        );
        let mut changed = defaults.clone();
        changed.monthly_usd = 99.0;
        assert_eq!(
            save_saved_if_current(&conn, Some(&defaults), &changed).unwrap(),
            SaveOutcome::Saved(changed.clone())
        );
        assert_eq!(
            save_saved_if_current(&conn, Some(&defaults), &defaults).unwrap(),
            SaveOutcome::Conflict
        );
        assert_eq!(read_saved(&conn).unwrap(), Some(changed));
        let broken = Connection::open_in_memory().unwrap();
        assert!(read_saved(&broken).is_err());
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

    #[test]
    fn guarded_update_rolls_back_all_keys_and_rejects_stale_snapshots() {
        let conn = fresh_conn();
        let expected = set(&conn, "auto", "team", 200.0).unwrap();
        let proposed = SubscriptionDto {
            mode: "subscription".into(),
            plan: "custom".into(),
            monthly_usd: 17.5,
        };
        conn.execute_batch("CREATE TRIGGER fail_subscription_plan BEFORE UPDATE ON user_settings WHEN NEW.key = 'claude.subscription_plan' BEGIN SELECT RAISE(ABORT, 'injected'); END;").unwrap();
        assert!(save_if_current(&conn, &expected, &proposed).is_err());
        assert_eq!(read(&conn).unwrap(), expected);
        conn.execute_batch("DROP TRIGGER fail_subscription_plan")
            .unwrap();
        assert_eq!(
            save_if_current(&conn, &expected, &proposed).unwrap(),
            SaveOutcome::Saved(proposed.clone())
        );
        assert_eq!(
            save_if_current(&conn, &expected, &SubscriptionDto::default()).unwrap(),
            SaveOutcome::Conflict
        );
        assert_eq!(read(&conn).unwrap(), proposed);
    }
}
