// Qwen CLI 配置管理器
// 管理 .qwen/settings.json 文件

use crate::qwen_models::{QwenConfig, QwenMcpServer};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tracing::info;

pub struct QwenConfigManager {
    config_path: PathBuf,
}

impl QwenConfigManager {
    /// 创建默认实例，使用项目根目录的 .qwen/settings.json
    pub fn default() -> Result<Self, String> {
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("无法获取当前目录: {}", e))?;
        let qwen_dir = current_dir.join(".qwen");
        let config_path = qwen_dir.join("settings.json");

        // 确保目录存在
        if !qwen_dir.exists() {
            fs::create_dir_all(&qwen_dir)
                .map_err(|e| format!("创建 .qwen 目录失败: {}", e))?;
        }

        Ok(Self { config_path })
    }

    /// 使用自定义路径创建实例（用于测试）
    #[allow(dead_code)]
    pub fn with_path(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    /// 读取配置文件
    pub fn read_config(&self) -> Result<QwenConfig, String> {
        if !self.config_path.exists() {
            info!("Qwen 配置文件不存在，返回空配置");
            return Ok(QwenConfig::default());
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("读取 Qwen 配置文件失败: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("解析 Qwen JSON 失败: {}", e))
    }

    /// 写入配置文件
    pub fn write_config(&self, config: &QwenConfig) -> Result<(), String> {
        let json_str = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化 Qwen 配置失败: {}", e))?;

        self.atomic_write(&self.config_path, &json_str)?;
        info!("Qwen 配置已写入: {:?}", self.config_path);
        Ok(())
    }

    /// 原子写入文件（tempfile + rename）
    fn atomic_write(&self, path: &PathBuf, content: &str) -> Result<(), String> {
        let parent = path
            .parent()
            .ok_or_else(|| "无法获取父目录".to_string())?;

        let mut temp_file = NamedTempFile::new_in(parent)
            .map_err(|e| format!("创建临时文件失败: {}", e))?;

        use std::io::Write;
        temp_file
            .write_all(content.as_bytes())
            .map_err(|e| format!("写入临时文件失败: {}", e))?;

        temp_file
            .persist(path)
            .map_err(|e| format!("持久化文件失败: {}", e))?;

        Ok(())
    }

    // ============ MCP 服务器管理 ============

    /// 列出所有 MCP 服务器
    pub fn list_mcp_servers(&self) -> Result<Vec<(String, QwenMcpServer)>, String> {
        let config = self.read_config()?;
        match config.mcp_servers {
            Some(servers) => Ok(servers.into_iter().collect()),
            None => Ok(Vec::new()),
        }
    }

    /// 添加 MCP 服务器
    pub fn add_mcp_server(&self, name: String, server: QwenMcpServer) -> Result<(), String> {
        let mut config = self.read_config()?;
        config
            .mcp_servers
            .get_or_insert_with(HashMap::new)
            .insert(name.clone(), server);
        self.write_config(&config)?;
        info!("已添加 Qwen MCP 服务器: {}", name);
        Ok(())
    }

    /// 更新 MCP 服务器
    pub fn update_mcp_server(&self, name: &str, server: QwenMcpServer) -> Result<(), String> {
        let mut config = self.read_config()?;

        if let Some(servers) = &mut config.mcp_servers {
            if servers.contains_key(name) {
                servers.insert(name.to_string(), server);
                self.write_config(&config)?;
                info!("已更新 Qwen MCP 服务器: {}", name);
                Ok(())
            } else {
                Err(format!("MCP 服务器 '{}' 不存在", name))
            }
        } else {
            Err("配置中没有 MCP 服务器".to_string())
        }
    }

    /// 删除 MCP 服务器
    pub fn delete_mcp_server(&self, name: &str) -> Result<(), String> {
        let mut config = self.read_config()?;

        if let Some(servers) = &mut config.mcp_servers {
            if servers.remove(name).is_some() {
                self.write_config(&config)?;
                info!("已删除 Qwen MCP 服务器: {}", name);
                Ok(())
            } else {
                Err(format!("MCP 服务器 '{}' 不存在", name))
            }
        } else {
            Err("配置中没有 MCP 服务器".to_string())
        }
    }

    // ============ 基础配置管理 ============

    /// 获取完整配置
    pub fn get_config(&self) -> Result<QwenConfig, String> {
        self.read_config()
    }

    /// 更新完整配置
    pub fn update_config(&self, config: &QwenConfig) -> Result<(), String> {
        self.write_config(config)?;
        info!("已更新 Qwen 完整配置");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_write_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("settings.json");
        let manager = QwenConfigManager::with_path(config_path.clone());

        let mut config = QwenConfig::default();
        config.mcp_servers = Some(HashMap::from([(
            "test".to_string(),
            QwenMcpServer {
                command: Some("test-command".to_string()),
                args: Some(vec!["arg1".to_string()]),
                env: None,
                url: None,
                http_url: None,
                headers: None,
                timeout: Some(10000),
                other: HashMap::new(),
            },
        )]));

        manager.write_config(&config).unwrap();
        let read_config = manager.read_config().unwrap();

        assert!(read_config.mcp_servers.is_some());
        assert!(read_config.mcp_servers.unwrap().contains_key("test"));
    }

    #[test]
    fn test_mcp_server_crud() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("settings.json");
        let manager = QwenConfigManager::with_path(config_path);

        // 添加
        let server = QwenMcpServer {
            command: Some("docker".to_string()),
            args: Some(vec!["run".to_string(), "-i".to_string()]),
            env: None,
            url: None,
            http_url: None,
            headers: None,
            timeout: Some(30000),
            other: HashMap::new(),
        };
        manager.add_mcp_server("test".to_string(), server.clone()).unwrap();

        // 列表
        let servers = manager.list_mcp_servers().unwrap();
        assert_eq!(servers.len(), 1);

        // 更新
        let mut updated_server = server;
        updated_server.timeout = Some(60000);
        manager.update_mcp_server("test", updated_server).unwrap();

        // 删除
        manager.delete_mcp_server("test").unwrap();
        let servers = manager.list_mcp_servers().unwrap();
        assert_eq!(servers.len(), 0);
    }
}
