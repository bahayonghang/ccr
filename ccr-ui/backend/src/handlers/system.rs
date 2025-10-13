// System Information Handler
// Get system information using sysinfo crate

use actix_web::{get, HttpResponse, Responder};
use sysinfo::System;

use crate::models::*;

/// GET /api/system - Get system information
#[get("/api/system")]
pub async fn get_system_info() -> impl Responder {
    // Initialize system information collector
    let mut sys = System::new_all();
    
    // Refresh all system information
    sys.refresh_all();
    
    // Wait a bit to get accurate CPU usage
    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu_all();

    // Get hostname
    let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string());
    
    // Get OS information
    let os = std::env::consts::OS.to_string();
    let os_version = System::long_os_version().unwrap_or_else(|| os.clone());
    let kernel_version = System::kernel_version().unwrap_or_else(|| "N/A".to_string());
    
    // Get CPU information
    let cpu_brand = sys.cpus().first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let cpu_cores = sys.cpus().len();
    let cpu_usage = sys.global_cpu_usage();
    
    // Get memory information (convert from bytes to GB)
    let total_memory_gb = sys.total_memory() as f64 / 1_073_741_824.0; // 1024^3
    let used_memory_gb = sys.used_memory() as f64 / 1_073_741_824.0;
    let memory_usage_percent = if total_memory_gb > 0.0 {
        (used_memory_gb / total_memory_gb * 100.0) as f32
    } else {
        0.0
    };
    
    // Get swap information
    let total_swap_gb = sys.total_swap() as f64 / 1_073_741_824.0;
    let used_swap_gb = sys.used_swap() as f64 / 1_073_741_824.0;
    
    // Get system uptime
    let uptime_seconds = System::uptime();

    let response = SystemInfoResponse {
        hostname,
        os,
        os_version,
        kernel_version,
        cpu_brand,
        cpu_cores,
        cpu_usage,
        total_memory_gb,
        used_memory_gb,
        memory_usage_percent,
        total_swap_gb,
        used_swap_gb,
        uptime_seconds,
    };

    tracing::info!(
        "System info: CPU: {:.1}%, Memory: {:.2}/{:.2} GB ({:.1}%), Uptime: {}s",
        cpu_usage,
        used_memory_gb,
        total_memory_gb,
        memory_usage_percent,
        uptime_seconds
    );

    HttpResponse::Ok().json(ApiResponse::success(response))
}

