use super::*;

/// 获取 Codex 完整配置（去掉 mcp_servers 和 profiles）
#[ccr_tauri_command_macros::command]
pub async fn codex_get_settings() -> Result<OpenJsonValueDto, String> {
    tokio::task::spawn_blocking(|| -> Result<Value, String> {
        let path = codex_config_path()?;
        let config = read_codex_config(&path)?;
        Ok(codex_settings_to_json(&config))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??
    .try_into()
}

/// 更新 Codex 配置（合并写入，不覆盖 mcp_servers/profiles）
#[ccr_tauri_command_macros::command]
pub async fn codex_update_settings(
    state: State<'_, AppState>,
    settings: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let settings: Value = settings.into();
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let path = codex_config_path()?;
        let mut config = read_codex_config(&path)?;
        apply_codex_settings_update(&mut config, &settings)?;

        write_codex_config(&path, &config)?;
        Ok(json!({ "message": "Codex 配置已更新" }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    open_json(response)
}
