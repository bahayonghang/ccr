// Version Management Handler
// Get version information by executing 'ccr --version' and check for updates from GitHub Cargo.toml

use axum::{extract::Query, response::IntoResponse};
use reqwest;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{
    LazyLock, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};
use tokio::task::JoinSet;

use crate::models::api::*;

// ===== CLI 版本检测缓存策略 =====

struct CachedCliVersions {
    data: CliVersionsResponse,
    cached_at: Instant,
    mode: CliVersionsMode,
}

static CLI_VERSIONS_CACHE: LazyLock<Mutex<Option<CachedCliVersions>>> =
    LazyLock::new(|| Mutex::new(None));
static CLI_VERSIONS_REFRESHING: AtomicBool = AtomicBool::new(false);

const CLI_VERSIONS_FRESH_TTL: Duration = Duration::from_secs(60);
const CLI_VERSIONS_STALE_TTL: Duration = Duration::from_secs(600);
const CLI_VERSION_FAST_TIMEOUT: Duration = Duration::from_millis(1200);
const CLI_VERSION_FULL_TIMEOUT: Duration = Duration::from_secs(8);
const CLI_VERSIONS_FAST_BUDGET: Duration = Duration::from_millis(2000);

const CLI_VERSION_PLATFORMS: [(&str, &str); 6] = [
    ("claude-code", "claude"),
    ("codex", "codex"),
    ("gemini-cli", "gemini"),
    ("qwen", "qwen"),
    ("iflow", "iflow"),
    ("droid", "droid"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CliVersionsMode {
    Fast,
    Full,
}

impl CliVersionsMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Full => "full",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CacheState {
    Fresh,
    Stale,
    Expired,
}

#[derive(Debug, Deserialize, Default)]
pub struct CliVersionsQuery {
    pub mode: Option<String>,
}

// GitHub raw Cargo.toml URL for CCR
const GITHUB_CARGO_TOML_URL: &str =
    "https://raw.githubusercontent.com/liyonghang/ccr/main/Cargo.toml";
const GITHUB_REPO_URL: &str = "https://github.com/liyonghang/ccr";

/// GET /api/version - Get current CCR version by executing 'ccr --version'
pub async fn get_version() -> impl IntoResponse {
    tracing::info!("Getting CCR version information");

    // 使用 spawn_blocking 避免阻塞异步执行器
    let result = tokio::task::spawn_blocking(get_ccr_version)
        .await
        .unwrap_or_else(|e| Err(format!("Task join error: {}", e)));

    match result {
        Ok(current_version) => {
            let version_info = VersionInfo {
                current_version,
                build_time: "N/A".to_string(),
                git_commit: "N/A".to_string(),
            };
            ApiResponse::success(version_info)
        }
        Err(e) => {
            tracing::error!("Failed to get CCR version: {}", e);
            ApiResponse::<VersionInfo>::error(format!("Failed to get CCR version: {}", e))
        }
    }
}

/// Execute 'ccr version' to get current version
fn get_ccr_version() -> Result<String, String> {
    // Try 'ccr version' command first (as seen in user terminal)
    let output = Command::new("ccr")
        .arg("version")
        .output()
        .map_err(|e| format!("Failed to execute ccr: {}", e))?;

    if !output.status.success() {
        return Err("CCR command failed".to_string());
    }

    let version_output = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to parse version output: {}", e))?;

    // Parse "Version: X.Y.Z" format from the banner
    for line in version_output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Version:") {
            // Extract "3.16.6" from "Version: 3.16.6"
            if let Some(v_str) = trimmed.split(':').nth(1) {
                return Ok(v_str.trim().to_string());
            }
        }
        // Also support "ccr X.Y.Z" simple format if it ever reverts
        if trimmed.starts_with("ccr ")
            && trimmed.chars().nth(4).is_some_and(|c| c.is_numeric())
            && let Some(v_str) = trimmed.split_whitespace().nth(1)
        {
            return Ok(v_str.to_string());
        }
    }

    // Fallback: try to find 'Version: ' anywhere
    if let Some(start) = version_output.find("Version: ") {
        let rest = &version_output[start + 9..];
        if let Some(end) = rest.find('\n') {
            return Ok(rest[..end].trim().to_string());
        }
    }

    Err("Failed to parse version from output".to_string())
}

