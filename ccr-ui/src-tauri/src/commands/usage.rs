//! Usage V2 命令 — SQLite 用量统计查询与导入。

use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum UsageLogsMode {
    #[default]
    Cursor,
    Offset,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UsageLogsQuery {
    pub platform: Option<String>,
    pub model: Option<String>,
    #[serde(alias = "startDate")]
    pub start_date: Option<String>,
    #[serde(alias = "endDate")]
    pub end_date: Option<String>,
    pub page: Option<i64>,
    #[serde(alias = "pageSize")]
    pub page_size: Option<i64>,
    pub cursor: Option<String>,
    #[serde(alias = "includeTotal")]
    pub include_total: Option<bool>,
    pub mode: Option<UsageLogsMode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedLogsV2 {
    pub records: Vec<ccr_db::database::repositories::usage_repo::UsageRecord>,
    pub total: Option<i64>,
    pub page: i64,
    pub page_size: i64,
    pub next_cursor: Option<String>,
    pub mode: UsageLogsMode,
}

fn elapsed_ms(started: Instant) -> f64 {
    started.elapsed().as_secs_f64() * 1000.0
}

fn record_command_duration(state: &AppState, command_started: Instant) {
    state.record_command_duration_ms(elapsed_ms(command_started));
}

fn record_db_duration(state: &AppState, db_ms: f64) {
    state.record_db_query_duration_ms(db_ms);
}

/// 获取使用量汇总
#[tauri::command]
pub async fn get_usage_summary_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let command_started = Instant::now();
    let pool = state.db_pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
        let summary = ccr_db::database::repositories::usage_repo::get_usage_summary(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Query error: {e}"))?;
        Ok::<_, String>((summary, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (summary, db_ms) = result?;
    record_db_duration(&state, db_ms);
    serde_json::to_value(summary).map_err(|e| format!("Serialize error: {e}"))
}

/// 获取每日趋势
#[tauri::command]
pub async fn get_usage_trends_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let command_started = Instant::now();
    let pool = state.db_pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
        let trends = ccr_db::database::repositories::usage_repo::get_daily_trends(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Query error: {e}"))?;
        Ok::<_, String>((trends, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (trends, db_ms) = result?;
    record_db_duration(&state, db_ms);
    serde_json::to_value(trends).map_err(|e| format!("Serialize error: {e}"))
}

/// 获取模型统计
#[tauri::command]
pub async fn get_usage_by_model_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let command_started = Instant::now();
    let pool = state.db_pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
        let stats = ccr_db::database::repositories::usage_repo::get_model_stats(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Query error: {e}"))?;
        Ok::<_, String>((stats, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (stats, db_ms) = result?;
    record_db_duration(&state, db_ms);
    serde_json::to_value(stats).map_err(|e| format!("Serialize error: {e}"))
}

/// 获取项目统计
#[tauri::command]
pub async fn get_usage_by_project_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let command_started = Instant::now();
    let pool = state.db_pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
        let stats = ccr_db::database::repositories::usage_repo::get_project_stats(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Query error: {e}"))?;
        Ok::<_, String>((stats, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (stats, db_ms) = result?;
    record_db_duration(&state, db_ms);
    serde_json::to_value(stats).map_err(|e| format!("Serialize error: {e}"))
}

/// 获取分页日志（统一查询契约）
#[tauri::command]
pub async fn get_usage_logs_v2(
    state: State<'_, AppState>,
    query: UsageLogsQuery,
) -> Result<Value, String> {
    let command_started = Instant::now();

    let mode = query.mode.unwrap_or_default();
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(50).clamp(1, 500);
    let include_total = query
        .include_total
        .unwrap_or(matches!(mode, UsageLogsMode::Offset));

    let pool = state.db_pool.clone();
    let platform = query.platform.clone();
    let model = query.model.clone();
    let start_date = query.start_date.clone();
    let end_date = query.end_date.clone();
    let cursor = query.cursor.clone();

    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;

        let logs = match mode {
            UsageLogsMode::Cursor => {
                ccr_db::database::repositories::usage_repo::get_logs_by_cursor(
                    &conn,
                    &platform,
                    &start_date,
                    &end_date,
                    page_size,
                    &model,
                    &cursor,
                    include_total,
                )
            }
            UsageLogsMode::Offset => {
                ccr_db::database::repositories::usage_repo::get_paginated_logs(
                    &conn,
                    &platform,
                    &start_date,
                    &end_date,
                    page,
                    page_size,
                    &model,
                    include_total,
                )
            }
        }
        .map_err(|e| format!("Query error: {e}"))?;

        Ok::<_, String>((logs, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (logs, db_ms) = result?;
    record_db_duration(&state, db_ms);

    let normalized = PaginatedLogsV2 {
        records: logs.records,
        total: logs.total,
        page: if matches!(mode, UsageLogsMode::Cursor) {
            page
        } else {
            logs.page
        },
        page_size: logs.page_size,
        next_cursor: logs.next_cursor,
        mode,
    };
    serde_json::to_value(normalized).map_err(|e| format!("Serialize error: {e}"))
}

/// 获取聚合仪表板（汇总 + 趋势 + 模型 + 项目）
#[tauri::command]
pub async fn get_usage_dashboard_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let command_started = Instant::now();
    let pool = state.db_pool.clone();
    let result = tokio::task::spawn_blocking(move || {
        let db_started = Instant::now();
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;

        let summary = ccr_db::database::repositories::usage_repo::get_usage_summary(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Summary query error: {e}"))?;

        let trends = ccr_db::database::repositories::usage_repo::get_daily_trends(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Trends query error: {e}"))?;

        let by_model = ccr_db::database::repositories::usage_repo::get_model_stats(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Model stats query error: {e}"))?;

        let by_project = ccr_db::database::repositories::usage_repo::get_project_stats(
            &conn,
            &platform,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("Project stats query error: {e}"))?;

        Ok::<Value, String>(serde_json::json!({
            "summary": summary,
            "trends": trends,
            "model_stats": by_model,
            "project_stats": by_project,
            "generated_at": chrono::Utc::now().to_rfc3339(),
        }))
        .map(|payload| (payload, elapsed_ms(db_started)))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?;

    record_command_duration(&state, command_started);
    let (dashboard, db_ms) = result?;
    record_db_duration(&state, db_ms);
    Ok(dashboard)
}

/// 从 JSONL 文件导入指定平台的用量数据
#[tauri::command]
pub async fn import_usage_v2(
    _state: State<'_, AppState>,
    platform: String,
) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let service = ccr_db::services::usage_import_service::UsageImportService::new(
            ccr_db::services::usage_import_service::ImportConfig::default(),
        );
        let result = service
            .import_platform(&platform)
            .map_err(|e| format!("Import error: {e}"))?;
        serde_json::to_value(result).map_err(|e| format!("Serialize error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 导入所有平台（claude / codex / gemini）的用量数据
#[tauri::command]
pub async fn import_all_usage_v2(_state: State<'_, AppState>) -> Result<Value, String> {
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    let semaphore = Arc::new(Semaphore::new(2));
    let mut tasks = tokio::task::JoinSet::new();
    for platform in ["claude", "codex", "gemini"] {
        let sem = Arc::clone(&semaphore);
        let platform_name = platform.to_string();
        tasks.spawn(async move {
            let _permit = sem
                .acquire_owned()
                .await
                .map_err(|e| format!("Semaphore error: {e}"))?;

            tokio::task::spawn_blocking(move || {
                let service = ccr_db::services::usage_import_service::UsageImportService::new(
                    ccr_db::services::usage_import_service::ImportConfig::default(),
                );
                service.import_platform(&platform_name)
            })
            .await
            .map_err(|e| format!("Task join error: {e}"))?
            .map(|result| serde_json::to_value(result).unwrap_or(Value::Null))
            .map_err(|e| e.to_string())
        });
    }

    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(value)) => results.push(value),
            Ok(Err(e)) => {
                results.push(serde_json::json!({ "error": e }));
            }
            Err(e) => {
                results.push(serde_json::json!({ "error": format!("Join error: {e}") }));
            }
        }
    }

    Ok(serde_json::json!({ "results": results }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_logs_query_supports_camel_case_aliases() {
        let value = serde_json::json!({
            "platform": "codex",
            "model": "gpt-5",
            "startDate": "2026-03-01",
            "endDate": "2026-03-05",
            "pageSize": 20,
            "includeTotal": true,
            "mode": "offset"
        });

        let query: UsageLogsQuery =
            serde_json::from_value(value).expect("query should deserialize");
        assert_eq!(query.start_date.as_deref(), Some("2026-03-01"));
        assert_eq!(query.end_date.as_deref(), Some("2026-03-05"));
        assert_eq!(query.page_size, Some(20));
        assert_eq!(query.include_total, Some(true));
        assert_eq!(query.mode, Some(UsageLogsMode::Offset));
    }

    #[test]
    fn usage_logs_mode_defaults_to_cursor() {
        let query = UsageLogsQuery {
            platform: None,
            model: None,
            start_date: None,
            end_date: None,
            page: None,
            page_size: None,
            cursor: None,
            include_total: None,
            mode: None,
        };

        assert_eq!(query.mode.unwrap_or_default(), UsageLogsMode::Cursor);
    }
}
