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
pub mod droid_routes;
pub mod gemini_routes;
pub mod hooks_routes;
pub mod iflow_routes;
pub mod marketplace_routes;
pub mod mcp_routes;
pub mod output_styles_routes;
pub mod platform_routes;
pub mod plugins_routes;
pub mod pricing_routes;
pub mod prompts_routes;
pub mod provider_health_routes;
pub mod qwen_routes;
pub mod sessions_routes;
pub mod skill_hub_routes;
pub mod skills_routes;
pub mod slash_commands_routes;
pub mod stats_routes;
pub mod statusline_routes;
pub mod sync_routes;
pub mod system_routes;
pub mod ui_state_routes;
pub mod usage_routes;
pub mod version_routes;

use axum::Router;

use crate::state::AppState;

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

/// 组装所有路由（使用 AppState）
pub fn create_app(app_state: AppState) -> Router {
    // 基础路由
    Router::new()
        // WebSocket 路由（使用 AppState 中的 ws）
        .route(
            "/ws",
            axum::routing::get(crate::services::websocket::ws_handler),
        )
        .with_state(app_state.ws.clone())
        // 健康检查
        .route("/health", axum::routing::get(health_check))
        // API 路由分组
        .nest("/api", create_api_routes())
        // 注入 AppState 到所有路由（供未来 Handler 使用）
        .with_state(app_state)
}

/// API 路由
///
/// 注意：中间件在 main.rs 中统一应用，这里不再重复应用
/// 使用 `Router::new()` 创建无状态路由，然后通过 `with_state` 在上层注入 AppState
fn create_api_routes() -> Router<AppState> {
    // 所有子路由返回 Router<()>，通过 with_state(()) 转换为 Router<AppState>
    // 这样可以保持向后兼容，同时允许未来的 Handler 使用 AppState
    Router::new()
        // 配置管理
        .merge(config_routes::routes().with_state(()))
        // 命令执行
        .merge(command_routes::routes().with_state(()))
        // 系统信息
        .merge(system_routes::routes().with_state(()))
        // 版本管理
        .merge(version_routes::routes().with_state(()))
        // 平台管理
        .merge(platform_routes::routes().with_state(()))
        // MCP 服务器管理
        .merge(mcp_routes::routes().with_state(()))
        // MCP 预设管理
        .merge(mcp_routes::presets_routes().with_state(()))
        // MCP 同步
        .merge(mcp_routes::sync_routes().with_state(()))
        // 内置提示词
        .merge(builtin_prompts_routes::routes().with_state(()))
        // 斜杠命令
        .merge(slash_commands_routes::routes().with_state(()))
        // Agent 管理
        .merge(agents_routes::routes().with_state(()))
        // 插件管理
        .merge(plugins_routes::routes().with_state(()))
        // Hooks 管理
        .merge(hooks_routes::routes().with_state(()))
        // Statusline 配置
        .merge(statusline_routes::routes().with_state(()))
        // 统计数据
        .merge(stats_routes::routes().with_state(()))
        // 技能管理
        .merge(skills_routes::routes().with_state(()))
        // Skill Hub
        .merge(skill_hub_routes::routes().with_state(()))
        // 提示词管理
        .merge(prompts_routes::routes().with_state(()))
        // Output Styles 管理
        .merge(output_styles_routes::routes().with_state(()))
        // 预算管理
        .merge(budget_routes::routes().with_state(()))
        // 定价管理
        .merge(pricing_routes::routes().with_state(()))
        // 使用记录
        .merge(usage_routes::routes().with_state(()))
        // 同步管理
        .merge(sync_routes::routes().with_state(()))
        // Codex 平台
        .merge(codex_routes::routes().with_state(()))
        // Gemini 平台
        .merge(gemini_routes::routes().with_state(()))
        // Qwen 平台
        .merge(qwen_routes::routes().with_state(()))
        // iFlow 平台
        .merge(iflow_routes::routes().with_state(()))
        // Droid 平台
        .merge(droid_routes::routes().with_state(()))
        // 配置转换
        .merge(converter_routes::routes().with_state(()))
        // UI 状态
        .merge(ui_state_routes::routes().with_state(()))
        // Sessions 管理
        .merge(sessions_routes::routes().with_state(()))
        // Provider 健康检查
        .merge(provider_health_routes::routes().with_state(()))
        // 签到管理
        .merge(checkin_routes::routes().with_state(()))
        // 资源市场
        .merge(marketplace_routes::routes().with_state(()))
}

/// 健康检查端点
async fn health_check() -> &'static str {
    "OK"
}
