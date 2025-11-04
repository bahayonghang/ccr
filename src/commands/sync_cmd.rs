// ☁️ sync 命令实现 - WebDAV 配置同步
// 📁 支持配置文件的云端同步功能

use crate::commands::sync_content_selector::{SyncContentSelection, SyncContentSelector};
use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::managers::sync_config::{SyncConfig, SyncConfigManager};
use crate::services::{MultiBackupService, SyncService};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// ⚙️ 配置 WebDAV 同步
///
/// 交互式配置 WebDAV 连接信息
pub fn sync_config_command() -> Result<()> {
    ColorOutput::title("配置 WebDAV 同步");
    println!();

    ColorOutput::info("请输入 WebDAV 服务器信息");
    ColorOutput::info("💡 坚果云用户请使用应用密码，而非账户密码");
    println!();

    // 1. WebDAV 服务器地址
    let webdav_url =
        prompt_with_default("WebDAV 服务器地址", Some("https://dav.jianguoyun.com/dav/"))?;

    // 2. 用户名
    let username = prompt_required("用户名/邮箱", "例如: user@example.com")?;

    // 3. 密码（隐藏输入）
    println!();
    ColorOutput::info("密码/应用密码:");
    println!("  💡 坚果云: 账户信息 -> 安全选项 -> 添加应用 -> 生成密码");
    print!("  请输入: ");
    io::stdout().flush().unwrap();
    let password = read_password()?;
    println!();

    // 4. 远程路径
    let remote_path = prompt_with_default("远程目录路径", Some("/ccr/"))?;

    println!();
    ColorOutput::separator();
    println!();

    // 构建同步配置
    let sync_config = SyncConfig {
        enabled: true,
        webdav_url: webdav_url.clone(),
        username: username.clone(),
        password: password.clone(),
        remote_path: remote_path.clone(),
        auto_sync: false,
    };

    // 测试连接
    ColorOutput::step("测试 WebDAV 连接");
    println!();

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| CcrError::SyncError(format!("创建异步运行时失败: {}", e)))?;

    runtime.block_on(async {
        let service = SyncService::new(&sync_config).await?;
        service.test_connection().await?;
        Ok::<(), CcrError>(())
    })?;

    ColorOutput::success("✓ WebDAV 连接测试成功");
    println!();

    // 保存配置到独立的 sync.toml 文件
    ColorOutput::step("保存同步配置");
    let sync_manager = SyncConfigManager::with_default()?;
    sync_manager.save(&sync_config)?;

    ColorOutput::success("✓ 同步配置已保存");
    println!();

    ColorOutput::info("可用命令:");
    println!("  ccr sync status    # 查看同步状态");
    println!("  ccr sync push      # 上传配置到云端");
    println!("  ccr sync pull      # 从云端下载配置");
    println!();

    Ok(())
}

