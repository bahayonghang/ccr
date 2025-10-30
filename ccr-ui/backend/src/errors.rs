// 统一错误处理模块
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

/// 统一的 API 错误响应体
#[derive(Debug, Serialize)]
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
    /// 404 Not Found - 资源不存在
    NotFound(String),
    /// 500 Internal Server Error - 服务器内部错误
    Internal(String),
    /// 503 Service Unavailable - 服务不可用
    ServiceUnavailable(String),
    /// 自定义状态码错误
    Custom(StatusCode, String),
}

impl ApiError {
    /// 创建 BadRequest 错误
    pub fn bad_request(msg: impl Into<String>) -> Self {
        ApiError::BadRequest(msg.into())
    }

    /// 创建 NotFound 错误
    pub fn not_found(msg: impl Into<String>) -> Self {
        ApiError::NotFound(msg.into())
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
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            ApiError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg.clone()),
            ApiError::Custom(status, msg) => (*status, msg.clone()),
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

/// 实现 IntoResponse，自动将错误转换为 HTTP 响应
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = self.status_and_message();
        
        let body = ApiErrorBody {
            code: status.as_u16(),
            message: message.clone(),
            details: None,
        };

        tracing::error!(
            status = status.as_u16(),
            message = %message,
            "API error occurred"
        );

        (status, Json(body)).into_response()
    }
}

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

/// ApiResult 类型别名，简化函数签名
pub type ApiResult<T> = Result<T, ApiError>;

#[cfg(test)]
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
    }
}
