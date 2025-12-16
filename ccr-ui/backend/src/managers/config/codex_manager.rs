// Codex CLI 配置管理器
// 负责读写 ~/.codex/config.toml 文件

use crate::models::platforms::codex::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use tracing::{info, warn};

/// Codex 配置管理器
pub struct CodexConfigManager {
    config_path: PathBuf,
}

impl CodexConfigManager {
    /// 创建默认的配置管理器（使用 ~/.codex/config.toml）
    pub fn default() -> Result<Self, String> {
        let home = dirs::home_dir().ok_or("无法获取用户主目录")?;
        let config_path = home.join(".codex").join("config.toml");
        Ok(Self { config_path })
    }

    /// 使用自定义配置路径创建管理器
    #[allow(dead_code)]
    pub fn with_path(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// 读取 Codex 配置文件
    pub fn read_config(&self) -> Result<CodexConfig, String> {
        info!("读取 Codex 配置: {:?}", self.config_path);

        // 如果文件不存在，返回空配置
        if !self.config_path.exists() {
            warn!("Codex 配置文件不存在，返回空配置");
            return Ok(CodexConfig::default());
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;

        toml::from_str(&content).map_err(|e| format!("解析 TOML 配置失败: {}", e))
    }

    /// 写入 Codex 配置文件（原子操作）
    pub fn write_config(&self, config: &CodexConfig) -> Result<(), String> {
        info!("写入 Codex 配置: {:?}", self.config_path);

        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
        }

        // 序列化为 TOML
        let toml_str =
            toml::to_string_pretty(config).map_err(|e| format!("序列化配置失败: {}", e))?;

        // 原子写入（临时文件 + 重命名）
        self.atomic_write(&self.config_path, &toml_str)?;

        info!("Codex 配置已成功写入");
        Ok(())
    }

    /// 原子写入文件
    fn atomic_write(&self, path: &Path, content: &str) -> Result<(), String> {
        let parent = path.parent().ok_or("无效的文件路径")?;

        let mut temp_file =
            NamedTempFile::new_in(parent).map_err(|e| format!("创建临时文件失败: {}", e))?;

        use std::io::Write;
        temp_file
            .write_all(content.as_bytes())
            .map_err(|e| format!("写入临时文件失败: {}", e))?;

        temp_file
            .flush()
            .map_err(|e| format!("刷新临时文件失败: {}", e))?;

        temp_file
            .persist(path)
            .map_err(|e| format!("持久化临时文件失败: {}", e))?;

        Ok(())
    }

    // ============ MCP 服务器管理 ============

    /// 列出所有 MCP 服务器
    pub fn list_mcp_servers(&self) -> Result<Vec<CodexMcpServerWithName>, String> {
        let config = self.read_config()?;

        let servers = config
            .mcp_servers
            .unwrap_or_default()
            .into_iter()
            .map(|(name, server)| CodexMcpServerWithName { name, server })
            .collect();

        Ok(servers)
    }

    /// 添加 MCP 服务器
    pub fn add_mcp_server(&self, name: String, server: CodexMcpServer) -> Result<(), String> {
        let mut config = self.read_config()?;

        // 检查是否已存在
        if let Some(ref servers) = config.mcp_servers
            && servers.contains_key(&name)
        {
            return Err(format!("MCP 服务器 '{}' 已存在", name));
        }

        // 添加服务器
        config
            .mcp_servers
            .get_or_insert_with(HashMap::new)
            .insert(name.clone(), server);

        self.write_config(&config)?;
        info!("已添加 Codex MCP 服务器: {}", name);
        Ok(())
    }

    /// 更新 MCP 服务器
    pub fn update_mcp_server(&self, name: &str, server: CodexMcpServer) -> Result<(), String> {
        let mut config = self.read_config()?;

        let servers = config
            .mcp_servers
            .as_mut()
            .ok_or("没有配置任何 MCP 服务器")?;

        if !servers.contains_key(name) {
            return Err(format!("MCP 服务器 '{}' 不存在", name));
        }

        servers.insert(name.to_string(), server);

        self.write_config(&config)?;
        info!("已更新 Codex MCP 服务器: {}", name);
        Ok(())
    }

    /// 删除 MCP 服务器
    pub fn delete_mcp_server(&self, name: &str) -> Result<(), String> {
        let mut config = self.read_config()?;

        let servers = config
            .mcp_servers
            .as_mut()
            .ok_or("没有配置任何 MCP 服务器")?;

        if servers.remove(name).is_none() {
            return Err(format!("MCP 服务器 '{}' 不存在", name));
        }

        self.write_config(&config)?;
        info!("已删除 Codex MCP 服务器: {}", name);
        Ok(())
    }

    // ============ 基础配置管理 ============

    /// 更新基础配置
    pub fn update_base_config(
        &self,
        model: Option<String>,
        model_provider: Option<String>,
        approval_policy: Option<String>,
        sandbox_mode: Option<String>,
        model_reasoning_effort: Option<String>,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        if let Some(m) = model {
            config.model = Some(m);
        }
        if let Some(mp) = model_provider {
            config.model_provider = Some(mp);
        }
        if let Some(ap) = approval_policy {
            config.approval_policy = Some(ap);
        }
        if let Some(sm) = sandbox_mode {
            config.sandbox_mode = Some(sm);
        }
        if let Some(mre) = model_reasoning_effort {
            config.model_reasoning_effort = Some(mre);
        }

        self.write_config(&config)?;
        info!("已更新 Codex 基础配置");
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_write_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let manager = CodexConfigManager::with_path(config_path.clone());

        // 写入配置
        let config = CodexConfig {
            model: Some("gpt-5".to_string()),
            mcp_servers: Some(HashMap::from([(
                "context7".to_string(),
                CodexMcpServer {
                    command: Some("npx".to_string()),
                    args: Some(vec!["-y".to_string(), "@upstash/context7-mcp".to_string()]),
                    env: None,
                    cwd: None,
                    startup_timeout_ms: Some(20000),
                    url: None,
                    bearer_token: None,
                    other: HashMap::new(),
                },
            )])),
            ..Default::default()
        };

        manager.write_config(&config).unwrap();

        // 读取配置
        let read_config = manager.read_config().unwrap();
        assert_eq!(read_config.model, Some("gpt-5".to_string()));
        assert!(read_config.mcp_servers.unwrap().contains_key("context7"));
    }

    #[test]
    fn test_add_mcp_server() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let manager = CodexConfigManager::with_path(config_path);

        let server = CodexMcpServer {
            command: Some("npx".to_string()),
            args: Some(vec!["-y".to_string(), "@upstash/context7-mcp".to_string()]),
            env: None,
            cwd: None,
            startup_timeout_ms: Some(20000),
            url: None,
            bearer_token: None,
            other: HashMap::new(),
        };

        manager
            .add_mcp_server("context7".to_string(), server)
            .unwrap();

        let servers = manager.list_mcp_servers().unwrap();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "context7");
    }
}
