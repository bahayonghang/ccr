// 🎯 MCP 预设管理器
// 管理 MCP 服务器预设模板，支持多平台同步
//
// 功能：
// - 📋 提供内置预设模板（fetch, context7, sequential-thinking, exa, serena）
// - 🔄 多平台同步（Claude → Codex/Gemini/Qwen/iFlow）
// - 💾 安装预设到指定平台
#![allow(dead_code)]

use crate::core::error::{CcrError, Result};
use crate::models::Platform;
use crate::models::mcp_preset::{McpPreset, McpServerSpec};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 内置 MCP 预设列表
pub fn get_builtin_presets() -> Vec<McpPreset> {
    vec![
        // 1. Fetch - 网页内容获取
        McpPreset::new(
            "fetch",
            "mcp-server-fetch",
            "获取网页内容，支持 HTML 转 Markdown",
            "uvx",
            vec!["mcp-server-fetch"],
        )
        .with_tags(vec!["http", "web", "fetch", "网络"])
        .with_links(
            "https://github.com/modelcontextprotocol/servers",
            "https://github.com/modelcontextprotocol/servers/tree/main/src/fetch",
        ),
        // 2. Context7 - 文档搜索和上下文增强
        McpPreset::new(
            "context7",
            "@upstash/context7-mcp",
            "文档搜索和上下文增强，自动获取库的最新文档",
            "npx",
            vec!["-y", "@upstash/context7-mcp"],
        )
        .with_tags(vec!["docs", "search", "context", "文档"])
        .with_links(
            "https://context7.com",
            "https://github.com/upstash/context7",
        ),
        // 3. Sequential Thinking - 顺序思维推理
        McpPreset::new(
            "sequential-thinking",
            "@modelcontextprotocol/server-sequential-thinking",
            "顺序思维推理，支持复杂问题分步解决",
            "npx",
            vec!["-y", "@modelcontextprotocol/server-sequential-thinking"],
        )
        .with_tags(vec!["thinking", "reasoning", "推理"])
        .with_links(
            "https://github.com/modelcontextprotocol/servers",
            "https://github.com/modelcontextprotocol/servers/tree/main/src/sequentialthinking",
        ),
        // 4. Exa - AI 搜索引擎
        McpPreset::new(
            "exa",
            "exa-mcp-server",
            "AI 驱动的搜索引擎，支持语义搜索和内容提取",
            "npx",
            vec!["-y", "exa-mcp-server"],
        )
        .with_tags(vec!["search", "ai", "搜索"])
        .with_links(
            "https://exa.ai",
            "https://github.com/exa-labs/exa-mcp-server",
        )
        .with_api_key("EXA_API_KEY"),
        // 5. Serena - 代码语义分析
        McpPreset::new(
            "serena",
            "serena",
            "代码语义分析和理解，支持多种编程语言",
            "uvx",
            vec!["serena"],
        )
        .with_tags(vec!["code", "semantic", "analysis", "代码"])
        .with_links(
            "https://github.com/oramasearch/serena",
            "https://github.com/oramasearch/serena",
        ),
    ]
}

/// MCP 预设管理器
pub struct McpPresetManager {
    /// 当前平台
    platform: Platform,
    /// 平台配置目录
    platform_dir: PathBuf,
    /// CCR 统一配置目录
    #[allow(dead_code)]
    ccr_dir: PathBuf,
}

