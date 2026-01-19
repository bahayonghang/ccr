// 📝 签到管理 API 处理器
// 提供中转站签到功能的 Web API

use axum::{
    extract::{Json, Path, Query},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::managers::checkin::{
    AccountManager, BalanceManager, ExportManager, ProviderManager, RecordManager,
    builtin_providers::{BuiltinProvider, get_builtin_providers},
};
use crate::models::checkin::{
    AccountInfo, AccountsResponse, BalanceHistoryResponse, BalanceResponse,
    CheckinAccountDashboardResponse, CheckinProvider, CheckinRecord, CheckinRecordInfo,
    CheckinRecordsResponse, CheckinStatus, CreateAccountRequest, CreateProviderRequest, ExportData,
    ExportOptions, ImportOptions, ImportPreviewResponse, ImportResult, ProvidersResponse,
    UpdateAccountRequest, UpdateProviderRequest,
};
use crate::services::checkin_service::{CheckinExecutionResult, CheckinService, TodayCheckinStats};

// ============================================================
// 辅助函数
// ============================================================

/// 获取签到数据目录
#[allow(clippy::result_large_err)]
fn get_checkin_dir() -> Result<std::path::PathBuf, Response> {
    CheckinService::default_checkin_dir().map_err(internal_error)
}

/// 将错误转换为内部服务器错误响应
fn internal_error<E: std::fmt::Display>(err: E) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Internal server error: {}", err),
    )
        .into_response()
}

/// 将错误转换为 404 响应
fn not_found_error<E: std::fmt::Display>(err: E) -> Response {
    (StatusCode::NOT_FOUND, format!("Not found: {}", err)).into_response()
}

/// 将错误转换为 400 响应
fn bad_request_error<E: std::fmt::Display>(err: E) -> Response {
    (StatusCode::BAD_REQUEST, format!("Bad request: {}", err)).into_response()
}

#[allow(clippy::result_large_err)]
fn enrich_accounts(
    accounts: &mut [AccountInfo],
    provider_manager: &ProviderManager,
    balance_manager: &BalanceManager,
) -> Result<(), Response> {
    let providers = provider_manager.load_all().map_err(internal_error)?;
    let provider_map: HashMap<String, String> =
        providers.into_iter().map(|p| (p.id, p.name)).collect();
    let balance_map = balance_manager.get_latest_map().map_err(internal_error)?;

    for account in accounts.iter_mut() {
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

    Ok(())
}

// ============================================================
// 提供商 API
// ============================================================

/// GET /api/checkin/providers - 获取所有提供商
pub async fn list_providers() -> Result<Json<ProvidersResponse>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let manager = ProviderManager::new(&checkin_dir);

    let response = manager.list().map_err(internal_error)?;
    Ok(Json(response))
}

/// GET /api/checkin/providers/:id - 获取单个提供商
pub async fn get_provider(Path(id): Path<String>) -> Result<Json<CheckinProvider>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let manager = ProviderManager::new(&checkin_dir);

    let provider = manager.get(&id).map_err(not_found_error)?;
    Ok(Json(provider))
}

/// POST /api/checkin/providers - 创建提供商
pub async fn create_provider(
    Json(req): Json<CreateProviderRequest>,
) -> Result<Json<CheckinProvider>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let manager = ProviderManager::new(&checkin_dir);

    let provider = manager.create(req).map_err(bad_request_error)?;
    Ok(Json(provider))
}

/// PUT /api/checkin/providers/:id - 更新提供商
pub async fn update_provider(
    Path(id): Path<String>,
    Json(req): Json<UpdateProviderRequest>,
) -> Result<Json<CheckinProvider>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let manager = ProviderManager::new(&checkin_dir);

    let provider = manager.update(&id, req).map_err(bad_request_error)?;
    Ok(Json(provider))
}

