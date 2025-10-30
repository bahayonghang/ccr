// 🎯 platform 命令实现 - 多平台管理
// 📋 管理和切换不同的 AI 平台 (Claude, Codex, Gemini 等)

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::managers::PlatformConfigManager;
use crate::models::{Platform, PlatformPaths};
use crate::platforms::{PlatformRegistry, create_platform};
use colored::Colorize;
use comfy_table::{
    Attribute, Cell, Color as TableColor, ContentArrangement, Table, presets::UTF8_FULL,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// 📊 平台列表 JSON 输出结构
#[derive(Debug, Serialize, Deserialize)]
struct PlatformListOutput {
    config_file: String,
    default_platform: String,
    current_platform: String,
    platforms: Vec<PlatformListItem>,
}

/// 📋 单个平台信息
#[derive(Debug, Serialize, Deserialize)]
struct PlatformListItem {
    name: String,
    is_current: bool,
    is_default: bool,
    enabled: bool,
    current_profile: Option<String>,
    description: String,
}

/// 📊 平台详情 JSON 输出结构
#[derive(Debug, Serialize, Deserialize)]
struct PlatformInfoOutput {
    name: String,
    display_name: String,
    is_current: bool,
    enabled: bool,
    current_profile: Option<String>,
    description: Option<String>,
    paths: PlatformPathsOutput,
    profiles: Vec<String>,
}

/// 📁 平台路径信息
#[derive(Debug, Serialize, Deserialize)]
struct PlatformPathsOutput {
    platform_dir: String,
    profiles_file: String,
    history_file: String,
    backups_dir: String,
}

/// 📜 列出所有可用平台
///
/// 显示内容:
/// - 🎯 当前激活的平台
/// - 📋 所有注册的平台列表
/// - 🔌 平台启用状态
/// - ▶️ 当前 profile
/// - 📝 平台描述
///
/// # 参数
///
/// * `json` - 是否以 JSON 格式输出
///
/// # 返回
///
/// * `Ok(())` - 成功执行
/// * `Err(CcrError)` - 配置文件加载失败或其他错误
///
/// # 示例
///
/// ```rust,no_run
/// use ccr::commands::platform_list_command;
///
/// // 表格格式输出
/// platform_list_command(false)?;
///
/// // JSON 格式输出
/// platform_list_command(true)?;
/// # Ok::<(), ccr::CcrError>(())
/// ```
///
/// # 输出格式
///
/// ## 表格模式 (json=false)
///
/// ```text
/// ╭─────────────────────────────────────────────────────╮
/// │                   平台列表                          │
/// ╰─────────────────────────────────────────────────────╯
///
/// 配置文件: /home/user/.ccr/config.toml
/// 默认平台: claude
/// 当前平台: claude
///
/// ╔════════╤════════════╤══════╤═══════════════╤═════════════════╗
/// ║ 状态   │ 平台名称   │ 启用 │ 当前 Profile  │ 描述            ║
/// ╠════════╪════════════╪══════╪═══════════════╪═════════════════╣
/// ║ ▶ 当前 │ claude     │ ✅   │ official      │ Claude Code     ║
/// ╟────────┼────────────┼──────┼───────────────┼─────────────────╢
/// ║        │ codex      │ ✅   │ github        │ Codex           ║
/// ╟────────┼────────────┼──────┼───────────────┼─────────────────╢
/// ║        │ gemini     │ ✅   │ google        │ Gemini CLI      ║
/// ╚════════╧════════════╧══════╧═══════════════╧═════════════════╝
/// ```
///
/// ## JSON 模式 (json=true)
///
/// ```json
/// {
///   "config_file": "/home/user/.ccr/config.toml",
///   "default_platform": "claude",
///   "current_platform": "claude",
///   "platforms": [
///     {
///       "name": "claude",
///       "is_current": true,
///       "is_default": true,
///       "enabled": true,
///       "current_profile": "official",
///       "description": "Claude Code"
///     }
///   ]
/// }
/// ```
pub fn platform_list_command(json: bool) -> Result<()> {
    let manager = PlatformConfigManager::with_default()?;
    let config = manager.load_or_create_default()?;

    // 获取所有支持的平台
    let registry = PlatformRegistry::new();
    let all_platforms = registry.list_platform_info();

    // 🔍 收集平台信息
    let mut platforms_data = Vec::new();

    for platform_info in &all_platforms {
        let platform_name = &platform_info.short_name;
        let registry_entry = config.platforms.get(platform_name);

        let is_current = platform_name == &config.current_platform;
        let is_default = platform_name == &config.default_platform;
        let enabled = registry_entry.map(|e| e.enabled).unwrap_or(false);
        let current_profile = registry_entry.and_then(|e| e.current_profile.clone());
        let description = registry_entry
            .and_then(|e| e.description.clone())
            .unwrap_or_else(|| platform_info.name.to_string());

        platforms_data.push(PlatformListItem {
            name: platform_name.clone(),
            is_current,
            is_default,
            enabled,
            current_profile,
            description,
        });
    }

    // 📤 输出格式选择
    if json {
        // JSON 输出
        let output = PlatformListOutput {
            config_file: manager.config_path().display().to_string(),
            default_platform: config.default_platform.clone(),
            current_platform: config.current_platform.clone(),
            platforms: platforms_data,
        };

        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{}", json_str);

        return Ok(());
    }

    // 📊 表格输出 (原有逻辑)
    ColorOutput::title("平台列表");

    println!();
    ColorOutput::info(&format!("配置文件: {}", manager.config_path().display()));
    ColorOutput::info(&format!(
        "默认平台: {}",
        config.default_platform.bright_yellow()
    ));
    ColorOutput::info(&format!(
        "当前平台: {}",
        config.current_platform.bright_green().bold()
    ));
    println!();

    // 创建表格
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("状态")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("平台名称")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("启用")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("当前 Profile")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("描述")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    // 使用已收集的数据填充表格
    for platform in &platforms_data {
        // 状态列
        let status = if platform.is_current {
            Cell::new("▶ 当前")
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else if platform.is_default {
            Cell::new("⭐ 默认").fg(TableColor::Yellow)
        } else {
            Cell::new("")
        };

        // 平台名称
        let name_cell = if platform.is_current {
            Cell::new(&platform.name)
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else {
            Cell::new(&platform.name)
        };

        // 启用状态
        let enabled_cell = if platform.enabled {
            Cell::new("✓")
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else {
            Cell::new("✗").fg(TableColor::Red)
        };

        // 当前 profile
        let current_profile = platform.current_profile.as_deref().unwrap_or("-");

        table.add_row(vec![
            status,
            name_cell,
            enabled_cell,
            Cell::new(current_profile),
            Cell::new(&platform.description).fg(TableColor::Blue),
        ]);
    }

    println!("{}", table);
    println!();

    ColorOutput::success(&format!("共找到 {} 个平台", platforms_data.len()));
    println!();
    ColorOutput::info("提示:");
    println!("  • 使用 'ccr platform switch <平台名>' 切换平台");
    println!("  • 使用 'ccr platform current' 查看当前平台详情");
    println!("  • 使用 'ccr platform info <平台名>' 查看平台信息");

    Ok(())
}

/// 🔄 切换当前平台
///
/// 将当前激活的平台切换到指定的平台。如果目标平台未注册，会自动注册该平台。
///
/// # 参数
///
/// * `platform_name` - 要切换到的平台名称 (`"claude"`, `"codex"`, `"gemini"` 等)
///
/// # 返回
///
/// * `Ok(())` - 成功切换平台
/// * `Err(CcrError::PlatformNotFound)` - 指定的平台不存在或未实现
/// * `Err(CcrError::ConfigError)` - 平台已禁用或配置错误
///
/// # 示例
///
/// ```rust,no_run
/// use ccr::commands::platform_switch_command;
///
/// // 切换到 Claude 平台
/// platform_switch_command("claude")?;
///
/// // 切换到 Codex (GitHub Copilot) 平台
/// platform_switch_command("codex")?;
///
/// // 切换到 Gemini 平台
/// platform_switch_command("gemini")?;
/// # Ok::<(), ccr::CcrError>(())
/// ```
///
/// # 行为说明
///
/// 1. **平台验证**: 检查平台名称是否有效（是否在支持列表中）
/// 2. **自动注册**: 如果平台未在配置中注册，自动注册并启用
/// 3. **状态更新**: 更新 `current_platform` 字段
/// 4. **时间戳记录**: 记录旧平台的最后使用时间
/// 5. **配置保存**: 将更改持久化到 `~/.ccr/config.toml`
/// 6. **提示信息**: 显示切换结果和当前 profile（如果有）
///
/// # 输出示例
///
/// ```text
/// ╭─────────────────────────────────────────────────────╮
/// │              切换到平台: codex                       │
/// ╰─────────────────────────────────────────────────────╯
///
/// ✅ 已从平台 'claude' 切换到 'codex'
///
/// ℹ️  当前 profile: github-official
/// ```
///
/// # 注意事项
///
/// - 切换平台不会自动切换该平台下的 profile
/// - 如果平台已禁用 (`enabled = false`)，切换会失败
/// - 切换不会影响其他平台的配置和状态
///
pub fn platform_switch_command(platform_name: &str) -> Result<()> {
    ColorOutput::title(&format!("切换到平台: {}", platform_name));

    let manager = PlatformConfigManager::with_default()?;
    let mut config = manager.load_or_create_default()?;

    // 验证平台是否存在
    let platform = Platform::from_str(platform_name)
        .map_err(|_| CcrError::PlatformNotFound(platform_name.to_string()))?;

    // 如果平台未注册，自动注册
    if !config.platforms.contains_key(platform_name) {
        ColorOutput::info(&format!("平台 '{}' 未注册，正在自动注册...", platform_name));

        let platform_impl = create_platform(platform)?;
        let registry = crate::managers::PlatformConfigEntry {
            description: Some(platform_impl.platform_name().to_string()),
            ..Default::default()
        };
        config.register_platform(platform_name.to_string(), registry)?;
    }

    // 切换平台
    let old_platform = config.current_platform.clone();
    config.set_current_platform(platform_name)?;

    // 保存配置
    manager.save(&config)?;

    println!();
    ColorOutput::success(&format!(
        "已从平台 '{}' 切换到 '{}'",
        old_platform.bright_yellow(),
        platform_name.bright_green().bold()
    ));

    // 显示当前 profile
    if let Some(profile) = config
        .platforms
        .get(platform_name)
        .and_then(|e| e.current_profile.as_ref())
    {
        println!();
        ColorOutput::info(&format!("当前 profile: {}", profile.bright_cyan()));
    } else {
        println!();
        ColorOutput::warning("该平台尚未配置 profile");
        println!("  提示: 使用相应平台的命令配置 profile");
    }

    Ok(())
}

/// 🔍 显示当前平台信息
///
/// 显示当前激活平台的详细信息，包括平台状态、profile、路径等。
///
/// # 参数
///
/// * `json` - 是否以 JSON 格式输出
///
/// # 返回
///
/// * `Ok(())` - 成功显示信息
/// * `Err(CcrError::PlatformNotFound)` - 当前平台不存在（配置损坏）
///
/// # 示例
///
/// ```rust,no_run
/// use ccr::commands::platform_current_command;
///
/// // 表格格式输出
/// platform_current_command(false)?;
///
/// // JSON 格式输出
/// platform_current_command(true)?;
/// # Ok::<(), ccr::CcrError>(())
/// ```
///
/// # 输出格式
///
/// ## 表格模式 (json=false)
///
/// ```text
/// ╭─────────────────────────────────────────────────────╮
/// │                 当前平台信息                         │
/// ╰─────────────────────────────────────────────────────╯
///
/// 平台名称: claude (Claude Code)
/// 平台状态: ✅ 已启用
/// 当前 Profile: official
///
/// ╭─────────────────────────────────────────────────────╮
/// │                   平台路径                          │
/// ╰─────────────────────────────────────────────────────╯
///
/// 平台目录: /home/user/.ccr/claude
/// Profiles: /home/user/.ccr/claude/profiles.toml
/// 历史记录: /home/user/.ccr/claude/history.json
/// 备份目录: /home/user/.ccr/claude/backups
/// ```
///
/// ## JSON 模式 (json=true)
///
/// ```json
/// {
///   "name": "claude",
///   "display_name": "Claude Code",
///   "is_current": true,
///   "enabled": true,
///   "current_profile": "official",
///   "description": "Claude Code AI Assistant",
///   "paths": {
///     "platform_dir": "/home/user/.ccr/claude",
///     "profiles_file": "/home/user/.ccr/claude/profiles.toml",
///     "history_file": "/home/user/.ccr/claude/history.json",
///     "backups_dir": "/home/user/.ccr/claude/backups"
///   },
///   "profiles": ["official", "test", "backup"]
/// }
/// ```
///
pub fn platform_current_command(json: bool) -> Result<()> {
    let manager = PlatformConfigManager::with_default()?;
    let config = manager.load_or_create_default()?;

    let current_platform = &config.current_platform;
    let registry = config.get_platform(current_platform)?;

    // 获取路径信息
    let paths = if let Ok(platform) = Platform::from_str(current_platform) {
        PlatformPaths::new(platform).ok()
    } else {
        None
    };

    // 获取 profile 列表
    let profiles = if let Ok(platform) = Platform::from_str(current_platform) {
        if let Ok(platform_impl) = create_platform(platform) {
            platform_impl.list_profile_names().unwrap_or_default()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // 📤 JSON 输出
    if json {
        let paths_output = paths.as_ref().map(|p| PlatformPathsOutput {
            platform_dir: p.platform_dir.display().to_string(),
            profiles_file: p.profiles_file.display().to_string(),
            history_file: p.history_file.display().to_string(),
            backups_dir: p.backups_dir.display().to_string(),
        });

        let output = PlatformInfoOutput {
            name: current_platform.clone(),
            display_name: registry
                .description
                .clone()
                .unwrap_or_else(|| current_platform.clone()),
            is_current: true,
            enabled: registry.enabled,
            current_profile: registry.current_profile.clone(),
            description: registry.description.clone(),
            paths: paths_output.unwrap_or_else(|| PlatformPathsOutput {
                platform_dir: String::new(),
                profiles_file: String::new(),
                history_file: String::new(),
                backups_dir: String::new(),
            }),
            profiles,
        };

        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{}", json_str);

        return Ok(());
    }

    // 📊 表格输出 (原有逻辑)
    ColorOutput::title("当前平台信息");

    println!();
    ColorOutput::info(&format!(
        "平台名称: {}",
        current_platform.bright_green().bold()
    ));

    if let Some(desc) = &registry.description {
        ColorOutput::info(&format!("描述: {}", desc));
    }

    ColorOutput::info(&format!(
        "启用状态: {}",
        if registry.enabled {
            "已启用 ✓".green()
        } else {
            "已禁用 ✗".red()
        }
    ));

    if let Some(profile) = &registry.current_profile {
        ColorOutput::info(&format!("当前 Profile: {}", profile.bright_cyan()));
    } else {
        ColorOutput::warning("当前 Profile: 未配置");
    }

    if let Some(last_used) = &registry.last_used {
        ColorOutput::info(&format!("最后使用: {}", last_used.bright_black()));
    }

    println!();

    // 显示路径信息
    if let Some(p) = paths {
        println!();
        ColorOutput::info("平台路径:");
        println!("  配置目录: {}", p.platform_dir.display());
        println!("  Profiles: {}", p.profiles_file.display());
        println!("  设置文件: {}", p.settings_file.display());
        println!("  历史记录: {}", p.history_file.display());
        println!("  备份目录: {}", p.backups_dir.display());
    }

    Ok(())
}

/// ℹ️ 显示指定平台的详细信息
///
/// 显示特定平台的完整信息，包括状态、profiles、路径配置等。
/// 比 `platform_current_command` 更灵活，可以查看任意平台的信息。
///
/// # 参数
///
/// * `platform_name` - 要查看的平台名称 (`"claude"`, `"codex"`, `"gemini"` 等)
/// * `json` - 是否以 JSON 格式输出
///
/// # 返回
///
/// * `Ok(())` - 成功显示信息
/// * `Err(CcrError::PlatformNotFound)` - 指定的平台不存在或未实现
///
/// # 示例
///
/// ```rust,no_run
/// use ccr::commands::platform_info_command;
///
/// // 查看 Claude 平台信息（表格格式）
/// platform_info_command("claude", false)?;
///
/// // 查看 Codex 平台信息（JSON 格式）
/// platform_info_command("codex", true)?;
///
/// // 查看 Gemini 平台信息
/// platform_info_command("gemini", false)?;
/// # Ok::<(), ccr::CcrError>(())
/// ```
///
/// # 输出格式
///
/// ## 表格模式 (json=false)
///
/// ```text
/// ╭─────────────────────────────────────────────────────╮
/// │             平台详细信息: codex                      │
/// ╰─────────────────────────────────────────────────────╯
///
/// 平台名称: codex (Codex)
/// 平台状态: ✅ 已启用
/// 是否为当前平台: 否
/// 当前 Profile: github-official
/// 描述: GitHub Copilot CLI
///
/// ╭─────────────────────────────────────────────────────╮
/// │                   平台路径                          │
/// ╰─────────────────────────────────────────────────────╯
///
/// 平台目录: /home/user/.ccr/codex
/// Profiles: /home/user/.ccr/codex/profiles.toml
/// 设置文件: /home/user/.ccr/codex/settings.json
/// 历史记录: /home/user/.ccr/codex/history.json
/// 备份目录: /home/user/.ccr/codex/backups
///
/// ╭─────────────────────────────────────────────────────╮
/// │                  平台状态                           │
/// ╰─────────────────────────────────────────────────────╯
///
/// Profiles 文件: 存在 ✓
/// 设置文件: 存在 ✓
/// 平台目录: 存在 ✓
///
/// ╭─────────────────────────────────────────────────────╮
/// │              已配置 Profiles                        │
/// ╰─────────────────────────────────────────────────────╯
///
/// ▶ github-official - Official GitHub Copilot
///   github-enterprise - GitHub Enterprise Copilot
///   personal - Personal GitHub Account
/// ```
///
/// ## JSON 模式 (json=true)
///
/// ```json
/// {
///   "name": "codex",
///   "display_name": "Codex",
///   "is_current": false,
///   "enabled": true,
///   "current_profile": "github-official",
///   "description": "GitHub Copilot CLI",
///   "paths": {
///     "platform_dir": "/home/user/.ccr/codex",
///     "profiles_file": "/home/user/.ccr/codex/profiles.toml",
///     "history_file": "/home/user/.ccr/codex/history.json",
///     "backups_dir": "/home/user/.ccr/codex/backups"
///   },
///   "profiles": [
///     "github-official",
///     "github-enterprise",
///     "personal"
///   ]
/// }
/// ```
///
/// # 使用场景
///
/// - 检查平台是否已初始化
/// - 查看平台的 profiles 列表
/// - 验证平台目录结构
/// - 获取平台路径信息（用于脚本）
/// - 诊断平台配置问题
///
pub fn platform_info_command(platform_name: &str, json: bool) -> Result<()> {
    // 验证平台是否存在
    let platform = Platform::from_str(platform_name)
        .map_err(|_| CcrError::PlatformNotFound(platform_name.to_string()))?;

    let platform_impl = create_platform(platform)?;
    let paths = PlatformPaths::new(platform)?;

    // 检查是否为当前平台
    let manager = PlatformConfigManager::with_default()?;
    let config = manager.load_or_create_default()?;
    let is_current = platform_name == config.current_platform;

    // 获取注册信息
    let registry = config.platforms.get(platform_name);
    let enabled = registry.map(|r| r.enabled).unwrap_or(false);
    let current_profile_name = registry.and_then(|r| r.current_profile.clone());
    let description = registry.and_then(|r| r.description.clone());

    // 获取 profiles 列表
    let profiles = platform_impl.list_profile_names().unwrap_or_default();

    // 📤 JSON 输出
    if json {
        let output = PlatformInfoOutput {
            name: platform_name.to_string(),
            display_name: platform_impl.platform_name().to_string(),
            is_current,
            enabled,
            current_profile: current_profile_name,
            description,
            paths: PlatformPathsOutput {
                platform_dir: paths.platform_dir.display().to_string(),
                profiles_file: paths.profiles_file.display().to_string(),
                history_file: paths.history_file.display().to_string(),
                backups_dir: paths.backups_dir.display().to_string(),
            },
            profiles,
        };

        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{}", json_str);

        return Ok(());
    }

    // 📊 表格输出 (原有逻辑)
    ColorOutput::title(&format!("平台信息: {}", platform_name));

    println!();
    ColorOutput::info(&format!(
        "平台类型: {}",
        format!("{:?}", platform_impl.platform_type()).bright_cyan()
    ));
    ColorOutput::info(&format!("显示名称: {}", platform_impl.platform_name()));

    // 获取路径信息
    println!();
    ColorOutput::info("路径配置:");
    println!("  根目录: {}", paths.root.display());
    println!("  注册表: {}", paths.registry_file.display());
    println!("  平台目录: {}", paths.platform_dir.display());
    println!("  Profiles 文件: {}", paths.profiles_file.display());
    println!("  设置文件: {}", paths.settings_file.display());
    println!("  历史文件: {}", paths.history_file.display());
    println!("  备份目录: {}", paths.backups_dir.display());

    // 检测平台状态
    println!();
    ColorOutput::info("平台状态:");
    println!(
        "  Profiles 文件: {}",
        if paths.profiles_file.exists() {
            "存在 ✓".green()
        } else {
            "不存在 ✗".red()
        }
    );
    println!(
        "  设置文件: {}",
        if paths.settings_file.exists() {
            "存在 ✓".green()
        } else {
            "不存在 ✗".red()
        }
    );
    println!(
        "  平台目录: {}",
        if paths.platform_dir.exists() {
            "存在 ✓".green()
        } else {
            "不存在 ✗".red()
        }
    );

    // 加载并显示 profiles
    println!();
    ColorOutput::info("已配置 Profiles:");

    match platform_impl.load_profiles() {
        Ok(profiles_map) => {
            if profiles_map.is_empty() {
                println!("  (无)");
            } else {
                // 获取当前 profile
                let current_profile = platform_impl.get_current_profile().ok().flatten();

                for (name, profile) in profiles_map {
                    let marker = if Some(&name) == current_profile.as_ref() {
                        "▶ ".green()
                    } else {
                        "  ".normal()
                    };

                    if let Some(desc) = &profile.description {
                        println!("{}{} - {}", marker, name.bright_cyan(), desc);
                    } else {
                        println!("{}{}", marker, name.bright_cyan());
                    }
                }
            }
        }
        Err(e) => {
            ColorOutput::warning(&format!("无法加载 profiles: {}", e));
        }
    }

    Ok(())
}

/// 🆕 初始化平台配置
///
/// 初始化指定平台的目录结构和配置文件，为平台使用做准备。
///
/// # 参数
///
/// * `platform_name` - 要初始化的平台名称 (`"claude"`, `"codex"`, `"gemini"` 等)
///
/// # 返回
///
/// * `Ok(())` - 成功初始化平台
/// * `Err(CcrError::PlatformNotFound)` - 指定的平台不存在或未实现
/// * `Err(CcrError::IoError)` - 创建目录或文件失败
///
/// # 示例
///
/// ```rust,no_run
/// use ccr::commands::platform_init_command;
///
/// // 初始化 Claude 平台
/// platform_init_command("claude")?;
///
/// // 初始化 Codex (GitHub Copilot) 平台
/// platform_init_command("codex")?;
///
/// // 初始化 Gemini 平台
/// platform_init_command("gemini")?;
/// # Ok::<(), ccr::CcrError>(())
/// ```
///
/// # 初始化操作
///
/// 1. **验证平台**: 检查平台名称是否在支持列表中
/// 2. **创建目录**: 创建平台专用目录结构
///    - `~/.ccr/{platform}/` - 平台根目录
///    - `~/.ccr/{platform}/backups/` - 备份目录
/// 3. **注册平台**: 在 `~/.ccr/config.toml` 中注册平台
/// 4. **设置默认**: 如果是首个平台，设为默认和当前平台
///
/// # 目录结构
///
/// ```text
/// ~/.ccr/{platform}/
///   ├── profiles.toml    # 平台 profiles（初次使用时创建）
///   ├── settings.json    # 平台设置（首次切换 profile 时创建）
///   ├── history.json     # 操作历史（首次操作时创建）
///   └── backups/         # 备份目录（立即创建）
/// ```
///
/// # 输出示例
///
/// ```text
/// ╭─────────────────────────────────────────────────────╮
/// │              初始化平台: codex                       │
/// ╰─────────────────────────────────────────────────────╯
///
/// ℹ️  正在创建平台目录结构...
///   ✅ 创建目录: /home/user/.ccr/codex
///   ✅ 创建目录: /home/user/.ccr/codex/backups
///
/// ℹ️  正在注册平台到配置文件...
///   ✅ 平台已注册: codex
///
/// ✅ 平台 'codex' 初始化完成！
///
/// 💡 提示: 使用以下命令管理该平台:
///   ccr platform switch codex  # 切换到该平台
///   ccr add                    # 添加 profile
/// ```
///
/// # 注意事项
///
/// - 如果平台已经初始化，此命令是幂等的（不会覆盖现有数据）
/// - 初始化不会创建任何 profile，需要手动添加
/// - 对于 Claude 平台，会尝试从 Legacy 模式迁移配置（如果存在）
///
pub fn platform_init_command(platform_name: &str) -> Result<()> {
    ColorOutput::title(&format!("初始化平台: {}", platform_name));

    // 验证平台是否存在
    let platform = Platform::from_str(platform_name)
        .map_err(|_| CcrError::PlatformNotFound(platform_name.to_string()))?;

    let platform_impl = create_platform(platform)?;
    let paths = PlatformPaths::new(platform)?;

    println!();
    ColorOutput::info("正在创建平台目录结构...");

    // 使用 PlatformPaths 的统一方法创建所有必需的目录
    paths.ensure_directories()?;

    ColorOutput::success(&format!("✓ 根目录: {}", paths.root.display()));
    ColorOutput::success(&format!("✓ 平台目录: {}", paths.platform_dir.display()));
    ColorOutput::success(&format!(
        "✓ 历史目录: {}",
        paths
            .history_file
            .parent()
            .ok_or_else(|| CcrError::ConfigError("历史文件路径没有父目录".into()))?
            .display()
    ));
    ColorOutput::success(&format!("✓ 备份目录: {}", paths.backups_dir.display()));

    // 注册平台到统一配置
    let manager = PlatformConfigManager::with_default()?;
    let mut config = manager.load_or_create_default()?;

    if !config.platforms.contains_key(platform_name) {
        let registry = crate::managers::PlatformConfigEntry {
            description: Some(platform_impl.platform_name().to_string()),
            ..Default::default()
        };
        config.register_platform(platform_name.to_string(), registry)?;
        manager.save(&config)?;

        println!();
        ColorOutput::success(&format!("✓ 平台 '{}' 已注册到配置文件", platform_name));
    } else {
        println!();
        ColorOutput::info(&format!("平台 '{}' 已经注册", platform_name));
    }

    println!();
    ColorOutput::success("平台初始化完成！");
    println!();
    ColorOutput::info("下一步:");
    println!("  1. 使用相应平台的命令配置 profile");
    println!(
        "  2. 使用 'ccr platform switch {}' 切换到该平台",
        platform_name
    );

    Ok(())
}
