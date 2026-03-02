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

use async_stream::stream;
use futures::stream::Stream;
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamChunk {
    Stdout { data: String },
    Stderr { data: String },
    Completion { exit_code: i32, duration_ms: u64 },
    Error { message: String },
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Command timed out after {0} seconds")]
    Timeout(u64),
    #[error("CCR binary not found in PATH")]
    BinaryNotFound,
    #[error("Stdio handle not available")]
    StdioHandleMissing,
}

pub type Result<T> = std::result::Result<T, ExecutorError>;

/// Execute a CCR command with the given arguments
pub async fn execute_command(args: Vec<String>) -> Result<CommandOutput> {
    execute_binary("ccr", args).await
}

/// Execute a binary with the given arguments
pub async fn execute_binary(binary: &str, args: Vec<String>) -> Result<CommandOutput> {
    execute_binary_with_timeout(binary, args, Duration::from_secs(30)).await
}

/// Execute a binary with custom timeout
pub async fn execute_binary_with_timeout(
    binary: &str,
    args: Vec<String>,
    timeout_duration: Duration,
) -> Result<CommandOutput> {
    let start = Instant::now();

    tracing::info!("Executing command: {} {}", binary, args.join(" "));

    // Spawn the process
    let mut child = Command::new(binary)
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
    let stdout = child
        .stdout
        .take()
        .ok_or(ExecutorError::StdioHandleMissing)?;
    let stderr = child
        .stderr
        .take()
        .ok_or(ExecutorError::StdioHandleMissing)?;

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
            )));
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
        success: exit_code == 0,
        stdout,
        stderr,
        exit_code,
        duration_ms,
    })
}

/// Execute a binary and stream output in real-time
pub fn execute_binary_stream(binary: String, args: Vec<String>) -> impl Stream<Item = StreamChunk> {
    stream! {
        let start = Instant::now();

        tracing::info!("Executing command (streaming): {} {}", binary, args.join(" "));

        // Spawn the process
        let mut child = match Command::new(&binary)
            .args(&args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                let message = if e.kind() == std::io::ErrorKind::NotFound {
                    format!("Binary '{}' not found in PATH", binary)
                } else {
                    format!("Failed to spawn process: {}", e)
                };
                yield StreamChunk::Error { message };
                return;
            }
        };

        // Get stdout and stderr handles
        let stdout = match child.stdout.take() {
            Some(handle) => handle,
            None => {
                yield StreamChunk::Error {
                    message: "Failed to capture stdout".to_string(),
                };
                return;
            }
        };
        let stderr = match child.stderr.take() {
            Some(handle) => handle,
            None => {
                yield StreamChunk::Error {
                    message: "Failed to capture stderr".to_string(),
                };
                return;
            }
        };

        // Create async readers
        let mut stdout_lines = BufReader::new(stdout).lines();
        let mut stderr_lines = BufReader::new(stderr).lines();

        // Track whether each stream is still open
        let mut stdout_open = true;
        let mut stderr_open = true;

        // Read and yield output line by line
        while stdout_open || stderr_open {
            tokio::select! {
                // Read from stdout (only if still open)
                result = stdout_lines.next_line(), if stdout_open => {
                    match result {
                        Ok(Some(line)) => {
                            yield StreamChunk::Stdout { data: line };
                        }
                        Ok(None) => {
                            // stdout closed, mark as closed but continue reading stderr
                            stdout_open = false;
                        }
                        Err(e) => {
                            yield StreamChunk::Error {
                                message: format!("Failed to read stdout: {}", e)
                            };
                            stdout_open = false;
                        }
                    }
                }
                // Read from stderr (only if still open)
                result = stderr_lines.next_line(), if stderr_open => {
                    match result {
                        Ok(Some(line)) => {
                            yield StreamChunk::Stderr { data: line };
                        }
                        Ok(None) => {
                            // stderr closed, mark as closed but continue reading stdout
                            stderr_open = false;
                        }
                        Err(e) => {
                            yield StreamChunk::Error {
                                message: format!("Failed to read stderr: {}", e)
                            };
                            stderr_open = false;
                        }
                    }
                }
            }
        }

        // Wait for process to complete
        match child.wait().await {
            Ok(status) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let exit_code = status.code().unwrap_or(-1);

                tracing::info!(
                    "Command completed (streaming): exit_code={}, duration={}ms",
                    exit_code,
                    duration_ms
                );

                yield StreamChunk::Completion {
                    exit_code,
                    duration_ms,
                };
            }
            Err(e) => {
                yield StreamChunk::Error {
                    message: format!("Failed to wait for process: {}", e)
                };
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// 测试执行version命令
    /// 注意：此测试需要ccr二进制在PATH中，CI环境可能不满足条件
    #[tokio::test]
    async fn test_execute_version_command() {
        // 先检查ccr是否可用
        let check_result = tokio::process::Command::new("ccr")
            .arg("--version")
            .output()
            .await;

        if check_result.is_err() {
            eprintln!("⚠️  Skipping test: ccr binary not found in PATH (expected in CI)");
            return;
        }

        let result = execute_command(vec!["version".to_string()]).await;
        assert!(result.is_ok(), "Command execution should succeed");

        let output = result.unwrap();
        assert_eq!(output.exit_code, 0, "Exit code should be 0");
        assert!(output.stdout.contains("CCR"), "Output should contain 'CCR'");
    }

    /// 测试BinaryNotFound错误
    #[tokio::test]
    async fn test_binary_not_found() {
        let result = execute_binary_with_timeout(
            "ccr",
            vec!["--version".to_string()],
            Duration::from_secs(5),
        )
        .await;

        // 如果ccr在PATH中，跳过此测试
        if result.is_ok() {
            eprintln!("ℹ️  Skipping test: ccr binary is available");
            return;
        }

        // 如果ccr不在PATH中，应该返回BinaryNotFound错误
        match result {
            Err(ExecutorError::BinaryNotFound) => {
                // 预期行为
            }
            _ => panic!("Expected BinaryNotFound error when ccr is not in PATH"),
        }
    }
}
