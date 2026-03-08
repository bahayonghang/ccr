//! ccr-db 统一错误类型。
//!
//! 独立于 axum，不包含 HTTP 响应转换逻辑。

use thiserror::Error;

/// 数据库层错误
#[derive(Debug, Error)]
pub enum DbError {
    #[error("Home directory not found")]
    HomeDirNotFound,

    #[error("Failed to create directory: {0}")]
    DirectoryCreation(String),

    #[error("Database connection failed: {0}")]
    ConnectionOpen(String),

    #[error("Migration failed: {0}")]
    Migration(String),

    #[error("Database not initialized")]
    NotInitialized,

    #[error("Query failed: {0}")]
    Query(String),

    #[error("Pool error: {0}")]
    Pool(String),

    #[error("Pool get connection failed: {0}")]
    PoolGet(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(String),
}

impl From<rusqlite::Error> for DbError {
    fn from(e: rusqlite::Error) -> Self {
        DbError::Query(e.to_string())
    }
}

impl From<std::io::Error> for DbError {
    fn from(e: std::io::Error) -> Self {
        DbError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for DbError {
    fn from(e: serde_json::Error) -> Self {
        DbError::Serialization(e.to_string())
    }
}

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
    /// 返回结构化错误分类代码，用于前端展示可操作的错误提示
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

/// 迁移错误
#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("JSON parse error: {0}")]
    Json(String),

    #[error("Migration already applied: v{0}")]
    AlreadyApplied(i32),
}

impl From<rusqlite::Error> for MigrationError {
    fn from(e: rusqlite::Error) -> Self {
        MigrationError::Database(e.to_string())
    }
}

impl From<std::io::Error> for MigrationError {
    fn from(e: std::io::Error) -> Self {
        MigrationError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for MigrationError {
    fn from(e: serde_json::Error) -> Self {
        MigrationError::Json(e.to_string())
    }
}

/// 命令执行器错误
#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Command timed out after {0}s")]
    Timeout(u64),

    #[error("Binary not found: {0}")]
    BinaryNotFound(String),

    #[error("Stdio handle missing")]
    StdioHandleMissing,
}
