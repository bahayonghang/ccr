// ☁️ sync 命令实现 - WebDAV 配置同步
// 📁 支持配置文件的云端同步功能

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::managers::config::{ConfigManager, SyncConfig};
use crate::services::SyncService;
use std::io::{self, Write};

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
    let remote_path = prompt_with_default("远程文件路径", Some("/ccr/.ccs_config.toml"))?;

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

    // 保存配置
    ColorOutput::step("保存同步配置");
    let manager = ConfigManager::default()?;
    let mut config = manager.load()?;
    config.settings.sync = Some(sync_config);
    manager.save(&config)?;

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

    let manager = ConfigManager::default()?;
    let config = manager.load()?;

    match &config.settings.sync {
        Some(sync_config) if sync_config.enabled => {
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
            print!("🔍 正在检查远程文件...");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();

            let runtime = tokio::runtime::Runtime::new()
                .map_err(|e| CcrError::SyncError(format!("创建异步运行时失败: {}", e)))?;

            let exists = runtime.block_on(async {
                let service = SyncService::new(sync_config).await?;
                service.remote_exists().await
            })?;

            print!("\r");
            if exists {
                println!("{}  {}", "✓".green().bold(), "远程配置文件存在".green());
            } else {
                println!("{}  {}", "⚠".yellow().bold(), "远程配置文件不存在".yellow());
                println!("   💡 提示: 运行 {} 首次上传配置", "ccr sync push".cyan());
            }
            println!();
        }
        _ => {
            println!("{}  {}", "⚠".yellow().bold(), "同步功能未配置".yellow());
            println!();
            println!("📝 配置步骤:");
            println!("   1. 运行 {} 开始配置", "ccr sync config".cyan());
            println!("   2. 输入 WebDAV 服务器信息");
            println!("   3. 测试连接成功后即可使用");
            println!();
        }
    }

    Ok(())
}

/// 🔼 上传配置到云端
pub fn sync_push_command(force: bool) -> Result<()> {
    use colored::*;

    ColorOutput::title("🔼  上传配置到云端");
    println!();

    let manager = ConfigManager::default()?;
    let config = manager.load()?;

    let sync_config =
        config.settings.sync.as_ref().ok_or_else(|| {
            CcrError::SyncError("同步功能未配置，请先运行 'ccr sync config'".into())
        })?;

    if !sync_config.enabled {
        return Err(CcrError::SyncError("同步功能已禁用".into()));
    }

    // 检查远程文件是否存在
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| CcrError::SyncError(format!("创建异步运行时失败: {}", e)))?;

    if !force {
        print!("🔍 正在检查远程文件...");
        io::stdout().flush().unwrap();

        let exists = runtime.block_on(async {
            let service = SyncService::new(sync_config).await?;
            service.remote_exists().await
        })?;

        print!("\r");
        if exists {
            println!("{}  {}", "⚠".yellow().bold(), "远程配置文件已存在".yellow());
            println!();
            print!("   是否覆盖远程配置？ {} ", "(y/N):".dimmed());
            io::stdout().flush().unwrap();

            let mut confirm = String::new();
            io::stdin().read_line(&mut confirm).unwrap();

            if !confirm.trim().eq_ignore_ascii_case("y") {
                println!();
                println!("{}  {}", "ℹ".blue().bold(), "已取消上传");
                return Ok(());
            }
            println!();
        } else {
            println!(
                "{}  {}",
                "ℹ".blue().bold(),
                "远程文件不存在，将创建新文件".blue()
            );
            println!();
        }
    }

    print!("🚀 正在上传配置...");
    io::stdout().flush().unwrap();

    runtime.block_on(async {
        let service = SyncService::new(sync_config).await?;
        service.push(manager.config_path()).await?;
        Ok::<(), CcrError>(())
    })?;

    print!("\r");
    println!("{}  {}", "✓".green().bold(), "配置已成功上传到云端".green());
    println!();
    println!("📊 同步信息:");
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

    let manager = ConfigManager::default()?;
    let config = manager.load()?;

    let sync_config =
        config.settings.sync.as_ref().ok_or_else(|| {
            CcrError::SyncError("同步功能未配置，请先运行 'ccr sync config'".into())
        })?;

    if !sync_config.enabled {
        return Err(CcrError::SyncError("同步功能已禁用".into()));
    }

    // 备份本地配置
    if !force {
        println!(
            "{}  {}",
            "⚠".yellow().bold(),
            "此操作将覆盖本地配置文件".yellow()
        );
        println!();
        print!("   是否继续？本地配置将被备份 {} ", "(y/N):".dimmed());
        io::stdout().flush().unwrap();

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm).unwrap();

        if !confirm.trim().eq_ignore_ascii_case("y") {
            println!();
            println!("{}  {}", "ℹ".blue().bold(), "已取消下载");
            return Ok(());
        }
        println!();
    }

    print!("💾 正在备份本地配置...");
    io::stdout().flush().unwrap();
    let backup_path = manager.backup(Some("before_pull"))?;
    print!("\r");
    println!("{}  {}", "✓".green().bold(), "本地配置已备份".green());
    println!(
        "   📁 备份位置: {}",
        backup_path.display().to_string().dimmed()
    );
    println!();

    print!("⬇️  正在从云端下载配置...");
    io::stdout().flush().unwrap();

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| CcrError::SyncError(format!("创建异步运行时失败: {}", e)))?;

    runtime.block_on(async {
        let service = SyncService::new(sync_config).await?;
        service.pull(manager.config_path()).await?;
        Ok::<(), CcrError>(())
    })?;

    print!("\r");
    println!(
        "{}  {}",
        "✓".green().bold(),
        "配置已从云端下载并应用".green()
    );
    println!();
    println!("📊 同步信息:");
    println!("   • 远程路径: {}", sync_config.remote_path.cyan());
    println!("   • 服务器: {}", sync_config.webdav_url.dimmed());
    println!();
    println!("💡 下一步: 运行 {} 查看配置", "ccr list".cyan());
    println!();

    Ok(())
}

// === 辅助函数 ===

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
            remote_path: "/ccr/.ccs_config.toml".to_string(),
            auto_sync: false,
        };

        assert!(config.enabled);
        assert_eq!(config.webdav_url, "https://dav.jianguoyun.com/dav/");
    }
}
