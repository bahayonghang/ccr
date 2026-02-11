// Usage Analytics API Handler
// 读取和解析 Claude Code 的 usage 日志
// 支持增量导入和 SQLite 缓存

use axum::{Json, extract::Query};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use walkdir::WalkDir;

use crate::api::handlers::response::ApiSuccess;
use crate::core::error::{ApiError, ApiResult};
use crate::models::usage::{UsageData, UsageRecord, UsageRecordsResponse};
use crate::services::usage_import_service::{ImportConfig, ImportResult, UsageImportService};

// 缓存结构
struct CachedUsageData {
    records: Vec<UsageRecord>,
    total_records: usize,
    timestamp: Instant,
}

// 全局缓存，按 platform 分组
lazy_static::lazy_static! {
    static ref USAGE_CACHE: Arc<Mutex<HashMap<String, CachedUsageData>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

// 缓存有效期：60 秒
const CACHE_TTL: Duration = Duration::from_secs(60);

/// Query 参数
#[derive(Debug, Deserialize)]
pub struct UsageRecordsQuery {
    /// 平台 (claude, codex, gemini)
    #[serde(default = "default_platform")]
    pub platform: String,

    /// 返回记录数限制
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_platform() -> String {
    "claude".to_string()
}

fn default_limit() -> usize {
    10000
}

/// GET /api/usage/records - 获取 usage 记录
///
/// 从 Claude Code 日志文件中读取 usage 数据（带缓存）
/// 注：此端点直接返回 UsageRecordsResponse（不包裹在 success envelope 中）
pub async fn get_usage_records(
    Query(params): Query<UsageRecordsQuery>,
) -> ApiResult<Json<UsageRecordsResponse>> {
    debug!(
        "Fetching usage records for platform: {}, limit: {}",
        params.platform, params.limit
    );

    // 验证 platform 参数
    let valid_platforms = ["claude", "codex", "gemini"];
    if !valid_platforms.contains(&params.platform.as_str()) {
        return Err(ApiError::bad_request(format!(
            "Platform '{}' not supported for usage analytics. Supported platforms: claude, codex, gemini",
            params.platform
        )));
    }

    // 验证 limit 参数
    let limit = params.limit.min(50000); // 最大 50K 记录
    if params.limit > 50000 {
        warn!(
            "Limit {} exceeds maximum 50000, capping to 50000",
            params.limit
        );
    }

    // 尝试从缓存获取
    {
        let cache = USAGE_CACHE.lock().expect("无法获取缓存锁");
        if let Some(cached) = cache.get(&params.platform)
            && cached.timestamp.elapsed() < CACHE_TTL
        {
            debug!("Using cached data for platform: {}", params.platform);
            let response = UsageRecordsResponse::new(
                cached.records.iter().take(limit).cloned().collect(),
                cached.total_records,
                limit,
            );
            return Ok(Json(response));
        }
    }

    // 缓存过期或不存在，重新读取
    debug!(
        "Cache miss or expired for platform: {}, reading from files",
        params.platform
    );
    let (records, total) = read_usage_files(&params.platform, limit)
        .await
        .map_err(|e| {
            error!("Failed to read usage files: {}", e);
            ApiError::internal(format!("Failed to read usage files: {}", e))
        })?;

    // 更新缓存
    {
        let mut cache = USAGE_CACHE.lock().expect("无法获取缓存锁");
        cache.insert(
            params.platform.clone(),
            CachedUsageData {
                records: records.clone(),
                total_records: total,
                timestamp: Instant::now(),
            },
        );
    }

    let response = UsageRecordsResponse::new(records, total, limit);
    debug!(
        "Successfully read {} records (total: {}, truncated: {})",
        response.records.len(),
        response.total_records,
        response.truncated
    );
    Ok(Json(response))
}

/// 读取平台 usage 文件
///
/// 从不同平台的 projects/**/*.jsonl 读取并解析 usage 记录
/// - claude: ~/.claude/projects/**/*.jsonl
/// - codex: ~/.codex/projects/**/*.jsonl
/// - gemini: ~/.gemini/projects/**/*.jsonl
async fn read_usage_files(
    platform: &str,
    limit: usize,
) -> Result<(Vec<UsageRecord>, usize), String> {
    // 获取 home 目录
    let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;

    // 根据平台确定projects目录
    let projects_dir = match platform {
        "claude" => home_dir.join(".claude/projects"),
        "codex" => home_dir.join(".codex/projects"),
        "gemini" => home_dir.join(".gemini/projects"),
        _ => return Err(format!("Unsupported platform: {}", platform)),
    };

    // 检查目录是否存在
    if !projects_dir.exists() {
        warn!(
            "Projects directory does not exist for platform {}: {:?}",
            platform, projects_dir
        );
        return Ok((vec![], 0));
    }

    // 收集所有 .jsonl 文件路径
    let jsonl_files: Vec<_> = WalkDir::new(&projects_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "jsonl"))
        .map(|e| e.path().to_path_buf())
        .collect();

    debug!(
        "Found {} JSONL files for platform {}",
        jsonl_files.len(),
        platform
    );

    // 使用 rayon 并行处理文件（如果文件数量 > 1）
    let all_records: Vec<UsageRecord> = if jsonl_files.len() > 1 {
        use rayon::prelude::*;
        jsonl_files
            .par_iter()
            .flat_map(|path| {
                debug!("Reading usage file: {:?}", path);
                let mut file_records = Vec::new();

                // 读取文件内容
                if let Ok(content) = std::fs::read_to_string(path) {
                    // 逐行解析 JSON
                    for (line_num, line) in content.lines().enumerate() {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }

                        // 解析 JSON
                        if let Ok(json) = serde_json::from_str::<Value>(line) {
                            // 尝试解析为 UsageRecord
                            if let Some(record) = parse_usage_record(&json)
                                && record.is_valid()
                            {
                                file_records.push(record);
                            }
                        } else {
                            warn!("Failed to parse JSON at {:?}:{}", path, line_num + 1);
                        }
                    }
                } else {
                    warn!("Failed to read file {:?}", path);
                }

                file_records
            })
            .collect()
    } else {
        // 单文件情况，不使用并行
        let mut file_records = Vec::new();
        for path in &jsonl_files {
            debug!("Reading usage file: {:?}", path);

            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    if let Ok(json) = serde_json::from_str::<Value>(line)
                        && let Some(record) = parse_usage_record(&json)
                        && record.is_valid()
                    {
                        file_records.push(record);
                    }
                }
            }
        }
        file_records
    };

    let total_scanned = all_records.len();
    let mut records = all_records;

    // 按时间戳降序排序（最新的在前）
    records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // 截断到 limit
    records.truncate(limit);

    debug!(
        "Read {} valid records from {} total scanned",
        records.len(),
        total_scanned
    );

    Ok((records, total_scanned))
}

