use super::*;

/// 列出所有 Claude Code Profiles（~/.ccr/platforms/claude/profiles.toml）。
#[tauri::command]
pub async fn claude_list_profiles() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Claude profile 失败: {e}"))?;
        let profiles: Vec<Value> = platform
            .load_profiles()
            .map_err(|e| format!("读取 Claude profiles 失败: {e}"))?
            .into_iter()
            .map(|(name, profile)| profile_to_json(current_profile.as_deref(), name, profile))
            .collect();

        Ok(json!({
            "profiles": profiles,
            "current_profile": current_profile,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 获取单个 Profile 详情。
#[tauri::command]
pub async fn claude_get_profile(name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Claude profile 失败: {e}"))?;
        let profile = platform
            .load_profiles()
            .map_err(|e| format!("读取 Claude profiles 失败: {e}"))?
            .shift_remove(&name)
            .ok_or_else(|| format!("Claude Profile '{name}' 不存在"))?;

        Ok(profile_to_json(current_profile.as_deref(), name, profile))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 创建新 Profile。
#[tauri::command]
pub async fn claude_add_profile(request: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let name = request
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| "Missing 'name' field".to_string())?
            .to_string();

        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 Claude profiles 失败: {e}"))?;
        if profiles.contains_key(&name) {
            return Err(format!("Claude Profile '{name}' 已存在"));
        }

        let profile = build_profile_from_config(&request)?;

        platform
            .save_profile(&name, &profile)
            .map_err(|e| format!("保存 Claude Profile 失败: {e}"))?;

        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Claude profile 失败: {e}"))?;

        Ok(profile_to_json(current_profile.as_deref(), name, profile))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 更新 Profile。
#[tauri::command]
pub async fn claude_update_profile(name: String, request: Value) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        let profiles = platform
            .load_profiles()
            .map_err(|e| format!("读取 Claude profiles 失败: {e}"))?;
        let current_profile = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Claude profile 失败: {e}"))?;
        let existing = profiles
            .get(&name)
            .cloned()
            .ok_or_else(|| format!("Claude Profile '{name}' 不存在"))?;

        let target_name = request
            .get("name")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .unwrap_or(name.as_str())
            .to_string();
        if target_name != name && profiles.contains_key(&target_name) {
            return Err(format!("Claude Profile '{target_name}' 已存在"));
        }

        let mut profile = existing;
        patch_profile_with_config(&mut profile, &request)?;

        platform
            .save_profile(&target_name, &profile)
            .map_err(|e| format!("更新 Claude Profile 失败: {e}"))?;

        if target_name != name {
            platform
                .delete_profile(&name)
                .map_err(|e| format!("删除旧 Claude Profile 失败: {e}"))?;

            if current_profile.as_deref() == Some(name.as_str()) {
                platform
                    .apply_profile(&target_name)
                    .map_err(|e| format!("同步当前 Claude Profile 失败: {e}"))?;
            }
        }

        let latest_current = platform
            .get_current_profile()
            .map_err(|e| format!("读取当前 Claude profile 失败: {e}"))?;

        Ok(profile_to_json(
            latest_current.as_deref(),
            target_name,
            profile,
        ))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 删除 Profile。
#[tauri::command]
pub async fn claude_delete_profile(name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        platform
            .delete_profile(&name)
            .map_err(|e| format!("删除 Claude Profile 失败: {e}"))?;
        Ok(json!({ "message": format!("Claude Profile '{name}' 已删除") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 应用 Profile。
#[tauri::command]
pub async fn claude_apply_profile(name: String) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let platform = ClaudePlatform::new().map_err(|e| format!("初始化 Claude 平台失败: {e}"))?;
        platform
            .apply_profile(&name)
            .map_err(|e| format!("应用 Claude Profile 失败: {e}"))?;
        Ok(json!({
            "success": true,
            "applied_profile": name,
            "message": format!("Claude Profile 已应用"),
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}
