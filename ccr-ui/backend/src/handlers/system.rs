// System Information Handler
// Get system information using sysinfo crate

use actix_web::{get, HttpResponse, Responder};

use crate::models::*;

/// GET /api/system - Get system information
#[get("/api/system")]
pub async fn get_system_info() -> impl Responder {
    // Execute 'ccr version' to get basic info
    let version_result = crate::executor::execute_command(vec!["version".to_string()]).await;

    // Build a simple system info response
    let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string());
    let os = std::env::consts::OS.to_string();

    let response = SystemInfoResponse {
        hostname,
        os: os.clone(),
        os_version: os,
        kernel_version: "N/A".to_string(),
        cpu_brand: "N/A".to_string(),
        cpu_cores: num_cpus::get(),
        cpu_usage: 0.0,
        total_memory_gb: 0.0,
        used_memory_gb: 0.0,
        memory_usage_percent: 0.0,
        total_swap_gb: 0.0,
        used_swap_gb: 0.0,
        uptime_seconds: 0,
    };

    if version_result.is_ok() {
        HttpResponse::Ok().json(ApiResponse::success(response))
    } else {
        HttpResponse::Ok().json(ApiResponse::success(response))
    }
}