/// GET /api/version/check - Check for updates from GitHub Cargo.toml
pub async fn check_update() -> impl IntoResponse {
    tracing::info!("Checking for updates from GitHub Cargo.toml");

    // 使用 spawn_blocking 获取当前版本（避免阻塞）
    let current_version_result = tokio::task::spawn_blocking(get_ccr_version)
        .await
        .unwrap_or_else(|e| Err(format!("Task join error: {}", e)));

    let current_version = match current_version_result {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to get current CCR version: {}", e);
            return ApiResponse::<UpdateCheckResponse>::error(format!(
                "Failed to get current CCR version: {}",
                e
            ));
        }
    };

    // Get latest version from GitHub
    match fetch_latest_version_from_github().await {
        Ok(latest_version) => {
            let has_update = compare_versions(&current_version, &latest_version);

            let response = UpdateCheckResponse {
                current_version: current_version.clone(),
                latest_version: latest_version.clone(),
                has_update,
                release_url: GITHUB_REPO_URL.to_string(),
                release_notes: None,
                published_at: None,
            };

            tracing::info!(
                "Update check completed: current={}, latest={}, has_update={}",
                current_version,
                latest_version,
                has_update
            );

            ApiResponse::success(response)
        }
        Err(e) => {
            tracing::error!("Failed to check for updates: {}", e);
            ApiResponse::<UpdateCheckResponse>::error(format!("Failed to check for updates: {}", e))
        }
    }
}

/// Fetch the latest version from GitHub Cargo.toml
async fn fetch_latest_version_from_github() -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().user_agent("ccr-ui").build()?;

    let response = client.get(GITHUB_CARGO_TOML_URL).send().await?;

    if !response.status().is_success() {
        return Err(format!("GitHub returned status: {}", response.status()).into());
    }

    let cargo_toml_content = response.text().await?;

    // Parse version from Cargo.toml
    for line in cargo_toml_content.lines() {
        let line = line.trim();
        if line.starts_with("version")
            && let Some(version) = line.split('=').nth(1)
        {
            // Parse: version = "1.2.3"
            let version = version
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string();
            return Ok(version);
        }
    }

    Err("Failed to parse version from Cargo.toml".into())
}

/// POST /api/version/update - Execute 'ccr update' command
pub async fn update_ccr() -> impl IntoResponse {
    tracing::info!("Executing CCR update command");

    // 使用 spawn_blocking 避免阻塞异步执行器
    let result = tokio::task::spawn_blocking(execute_ccr_update)
        .await
        .unwrap_or_else(|e| Err(format!("Task join error: {}", e)));

    match result {
        Ok(output) => {
            let response = UpdateExecutionResponse {
                success: output.status.success(),
                output: output.stdout,
                error: output.stderr,
                exit_code: output.status.code().unwrap_or(-1),
            };

            if response.success {
                tracing::info!("CCR update completed successfully");
            } else {
                tracing::warn!("CCR update failed with exit code: {}", response.exit_code);
            }

            ApiResponse::success(response)
        }
        Err(e) => {
            tracing::error!("Failed to execute CCR update: {}", e);
            ApiResponse::<UpdateExecutionResponse>::error(format!(
                "Failed to execute CCR update: {}",
                e
            ))
        }
    }
}