/// DELETE /api/checkin/providers/:id - 删除提供商
pub async fn delete_provider(Path(id): Path<String>) -> Result<StatusCode, Response> {
    let checkin_dir = get_checkin_dir()?;
    let provider_manager = ProviderManager::new(&checkin_dir);
    let account_manager = AccountManager::new(&checkin_dir);

    // 检查是否有关联账号
    let has_accounts = account_manager
        .has_accounts_for_provider(&id)
        .map_err(internal_error)?;

    provider_manager
        .delete(&id, has_accounts)
        .map_err(bad_request_error)?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================
// 内置提供商 API
// ============================================================

/// 内置提供商列表响应
#[derive(Debug, Serialize)]
pub struct BuiltinProvidersResponse {
    pub providers: Vec<BuiltinProvider>,
    pub total: usize,
}

/// GET /api/checkin/providers/builtin - 获取所有内置提供商
pub async fn list_builtin_providers() -> Json<BuiltinProvidersResponse> {
    let providers = get_builtin_providers();
    let total = providers.len();
    Json(BuiltinProvidersResponse { providers, total })
}

/// 添加内置提供商请求
#[derive(Debug, Deserialize)]
pub struct AddBuiltinProviderRequest {
    pub builtin_id: String,
}

/// POST /api/checkin/providers/builtin/add - 添加内置提供商到用户配置
pub async fn add_builtin_provider(
    Json(req): Json<AddBuiltinProviderRequest>,
) -> Result<Json<CheckinProvider>, Response> {
    use crate::managers::checkin::builtin_providers::get_builtin_provider_by_id;

    let checkin_dir = get_checkin_dir()?;
    let manager = ProviderManager::new(&checkin_dir);

    // 获取内置提供商配置
    let builtin = get_builtin_provider_by_id(&req.builtin_id)
        .ok_or_else(|| bad_request_error(format!("内置提供商不存在: {}", req.builtin_id)))?;

    // 检查是否已存在同名提供商
    let existing = manager.list().map_err(internal_error)?;
    if existing.providers.iter().any(|p| p.name == builtin.name) {
        return Err(bad_request_error(format!("提供商 {} 已存在", builtin.name)));
    }

    // 创建提供商 (使用内置配置转换)
    let create_req = CreateProviderRequest {
        name: builtin.name.clone(),
        base_url: builtin.base_url.clone(),
        checkin_path: builtin.checkin_path.clone(),
        balance_path: Some(builtin.balance_path.clone()),
        user_info_path: Some(builtin.user_info_path.clone()),
        auth_header: Some(builtin.auth_header.clone()),
        auth_prefix: Some(builtin.auth_prefix.clone()),
    };

    let provider = manager.create(create_req).map_err(bad_request_error)?;
    Ok(Json(provider))
}

// ============================================================
// 账号 API
// ============================================================

/// 账号列表查询参数
#[derive(Debug, Deserialize)]
pub struct AccountsQuery {
    pub provider_id: Option<String>,
}

/// GET /api/checkin/accounts - 获取所有账号
pub async fn list_accounts(
    Query(query): Query<AccountsQuery>,
) -> Result<Json<AccountsResponse>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let manager = AccountManager::new(&checkin_dir);
    let provider_manager = ProviderManager::new(&checkin_dir);
    let balance_manager = BalanceManager::new(&checkin_dir);

    let response = if let Some(provider_id) = query.provider_id {
        let mut accounts = manager
            .list_by_provider(&provider_id)
            .map_err(internal_error)?;
        enrich_accounts(&mut accounts, &provider_manager, &balance_manager)?;
        AccountsResponse {
            total: accounts.len(),
            accounts,
        }
    } else {
        let mut response = manager.list().map_err(internal_error)?;
        enrich_accounts(&mut response.accounts, &provider_manager, &balance_manager)?;
        response
    };

    Ok(Json(response))
}

/// GET /api/checkin/accounts/:id - 获取单个账号
pub async fn get_account(Path(id): Path<String>) -> Result<Json<AccountInfo>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let manager = AccountManager::new(&checkin_dir);
    let provider_manager = ProviderManager::new(&checkin_dir);
    let balance_manager = BalanceManager::new(&checkin_dir);

    let mut account = manager.get_info(&id).map_err(not_found_error)?;
    enrich_accounts(
        std::slice::from_mut(&mut account),
        &provider_manager,
        &balance_manager,
    )?;
    Ok(Json(account))
}

