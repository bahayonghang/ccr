//! CheckIn 命令模块，涵盖 Provider/Account/签到/Balance/Export/CDK/WAF Cookie 等功能。

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::timeout;
use uuid::Uuid;

use ccr_db::managers::checkin::{
    AccountManager, BalanceManager, ExportManager, ProviderManager, RecordManager,
    WafCookieManager, get_checkin_dir,
};
use ccr_db::models::checkin::{
    CheckinStatus, CreateAccountRequest, CreateProviderRequest, ExportOptions,
    UpdateAccountRequest, UpdateProviderRequest,
};
use ccr_db::services::cdk_service::{CdkExtraConfig, CdkService};
use ccr_db::services::checkin_service::{CheckinExecutionResult, CheckinService};

use chrono::Datelike;

use crate::checkin_jobs::{CheckinJobLogEntry, CheckinJobSnapshot, CheckinJobStatus};
use crate::monitoring::{checkin_job_entry, record_monitoring_entry, should_persist};
use crate::state::AppState;

/// Provider 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ProviderInfo {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub enabled: bool,
}

/// Account 信息
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

// —— 通用辅助方法 ——

fn checkin_dir_str() -> Result<std::path::PathBuf, String> {
    get_checkin_dir().map_err(|e| format!("Failed to get checkin dir: {}", e))
}

async fn run_blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|e| format!("Blocking task failed: {e}"))?
}

#[derive(Debug, Clone, Serialize)]
pub struct StartCheckinJobResponse {
    pub job_id: String,
    pub snapshot: CheckinJobSnapshot,
}

#[derive(Debug, Clone)]
struct CheckinJobAccountMeta {
    account_id: String,
    account_name: String,
    provider_name: String,
}

fn build_failed_checkin_result(
    meta: &CheckinJobAccountMeta,
    message: impl Into<String>,
) -> CheckinExecutionResult {
    CheckinExecutionResult {
        account_id: meta.account_id.clone(),
        account_name: meta.account_name.clone(),
        provider_name: meta.provider_name.clone(),
        status: CheckinStatus::Failed,
        message: Some(message.into()),
        reward: None,
        balance: None,
    }
}

async fn load_checkin_job_accounts(
    account_ids: Vec<String>,
) -> Result<Vec<CheckinJobAccountMeta>, String> {
    let checkin_dir = checkin_dir_str()?;
    run_blocking(move || {
        let account_manager = AccountManager::new(&checkin_dir);
        let provider_manager = ProviderManager::new();

        let mut deduped_ids = Vec::new();
        let mut seen = HashSet::new();
        for account_id in account_ids {
            if seen.insert(account_id.clone()) {
                deduped_ids.push(account_id);
            }
        }

        let accounts = account_manager
            .load_all()
            .map_err(|e| format!("Failed to load accounts: {}", e))?;
        let account_map: HashMap<String, _> =
            accounts.into_iter().map(|account| (account.id.clone(), account)).collect();

        let providers = provider_manager
            .load_all()
            .map_err(|e| format!("Failed to load providers: {}", e))?;
        let provider_map: HashMap<String, String> =
            providers.into_iter().map(|provider| (provider.id, provider.name)).collect();

        let mut metas = Vec::with_capacity(deduped_ids.len());
        let mut missing = Vec::new();

        for account_id in deduped_ids {
            if let Some(account) = account_map.get(&account_id) {
                let provider_name = provider_map
                    .get(&account.provider_id)
                    .cloned()
                    .unwrap_or_else(|| account.provider_id.clone());
                metas.push(CheckinJobAccountMeta {
                    account_id: account.id.clone(),
                    account_name: account.name.clone(),
                    provider_name,
                });
            } else {
                missing.push(account_id);
            }
        }

        if !missing.is_empty() {
            return Err(format!("Accounts not found: {}", missing.join(", ")));
        }

        Ok(metas)
    })
    .await
}

