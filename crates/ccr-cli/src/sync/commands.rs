// ☁️ sync 命令实现 - WebDAV 配置同步
// 📁 支持配置文件的云端同步功能

use crate::commands::common::new_utf8_table;
use crate::services::MultiBackupService;
use ccr_core::core::error::{CcrError, Result};
use ccr_core::core::logging::ColorOutput;
use ccr_sync::{SyncConfig, SyncConfigManager};
use ccr_sync::{
    SyncContentSelection, SyncContentSelector, SyncFolder, SyncFolderManager, SyncService,
};
use colored::Colorize;
use comfy_table::{Cell, CellAlignment, Color, ColumnConstraint, ContentArrangement};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// 📖 显示 sync 命令的详细帮助信息
fn show_sync_help() {
    use colored::*;

    println!();
    println!("{}", "☁️  CCR Sync 命令帮助".cyan().bold());
    println!();
    println!("{}", "━".repeat(70).dimmed());
    println!();

    println!("{}", "概述:".green().bold());
    println!("  WebDAV 多文件夹同步管理 - 支持独立管理和同步多个目录");
    println!();

    println!("{}", "基本命令:".green().bold());
    println!("  {}  配置 WebDAV 连接信息", "ccr sync config".cyan());
    println!(
        "  {}  显示当前同步状态和所有文件夹",
        "ccr sync status".cyan()
    );
    println!("  {}  上传配置文件到云端", "ccr sync push".cyan());
    println!("  {}  从云端下载配置文件", "ccr sync pull".cyan());
    println!("  {}  显示此帮助信息", "ccr sync help".cyan());
    println!();

    println!("{}", "文件夹管理命令:".green().bold());
    println!(
        "  {}      列出所有注册的同步文件夹",
        "ccr sync folder list".cyan()
    );
    println!(
        "  {}  添加新的同步文件夹",
        "ccr sync folder add <name> <path>".cyan()
    );
    println!(
        "  {}    删除文件夹注册 (不删除本地文件)",
        "ccr sync folder remove <name>".cyan()
    );
    println!(
        "  {}      显示文件夹详细信息",
        "ccr sync folder info <name>".cyan()
    );
    println!(
        "  {}    启用文件夹同步",
        "ccr sync folder enable <name>".cyan()
    );
    println!(
        "  {}   禁用文件夹同步 (保留配置)",
        "ccr sync folder disable <name>".cyan()
    );
    println!();

    println!("{}", "文件夹同步命令:".green().bold());
    println!(
        "  {}      上传指定文件夹到云端",
        "ccr sync <folder> push".cyan()
    );
    println!(
        "  {}      从云端下载指定文件夹",
        "ccr sync <folder> pull".cyan()
    );
    println!(
        "  {}    显示文件夹同步状态",
        "ccr sync <folder> status".cyan()
    );
    println!();

    println!("{}", "批量操作命令:".green().bold());
    println!("  {}     上传所有启用的文件夹", "ccr sync all push".cyan());
    println!("  {}     下载所有启用的文件夹", "ccr sync all pull".cyan());
    println!("  {}   显示所有文件夹的状态", "ccr sync all status".cyan());
    println!();

    println!("{}", "使用示例:".green().bold());
    println!();
    println!("  {}  配置 WebDAV", "1.".yellow());
    println!("     {}", "ccr sync config".dimmed());
    println!();
    println!("  {}  添加要同步的文件夹", "2.".yellow());
    println!("     {}", "ccr sync folder add claude ~/.claude".dimmed());
    println!("     {}", "ccr sync folder add gemini ~/.gemini".dimmed());
    println!(
        "     {}",
        "ccr sync folder add conf ~/.ccr/config.toml -d \"主配置文件\"".dimmed()
    );
    println!();
    println!("  {}  上传文件夹到云端", "3.".yellow());
    println!(
        "     {}",
        "ccr sync claude push              # 上传单个文件夹".dimmed()
    );
    println!(
        "     {}",
        "ccr sync all push                 # 上传所有启用的文件夹".dimmed()
    );
    println!();
    println!("  {}  从云端下载文件夹", "4.".yellow());
    println!(
        "     {}",
        "ccr sync gemini pull              # 下载单个文件夹".dimmed()
    );
    println!(
        "     {}",
        "ccr sync all pull                 # 下载所有启用的文件夹".dimmed()
    );
    println!();
    println!("  {}  查看状态", "5.".yellow());
    println!(
        "     {}",
        "ccr sync status                   # 查看所有文件夹状态".dimmed()
    );
    println!(
        "     {}",
        "ccr sync claude status            # 查看单个文件夹状态".dimmed()
    );
    println!();

    println!("{}", "选项:".green().bold());
    println!("  {}       强制覆盖，不提示确认", "--force".cyan());
    println!(
        "  {}  交互式选择要同步的内容 (仅 sync push)",
        "--interactive".cyan()
    );
    println!(
        "  {}   指定远程路径 (folder add 命令)",
        "-r, --remote-path".cyan()
    );
    println!(
        "  {}  指定文件夹描述 (folder add 命令)",
        "-d, --description".cyan()
    );
    println!();

    println!("{}", "特性:".green().bold());
    println!("  📁 独立文件夹管理 - 每个文件夹独立注册和同步");
    println!("  🔄 细粒度控制     - 可以单独启用/禁用文件夹");
    println!("  💾 自动备份       - 下载前自动备份本地内容");
    println!("  🔒 安全操作       - 支持 WebDAV 认证和 HTTPS");
    println!();

    println!("{}", "支持的 WebDAV 服务:".green().bold());
    println!("  • 坚果云 (https://dav.jianguoyun.com/dav/)");
    println!("  • Nextcloud");
    println!("  • ownCloud");
    println!("  • 任何标准 WebDAV 服务器");
    println!();

    println!("{}", "━".repeat(70).dimmed());
    println!();
    println!("💡 提示: 运行 {} 查看更多命令", "ccr --help".cyan());
    println!();
}

