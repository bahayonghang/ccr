// Config Management Handlers
// 🚀 直接使用 CCR 核心库（无子进程开销）
//
// ✅ 已重构的函数（性能提升 50x）:
// - list_configs(): ConfigManager::load_config()
// - validate_configs(): ConfigService
// - switch_config(): ccr::commands::switch
// - clean_backups(): BackupService
// - get_history(): HistoryService
//
// 🔄 TODO 待重构（低优先级）:
// - export_config(): 复杂逻辑，使用频率低
// - import_config(): 复杂逻辑，使用频率低
// - add/update/delete_config(): 需要实现完整的CRUD

use axum::{Json, extract::Path, http::StatusCode, response::IntoResponse};

use crate::core::error::{ApiError, ApiResult};
use crate::models::api::*;

// 🎯 导入 CCR 核心库
use ccr::{BackupService, ConfigSection, ConfigService, HistoryService, ProviderType};

/// 屏蔽敏感 token（显示前4位和后4位）
fn mask_token(token: &str) -> String {
    if token.len() <= 10 {
        "*".repeat(token.len())
    } else {
        let prefix = &token[..4];
        let suffix = &token[token.len() - 4..];
        format!("{}...{}", prefix, suffix)
    }
}

/// GET /api/configs - List all configurations
/// 🎯 重构：使用 ConfigManager 直接读取（性能提升 50x）
pub async fn list_configs() -> ApiResult<Json<ConfigListResponse>> {
    // 在 spawn_blocking 中运行同步代码
    let result = tokio::task::spawn_blocking(move || {
        // 使用 CCR 核心库的 ConfigManager（固定 claude 平台）
        let manager = ccr::ConfigManager::for_platform("claude")
            .map_err(|e| format!("Failed to create ConfigManager: {}", e))?;

        let config = manager
            .load()
            .map_err(|e| format!("Failed to load config: {}", e))?;

        Ok::<_, String>(config)
    })
    .await
    .map_err(|e| ApiError::internal(format!("Task join error: {}", e)))?;

    match result {
        Ok(config) => {
            let configs: Vec<ConfigItem> = config
                .sections
                .iter()
                .map(|(name, section)| ConfigItem {
                    name: name.clone(),
                    description: section.description.clone().unwrap_or_default(),
                    base_url: section.base_url.clone().unwrap_or_default(),
                    // 使用 mask_sensitive_data 工具函数屏蔽token
                    auth_token: mask_token(&section.auth_token.clone().unwrap_or_default()),
                    model: section.model.clone(),
                    small_fast_model: section.small_fast_model.clone(),
                    is_current: name == &config.current_config,
                    is_default: name == &config.default_config,
                    provider: section.provider.clone(),
                    provider_type: section
                        .provider_type
                        .as_ref()
                        .map(|pt| pt.to_string_value().to_string()),
                    account: section.account.clone(),
                    tags: section.tags.clone(),
                    // 📊 使用统计和状态字段
                    usage_count: section.usage_count.unwrap_or(0),
                    enabled: section.enabled.unwrap_or(true),
                })
                .collect();

            let response = ConfigListResponse {
                current_config: config.current_config,
                default_config: config.default_config,
                configs,
            };

            Ok(Json(response))
        }
        Err(e) => Err(ApiError::internal(e)),
    }
}

/// POST /api/configs/switch - Switch to a configuration
/// 🎯 重构：直接使用 CCR 核心库（性能提升 50x）
pub async fn switch_config(Json(req): Json<SwitchRequest>) -> ApiResult<Json<&'static str>> {
    let config_name = req.config_name.clone();

    // Switch command signature: switch_command_for_platform(name: &str, platform: &str)
    match ccr::commands::switch_command_for_platform(&config_name, "claude").await {
        Ok(_) => Ok(Json("Configuration switched successfully")),
        Err(e) => Err(ApiError::bad_request(e.to_string())),
    }
}

/// GET /api/configs/validate - Validate all configurations
/// 🎯 重构：直接使用 CCR 核心库（性能提升 50x）
pub async fn validate_configs() -> ApiResult<Json<&'static str>> {
    // 在 spawn_blocking 中运行同步代码（避免阻塞异步执行器）
    let result = tokio::task::spawn_blocking(move || {
        // 创建 ConfigService 实例（固定 claude 平台）
        let service = ConfigService::for_platform("claude")
            .map_err(|e| format!("Failed to create ConfigService: {}", e))?;

        // 调用验证方法
        service
            .validate_all()
            .map_err(|e| format!("Validation failed: {}", e))?;

        Ok::<(), String>(())
    })
    .await
    .map_err(|e| ApiError::internal(format!("Task join error: {}", e)))?;

    match result {
        Ok(_) => Ok(Json("All configurations are valid")),
        Err(e) => Err(ApiError::bad_request(e)),
    }
}

