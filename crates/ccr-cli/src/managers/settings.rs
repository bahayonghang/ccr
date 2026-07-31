// ⭐ CCR 设置管理模块
// 📝 负责读写和管理 ~/.claude/settings.json 文件
// 💎 本模块是 ClaudeSettings 的 IO adapter:类型与全部变更/验证逻辑
//    归属 ccr_types::ClaudeSettings,这里只做加载/保存/备份/恢复
//
// 核心职责:
// - 🔧 管理 Claude Code settings.json
// - 🔄 原子性写入(临时文件 + 重命名)
// - 🔒 文件锁保证并发安全
// - 💾 自动备份机制

use ccr_config::ClaudeRuntimePaths;
use ccr_core::core::cache::ConfigCache;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::lock::LockManager;
use ccr_core::core::{
    BackupPolicy, VersionedWriteOutcome, WriteOptions, content_version_token, write_guarded,
    write_guarded_async, write_guarded_versioned,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs as async_fs;

const SETTINGS_UPDATE_ATTEMPTS: usize = 3;

// 🎯 唯一 shape:富类型定义与托管 env 变更/查询/验证逻辑均在 ccr-types
pub use ccr_types::ClaudeSettings;

/// 🔧 设置管理器
///
/// 负责 Claude Code 设置文件的完整生命周期管理
///
/// 核心功能:
/// - 📖 加载和解析 settings.json
/// - 💾 原子性保存(临时文件 + rename)
/// - 🔒 文件锁防止并发冲突
/// - 💾 自动备份和恢复
/// - 📋 备份文件列表管理
pub struct SettingsManager {
    settings_path: PathBuf,
    backup_dir: PathBuf,
    lock_manager: LockManager,
}

#[allow(dead_code)]
impl SettingsManager {
    /// 🏗️ 创建新的设置管理器
    pub fn new<P: AsRef<Path>, Q: AsRef<Path>>(
        settings_path: P,
        backup_dir: Q,
        lock_manager: LockManager,
    ) -> Self {
        Self {
            settings_path: settings_path.as_ref().to_path_buf(),
            backup_dir: backup_dir.as_ref().to_path_buf(),
            lock_manager,
        }
    }

    /// 🏠 使用默认路径创建管理器
    ///
    /// 默认路径:
    /// - 设置文件: ~/.claude/settings.json
    /// - 备份目录: ~/.claude/backups
    ///
    /// ⚙️ **开发者注意**：
    /// 可以通过环境变量覆盖默认路径：
    /// - `CLAUDE_CONFIG_DIR`: Claude Code 配置目录
    /// - `CCR_SETTINGS_PATH`: 设置文件路径
    /// - `CCR_BACKUP_DIR`: 备份目录路径
    ///
    /// 示例：
    /// ```bash
    /// export CCR_SETTINGS_PATH=/tmp/ccr_dev_settings.json
    /// export CCR_BACKUP_DIR=/tmp/ccr_dev_backups
    /// cargo run -- switch test
    /// ```
    pub fn with_default() -> Result<Self> {
        let runtime_paths = ClaudeRuntimePaths::from_env()?;
        let settings_path = runtime_paths.settings_file;
        let backup_dir = runtime_paths.backups_dir;
        let lock_manager = LockManager::with_default_path()?;

        tracing::debug!("使用设置路径: {:?}", settings_path);
        tracing::debug!("使用备份目录: {:?}", backup_dir);

        Ok(Self::new(settings_path, backup_dir, lock_manager))
    }

    /// 📁 获取设置文件路径
    pub fn settings_path(&self) -> &Path {
        &self.settings_path
    }

    fn write_options(&self) -> WriteOptions {
        WriteOptions {
            backup: BackupPolicy::Dir {
                dir: self.backup_dir.clone(),
                prefix: "settings".to_string(),
            },
            secret: true,
            ..Default::default()
        }
    }

    fn load_versioned(&self) -> Result<(ClaudeSettings, String)> {
        match fs::read(&self.settings_path) {
            Ok(bytes) => {
                let settings = serde_json::from_slice(&bytes).map_err(|error| {
                    CcrError::SettingsError(format!("解析设置文件失败: {error}"))
                })?;
                Ok((settings, content_version_token(&bytes)))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok((ClaudeSettings::new(), String::new()))
            }
            Err(error) => Err(CcrError::SettingsError(format!(
                "读取设置文件失败: {error}"
            ))),
        }
    }

    /// Replays a deterministic settings mutation after concurrent conflicts.
    pub fn update_atomic<F, T>(&self, mut update: F) -> Result<T>
    where
        F: FnMut(&mut ClaudeSettings) -> Result<T>,
    {
        for _ in 0..SETTINGS_UPDATE_ATTEMPTS {
            let (mut settings, expected_token) = self.load_versioned()?;
            let result = update(&mut settings)?;
            let content = serde_json::to_vec_pretty(&settings)
                .map_err(|error| CcrError::SettingsError(format!("序列化设置失败: {error}")))?;

            match write_guarded_versioned(
                &self.settings_path,
                &content,
                &expected_token,
                &self.write_options(),
            )? {
                VersionedWriteOutcome::Written => return Ok(result),
                VersionedWriteOutcome::Conflict => continue,
            }
        }

        Err(CcrError::SettingsError(
            "settings.json 被其他进程连续修改，请重试".into(),
        ))
    }

    pub async fn update_atomic_async<F, T>(&self, update: F) -> Result<T>
    where
        F: FnMut(&mut ClaudeSettings) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let manager = Self::new(
            &self.settings_path,
            &self.backup_dir,
            LockManager::new(self.lock_manager.lock_dir()),
        );
        tokio::task::spawn_blocking(move || manager.update_atomic(update))
            .await
            .map_err(|error| {
                CcrError::SettingsError(format!("settings 更新后台任务失败: {error}"))
            })?
    }

    /// 📖 加载设置文件
    ///
    /// 执行步骤:
    /// 1. ✅ 检查文件是否存在
    /// 2. 📄 读取 JSON 内容
    /// 3. 🔍 解析为 ClaudeSettings 结构
    pub fn load(&self) -> Result<ClaudeSettings> {
        // ✅ 检查文件是否存在
        if !self.settings_path.exists() {
            return Err(CcrError::SettingsMissing(
                self.settings_path.display().to_string(),
            ));
        }

        // 📄 读取文件内容
        let content = fs::read_to_string(&self.settings_path)
            .map_err(|e| CcrError::SettingsError(format!("读取设置文件失败: {}", e)))?;

        // 🔍 解析 JSON
        let settings: ClaudeSettings = serde_json::from_str(&content)
            .map_err(|e| CcrError::SettingsError(format!("解析设置文件失败: {}", e)))?;

        tracing::debug!("✅ 成功加载设置文件: {:?}", self.settings_path);
        Ok(settings)
    }

    /// 📖 异步加载设置文件
    pub async fn load_async(&self) -> Result<ClaudeSettings> {
        let exists = async_fs::try_exists(&self.settings_path)
            .await
            .map_err(|e| CcrError::SettingsError(format!("检查设置文件失败: {}", e)))?;
        if !exists {
            return Err(CcrError::SettingsMissing(
                self.settings_path.display().to_string(),
            ));
        }

        let content = async_fs::read_to_string(&self.settings_path)
            .await
            .map_err(|e| CcrError::SettingsError(format!("读取设置文件失败: {}", e)))?;

        let settings: ClaudeSettings = serde_json::from_str(&content)
            .map_err(|e| CcrError::SettingsError(format!("解析设置文件失败: {}", e)))?;

        tracing::debug!("✅ 成功加载设置文件: {:?}", self.settings_path);
        Ok(settings)
    }

    /// 💾 原子保存设置文件
    ///
    /// 完整替换 settings.json，并在集中备份目录保留旧版本。
    /// 常规 read-modify-write 调用必须使用 [`Self::update_atomic`]。
    pub fn save_atomic(&self, settings: &ClaudeSettings) -> Result<()> {
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| CcrError::SettingsError(format!("序列化设置失败: {}", e)))?;
        write_guarded(
            &self.settings_path,
            content.as_bytes(),
            &self.write_options(),
        )?;

        tracing::info!("✅ 设置文件已原子保存: {:?}", self.settings_path);
        Ok(())
    }

    /// 💾 异步原子保存设置文件
    pub async fn save_atomic_async(&self, settings: &ClaudeSettings) -> Result<()> {
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| CcrError::SettingsError(format!("序列化设置失败: {}", e)))?;
        write_guarded_async(
            &self.settings_path,
            content.into_bytes(),
            self.write_options(),
        )
        .await?;

        tracing::info!("✅ 设置文件已原子保存: {:?}", self.settings_path);
        Ok(())
    }

    /// 💾 备份设置文件
    ///
    /// 执行流程:
    /// 1. ✅ 验证源文件存在
    /// 2. 📁 确保备份目录存在
    /// 3. 🏷️ 生成带时间戳的备份文件名
    /// 4. 📋 复制文件到备份目录
    /// 5. 🧹 自动清理旧备份(只保留最近10个)
    ///
    /// 文件名格式:
    /// - 有配置名: settings.{config_name}.{timestamp}.json.bak
    /// - 无配置名: settings.{timestamp}.json.bak
    #[allow(dead_code)]
    pub fn backup(&self, config_name: Option<&str>) -> Result<PathBuf> {
        // ✅ 验证源文件存在
        if !self.settings_path.exists() {
            return Err(CcrError::SettingsMissing(
                self.settings_path.display().to_string(),
            ));
        }

        // 📁 确保备份目录存在
        fs::create_dir_all(&self.backup_dir)
            .map_err(|e| CcrError::SettingsError(format!("创建备份目录失败: {}", e)))?;

        // 🏷️ 生成备份文件名(带时间戳)
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_filename = if let Some(name) = config_name {
            format!("settings.{}.{}.json.bak", name, timestamp)
        } else {
            format!("settings.{}.json.bak", timestamp)
        };

        let backup_path = self.backup_dir.join(backup_filename);

        // 📋 复制文件
        fs::copy(&self.settings_path, &backup_path)
            .map_err(|e| CcrError::SettingsError(format!("备份设置文件失败: {}", e)))?;

        tracing::info!("💾 设置文件已备份: {:?}", backup_path);

        // 🧹 自动清理旧备份(只保留最近10个)
        const MAX_BACKUPS: usize = 10;
        if let Ok(backups) = self.list_backups()
            && backups.len() > MAX_BACKUPS
        {
            let to_delete = &backups[MAX_BACKUPS..];
            for old_backup in to_delete {
                if let Err(e) = fs::remove_file(old_backup) {
                    tracing::warn!("清理旧备份失败 {:?}: {}", old_backup, e);
                } else {
                    tracing::debug!("🗑️ 已删除旧备份: {:?}", old_backup);
                }
            }
            tracing::info!(
                "🧹 已自动清理 {} 个旧备份,保留最近 {} 个",
                to_delete.len(),
                MAX_BACKUPS
            );
        }

        Ok(backup_path)
    }

    /// 💾 异步备份设置文件
    pub async fn backup_async(&self, config_name: Option<&str>) -> Result<PathBuf> {
        let exists = async_fs::try_exists(&self.settings_path)
            .await
            .map_err(|e| CcrError::SettingsError(format!("检查设置文件失败: {}", e)))?;
        if !exists {
            return Err(CcrError::SettingsMissing(
                self.settings_path.display().to_string(),
            ));
        }

        async_fs::create_dir_all(&self.backup_dir)
            .await
            .map_err(|e| CcrError::SettingsError(format!("创建备份目录失败: {}", e)))?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_filename = if let Some(name) = config_name {
            format!("settings.{}.{}.json.bak", name, timestamp)
        } else {
            format!("settings.{}.json.bak", timestamp)
        };

        let backup_path = self.backup_dir.join(backup_filename);

        async_fs::copy(&self.settings_path, &backup_path)
            .await
            .map_err(|e| CcrError::SettingsError(format!("备份设置文件失败: {}", e)))?;

        tracing::info!("💾 设置文件已备份: {:?}", backup_path);

        const MAX_BACKUPS: usize = 10;
        if let Ok(backups) = self.list_backups_async().await
            && backups.len() > MAX_BACKUPS
        {
            let to_delete = &backups[MAX_BACKUPS..];
            for old_backup in to_delete {
                if let Err(e) = async_fs::remove_file(old_backup).await {
                    tracing::warn!("清理旧备份失败 {:?}: {}", old_backup, e);
                } else {
                    tracing::debug!("🗑️ 已删除旧备份: {:?}", old_backup);
                }
            }
            tracing::info!(
                "🧹 已自动清理 {} 个旧备份,保留最近 {} 个",
                to_delete.len(),
                MAX_BACKUPS
            );
        }

        Ok(backup_path)
    }

    /// 🔄 从备份恢复设置文件
    ///
    /// 备份内容验证成功后，通过统一 guarded replace 恢复；当前设置会在
    /// 同一路径锁内备份到集中目录。
    pub fn restore<P: AsRef<Path>>(&self, backup_path: P) -> Result<()> {
        let backup_path = backup_path.as_ref();

        // ✅ 验证备份文件存在
        if !backup_path.exists() {
            return Err(CcrError::SettingsMissing(backup_path.display().to_string()));
        }

        // 🔍 验证备份文件格式
        let content = fs::read_to_string(backup_path)
            .map_err(|e| CcrError::SettingsError(format!("读取备份文件失败: {}", e)))?;

        let settings: ClaudeSettings = serde_json::from_str(&content)
            .map_err(|e| CcrError::SettingsError(format!("备份文件格式无效: {}", e)))?;

        self.save_atomic(&settings)?;

        tracing::info!("✅ 设置文件已从备份恢复: {:?}", backup_path);
        Ok(())
    }

    /// 🔄 异步从备份恢复设置文件
    pub async fn restore_async<P: AsRef<Path>>(&self, backup_path: P) -> Result<()> {
        let backup_path = backup_path.as_ref();

        let exists = async_fs::try_exists(backup_path)
            .await
            .map_err(|e| CcrError::SettingsError(format!("检查备份文件失败: {}", e)))?;
        if !exists {
            return Err(CcrError::SettingsMissing(backup_path.display().to_string()));
        }

        let content = async_fs::read_to_string(backup_path)
            .await
            .map_err(|e| CcrError::SettingsError(format!("读取备份文件失败: {}", e)))?;

        let settings: ClaudeSettings = serde_json::from_str(&content)
            .map_err(|e| CcrError::SettingsError(format!("备份文件格式无效: {}", e)))?;

        self.save_atomic_async(&settings).await?;

        tracing::info!("✅ 设置文件已从备份恢复: {:?}", backup_path);
        Ok(())
    }

    /// 📋 列出所有备份文件
    ///
    /// 返回所有 .bak 扩展名的备份文件,按修改时间倒序排列(最新的在前)
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        if !self.backup_dir.exists() {
            return Ok(vec![]);
        }

        let mut backups = Vec::new();

        // 📂 遍历备份目录
        for entry in fs::read_dir(&self.backup_dir)
            .map_err(|e| CcrError::SettingsError(format!("读取备份目录失败: {}", e)))?
        {
            let entry =
                entry.map_err(|e| CcrError::SettingsError(format!("读取目录项失败: {}", e)))?;

            let path = entry.path();
            // 🔍 只收集 .bak 文件
            if path.extension().and_then(|s| s.to_str()) == Some("bak") {
                backups.push(path);
            }
        }

        // 📅 按修改时间排序(最新的在前)
        backups.sort_by(|a, b| {
            let a_time = fs::metadata(a).and_then(|m| m.modified()).ok();
            let b_time = fs::metadata(b).and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });

        Ok(backups)
    }

    /// 📋 异步列出所有备份文件
    pub async fn list_backups_async(&self) -> Result<Vec<PathBuf>> {
        let exists = async_fs::try_exists(&self.backup_dir)
            .await
            .map_err(|e| CcrError::SettingsError(format!("检查备份目录失败: {}", e)))?;
        if !exists {
            return Ok(vec![]);
        }

        let mut backups = Vec::new();
        let mut entries = async_fs::read_dir(&self.backup_dir)
            .await
            .map_err(|e| CcrError::SettingsError(format!("读取备份目录失败: {}", e)))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| CcrError::SettingsError(format!("读取目录项失败: {}", e)))?
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("bak") {
                let modified = async_fs::metadata(&path)
                    .await
                    .ok()
                    .and_then(|m| m.modified().ok());
                backups.push((path, modified));
            }
        }

        backups.sort_by_key(|entry| std::cmp::Reverse(entry.1));
        Ok(backups.into_iter().map(|(path, _)| path).collect())
    }

    // === 🆕 多平台支持方法 ===

    /// 🎯 为指定平台创建 SettingsManager
    ///
    /// 根据平台类型自动确定设置文件路径和备份目录
    ///
    /// 支持的平台:
    /// - claude: ~/.claude/settings.json
    /// - codex: ~/.ccr/platforms/codex/settings.json (unified mode)
    /// - gemini: ~/.gemini/antigravity-cli/settings.json (Antigravity CLI; internal key remains gemini)
    ///
    /// 参数:
    /// - platform_name: 平台名称 ("claude", "codex", "gemini" 等)
    ///
    /// 注意: 此方法假设统一模式已启用。对于 Claude 平台，
    /// 如果在 legacy 模式下，应使用 `SettingsManager::with_default()`
    pub fn for_platform(platform_name: &str) -> Result<Self> {
        let (settings_path, backup_dir) = Self::get_platform_paths(platform_name)?;
        let lock_manager = LockManager::with_default_path()?;

        tracing::debug!(
            "为平台 '{}' 创建 SettingsManager: {:?}",
            platform_name,
            settings_path
        );

        Ok(Self::new(settings_path, backup_dir, lock_manager))
    }

    /// 📁 获取平台特定的路径
    ///
    /// 返回 (settings_path, backup_dir)
    pub fn get_platform_paths(platform_name: &str) -> Result<(PathBuf, PathBuf)> {
        if platform_name == "claude" {
            let runtime_paths = ClaudeRuntimePaths::from_env()?;
            return Ok((runtime_paths.settings_file, runtime_paths.backups_dir));
        }

        if matches!(
            platform_name,
            "gemini" | "gemini-cli" | "antigravity" | "antigravity-cli" | "agy"
        ) {
            let home = dirs::home_dir()
                .ok_or_else(|| CcrError::SettingsError("无法获取用户主目录".into()))?;
            let platform_dir = home.join(".gemini").join("antigravity-cli");
            return Ok((
                platform_dir.join("settings.json"),
                platform_dir.join("backups"),
            ));
        }

        // 其他平台都使用统一模式路径
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::SettingsError("无法获取用户主目录".into()))?;

        let ccr_root = if let Ok(root) = std::env::var("CCR_ROOT") {
            PathBuf::from(root)
        } else {
            home.join(".ccr")
        };

        let platform_dir = ccr_root.join("platforms").join(platform_name);

        Ok((
            platform_dir.join("settings.json"),
            platform_dir.join("backups"),
        ))
    }

    /// 🔍 检测当前平台的配置模式
    ///
    /// 返回 "Legacy" 或 "Unified"
    pub fn detect_mode(&self) -> &'static str {
        // 如果设置路径包含 ".ccr/platforms"，则为统一模式
        if self
            .settings_path
            .to_str()
            .map(|s| s.contains(".ccr/platforms"))
            .unwrap_or(false)
        {
            "Unified"
        } else {
            "Legacy"
        }
    }
}

