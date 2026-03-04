//! Usage V2 命令 — SQLite 用量统计查询与导入。

use serde_json::Value;
use tauri::State;

use crate::state::AppState;

/// 获取使用量汇总
#[tauri::command]
pub async fn get_usage_summary_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
        let summary =
            ccr_db::database::repositories::usage_repo::get_usage_summary(&conn, &platform, &start_date, &end_date)
                .map_err(|e| format!("Query error: {e}"))?;
        serde_json::to_value(summary).map_err(|e| format!("Serialize error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 获取每日趋势
#[tauri::command]
pub async fn get_usage_trends_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
        let trends =
            ccr_db::database::repositories::usage_repo::get_daily_trends(&conn, &platform, &start_date, &end_date)
                .map_err(|e| format!("Query error: {e}"))?;
        serde_json::to_value(trends).map_err(|e| format!("Serialize error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 获取模型统计
#[tauri::command]
pub async fn get_usage_by_model_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
        let stats =
            ccr_db::database::repositories::usage_repo::get_model_stats(&conn, &platform, &start_date, &end_date)
                .map_err(|e| format!("Query error: {e}"))?;
        serde_json::to_value(stats).map_err(|e| format!("Serialize error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 获取项目统计
#[tauri::command]
pub async fn get_usage_by_project_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;
        let stats =
            ccr_db::database::repositories::usage_repo::get_project_stats(&conn, &platform, &start_date, &end_date)
                .map_err(|e| format!("Query error: {e}"))?;
        serde_json::to_value(stats).map_err(|e| format!("Serialize error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 获取分页日志
#[tauri::command]
pub async fn get_usage_logs_v2(
    state: State<'_, AppState>,
    page: Option<i64>,
    page_size: Option<i64>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let pool = state.db_pool.clone();
    let page = page.unwrap_or(1).max(1);
    let page_size = page_size.unwrap_or(50).clamp(1, 500);

    tokio::task::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;

        // 如果有日期过滤，先做一次基于 recorded_at 的筛选；
        // get_paginated_logs 仅支持 platform + model 过滤，
        // 对日期范围我们用 get_logs_by_cursor 的底层参数或直接构造。
        // 此处使用 get_paginated_logs（platform 过滤），日期过滤通过前端传递。
        // 注意：get_paginated_logs 不支持 start_date/end_date，
        // 若需要日期过滤，使用 get_usage_summary 结合分页数据即可。
        // 实现方式：调用 get_paginated_logs 并在结果中附上日期范围元信息。
        let _ = (start_date, end_date); // 当前版本不做服务端日期过滤，由前端处理

        let logs = ccr_db::database::repositories::usage_repo::get_paginated_logs(
            &conn,
            &platform,
            page,
            page_size,
            &None, // model_filter
            true,  // include_total
        )
        .map_err(|e| format!("Query error: {e}"))?;
        serde_json::to_value(logs).map_err(|e| format!("Serialize error: {e}"))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

/// 获取聚合仪表板（汇总 + 趋势 + 模型 + 项目）
#[tauri::command]
pub async fn get_usage_dashboard_v2(
    state: State<'_, AppState>,
    platform: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Value, String> {
    let pool = state.db_pool.clone();
    tokio::task::spawn_blocking(move || {
        let conn = pool.get().map_err(|e| format!("DB error: {e}"))?;

        let summary =
            ccr_db::database::repositories::usage_repo::get_usage_summary(&conn, &platform, &start_date, &end_date)
                .map_err(|e| format!("Summary query error: {e}"))?;

        let trends =
            ccr_db::database::repositories::usage_repo::get_daily_trends(&conn, &platform, &start_date, &end_date)
                .map_err(|e| format!("Trends query error: {e}"))?;

        let by_model =
            ccr_db::database::repositories::usage_repo::get_model_stats(&conn, &platform, &start_date, &end_date)
                .map_err(|e| format!("Model stats query error: {e}"))?;

        let by_project =
            ccr_db::database::repositories::usage_repo::get_project_stats(&conn, &platform, &start_date, &end_date)
                .map_err(|e| format!("Project stats query error: {e}"))?;

        Ok::<Value, String>(serde_json::json!({
            "summary": summary,
            "trends": trends,
            "model_stats": by_model,
            "project_stats": by_project,
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
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
    tokio::task::spawn_blocking(move || {
        let service = ccr_db::services::usage_import_service::UsageImportService::new(
            ccr_db::services::usage_import_service::ImportConfig::default(),
        );

        let platforms = ["claude", "codex", "gemini"];
        let mut results = Vec::new();

        for platform in platforms {
            match service.import_platform(platform) {
                Ok(result) => {
                    results.push(serde_json::to_value(result).unwrap_or(Value::Null));
                }
                Err(e) => {
                    results.push(serde_json::json!({
                        "platform": platform,
                        "error": e,
                    }));
                }
            }
        }

        Ok::<Value, String>(serde_json::json!({ "results": results }))
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}
