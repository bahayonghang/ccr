use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum UsageError {
    #[error("llmusage_db_missing: {0}")]
    DbMissing(PathBuf),
    #[error("llmusage_db_unreadable: {0}")]
    DbUnreadable(String),
    #[error("llmusage_schema_unsupported: expected schema >= {expected}, got {actual:?}")]
    SchemaUnsupported { expected: i64, actual: Option<i64> },
    #[error("llmusage_feature_unavailable: {feature}: {reason}")]
    FeatureUnavailable {
        feature: &'static str,
        reason: String,
    },
    #[error("llmusage_query_error: {0}")]
    Query(String),
}

impl From<rusqlite::Error> for UsageError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Query(value.to_string())
    }
}
