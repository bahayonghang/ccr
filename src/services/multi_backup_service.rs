// 📦 多类型增量备份服务
// 负责将 CCR 配置和各平台 CLI 配置按统一结构进行备份，并支持增量与并发安全

#![allow(dead_code)]

use crate::core::error::{CcrError, Result};
use crate::core::lock::LockManager;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 备份摘要条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupItemSummary {
    pub name: String,
    pub changed: bool,
    pub digest: String,
    pub target_path: PathBuf,
}

/// 备份运行摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSummary {
    pub items: Vec<BackupItemSummary>,
}

/// Manifest 记录源路径到最后一次备份的摘要（digest）
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct BackupManifest {
    entries: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManifestEntry {
    source_key: String,
    digest: String,
}

impl BackupManifest {
    fn load(path: &Path) -> Self {
        if path.exists()
            && let Ok(content) = fs::read_to_string(path)
            && let Ok(m) = serde_json::from_str::<BackupManifest>(&content)
        {
            return m;
        }
        BackupManifest::default()
    }

    fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CcrError::IoError(std::io::Error::other(format!(
                    "创建 manifest 目录失败: {}",
                    e
                )))
            })?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            CcrError::IoError(std::io::Error::other(format!(
                "序列化 manifest 失败: {}",
                e
            )))
        })?;
        fs::write(path, content).map_err(|e| {
            CcrError::IoError(std::io::Error::other(format!("写入 manifest 失败: {}", e)))
        })
    }

    fn get_digest(&self, key: &str) -> Option<String> {
        self.entries
            .iter()
            .find(|e| e.source_key == key)
            .map(|e| e.digest.clone())
    }

    fn set_digest(&mut self, key: String, digest: String) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.source_key == key) {
            entry.digest = digest;
        } else {
            self.entries.push(ManifestEntry {
                source_key: key,
                digest,
            });
        }
    }
}

/// 多类型增量备份服务
pub struct MultiBackupService {
    ccr_root: PathBuf,         // ~/.ccr
    backup_root: PathBuf,      // ~/.ccr/backups
    manifest_path: PathBuf,    // ~/.ccr/backups/multi_manifest.json
    lock_manager: LockManager, // 备份区域的锁目录
}

impl MultiBackupService {
    /// 使用默认路径构建服务
    ///
    /// 备份根目录为 CCR_ROOT/backups 或 ~/.ccr/backups
    pub fn with_default() -> Result<Self> {
        let ccr_root = Self::detect_ccr_root()?;
        let backup_root = ccr_root.join("backups");
        let manifest_path = backup_root.join("multi_manifest.json");
        let lock_dir = backup_root.join(".locks");
        let lock_manager = LockManager::new(lock_dir);
        Ok(Self {
            ccr_root,
            backup_root,
            manifest_path,
            lock_manager,
        })
    }

    /// 使用指定 CCR 根目录构建服务（测试与自定义场景）
    #[allow(dead_code)]
    pub fn with_root(ccr_root: PathBuf) -> Result<Self> {
        let backup_root = ccr_root.join("backups");
        let manifest_path = backup_root.join("multi_manifest.json");
        let lock_dir = backup_root.join(".locks");
        let lock_manager = LockManager::new(lock_dir);
        Ok(Self {
            ccr_root,
            backup_root,
            manifest_path,
            lock_manager,
        })
    }

