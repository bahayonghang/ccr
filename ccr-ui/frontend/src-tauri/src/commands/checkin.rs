//! CheckIn 命令 — Provider/Account/执行/Balance/Export/CDK/WAF Cookie。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use ccr_db::managers::checkin::{
    AccountManager, BalanceManager, ExportManager, ProviderManager, RecordManager,
    WafCookieManager, get_checkin_dir,
};
use ccr_db::models::checkin::{
    CreateAccountRequest, CreateProviderRequest, ExportOptions, UpdateAccountRequest,
    UpdateProviderRequest,
};
use ccr_db::services::cdk_service::{CdkExtraConfig, CdkService};
use ccr_db::services::checkin_service::CheckinService;

use chrono::Datelike;

use crate::state::AppState;

/// Provider 概要
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ProviderInfo {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub enabled: bool,
}

/// Account 概要
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AccountInfo {
    pub id: i64,
    pub provider_id: i64,
    pub provider_name: String,
    pub username: String,
    pub enabled: bool,
    pub last_checkin: Option<String>,
}

// ── 辅助函数 ──

fn checkin_dir_str() -> Result<std::path::PathBuf, String> {
    get_checkin_dir().map_err(|e| format!("Failed to get checkin dir: {}", e))
}

// ── Provider 管理 ──

