// 📦 MCP 预设数据模型
// 定义 MCP 服务器预设模板
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP 服务器预设模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPreset {
    /// 预设唯一标识符
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 功能描述
    pub description: String,
    /// 服务器配置
    pub server: McpServerSpec,
    /// 标签（用于分类和搜索）
    pub tags: Vec<String>,
    /// 主页链接
    pub homepage: Option<String>,
    /// 文档链接
    pub docs: Option<String>,
    /// 是否需要 API Key
    pub requires_api_key: bool,
    /// API Key 环境变量名（如果需要）
    pub api_key_env: Option<String>,
}

/// MCP 服务器配置规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerSpec {
    /// 服务器类型：stdio, http, sse
    #[serde(rename = "type", default = "default_server_type")]
    pub server_type: String,
    /// 命令（stdio 类型必需）
    pub command: Option<String>,
    /// 命令参数
    #[serde(default)]
    pub args: Vec<String>,
    /// 环境变量
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// URL（http/sse 类型必需）
    pub url: Option<String>,
}

fn default_server_type() -> String {
    "stdio".to_string()
}

impl McpPreset {
    /// 创建一个新的预设
    pub fn new(id: &str, name: &str, description: &str, command: &str, args: Vec<&str>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            server: McpServerSpec {
                server_type: "stdio".to_string(),
                command: Some(command.to_string()),
                args: args.into_iter().map(|s| s.to_string()).collect(),
                env: HashMap::new(),
                url: None,
            },
            tags: vec![],
            homepage: None,
            docs: None,
            requires_api_key: false,
            api_key_env: None,
        }
    }

    /// 添加标签
    pub fn with_tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(|s| s.to_string()).collect();
        self
    }

    /// 添加主页和文档链接
    pub fn with_links(mut self, homepage: &str, docs: &str) -> Self {
        self.homepage = Some(homepage.to_string());
        self.docs = Some(docs.to_string());
        self
    }

    /// 标记需要 API Key
    pub fn with_api_key(mut self, env_name: &str) -> Self {
        self.requires_api_key = true;
        self.api_key_env = Some(env_name.to_string());
        self
    }

    /// 添加环境变量
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.server.env.insert(key.to_string(), value.to_string());
        self
    }
}

/// 预设类别（用于前端分组显示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPresetCategory {
    /// 类别标识
    pub id: String,
    /// 类别名称
    pub name: String,
    /// 类别描述
    pub description: String,
    /// 该类别下的预设 ID 列表
    pub preset_ids: Vec<String>,
}
