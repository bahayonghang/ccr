// 🧹 clean 命令实现 - 清理旧备份文件
// 📅 根据时间策略删除过期的 .bak 备份文件

use crate::error::{CcrError, Result};
use crate::logging::ColorOutput;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// 🧹 清理旧备份文件
/// 
/// 执行流程:
/// 1. 📁 扫描备份目录 (~/.claude/backups/)
/// 2. 🔍 识别 .bak 文件
/// 3. 📅 检查文件修改时间
/// 4. 🗑️ 删除超过指定天数的文件
/// 5. 📊 统计清理结果（文件数、释放空间）
/// 
/// 参数:
/// - days: 保留天数（删除 N 天前的文件）
/// - dry_run: 模拟运行（不实际删除）
pub fn clean_command(days: u64, dry_run: bool) -> Result<()> {
    ColorOutput::title("清理备份文件");
    println!();

    // 获取备份目录路径
    let backup_dir = get_backup_dir()?;
    
    if !backup_dir.exists() {
        ColorOutput::info("备份目录不存在，无需清理");
        return Ok(());
    }

    ColorOutput::info(&format!("备份目录: {}", backup_dir.display()));
    ColorOutput::info(&format!("清理策略: 删除 {} 天前的备份", days));
    
    if dry_run {
        ColorOutput::warning("⚠ 模拟运行模式（不会实际删除文件）");
    }
    
    println!();
    ColorOutput::separator();
    println!();

    // 扫描并清理备份文件
    let cutoff_time = SystemTime::now() - Duration::from_secs(days * 24 * 60 * 60);
    let result = scan_and_clean(&backup_dir, cutoff_time, dry_run)?;

    println!();
    ColorOutput::separator();
    println!();

    // 显示结果
    if result.deleted_count > 0 || result.skipped_count > 0 {
        ColorOutput::title("清理摘要");
        println!();
        
        if result.deleted_count > 0 {
            if dry_run {
                ColorOutput::info(&format!("将删除文件: {} 个", result.deleted_count));
            } else {
                ColorOutput::success(&format!("✓ 已删除文件: {} 个", result.deleted_count));
            }
        }
        
        if result.skipped_count > 0 {
            ColorOutput::info(&format!("保留文件: {} 个", result.skipped_count));
        }
        
        if result.total_size > 0 {
            let size_mb = result.total_size as f64 / 1024.0 / 1024.0;
            if dry_run {
                ColorOutput::info(&format!("将释放空间: {:.2} MB", size_mb));
            } else {
                ColorOutput::success(&format!("✓ 释放空间: {:.2} MB", size_mb));
            }
        }
    } else {
        ColorOutput::success("✓ 没有需要清理的文件");
    }

    if dry_run {
        println!();
        ColorOutput::info("提示: 运行 'ccr clean' (不带 --dry-run) 执行实际清理");
    }

    Ok(())
}

/// 获取备份目录路径
fn get_backup_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
    Ok(home.join(".claude").join("backups"))
}

/// 扫描并清理备份文件
fn scan_and_clean(backup_dir: &PathBuf, cutoff_time: SystemTime, dry_run: bool) -> Result<CleanResult> {
    let mut result = CleanResult {
        deleted_count: 0,
        skipped_count: 0,
        total_size: 0,
    };

    let entries = fs::read_dir(backup_dir)
        .map_err(|e| CcrError::ConfigError(format!("读取备份目录失败: {}", e)))?;

    for entry in entries {
        let entry = entry.map_err(|e| CcrError::ConfigError(format!("读取目录项失败: {}", e)))?;
        let path = entry.path();

        // 只处理备份文件（.bak 扩展名）
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("bak") {
            continue;
        }

        let metadata = fs::metadata(&path)
            .map_err(|e| CcrError::ConfigError(format!("读取文件元数据失败: {}", e)))?;

        let modified_time = metadata.modified()
            .map_err(|e| CcrError::ConfigError(format!("获取文件修改时间失败: {}", e)))?;

        if modified_time < cutoff_time {
            // 文件超过指定天数，需要删除
            let file_size = metadata.len();
            
            if dry_run {
                ColorOutput::warning(&format!("将删除: {}", path.file_name().unwrap().to_string_lossy()));
            } else {
                fs::remove_file(&path)
                    .map_err(|e| CcrError::ConfigError(format!("删除文件失败: {}", e)))?;
                ColorOutput::success(&format!("✓ 已删除: {}", path.file_name().unwrap().to_string_lossy()));
            }
            
            result.deleted_count += 1;
            result.total_size += file_size;
        } else {
            // 文件较新，保留
            result.skipped_count += 1;
        }
    }

    Ok(result)
}

/// 清理结果
struct CleanResult {
    deleted_count: usize,
    skipped_count: usize,
    total_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_scan_and_clean() {
        let temp_dir = tempfile::tempdir().unwrap();
        let backup_dir = temp_dir.path().to_path_buf();

        // 创建测试备份文件
        let old_file = backup_dir.join("old.bak");
        let new_file = backup_dir.join("new.bak");
        let other_file = backup_dir.join("other.txt");

        File::create(&old_file).unwrap().write_all(b"old").unwrap();
        File::create(&new_file).unwrap().write_all(b"new").unwrap();
        File::create(&other_file).unwrap().write_all(b"other").unwrap();

        // 设置旧文件的修改时间为 10 天前
        let old_time = SystemTime::now() - Duration::from_secs(10 * 24 * 60 * 60);
        filetime::set_file_mtime(&old_file, filetime::FileTime::from_system_time(old_time)).unwrap();

        // 清理 7 天前的文件（dry run）
        let cutoff = SystemTime::now() - Duration::from_secs(7 * 24 * 60 * 60);
        let result = scan_and_clean(&backup_dir, cutoff, true).unwrap();

        assert_eq!(result.deleted_count, 1); // old.bak 应该被标记删除
        assert_eq!(result.skipped_count, 1); // new.bak 应该被保留
        assert!(old_file.exists()); // dry run 不应实际删除

        // 实际清理
        let result = scan_and_clean(&backup_dir, cutoff, false).unwrap();
        assert_eq!(result.deleted_count, 1);
        assert!(!old_file.exists()); // 应该被删除
        assert!(new_file.exists()); // 应该保留
        assert!(other_file.exists()); // 非 .bak 文件应该保留
    }

    #[test]
    fn test_get_backup_dir() {
        let dir = get_backup_dir().unwrap();
        assert!(dir.to_string_lossy().contains(".claude"));
        assert!(dir.to_string_lossy().contains("backups"));
    }
}

