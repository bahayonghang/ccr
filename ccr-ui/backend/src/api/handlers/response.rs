//! API 响应工具模块
//!
//! 提供统一的 API 响应格式和错误处理，减少 Handler 中的重复代码。
//!
//! ## 推荐用法（新代码）
//!
//! Handler 返回 `ApiResult<ApiSuccess<T>>` 即可自动生成标准化 JSON 响应：
//!
//! ```rust,ignore
//! pub async fn list_items() -> ApiResult<ApiSuccess<Vec<Item>>> {
//!     let items = load_items()?; // 错误自动转为 ApiError
//!     Ok(ApiSuccess(items))
//! }
//! ```

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::json;

// ============================================================
// 新统一响应类型（推荐新代码使用）
// ============================================================

/// 内部序列化用的响应体
#[derive(Serialize)]
struct SuccessBody<T: Serialize> {
    success: bool,
    data: T,
    message: Option<String>,
}

/// 统一成功响应包装器
///
/// 与 `ApiResult<T>` 配合使用，实现零样板 Handler：
///
/// ```rust,ignore
/// pub async fn get_data() -> ApiResult<ApiSuccess<MyData>> {
///     let data = fetch_data()?;
///     Ok(ApiSuccess(data))
/// }
/// ```
///
/// 生成的 JSON 响应：`{ "success": true, "data": <T>, "message": null }`
pub struct ApiSuccess<T: Serialize>(pub T);

impl<T: Serialize> IntoResponse for ApiSuccess<T> {
    fn into_response(self) -> Response {
        let body = SuccessBody {
            success: true,
            data: self.0,
            message: None,
        };
        (StatusCode::OK, Json(body)).into_response()
    }
}

impl<T: Serialize> ApiSuccess<T> {
    /// 创建带消息的成功响应
    #[allow(dead_code)]
    pub fn with_message(data: T, message: impl Into<String>) -> ApiSuccessWithMessage<T> {
        ApiSuccessWithMessage {
            data,
            message: message.into(),
        }
    }
}

/// 带消息的成功响应（通过 `ApiSuccess::with_message()` 创建）
pub struct ApiSuccessWithMessage<T: Serialize> {
    data: T,
    message: String,
}

impl<T: Serialize> IntoResponse for ApiSuccessWithMessage<T> {
    fn into_response(self) -> Response {
        let body = SuccessBody {
            success: true,
            data: self.data,
            message: Some(self.message),
        };
        (StatusCode::OK, Json(body)).into_response()
    }
}

// ============================================================
// 旧版响应类型（保留向后兼容，逐步迁移到 ApiSuccess）
// ============================================================

/// 统一的 API 响应结构（旧版，新代码请使用 `ApiSuccess<T>`）
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    /// 创建成功响应（带数据）
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    /// 创建成功响应（带消息）
    #[allow(dead_code)]
    pub fn success_message(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: true,
            data: None,
            message: Some(message.into()),
        }
    }

    /// 创建错误响应
    #[allow(dead_code)]
    pub fn error(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            message: Some(message.into()),
        }
    }
}

/// 成功响应（带数据）
pub fn ok<T: Serialize>(data: T) -> impl IntoResponse {
    (StatusCode::OK, Json(ApiResponse::success(data)))
}

/// 成功响应（带消息）
pub fn ok_message(message: impl Into<String>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "data": null,
            "message": message.into()
        })),
    )
}

/// 成功响应（带数据和消息）
#[allow(dead_code)]
pub fn ok_with_message<T: Serialize>(data: T, message: impl Into<String>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "data": data,
            "message": message.into()
        })),
    )
}

/// 错误响应 - 内部服务器错误
pub fn internal_error(message: impl Into<String>) -> impl IntoResponse {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "success": false,
            "data": null,
            "message": message.into()
        })),
    )
}

/// 错误响应 - 请求错误
pub fn bad_request(message: impl Into<String>) -> impl IntoResponse {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "success": false,
            "data": null,
            "message": message.into()
        })),
    )
}

/// 错误响应 - 未找到
pub fn not_found(message: impl Into<String>) -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "success": false,
            "data": null,
            "message": message.into()
        })),
    )
}

/// 通用的 Manager 初始化错误处理
#[allow(dead_code)]
pub fn manager_init_error(platform: &str, error: impl std::fmt::Display) -> impl IntoResponse {
    internal_error(format!("初始化 {} 配置管理器失败: {}", platform, error))
}

/// 通用的操作成功响应
#[allow(dead_code)]
pub fn operation_success(
    operation: &str,
    resource_type: &str,
    resource_name: &str,
) -> impl IntoResponse {
    ok_message(format!(
        "{} '{}' {}成功",
        resource_type, resource_name, operation
    ))
}

/// 通用的操作失败响应
#[allow(dead_code)]
pub fn operation_error(
    operation: &str,
    resource_type: &str,
    error: impl std::fmt::Display,
) -> impl IntoResponse {
    bad_request(format!("{} {} 失败: {}", operation, resource_type, error))
}

/// Result 类型的辅助 trait，简化 Handler 代码
#[allow(dead_code)]
pub trait ResultExt<T> {
    /// 将 Result 转换为 API 响应
    fn to_response(
        self,
        success_message: impl Into<String>,
        error_prefix: impl Into<String>,
    ) -> impl IntoResponse;

    /// 将 Result 转换为带数据的 API 响应
    fn to_data_response(self, error_prefix: impl Into<String>) -> impl IntoResponse
    where
        T: Serialize;
}

#[allow(dead_code)]
impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn to_response(
        self,
        success_message: impl Into<String>,
        error_prefix: impl Into<String>,
    ) -> impl IntoResponse {
        match self {
            Ok(_) => ok_message(success_message).into_response(),
            Err(e) => bad_request(format!("{}: {}", error_prefix.into(), e)).into_response(),
        }
    }

    fn to_data_response(self, error_prefix: impl Into<String>) -> impl IntoResponse
    where
        T: Serialize,
    {
        match self {
            Ok(data) => ok(data).into_response(),
            Err(e) => internal_error(format!("{}: {}", error_prefix.into(), e)).into_response(),
        }
    }
}

/// 宏：简化 Manager 初始化和操作
#[macro_export]
macro_rules! with_manager {
    ($manager_type:ty, $platform:expr, $body:expr) => {
        match <$manager_type>::default() {
            Ok(manager) => $body(manager),
            Err(e) => {
                $crate::api::handlers::response::manager_init_error($platform, e).into_response()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data");
        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
        assert!(response.message.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response = ApiResponse::<()>::error("test error");
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.message, Some("test error".to_string()));
    }
}