/// 账号 Dashboard 查询参数
#[derive(Debug, Deserialize)]
pub struct AccountDashboardQuery {
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub days: Option<u32>,
}

/// GET /api/checkin/accounts/:id/dashboard - 获取账号 Dashboard 数据
pub async fn get_account_dashboard(
    Path(id): Path<String>,
    Query(query): Query<AccountDashboardQuery>,
) -> Result<Json<CheckinAccountDashboardResponse>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let service = CheckinService::new(checkin_dir);

    let now = Utc::now().date_naive();
    let year = query.year.unwrap_or(now.year());
    let month = query.month.unwrap_or(now.month());
    let days = query.days.unwrap_or(30);

    if !(1..=12).contains(&month) {
        return Err(bad_request_error("Invalid month"));
    }

    if !(1..=365).contains(&days) {
        return Err(bad_request_error("Invalid days"));
    }

    let dashboard = service
        .get_account_dashboard(&id, year, month, days)
        .map_err(|e| match e {
            crate::services::checkin_service::CheckinServiceError::AccountError(_) => {
                not_found_error(e)
            }
            _ => internal_error(e),
        })?;

    Ok(Json(dashboard))
}

/// POST /api/checkin/accounts - 创建账号
pub async fn create_account(
    Json(req): Json<CreateAccountRequest>,
) -> Result<Json<AccountInfo>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let manager = AccountManager::new(&checkin_dir);

    let account = manager.create(req).map_err(bad_request_error)?;
    // 转换为 AccountInfo（包含遮罩的 Cookies）
    let account_info = manager.get_info(&account.id).map_err(internal_error)?;
    Ok(Json(account_info))
}

/// PUT /api/checkin/accounts/:id - 更新账号
pub async fn update_account(
    Path(id): Path<String>,
    Json(req): Json<UpdateAccountRequest>,
) -> Result<Json<AccountInfo>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let manager = AccountManager::new(&checkin_dir);

    let account = manager.update(&id, req).map_err(bad_request_error)?;
    // 转换为 AccountInfo（包含遮罩的 Cookies）
    let account_info = manager.get_info(&account.id).map_err(internal_error)?;
    Ok(Json(account_info))
}

/// DELETE /api/checkin/accounts/:id - 删除账号
pub async fn delete_account(Path(id): Path<String>) -> Result<StatusCode, Response> {
    let checkin_dir = get_checkin_dir()?;
    let account_manager = AccountManager::new(&checkin_dir);
    let record_manager = RecordManager::new(&checkin_dir);
    let balance_manager = BalanceManager::new(&checkin_dir);

    // 删除账号
    account_manager.delete(&id).map_err(not_found_error)?;

    // 删除关联的签到记录和余额记录
    let _ = record_manager.delete_by_account(&id);
    let _ = balance_manager.delete_by_account(&id);

    Ok(StatusCode::NO_CONTENT)
}

/// 获取 Cookies 响应（用于编辑）
#[derive(Debug, Serialize)]
pub struct AccountCookiesResponse {
    pub cookies_json: String,
    pub api_user: String,
}

/// GET /api/checkin/accounts/:id/cookies - 获取账号的解密后 Cookies（用于编辑）
pub async fn get_account_cookies(
    Path(id): Path<String>,
) -> Result<Json<AccountCookiesResponse>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let manager = AccountManager::new(&checkin_dir);

    let (cookies_json, api_user) = manager.get_cookies_json(&id).map_err(not_found_error)?;
    Ok(Json(AccountCookiesResponse {
        cookies_json,
        api_user,
    }))
}

// ============================================================
// 签到操作 API
// ============================================================

/// 签到请求
#[derive(Debug, Deserialize)]
pub struct CheckinRequest {
    pub account_ids: Option<Vec<String>>,
}