/// 📊 显示同步状态
pub fn sync_status_command() -> Result<()> {
    use colored::*;
    use comfy_table::{Attribute, Cell, Color as TableColor, Table};

    ColorOutput::title("☁️  WebDAV 同步状态");
    println!();

    let sync_manager = SyncConfigManager::with_default()?;
    let sync_config = sync_manager.load()?;

    if sync_config.enabled {
        // 使用 comfy-table 创建表格
        let mut table = Table::new();
        table.load_preset(comfy_table::presets::UTF8_FULL);
        table.set_header(vec![
            Cell::new("配置项").add_attribute(Attribute::Bold),
            Cell::new("值").add_attribute(Attribute::Bold),
        ]);

        // 状态行
        table.add_row(vec![
            Cell::new("状态"),
            Cell::new("✓ 已启用")
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold),
        ]);

        // WebDAV 服务器
        let url_display = if sync_config.webdav_url.len() > 50 {
            format!("{}...", &sync_config.webdav_url[..47])
        } else {
            sync_config.webdav_url.clone()
        };
        table.add_row(vec![Cell::new("WebDAV 服务器"), Cell::new(url_display)]);

        // 用户名
        table.add_row(vec![Cell::new("用户名"), Cell::new(&sync_config.username)]);

        // 密码（掩码）
        let masked_pwd = format!("{}...", &"*".repeat(8));
        table.add_row(vec![
            Cell::new("密码"),
            Cell::new(masked_pwd).fg(TableColor::DarkGrey),
        ]);

        // 远程路径
        table.add_row(vec![
            Cell::new("远程路径"),
            Cell::new(&sync_config.remote_path),
        ]);

        // 🆕 同步类型
        let sync_path = get_ccr_sync_path()?;
        let sync_type = if sync_path.is_dir() {
            "📁 目录同步"
        } else {
            "📄 文件同步"
        };
        table.add_row(vec![
            Cell::new("同步类型"),
            Cell::new(sync_type).fg(TableColor::Cyan),
        ]);

        // 🆕 本地路径
        table.add_row(vec![
            Cell::new("本地路径"),
            Cell::new(sync_path.display().to_string()),
        ]);

        // 自动同步
        let auto_sync_text = if sync_config.auto_sync {
            "✓ 开启"
        } else {
            "✗ 关闭"
        };
        let auto_sync_color = if sync_config.auto_sync {
            TableColor::Green
        } else {
            TableColor::DarkGrey
        };
        table.add_row(vec![
            Cell::new("自动同步"),
            Cell::new(auto_sync_text).fg(auto_sync_color),
        ]);

        println!("{}", table);
        println!();

        // 检查远程文件状态
        print!("🔍 正在检查远程状态...");
        std::io::Write::flush(&mut std::io::stdout()).unwrap();

        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| CcrError::SyncError(format!("创建异步运行时失败: {}", e)))?;

        let exists = runtime.block_on(async {
            let service = SyncService::new(&sync_config).await?;
            service.remote_exists().await
        })?;

        print!("\r");
        if exists {
            println!("{}  {}", "✓".green().bold(), "远程内容存在".green());
        } else {
            println!("{}  {}", "⚠".yellow().bold(), "远程内容不存在".yellow());
            println!("   💡 提示: 运行 {} 首次上传", "ccr sync push".cyan());
        }
        println!();
    } else {
        println!("{}  {}", "⚠".yellow().bold(), "同步功能未配置".yellow());
        println!();
        println!("📝 配置步骤:");
        println!("   1. 运行 {} 开始配置", "ccr sync config".cyan());
        println!("   2. 输入 WebDAV 服务器信息");
        println!("   3. 测试连接成功后即可使用");
        println!();
    }

    Ok(())
}

