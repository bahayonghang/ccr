// 命令执行服务 - 支持并发控制和流式输出
// Reserved for future streaming command execution (see handlers/command.rs:168)
// Currently blocked by SSE stream lifecycle issues - implementation pending

use crate::core::error::{ApiError, ApiResult};
use futures::Stream;
use std::sync::Arc;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::Semaphore,
};
use tracing::{debug, error, info, instrument};

/// 命令执行服务
#[allow(dead_code)] // Reserved for streaming implementation
#[derive(Clone)]
pub struct CommandService {
    /// 并发信号量，限制同时执行的命令数
    semaphore: Arc<Semaphore>,
    /// 最大并发数
    max_concurrent: usize,
}

#[allow(dead_code)] // Reserved for streaming implementation
impl CommandService {
    /// 创建新的命令服务
    ///
    /// # Arguments
    /// * `max_concurrent` - 最大并发执行命令数，默认推荐 4
    pub fn new(max_concurrent: usize) -> Self {
        info!(
            "Initializing CommandService with max_concurrent: {}",
            max_concurrent
        );
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
        }
    }

    /// 执行命令（阻塞式，等待完成）
    ///
    /// # Arguments
    /// * `program` - 程序名称（如 "ccr"）
    /// * `args` - 参数列表
    #[instrument(skip(self), fields(program = %program, args = ?args))]
    pub async fn execute(&self, program: &str, args: &[String]) -> ApiResult<CommandOutput> {
        // 获取信号量许可
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            error!("Failed to acquire semaphore: {}", e);
            ApiError::service_unavailable("Too many concurrent commands")
        })?;

        debug!("Executing command: {} {:?}", program, args);

        let output = Command::new(program)
            .args(args)
            .output()
            .await
            .map_err(|e| {
                error!("Command execution failed: {}", e);
                ApiError::internal(format!("Failed to execute command: {}", e))
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();

        if success {
            info!("Command executed successfully");
        } else {
            error!("Command failed with exit code: {:?}", output.status.code());
        }

        Ok(CommandOutput {
            stdout,
            stderr,
            success,
            exit_code: output.status.code(),
        })
    }

    /// 执行命令并返回流式输出
    ///
    /// # Arguments
    /// * `program` - 程序名称
    /// * `args` - 参数列表
    ///
    /// # Returns
    /// 返回一个异步流，每次 yield 一行输出
    #[instrument(skip(self), fields(program = %program, args = ?args))]
    pub async fn execute_stream(
        &self,
        program: &str,
        args: &[String],
    ) -> ApiResult<impl Stream<Item = Result<String, std::io::Error>>> {
        // 获取信号量许可（需要保持到流结束）
        let permit = self.semaphore.clone().acquire_owned().await.map_err(|e| {
            error!("Failed to acquire semaphore for stream: {}", e);
            ApiError::service_unavailable("Too many concurrent commands")
        })?;

        debug!("Starting stream command: {} {:?}", program, args);

        let mut child = Command::new(program)
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                error!("Failed to spawn command: {}", e);
                ApiError::internal(format!("Failed to spawn command: {}", e))
            })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            error!("Failed to capture stdout");
            ApiError::internal("Failed to capture command output")
        })?;

        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        // 创建流
        let stream = async_stream::stream! {
            while let Ok(Some(line)) = lines.next_line().await {
                yield Ok(line);
            }

            // 等待进程结束
            if let Ok(status) = child.wait().await {
                debug!("Stream command finished with status: {:?}", status);
                if !status.success() {
                    yield Err(std::io::Error::other(
                        format!("Command failed with exit code: {:?}", status.code())
                    ));
                }
            }

            // 释放许可（permit 在这里 drop）
            drop(permit);
            info!("Stream command completed and permit released");
        };

        Ok(stream)
    }

    /// 获取当前并发数信息
    pub fn concurrency_info(&self) -> ConcurrencyInfo {
        let available = self.semaphore.available_permits();
        ConcurrencyInfo {
            max_concurrent: self.max_concurrent,
            available_slots: available,
            active_commands: self.max_concurrent - available,
        }
    }
}

/// 命令执行输出
#[allow(dead_code)] // Reserved for streaming implementation
#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub exit_code: Option<i32>,
}

/// 并发信息
#[allow(dead_code)] // Reserved for streaming implementation
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConcurrencyInfo {
    pub max_concurrent: usize,
    pub available_slots: usize,
    pub active_commands: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_command_service_creation() {
        let service = CommandService::new(4);
        let info = service.concurrency_info();
        assert_eq!(info.max_concurrent, 4);
        assert_eq!(info.available_slots, 4);
        assert_eq!(info.active_commands, 0);
    }

    #[tokio::test]
    async fn test_simple_command() {
        let service = CommandService::new(2);
        let result = service.execute("echo", &["test".to_string()]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
        assert!(output.stdout.contains("test"));
    }
}
