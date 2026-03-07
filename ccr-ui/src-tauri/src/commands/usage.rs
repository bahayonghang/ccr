//! Usage V2 閸涙垝鎶?閳?SQLite 閻劑鍣虹紒鐔活吀閺屻儴顕楁稉搴☆嚤閸忋儯鈧?

use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::events::{self, UsageImportPayload};
use crate::monitoring::{emit_and_record_monitoring_event, should_persist, usage_import_entry};
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

/// 閼惧嘲褰囨担璺ㄦ暏闁插繑鐪归幀?
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

/// 閼惧嘲褰囧В蹇旀）鐡掑濞?
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

/// 閼惧嘲褰囧Ο鈥崇€风紒鐔活吀
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

/// 閼惧嘲褰囨い鍦窗缂佺喕顓?
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

/// 閼惧嘲褰囬崚鍡涖€夐弮銉ョ箶閿涘牏绮烘稉鈧弻銉嚄婵傛垹瀹抽敍?
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

/// 閼惧嘲褰囬懕姘値娴狀亣銆冮弶鍖＄礄濮瑰洦鈧?+ 鐡掑濞?+ 濡€崇€?+ 妞ゅ湱娲伴敍?
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

/// 娴?JSONL 閺傚洣娆㈢€电厧鍙嗛幐鍥х暰楠炲啿褰撮惃鍕暏闁插繑鏆熼幑?
#[tauri::command]
pub async fn import_usage_v2(
    app_handle: tauri::AppHandle,
    _state: State<'_, AppState>,
    platform: String,
) -> Result<Value, String> {
    let result = tokio::task::spawn_blocking(move || {
        let service = ccr_db::services::usage_import_service::UsageImportService::new(
            ccr_db::services::usage_import_service::ImportConfig::default(),
        );
        service
            .import_platform(&platform)
            .map_err(|e| format!("Import error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))??;

    let payload = UsageImportPayload {
        imported_count: result.records_imported,
        platform: result.platform.clone(),
    };
    let entry = usage_import_entry(&payload);
    let persist = should_persist(entry.level, &entry.event_type);
    emit_and_record_monitoring_event(
        &app_handle,
        events::channels::USAGE_IMPORT,
        &payload,
        entry,
        persist,
    )
    .await;

    serde_json::to_value(result).map_err(|e| format!("Serialize error: {e}"))
}
#[tauri::command]
pub async fn import_all_usage_v2(
    app_handle: tauri::AppHandle,
    _state: State<'_, AppState>,
) -> Result<Value, String> {
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
            .map_err(|e| e.to_string())
        });
    }

    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok(import_result)) => {
                let payload = UsageImportPayload {
                    imported_count: import_result.records_imported,
                    platform: import_result.platform.clone(),
                };
                let entry = usage_import_entry(&payload);
                let persist = should_persist(entry.level, &entry.event_type);
                emit_and_record_monitoring_event(
                    &app_handle,
                    events::channels::USAGE_IMPORT,
                    &payload,
                    entry,
                    persist,
                )
                .await;

                results.push(serde_json::to_value(import_result).unwrap_or(Value::Null));
            }
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
