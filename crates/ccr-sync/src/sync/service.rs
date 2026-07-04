// ☁️ CCR WebDAV 同步服务
// 📁 负责配置文件的云端同步
//
// 核心功能:
// - 🔼 上传配置到 WebDAV 服务器
// - 🔽 从 WebDAV 服务器下载配置
// - 🔄 智能同步(基于时间戳)
// - ✅ 连接测试

use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::guarded_write::{WriteOptions, write_guarded_async};
use reqwest_dav::list_cmd::ListEntity;
use reqwest_dav::re_exports::reqwest::StatusCode;
use reqwest_dav::{Auth, Client, ClientBuilder, DecodeError, Depth, Error as DavError};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::sync::config::SyncConfig;

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

        // 💾 原子保存到本地（guarded write：锁 + temp → fsync → 原子替换，防半成品下载）
        write_guarded_async(local_path, content.to_vec(), WriteOptions::default())
            .await
            .map_err(|e| {
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
    ///
    /// 存在性判定改由 HTTP 状态码三态分类（[`classify_dav_outcome`]）决定：
    /// 资源存在 ⇒ `Ok(true)`；真正 404 ⇒ `Ok(false)`；无法判定 ⇒ `Err`。
    pub async fn remote_exists(&self) -> Result<bool> {
        let remote_path = self.remote_path.as_str();

        if remote_path.ends_with('/') {
            tracing::debug!("🔍 检查远程目录: {}", remote_path);

            // 使用 PROPFIND/LIST 检查目录是否存在
            let outcome = self
                .client
                .list(remote_path, Depth::Number(0))
                .await
                .map(|_| ());
            self.probe_to_result(outcome)
        } else {
            tracing::debug!("🔍 检查远程文件: {}", remote_path);

            let outcome = self.client.get(remote_path).await.map(|_| ());
            self.probe_to_result(outcome)
        }
    }

    /// 把一次探测结果（[`classify_dav_outcome`] 的三态分类）映射为 `Result<bool>`：
    /// - `Exists` → `Ok(true)`
    /// - `Absent` → `Ok(false)`
    /// - `Undetermined` → `Err(self.map_dav_error(err, "检查远程内容"))`
    fn probe_to_result(&self, outcome: std::result::Result<(), DavError>) -> Result<bool> {
        match classify_dav_outcome(&outcome) {
            RemoteProbe::Exists => Ok(true),
            RemoteProbe::Absent => Ok(false),
            // `Undetermined` 仅在错误侧产生；映射为有意义的错误（UI 显示「未测试」）。
            // `Ok` 不会被分类为 `Undetermined`，退化为存在以避免任何 panic。
            RemoteProbe::Undetermined => match outcome {
                Err(err) => Err(self.map_dav_error(err, "检查远程内容")),
                Ok(()) => Ok(true),
            },
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

/// 🔭 一次远端存在性探测的三态判定结果。
///
/// 把 `reqwest_dav` 的成功/错误结果分类为「存在 / 缺失 / 无法判定」，
/// 由 [`SyncService::remote_exists`] 映射为 `Ok(true)` / `Ok(false)` / `Err`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RemoteProbe {
    /// 资源存在 ⇒ `Ok(true)`
    Exists,
    /// 资源真正缺失（HTTP 404）⇒ `Ok(false)`
    Absent,
    /// 检查无法完成（401/403/409/5xx、无状态码的传输/认证错误）⇒ `Err`
    Undetermined,
}

/// 从 `DavError` 抽取真实 HTTP 状态码（依据 `reqwest_dav 0.3.3` 错误模型）。
///
/// - `Decode(StatusMismatched { response_code, .. })` → `Some(response_code)`
/// - `Decode(Server { response_code, .. })` → `Some(response_code)`
/// - `Reqwest(e)` → `e.status().map(|s| s.as_u16())`（PROPFIND/GET 路径下通常为 `None`）
/// - 其他 `Decode` 变体（`SerdeXml`/`FieldNotSupported`/`FieldNotFound`/`DigestAuth`/
///   `NoAuthHeaderInResponse`）及 `ReqwestDecode`/`MissingAuthContext` → `None`
fn dav_status_code(err: &DavError) -> Option<u16> {
    match err {
        DavError::Decode(DecodeError::StatusMismatched(e)) => Some(e.response_code),
        DavError::Decode(DecodeError::Server(e)) => Some(e.response_code),
        DavError::Reqwest(e) => e.status().map(|s| s.as_u16()),
        _ => None,
    }
}

/// 纯函数：把一次探测的 `Result<T, DavError>` 分类为 [`RemoteProbe`]（不发起网络请求）。
///
/// 分类规则（依据 design「Fix Implementation」）：
/// - 调用成功（`Ok`）→ `Exists`
/// - 错误且状态码 == `404` → `Absent`
/// - 错误且状态码 ∈ `2xx`（含 `207`）→ `Exists`
/// - 「成功后解析失败」语义的 `Decode` 变体（`SerdeXml`/`FieldNotSupported`/
///   `FieldNotFound`）→ `Exists`
/// - 其他（`401`/`403`/`409`/`5xx`、无状态码的传输/认证错误）→ `Undetermined`
fn classify_dav_outcome<T>(outcome: &std::result::Result<T, DavError>) -> RemoteProbe {
    match outcome {
        Ok(_) => RemoteProbe::Exists,
        Err(err) => classify_dav_error(err),
    }
}

/// 纯函数：把一个 `DavError` 分类为 [`RemoteProbe`]（错误侧分类，见 [`classify_dav_outcome`]）。
fn classify_dav_error(err: &DavError) -> RemoteProbe {
    if let Some(code) = dav_status_code(err) {
        if code == 404 {
            return RemoteProbe::Absent;
        }
        if (200..=299).contains(&code) {
            return RemoteProbe::Exists;
        }
        return RemoteProbe::Undetermined;
    }

    // 无状态码：区分「成功后解析失败」（资源存在）与「无法判定」。
    match err {
        DavError::Decode(
            DecodeError::SerdeXml(_)
            | DecodeError::FieldNotSupported(_)
            | DecodeError::FieldNotFound(_),
        ) => RemoteProbe::Exists,
        _ => RemoteProbe::Undetermined,
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod remote_exists_bug_exploration {
    //! Bug Condition 探索性属性测试（修复 **之后** 复跑，验证缺陷已修复）。
    //!
    //! 此测试编码 design.md 中 **Property 1** 的「期望行为」（Expected Behavior）。
    //! 任务 1 最初在 **未修复** 语义（所有 `Decode(_)` → `Ok(false)`）上运行以暴露
    //! 反例（必失败，证明缺陷存在）。任务 3.2 起，**同一组属性、场景、生成器与断言**
    //! 改为针对 **真实修复代码**（[`super::classify_dav_outcome`] / [`super::RemoteProbe`]）
    //! 求值，使其通过即真正验证修复有效。
    //!
    //! 预期结果：测试 **PASSES** —— 确认存在性判定改由 HTTP 状态码三态分类决定，
    //! Bug Condition 输入不再被误判为 `Ok(false)`（远端缺失）。
    //!
    //! 依据 design「reqwest_dav 0.3.3 错误模型」构造满足 `isBugCondition` 的输入：
    //! - 目录 207 解析失败：`Decode(SerdeXml(_))`（资源存在，期望 `Ok(true)`）
    //! - 认证失败：`Decode(StatusMismatched { response_code: 401, .. })`（期望 `Err`）
    //! - 冲突：`Decode(Server { response_code: 409, .. })`（期望 `Err`）
    //! - 服务器错误：`Decode(StatusMismatched { response_code: 503, .. })`（期望 `Err`）

    use proptest::prelude::*;
    use reqwest_dav::{DecodeError, Error as DavError, ServerError, StatusMismatchedError};

    /// 期望（修复后）的存在性判定结果。
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Expected {
        /// 资源存在 ⇒ `Ok(true)`
        Exists,
        /// 检查无法完成 ⇒ `Err`
        Undetermined,
    }

    /// 满足 `isBugCondition` 的输入场景：既非真正 404，又被原实现误报为 `Ok(false)`。
    #[derive(Debug, Clone)]
    enum BugScenario {
        /// 目录 207 解析失败：`list()` 在 `is_success()` 之后 serde-xml-rs 解析失败，
        /// 资源实际存在。
        DirListXmlParseFailure,
        /// 携带非 404、非 2xx 状态码的失败（401/403/409/5xx）：检查无法完成。
        UndeterminedStatus { code: u16, as_server: bool },
    }

    impl BugScenario {
        /// 构造 `reqwest_dav 0.3.3` 在该场景下 **真实会产生** 的错误「结果」。
        fn to_outcome(&self) -> Result<(), DavError> {
            match self {
                BugScenario::DirListXmlParseFailure => {
                    // list() 成功(207) 之后 serde-xml-rs 解析失败 → Decode(SerdeXml)
                    let err = reqwest_dav::re_exports::serde_xml_rs::from_str::<i64>(
                        "this is not valid xml <<<",
                    )
                    .expect_err("malformed XML must fail to deserialize");
                    Err(DavError::Decode(DecodeError::SerdeXml(err)))
                }
                BugScenario::UndeterminedStatus { code, as_server } => {
                    if *as_server {
                        // get() → dav2xx() 非 2xx → Decode(Server { response_code })
                        Err(DavError::Decode(DecodeError::Server(ServerError {
                            response_code: *code,
                            exception: "exception".to_owned(),
                            message: "message".to_owned(),
                        })))
                    } else {
                        // list() → list_rsp() 非成功 → Decode(StatusMismatched { response_code })
                        Err(DavError::Decode(DecodeError::StatusMismatched(
                            StatusMismatchedError {
                                response_code: *code,
                                expected_code: 207,
                            },
                        )))
                    }
                }
            }
        }

        /// design「Property 1」中该场景的期望（修复后）行为。
        fn expected(&self) -> Expected {
            match self {
                BugScenario::DirListXmlParseFailure => Expected::Exists,
                BugScenario::UndeterminedStatus { .. } => Expected::Undetermined,
            }
        }
    }

    /// 经由 **真实修复代码**（[`super::classify_dav_outcome`] / [`super::RemoteProbe`]）
    /// 求值的存在性判定，映射为与断言一致的 `Result<bool, String>` 形状：
    /// - `RemoteProbe::Exists` → `Ok(true)`
    /// - `RemoteProbe::Absent` → `Ok(false)`
    /// - `RemoteProbe::Undetermined` → `Err(..)`（UI 显示「未测试」）
    ///
    /// 这与 [`super::SyncService::probe_to_result`] 的三态映射一致，因此本探索性
    /// 属性测试直接验证修复后实际走的代码路径。
    fn remote_exists_via_fix(outcome: Result<(), DavError>) -> Result<bool, String> {
        match super::classify_dav_outcome(&outcome) {
            super::RemoteProbe::Exists => Ok(true),
            super::RemoteProbe::Absent => Ok(false),
            super::RemoteProbe::Undetermined => match outcome {
                Err(err) => Err(format!("{:?}", err)),
                Ok(()) => Ok(true),
            },
        }
    }

    /// 生成满足 Bug Condition 的输入。
    ///
    /// Scoped PBT：对确定性的错误变体收敛到具体失败用例（非 404、非 2xx 的
    /// 401/403/409 + 5xx），以保证反例可复现。
    fn bug_scenario_strategy() -> impl Strategy<Value = BugScenario> {
        let undetermined_codes =
            prop_oneof![Just(401u16), Just(403u16), Just(409u16), 500u16..=599u16];
        prop_oneof![
            Just(BugScenario::DirListXmlParseFailure),
            (undetermined_codes, any::<bool>())
                .prop_map(|(code, as_server)| BugScenario::UndeterminedStatus { code, as_server }),
        ]
    }

    proptest! {
        /// **Property 1 (Bug Condition)**：存在性判定改由 HTTP 状态码决定。
        ///
        /// 对任何满足 `isBugCondition` 的输入，修复后的实现应满足：
        /// - 资源存在 ⇒ `Ok(true)`
        /// - 检查无法完成 ⇒ `Err`
        /// - **任何情况下都不得返回 `Ok(false)`**
        ///
        /// 经由 **真实修复代码**（`classify_dav_outcome` / `RemoteProbe`）求值
        /// ⇒ 预期 **PASSES**（证明缺陷已修复）。
        ///
        /// **Validates: Requirements 1.1, 1.2, 1.3, 2.1, 2.2, 2.3**
        #[test]
        fn prop_bug_condition_existence_by_status_code(scenario in bug_scenario_strategy()) {
            let expected = scenario.expected();
            let actual = remote_exists_via_fix(scenario.to_outcome());
            let actual_ok = actual.as_ref().ok().copied();

            // 缺陷核心断言：满足 Bug Condition 的输入绝不应被判为「远端缺失」。
            prop_assert_ne!(
                actual_ok,
                Some(false),
                "Bug Condition 输入被误判为 Ok(false)（远端缺失）: {:?}",
                scenario
            );

            match expected {
                Expected::Exists => prop_assert_eq!(
                    actual_ok,
                    Some(true),
                    "资源存在应判为 Ok(true): {:?}",
                    scenario
                ),
                Expected::Undetermined => prop_assert!(
                    actual.is_err(),
                    "检查无法完成应返回 Err: {:?}",
                    scenario
                ),
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod remote_exists_preservation {
    //! Preservation 属性测试。
    //!
    //! 遵循 observation-first（先观察）方法论：先在 **未修复** 语义上观察
    //! 「非 Bug 输入」(`isBugCondition` 为假) 的实际输出，再把这些必须保持
    //! 不变的不变式编码为属性，作为修复的回归基线（design「Preservation
    //! Requirements」与「Property-Based Tests」）。
    //!
    //! 任务 3.3 起，**同一组不变式、场景、生成器与断言**改为针对 **真实修复
    //! 代码**（[`super::classify_dav_outcome`] / [`super::RemoteProbe`] /
    //! [`super::dav_status_code`]）求值，使其通过即真正验证 **修复保留了基线
    //! 行为**（与任务 3.2 对 bug-exploration 模块的重定向一致）。这些
    //! Preservation 不变式（真正 404 → `Ok(false)`、可完全反序列化的成功响应
    //! → `Ok(true)`）在原实现与修复后实现下完全一致，因此重定向后断言含义不变。
    //!
    //! 非 Bug 输入 = ¬isBugCondition：
    //! - 真正 404：`Decode(StatusMismatched/Server { response_code: 404 })`
    //!   → `Ok(false)`（远端缺失）。
    //! - 可完全反序列化的成功响应：`Ok(..)`（含可解析的 207）
    //!   → `Ok(true)`（远端就绪）。
    //!
    //! 预期结果：测试 **PASS** —— 确认修复无回归，基线行为保持不变。
    //!
    //! 关于 design「Property 3」的 P3 关联（「判为缺失 ⟺ s == 404」）：
    //! 该双条件描述的是分类语义。它现在针对 **真实** `classify_dav_outcome`
    //! 在状态码域 `s ∈ 100..=599` 上求值（见 `prop_p3_expected_absent_iff_404`），
    //! 断言真实代码恰好仅在 `s == 404` 时判为 `Absent`。

    use proptest::prelude::*;
    use reqwest_dav::{DecodeError, Error as DavError, ServerError, StatusMismatchedError};

    /// 构造 `reqwest_dav 0.3.3` 在「携带 HTTP 状态码的失败」下 **真实会产生**
    /// 的错误「结果」（依据 design「reqwest_dav 0.3.3 错误模型」）。
    ///
    /// - `as_server == true`：`get()` → `dav2xx()` 非 2xx → `Decode(Server { response_code })`
    /// - `as_server == false`：`list()` → `list_rsp()` 非成功 → `Decode(StatusMismatched { response_code })`
    fn make_status_outcome(code: u16, as_server: bool) -> Result<(), DavError> {
        if as_server {
            Err(DavError::Decode(DecodeError::Server(ServerError {
                response_code: code,
                exception: "exception".to_owned(),
                message: "message".to_owned(),
            })))
        } else {
            Err(DavError::Decode(DecodeError::StatusMismatched(
                StatusMismatchedError {
                    response_code: code,
                    expected_code: 207,
                },
            )))
        }
    }

    /// 经由 **真实修复代码**（[`super::classify_dav_outcome`] / [`super::RemoteProbe`]）
    /// 求值的存在性判定，映射为与断言一致的 `Result<bool, String>` 形状：
    /// - `RemoteProbe::Exists` → `Ok(true)`
    /// - `RemoteProbe::Absent` → `Ok(false)`
    /// - `RemoteProbe::Undetermined` → `Err(..)`（UI 显示「未测试」）
    ///
    /// 这与 [`super::SyncService::probe_to_result`] 的三态映射一致，因此本
    /// Preservation 属性测试直接验证修复后实际走的代码路径仍保留基线行为。
    fn remote_exists_via_fix(outcome: Result<(), DavError>) -> Result<bool, String> {
        match super::classify_dav_outcome(&outcome) {
            super::RemoteProbe::Exists => Ok(true),
            super::RemoteProbe::Absent => Ok(false),
            super::RemoteProbe::Undetermined => match outcome {
                Err(err) => Err(format!("{:?}", err)),
                Ok(()) => Ok(true),
            },
        }
    }

    #[test]
    fn preservation_success_response_is_exists() {
        // 非 Bug 输入：可完全反序列化的成功响应（含可解析的 207）。
        // 经由真实修复代码求值 → Ok(true)。这是修复必须保持的基线（需求 3.1、3.5）。
        let outcome: Result<(), DavError> = Ok(());
        assert_eq!(remote_exists_via_fix(outcome).ok(), Some(true));
    }

    proptest! {
        /// **Property 2 (Preservation)**：真正 404 仍判为「远端缺失」。
        ///
        /// 非 Bug 输入：真正 404（藏在 `Decode(StatusMismatched/Server { 404 })`）。
        /// 经由真实修复代码求值 → `Ok(false)`。这是修复必须保持的基线。
        ///
        /// 针对 **真实修复代码** 求值 ⇒ 预期 **PASS**。
        ///
        /// **Validates: Requirements 3.2**
        #[test]
        fn prop_preservation_real_404_is_absent(as_server in any::<bool>()) {
            let outcome = make_status_outcome(404, as_server);
            prop_assert_eq!(
                remote_exists_via_fix(outcome).ok(),
                Some(false),
                "真正 404 必须保持为 Ok(false)（远端缺失）, as_server={}",
                as_server
            );
        }

        /// **P3 关联（分类语义）**：对状态码域 `s ∈ 100..=599` 生成
        /// `StatusMismatched{s}` 与 `Server{s}`，断言「判为缺失 ⟺ s == 404」。
        ///
        /// 针对 **真实** `super::dav_status_code` / `super::classify_dav_outcome`
        /// 求值：验证状态码被正确抽取，且真实分类「判为 `Absent`（→`Ok(false)`）」
        /// 当且仅当抽取到的状态码恰为 404。
        ///
        /// **Validates: Requirements 3.2 (P3 关联)**
        #[test]
        fn prop_p3_expected_absent_iff_404(code in 100u16..=599, as_server in any::<bool>()) {
            let outcome = make_status_outcome(code, as_server);
            let err = outcome.expect_err("携带状态码的探测结果应为错误");

            // 状态码应被真实 dav_status_code 正确抽取（404 必须能从 Decode 变体中读出）。
            let extracted = super::dav_status_code(&err);
            prop_assert_eq!(
                extracted,
                Some(code),
                "应从 Decode 变体抽取真实状态码, code={}, as_server={}",
                code,
                as_server
            );

            // P3：真实分类判为缺失（Absent）⟺ 抽取到的状态码 == 404。
            let actual_absent = super::classify_dav_outcome::<()>(&Err(err))
                == super::RemoteProbe::Absent;
            prop_assert_eq!(
                actual_absent,
                code == 404,
                "真实分类应满足「判为缺失 ⟺ s == 404」, code={}",
                code
            );
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod classify_outcome_unit {
    //! Task 4 单元/回归测试：对 [`super::classify_dav_outcome`] 与
    //! [`super::dav_status_code`] 做逐变体的示例断言（design「Unit Tests」）。
    //!
    //! 这些示例单测补充（而非重复）`remote_exists_preservation` 中的属性测试：
    //! 属性测试在状态码域上自动覆盖「判为缺失 ⟺ s == 404」，这里则对 design
    //! 明确枚举的具体变体（`Ok`、`StatusMismatched{404}`/`Server{404}`、
    //! `401/403/409/500/503`、`SerdeXml`/`FieldNotFound`/`FieldNotSupported`）
    //! 做可读的、可复现的点状断言。

    use reqwest_dav::{
        DecodeError, Error as DavError, FieldError, ServerError, StatusMismatchedError,
    };

    /// `list()` → `list_rsp()` 非成功 → `Decode(StatusMismatched { response_code })`。
    fn status_mismatched(code: u16) -> DavError {
        DavError::Decode(DecodeError::StatusMismatched(StatusMismatchedError {
            response_code: code,
            expected_code: 207,
        }))
    }

    /// `get()` → `dav2xx()` 非 2xx → `Decode(Server { response_code })`。
    fn server(code: u16) -> DavError {
        DavError::Decode(DecodeError::Server(ServerError {
            response_code: code,
            exception: "exception".to_owned(),
            message: "message".to_owned(),
        }))
    }

    /// `list()` 成功(207) 之后 serde-xml-rs 解析失败 → `Decode(SerdeXml)`（资源存在）。
    fn serde_xml() -> DavError {
        let err = reqwest_dav::re_exports::serde_xml_rs::from_str::<i64>("not valid xml <<<")
            .expect_err("malformed XML must fail to deserialize");
        DavError::Decode(DecodeError::SerdeXml(err))
    }

    /// 条目转换失败 → `Decode(FieldNotFound)`（HTTP 成功之后，资源存在）。
    fn field_not_found() -> DavError {
        DavError::Decode(DecodeError::FieldNotFound(FieldError {
            field: "d:href".to_owned(),
        }))
    }

    /// 条目转换失败 → `Decode(FieldNotSupported)`（HTTP 成功之后，资源存在）。
    fn field_not_supported() -> DavError {
        DavError::Decode(DecodeError::FieldNotSupported(FieldError {
            field: "d:resourcetype".to_owned(),
        }))
    }

    // ── classify_dav_outcome ──

    #[test]
    fn ok_outcome_is_exists() {
        let outcome: Result<(), DavError> = Ok(());
        assert_eq!(
            super::classify_dav_outcome(&outcome),
            super::RemoteProbe::Exists
        );
    }

    #[test]
    fn status_404_is_absent() {
        assert_eq!(
            super::classify_dav_outcome::<()>(&Err(status_mismatched(404))),
            super::RemoteProbe::Absent
        );
        assert_eq!(
            super::classify_dav_outcome::<()>(&Err(server(404))),
            super::RemoteProbe::Absent
        );
    }

    #[test]
    fn non_404_status_is_undetermined() {
        for code in [401u16, 403, 409, 500, 503] {
            assert_eq!(
                super::classify_dav_outcome::<()>(&Err(status_mismatched(code))),
                super::RemoteProbe::Undetermined,
                "StatusMismatched {code} 应判为 Undetermined"
            );
            assert_eq!(
                super::classify_dav_outcome::<()>(&Err(server(code))),
                super::RemoteProbe::Undetermined,
                "Server {code} 应判为 Undetermined"
            );
        }
    }

    #[test]
    fn parse_failure_variants_are_exists() {
        // 「HTTP 成功之后才会出现」的解析失败语义 ⇒ 资源存在。
        assert_eq!(
            super::classify_dav_outcome::<()>(&Err(serde_xml())),
            super::RemoteProbe::Exists
        );
        assert_eq!(
            super::classify_dav_outcome::<()>(&Err(field_not_found())),
            super::RemoteProbe::Exists
        );
        assert_eq!(
            super::classify_dav_outcome::<()>(&Err(field_not_supported())),
            super::RemoteProbe::Exists
        );
    }

    // ── dav_status_code ──

    #[test]
    fn dav_status_code_extracts_response_code() {
        assert_eq!(super::dav_status_code(&status_mismatched(404)), Some(404));
        assert_eq!(super::dav_status_code(&status_mismatched(503)), Some(503));
        assert_eq!(super::dav_status_code(&server(409)), Some(409));
        assert_eq!(super::dav_status_code(&server(401)), Some(401));
    }

    #[test]
    fn dav_status_code_is_none_for_no_status_variants() {
        assert_eq!(super::dav_status_code(&serde_xml()), None);
        assert_eq!(super::dav_status_code(&field_not_found()), None);
        assert_eq!(super::dav_status_code(&field_not_supported()), None);
    }
}
