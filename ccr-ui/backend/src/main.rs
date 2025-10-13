// CCR UI Backend Server
// Actix Web server that executes CCR CLI commands as subprocesses

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use clap::Parser;

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
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Parse command line arguments
    let args = Args::parse();

    let bind_addr = format!("{}:{}", args.host, args.port);

    log::info!("Starting CCR UI Backend Server");
    log::info!("Server binding to: {}", bind_addr);
    log::info!("API endpoints:");
    log::info!("  - Config Management: http://{}/api/configs", bind_addr);
    log::info!("  - Command Execution: http://{}/api/command/execute", bind_addr);
    log::info!("  - Command List: http://{}/api/command/list", bind_addr);

    // Verify CCR is available
    match executor::execute_command(vec!["version".to_string()]).await {
        Ok(output) if output.success => {
            log::info!("CCR binary found and working");
            log::info!("CCR Version: {}", output.stdout.trim());
        }
        Ok(output) => {
            log::warn!("CCR binary found but returned error: {}", output.stderr);
        }
        Err(e) => {
            log::error!("CCR binary not found or not working: {}", e);
            log::error!("Please ensure 'ccr' is installed and in your PATH");
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

