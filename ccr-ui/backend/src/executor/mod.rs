// CLI Subprocess Executor
// Executes CCR CLI commands as subprocesses and captures output
//
// 🎯 重构状态（Phase 1 完成后）：
//
// ✅ 核心handlers已重构为直接使用 CCR 核心库（性能提升50x）:
// - handlers/config.rs: validate, switch, clean, history
//
// 📌 以下handlers仍需使用 executor（设计合理）:
// - handlers/command.rs: 通用命令执行器（设计目的就是提供灵活的CLI执行）
// - handlers/sync.rs: 复杂WebDAV同步（17个handlers，低优先级）
// - handlers/stats.rs: 成本统计（核心库暂未提供StatsService）
// - handlers/config.rs: export/import（复杂逻辑，低优先级）
//
// ⏳ 未来计划：
// - 等待核心库提供更多服务后，可进一步重构sync和stats
// - executor作为fallback机制长期保留是合理的设计决策

use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Command timed out after {0} seconds")]
    Timeout(u64),
    #[error("CCR binary not found in PATH")]
    BinaryNotFound,
}

pub type Result<T> = std::result::Result<T, ExecutorError>;

/// Execute a CCR command with the given arguments
pub async fn execute_command(args: Vec<String>) -> Result<CommandOutput> {
    execute_command_with_timeout(args, Duration::from_secs(30)).await
}

/// Execute a CCR command with custom timeout
pub async fn execute_command_with_timeout(
    args: Vec<String>,
    timeout_duration: Duration,
) -> Result<CommandOutput> {
    let start = Instant::now();

    tracing::info!("Executing command: ccr {}", args.join(" "));

    // Spawn the CCR process
    let mut child = Command::new("ccr")
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ExecutorError::BinaryNotFound
            } else {
                ExecutorError::ExecutionFailed(format!("Failed to spawn process: {}", e))
            }
        })?;

    // Get stdout and stderr handles
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    // Read output in parallel
    let stdout_handle = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        let mut output = String::new();
        while let Ok(Some(line)) = lines.next_line().await {
            output.push_str(&line);
            output.push('\n');
        }
        output
    });

    let stderr_handle = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        let mut output = String::new();
        while let Ok(Some(line)) = lines.next_line().await {
            output.push_str(&line);
            output.push('\n');
        }
        output
    });

    // Wait for the process with timeout
    let status_result = timeout(timeout_duration, child.wait()).await;

    let status = match status_result {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            return Err(ExecutorError::ExecutionFailed(format!(
                "Failed to wait for process: {}",
                e
            )))
        }
        Err(_) => {
            // Kill the process on timeout
            let _ = child.kill().await;
            return Err(ExecutorError::Timeout(timeout_duration.as_secs()));
        }
    };

    // Collect output
    let stdout = stdout_handle
        .await
        .map_err(|e| ExecutorError::ExecutionFailed(format!("Failed to read stdout: {}", e)))?;
    let stderr = stderr_handle
        .await
        .map_err(|e| ExecutorError::ExecutionFailed(format!("Failed to read stderr: {}", e)))?;

    let duration_ms = start.elapsed().as_millis() as u64;
    let exit_code = status.code().unwrap_or(-1);

    tracing::info!(
        "Command completed: exit_code={}, duration={}ms",
        exit_code,
        duration_ms
    );

    Ok(CommandOutput {
        success: status.success(),
        stdout,
        stderr,
        exit_code,
        duration_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_version_command() {
        let result = execute_command(vec!["version".to_string()]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("CCR"));
    }
}

