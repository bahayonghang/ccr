// 🎬 init 命令实现 - 初始化配置文件
// 📦 初始化 CCR 多平台配置结构 (~/.ccr/) 或兼容旧版模式 (~/.ccs_config.toml)

use crate::core::error::{CcrError, Result};
use crate::core::logging::ColorOutput;
use crate::managers::PlatformConfigManager;
use crate::managers::config::ConfigManager;
use crate::models::{Platform, PlatformPaths};
use indexmap::IndexMap;
use std::fs;
use std::path::PathBuf;

/// 📋 生成示例配置文件内容
fn generate_example_config() -> Result<String> {
    use crate::managers::config::{ConfigSection, GlobalSettings, ProviderType};

    // 构建示例配置节
    let mut sections = IndexMap::new();
    sections.insert(
        "anyrouter_main".to_string(),
        ConfigSection {
            description: Some("AnyRouter 主节点 API".to_string()),
            base_url: Some("https://api.example.com".to_string()),
            auth_token: Some("sk-YOUR_API_TOKEN_HERE".to_string()),
            model: Some("claude-sonnet-4-5-20250929".to_string()),
            small_fast_model: Some("claude-3-5-haiku-20241022".to_string()),
            provider: Some("AnyRouter".to_string()),
            provider_type: Some(ProviderType::OfficialRelay),
            account: Some("example_account".to_string()),
            tags: Some(vec!["stable".to_string(), "high-speed".to_string()]),
            usage_count: Some(0),
            enabled: Some(true),
            other: IndexMap::new(),
        },
    );

    sections.insert(
        "anthropic".to_string(),
        ConfigSection {
            description: Some("Anthropic 官方 API".to_string()),
            base_url: Some("https://api.anthropic.com".to_string()),
            auth_token: Some("sk-ant-api03-YOUR_TOKEN_HERE".to_string()),
            model: Some("claude-sonnet-4-5-20250929".to_string()),
            small_fast_model: Some("claude-3-5-haiku-20241022".to_string()),
            provider: Some("Anthropic".to_string()),
            provider_type: Some(ProviderType::OfficialRelay),
            account: None,
            tags: Some(vec!["official".to_string()]),
            usage_count: Some(0),
            enabled: Some(true),
            other: IndexMap::new(),
        },
    );

    let settings = GlobalSettings {
        skip_confirmation: false,
        tui_theme: None,
        #[allow(deprecated)]
        sync: Default::default(),
    };

    let config = crate::managers::config::CcsConfig {
        default_config: "anyrouter_main".to_string(),
        current_config: "anyrouter_main".to_string(),
        settings,
        sections,
    };

    toml::to_string_pretty(&config)
        .map_err(|e| CcrError::ConfigError(format!("生成示例配置失败: {}", e)))
}

/// 🎬 初始化配置文件
///
/// **新的行为 (2025)**: 默认使用 Unified Mode (~/.ccr/ 目录结构)
pub fn init_command(force: bool) -> Result<()> {
    ColorOutput::title("CCR 配置初始化");
    println!();

    // 🔍 检测配置模式
    let use_legacy = std::env::var("CCR_LEGACY_MODE").is_ok();

    if use_legacy {
        ColorOutput::warning("检测到 CCR_LEGACY_MODE 环境变量，使用 Legacy 模式");
        println!();
        return init_legacy_mode(force);
    }

    // 🆕 使用新的 Unified Mode
    init_unified_mode(force)
}

