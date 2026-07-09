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
