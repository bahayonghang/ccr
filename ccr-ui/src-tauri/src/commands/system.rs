//! 缂備緡鍨靛畷鐢靛垝濞差亜宸濋柟瀛樺笚婵?闂?缂備緡鍨靛畷鐢靛垝閻戞鈹嶉柍鈺佸暕缁辨牠鏌曢崱鏇″厡濠⒀勵殜瀵敻顢楁笟鍥ㄢ挄闂佸搫琚崕顖炲焵椤戣法顦﹀ù鐘茬摠閹棃鏌ㄧ€ｅ灚鈷曢梺鍝勮閸庮垶鍩€椤戞寧顦风紒銊ｅ妽缁傛帡鎳滈悽闈涘绩闁荤姴娴傚鍧楀焵?

use ccr_types::{FrontendLogInput, MonitoringEntry, MonitoringFeedQuery};

use serde::{Deserialize, Serialize};
use std::{io::ErrorKind, sync::Arc, time::Instant};
use tauri::State;
use tokio::sync::Semaphore;
use tokio::time::{Duration, timeout};

use crate::monitoring::{event_to_monitoring_entry, frontend_log_entry, record_monitoring_entry, should_persist};
use crate::process::tokio_command;
use crate::state::AppState;

/// 缂備緡鍨靛畷鐢靛垝閻戞鈹嶉柍鈺佸暕缁?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub arch: String,
    pub hostname: String,
    pub cpu_count: usize,
    pub total_memory_mb: u64,
    pub ccr_version: String,
}

/// 闂佺粯顨呴悧濠傦耿閺夎娑㈠焵椤掑嫬钃熼柕澹懏灏濋梺?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub current: String,
    pub latest: Option<String>,
    pub update_available: bool,
}

/// 闁哄鏅滈崝姗€銆侀幋锕€绫嶉柤鎼佹涧閻﹀綊鏌?
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
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_all();

    Ok(SystemInfo {
        os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
        os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
        arch: std::env::consts::ARCH.to_string(),
        hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
        cpu_count: sys.cpus().len(),
        total_memory_mb: sys.total_memory() / 1024 / 1024,
        ccr_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tauri::command]
pub async fn check_version() -> Result<VersionInfo, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    // TODO: Phase B4 闂?闂佸搫琚崕鎾敋?GitHub releases API
    Ok(VersionInfo {
        current,
        latest: None,
        update_available: false,
    })
}

#[tauri::command]
pub async fn health_check(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    // 濠碘槅鍋€閸嬫捇鏌＄仦璇插姕闁哄棛鍠栭獮鎴︻敊閼测晜鐨戦柡澶嗘櫇閸嬬偟鏁?
    let db_ok = state.db_pool.get().map(|_| true).unwrap_or(false);

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
    options: Option<CliVersionsOptions>,
) -> Result<serde_json::Value, String> {
    let options = options.unwrap_or(CliVersionsOptions {
        mode: None,
        timeout_ms: None,
        parallelism: None,
    });
    let mode = CliProbeMode::from_options(&options);
    let timeout_ms = options.timeout_ms.unwrap_or(mode.default_timeout_ms());
    let tools = ["ccr", "claude", "codex", "gemini"];
    let parallelism = options
        .parallelism
        .unwrap_or(mode.default_parallelism())
        .max(1)
        .min(tools.len());

    let semaphore = Arc::new(Semaphore::new(parallelism));
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

    let mut versions = serde_json::Map::new();
    for entry in &entries {
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

    Ok(serde_json::json!({
        "versions": versions,
        "entries": entries,
        "mode": mode.as_str(),
        "timeout_ms": timeout_ms,
        "parallelism": parallelism,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn cli_versions_fast_mode_returns_expected_shape() {
        let started_at = Instant::now();
        let payload = get_cli_versions(Some(CliVersionsOptions {
            mode: Some("fast".to_string()),
            timeout_ms: Some(3_500),
            parallelism: Some(4),
        }))
        .await
        .expect("get_cli_versions should succeed");

        assert_eq!(payload.get("mode").and_then(|v| v.as_str()), Some("fast"));
        assert_eq!(
            payload.get("timeout_ms").and_then(|v| v.as_u64()),
            Some(3_500)
        );
        assert_eq!(payload.get("versions").and_then(|v| v.as_object()).map(|m| m.len()), Some(4));
        assert_eq!(payload.get("entries").and_then(|v| v.as_array()).map(|a| a.len()), Some(4));

        // fast 濠碘槅鍨埀顒€纾涵鈧繛鎴炴尭椤戝懎鐣甸崱妯诲闁割煈鍠氶幗鐘绘煕閿斿搫濮€缂佸鎸冲顔炬媼閸︻厾顦繛鎴炴⒒閸犲秶绮径瀣劅闁哄啫娲ㄥ瓭闁哄鏅涘ú锝囩博鐎涙鈻旀い蹇撳鐠佹煡鏌熼幁鎺戝姎鐟滄澘鐗撻幆鍐礋椤愵偅鈷栭梻浣烘櫕閸犳牕危閸ヮ剙纾?
        assert!(started_at.elapsed() <= Duration::from_millis(5_000));
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

        assert!(monitoring_matches_query(&entry, &MonitoringFeedQuery::default()));
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
