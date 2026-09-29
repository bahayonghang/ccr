//! Path-scoped profile transactions. Lock order: operation -> resource -> leaf.

use super::{CcsConfig, ConfigManager, ConfigSection};
use ccr_core::core::guarded_write::{
    VersionedWriteOutcome, content_version_token, enforce_secret_permissions_versioned,
    normalized_resource_path, write_guarded_versioned,
};
use ccr_core::core::{BackupPolicy, FileLock, LockManager, WriteOptions};
use ccr_core::{CcrError, Result};
use indexmap::IndexMap;
use std::fs;
use std::path::Path;
use std::time::Duration;

/// A field omitted from a patch is retained. Removal is explicit.
#[derive(Clone)]
pub enum FieldPatch {
    Set(toml::Value),
    Remove,
}

/// Values deliberately have no Debug implementation because fields may hold secrets.
#[derive(Clone, Default)]
pub struct ConfigPatch {
    pub fields: IndexMap<String, FieldPatch>,
}

impl ConfigPatch {
    pub fn apply(&self, section: &ConfigSection) -> Result<ConfigSection> {
        let mut value = toml::Value::try_from(section)
            .map_err(|_| CcrError::ConfigError("无法序列化 profile".into()))?;
        let fields = value
            .as_table_mut()
            .ok_or_else(|| CcrError::ConfigError("profile 必须是表".into()))?;
        for (key, patch) in &self.fields {
            if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                return Err(CcrError::ValidationError("patch 字段名无效".into()));
            }
            match patch {
                FieldPatch::Set(value) => {
                    fields.insert(key.clone(), value.clone());
                }
                FieldPatch::Remove => {
                    fields.remove(key);
                }
            }
        }
        value
            .try_into()
            .map_err(|_| CcrError::ValidationError("patch 字段类型无效".into()))
    }
}

/// Read result for editors. The token covers the original bytes, not a serialization.
pub struct ConfigSnapshot {
    pub config: CcsConfig,
    pub version: String,
}

/// Project stored activation intent from the same bytes as the profile data.
/// The low-level parser's compatibility default is not a declared marker.
pub(crate) fn parse_repository_config(content: &str) -> Result<CcsConfig> {
    let mut config = crate::platforms::base::parse_config_from_str(content)?;
    let document: toml::Value = toml::from_str(content)
        .map_err(|_| CcrError::ConfigFormatInvalid("配置 TOML 无效".into()))?;
    if document
        .get("current_config")
        .and_then(toml::Value::as_str)
        .is_none()
    {
        config.current_config.clear();
    }
    Ok(config)
}

/// A resource identity is independent of the adapter and the platform label.
pub fn config_resource_name(path: &Path) -> Result<String> {
    let normalized = normalized_resource_path(path)
        .map_err(|error| CcrError::ConfigError(format!("解析配置路径失败: {error}")))?;
    let name = normalized.to_string_lossy();
    let hash = name
        .as_bytes()
        .iter()
        .fold(0xcbf29ce484222325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        });
    Ok(format!("config_resource_{hash:016x}"))
}

pub(crate) fn profile_write_options(path: &Path) -> WriteOptions {
    let file_name_matches = |path: &Path, expected: &str| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| {
                name == expected || (cfg!(windows) && name.eq_ignore_ascii_case(expected))
            })
    };
    let backup = if file_name_matches(path, "profiles.toml") {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let dir = parent
            .parent()
            .filter(|ancestor| file_name_matches(ancestor, "platforms"))
            .and_then(Path::parent)
            .map(|root| {
                root.join("backups")
                    .join(parent.file_name().unwrap_or_default())
            })
            .unwrap_or_else(|| parent.join("backups"));
        BackupPolicy::Dir {
            dir,
            prefix: "profiles".into(),
        }
    } else {
        BackupPolicy::None
    };
    WriteOptions {
        backup,
        secret: true,
        ..Default::default()
    }
}

impl ConfigManager {
    /// Explicit deactivation. A missing profile file stays missing.
    pub fn clear_current_if_present(&self) -> Result<()> {
        match self.mutate(|config| {
            config.current_config.clear();
            Ok(())
        }) {
            Err(CcrError::ConfigMissing(_)) => Ok(()),
            result => result,
        }
    }

    /// Acquire before reading. Do not call another mutation while this guard is held.
    pub fn lock_mutation(&self) -> Result<FileLock> {
        LockManager::with_default_path()?.lock_resource(
            &config_resource_name(self.config_path())?,
            Duration::from_secs(10),
        )
    }

