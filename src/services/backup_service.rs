// 🧹 备份服务
// 封装备份清理相关的业务逻辑

use crate::core::error::{CcrError, Result};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// 🧹 清理结果
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CleanResult {
    pub deleted_count: usize,
    pub skipped_count: usize,
    pub total_size: u64,
}

/// 📦 备份文件信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BackupFileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
}

/// 🧹 备份服务
///
/// 封装备份文件清理相关的业务逻辑
#[allow(dead_code)]
pub struct BackupService {
    backup_dir: PathBuf,
}

#[allow(dead_code)]
impl BackupService {
    /// 🏗️ 创建新的备份服务
    ///
    /// # Arguments
    /// - `backup_dir` - 备份目录路径
    pub fn new(backup_dir: PathBuf) -> Self {
        Self { backup_dir }
    }

    /// 🏠 使用默认备份目录创建服务
    ///
    /// 默认目录: ~/.claude/backups
    pub fn default() -> Result<Self> {
        let home =
            dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
        let backup_dir = home.join(".claude").join("backups");
        Ok(Self::new(backup_dir))
    }

    /// 🧹 清理旧备份文件
    ///
    /// # Arguments
    /// - `days` - 保留天数(删除 N 天前的文件)
    /// - `dry_run` - 模拟运行(不实际删除)
    ///
    /// # Returns
    /// 清理结果(删除数量、跳过数量、释放空间)
    pub fn clean_old_backups(&self, days: u64, dry_run: bool) -> Result<CleanResult> {
        if !self.backup_dir.exists() {
            return Ok(CleanResult {
                deleted_count: 0,
                skipped_count: 0,
                total_size: 0,
            });
        }

        let cutoff_time = SystemTime::now() - Duration::from_secs(days * 24 * 60 * 60);
        self.scan_and_clean(cutoff_time, dry_run)
    }

    /// 📂 扫描备份目录
    ///
    /// # Returns
    /// 所有 .bak 文件的信息列表
    ///
    /// 🎯 优化：使用 rayon 并行扫描文件，提升大量备份文件时的性能
    pub fn scan_backup_directory(&self) -> Result<Vec<BackupFileInfo>> {
        use rayon::prelude::*;

        if !self.backup_dir.exists() {
            return Ok(Vec::new());
        }

        // 🚀 收集所有目录项
        let entries: Vec<_> = fs::read_dir(&self.backup_dir)
            .map_err(|e| CcrError::ConfigError(format!("读取备份目录失败: {}", e)))?
            .filter_map(|e| e.ok())
            .collect();

        // 🚀 并行处理每个文件，收集备份信息
        let mut backups: Vec<BackupFileInfo> = entries
            .par_iter()
            .filter_map(|entry| {
                let path = entry.path();

                // 只处理 .bak 文件
                if !path.is_file() || path.extension()?.to_str()? != "bak" {
                    return None;
                }

                let metadata = fs::metadata(&path).ok()?;
                let modified = metadata.modified().ok()?;

                Some(BackupFileInfo {
                    path,
                    size: metadata.len(),
                    modified,
                })
            })
            .collect();

        // 按修改时间倒序排列(最新的在前)
        // 注意：排序仍需串行，但扫描部分已并行化
        backups.sort_by(|a, b| b.modified.cmp(&a.modified));

        Ok(backups)
    }

    /// 🔍 扫描并清理备份文件
    fn scan_and_clean(&self, cutoff_time: SystemTime, dry_run: bool) -> Result<CleanResult> {
        let mut result = CleanResult {
            deleted_count: 0,
            skipped_count: 0,
            total_size: 0,
        };

        let entries = fs::read_dir(&self.backup_dir)
            .map_err(|e| CcrError::ConfigError(format!("读取备份目录失败: {}", e)))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| CcrError::ConfigError(format!("读取目录项失败: {}", e)))?;
            let path = entry.path();

            // 只处理 .bak 文件
            if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("bak") {
                continue;
            }

            let metadata = fs::metadata(&path)
                .map_err(|e| CcrError::ConfigError(format!("读取文件元数据失败: {}", e)))?;

            let modified_time = metadata
                .modified()
                .map_err(|e| CcrError::ConfigError(format!("获取文件修改时间失败: {}", e)))?;

            if modified_time < cutoff_time {
                // 文件超过指定天数,需要删除
                let file_size = metadata.len();

                if !dry_run {
                    fs::remove_file(&path)
                        .map_err(|e| CcrError::ConfigError(format!("删除文件失败: {}", e)))?;
                }

                result.deleted_count += 1;
                result.total_size += file_size;
            } else {
                // 文件较新,保留
                result.skipped_count += 1;
            }
        }

        Ok(result)
    }

    /// 📁 获取备份目录路径
    pub fn backup_dir(&self) -> &PathBuf {
        &self.backup_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_backup_service_clean() {
        let temp_dir = tempdir().unwrap();
        let backup_dir = temp_dir.path().to_path_buf();

        // 创建测试备份文件
        let old_file = backup_dir.join("old.bak");
        let new_file = backup_dir.join("new.bak");
        let other_file = backup_dir.join("other.txt");

        File::create(&old_file).unwrap().write_all(b"old").unwrap();
        File::create(&new_file).unwrap().write_all(b"new").unwrap();
        File::create(&other_file)
            .unwrap()
            .write_all(b"other")
            .unwrap();

        // 设置旧文件的修改时间为 10 天前
        let old_time = SystemTime::now() - Duration::from_secs(10 * 24 * 60 * 60);
        filetime::set_file_mtime(&old_file, filetime::FileTime::from_system_time(old_time))
            .unwrap();

        let service = BackupService::new(backup_dir);

        // 清理 7 天前的文件(dry run)
        let result = service.clean_old_backups(7, true).unwrap();
        assert_eq!(result.deleted_count, 1); // old.bak 应该被标记删除
        assert_eq!(result.skipped_count, 1); // new.bak 应该被保留
        assert!(old_file.exists()); // dry run 不应实际删除

        // 实际清理
        let result = service.clean_old_backups(7, false).unwrap();
        assert_eq!(result.deleted_count, 1);
        assert!(!old_file.exists()); // 应该被删除
        assert!(new_file.exists()); // 应该保留
        assert!(other_file.exists()); // 非 .bak 文件应该保留
    }

    #[test]
    fn test_backup_service_scan() {
        let temp_dir = tempdir().unwrap();
        let backup_dir = temp_dir.path().to_path_buf();

        // 创建多个备份文件
        for i in 0..5 {
            let filename = format!("backup{}.bak", i);
            File::create(backup_dir.join(&filename))
                .unwrap()
                .write_all(format!("content{}", i).as_bytes())
                .unwrap();
        }

        let service = BackupService::new(backup_dir);
        let backups = service.scan_backup_directory().unwrap();

        assert_eq!(backups.len(), 5);
        // 验证按修改时间排序
        for i in 0..backups.len() - 1 {
            assert!(backups[i].modified >= backups[i + 1].modified);
        }
    }
}
