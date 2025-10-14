// CCR UI Backend Server
// Actix Web server that executes CCR CLI commands as subprocesses

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

mod config_reader;
mod executor;
mod handlers;
mod models;
mod claude_config_manager;
mod markdown_manager;
mod plugins_manager;

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logger with file rotation
    setup_logging()?;

    // Parse command line arguments (already done above)
    // let args = Args::parse();

    let bind_addr = format!("{}:{}", args.host, args.port);

    tracing::info!("Starting CCR UI Backend Server");
    tracing::info!("Server binding to: {}", bind_addr);
    tracing::info!("API endpoints:");
    tracing::info!("  - Config Management: http://{}/api/configs", bind_addr);
    tracing::info!("  - Command Execution: http://{}/api/command/execute", bind_addr);
    tracing::info!("  - Command List: http://{}/api/command/list", bind_addr);

    // Verify CCR is available
    match executor::execute_command(vec!["version".to_string()]).await {
        Ok(output) if output.success => {
            tracing::info!("CCR binary found and working");
            tracing::info!("CCR Version: {}", output.stdout.trim());
        }
        Ok(output) => {
            tracing::warn!("CCR binary found but returned error: {}", output.stderr);
        }
        Err(e) => {
            tracing::error!("CCR binary not found or not working: {}", e);
            tracing::error!("Please ensure 'ccr' is installed and in your PATH");
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "CCR binary not found",
            ));
        }
    }

    // Start HTTP server
    HttpServer::new(|| {
        // Configure CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            // Config management endpoints
            .service(handlers::list_configs)
            .service(handlers::switch_config)
            .service(handlers::validate_configs)
            .service(handlers::clean_backups)
            .service(handlers::export_config)
            .service(handlers::import_config)
            .service(handlers::get_history)
            .service(handlers::add_config)
            .service(handlers::update_config)
            .service(handlers::delete_config)
            // Command execution endpoints
            .service(handlers::execute_command)
            .service(handlers::list_commands)
            .service(handlers::get_command_help)
            // System info endpoint
            .service(handlers::get_system_info)
            // Version management endpoints
            .service(handlers::get_version)
            .service(handlers::check_update)
            .service(handlers::update_ccr)
            // MCP server management endpoints
            .service(handlers::list_mcp_servers)
            .service(handlers::add_mcp_server)
            .service(handlers::update_mcp_server)
            .service(handlers::delete_mcp_server)
            .service(handlers::toggle_mcp_server)
            // Slash command management endpoints
            .service(handlers::list_slash_commands)
            .service(handlers::add_slash_command)
            .service(handlers::update_slash_command)
            .service(handlers::delete_slash_command)
            .service(handlers::toggle_slash_command)
            // Agent management endpoints
            .service(handlers::list_agents)
            .service(handlers::add_agent)
            .service(handlers::update_agent)
            .service(handlers::delete_agent)
            .service(handlers::toggle_agent)
            // Plugin management endpoints
            .service(handlers::list_plugins)
            .service(handlers::add_plugin)
            .service(handlers::update_plugin)
            .service(handlers::delete_plugin)
            .service(handlers::toggle_plugin)
            // Health check
            .route(
                "/health",
                web::get().to(|| async { actix_web::HttpResponse::Ok().body("OK") }),
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}

/// Setup logging with daily rotation and 14-day retention
fn setup_logging() -> std::io::Result<()> {
    // Create logs directory
    let log_dir = PathBuf::from("logs");
    std::fs::create_dir_all(&log_dir)?;

    // Create file appender with daily rotation
    let file_appender = tracing_appender::rolling::daily(&log_dir, "backend.log");
    
    // Create stdout layer for console output (only warn and above)
    // 终端输出：只显示 warning、error、critical 级别
    let stdout_filter = EnvFilter::new("warn,ccr_ui_backend=warn,actix_web=warn");
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_filter(stdout_filter);

    // Create file layer for file output (all levels including info, debug)
    // 文件输出：记录所有级别的日志
    let file_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,ccr_ui_backend=info,actix_web=info"));
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_filter(file_filter);

    // Combine layers (每个层都有自己的过滤器)
    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();

    // Log cleanup task will be handled by external script
    // 这条 info 日志只会写入文件，不会显示在终端
    tracing::info!("Logging initialized - logs directory: {:?}", log_dir);
    
    Ok(())
}