    pub fn snapshot(&self) -> Result<ConfigSnapshot> {
        let bytes = fs::read(self.config_path()).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                CcrError::ConfigMissing(self.config_path().display().to_string())
            } else {
                CcrError::FileIoError(format!("读取配置失败: {error}"))
            }
        })?;
        let content = std::str::from_utf8(&bytes)
            .map_err(|_| CcrError::ConfigFormatInvalid("配置必须是 UTF-8".into()))?;
        Ok(ConfigSnapshot {
            config: parse_repository_config(content)?,
            version: content_version_token(&bytes),
        })
    }

    /// Mutate only the latest document. The closure runs once and is never replayed.
    /// Cross-file side effects still require caller-owned recovery.
    pub fn mutate<T>(&self, mutate: impl FnOnce(&mut CcsConfig) -> Result<T>) -> Result<T> {
        self.mutate_versioned(None, false, mutate)
    }

    pub fn mutate_or_create<T>(
        &self,
        mutate: impl FnOnce(&mut CcsConfig) -> Result<T>,
    ) -> Result<T> {
        self.mutate_versioned(None, true, mutate)
    }

    pub fn mutate_versioned<T>(
        &self,
        expected: Option<&str>,
        create: bool,
        mutate: impl FnOnce(&mut CcsConfig) -> Result<T>,
    ) -> Result<T> {
        let _lock = self.lock_mutation()?;
        let bytes = match fs::read(self.config_path()) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && create => None,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(CcrError::ConfigMissing(
                    self.config_path().display().to_string(),
                ));
            }
            Err(error) => return Err(CcrError::FileIoError(format!("读取配置失败: {error}"))),
        };
        let token = bytes
            .as_deref()
            .map(content_version_token)
            .unwrap_or_default();
        if expected.is_some_and(|expected| expected != token) {
            return Err(version_conflict());
        }
        let content = std::str::from_utf8(bytes.as_deref().unwrap_or_default())
            .map_err(|_| CcrError::ConfigFormatInvalid("配置必须是 UTF-8".into()))?;
        let mut config = if bytes.is_some() {
            parse_repository_config(content)?
        } else {
            // Explicit first creation retains the legacy initialization defaults.
            crate::platforms::base::parse_config_from_str(content)?
        };
        let mut before = toml::Value::try_from(&config)
            .map_err(|_| CcrError::ConfigError("无法序列化配置".into()))?;
        let result = mutate(&mut config)?;
        let mut after = toml::Value::try_from(&config)
            .map_err(|_| CcrError::ConfigError("无法序列化配置".into()))?;
        if bytes.is_some() && before == after {
            if !enforce_secret_permissions_versioned(
                self.config_path(),
                &token,
                profile_write_options(self.config_path()).lock_timeout,
            )? {
                return Err(version_conflict());
            }
            return Ok(result);
        }
        // Keep unknown settings and unedited values from the original document.
        let mut document: toml::Value = toml::from_str(content)
            .map_err(|_| CcrError::ConfigFormatInvalid("配置 TOML 无效".into()))?;
        let keep_simplified = bytes.is_some()
            && document
                .get("current_config")
                .and_then(toml::Value::as_str)
                .is_none()
            && config.current_config.is_empty();
        if keep_simplified {
            for value in [&mut before, &mut after] {
                if let Some(table) = value.as_table_mut() {
                    table.remove("default_config");
                    table.remove("current_config");
                }
            }
        }
        apply_delta(&mut document, &before, &after);
        if !keep_simplified
            && let (Some(document), Some(after)) = (document.as_table_mut(), after.as_table())
        {
            for key in ["default_config", "current_config"] {
                if let Some(value) = after.get(key) {
                    document.insert(key.into(), value.clone());
                }
            }
        }
        let content = toml::to_string_pretty(&document)
            .map_err(|_| CcrError::ConfigError("无法序列化配置".into()))?;
        match write_guarded_versioned(
            self.config_path(),
            content.as_bytes(),
            &token,
            &profile_write_options(self.config_path()),
        )? {
            VersionedWriteOutcome::Written => Ok(result),
            VersionedWriteOutcome::Conflict => Err(version_conflict()),
        }
    }

    /// Apply a strict partial update and check the caller's platform rules under the lock.
    pub fn patch(
        &self,
        name: &str,
        new_name: Option<&str>,
        patch: &ConfigPatch,
        expected: &str,
        validate: impl FnOnce(&ConfigSection) -> Result<()>,
    ) -> Result<()> {
        self.mutate_versioned(Some(expected), false, |config| {
            let target = new_name.unwrap_or(name);
            validate_profile_name(target)?;
            let current = config.get_section(name)?;
            if target != name && config.sections.contains_key(target) {
                return Err(CcrError::ValidationError("目标配置名称已存在".into()));
            }
            let section = patch.apply(current)?;
            validate(&section)?;
            if target != name {
                config.sections.shift_remove(name);
                if config.current_config == name {
                    config.current_config = target.into();
                }
                if config.default_config == name {
                    config.default_config = target.into();
                }
            }
            config.sections.insert(target.into(), section);
            Ok(())
        })
    }
}

pub(crate) fn validate_profile_name(name: &str) -> Result<()> {
    if name.trim().is_empty() || matches!(name, "default_config" | "current_config" | "settings") {
        return Err(CcrError::ValidationError(
            "配置名称为空或属于保留字段".into(),
        ));
    }
    Ok(())
}

fn version_conflict() -> CcrError {
    CcrError::ValidationError("配置版本冲突，请重新读取后重试".into())
}

/// Preserve TOML extension values that the ProfileConfig JSON projection cannot
/// represent exactly, such as datetimes. Only changed projected fields apply.
pub(crate) fn merge_section_delta(
    original: &ConfigSection,
    before: &ConfigSection,
    after: &ConfigSection,
) -> Result<ConfigSection> {
    let encode = |section: &ConfigSection| {
        toml::Value::try_from(section)
            .map_err(|_| CcrError::ConfigError("无法序列化 profile".into()))
    };
    let mut value = encode(original)?;
    apply_delta(&mut value, &encode(before)?, &encode(after)?);
    value
        .try_into()
        .map_err(|_| CcrError::ValidationError("profile 字段类型无效".into()))
}

fn apply_delta(target: &mut toml::Value, before: &toml::Value, after: &toml::Value) {
    if before == after {
        return;
    }
    if let (Some(target), Some(before), Some(after)) =
        (target.as_table_mut(), before.as_table(), after.as_table())
    {
        for key in before.keys().filter(|key| !after.contains_key(*key)) {
            target.remove(key);
        }
        for (key, value) in after {
            match (before.get(key), target.get_mut(key)) {
                (Some(old), Some(current)) => apply_delta(current, old, value),
                (Some(old), None) if old == value => {}
                _ => {
                    target.insert(key.clone(), value.clone());
                }
            }
        }
    } else {
        *target = after.clone();
    }
}
