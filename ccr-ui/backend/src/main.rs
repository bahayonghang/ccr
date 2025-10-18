// CCR UI Backend Server
// Axum server that executes CCR CLI commands as subprocesses

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use clap::Parser;
use std::{fs, net::SocketAddr, path::PathBuf};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::{info, warn, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

mod claude_config_manager;
mod codex_config_manager;
mod codex_models;
mod config_converter;
mod config_reader;
mod converter_models;
mod executor;
mod gemini_config_manager;
mod gemini_models;
mod handlers;
mod markdown_manager;
mod models;
mod plugins_manager;
mod qwen_config_manager;
mod qwen_models;
mod settings_manager;

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
    info!("  - Command Execution: http://{}/api/command/execute", bind_addr);
    info!("  - System Info: http://{}/api/system", bind_addr);
    info!("  - Version Info: http://{}/api/version", bind_addr);

    // Verify CCR is available
    match executor::execute_command(vec!["version".to_string()]).await {
        Ok(output) if output.success => {
            info!("CCR binary found and working");
            info!("CCR Version: {}", output.stdout.trim());
        }
        Ok(output) => {
            warn!("CCR binary found but returned error: {}", output.stderr);
        }
        Err(e) => {
            warn!("CCR binary not found or not working: {}", e);
            warn!("Continuing to start server without CCR; API calls that require CCR will return errors.");
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
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

/// Create the main application router with all middlewares and routes
fn create_router() -> Router {
    // Create middleware stack
    let middleware = ServiceBuilder::new()
        // Logging middleware
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
        .route("/api/configs", get(handlers::config::list_configs))
        .route("/api/switch", post(handlers::config::switch_config))
        .route("/api/validate", get(handlers::config::validate_configs))
        .route("/api/clean", post(handlers::config::clean_backups))
        .route("/api/export", post(handlers::config::export_config))
        .route("/api/import", post(handlers::config::import_config))
        .route("/api/history", get(handlers::config::get_history))
        .route("/api/configs", post(handlers::config::add_config))
        .route("/api/configs/:name", put(handlers::config::update_config))
        .route("/api/configs/:name", delete(handlers::config::delete_config))
        // Command execution endpoints
        .route("/api/command/execute", post(handlers::command::execute_command))
        .route("/api/command/list", get(handlers::command::list_commands))
        .route("/api/command/help/:command", get(handlers::command::get_command_help))
        // System info endpoint
        .route("/api/system", get(handlers::system::get_system_info))
        // Version management endpoints
        .route("/api/version", get(handlers::version::get_version))
        .route("/api/version/check-update", get(handlers::version::check_update))
        .route("/api/version/update", post(handlers::version::update_ccr))
        // MCP server management endpoints
        .route("/api/mcp", get(handlers::mcp::list_mcp_servers))
        .route("/api/mcp", post(handlers::mcp::add_mcp_server))
        .route("/api/mcp/:name", put(handlers::mcp::update_mcp_server))
        .route("/api/mcp/:name", delete(handlers::mcp::delete_mcp_server))
        .route("/api/mcp/:name/toggle", put(handlers::mcp::toggle_mcp_server))
        // Slash command management endpoints
        .route("/api/slash-commands", get(handlers::slash_commands::list_slash_commands))
        .route("/api/slash-commands", post(handlers::slash_commands::add_slash_command))
        .route("/api/slash-commands/:name", put(handlers::slash_commands::update_slash_command))
        .route("/api/slash-commands/:name", delete(handlers::slash_commands::delete_slash_command))
        .route("/api/slash-commands/:name/toggle", put(handlers::slash_commands::toggle_slash_command))
        // Agent management endpoints
        .route("/api/agents", get(handlers::agents::list_agents))
        .route("/api/agents", post(handlers::agents::add_agent))
        .route("/api/agents/:name", put(handlers::agents::update_agent))
        .route("/api/agents/:name", delete(handlers::agents::delete_agent))
        .route("/api/agents/:name/toggle", put(handlers::agents::toggle_agent))
        // Plugin management endpoints
        .route("/api/plugins", get(handlers::plugins::list_plugins))
        .route("/api/plugins", post(handlers::plugins::add_plugin))
        .route("/api/plugins/:id", put(handlers::plugins::update_plugin))
        .route("/api/plugins/:id", delete(handlers::plugins::delete_plugin))
        .route("/api/plugins/:id/toggle", put(handlers::plugins::toggle_plugin))
        // Sync (WebDAV) endpoints
        .route("/api/sync/status", get(handlers::sync::get_sync_status))
        .route("/api/sync/push", post(handlers::sync::push_config))
        .route("/api/sync/pull", post(handlers::sync::pull_config))
        .route("/api/sync/info", get(handlers::sync::get_sync_info))
        .route("/api/sync/config", post(handlers::sync::configure_sync))
        // Codex MCP server management endpoints
        .route("/api/codex/mcp", get(handlers::codex::list_codex_mcp_servers))
        .route("/api/codex/mcp", post(handlers::codex::add_codex_mcp_server))
        .route("/api/codex/mcp/:name", put(handlers::codex::update_codex_mcp_server))
        .route("/api/codex/mcp/:name", delete(handlers::codex::delete_codex_mcp_server))
        // Codex Profile management endpoints
        .route("/api/codex/profiles", get(handlers::codex::list_codex_profiles))
        .route("/api/codex/profiles", post(handlers::codex::add_codex_profile))
        .route("/api/codex/profiles/:name", put(handlers::codex::update_codex_profile))
        .route("/api/codex/profiles/:name", delete(handlers::codex::delete_codex_profile))
        // Codex base config management endpoints
        .route("/api/codex/config", get(handlers::codex::get_codex_config))
        .route("/api/codex/config", put(handlers::codex::update_codex_base_config))
        // Gemini MCP server management endpoints
        .route("/api/gemini/mcp", get(handlers::gemini::list_gemini_mcp_servers))
        .route("/api/gemini/mcp", post(handlers::gemini::add_gemini_mcp_server))
        .route("/api/gemini/mcp/:name", put(handlers::gemini::update_gemini_mcp_server))
        .route("/api/gemini/mcp/:name", delete(handlers::gemini::delete_gemini_mcp_server))
        // Gemini base config management endpoints
        .route("/api/gemini/config", get(handlers::gemini::get_gemini_config))
        .route("/api/gemini/config", put(handlers::gemini::update_gemini_config))
        // Qwen MCP server management endpoints
        .route("/api/qwen/mcp", get(handlers::qwen::list_qwen_mcp_servers))
        .route("/api/qwen/mcp", post(handlers::qwen::add_qwen_mcp_server))
        .route("/api/qwen/mcp/:name", put(handlers::qwen::update_qwen_mcp_server))
        .route("/api/qwen/mcp/:name", delete(handlers::qwen::delete_qwen_mcp_server))
        // Qwen base config management endpoints
        .route("/api/qwen/config", get(handlers::qwen::get_qwen_config))
        .route("/api/qwen/config", put(handlers::qwen::update_qwen_config))
        // Config Converter endpoints
        .route("/api/converter/convert", post(handlers::converter::convert_config))
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