/// ⚙️ 配置 WebDAV 同步
///
/// 交互式配置 WebDAV 连接信息
pub async fn sync_config_command() -> Result<()> {
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
    let _ = io::stdout().flush();
    let password = ccr_core::Secret::new(read_password()?);
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

    let service = SyncService::new(&sync_config).await?;
    service.test_connection().await?;

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
pub async fn sync_status_command() -> Result<()> {
    use colored::*;
    use comfy_table::{Attribute, Cell, Color as TableColor};

    ColorOutput::title("☁️  WebDAV 同步状态");
    println!();

    let sync_manager = SyncConfigManager::with_default()?;
    let sync_config = sync_manager.load()?;

    if sync_config.enabled {
        // 使用 comfy-table 创建表格
        let mut table = new_utf8_table();
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
        let masked_pwd = format!("{}...", "*".repeat(8));
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
        std::io::Write::flush(&mut std::io::stdout())?;

        let service = SyncService::new(&sync_config).await?;
        let exists = service.remote_exists().await?;

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
pub async fn sync_push_command_with_selection(
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

    let service = SyncService::new(&sync_config).await?;

    if !force {
        print!("🔍 正在检查远程状态...");
        let _ = io::stdout().flush();

        let exists = service.remote_exists().await?;

        print!("\r");
        if exists {
            println!("{}  {}", "⚠".yellow().bold(), "远程已存在同名内容".yellow());
            println!();
            print!("   是否覆盖远程配置？ {} ", "(y/N):".dimmed());
            let _ = io::stdout().flush();

            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm)?;

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
        let _ = io::stdout().flush();
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
    let _ = io::stdout().flush();

    // 🎯 根据内容选择创建临时过滤目录进行同步
    let temp_sync_path =
        if filtered_paths.len() == 1 && filtered_paths[0] == "config.toml" && !is_dir {
            // 如果是单个config文件且当前是文件模式，直接同步原文件
            sync_path.clone()
        } else {
            // 需要创建临时目录包含选中的内容
            create_temp_sync_directory_async(sync_path.clone(), filtered_paths.clone(), is_dir)
                .await?
        };

    print!("🚀 正在上传...");
    let _ = io::stdout().flush();

    service.push(&temp_sync_path, None).await?;

    // 清理临时目录（如果需要）
    if temp_sync_path != sync_path
        && let Err(error) = tokio::fs::remove_dir_all(&temp_sync_path).await
    {
        tracing::warn!(
            "清理临时同步目录失败 ({}): {}",
            temp_sync_path.display(),
            error
        );
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
pub async fn sync_pull_command(force: bool) -> Result<()> {
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

    // 💾 备份本地 config.toml 内容，避免在 pull 时被远程覆盖
    let mut local_config_backup: Option<(PathBuf, String)> = None;
    if is_dir {
        let config_path = sync_path.join("config.toml");
        if tokio::fs::try_exists(&config_path).await.unwrap_or(false)
            && let Ok(content) = tokio::fs::read_to_string(&config_path).await
        {
            local_config_backup = Some((config_path, content));
        }
    }

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
        let _ = io::stdout().flush();

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;

        if !confirm.trim().eq_ignore_ascii_case("y") {
            println!();
            println!("{}  已取消下载", "ℹ".blue().bold());
            return Ok(());
        }
        println!();
    }

    // 🔍 检查远程是否存在（在备份前检查，避免不必要的备份）
    let service = SyncService::new(&sync_config).await?;
    let remote_exists = service.remote_exists().await?;

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
    if tokio::fs::try_exists(&sync_path).await.unwrap_or(false) {
        print!("💾 正在备份本地内容...");
        let _ = io::stdout().flush();

        // 如果是文件，使用 ConfigManager 的备份功能
        // 如果是目录，创建带时间戳的 .bak 备份
        let backup_path = if is_dir {
            // 🏷️ 生成带时间戳的备份目录名，避免冲突
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let backup_name = format!("{}.{}.bak", sync_path.display(), timestamp);
            let backup = PathBuf::from(backup_name);

            // 🔄 如果目标备份路径已存在（极少见），先删除
            if tokio::fs::try_exists(&backup).await.unwrap_or(false) {
                tokio::fs::remove_dir_all(&backup)
                    .await
                    .map_err(|e| CcrError::SyncError(format!("删除旧备份失败: {}", e)))?;
            }

            // 📦 移动目录到备份位置（原子操作）
            tokio::fs::rename(&sync_path, &backup)
                .await
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
        let _ = io::stdout().flush();
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
    let _ = io::stdout().flush();

    service.pull(&sync_path).await?;

    // 🔁 还原本地 config.toml，确保其不受远程内容影响
    if let Some((config_path, content)) = local_config_backup
        && let Some(parent) = config_path.parent()
    {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            ColorOutput::warning(&format!("恢复本地 config.toml 目录失败: {}", e));
        } else if let Err(e) = tokio::fs::write(&config_path, content).await {
            ColorOutput::warning(&format!("恢复本地 config.toml 失败: {}", e));
        }
    }

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
/// 默认使用 SyncContentSelection::default()（当前为平台目录）
pub async fn sync_push_command(force: bool) -> Result<()> {
    let default_selection = SyncContentSelection::default();
    sync_push_command_with_selection(force, Some(default_selection)).await
}
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

async fn create_temp_sync_directory_async(
    original_path: PathBuf,
    filtered_paths: Vec<String>,
    is_dir: bool,
) -> Result<PathBuf> {
    tokio::task::spawn_blocking(move || {
        create_temp_sync_directory(&original_path, &filtered_paths, is_dir)
    })
    .await
    .map_err(|e| CcrError::SyncError(format!("创建临时同步目录任务失败: {}", e)))?
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
/// 使用 sync 模块统一实现:
/// - 优先 CCR_ROOT
/// - 然后 ~/.ccr/
/// - 不再支持 Legacy ~/.ccs_config.toml 作为同步根
fn get_ccr_sync_path() -> Result<PathBuf> {
    crate::sync::get_ccr_sync_path()
}

/// 必填字段提示
fn prompt_required(field_name: &str, example: &str) -> Result<String> {
    loop {
        println!();
        ColorOutput::info(&format!("{} *", field_name));
        println!("  例如: {}", example);
        print!("  请输入: ");
        let _ = io::stdout().flush();

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
    let _ = io::stdout().flush();

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

// ============================================================================
// 🗂️ 文件夹管理命令
// ============================================================================

/// 📋 列出所有同步文件夹
pub fn sync_folder_list_command() -> Result<()> {
    ColorOutput::title("同步文件夹列表");
    println!();

    let manager = SyncFolderManager::with_default()?;
    let folders = manager.list_folders()?;

    if folders.is_empty() {
        ColorOutput::warning("暂无注册的同步文件夹");
        ColorOutput::info("使用 'ccr sync folder add' 添加文件夹");
        return Ok(());
    }

    // 创建表格
    let mut table = new_utf8_table();
    table.set_content_arrangement(ContentArrangement::DynamicFullWidth);

    // 表头
    table.set_header(vec![
        Cell::new("名称").fg(Color::Cyan),
        Cell::new("状态").fg(Color::Cyan),
        Cell::new("本地路径").fg(Color::Cyan),
        Cell::new("远程路径").fg(Color::Cyan),
        Cell::new("描述").fg(Color::Cyan),
    ]);

    // 数据行
    for folder in &folders {
        let status = if folder.enabled {
            Cell::new("✓").fg(Color::Green)
        } else {
            Cell::new("✗").fg(Color::Red)
        };

        table.add_row(vec![
            Cell::new(&folder.name),
            status,
            Cell::new(&folder.local_path),
            Cell::new(&folder.remote_path),
            Cell::new(&folder.description),
        ]);
    }

    // 为"状态"列设置固定宽度和居中对齐
    if let Some(column) = table.column_mut(1) {
        column.set_constraint(ColumnConstraint::ContentWidth);
        column.set_cell_alignment(CellAlignment::Center);
    }

    println!("{table}");
    println!();

    // 统计信息
    let stats = manager.stats()?;
    ColorOutput::info(&format!(
        "总计: {} 个文件夹 (启用: {}, 禁用: {})",
        stats.total, stats.enabled, stats.disabled
    ));

    Ok(())
}

/// ➕ 添加同步文件夹
pub fn sync_folder_add_command(
    name: &str,
    local_path: &str,
    remote_path: Option<&String>,
    description: Option<&String>,
) -> Result<()> {
    ColorOutput::title(&format!("添加同步文件夹: {}", name));
    println!();

    let mut manager = SyncFolderManager::with_default()?;

    // 检查是否已存在
    if manager.has_folder(name) {
        return Err(CcrError::ConfigError(format!(
            "文件夹 '{}' 已存在，请使用不同的名称",
            name
        )));
    }

    // 获取 WebDAV 配置以生成默认远程路径
    let webdav_config = manager.get_webdav_config()?;
    let default_remote_path = format!("{}/{}", webdav_config.base_remote_path, name);

    let final_remote_path = remote_path
        .map(|s| s.to_string())
        .unwrap_or(default_remote_path);

    let final_description = description
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{} 文件夹", name));

    // 构建文件夹
    let folder = SyncFolder::builder()
        .name(name)
        .description(final_description)
        .local_path(local_path)
        .remote_path(&final_remote_path)
        .enabled(true)
        .build()?;

    // 显示配置信息
    ColorOutput::info("文件夹配置:");
    println!("  名称:     {}", folder.name);
    println!("  描述:     {}", folder.description);
    println!("  本地路径: {}", folder.local_path);
    println!("  远程路径: {}", folder.remote_path);
    println!("  状态:     启用");
    println!();

    // 添加文件夹
    manager.add_folder(folder)?;

    ColorOutput::success(&format!("✓ 已添加同步文件夹 '{}'", name));
    ColorOutput::info("提示: 使用 'ccr sync <folder> push' 开始同步");

    Ok(())
}

/// ❌ 删除同步文件夹
pub fn sync_folder_remove_command(name: &str) -> Result<()> {
    ColorOutput::title(&format!("删除同步文件夹: {}", name));
    println!();

    let mut manager = SyncFolderManager::with_default()?;

    // 检查是否存在
    let folder = manager.get_folder(name)?;

    ColorOutput::warning(&format!("即将删除文件夹 '{}' 的同步配置", folder.name));
    println!("  本地路径: {}", folder.local_path);
    println!("  远程路径: {}", folder.remote_path);
    println!();
    ColorOutput::info("注意: 不会删除本地文件，仅移除同步配置");
    println!();

    // 确认
    print!("确认删除? (y/N): ");
    let _ = io::stdout().flush();
    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;

    if !confirm.trim().eq_ignore_ascii_case("y") {
        ColorOutput::info("已取消删除");
        return Ok(());
    }

    manager.remove_folder(name)?;

    ColorOutput::success(&format!("✓ 已删除同步文件夹 '{}'", name));

    Ok(())
}

/// ℹ️ 显示文件夹详细信息
pub fn sync_folder_info_command(name: &str) -> Result<()> {
    ColorOutput::title(&format!("同步文件夹信息: {}", name));
    println!();

    let manager = SyncFolderManager::with_default()?;
    let folder = manager.get_folder(name)?;

    // 基本信息
    println!("{}", "基本信息".bold());
    println!("  名称:     {}", folder.name);
    println!("  描述:     {}", folder.description);
    println!(
        "  状态:     {}",
        if folder.enabled {
            "✓ 启用"
        } else {
            "✗ 禁用"
        }
    );
    println!();

    // 路径信息
    println!("{}", "路径配置".bold());
    println!("  本地路径: {}", folder.local_path);
    println!("  远程路径: {}", folder.remote_path);

    // 检查本地路径是否存在
    if let Ok(expanded) = folder.expand_local_path() {
        if expanded.exists() {
            if expanded.is_dir() {
                ColorOutput::success("  本地路径: ✓ 存在 (目录)");
            } else {
                ColorOutput::success("  本地路径: ✓ 存在 (文件)");
            }
        } else {
            ColorOutput::warning("  本地路径: ✗ 不存在");
        }
    }
    println!();

    // 排除规则
    if !folder.exclude_patterns.is_empty() {
        println!("{}", "排除模式".bold());
        for pattern in &folder.exclude_patterns {
            println!("  - {}", pattern);
        }
        println!();
    }

    // 自动同步
    println!("{}", "其他设置".bold());
    println!(
        "  自动同步: {}",
        if folder.auto_sync { "启用" } else { "禁用" }
    );

    Ok(())
}

/// ✅ 启用文件夹同步
pub fn sync_folder_enable_command(name: &str) -> Result<()> {
    let mut manager = SyncFolderManager::with_default()?;

    // 检查是否存在
    let folder = manager.get_folder(name)?;

    if folder.enabled {
        ColorOutput::warning(&format!("文件夹 '{}' 已经是启用状态", name));
        return Ok(());
    }

    manager.enable_folder(name)?;

    ColorOutput::success(&format!("✓ 已启用文件夹 '{}'", name));
    ColorOutput::info("该文件夹将参与批量同步操作");

    Ok(())
}

/// ❌ 禁用文件夹同步
pub fn sync_folder_disable_command(name: &str) -> Result<()> {
    let mut manager = SyncFolderManager::with_default()?;

    // 检查是否存在
    let folder = manager.get_folder(name)?;

    if !folder.enabled {
        ColorOutput::warning(&format!("文件夹 '{}' 已经是禁用状态", name));
        return Ok(());
    }

    manager.disable_folder(name)?;

    ColorOutput::success(&format!("✓ 已禁用文件夹 '{}'", name));
    ColorOutput::info("该文件夹不会参与批量同步操作");

    Ok(())
}

// ============================================================================
// 🔄 批量同步命令
// ============================================================================

/// 📤 批量上传所有启用的文件夹
pub async fn sync_all_push_command(force: bool) -> Result<()> {
    use colored::*;

    ColorOutput::title("批量上传同步文件夹");
    println!();

    let manager = SyncFolderManager::with_default()?;
    let config = manager.load_config()?;

    let enabled_folders = config.enabled_folders();

    if enabled_folders.is_empty() {
        ColorOutput::warning("没有启用的同步文件夹");
        ColorOutput::info("使用 'ccr sync folder list' 查看所有文件夹");
        return Ok(());
    }

    ColorOutput::info(&format!("准备上传 {} 个文件夹...", enabled_folders.len()));
    println!();

    let mut success_count = 0;
    let mut failed_count = 0;
    let mut failed_folders = Vec::new();

    for (index, folder) in enabled_folders.iter().enumerate() {
        println!(
            "{}  [{}/{}] {}",
            "▶".cyan(),
            index + 1,
            enabled_folders.len(),
            folder.name.bold()
        );

        match sync_folder_push_internal(folder, &manager, force).await {
            Ok(_) => {
                println!("   {}", "✓ 上传成功".green());
                success_count += 1;
            }
            Err(e) => {
                println!("   {} {}", "✗ 上传失败:".red(), e);
                failed_count += 1;
                failed_folders.push(folder.name.clone());
            }
        }
        println!();
    }

    // 📊 显示汇总
    ColorOutput::separator();
    println!();
    ColorOutput::title("批量上传汇总");
    println!("  总计: {}", enabled_folders.len());
    println!("  成功: {}", success_count.to_string().green());
    if failed_count > 0 {
        println!("  失败: {}", failed_count.to_string().red());
        println!();
        println!("失败的文件夹:");
        for name in &failed_folders {
            println!("  • {}", name.red());
        }
    }
    println!();

    if failed_count > 0 {
        Err(CcrError::SyncError(format!(
            "{} 个文件夹上传失败",
            failed_count
        )))
    } else {
        Ok(())
    }
}

/// 📥 批量下载所有启用的文件夹
pub async fn sync_all_pull_command(force: bool) -> Result<()> {
    use colored::*;

    ColorOutput::title("批量下载同步文件夹");
    println!();

    let manager = SyncFolderManager::with_default()?;
    let config = manager.load_config()?;

    let enabled_folders = config.enabled_folders();

    if enabled_folders.is_empty() {
        ColorOutput::warning("没有启用的同步文件夹");
        ColorOutput::info("使用 'ccr sync folder list' 查看所有文件夹");
        return Ok(());
    }

    ColorOutput::info(&format!("准备下载 {} 个文件夹...", enabled_folders.len()));
    println!();

    let mut success_count = 0;
    let mut failed_count = 0;
    let mut failed_folders = Vec::new();

    for (index, folder) in enabled_folders.iter().enumerate() {
        println!(
            "{}  [{}/{}] {}",
            "▶".cyan(),
            index + 1,
            enabled_folders.len(),
            folder.name.bold()
        );

        match sync_folder_pull_internal(folder, &manager, force).await {
            Ok(_) => {
                println!("   {}", "✓ 下载成功".green());
                success_count += 1;
            }
            Err(e) => {
                println!("   {} {}", "✗ 下载失败:".red(), e);
                failed_count += 1;
                failed_folders.push(folder.name.clone());
            }
        }
        println!();
    }

    // 📊 显示汇总
    ColorOutput::separator();
    println!();
    ColorOutput::title("批量下载汇总");
    println!("  总计: {}", enabled_folders.len());
    println!("  成功: {}", success_count.to_string().green());
    if failed_count > 0 {
        println!("  失败: {}", failed_count.to_string().red());
        println!();
        println!("失败的文件夹:");
        for name in &failed_folders {
            println!("  • {}", name.red());
        }
    }
    println!();

    if failed_count > 0 {
        Err(CcrError::SyncError(format!(
            "{} 个文件夹下载失败",
            failed_count
        )))
    } else {
        Ok(())
    }
}

/// 📊 显示所有文件夹的同步状态
pub async fn sync_all_status_command() -> Result<()> {
    ColorOutput::title("同步文件夹状态");
    println!();

    let manager = SyncFolderManager::with_default()?;
    let folders = manager.list_folders()?;
    let webdav_config = manager.get_webdav_config().ok();

    if folders.is_empty() {
        ColorOutput::warning("暂无注册的同步文件夹");
        ColorOutput::info("使用 'ccr sync folder add' 添加文件夹");
        return Ok(());
    }

    // 创建表格
    let mut table = new_utf8_table();
    table.set_content_arrangement(ContentArrangement::DynamicFullWidth);

    // 表头
    table.set_header(vec![
        Cell::new("名称").fg(Color::Cyan),
        Cell::new("启用").fg(Color::Cyan),
        Cell::new("本地存在").fg(Color::Cyan),
        Cell::new("同步状态").fg(Color::Cyan),
    ]);

    // 数据行
    for folder in &folders {
        let enabled = if folder.enabled {
            Cell::new("✓").fg(Color::Green)
        } else {
            Cell::new("✗").fg(Color::Red)
        };

        let exists = if folder.local_path_exists() {
            Cell::new("✓").fg(Color::Green)
        } else {
            Cell::new("✗").fg(Color::Yellow)
        };

        let sync_status = if !folder.enabled {
            Cell::new("已禁用").fg(Color::DarkGrey)
        } else if let Some(webdav) = &webdav_config {
            let sync_config = SyncConfig {
                enabled: true,
                webdav_url: webdav.url.clone(),
                username: webdav.username.clone(),
                password: webdav.password.clone(),
                remote_path: folder.remote_path.clone(),
                auto_sync: false,
            };

            match SyncService::new(&sync_config).await {
                Ok(service) => match service.remote_exists().await {
                    Ok(true) => Cell::new("远程存在").fg(Color::Green),
                    Ok(false) => Cell::new("远程不存在").fg(Color::Yellow),
                    Err(_) => Cell::new("检查失败").fg(Color::Red),
                },
                Err(_) => Cell::new("连接失败").fg(Color::Red),
            }
        } else {
            Cell::new("未配置 WebDAV").fg(Color::Red)
        };

        table.add_row(vec![Cell::new(&folder.name), enabled, exists, sync_status]);
    }

    // 为"启用"和"本地存在"列设置固定宽度和居中对齐
    if let Some(column) = table.column_mut(1) {
        column.set_constraint(ColumnConstraint::ContentWidth);
        column.set_cell_alignment(CellAlignment::Center);
    }
    if let Some(column) = table.column_mut(2) {
        column.set_constraint(ColumnConstraint::ContentWidth);
        column.set_cell_alignment(CellAlignment::Center);
    }

    println!("{table}");
    println!();

    // 统计信息
    let stats = manager.stats()?;
    ColorOutput::info(&format!(
        "总计: {} 个文件夹 (启用: {}, 禁用: {})",
        stats.total, stats.enabled, stats.disabled
    ));

    Ok(())
}

// ============================================================================
// 📁 文件夹特定同步命令
// ============================================================================

/// 🎯 处理文件夹特定的同步命令
///
/// 动态分发到具体操作: push, pull, status
pub async fn sync_folder_specific_command(args: &[String]) -> Result<()> {
    // 🆘 特殊处理 help 命令
    if args.len() == 1 && args[0].eq_ignore_ascii_case("help") {
        show_sync_help();
        return Ok(());
    }

    if args.len() < 2 {
        return Err(CcrError::ConfigError(
            "用法: ccr sync <folder> <action>\n  action: push | pull | status".into(),
        ));
    }

    let folder_name = &args[0];
    let action = &args[1];

    // 检查文件夹是否存在
    let manager = SyncFolderManager::with_default()?;
    let _folder = manager.get_folder(folder_name)?;

    match action.as_str() {
        "push" => sync_folder_push_command(folder_name).await,
        "pull" => sync_folder_pull_command(folder_name).await,
        "status" => sync_folder_status_command(folder_name).await,
        _ => Err(CcrError::ConfigError(format!(
            "未知操作: '{}'。支持的操作: push, pull, status",
            action
        ))),
    }
}

/// 📤 上传指定文件夹
async fn sync_folder_push_command(folder_name: &str) -> Result<()> {
    use colored::*;

    ColorOutput::title(&format!("上传文件夹: {}", folder_name));
    println!();

    let manager = SyncFolderManager::with_default()?;
    let folder = manager.get_folder(folder_name)?;

    if !folder.enabled {
        ColorOutput::warning(&format!("文件夹 '{}' 已禁用", folder_name));
        ColorOutput::info("使用 'ccr sync folder enable <name>' 启用该文件夹");
        return Ok(());
    }

    // 🏠 展开本地路径
    let local_path = folder.expand_local_path()?;

    // ✅ 检查本地路径是否存在
    if !local_path.exists() {
        return Err(CcrError::SyncError(format!(
            "本地路径不存在: {}",
            local_path.display()
        )));
    }

    ColorOutput::info(&format!("本地路径: {}", local_path.display()));
    ColorOutput::info(&format!("远程路径: {}", folder.remote_path));
    println!();

    // 🔧 获取 WebDAV 配置
    let webdav_config = manager.get_webdav_config()?;

    // 📝 构建 SyncConfig
    let sync_config = SyncConfig {
        enabled: true,
        webdav_url: webdav_config.url.clone(),
        username: webdav_config.username.clone(),
        password: webdav_config.password.clone(),
        remote_path: folder.remote_path.clone(),
        auto_sync: false,
    };

    // 🔍 检查远程是否已存在
    print!("🔍 正在检查远程状态...");
    let _ = io::stdout().flush();

    let service = SyncService::new(&sync_config).await?;
    let exists = service.remote_exists().await?;

    print!("\r");
    if exists {
        println!("{}  {}", "⚠".yellow().bold(), "远程已存在同名内容".yellow());
        println!();
        print!("   是否覆盖远程配置？ {} ", "(y/N):".dimmed());
        let _ = io::stdout().flush();

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;

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

    // 🚀 上传到云端
    print!("🚀 正在上传...");
    let _ = io::stdout().flush();

    // 🎯 不传递 allowed_paths，让 SyncService 使用内部的排除逻辑
    service.push(&local_path, None).await?;

    print!("\r");
    println!(
        "{}  {}",
        "✓".green().bold(),
        "文件夹已成功上传到云端".green()
    );
    println!();
    println!("📊 同步信息:");
    println!("   • 本地路径: {}", local_path.display().to_string().cyan());
    println!("   • 远程路径: {}", folder.remote_path.cyan());
    println!("   • 服务器: {}", webdav_config.url.dimmed());
    println!();

    Ok(())
}

/// 📥 下载指定文件夹
async fn sync_folder_pull_command(folder_name: &str) -> Result<()> {
    use colored::*;

    ColorOutput::title(&format!("下载文件夹: {}", folder_name));
    println!();

    let manager = SyncFolderManager::with_default()?;
    let folder = manager.get_folder(folder_name)?;

    // 🏠 展开本地路径
    let local_path = folder.expand_local_path()?;

    ColorOutput::info(&format!("远程路径: {}", folder.remote_path));
    ColorOutput::info(&format!("本地路径: {}", local_path.display()));
    println!();

    // 🔧 获取 WebDAV 配置
    let webdav_config = manager.get_webdav_config()?;

    // 📝 构建 SyncConfig
    let sync_config = SyncConfig {
        enabled: true,
        webdav_url: webdav_config.url.clone(),
        username: webdav_config.username.clone(),
        password: webdav_config.password.clone(),
        remote_path: folder.remote_path.clone(),
        auto_sync: false,
    };

    // 🔍 检查远程是否存在
    let service = SyncService::new(&sync_config).await?;
    let remote_exists = service.remote_exists().await?;

    if !remote_exists {
        println!();
        ColorOutput::error("远程目录不存在");
        println!();
        println!("   💡 提示: 首次使用需要先上传配置到云端");
        println!(
            "   运行命令: {}",
            format!("ccr sync {} push", folder_name).cyan()
        );
        println!();
        return Err(CcrError::SyncError("远程内容不存在".to_string()));
    }

    // ⚠️ 本地路径存在时的警告
    let local_exists = tokio::fs::try_exists(&local_path)
        .await
        .map_err(|e| CcrError::SyncError(format!("检查本地路径失败: {}", e)))?;

    if local_exists {
        println!(
            "{}  {}",
            "⚠".yellow().bold(),
            "此操作将覆盖本地内容".yellow()
        );
        println!();
        print!("   是否继续？本地内容将被备份 {} ", "(y/N):".dimmed());
        let _ = io::stdout().flush();

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;

        if !confirm.trim().eq_ignore_ascii_case("y") {
            println!();
            println!("{}  已取消下载", "ℹ".blue().bold());
            return Ok(());
        }
        println!();

        // 💾 备份本地文件夹
        print!("💾 正在备份本地内容...");
        let _ = io::stdout().flush();

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("{}.{}.bak", local_path.display(), timestamp);
        let backup_path = PathBuf::from(backup_name);

        if tokio::fs::try_exists(&backup_path).await.unwrap_or(false) {
            tokio::fs::remove_dir_all(&backup_path)
                .await
                .map_err(|e| CcrError::SyncError(format!("删除旧备份失败: {}", e)))?;
        }

        tokio::fs::rename(&local_path, &backup_path)
            .await
            .map_err(|e| CcrError::SyncError(format!("备份失败: {}", e)))?;

        print!("\r");
        println!("{}  {}", "✓".green().bold(), "本地内容已备份".green());
        println!(
            "   📁 备份位置: {}",
            backup_path.display().to_string().dimmed()
        );
        println!();
    }

    // ⬇️ 从云端下载
    print!("⬇️  正在从云端下载...");
    let _ = io::stdout().flush();

    service.pull(&local_path).await?;

    print!("\r");
    println!(
        "{}  {}",
        "✓".green().bold(),
        "文件夹已从云端下载并应用".green()
    );
    println!();
    println!("📊 同步信息:");
    println!("   • 本地路径: {}", local_path.display().to_string().cyan());
    println!("   • 远程路径: {}", folder.remote_path.cyan());
    println!("   • 服务器: {}", webdav_config.url.dimmed());
    println!();

    Ok(())
}

/// 📊 显示指定文件夹的同步状态
async fn sync_folder_status_command(folder_name: &str) -> Result<()> {
    use colored::*;

    ColorOutput::title(&format!("文件夹状态: {}", folder_name));
    println!();

    let manager = SyncFolderManager::with_default()?;
    let folder = manager.get_folder(folder_name)?;

    println!("{}", "基本信息".bold());
    println!("  名称:     {}", folder.name);
    println!(
        "  状态:     {}",
        if folder.enabled {
            "✓ 启用".green()
        } else {
            "✗ 禁用".red()
        }
    );
    println!("  本地路径: {}", folder.local_path);
    println!("  远程路径: {}", folder.remote_path);
    println!();

    // 🔧 获取 WebDAV 配置
    let webdav_config = manager.get_webdav_config()?;

    // 📝 构建 SyncConfig
    let sync_config = SyncConfig {
        enabled: true,
        webdav_url: webdav_config.url.clone(),
        username: webdav_config.username.clone(),
        password: webdav_config.password.clone(),
        remote_path: folder.remote_path.clone(),
        auto_sync: false,
    };

    println!("{}", "同步状态".bold());

    // ✅ 检查本地路径是否存在
    let local_path = folder.expand_local_path()?;
    if local_path.exists() {
        if local_path.is_dir() {
            println!("  本地路径: {} (目录)", "✓ 存在".green());
        } else {
            println!("  本地路径: {} (文件)", "✓ 存在".green());
        }
    } else {
        println!("  本地路径: {}", "✗ 不存在".yellow());
    }

    // 🔍 检查远程状态
    print!("  远程状态: 正在检查...");
    let _ = io::stdout().flush();

    let service = SyncService::new(&sync_config).await?;
    let remote_exists = service.remote_exists().await?;

    print!("\r");
    if remote_exists {
        println!("  远程状态: {}", "✓ 存在".green());
    } else {
        println!("  远程状态: {}", "✗ 不存在".yellow());
        println!();
        println!(
            "  💡 提示: 运行 {} 首次上传",
            format!("ccr sync {} push", folder_name).cyan()
        );
    }
    println!();

    Ok(())
}

// ============================================================================
// 🔧 内部辅助函数
// ============================================================================

/// 🔧 内部上传函数 - 用于批量上传和单独上传
///
/// # 参数
/// - folder: 要上传的文件夹配置
/// - manager: SyncFolderManager 实例
/// - force: 是否强制上传（跳过确认）
async fn sync_folder_push_internal(
    folder: &SyncFolder,
    manager: &SyncFolderManager,
    force: bool,
) -> Result<()> {
    // 🏠 展开本地路径
    let local_path = folder.expand_local_path()?;

    // ✅ 检查本地路径是否存在
    if !local_path.exists() {
        return Err(CcrError::SyncError(format!(
            "本地路径不存在: {}",
            local_path.display()
        )));
    }

    // 🔧 获取 WebDAV 配置
    let webdav_config = manager.get_webdav_config()?;

    // 📝 构建 SyncConfig
    let sync_config = SyncConfig {
        enabled: true,
        webdav_url: webdav_config.url.clone(),
        username: webdav_config.username.clone(),
        password: webdav_config.password.clone(),
        remote_path: folder.remote_path.clone(),
        auto_sync: false,
    };

    // 🔍 检查远程是否已存在（仅在非强制模式下）
    if !force {
        let service = SyncService::new(&sync_config).await?;
        let exists = service.remote_exists().await?;

        if exists {
            // 在批量模式下跳过已存在的文件夹
            return Err(CcrError::SyncError("远程已存在，使用 --force 覆盖".into()));
        }
    }

    // 🚀 上传到云端
    let service = SyncService::new(&sync_config).await?;
    service.push(&local_path, None).await?;

    Ok(())
}

/// 🔧 内部下载函数 - 用于批量下载和单独下载
///
/// # 参数
/// - folder: 要下载的文件夹配置
/// - manager: SyncFolderManager 实例
/// - force: 是否强制下载（跳过确认）
async fn sync_folder_pull_internal(
    folder: &SyncFolder,
    manager: &SyncFolderManager,
    force: bool,
) -> Result<()> {
    // 🏠 展开本地路径
    let local_path = folder.expand_local_path()?;

    // 🔧 获取 WebDAV 配置
    let webdav_config = manager.get_webdav_config()?;

    // 📝 构建 SyncConfig
    let sync_config = SyncConfig {
        enabled: true,
        webdav_url: webdav_config.url.clone(),
        username: webdav_config.username.clone(),
        password: webdav_config.password.clone(),
        remote_path: folder.remote_path.clone(),
        auto_sync: false,
    };

    // 🔍 检查远程是否存在
    let service = SyncService::new(&sync_config).await?;
    let remote_exists = service.remote_exists().await?;

    if !remote_exists {
        return Err(CcrError::SyncError("远程内容不存在".to_string()));
    }

    // ⚠️ 本地路径存在时的警告（仅在非强制模式下）
    let local_exists = tokio::fs::try_exists(&local_path)
        .await
        .map_err(|e| CcrError::SyncError(format!("检查本地路径失败: {}", e)))?;

    if local_exists && !force {
        // 在批量模式下直接跳过已存在的
        return Err(CcrError::SyncError("本地已存在，使用 --force 覆盖".into()));
    }

    // 💾 备份本地文件夹（如果存在）
    if local_exists {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("{}.{}.bak", local_path.display(), timestamp);
        let backup_path = PathBuf::from(backup_name);

        if tokio::fs::try_exists(&backup_path).await.unwrap_or(false) {
            tokio::fs::remove_dir_all(&backup_path)
                .await
                .map_err(|e| CcrError::SyncError(format!("删除旧备份失败: {}", e)))?;
        }

        tokio::fs::rename(&local_path, &backup_path)
            .await
            .map_err(|e| CcrError::SyncError(format!("备份失败: {}", e)))?;
    }

    // ⬇️ 从云端下载
    service.pull(&local_path).await?;

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_sync_config_creation() {
        let config = SyncConfig {
            enabled: true,
            webdav_url: "https://dav.jianguoyun.com/dav/".to_string(),
            username: "test@example.com".to_string(),
            password: ccr_core::Secret::from("test_password"),
            remote_path: "/ccr/".to_string(), // 🆕 改为目录路径
            auto_sync: false,
        };

        assert!(config.enabled);
        assert_eq!(config.webdav_url, "https://dav.jianguoyun.com/dav/");
        assert_eq!(config.remote_path, "/ccr/");
    }

    #[tokio::test]
    async fn local_sync_commands_use_isolated_config_and_folder_state() {
        let mut home = crate::test_support::TestHome::new();
        let sync_config = home.root().join("sync.toml");
        let folders_config = home.root().join("sync_folders.toml");
        home.set_env("CCR_SYNC_CONFIG_PATH", sync_config.as_os_str());
        home.set_env("CCR_SYNC_FOLDERS_CONFIG", folders_config.as_os_str());

        show_sync_help();
        sync_status_command().await.unwrap();
        assert!(sync_push_command(true).await.is_err());
        assert!(sync_pull_command(true).await.is_err());
        sync_folder_list_command().unwrap();
        sync_all_push_command(true).await.unwrap();
        sync_all_pull_command(true).await.unwrap();
        sync_all_status_command().await.unwrap();
        sync_folder_specific_command(&["help".into()])
            .await
            .unwrap();
        assert!(sync_folder_specific_command(&[]).await.is_err());

        let local_dir = home.root().join("fixture-folder");
        fs::create_dir_all(local_dir.join("nested")).unwrap();
        fs::write(local_dir.join("root.txt"), "root").unwrap();
        fs::write(local_dir.join("nested").join("child.txt"), "child").unwrap();
        let mut manager = SyncFolderManager::with_default().unwrap();
        manager
            .update_webdav_config(ccr_sync::WebDavConfig {
                enabled: true,
                url: "https://example.invalid/dav/".into(),
                username: "fixture".into(),
                password: ccr_core::Secret::from("fixture-secret"),
                base_remote_path: "/fixtures".into(),
                auto_sync: false,
            })
            .unwrap();
        let remote_path = "/fixtures/demo".to_string();
        let description = "isolated folder".to_string();
        sync_folder_add_command(
            "demo",
            local_dir.to_string_lossy().as_ref(),
            Some(&remote_path),
            Some(&description),
        )
        .unwrap();
        assert!(
            sync_folder_add_command("demo", local_dir.to_string_lossy().as_ref(), None, None,)
                .is_err()
        );
        sync_folder_list_command().unwrap();
        sync_folder_info_command("demo").unwrap();
        sync_folder_disable_command("demo").unwrap();
        sync_folder_disable_command("demo").unwrap();
        sync_all_status_command().await.unwrap();
        sync_folder_specific_command(&["demo".into(), "push".into()])
            .await
            .unwrap();
        sync_folder_enable_command("demo").unwrap();
        sync_folder_enable_command("demo").unwrap();
        assert!(
            sync_folder_specific_command(&["demo".into(), "unknown".into()])
                .await
                .is_err()
        );
        assert!(sync_folder_info_command("missing").is_err());

        let filtered = create_temp_sync_directory(
            &local_dir,
            &["root.txt".into(), "nested".into(), "missing".into()],
            true,
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(filtered.join("root.txt")).unwrap(),
            "root"
        );
        assert_eq!(
            fs::read_to_string(filtered.join("nested").join("child.txt")).unwrap(),
            "child"
        );
        fs::remove_dir_all(filtered).unwrap();

        let config_file = home.root().join("config.toml");
        fs::write(&config_file, "default_platform = 'codex'\n").unwrap();
        let filtered =
            create_temp_sync_directory(&config_file, &["config.toml".into()], false).unwrap();
        assert!(filtered.join("config.toml").is_file());
        fs::remove_dir_all(filtered).unwrap();

        assert_eq!(get_ccr_sync_path().unwrap(), home.root());
    }
}
