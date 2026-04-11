use super::*;

fn build_codex_usage_payload(rolling: ccr_codex::services::CodexRollingUsage) -> Value {
    let by_model: serde_json::Map<String, Value> = rolling
        .by_model
        .into_iter()
        .map(|(model, stats)| {
            (
                model,
                json!({
                    "total_input_tokens": stats.total_input_tokens,
                    "total_output_tokens": stats.total_output_tokens,
                    "total_requests": stats.total_requests,
                    "window_start": stats.window_start.map(|dt| dt.to_rfc3339()),
                    "window_end": stats.window_end.map(|dt| dt.to_rfc3339()),
                }),
            )
        })
        .collect();

    json!({
        "five_hour": {
            "total_input_tokens": rolling.five_hour.total_input_tokens,
            "total_output_tokens": rolling.five_hour.total_output_tokens,
            "total_requests": rolling.five_hour.total_requests,
            "window_start": rolling.five_hour.window_start.map(|dt| dt.to_rfc3339()),
            "window_end": rolling.five_hour.window_end.map(|dt| dt.to_rfc3339()),
        },
        "seven_day": {
            "total_input_tokens": rolling.seven_day.total_input_tokens,
            "total_output_tokens": rolling.seven_day.total_output_tokens,
            "total_requests": rolling.seven_day.total_requests,
            "window_start": rolling.seven_day.window_start.map(|dt| dt.to_rfc3339()),
            "window_end": rolling.seven_day.window_end.map(|dt| dt.to_rfc3339()),
        },
        "all_time": {
            "total_input_tokens": rolling.all_time.total_input_tokens,
            "total_output_tokens": rolling.all_time.total_output_tokens,
            "total_requests": rolling.all_time.total_requests,
            "window_start": rolling.all_time.window_start.map(|dt| dt.to_rfc3339()),
            "window_end": rolling.all_time.window_end.map(|dt| dt.to_rfc3339()),
        },
        "by_model": Value::Object(by_model),
    })
}

pub(super) fn compute_codex_usage_payload() -> Result<Value, String> {
    let codex_dir = dirs::home_dir()
        .ok_or_else(|| "无法获取用户主目录".to_string())?
        .join(".codex");

    let service = CodexUsageService::new(codex_dir);
    let rolling = service
        .compute_rolling_usage()
        .map_err(|e| format!("计算使用量失败: {e}"))?;

    Ok(build_codex_usage_payload(rolling))
}

/// 查询所有 Codex 账号的配额余额
#[tauri::command]
pub async fn codex_get_all_quotas() -> Result<Value, String> {
    let service =
        ccr_codex::CodexQuotaService::new().map_err(|e| format!("初始化配额服务失败: {e}"))?;
    let quotas = service.fetch_all_quotas().await;
    serde_json::to_value(&quotas).map_err(|e| format!("序列化配额数据失败: {e}"))
}

/// 查询指定 Codex 账号的配额余额
#[tauri::command]
pub async fn codex_get_quota(account: String) -> Result<Value, String> {
    let service =
        ccr_codex::CodexQuotaService::new().map_err(|e| format!("初始化配额服务失败: {e}"))?;
    let quota = service.fetch_account_quota(&account).await;
    serde_json::to_value(&quota).map_err(|e| format!("序列化配额数据失败: {e}"))
}

/// 获取 Codex 使用量统计
#[tauri::command]
pub async fn codex_get_usage(
    state: State<'_, AppState>,
    force: Option<bool>,
) -> Result<Value, String> {
    get_cached_codex_usage_payload(&state, force.unwrap_or(false)).await
}

/// 获取 Codex 仪表盘概览（轻量数据）
#[tauri::command]
pub async fn codex_get_dashboard_overview(
    state: State<'_, AppState>,
    force: Option<bool>,
) -> Result<Value, String> {
    get_cached_codex_dashboard_overview_payload(&state, force.unwrap_or(false)).await
}

/// 获取 Codex 仪表盘使用量摘要（重数据，独立异步加载）
#[tauri::command]
pub async fn codex_get_dashboard_usage_summary(
    state: State<'_, AppState>,
    force: Option<bool>,
) -> Result<Value, String> {
    let usage = get_cached_codex_usage_payload(&state, force.unwrap_or(false)).await?;
    Ok(build_codex_usage_summary(&usage))
}
