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
mod database; // 统一 SQLite 存储模块
mod managers;
mod models;
mod routes; // 新增路由模块
mod services;
mod state; // 统一应用状态模块
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

fn env_flag(name: &str, default: bool) -> bool {
    std::env::var(name)
        .ok()
        .map(|v| {
            let normalized = v.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(default)
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(default)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize logger with file rotation
    let log_config = setup_logging()?;

    // Spawn async cleanup task for old logs
    core::log_manager::spawn_cleanup_task(log_config.log_dir, log_config.retention_days);

    // Initialize unified SQLite database
    if let Err(e) = database::initialize() {
        warn!("Failed to initialize database: {}", e);
        warn!("Some features may be unavailable");
    }

    // Initialize unified AppState
    let app_state = match state::AppState::initialize() {
        Ok(state) => {
            info!("AppState initialized successfully");
            state
        }
        Err(e) => {
            warn!("Failed to initialize AppState: {}", e);
            warn!("Using fallback initialization");
            // Fallback: 创建最小化的 AppState
            state::AppState::new(
                database::create_app_pool().expect("Database pool required"),
                reqwest::Client::new(),
                std::sync::Arc::new(services::websocket::WsState::new()),
                cache::GLOBAL_SETTINGS_CACHE.clone(),
                services::checkin_service::CheckinService::default_checkin_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from(".ccr/checkin")),
            )
        }
    };

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

    // Build the router with modular routes and AppState
    let app = routes::apply_middleware(routes::create_app(app_state));

    let usage_import_initial_delay_secs = env_u64("USAGE_IMPORT_INITIAL_DELAY_SECS", 25);
    let session_index_initial_delay_secs = env_u64("SESSION_INDEX_INITIAL_DELAY_SECS", 30);

    // 后台使用数据导入调度（每 60s 增量导入各平台 JSONL）
    tokio::spawn(async move {
        use services::usage_import_service::{ImportConfig, UsageImportService};
        use tokio::time::{Duration, interval, sleep};

        // 默认延迟 25s，避免与首批前端请求争抢 I/O 和阻塞线程
        sleep(Duration::from_secs(usage_import_initial_delay_secs)).await;

        // 启动时执行一次导入（使用 spawn_blocking 避免阻塞异步工作线程）
        let _ = tokio::task::spawn_blocking(|| {
            let svc = UsageImportService::new(ImportConfig::default());
            for platform in &["claude", "codex", "gemini"] {
                if let Err(e) = svc.import_platform(platform) {
                    warn!("Initial usage import for {} failed: {}", platform, e);
                }
            }
            info!("Initial usage data import complete");
        })
        .await;

        // 每 60s 增量导入
        let mut tick = interval(Duration::from_secs(60));
        loop {
            tick.tick().await;
            let _ = tokio::task::spawn_blocking(|| {
                let svc = UsageImportService::new(ImportConfig::default());
                for platform in &["claude", "codex", "gemini"] {
                    let _ = svc.import_platform(platform);
                }
            })
            .await;
        }
    });

    if env_flag("SESSIONS_DAILY_CACHE", true) {
        tokio::spawn(async move {
            use ccr::sessions::SessionIndexer;
            use tokio::time::{Duration, interval, sleep};

            // 默认延迟 30s，降低冷启动阶段的资源竞争
            sleep(Duration::from_secs(session_index_initial_delay_secs)).await;

            // 初始索引（使用 spawn_blocking 避免阻塞异步工作线程）
            let _ = tokio::task::spawn_blocking(|| {
                if let Ok(indexer) = SessionIndexer::new() {
                    let _ = indexer.index_all();
                }
            })
            .await;

            let mut tick = interval(Duration::from_secs(60));
            loop {
                tick.tick().await;
                let _ = tokio::task::spawn_blocking(|| match SessionIndexer::new() {
                    Ok(indexer) => {
                        let _ = indexer.index_all();
                    }
                    Err(e) => {
                        warn!(
                            "Failed to create SessionIndexer for background refresh: {}",
                            e
                        );
                    }
                })
                .await;
            }
        });
        info!("Session daily cache background index refresh enabled");
    } else {
        info!("Session daily cache background index refresh disabled");
    }

    // 启动后 1s 预热 CLI 版本缓存（fast 模式，非阻塞）
    tokio::spawn(async {
        use tokio::time::{Duration, sleep};
        sleep(Duration::from_secs(1)).await;
        crate::api::handlers::version::trigger_cli_versions_prewarm();
        info!("CLI versions cache prewarm triggered");
    });

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
