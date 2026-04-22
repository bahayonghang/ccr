use super::*;

#[tauri::command]
pub async fn claude_list_auth_accounts() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        let snapshot = service
            .read_auth_snapshot()
            .map_err(|e| format!("读取认证快照失败: {e}"))?;
        let runtime_summary = service
            .get_runtime_summary()
            .map_err(|e| format!("读取运行时摘要失败: {e}"))?;
        let accounts = service
            .build_account_items(&snapshot, Some(&runtime_summary))
            .map_err(|e| format!("列出账号失败: {e}"))?;

        let accounts: Vec<Value> = accounts
            .into_iter()
            .map(|item| {
                json!({
                    "name": item.name,
                    "description": item.description,
                    "email": item.email,
                    "billing_type": item.billing_type,
                    "subscription_type": item.subscription_type,
                    "rate_limit_tier": item.rate_limit_tier,
                    "is_current": item.is_current,
                    "is_logged_in": item.is_logged_in,
                    "saved_at": item.saved_at.to_rfc3339(),
                    "last_used": item.last_used.map(|dt| dt.to_rfc3339()),
                    "expires_at": item.expires_at.map(|dt| dt.to_rfc3339()),
                })
            })
            .collect();

        Ok(json!({
            "accounts": accounts,
            "login_state": snapshot.login_state,
            "runtime_summary": runtime_summary,
            "current_profile_auth_mode": runtime_summary.current_profile_auth_mode.map(|mode| mode.as_str()),
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn claude_get_auth_current() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        let runtime_summary = service
            .get_runtime_summary()
            .map_err(|e| format!("读取运行时摘要失败: {e}"))?;
        let current_info = service.get_current_auth_info().ok();
        let logged_in = current_info.is_some();

        let info = current_info.map(|info| {
            json!({
                "account_uuid": info.account_uuid,
                "email": info.email,
                "billing_type": info.billing_type,
                "subscription_type": info.subscription_type,
                "rate_limit_tier": info.rate_limit_tier,
                "expires_at": info.expires_at.map(|dt| dt.to_rfc3339()),
            })
        });

        Ok(json!({
            "logged_in": logged_in,
            "info": info,
            "runtime_summary": runtime_summary,
            "login_state": runtime_summary.login_state,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn claude_save_auth(
    name: String,
    description: Option<String>,
    force: Option<bool>,
) -> Result<Value, String> {
    tokio::task::spawn_blocking(move || {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        service
            .save_current(&name, description, force.unwrap_or(false))
            .map_err(|e| format!("{e}"))?;
        Ok(json!({
            "success": true,
            "message": format!("Claude 官方账号 '{}' 已成功保存", name),
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

#[tauri::command]
pub async fn claude_switch_auth(name: String) -> Result<Value, String> {
    let name_resp = name.clone();
    tokio::task::spawn_blocking(move || {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        service.switch_account(&name).map_err(|e| format!("{e}"))?;
        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    Ok(json!({
        "success": true,
        "message": format!("已切换到 Claude 官方账号 '{}'", name_resp),
    }))
}

#[tauri::command]
pub async fn claude_delete_auth(name: String) -> Result<Value, String> {
    let name_resp = name.clone();
    tokio::task::spawn_blocking(move || {
        let service =
            ClaudeAuthService::new().map_err(|e| format!("初始化 Claude Auth 服务失败: {e}"))?;
        service.delete_account(&name).map_err(|e| format!("{e}"))?;
        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    Ok(json!({
        "success": true,
        "message": format!("Claude 官方账号 '{}' 已成功删除", name_resp),
    }))
}
