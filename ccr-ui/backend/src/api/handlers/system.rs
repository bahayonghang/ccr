// System Information Handler

use axum::response::IntoResponse;
use std::time::{Duration, Instant};
use sysinfo::System;

use crate::models::api::{ApiResponse, SystemInfoResponse};

/// GET /api/system - Get system information
pub async fn get_system_info() -> impl IntoResponse {
    let started = Instant::now();
    let info = tokio::task::spawn_blocking(collect_system_info).await;

    match info {
        Ok(info) => {
            tracing::info!(
                elapsed_ms = started.elapsed().as_millis() as u64,
                "system info endpoint finished"
            );
            ApiResponse::success(info)
        }
        Err(e) => ApiResponse::error(format!("Failed to get system info: {}", e)),
    }
}

fn collect_system_info() -> SystemInfoResponse {
    // 使用按需刷新替代 refresh_all，降低一次性扫描开销
    let mut sys = System::new();
    sys.refresh_memory();
    sys.refresh_cpu_all();

    // 短暂采样窗口获取 CPU 使用率
    std::thread::sleep(Duration::from_millis(80));
    sys.refresh_cpu_all();

    let os = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());

    let cpu_brand = sys
        .cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let cpu_cores = num_cpus::get();
    let cpu_usage = if sys.cpus().is_empty() {
        0.0
    } else {
        sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32
    };

    let total_memory = sys.total_memory() as f64;
    let used_memory = sys.used_memory() as f64;
    let total_memory_gb = total_memory / (1024.0 * 1024.0 * 1024.0);
    let used_memory_gb = used_memory / (1024.0 * 1024.0 * 1024.0);
    let memory_usage_percent = if total_memory > 0.0 {
        (used_memory / total_memory * 100.0) as f32
    } else {
        0.0
    };

    let total_swap_gb = sys.total_swap() as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_swap_gb = sys.used_swap() as f64 / (1024.0 * 1024.0 * 1024.0);
    let uptime_seconds = System::uptime();

    SystemInfoResponse {
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
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_system_info_fields_are_valid() {
        let info = collect_system_info();

        assert!(!info.hostname.is_empty());
        assert!(!info.os.is_empty());
        assert!(info.cpu_cores > 0);
        assert!(info.total_memory_gb >= 0.0);
        assert!(info.used_memory_gb >= 0.0);
        assert!(info.total_swap_gb >= 0.0);
        assert!(info.used_swap_gb >= 0.0);
        assert!((0.0..=100.0).contains(&info.memory_usage_percent));
    }
}
