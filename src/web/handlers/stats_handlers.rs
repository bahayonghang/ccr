// 📊 统计相关处理器
// 提供基于 profiles 配置的提供商使用次数统计

use crate::web::error_utils::internal_server_error;
use crate::web::handlers::AppState;
use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;

/// GET /api/stats/provider-usage
///
/// 从当前加载的配置（profiles.toml 或 legacy 配置）中聚合
/// provider -> usage_count 的总和。
///
/// 注意：
/// - 前端期望直接返回 `{"provider": count, ...}` 这种 Map 结构，
///   而不是统一的 ApiResponse 包装，因此这里直接返回原始 JSON Map。
pub async fn handle_provider_usage(State(state): State<AppState>) -> Response {
    // 从内存中的配置缓存聚合统计，避免在请求路径上频繁读盘
    let cache = match state.config_cache.read() {
        Ok(guard) => guard,
        Err(e) => {
            return internal_server_error(format!("获取配置缓存读锁失败: {}", e));
        }
    };

    let mut map: HashMap<String, u64> = HashMap::new();

    for (_name, section) in cache.sections.iter() {
        let provider = section
            .provider
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let count = section.usage_count.unwrap_or(0) as u64;

        map.entry(provider)
            .and_modify(|c| *c += count)
            .or_insert(count);
    }

    Json(map).into_response()
}
