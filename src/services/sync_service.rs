// ☁️ CCR WebDAV 同步服务
// 📁 负责配置文件的云端同步
//
// 核心功能:
// - 🔼 上传配置到 WebDAV 服务器
// - 🔽 从 WebDAV 服务器下载配置
// - 🔄 智能同步(基于时间戳)
// - ✅ 连接测试

use crate::core::error::{CcrError, Result};
use crate::managers::config::SyncConfig;
use reqwest_dav::{Auth, Client, ClientBuilder, Depth, Error as DavError};
use reqwest_dav::re_exports::reqwest::StatusCode;
use std::path::Path;
use std::fs;

/// ☁️ WebDAV 同步服务
///
/// 封装 reqwest_dav 客户端，提供配置文件的云端同步功能
pub struct SyncService {
    client: Client,
    remote_path: String,
}

impl SyncService {
    /// 🏗️ 从配置创建同步服务
    ///
    /// # 参数
    /// - config: WebDAV 同步配置
    ///
    /// # 返回
    /// - Ok(SyncService): 成功创建的服务实例
    /// - Err: 创建失败（如网络配置错误）
    pub async fn new(config: &SyncConfig) -> Result<Self> {
        log::debug!("🔌 创建 WebDAV 客户端: {}", config.webdav_url);

        // 🔧 构建 WebDAV 客户端
        let client = ClientBuilder::new()
            .set_host(config.webdav_url.clone())
            .set_auth(Auth::Basic(
                config.username.clone(),
                config.password.clone(),
            ))
            .build()
            .map_err(|e| {
                CcrError::SyncError(format!("创建 WebDAV 客户端失败: {}", e))
            })?;

        Ok(Self {
            client,
            remote_path: config.remote_path.clone(),
        })
    }

    /// ✅ 测试连接
    ///
    /// 尝试列出远程目录以验证连接配置是否正确
    pub async fn test_connection(&self) -> Result<()> {
        log::debug!("🧪 测试 WebDAV 连接");

        // 🔍 尝试列出根目录
        self.client
            .list("/", Depth::Number(0))
            .await
            .map_err(|e| self.map_dav_error(e, "测试连接"))?;

        log::info!("✅ WebDAV 连接成功");
        Ok(())
    }

    /// 🔼 上传配置文件到 WebDAV
    ///
    /// # 参数
    /// - local_path: 本地配置文件路径
    ///
    /// # 返回
    /// - Ok(()): 上传成功
    /// - Err: 上传失败
    pub async fn push(&self, local_path: &Path) -> Result<()> {
        log::info!("🔼 上传配置到 WebDAV: {}", self.remote_path);

        // 📄 读取本地文件
        let content = fs::read(local_path).map_err(|e| {
            CcrError::SyncError(format!("读取本地配置失败: {}", e))
        })?;

        // 📁 确保远程目录存在
        self.ensure_remote_dir().await?;

        // 🔼 上传文件
        self.client
            .put(&self.remote_path, content)
            .await
            .map_err(|e| self.map_dav_error(e, "上传配置"))?;

        log::info!("✅ 配置已上传到云端");
        Ok(())
    }

    /// 🔽 从 WebDAV 下载配置文件
    ///
    /// # 参数
    /// - local_path: 本地保存路径
    ///
    /// # 返回
    /// - Ok(()): 下载成功
    /// - Err: 下载失败（如文件不存在）
    pub async fn pull(&self, local_path: &Path) -> Result<()> {
        log::info!("🔽 从 WebDAV 下载配置: {}", self.remote_path);

        // 🔽 下载文件
        let response = self
            .client
            .get(&self.remote_path)
            .await
            .map_err(|e| self.map_dav_error(e, "下载配置"))?;

        // 📄 读取响应内容
        let content = response
            .bytes()
            .await
            .map_err(|e| CcrError::SyncError(format!("读取响应内容失败: {}", e)))?;

        // 💾 保存到本地
        fs::write(local_path, content).map_err(|e| {
            CcrError::SyncError(format!("保存配置到本地失败: {}", e))
        })?;

        log::info!("✅ 配置已从云端下载");
        Ok(())
    }

    /// 🔍 检查远程文件是否存在
    pub async fn remote_exists(&self) -> Result<bool> {
        log::debug!("🔍 检查远程文件: {}", self.remote_path);

        match self.client.get(&self.remote_path).await {
            Ok(_) => Ok(true),
            // 文件不存在（404）
            Err(DavError::Reqwest(e)) if e.status() == Some(StatusCode::NOT_FOUND) => {
                Ok(false)
            }
            // 父目录不存在（409 - Conflict）或其他 Decode 错误
            // 坚果云在父目录不存在时返回 409 + AncestorsNotFound
            Err(DavError::Decode(_)) => {
                log::debug!("远程目录或文件不存在（409）");
                Ok(false)
            }
            Err(e) => Err(self.map_dav_error(e, "检查远程文件")),
        }
    }

    /// 📁 确保远程目录存在
    ///
    /// 自动创建远程文件路径中的目录
    async fn ensure_remote_dir(&self) -> Result<()> {
        // 🔍 提取目录路径
        let dir_path = Path::new(&self.remote_path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("/");

        if dir_path == "/" || dir_path.is_empty() {
            return Ok(());
        }

        log::debug!("📁 确保远程目录存在: {}", dir_path);

        // 📂 创建目录（如果不存在）
        // WebDAV mkcol 命令会在目录已存在时返回错误，我们忽略这个错误
        match self.client.mkcol(dir_path).await {
            Ok(_) => {
                log::debug!("✅ 远程目录已创建");
                Ok(())
            }
            Err(DavError::Reqwest(e))
                if e.status() == Some(StatusCode::METHOD_NOT_ALLOWED) =>
            {
                // 目录已存在，这不是错误
                log::debug!("ℹ️ 远程目录已存在");
                Ok(())
            }
            Err(e) => Err(self.map_dav_error(e, "创建远程目录")),
        }
    }

    /// 🔄 映射 WebDAV 错误到 CCR 错误
    fn map_dav_error(&self, err: DavError, operation: &str) -> CcrError {
        let msg = match &err {
            DavError::Reqwest(e) => {
                if let Some(status) = e.status() {
                    match status {
                        StatusCode::UNAUTHORIZED => {
                            "认证失败：用户名或密码错误".to_string()
                        }
                        StatusCode::NOT_FOUND => {
                            format!("文件不存在: {}", self.remote_path)
                        }
                        StatusCode::FORBIDDEN => {
                            "权限不足：无法访问该资源".to_string()
                        }
                        _ => format!("HTTP 错误 {}: {}", status, e),
                    }
                } else {
                    format!("网络错误: {}", e)
                }
            }
            _ => format!("{:?}", err),
        };

        CcrError::SyncError(format!("{} 失败: {}", operation, msg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> SyncConfig {
        SyncConfig {
            enabled: true,
            webdav_url: "https://dav.jianguoyun.com/dav/".to_string(),
            username: "test@example.com".to_string(),
            password: "test_password".to_string(),
            remote_path: "/ccr/.ccs_config.toml".to_string(),
            auto_sync: false,
        }
    }

    #[tokio::test]
    #[ignore] // 需要真实的 WebDAV 服务器
    async fn test_sync_service_creation() {
        let config = create_test_config();
        let service = SyncService::new(&config).await;
        assert!(service.is_ok());
    }
}