/// 🔼 上传配置到云端
///
/// # 参数
/// - force: 是否强制上传，跳过确认
/// - content_selection: 内容选择（可选），如果为None则显示选择面板
pub fn sync_push_command_with_selection(
    force: bool,
    content_selection: Option<SyncContentSelection>,
) -> Result<()> {
    use colored::*;

    ColorOutput::title("🔼  上传配置到云端");
    println!();

    // 🎯 获取内容选择
    let content_selection = if let Some(selection) = content_selection {
        selection
    } else {
        // 显示选择面板
        let mut selector = SyncContentSelector::new();
        selector.select_content()?
    };

    // 显示选择的内容
    if content_selection.count() > 0 {
        println!("{}  选择同步内容:", "📋".blue());
        for content_type in &content_selection.selected_types {
            println!("   • {}", content_type.display_name().cyan());
        }
        println!();
    }

    let sync_manager = SyncConfigManager::with_default()?;
    let sync_config = sync_manager.load()?;

    if !sync_config.enabled {
        return Err(CcrError::SyncError(
            "同步功能未配置，请先运行 'ccr sync config'".into(),
        ));
    }

    // 🏠 获取要同步的路径（目录或文件）
    let sync_path = get_ccr_sync_path()?;
    let is_dir = sync_path.is_dir();

    // 显示同步信息
    if is_dir {
        println!(
            "{}  {}",
            "📁".blue().bold(),
            format!("同步目录: {}", sync_path.display()).blue()
        );
    } else {
        println!(
            "{}  {}",
            "📄".blue().bold(),
            format!("同步文件: {}", sync_path.display()).blue()
        );
    }
    println!("   → 远程路径: {}", sync_config.remote_path.cyan());
    println!();

    // 检查远程文件/目录是否存在
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| CcrError::SyncError(format!("创建异步运行时失败: {}", e)))?;

    if !force {
        print!("🔍 正在检查远程状态...");
        io::stdout().flush().unwrap();

        let exists = runtime.block_on(async {
            let service = SyncService::new(&sync_config).await?;
            service.remote_exists().await
        })?;

        print!("\r");
        if exists {
            println!("{}  {}", "⚠".yellow().bold(), "远程已存在同名内容".yellow());
            println!();
            print!("   是否覆盖远程配置？ {} ", "(y/N):".dimmed());
            io::stdout().flush().unwrap();

            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).unwrap();

            if !confirm.trim().eq_ignore_ascii_case("y") {
                println!();
                println!("{}  已取消上传", "ℹ".blue().bold());
                return Ok(());
            }
            println!();
        } else {
            println!(
                "{}  {}",
                "ℹ".blue().bold(),
                "远程不存在，将创建新内容".blue()
            );
            println!();
        }
    }

    // 🧩 在上传前执行多类型增量备份（统一目录结构）
    {
        print!("💾 正在执行增量备份...");
        io::stdout().flush().unwrap();
        let svc = MultiBackupService::with_default()?;
        let summary = svc.backup_all()?;
        print!("\r");
        use colored::*;
        println!("{}  {}", "✓".green().bold(), "增量备份完成".green());
        let changed_count = summary.items.iter().filter(|i| i.changed).count();
        let skipped_count = summary.items.iter().filter(|i| !i.changed).count();
        println!("   • 变化项: {}", changed_count.to_string().cyan());
        println!("   • 跳过项: {}", skipped_count.to_string().cyan());
        if changed_count > 0 {
            println!(
                "   • 备份位置示例: {}",
                summary
                    .items
                    .iter()
                    .find(|i| i.changed)
                    .map(|i| i.target_path.display().to_string())
                    .unwrap_or_else(|| "~/.ccr/backups".to_string())
                    .dimmed()
            );
        }
        println!();
    }

    // 🎯 根据选择的内容类型过滤上传路径
    let filtered_paths = content_selection.to_paths();
    if filtered_paths.is_empty() {
        ColorOutput::warning("未选择任何同步内容，操作取消");
        return Ok(());
    }

    println!("{}  将同步以下内容:", "🎯".blue());
    for path in &filtered_paths {
        println!("   • {}", path.cyan());
    }
    println!();

    print!("🚀 正在上传...");
    io::stdout().flush().unwrap();

    // 🎯 根据内容选择创建临时过滤目录进行同步
    let temp_sync_path =
        if filtered_paths.len() == 1 && filtered_paths[0] == "config.toml" && !is_dir {
            // 如果是单个config文件且当前是文件模式，直接同步原文件
            sync_path.clone()
        } else {
            // 需要创建临时目录包含选中的内容
            create_temp_sync_directory(&sync_path, &filtered_paths, is_dir)?
        };

    print!("🚀 正在上传...");
    io::stdout().flush().unwrap();

    runtime.block_on(async {
        let service = SyncService::new(&sync_config).await?;
        service.push(&temp_sync_path, None).await?;
        Ok::<(), CcrError>(())
    })?;

    // 清理临时目录（如果需要）
    if temp_sync_path != sync_path {
        std::fs::remove_dir_all(&temp_sync_path).ok();
    }

    print!("\r");
    if is_dir {
        println!("{}  {}", "✓".green().bold(), "目录已成功上传到云端".green());
    } else {
        println!("{}  {}", "✓".green().bold(), "文件已成功上传到云端".green());
    }
    println!();
    println!("📊 同步信息:");
    println!("   • 本地路径: {}", sync_path.display().to_string().cyan());
    println!("   • 远程路径: {}", sync_config.remote_path.cyan());
    println!("   • 服务器: {}", sync_config.webdav_url.dimmed());
    println!();

    Ok(())
}

