// 🎯 系统信息缓存模块
// 提供高性能的系统信息查询，避免每次请求都扫描系统

use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use sysinfo::System;

/// 📊 缓存的系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSystemInfo {
    pub hostname: String,
    pub os: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub cpu_usage: f32,
    pub total_memory_gb: f64,
    pub used_memory_gb: f64,
    pub memory_usage_percent: f32,
    pub total_swap_gb: f64,
    pub used_swap_gb: f64,
    pub uptime_seconds: u64,
}

/// 🔄 系统信息缓存管理器
///
/// 特性：
/// - 🎯 后台线程定期更新系统信息
/// - 🔒 线程安全的读写锁
/// - ⚡ 极低延迟的读取（无阻塞）
pub struct SystemInfoCache {
    cache: Arc<RwLock<CachedSystemInfo>>,
}

impl SystemInfoCache {
    /// 🏗️ 创建新的系统信息缓存
    ///
    /// # Arguments
    /// - `update_interval` - 更新间隔（建议 2-5 秒）
    ///
    /// # Returns
    /// 返回缓存管理器，并自动启动后台更新线程
    pub fn new(update_interval: Duration) -> Self {
        // 🎯 首次获取系统信息
        let initial_info = Self::fetch_system_info();
        let cache = Arc::new(RwLock::new(initial_info));

        // 🔄 启动后台更新线程
        let cache_clone = Arc::clone(&cache);
        thread::spawn(move || {
            log::info!("🔄 系统信息缓存后台线程已启动，更新间隔: {:?}", update_interval);
            loop {
                thread::sleep(update_interval);
                let new_info = Self::fetch_system_info();

                if let Ok(mut cached) = cache_clone.write() {
                    *cached = new_info;
                    log::trace!("✅ 系统信息已更新");
                } else {
                    log::warn!("⚠️ 无法获取写锁更新系统信息");
                }
            }
        });

        Self { cache }
    }

    /// 📖 读取缓存的系统信息
    ///
    /// 这是一个非常快速的操作，因为数据已经在内存中
    pub fn get(&self) -> CachedSystemInfo {
        self.cache
            .read()
            .expect("读取系统信息缓存失败")
            .clone()
    }

    /// 🔍 获取系统信息（内部方法）
    fn fetch_system_info() -> CachedSystemInfo {
        // 🎯 只创建需要的组件
        let mut sys = System::new();
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
        let os = System::name().unwrap_or_else(|| "Unknown".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
        let kernel_version = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());

        // 获取 CPU 信息
        let cpus = sys.cpus();
        let cpu_brand = if !cpus.is_empty() {
            cpus[0].brand().to_string()
        } else {
            "Unknown".to_string()
        };
        let cpu_cores = cpus.len();

        // 计算平均 CPU 使用率
        let cpu_usage = if !cpus.is_empty() {
            cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cpus.len() as f32
        } else {
            0.0
        };

        let total_memory = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_memory = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let memory_usage_percent = (used_memory / total_memory * 100.0) as f32;

        let total_swap = sys.total_swap() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used_swap = sys.used_swap() as f64 / 1024.0 / 1024.0 / 1024.0;

        let uptime = System::uptime();

        CachedSystemInfo {
            hostname,
            os,
            os_version,
            kernel_version,
            cpu_brand,
            cpu_cores,
            cpu_usage,
            total_memory_gb: total_memory,
            used_memory_gb: used_memory,
            memory_usage_percent,
            total_swap_gb: total_swap,
            used_swap_gb: used_swap,
            uptime_seconds: uptime,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_info_cache() {
        let cache = SystemInfoCache::new(Duration::from_secs(1));

        // 第一次读取
        let info1 = cache.get();
        assert!(!info1.hostname.is_empty());
        assert!(info1.cpu_cores > 0);

        // 等待一段时间后再次读取（应该得到更新的数据）
        thread::sleep(Duration::from_millis(1200));
        let info2 = cache.get();

        // CPU 使用率可能会变化
        // 但核心数量应该保持不变
        assert_eq!(info1.cpu_cores, info2.cpu_cores);
        assert_eq!(info1.hostname, info2.hostname);
    }

    #[test]
    fn test_fetch_system_info() {
        let info = SystemInfoCache::fetch_system_info();
        assert!(!info.hostname.is_empty());
        assert!(info.cpu_cores > 0);
        assert!(info.total_memory_gb > 0.0);
    }
}