/// 从 JSON 中解析 UsageRecord
///
/// 支持两种 schema:
/// 1. 顶层: json["model"], json["usage"]
/// 2. 嵌套: json["message"]["model"], json["message"]["usage"]
fn parse_usage_record(json: &Value) -> Option<UsageRecord> {
    // 提取 uuid
    let uuid = json
        .get("uuid")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())?;

    // 提取 timestamp
    let timestamp = json
        .get("timestamp")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())?;

    // 提取 model - 优先从顶层，其次从 message
    let model = json
        .get("model")
        .or_else(|| json.get("message").and_then(|m| m.get("model")))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // 提取 usage - 优先从顶层，其次从 message
    let usage_obj = json
        .get("usage")
        .or_else(|| json.get("message").and_then(|m| m.get("usage")));

    let usage = usage_obj.and_then(parse_usage_data);

    Some(UsageRecord {
        uuid,
        timestamp,
        model,
        usage,
    })
}

/// 从 JSON 中解析 UsageData
fn parse_usage_data(json: &Value) -> Option<UsageData> {
    let input_tokens = json.get("input_tokens").and_then(|v| v.as_u64());

    let output_tokens = json.get("output_tokens").and_then(|v| v.as_u64());

    let cache_read_input_tokens = json.get("cache_read_input_tokens").and_then(|v| v.as_u64());

    // 至少需要一个 token 字段
    if input_tokens.is_none() && output_tokens.is_none() && cache_read_input_tokens.is_none() {
        return None;
    }

    Some(UsageData {
        input_tokens,
        output_tokens,
        cache_read_input_tokens,
    })
}

// ═══════════════════════════════════════════════════════════
// Usage Import Pipeline Endpoints (SQLite-backed)
// ═══════════════════════════════════════════════════════════

/// 导入请求参数
#[derive(Debug, Deserialize)]
pub struct ImportQuery {
    /// 平台 (claude, codex, gemini)
    #[serde(default = "default_platform")]
    pub platform: String,
}

/// 导入响应
#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub result: ImportResult,
    pub message: String,
}

/// POST /api/usage/import - 触发增量导入
///
/// 从平台日志文件增量导入 usage 数据到 SQLite
pub async fn import_usage(
    Query(params): Query<ImportQuery>,
) -> ApiResult<ApiSuccess<ImportResponse>> {
    info!("Triggering usage import for platform: {}", params.platform);

    // 验证 platform 参数
    let valid_platforms = ["claude", "codex", "gemini"];
    if !valid_platforms.contains(&params.platform.as_str()) {
        return Err(ApiError::bad_request(format!(
            "Platform '{}' not supported. Supported platforms: claude, codex, gemini",
            params.platform
        )));
    }

    let service = UsageImportService::new(ImportConfig::default());

    let result = service.import_platform(&params.platform).map_err(|e| {
        error!("Import failed for {}: {}", params.platform, e);
        ApiError::internal(format!("Import failed: {}", e))
    })?;

    let message = format!(
        "Imported {} records from {} files for {} ({})",
        result.records_imported,
        result.files_processed,
        result.platform,
        if result.completed {
            "completed"
        } else {
            "partial"
        }
    );
    info!("{}", message);

    Ok(ApiSuccess(ImportResponse { result, message }))
}

/// 导入所有平台响应
#[derive(Debug, Serialize)]
pub struct ImportAllResponse {
    pub results: Vec<ImportResult>,
    pub total_imported: usize,
    pub total_files: usize,
}