/// Execute 'ccr update' command and capture output
fn execute_ccr_update() -> Result<CommandOutput, String> {
    let mut child = Command::new("ccr")
        .arg("update")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn ccr update: {}", e))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to capture stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Failed to capture stderr".to_string())?;

    let mut stdout_lines = Vec::new();
    let mut stderr_lines = Vec::new();

    // Read stdout
    let stdout_reader = BufReader::new(stdout);
    for line in stdout_reader.lines().map_while(Result::ok) {
        tracing::debug!("CCR update stdout: {}", line);
        stdout_lines.push(line);
    }

    // Read stderr
    let stderr_reader = BufReader::new(stderr);
    for line in stderr_reader.lines().map_while(Result::ok) {
        tracing::debug!("CCR update stderr: {}", line);
        stderr_lines.push(line);
    }

    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for ccr update: {}", e))?;

    Ok(CommandOutput {
        status,
        stdout: stdout_lines.join("\n"),
        stderr: stderr_lines.join("\n"),
    })
}

struct CommandOutput {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}

/// 启动后预热 CLI 版本缓存（非阻塞）
pub fn trigger_cli_versions_prewarm() {
    spawn_cli_versions_cache_refresh("startup_prewarm");
}

/// GET /api/version/cli-versions - Detect installed CLI versions for all platforms
/// 支持 mode=fast|full（默认 fast）
pub async fn get_cli_versions(Query(query): Query<CliVersionsQuery>) -> impl IntoResponse {
    let mode = parse_cli_versions_mode(query.mode.as_deref());
    let request_started = Instant::now();

    // 缓存命中判定：Fresh 直接返回，Fast + Stale 走 SWR
    let cached = {
        let cache = CLI_VERSIONS_CACHE.lock().unwrap_or_else(|e| e.into_inner());
        cache
            .as_ref()
            .map(|entry| (entry.data.clone(), entry.cached_at.elapsed(), entry.mode))
    };

    if let Some((cached_data, age, cached_mode)) = cached {
        match cache_state_for_age(age) {
            CacheState::Fresh => {
                if mode == CliVersionsMode::Full && cached_mode != CliVersionsMode::Full {
                    tracing::info!(
                        mode = mode.as_str(),
                        age_ms = age.as_millis() as u64,
                        cached_mode = cached_mode.as_str(),
                        "cli versions fresh cache skipped for full mode"
                    );
                } else {
                    tracing::info!(
                        mode = mode.as_str(),
                        age_ms = age.as_millis() as u64,
                        cached_mode = cached_mode.as_str(),
                        "cli versions cache hit (fresh)"
                    );
                    return ApiResponse::success(cached_data);
                }
            }
            CacheState::Stale if mode == CliVersionsMode::Fast => {
                tracing::info!(
                    mode = mode.as_str(),
                    age_ms = age.as_millis() as u64,
                    cached_mode = cached_mode.as_str(),
                    "cli versions cache hit (stale), serving stale and refreshing in background"
                );
                spawn_cli_versions_cache_refresh("stale_cache");
                return ApiResponse::success(cached_data);
            }
            CacheState::Stale | CacheState::Expired => {
                tracing::info!(
                    mode = mode.as_str(),
                    age_ms = age.as_millis() as u64,
                    cached_mode = cached_mode.as_str(),
                    "cli versions cache stale/expired, recomputing"
                );
            }
        }
    } else {
        tracing::info!(mode = mode.as_str(), "cli versions cache miss");
    }

    let response = detect_cli_versions(mode).await;
    store_cli_versions_cache(response.clone(), mode);

    tracing::info!(
        mode = mode.as_str(),
        elapsed_ms = request_started.elapsed().as_millis() as u64,
        "cli versions request computed"
    );

    ApiResponse::success(response)
}

fn parse_cli_versions_mode(raw: Option<&str>) -> CliVersionsMode {
    match raw.map(|v| v.trim().to_ascii_lowercase()) {
        Some(mode) if mode == "full" => CliVersionsMode::Full,
        _ => CliVersionsMode::Fast,
    }
}

fn cache_state_for_age(age: Duration) -> CacheState {
    if age < CLI_VERSIONS_FRESH_TTL {
        CacheState::Fresh
    } else if age < CLI_VERSIONS_STALE_TTL {
        CacheState::Stale
    } else {
        CacheState::Expired
    }
}

