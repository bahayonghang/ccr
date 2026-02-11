// 统一错误处理模块
//
// 提供统一的 API 错误类型和响应格式，支持从各种内部错误类型自动转换。

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::fmt;

use crate::database::DatabaseError;
use crate::services::checkin_service::CheckinServiceError;

/// 统一的 API 错误响应体（旧格式，保留用于兼容）
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct ApiErrorBody {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// 统一的 API 错误类型
#[derive(Debug)]
pub enum ApiError {
    /// 400 Bad Request - 客户端请求错误
    BadRequest(String),
    /// 401 Unauthorized - 未授权
    #[allow(dead_code)]
    Unauthorized(String),
    /// 403 Forbidden - 禁止访问
    #[allow(dead_code)]
    Forbidden(String),
    /// 404 Not Found - 资源不存在
    NotFound(String),
    /// 409 Conflict - 资源冲突
    #[allow(dead_code)]
    Conflict(String),
    /// 422 Unprocessable Entity - 无法处理的实体
    #[allow(dead_code)]
    UnprocessableEntity(String),
    /// 500 Internal Server Error - 服务器内部错误
    Internal(String),
    /// 503 Service Unavailable - 服务不可用
    ServiceUnavailable(String),
}

impl ApiError {
    /// 创建 BadRequest 错误
    pub fn bad_request(msg: impl Into<String>) -> Self {
        ApiError::BadRequest(msg.into())
    }

    /// 创建 Unauthorized 错误
    #[allow(dead_code)]
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        ApiError::Unauthorized(msg.into())
    }

    /// 创建 Forbidden 错误
    #[allow(dead_code)]
    pub fn forbidden(msg: impl Into<String>) -> Self {
        ApiError::Forbidden(msg.into())
    }

    /// 创建 NotFound 错误
    pub fn not_found(msg: impl Into<String>) -> Self {
        ApiError::NotFound(msg.into())
    }

    /// 创建 Conflict 错误
    #[allow(dead_code)]
    pub fn conflict(msg: impl Into<String>) -> Self {
        ApiError::Conflict(msg.into())
    }

    /// 创建 UnprocessableEntity 错误
    #[allow(dead_code)]
    pub fn unprocessable_entity(msg: impl Into<String>) -> Self {
        ApiError::UnprocessableEntity(msg.into())
    }

    /// 创建 Internal 错误
    pub fn internal(msg: impl Into<String>) -> Self {
        ApiError::Internal(msg.into())
    }

    /// 创建 ServiceUnavailable 错误
    pub fn service_unavailable(msg: impl Into<String>) -> Self {
        ApiError::ServiceUnavailable(msg.into())
    }

    /// 获取 HTTP 状态码和错误消息
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            ApiError::UnprocessableEntity(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ApiError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (_, msg) = self.status_and_message();
        write!(f, "{}", msg)
    }
}

impl std::error::Error for ApiError {}

/// 统一的错误响应体（与前端 `{ success, data, message }` 格式一致）
#[derive(Debug, Serialize)]
struct ApiErrorResponse {
    success: bool,
    data: Option<()>,
    message: String,
}

/// 实现 IntoResponse，自动将错误转换为 HTTP 响应
///
/// 输出格式与前端约定一致：`{ "success": false, "data": null, "message": "..." }`
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = self.status_and_message();

        tracing::error!(
            status = status.as_u16(),
            message = %message,
            "API error occurred"
        );

        let body = ApiErrorResponse {
            success: false,
            data: None,
            message,
        };

        (status, Json(body)).into_response()
    }
}

// ============================================================
// From 实现 - 自动错误转换
// ============================================================

/// 从 std::io::Error 转换
impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::Internal(format!("IO error: {}", err))
    }
}

/// 从 serde_json::Error 转换
impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest(format!("JSON error: {}", err))
    }
}

