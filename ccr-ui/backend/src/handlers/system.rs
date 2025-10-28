// System Information Handler

use axum::response::IntoResponse;
use sysinfo::System;

use crate::models::{ApiResponse, SystemInfoResponse};

/// GET /api/system - Get system information
pub async fn get_system_info() -> impl IntoResponse {
    // Create a System instance
    let mut sys = System::new_all();
    
    // First refresh to establish baseline
    sys.refresh_all();
    
    // Wait a bit to get accurate CPU measurements
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    // Second refresh to calculate CPU usage
    sys.refresh_cpu_all();
    
    // Get OS information
    let os = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    
    // Get hostname
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
    
    // Get CPU information
    let cpu_brand = sys
        .cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let cpu_cores = num_cpus::get();
    let cpu_usage = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;
    
    // Get memory information (convert from bytes to GB)
    let total_memory_gb = sys.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_memory_gb = sys.used_memory() as f64 / (1024.0 * 1024.0 * 1024.0);
    let memory_usage_percent = (sys.used_memory() as f32 / sys.total_memory() as f32) * 100.0;
    
    // Get swap information
    let total_swap_gb = sys.total_swap() as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_swap_gb = sys.used_swap() as f64 / (1024.0 * 1024.0 * 1024.0);
    
    // Get uptime
    let uptime_seconds = System::uptime();
    
    let info = SystemInfoResponse {
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
    
    ApiResponse::success(info)
}