/// 签到响应
#[derive(Debug, Serialize)]
pub struct CheckinResponse {
    pub results: Vec<CheckinExecutionResult>,
    pub summary: CheckinSummary,
}

/// 签到汇总
#[derive(Debug, Serialize)]
pub struct CheckinSummary {
    pub total: usize,
    pub success: usize,
    pub already_checked_in: usize,
    pub failed: usize,
}

/// POST /api/checkin/execute - 执行签到
pub async fn execute_checkin(
    Json(req): Json<CheckinRequest>,
) -> Result<Json<CheckinResponse>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let service = CheckinService::new(checkin_dir);

    let results = if let Some(account_ids) = req.account_ids {
        service.batch_checkin(&account_ids).await
    } else {
        service.checkin_all().await
    };

    // 统计结果
    let mut success = 0;
    let mut already_checked_in = 0;
    let mut failed = 0;

    for result in &results {
        match result.status {
            crate::models::checkin::CheckinStatus::Success => success += 1,
            crate::models::checkin::CheckinStatus::AlreadyCheckedIn => already_checked_in += 1,
            crate::models::checkin::CheckinStatus::Failed => failed += 1,
        }
    }

    let response = CheckinResponse {
        summary: CheckinSummary {
            total: results.len(),
            success,
            already_checked_in,
            failed,
        },
        results,
    };

    Ok(Json(response))
}

/// POST /api/checkin/accounts/:id/checkin - 执行单个账号签到
pub async fn checkin_account(
    Path(id): Path<String>,
) -> Result<Json<CheckinExecutionResult>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let service = CheckinService::new(checkin_dir);

    let result = service.checkin(&id).await.map_err(internal_error)?;
    Ok(Json(result))
}

// ============================================================
// 余额查询 API
// ============================================================

/// POST /api/checkin/accounts/:id/balance - 查询账号余额
pub async fn query_balance(Path(id): Path<String>) -> Result<Json<BalanceResponse>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let service = CheckinService::new(checkin_dir);

    let snapshot = service.query_balance(&id).await.map_err(internal_error)?;
    let response: BalanceResponse = snapshot.into();
    Ok(Json(response))
}

/// 历史记录查询参数
#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<usize>,
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub status: Option<String>,
    pub account_id: Option<String>,
    pub provider_id: Option<String>,
    pub keyword: Option<String>,
}

const DEFAULT_RECORDS_PAGE_SIZE: usize = 100;
const MAX_RECORDS_PAGE_SIZE: usize = 500;

fn parse_status_filter(value: &str) -> Option<CheckinStatus> {
    let normalized = value.trim().replace(['-', ' '], "_").to_lowercase();
    match normalized.as_str() {
        "success" => Some(CheckinStatus::Success),
        "already_checked_in" | "alreadycheckedin" | "checked_in" | "checkedin" => {
            Some(CheckinStatus::AlreadyCheckedIn)
        }
        "failed" | "failure" | "error" => Some(CheckinStatus::Failed),
        _ => None,
    }
}

fn filter_checkin_records(
    records: Vec<CheckinRecord>,
    status_filter: Option<CheckinStatus>,
    account_id_filter: Option<&str>,
    provider_account_ids: Option<&HashSet<String>>,
    keyword_filter: Option<&str>,
    account_name_map: &HashMap<String, String>,
    account_provider_name_map: &HashMap<String, String>,
) -> Vec<CheckinRecord> {
    let keyword = keyword_filter
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty());

    records
        .into_iter()
        .filter(|record| {
            if let Some(status) = status_filter
                && record.status != status
            {
                return false;
            }

            if let Some(account_id) = account_id_filter
                && record.account_id != account_id
            {
                return false;
            }

            if let Some(account_ids) = provider_account_ids
                && !account_ids.contains(&record.account_id)
            {
                return false;
            }

            if let Some(ref keyword) = keyword {
                let mut matched = record.account_id.to_lowercase().contains(keyword);
                if !matched && let Some(message) = record.message.as_ref() {
                    matched = message.to_lowercase().contains(keyword);
                }
                if !matched && let Some(name) = account_name_map.get(&record.account_id) {
                    matched = name.to_lowercase().contains(keyword);
                }
                if !matched && let Some(name) = account_provider_name_map.get(&record.account_id) {
                    matched = name.to_lowercase().contains(keyword);
                }
                if !matched {
                    return false;
                }
            }

            true
        })
        .collect()
}

