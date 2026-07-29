use super::*;

#[ccr_tauri_command_macros::command]
pub async fn claude_list_hooks(state: State<'_, AppState>) -> Result<OpenJsonValueDto, String> {
    let settings = load_settings(state.inner()).await?;
    let hooks =
        serde_json::to_value(&settings.hooks).map_err(|e| format!("Serialization error: {}", e))?;
    open_json(serde_json::json!({ "hooks": hooks }))
}

/// 整体替换 hooks 配置（官方 grouped hooks 对象）。
#[ccr_tauri_command_macros::command]
pub async fn claude_update_hooks(
    state: State<'_, AppState>,
    hooks: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let new_hooks: ccr_types::HooksConfig = serde_json::from_value(hooks.into())
        .map_err(|e| format!("Invalid hooks payload: {}", e))?;
    let result = update_settings(state.inner(), move |settings| {
        settings.hooks = new_hooks.clone();
        serde_json::to_value(&settings.hooks)
            .map_err(|error| format!("Serialization error: {error}"))
    })
    .await?;
    open_json(serde_json::json!({ "hooks": result }))
}
