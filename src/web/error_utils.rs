// 🎯 统一的错误处理工具
// 提供标准化的错误处理和响应生成工具

use crate::core::error::CcrError;
use crate::web::models::ApiResponse;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// 🎯 异步执行阻塞任务并转换为 String 错误
///
/// 这个函数包装了 tokio::task::spawn_blocking，并将所有错误转换为 String。
/// 这样可以统一错误处理，使其与 error_utils 函数兼容。
pub async fn spawn_blocking_string<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, CcrError> + Send + 'static,
    T: Send + 'static,
{
    match tokio::task::spawn_blocking(f).await {
        Ok(result) => result.map_err(|e| e.user_message()),
        Err(e) => Err(format!("任务执行失败: {}", e)),
    }
}

/// 🎯 创建标准化的错误响应
pub fn create_error_response<E: Into<String>>(status: StatusCode, message: E) -> Response {
    let error_response: ApiResponse<()> = ApiResponse::error_without_data(message.into());
    (status, Json(error_response)).into_response()
}

/// 🎯 创建内部服务器错误响应 (500)
pub fn internal_server_error<E: Into<String>>(message: E) -> Response {
    create_error_response(StatusCode::INTERNAL_SERVER_ERROR, message)
}

/// 🎯 创建坏请求错误响应 (400)
pub fn bad_request<E: Into<String>>(message: E) -> Response {
    create_error_response(StatusCode::BAD_REQUEST, message)
}

/// 🎯 创建未找到错误响应 (404)
pub fn not_found<E: Into<String>>(message: E) -> Response {
    create_error_response(StatusCode::NOT_FOUND, message)
}

/// 🎯 成功响应包装器
pub fn success_response<T: serde::Serialize>(data: T) -> Response {
    Json(ApiResponse::success(data)).into_response()
}

/// 🎯 空成功响应
pub fn empty_success_response() -> Response {
    Json(ApiResponse::success("操作成功")).into_response()
}

/// 🎯 宏简化 spawn_blocking 错误处理
#[macro_export]
macro_rules! spawn_blocking_with_error {
    ($future:expr) => {
        tokio::task::spawn_blocking($future)
            .await
            .unwrap_or_else(|e| Err(format!("任务执行失败: {}", e)))
    };
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_error_response_creation() {
        let resp = create_error_response(StatusCode::BAD_REQUEST, "测试错误");
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_success_response() {
        let data = serde_json::json!({ "message": "success" });
        let resp = success_response(data);
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
