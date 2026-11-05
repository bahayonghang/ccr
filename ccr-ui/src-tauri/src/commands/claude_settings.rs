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
        let dir = output_styles_dir()
            .map_err(|e| format!("Cannot determine output-styles dir: {}", e))?;

        if !dir.exists() {
            return Ok(serde_json::json!({ "styles": [] }));
        }

        let entries =
            fs::read_dir(&dir).map_err(|e| format!("Failed to read output-styles dir: {}", e))?;

        let mut styles: Vec<OutputStyle> = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let Ok(content) = fs::read_to_string(&path) else {
                continue;
            };
            styles.push(OutputStyle {
                name: stem.to_string(),
                content,
            });
        }

        styles.sort_by(|a, b| a.name.cmp(&b.name));
        let result =
            serde_json::to_value(&styles).map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "styles": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??
    .try_into()
}

/// 写入或覆盖一个 output style 文件（styles 是 [{name, content}] 数组）。
#[ccr_tauri_command_macros::command]
pub async fn claude_update_output_styles(
    styles: OpenJsonValueDto,
) -> Result<OpenJsonValueDto, String> {
    let styles: Value = styles.into();
    tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let dir = output_styles_dir()
            .map_err(|e| format!("Cannot determine output-styles dir: {}", e))?;

        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create output-styles dir: {}", e))?;

        let items: Vec<OutputStyle> = serde_json::from_value(styles)
            .map_err(|e| format!("Invalid output styles payload: {}", e))?;

        for item in &items {
            let path = dir.join(format!("{}.md", item.name));
            fs::write(&path, &item.content)
                .map_err(|e| format!("Failed to write style '{}': {}", item.name, e))?;
        }

        let result =
            serde_json::to_value(&items).map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "styles": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??
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
        let manager = PromptsManager::new(Platform::Claude)
            .map_err(|e| format!("PromptsManager init error: {}", e))?;

        if let Some(arr) = prompts.as_array() {
            let existing = manager
                .list_presets()
                .map_err(|e| format!("Failed to list presets: {}", e))?;
            for preset in &existing {
                let _ = manager.remove_preset(&preset.name);
            }
            for item in arr {
                let preset: PromptPreset = serde_json::from_value(item.clone())
                    .map_err(|e| format!("Invalid preset item: {}", e))?;
                manager
                    .add_preset(preset)
                    .map_err(|e| format!("Failed to add preset: {}", e))?;
            }
        } else {
            let preset: PromptPreset = serde_json::from_value(prompts)
                .map_err(|e| format!("Invalid preset payload: {}", e))?;
            let _ = manager.remove_preset(&preset.name);
            manager
                .add_preset(preset)
                .map_err(|e| format!("Failed to add preset: {}", e))?;
        }

        let updated = manager
            .list_presets()
            .map_err(|e| format!("Failed to list presets: {}", e))?;
        let result =
            serde_json::to_value(&updated).map_err(|e| format!("Serialization error: {}", e))?;
        Ok(serde_json::json!({ "presets": result }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??
    .try_into()
}