fn store_cli_versions_cache(data: CliVersionsResponse, mode: CliVersionsMode) {
    let mut cache = CLI_VERSIONS_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    *cache = Some(CachedCliVersions {
        data,
        cached_at: Instant::now(),
        mode,
    });
}

fn spawn_cli_versions_cache_refresh(reason: &'static str) {
    if CLI_VERSIONS_REFRESHING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        tracing::debug!(
            reason = reason,
            "cli versions cache refresh already in-flight"
        );
        return;
    }

    tokio::spawn(async move {
        struct RefreshGuard;
        impl Drop for RefreshGuard {
            fn drop(&mut self) {
                CLI_VERSIONS_REFRESHING.store(false, Ordering::Release);
            }
        }

        let _guard = RefreshGuard;
        let started = Instant::now();
        let response = detect_cli_versions(CliVersionsMode::Fast).await;
        store_cli_versions_cache(response, CliVersionsMode::Fast);

        tracing::info!(
            reason = reason,
            elapsed_ms = started.elapsed().as_millis() as u64,
            "cli versions cache refresh completed"
        );
    });
}

async fn detect_cli_versions(mode: CliVersionsMode) -> CliVersionsResponse {
    let started = Instant::now();
    let versions = match mode {
        CliVersionsMode::Fast => detect_cli_versions_fast().await,
        CliVersionsMode::Full => detect_cli_versions_full().await,
    };

    tracing::info!(
        mode = mode.as_str(),
        elapsed_ms = started.elapsed().as_millis() as u64,
        "cli version detection batch finished"
    );

    CliVersionsResponse { versions }
}