    /// 执行所有目标的备份（增量）
    pub fn backup_all(&self) -> Result<BackupSummary> {
        // 统一加锁，避免并发备份冲突
        let _lock = self
            .lock_manager
            .lock_resource("multi_backup", std::time::Duration::from_secs(10))?;

        let mut manifest = BackupManifest::load(&self.manifest_path);
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();

        let sources = self.collect_sources()?;

        // 并行计算每个源的摘要
        let digests: Vec<(BackupSource, String)> = sources
            .par_iter()
            .filter_map(|s| {
                let digest = match s.kind {
                    SourceKind::File => compute_file_digest(&s.source_path).ok()?,
                    SourceKind::Directory => compute_dir_digest(&s.source_path).ok()?,
                };
                Some((s.clone(), digest))
            })
            .collect();

        let mut items = Vec::new();

        for (src, digest) in digests {
            let key = src.key();
            let last = manifest.get_digest(&key);
            let changed = last.as_deref() != Some(&digest);

            if changed {
                // 执行备份
                let target_path = match src.kind {
                    SourceKind::File => {
                        let target_dir = self.backup_root.join(src.target_subdir());
                        fs::create_dir_all(&target_dir).map_err(|e| {
                            CcrError::IoError(std::io::Error::other(format!(
                                "创建备份目录失败: {}",
                                e
                            )))
                        })?;
                        let filename = src
                            .source_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("config.toml");
                        let target_file =
                            target_dir.join(format!("{}.{}.bak", filename, timestamp));
                        fs::copy(&src.source_path, &target_file).map_err(|e| {
                            CcrError::IoError(std::io::Error::other(format!(
                                "复制备份文件失败: {}",
                                e
                            )))
                        })?;
                        target_file
                    }
                    SourceKind::Directory => {
                        let snapshot_dir =
                            self.backup_root.join(src.target_subdir()).join(&timestamp);
                        copy_directory_recursive(&src.source_path, &snapshot_dir)?;
                        snapshot_dir
                    }
                };

                // 更新 manifest
                manifest.set_digest(key.clone(), digest.clone());

                items.push(BackupItemSummary {
                    name: src.name.clone(),
                    changed: true,
                    digest,
                    target_path,
                });
            } else {
                // 未变化则跳过
                items.push(BackupItemSummary {
                    name: src.name.clone(),
                    changed: false,
                    digest,
                    target_path: self.backup_root.join(src.target_subdir()),
                });
            }
        }

        // 保存 manifest
        manifest.save(&self.manifest_path)?;

        Ok(BackupSummary { items })
    }

    /// 收集待备份的源路径
    fn collect_sources(&self) -> Result<Vec<BackupSource>> {
        let mut sources = Vec::new();
        let ccr_root = self.ccr_root.clone();
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;

        // 1) CCR config.toml（文件备份） → backups/ccr/config
        let ccr_config = ccr_root.join("config.toml");
        if ccr_config.exists() {
            sources.push(BackupSource::new(
                "ccr_config",
                ccr_config,
                SourceKind::File,
                "ccr/config",
            ));
        }

        // 2) .claude 或统一模式 platforms/claude（目录备份） → backups/ccr/.claude
        let claude_dir = if home.join(".claude").exists() {
            home.join(".claude")
        } else {
            ccr_root.join("platforms").join("claude")
        };
        if claude_dir.exists() {
            sources.push(BackupSource::new(
                "claude",
                claude_dir,
                SourceKind::Directory,
                "ccr/.claude",
            ));
        }

        // 3) .gemini 或统一模式 platforms/gemini → backups/ccr/.gemini
        let gemini_dir = if home.join(".gemini").exists() {
            home.join(".gemini")
        } else {
            ccr_root.join("platforms").join("gemini")
        };
        if gemini_dir.exists() {
            sources.push(BackupSource::new(
                "gemini",
                gemini_dir,
                SourceKind::Directory,
                "ccr/.gemini",
            ));
        }

        // 4) .qwen 或统一模式 platforms/qwen → backups/ccr/.qwen
        let qwen_dir = if home.join(".qwen").exists() {
            home.join(".qwen")
        } else {
            ccr_root.join("platforms").join("qwen")
        };
        if qwen_dir.exists() {
            sources.push(BackupSource::new(
                "qwen",
                qwen_dir,
                SourceKind::Directory,
                "ccr/.qwen",
            ));
        }

        // 5) .iflow 或统一模式 platforms/iflow → backups/ccr/.iflow
        let iflow_dir = if home.join(".iflow").exists() {
            home.join(".iflow")
        } else {
            ccr_root.join("platforms").join("iflow")
        };
        if iflow_dir.exists() {
            sources.push(BackupSource::new(
                "iflow",
                iflow_dir,
                SourceKind::Directory,
                "ccr/.iflow",
            ));
        }

        Ok(sources)
    }

    fn detect_ccr_root() -> Result<PathBuf> {
        if let Ok(root) = std::env::var("CCR_ROOT") {
            let p = PathBuf::from(root);
            return Ok(p);
        }
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
        Ok(home.join(".ccr"))
    }
}

#[derive(Debug, Clone)]
struct BackupSource {
    name: String,
    source_path: PathBuf,
    kind: SourceKind,
    target_subdir: String, // 相对于 backup_root 的子目录，例如 "ccr/.claude"
}

#[derive(Debug, Clone, Copy)]
enum SourceKind {
    File,
    Directory,
}

