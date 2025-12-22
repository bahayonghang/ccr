// Version Management Handler
// Get version information by executing 'ccr --version' and check for updates from GitHub Cargo.toml

use axum::response::IntoResponse;
use reqwest;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use crate::models::api::*;

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

/// Execute 'ccr --version' to get current version
fn get_ccr_version() -> Result<String, String> {
    let output = Command::new("ccr")
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to execute ccr: {}", e))?;

    if !output.status.success() {
        return Err("CCR command failed".to_string());
    }

    let version_output = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to parse version output: {}", e))?;
    // Parse "ccr X.Y.Z" format
    let version = version_output
        .split_whitespace()
        .nth(1)
        .ok_or("Failed to parse version")?
        .to_string();

    Ok(version)
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
}
