//! SSH 执行环境 — 通过系统 OpenSSH 访问远程主机。
//!
//! 说明：
//! - 依赖本机可用的 `ssh` CLI（OpenSSH）
//! - 配置读取/写入只走 SFTP，不构造远端 shell 文件命令

use super::config_path::normalize_config_relative_path;
use super::{CliStatus, EnvError, EnvironmentType, ExecutionEnvironment, PlatformInfo};
use crate::ssh::security::{
    RemotePosixPath, SftpWritePlan, SshTarget, app_known_hosts_path, openssh_error,
    posix_single_quote, run_openssh_command, sftp_read_batch, sftp_write_plan,
};

/// SSH 主机配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SshHostConfig {
    /// 逻辑 ID（可选，未提供时回退 host）
    pub id: Option<String>,
    /// 展示名称（可选）
    pub name: Option<String>,
    /// 主机地址（IP / 域名）
    pub host: String,
    /// 端口（默认 22）
    pub port: Option<u16>,
    /// 用户名（可选，默认当前用户）
    pub user: Option<String>,
    /// 私钥路径（可选）
    pub identity_file: Option<String>,
    /// 远程 home 目录（可选，默认 ~）
    pub remote_home: Option<String>,
}

impl SshHostConfig {
    pub fn validate(&self) -> Result<(), String> {
        self.target()?;
        self.remote_root()?;
        Ok(())
    }

    pub(crate) fn target(&self) -> Result<SshTarget, String> {
        SshTarget::new(
            &self.host,
            self.port.unwrap_or(22),
            self.user.as_deref(),
            self.identity_file.as_deref(),
        )
    }

    fn remote_root(&self) -> Result<RemotePosixPath, String> {
        RemotePosixPath::root(self.remote_home.as_deref())
    }
}

pub struct SshEnvironment {
    config: SshHostConfig,
}

impl SshEnvironment {
    pub fn new(config: SshHostConfig) -> Self {
        Self { config }
    }

    fn env_key(&self) -> String {
        self.config
            .id
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| self.config.host.clone())
    }

    fn display_label(&self) -> String {
        if let Some(name) = &self.config.name
            && !name.trim().is_empty()
        {
            return name.clone();
        }
        format!("{}:{}", self.config.host, self.config.port.unwrap_or(22))
    }

    fn platform_config_dir(&self, platform: &str) -> Result<RemotePosixPath, EnvError> {
        let relative = match platform {
            "claude" => ".claude",
            "codex" => ".codex",
            "gemini" => ".gemini/antigravity-cli",
            "opencode" => ".opencode",
            _ => return Err(EnvError::PlatformNotSupported(platform.to_string())),
        };
        self.config
            .remote_root()
            .and_then(|root| root.join_relative(relative))
            .map_err(EnvError::Other)
    }

    fn config_path(&self, platform: &str, path: &str) -> Result<RemotePosixPath, EnvError> {
        let base_dir = self.platform_config_dir(platform)?;
        let safe_relative_path = normalize_config_relative_path(path)?;
        base_dir
            .join_relative(&safe_relative_path)
            .map_err(EnvError::Other)
    }

    async fn run_ssh(&self, remote_cmd: &str) -> Result<std::process::Output, EnvError> {
        let target = self.config.target().map_err(EnvError::Other)?;
        let known_hosts = app_known_hosts_path().map_err(EnvError::Other)?;
        let mut command = target.ssh_command(&known_hosts, 5);
        command.arg(remote_cmd);
        run_openssh_command(command, None)
            .await
            .map_err(EnvError::ConnectionFailed)
    }

    async fn run_sftp(&self, batch: &str) -> Result<std::process::Output, EnvError> {
        let target = self.config.target().map_err(EnvError::Other)?;
        let known_hosts = app_known_hosts_path().map_err(EnvError::Other)?;
        run_openssh_command(target.sftp_command(&known_hosts), Some(batch.as_bytes()))
            .await
            .map_err(EnvError::ConnectionFailed)
    }
}

#[derive(Debug)]
struct SftpBatchOutput {
    success: bool,
    stderr: String,
}

#[async_trait::async_trait]
trait SftpBatchRunner {
    async fn run_batch(&self, batch: &str) -> Result<SftpBatchOutput, EnvError>;
}