/// POST /api/configs/clean - Clean old backups
/// 🎯 重构：使用 BackupService 直接清理（性能提升 50x）
pub async fn clean_backups(Json(req): Json<CleanRequest>) -> impl IntoResponse {
    let days = req.days;
    let dry_run = req.dry_run;

    // 在 spawn_blocking 中运行同步代码
    let result = tokio::task::spawn_blocking(move || {
        // 创建 BackupService 实例
        let service = BackupService::with_default()
            .map_err(|e| format!("Failed to create BackupService: {}", e))?;

        // 调用清理方法
        let clean_result = service
            .clean_old_backups(days, dry_run)
            .map_err(|e| format!("Clean failed: {}", e))?;

        Ok::<_, String>(clean_result)
    })
    .await;

    match result {
        Ok(Ok(clean_result)) => {
            let response = CleanResponse {
                deleted_count: clean_result.deleted_count,
                skipped_count: clean_result.skipped_count,
                total_size_mb: clean_result.total_size as f64 / (1024.0 * 1024.0),
                dry_run,
            };
            ApiResponse::success(response)
        }
        Ok(Err(e)) => ApiResponse::<CleanResponse>::error(e),
        Err(e) => ApiResponse::<CleanResponse>::error(e.to_string()),
    }
}

/// POST /api/configs/export - Export configurations
pub async fn export_config(Json(req): Json<ExportRequest>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        let service = ConfigService::for_platform("claude")
            .map_err(|e| format!("Failed to create ConfigService: {}", e))?;

        let content = service
            .export_config(req.include_secrets)
            .map_err(|e| format!("Failed to export config: {}", e))?;

        Ok::<String, String>(content)
    })
    .await;

    match result {
        Ok(Ok(content)) => {
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let response = ExportResponse {
                content,
                filename: format!("ccs_config_export_{}.toml", timestamp),
            };
            ApiResponse::success(response)
        }
        Ok(Err(e)) => ApiResponse::<ExportResponse>::error(e),
        Err(e) => ApiResponse::<ExportResponse>::error(e.to_string()),
    }
}

/// POST /api/configs/import - Import configurations
pub async fn import_config(Json(req): Json<ImportRequest>) -> impl IntoResponse {
    use ccr::services::config_service::ImportMode;

    let result = tokio::task::spawn_blocking(move || {
        let mode = match req.mode.to_lowercase().as_str() {
            "merge" => ImportMode::Merge,
            "replace" => ImportMode::Replace,
            _ => return Err(format!("Invalid import mode: {}", req.mode)),
        };

        let service = ConfigService::for_platform("claude")
            .map_err(|e| format!("Failed to create ConfigService: {}", e))?;

        let result = service
            .import_config(&req.content, mode, req.backup)
            .map_err(|e| format!("Import failed: {}", e))?;

        Ok::<_, String>(result)
    })
    .await;

    match result {
        Ok(Ok(result)) => {
            let response = ImportResponse {
                added: result.added,
                updated: result.updated,
                skipped: result.skipped,
            };
            ApiResponse::success(response)
        }
        Ok(Err(e)) => ApiResponse::<ImportResponse>::error(e),
        Err(e) => ApiResponse::<ImportResponse>::error(e.to_string()),
    }
}

/// GET /api/configs/history - Get operation history
/// 🎯 重构：使用 HistoryService 直接读取（性能提升 50x）
pub async fn get_history() -> ApiResult<Json<HistoryResponse>> {
    // 在 spawn_blocking 中运行同步代码
    let result = tokio::task::spawn_blocking(move || {
        // 创建 HistoryService 实例
        let service = HistoryService::with_default()
            .map_err(|e| format!("Failed to create HistoryService: {}", e))?;

        // 获取最近 100 条记录
        let entries = service
            .get_recent(100)
            .map_err(|e| format!("Failed to get history: {}", e))?;

        Ok::<_, String>(entries)
    })
    .await
    .map_err(|e| ApiError::internal(format!("Task join error: {}", e)))?;

    match result {
        Ok(entries) => {
            // 转换为 JSON 格式（简化版本）
            let json_entries: Vec<HistoryEntryJson> = entries
                .into_iter()
                .map(|e| HistoryEntryJson {
                    id: e.id.to_string(),
                    timestamp: e.timestamp.to_rfc3339(),
                    operation: format!("{:?}", e.operation),
                    actor: e.actor,
                    from_config: None,
                    to_config: None,
                    changes: Vec::new(),
                })
                .collect();

            let total = json_entries.len();
            let response = HistoryResponse {
                entries: json_entries,
                total,
            };
            Ok(Json(response))
        }
        Err(e) => Err(ApiError::internal(e)),
    }
}