/// 🆕 初始化 Unified Mode
fn init_unified_mode(force: bool) -> Result<()> {
    let manager = PlatformConfigManager::with_default()?;
    let config_path = manager.config_path();

    // 检查配置是否已存在
    if config_path.exists() {
        if !force {
            ColorOutput::warning(&format!("配置已存在: {}", config_path.display()));
            println!();
            ColorOutput::info("配置已经初始化，无需重复执行");
            ColorOutput::info("提示:");
            println!("  • 查看平台列表: ccr platform list");
            println!("  • 初始化特定平台: ccr platform init <平台名>");
            println!("  • 强制重新初始化: ccr init --force");
            println!();
            return Ok(());
        }

        // 使用 --force 时需要确认
        println!();
        ColorOutput::warning("⚠️  警告: 即将覆盖现有配置！");
        ColorOutput::info("提示: 现有配置会自动备份");
        println!();

        print!("确认强制重新初始化? (y/N): ");
        use std::io::{self, Write};
        io::stdout().flush().expect("无法刷新标准输出");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("无法读取用户输入");

        if !input.trim().eq_ignore_ascii_case("y") {
            ColorOutput::info("已取消初始化操作");
            return Ok(());
        }
        println!();

        // 备份现有配置
        ColorOutput::step("备份现有配置");
        if let Ok(content) = fs::read_to_string(config_path) {
            let backup_path = config_path.with_extension("toml.bak");
            fs::write(&backup_path, content)?;
            ColorOutput::success(&format!("已备份到: {}", backup_path.display()));
        }
        println!();
    }

    // 创建目录结构
    ColorOutput::step("创建 CCR 目录结构");

    let home =
        dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
    let ccr_root = home.join(".ccr");
    let platforms_dir = ccr_root.join("platforms");

    fs::create_dir_all(&platforms_dir).map_err(CcrError::from)?;

    ColorOutput::success(&format!("✓ CCR 根目录: {}", ccr_root.display()));
    ColorOutput::success(&format!("✓ 平台目录: {}", platforms_dir.display()));

    // 初始化默认平台（Claude）
    println!();
    ColorOutput::step("初始化默认平台: Claude");

    let claude_paths = PlatformPaths::new(Platform::Claude)?;
    claude_paths.ensure_directories()?;

    ColorOutput::success(&format!(
        "✓ Claude 平台目录: {}",
        claude_paths.platform_dir.display()
    ));
    ColorOutput::success(&format!(
        "✓ 历史目录: {}",
        claude_paths
            .history_file
            .parent()
            .expect("无法获取历史文件父目录")
            .display()
    ));
    ColorOutput::success(&format!(
        "✓ 备份目录: {}",
        claude_paths.backups_dir.display()
    ));

    // 创建默认 profiles.toml
    if !claude_paths.profiles_file.exists() {
        ColorOutput::step("创建默认 Claude profiles.toml");

        let default_ccs = crate::managers::config::CcsConfig {
            default_config: "default".to_string(),
            current_config: "default".to_string(),
            settings: crate::managers::config::GlobalSettings::default(),
            sections: IndexMap::new(),
        };

        let content = toml::to_string_pretty(&default_ccs)
            .map_err(|e| CcrError::ConfigError(format!("序列化默认配置失败: {}", e)))?;
        fs::write(&claude_paths.profiles_file, content)
            .map_err(|e| CcrError::ConfigError(format!("写入默认 profiles.toml 失败: {}", e)))?;

        ColorOutput::success(&format!(
            "✓ 已创建: {}",
            claude_paths.profiles_file.display()
        ));
    }

    // 创建平台注册表配置
    println!();
    ColorOutput::step("创建平台注册表");

    let config = manager.load_or_create_default()?;
    manager.save(&config)?;

    ColorOutput::success(&format!("✓ 配置文件: {}", config_path.display()));

    // 显示完成信息
    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::success("✓ CCR 配置初始化成功 (Unified Mode)");
    println!();

    ColorOutput::info("已创建的目录结构:");
    println!("  ~/.ccr/                    # CCR 根目录");
    println!("  └── config.toml            # 平台注册表");
    println!("  └── platforms/");
    println!("      └── claude/            # Claude 平台（默认）");
    println!();

    ColorOutput::info("后续步骤:");
    println!("  1. 使用 'ccr platform list' 查看所有平台");
    println!("  2. 使用 'ccr platform init <平台>' 初始化其他平台");
    println!("  3. 使用 'ccr add' 添加配置 profile");
    println!("  4. 使用 'ccr list' 查看配置列表");
    println!();

    ColorOutput::info("💡 提示:");
    println!("  • 如需迁移旧配置，运行: ccr migrate");
    println!("  • 查看帮助: ccr --help");
    println!();

    Ok(())
}

