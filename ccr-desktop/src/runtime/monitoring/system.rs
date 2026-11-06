//! Explicit host facts. No process enumeration, CLI, network, or background timer.
use crate::runtime::Services;
use ccr_app::runtime::{AdmissionError, EnvironmentId, RequestKey, TaskSubscription};
use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    LocalOnly,
    Fixture,
    Unsupported,
    Interrupted,
    Unavailable,
    QueueBusy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Memory {
    pub total_bytes: Option<u64>,
    pub used_bytes: Option<u64>,
}
impl Memory {
    /// sysinfo returns zero when a capacity is absent or unavailable. Preserve
    /// that uncertainty instead of publishing a fabricated zero/percentage.
    fn measured(total: u64, used: u64) -> Self {
        Self {
            total_bytes: (total > 0).then_some(total),
            used_bytes: (total > 0 && used <= total).then_some(used),
        }
    }
    pub fn used_percent(&self) -> Option<f64> {
        let (total, used) = (self.total_bytes?, self.used_bytes?);
        (total > 0 && used <= total).then(|| used as f64 / total as f64 * 100.)
    }
}

#[derive(Clone, Debug)]
pub struct Report {
    pub sampled_at: DateTime<Utc>,
    pub cpu_sample_ms: u64,
    pub hostname: Option<String>,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,
    pub arch: String,
    pub cpu_brand: Option<String>,
    /// Matches Tauri's cpus().len(), not a physical-core count.
    pub logical_cpus: Option<usize>,
    pub cpu_usage_percent: Option<f32>,
    pub memory: Memory,
    pub swap: Memory,
    pub uptime_seconds: Option<u64>,
}

fn text(value: Option<String>) -> Option<String> {
    value
        .map(|v| {
            v.trim()
                .chars()
                .filter(|c| !c.is_control())
                .collect::<String>()
        })
        .filter(|v| !v.is_empty())
}
fn average_cpu(values: &[f32]) -> Option<f32> {
    if values.is_empty()
        || values
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=100.0).contains(v))
    {
        return None;
    }
    let average = values.iter().map(|v| f64::from(*v)).sum::<f64>() / values.len() as f64;
    Some(average as f32)
}
fn collect(cancelled: &impl Fn() -> bool) -> Result<Report, Error> {
    use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, System};
    if !sysinfo::IS_SUPPORTED_SYSTEM {
        return Err(Error::Unsupported);
    }
    if cancelled() {
        return Err(Error::Interrupted);
    }
    // new_all/refresh_all would enumerate processes. Refresh only these two
    // host categories; CPU usage needs two observations, just like Tauri.
    let mut system = System::new();
    system.refresh_cpu_usage();
    let first = Instant::now();
    while first.elapsed() < MINIMUM_CPU_UPDATE_INTERVAL {
        if cancelled() {
            return Err(Error::Interrupted);
        }
        std::thread::sleep(
            Duration::from_millis(20)
                .min(MINIMUM_CPU_UPDATE_INTERVAL.saturating_sub(first.elapsed())),
        );
    }
    if cancelled() {
        return Err(Error::Interrupted);
    }
    system.refresh_cpu_usage();
    let cpu_sample_ms = first.elapsed().as_millis().min(u64::MAX as u128) as u64;
    system.refresh_memory();
    if cancelled() {
        return Err(Error::Interrupted);
    }
    let values = system
        .cpus()
        .iter()
        .map(|cpu| cpu.cpu_usage())
        .collect::<Vec<_>>();
    let uptime = sysinfo::System::uptime();
    Ok(Report {
        sampled_at: Utc::now(),
        cpu_sample_ms,
        hostname: text(System::host_name()),
        os_name: text(System::name()),
        os_version: text(System::os_version()),
        kernel_version: text(System::kernel_version()),
        arch: std::env::consts::ARCH.into(),
        cpu_brand: text(system.cpus().first().map(|cpu| cpu.brand().into())),
        logical_cpus: (!values.is_empty()).then_some(values.len()),
        cpu_usage_percent: average_cpu(&values),
        memory: Memory::measured(system.total_memory(), system.used_memory()),
        swap: Memory::measured(system.total_swap(), system.used_swap()),
        uptime_seconds: (uptime > 0).then_some(uptime),
    })
}

fn permitted<T>(
    environment: &EnvironmentId,
    synthetic: bool,
    probe: impl FnOnce() -> Result<T, Error>,
) -> Result<T, Error> {
    if !matches!(environment, EnvironmentId::Local(_)) {
        return Err(Error::LocalOnly);
    }
    if synthetic {
        return Err(Error::Fixture);
    }
    probe()
}

impl Services {
    pub fn system_info(
        &self,
        key: RequestKey,
    ) -> Result<TaskSubscription<Report, Error>, AdmissionError> {
        let synthetic = self.is_fixture();
        let environment = key.environment.clone();
        self.executor.read(key, move |cancel, _| {
            permitted(&environment, synthetic, || {
                collect(&|| cancel.is_cancelled())
            })
        })
    }
    #[cfg(test)]
    pub(crate) fn system_info_synthetic(
        &self,
        key: RequestKey,
        report: Report,
        delay: Duration,
    ) -> Result<TaskSubscription<Report, Error>, AdmissionError> {
        let synthetic = self.is_fixture();
        self.executor.read(key, move |cancel, _| {
            if !synthetic {
                return Err(Error::Fixture);
            }
            std::thread::sleep(delay);
            if cancel.is_cancelled() {
                return Err(Error::Interrupted);
            }
            Ok(report)
        })
    }
}

#[cfg(test)]
mod tests;
