// CCR UI Backend Server
// Axum server that executes CCR CLI commands as subprocesses

use axum::{
    Router,
    routing::{delete, get, patch, post, put},
};
use clap::Parser;
use std::{fs, net::SocketAddr, path::PathBuf};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::{Level, info, warn};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

// New layered architecture modules
mod api;
mod core;
mod managers;
mod models;
mod services;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to bind the server to
    #[arg(short, long, default_value = "8081")]
    port: u16,

    /// Host to bind the server to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logger with file rotation
    setup_logging()?;

    let bind_addr: SocketAddr = format!("{}:{}", args.host, args.port)
        .parse()
        .expect("Failed to parse bind address");

    info!("Starting CCR UI Backend Server");
    info!("Server binding to: {}", bind_addr);
    info!("API endpoints:");
    info!("  - Health Check: http://{}/health", bind_addr);
    info!("  - Config Management: http://{}/api/configs", bind_addr);
    info!(
        "  - Command Execution: http://{}/api/command/execute",
        bind_addr
    );
    info!("  - System Info: http://{}/api/system", bind_addr);
    info!("  - Version Info: http://{}/api/version", bind_addr);

    // Verify CCR is available
    match core::executor::execute_command(vec!["version".to_string()]).await {
        Ok(output) if output.success => {
            info!("CCR binary found and working");
            info!("CCR Version: {}", output.stdout.trim());
        }
        Ok(output) => {
            warn!("CCR binary found but returned error: {}", output.stderr);
        }
        Err(e) => {
            warn!("CCR binary not found or not working: {}", e);
            warn!(
                "Continuing to start server without CCR; API calls that require CCR will return errors."
            );
            // Do not abort server startup; proceed.
        }
    }

    // Build the router with all routes
    let app = create_router();

    // Start the server
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    info!("Server started successfully, listening on {}", bind_addr);

    axum::serve(listener, app)
        .await
        .map_err(std::io::Error::other)
}

