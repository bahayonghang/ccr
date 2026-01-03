// CCR UI Backend Server
// Axum server that executes CCR CLI commands as subprocesses

use chrono::Local;
use clap::Parser;
use std::net::SocketAddr;
use tracing::{info, warn};
use tracing_log::LogTracer;
use tracing_subscriber::{
    EnvFilter, Layer,
    fmt::{self, time::FormatTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

// New layered architecture modules
mod api;
mod cache; // 全局缓存模块
mod core;
mod managers;
mod models;
mod routes; // 新增路由模块
mod services;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to bind the server to
    #[arg(short, long, default_value = "38081")]
    port: u16,

    /// Host to bind the server to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

/// Custom time formatter for local timezone
struct LocalTime;

impl FormatTime for LocalTime {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%Y-%m-%d %H:%M:%S"))
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logger with file rotation
    let log_config = setup_logging()?;

    // Spawn async cleanup task for old logs
    core::log_manager::spawn_cleanup_task(log_config.log_dir, log_config.retention_days);

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

    // Build the router with modular routes (先启动服务器，不阻塞)
    let app = routes::apply_middleware(routes::create_app());

    // 异步验证 CCR 是否可用（不阻塞服务器启动）
    tokio::spawn(async {
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
                warn!("Server running without CCR; API calls that require CCR will return errors.");
            }
        }
    });

    // Start the server
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    info!("Server started successfully, listening on {}", bind_addr);

    axum::serve(listener, app)
        .await
        .map_err(std::io::Error::other)
}

/// Setup logging with daily rotation and configurable retention
/// Returns the log configuration for use by the cleanup task
fn setup_logging() -> std::io::Result<core::log_manager::LogConfig> {
    // Bridge log facade to tracing (for dependencies like reqwest)
    let _ = LogTracer::init();

    // Load configuration from environment
    let config = core::log_manager::LogConfig::from_env();

    // Create log directories synchronously at startup
    core::log_manager::create_log_directories_sync(&config.log_dir)?;

    // Create file appender with daily rotation in backend subdirectory
    let backend_log_dir = config.log_dir.join("backend");
    let file_appender = tracing_appender::rolling::daily(&backend_log_dir, "");

    // Wrap with BOM writer for Windows compatibility (ensures UTF-8 BOM is added)
    let bom_writer = core::bom_writer::MakeBomWriter::new(file_appender, &backend_log_dir);

    // Console layer: colored output with local time (only warn and above for less noise)
    let console_filter = EnvFilter::new("warn,ccr_ui_backend=info");
    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_timer(LocalTime)
        .with_ansi(true)
        .with_filter(console_filter);

    // File layer: no colors, local time, with UTF-8 BOM for Windows compatibility
    let file_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level));
    let file_layer = fmt::layer()
        .with_writer(bom_writer)
        .with_timer(LocalTime)
        .with_ansi(false)
        .with_filter(file_filter);

    // Combine layers（使用 try_init 避免重复初始化时 panic）
    let _ = tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .try_init();

    // Log initialization (may not be visible if already initialized elsewhere)
    info!("Logging initialized - logs directory: {:?}", config.log_dir);
    info!(
        "Log retention: {} days, level: {}",
        config.retention_days, config.log_level
    );

    Ok(config)
}