async fn detect_cli_versions_fast() -> Vec<CliVersionEntry> {
    let mut set = JoinSet::new();
    for (platform, binary) in CLI_VERSION_PLATFORMS {
        set.spawn(detect_single_cli_version(
            platform.to_string(),
            binary.to_string(),
            CliVersionsMode::Fast,
        ));
    }

    let mut entries = Vec::with_capacity(CLI_VERSION_PLATFORMS.len());
    let deadline = tokio::time::Instant::now() + CLI_VERSIONS_FAST_BUDGET;

    while entries.len() < CLI_VERSION_PLATFORMS.len() {
        let now = tokio::time::Instant::now();
        if now >= deadline {
            break;
        }

        let remaining = deadline.saturating_duration_since(now);
        match tokio::time::timeout(remaining, set.join_next()).await {
            Ok(Some(Ok(entry))) => entries.push(entry),
            Ok(Some(Err(err))) => {
                tracing::warn!(error = %err, "cli version detection task failed");
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }

    if entries.len() < CLI_VERSION_PLATFORMS.len() {
        set.abort_all();
        let existing_platforms: HashSet<String> =
            entries.iter().map(|entry| entry.platform.clone()).collect();
        for (platform, _) in CLI_VERSION_PLATFORMS {
            if !existing_platforms.contains(platform) {
                entries.push(build_cli_version_entry(
                    platform.to_string(),
                    false,
                    None,
                    "timeout",
                    CLI_VERSIONS_FAST_BUDGET.as_millis() as u64,
                ));
            }
        }
    }

    sort_cli_entries(entries)
}

async fn detect_cli_versions_full() -> Vec<CliVersionEntry> {
    let mut set = JoinSet::new();
    for (platform, binary) in CLI_VERSION_PLATFORMS {
        set.spawn(detect_single_cli_version(
            platform.to_string(),
            binary.to_string(),
            CliVersionsMode::Full,
        ));
    }

    let mut entries = Vec::with_capacity(CLI_VERSION_PLATFORMS.len());
    while let Some(result) = set.join_next().await {
        match result {
            Ok(entry) => entries.push(entry),
            Err(err) => tracing::warn!(error = %err, "cli version detection task failed"),
        }
    }

    // full 模式也保证返回完整平台集合
    let existing_platforms: HashSet<String> =
        entries.iter().map(|entry| entry.platform.clone()).collect();
    for (platform, _) in CLI_VERSION_PLATFORMS {
        if !existing_platforms.contains(platform) {
            entries.push(build_cli_version_entry(
                platform.to_string(),
                false,
                None,
                "error",
                0,
            ));
        }
    }

    sort_cli_entries(entries)
}

fn sort_cli_entries(entries: Vec<CliVersionEntry>) -> Vec<CliVersionEntry> {
    let mut by_platform: HashMap<String, CliVersionEntry> = entries
        .into_iter()
        .map(|entry| (entry.platform.clone(), entry))
        .collect();

    let mut sorted = Vec::with_capacity(CLI_VERSION_PLATFORMS.len());
    for (platform, _) in CLI_VERSION_PLATFORMS {
        if let Some(entry) = by_platform.remove(platform) {
            sorted.push(entry);
        }
    }

    sorted
}

async fn detect_single_cli_version(
    platform: String,
    binary: String,
    mode: CliVersionsMode,
) -> CliVersionEntry {
    let started = Instant::now();

    let output = match mode {
        CliVersionsMode::Fast => {
            match tokio::time::timeout(CLI_VERSION_FAST_TIMEOUT, run_cli_version_command(&binary))
                .await
            {
                Ok(result) => result,
                Err(_) => {
                    let elapsed_ms = started.elapsed().as_millis() as u64;
                    let entry = build_cli_version_entry(
                        platform.clone(),
                        false,
                        None,
                        "timeout",
                        elapsed_ms,
                    );
                    log_cli_detection_entry(mode, &entry);
                    return entry;
                }
            }
        }
        CliVersionsMode::Full => {
            match tokio::time::timeout(CLI_VERSION_FULL_TIMEOUT, run_cli_version_command(&binary))
                .await
            {
                Ok(result) => result,
                Err(_) => {
                    let elapsed_ms = started.elapsed().as_millis() as u64;
                    let entry = build_cli_version_entry(
                        platform.clone(),
                        false,
                        None,
                        "timeout",
                        elapsed_ms,
                    );
                    log_cli_detection_entry(mode, &entry);
                    return entry;
                }
            }
        }
    };

    let elapsed_ms = started.elapsed().as_millis() as u64;
    let entry = match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let version = parse_version_output(&stdout).or_else(|| parse_version_output(&stderr));
            build_cli_version_entry(platform.clone(), true, version, "ok", elapsed_ms)
        }
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let missing = is_command_missing(output.status.code(), &stdout, &stderr);
            if missing {
                build_cli_version_entry(platform.clone(), false, None, "not_installed", elapsed_ms)
            } else {
                build_cli_version_entry(platform.clone(), false, None, "error", elapsed_ms)
            }
        }
        Err(err) => {
            if err.kind() == std::io::ErrorKind::NotFound {
                build_cli_version_entry(platform.clone(), false, None, "not_installed", elapsed_ms)
            } else {
                build_cli_version_entry(platform.clone(), false, None, "error", elapsed_ms)
            }
        }
    };

    log_cli_detection_entry(mode, &entry);
    entry
}

fn build_cli_version_entry(
    platform: String,
    installed: bool,
    version: Option<String>,
    status: &str,
    elapsed_ms: u64,
) -> CliVersionEntry {
    CliVersionEntry {
        platform,
        installed,
        version,
        status: Some(status.to_string()),
        elapsed_ms: Some(elapsed_ms),
    }
}

fn log_cli_detection_entry(mode: CliVersionsMode, entry: &CliVersionEntry) {
    tracing::info!(
        mode = mode.as_str(),
        platform = %entry.platform,
        status = entry.status.as_deref().unwrap_or("unknown"),
        elapsed_ms = entry.elapsed_ms.unwrap_or_default(),
        "cli version detection finished"
    );
}

async fn run_cli_version_command(binary: &str) -> std::io::Result<std::process::Output> {
    if cfg!(target_os = "windows") {
        tokio::process::Command::new("cmd")
            .args(["/c", binary, "--version"])
            .output()
            .await
    } else {
        tokio::process::Command::new(binary)
            .arg("--version")
            .output()
            .await
    }
}

