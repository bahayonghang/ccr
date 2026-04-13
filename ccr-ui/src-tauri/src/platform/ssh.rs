//! SSH 执行环境 — 通过系统 `ssh` 命令访问远程主机。
//!
//! 说明：
//! - 依赖本机可用的 `ssh` CLI（OpenSSH）
//! - 当前阶段聚焦配置管理场景（读取/写入配置文件、检测 CLI）

use tokio::io::AsyncWriteExt;

use crate::process::tokio_command;

use super::config_path::normalize_config_relative_path;
use super::{CliStatus, EnvError, EnvironmentType, ExecutionEnvironment, PlatformInfo};

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
    /// 远程 home 目录（可选，默认 /home/<user>）
    pub remote_home: Option<String>,
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

    fn ssh_target(&self) -> String {
        match &self.config.user {
            Some(user) if !user.trim().is_empty() => format!("{user}@{}", self.config.host),
            _ => self.config.host.clone(),
        }
    }

    fn display_label(&self) -> String {
        if let Some(name) = &self.config.name
            && !name.trim().is_empty()
        {
            return name.clone();
        }
        format!("{}:{}", self.config.host, self.config.port.unwrap_or(22))
    }

    fn platform_config_dir(&self, platform: &str) -> Result<String, EnvError> {
        let default_home = self
            .config
            .user
            .as_ref()
            .map(|u| format!("/home/{u}"))
            .unwrap_or_else(|| "~".to_string());
        let home = self
            .config
            .remote_home
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(default_home);

        let dir = match platform {
            "claude" => format!("{home}/.claude"),
            "codex" => format!("{home}/.codex"),
            "gemini" => format!("{home}/.gemini"),
            "droid" => format!("{home}/.droid"),
            "opencode" => format!("{home}/.opencode"),
            _ => return Err(EnvError::PlatformNotSupported(platform.to_string())),
        };

        Ok(dir)
    }

    fn shell_escape_single(value: &str) -> String {
        value
            .replace('"', "\\\"")
            .replace('`', "\\`")
            .replace('$', "\\$")
    }

    async fn run_ssh(&self, remote_cmd: &str) -> Result<std::process::Output, EnvError> {
        let mut cmd = tokio_command("ssh");
        cmd.arg("-o").arg("BatchMode=yes");
        cmd.arg("-o").arg("ConnectTimeout=5");

        if let Some(identity) = &self.config.identity_file
            && !identity.trim().is_empty()
        {
            cmd.arg("-i").arg(identity);
        }

        cmd.arg("-p")
            .arg(self.config.port.unwrap_or(22).to_string())
            .arg(self.ssh_target())
            .arg(remote_cmd);

        cmd.output()
            .await
            .map_err(|e| EnvError::ConnectionFailed(format!("执行 ssh 命令失败: {e}")))
    }
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
                    "gemini" => "Gemini CLI",
                    "droid" => "Droid",
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
        let base_dir = self.platform_config_dir(platform)?;
        let safe_rel_path = normalize_config_relative_path(path)?;
        let remote_path = format!("{base_dir}/{safe_rel_path}");

        let escaped = Self::shell_escape_single(&remote_path);
        let output = self.run_ssh(&format!("cat \"{escaped}\"")).await?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(EnvError::ConfigNotFound(format!("{remote_path}: {err}")));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    async fn write_config(
        &self,
        platform: &str,
        path: &str,
        content: &str,
    ) -> Result<(), EnvError> {
        let base_dir = self.platform_config_dir(platform)?;
        let safe_rel_path = normalize_config_relative_path(path)?;
        let remote_path = format!("{base_dir}/{safe_rel_path}");

        let parent = std::path::Path::new(&remote_path)
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();

        let mut cmd = tokio_command("ssh");
        cmd.arg("-o").arg("BatchMode=yes");
        cmd.arg("-o").arg("ConnectTimeout=5");

        if let Some(identity) = &self.config.identity_file
            && !identity.trim().is_empty()
        {
            cmd.arg("-i").arg(identity);
        }

        let escaped_parent = Self::shell_escape_single(&parent);
        let escaped_path = Self::shell_escape_single(&remote_path);

        cmd.arg("-p")
            .arg(self.config.port.unwrap_or(22).to_string())
            .arg(self.ssh_target())
            .arg(format!(
                "mkdir -p \"{escaped_parent}\" && cat > \"{escaped_path}\""
            ))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| EnvError::ConnectionFailed(format!("启动 ssh 写入失败: {e}")))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(content.as_bytes())
                .await
                .map_err(|e| EnvError::Other(format!("写入远程 stdin 失败: {e}")))?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| EnvError::Other(format!("等待 ssh 写入结果失败: {e}")))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(EnvError::Other(format!(
                "远程写入失败 ({remote_path}): {err}"
            )));
        }

        Ok(())
    }

    async fn detect_cli_status(&self) -> Result<Vec<CliStatus>, EnvError> {
        let tools = ["claude", "codex", "gemini", "droid", "opencode"];
        let mut result = Vec::new();

        for tool in tools {
            let output = self.run_ssh(&format!("command -v {tool} || true")).await?;

            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            result.push(CliStatus {
                name: tool.to_string(),
                installed: !path.is_empty(),
                path: if path.is_empty() { None } else { Some(path) },
                version: None,
            });
        }

        Ok(result)
    }
}
