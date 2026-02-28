// ⚙️ Codex 配置管理模块
// 📦 负责读写和管理 ~/.codex/config.toml 和 ~/.codex/auth.json
//
// 核心职责:
// - 🔧 管理 Codex config.toml (TOML 格式)
// - 🔑 管理 Codex auth.json (JSON 格式)
// - 🔄 原子性写入 (临时文件 + 重命名)
// - 🔒 文件锁保证并发安全
// - 💾 自动备份机制

use crate::core::cache::ConfigCache;
use crate::core::error::{CcrError, Result};
use crate::core::lock::LockManager;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::NamedTempFile;

/// ⚙️ Codex 配置管理器
///
/// 负责 Codex CLI 配置文件的完整生命周期管理
///
/// 核心功能:
/// - 📖 加载和解析 config.toml / auth.json
/// - 💾 原子性保存 (临时文件 + rename)
/// - 🔒 文件锁防止并发冲突
/// - 💾 自动备份和清理
pub struct CodexConfigManager {
    config_path: PathBuf,
    auth_path: PathBuf,
    backup_dir: PathBuf,
    lock_manager: LockManager,
}

/// 锁资源名称常量
const LOCK_RESOURCE: &str = "codex_config";
/// 锁超时时长
const LOCK_TIMEOUT: Duration = Duration::from_secs(10);
/// 最大备份保留数量
const MAX_BACKUPS: usize = 10;

