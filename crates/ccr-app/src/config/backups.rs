//! Managed configuration backup policy shared by the desktop adapters.

use ccr_config::{ConfigService, platforms::base::parse_config_from_str};
use ccr_core::{CcrError, Result};
use std::path::{Component, Path, PathBuf};

pub fn resolve_managed_restore_path(config_path: &Path, backup_path: &str) -> Result<PathBuf> {
    let requested = Path::new(backup_path);
    if requested.is_absolute()
        || requested.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::Prefix(_) | Component::RootDir
            )
        })
    {
        return Err(CcrError::ValidationError(
            "只能恢复当前 CCR 配置目录中的托管备份文件".into(),
        ));
    }
    let config_dir = config_path
        .parent()
        .ok_or_else(|| CcrError::ConfigError("无法获取配置目录".into()))?;
    let resolved = config_dir.join(requested);
    if resolved.parent() != Some(config_dir) {
        return Err(CcrError::ValidationError(
            "只能恢复当前 CCR 配置目录中的托管备份文件".into(),
        ));
    }
    let config_filename = config_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CcrError::ConfigError("无效的配置文件名".into()))?;
    let backup_filename = resolved
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| CcrError::ValidationError("无效的备份文件名".into()))?;
    if !backup_filename.starts_with(config_filename) || !backup_filename.ends_with(".bak") {
        return Err(CcrError::ValidationError(
            "只能恢复由 CCR 生成的当前配置备份文件".into(),
        ));
    }
    Ok(resolved)
}

/// Parse with the domain's source-safe parser before its locked preimage-backup/save.
pub fn restore(service: &ConfigService, backup_path: &Path) -> Result<()> {
    if !backup_path.exists() {
        return Err(CcrError::ConfigMissing(backup_path.display().to_string()));
    }
    let content = std::fs::read_to_string(backup_path)
        .map_err(|error| CcrError::FileIoError(format!("读取备份失败: {error}")))?;
    let config = parse_config_from_str(&content)?;
    service.restore_config_from_backup(&config)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackupCleanup {
    pub root_exists: bool,
    pub deleted: usize,
}

/// Explicit root; same extensions, depth and successful-deletion count as Tauri.
/// Never follow a directory symlink outside the managed root.
pub fn clean(root: &Path) -> BackupCleanup {
    let root_exists = root.exists();
    let mut deleted = 0;
    for entry in walkdir::WalkDir::new(root)
        .max_depth(3)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        if matches!(ext, "bak" | "backup") && std::fs::remove_file(path).is_ok() {
            deleted += 1;
        }
    }
    BackupCleanup {
        root_exists,
        deleted,
    }
}