async fn emit_checkin_job_snapshot(
    app_handle: &AppHandle,
    event: &str,
    snapshot: &CheckinJobSnapshot,
) {
    if let Err(error) = app_handle.emit(event, snapshot.clone()) {
        tracing::warn!(event, ?error, job_id = %snapshot.job_id, "Failed to emit checkin job event");
    }

    let entry = checkin_job_entry(event, snapshot);
    let persist = should_persist(entry.level, &entry.event_type);
    record_monitoring_entry(app_handle, entry, persist).await;
}
async fn execute_checkin_job_accounts(
    app_handle: AppHandle,
    job_id: String,
    account_metas: Vec<CheckinJobAccountMeta>,
    checkin_dir: PathBuf,
    http_client: reqwest::Client,
) -> Result<(), String> {
    let semaphore = Arc::new(Semaphore::new(5));
    let mut join_set = JoinSet::new();

    for meta in account_metas {
        let app_handle = app_handle.clone();
        let job_id = job_id.clone();
        let checkin_dir = checkin_dir.clone();
        let http_client = http_client.clone();
        let semaphore = semaphore.clone();

        join_set.spawn(async move {
            let _permit = semaphore
                .acquire_owned()
                .await
                .map_err(|e| format!("Failed to acquire checkin permit: {}", e))?;

            let state = app_handle.state::<AppState>();
            if let Some(snapshot) = state
                .update_checkin_job(&job_id, |job| job.mark_processing(&meta.account_id))
                .await
            {
                emit_checkin_job_snapshot(&app_handle, "checkin:job-progress", &snapshot).await;
            }

            let service = CheckinService::with_client(checkin_dir, http_client);
            let result = match timeout(Duration::from_secs(90), service.checkin(&meta.account_id)).await {
                Ok(Ok(result)) => result,
                Ok(Err(error)) => build_failed_checkin_result(&meta, format!("Checkin failed: {}", error)),
                Err(_) => build_failed_checkin_result(&meta, "签到超时"),
            };

            let state = app_handle.state::<AppState>();
            if let Some(snapshot) = state
                .update_checkin_job(&job_id, |job| job.apply_result(result))
                .await
            {
                emit_checkin_job_snapshot(&app_handle, "checkin:job-progress", &snapshot).await;
            }

            Ok::<(), String>(())
        });
    }

    let mut task_failed = false;
    while let Some(joined) = join_set.join_next().await {
        match joined {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                task_failed = true;
                tracing::warn!(job_id = %job_id, ?error, "Checkin account task failed");
            }
            Err(error) => {
                task_failed = true;
                tracing::warn!(job_id = %job_id, ?error, "Checkin account task join failed");
            }
        }
    }

    if task_failed {
        let state = app_handle.state::<AppState>();
        if let Some(snapshot) = state
            .update_checkin_job(&job_id, |job| {
                job.mark_pending_failed("签到任务失败");
                if !matches!(job.status, CheckinJobStatus::Finished | CheckinJobStatus::TimedOut) {
                    job.mark_finished(CheckinJobStatus::Finished);
                }
            })
            .await
        {
            emit_checkin_job_snapshot(&app_handle, "checkin:job-progress", &snapshot).await;
        }
    }

    Ok(())
}

async fn run_checkin_job(
    app_handle: AppHandle,
    job_id: String,
    account_metas: Vec<CheckinJobAccountMeta>,
    checkin_dir: PathBuf,
    http_client: reqwest::Client,
) {
    let execution = execute_checkin_job_accounts(
        app_handle.clone(),
        job_id.clone(),
        account_metas,
        checkin_dir,
        http_client,
    );

    match timeout(Duration::from_secs(600), execution).await {
        Ok(Ok(())) => {
            if let Some(snapshot) = app_handle.state::<AppState>().get_checkin_job(&job_id).await {
                emit_checkin_job_snapshot(&app_handle, "checkin:job-finished", &snapshot).await;
            }
        }
        Ok(Err(error)) => {
            tracing::error!(job_id = %job_id, ?error, "Checkin job failed");
            if let Some(snapshot) = app_handle
                .state::<AppState>()
                .update_checkin_job(&job_id, |job| {
                    job.mark_pending_failed("签到任务失败");
                    if !matches!(job.status, CheckinJobStatus::Finished | CheckinJobStatus::TimedOut) {
                        job.mark_finished(CheckinJobStatus::Finished);
                    }
                })
                .await
            {
                emit_checkin_job_snapshot(&app_handle, "checkin:job-finished", &snapshot).await;
            }
        }
        Err(_) => {
            tracing::warn!(job_id = %job_id, "Checkin job timed out");
            if let Some(snapshot) = app_handle
                .state::<AppState>()
                .update_checkin_job(&job_id, |job| job.mark_timed_out())
                .await
            {
                emit_checkin_job_snapshot(&app_handle, "checkin:job-timeout", &snapshot).await;
            }
        }
    }
}

// —— Provider 管理 ——

#[tauri::command]
pub async fn list_providers(_state: State<'_, AppState>) -> Result<Value, String> {
    run_blocking(|| {
        let manager = ProviderManager::new();
        let response = manager
            .list()
            .map_err(|e| format!("Failed to list providers: {}", e))?;
        serde_json::to_value(response).map_err(|e| format!("Serialization error: {}", e))
    })
    .await
}