#[tauri::command]
pub async fn list_providers(_state: State<'_, AppState>) -> Result<Value, String> {
    let manager = ProviderManager::new();
    let response = manager
        .list()
        .map_err(|e| format!("Failed to list providers: {}", e))?;
    serde_json::to_value(response).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn add_provider(_state: State<'_, AppState>, data: Value) -> Result<Value, String> {
    let req: CreateProviderRequest =
        serde_json::from_value(data).map_err(|e| format!("Invalid request data: {}", e))?;
    let manager = ProviderManager::new();
    let provider = manager
        .create(req)
        .map_err(|e| format!("Failed to create provider: {}", e))?;
    serde_json::to_value(provider).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn update_provider(
    _state: State<'_, AppState>,
    id: String,
    data: Value,
) -> Result<Value, String> {
    let req: UpdateProviderRequest =
        serde_json::from_value(data).map_err(|e| format!("Invalid request data: {}", e))?;
    let manager = ProviderManager::new();
    let provider = manager
        .update(&id, req)
        .map_err(|e| format!("Failed to update provider: {}", e))?;
    serde_json::to_value(provider).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn delete_provider(_state: State<'_, AppState>, id: String) -> Result<String, String> {
    let checkin_dir = checkin_dir_str()?;
    let provider_manager = ProviderManager::new();
    let account_manager = AccountManager::new(&checkin_dir);

    let has_accounts = account_manager
        .has_accounts_for_provider(&id)
        .map_err(|e| format!("Failed to check accounts: {}", e))?;

    provider_manager
        .delete(&id, has_accounts)
        .map_err(|e| format!("Failed to delete provider: {}", e))?;

    Ok(format!("Provider {} deleted", id))
}

#[tauri::command]
pub async fn test_provider_connection(
    _state: State<'_, AppState>,
    id: String,
) -> Result<Value, String> {
    // Validate the provider exists and return its configuration as a test result
    let provider_manager = ProviderManager::new();
    let provider = provider_manager
        .get(&id)
        .map_err(|e| format!("Provider not found: {}", e))?;

    let result = serde_json::json!({
        "success": true,
        "provider_id": provider.id,
        "provider_name": provider.name,
        "base_url": provider.base_url,
        "message": "Provider configuration is valid"
    });

    Ok(result)
}

// ── Account 管理 ──

#[tauri::command]
pub async fn list_accounts(
    _state: State<'_, AppState>,
    provider_id: Option<String>,
) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let account_manager = AccountManager::new(&checkin_dir);
    let provider_manager = ProviderManager::new();
    let balance_manager = BalanceManager::new();

    // Get accounts
    let mut accounts = if let Some(pid) = provider_id.as_deref() {
        account_manager
            .list_by_provider(pid)
            .map_err(|e| format!("Failed to list accounts: {}", e))?
    } else {
        account_manager
            .list()
            .map_err(|e| format!("Failed to list accounts: {}", e))?
            .accounts
    };

    // Enrich with provider names and balance data
    let providers = provider_manager.load_all().unwrap_or_default();
    let provider_map: std::collections::HashMap<String, String> =
        providers.into_iter().map(|p| (p.id, p.name)).collect();

    let balance_map = balance_manager.get_latest_map().unwrap_or_default();

    for account in &mut accounts {
        if let Some(name) = provider_map.get(&account.provider_id) {
            account.provider_name = Some(name.clone());
        }
        if let Some(balance) = balance_map.get(&account.id) {
            account.latest_balance = Some(balance.remaining_quota);
            account.balance_currency = Some(balance.currency.clone());
            account.total_quota = Some(balance.total_quota);
            account.total_consumed = Some(balance.used_quota);
        }
    }

    let total = accounts.len();
    let response = serde_json::json!({ "accounts": accounts, "total": total });
    Ok(response)
}

#[tauri::command]
pub async fn add_account(_state: State<'_, AppState>, data: Value) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let req: CreateAccountRequest =
        serde_json::from_value(data).map_err(|e| format!("Invalid request data: {}", e))?;
    let account_manager = AccountManager::new(&checkin_dir);
    let account = account_manager
        .create(req)
        .map_err(|e| format!("Failed to create account: {}", e))?;
    let info = account_manager
        .get_info(&account.id)
        .map_err(|e| format!("Failed to get account info: {}", e))?;
    serde_json::to_value(info).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn update_account(
    _state: State<'_, AppState>,
    id: String,
    data: Value,
) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let req: UpdateAccountRequest =
        serde_json::from_value(data).map_err(|e| format!("Invalid request data: {}", e))?;
    let account_manager = AccountManager::new(&checkin_dir);
    let account = account_manager
        .update(&id, req)
        .map_err(|e| format!("Failed to update account: {}", e))?;
    let info = account_manager
        .get_info(&account.id)
        .map_err(|e| format!("Failed to get account info: {}", e))?;
    serde_json::to_value(info).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn delete_account(_state: State<'_, AppState>, id: String) -> Result<String, String> {
    let checkin_dir = checkin_dir_str()?;
    let account_manager = AccountManager::new(&checkin_dir);
    let record_manager = RecordManager::new();
    let balance_manager = BalanceManager::new();

    account_manager
        .delete(&id)
        .map_err(|e| format!("Failed to delete account: {}", e))?;

    // Clean up associated records and balance snapshots
    let _ = record_manager.delete_by_account(&id);
    let _ = balance_manager.delete_by_account(&id);

    Ok(format!("Account {} deleted", id))
}

#[tauri::command]
pub async fn batch_delete_accounts(
    _state: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<String, String> {
    let checkin_dir = checkin_dir_str()?;
    let account_manager = AccountManager::new(&checkin_dir);
    let record_manager = RecordManager::new();
    let balance_manager = BalanceManager::new();

    let mut deleted = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for id in &ids {
        match account_manager.delete(id) {
            Ok(()) => {
                deleted += 1;
                let _ = record_manager.delete_by_account(id);
                let _ = balance_manager.delete_by_account(id);
            }
            Err(e) => errors.push(format!("{}: {}", id, e)),
        }
    }

    if errors.is_empty() {
        Ok(format!("Deleted {} accounts", deleted))
    } else {
        Err(format!(
            "Deleted {} accounts, errors: {}",
            deleted,
            errors.join("; ")
        ))
    }
}

// ── 签到执行 ──

#[tauri::command]
pub async fn execute_checkin(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let service = CheckinService::with_client(checkin_dir, state.http_client.clone());

    let result = service
        .checkin(&account_id)
        .await
        .map_err(|e| format!("Checkin failed: {}", e))?;

    serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn batch_checkin(
    state: State<'_, AppState>,
    account_ids: Vec<String>,
) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let service = CheckinService::with_client(checkin_dir, state.http_client.clone());

    let results = service.batch_checkin(&account_ids).await;

    let mut success = 0usize;
    let mut already_checked_in = 0usize;
    let mut failed = 0usize;

    for result in &results {
        match result.status {
            ccr_db::models::checkin::CheckinStatus::Success => success += 1,
            ccr_db::models::checkin::CheckinStatus::AlreadyCheckedIn => already_checked_in += 1,
            ccr_db::models::checkin::CheckinStatus::Failed => failed += 1,
        }
    }

    let response = serde_json::json!({
        "results": results,
        "summary": {
            "total": results.len(),
            "success": success,
            "already_checked_in": already_checked_in,
            "failed": failed
        }
    });

    Ok(response)
}

#[tauri::command]
pub async fn get_checkin_records(
    _state: State<'_, AppState>,
    account_id: Option<String>,
    limit: Option<usize>,
) -> Result<Value, String> {
    let record_manager = RecordManager::new();

    let response = if let Some(aid) = account_id.as_deref() {
        record_manager
            .get_by_account(aid, limit)
            .map_err(|e| format!("Failed to get records: {}", e))?
    } else {
        record_manager
            .get_all(limit)
            .map_err(|e| format!("Failed to get records: {}", e))?
    };

    serde_json::to_value(response).map_err(|e| format!("Serialization error: {}", e))
}

// ── Balance ──

#[tauri::command]
pub async fn get_balance(state: State<'_, AppState>, account_id: String) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let service = CheckinService::with_client(checkin_dir, state.http_client.clone());

    let snapshot = service
        .query_balance(&account_id)
        .await
        .map_err(|e| format!("Failed to query balance: {}", e))?;

    serde_json::to_value(snapshot).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn get_balance_history(
    _state: State<'_, AppState>,
    account_id: String,
    days: Option<usize>,
) -> Result<Value, String> {
    let balance_manager = BalanceManager::new();
    let history = balance_manager
        .get_history(&account_id, days)
        .map_err(|e| format!("Failed to get balance history: {}", e))?;
    serde_json::to_value(history).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn get_balance_stats(_state: State<'_, AppState>) -> Result<Value, String> {
    let balance_manager = BalanceManager::new();
    let balance_map = balance_manager
        .get_latest_map()
        .map_err(|e| format!("Failed to get balances: {}", e))?;

    let total_accounts = balance_map.len();
    let total_remaining: f64 = balance_map.values().map(|b| b.remaining_quota).sum();
    let total_quota: f64 = balance_map.values().map(|b| b.total_quota).sum();
    let total_used: f64 = balance_map.values().map(|b| b.used_quota).sum();

    let response = serde_json::json!({
        "total_accounts": total_accounts,
        "total_remaining_quota": total_remaining,
        "total_quota": total_quota,
        "total_used_quota": total_used,
        "balances": balance_map.values().collect::<Vec<_>>()
    });

    Ok(response)
}

// ── Export ──

#[tauri::command]
pub async fn export_checkin_data(
    _state: State<'_, AppState>,
    options: Value,
) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let opts: ExportOptions =
        serde_json::from_value(options).map_err(|e| format!("Invalid export options: {}", e))?;
    let export_manager = ExportManager::new(&checkin_dir);
    let data = export_manager
        .export(&opts)
        .map_err(|e| format!("Failed to export data: {}", e))?;
    serde_json::to_value(data).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn export_checkin_stats(_state: State<'_, AppState>) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let service = CheckinService::new(checkin_dir);

    let stats = service
        .get_today_stats()
        .map_err(|e| format!("Failed to get stats: {}", e))?;

    serde_json::to_value(stats).map_err(|e| format!("Serialization error: {}", e))
}

// ── CDK 充值 ──

#[tauri::command]
pub async fn execute_cdk_recharge(
    state: State<'_, AppState>,
    account_id: String,
    cdk_code: String,
) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let account_manager = AccountManager::new(&checkin_dir);
    let provider_manager = ProviderManager::new();

    let account = account_manager
        .get(&account_id)
        .map_err(|e| format!("Account not found: {}", e))?;
    let provider = provider_manager
        .get(&account.provider_id)
        .map_err(|e| format!("Provider not found: {}", e))?;

    // Look up builtin provider CDK config
    use ccr_db::managers::checkin::builtin_providers::get_builtin_providers;
    let builtin_providers = get_builtin_providers();
    let cdk_config = builtin_providers
        .iter()
        .find(|bp| {
            bp.name == provider.name || bp.id == format!("builtin-{}", provider.name.to_lowercase())
        })
        .and_then(|bp| bp.cdk_config.as_ref());

    let cdk_config = match cdk_config {
        Some(config) => config,
        None => {
            return Err(format!(
                "Provider {} does not support CDK recharge",
                provider.name
            ));
        }
    };

    // Parse account extra_config for CDK credentials
    let extra_config = CdkExtraConfig::from_json(&account.extra_config);

    // Decrypt cookies for topup request
    use ccr_db::core::crypto::CryptoManager;
    let crypto = CryptoManager::new(&checkin_dir)
        .map_err(|e| format!("Crypto initialization error: {}", e))?;
    let cookies_json = crypto
        .decrypt(&account.cookies_json_encrypted)
        .map_err(|e| format!("Failed to decrypt cookies: {}", e))?;

    let topup_cookies: std::collections::HashMap<String, String> =
        serde_json::from_str(&cookies_json).unwrap_or_default();

    // Build topup URL and include the provided CDK code
    let topup_url = cdk_config
        .topup_path
        .as_ref()
        .map(|path| format!("{}{}", provider.base_url.trim_end_matches('/'), path));

    // If a specific cdk_code is provided, use it directly as a topup key
    if !cdk_code.is_empty() {
        let topup_url = topup_url
            .as_deref()
            .ok_or_else(|| format!("No topup URL configured for provider {}", provider.name))?;

        let cookie_str = topup_cookies
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ");

        let body = serde_json::json!({ "key": cdk_code });
        let mut req = state
            .http_client
            .post(topup_url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        if !cookie_str.is_empty() {
            req = req.header("Cookie", cookie_str);
        }
        if !account.api_user.is_empty() {
            req = req.header("new-api-user", &account.api_user);
        }

        let resp = req
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = resp.status();
        let resp_body = resp.text().await.unwrap_or_default();

        let result = serde_json::json!({
            "success": status.is_success(),
            "cdk_code": cdk_code,
            "status_code": status.as_u16(),
            "response": resp_body
        });

        return Ok(result);
    }

    // Auto-fetch CDK via CdkService
    let cdk_service = CdkService::new(None);
    let result = cdk_service
        .fetch_and_topup(
            &cdk_config.cdk_type,
            &extra_config,
            topup_url.as_deref(),
            &topup_cookies,
            &account.api_user,
        )
        .await;

    serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn get_cdk_history(
    _state: State<'_, AppState>,
    account_id: Option<String>,
) -> Result<Value, String> {
    // CDK history is not separately stored; return checkin records as a proxy
    let record_manager = RecordManager::new();

    let response = if let Some(aid) = account_id.as_deref() {
        record_manager
            .get_by_account(aid, Some(50))
            .map_err(|e| format!("Failed to get CDK history: {}", e))?
    } else {
        record_manager
            .get_all(Some(50))
            .map_err(|e| format!("Failed to get CDK history: {}", e))?
    };

    serde_json::to_value(response).map_err(|e| format!("Serialization error: {}", e))
}

// ── WAF Cookie 管理 ──

#[tauri::command]
pub async fn list_waf_cookies(_state: State<'_, AppState>) -> Result<Value, String> {
    // WafCookieManager provides per-provider lookup; list all by getting all providers
    let provider_manager = ProviderManager::new();
    let providers = provider_manager.load_all().unwrap_or_default();

    let waf_manager = WafCookieManager::new();
    let mut waf_entries: Vec<Value> = Vec::new();

    for provider in &providers {
        if let Ok(Some(cookies)) = waf_manager.get_valid(&provider.id) {
            waf_entries.push(serde_json::json!({
                "provider_id": provider.id,
                "provider_name": provider.name,
                "cookies": cookies
            }));
        }
    }

    let response = serde_json::json!({
        "waf_cookies": waf_entries,
        "total": waf_entries.len()
    });

    Ok(response)
}

#[tauri::command]
pub async fn add_waf_cookie(
    _state: State<'_, AppState>,
    provider_id: String,
    cookie: String,
) -> Result<Value, String> {
    // Parse cookie string into a HashMap
    let mut cookies: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    // Try JSON first
    if let Ok(parsed) = serde_json::from_str::<std::collections::HashMap<String, String>>(&cookie) {
        cookies = parsed;
    } else {
        // Parse as key=value; pairs
        for pair in cookie.split(';') {
            let pair = pair.trim();
            if let Some((k, v)) = pair.split_once('=') {
                cookies.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }

    if cookies.is_empty() {
        return Err("No valid cookies provided".to_string());
    }

    let waf_manager = WafCookieManager::new();
    waf_manager
        .save(&provider_id, cookies.clone())
        .map_err(|e| format!("Failed to save WAF cookie: {}", e))?;

    let result = serde_json::json!({
        "provider_id": provider_id,
        "cookies_count": cookies.len(),
        "message": "WAF cookies saved successfully"
    });

    Ok(result)
}

#[tauri::command]
pub async fn delete_waf_cookie(_state: State<'_, AppState>, id: String) -> Result<String, String> {
    // `id` is treated as provider_id for WAF cookies
    let waf_manager = WafCookieManager::new();
    let deleted = waf_manager
        .delete(&id)
        .map_err(|e| format!("Failed to delete WAF cookie: {}", e))?;

    if deleted {
        Ok(format!("WAF cookies for provider {} deleted", id))
    } else {
        Ok(format!("No WAF cookies found for provider {}", id))
    }
}

// ── 内置 Provider + 导入导出扩展 ──

#[tauri::command]
pub async fn list_builtin_providers() -> Result<Value, String> {
    use ccr_db::managers::checkin::builtin_providers::get_builtin_providers;
    let providers = get_builtin_providers();
    serde_json::to_value(providers).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn add_builtin_provider(provider_id: String) -> Result<Value, String> {
    use ccr_db::managers::checkin::builtin_providers::get_builtin_providers;

    let providers = get_builtin_providers();
    let builtin = providers
        .iter()
        .find(|p| p.id == provider_id)
        .ok_or_else(|| format!("Builtin provider '{}' not found", provider_id))?;

    let checkin_provider = builtin.to_checkin_provider();

    let manager = ProviderManager::new();
    let req = CreateProviderRequest {
        name: checkin_provider.name,
        base_url: checkin_provider.base_url,
        checkin_path: Some(checkin_provider.checkin_path),
        balance_path: Some(checkin_provider.balance_path),
        user_info_path: Some(checkin_provider.user_info_path),
        auth_header: Some(checkin_provider.auth_header),
        auth_prefix: Some(checkin_provider.auth_prefix),
    };
    let provider = manager
        .create(req)
        .map_err(|e| format!("Failed to create provider: {}", e))?;
    serde_json::to_value(provider).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn get_checkin_account_cookies(
    _state: State<'_, AppState>,
    account_id: String,
) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let account_manager = AccountManager::new(&checkin_dir);

    let account = account_manager
        .get(&account_id)
        .map_err(|e| format!("Account not found: {}", e))?;

    use ccr_db::core::crypto::CryptoManager;
    let crypto = CryptoManager::new(&checkin_dir)
        .map_err(|e| format!("Crypto initialization error: {}", e))?;
    let cookies_json = crypto
        .decrypt(&account.cookies_json_encrypted)
        .map_err(|e| format!("Failed to decrypt cookies: {}", e))?;

    Ok(serde_json::json!({
        "account_id": account_id,
        "cookies_json": cookies_json,
    }))
}

#[tauri::command]
pub async fn export_checkin_config(options: Option<Value>) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let opts: ccr_db::models::checkin::ExportOptions = if let Some(v) = options {
        serde_json::from_value(v).map_err(|e| format!("Invalid export options: {}", e))?
    } else {
        ccr_db::models::checkin::ExportOptions::default()
    };
    let export_manager = ExportManager::new(&checkin_dir);
    let data = export_manager
        .export(&opts)
        .map_err(|e| format!("Failed to export config: {}", e))?;
    serde_json::to_value(data).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn preview_checkin_import(data: Value) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let export_data: ccr_db::models::checkin::ExportData =
        serde_json::from_value(data).map_err(|e| format!("Invalid import data: {}", e))?;
    let export_manager = ExportManager::new(&checkin_dir);
    let preview = export_manager
        .preview_import(&export_data)
        .map_err(|e| format!("Failed to preview import: {}", e))?;
    serde_json::to_value(preview).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn import_checkin_config(data: Value, options: Option<Value>) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let export_data: ccr_db::models::checkin::ExportData =
        serde_json::from_value(data).map_err(|e| format!("Invalid import data: {}", e))?;
    let import_opts: ccr_db::models::checkin::ImportOptions = if let Some(v) = options {
        serde_json::from_value(v).map_err(|e| format!("Invalid import options: {}", e))?
    } else {
        ccr_db::models::checkin::ImportOptions::default()
    };
    let export_manager = ExportManager::new(&checkin_dir);
    let result = export_manager
        .import(export_data, &import_opts)
        .map_err(|e| format!("Failed to import config: {}", e))?;
    serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
}

// ── Dashboard ──

#[tauri::command]
pub async fn get_account_dashboard(
    _state: State<'_, AppState>,
    account_id: String,
    year: Option<i32>,
    month: Option<u32>,
    days: Option<u32>,
) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let service = CheckinService::new(checkin_dir);

    let now = chrono::Local::now();
    let y = year.unwrap_or(now.year());
    let m = month.unwrap_or(now.month());
    let d = days.unwrap_or(30);

    let dashboard = service
        .get_account_dashboard(&account_id, y, m, d)
        .map_err(|e| format!("Failed to get dashboard: {}", e))?;

    serde_json::to_value(dashboard).map_err(|e| format!("Serialization error: {}", e))
}
