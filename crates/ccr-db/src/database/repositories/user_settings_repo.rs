// 通用 key/value 用户设置仓储
// 用于承载 UI 偏好、订阅模式、月费等非敏感配置
// 设计原则：明文存储，键命名使用 namespace.key 形式（如 "claude.user_mode"）
//
// 注意：本仓储不做任何加密。如果将来要存敏感字段（API token 等），
// 应使用 checkin_repo 里的加密路径，而不是这里。

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, params};
use tracing::debug;

/// 设置项（用于批量读出）
#[derive(Debug, Clone)]
pub struct SettingEntry {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

/// 读取单个键
#[allow(dead_code)]
pub fn get(conn: &Connection, key: &str) -> Result<Option<String>, rusqlite::Error> {
    conn.query_row(
        "SELECT value FROM user_settings WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    )
    .optional()
}

/// upsert 单个键
#[allow(dead_code)]
pub fn set(conn: &Connection, key: &str, value: &str) -> Result<(), rusqlite::Error> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO user_settings (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value, now],
    )?;
    debug!("user_settings upsert: {} = {}", key, value);
    Ok(())
}

/// 删除单个键，返回是否真的删除了
#[allow(dead_code)]
pub fn delete(conn: &Connection, key: &str) -> Result<bool, rusqlite::Error> {
    let n = conn.execute("DELETE FROM user_settings WHERE key = ?1", params![key])?;
    Ok(n > 0)
}

/// 列出某个 namespace 下的所有键（前缀匹配，例如 "claude."）
#[allow(dead_code)]
pub fn list_by_prefix(
    conn: &Connection,
    prefix: &str,
) -> Result<Vec<SettingEntry>, rusqlite::Error> {
    let like_pattern = format!("{prefix}%");
    let mut stmt = conn.prepare(
        "SELECT key, value, updated_at FROM user_settings WHERE key LIKE ?1 ORDER BY key ASC",
    )?;
    let rows = stmt.query_map(params![like_pattern], |row| {
        Ok(SettingEntry {
            key: row.get(0)?,
            value: row.get(1)?,
            updated_at: row.get(2)?,
        })
    })?;
    rows.collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::database::migrations::{run_initial_migration, run_migration_v14};

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_initial_migration(&conn).unwrap();
        run_migration_v14(&conn).unwrap();
        conn
    }

    #[test]
    fn set_then_get_roundtrip() {
        let conn = fresh_conn();
        assert_eq!(get(&conn, "claude.user_mode").unwrap(), None);

        set(&conn, "claude.user_mode", "subscription").unwrap();
        assert_eq!(
            get(&conn, "claude.user_mode").unwrap().as_deref(),
            Some("subscription")
        );
    }

    #[test]
    fn set_overrides_existing_value() {
        let conn = fresh_conn();
        set(&conn, "claude.subscription_monthly_usd", "200").unwrap();
        set(&conn, "claude.subscription_monthly_usd", "100").unwrap();
        assert_eq!(
            get(&conn, "claude.subscription_monthly_usd")
                .unwrap()
                .as_deref(),
            Some("100")
        );
    }

    #[test]
    fn delete_removes_key() {
        let conn = fresh_conn();
        set(&conn, "tmp.flag", "yes").unwrap();
        assert!(delete(&conn, "tmp.flag").unwrap());
        assert_eq!(get(&conn, "tmp.flag").unwrap(), None);
        assert!(!delete(&conn, "tmp.flag").unwrap());
    }

    #[test]
    fn list_by_prefix_filters_namespace() {
        let conn = fresh_conn();
        set(&conn, "claude.user_mode", "subscription").unwrap();
        set(&conn, "claude.subscription_plan", "max20x").unwrap();
        set(&conn, "codex.something", "x").unwrap();

        let claude = list_by_prefix(&conn, "claude.").unwrap();
        assert_eq!(claude.len(), 2);
        assert!(claude.iter().all(|e| e.key.starts_with("claude.")));
    }
}