/// 🔙 Legacy Mode
fn init_legacy_mode(force: bool) -> Result<()> {
    let home =
        dirs::home_dir().ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".into()))?;
    let config_path = home.join(".ccs_config.toml");

    let auto_confirm = if config_path.exists() {
        let config_manager = ConfigManager::new(&config_path);
        config_manager
            .load()
            .ok()
            .map(|c| c.settings.skip_confirmation)
            .unwrap_or(false)
    } else {
        false
    };
    let skip_confirmation = force || auto_confirm;

    if auto_confirm && force {
        ColorOutput::info("⚡ 自动确认模式已启用，将跳过确认");
    }

    if config_path.exists() {
        if !force {
            ColorOutput::warning(&format!("配置文件已存在: {}", config_path.display()));
            println!();
            ColorOutput::info("配置文件已经初始化,无需重复执行");
            ColorOutput::info("提示:");
            println!("  • 查看配置: ccr list");
            println!("  • 编辑配置: vim ~/.ccs_config.toml");
            println!("  • 强制重新初始化: ccr init --force");
            println!();
            return Ok(());
        }

        if !skip_confirmation {
            println!();
            ColorOutput::warning("⚠️  警告: 即将覆盖现有配置文件！");
            ColorOutput::info("提示: 现有配置会自动备份");
            println!();

            print!("确认强制重新初始化? (y/N): ");
            use std::io::{self, Write};
            io::stdout().flush().expect("无法刷新标准输出");

            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("无法读取用户输入");

            if !input.trim().eq_ignore_ascii_case("y") {
                ColorOutput::info("已取消初始化操作");
                return Ok(());
            }
            println!();
        }

        let status_msg = if skip_confirmation {
            "⚡ 使用 --force 模式,将覆盖现有配置 (自动确认模式)"
        } else {
            "使用 --force 模式,将覆盖现有配置"
        };
        ColorOutput::warning(status_msg);
        println!();
        ColorOutput::step("备份现有配置");
        backup_existing_config(&config_path)?;
        println!();
    }

    ColorOutput::step("创建配置文件");
    create_config_file(&config_path)?;

    println!();
    ColorOutput::separator();
    println!();
    ColorOutput::success("✓ 配置文件初始化成功 (Legacy Mode)");
    ColorOutput::info(&format!("配置文件位置: {}", config_path.display()));
    println!();

    ColorOutput::info("后续步骤:");
    println!("  1. 编辑配置文件: ~/.ccs_config.toml");
    println!("  2. 填写您的 API 密钥");
    println!("  3. 运行 'ccr list' 查看所有配置");
    println!("  4. 运行 'ccr switch <config>' 切换配置");
    println!();

    ColorOutput::warning("💡 建议: 迁移到新的 Unified Mode");
    println!("  运行 'ccr migrate' 迁移到新的多平台配置结构");
    println!();

    Ok(())
}

fn create_config_file(config_path: &PathBuf) -> Result<()> {
    let example_content = generate_example_config()?;
    fs::write(config_path, example_content)
        .map_err(|e| CcrError::ConfigError(format!("写入配置文件失败: {}", e)))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::Permissions::from_mode(0o644);
        fs::set_permissions(config_path, permissions)
            .map_err(|e| CcrError::ConfigError(format!("设置文件权限失败: {}", e)))?;
    }

    ColorOutput::success(&format!("已创建配置文件: {}", config_path.display()));
    Ok(())
}

fn backup_existing_config(config_path: &PathBuf) -> Result<()> {
    let config_manager = ConfigManager::new(config_path);
    let backup_path = config_manager.backup(Some("init"))?;
    ColorOutput::success(&format!("已备份到: {}", backup_path.display()));
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_example_config_not_empty() {
        let example = generate_example_config().unwrap();
        assert!(!example.is_empty());
        assert!(example.contains("default_config"));
        assert!(example.contains("[anyrouter_main]"));
    }

    #[test]
    fn test_create_config_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".ccs_config.toml");

        create_config_file(&config_path).unwrap();

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("default_config"));
    }
}
