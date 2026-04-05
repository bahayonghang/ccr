//! 系统命令模块，提供系统信息、版本检查、健康检查、监控事件查询与运行时指标采样。

use ccr_types::{FrontendLogInput, MonitoringEntry, MonitoringFeedQuery};

use serde::{Deserialize, Serialize};
use std::{io::ErrorKind, sync::Arc, time::Instant};
use tauri::State;
use tokio::sync::Semaphore;
use tokio::time::{Duration, timeout};

use crate::monitoring::{
    event_to_monitoring_entry, frontend_log_entry, record_monitoring_entry, should_persist,
};
use crate::process::tokio_command;
use crate::state::{AppState, CacheFillRegistration};

/// 系统信息响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub arch: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub cpu_count: usize,
    pub cpu_usage: f32,
    pub total_memory_gb: f64,
    pub used_memory_gb: f64,
    pub memory_usage_percent: f64,
    pub total_memory_mb: u64,
    pub total_swap_gb: f64,
    pub used_swap_gb: f64,
    pub uptime_seconds: u64,
    pub ccr_version: String,
}

/// 版本检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub current: String,
    pub latest: Option<String>,
    pub update_available: bool,
}

/// 运行时指标响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMetricsResponse {
    pub cache_entries: usize,
    pub event_log_entries: usize,
    pub event_log_memory_bytes: usize,
    pub ssh_state_count: usize,
    pub ssh_password_cache_count: usize,
    pub process_rss_bytes: u64,
    pub command_p95_ms: Option<f64>,
    pub db_query_p95_ms: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliVersionsOptions {
    pub mode: Option<String>,
    pub timeout_ms: Option<u64>,
    pub parallelism: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliVersionOptions {
    pub tool: String,
    pub timeout_ms: Option<u64>,
    pub force: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliVersionEntry {
    pub platform: String,
    pub installed: bool,
    pub version: Option<String>,
    pub status: String,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Copy)]
enum CliProbeMode {
    Fast,
    Full,
}

impl CliProbeMode {
    fn from_options(options: &CliVersionsOptions) -> Self {
        match options
            .mode
            .as_deref()
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("full") => Self::Full,
            _ => Self::Fast,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Full => "full",
        }
    }

    fn default_timeout_ms(&self) -> u64 {
        match self {
            Self::Fast => 3_500,
            Self::Full => 10_000,
        }
    }

    fn default_parallelism(&self) -> usize {
        match self {
            Self::Fast => 4,
            Self::Full => 4,
        }
    }
}

#[tauri::command]
pub async fn get_system_info() -> Result<SystemInfo, String> {
    tokio::task::spawn_blocking(|| {
        use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, System};

        const BYTES_PER_GIB: f64 = 1024.0 * 1024.0 * 1024.0;

        let mut sys = System::new_all();
        sys.refresh_cpu_usage();
        std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_cpu_usage();
        sys.refresh_memory();

        let cpu_cores = sys.cpus().len();
        let cpu_usage = if cpu_cores == 0 {
            0.0
        } else {
            sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cpu_cores as f32
        };

        let total_memory_bytes = sys.total_memory();
        let used_memory_bytes = sys.used_memory();
        let total_swap_bytes = sys.total_swap();
        let used_swap_bytes = sys.used_swap();
        let total_memory_gb = total_memory_bytes as f64 / BYTES_PER_GIB;
        let used_memory_gb = used_memory_bytes as f64 / BYTES_PER_GIB;
        let total_swap_gb = total_swap_bytes as f64 / BYTES_PER_GIB;
        let used_swap_gb = used_swap_bytes as f64 / BYTES_PER_GIB;
        let memory_usage_percent = if total_memory_bytes == 0 {
            0.0
        } else {
            (used_memory_bytes as f64 / total_memory_bytes as f64) * 100.0
        };
        let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());

        Ok::<_, String>(SystemInfo {
            hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            os: os_name.clone(),
            os_name,
            os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            arch: std::env::consts::ARCH.to_string(),
            cpu_brand: sys
                .cpus()
                .first()
                .map(|cpu| cpu.brand().trim().to_string())
                .filter(|brand| !brand.is_empty())
                .unwrap_or_else(|| "Unknown".to_string()),
            cpu_cores,
            cpu_count: cpu_cores,
            cpu_usage,
            total_memory_gb,
            used_memory_gb,
            memory_usage_percent,
            total_memory_mb: total_memory_bytes / 1024 / 1024,
            total_swap_gb,
            used_swap_gb,
            uptime_seconds: System::uptime(),
            ccr_version: env!("CARGO_PKG_VERSION").to_string(),
        })
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}