#[tauri::command]
pub async fn add_provider(_state: State<'_, AppState>, data: Value) -> Result<Value, String> {
    let req: CreateProviderRequest =
        serde_json::from_value(data).map_err(|e| format!("Invalid request data: {}", e))?;
    run_blocking(move || {
        let manager = ProviderManager::new();
        let provider = manager
            .create(req)
            .map_err(|e| format!("Failed to create provider: {}", e))?;
        serde_json::to_value(provider).map_err(|e| format!("Serialization error: {}", e))
    })
    .await
}

#[tauri::command]
pub async fn update_provider(
    _state: State<'_, AppState>,
    id: String,
    data: Value,
) -> Result<Value, String> {
    let req: UpdateProviderRequest =
        serde_json::from_value(data).map_err(|e| format!("Invalid request data: {}", e))?;
    run_blocking(move || {
        let manager = ProviderManager::new();
        let provider = manager
            .update(&id, req)
            .map_err(|e| format!("Failed to update provider: {}", e))?;
        serde_json::to_value(provider).map_err(|e| format!("Serialization error: {}", e))
    })
    .await
}

#[tauri::command]
pub async fn delete_provider(_state: State<'_, AppState>, id: String) -> Result<String, String> {
    let checkin_dir = checkin_dir_str()?;
    run_blocking(move || {
        let provider_manager = ProviderManager::new();
        let account_manager = AccountManager::new(&checkin_dir);

        let has_accounts = account_manager
            .has_accounts_for_provider(&id)
            .map_err(|e| format!("Failed to check accounts: {}", e))?;

        provider_manager
            .delete(&id, has_accounts)
            .map_err(|e| format!("Failed to delete provider: {}", e))?;

        Ok(format!("Provider {} deleted", id))
    })
    .await
}

#[tauri::command]
pub async fn test_provider_connection(
    _state: State<'_, AppState>,
    id: String,
) -> Result<Value, String> {
    run_blocking(move || {
        // Validate the provider exists and return its configuration as a test result
        let provider_manager = ProviderManager::new();
        let provider = provider_manager
            .get(&id)
            .map_err(|e| format!("Provider not found: {}", e))?;

        Ok(serde_json::json!({
            "success": true,
            "provider_id": provider.id,
            "provider_name": provider.name,
            "base_url": provider.base_url,
            "message": "Provider configuration is valid"
        }))
    })
    .await
}

// —— Account 管理 ——

#[tauri::command]
pub async fn list_accounts(
    _state: State<'_, AppState>,
    provider_id: Option<String>,
) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    run_blocking(move || {
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
        Ok(serde_json::json!({ "accounts": accounts, "total": total }))
    })
    .await
}

#[tauri::command]
pub async fn add_account(_state: State<'_, AppState>, data: Value) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let req: CreateAccountRequest =
        serde_json::from_value(data).map_err(|e| format!("Invalid request data: {}", e))?;
    run_blocking(move || {
        let account_manager = AccountManager::new(&checkin_dir);
        let account = account_manager
            .create(req)
            .map_err(|e| format!("Failed to create account: {}", e))?;
        let info = account_manager
            .get_info(&account.id)
            .map_err(|e| format!("Failed to get account info: {}", e))?;
        serde_json::to_value(info).map_err(|e| format!("Serialization error: {}", e))
    })
    .await
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
    run_blocking(move || {
        let account_manager = AccountManager::new(&checkin_dir);
        let account = account_manager
            .update(&id, req)
            .map_err(|e| format!("Failed to update account: {}", e))?;
        let info = account_manager
            .get_info(&account.id)
            .map_err(|e| format!("Failed to get account info: {}", e))?;
        serde_json::to_value(info).map_err(|e| format!("Serialization error: {}", e))
    })
    .await
}

#[tauri::command]
pub async fn delete_account(_state: State<'_, AppState>, id: String) -> Result<String, String> {
    let checkin_dir = checkin_dir_str()?;
    run_blocking(move || {
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
    })
    .await
}