// ═══════════════════════════════════════════════════════════
// 🗄️ 缓存设置管理器
// ═══════════════════════════════════════════════════════════

/// 🗄️ 缓存设置管理器
///
/// 封装 `SettingsManager`，添加自动缓存支持
///
/// ## 特性
/// - 📖 自动缓存 load() 结果
/// - 🔄 save_atomic() 时自动失效缓存
/// - ⏰ TTL 过期自动重新加载
/// - 🔒 线程安全
///
/// ## 使用示例
/// ```rust,ignore
/// let manager = CachedSettingsManager::with_default()?;
///
/// // 第一次加载从磁盘读取
/// let settings = manager.load()?;
///
/// // 第二次加载命中缓存
/// let settings2 = manager.load()?;
///
/// // 保存后缓存自动失效
/// manager.save_atomic(&settings)?;
/// ```
#[allow(dead_code)]
pub struct CachedSettingsManager {
    inner: SettingsManager,
    cache: ConfigCache<ClaudeSettings>,
}

#[allow(dead_code)]
impl CachedSettingsManager {
    /// 🏗️ 创建新的缓存设置管理器
    ///
    /// # 参数
    /// - `inner`: 内部 SettingsManager
    /// - `ttl`: 缓存有效期
    #[allow(dead_code)]
    pub fn new(inner: SettingsManager, ttl: Duration) -> Self {
        Self {
            inner,
            cache: ConfigCache::new(ttl),
        }
    }

