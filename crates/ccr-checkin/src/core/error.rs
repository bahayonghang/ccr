use thiserror::Error;

pub use ccr_db::core::error::{DbError, ExecutorError, MigrationError};

/// 签到服务错误
#[derive(Debug, Error)]
pub enum CheckinServiceError {
    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Account error: {0}")]
    Account(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Record error: {0}")]
    Record(String),

    #[error("Balance error: {0}")]
    Balance(String),

    #[error("Database error: {0}")]
    Database(#[from] DbError),
}

impl CheckinServiceError {
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Provider(_) => "provider_error",
            Self::Account(_) => "account_error",
            Self::Crypto(_) => "crypto_error",
            Self::Network(_) => "network_error",
            Self::Api(msg) => {
                if msg.contains("WAF") || msg.contains("waf") {
                    "waf_blocked"
                } else if msg.contains("Cloudflare")
                    || msg.contains("cf_clearance")
                    || msg.contains("cloudflare")
                {
                    "cf_blocked"
                } else if msg.contains("401")
                    || msg.contains("403")
                    || msg.contains("Unauthorized")
                    || msg.contains("cookie")
                    || msg.contains("Cookie")
                    || msg.contains("token")
                    || msg.contains("expired")
                {
                    "cookie_expired"
                } else {
                    "api_error"
                }
            }
            Self::Record(_) => "api_error",
            Self::Balance(_) => "api_error",
            Self::Database(_) => "api_error",
        }
    }
}

// error_code() 的消息关键词分类是前端 hint/label 与 WAF 自动补救的隐式契约，
// 以下矩阵测试锁住关键词与优先级（WAF > Cloudflare > cookie），改动需同步前端映射
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_error_codes_are_stable() {
        let cases = [
            (
                CheckinServiceError::Provider("bad provider".into()),
                "provider_error",
            ),
            (
                CheckinServiceError::Account("missing account".into()),
                "account_error",
            ),
            (
                CheckinServiceError::Crypto("decrypt failed".into()),
                "crypto_error",
            ),
            (
                CheckinServiceError::Network("connection refused".into()),
                "network_error",
            ),
            (
                CheckinServiceError::Record("insert failed".into()),
                "api_error",
            ),
            (
                CheckinServiceError::Balance("balance missing".into()),
                "api_error",
            ),
            (
                CheckinServiceError::Database(DbError::Query("query failed".into())),
                "api_error",
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(error.error_code(), expected, "error: {error}");
        }
    }

    #[test]
    fn api_waf_keywords_map_to_waf_blocked() {
        for msg in [
            "检测到 WAF 挑战页面（响应为 HTML）",
            "waf challenge detected",
        ] {
            assert_eq!(
                CheckinServiceError::Api(msg.into()).error_code(),
                "waf_blocked",
                "message: {msg}"
            );
        }
    }

    #[test]
    fn api_cloudflare_keywords_map_to_cf_blocked() {
        for msg in [
            "Cloudflare challenge page",
            "missing cf_clearance cookie value",
            "cloudflare blocked the request",
        ] {
            assert_eq!(
                CheckinServiceError::Api(msg.into()).error_code(),
                "cf_blocked",
                "message: {msg}"
            );
        }
    }

    #[test]
    fn api_auth_keywords_map_to_cookie_expired() {
        for msg in [
            "HTTP 401: 签到失败",
            "HTTP 403 Forbidden",
            "Unauthorized request",
            "cookie missing in request",
            "Cookie 无效",
            "token invalid",
            "session expired",
        ] {
            assert_eq!(
                CheckinServiceError::Api(msg.into()).error_code(),
                "cookie_expired",
                "message: {msg}"
            );
        }
    }

    #[test]
    fn api_without_keywords_maps_to_api_error() {
        assert_eq!(
            CheckinServiceError::Api("签到接口返回异常".into()).error_code(),
            "api_error"
        );
    }

    #[test]
    fn api_keyword_priority_is_waf_then_cloudflare_then_cookie() {
        // WAF 优先于 cookie：同时命中时归类为 waf_blocked
        assert_eq!(
            CheckinServiceError::Api("WAF 挑战导致 cookie 校验失败".into()).error_code(),
            "waf_blocked"
        );
        // Cloudflare 优先于 cookie/401
        assert_eq!(
            CheckinServiceError::Api("Cloudflare returned HTTP 403".into()).error_code(),
            "cf_blocked"
        );
        // WAF 优先于 Cloudflare
        assert_eq!(
            CheckinServiceError::Api("WAF/cloudflare mixed challenge".into()).error_code(),
            "waf_blocked"
        );
    }
}