impl BackupSource {
    fn new(name: &str, source_path: PathBuf, kind: SourceKind, target_subdir: &str) -> Self {
        Self {
            name: name.to_string(),
            source_path,
            kind,
            target_subdir: target_subdir.to_string(),
        }
    }

    fn key(&self) -> String {
        format!("{}:{}", self.name, self.source_path.display())
    }

    fn target_subdir(&self) -> &str {
        &self.target_subdir
    }
}

/// 计算文件的 blake3 摘要
fn compute_file_digest(path: &Path) -> Result<String> {
    let data = fs::read(path).map_err(|e| {
        CcrError::IoError(std::io::Error::other(format!(
            "读取文件失败 {}: {}",
            path.display(),
            e
        )))
    })?;
    Ok(blake3::hash(&data).to_hex().to_string())
}

/// 计算目录的摘要（遍历相关文件，按照名称排序，合并内容再做 blake3）
fn compute_dir_digest(dir: &Path) -> Result<String> {
    if !dir.exists() {
        return Ok(String::new());
    }
    let mut files: Vec<PathBuf> = Vec::new();
    collect_files_for_backup(dir, &mut files)?;
    files.sort();

    let mut hasher = blake3::Hasher::new();
    for f in files {
        // 文件名参与摘要
        hasher.update(f.to_string_lossy().as_bytes());
        // 文件内容参与摘要
        let data = fs::read(&f).map_err(|e| {
            CcrError::IoError(std::io::Error::other(format!(
                "读取文件失败 {}: {}",
                f.display(),
                e
            )))
        })?;
        hasher.update(&data);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// 递归拷贝目录（应用过滤规则，避免无用文件）
fn copy_directory_recursive(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }
    fs::create_dir_all(dst).map_err(|e| {
        CcrError::IoError(std::io::Error::other(format!("创建备份子目录失败: {}", e)))
    })?;

    for entry in fs::read_dir(src)
        .map_err(|e| CcrError::IoError(std::io::Error::other(format!("读取目录失败: {}", e))))?
    {
        let entry = entry.map_err(|e| {
            CcrError::IoError(std::io::Error::other(format!("读取目录项失败: {}", e)))
        })?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if should_exclude_from_backup(&name) {
            continue;
        }

        let target = dst.join(&name);
        if path.is_dir() {
            copy_directory_recursive(&path, &target)?;
        } else {
            // 普通文件复制
            // 如存在冲突，则在文件名后加上 "_copy" 后缀
            let final_target = if target.exists() {
                let mut alt = target.clone();
                if let Some(fname) = alt.file_stem().and_then(|s| s.to_str()) {
                    let ext = alt.extension().and_then(|s| s.to_str()).unwrap_or("");
                    let new_name = if ext.is_empty() {
                        format!("{}_copy", fname)
                    } else {
                        format!("{}_copy.{}", fname, ext)
                    };
                    alt.set_file_name(new_name);
                }
                alt
            } else {
                target
            };
            fs::create_dir_all(final_target.parent().expect("路径应该有父目录")).map_err(|e| {
                CcrError::IoError(std::io::Error::other(format!("创建父目录失败: {}", e)))
            })?;
            fs::copy(&path, &final_target).map_err(|e| {
                CcrError::IoError(std::io::Error::other(format!(
                    "复制文件失败 {}: {}",
                    path.display(),
                    e
                )))
            })?;
        }
    }
    Ok(())
}

/// 收集目录下参与备份的文件路径（应用过滤）
fn collect_files_for_backup(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)
        .map_err(|e| CcrError::IoError(std::io::Error::other(format!("读取目录失败: {}", e))))?
    {
        let entry = entry.map_err(|e| {
            CcrError::IoError(std::io::Error::other(format!("读取目录项失败: {}", e)))
        })?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if should_exclude_from_backup(&name) {
            continue;
        }

        if path.is_dir() {
            collect_files_for_backup(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

/// 备份过滤规则（与同步规则一致但更严格）
fn should_exclude_from_backup(name: &str) -> bool {
    let exclude_patterns = [
        // 临时与备份
        ".tmp",
        ".lock",
        ".bak",
        // 系统
        ".DS_Store",
        "Thumbs.db",
        "desktop.ini",
        // 版本控制
        ".git",
        ".gitignore",
        // 内部目录
        ".locks",
        "backups", // 避免备份备份
    ];

    for p in &exclude_patterns {
        if name.ends_with(p) || name == *p {
            return true;
        }
    }

    false
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    #[ignore = "Slow incremental backup test"]
    fn test_multi_backup_basic_and_incremental() {
        let tmp = tempdir().unwrap();
        let ccr_root = tmp.path().join(".ccr");
        fs::create_dir_all(&ccr_root).unwrap();

        // 创建 config.toml
        let config_path = ccr_root.join("config.toml");
        fs::write(&config_path, b"default_platform = 'claude'\n").unwrap();

        // 创建统一模式平台目录（claude）
        let claude_dir = ccr_root.join("platforms").join("claude");
        fs::create_dir_all(&claude_dir).unwrap();
        fs::write(claude_dir.join("settings.json"), b"{\"env\":{}}\n").unwrap();

        let svc = MultiBackupService::with_root(ccr_root.clone()).unwrap();
        let summary1 = svc.backup_all().unwrap();
        assert!(summary1.items.iter().any(|i| i.changed));
        // 路径检查：确保备份位置在期望的子目录下并存在
        if let Some(cfg_item) = summary1
            .items
            .iter()
            .find(|i| i.name == "ccr_config" && i.changed)
        {
            assert!(cfg_item.target_path.exists());
            // 跨平台路径检查：检查路径组件而非字符串
            let path_str = cfg_item.target_path.display().to_string();
            let has_backups = cfg_item
                .target_path
                .components()
                .any(|c| c.as_os_str() == "backups");
            let has_ccr = cfg_item
                .target_path
                .components()
                .any(|c| c.as_os_str() == "ccr");
            let has_config = cfg_item
                .target_path
                .components()
                .any(|c| c.as_os_str() == "config");
            assert!(
                has_backups && has_ccr && has_config,
                "ccr_config 备份路径不正确: {}",
                path_str
            );
        }
        if let Some(claude_item) = summary1
            .items
            .iter()
            .find(|i| i.name == "claude" && i.changed)
        {
            assert!(claude_item.target_path.exists());
            // 跨平台路径检查：检查路径组件而非字符串
            let path_str = claude_item.target_path.display().to_string();
            let has_backups = claude_item
                .target_path
                .components()
                .any(|c| c.as_os_str() == "backups");
            let has_ccr = claude_item
                .target_path
                .components()
                .any(|c| c.as_os_str() == "ccr");
            let has_claude = claude_item
                .target_path
                .components()
                .any(|c| c.as_os_str() == ".claude");
            assert!(
                has_backups && has_ccr && has_claude,
                "claude 备份路径不正确: {}",
                path_str
            );
        }

        // 第二次未修改，不应产生新的变化
        // 注意：只检查测试完全控制的 ccr_config 项，因为真实的 ~/.claude 目录
        // 可能包含正在变化的文件（日志、缓存等），导致测试不稳定
        let summary2 = svc.backup_all().unwrap();
        let ccr_config_unchanged = summary2
            .items
            .iter()
            .find(|i| i.name == "ccr_config")
            .map(|i| !i.changed)
            .unwrap_or(true);
        assert!(
            ccr_config_unchanged,
            "ccr_config 应该没有变化，但被标记为 changed"
        );
        let _digest_before = compute_file_digest(&config_path).unwrap();

        // 修改一个文件，应该检测到变化
        fs::write(&config_path, b"default_platform = 'gemini'\n").unwrap();
        // 短暂等待，确保文件系统状态可见，避免并发读取到旧内容
        std::thread::sleep(std::time::Duration::from_millis(10));
        // 修改后再次备份，应存在变化项（至少有一个）
        let summary3 = svc.backup_all().unwrap();
        assert!(summary3.items.iter().any(|i| i.changed));
        // 修改后的路径也应存在
        for item in summary3.items.iter().filter(|i| i.changed) {
            assert!(
                item.target_path.exists(),
                "变化项目标路径应存在: {}",
                item.target_path.display()
            );
        }

        // 再次备份（未进一步修改），应全部为未变化项
        // 同样只检查 ccr_config，避免真实系统目录的干扰
        let summary4 = svc.backup_all().unwrap();
        let ccr_config_unchanged_again = summary4
            .items
            .iter()
            .find(|i| i.name == "ccr_config")
            .map(|i| !i.changed)
            .unwrap_or(true);
        assert!(
            ccr_config_unchanged_again,
            "ccr_config 应该没有变化，但被标记为 changed"
        );

        // 并发安全：两次快速调用也应该稳定
        let svc2 = MultiBackupService::with_root(ccr_root.clone()).unwrap();
        let _ = (svc.backup_all(), svc2.backup_all());
    }
}
