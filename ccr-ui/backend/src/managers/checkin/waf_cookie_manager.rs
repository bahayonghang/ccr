// WAF Cookies 管理器
// 用于缓存通过浏览器获取的 WAF Cookies（如 acw_tc / cdn_sec_tc / acw_sc__v2）
// 使用 SQLite 统一存储（替代 JSON 文件）

use crate::database::{self, DatabaseError, repositories::checkin_repo};
use chrono::{Duration, Utc};
use std::collections::HashMap;

const DEFAULT_CACHE_HOURS: i64 = 24;

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
pub enum WafCookieError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    #[error("Failed to parse WAF cookies: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, WafCookieError>;

/// WAF Cookies 缓存管理器
/// 使用 SQLite 统一存储
pub struct WafCookieManager;

impl WafCookieManager {
    /// 创建新的 WAF Cookies 管理器
    /// 注意：不再需要 checkin_dir 参数，使用全局 SQLite 数据库
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    /// 获取有效的 WAF cookies（未命中或已过期则返回 None）
    pub fn get_valid(&self, provider_id: &str) -> Result<Option<HashMap<String, String>>> {
        // 清理过期的 cookies
        let _ = database::with_connection(checkin_repo::delete_expired_waf_cookies);

        let cookies_json = database::with_connection(|conn| {
            checkin_repo::get_valid_waf_cookies(conn, provider_id)
        })?;

        match cookies_json {
            Some(json) => {
                let cookies: HashMap<String, String> = serde_json::from_str(&json)
                    .map_err(|e| WafCookieError::ParseError(e.to_string()))?;
                Ok(Some(cookies))
            }
            None => Ok(None),
        }
    }

    /// 保存 WAF cookies（默认缓存 24 小时）
    pub fn save(&self, provider_id: &str, cookies: HashMap<String, String>) -> Result<()> {
        // 清理过期的 cookies
        let _ = database::with_connection(checkin_repo::delete_expired_waf_cookies);

        let cookies_json = serde_json::to_string(&cookies)
            .map_err(|e| WafCookieError::ParseError(e.to_string()))?;

        let fetched_at = Utc::now();
        let expires_at = fetched_at + Duration::hours(DEFAULT_CACHE_HOURS);

        database::with_connection(|conn| {
            checkin_repo::upsert_waf_cookies(
                conn,
                provider_id,
                &cookies_json,
                fetched_at,
                expires_at,
            )
        })?;

        tracing::debug!("Saved WAF cookies for provider: {}", provider_id);
        Ok(())
    }

    /// 删除某个 provider 的缓存
    #[allow(dead_code)]
    pub fn delete(&self, provider_id: &str) -> Result<bool> {
        let deleted = database::with_connection(|conn| {
            checkin_repo::delete_waf_cookies_by_provider(conn, provider_id)
        })?;
        Ok(deleted)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::database::schema::CREATE_TABLES_SQL;
    use once_cell::sync::Lazy;
    use rusqlite::Connection;
    use std::sync::Mutex;

    // Use a single in-memory database for tests
    static TEST_DB: Lazy<Mutex<Connection>> = Lazy::new(|| {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(CREATE_TABLES_SQL).unwrap();
        Mutex::new(conn)
    });

    fn with_test_db<F, R>(f: F) -> R
    where
        F: FnOnce(&Connection) -> R,
    {
        let conn = TEST_DB.lock().unwrap();
        // Clean up before each test
        conn.execute("DELETE FROM checkin_waf_cookies", []).unwrap();
        f(&conn)
    }

    #[test]
    fn test_save_and_get_valid() {
        with_test_db(|conn| {
            let mut cookies = HashMap::new();
            cookies.insert("acw_tc".to_string(), "value".to_string());

            let cookies_json = serde_json::to_string(&cookies).unwrap();
            let now = Utc::now();
            let expires = now + Duration::hours(1);

            checkin_repo::upsert_waf_cookies(conn, "provider-1", &cookies_json, now, expires)
                .unwrap();

            let loaded = checkin_repo::get_valid_waf_cookies(conn, "provider-1")
                .unwrap()
                .unwrap();
            let loaded_cookies: HashMap<String, String> = serde_json::from_str(&loaded).unwrap();
            assert_eq!(loaded_cookies.get("acw_tc"), Some(&"value".to_string()));
        });
    }

    #[test]
    fn test_delete() {
        with_test_db(|conn| {
            let mut cookies = HashMap::new();
            cookies.insert("acw_tc".to_string(), "value".to_string());

            let cookies_json = serde_json::to_string(&cookies).unwrap();
            let now = Utc::now();
            let expires = now + Duration::hours(1);

            checkin_repo::upsert_waf_cookies(conn, "provider-1", &cookies_json, now, expires)
                .unwrap();

            assert!(checkin_repo::delete_waf_cookies_by_provider(conn, "provider-1").unwrap());
            assert!(
                checkin_repo::get_valid_waf_cookies(conn, "provider-1")
                    .unwrap()
                    .is_none()
            );
        });
    }
}
