use super::*;

#[tauri::command]
pub async fn codex_list_sessions(
    limit: Option<usize>,
    query: Option<String>,
) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let codex_dir = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?
            .join(".codex");
        let service = CodexSessionService::new(codex_dir);
        let sessions = service
            .list_sessions(limit.unwrap_or(120).max(1), query.as_deref())
            .map_err(|e| format!("读取 Codex sessions 失败: {e}"))?;
        Ok(json!({ "sessions": sessions }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn codex_get_session_detail(
    file_path: String,
    message_limit: Option<usize>,
) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let codex_dir = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?
            .join(".codex");
        let service = CodexSessionService::new(codex_dir);
        let detail = service
            .get_session_detail(PathBuf::from(file_path).as_path(), message_limit)
            .map_err(|e| format!("读取 Codex session 详情失败: {e}"))?;
        Ok(json!(detail))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn codex_export_session(
    file_path: String,
    max_messages: Option<usize>,
) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let codex_dir = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?
            .join(".codex");
        let service = CodexSessionService::new(codex_dir);
        let export = service
            .export_session_markdown(PathBuf::from(file_path).as_path(), max_messages)
            .map_err(|e| format!("导出 Codex session 失败: {e}"))?;
        Ok(json!(export))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn codex_clone_session(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<Value, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let codex_dir = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?
            .join(".codex");
        let service = CodexSessionService::new(codex_dir);
        let session = service
            .clone_session(PathBuf::from(file_path).as_path())
            .map_err(|e| format!("克隆 Codex session 失败: {e}"))?;
        Ok(json!({
            "message": "Codex session 已克隆",
            "session": session,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_session_inventory_cache()?;
    invalidate_codex_dashboard_overview_cache(&state).await;
    invalidate_codex_usage_cache(&state).await;
    Ok(response)
}

#[tauri::command]
pub async fn codex_delete_session(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<Value, String> {
    let target_file_path = file_path.clone();
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let codex_dir = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?
            .join(".codex");
        let service = CodexSessionService::new(codex_dir);
        let target_path = PathBuf::from(&file_path);
        service
            .delete_session(target_path.as_path())
            .map_err(|e| format!("删除 Codex session 失败: {e}"))?;
        Ok(json!({
            "message": "Codex session 已删除",
            "file_path": file_path,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    let conn = state
        .usage_db_pool
        .get()
        .map_err(|e| format!("读取 usage archive 数据库失败: {e}"))?;
    ccr_db::database::repositories::usage_repo::mark_source_deleted_by_path(
        &conn,
        "codex",
        &target_file_path,
    )
    .map_err(|e| format!("更新 usage source 归档状态失败: {e}"))?;
    ccr_db::database::repositories::usage_repo::mark_session_archive_deleted_by_path(
        &conn,
        "codex",
        &target_file_path,
    )
    .map_err(|e| format!("更新 session 摘要归档状态失败: {e}"))?;

    invalidate_codex_session_inventory_cache()?;
    invalidate_codex_dashboard_overview_cache(&state).await;
    invalidate_codex_usage_cache(&state).await;
    Ok(response)
}
