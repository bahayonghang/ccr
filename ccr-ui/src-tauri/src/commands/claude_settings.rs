use super::*;

/// 读取 ~/.claude/settings.json，以 JSON Value 返回完整内容。
#[tauri::command]
pub async fn claude_get_settings(state: State<'_, AppState>) -> Result<Value, String> {
    read_active_claude_settings_raw(state.inner()).await
}

/// 将调用方提供的 JSON 合并写入 ~/.claude/settings.json。
#[tauri::command]
pub async fn claude_update_settings(
    state: State<'_, AppState>,
    settings: Value,
) -> Result<Value, String> {
    let mut current = read_active_claude_settings_raw(state.inner()).await?;
    merge_settings_patch(&mut current, settings)?;

    let validated: ccr_types::ClaudeSettings =
        serde_json::from_value(current).map_err(|e| format!("Invalid settings payload: {e}"))?;
    let result =
        serde_json::to_value(&validated).map_err(|e| format!("Serialization error: {e}"))?;

    write_active_claude_settings_raw(state.inner(), &result).await?;
    Ok(result)
}

#[tauri::command]
pub async fn claude_get_output_styles() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
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
    .map_err(|e| format!("Task join error: {}", e))?
}

/// 写入或覆盖一个 output style 文件（styles 是 [{name, content}] 数组）。
#[tauri::command]
pub async fn claude_update_output_styles(styles: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
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
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_get_statusline(state: State<'_, AppState>) -> Result<Value, String> {
    let settings = load_settings(state.inner()).await?;
    let statusline = settings
        .other
        .get("statusline")
        .cloned()
        .unwrap_or(Value::Null);
    Ok(statusline)
}

#[tauri::command]
pub async fn claude_update_statusline(
    state: State<'_, AppState>,
    config: Value,
) -> Result<Value, String> {
    let mut settings = load_settings(state.inner()).await?;
    settings
        .other
        .insert("statusline".to_string(), config.clone());
    save_settings(state.inner(), &settings).await?;
    Ok(config)
}

#[tauri::command]
pub async fn claude_get_budgets() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let budget_manager = BudgetManager::with_default()
            .map_err(|e| format!("BudgetManager init error: {}", e))?;

        let storage_dir = CostTracker::default_storage_dir()
            .map_err(|e| format!("CostTracker storage dir error: {}", e))?;
        let tracker =
            CostTracker::new(storage_dir).map_err(|e| format!("CostTracker init error: {}", e))?;

        let status = budget_manager
            .check_status(&tracker)
            .map_err(|e| format!("Budget status error: {}", e))?;
        let config = budget_manager.get_config();

        Ok(serde_json::json!({
            "enabled": status.enabled,
            "dailyLimit": config.daily_limit,
            "weeklyLimit": config.weekly_limit,
            "monthlyLimit": config.monthly_limit,
            "warnAtPercent": config.warn_at_percent,
            "currentCosts": {
                "today": status.current_costs.today,
                "thisWeek": status.current_costs.this_week,
                "thisMonth": status.current_costs.this_month,
            },
            "warnings": status.warnings.iter().map(|w| serde_json::json!({
                "period": w.period.to_string(),
                "currentCost": w.current_cost,
                "limit": w.limit,
                "usagePercent": w.usage_percent,
            })).collect::<Vec<_>>(),
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

/// 更新预算配置。budgets 可包含字段：enabled, dailyLimit, weeklyLimit, monthlyLimit,
/// warnAtPercent。
#[tauri::command]
pub async fn claude_update_budgets(budgets: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let mut budget_manager = BudgetManager::with_default()
            .map_err(|e| format!("BudgetManager init error: {}", e))?;

        if let Some(enabled) = budgets.get("enabled").and_then(|v| v.as_bool()) {
            if enabled {
                budget_manager
                    .enable()
                    .map_err(|e| format!("Failed to enable budget: {}", e))?;
            } else {
                budget_manager
                    .disable()
                    .map_err(|e| format!("Failed to disable budget: {}", e))?;
            }
        }

        if let Some(daily_val) = budgets.get("dailyLimit") {
            let limit = if daily_val.is_null() {
                None
            } else {
                daily_val
                    .as_f64()
                    .map(Some)
                    .ok_or_else(|| "dailyLimit must be a number or null".to_string())?
            };
            budget_manager
                .set_daily_limit(limit)
                .map_err(|e| format!("Failed to set daily limit: {}", e))?;
        }

        if let Some(weekly_val) = budgets.get("weeklyLimit") {
            let limit = if weekly_val.is_null() {
                None
            } else {
                weekly_val
                    .as_f64()
                    .map(Some)
                    .ok_or_else(|| "weeklyLimit must be a number or null".to_string())?
            };
            budget_manager
                .set_weekly_limit(limit)
                .map_err(|e| format!("Failed to set weekly limit: {}", e))?;
        }

        if let Some(monthly_val) = budgets.get("monthlyLimit") {
            let limit = if monthly_val.is_null() {
                None
            } else {
                monthly_val
                    .as_f64()
                    .map(Some)
                    .ok_or_else(|| "monthlyLimit must be a number or null".to_string())?
            };
            budget_manager
                .set_monthly_limit(limit)
                .map_err(|e| format!("Failed to set monthly limit: {}", e))?;
        }

        if let Some(pct) = budgets
            .get("warnAtPercent")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8)
        {
            budget_manager
                .set_warn_threshold(pct)
                .map_err(|e| format!("Failed to set warn threshold: {}", e))?;
        }

        let config = budget_manager.get_config();
        Ok(serde_json::json!({
            "enabled": config.enabled,
            "dailyLimit": config.daily_limit,
            "weeklyLimit": config.weekly_limit,
            "monthlyLimit": config.monthly_limit,
            "warnAtPercent": config.warn_at_percent,
        }))
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
pub async fn claude_list_prompts() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
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
    .map_err(|e| format!("Task join error: {}", e))?
}

/// 整体替换 prompt presets 列表，或添加/更新单个 preset。
/// prompts 可以是 PromptPreset 数组，或单个 PromptPreset 对象（则执行 add/update）。
#[tauri::command]
pub async fn claude_update_prompts(prompts: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
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
    .map_err(|e| format!("Task join error: {}", e))?
}
