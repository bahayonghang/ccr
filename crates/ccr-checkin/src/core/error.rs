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