impl McpPresetManager {
    /// 创建新的预设管理器
    pub fn new(platform: Platform) -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("Cannot find home directory".into()))?;

        let platform_dir = match platform {
            Platform::Claude => home.join(".claude"),
            Platform::Codex => home.join(".codex"),
            Platform::Gemini => home.join(".gemini"),
            Platform::Qwen => home.join(".qwen"),
            Platform::IFlow => home.join(".iflow"),
        };

        let ccr_dir = home.join(".ccr");

        Ok(Self {
            platform,
            platform_dir,
            ccr_dir,
        })
    }

    /// 获取所有内置预设
    pub fn list_presets(&self) -> Vec<McpPreset> {
        get_builtin_presets()
    }

    /// 根据 ID 获取预设
    pub fn get_preset(&self, id: &str) -> Option<McpPreset> {
        get_builtin_presets().into_iter().find(|p| p.id == id)
    }

    /// 安装预设到当前平台
    pub fn install_preset(
        &self,
        preset_id: &str,
        custom_env: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let preset = self.get_preset(preset_id).ok_or_else(|| {
            CcrError::ResourceNotFound(format!("Preset '{}' not found", preset_id))
        })?;

        // 合并自定义环境变量
        let mut server_spec = preset.server.clone();
        if let Some(env) = custom_env {
            for (k, v) in env {
                server_spec.env.insert(k, v);
            }
        }

        // 根据平台写入配置
        match self.platform {
            Platform::Claude => self.install_to_claude(&preset.id, &server_spec),
            Platform::Codex => self.install_to_codex(&preset.id, &server_spec),
            Platform::Gemini => self.install_to_gemini(&preset.id, &server_spec),
            Platform::Qwen => self.install_to_qwen(&preset.id, &server_spec),
            Platform::IFlow => self.install_to_iflow(&preset.id, &server_spec),
        }
    }

    /// 直接安装 MCP 服务器配置（用于同步功能）
    pub fn install_mcp_server(&self, name: &str, spec: &McpServerSpec) -> Result<()> {
        match self.platform {
            Platform::Claude => self.install_to_claude(name, spec),
            Platform::Codex => self.install_to_codex(name, spec),
            Platform::Gemini => self.install_to_gemini(name, spec),
            Platform::Qwen => self.install_to_qwen(name, spec),
            Platform::IFlow => self.install_to_iflow(name, spec),
        }
    }

    fn install_to_claude(&self, id: &str, spec: &McpServerSpec) -> Result<()> {
        let config_path = self.platform_dir.join("claude.json");
        let mut config = self.load_json_config(&config_path)?;

        // 确保 mcpServers 存在
        let mcp_servers = config
            .as_object_mut()
            .ok_or_else(|| CcrError::ConfigError("Invalid Claude config format".into()))?
            .entry("mcpServers")
            .or_insert_with(|| serde_json::json!({}));

        // 添加 MCP 服务器
        let server_config = self.spec_to_claude_format(spec);
        mcp_servers
            .as_object_mut()
            .ok_or_else(|| CcrError::ConfigError("Invalid mcpServers format".into()))?
            .insert(id.to_string(), server_config);

        // 写入配置
        self.save_json_config(&config_path, &config)?;
        tracing::info!("Installed MCP preset '{}' to Claude", id);
        Ok(())
    }

    /// 安装到 Codex (~/.codex/config.toml)
    fn install_to_codex(&self, id: &str, spec: &McpServerSpec) -> Result<()> {
        let config_path = self.platform_dir.join("config.toml");
        let mut config = self.load_toml_config(&config_path)?;

        // 确保 config 是一个 Table
        let table = match &mut config {
            toml::Value::Table(t) => t,
            _ => return Err(CcrError::ConfigError("Invalid Codex config format".into())),
        };

        // 确保 mcp_servers 存在
        if !table.contains_key("mcp_servers") {
            table.insert(
                "mcp_servers".to_string(),
                toml::Value::Table(toml::map::Map::new()),
            );
        }

        // 添加 MCP 服务器
        let server_config = self.spec_to_toml_format(spec);
        if let Some(toml::Value::Table(mcp_table)) = table.get_mut("mcp_servers") {
            mcp_table.insert(id.to_string(), server_config);
        }

        // 写入配置
        self.save_toml_config(&config_path, &config)?;
        tracing::info!("Installed MCP preset '{}' to Codex", id);
        Ok(())
    }

    /// 安装到 Gemini (~/.gemini/settings.json)
    fn install_to_gemini(&self, id: &str, spec: &McpServerSpec) -> Result<()> {
        let config_path = self.platform_dir.join("settings.json");
        let mut config = self.load_json_config(&config_path)?;

        // Gemini 使用 mcpServers 字段
        let mcp_servers = config
            .as_object_mut()
            .ok_or_else(|| CcrError::ConfigError("Invalid Gemini config format".into()))?
            .entry("mcpServers")
            .or_insert_with(|| serde_json::json!({}));

        let server_config = self.spec_to_gemini_format(spec);
        mcp_servers
            .as_object_mut()
            .ok_or_else(|| CcrError::ConfigError("Invalid mcpServers format".into()))?
            .insert(id.to_string(), server_config);

        self.save_json_config(&config_path, &config)?;
        tracing::info!("Installed MCP preset '{}' to Gemini", id);
        Ok(())
    }

    /// 安装到 Qwen
    fn install_to_qwen(&self, id: &str, spec: &McpServerSpec) -> Result<()> {
        // Qwen 使用与 Claude 类似的 JSON 格式
        let config_path = self.platform_dir.join("config.json");
        let mut config = self.load_json_config(&config_path)?;

        let mcp_servers = config
            .as_object_mut()
            .ok_or_else(|| CcrError::ConfigError("Invalid Qwen config format".into()))?
            .entry("mcpServers")
            .or_insert_with(|| serde_json::json!({}));

        let server_config = self.spec_to_claude_format(spec);
        mcp_servers
            .as_object_mut()
            .ok_or_else(|| CcrError::ConfigError("Invalid mcpServers format".into()))?
            .insert(id.to_string(), server_config);

        self.save_json_config(&config_path, &config)?;
        tracing::info!("Installed MCP preset '{}' to Qwen", id);
        Ok(())
    }

    /// 安装到 iFlow
    fn install_to_iflow(&self, id: &str, spec: &McpServerSpec) -> Result<()> {
        // iFlow 使用与 Claude 类似的 JSON 格式
        let config_path = self.platform_dir.join("config.json");
        let mut config = self.load_json_config(&config_path)?;

        let mcp_servers = config
            .as_object_mut()
            .ok_or_else(|| CcrError::ConfigError("Invalid iFlow config format".into()))?
            .entry("mcpServers")
            .or_insert_with(|| serde_json::json!({}));

        let server_config = self.spec_to_claude_format(spec);
        mcp_servers
            .as_object_mut()
            .ok_or_else(|| CcrError::ConfigError("Invalid mcpServers format".into()))?
            .insert(id.to_string(), server_config);

        self.save_json_config(&config_path, &config)?;
        tracing::info!("Installed MCP preset '{}' to iFlow", id);
        Ok(())
    }

    /// 将 McpServerSpec 转换为 Claude JSON 格式
    fn spec_to_claude_format(&self, spec: &McpServerSpec) -> serde_json::Value {
        let mut obj = serde_json::Map::new();

        if let Some(ref cmd) = spec.command {
            obj.insert("command".to_string(), serde_json::json!(cmd));
        }

        if !spec.args.is_empty() {
            obj.insert("args".to_string(), serde_json::json!(spec.args));
        }

        if !spec.env.is_empty() {
            obj.insert("env".to_string(), serde_json::json!(spec.env));
        }

        if let Some(ref url) = spec.url {
            obj.insert("url".to_string(), serde_json::json!(url));
            obj.insert("type".to_string(), serde_json::json!(spec.server_type));
        }

        serde_json::Value::Object(obj)
    }

    /// 将 McpServerSpec 转换为 Gemini JSON 格式
    fn spec_to_gemini_format(&self, spec: &McpServerSpec) -> serde_json::Value {
        // Gemini 格式与 Claude 基本相同
        self.spec_to_claude_format(spec)
    }

    /// 将 McpServerSpec 转换为 Codex TOML 格式
    fn spec_to_toml_format(&self, spec: &McpServerSpec) -> toml::Value {
        let mut table = toml::map::Map::new();

        if let Some(ref cmd) = spec.command {
            table.insert("command".to_string(), toml::Value::String(cmd.clone()));
        }

        if !spec.args.is_empty() {
            let args: Vec<toml::Value> = spec
                .args
                .iter()
                .map(|a| toml::Value::String(a.clone()))
                .collect();
            table.insert("args".to_string(), toml::Value::Array(args));
        }

        if !spec.env.is_empty() {
            let mut env_table = toml::map::Map::new();
            for (k, v) in &spec.env {
                env_table.insert(k.clone(), toml::Value::String(v.clone()));
            }
            table.insert("env".to_string(), toml::Value::Table(env_table));
        }

        toml::Value::Table(table)
    }

    /// 加载 JSON 配置文件
    fn load_json_config(&self, path: &PathBuf) -> Result<serde_json::Value> {
        if path.exists() {
            let content = fs::read_to_string(path).map_err(CcrError::IoError)?;
            serde_json::from_str(&content).map_err(|e| CcrError::ConfigFormatInvalid(e.to_string()))
        } else {
            Ok(serde_json::json!({}))
        }
    }

    /// 保存 JSON 配置文件（原子写入）
    fn save_json_config(&self, path: &PathBuf, config: &serde_json::Value) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).map_err(CcrError::IoError)?;
        }

        // 原子写入：先写入临时文件，再重命名
        let temp_path = path.with_extension("json.tmp");
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| CcrError::ConfigError(e.to_string()))?;
        fs::write(&temp_path, content).map_err(CcrError::IoError)?;
        fs::rename(&temp_path, path).map_err(CcrError::IoError)?;

        Ok(())
    }

    /// 加载 TOML 配置文件
    fn load_toml_config(&self, path: &PathBuf) -> Result<toml::Value> {
        if path.exists() {
            let content = fs::read_to_string(path).map_err(CcrError::IoError)?;
            toml::from_str(&content).map_err(|e| CcrError::ConfigFormatInvalid(e.to_string()))
        } else {
            Ok(toml::Value::Table(toml::map::Map::new()))
        }
    }

    /// 保存 TOML 配置文件（原子写入）
    fn save_toml_config(&self, path: &PathBuf, config: &toml::Value) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent).map_err(CcrError::IoError)?;
        }

        // 原子写入
        let temp_path = path.with_extension("toml.tmp");
        let content =
            toml::to_string_pretty(config).map_err(|e| CcrError::ConfigError(e.to_string()))?;
        fs::write(&temp_path, content).map_err(CcrError::IoError)?;
        fs::rename(&temp_path, path).map_err(CcrError::IoError)?;

        Ok(())
    }
}

