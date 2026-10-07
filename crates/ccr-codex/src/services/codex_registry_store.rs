// 📦 Codex Auth Registry 统一存储层
//
// 消除 CodexAuthService / CodexOAuthTokenService / CodexQuotaService
// 三处重复的 registry load/save 逻辑，统一注册表访问模式：
// - load: 只读，不加锁
// - save: 文件锁 + 备份 + 原子写入
// - backup: 独立备份操作

use super::codex_auth_backup::{AuthBackupPool, backup_auth_file};
use crate::models::CodexAuthRegistry;
use ccr_core::core::atomic_writer::AtomicWriter;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::lock::LockManager;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const REGISTRY_LOCK_RESOURCE: &str = "codex_auth_registry";
const REGISTRY_LOCK_TIMEOUT: Duration = Duration::from_secs(10);

pub const SUPPORTED_REGISTRY_MAJOR: u32 = 1;
pub const REGISTRY_READ_ONLY_PREFIX: &str = "注册表只读：";

fn registry_version_is_read_only(version: &str) -> bool {
    version
        .split('.')
        .next()
        .and_then(|major| major.trim().parse::<u32>().ok())
        .is_none_or(|major| major > SUPPORTED_REGISTRY_MAJOR)
}

impl CodexAuthRegistry {
    /// 更新或无法解析的主版本只允许读取。
    pub fn is_read_only(&self) -> bool {
        registry_version_is_read_only(&self.version)
    }
}

pub fn registry_read_only_message(version: &str) -> String {
    format!(
        "{REGISTRY_READ_ONLY_PREFIX}auth_registry.toml 版本 {version} 由更新版本的 CCR 写入，当前版本仅支持 {SUPPORTED_REGISTRY_MAJOR}.x；请升级 CCR 后再执行此操作"
    )
}

/// 提取服务错误中的注册表版本，包括 CcrError 的 Display 前缀。
pub fn registry_read_only_version(error: &str) -> Option<&str> {
    let start = error.find(REGISTRY_READ_ONLY_PREFIX)?;
    let detail = error[start..].strip_prefix(REGISTRY_READ_ONLY_PREFIX)?;
    let detail = detail.strip_prefix("auth_registry.toml 版本 ")?;
    detail
        .rsplit_once(" 由更新版本的 CCR 写入")
        .map(|(version, _)| version)
}

pub(crate) fn ensure_registry_writable(registry: &CodexAuthRegistry) -> Result<()> {
    if registry.is_read_only() {
        return Err(CcrError::ConfigError(registry_read_only_message(
            &registry.version,
        )));
    }
    Ok(())
}

/// Codex Auth 注册表统一存储层
///
/// 提供对 `auth_registry.toml` 的标准化访问：
/// - 读取：无锁，直接解析文件
/// - 写入：文件锁 + 备份 + 原子写入（最安全路径）
pub struct CodexRegistryStore {
    /// auth_registry.toml 路径
    registry_path: PathBuf,
    /// 备份目录
    backup_dir: PathBuf,
    /// 锁目录
    lock_dir: PathBuf,
}

impl CodexRegistryStore {
    /// 从 CCR codex 平台目录构造
    pub fn new(ccr_codex_dir: &Path) -> Self {
        Self {
            registry_path: ccr_codex_dir.join("auth_registry.toml"),
            backup_dir: ccr_codex_dir.join("auth").join("backups"),
            lock_dir: ccr_codex_dir.join(".locks"),
        }
    }

