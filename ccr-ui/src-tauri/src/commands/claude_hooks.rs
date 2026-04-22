use super::*;

#[tauri::command]
pub async fn claude_list_hooks(state: State<'_, AppState>) -> Result<Value, String> {
    let settings = load_settings(state.inner()).await?;
    let hooks =
        serde_json::to_value(&settings.hooks).map_err(|e| format!("Serialization error: {}", e))?;
    Ok(serde_json::json!({ "hooks": hooks }))
}

/// 整体替换 hooks 配置（官方 grouped hooks 对象）。
#[tauri::command]
pub async fn claude_update_hooks(
    state: State<'_, AppState>,
    hooks: Value,
) -> Result<Value, String> {
    let mut settings = load_settings(state.inner()).await?;

    let new_hooks: ccr_types::HooksConfig =
        serde_json::from_value(hooks).map_err(|e| format!("Invalid hooks payload: {}", e))?;
    settings.hooks = new_hooks;

    save_settings(state.inner(), &settings).await?;

    let result =
        serde_json::to_value(&settings.hooks).map_err(|e| format!("Serialization error: {}", e))?;
    Ok(serde_json::json!({ "hooks": result }))
}
