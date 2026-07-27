//! 本地执行环境 — 直接委托到 `ccr` 核心库。

use super::config_path::normalize_config_relative_path;
use super::{CliStatus, EnvError, EnvironmentType, ExecutionEnvironment, PlatformInfo};
use crate::process::{ProcessDescriptor, ProcessGateway};
use ccr_core::core::{BackupPolicy, WriteOptions, write_guarded_async};

/// 本地环境实现 — 始终可用，委托到 ccr 核心库。
pub struct LocalEnvironment;

impl LocalEnvironment {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LocalEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl ExecutionEnvironment for LocalEnvironment {
    fn env_type(&self) -> EnvironmentType {
        EnvironmentType::Local
    }

    fn display_name(&self) -> String {
        "Local".to_string()
    }

    fn env_id(&self) -> String {
        "local".to_string()
    }

    async fn list_platforms(&self) -> Result<Vec<PlatformInfo>, EnvError> {
        // 支持的平台列表
        let platforms = vec![
            PlatformInfo {
                name: "claude".to_string(),
                display_name: "Claude Code".to_string(),
                installed: true, // TODO: 实际检测
                version: None,
            },
            PlatformInfo {
                name: "codex".to_string(),
                display_name: "Codex CLI".to_string(),
                installed: true,
                version: None,
            },
            PlatformInfo {
                name: "gemini".to_string(),
                display_name: "Antigravity CLI".to_string(),
                installed: true,
                version: None,
            },
            PlatformInfo {
                name: "opencode".to_string(),
                display_name: "OpenCode".to_string(),
                installed: true,
                version: None,
            },
        ];
        Ok(platforms)
    }

    async fn read_config(&self, platform: &str, path: &str) -> Result<String, EnvError> {
        // 委托到 ccr 核心库读取本地配置文件
        let full_path = resolve_config_path(platform, path)?;
        tokio::fs::read_to_string(&full_path)
            .await
            .map_err(EnvError::Io)
    }

    async fn write_config(
        &self,
        platform: &str,
        path: &str,
        content: &str,
    ) -> Result<(), EnvError> {
        let full_path = resolve_config_path(platform, path)?;
        // 确保父目录存在
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(EnvError::Io)?;
        }
        write_guarded_async(
            &full_path,
            content.as_bytes().to_vec(),
            WriteOptions {
                backup: BackupPolicy::SameDir {
                    tag: Some("ccr_ui".to_string()),
                },
                ..Default::default()
            },
        )
        .await
        .map_err(|error| EnvError::Other(format!("guarded config write failed: {error}")))
    }

    async fn detect_cli_status(&self) -> Result<Vec<CliStatus>, EnvError> {
        let tools = [
            ("claude", "claude"),
            ("codex", "codex"),
            ("gemini", "agy"),
            ("opencode", "opencode"),
        ];
        let mut statuses = Vec::new();

        for (platform, command) in &tools {
            let installed = which_tool(command).await;
            statuses.push(CliStatus {
                name: platform.to_string(),
                installed: installed.is_some(),
                path: installed.clone(),
                version: None,
            });
        }

        Ok(statuses)
    }
}

/// 解析平台配置文件路径
fn resolve_config_path(
    platform: &str,
    relative_path: &str,
) -> Result<std::path::PathBuf, EnvError> {
    let home =
        dirs::home_dir().ok_or_else(|| EnvError::Other("home directory not found".to_string()))?;

    let base = match platform {
        "claude" => home.join(".claude"),
        "codex" => home.join(".codex"),
        "gemini" => home.join(".gemini").join("antigravity-cli"),
        "opencode" => home.join(".opencode"),
        _ => return Err(EnvError::PlatformNotSupported(platform.to_string())),
    };

    let safe_relative_path = normalize_config_relative_path(relative_path)?;
    Ok(base.join(safe_relative_path))
}

/// 检测 CLI 工具是否可用（通过 PATH 查找）
async fn which_tool(name: &str) -> Option<String> {
    let output = ProcessGateway::execute(
        &ProcessDescriptor::path_lookup(),
        &[std::ffi::OsString::from(name)],
    )
    .await
    .ok()?;

    match output {
        output if output.status.success() && !output.timed_out && !output.stdout_truncated => {
            let path = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if path.is_empty() { None } else { Some(path) }
        }
        _ => None,
    }
}