#[allow(dead_code)]
impl CodexConfigManager {
    /// 🏗️ 创建新的配置管理器
    pub fn new<P: AsRef<Path>, Q: AsRef<Path>, R: AsRef<Path>>(
        config_path: P,
        auth_path: Q,
        backup_dir: R,
        lock_manager: LockManager,
    ) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            auth_path: auth_path.as_ref().to_path_buf(),
            backup_dir: backup_dir.as_ref().to_path_buf(),
            lock_manager,
        }
    }

    /// 🏠 使用默认路径创建管理器
    ///
    /// 默认路径:
    /// - config: ~/.codex/config.toml
    /// - auth: ~/.codex/auth.json
    /// - backup: ~/.codex/backups/
    ///
    /// 支持 `CCR_CODEX_DIR` 环境变量覆盖
    pub fn with_default() -> Result<Self> {
        let codex_dir = Self::resolve_codex_dir()?;
        let lock_manager = LockManager::with_default_path()?;

        tracing::debug!("Codex 配置目录: {:?}", codex_dir);

        Ok(Self {
            config_path: codex_dir.join("config.toml"),
            auth_path: codex_dir.join("auth.json"),
            backup_dir: codex_dir.join("backups"),
            lock_manager,
        })
    }

    /// 📁 解析 Codex 配置目录
    fn resolve_codex_dir() -> Result<PathBuf> {
        if let Ok(custom) = std::env::var("CCR_CODEX_DIR") {
            return Ok(PathBuf::from(custom));
        }
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
        Ok(home.join(".codex"))
    }

    /// 📁 获取 config.toml 路径
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// 📁 获取 auth.json 路径
    pub fn auth_path(&self) -> &Path {
        &self.auth_path
    }

    /// 📁 确保配置目录存在
    fn ensure_dir(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CcrError::SettingsError(format!("创建 Codex 目录失败: {}", e)))?;
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════
    // 📖 加载方法
    // ═══════════════════════════════════════════════════════════

    /// 📖 加载 config.toml
    ///
    /// 文件不存在时返回空 Table
    pub fn load_config(&self) -> Result<toml::Value> {
        if !self.config_path.exists() {
            return Ok(toml::Value::Table(toml::map::Map::new()));
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| CcrError::SettingsError(format!("读取 Codex config.toml 失败: {}", e)))?;

        toml::from_str(&content)
            .map_err(|e| CcrError::SettingsError(format!("解析 Codex config.toml 失败: {}", e)))
    }

    /// 🔑 加载 auth.json
    ///
    /// 文件不存在时返回空 Object
    pub fn load_auth(&self) -> Result<JsonMap<String, JsonValue>> {
        if !self.auth_path.exists() {
            return Ok(JsonMap::new());
        }

        let content = fs::read_to_string(&self.auth_path)
            .map_err(|e| CcrError::SettingsError(format!("读取 Codex auth.json 失败: {}", e)))?;

        serde_json::from_str::<JsonMap<String, JsonValue>>(&content)
            .map_err(|e| CcrError::SettingsError(format!("解析 Codex auth.json 失败: {}", e)))
    }

    // ═══════════════════════════════════════════════════════════
    // 💾 原子保存方法
    // ═══════════════════════════════════════════════════════════

    /// 💾 原子保存 config.toml
    ///
    /// 执行步骤:
    /// 1. 🔒 获取文件锁
    /// 2. 📁 确保目录存在
    /// 3. 📝 序列化为 TOML
    /// 4. 📄 写入临时文件
    /// 5. 🔄 原子替换
    pub fn save_config_atomic(&self, config: &toml::Value) -> Result<()> {
        let _lock = self
            .lock_manager
            .lock_resource(LOCK_RESOURCE, LOCK_TIMEOUT)?;

        self.ensure_dir()?;

        let content = toml::to_string_pretty(config).map_err(|e| {
            CcrError::SettingsError(format!("序列化 Codex config.toml 失败: {}", e))
        })?;

        self.atomic_write(&self.config_path, content.as_bytes())?;

        tracing::info!("✅ Codex config.toml 已原子保存: {:?}", self.config_path);
        Ok(())
    }

    /// 🔑 原子保存 auth.json
    ///
    /// 执行步骤同 save_config_atomic
    pub fn save_auth_atomic(&self, auth: &JsonMap<String, JsonValue>) -> Result<()> {
        let _lock = self
            .lock_manager
            .lock_resource(LOCK_RESOURCE, LOCK_TIMEOUT)?;

        self.ensure_dir()?;

        let content = serde_json::to_string_pretty(&JsonValue::Object(auth.clone()))
            .map_err(|e| CcrError::SettingsError(format!("序列化 auth.json 失败: {}", e)))?;

        self.atomic_write(&self.auth_path, content.as_bytes())?;

        tracing::info!("✅ Codex auth.json 已原子保存: {:?}", self.auth_path);
        Ok(())
    }

    /// 🔄 原子写入文件 (临时文件 + persist)
    fn atomic_write(&self, target: &Path, data: &[u8]) -> Result<()> {
        let temp_file = if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CcrError::SettingsError(format!("创建目录失败: {}", e)))?;
            NamedTempFile::new_in(parent)
        } else {
            NamedTempFile::new()
        }
        .map_err(|e| CcrError::SettingsError(format!("创建临时文件失败: {}", e)))?;

        fs::write(temp_file.path(), data)
            .map_err(|e| CcrError::SettingsError(format!("写入临时文件失败: {}", e)))?;

        temp_file
            .persist(target)
            .map_err(|e| CcrError::SettingsError(format!("原子替换文件失败: {}", e)))?;

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════
    // 💾 备份方法
    // ═══════════════════════════════════════════════════════════

    /// 💾 备份 config.toml
    ///
    /// 文件名格式: config.{label}.{timestamp}.toml.bak
    pub fn backup_config(&self, label: &str) -> Result<Option<PathBuf>> {
        self.backup_file(&self.config_path, "config", label, "toml")
    }

    /// 💾 备份 auth.json
    ///
    /// 文件名格式: auth.{label}.{timestamp}.json.bak
    pub fn backup_auth(&self, label: &str) -> Result<Option<PathBuf>> {
        self.backup_file(&self.auth_path, "auth", label, "json")
    }

    /// 💾 通用备份方法
    fn backup_file(
        &self,
        source: &Path,
        prefix: &str,
        label: &str,
        ext: &str,
    ) -> Result<Option<PathBuf>> {
        if !source.exists() {
            tracing::debug!("备份源文件不存在，跳过: {:?}", source);
            return Ok(None);
        }

        fs::create_dir_all(&self.backup_dir)
            .map_err(|e| CcrError::SettingsError(format!("创建备份目录失败: {}", e)))?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}.{}.{}.{}.bak", prefix, label, timestamp, ext);
        let backup_path = self.backup_dir.join(filename);

        fs::copy(source, &backup_path)
            .map_err(|e| CcrError::SettingsError(format!("备份文件失败: {}", e)))?;

        tracing::info!("💾 已备份 {:?} → {:?}", source, backup_path);

        // 自动清理旧备份 (只保留同前缀的最近 MAX_BACKUPS 个)
        self.cleanup_old_backups(prefix)?;

        Ok(Some(backup_path))
    }

    /// 🧹 清理旧备份 (只保留最近 MAX_BACKUPS 个同前缀的备份)
    fn cleanup_old_backups(&self, prefix: &str) -> Result<()> {
        if !self.backup_dir.exists() {
            return Ok(());
        }

        let mut backups: Vec<PathBuf> = fs::read_dir(&self.backup_dir)
            .map_err(|e| CcrError::SettingsError(format!("读取备份目录失败: {}", e)))?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|name| name.starts_with(prefix) && name.ends_with(".bak"))
            })
            .collect();

        // 按修改时间倒序 (最新在前)
        backups.sort_by(|a, b| {
            let a_time = fs::metadata(a).and_then(|m| m.modified()).ok();
            let b_time = fs::metadata(b).and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });

        if backups.len() > MAX_BACKUPS {
            for old in &backups[MAX_BACKUPS..] {
                if let Err(e) = fs::remove_file(old) {
                    tracing::warn!("清理旧备份失败 {:?}: {}", old, e);
                } else {
                    tracing::debug!("🗑️ 已删除旧备份: {:?}", old);
                }
            }
            tracing::info!(
                "🧹 已清理 {} 个旧备份 (前缀: {})",
                backups.len() - MAX_BACKUPS,
                prefix
            );
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════
    // 🔄 重置方法
    // ═══════════════════════════════════════════════════════════

    /// 🔄 重置到默认状态
    ///
    /// 1. 备份当前的 config.toml 和 auth.json
    /// 2. 写入空 Table / 空 Object
    pub fn reset_to_defaults(&self, label: &str) -> Result<()> {
        // 备份
        self.backup_config(label)?;
        self.backup_auth(label)?;

        // 重置
        let empty_table = toml::Value::Table(toml::map::Map::new());
        self.save_config_atomic(&empty_table)?;

        let empty_object = JsonMap::new();
        self.save_auth_atomic(&empty_object)?;

        tracing::info!("🔄 Codex 配置已重置到默认状态");
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════
// 🗄️ 缓存配置管理器
// ═══════════════════════════════════════════════════════════

/// 🗄️ 缓存 Codex 配置管理器
///
/// 封装 `CodexConfigManager`，添加自动缓存支持
///
/// ## 特性
/// - 📖 自动缓存 load_config() / load_auth() 结果
/// - 🔄 save 时自动失效缓存
/// - ⏰ TTL 过期自动重新加载 (默认 30s)
/// - 🔒 线程安全
#[allow(dead_code)]
pub struct CachedCodexConfigManager {
    inner: CodexConfigManager,
    config_cache: ConfigCache<toml::Value>,
    auth_cache: ConfigCache<JsonMap<String, JsonValue>>,
}

#[allow(dead_code)]
impl CachedCodexConfigManager {
    /// 🏗️ 创建新的缓存管理器
    pub fn new(inner: CodexConfigManager, ttl: Duration) -> Self {
        Self {
            inner,
            config_cache: ConfigCache::new(ttl),
            auth_cache: ConfigCache::new(ttl),
        }
    }

    /// 🏠 使用默认路径和 TTL 创建
    ///
    /// 默认 TTL: 30 秒
    pub fn with_default() -> Result<Self> {
        let inner = CodexConfigManager::with_default()?;
        Ok(Self::new(inner, Duration::from_secs(30)))
    }

    /// 📁 获取 config.toml 路径
    pub fn config_path(&self) -> &Path {
        self.inner.config_path()
    }

    /// 📁 获取 auth.json 路径
    pub fn auth_path(&self) -> &Path {
        self.inner.auth_path()
    }

    /// 📖 加载 config.toml (带缓存)
    pub fn load_config(&self) -> Result<toml::Value> {
        self.config_cache.get_or_load(|| self.inner.load_config())
    }

    /// 🔑 加载 auth.json (带缓存)
    pub fn load_auth(&self) -> Result<JsonMap<String, JsonValue>> {
        self.auth_cache.get_or_load(|| self.inner.load_auth())
    }

    /// 💾 原子保存 config.toml 并失效缓存
    pub fn save_config_atomic(&self, config: &toml::Value) -> Result<()> {
        self.inner.save_config_atomic(config)?;
        self.config_cache.invalidate();
        Ok(())
    }

    /// 🔑 原子保存 auth.json 并失效缓存
    pub fn save_auth_atomic(&self, auth: &JsonMap<String, JsonValue>) -> Result<()> {
        self.inner.save_auth_atomic(auth)?;
        self.auth_cache.invalidate();
        Ok(())
    }

    /// 💾 备份 config.toml
    pub fn backup_config(&self, label: &str) -> Result<Option<PathBuf>> {
        self.inner.backup_config(label)
    }

    /// 💾 备份 auth.json
    pub fn backup_auth(&self, label: &str) -> Result<Option<PathBuf>> {
        self.inner.backup_auth(label)
    }

    /// 🔄 重置到默认状态并失效缓存
    pub fn reset_to_defaults(&self, label: &str) -> Result<()> {
        self.inner.reset_to_defaults(label)?;
        self.invalidate_cache();
        Ok(())
    }

    /// 🧹 手动失效所有缓存
    pub fn invalidate_cache(&self) {
        self.config_cache.invalidate();
        self.auth_cache.invalidate();
    }

    /// 🔍 获取内部管理器引用
    pub fn inner(&self) -> &CodexConfigManager {
        &self.inner
    }
}

// ═══════════════════════════════════════════════════════════
// 🧪 测试
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::core::lock::LockManager;

    fn create_test_manager(temp_dir: &Path) -> CodexConfigManager {
        let lock_dir = temp_dir.join("locks");
        let lock_manager = LockManager::new(lock_dir);
        CodexConfigManager::new(
            temp_dir.join("config.toml"),
            temp_dir.join("auth.json"),
            temp_dir.join("backups"),
            lock_manager,
        )
    }

    #[test]
    fn test_load_save_config_roundtrip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = create_test_manager(temp_dir.path());

        // 初始加载应返回空 Table
        let config = manager.load_config().unwrap();
        assert!(config.is_table());
        assert!(config.as_table().unwrap().is_empty());

        // 构造测试数据
        let mut table = toml::map::Map::new();
        table.insert("model".into(), toml::Value::String("gpt-4.1".into()));
        table.insert(
            "model_provider".into(),
            toml::Value::String("custom".into()),
        );
        let config = toml::Value::Table(table);

        // 保存并重新加载
        manager.save_config_atomic(&config).unwrap();
        let loaded = manager.load_config().unwrap();

        assert_eq!(
            loaded.get("model").and_then(|v| v.as_str()),
            Some("gpt-4.1")
        );
        assert_eq!(
            loaded.get("model_provider").and_then(|v| v.as_str()),
            Some("custom")
        );
    }

    #[test]
    fn test_load_save_auth_roundtrip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = create_test_manager(temp_dir.path());

        // 初始加载应返回空 Object
        let auth = manager.load_auth().unwrap();
        assert!(auth.is_empty());

        // 构造测试数据
        let mut auth = JsonMap::new();
        auth.insert(
            "OPENAI_API_KEY".into(),
            JsonValue::String("sk-test-key".into()),
        );

        // 保存并重新加载
        manager.save_auth_atomic(&auth).unwrap();
        let loaded = manager.load_auth().unwrap();

        assert_eq!(
            loaded.get("OPENAI_API_KEY").and_then(|v| v.as_str()),
            Some("sk-test-key")
        );
    }

    #[test]
    fn test_backup_config() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = create_test_manager(temp_dir.path());

        // 先创建一个 config 文件
        let mut table = toml::map::Map::new();
        table.insert("model".into(), toml::Value::String("test".into()));
        manager
            .save_config_atomic(&toml::Value::Table(table))
            .unwrap();

        // 备份
        let backup_path = manager.backup_config("test_label").unwrap();
        assert!(backup_path.is_some());
        assert!(backup_path.unwrap().exists());

        // 备份目录应包含备份文件
        let backup_dir = temp_dir.path().join("backups");
        assert!(backup_dir.exists());
    }

    #[test]
    fn test_backup_nonexistent_returns_none() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = create_test_manager(temp_dir.path());

        // 不存在的文件备份应返回 None
        let result = manager.backup_config("label").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_reset_to_defaults() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = create_test_manager(temp_dir.path());

        // 先写入一些数据
        let mut table = toml::map::Map::new();
        table.insert(
            "model_provider".into(),
            toml::Value::String("custom".into()),
        );
        manager
            .save_config_atomic(&toml::Value::Table(table))
            .unwrap();

        let mut auth = JsonMap::new();
        auth.insert("OPENAI_API_KEY".into(), JsonValue::String("sk-key".into()));
        manager.save_auth_atomic(&auth).unwrap();

        // 重置
        manager.reset_to_defaults("pre_reset").unwrap();

        // 验证已重置为空
        let config = manager.load_config().unwrap();
        assert!(config.as_table().unwrap().is_empty());

        let auth = manager.load_auth().unwrap();
        assert!(auth.is_empty());
    }

    #[test]
    fn test_cache_hit_and_invalidation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let lock_dir = temp_dir.path().join("locks");
        let lock_manager = LockManager::new(lock_dir);
        let inner = CodexConfigManager::new(
            temp_dir.path().join("config.toml"),
            temp_dir.path().join("auth.json"),
            temp_dir.path().join("backups"),
            lock_manager,
        );
        let cached = CachedCodexConfigManager::new(inner, Duration::from_secs(60));

        // 写入初始数据
        let mut table = toml::map::Map::new();
        table.insert("model".into(), toml::Value::String("v1".into()));
        cached
            .save_config_atomic(&toml::Value::Table(table))
            .unwrap();

        // 第一次加载 (缓存未命中)
        let config1 = cached.load_config().unwrap();
        assert_eq!(config1.get("model").and_then(|v| v.as_str()), Some("v1"));

        // 第二次加载 (应命中缓存)
        let config2 = cached.load_config().unwrap();
        assert_eq!(config2.get("model").and_then(|v| v.as_str()), Some("v1"));

        // 通过 save 更新 (应自动失效缓存)
        let mut table2 = toml::map::Map::new();
        table2.insert("model".into(), toml::Value::String("v2".into()));
        cached
            .save_config_atomic(&toml::Value::Table(table2))
            .unwrap();

        // 再次加载应得到新值
        let config3 = cached.load_config().unwrap();
        assert_eq!(config3.get("model").and_then(|v| v.as_str()), Some("v2"));

        // 手动失效
        cached.invalidate_cache();
        let config4 = cached.load_config().unwrap();
        assert_eq!(config4.get("model").and_then(|v| v.as_str()), Some("v2"));
    }
}