#[async_trait::async_trait]
impl SftpBatchRunner for SshEnvironment {
    async fn run_batch(&self, batch: &str) -> Result<SftpBatchOutput, EnvError> {
        let output = self.run_sftp(batch).await?;
        Ok(SftpBatchOutput {
            success: output.status.success(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

async fn execute_sftp_write<R: SftpBatchRunner + Sync>(
    runner: &R,
    plan: &SftpWritePlan,
) -> Result<(), EnvError> {
    for (stage, batch) in [
        ("upload", plan.upload_batch.as_str()),
        ("rename", plan.rename_batch.as_str()),
    ] {
        match runner.run_batch(batch).await {
            Ok(output) if output.success => {}
            Ok(output) => {
                let _ = runner.run_batch(&plan.cleanup_batch).await;
                return Err(EnvError::Other(format!(
                    "远程写入 {stage} 失败: {}",
                    openssh_error(&output.stderr)
                )));
            }
            Err(error) => {
                let _ = runner.run_batch(&plan.cleanup_batch).await;
                return Err(error);
            }
        }
    }
    Ok(())
}

#[async_trait::async_trait]
impl ExecutionEnvironment for SshEnvironment {
    fn env_type(&self) -> EnvironmentType {
        EnvironmentType::Ssh
    }

    fn display_name(&self) -> String {
        format!("SSH: {}", self.display_label())
    }

    fn env_id(&self) -> String {
        format!("ssh:{}", self.env_key())
    }

    async fn list_platforms(&self) -> Result<Vec<PlatformInfo>, EnvError> {
        let statuses = self.detect_cli_status().await?;
        Ok(statuses
            .into_iter()
            .map(|s| PlatformInfo {
                name: s.name.clone(),
                display_name: match s.name.as_str() {
                    "claude" => "Claude Code",
                    "codex" => "Codex CLI",
                    "gemini" => "Antigravity CLI",
                    "opencode" => "OpenCode",
                    _ => s.name.as_str(),
                }
                .to_string(),
                installed: s.installed,
                version: s.version,
            })
            .collect())
    }

    async fn read_config(&self, platform: &str, path: &str) -> Result<String, EnvError> {
        let remote_path = self.config_path(platform, path)?;
        let temp_dir = tempfile::tempdir()
            .map_err(|error| EnvError::Other(format!("创建 SFTP 临时目录失败: {error}")))?;
        let local_path = temp_dir.path().join("download");
        let output = self
            .run_sftp(&sftp_read_batch(&remote_path, &local_path))
            .await?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(EnvError::ConfigNotFound(format!(
                "{}: {}",
                remote_path.as_str(),
                openssh_error(&err)
            )));
        }

        tokio::fs::read_to_string(&local_path)
            .await
            .map_err(|error| EnvError::ConfigNotFound(format!("{}: {error}", remote_path.as_str())))
    }

    async fn write_config(
        &self,
        platform: &str,
        path: &str,
        content: &str,
    ) -> Result<(), EnvError> {
        let remote_path = self.config_path(platform, path)?;
        let temp_dir = tempfile::tempdir()
            .map_err(|error| EnvError::Other(format!("创建 SFTP 临时目录失败: {error}")))?;
        let local_path = temp_dir.path().join("upload");
        tokio::fs::write(&local_path, content)
            .await
            .map_err(|error| EnvError::Other(format!("写入 SFTP 临时文件失败: {error}")))?;

        let plan = sftp_write_plan(&remote_path, &local_path, uuid::Uuid::new_v4())
            .map_err(EnvError::Other)?;
        execute_sftp_write(self, &plan).await.map_err(|error| {
            EnvError::Other(format!("远程写入失败 ({}): {error}", remote_path.as_str()))
        })
    }

    async fn detect_cli_status(&self) -> Result<Vec<CliStatus>, EnvError> {
        let tools = [
            ("claude", "claude"),
            ("codex", "codex"),
            ("gemini", "agy"),
            ("opencode", "opencode"),
        ];
        let mut result = Vec::new();

        for (platform, command) in tools {
            let quoted_command = posix_single_quote(command);
            let output = self
                .run_ssh(&format!("command -v {quoted_command} || true"))
                .await?;

            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            result.push(CliStatus {
                name: platform.to_string(),
                installed: !path.is_empty(),
                path: if path.is_empty() { None } else { Some(path) },
                version: None,
            });
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Mutex;

    use super::*;

    struct FakeOpenSshRunner {
        responses: Mutex<VecDeque<Result<SftpBatchOutput, EnvError>>>,
        batches: Mutex<Vec<String>>,
    }

    impl FakeOpenSshRunner {
        fn new(responses: Vec<Result<SftpBatchOutput, EnvError>>) -> Self {
            Self {
                responses: Mutex::new(responses.into()),
                batches: Mutex::new(Vec::new()),
            }
        }

        fn batches(&self) -> Vec<String> {
            self.batches.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl SftpBatchRunner for FakeOpenSshRunner {
        async fn run_batch(&self, batch: &str) -> Result<SftpBatchOutput, EnvError> {
            self.batches.lock().unwrap().push(batch.to_string());
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .expect("fake OpenSSH response should exist")
        }
    }

    fn success() -> Result<SftpBatchOutput, EnvError> {
        Ok(SftpBatchOutput {
            success: true,
            stderr: String::new(),
        })
    }

    fn failure(detail: &str) -> Result<SftpBatchOutput, EnvError> {
        Ok(SftpBatchOutput {
            success: false,
            stderr: detail.to_string(),
        })
    }

    fn write_plan() -> SftpWritePlan {
        let remote = RemotePosixPath::root(Some("/home/user"))
            .unwrap()
            .join_relative(".claude/settings.json")
            .unwrap();
        sftp_write_plan(
            &remote,
            std::path::Path::new("C:/Temp/config"),
            uuid::Uuid::nil(),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn upload_failure_runs_cleanup_without_rename() {
        let plan = write_plan();
        let runner = FakeOpenSshRunner::new(vec![failure("upload failed"), success()]);

        let error = execute_sftp_write(&runner, &plan).await.unwrap_err();

        assert!(error.to_string().contains("upload"));
        assert_eq!(
            runner.batches(),
            vec![plan.upload_batch, plan.cleanup_batch]
        );
    }

    #[tokio::test]
    async fn rename_failure_runs_cleanup_after_upload() {
        let plan = write_plan();
        let runner = FakeOpenSshRunner::new(vec![success(), failure("rename failed"), success()]);

        let error = execute_sftp_write(&runner, &plan).await.unwrap_err();

        assert!(error.to_string().contains("rename"));
        assert_eq!(
            runner.batches(),
            vec![plan.upload_batch, plan.rename_batch, plan.cleanup_batch]
        );
    }
}