#[tauri::command]
pub async fn check_version() -> Result<VersionInfo, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    // TODO: Phase B4 接入 GitHub Releases API
    Ok(VersionInfo {
        current,
        latest: None,
        update_available: false,
    })
}

#[tauri::command]
pub async fn health_check(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let pool = state.db_pool.clone();
    let db_ok = tokio::task::spawn_blocking(move || pool.get().map(|_| true).unwrap_or(false))
        .await
        .map_err(|e| format!("Task join error: {e}"))?;

    Ok(serde_json::json!({
        "status": if db_ok { "healthy" } else { "degraded" },
        "database": db_ok,
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

fn monitoring_matches_query(entry: &MonitoringEntry, query: &MonitoringFeedQuery) -> bool {
    let level_matches = query
        .level
        .map(|level| entry.level == level)
        .unwrap_or(true);
    let channel_matches = query
        .channel
        .as_deref()
        .map(|channel| entry.channel == channel)
        .unwrap_or(true);
    level_matches && channel_matches
}

#[tauri::command]
pub async fn get_monitoring_feed(
    state: State<'_, AppState>,
    query: Option<MonitoringFeedQuery>,
) -> Result<Vec<MonitoringEntry>, String> {
    let query = query.unwrap_or_default();
    let count = query.count.unwrap_or(100).clamp(1, 500);
    let level = query.level.map(|item| item.as_str().to_string());
    let channel = query.channel.clone();

    let mut entries = state
        .monitoring_logs
        .query_logs(level.as_deref(), channel.as_deref(), count)
        .await;

    let recent = state.event_log.recent(count).await;
    for item in recent {
        if let Some(entry) = event_to_monitoring_entry(&item.event) {
            entries.push(entry);
        }
    }

    entries.retain(|entry| monitoring_matches_query(entry, &query));
    entries.sort_by(|left, right| right.timestamp.cmp(&left.timestamp));

    let mut seen = std::collections::HashSet::new();
    entries.retain(|entry| seen.insert(entry.id.clone()));
    entries.truncate(count);

    Ok(entries)
}

#[tauri::command]
pub async fn append_frontend_logs(
    app_handle: tauri::AppHandle,
    entries: Vec<FrontendLogInput>,
) -> Result<(), String> {
    for input in entries {
        let entry = frontend_log_entry(input);
        let persist = should_persist(entry.level, &entry.event_type);
        record_monitoring_entry(&app_handle, entry, persist).await;
    }

    Ok(())
}
#[tauri::command]
pub async fn get_recent_events(
    state: State<'_, AppState>,
    count: Option<usize>,
) -> Result<Vec<crate::events::EventLogEntry>, String> {
    let count = count.unwrap_or(50);
    Ok(state.event_log.recent(count).await)
}

#[tauri::command]
pub async fn get_runtime_metrics(
    state: State<'_, AppState>,
) -> Result<RuntimeMetricsResponse, String> {
    let snapshot = state.runtime_metrics_snapshot().await;

    Ok(RuntimeMetricsResponse {
        cache_entries: snapshot.cache_entries,
        event_log_entries: snapshot.event_log_entries,
        event_log_memory_bytes: snapshot.event_log_memory_bytes,
        ssh_state_count: snapshot.ssh_state_count,
        ssh_password_cache_count: snapshot.ssh_password_cache_count,
        process_rss_bytes: current_process_rss_bytes(),
        command_p95_ms: snapshot.command_p95_ms,
        db_query_p95_ms: snapshot.db_query_p95_ms,
    })
}

fn current_process_rss_bytes() -> u64 {
    use sysinfo::{Pid, ProcessesToUpdate, System};

    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);
    let pid = Pid::from_u32(std::process::id());
    system.process(pid).map(|p| p.memory()).unwrap_or(0)
}

async fn run_command_with_timeout(
    program: &str,
    args: &[&str],
    timeout_secs: u64,
) -> Result<std::process::Output, String> {
    let mut cmd = tokio_command(program);
    cmd.args(args);
    match timeout(Duration::from_secs(timeout_secs), cmd.output()).await {
        Ok(result) => result.map_err(|e| format!("Failed to spawn command: {e}")),
        Err(_) => Err(format!("Command timeout after {timeout_secs}s")),
    }
}

fn extract_version_line(output: &std::process::Output) -> Option<String> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let line = stdout
        .lines()
        .find(|line| !line.trim().is_empty())
        .or_else(|| stderr.lines().find(|line| !line.trim().is_empty()))?;

    Some(line.trim().to_string())
}

async fn probe_cli_version(tool: &'static str, timeout_ms: u64) -> CliVersionEntry {
    let started_at = Instant::now();
    let mut cmd = tokio_command(tool);
    cmd.args(["--version"]);

    let result = timeout(Duration::from_millis(timeout_ms), cmd.output()).await;
    let elapsed_ms = started_at.elapsed().as_millis() as u64;

    match result {
        Err(_) => CliVersionEntry {
            platform: tool.to_string(),
            installed: false,
            version: None,
            status: "timeout".to_string(),
            elapsed_ms,
        },
        Ok(Err(err)) if err.kind() == ErrorKind::NotFound => CliVersionEntry {
            platform: tool.to_string(),
            installed: false,
            version: None,
            status: "not_installed".to_string(),
            elapsed_ms,
        },
        Ok(Err(_)) => CliVersionEntry {
            platform: tool.to_string(),
            installed: false,
            version: None,
            status: "error".to_string(),
            elapsed_ms,
        },
        Ok(Ok(output)) => {
            let version = extract_version_line(&output);

            if version.is_some() {
                CliVersionEntry {
                    platform: tool.to_string(),
                    installed: true,
                    version,
                    status: "ok".to_string(),
                    elapsed_ms,
                }
            } else {
                CliVersionEntry {
                    platform: tool.to_string(),
                    installed: false,
                    version: None,
                    status: "error".to_string(),
                    elapsed_ms,
                }
            }
        }
    }
}

fn normalize_cli_tool(tool: &str) -> Option<&'static str> {
    match tool.trim().to_ascii_lowercase().as_str() {
        "ccr" => Some("ccr"),
        "claude" => Some("claude"),
        "codex" => Some("codex"),
        "gemini" => Some("gemini"),
        _ => None,
    }
}

async fn get_cached_cli_version(
    state: &AppState,
    tool: &'static str,
    timeout_ms: u64,
    force: bool,
) -> Result<CliVersionEntry, String> {
    let cache_key = format!("system:cli_version:{tool}:{timeout_ms}");

    if !force {
        if let Some(cached) = state.cache_get(&cache_key).await {
            let entry: CliVersionEntry = serde_json::from_value(cached)
                .map_err(|e| format!("CLI version cache decode failed: {e}"))?;
            return Ok(entry);
        }

        match state.begin_cache_fill(&cache_key).await {
            CacheFillRegistration::Wait(notify) => {
                notify.notified().await;
                if let Some(cached) = state.cache_get(&cache_key).await {
                    let entry: CliVersionEntry = serde_json::from_value(cached)
                        .map_err(|e| format!("CLI version cache decode failed: {e}"))?;
                    return Ok(entry);
                }
            }
            CacheFillRegistration::Leader => {
                let entry = probe_cli_version(tool, timeout_ms).await;
                let cached_entry = match serde_json::to_value(&entry) {
                    Ok(value) => value,
                    Err(error) => {
                        state.finish_cache_fill(&cache_key).await;
                        return Err(format!("CLI version cache encode failed: {error}"));
                    }
                };
                state.cache_set(cache_key.clone(), cached_entry, 60).await;
                state.finish_cache_fill(&cache_key).await;
                return Ok(entry);
            }
        }
    }

    Ok(probe_cli_version(tool, timeout_ms).await)
}

fn legacy_versions_map(entries: &[CliVersionEntry]) -> serde_json::Map<String, serde_json::Value> {
    let mut versions = serde_json::Map::new();
    for entry in entries {
        let legacy_value = if entry.installed {
            entry
                .version
                .clone()
                .unwrap_or_else(|| "installed".to_string())
        } else {
            "not found".to_string()
        };
        versions.insert(
            entry.platform.clone(),
            serde_json::Value::String(legacy_value),
        );
    }
    versions
}

fn cli_versions_payload(
    entries: Vec<CliVersionEntry>,
    mode: CliProbeMode,
    timeout_ms: u64,
    parallelism: usize,
) -> serde_json::Value {
    serde_json::json!({
        "versions": legacy_versions_map(&entries),
        "entries": entries,
        "mode": mode.as_str(),
        "timeout_ms": timeout_ms,
        "parallelism": parallelism,
    })
}

async fn compute_cli_versions(
    timeout_ms: u64,
    parallelism: usize,
) -> Result<Vec<CliVersionEntry>, String> {
    let tools = ["ccr", "claude", "codex", "gemini"];
    let effective_parallelism = parallelism.max(1).min(tools.len());
    let semaphore = Arc::new(Semaphore::new(effective_parallelism));
    let mut handles = Vec::with_capacity(tools.len());

    for tool in tools {
        let permit_pool = Arc::clone(&semaphore);
        handles.push(tokio::spawn(async move {
            let _permit = permit_pool
                .acquire_owned()
                .await
                .map_err(|e| format!("Semaphore acquire failed: {e}"))?;
            Ok::<CliVersionEntry, String>(probe_cli_version(tool, timeout_ms).await)
        }));
    }

    let mut entries = Vec::new();
    for handle in handles {
        let entry = handle
            .await
            .map_err(|e| format!("CLI version task failed: {e}"))??;
        entries.push(entry);
    }
    entries.sort_by(|a, b| a.platform.cmp(&b.platform));
    Ok(entries)
}

#[tauri::command]
pub async fn update_ccr() -> Result<serde_json::Value, String> {
    let output = run_command_with_timeout("cargo", &["install", "ccr"], 60).await?;

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(serde_json::json!({
        "success": success,
        "stdout": stdout,
        "stderr": stderr,
    }))
}

#[tauri::command]
pub async fn get_cli_versions(
    state: State<'_, AppState>,
    options: Option<CliVersionsOptions>,
) -> Result<serde_json::Value, String> {
    let options = options.unwrap_or(CliVersionsOptions {
        mode: None,
        timeout_ms: None,
        parallelism: None,
    });
    let mode = CliProbeMode::from_options(&options);
    let timeout_ms = options.timeout_ms.unwrap_or(mode.default_timeout_ms());
    let parallelism = options
        .parallelism
        .unwrap_or(mode.default_parallelism())
        .max(1);

    let should_cache = matches!(mode, CliProbeMode::Fast);
    let cache_key = "system:cli_versions:fast";

    if should_cache {
        if let Some(cached) = state.cache_get(cache_key).await {
            let entries: Vec<CliVersionEntry> = serde_json::from_value(cached)
                .map_err(|e| format!("CLI version cache decode failed: {e}"))?;
            return Ok(cli_versions_payload(entries, mode, timeout_ms, parallelism));
        }

        match state.begin_cache_fill(cache_key).await {
            CacheFillRegistration::Wait(notify) => {
                notify.notified().await;
                if let Some(cached) = state.cache_get(cache_key).await {
                    let entries: Vec<CliVersionEntry> = serde_json::from_value(cached)
                        .map_err(|e| format!("CLI version cache decode failed: {e}"))?;
                    return Ok(cli_versions_payload(entries, mode, timeout_ms, parallelism));
                }
            }
            CacheFillRegistration::Leader => {
                let result = compute_cli_versions(timeout_ms, parallelism).await;
                let entries = match result {
                    Ok(entries) => entries,
                    Err(error) => {
                        state.finish_cache_fill(cache_key).await;
                        return Err(error);
                    }
                };
                let cached_entries = match serde_json::to_value(&entries) {
                    Ok(value) => value,
                    Err(error) => {
                        state.finish_cache_fill(cache_key).await;
                        return Err(format!("CLI version cache encode failed: {error}"));
                    }
                };
                state
                    .cache_set(cache_key.to_string(), cached_entries, 60)
                    .await;
                state.finish_cache_fill(cache_key).await;
                return Ok(cli_versions_payload(entries, mode, timeout_ms, parallelism));
            }
        }
    }

    let entries = compute_cli_versions(timeout_ms, parallelism).await?;
    Ok(cli_versions_payload(entries, mode, timeout_ms, parallelism))
}

#[tauri::command]
pub async fn get_cli_version(
    state: State<'_, AppState>,
    options: CliVersionOptions,
) -> Result<CliVersionEntry, String> {
    let tool = normalize_cli_tool(&options.tool)
        .ok_or_else(|| format!("Unsupported CLI tool: {}", options.tool))?;
    let timeout_ms = options.timeout_ms.unwrap_or(1_500);
    let force = options.force.unwrap_or(false);

    get_cached_cli_version(&state, tool, timeout_ms, force).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn cli_versions_fast_mode_returns_expected_shape() {
        let started_at = Instant::now();
        let mode = CliProbeMode::Fast;
        let timeout_ms = 3_500;
        let parallelism = 4;
        let entries = compute_cli_versions(timeout_ms, parallelism)
            .await
            .expect("compute_cli_versions should succeed");
        let payload = cli_versions_payload(entries, mode, timeout_ms, parallelism);

        assert_eq!(payload.get("mode").and_then(|v| v.as_str()), Some("fast"));
        assert_eq!(
            payload.get("timeout_ms").and_then(|v| v.as_u64()),
            Some(3_500)
        );
        assert_eq!(
            payload
                .get("versions")
                .and_then(|v| v.as_object())
                .map(|m| m.len()),
            Some(4)
        );
        assert_eq!(
            payload
                .get("entries")
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(4)
        );

        // fast 模式下应在合理时间内返回，避免回归导致探测超时
        assert!(started_at.elapsed() <= Duration::from_millis(5_000));
    }

    #[test]
    fn normalize_cli_tool_rejects_unknown_tool() {
        assert_eq!(normalize_cli_tool("codex"), Some("codex"));
        assert_eq!(normalize_cli_tool(" CODEX "), Some("codex"));
        assert_eq!(normalize_cli_tool("unknown"), None);
    }

    #[tokio::test]
    async fn single_cli_probe_returns_requested_platform() {
        let entry = probe_cli_version("codex", 3_500).await;

        assert_eq!(entry.platform, "codex");
        assert!(matches!(
            entry.status.as_str(),
            "ok" | "not_installed" | "timeout" | "error"
        ));
    }

    #[tokio::test]
    async fn get_system_info_exposes_home_dashboard_metrics() {
        let info = get_system_info()
            .await
            .expect("get_system_info should succeed");
        let payload = serde_json::to_value(&info).expect("system info should serialize");

        assert!(payload.get("hostname").is_some());
        assert!(payload.get("os").is_some());
        assert!(payload.get("cpu_usage").is_some());
        assert!(payload.get("memory_usage_percent").is_some());
        assert!(payload.get("cpu_cores").is_some());
        assert!(payload.get("total_memory_gb").is_some());
        assert!(payload.get("used_memory_gb").is_some());
        assert!(payload.get("uptime_seconds").is_some());
    }
}

#[cfg(test)]
mod monitoring_query_tests {
    use super::*;
    use ccr_types::MonitoringLevel;

    #[test]
    fn monitoring_matches_query_accepts_empty_query() {
        let entry = MonitoringEntry::new(
            MonitoringLevel::Info,
            "system",
            "runtime.started",
            "desktop",
            "started",
        );

        assert!(monitoring_matches_query(
            &entry,
            &MonitoringFeedQuery::default()
        ));
    }

    #[test]
    fn monitoring_matches_query_filters_level_and_channel() {
        let entry = MonitoringEntry::new(
            MonitoringLevel::Error,
            "frontend",
            "frontend.error",
            "frontend",
            "boom",
        );

        assert!(monitoring_matches_query(
            &entry,
            &MonitoringFeedQuery {
                count: None,
                level: Some(MonitoringLevel::Error),
                channel: Some("frontend".to_string()),
            }
        ));
        assert!(!monitoring_matches_query(
            &entry,
            &MonitoringFeedQuery {
                count: None,
                level: Some(MonitoringLevel::Warn),
                channel: Some("frontend".to_string()),
            }
        ));
        assert!(!monitoring_matches_query(
            &entry,
            &MonitoringFeedQuery {
                count: None,
                level: Some(MonitoringLevel::Error),
                channel: Some("usage".to_string()),
            }
        ));
    }
}
