// WAF Cookies 管理器
// 用于缓存通过浏览器获取的 WAF Cookies（如 acw_tc / cdn_sec_tc / acw_sc__v2）
// 使用 SQLite 统一存储（替代 JSON 文件）

use crate::core::error::DbError;
use crate::database::{self, repositories::checkin_repo};
use crate::models::checkin::CheckinProvider;
use chrono::{Duration, Utc};
use std::collections::HashMap;

const DEFAULT_CACHE_HOURS: i64 = 24;
pub const ANYROUTER_REQUIRED_WAF_COOKIE_NAMES: &[&str] = &["acw_tc", "cdn_sec_tc", "acw_sc__v2"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WafCookiePolicy {
    pub required_cookie_names: Vec<String>,
    pub validation_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WafCookieSelection {
    pub cookies: HashMap<String, String>,
    pub found_cookie_names: Vec<String>,
    pub missing_cookie_names: Vec<String>,
}

impl WafCookieSelection {
    pub fn is_complete(&self) -> bool {
        self.missing_cookie_names.is_empty() && !self.cookies.is_empty()
    }
}

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
pub enum WafCookieError {
    #[error("Database error: {0}")]
    Database(#[from] DbError),
    #[error("Failed to parse WAF cookies: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, WafCookieError>;

/// WAF Cookies 缓存管理器
/// 使用 SQLite 统一存储
pub struct WafCookieManager;

impl WafCookieManager {
    /// 创建新的 WAF Cookies 管理器
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

    pub fn policy_for_provider(provider: &CheckinProvider) -> Option<WafCookiePolicy> {
        Self::policy_for_provider_parts(&provider.id, Some(&provider.name), &provider.base_url)
    }

    pub fn policy_for_provider_parts(
        provider_id: &str,
        provider_name: Option<&str>,
        base_url: &str,
    ) -> Option<WafCookiePolicy> {
        let provider_id = provider_id.to_ascii_lowercase();
        let provider_name = provider_name.unwrap_or_default().to_ascii_lowercase();
        let base_url = base_url.to_ascii_lowercase();

        let is_anyrouter = provider_id == "builtin-anyrouter"
            || provider_name == "anyrouter"
            || base_url.contains("anyrouter.top");

        if !is_anyrouter {
            return None;
        }

        Some(WafCookiePolicy {
            required_cookie_names: ANYROUTER_REQUIRED_WAF_COOKIE_NAMES
                .iter()
                .map(|name| (*name).to_string())
                .collect(),
            validation_path: Some("/api/user/self".to_string()),
        })
    }

    pub fn parse_cookie_pairs(cookie_str: &str) -> HashMap<String, String> {
        let mut cookies = HashMap::new();

        for pair in cookie_str.split(';') {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }

            if let Some((key, value)) = pair.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                if !key.is_empty() && !value.is_empty() {
                    cookies.insert(key.to_string(), value.to_string());
                }
            }
        }

        cookies
    }

    pub fn select_required_cookies(
        cookies: &HashMap<String, String>,
        required_cookie_names: &[String],
    ) -> WafCookieSelection {
        if required_cookie_names.is_empty() {
            let mut found_cookie_names: Vec<String> = cookies.keys().cloned().collect();
            found_cookie_names.sort();
            return WafCookieSelection {
                cookies: cookies.clone(),
                found_cookie_names,
                missing_cookie_names: Vec::new(),
            };
        }

        let mut selected = HashMap::new();
        let mut found_cookie_names = Vec::new();
        let mut missing_cookie_names = Vec::new();

        for required_name in required_cookie_names {
            match cookies.get(required_name) {
                Some(value) if !value.trim().is_empty() => {
                    selected.insert(required_name.clone(), value.clone());
                    found_cookie_names.push(required_name.clone());
                }
                _ => missing_cookie_names.push(required_name.clone()),
            }
        }

        WafCookieSelection {
            cookies: selected,
            found_cookie_names,
            missing_cookie_names,
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

    /// 通过浏览器自动获取 WAF bypass cookies
    /// NOTE: Will be reimplemented with Tauri WebView
    #[allow(dead_code)]
    pub fn fetch_via_browser(
        &self,
        _provider_id: &str,
        _url: &str,
    ) -> Result<std::collections::HashMap<String, String>> {
        Err(WafCookieError::ParseError(
            "Browser WAF cookie fetch is not implemented in this build".to_string(),
        ))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::database::repositories::checkin_repo;
    use crate::database::schema::CREATE_TABLES_SQL;
    use once_cell::sync::Lazy;
    use rusqlite::Connection;
    use std::sync::Mutex;

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

    #[test]
    fn test_fetch_via_browser_returns_error_until_implemented() {
        let manager = WafCookieManager::new();

        let err = manager
            .fetch_via_browser("provider-1", "https://example.com")
            .unwrap_err();

        assert!(
            err.to_string()
                .contains("Browser WAF cookie fetch is not implemented")
        );
    }

    #[test]
    fn test_anyrouter_policy_matches_builtin_and_domain() {
        let provider =
            CheckinProvider::new("AnyRouter".to_string(), "https://anyrouter.top".to_string());

        let policy = WafCookieManager::policy_for_provider(&provider).unwrap();

        assert_eq!(
            policy.required_cookie_names,
            vec![
                "acw_tc".to_string(),
                "cdn_sec_tc".to_string(),
                "acw_sc__v2".to_string()
            ]
        );
        assert_eq!(policy.validation_path.as_deref(), Some("/api/user/self"));
    }

    #[test]
    fn test_parse_cookie_pairs_ignores_empty_segments() {
        let cookies = WafCookieManager::parse_cookie_pairs(
            "acw_tc=tc-value; ; cdn_sec_tc=cdn-value; no-value; acw_sc__v2=v2-value",
        );

        assert_eq!(cookies.get("acw_tc"), Some(&"tc-value".to_string()));
        assert_eq!(cookies.get("cdn_sec_tc"), Some(&"cdn-value".to_string()));
        assert_eq!(cookies.get("acw_sc__v2"), Some(&"v2-value".to_string()));
        assert!(!cookies.contains_key("no-value"));
    }

    #[test]
    fn test_select_required_cookies_reports_missing_names_without_values() {
        let mut cookies = HashMap::new();
        cookies.insert("acw_tc".to_string(), "secret-tc".to_string());
        cookies.insert("cdn_sec_tc".to_string(), "secret-cdn".to_string());
        cookies.insert("session".to_string(), "secret-session".to_string());
        let required = ANYROUTER_REQUIRED_WAF_COOKIE_NAMES
            .iter()
            .map(|name| (*name).to_string())
            .collect::<Vec<_>>();

        let selection = WafCookieManager::select_required_cookies(&cookies, &required);

        assert!(!selection.is_complete());
        assert_eq!(
            selection.found_cookie_names,
            vec!["acw_tc".to_string(), "cdn_sec_tc".to_string()]
        );
        assert_eq!(
            selection.missing_cookie_names,
            vec!["acw_sc__v2".to_string()]
        );
        assert!(!selection.cookies.contains_key("session"));
    }
}