fn paginate_checkin_records(
    mut records: Vec<CheckinRecord>,
    query: &HistoryQuery,
) -> (Vec<CheckinRecord>, usize) {
    records.sort_by(|a, b| b.checked_in_at.cmp(&a.checked_in_at));
    let total = records.len();

    let use_limit = query.limit.is_some() && query.page.is_none() && query.page_size.is_none();
    if use_limit {
        if let Some(limit) = query.limit {
            records.truncate(limit);
        }
        return (records, total);
    }

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query
        .page_size
        .unwrap_or(DEFAULT_RECORDS_PAGE_SIZE)
        .clamp(1, MAX_RECORDS_PAGE_SIZE);
    let start = (page - 1) * page_size;
    if start >= records.len() {
        return (Vec::new(), total);
    }
    let end = (start + page_size).min(records.len());
    let paged = records.drain(start..end).collect();
    (paged, total)
}

fn build_record_response(
    records: Vec<CheckinRecord>,
    total: usize,
    account_name_map: &HashMap<String, String>,
    account_provider_name_map: &HashMap<String, String>,
) -> CheckinRecordsResponse {
    let mut record_infos: Vec<CheckinRecordInfo> = records.into_iter().map(|r| r.into()).collect();
    for record in record_infos.iter_mut() {
        if let Some(name) = account_name_map.get(&record.account_id) {
            record.account_name = Some(name.clone());
        }
        if let Some(name) = account_provider_name_map.get(&record.account_id) {
            record.provider_name = Some(name.clone());
        }
    }
    CheckinRecordsResponse {
        records: record_infos,
        total,
    }
}

/// GET /api/checkin/accounts/:id/balance/history - 获取余额历史
pub async fn get_balance_history(
    Path(id): Path<String>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<BalanceHistoryResponse>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let service = CheckinService::new(checkin_dir);

    let history = service
        .get_balance_history(&id, query.limit)
        .map_err(internal_error)?;
    Ok(Json(history))
}

// ============================================================
// 签到记录 API
// ============================================================

/// GET /api/checkin/records - 获取所有签到记录
pub async fn list_records(
    Query(query): Query<HistoryQuery>,
) -> Result<Json<CheckinRecordsResponse>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let record_manager = RecordManager::new(&checkin_dir);
    let account_manager = AccountManager::new(&checkin_dir);
    let provider_manager = ProviderManager::new(&checkin_dir);

    let accounts = account_manager.load_all().map_err(internal_error)?;
    let providers = provider_manager.load_all().map_err(internal_error)?;
    let provider_map: HashMap<String, String> =
        providers.into_iter().map(|p| (p.id, p.name)).collect();

    let mut account_name_map = HashMap::new();
    let mut account_provider_name_map = HashMap::new();
    let mut provider_account_ids: Option<HashSet<String>> =
        query.provider_id.as_ref().map(|_| HashSet::new());

    for account in accounts {
        account_name_map.insert(account.id.clone(), account.name.clone());
        if let Some(provider_name) = provider_map.get(&account.provider_id) {
            account_provider_name_map.insert(account.id.clone(), provider_name.clone());
        }
        if let (Some(provider_id), Some(ref mut id_set)) =
            (query.provider_id.as_ref(), provider_account_ids.as_mut())
            && &account.provider_id == provider_id
        {
            id_set.insert(account.id.clone());
        }
    }

    let status_filter = match query.status.as_deref() {
        Some(value) => Some(
            parse_status_filter(value).ok_or_else(|| bad_request_error("Invalid status filter"))?,
        ),
        None => None,
    };

    let records = record_manager.get_all_raw().map_err(internal_error)?;
    let filtered = filter_checkin_records(
        records,
        status_filter,
        query.account_id.as_deref(),
        provider_account_ids.as_ref(),
        query.keyword.as_deref(),
        &account_name_map,
        &account_provider_name_map,
    );

    let (paged, total) = paginate_checkin_records(filtered, &query);
    let response =
        build_record_response(paged, total, &account_name_map, &account_provider_name_map);
    Ok(Json(response))
}