fn is_command_missing(status_code: Option<i32>, stdout: &str, stderr: &str) -> bool {
    if status_code == Some(127) {
        return true;
    }

    let lower = format!(
        "{} {}",
        stdout.to_ascii_lowercase(),
        stderr.to_ascii_lowercase()
    );
    let patterns = [
        "not found",
        "not recognized",
        "is not recognized",
        "no such file or directory",
        "cannot find",
        "找不到",
        "不是内部或外部命令",
    ];

    patterns.iter().any(|pattern| lower.contains(pattern))
}

/// 从 CLI 输出中提取版本号
fn parse_version_output(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(v) = trimmed.strip_prefix("Version:") {
            let v = v.trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
        for word in trimmed.split_whitespace() {
            let clean = word.trim_start_matches('v');
            if clean.contains('.') && clean.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                return Some(clean.to_string());
            }
        }
    }
    None
}

/// Compare version strings and return true if latest > current
/// Simple comparison: split by '.' and compare each part numerically
fn compare_versions(current: &str, latest: &str) -> bool {
    let current_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();

    let latest_parts: Vec<u32> = latest.split('.').filter_map(|s| s.parse().ok()).collect();

    // Compare each part
    for i in 0..std::cmp::max(current_parts.len(), latest_parts.len()) {
        let current_part = current_parts.get(i).unwrap_or(&0);
        let latest_part = latest_parts.get(i).unwrap_or(&0);

        if latest_part > current_part {
            return true;
        } else if latest_part < current_part {
            return false;
        }
    }

    false // versions are equal
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        // Latest is greater
        assert!(compare_versions("1.0.0", "1.1.0"));
        assert!(compare_versions("1.0.0", "2.0.0"));
        assert!(compare_versions("1.2.3", "1.2.4"));

        // Current is greater or equal
        assert!(!compare_versions("1.1.0", "1.0.0"));
        assert!(!compare_versions("2.0.0", "1.0.0"));
        assert!(!compare_versions("1.2.4", "1.2.3"));
        assert!(!compare_versions("1.0.0", "1.0.0"));

        // Different lengths
        assert!(compare_versions("1.0", "1.0.1"));
        assert!(!compare_versions("1.0.1", "1.0"));
    }

    #[test]
    fn test_parse_cli_versions_mode_defaults_to_fast() {
        assert_eq!(parse_cli_versions_mode(None), CliVersionsMode::Fast);
        assert_eq!(parse_cli_versions_mode(Some("FAST")), CliVersionsMode::Fast);
        assert_eq!(
            parse_cli_versions_mode(Some("invalid")),
            CliVersionsMode::Fast
        );
        assert_eq!(parse_cli_versions_mode(Some("full")), CliVersionsMode::Full);
    }

    #[test]
    fn test_cache_state_for_age() {
        assert_eq!(
            cache_state_for_age(Duration::from_secs(10)),
            CacheState::Fresh
        );
        assert_eq!(
            cache_state_for_age(Duration::from_secs(120)),
            CacheState::Stale
        );
        assert_eq!(
            cache_state_for_age(Duration::from_secs(1200)),
            CacheState::Expired
        );
    }

    #[test]
    fn test_is_command_missing_patterns() {
        assert!(is_command_missing(
            Some(1),
            "",
            "'abc' is not recognized as an internal or external command"
        ));
        assert!(is_command_missing(Some(127), "", ""));
        assert!(!is_command_missing(Some(1), "", "permission denied"));
    }

    #[test]
    fn test_timeout_entry_has_timeout_status() {
        let entry = build_cli_version_entry("codex".to_string(), false, None, "timeout", 1200);
        assert_eq!(entry.platform, "codex");
        assert_eq!(entry.status.as_deref(), Some("timeout"));
        assert_eq!(entry.elapsed_ms, Some(1200));
    }
}