/// POST /api/usage/import/all - 导入所有平台
///
/// 依次导入 claude, codex, gemini 平台的 usage 数据
pub async fn import_all_usage() -> ApiResult<ApiSuccess<ImportAllResponse>> {
    info!("Triggering usage import for all platforms");

    let service = UsageImportService::new(ImportConfig::default());
    let mut results = Vec::new();
    let mut total_imported = 0;
    let mut total_files = 0;

    for platform in &["claude", "codex", "gemini"] {
        match service.import_platform(platform) {
            Ok(result) => {
                total_imported += result.records_imported;
                total_files += result.files_processed;
                results.push(result);
            }
            Err(e) => {
                warn!("Import failed for {}: {}", platform, e);
            }
        }
    }

    info!(
        "Import complete: {} records from {} files across {} platforms",
        total_imported,
        total_files,
        results.len()
    );

    Ok(ApiSuccess(ImportAllResponse {
        results,
        total_imported,
        total_files,
    }))
}

/// 统计信息响应
#[derive(Debug, Serialize)]
pub struct UsageStatsResponse {
    pub total_sources: i64,
    pub total_records: i64,
    pub records_by_platform: Vec<(String, i64)>,
}

/// GET /api/usage/stats - 获取使用统计
///
/// 返回数据库中的记录统计信息
pub async fn get_usage_stats() -> ApiResult<ApiSuccess<UsageStatsResponse>> {
    let service = UsageImportService::new(ImportConfig::default());

    let stats = service.get_stats().map_err(|e| {
        error!("Failed to get usage stats: {}", e);
        ApiError::internal(format!("Failed to get stats: {}", e))
    })?;

    Ok(ApiSuccess(UsageStatsResponse {
        total_sources: stats.total_sources,
        total_records: stats.total_records,
        records_by_platform: stats.records_by_platform,
    }))
}

/// 清理请求参数
#[derive(Debug, Deserialize)]
pub struct CleanupQuery {
    /// 保留天数 (默认 90)
    #[serde(default = "default_retention_days")]
    pub retention_days: i64,
}

fn default_retention_days() -> i64 {
    90
}

/// 清理响应
#[derive(Debug, Serialize)]
pub struct CleanupResponse {
    pub records_deleted: usize,
}

/// POST /api/usage/cleanup - 清理旧记录
///
/// 删除超过保留期限的旧记录
pub async fn cleanup_usage(
    Query(params): Query<CleanupQuery>,
) -> ApiResult<ApiSuccess<CleanupResponse>> {
    info!(
        "Cleaning up records older than {} days",
        params.retention_days
    );

    let config = ImportConfig {
        retention_days: params.retention_days,
        ..Default::default()
    };
    let service = UsageImportService::new(config);

    let deleted = service.cleanup_old_records().map_err(|e| {
        error!("Cleanup failed: {}", e);
        ApiError::internal(format!("Cleanup failed: {}", e))
    })?;

    info!("Cleaned up {} old records", deleted);
    Ok(ApiSuccess(CleanupResponse {
        records_deleted: deleted,
    }))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_usage_data() {
        let json = json!({
            "input_tokens": 100,
            "output_tokens": 50,
            "cache_read_input_tokens": 25
        });

        let usage = parse_usage_data(&json).unwrap();
        assert_eq!(usage.input_tokens, Some(100));
        assert_eq!(usage.output_tokens, Some(50));
        assert_eq!(usage.cache_read_input_tokens, Some(25));
    }

    #[test]
    fn test_parse_usage_record_top_level() {
        let json = json!({
            "uuid": "abc123",
            "timestamp": "2025-11-18T10:30:00Z",
            "model": "claude-sonnet-4-5",
            "usage": {
                "input_tokens": 100,
                "output_tokens": 50
            }
        });

        let record = parse_usage_record(&json).unwrap();
        assert_eq!(record.uuid, "abc123");
        assert_eq!(record.model, Some("claude-sonnet-4-5".to_string()));
        assert!(record.usage.is_some());
    }

    #[test]
    fn test_parse_usage_record_nested() {
        let json = json!({
            "uuid": "def456",
            "timestamp": "2025-11-18T11:00:00Z",
            "message": {
                "model": "claude-opus-4",
                "usage": {
                    "input_tokens": 200,
                    "output_tokens": 100,
                    "cache_read_input_tokens": 50
                }
            }
        });

        let record = parse_usage_record(&json).unwrap();
        assert_eq!(record.uuid, "def456");
        assert_eq!(record.model, Some("claude-opus-4".to_string()));
        assert!(record.usage.is_some());
        assert_eq!(record.usage.unwrap().cache_read_input_tokens, Some(50));
    }

    #[test]
    fn test_parse_usage_record_invalid() {
        let json = json!({
            "uuid": "invalid",
            "timestamp": "2025-11-18T10:30:00Z"
            // No model or usage
        });

        let record = parse_usage_record(&json).unwrap();
        assert_eq!(record.model, None);
        assert_eq!(record.usage, None);
        assert!(!record.is_valid()); // Should be invalid
    }
}
