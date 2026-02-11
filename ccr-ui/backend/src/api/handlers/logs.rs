// 日志持久化 API 端点
#![allow(dead_code)]

use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::models::api::ApiResponse;
use crate::services::log_persistence::{LogPersistenceService, PersistedLog};

/// 日志查询参数
#[derive(Debug, Deserialize)]
pub struct LogQueryParams {
    /// 日期筛选 (YYYY-MM-DD)
    pub date: Option<String>,
    /// 返回数量限制
    pub limit: Option<usize>,
    /// 日志级别筛选
    pub level: Option<String>,
}

/// 日志列表响应
#[derive(Debug, Serialize)]
pub struct LogListResponse {
    pub logs: Vec<PersistedLog>,
    pub total: usize,
}

/// GET /api/logs - 获取日志列表
pub async fn list_logs(
    State(service): State<Arc<LogPersistenceService>>,
    Query(params): Query<LogQueryParams>,
) -> impl IntoResponse {
    let logs = if let Some(date) = params.date {
        service.read_logs_by_date(&date).await
    } else {
        let limit = params.limit.unwrap_or(100);
        service.read_recent_logs(limit).await
    };

    // 可选的级别筛选
    let filtered_logs: Vec<PersistedLog> = if let Some(level) = params.level {
        logs.into_iter()
            .filter(|log| log.level.to_lowercase() == level.to_lowercase())
            .collect()
    } else {
        logs
    };

    let total = filtered_logs.len();
    Json(ApiResponse::success(LogListResponse {
        logs: filtered_logs,
        total,
    }))
}

/// GET /api/logs/stats - 获取日志统计
pub async fn get_log_stats(State(service): State<Arc<LogPersistenceService>>) -> impl IntoResponse {
    let stats = service.get_stats().await;
    ApiResponse::success(stats)
}

/// GET /api/logs/dates - 获取可用的日志日期列表
pub async fn list_log_dates(
    State(service): State<Arc<LogPersistenceService>>,
) -> impl IntoResponse {
    let dates = service.get_available_dates().await;
    ApiResponse::success(dates)
}

/// POST /api/logs/flush - 强制刷新缓冲区
pub async fn flush_logs(State(service): State<Arc<LogPersistenceService>>) -> impl IntoResponse {
    service.force_flush().await;
    ApiResponse::success("Logs flushed successfully")
}

/// POST /api/logs/cleanup - 清理过期日志
pub async fn cleanup_old_logs(
    State(service): State<Arc<LogPersistenceService>>,
) -> impl IntoResponse {
    service.cleanup_old_logs().await;
    ApiResponse::success("Old logs cleaned up")
}

/// DELETE /api/logs/:date - 删除指定日期的日志
pub async fn delete_logs_by_date(
    State(service): State<Arc<LogPersistenceService>>,
    Path(date): Path<String>,
) -> impl IntoResponse {
    match service.delete_logs_by_date(&date).await {
        Ok(deleted) => {
            if deleted > 0 {
                ApiResponse::success(format!("Deleted {} logs for {}", deleted, date))
            } else {
                ApiResponse::error(format!("No logs found for {}", date))
            }
        }
        Err(e) => ApiResponse::error(format!("Failed to delete: {}", e)),
    }
}
