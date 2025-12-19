// CCR UI Backend Server
// Axum server that executes CCR CLI commands as subprocesses

use clap::Parser;
use std::{fs, net::SocketAddr, path::PathBuf};
use tracing::{info, warn};
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

// New layered architecture modules
mod api;
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

    // Build the router with modular routes (先启动服务器，不阻塞)
    let app = routes::create_app();

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
                warn!(
                    "Server running without CCR; API calls that require CCR will return errors."
                );
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

/// Setup logging with daily rotation and 14-day retention
fn setup_logging() -> std::io::Result<()> {
    // Bridge log facade to tracing (for dependencies like reqwest)
    let _ = LogTracer::init();

    // 使用可执行文件所在目录的 logs 子目录，避免依赖工作目录
    let log_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("logs")))
        .unwrap_or_else(|| PathBuf::from("logs"));
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

    // Combine layers（使用 try_init 避免重复初始化时 panic）
    let _ = tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .try_init();

    // Log initialization (may not be visible if already initialized elsewhere)
    info!("Logging initialized - logs directory: {:?}", log_dir);

    Ok(())
}