/// 从 DatabaseError 转换
impl From<DatabaseError> for ApiError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::NotInitialized => {
                ApiError::ServiceUnavailable("Database not initialized".to_string())
            }
            DatabaseError::Query(e) => ApiError::Internal(format!("Database query error: {}", e)),
            DatabaseError::Pool(e) => ApiError::Internal(format!("Database pool error: {}", e)),
            DatabaseError::PoolGet(e) => {
                ApiError::Internal(format!("Failed to get database connection: {}", e))
            }
            _ => ApiError::Internal(format!("Database error: {}", err)),
        }
    }
}

/// 从 CheckinServiceError 转换
impl From<CheckinServiceError> for ApiError {
    fn from(err: CheckinServiceError) -> Self {
        match err {
            CheckinServiceError::ProviderError(msg) => ApiError::NotFound(msg),
            CheckinServiceError::AccountError(msg) => ApiError::NotFound(msg),
            CheckinServiceError::CryptoError(msg) => ApiError::Internal(msg),
            CheckinServiceError::NetworkError(msg) => ApiError::ServiceUnavailable(msg),
            CheckinServiceError::ApiError(msg) => ApiError::BadRequest(msg),
            CheckinServiceError::RecordError(msg) => ApiError::Internal(msg),
            CheckinServiceError::BalanceError(msg) => ApiError::Internal(msg),
        }
    }
}

/// 从 anyhow::Error 转换
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal(err.to_string())
    }
}

/// 从 rusqlite::Error 转换
impl From<rusqlite::Error> for ApiError {
    fn from(err: rusqlite::Error) -> Self {
        ApiError::Internal(format!("SQLite error: {}", err))
    }
}

/// 从 reqwest::Error 转换
impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            ApiError::ServiceUnavailable(format!("Request timeout: {}", err))
        } else if err.is_connect() {
            ApiError::ServiceUnavailable(format!("Connection error: {}", err))
        } else {
            ApiError::Internal(format!("HTTP client error: {}", err))
        }
    }
}

/// 从 String 转换（便于 `.map_err(|e| format!("..."))?` 模式）
impl From<String> for ApiError {
    fn from(msg: String) -> Self {
        ApiError::Internal(msg)
    }
}

/// 从 tokio::task::JoinError 转换（spawn_blocking 场景）
impl From<tokio::task::JoinError> for ApiError {
    fn from(err: tokio::task::JoinError) -> Self {
        ApiError::Internal(format!("Task join error: {}", err))
    }
}

// ============================================================
// 类型别名
// ============================================================

/// ApiResult 类型别名，简化函数签名
pub type ApiResult<T> = Result<T, ApiError>;

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_creation() {
        let err = ApiError::bad_request("Invalid input");
        assert!(matches!(err, ApiError::BadRequest(_)));

        let err = ApiError::not_found("Resource not found");
        assert!(matches!(err, ApiError::NotFound(_)));

        let err = ApiError::internal("Database error");
        assert!(matches!(err, ApiError::Internal(_)));
    }

    #[test]
    fn test_api_error_status_codes() {
        let err = ApiError::BadRequest("test".to_string());
        let (status, _) = err.status_and_message();
        assert_eq!(status, StatusCode::BAD_REQUEST);

        let err = ApiError::NotFound("test".to_string());
        let (status, _) = err.status_and_message();
        assert_eq!(status, StatusCode::NOT_FOUND);

        let err = ApiError::Internal("test".to_string());
        let (status, _) = err.status_and_message();
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);

        let err = ApiError::Unauthorized("test".to_string());
        let (status, _) = err.status_and_message();
        assert_eq!(status, StatusCode::UNAUTHORIZED);

        let err = ApiError::Forbidden("test".to_string());
        let (status, _) = err.status_and_message();
        assert_eq!(status, StatusCode::FORBIDDEN);

        let err = ApiError::Conflict("test".to_string());
        let (status, _) = err.status_and_message();
        assert_eq!(status, StatusCode::CONFLICT);

        let err = ApiError::ServiceUnavailable("test".to_string());
        let (status, _) = err.status_and_message();
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let api_err: ApiError = io_err.into();
        assert!(matches!(api_err, ApiError::Internal(_)));
    }

    #[test]
    fn test_display() {
        let err = ApiError::bad_request("test message");
        assert_eq!(err.to_string(), "test message");
    }
}