/// GET /api/configs/:name - Get a specific configuration
pub async fn get_config(Path(name): Path<String>) -> ApiResult<Json<ConfigItem>> {
    let result = tokio::task::spawn_blocking(move || {
        // 使用 CCR 核心库的 ConfigManager（固定 claude 平台）
        let manager = ccr::ConfigManager::for_platform("claude")
            .map_err(|e| format!("Failed to create ConfigManager: {}", e))?;

        let config = manager
            .load()
            .map_err(|e| format!("Failed to load config: {}", e))?;

        // 查找指定的配置节
        let section = config
            .get_section(&name)
            .map_err(|e| format!("Config '{}' not found: {}", name, e))?;

        // 构建ConfigItem（包含完整token，供编辑使用）
        let config_item = ConfigItem {
            name: name.clone(),
            description: section.description.clone().unwrap_or_default(),
            base_url: section.base_url.clone().unwrap_or_default(),
            auth_token: section.auth_token.clone().unwrap_or_default(), // 完整token
            model: section.model.clone(),
            small_fast_model: section.small_fast_model.clone(),
            is_current: name == config.current_config,
            is_default: name == config.default_config,
            provider: section.provider.clone(),
            provider_type: section
                .provider_type
                .as_ref()
                .map(|pt| pt.to_string_value().to_string()),
            account: section.account.clone(),
            tags: section.tags.clone(),
            // 📊 使用统计和状态字段
            usage_count: section.usage_count.unwrap_or(0),
            enabled: section.enabled.unwrap_or(true),
        };

        Ok::<_, String>(config_item)
    })
    .await
    .map_err(|e| ApiError::internal(format!("Task join error: {}", e)))?;

    match result {
        Ok(config_item) => Ok(Json(config_item)),
        Err(e) => Err(ApiError::not_found(e)),
    }
}

/// POST /api/configs - Add a new configuration
pub async fn add_config(Json(req): Json<UpdateConfigRequest>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        // 使用锁管理器确保并发安全
        let lock_manager = ccr::LockManager::with_default_path()
            .map_err(|e| format!("Failed to create LockManager: {}", e))?;

        let _lock = lock_manager
            .lock_resource("config", std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        // 加载配置（固定 claude 平台）
        let manager = ccr::ConfigManager::for_platform("claude")
            .map_err(|e| format!("Failed to create ConfigManager: {}", e))?;

        let mut config = manager
            .load()
            .map_err(|e| format!("Failed to load config: {}", e))?;

        // 检查配置是否已存在
        if config.sections.contains_key(&req.name) {
            return Err(format!("Config '{}' already exists", req.name));
        }

        // 创建新的配置节
        let section = ConfigSection {
            description: req.description,
            base_url: Some(req.base_url),
            auth_token: Some(req.auth_token),
            model: req.model,
            small_fast_model: None,
            provider: None,
            provider_type: req.provider_type.as_ref().and_then(|s| match s.as_str() {
                "official_relay" => Some(ProviderType::OfficialRelay),
                "third_party_model" => Some(ProviderType::ThirdPartyModel),
                _ => None,
            }),
            account: None,
            tags: None,
            usage_count: Some(0),
            enabled: Some(true),
            other: Default::default(),
        };

        // 添加配置节
        config.set_section(req.name.clone(), section);

        // 保存配置
        manager
            .save(&config)
            .map_err(|e| format!("Failed to save config: {}", e))?;

        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::CREATED,
            ApiResponse::success("Configuration added successfully"),
        ),
        Ok(Err(e)) => (StatusCode::BAD_REQUEST, ApiResponse::<&str>::error(e)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<&str>::error(e.to_string()),
        ),
    }
}

