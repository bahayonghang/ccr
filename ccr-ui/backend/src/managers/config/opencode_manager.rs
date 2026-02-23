// OpenCode 配置文件管理器
//
// 管理 ~/.config/opencode/opencode.json 的读写操作
// 跨平台路径解析：
//   Linux:   ~/.config/opencode/opencode.json
//   macOS:   ~/Library/Application Support/opencode/opencode.json
//   Windows: %APPDATA%\opencode\opencode.json

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

// ============ 数据结构定义 ============

/// Provider 选项（API Key、Base URL 等）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenCodeProviderOptions {
    #[serde(rename = "baseURL", skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    #[serde(rename = "apiKey", skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    /// 保留未知字段
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// 模型限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeModelLimit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<u64>,
}

/// 单个模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeModel {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<OpenCodeModelLimit>,

    /// 保留未知字段
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Provider 配置（叠加式 npm AI SDK provider）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeProvider {
    /// npm 包名，如 "@ai-sdk/anthropic"
    pub npm: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(default)]
    pub options: OpenCodeProviderOptions,

    #[serde(default)]
    pub models: IndexMap<String, OpenCodeModel>,
}

/// MCP 服务器配置（原生 OpenCode 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeMcpServer {
    /// 服务器类型："local" | "remote"
    #[serde(rename = "type")]
    pub server_type: String,

    /// local 类型：命令数组 [cmd, ...args]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,

    /// local 类型：环境变量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,

    /// remote 类型：URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// remote 类型：请求头
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

/// OpenCode 完整配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenCodeConfig {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,

    #[serde(default)]
    pub provider: IndexMap<String, OpenCodeProvider>,

    #[serde(default)]
    pub mcp: IndexMap<String, OpenCodeMcpServer>,

    #[serde(default)]
    pub plugin: Vec<String>,

    /// 保留未知顶层字段
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

// ============ 管理器实现 ============

/// OpenCode 配置文件管理器
pub struct OpenCodeConfigManager {
    config_path: PathBuf,
}

impl OpenCodeConfigManager {
    /// 使用系统默认配置目录初始化管理器
    pub fn default() -> io::Result<Self> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "无法获取系统配置目录（config_dir）",
            )
        })?;
        let config_path = config_dir.join("opencode").join("opencode.json");
        Ok(Self { config_path })
    }

    /// 读取配置文件，文件不存在时返回空配置
    pub fn read(&self) -> io::Result<OpenCodeConfig> {
        if !self.config_path.exists() {
            return Ok(OpenCodeConfig::default());
        }

        let content = fs::read_to_string(&self.config_path)?;
        if content.trim().is_empty() {
            return Ok(OpenCodeConfig::default());
        }

        serde_json::from_str(&content).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("解析 opencode.json 失败: {}", e),
            )
        })
    }

    /// 原子写入配置文件（tempfile + rename）
    pub fn write(&self, config: &OpenCodeConfig) -> io::Result<()> {
        // 创建父目录（如果不存在）
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 写入临时文件
        let temp_file = NamedTempFile::new_in(
            self.config_path
                .parent()
                .unwrap_or_else(|| Path::new(".")),
        )?;

        let content = serde_json::to_string_pretty(config).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("序列化配置失败: {}", e),
            )
        })?;

        fs::write(temp_file.path(), content)?;

        // 原子重命名
        temp_file
            .persist(&self.config_path)
            .map_err(|e| io::Error::other(format!("写入配置文件失败: {}", e)))?;

        Ok(())
    }

    // ============ Provider 管理 ============

    /// 列出所有 Provider
    pub fn list_providers(&self) -> io::Result<IndexMap<String, OpenCodeProvider>> {
        let config = self.read()?;
        Ok(config.provider)
    }

    /// 添加或更新 Provider
    pub fn set_provider(&self, id: String, provider: OpenCodeProvider) -> io::Result<()> {
        let mut config = self.read()?;
        config.provider.insert(id, provider);
        self.write(&config)
    }

    /// 删除 Provider
    pub fn delete_provider(&self, id: &str) -> io::Result<()> {
        let mut config = self.read()?;
        if config.provider.shift_remove(id).is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Provider '{}' 不存在", id),
            ));
        }
        self.write(&config)
    }

    // ============ MCP 服务器管理 ============

    /// 列出所有 MCP 服务器
    pub fn list_mcp_servers(&self) -> io::Result<IndexMap<String, OpenCodeMcpServer>> {
        let config = self.read()?;
        Ok(config.mcp)
    }

    /// 添加或更新 MCP 服务器
    pub fn set_mcp_server(&self, id: String, server: OpenCodeMcpServer) -> io::Result<()> {
        let mut config = self.read()?;
        config.mcp.insert(id, server);
        self.write(&config)
    }

    /// 删除 MCP 服务器
    pub fn delete_mcp_server(&self, id: &str) -> io::Result<()> {
        let mut config = self.read()?;
        if config.mcp.shift_remove(id).is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("MCP 服务器 '{}' 不存在", id),
            ));
        }
        self.write(&config)
    }

    // ============ Plugin 管理 ============

    /// 列出所有 Plugin
    pub fn list_plugins(&self) -> io::Result<Vec<String>> {
        let config = self.read()?;
        Ok(config.plugin)
    }

    /// 添加 Plugin（如已存在则跳过）
    pub fn add_plugin(&self, npm_package: String) -> io::Result<()> {
        let mut config = self.read()?;
        if !config.plugin.contains(&npm_package) {
            config.plugin.push(npm_package);
            self.write(&config)?;
        }
        Ok(())
    }

    /// 删除 Plugin
    pub fn remove_plugin(&self, npm_package: &str) -> io::Result<()> {
        let mut config = self.read()?;
        let original_len = config.plugin.len();
        config.plugin.retain(|p| p != npm_package);
        if config.plugin.len() == original_len {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Plugin '{}' 不存在", npm_package),
            ));
        }
        self.write(&config)
    }

    /// 获取完整配置
    pub fn get_config(&self) -> io::Result<OpenCodeConfig> {
        self.read()
    }
}