/// GET /api/checkin/accounts/:id/records - 获取账号签到记录
pub async fn get_account_records(
    Path(id): Path<String>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<CheckinRecordsResponse>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let record_manager = RecordManager::new(&checkin_dir);
    let account_manager = AccountManager::new(&checkin_dir);
    let provider_manager = ProviderManager::new(&checkin_dir);

    let accounts = account_manager.load_all().map_err(internal_error)?;
    let providers = provider_manager.load_all().map_err(internal_error)?;
    let provider_map: HashMap<String, String> =
        providers.into_iter().map(|p| (p.id, p.name)).collect();

    let mut account_name_map = HashMap::new();
    let mut account_provider_name_map = HashMap::new();

    for account in accounts {
        account_name_map.insert(account.id.clone(), account.name.clone());
        if let Some(provider_name) = provider_map.get(&account.provider_id) {
            account_provider_name_map.insert(account.id.clone(), provider_name.clone());
        }
    }

    let status_filter = match query.status.as_deref() {
        Some(value) => Some(
            parse_status_filter(value).ok_or_else(|| bad_request_error("Invalid status filter"))?,
        ),
        None => None,
    };

    let records = record_manager.get_all_raw().map_err(internal_error)?;
    let filtered = filter_checkin_records(
        records,
        status_filter,
        Some(&id),
        None,
        query.keyword.as_deref(),
        &account_name_map,
        &account_provider_name_map,
    );

    let (paged, total) = paginate_checkin_records(filtered, &query);
    let response =
        build_record_response(paged, total, &account_name_map, &account_provider_name_map);
    Ok(Json(response))
}

/// GET /api/checkin/records/export - 导出签到记录
pub async fn export_records(Query(query): Query<HistoryQuery>) -> Result<Response, Response> {
    let checkin_dir = get_checkin_dir()?;
    let record_manager = RecordManager::new(&checkin_dir);
    let account_manager = AccountManager::new(&checkin_dir);
    let provider_manager = ProviderManager::new(&checkin_dir);

    let accounts = account_manager.load_all().map_err(internal_error)?;
    let providers = provider_manager.load_all().map_err(internal_error)?;
    let provider_map: HashMap<String, String> =
        providers.into_iter().map(|p| (p.id, p.name)).collect();

    let mut account_name_map = HashMap::new();
    let mut account_provider_name_map = HashMap::new();
    let mut provider_account_ids: Option<HashSet<String>> =
        query.provider_id.as_ref().map(|_| HashSet::new());

    for account in accounts {
        account_name_map.insert(account.id.clone(), account.name.clone());
        if let Some(provider_name) = provider_map.get(&account.provider_id) {
            account_provider_name_map.insert(account.id.clone(), provider_name.clone());
        }
        if let (Some(provider_id), Some(ref mut id_set)) =
            (query.provider_id.as_ref(), provider_account_ids.as_mut())
            && &account.provider_id == provider_id
        {
            id_set.insert(account.id.clone());
        }
    }

    let status_filter = match query.status.as_deref() {
        Some(value) => Some(
            parse_status_filter(value).ok_or_else(|| bad_request_error("Invalid status filter"))?,
        ),
        None => None,
    };

    let records = record_manager.get_all_raw().map_err(internal_error)?;
    let filtered = filter_checkin_records(
        records,
        status_filter,
        query.account_id.as_deref(),
        provider_account_ids.as_ref(),
        query.keyword.as_deref(),
        &account_name_map,
        &account_provider_name_map,
    );

    let total = filtered.len();
    let response_data = build_record_response(
        filtered,
        total,
        &account_name_map,
        &account_provider_name_map,
    );
    let content = serde_json::to_string_pretty(&response_data).map_err(internal_error)?;
    let filename = format!(
        "checkin_records_{}.json",
        Utc::now().format("%Y%m%d_%H%M%S")
    );

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(content.into())
        .map_err(internal_error)?;

    Ok(response)
}