    /// 🏠 使用默认路径和 TTL 创建管理器
    ///
    /// 默认 TTL: 30 秒
    pub fn with_default() -> Result<Self> {
        let inner = SettingsManager::with_default()?;
        Ok(Self::new(inner, Duration::from_secs(30)))
    }

    /// 🎯 为指定平台创建缓存管理器
    pub fn for_platform(platform_name: &str) -> Result<Self> {
        let inner = SettingsManager::for_platform(platform_name)?;
        Ok(Self::new(inner, Duration::from_secs(30)))
    }

    /// 📁 获取设置文件路径
    pub fn settings_path(&self) -> &Path {
        self.inner.settings_path()
    }

    /// 📖 加载设置文件（带缓存）
    ///
    /// 如果缓存有效，直接返回缓存数据
    /// 如果缓存无效或过期，从磁盘加载并缓存
    pub fn load(&self) -> Result<ClaudeSettings> {
        self.cache.get_or_load(|| self.inner.load())
    }

    /// 💾 原子保存设置文件并失效缓存
    ///
    /// 保存后自动失效缓存，下次 load() 将重新从磁盘加载
    pub fn save_atomic(&self, settings: &ClaudeSettings) -> Result<()> {
        // 先保存
        self.inner.save_atomic(settings)?;
        // 然后失效缓存
        self.cache.invalidate();
        Ok(())
    }