/// 多平台同步管理器
pub struct McpSyncManager {
    /// 源平台（默认 Claude）
    #[allow(dead_code)]
    source: Platform,
    /// 源平台配置目录
    source_dir: PathBuf,
}

impl McpSyncManager {
    /// 创建同步管理器
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| {
            tracing::warn!("无法获取用户主目录，使用空路径");
            PathBuf::new()
        });
        Self {
            source: Platform::Claude,
            source_dir: home.join(".claude"),
        }
    }

    /// 同步预设到所有平台
    pub fn sync_preset_to_all(
        &self,
        preset_id: &str,
        custom_env: Option<HashMap<String, String>>,
        target_platforms: &[Platform],
    ) -> Result<Vec<(Platform, Result<()>)>> {
        let mut results = Vec::new();

        for platform in target_platforms {
            let manager = McpPresetManager::new(*platform)?;
            let result = manager.install_preset(preset_id, custom_env.clone());
            results.push((*platform, result));
        }

        Ok(results)
    }

    /// 同步预设到指定平台
    pub fn sync_preset(
        &self,
        preset_id: &str,
        custom_env: Option<HashMap<String, String>>,
        target: Platform,
    ) -> Result<()> {
        let manager = McpPresetManager::new(target)?;
        manager.install_preset(preset_id, custom_env)
    }

    /// 从源平台读取所有 MCP 服务器
    pub fn list_source_mcp_servers(&self) -> Result<HashMap<String, McpServerSpec>> {
        let config_path = self.source_dir.join("claude.json");

        if !config_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&config_path).map_err(CcrError::IoError)?;
        let config: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| CcrError::ConfigError(e.to_string()))?;

        let mut servers = HashMap::new();

        if let Some(mcp_servers) = config.get("mcpServers").and_then(|v| v.as_object()) {
            for (name, server_config) in mcp_servers {
                if let Some(spec) = self.parse_mcp_server_config(server_config) {
                    servers.insert(name.clone(), spec);
                }
            }
        }

        Ok(servers)
    }

    /// 解析 MCP 服务器配置
    fn parse_mcp_server_config(&self, config: &serde_json::Value) -> Option<McpServerSpec> {
        let command = config.get("command")?.as_str()?.to_string();
        let args = config
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let env = config
            .get("env")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        Some(McpServerSpec {
            server_type: "stdio".to_string(),
            command: Some(command),
            args,
            env,
            url: None,
        })
    }

    /// 同步指定 MCP 服务器到目标平台
    pub fn sync_mcp_server(
        &self,
        server_name: &str,
        target_platforms: &[Platform],
    ) -> Result<Vec<(Platform, Result<()>)>> {
        let servers = self.list_source_mcp_servers()?;
        let spec = servers.get(server_name).ok_or_else(|| {
            CcrError::ResourceNotFound(format!(
                "MCP server '{}' not found in source platform",
                server_name
            ))
        })?;

        let mut results = Vec::new();

        for platform in target_platforms {
            if *platform == Platform::Claude {
                continue; // 跳过源平台
            }

            let manager = McpPresetManager::new(*platform)?;
            let result = manager.install_mcp_server(server_name, spec);
            results.push((*platform, result));
        }

        Ok(results)
    }

    /// 同步所有 MCP 服务器到目标平台
    #[allow(clippy::type_complexity)]
    pub fn sync_all_mcp_servers(
        &self,
        target_platforms: &[Platform],
    ) -> Result<HashMap<String, Vec<(Platform, Result<()>)>>> {
        let servers = self.list_source_mcp_servers()?;
        let mut all_results = HashMap::new();

        for (name, spec) in servers {
            let mut results = Vec::new();

            for platform in target_platforms {
                if *platform == Platform::Claude {
                    continue;
                }

                let manager = McpPresetManager::new(*platform)?;
                let result = manager.install_mcp_server(&name, &spec);
                results.push((*platform, result));
            }

            all_results.insert(name, results);
        }

        Ok(all_results)
    }
}

impl Default for McpSyncManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_presets() {
        let presets = get_builtin_presets();
        assert_eq!(presets.len(), 5);

        // 验证预设 ID
        let ids: Vec<&str> = presets.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains(&"fetch"));
        assert!(ids.contains(&"context7"));
        assert!(ids.contains(&"sequential-thinking"));
        assert!(ids.contains(&"exa"));
        assert!(ids.contains(&"serena"));
    }

    #[test]
    fn test_get_preset() {
        let manager = McpPresetManager::new(Platform::Claude).unwrap();

        let fetch = manager.get_preset("fetch");
        assert!(fetch.is_some());
        assert_eq!(fetch.unwrap().name, "mcp-server-fetch");

        let nonexistent = manager.get_preset("nonexistent");
        assert!(nonexistent.is_none());
    }
}