/// Create the main application router with all middlewares and routes
fn create_router() -> Router {
    // Note: Timeout and ConcurrencyLimit layers are applied at the server level
    // to avoid error type conflicts. See main() function for their usage.

    // Create middleware stack (only infallible layers here)
    let middleware = ServiceBuilder::new()
        // Request ID generation and propagation (must be first)
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        // Logging middleware with request ID
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        // CORS middleware
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
                .max_age(std::time::Duration::from_secs(3600)),
        )
        // Compression middleware
        .layer(CompressionLayer::new());

    // Build the router with all routes
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // Config management endpoints
        .route("/api/configs", get(api::handlers::config::list_configs))
        .route("/api/switch", post(api::handlers::config::switch_config))
        .route(
            "/api/validate",
            get(api::handlers::config::validate_configs),
        )
        .route("/api/clean", post(api::handlers::config::clean_backups))
        .route("/api/export", post(api::handlers::config::export_config))
        .route("/api/import", post(api::handlers::config::import_config))
        .route("/api/history", get(api::handlers::config::get_history))
        .route("/api/configs", post(api::handlers::config::add_config))
        .route(
            "/api/configs/{name}",
            get(api::handlers::config::get_config),
        )
        .route(
            "/api/configs/{name}",
            put(api::handlers::config::update_config),
        )
        .route(
            "/api/configs/{name}",
            delete(api::handlers::config::delete_config),
        )
        // 启用/禁用配置
        .route(
            "/api/configs/{name}/enable",
            patch(api::handlers::config::enable_config),
        )
        .route(
            "/api/configs/{name}/disable",
            patch(api::handlers::config::disable_config),
        )
        // Command execution endpoints
        .route(
            "/api/command/execute",
            post(api::handlers::command::execute_command),
        )
        .route(
            "/api/command/execute/stream",
            post(api::handlers::command::execute_command_stream),
        )
        .route(
            "/api/command/list",
            get(api::handlers::command::list_commands),
        )
        .route(
            "/api/command/help/{command}",
            get(api::handlers::command::get_command_help),
        )
        // System info endpoint
        .route("/api/system", get(api::handlers::system::get_system_info))
        // Version management endpoints
        .route("/api/version", get(api::handlers::version::get_version))
        .route(
            "/api/version/check-update",
            get(api::handlers::version::check_update),
        )
        .route(
            "/api/version/update",
            post(api::handlers::version::update_ccr),
        )
        // Platform management endpoints (Unified mode)
        .route(
            "/api/platforms",
            get(api::handlers::platform::list_platforms),
        )
        .route(
            "/api/platforms/current",
            get(api::handlers::platform::get_current_platform),
        )
        .route(
            "/api/platforms/switch",
            post(api::handlers::platform::switch_platform),
        )
        .route(
            "/api/platforms/{name}",
            get(api::handlers::platform::get_platform),
        )
        .route(
            "/api/platforms/{name}",
            put(api::handlers::platform::update_platform),
        )
        .route(
            "/api/platforms/{name}/enable",
            post(api::handlers::platform::enable_platform),
        )
        .route(
            "/api/platforms/{name}/disable",
            post(api::handlers::platform::disable_platform),
        )
        .route(
            "/api/platforms/{name}/profile",
            get(api::handlers::platform::get_platform_profile),
        )
        .route(
            "/api/platforms/{name}/profile",
            post(api::handlers::platform::set_platform_profile),
        )
        // MCP server management endpoints
        .route("/api/mcp", get(api::handlers::mcp::list_mcp_servers))
        .route("/api/mcp", post(api::handlers::mcp::add_mcp_server))
        .route(
            "/api/mcp/{name}",
            put(api::handlers::mcp::update_mcp_server),
        )
        .route(
            "/api/mcp/{name}",
            delete(api::handlers::mcp::delete_mcp_server),
        )
        .route(
            "/api/mcp/{name}/toggle",
            put(api::handlers::mcp::toggle_mcp_server),
        )
        // Slash command management endpoints
        .route(
            "/api/slash-commands",
            get(api::handlers::slash_commands::list_slash_commands),
        )
        .route(
            "/api/slash-commands",
            post(api::handlers::slash_commands::add_slash_command),
        )
        .route(
            "/api/slash-commands/{name}",
            put(api::handlers::slash_commands::update_slash_command),
        )
        .route(
            "/api/slash-commands/{name}",
            delete(api::handlers::slash_commands::delete_slash_command),
        )
        .route(
            "/api/slash-commands/{name}/toggle",
            put(api::handlers::slash_commands::toggle_slash_command),
        )
        // Agent management endpoints
        .route("/api/agents", get(api::handlers::agents::list_agents))
        .route("/api/agents", post(api::handlers::agents::add_agent))
        .route(
            "/api/agents/{name}",
            put(api::handlers::agents::update_agent),
        )
        .route(
            "/api/agents/{name}",
            delete(api::handlers::agents::delete_agent),
        )
        .route(
            "/api/agents/{name}/toggle",
            put(api::handlers::agents::toggle_agent),
        )
        // Plugin management endpoints
        .route("/api/plugins", get(api::handlers::plugins::list_plugins))
        .route("/api/plugins", post(api::handlers::plugins::add_plugin))
        .route(
            "/api/plugins/{id}",
            put(api::handlers::plugins::update_plugin),
        )
        .route(
            "/api/plugins/{id}",
            delete(api::handlers::plugins::delete_plugin),
        )
        .route(
            "/api/plugins/{id}/toggle",
            put(api::handlers::plugins::toggle_plugin),
        )
        // Statistics endpoints
        .route("/api/stats/cost", get(api::handlers::stats::cost_overview))
        .route(
            "/api/stats/cost/today",
            get(api::handlers::stats::cost_today),
        )
        .route("/api/stats/cost/week", get(api::handlers::stats::cost_week))
        .route(
            "/api/stats/cost/month",
            get(api::handlers::stats::cost_month),
        )
        .route(
            "/api/stats/cost/trend",
            get(api::handlers::stats::cost_trend),
        )
        .route(
            "/api/stats/cost/by-model",
            get(api::handlers::stats::cost_by_model),
        )
        .route(
            "/api/stats/cost/by-project",
            get(api::handlers::stats::cost_by_project),
        )
        .route(
            "/api/stats/provider-usage",
            get(api::handlers::stats::provider_usage),
        )
        .route(
            "/api/stats/cost/top-sessions",
            get(api::handlers::stats::cost_top_sessions),
        )
        .route(
            "/api/stats/summary",
            get(api::handlers::stats::stats_summary),
        )
        // Usage Analytics endpoints
        .route(
            "/api/usage/records",
            get(api::handlers::usage::get_usage_records),
        )
        // Sync (WebDAV) endpoints
        .route(
            "/api/sync/status",
            get(api::handlers::sync::get_sync_status),
        )
        .route("/api/sync/push", post(api::handlers::sync::push_config))
        .route("/api/sync/pull", post(api::handlers::sync::pull_config))
        .route("/api/sync/info", get(api::handlers::sync::get_sync_info))
        .route(
            "/api/sync/config",
            post(api::handlers::sync::configure_sync),
        )
        // Sync Folder Management endpoints (Multi-folder sync v2.5+)
        .route(
            "/api/sync/folders",
            get(api::handlers::sync::list_sync_folders),
        )
        .route(
            "/api/sync/folders",
            post(api::handlers::sync::add_sync_folder),
        )
        .route(
            "/api/sync/folders/{name}",
            delete(api::handlers::sync::remove_sync_folder),
        )
        .route(
            "/api/sync/folders/{name}",
            get(api::handlers::sync::get_sync_folder_info),
        )
        .route(
            "/api/sync/folders/{name}/enable",
            put(api::handlers::sync::enable_sync_folder),
        )
        .route(
            "/api/sync/folders/{name}/disable",
            put(api::handlers::sync::disable_sync_folder),
        )
        .route(
            "/api/sync/folders/{name}/push",
            post(api::handlers::sync::push_sync_folder),
        )
        .route(
            "/api/sync/folders/{name}/pull",
            post(api::handlers::sync::pull_sync_folder),
        )
        .route(
            "/api/sync/folders/{name}/status",
            get(api::handlers::sync::get_sync_folder_status),
        )
        // Sync Batch Operations endpoints
        .route(
            "/api/sync/all/push",
            post(api::handlers::sync::push_all_folders),
        )
        .route(
            "/api/sync/all/pull",
            post(api::handlers::sync::pull_all_folders),
        )
        .route(
            "/api/sync/all/status",
            get(api::handlers::sync::get_all_folders_status),
        )
        // Codex MCP server management endpoints
        .route(
            "/api/codex/mcp",
            get(api::handlers::platforms::codex::list_codex_mcp_servers),
        )
        .route(
            "/api/codex/mcp",
            post(api::handlers::platforms::codex::add_codex_mcp_server),
        )
        .route(
            "/api/codex/mcp/{name}",
            put(api::handlers::platforms::codex::update_codex_mcp_server),
        )
        .route(
            "/api/codex/mcp/{name}",
            delete(api::handlers::platforms::codex::delete_codex_mcp_server),
        )
        // Codex Profile management endpoints
        .route(
            "/api/codex/profiles",
            get(api::handlers::platforms::codex::list_codex_profiles),
        )
        .route(
            "/api/codex/profiles",
            post(api::handlers::platforms::codex::add_codex_profile),
        )
        .route(
            "/api/codex/profiles/{name}",
            put(api::handlers::platforms::codex::update_codex_profile),
        )
        .route(
            "/api/codex/profiles/{name}",
            delete(api::handlers::platforms::codex::delete_codex_profile),
        )
        // Codex base config management endpoints
        .route(
            "/api/codex/config",
            get(api::handlers::platforms::codex::get_codex_config),
        )
        .route(
            "/api/codex/config",
            put(api::handlers::platforms::codex::update_codex_base_config),
        )
        // Gemini MCP server management endpoints
        .route(
            "/api/gemini/mcp",
            get(api::handlers::platforms::gemini::list_gemini_mcp_servers),
        )
        .route(
            "/api/gemini/mcp",
            post(api::handlers::platforms::gemini::add_gemini_mcp_server),
        )
        .route(
            "/api/gemini/mcp/{name}",
            put(api::handlers::platforms::gemini::update_gemini_mcp_server),
        )
        .route(
            "/api/gemini/mcp/{name}",
            delete(api::handlers::platforms::gemini::delete_gemini_mcp_server),
        )
        // Gemini base config management endpoints
        .route(
            "/api/gemini/config",
            get(api::handlers::platforms::gemini::get_gemini_config),
        )
        .route(
            "/api/gemini/config",
            put(api::handlers::platforms::gemini::update_gemini_config),
        )
        // Qwen MCP server management endpoints
        .route(
            "/api/qwen/mcp",
            get(api::handlers::platforms::qwen::list_qwen_mcp_servers),
        )
        .route(
            "/api/qwen/mcp",
            post(api::handlers::platforms::qwen::add_qwen_mcp_server),
        )
        .route(
            "/api/qwen/mcp/{name}",
            put(api::handlers::platforms::qwen::update_qwen_mcp_server),
        )
        .route(
            "/api/qwen/mcp/{name}",
            delete(api::handlers::platforms::qwen::delete_qwen_mcp_server),
        )
        // Qwen base config management endpoints
        .route(
            "/api/qwen/config",
            get(api::handlers::platforms::qwen::get_qwen_config),
        )
        .route(
            "/api/qwen/config",
            put(api::handlers::platforms::qwen::update_qwen_config),
        )
        // Config Converter endpoints
        .route(
            "/api/converter/convert",
            post(api::handlers::converter::convert_config),
        )
        // Apply middleware
        .layer(middleware)
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

/// Setup logging with daily rotation and 14-day retention
fn setup_logging() -> std::io::Result<()> {
    // Create logs directory
    let log_dir = PathBuf::from("logs");
    fs::create_dir_all(&log_dir)?;

    // Create file appender with daily rotation
    let file_appender = tracing_appender::rolling::daily(&log_dir, "backend.log");

    // Create stdout layer for console output (only warn and above)
    let stdout_filter = EnvFilter::new("warn,ccr_ui_backend=warn");
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_filter(stdout_filter);

    // Create file layer for file output (all levels including info, debug)
    let file_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,ccr_ui_backend=info"));
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_filter(file_filter);

    // Combine layers
    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();

    // Log initialization
    info!("Logging initialized - logs directory: {:?}", log_dir);

    Ok(())
}
