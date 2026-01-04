// Routes module
// 模块化路由定义，提高可维护性

pub mod agents_routes;
pub mod budget_routes;
pub mod builtin_prompts_routes;
pub mod checkin_routes;
pub mod codex_routes;
pub mod command_routes;
pub mod config_routes;
pub mod converter_routes;
pub mod gemini_routes;
pub mod marketplace_routes;
pub mod mcp_routes;
pub mod platform_routes;
pub mod plugins_routes;
pub mod pricing_routes;
pub mod prompts_routes;
pub mod provider_health_routes;
pub mod qwen_routes;
pub mod sessions_routes;
pub mod skills_routes;
pub mod slash_commands_routes;
pub mod stats_routes;
pub mod sync_routes;
pub mod system_routes;
pub mod ui_state_routes;
pub mod usage_routes;
pub mod version_routes;

use axum::Router;

/// 应用中间件
pub fn apply_middleware(app: Router) -> Router {
    use tower::ServiceBuilder;
    use tower_http::{
        compression::CompressionLayer,
        cors::{Any, CorsLayer},
        request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
        trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    };
    use tracing::Level;

    // 创建中间件堆栈 (仅包含不失败层)
    let middleware = ServiceBuilder::new()
        // 请求ID生成和传播 (必须是第一个)
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        // 带请求ID的日志中间件
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        // CORS中间件
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
                .max_age(std::time::Duration::from_secs(3600)),
        )
        // 压缩中间件
        .layer(CompressionLayer::new());

    app.layer(middleware)
}

/// 组装所有路由
pub fn create_app() -> Router {
    // 创建 WebSocket 状态
    let ws_state = std::sync::Arc::new(crate::services::websocket::WsState::new());

    // 基础路由
    Router::new()
        // WebSocket 路由
        .route(
            "/ws",
            axum::routing::get(crate::services::websocket::ws_handler),
        )
        .with_state(ws_state)
        // 健康检查
        .route("/health", axum::routing::get(health_check))
        // API 路由分组
        .nest("/api", create_api_routes())
}

/// API 路由
fn create_api_routes() -> Router {
    let app = Router::new()
        // 配置管理
        .merge(config_routes::routes())
        // 命令执行
        .merge(command_routes::routes())
        // 系统信息
        .merge(system_routes::routes())
        // 版本管理
        .merge(version_routes::routes())
        // 平台管理
        .merge(platform_routes::routes())
        // MCP 服务器管理
        .merge(mcp_routes::routes())
        // MCP 预设管理
        .merge(mcp_routes::presets_routes())
        // MCP 同步
        .merge(mcp_routes::sync_routes())
        // 内置提示词
        .merge(builtin_prompts_routes::routes())
        // 斜杠命令
        .merge(slash_commands_routes::routes())
        // Agent 管理
        .merge(agents_routes::routes())
        // 插件管理
        .merge(plugins_routes::routes())
        // 统计数据
        .merge(stats_routes::routes())
        // 技能管理
        .merge(skills_routes::routes())
        // 提示词管理
        .merge(prompts_routes::routes())
        // 预算管理
        .merge(budget_routes::routes())
        // 定价管理
        .merge(pricing_routes::routes())
        // 使用记录
        .merge(usage_routes::routes())
        // 同步管理
        .merge(sync_routes::routes())
        // Codex 平台
        .merge(codex_routes::routes())
        // Gemini 平台
        .merge(gemini_routes::routes())
        // Qwen 平台
        .merge(qwen_routes::routes())
        // 配置转换
        .merge(converter_routes::routes())
        // UI 状态
        .merge(ui_state_routes::routes())
        // Sessions 管理
        .merge(sessions_routes::routes())
        // Provider 健康检查
        .merge(provider_health_routes::routes())
        // 签到管理
        .merge(checkin_routes::routes())
        // 资源市场
        .merge(marketplace_routes::routes());

    apply_middleware(app)
}

/// 健康检查端点
async fn health_check() -> &'static str {
    "OK"
}
