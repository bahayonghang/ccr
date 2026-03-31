// ☁️ CCR WebDAV 同步服务
// 📁 负责配置文件的云端同步
//
// 核心功能:
// - 🔼 上传配置到 WebDAV 服务器
// - 🔽 从 WebDAV 服务器下载配置
// - 🔄 智能同步(基于时间戳)
// - ✅ 连接测试

use ccr_config::SyncConfig;
use ccr_core::core::error::{CcrError, Result};
use reqwest_dav::list_cmd::ListEntity;
use reqwest_dav::re_exports::reqwest::StatusCode;
use reqwest_dav::{Auth, Client, ClientBuilder, Depth, Error as DavError};
use std::path::{Path, PathBuf};
use tokio::fs;

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
        tracing::debug!("🔌 创建 WebDAV 客户端: {}", config.webdav_url);

        // 🔧 构建 WebDAV 客户端
        let client = ClientBuilder::new()
            .set_host(config.webdav_url.clone())
            .set_auth(Auth::Basic(
                config.username.clone(),
                config.password.clone(),
            ))
            .build()
            .map_err(|e| CcrError::SyncError(format!("创建 WebDAV 客户端失败: {}", e)))?;

        Ok(Self {
            client,
            remote_path: config.remote_path.clone(),
        })
    }

    /// ✅ 测试连接
    ///
    /// 尝试列出远程目录以验证连接配置是否正确
    pub async fn test_connection(&self) -> Result<()> {
        tracing::debug!("🧪 测试 WebDAV 连接");

        // 🔍 尝试列出根目录
        self.client
            .list("/", Depth::Number(0))
            .await
            .map_err(|e| self.map_dav_error(e, "测试连接"))?;

        tracing::info!("✅ WebDAV 连接成功");
        Ok(())
    }

    /// 🔼 上传配置文件或目录到 WebDAV
    ///
    /// # 参数
    /// - local_path: 本地配置文件或目录路径
    /// - allowed_paths: 允许上传的路径列表（可选），如果提供则只上传这些路径
    ///
    /// # 返回
    /// - Ok(()): 上传成功
    /// - Err: 上传失败
    pub async fn push(&self, local_path: &Path, allowed_paths: Option<&[String]>) -> Result<()> {
        if local_path.is_dir() {
            tracing::info!(
                "🔼 上传目录到 WebDAV: {} -> {}",
                local_path.display(),
                self.remote_path
            );
            self.push_directory_filtered(local_path, &self.remote_path, allowed_paths)
                .await
        } else {
            tracing::info!(
                "🔼 上传文件到 WebDAV: {} -> {}",
                local_path.display(),
                self.remote_path
            );
            // 对于单个文件，检查是否在允许的路径中
            if let Some(allowed) = allowed_paths {
                let file_name = local_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| CcrError::SyncError("无效的文件名".into()))?;

                if !allowed.iter().any(|path| path == file_name) {
                    tracing::info!("文件 {} 不在允许的路径列表中，跳过上传", file_name);
                    return Ok(());
                }
            }
            self.push_file(local_path, &self.remote_path).await
        }
    }

    /// 🔼 上传单个文件到 WebDAV
    async fn push_file(&self, local_path: &Path, remote_path: &str) -> Result<()> {
        // 📄 读取本地文件
        let content = fs::read(local_path).await.map_err(|e| {
            CcrError::SyncError(format!("读取本地文件失败 {}: {}", local_path.display(), e))
        })?;

        // 📁 确保远程目录存在
        self.ensure_remote_dir_for_file(remote_path).await?;

        // 🔼 上传文件
        self.client
            .put(remote_path, content)
            .await
            .map_err(|e| self.map_dav_error(e, &format!("上传文件 {}", remote_path)))?;

        tracing::debug!("✅ 文件已上传: {}", remote_path);
        Ok(())
    }

    /// 🔼 递归上传目录到 WebDAV（带过滤）
    async fn push_directory_filtered(
        &self,
        local_dir: &Path,
        remote_dir: &str,
        allowed_paths: Option<&[String]>,
    ) -> Result<()> {
        tracing::debug!("📁 处理目录: {} -> {}", local_dir.display(), remote_dir);

        // 📁 确保远程目录存在
        self.ensure_remote_directory(remote_dir).await?;

        // 🔍 读取本地目录
        let mut entries = fs::read_dir(local_dir).await.map_err(|e| {
            CcrError::SyncError(format!("读取目录失败 {}: {}", local_dir.display(), e))
        })?;

        let mut file_count = 0;
        let mut dir_count = 0;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| CcrError::SyncError(format!("读取目录项失败: {}", e)))?
        {
            let path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // 🚫 跳过需要排除的文件和目录
            if should_exclude_from_sync(&file_name_str) {
                tracing::debug!("⏭️  跳过: {}", file_name_str);
                continue;
            }

            // 🎯 如果指定了允许的路径，检查是否在列表中
            if let Some(allowed) = allowed_paths {
                let relative_path = path
                    .strip_prefix(local_dir)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| file_name_str.to_string());

                // 检查是否在允许的路径中
                let is_allowed = allowed.iter().any(|allowed_path| {
                    // 支持精确匹配和前缀匹配
                    relative_path == *allowed_path
                        || relative_path.starts_with(&format!("{}/", allowed_path))
                });

                if !is_allowed {
                    tracing::debug!("⏭️  路径 {} 不在允许列表中，跳过", relative_path);
                    continue;
                }
            }

            // 构建远程路径
            let remote_item_path =
                format!("{}/{}", remote_dir.trim_end_matches('/'), file_name_str);

            if path.is_dir() {
                // 📂 递归处理子目录
                dir_count += 1;
                // 🔧 使用 Box::pin 来处理递归 async 调用
                Box::pin(self.push_directory_filtered(&path, &remote_item_path, allowed_paths))
                    .await?;
            } else {
                // 📄 上传文件
                file_count += 1;
                self.push_file(&path, &remote_item_path).await?;
            }
        }

        tracing::info!(
            "✅ 目录已上传: {} ({} 文件, {} 子目录)",
            remote_dir,
            file_count,
            dir_count
        );
        Ok(())
    }

    /// 🔽 从 WebDAV 下载配置文件或目录
    ///
    /// # 参数
    /// - local_path: 本地保存路径
    ///
    /// # 返回
    /// - Ok(()): 下载成功
    /// - Err: 下载失败（如文件不存在）
    pub async fn pull(&self, local_path: &Path) -> Result<()> {
        // 🔍 检查远程是文件还是目录
        // 通过尝试GET请求来判断
        let is_dir = self.remote_path.ends_with('/');

        if is_dir {
            tracing::info!(
                "🔽 从 WebDAV 下载目录: {} -> {}",
                self.remote_path,
                local_path.display()
            );
            self.pull_directory(&self.remote_path, local_path).await
        } else {
            tracing::info!(
                "🔽 从 WebDAV 下载文件: {} -> {}",
                self.remote_path,
                local_path.display()
            );
            self.pull_file(&self.remote_path, local_path).await
        }
    }

    /// 🔽 从 WebDAV 下载单个文件
    async fn pull_file(&self, remote_path: &str, local_path: &Path) -> Result<()> {
        // 🔽 下载文件
        let response = self
            .client
            .get(remote_path)
            .await
            .map_err(|e| self.map_dav_error(e, &format!("下载文件 {}", remote_path)))?;

        // 📄 读取响应内容
        let content = response
            .bytes()
            .await
            .map_err(|e| CcrError::SyncError(format!("读取响应内容失败: {}", e)))?;

        // 📁 确保本地目录存在
        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                CcrError::SyncError(format!("创建本地目录失败 {}: {}", parent.display(), e))
            })?;
        }

        // 💾 保存到本地
        fs::write(local_path, content).await.map_err(|e| {
            CcrError::SyncError(format!(
                "保存文件到本地失败 {}: {}",
                local_path.display(),
                e
            ))
        })?;

        tracing::debug!("✅ 文件已下载: {}", local_path.display());
        Ok(())
    }

    /// 🔽 递归从 WebDAV 下载目录
    async fn pull_directory(&self, remote_dir: &str, local_dir: &Path) -> Result<()> {
        tracing::debug!("📁 处理目录: {} -> {}", remote_dir, local_dir.display());

        // 📁 确保本地目录存在
        fs::create_dir_all(local_dir).await.map_err(|e| {
            CcrError::SyncError(format!("创建本地目录失败 {}: {}", local_dir.display(), e))
        })?;

        // 🔍 列出远程目录内容
        let entities = self
            .client
            .list(remote_dir, Depth::Number(1))
            .await
            .map_err(|e| self.map_dav_error(e, &format!("列出远程目录 {}", remote_dir)))?;

        let mut file_count = 0;
        let mut dir_count = 0;

        for entity in entities {
            match entity {
                ListEntity::File(file) => {
                    // 📄 下载文件
                    // 从 href 中提取文件名
                    let file_name = extract_filename(&file.href);

                    // 🚫 跳过需要排除的文件
                    if should_exclude_from_sync(&file_name) {
                        tracing::debug!("⏭️  跳过文件: {}", file_name);
                        continue;
                    }

                    let local_file_path = local_dir.join(&file_name);

                    file_count += 1;
                    self.pull_file(&file.href, &local_file_path).await?;
                }
                ListEntity::Folder(folder) => {
                    // 📂 递归处理子目录
                    // 从 href 中提取目录名
                    let folder_name = extract_filename(&folder.href);

                    // 🚫 跳过需要排除的目录
                    if should_exclude_from_sync(&folder_name) {
                        tracing::debug!("⏭️  跳过目录: {}", folder_name);
                        continue;
                    }

                    // 跳过当前目录自身（href 可能等于 remote_dir）
                    if folder.href.trim_end_matches('/') == remote_dir.trim_end_matches('/') {
                        continue;
                    }

                    let local_sub_dir = local_dir.join(&folder_name);

                    dir_count += 1;
                    // 🔧 使用 Box::pin 来处理递归 async 调用
                    Box::pin(self.pull_directory(&folder.href, &local_sub_dir)).await?;
                }
            }
        }

        tracing::info!(
            "✅ 目录已下载: {} ({} 文件, {} 子目录)",
            local_dir.display(),
            file_count,
            dir_count
        );
        Ok(())
    }

    /// 🔍 检查远程内容是否存在
    ///
    /// - 对于目录路径（以 `/` 结尾），使用 WebDAV `LIST/PROPFIND` 检查目录是否存在
    /// - 对于文件路径，仍然使用 `GET` 检查文件是否存在
    pub async fn remote_exists(&self) -> Result<bool> {
        let remote_path = self.remote_path.as_str();

        if remote_path.ends_with('/') {
            tracing::debug!("🔍 检查远程目录: {}", remote_path);

            // 使用 PROPFIND/LIST 检查目录是否存在
            match self.client.list(remote_path, Depth::Number(0)).await {
                // 只要能成功列出(即使为空)，就认为目录存在
                Ok(_) => Ok(true),
                // 目录不存在（404）
                Err(DavError::Reqwest(e)) if e.status() == Some(StatusCode::NOT_FOUND) => {
                    tracing::debug!("远程目录不存在 (404): {}", remote_path);
                    Ok(false)
                }
                // 父目录不存在或服务器返回 409/解析错误
                Err(DavError::Decode(_)) => {
                    tracing::debug!("远程目录不存在或不可达（Decode/409）: {}", remote_path);
                    Ok(false)
                }
                Err(e) => Err(self.map_dav_error(e, "检查远程目录")),
            }
        } else {
            tracing::debug!("🔍 检查远程文件: {}", remote_path);

            match self.client.get(remote_path).await {
                Ok(_) => Ok(true),
                // 文件不存在（404）
                Err(DavError::Reqwest(e)) if e.status() == Some(StatusCode::NOT_FOUND) => {
                    tracing::debug!("远程文件不存在 (404): {}", remote_path);
                    Ok(false)
                }
                // 父目录不存在（409 - Conflict）或其他 Decode 错误
                // 坚果云在父目录不存在时返回 409 + AncestorsNotFound
                Err(DavError::Decode(_)) => {
                    tracing::debug!("远程目录或文件不存在（Decode/409）: {}", remote_path);
                    Ok(false)
                }
                Err(e) => Err(self.map_dav_error(e, "检查远程文件")),
            }
        }
    }

    /// 📁 确保远程目录存在（针对文件路径）
    ///
    /// 自动创建远程文件路径中的父目录
    async fn ensure_remote_dir_for_file(&self, file_path: &str) -> Result<()> {
        // 🔍 提取目录路径
        let dir_path = Path::new(file_path)
            .parent()
            .and_then(|p| p.to_str())
            .unwrap_or("/");

        if dir_path == "/" || dir_path.is_empty() {
            return Ok(());
        }

        self.ensure_remote_directory(dir_path).await
    }

    /// 📁 确保远程目录存在（递归创建）
    ///
    /// 自动创建远程目录路径中的所有父目录
    async fn ensure_remote_directory(&self, dir_path: &str) -> Result<()> {
        if dir_path == "/" || dir_path.is_empty() {
            return Ok(());
        }

        tracing::debug!("📁 确保远程目录存在: {}", dir_path);

        // 🔍 尝试创建目录
        match self.client.mkcol(dir_path).await {
            Ok(_) => {
                tracing::debug!("✅ 远程目录已创建: {}", dir_path);
                Ok(())
            }
            Err(DavError::Reqwest(e)) if e.status() == Some(StatusCode::METHOD_NOT_ALLOWED) => {
                // 目录已存在，这不是错误
                tracing::debug!("ℹ️  远程目录已存在: {}", dir_path);
                Ok(())
            }
            Err(DavError::Reqwest(e)) if e.status() == Some(StatusCode::CONFLICT) => {
                // 父目录不存在，递归创建
                tracing::debug!("⚠️  父目录不存在，递归创建: {}", dir_path);

                // 获取父目录路径
                if let Some(parent) = Path::new(dir_path).parent().and_then(|p| p.to_str())
                    && parent != "/"
                    && !parent.is_empty()
                {
                    // 🔧 使用 Box::pin 来处理递归 async 调用
                    Box::pin(self.ensure_remote_directory(parent)).await?;
                    // 再次尝试创建当前目录
                    return Box::pin(self.ensure_remote_directory(dir_path)).await;
                }

                Err(self.map_dav_error(DavError::Reqwest(e), &format!("创建远程目录 {}", dir_path)))
            }
            Err(e) => Err(self.map_dav_error(e, &format!("创建远程目录 {}", dir_path))),
        }
    }

    /// 🔄 映射 WebDAV 错误到 CCR 错误
    fn map_dav_error(&self, err: DavError, operation: &str) -> CcrError {
        let msg = match &err {
            DavError::Reqwest(e) => {
                if let Some(status) = e.status() {
                    match status {
                        StatusCode::UNAUTHORIZED => "认证失败：用户名或密码错误".to_string(),
                        StatusCode::NOT_FOUND => {
                            format!("文件不存在: {}", self.remote_path)
                        }
                        StatusCode::FORBIDDEN => "权限不足：无法访问该资源".to_string(),
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

/// 获取 CCR 同步根路径（供 CLI/Web 共用）
///
/// 优先级：
/// 1. CCR_ROOT 环境变量
/// 2. ~/.ccr/
pub fn get_ccr_sync_path() -> Result<PathBuf> {
    // 1. 检查 CCR_ROOT 环境变量
    if let Ok(ccr_root) = std::env::var("CCR_ROOT") {
        let root_path = PathBuf::from(ccr_root);
        if root_path.exists() {
            return Ok(root_path);
        }
    }

    // 2. 检查 ~/.ccr/ 统一模式目录
    if let Some(home) = dirs::home_dir() {
        let ccr_root = home.join(".ccr");
        if ccr_root.exists() {
            return Ok(ccr_root);
        }
    }

    // 不再支持 Legacy ~/.ccs_config.toml 作为同步根
    Err(CcrError::ConfigError(
        "未找到 CCR 根目录，请先运行 'ccr init' 创建 ~/.ccr".into(),
    ))
}

/// 📝 从 WebDAV href 中提取文件名或目录名
///
/// WebDAV 的 href 通常是完整的路径，如 "/ccr/config.toml" 或 "/ccr/platforms/"
/// 这个函数提取最后一个路径段作为文件名或目录名
fn extract_filename(href: &str) -> String {
    // 移除末尾的 /
    let trimmed = href.trim_end_matches('/');

    // 分割路径并获取最后一段
    trimmed.rsplit('/').next().unwrap_or(trimmed).to_string()
}

/// 🚫 判断文件或目录是否应该从同步中排除
///
/// 排除规则:
/// - 临时文件和备份文件 (.bak, .tmp, .lock)
/// - 系统文件 (.DS_Store, Thumbs.db)
/// - 版本控制目录 (.git)
/// - CCR 内部目录 (.locks)
/// - 备份目录 (backups, history 中的 .bak 文件)
/// - 隐藏文件 (以 . 开头)
fn should_exclude_from_sync(name: &str) -> bool {
    // 📝 排除规则列表
    let exclude_patterns = [
        // 临时文件
        ".tmp",
        ".lock",
        ".bak",
        // 系统文件
        ".DS_Store",
        "Thumbs.db",
        "desktop.ini",
        // 版本控制
        ".git",
        ".gitignore",
        // CCR 内部
        ".locks",
    ];

    // 📝 排除目录列表
    let exclude_dirs = [
        ".locks",  // 锁文件目录
        "backups", // 备份目录（太多文件）
        "history", // 历史记录目录（不需要同步）
        "ccr-ui",  // ccr-ui 应用目录（包含源码和编译产物，不需要同步）
    ];

    // 检查文件扩展名或完整名称
    for pattern in &exclude_patterns {
        if name.ends_with(pattern) || name == *pattern {
            return true;
        }
    }

    // 检查目录名称
    for dir in &exclude_dirs {
        if name == *dir {
            return true;
        }
    }

    // 排除隐藏文件（以 . 开头但不是当前/父目录）
    if name.starts_with('.') && name != "." && name != ".." {
        // 但保留 .ccr 目录本身的配置文件
        if name == ".ccs_config.toml" || name.ends_with(".toml") {
            return false;
        }
        return true;
    }

    false
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn create_test_config() -> SyncConfig {
        SyncConfig {
            enabled: true,
            webdav_url: "https://dav.jianguoyun.com/dav/".to_string(),
            username: "test@example.com".to_string(),
            password: "test_password".to_string(),
            remote_path: "/ccr/".to_string(), // 🆕 改为目录路径
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

    #[test]
    fn test_should_exclude() {
        // 测试文件过滤规则 - 应该被排除的
        assert!(should_exclude_from_sync(".DS_Store"));
        assert!(should_exclude_from_sync("test.tmp"));
        assert!(should_exclude_from_sync("file.lock"));
        assert!(should_exclude_from_sync("backup.bak"));
        assert!(should_exclude_from_sync(".locks"));
        assert!(should_exclude_from_sync("backups"));
        assert!(should_exclude_from_sync("history")); // 🆕 历史记录目录应该被排除
        assert!(should_exclude_from_sync("ccr-ui")); // 🆕 ccr-ui 应用目录应该被排除

        // 不应该被排除的 - 配置文件
        assert!(!should_exclude_from_sync("config.toml"));
        assert!(!should_exclude_from_sync(".ccs_config.toml"));
        assert!(!should_exclude_from_sync("profiles.toml"));
        assert!(!should_exclude_from_sync("platforms")); // platforms 目录应该同步
    }

    #[test]
    fn test_extract_filename() {
        // 测试从 href 提取文件名
        assert_eq!(extract_filename("/ccr/config.toml"), "config.toml");
        assert_eq!(extract_filename("/ccr/platforms/claude/"), "claude");
        assert_eq!(extract_filename("/ccr/platforms/"), "platforms");
        assert_eq!(extract_filename("/test.txt"), "test.txt");
        assert_eq!(extract_filename("config.toml"), "config.toml");

        // 边界情况
        assert_eq!(extract_filename("/"), "");
        assert_eq!(extract_filename(""), "");
    }
}