/// 🔽 从云端下载配置
pub fn sync_pull_command(force: bool) -> Result<()> {
    use colored::*;

    ColorOutput::title("🔽  从云端下载配置");
    println!();

    let sync_manager = SyncConfigManager::with_default()?;
    let sync_config = sync_manager.load()?;

    if !sync_config.enabled {
        return Err(CcrError::SyncError(
            "同步功能未配置，请先运行 'ccr sync config'".into(),
        ));
    }

    // 🏠 获取要同步的路径（目录或文件）
    let sync_path = get_ccr_sync_path()?;
    let is_dir = sync_path.is_dir();

    // 显示同步信息
    if is_dir {
        println!(
            "{}  {}",
            "📁".blue().bold(),
            format!("同步目录: {}", sync_path.display()).blue()
        );
    } else {
        println!(
            "{}  {}",
            "📄".blue().bold(),
            format!("同步文件: {}", sync_path.display()).blue()
        );
    }
    println!("   ← 远程路径: {}", sync_config.remote_path.cyan());
    println!();

    // 备份本地配置
    if !force {
        println!(
            "{}  {}",
            "⚠".yellow().bold(),
            "此操作将覆盖本地内容".yellow()
        );
        println!();
        print!("   是否继续？本地内容将被备份 {} ", "(y/N):".dimmed());
        io::stdout().flush().unwrap();

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm).unwrap();

        if !confirm.trim().eq_ignore_ascii_case("y") {
            println!();
            println!("{}  已取消下载", "ℹ".blue().bold());
            return Ok(());
        }
        println!();
    }

    // 🔍 检查远程是否存在（在备份前检查，避免不必要的备份）
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| CcrError::SyncError(format!("创建异步运行时失败: {}", e)))?;

    let remote_exists = runtime.block_on(async {
        let service = SyncService::new(&sync_config).await?;
        service.remote_exists().await
    })?;

    if !remote_exists {
        println!();
        ColorOutput::error("远程目录不存在");
        println!();
        println!("   💡 提示: 首次使用需要先上传配置到云端");
        println!("   运行命令: {}", "ccr sync push".cyan());
        println!();
        return Err(CcrError::SyncError("远程内容不存在".to_string()));
    }

    // 备份逻辑
    if sync_path.exists() {
        print!("💾 正在备份本地内容...");
        io::stdout().flush().unwrap();

        // 如果是文件，使用 ConfigManager 的备份功能
        // 如果是目录，创建带时间戳的 .bak 备份
        let backup_path = if is_dir {
            // 🏷️ 生成带时间戳的备份目录名，避免冲突
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let backup_name = format!("{}.{}.bak", sync_path.display(), timestamp);
            let backup = PathBuf::from(backup_name);

            // 🔄 如果目标备份路径已存在（极少见），先删除
            if backup.exists() {
                std::fs::remove_dir_all(&backup)
                    .map_err(|e| CcrError::SyncError(format!("删除旧备份失败: {}", e)))?;
            }

            // 📦 移动目录到备份位置（原子操作）
            std::fs::rename(&sync_path, &backup)
                .map_err(|e| CcrError::SyncError(format!("备份目录失败: {}", e)))?;
            backup
        } else {
            // 对于单个配置文件，使用 ConfigManager 的备份功能
            use crate::managers::config::ConfigManager;
            let config_manager = ConfigManager::with_default()?;
            config_manager.backup(Some("before_pull"))?
        };

        print!("\r");
        println!("{}  {}", "✓".green().bold(), "本地内容已备份".green());
        println!(
            "   📁 备份位置: {}",
            backup_path.display().to_string().dimmed()
        );
        println!();
    }

    // 🧩 在拉取前执行多类型增量备份（统一目录结构）
    {
        print!("💾 正在执行增量备份...");
        io::stdout().flush().unwrap();
        let svc = MultiBackupService::with_default()?;
        let summary = svc.backup_all()?;
        print!("\r");
        use colored::*;
        println!("{}  {}", "✓".green().bold(), "增量备份完成".green());
        let changed_count = summary.items.iter().filter(|i| i.changed).count();
        let skipped_count = summary.items.iter().filter(|i| !i.changed).count();
        println!("   • 变化项: {}", changed_count.to_string().cyan());
        println!("   • 跳过项: {}", skipped_count.to_string().cyan());
        println!();
    }

    print!("⬇️  正在从云端下载...");
    io::stdout().flush().unwrap();

    runtime.block_on(async {
        let service = SyncService::new(&sync_config).await?;
        service.pull(&sync_path).await?;
        Ok::<(), CcrError>(())
    })?;

    print!("\r");
    if is_dir {
        println!(
            "{}  {}",
            "✓".green().bold(),
            "目录已从云端下载并应用".green()
        );
    } else {
        println!(
            "{}  {}",
            "✓".green().bold(),
            "文件已从云端下载并应用".green()
        );
    }
    println!();
    println!("📊 同步信息:");
    println!("   • 本地路径: {}", sync_path.display().to_string().cyan());
    println!("   • 远程路径: {}", sync_config.remote_path.cyan());
    println!("   • 服务器: {}", sync_config.webdav_url.dimmed());
    println!();
    println!("💡 下一步: 运行 {} 查看配置", "ccr list".cyan());
    println!();

    Ok(())
}

/// 🔼 上传配置到云端（向后兼容接口）
///
/// 默认只同步config内容，保持与现有行为一致
pub fn sync_push_command(force: bool) -> Result<()> {
    // 使用默认选择（仅config）
    let default_selection = SyncContentSelection::default();
    sync_push_command_with_selection(force, Some(default_selection))
}

