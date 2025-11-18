// Usage Analytics API Handler
// 读取和解析 Claude Code 的 usage 日志

use axum::{Json, extract::Query, http::StatusCode};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};
use walkdir::WalkDir;

use crate::models::usage::{UsageData, UsageRecord, UsageRecordsResponse};

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
pub async fn get_usage_records(
    Query(params): Query<UsageRecordsQuery>,
) -> Result<Json<UsageRecordsResponse>, (StatusCode, String)> {
    debug!(
        "Fetching usage records for platform: {}, limit: {}",
        params.platform, params.limit
    );

    // 验证 platform 参数
    let valid_platforms = ["claude", "codex", "gemini"];
    if !valid_platforms.contains(&params.platform.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Platform '{}' not supported for usage analytics. Supported platforms: claude, codex, gemini",
                params.platform
            ),
        ));
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
        let cache = USAGE_CACHE.lock().unwrap();
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
    match read_usage_files(&params.platform, limit).await {
        Ok((records, total)) => {
            // 更新缓存
            {
                let mut cache = USAGE_CACHE.lock().unwrap();
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
        Err(e) => {
            error!("Failed to read usage files: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read usage files: {}", e),
            ))
        }
    }
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

#[cfg(test)]
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
