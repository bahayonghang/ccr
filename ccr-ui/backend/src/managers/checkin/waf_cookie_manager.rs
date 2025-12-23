// WAF Cookies 管理器
// 用于缓存通过浏览器获取的 WAF Cookies（如 acw_tc / cdn_sec_tc / acw_sc__v2）

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

const WAF_COOKIES_FILE: &str = "waf_cookies.json";
const DEFAULT_CACHE_HOURS: i64 = 24;

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
pub enum WafCookieError {
    #[error("Failed to read WAF cookies: {0}")]
    ReadError(String),
    #[error("Failed to write WAF cookies: {0}")]
    WriteError(String),
    #[error("Failed to parse WAF cookies: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, WafCookieError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredWafCookies {
    provider_id: String,
    cookies: HashMap<String, String>,
    fetched_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

/// WAF Cookies 缓存管理器
pub struct WafCookieManager {
    file_path: PathBuf,
}

impl WafCookieManager {
    /// 创建新的 WAF Cookies 管理器
    pub fn new(checkin_dir: &Path) -> Self {
        Self {
            file_path: checkin_dir.join(WAF_COOKIES_FILE),
        }
    }

    fn load_all(&self) -> Result<Vec<StoredWafCookies>> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| WafCookieError::ReadError(e.to_string()))?;
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }

        serde_json::from_str(&content).map_err(|e| WafCookieError::ParseError(e.to_string()))
    }

    fn save_all(&self, items: &[StoredWafCookies]) -> Result<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                WafCookieError::WriteError(format!("Failed to create directory: {}", e))
            })?;
        }

        let content = serde_json::to_string_pretty(items)
            .map_err(|e| WafCookieError::WriteError(format!("Failed to serialize: {}", e)))?;

        let temp_dir = self.file_path.parent().unwrap_or(Path::new("."));
        let mut temp_file = NamedTempFile::new_in(temp_dir).map_err(|e| {
            WafCookieError::WriteError(format!("Failed to create temp file: {}", e))
        })?;

        temp_file
            .write_all(content.as_bytes())
            .map_err(|e| WafCookieError::WriteError(format!("Failed to write temp file: {}", e)))?;

        temp_file
            .persist(&self.file_path)
            .map_err(|e| WafCookieError::WriteError(format!("Failed to persist file: {}", e)))?;

        Ok(())
    }

    fn cleanup_expired(items: &mut Vec<StoredWafCookies>) -> bool {
        let now = Utc::now();
        let before = items.len();
        items.retain(|item| item.expires_at > now);
        before != items.len()
    }

    /// 获取有效的 WAF cookies（未命中或已过期则返回 None）
    pub fn get_valid(&self, provider_id: &str) -> Result<Option<HashMap<String, String>>> {
        let mut items = self.load_all()?;
        let changed = Self::cleanup_expired(&mut items);
        if changed {
            let _ = self.save_all(&items);
        }

        Ok(items
            .into_iter()
            .find(|item| item.provider_id == provider_id)
            .map(|item| item.cookies))
    }

    /// 保存 WAF cookies（默认缓存 24 小时）
    pub fn save(&self, provider_id: &str, cookies: HashMap<String, String>) -> Result<()> {
        let mut items = self.load_all()?;
        let _ = Self::cleanup_expired(&mut items);

        items.retain(|item| item.provider_id != provider_id);

        let fetched_at = Utc::now();
        items.push(StoredWafCookies {
            provider_id: provider_id.to_string(),
            cookies,
            fetched_at,
            expires_at: fetched_at + Duration::hours(DEFAULT_CACHE_HOURS),
        });

        self.save_all(&items)
    }

    /// 删除某个 provider 的缓存
    #[allow(dead_code)]
    pub fn delete(&self, provider_id: &str) -> Result<bool> {
        let mut items = self.load_all()?;
        let before = items.len();
        items.retain(|item| item.provider_id != provider_id);

        let deleted = before != items.len();
        if deleted {
            self.save_all(&items)?;
        }

        Ok(deleted)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_get_valid() {
        let temp_dir = TempDir::new().unwrap();
        let manager = WafCookieManager::new(temp_dir.path());

        let mut cookies = HashMap::new();
        cookies.insert("acw_tc".to_string(), "value".to_string());

        manager.save("provider-1", cookies.clone()).unwrap();

        let loaded = manager.get_valid("provider-1").unwrap().unwrap();
        assert_eq!(loaded.get("acw_tc"), Some(&"value".to_string()));
    }

    #[test]
    fn test_delete() {
        let temp_dir = TempDir::new().unwrap();
        let manager = WafCookieManager::new(temp_dir.path());

        let mut cookies = HashMap::new();
        cookies.insert("acw_tc".to_string(), "value".to_string());
        manager.save("provider-1", cookies).unwrap();

        assert!(manager.delete("provider-1").unwrap());
        assert!(manager.get_valid("provider-1").unwrap().is_none());
    }
}
