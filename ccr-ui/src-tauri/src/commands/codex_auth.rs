use super::*;

/// 列出所有 Codex Auth 账号
#[tauri::command]
pub async fn codex_list_auth_accounts() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        let snapshot = service
            .read_auth_snapshot()
            .map_err(|e| format!("读取认证快照失败: {e}"))?;
        let accounts = service
            .build_account_items(&snapshot)
            .map_err(|e| format!("列出账号失败: {e}"))?;

        let accounts: Vec<Value> = accounts
            .into_iter()
            .map(|item| {
                let freshness = &item.freshness;
                let is_expired = CodexAuthService::is_expired(item.expires_at);
                json!({
                    "name": item.name,
                    "description": item.description,
                    "email": item.email,
                    "is_current": item.is_current,
                    "is_virtual": item.is_virtual,
                    "saved_at": item.saved_at.map(|dt| dt.to_rfc3339()),
                    "last_used": item.last_used.map(|dt| dt.to_rfc3339()),
                    "last_refresh": item.last_refresh.map(|dt| dt.to_rfc3339()),
                    "freshness": freshness,
                    "freshness_icon": freshness.icon(),
                    "freshness_description": freshness.description(),
                    "expires_at": item.expires_at.map(|dt| dt.to_rfc3339()),
                    "is_expired": is_expired,
                })
            })
            .collect();

        Ok(json!({ "accounts": accounts, "login_state": snapshot.login_state }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 获取当前 Codex Auth 信息
#[tauri::command]
pub async fn codex_get_auth_current() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        let snapshot = service
            .read_auth_snapshot()
            .map_err(|e| format!("读取认证快照失败: {e}"))?;

        let info = match snapshot.current_info.as_ref() {
            Some(current) => {
                let freshness = &current.freshness;
                let expires_at = snapshot.current_expires_at;
                let is_expired = CodexAuthService::is_expired(expires_at);
                Some(json!({
                    "account_id": current.account_id,
                    "email": current.email,
                    "last_refresh": current.last_refresh.map(|dt| dt.to_rfc3339()),
                    "freshness": freshness,
                    "freshness_icon": freshness.icon(),
                    "freshness_description": freshness.description(),
                    "expires_at": expires_at.map(|dt| dt.to_rfc3339()),
                    "is_expired": is_expired,
                }))
            }
            None => None,
        };

        let logged_in = info.is_some();

        Ok(json!({
            "logged_in": logged_in,
            "info": info,
            "login_state": snapshot.login_state,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}

/// 保存当前登录到命名账号
#[tauri::command]
pub async fn codex_save_auth(
    state: State<'_, AppState>,
    name: String,
    description: Option<String>,
    expires_at: Option<String>,
    force: Option<bool>,
) -> Result<Value, String> {
    let response = tokio::task::spawn_blocking(move || -> Result<Value, String> {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        let parsed_expires_at = expires_at
            .as_deref()
            .map(|value| {
                DateTime::parse_from_rfc3339(value)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| format!("expires_at 必须是 RFC3339 时间: {e}"))
            })
            .transpose()?;

        service
            .save_current(
                &name,
                description,
                parsed_expires_at,
                force.unwrap_or(false),
            )
            .map_err(|e| format!("{e}"))?;

        Ok(json!({ "success": true, "message": format!("Codex Auth 账号 '{name}' 已成功保存") }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(response)
}

/// 切换到指定账号
#[tauri::command]
pub async fn codex_switch_auth(state: State<'_, AppState>, name: String) -> Result<Value, String> {
    let name_resp = name.clone();
    tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        service.switch_account(&name).map_err(|e| format!("{e}"))?;

        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(json!({ "success": true, "message": format!("已切换到 Codex Auth 账号 '{name_resp}'") }))
}

/// 删除指定账号
#[tauri::command]
pub async fn codex_delete_auth(state: State<'_, AppState>, name: String) -> Result<Value, String> {
    let name_resp = name.clone();
    tokio::task::spawn_blocking(move || {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        service.delete_account(&name).map_err(|e| format!("{e}"))?;

        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))??;

    invalidate_codex_dashboard_overview_cache(&state).await;
    Ok(json!({ "success": true, "message": format!("Codex Auth 账号 '{name_resp}' 已成功删除") }))
}

/// 检测运行中的 Codex 进程
#[tauri::command]
pub async fn codex_detect_process() -> Result<Value, String> {
    tokio::task::spawn_blocking(|| {
        let service =
            CodexAuthService::new().map_err(|e| format!("初始化 Codex Auth 服务失败: {e}"))?;

        let pids = service.detect_codex_process();
        let has_running_process = !pids.is_empty();

        let warning = if has_running_process {
            Some(format!(
                "检测到 {} 个运行中的 Codex 进程 (PID: {})，切换账号前请先关闭这些进程",
                pids.len(),
                pids.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        } else {
            None
        };

        Ok(json!({
            "has_running_process": has_running_process,
            "pids": pids,
            "warning": warning,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {e}"))?
}
