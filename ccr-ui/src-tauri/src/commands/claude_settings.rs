use super::*;

/// 读取入场环境的 Claude settings；可选环境 ID 不匹配时在 I/O 前拒绝。
#[ccr_tauri_command_macros::command]
pub async fn claude_get_settings(
    state: State<'_, AppState>,
    expected_environment_id: Option<String>,
) -> Result<OpenJsonValueDto, String> {
    open_json(
        read_settings_from_registry(&state.env_registry, expected_environment_id.as_deref())
            .await?,
    )
}

/// 在同一个入场环境中读取并合并写入 Claude settings。
#[ccr_tauri_command_macros::command]
pub async fn claude_update_settings(
    state: State<'_, AppState>,
    settings: OpenJsonValueDto,
    expected_environment_id: Option<String>,
) -> Result<OpenJsonValueDto, String> {
    let patch: Value = settings.into();
    let result = update_settings_from_registry(
        &state.env_registry,
        expected_environment_id.as_deref(),
        move |settings| {
            let mut current = serde_json::to_value(&*settings)
                .map_err(|error| format!("Serialization error: {error}"))?;
            merge_settings_patch(&mut current, patch.clone())?;
            let validated = serde_json::from_value(current)
                .map_err(|error| format!("Invalid settings payload: {error}"))?;
            *settings = validated;
            serde_json::to_value(&*settings)
                .map_err(|error| format!("Serialization error: {error}"))
        },
    )
    .await?;
    open_json(result)
}

#[ccr_tauri_command_macros::command]
pub async fn claude_get_output_styles() -> Result<OpenJsonValueDto, String> {
    tokio::task::spawn_blocking(|| -> Result<Value, String> {
        let styles = ccr_app::resources::claude_files::list_output_styles(&output_styles_store()?)
            .map_err(|error| error.to_string())?;
        Ok(serde_json::json!({ "styles": styles }))
    })
    .await
    .map_err(|error| format!("Task join error: {error}"))??
    .try_into()
}

/// Batch upsert preserves the existing array payload; omitted styles are retained.
#[ccr_tauri_command_macros::command]
pub async fn claude_update_output_styles(
    styles: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let styles: Value = styles.into();
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let items: Vec<ccr_app::resources::claude_files::OutputStyle> =
            serde_json::from_value(styles)
                .map_err(|error| format!("Invalid output styles payload: {error}"))?;
        ccr_app::resources::claude_files::save_output_styles(&output_styles_store()?, &items)
            .map_err(|error| error.to_string())?;
        Ok(serde_json::json!({ "styles": items }))
    })
    .await
    .map_err(|error| format!("Task join error: {error}"))??
    .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn claude_get_statusline(state: State<'_, AppState>) -> Result<OpenJsonValueDto, String> {
    let settings = load_settings(state.inner()).await?;
    let statusline = settings
        .other
        .get("statusline")
        .cloned()
        .unwrap_or(Value::Null);
    open_json(statusline)
}

#[ccr_tauri_command_macros::command]
pub async fn claude_update_statusline(
    state: State<'_, AppState>,
    config: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let config: Value = config.into();
    let settings_value = config.clone();
    update_settings(state.inner(), move |settings| {
        settings
            .other
            .insert("statusline".to_string(), settings_value.clone());
        Ok(())
    })
    .await?;
    open_json(config)
}

#[ccr_tauri_command_macros::command]
pub async fn claude_get_budgets() -> Result<OpenJsonValueDto, String> {
    tokio::task::spawn_blocking(|| -> Result<Value, String> {
        let config_path = BudgetManager::default_config_path()
            .map_err(|e| format!("BudgetManager init error: {}", e))?;
        let storage_dir = CostTracker::default_storage_dir()
            .map_err(|e| format!("CostTracker storage dir error: {}", e))?;
        ccr_app::usage::policies::read_budget(config_path, storage_dir)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??
    .try_into()
}

/// 更新预算配置。budgets 可包含字段：enabled, dailyLimit, weeklyLimit, monthlyLimit,
/// warnAtPercent。
#[ccr_tauri_command_macros::command]
pub async fn claude_update_budgets(budgets: OpenJsonValueDto) -> Result<OpenJsonValueDto, String> {
    let budgets: Value = budgets.into();
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let config_path = BudgetManager::default_config_path()
            .map_err(|e| format!("BudgetManager init error: {}", e))?;
        ccr_app::usage::policies::update_budget(config_path, budgets)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??
    .try_into()
}

#[ccr_tauri_command_macros::command]
pub async fn claude_list_prompts() -> Result<OpenJsonValueDto, String> {
    tokio::task::spawn_blocking(|| -> Result<Value, String> {
        let manager = PromptsManager::new(Platform::Claude)
            .map_err(|e| format!("PromptsManager init error: {}", e))?;

        let presets = manager
            .list_presets()
            .map_err(|e| format!("Failed to list presets: {}", e))?;

        let result =
            serde_json::to_value(&presets).map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "presets": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??
    .try_into()
}

/// 整体替换 prompt presets 列表，或添加/更新单个 preset。
/// prompts 可以是 PromptPreset 数组，或单个 PromptPreset 对象（则执行 add/update）。
#[ccr_tauri_command_macros::command]
pub async fn claude_update_prompts(prompts: OpenJsonValueDto) -> Result<OpenJsonValueDto, String> {
    let prompts: Value = prompts.into();
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let home =
            dirs::home_dir().ok_or_else(|| "Unable to determine home directory".to_string())?;
        let paths = PlatformPaths::new(Platform::Claude).map_err(|error| error.to_string())?;
        let locks =
            ccr_core::core::LockManager::with_default_path().map_err(|error| error.to_string())?;
        let catalog = ccr_app::resources::prompt_presets::Catalog::read(
            Platform::Claude,
            &home,
            &paths.backups_dir,
            locks.lock_dir(),
        )
        .map_err(|error| error.to_string())?;
        let mutation = if prompts.is_array() {
            ccr_app::resources::prompt_presets::Mutation::Replace(
                serde_json::from_value(prompts)
                    .map_err(|error| format!("Invalid preset list: {error}"))?,
            )
        } else {
            ccr_app::resources::prompt_presets::Mutation::Upsert(
                serde_json::from_value(prompts)
                    .map_err(|error| format!("Invalid preset payload: {error}"))?,
            )
        };
        catalog
            .commit(mutation)
            .map_err(|error| error.to_string())?;
        let updated = PromptsManager::for_home(Platform::Claude, &home)
            .list_presets()
            .map_err(|error| format!("Failed to list presets: {error}"))?;
        let result =
            serde_json::to_value(&updated).map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "presets": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??
    .try_into()
}