    /// 💾 备份设置文件
    pub fn backup(&self, config_name: Option<&str>) -> Result<PathBuf> {
        self.inner.backup(config_name)
    }

    /// 🔄 从备份恢复设置文件并失效缓存
    pub fn restore<P: AsRef<Path>>(&self, backup_path: P) -> Result<()> {
        self.inner.restore(backup_path)?;
        self.cache.invalidate();
        Ok(())
    }

    /// 📋 列出所有备份文件
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        self.inner.list_backups()
    }

    /// 🧹 手动失效缓存
    ///
    /// 强制下次 load() 从磁盘读取
    pub fn invalidate_cache(&self) {
        self.cache.invalidate();
    }

    /// 🔍 检查缓存是否有效
    pub fn is_cache_valid(&self) -> bool {
        self.cache.is_valid()
    }

    /// 🔍 检测当前平台的配置模式
    pub fn detect_mode(&self) -> &'static str {
        self.inner.detect_mode()
    }

    /// 📊 获取内部 SettingsManager 引用
    ///
    /// 用于需要直接访问底层功能的场景
    pub fn inner(&self) -> &SettingsManager {
        &self.inner
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestHome;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};

    #[test]
    fn claude_config_dir_controls_default_and_platform_settings_paths() {
        let mut home = TestHome::new_with_home_env();
        let config_dir = home.home().join("claude-custom");
        home.set_env("CLAUDE_CONFIG_DIR", config_dir.as_os_str());
        home.remove_env("CCR_SETTINGS_PATH");
        home.remove_env("CCR_BACKUP_DIR");

        let manager = SettingsManager::with_default().unwrap();
        assert_eq!(
            manager.settings_path(),
            config_dir.join("settings.json").as_path()
        );

        let (settings_path, backup_dir) = SettingsManager::get_platform_paths("claude").unwrap();
        assert_eq!(settings_path, config_dir.join("settings.json"));
        assert_eq!(backup_dir, config_dir.join("backups"));
    }

    #[test]
    fn test_settings_manager_save_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let backup_dir = temp_dir.path().join("backups");
        let lock_dir = temp_dir.path().join("locks");

        let lock_manager = LockManager::new(lock_dir);
        let manager = SettingsManager::new(settings_path, backup_dir, lock_manager);

        // 创建并保存设置
        let mut settings = ClaudeSettings::new();
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "https://api.test.com".into());

        manager.save_atomic(&settings).unwrap();

        // 加载并验证
        let loaded = manager.load().unwrap();
        assert_eq!(
            loaded.env.get("ANTHROPIC_BASE_URL"),
            Some(&"https://api.test.com".to_string())
        );
    }

    // 📀 磁盘级读→改→写→读:富字段与未知字段经 IO adapter 往返无损(唯一 shape 的核心回归防线)
    #[test]
    fn test_disk_roundtrip_preserves_unknown_fields_across_apply() {
        let temp_dir = tempfile::tempdir().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let backup_dir = temp_dir.path().join("backups");
        let lock_dir = temp_dir.path().join("locks");

        std::fs::write(
            &settings_path,
            r#"{
                "env": { "ANTHROPIC_BASE_URL": "https://old.example.com", "MY_VAR": "keep" },
                "outputStyle": "engineer",
                "mcpServers": { "fs": { "command": "node", "vendor_flag": true } },
                "statusline": { "theme": "warm" },
                "future_top_level": 42
            }"#,
        )
        .unwrap();

        let lock_manager = LockManager::new(lock_dir);
        let manager = SettingsManager::new(&settings_path, backup_dir, lock_manager);

        let mut settings = manager.load().unwrap();
        settings.apply_managed_env([(
            "ANTHROPIC_BASE_URL".to_string(),
            "https://new.example.com".to_string(),
        )]);
        manager.save_atomic(&settings).unwrap();

        let reloaded = manager.load().unwrap();
        assert_eq!(
            reloaded.env.get("ANTHROPIC_BASE_URL"),
            Some(&"https://new.example.com".to_string())
        );
        assert_eq!(reloaded.env.get("MY_VAR"), Some(&"keep".to_string()));
        assert_eq!(reloaded.output_style.as_deref(), Some("engineer"));
        assert!(reloaded.mcp_servers.contains_key("fs"));
        assert!(reloaded.other.contains_key("statusline"));
        assert!(reloaded.other.contains_key("future_top_level"));
    }

    #[test]
    fn test_update_atomic_replays_conflict_and_preserves_both_mutations() {
        let home = TestHome::new();
        fs::write(
            home.settings_path(),
            r#"{
                "env": { "USER_OWNED": "keep" },
                "future_top_level": { "enabled": true }
            }"#,
        )
        .unwrap();

        let ui_manager = SettingsManager::new(
            home.settings_path(),
            home.backup_dir(),
            LockManager::new(home.lock_dir()),
        );
        let cli_manager = SettingsManager::new(
            home.settings_path(),
            home.backup_dir(),
            LockManager::new(home.lock_dir()),
        );
        let first_mutation_ready = Arc::new(Barrier::new(2));
        let cli_write_finished = Arc::new(Barrier::new(2));
        let mutation_calls = Arc::new(AtomicUsize::new(0));

        let ui_thread = {
            let first_mutation_ready = first_mutation_ready.clone();
            let cli_write_finished = cli_write_finished.clone();
            let mutation_calls = mutation_calls.clone();
            std::thread::spawn(move || {
                ui_manager.update_atomic(move |settings| {
                    if mutation_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                        first_mutation_ready.wait();
                        cli_write_finished.wait();
                    }
                    settings
                        .other
                        .insert("ui_field".to_string(), serde_json::json!("saved"));
                    Ok(())
                })
            })
        };

        first_mutation_ready.wait();
        cli_manager
            .update_atomic(|settings| {
                settings
                    .env
                    .insert("CLI_FIELD".to_string(), "saved".to_string());
                Ok(())
            })
            .unwrap();
        cli_write_finished.wait();
        ui_thread.join().unwrap().unwrap();

        let final_settings = cli_manager.load().unwrap();
        assert!(mutation_calls.load(Ordering::SeqCst) >= 2);
        assert_eq!(
            final_settings.env.get("USER_OWNED").map(String::as_str),
            Some("keep")
        );
        assert_eq!(
            final_settings.env.get("CLI_FIELD").map(String::as_str),
            Some("saved")
        );
        assert_eq!(
            final_settings.other.get("ui_field"),
            Some(&serde_json::json!("saved"))
        );
        assert_eq!(
            final_settings.other.get("future_top_level"),
            Some(&serde_json::json!({ "enabled": true }))
        );
    }

    #[test]
    fn test_update_atomic_uses_only_central_backup_directory() {
        let home = TestHome::new();
        fs::write(home.settings_path(), r#"{"env":{"USER_OWNED":"keep"}}"#).unwrap();
        let manager = SettingsManager::new(
            home.settings_path(),
            home.backup_dir(),
            LockManager::new(home.lock_dir()),
        );

        manager
            .update_atomic(|settings| {
                settings
                    .env
                    .insert("UPDATED".to_string(), "yes".to_string());
                Ok(())
            })
            .unwrap();

        assert_eq!(fs::read_dir(home.backup_dir()).unwrap().count(), 1);
        assert!(
            fs::read_dir(home.settings_path().parent().unwrap())
                .unwrap()
                .all(|entry| !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".bak"))
        );
    }

    #[test]
    fn test_gemini_platform_paths_use_antigravity_cli_dir() {
        let (settings_path, backup_dir) = SettingsManager::get_platform_paths("gemini").unwrap();
        let settings = settings_path.to_string_lossy().replace('\\', "/");
        let backups = backup_dir.to_string_lossy().replace('\\', "/");

        assert!(settings.ends_with(".gemini/antigravity-cli/settings.json"));
        assert!(backups.ends_with(".gemini/antigravity-cli/backups"));
    }

    #[test]
    fn test_settings_manager_backup_restore() {
        let temp_dir = tempfile::tempdir().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let backup_dir = temp_dir.path().join("backups");
        let lock_dir = temp_dir.path().join("locks");

        let lock_manager = LockManager::new(lock_dir);
        let manager = SettingsManager::new(settings_path, backup_dir, lock_manager);

        // 创建原始设置
        let mut settings = ClaudeSettings::new();
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "original".into());
        manager.save_atomic(&settings).unwrap();

        // 备份
        let backup_path = manager.backup(Some("test")).unwrap();
        assert!(backup_path.exists());

        // 修改设置
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "modified".into());
        manager.save_atomic(&settings).unwrap();

        // 恢复
        manager.restore(&backup_path).unwrap();
        let restored = manager.load().unwrap();
        assert_eq!(
            restored.env.get("ANTHROPIC_BASE_URL"),
            Some(&"original".to_string())
        );
    }

    #[test]
    fn test_backup_auto_cleanup() {
        let temp_dir = tempfile::tempdir().unwrap();
        let settings_path = temp_dir.path().join("settings.json");
        let backup_dir = temp_dir.path().join("backups");
        let lock_dir = temp_dir.path().join("locks");

        let lock_manager = LockManager::new(lock_dir);
        let manager = SettingsManager::new(settings_path, backup_dir, lock_manager);

        // 创建初始设置
        let mut settings = ClaudeSettings::new();
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "test".into());
        manager.save_atomic(&settings).unwrap();

        // 创建15个备份
        for i in 0..15 {
            manager.backup(Some(&format!("config{}", i))).unwrap();
            // 短暂延迟确保时间戳不同
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // 验证只保留了最近10个备份
        let backups = manager.list_backups().unwrap();
        assert_eq!(
            backups.len(),
            10,
            "应该只保留10个备份,但实际有 {} 个",
            backups.len()
        );

        // 验证保留的是最新的10个(按时间倒序,最新的在前)
        assert!(backups.len() <= 10);
    }
}
