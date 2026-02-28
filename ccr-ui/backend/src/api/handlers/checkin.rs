// 📝 签到管理 API 处理器
// 提供中转站签到功能的 Web API

use axum::{
    extract::{Json, Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::{Datelike, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::managers::checkin::{
    AccountManager, BalanceManager, ExportManager, ProviderManager, RecordManager,
    builtin_providers::{BuiltinProvider, get_builtin_providers},
};
use crate::models::checkin::{
    AccountInfo, AccountsResponse, BalanceHistoryResponse, BalanceResponse,
    CheckinAccountDashboardResponse, CheckinProvider, CheckinRecord, CheckinRecordInfo,
    CheckinRecordsResponse, CreateAccountRequest, CreateProviderRequest, ExportData, ExportOptions,
    ImportOptions, ImportPreviewResponse, ImportResult, ProvidersResponse, UpdateAccountRequest,
    UpdateProviderRequest,
};
use crate::services::cdk_service::{CdkExtraConfig, CdkService, CdkTopupResult};
use crate::services::checkin_service::{CheckinExecutionResult, CheckinService, TodayCheckinStats};
use crate::state::AppState;

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

// ============================================================
// 缓存层
// ============================================================

#[derive(Clone)]
struct CacheEntry<T> {
    expires_at: Instant,
    value: T,
}

/// 可缓存的记录名称映射（账号名与提供商名）
#[derive(Clone)]
struct RecordNameMapsCache {
    account_name_map: HashMap<String, String>,
    account_provider_name_map: HashMap<String, String>,
}

type ProviderMapCache = Lazy<RwLock<Option<CacheEntry<HashMap<String, String>>>>>;
type BalanceMapCache =
    Lazy<RwLock<Option<CacheEntry<HashMap<String, crate::models::checkin::BalanceSnapshot>>>>>;
type NameMapsCache = Lazy<RwLock<Option<CacheEntry<RecordNameMapsCache>>>>;

/// Provider name map 缓存 (30s TTL)
static PROVIDER_MAP_CACHE: ProviderMapCache = Lazy::new(|| RwLock::new(None));
const PROVIDER_MAP_TTL: Duration = Duration::from_secs(30);

/// Balance map 缓存 (10s TTL)
static BALANCE_MAP_CACHE: BalanceMapCache = Lazy::new(|| RwLock::new(None));
const BALANCE_MAP_TTL: Duration = Duration::from_secs(10);

/// Record name maps 缓存 (30s TTL)
static NAME_MAPS_CACHE: NameMapsCache = Lazy::new(|| RwLock::new(None));
const NAME_MAPS_TTL: Duration = Duration::from_secs(30);

/// 使 provider map 缓存失效
async fn invalidate_provider_cache() {
    *PROVIDER_MAP_CACHE.write().await = None;
}

/// 使 balance map 缓存失效
async fn invalidate_balance_cache() {
    *BALANCE_MAP_CACHE.write().await = None;
}

/// 使 name maps 缓存失效
async fn invalidate_name_maps_cache() {
    *NAME_MAPS_CACHE.write().await = None;
}

/// 使所有签到相关缓存失效
async fn invalidate_all_checkin_caches() {
    invalidate_provider_cache().await;
    invalidate_balance_cache().await;
    invalidate_name_maps_cache().await;
}

#[allow(clippy::result_large_err)]
fn enrich_accounts(
    accounts: &mut [AccountInfo],
    provider_manager: &ProviderManager,
    balance_manager: &BalanceManager,
) -> Result<(), Response> {
    let provider_map = get_provider_map_cached(provider_manager)?;
    let balance_map = get_balance_map_cached(balance_manager)?;

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

/// 获取 provider name map（带缓存）
#[allow(clippy::result_large_err)]
fn get_provider_map_cached(
    provider_manager: &ProviderManager,
) -> Result<HashMap<String, String>, Response> {
    // 同步上下文中使用 try_read 检查缓存
    if let Ok(cache) = PROVIDER_MAP_CACHE.try_read()
        && let Some(entry) = cache.as_ref()
        && Instant::now() < entry.expires_at
    {
        return Ok(entry.value.clone());
    }

    let providers = provider_manager.load_all().map_err(internal_error)?;
    let map: HashMap<String, String> = providers.into_iter().map(|p| (p.id, p.name)).collect();

    // 尝试写入缓存（非阻塞）
    if let Ok(mut cache) = PROVIDER_MAP_CACHE.try_write() {
        *cache = Some(CacheEntry {
            expires_at: Instant::now() + PROVIDER_MAP_TTL,
            value: map.clone(),
        });
    }

    Ok(map)
}

/// 获取 balance map（带缓存）
#[allow(clippy::result_large_err)]
fn get_balance_map_cached(
    balance_manager: &BalanceManager,
) -> Result<HashMap<String, crate::models::checkin::BalanceSnapshot>, Response> {
    if let Ok(cache) = BALANCE_MAP_CACHE.try_read()
        && let Some(entry) = cache.as_ref()
        && Instant::now() < entry.expires_at
    {
        return Ok(entry.value.clone());
    }

    let map = balance_manager.get_latest_map().map_err(internal_error)?;

    if let Ok(mut cache) = BALANCE_MAP_CACHE.try_write() {
        *cache = Some(CacheEntry {
            expires_at: Instant::now() + BALANCE_MAP_TTL,
            value: map.clone(),
        });
    }

    Ok(map)
}

// ============================================================
// 提供商 API
// ============================================================

/// GET /api/checkin/providers - 获取所有提供商
pub async fn list_providers() -> Result<Json<ProvidersResponse>, Response> {
    let manager = ProviderManager::new();

    let response = manager.list().map_err(internal_error)?;
    Ok(Json(response))
}

/// GET /api/checkin/providers/:id - 获取单个提供商
pub async fn get_provider(Path(id): Path<String>) -> Result<Json<CheckinProvider>, Response> {
    let manager = ProviderManager::new();

    let provider = manager.get(&id).map_err(not_found_error)?;
    Ok(Json(provider))
}

/// POST /api/checkin/providers - 创建提供商
pub async fn create_provider(
    Json(req): Json<CreateProviderRequest>,
) -> Result<Json<CheckinProvider>, Response> {
    let manager = ProviderManager::new();

    let provider = manager.create(req).map_err(bad_request_error)?;
    invalidate_provider_cache().await;
    invalidate_name_maps_cache().await;
    Ok(Json(provider))
}

/// PUT /api/checkin/providers/:id - 更新提供商
pub async fn update_provider(
    Path(id): Path<String>,
    Json(req): Json<UpdateProviderRequest>,
) -> Result<Json<CheckinProvider>, Response> {
    let manager = ProviderManager::new();

    let provider = manager.update(&id, req).map_err(bad_request_error)?;
    invalidate_provider_cache().await;
    invalidate_name_maps_cache().await;
    Ok(Json(provider))
}

/// DELETE /api/checkin/providers/:id - 删除提供商
pub async fn delete_provider(Path(id): Path<String>) -> Result<StatusCode, Response> {
    let checkin_dir = get_checkin_dir()?;
    let provider_manager = ProviderManager::new();
    let account_manager = AccountManager::new(&checkin_dir);

    // 检查是否有关联账号
    let has_accounts = account_manager
        .has_accounts_for_provider(&id)
        .map_err(internal_error)?;

    provider_manager
        .delete(&id, has_accounts)
        .map_err(bad_request_error)?;

    invalidate_all_checkin_caches().await;
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

    let manager = ProviderManager::new();

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
    let provider_manager = ProviderManager::new();
    let balance_manager = BalanceManager::new();

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
    let provider_manager = ProviderManager::new();
    let balance_manager = BalanceManager::new();

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
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<AccountDashboardQuery>,
) -> Result<Json<CheckinAccountDashboardResponse>, Response> {
    let service = state.checkin_service();

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
    invalidate_name_maps_cache().await;
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
    invalidate_name_maps_cache().await;
    Ok(Json(account_info))
}

/// DELETE /api/checkin/accounts/:id - 删除账号
pub async fn delete_account(Path(id): Path<String>) -> Result<StatusCode, Response> {
    let checkin_dir = get_checkin_dir()?;
    let account_manager = AccountManager::new(&checkin_dir);
    let record_manager = RecordManager::new();
    let balance_manager = BalanceManager::new();

    // 删除账号
    account_manager.delete(&id).map_err(not_found_error)?;

    // 删除关联的签到记录和余额记录
    let _ = record_manager.delete_by_account(&id);
    let _ = balance_manager.delete_by_account(&id);

    invalidate_name_maps_cache().await;
    invalidate_balance_cache().await;
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
    State(state): State<AppState>,
    Json(req): Json<CheckinRequest>,
) -> Result<Json<CheckinResponse>, Response> {
    let service = state.checkin_service();

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

    invalidate_balance_cache().await;

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
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<CheckinExecutionResult>, Response> {
    let service = state.checkin_service();

    let result = service.checkin(&id).await.map_err(internal_error)?;
    invalidate_balance_cache().await;
    Ok(Json(result))
}

// ============================================================
// 余额查询 API
// ============================================================

/// POST /api/checkin/accounts/:id/balance - 查询账号余额
pub async fn query_balance(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<BalanceResponse>, Response> {
    let service = state.checkin_service();

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

const DEFAULT_RECORDS_PAGE_SIZE: usize = 20;
const MAX_RECORDS_PAGE_SIZE: usize = 100;

fn parse_status_filter(value: &str) -> Option<&'static str> {
    let normalized = value.trim().replace(['-', ' '], "_").to_lowercase();
    match normalized.as_str() {
        "success" => Some("success"),
        "already_checked_in" | "alreadycheckedin" | "checked_in" | "checkedin" => {
            Some("already_checked_in")
        }
        "failed" | "failure" | "error" => Some("failed"),
        _ => None,
    }
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
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<BalanceHistoryResponse>, Response> {
    let service = state.checkin_service();

    let history = service
        .get_balance_history(&id, query.limit)
        .map_err(internal_error)?;
    Ok(Json(history))
}

// ============================================================
// 签到记录 API
// ============================================================

/// 构建签到记录的名称映射（去重辅助函数）
struct RecordNameMaps {
    account_name_map: HashMap<String, String>,
    account_provider_name_map: HashMap<String, String>,
}

#[allow(clippy::result_large_err)]
fn build_record_name_maps(checkin_dir: &std::path::Path) -> Result<RecordNameMaps, Response> {
    // 尝试从缓存获取基础映射
    let base = if let Ok(cache) = NAME_MAPS_CACHE.try_read()
        && let Some(entry) = cache.as_ref()
        && Instant::now() < entry.expires_at
    {
        Some(entry.value.clone())
    } else {
        None
    };

    let base = match base {
        Some(b) => b,
        None => {
            let account_manager = AccountManager::new(checkin_dir);
            let provider_manager = ProviderManager::new();

            let accounts = account_manager.load_all().map_err(internal_error)?;
            let providers = provider_manager.load_all().map_err(internal_error)?;
            let provider_map: HashMap<String, String> =
                providers.into_iter().map(|p| (p.id, p.name)).collect();

            let mut account_name_map = HashMap::new();
            let mut account_provider_name_map = HashMap::new();

            for account in &accounts {
                account_name_map.insert(account.id.clone(), account.name.clone());
                if let Some(provider_name) = provider_map.get(&account.provider_id) {
                    account_provider_name_map.insert(account.id.clone(), provider_name.clone());
                }
            }

            let new_base = RecordNameMapsCache {
                account_name_map,
                account_provider_name_map,
            };

            if let Ok(mut cache) = NAME_MAPS_CACHE.try_write() {
                *cache = Some(CacheEntry {
                    expires_at: Instant::now() + NAME_MAPS_TTL,
                    value: new_base.clone(),
                });
            }

            new_base
        }
    };

    Ok(RecordNameMaps {
        account_name_map: base.account_name_map,
        account_provider_name_map: base.account_provider_name_map,
    })
}

/// GET /api/checkin/records - 获取所有签到记录
pub async fn list_records(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<CheckinRecordsResponse>, Response> {
    let maps = build_record_name_maps(&state.checkin_dir)?;
    let record_manager = RecordManager::new();
    let status_str = match query.status.as_deref() {
        Some(value) => Some(
            parse_status_filter(value).ok_or_else(|| bad_request_error("Invalid status filter"))?,
        ),
        None => None,
    };
    let keyword = query
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let page = query.page.unwrap_or(1).max(1);
    let page_size = if query.limit.is_some() && query.page.is_none() && query.page_size.is_none() {
        query
            .limit
            .unwrap_or(DEFAULT_RECORDS_PAGE_SIZE)
            .clamp(1, MAX_RECORDS_PAGE_SIZE)
    } else {
        query
            .page_size
            .unwrap_or(DEFAULT_RECORDS_PAGE_SIZE)
            .clamp(1, MAX_RECORDS_PAGE_SIZE)
    };

    let (records, total) = record_manager
        .get_paginated_advanced(
            status_str,
            query.account_id.as_deref(),
            query.provider_id.as_deref(),
            keyword,
            page,
            page_size,
        )
        .map_err(internal_error)?;

    let response = build_record_response(
        records,
        total,
        &maps.account_name_map,
        &maps.account_provider_name_map,
    );
    Ok(Json(response))
}

/// GET /api/checkin/accounts/:id/records - 获取账号签到记录
pub async fn get_account_records(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<CheckinRecordsResponse>, Response> {
    let maps = build_record_name_maps(&state.checkin_dir)?;
    let record_manager = RecordManager::new();
    let status_str = match query.status.as_deref() {
        Some(value) => Some(
            parse_status_filter(value).ok_or_else(|| bad_request_error("Invalid status filter"))?,
        ),
        None => None,
    };
    let keyword = query
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());
    let page = query.page.unwrap_or(1).max(1);
    let page_size = if query.limit.is_some() && query.page.is_none() && query.page_size.is_none() {
        query
            .limit
            .unwrap_or(DEFAULT_RECORDS_PAGE_SIZE)
            .clamp(1, MAX_RECORDS_PAGE_SIZE)
    } else {
        query
            .page_size
            .unwrap_or(DEFAULT_RECORDS_PAGE_SIZE)
            .clamp(1, MAX_RECORDS_PAGE_SIZE)
    };

    let (records, total) = record_manager
        .get_paginated_advanced(status_str, Some(&id), None, keyword, page, page_size)
        .map_err(internal_error)?;

    let response = build_record_response(
        records,
        total,
        &maps.account_name_map,
        &maps.account_provider_name_map,
    );
    Ok(Json(response))
}

/// GET /api/checkin/records/export - 导出签到记录
pub async fn export_records(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>,
) -> Result<Response, Response> {
    let maps = build_record_name_maps(&state.checkin_dir)?;
    let record_manager = RecordManager::new();

    let status_str = match query.status.as_deref() {
        Some(value) => Some(
            parse_status_filter(value).ok_or_else(|| bad_request_error("Invalid status filter"))?,
        ),
        None => None,
    };
    let keyword = query
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty());

    let filtered = record_manager
        .get_filtered_advanced(
            status_str,
            query.account_id.as_deref(),
            query.provider_id.as_deref(),
            keyword,
        )
        .map_err(internal_error)?;

    let total = filtered.len();
    let response_data = build_record_response(
        filtered,
        total,
        &maps.account_name_map,
        &maps.account_provider_name_map,
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
pub async fn get_today_stats(
    State(state): State<AppState>,
) -> Result<Json<TodayCheckinStats>, Response> {
    let service = state.checkin_service();

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
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TestConnectionResponse>, Response> {
    let service = state.checkin_service();

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
// CDK 充值 API
// ============================================================

/// POST /api/checkin/accounts/:id/topup - 执行 CDK 充值
pub async fn execute_topup(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<CdkTopupResult>, Response> {
    let account_manager = AccountManager::new(&state.checkin_dir);
    let provider_manager = ProviderManager::new();

    // 获取账号
    let account = account_manager.get(&id).map_err(not_found_error)?;

    // 获取提供商
    let provider = provider_manager
        .get(&account.provider_id)
        .map_err(not_found_error)?;

    // 查找内置提供商的 CDK 配置
    use crate::managers::checkin::builtin_providers::get_builtin_providers;
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
            return Err(bad_request_error(format!(
                "Provider {} does not support CDK topup",
                provider.name
            )));
        }
    };

    // 解析账号的 extra_config
    let extra_config = CdkExtraConfig::from_json(&account.extra_config);

    // 解密 Cookies 用于 topup
    let crypto = crate::core::crypto::CryptoManager::new(&state.checkin_dir)
        .map_err(|e| internal_error(format!("Crypto error: {}", e)))?;
    let cookies_json = crypto
        .decrypt(&account.cookies_json_encrypted)
        .map_err(|e| internal_error(format!("Decrypt error: {}", e)))?;

    // 解析 cookies JSON 为 HashMap
    let topup_cookies: HashMap<String, String> =
        serde_json::from_str(&cookies_json).unwrap_or_default();

    // 构造 topup URL
    let topup_url = cdk_config
        .topup_path
        .as_ref()
        .map(|path| format!("{}{}", provider.base_url, path));

    // 创建 CDK 服务并执行
    let cdk_service = CdkService::new(None); // TODO: 从环境获取 proxy
    let result = cdk_service
        .fetch_and_topup(
            &cdk_config.cdk_type,
            &extra_config,
            topup_url.as_deref(),
            &topup_cookies,
            &account.api_user,
        )
        .await;

    Ok(Json(result))
}

// ============================================================
// OAuth 引导登录 API
// ============================================================

/// OAuth state 请求
#[derive(Debug, Deserialize)]
pub struct OAuthStateRequest {
    pub provider_id: String,
    pub oauth_type: String, // "github" | "linuxdo"
}

/// OAuth state 响应
#[derive(Debug, Serialize)]
pub struct OAuthStateResponse {
    pub success: bool,
    pub authorize_url: Option<String>,
    pub provider_name: String,
    pub oauth_type: String,
    pub message: Option<String>,
    /// 引导用户提取 cookies 的说明
    pub extraction_guide: Vec<String>,
}

/// POST /api/checkin/oauth/authorize-url - 获取 OAuth 授权 URL
/// 引导式 OAuth：构造授权 URL，用户在浏览器中完成登录后手动提取 cookies
pub async fn get_oauth_authorize_url(
    State(state): State<AppState>,
    Json(request): Json<OAuthStateRequest>,
) -> Result<Json<OAuthStateResponse>, Response> {
    use crate::managers::checkin::builtin_providers::get_builtin_providers;

    // 查找提供商的 OAuth 配置
    let builtin_providers = get_builtin_providers();
    let provider = builtin_providers
        .iter()
        .find(|p| p.id == request.provider_id);

    let provider = match provider {
        Some(p) => p,
        None => {
            return Ok(Json(OAuthStateResponse {
                success: false,
                authorize_url: None,
                provider_name: String::new(),
                oauth_type: request.oauth_type.clone(),
                message: Some(format!("Provider {} not found", request.provider_id)),
                extraction_guide: vec![],
            }));
        }
    };

    let oauth_config = match &provider.oauth_config {
        Some(config) => config,
        None => {
            return Ok(Json(OAuthStateResponse {
                success: false,
                authorize_url: None,
                provider_name: provider.name.clone(),
                oauth_type: request.oauth_type.clone(),
                message: Some(format!("Provider {} does not support OAuth", provider.name)),
                extraction_guide: vec![],
            }));
        }
    };

    // 根据 OAuth 类型选择 client_id
    let client_id = match request.oauth_type.as_str() {
        "github" => oauth_config.github_client_id.as_deref(),
        "linuxdo" => oauth_config.linuxdo_client_id.as_deref(),
        _ => None,
    };

    let client_id = match client_id {
        Some(id) => id.to_string(),
        None => {
            return Ok(Json(OAuthStateResponse {
                success: false,
                authorize_url: None,
                provider_name: provider.name.clone(),
                oauth_type: request.oauth_type.clone(),
                message: Some(format!(
                    "Provider {} does not support {} OAuth",
                    provider.name, request.oauth_type
                )),
                extraction_guide: vec![],
            }));
        }
    };

    // 尝试从 provider 获取 OAuth state
    let state_url = format!(
        "{}{}?client_id={}",
        provider.base_url.trim_end_matches('/'),
        oauth_config.oauth_state_path,
        client_id
    );

    let state_result = state
        .http_client
        .get(&state_url)
        .header("Accept", "application/json")
        .send()
        .await;

    let (authorize_url, message) = match state_result {
        Ok(response) if response.status().is_success() => {
            let body = response.text().await.unwrap_or_default();
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                // 从响应中提取 state
                let state = json
                    .get("data")
                    .or(Some(&json))
                    .and_then(|d| d.get("state").or(d.get("oauth_state")))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                // 构造授权 URL
                let url = match request.oauth_type.as_str() {
                    "github" => format!(
                        "https://github.com/login/oauth/authorize?client_id={}&state={}&scope=user:email",
                        client_id, state
                    ),
                    "linuxdo" => format!(
                        "https://connect.linux.do/oauth2/authorize?client_id={}&response_type=code&state={}&scope=read",
                        client_id, state
                    ),
                    _ => String::new(),
                };

                if url.is_empty() {
                    (None, Some("Unsupported OAuth type".to_string()))
                } else {
                    (Some(url), None)
                }
            } else {
                (
                    None,
                    Some(format!("Failed to parse OAuth state response: {}", body)),
                )
            }
        }
        Ok(response) => (
            None,
            Some(format!(
                "OAuth state request failed: HTTP {}",
                response.status()
            )),
        ),
        Err(e) => (None, Some(format!("OAuth state request failed: {}", e))),
    };

    // 生成引导说明
    let extraction_guide = vec![
        "1. 点击上方授权链接，在浏览器中完成登录和授权".to_string(),
        "2. 授权完成后，页面会跳转回提供商站点".to_string(),
        "3. 在浏览器中按 F12 打开开发者工具".to_string(),
        "4. 切换到 Application → Cookies → 复制所有 cookies".to_string(),
        format!(
            "5. 或在 Console 中输入: JSON.stringify({{cookies: document.cookie, api_user: localStorage.getItem('user')}})"
        ),
        "6. 将复制的内容粘贴到下方的输入框中".to_string(),
    ];

    Ok(Json(OAuthStateResponse {
        success: authorize_url.is_some(),
        authorize_url,
        provider_name: provider.name.clone(),
        oauth_type: request.oauth_type,
        message,
        extraction_guide,
    }))
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