#[tauri::command]
pub async fn batch_delete_accounts(
    _state: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<String, String> {
    let checkin_dir = checkin_dir_str()?;
    run_blocking(move || {
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
    })
    .await
}

// —— 签到执行 ——

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
pub async fn start_checkin_job(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    account_ids: Vec<String>,
) -> Result<Value, String> {
    let account_metas = load_checkin_job_accounts(account_ids).await?;
    if account_metas.is_empty() {
        return Err("No accounts selected for checkin".to_string());
    }

    let job_id = format!("checkin-{}", Uuid::new_v4());
    let logs = account_metas
        .iter()
        .map(|meta| {
            CheckinJobLogEntry::pending(
                meta.account_id.clone(),
                meta.account_name.clone(),
                meta.provider_name.clone(),
            )
        })
        .collect();
    let snapshot = CheckinJobSnapshot::new(job_id.clone(), logs);
    state.insert_checkin_job(snapshot.clone()).await;

    let response = StartCheckinJobResponse {
        job_id: job_id.clone(),
        snapshot: snapshot.clone(),
    };

    let checkin_dir = checkin_dir_str()?;
    let http_client = state.http_client.clone();
    tauri::async_runtime::spawn(run_checkin_job(
        app_handle,
        job_id,
        account_metas,
        checkin_dir,
        http_client,
    ));

    serde_json::to_value(response).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn get_checkin_job_status(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<Value, String> {
    let snapshot = state
        .get_checkin_job(&job_id)
        .await
        .ok_or_else(|| format!("Checkin job '{}' not found", job_id))?;

    serde_json::to_value(snapshot).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn get_checkin_records(
    _state: State<'_, AppState>,
    account_id: Option<String>,
    limit: Option<usize>,
) -> Result<Value, String> {
    run_blocking(move || {
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
    })
    .await
}

// —— Balance 查询 ——

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
    run_blocking(move || {
        let balance_manager = BalanceManager::new();
        let history = balance_manager
            .get_history(&account_id, days)
            .map_err(|e| format!("Failed to get balance history: {}", e))?;
        serde_json::to_value(history).map_err(|e| format!("Serialization error: {}", e))
    })
    .await
}

#[tauri::command]
pub async fn get_balance_stats(_state: State<'_, AppState>) -> Result<Value, String> {
    run_blocking(|| {
        let balance_manager = BalanceManager::new();
        let balance_map = balance_manager
            .get_latest_map()
            .map_err(|e| format!("Failed to get balances: {}", e))?;

        let total_accounts = balance_map.len();
        let total_remaining: f64 = balance_map.values().map(|b| b.remaining_quota).sum();
        let total_quota: f64 = balance_map.values().map(|b| b.total_quota).sum();
        let total_used: f64 = balance_map.values().map(|b| b.used_quota).sum();

        Ok(serde_json::json!({
            "total_accounts": total_accounts,
            "total_remaining_quota": total_remaining,
            "total_quota": total_quota,
            "total_used_quota": total_used,
            "balances": balance_map.values().collect::<Vec<_>>()
        }))
    })
    .await
}

// —— Export 导出 ——

#[tauri::command]
pub async fn export_checkin_data(
    _state: State<'_, AppState>,
    options: Value,
) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let opts: ExportOptions =
        serde_json::from_value(options).map_err(|e| format!("Invalid export options: {}", e))?;
    run_blocking(move || {
        let export_manager = ExportManager::new(&checkin_dir);
        let data = export_manager
            .export(&opts)
            .map_err(|e| format!("Failed to export data: {}", e))?;
        serde_json::to_value(data).map_err(|e| format!("Serialization error: {}", e))
    })
    .await
}

#[tauri::command]
pub async fn export_checkin_stats(_state: State<'_, AppState>) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    run_blocking(move || {
        let service = CheckinService::new(checkin_dir);

        let stats = service
            .get_today_stats()
            .map_err(|e| format!("Failed to get stats: {}", e))?;

        serde_json::to_value(stats).map_err(|e| format!("Serialization error: {}", e))
    })
    .await
}

// —— CDK 充值 ——

#[tauri::command]
pub async fn execute_cdk_recharge(
    state: State<'_, AppState>,
    account_id: String,
    cdk_code: String,
) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let context = run_blocking(move || {
        struct CdkRechargeContext {
            provider_name: String,
            cdk_type: String,
            extra_config: CdkExtraConfig,
            topup_cookies: std::collections::HashMap<String, String>,
            topup_url: Option<String>,
            api_user: String,
        }

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
                bp.name == provider.name
                    || bp.id == format!("builtin-{}", provider.name.to_lowercase())
            })
            .and_then(|bp| bp.cdk_config.as_ref())
            .ok_or_else(|| format!("Provider {} does not support CDK recharge", provider.name))?;

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

        Ok::<_, String>(CdkRechargeContext {
            provider_name: provider.name,
            cdk_type: cdk_config.cdk_type.clone(),
            extra_config,
            topup_cookies,
            topup_url,
            api_user: account.api_user,
        })
    })
    .await?;

    // If a specific cdk_code is provided, use it directly as a topup key
    if !cdk_code.is_empty() {
        let topup_url = context.topup_url.as_deref().ok_or_else(|| {
            format!(
                "No topup URL configured for provider {}",
                context.provider_name
            )
        })?;

        let cookie_str = context
            .topup_cookies
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
        if !context.api_user.is_empty() {
            req = req.header("new-api-user", &context.api_user);
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
            &context.cdk_type,
            &context.extra_config,
            context.topup_url.as_deref(),
            &context.topup_cookies,
            &context.api_user,
        )
        .await;

    serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
}