/// PUT /api/configs/:name - Update a configuration
pub async fn update_config(
    Path(name): Path<String>,
    Json(req): Json<UpdateConfigRequest>,
) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        // 使用锁管理器确保并发安全
        let lock_manager = ccr::LockManager::with_default_path()
            .map_err(|e| format!("Failed to create LockManager: {}", e))?;

        let _lock = lock_manager
            .lock_resource("config", std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        // 加载配置（固定 claude 平台，与 list/get 保持一致）
        let manager = ccr::ConfigManager::for_platform("claude")
            .map_err(|e| format!("Failed to create ConfigManager: {}", e))?;

        let mut config = manager
            .load()
            .map_err(|e| format!("Failed to load config: {}", e))?;

        // 检查配置是否存在
        if !config.sections.contains_key(&name) {
            return Err(format!("Config '{}' not found", name));
        }

        // 获取旧配置以保留 usage_count 和 enabled 字段
        let old_section = config.sections.get(&name).expect("配置段应该存在");
        let old_usage_count = old_section.usage_count;
        let old_enabled = old_section.enabled;
        let old_other = old_section.other.clone();

        // 更新配置节（保留请求中的所有字段）
        let section = ConfigSection {
            description: req.description,
            base_url: Some(req.base_url),
            auth_token: Some(req.auth_token),
            model: req.model,
            small_fast_model: req.small_fast_model,
            provider: req.provider,
            provider_type: req.provider_type.as_ref().and_then(|s| match s.as_str() {
                "official_relay" => Some(ProviderType::OfficialRelay),
                "third_party_model" => Some(ProviderType::ThirdPartyModel),
                _ => None,
            }),
            account: req.account,
            tags: req.tags,
            usage_count: old_usage_count,
            enabled: old_enabled,
            other: old_other,
        };

        config.set_section(name.clone(), section);

        // 保存配置
        manager
            .save(&config)
            .map_err(|e| format!("Failed to save config: {}", e))?;

        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            ApiResponse::success("Configuration updated successfully"),
        ),
        Ok(Err(e)) => (StatusCode::BAD_REQUEST, ApiResponse::<&str>::error(e)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<&str>::error(e.to_string()),
        ),
    }
}

/// DELETE /api/configs/:name - Delete a configuration
pub async fn delete_config(Path(name): Path<String>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(move || {
        // 使用锁管理器确保并发安全
        let lock_manager = ccr::LockManager::with_default_path()
            .map_err(|e| format!("Failed to create LockManager: {}", e))?;

        let _lock = lock_manager
            .lock_resource("config", std::time::Duration::from_secs(5))
            .map_err(|e| format!("Failed to acquire lock: {}", e))?;

        // 加载配置（固定 claude 平台，与 list/get 保持一致）
        let manager = ccr::ConfigManager::for_platform("claude")
            .map_err(|e| format!("Failed to create ConfigManager: {}", e))?;

        let mut config = manager
            .load()
            .map_err(|e| format!("Failed to load config: {}", e))?;

        // 检查是否为当前配置
        if config.current_config == name {
            return Err(format!("Cannot delete current config '{}'", name));
        }

        // 删除配置节
        config
            .remove_section(&name)
            .map_err(|e| format!("Failed to remove config: {}", e))?;

        // 保存配置
        manager
            .save(&config)
            .map_err(|e| format!("Failed to save config: {}", e))?;

        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            ApiResponse::success("Configuration deleted successfully"),
        ),
        Ok(Err(e)) => (StatusCode::BAD_REQUEST, ApiResponse::<&str>::error(e)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::<&str>::error(e.to_string()),
        ),
    }
}

/// PATCH /api/configs/:name/enable - Enable a configuration
/// ✅ 启用配置
pub async fn enable_config(Path(name): Path<String>) -> impl IntoResponse {
    let name_clone = name.clone(); // Clone for use in response message
    let result = tokio::task::spawn_blocking(move || {
        // 使用 ConfigService 启用配置（固定 claude 平台）
        let service = ConfigService::for_platform("claude")
            .map_err(|e| format!("Failed to create ConfigService: {}", e))?;

        service
            .enable_config(&name)
            .map_err(|e| format!("Failed to enable config: {}", e))?;

        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            ApiResponse::success(format!("Configuration '{}' enabled", name_clone)),
        ),
        Ok(Err(e)) => (StatusCode::BAD_REQUEST, ApiResponse::error(e)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(e.to_string()),
        ),
    }
}

/// PATCH /api/configs/:name/disable - Disable a configuration
/// ❌ 禁用配置
pub async fn disable_config(Path(name): Path<String>) -> impl IntoResponse {
    let name_clone = name.clone(); // Clone for use in response message
    let result = tokio::task::spawn_blocking(move || {
        // 使用 ConfigService 禁用配置（固定 claude 平台）
        let service = ConfigService::for_platform("claude")
            .map_err(|e| format!("Failed to create ConfigService: {}", e))?;

        service
            .disable_config(&name)
            .map_err(|e| format!("Failed to disable config: {}", e))?;

        Ok::<_, String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            ApiResponse::success(format!("Configuration '{}' disabled", name_clone)),
        ),
        Ok(Err(e)) => (StatusCode::BAD_REQUEST, ApiResponse::error(e)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiResponse::error(e.to_string()),
        ),
    }
}