    /// 加载注册表（只读，无锁）
    pub fn load(&self) -> Result<CodexAuthRegistry> {
        if !self.registry_path.exists() {
            return Ok(CodexAuthRegistry::default());
        }

        let content = fs::read_to_string(&self.registry_path)
            .map_err(|e| CcrError::ConfigError(format!("读取注册表失败: {}", e)))?;

        toml::from_str(&content).map_err(|e| {
            if let Ok(table) = toml::from_str::<toml::Table>(&content)
                && let Some(version) = table.get("version")
            {
                let version = version
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| version.to_string());
                if registry_version_is_read_only(&version) {
                    return CcrError::ConfigError(format!(
                        "解析注册表失败: 版本 {version} 由更新版本的 CCR 写入，请升级 CCR（{e}）"
                    ));
                }
            }
            CcrError::ConfigError(format!("解析注册表失败: {e}"))
        })
    }

    /// 保存注册表（文件锁 + 备份 + 原子写入）
    pub fn save(&self, registry: &CodexAuthRegistry) -> Result<()> {
        let lock_manager = LockManager::new(&self.lock_dir);
        let _lock = lock_manager.lock_resource(REGISTRY_LOCK_RESOURCE, REGISTRY_LOCK_TIMEOUT)?;

        ensure_registry_writable(registry)?;

        self.backup()?;
        self.write_locked(registry)
    }

    /// Delete/rename callbacks must finish every required backup before removing files.
    /// The registry lock covers the latest preimage, backups, and final publication.
    pub(crate) fn update_with_prepared_backup<T>(
        &self,
        update: impl FnOnce(&mut CodexAuthRegistry) -> Result<T>,
    ) -> Result<T> {
        let lock_manager = LockManager::new(&self.lock_dir);
        let _lock = lock_manager.lock_resource(REGISTRY_LOCK_RESOURCE, REGISTRY_LOCK_TIMEOUT)?;
        let mut registry = self.load()?;
        ensure_registry_writable(&registry)?;
        let result = update(&mut registry)?;
        self.write_locked(&registry)?;
        Ok(result)
    }

    fn write_locked(&self, registry: &CodexAuthRegistry) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CcrError::ConfigError(format!("创建目录失败: {}", e)))?;
        }

        let content = toml::to_string_pretty(registry)
            .map_err(|e| CcrError::ConfigError(format!("序列化注册表失败: {}", e)))?;

        // 写入内容前设置私有权限，消除写入后再 chmod 的默认权限窗口
        AtomicWriter::new(&self.registry_path)
            .secret(true)
            .write_string(&content)
            .map_err(|e| CcrError::ConfigError(format!("写入注册表失败: {}", e)))?;

        self.ensure_private_permissions(&self.registry_path)?;
        Ok(())
    }

    /// 备份当前注册表文件
    pub fn backup(&self) -> Result<Option<PathBuf>> {
        backup_auth_file(
            &self.registry_path,
            &self.backup_dir,
            AuthBackupPool::Registry,
        )
    }

    fn ensure_private_permissions(&self, path: &Path) -> Result<()> {
        crate::utils::ensure_private_permissions(path)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::models::{CodexAuthAccount, OpenAiAuthMethod};
    use chrono::Utc;

    #[test]
    fn p3_registry_backup_reuses_latest_identical_bytes() {
        let _env = crate::test_support::TestCodexEnv::new();
        let temp = tempfile::tempdir().unwrap();
        let store = CodexRegistryStore::new(temp.path());
        store.save(&CodexAuthRegistry::default()).unwrap();
        let created = store.backup().unwrap().unwrap();
        let first = store.backup_dir.join("auth_registry_20251001_000000.toml");
        fs::rename(created, &first).unwrap();
        filetime::set_file_mtime(&first, filetime::FileTime::from_unix_time(1, 0)).unwrap();
        let second = store.backup().unwrap().unwrap();
        assert_eq!(first, second);
        assert_eq!(fs::read_dir(&store.backup_dir).unwrap().count(), 1);
        assert!(
            fs::metadata(second).unwrap().modified().unwrap()
                > std::time::UNIX_EPOCH + Duration::from_secs(1)
        );
    }

    #[test]
    fn p3_registry_save_stops_when_prewrite_backup_fails() {
        let _env = crate::test_support::TestCodexEnv::new();
        let temp = tempfile::tempdir().unwrap();
        let store = CodexRegistryStore::new(temp.path());
        store.save(&CodexAuthRegistry::default()).unwrap();
        let before = fs::read(&store.registry_path).unwrap();
        fs::create_dir_all(store.backup_dir.parent().unwrap()).unwrap();
        fs::write(&store.backup_dir, b"blocked backup directory").unwrap();
        let registry = CodexAuthRegistry {
            current_auth: Some("changed".into()),
            ..Default::default()
        };
        assert!(store.save(&registry).is_err());
        assert_eq!(fs::read(&store.registry_path).unwrap(), before);
    }

    #[test]
    fn load_save_preserves_unknown_tables_and_account_fields() {
        let _env = crate::test_support::TestCodexEnv::new();
        let content = r#"
version = "1.7"
current_auth = "test"
future_flag = true
future_count = 42
future_timestamp = 2026-10-06T00:00:00Z

[accounts.test]
account_id = "acc-1"
saved_at = "2026-10-06T00:00:00Z"
future_key = "retained"

[accounts.test.future_details]
enabled = true
values = [1, 2, 3]

[future_table]
name = "retained table"

[future_table.nested]
weight = 1.5

[[usage_ledger]]
account_name = "test"
account_id = "acc-1"
started_at = "2026-10-06T00:00:00Z"
"#;
        let temp = tempfile::tempdir().unwrap();
        let store = CodexRegistryStore::new(temp.path());
        fs::write(&store.registry_path, content).unwrap();
        let original: toml::Table = toml::from_str(content).unwrap();
        let registry = store.load().unwrap();
        assert_eq!(registry.extra["future_count"].as_integer(), Some(42));
        assert!(
            registry.accounts["test"]
                .extra
                .contains_key("future_details")
        );
        store.save(&registry).unwrap();
        let output = fs::read_to_string(&store.registry_path).unwrap();
        let roundtrip: toml::Table = toml::from_str(&output).unwrap();
        assert_eq!(roundtrip, original);
        assert_eq!(store.load().unwrap().version, "1.7");
    }

    #[test]
    fn registry_version_gate_accepts_supported_major_and_defaults() {
        for (version, read_only) in [
            ("1.0", false),
            ("1.7", false),
            (" 1 .7", false),
            ("2.0", true),
            ("abc", true),
            ("", true),
            ("4294967296.0", true),
        ] {
            let registry: CodexAuthRegistry =
                toml::from_str(&format!("version = {version:?}")).unwrap();
            assert_eq!(registry.is_read_only(), read_only, "version {version:?}");
        }
        let registry: CodexAuthRegistry = toml::from_str("").unwrap();
        assert_eq!(registry.version, "1.0");
        assert!(!registry.is_read_only());
    }

    #[test]
    fn registry_read_only_message_preserves_version_in_display_errors() {
        for version in ["2.0", "abc", "abc 由future", ""] {
            let message = registry_read_only_message(version);
            assert!(message.starts_with(REGISTRY_READ_ONLY_PREFIX));
            assert_eq!(registry_read_only_version(&message), Some(version));
            let displayed = CcrError::ConfigError(message).to_string();
            assert_eq!(registry_read_only_version(&displayed), Some(version));
        }
        assert_eq!(registry_read_only_version("原始错误"), None);
        assert_eq!(registry_read_only_version("注册表只读：无版本"), None);
    }

    #[test]
    fn save_read_only_registry_preserves_bytes_and_backup_set() {
        for version in ["2.0", "abc"] {
            let temp = tempfile::tempdir().unwrap();
            let store = CodexRegistryStore::new(temp.path());
            let content = format!("version = {version:?}\n");
            fs::write(&store.registry_path, &content).unwrap();
            let mut registry = store.load().unwrap();
            registry.current_auth = Some("changed".into());

            for existing_backup in [false, true] {
                if existing_backup {
                    fs::create_dir_all(&store.backup_dir).unwrap();
                    fs::write(store.backup_dir.join("retained.toml"), b"retained").unwrap();
                }
                let error = store.save(&registry).unwrap_err();
                assert!(matches!(error, CcrError::ConfigError(ref message)
                    if message.starts_with(REGISTRY_READ_ONLY_PREFIX)));
                assert_eq!(fs::read(&store.registry_path).unwrap(), content.as_bytes());
                if existing_backup {
                    assert_eq!(fs::read_dir(&store.backup_dir).unwrap().count(), 1);
                    assert_eq!(
                        fs::read(store.backup_dir.join("retained.toml")).unwrap(),
                        b"retained"
                    );
                } else {
                    assert!(!store.backup_dir.exists());
                }
            }
        }
    }

    #[test]
    fn incompatible_registry_structure_reports_upgrade_for_future_versions() {
        for version in ["2.0", "abc"] {
            let temp = tempfile::tempdir().unwrap();
            let store = CodexRegistryStore::new(temp.path());
            let content = format!("version = {version:?}\naccounts = 42\n");
            fs::write(&store.registry_path, &content).unwrap();
            let error = store.load().unwrap_err();
            let CcrError::ConfigError(message) = error else {
                panic!("expected a config error");
            };
            assert!(message.starts_with("解析注册表失败: "));
            assert!(message.contains(version));
            assert!(message.contains("请升级 CCR"));
            assert_eq!(fs::read_to_string(&store.registry_path).unwrap(), content);
        }
    }

    #[test]
    fn supported_registry_parse_errors_keep_original_text() {
        for content in [
            "version = \"1.7\"\naccounts = 42\n",
            "accounts = 42\n",
            "version = \"2.0\"\naccounts = [\n",
        ] {
            let temp = tempfile::tempdir().unwrap();
            let store = CodexRegistryStore::new(temp.path());
            fs::write(&store.registry_path, content).unwrap();
            let original = toml::from_str::<CodexAuthRegistry>(content).unwrap_err();
            let CcrError::ConfigError(message) = store.load().unwrap_err() else {
                panic!("expected a config error");
            };
            assert_eq!(message, format!("解析注册表失败: {original}"));
        }
    }

    #[test]
    fn test_load_missing_returns_default() {
        let temp = tempfile::tempdir().unwrap();
        let store = CodexRegistryStore::new(temp.path());
        let registry = store.load().unwrap();
        assert_eq!(registry.version, "1.0");
        assert!(registry.accounts.is_empty());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let _env = crate::test_support::TestCodexEnv::new();
        let temp = tempfile::tempdir().unwrap();
        let store = CodexRegistryStore::new(temp.path());

        let mut registry = CodexAuthRegistry {
            current_auth: Some("test".to_string()),
            ..Default::default()
        };
        registry.accounts.insert(
            "test".to_string(),
            CodexAuthAccount {
                description: Some("Test account".to_string()),
                account_id: "acc-1".to_string(),
                identity_key: None,
                auth_method: Some(OpenAiAuthMethod::Chatgpt),
                api_base_url: None,
                api_provider_name: None,
                email: None,
                plan_type: None,
                saved_at: Utc::now(),
                last_used: None,
                last_refresh: None,
                expires_at: None,
                extra: toml::Table::new(),
            },
        );

        store.save(&registry).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.current_auth, Some("test".to_string()));
        assert!(loaded.accounts.contains_key("test"));
    }

    #[cfg(unix)]
    #[test]
    fn save_writes_owner_only_registry() {
        let _env = crate::test_support::TestCodexEnv::new();
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let store = CodexRegistryStore::new(temp.path());
        store.save(&CodexAuthRegistry::default()).unwrap();
        // 既有宽权限文件被替换后同样收紧
        fs::set_permissions(&store.registry_path, fs::Permissions::from_mode(0o644)).unwrap();
        store.save(&CodexAuthRegistry::default()).unwrap();

        let mode = fs::metadata(&store.registry_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn test_backup_creates_file() {
        let _env = crate::test_support::TestCodexEnv::new();
        let temp = tempfile::tempdir().unwrap();
        let store = CodexRegistryStore::new(temp.path());

        // 先保存内容
        store.save(&CodexAuthRegistry::default()).unwrap();

        // 备份
        let backup_path = store.backup().unwrap();
        assert!(backup_path.is_some());
        assert!(backup_path.unwrap().exists());
    }

    #[test]
    fn test_backup_missing_file_returns_none() {
        let _env = crate::test_support::TestCodexEnv::new();
        let temp = tempfile::tempdir().unwrap();
        let store = CodexRegistryStore::new(temp.path());
        let result = store.backup().unwrap();
        assert!(result.is_none());
    }
}