#[tauri::command]
pub async fn get_cdk_history(
    _state: State<'_, AppState>,
    account_id: Option<String>,
) -> Result<Value, String> {
    run_blocking(move || {
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
    })
    .await
}

// —— WAF Cookie 管理 ——

#[tauri::command]
pub async fn list_waf_cookies(_state: State<'_, AppState>) -> Result<Value, String> {
    run_blocking(|| {
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

        Ok(serde_json::json!({
            "waf_cookies": waf_entries,
            "total": waf_entries.len()
        }))
    })
    .await
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

    run_blocking(move || {
        let waf_manager = WafCookieManager::new();
        waf_manager
            .save(&provider_id, cookies.clone())
            .map_err(|e| format!("Failed to save WAF cookie: {}", e))?;

        Ok(serde_json::json!({
            "provider_id": provider_id,
            "cookies_count": cookies.len(),
            "message": "WAF cookies saved successfully"
        }))
    })
    .await
}

#[tauri::command]
pub async fn delete_waf_cookie(_state: State<'_, AppState>, id: String) -> Result<String, String> {
    run_blocking(move || {
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
    })
    .await
}

// —— 内置 Provider 与导入导出 ——

#[tauri::command]
pub async fn list_builtin_providers() -> Result<Value, String> {
    run_blocking(|| {
        use ccr_db::managers::checkin::builtin_providers::get_builtin_providers;
        let providers = get_builtin_providers();
        let total = providers.len();
        Ok(serde_json::json!({
            "providers": providers,
            "total": total,
        }))
    })
    .await
}

#[tauri::command]
pub async fn add_builtin_provider(provider_id: String) -> Result<Value, String> {
    run_blocking(move || {
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
    })
    .await
}

#[tauri::command]
pub async fn get_checkin_account_cookies(
    _state: State<'_, AppState>,
    account_id: String,
) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    run_blocking(move || {
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
    })
    .await
}

#[tauri::command]
pub async fn export_checkin_config(options: Option<Value>) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let opts: ccr_db::models::checkin::ExportOptions = if let Some(v) = options {
        serde_json::from_value(v).map_err(|e| format!("Invalid export options: {}", e))?
    } else {
        ccr_db::models::checkin::ExportOptions::default()
    };
    run_blocking(move || {
        let export_manager = ExportManager::new(&checkin_dir);
        let data = export_manager
            .export(&opts)
            .map_err(|e| format!("Failed to export config: {}", e))?;
        serde_json::to_value(data).map_err(|e| format!("Serialization error: {}", e))
    })
    .await
}

#[tauri::command]
pub async fn preview_checkin_import(data: Value) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    let export_data: ccr_db::models::checkin::ExportData =
        serde_json::from_value(data).map_err(|e| format!("Invalid import data: {}", e))?;
    run_blocking(move || {
        let export_manager = ExportManager::new(&checkin_dir);
        let preview = export_manager
            .preview_import(&export_data)
            .map_err(|e| format!("Failed to preview import: {}", e))?;
        serde_json::to_value(preview).map_err(|e| format!("Serialization error: {}", e))
    })
    .await
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
    run_blocking(move || {
        let export_manager = ExportManager::new(&checkin_dir);
        let result = export_manager
            .import(export_data, &import_opts)
            .map_err(|e| format!("Failed to import config: {}", e))?;
        serde_json::to_value(result).map_err(|e| format!("Serialization error: {}", e))
    })
    .await
}

// —— Dashboard ——

#[tauri::command]
pub async fn get_account_dashboard(
    _state: State<'_, AppState>,
    account_id: String,
    year: Option<i32>,
    month: Option<u32>,
    days: Option<u32>,
) -> Result<Value, String> {
    let checkin_dir = checkin_dir_str()?;
    run_blocking(move || {
        let service = CheckinService::new(checkin_dir);

        let now = chrono::Local::now();
        let y = year.unwrap_or(now.year());
        let m = month.unwrap_or(now.month());
        let d = days.unwrap_or(30);

        let dashboard = service
            .get_account_dashboard(&account_id, y, m, d)
            .map_err(|e| format!("Failed to get dashboard: {}", e))?;

        serde_json::to_value(dashboard).map_err(|e| format!("Serialization error: {}", e))
    })
    .await
}
