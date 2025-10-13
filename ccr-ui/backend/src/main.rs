// CCR UI Backend Server
// Actix Web server that executes CCR CLI commands as subprocesses

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod config_reader;
mod executor;
mod handlers;
mod models;

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
    
    // Create stdout layer for console output
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true);

    // Create file layer for file output
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false);

    // Set up env filter (default to "info" level)
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Combine layers
    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    // Log cleanup task will be handled by external script
    tracing::info!("Logging initialized - logs directory: {:?}", log_dir);
    
    Ok(())
}

