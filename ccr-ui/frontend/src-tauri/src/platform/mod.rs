//! 执行环境抽象层 — 统一 Local / WSL / SSH 平台操作。
//!
//! 核心 trait `ExecutionEnvironment` 定义了平台无关的配置管理接口，
//! `EnvironmentRegistry` 管理多个环境实例并支持运行时切换。

pub mod local;
pub mod ssh;
#[cfg(target_os = "windows")]
pub mod wsl;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// 环境类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnvironmentType {
    /// 本地环境（始终可用）
    Local,
    /// WSL 发行版（仅 Windows）
    Wsl,
    /// SSH 远程主机
    Ssh,
}

/// 平台信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub name: String,
    pub display_name: String,
    pub installed: bool,
    pub version: Option<String>,
}

/// CLI 工具安装状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliStatus {
    pub name: String,
    pub installed: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

/// 环境操作错误
#[derive(Debug, thiserror::Error)]
pub enum EnvError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("config not found: {0}")]
    ConfigNotFound(String),

    #[error("platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("operation timeout")]
    #[allow(dead_code)]
    Timeout,

    #[error("{0}")]
    Other(String),
}

impl Serialize for EnvError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// 执行环境 trait — 所有环境（Local/WSL/SSH）必须实现此接口。
#[async_trait::async_trait]
pub trait ExecutionEnvironment: Send + Sync {
    /// 环境类型
    fn env_type(&self) -> EnvironmentType;

    /// 人类可读的环境名称
    fn display_name(&self) -> String;

    /// 唯一标识符
    fn env_id(&self) -> String;

    /// 列出该环境中可用的 AI CLI 平台
    async fn list_platforms(&self) -> Result<Vec<PlatformInfo>, EnvError>;

    /// 读取指定平台的配置文件内容
    async fn read_config(&self, platform: &str, path: &str) -> Result<String, EnvError>;

    /// 写入指定平台的配置文件
    async fn write_config(&self, platform: &str, path: &str, content: &str)
    -> Result<(), EnvError>;

    /// 检测已安装的 CLI 工具状态
    async fn detect_cli_status(&self) -> Result<Vec<CliStatus>, EnvError>;
}

/// 环境注册表 — 管理多个执行环境，支持运行时切换。
pub struct EnvironmentRegistry {
    /// 已注册的环境列表
    environments: Vec<Arc<dyn ExecutionEnvironment>>,
    /// 当前活跃环境索引
    active_index: usize,
}

impl EnvironmentRegistry {
    /// 创建空注册表
    pub fn new() -> Self {
        Self {
            environments: Vec::new(),
            active_index: 0,
        }
    }

    /// 注册新环境
    pub fn register(&mut self, env: Arc<dyn ExecutionEnvironment>) {
        self.environments.push(env);
    }

    /// 清空环境并重置活跃索引
    pub fn clear(&mut self) {
        self.environments.clear();
        self.active_index = 0;
    }

    /// 获取当前活跃环境
    pub fn active(&self) -> Option<Arc<dyn ExecutionEnvironment>> {
        self.environments.get(self.active_index).cloned()
    }

    /// 切换活跃环境（按索引）
    #[allow(dead_code)]
    pub fn switch(&mut self, index: usize) -> Result<(), EnvError> {
        if index >= self.environments.len() {
            return Err(EnvError::Other(format!(
                "environment index {} out of range (total: {})",
                index,
                self.environments.len()
            )));
        }
        self.active_index = index;
        Ok(())
    }

    /// 按 ID 切换活跃环境
    pub fn switch_by_id(&mut self, id: &str) -> Result<(), EnvError> {
        let index = self
            .environments
            .iter()
            .position(|e| e.env_id() == id)
            .ok_or_else(|| EnvError::Other(format!("environment not found: {}", id)))?;
        self.active_index = index;
        Ok(())
    }

    /// 列出所有已注册环境
    pub fn list(&self) -> Vec<EnvironmentInfo> {
        self.environments
            .iter()
            .enumerate()
            .map(|(i, env)| {
                let env_type = env.env_type();
                let display_name = env.display_name();
                let id = env.env_id();
                let description = match env_type {
                    EnvironmentType::Local => "本机环境".to_string(),
                    EnvironmentType::Wsl => "Windows Subsystem for Linux".to_string(),
                    EnvironmentType::Ssh => "远程 SSH 主机".to_string(),
                };
                EnvironmentInfo {
                    id,
                    env_type,
                    name: display_name.clone(),
                    display_name,
                    description,
                    is_active: i == self.active_index,
                }
            })
            .collect()
    }

    /// 获取环境数量
    pub fn len(&self) -> usize {
        self.environments.len()
    }

    /// 是否为空
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.environments.is_empty()
    }
}

impl Default for EnvironmentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 环境概要信息（用于前端展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub id: String,
    pub env_type: EnvironmentType,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub is_active: bool,
}