/// 🎯 创建临时同步目录（包含选中的内容）
fn create_temp_sync_directory(
    original_path: &Path,
    filtered_paths: &[String],
    is_dir: bool,
) -> Result<PathBuf> {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().map_err(|e| {
        CcrError::IoError(std::io::Error::other(format!("创建临时目录失败: {}", e)))
    })?;

    let temp_path = temp_dir.path().to_path_buf();

    // 保留临时目录的所有权，防止被自动删除
    let temp_path_clone = temp_path.clone();
    std::mem::forget(temp_dir);

    if is_dir {
        // 如果是目录模式，复制选中的内容到临时目录
        let base_path = original_path;

        for filtered_path in filtered_paths {
            let source_path = base_path.join(filtered_path);
            if source_path.exists() {
                let target_path = temp_path.join(filtered_path);

                if source_path.is_dir() {
                    copy_directory_recursive(&source_path, &target_path)?;
                } else {
                    if let Some(parent) = target_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::copy(&source_path, &target_path)?;
                }
            }
        }
    } else {
        // 如果是文件模式，创建目录结构并复制选中的文件
        let base_path = original_path
            .parent()
            .ok_or_else(|| CcrError::SyncError("无法获取配置文件父目录".into()))?;

        for filtered_path in filtered_paths {
            let source_path = if filtered_path == "config.toml" {
                original_path.to_path_buf()
            } else {
                base_path.join(filtered_path)
            };

            if source_path.exists() {
                let target_path = temp_path.join(filtered_path);

                if let Some(parent) = target_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                if source_path.is_dir() {
                    copy_directory_recursive(&source_path, &target_path)?;
                } else {
                    std::fs::copy(&source_path, &target_path)?;
                }
            }
        }
    }

    Ok(temp_path_clone)
}

/// 📁 递归复制目录
fn copy_directory_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let target_path = dst.join(&file_name);

        if path.is_dir() {
            copy_directory_recursive(&path, &target_path)?;
        } else {
            std::fs::copy(&path, &target_path)?;
        }
    }

    Ok(())
}

// === 辅助函数 ===

/// 🏠 获取 CCR 根目录路径
///
/// 优先级：
/// 1. CCR_ROOT 环境变量
/// 2. ~/.ccr/ (统一模式)
/// 3. 回退到使用配置文件路径（兼容旧版）
fn get_ccr_sync_path() -> Result<PathBuf> {
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

    // 3. 回退到配置文件（Legacy 模式）
    // 这种情况下我们同步单个配置文件
    let home =
        dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
    Ok(home.join(".ccs_config.toml"))
}

/// 必填字段提示
fn prompt_required(field_name: &str, example: &str) -> Result<String> {
    loop {
        println!();
        ColorOutput::info(&format!("{} *", field_name));
        println!("  例如: {}", example);
        print!("  请输入: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| CcrError::ConfigError(format!("读取输入失败: {}", e)))?;

        let trimmed = input.trim();
        if !trimmed.is_empty() {
            println!();
            return Ok(trimmed.to_string());
        }

        ColorOutput::error("此字段为必填项，不能为空");
        println!();
    }
}

/// 带默认值的提示
fn prompt_with_default(field_name: &str, default: Option<&str>) -> Result<String> {
    println!();
    ColorOutput::info(field_name);
    if let Some(def) = default {
        println!("  默认: {}", def);
    }
    print!("  请输入: ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| CcrError::ConfigError(format!("读取输入失败: {}", e)))?;

    let trimmed = input.trim();
    println!();

    if trimmed.is_empty()
        && let Some(def) = default
    {
        return Ok(def.to_string());
    }

    Ok(trimmed.to_string())
}

/// 读取密码（隐藏输入）
fn read_password() -> Result<String> {
    // 简化版：直接读取（后续可以集成 rpassword crate）
    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .map_err(|e| CcrError::ConfigError(format!("读取密码失败: {}", e)))?;

    Ok(password.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_config_creation() {
        let config = SyncConfig {
            enabled: true,
            webdav_url: "https://dav.jianguoyun.com/dav/".to_string(),
            username: "test@example.com".to_string(),
            password: "test_password".to_string(),
            remote_path: "/ccr/".to_string(), // 🆕 改为目录路径
            auto_sync: false,
        };

        assert!(config.enabled);
        assert_eq!(config.webdav_url, "https://dav.jianguoyun.com/dav/");
        assert_eq!(config.remote_path, "/ccr/");
    }
}