// ============================================================
// 统计 API
// ============================================================

/// GET /api/checkin/stats/today - 获取今日签到统计
pub async fn get_today_stats() -> Result<Json<TodayCheckinStats>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let service = CheckinService::new(checkin_dir);

    let stats = service.get_today_stats().map_err(internal_error)?;
    Ok(Json(stats))
}

// ============================================================
// 导入/导出 API
// ============================================================

/// POST /api/checkin/export - 导出配置
pub async fn export_config(
    Json(options): Json<ExportOptions>,
) -> Result<Json<ExportData>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let manager = ExportManager::new(&checkin_dir);

    let data = manager.export(&options).map_err(internal_error)?;
    Ok(Json(data))
}

/// POST /api/checkin/import/preview - 预览导入
pub async fn preview_import(
    Json(data): Json<ExportData>,
) -> Result<Json<ImportPreviewResponse>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let manager = ExportManager::new(&checkin_dir);

    let preview = manager.preview_import(&data).map_err(internal_error)?;
    Ok(Json(preview))
}

/// 导入请求
#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    pub data: ExportData,
    pub options: ImportOptions,
}

/// POST /api/checkin/import - 执行导入
pub async fn execute_import(
    Json(req): Json<ImportRequest>,
) -> Result<Json<ImportResult>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let manager = ExportManager::new(&checkin_dir);

    let result = manager
        .import(req.data, &req.options)
        .map_err(internal_error)?;
    Ok(Json(result))
}

// ============================================================
// 连接测试 API
// ============================================================

/// 连接测试响应
#[derive(Debug, Serialize)]
pub struct TestConnectionResponse {
    pub success: bool,
    pub message: String,
}

/// POST /api/checkin/accounts/:id/test - 测试账号连接
pub async fn test_connection(
    Path(id): Path<String>,
) -> Result<Json<TestConnectionResponse>, Response> {
    let checkin_dir = get_checkin_dir()?;
    let service = CheckinService::new(checkin_dir);

    match service.test_connection(&id).await {
        Ok(true) => Ok(Json(TestConnectionResponse {
            success: true,
            message: "连接成功".to_string(),
        })),
        Ok(false) => Ok(Json(TestConnectionResponse {
            success: false,
            message: "连接失败，请检查 Cookies 配置".to_string(),
        })),
        Err(e) => Ok(Json(TestConnectionResponse {
            success: false,
            message: format!("连接失败: {}", e),
        })),
    }
}

// ============================================================
// 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkin_summary_serialization() {
        let summary = CheckinSummary {
            total: 10,
            success: 7,
            already_checked_in: 2,
            failed: 1,
        };

        let json = serde_json::to_string(&summary).expect("Failed to serialize test summary");
        assert!(json.contains("\"total\":10"));
        assert!(json.contains("\"success\":7"));
    }

    #[test]
    fn test_accounts_query_deserialization() {
        let json = r#"{"provider_id": "test-provider"}"#;
        let query: AccountsQuery =
            serde_json::from_str(json).expect("Failed to deserialize test AccountsQuery");
        assert_eq!(query.provider_id, Some("test-provider".to_string()));
    }

    #[test]
    fn test_checkin_request_deserialization() {
        let json = r#"{"account_ids": ["acc-1", "acc-2"]}"#;
        let req: CheckinRequest =
            serde_json::from_str(json).expect("Failed to deserialize test CheckinRequest");
        assert_eq!(
            req.account_ids
                .expect("account_ids should exist in test")
                .len(),
            2
        );

        let json_empty = r#"{}"#;
        let req_empty: CheckinRequest = serde_json::from_str(json_empty)
            .expect("Failed to deserialize empty test CheckinRequest");
        assert!(req_empty.account_ids.is_none());
    }
}
